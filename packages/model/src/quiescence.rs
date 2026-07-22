//! Quiescence scheduler reports, eligibility classification, audit
//! test helpers, the residue-reader policy, and the
//! protocol-observable projection.
//!
//! Implements `(´def:verification:quiescence-report´)`,
//! `(´rule:verification:lifecycle-report´)`,
//! `(´def:verification:quiescence-eligibility´)`,
//! `(´rule:verification:classify-quiescence-eligibility´)`,
//! `(´thm:scalability:sweepability´)` (predicate form),
//! `(´thm:ledgers:residue-noninterference´)`,
//! `(´rule:verification:residue-readers´)`,
//! `(´def:verification:residue-perturbation´)`,
//! `(´def:verification:protocol-observable´)`, and
//! `(´rule:verification:project-protocol-observable´)`.

use std::collections::{BTreeMap, BTreeSet};

use crate::asset::{Asset, ReceiptClass};
use crate::guard::Guard;
use crate::history::{DistributionResidueProjection, History};
use crate::maintenance::admission_capacity_plan;
use crate::object::{Meta, Utxo};
use crate::pool::PoolState;
use crate::recognition::{
    CanonicalObject, StateClass, classify_state_object, validate_request_for_admission,
};
use crate::scalar::{CanonicalOrder, OutPoint, Sat};
use crate::transition::Transition;
use crate::world::{RootCursor, World};

// ´rule:verification:residue-readers´

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuantityId {
    Floor,
    RedemptionPayout,
    CycleIssuance,
    AttestationDelta,
    InvariantCheck,
    ExternalAudit,
}

pub fn quantity_reads_residue(quantity: QuantityId) -> bool {
    matches!(
        quantity,
        QuantityId::InvariantCheck | QuantityId::ExternalAudit
    )
}

/// The typed manifest in `tripod-architecture` is the
/// normative reader policy; `manifest::validate_architecture_conformance`
/// checks this Rust copy against it.
pub fn assert_residue_reader_policy() -> Result<(), Guard> {
    for quantity in [
        QuantityId::Floor,
        QuantityId::RedemptionPayout,
        QuantityId::CycleIssuance,
        QuantityId::AttestationDelta,
    ] {
        if quantity_reads_residue(quantity) {
            return Err(Guard::BadAuthorization);
        }
    }

    Ok(())
}

// ´def:verification:quiescence-report´

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuiescenceReport {
    pub admissible_requests: usize,
    pub inert_open_junk: usize,

    pub entitlements: usize,
    pub live_distributions: usize,

    pub ash_outputs: usize,
    pub anchors: usize,

    pub live_receipts: usize,
    pub time_locked_receipts: usize,
}

// ´rule:verification:lifecycle-report´

pub fn lifecycle_report(world: &World) -> Result<QuiescenceReport, Guard> {
    let mut report = QuiescenceReport::default();

    for (outpoint, utxo) in &world.utxos {
        match classify_state_object(world, *outpoint, utxo) {
            StateClass::InertExternal => match (utxo.asset, utxo.meta) {
                (Asset::Lbtc, Meta::DepositRequest { .. }) => {
                    if validate_request_for_admission(&world.constants, utxo).is_ok() {
                        report.admissible_requests += 1;
                    } else {
                        report.inert_open_junk += 1;
                    }
                }

                (Asset::Lbtc, Meta::CpfpAnchor) => {
                    report.anchors += 1;
                }

                _ => {
                    report.inert_open_junk += 1;
                }
            },

            StateClass::Canonical(CanonicalObject::DepositEntitlement { .. }) => {
                report.entitlements += 1;
            }

            StateClass::Canonical(CanonicalObject::DistributionControl) => {
                report.live_distributions += 1;
            }

            StateClass::Canonical(CanonicalObject::Ash) => {
                report.ash_outputs += 1;
            }

            StateClass::Canonical(CanonicalObject::Receipt {
                class: ReceiptClass::Live,
                ..
            }) => {
                report.live_receipts += 1;
            }

            StateClass::Canonical(CanonicalObject::Receipt {
                class: ReceiptClass::TimeLocked,
                ..
            }) => {
                report.time_locked_receipts += 1;
            }

            StateClass::Canonical(_) | StateClass::ActiveResv | StateClass::CanonicalViolation => {}
        }
    }

    Ok(report)
}

// (´thm:scalability:sweepability´)

/// This predicate assumes:
///
/// - no pending admissible requests;
/// - `Q = 0`;
/// - sponsored settlement and ASH maintenance.
///
/// It intentionally ignores final user-owned receipts and inert open
/// junk.
pub fn shared_state_is_swept(report: &QuiescenceReport) -> bool {
    report.entitlements == 0 && report.live_distributions == 0 && report.ash_outputs <= 1
}

// ´def:verification:quiescence-eligibility´

/// Named residual classes that the sponsored-quiescence theorem
/// excludes from full discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuiescenceResidual {
    /// An admissible request targets a sealed pool: it cannot be
    /// admitted and requires its refund key to recover.
    SealedPoolWithPendingRequests,

    /// A request-shaped output fails admission validation; it remains
    /// inert and requires its refund key.
    UnderfundedLostRefundRequest,

    /// Malformed open junk outside the pool queue; it never enters the
    /// safety state and cannot be collected by maintenance.
    MalformedOpenJunkOnly,

    /// One or more locally valid requests cannot fit the remaining
    /// active-backing headroom (`(´rule:domains:active-backing-cap´)`).
    /// Admission and cycle processing do not restore that headroom;
    /// redemption or owner-authorized cancellation is required. This
    /// blocks full discharge on a live pool.
    ActiveBackingCapacityBlocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuiescenceEligibility {
    /// Sponsored quiescence must fully discharge the shared queues.
    Eligible,

    /// Full discharge is blocked; the named residuals describe what
    /// remains.
    Residual(BTreeSet<QuiescenceResidual>),
}

// ´rule:verification:classify-quiescence-eligibility´

/// Classifies whether the sponsored-quiescence theorem's precondition
/// holds.
///
/// `Eligible` means the deterministic sponsored driver must succeed;
/// `Residual` names exactly why full discharge is blocked. Non-blocking
/// residuals (inert junk, unadmittable requests on a live pool) do not
/// make a world ineligible: they remain outside the shared queues that
/// the theorem discharges.
///
/// The two blocking residuals are a sealed pool with pending locally
/// valid requests and active-backing capacity exhaustion. Malformed
/// junk and owner-cancelable invalid requests are named when a blocking
/// residual is also present, but never make a live, capacity-sufficient
/// pool ineligible on their own.
pub fn classify_quiescence_eligibility(world: &World) -> Result<QuiescenceEligibility, Guard> {
    let state = world.state()?.1;

    let report = lifecycle_report(world)?;

    let capacity = admission_capacity_plan(world)?;

    let sealed = state.y()?.is_zero();

    let mut residuals = BTreeSet::new();

    if sealed && !capacity.locally_valid.is_empty() {
        residuals.insert(QuiescenceResidual::SealedPoolWithPendingRequests);
    }

    if !sealed && !capacity.all_fit {
        residuals.insert(QuiescenceResidual::ActiveBackingCapacityBlocked);
    }

    let unadmittable = unadmittable_request_count(world);

    if unadmittable > 0 {
        residuals.insert(QuiescenceResidual::UnderfundedLostRefundRequest);
    }

    if report.inert_open_junk > unadmittable {
        residuals.insert(QuiescenceResidual::MalformedOpenJunkOnly);
    }

    if residuals.contains(&QuiescenceResidual::SealedPoolWithPendingRequests)
        || residuals.contains(&QuiescenceResidual::ActiveBackingCapacityBlocked)
    {
        Ok(QuiescenceEligibility::Residual(residuals))
    } else {
        Ok(QuiescenceEligibility::Eligible)
    }
}

fn unadmittable_request_count(world: &World) -> usize {
    world
        .utxos
        .values()
        .filter(|utxo| {
            matches!(
                (utxo.asset, utxo.meta),
                (Asset::Lbtc, Meta::DepositRequest { .. }),
            ) && validate_request_for_admission(&world.constants, utxo).is_err()
        })
        .count()
}

/// Checks the classified residuals against the actual lifecycle report
/// of the blocked world.
pub fn residuals_match_report(
    world: &World,
    report: &QuiescenceReport,
    residuals: &BTreeSet<QuiescenceResidual>,
) -> Result<bool, Guard> {
    let state = world.state()?.1;

    for residual in residuals {
        let matches = match residual {
            QuiescenceResidual::SealedPoolWithPendingRequests => {
                state.y()?.is_zero() && report.admissible_requests > 0
            }

            QuiescenceResidual::UnderfundedLostRefundRequest => {
                unadmittable_request_count(world) > 0
            }

            QuiescenceResidual::MalformedOpenJunkOnly => report.inert_open_junk > 0,

            QuiescenceResidual::ActiveBackingCapacityBlocked => {
                let capacity = admission_capacity_plan(world)?;

                !state.y()?.is_zero() && !capacity.locally_valid.is_empty() && !capacity.all_fit
            }
        };

        if !matches {
            return Ok(false);
        }
    }

    Ok(!residuals.is_empty())
}

// ´thm:ledgers:residue-noninterference´

/// The computation set must include every covenant branch result,
/// floor query, cycle issuance query, redemption payout query, and
/// attestation query.
pub fn assert_residue_noninterference<F, R>(world: &World, perturb_history: F, computation: R)
where
    F: FnOnce(&mut History),
    R: Fn(&World) -> Vec<u8>,
{
    let before = computation(world);

    let mut perturbed = world.clone();

    perturb_history(&mut perturbed.history);

    let after = computation(&perturbed);

    assert_eq!(
        before, after,
        "audit-only residue affected a forbidden consumer",
    );
}

// ´def:verification:residue-perturbation´

/// A proof-side corruption helper. It does not represent a valid
/// transition and is used only to test the declared noninterference
/// boundary.
pub fn perturb_residue_projection(history: &mut History, live_delta: Sat, time_locked_delta: Sat) {
    let Some(certificate) = history.transitions.last_mut() else {
        return;
    };

    certificate.distribution_residue = Some(DistributionResidueProjection {
        cycle: 999,
        control_input: 0,
        vault_input: None,
        live_residue: live_delta,
        time_locked_residue: time_locked_delta,
    });
}

// ´def:verification:protocol-observable´

/// History-derived audit terms are intentionally absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolObservable {
    pub roots: RootCursor,

    pub canonical_utxos: BTreeMap<OutPoint, Utxo>,

    pub pool_state: PoolState,

    pub pace_age_blocks: u64,
}

// ´rule:verification:project-protocol-observable´

pub fn protocol_observable(world: &World) -> Result<ProtocolObservable, Guard> {
    let mut canonical_utxos = BTreeMap::new();

    for (outpoint, utxo) in &world.utxos {
        match classify_state_object(world, *outpoint, utxo) {
            StateClass::Canonical(_) | StateClass::ActiveResv => {
                canonical_utxos.insert(*outpoint, *utxo);
            }

            StateClass::InertExternal => {}

            StateClass::CanonicalViolation => {
                return Err(Guard::InvariantFailure);
            }
        }
    }

    Ok(ProtocolObservable {
        roots: world.roots.clone(),
        canonical_utxos,
        pool_state: world.state()?.1,
        pace_age_blocks: world.pace_age_blocks,
    })
}

// (´thm:ledgers:residue-noninterference´) (branch-local form)

/// Compares protocol computations that are forbidden to read residue.
///
/// This bypasses the global invariant wrapper, which correctly notices
/// a corrupted audit history: the covenant does not read the history;
/// the off-chain invariant checker does.
pub fn assert_protocol_noninterference<T>(
    world: &World,
    transition: &T,
    order: CanonicalOrder,
    perturb: impl FnOnce(&mut History),
) -> Result<(), Guard>
where
    T: Transition,
{
    let clean = transition.apply(world, order);

    let mut perturbed = world.clone();

    perturb(&mut perturbed.history);

    let dirty = transition.apply(&perturbed, order);

    match (clean, dirty) {
        (Ok(clean_world), Ok(dirty_world)) => {
            assert_eq!(
                protocol_observable(&clean_world)?,
                protocol_observable(&dirty_world)?,
            );
        }

        (Err(clean_error), Err(dirty_error)) => {
            assert_eq!(clean_error, dirty_error);
        }

        _ => {
            panic!("audit-only residue affected branch-local semantics");
        }
    }

    Ok(())
}
