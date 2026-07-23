//! Operation-independent branch-shape validation and branch-specific
//! semantic postconditions.
//!
//! Implements `(´def:verification:object-kind´)`,
//! `(´rule:verification:object-kind´)`,
//! `(´def:verification:shape-policy´)`,
//! `(´rule:verification:shape-policy´)`,
//! `(´rule:verification:branch-shape-validation´)`,
//! `(´rule:verification:declared-cardinality´)`,
//! `(´rule:verification:required-cardinality´)`,
//! `(´rule:verification:data-output-family´)`,
//! `(´rule:verification:postcommit-shape´)`,
//! `(´rule:verification:branch-state-delta´)`,
//! `(´rule:verification:cycle-maturity-delta´)`,
//! `(´rule:verification:branch-event-projections´)`,
//! `(´rule:verification:transaction-object-helpers´)`,
//! `(´rule:verification:state-edge-pair´)`,
//! `(´rule:verification:issuance-lookup´)`,
//! `(´rule:verification:admission-postconditions´)`,
//! `(´def:verification:cycle-arithmetic´)`,
//! `(´rule:verification:derive-cycle-arithmetic´)`,
//! `(´rule:verification:cycle-postconditions´)`,
//! `(´rule:verification:settlement-postconditions´)`,
//! `(´rule:verification:relabel-positional-bijection´)`,
//! `(´rule:verification:open-flow-object-helpers´)`,
//! `(´rule:verification:open-flow-semantics´)`,
//! `(´rule:verification:create-request-postconditions´)`,
//! `(´rule:verification:cancel-request-postconditions´)`,
//! `(´rule:verification:transfer-postconditions´)`,
//! `(´rule:verification:redemption-postconditions´)`,
//! `(´rule:verification:burn-postconditions´)`,
//! `(´rule:verification:ash-compaction-postconditions´)`,
//! `(´rule:verification:clear-postconditions´)`,
//! `(´rule:verification:maturity-announcement-postconditions´)`, and
//! `(´rule:verification:branch-semantic-postconditions´)`.
//!
//! The postcommit semantic validators can derive values, recipients,
//! classes, roots, history projections, and open-flow purposes. They
//! cannot independently derive cryptographic signature validity from
//! the abstract `World`. Therefore: branch constructors check
//! `SignerSet`; the compiler emits the required signature checks;
//! deployment/script tests verify exact sighash behavior; and the
//! evidence taxonomy labels owner-authorization claims as
//! model-authorization-backed plus compiler-obligation-backed, not
//! purely oracle-backed.
//!
//! Branch-specific postcommit validation handles dynamic conditions
//! such as: sealing redemption omitting RESV; terminal settlement
//! omitting control/vault; empty cycle omitting issuance/control/vault;
//! maturity cycle requiring an anchor.
//!
//! Open-asset conservation proves that L-BTC balances, but it does not
//! prove which L-BTC output fulfills a branch-specific obligation such
//! as "redemption payout," "request refund," or "admission reward."
//! For fungible L-BTC, recipient/value matching is sufficient even if
//! the physical satoshis come from sponsor inputs rather than RESV. The
//! safety obligation is therefore: the required recipient receives the
//! required amount; the active reserve changes by the required amount;
//! and all L-BTC balances including fee and sponsor change. No stronger
//! "these exact satoshis came from RESV" provenance is meaningful.

use std::collections::{BTreeMap, BTreeSet};

use crate::asset::{Asset, Maturity, ReceiptClass};
use crate::constants::Constants;
use crate::genesis::OPERATOR_KEY;
use crate::guard::Guard;
use crate::history::{
    BranchKind, CertifiedIssuance, OpenFlowKind, OpenFlowProjection, RootEdge,
    TransitionCertificate,
};
use crate::kernel::{PendingOutput, branch_allows_open_flow};
use crate::object::{DataOutput, Meta, Utxo};
use crate::pool::PoolState;
use crate::recognition::{
    read_ash, read_distribution_control, read_entitlement, read_receipt,
    validate_request_for_admission,
};
use crate::scalar::{
    Cycle, OutPoint, OwnerKey, Sat, checked_add_to_map, floor_mul_div, floor_ratio,
};
use crate::world::World;

// ´def:verification:object-kind´

/// `ObjectKind` is a transaction-shape classifier. It is not the
/// safety-state projection classifier and must not be used to decide
/// whether an open object belongs to the safety state.
///
/// Discriminants are the architecture's stable object codes
/// (`architecture::ObjectId`); the manifest welds the pairing
/// code-for-code at compile time. `Foreign` is model-only and has no
/// declared counterpart.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectKind {
    State = 1,
    Resv = 2,
    Pace = 3,
    EntitlementAuthority = 4,
    DistributionAuthority = 5,

    ReceiptLive = 6,
    ReceiptTimeLocked = 7,

    DepositRequest = 8,
    DepositEntitlement = 9,

    DistributionControl = 10,
    DistributionVault = 11,

    Ash = 12,

    PlainLbtc = 13,
    CpfpAnchor = 14,

    Foreign = 15,
}

// ´rule:verification:object-kind´

fn object_kind(utxo: &Utxo) -> ObjectKind {
    match (utxo.asset, utxo.meta) {
        (Asset::Pid, Meta::State(_)) => ObjectKind::State,

        (Asset::Lbtc, Meta::Resv) => ObjectKind::Resv,

        (Asset::Pace, Meta::Pace) => ObjectKind::Pace,

        (Asset::EntAuth, Meta::EntitlementAuthority) => ObjectKind::EntitlementAuthority,

        (Asset::DistAuth, Meta::DistributionAuthority) => ObjectKind::DistributionAuthority,

        (
            Asset::U,
            Meta::Receipt {
                class: ReceiptClass::Live,
                ..
            },
        ) => ObjectKind::ReceiptLive,

        (
            Asset::U,
            Meta::Receipt {
                class: ReceiptClass::TimeLocked,
                ..
            },
        ) => ObjectKind::ReceiptTimeLocked,

        (Asset::Lbtc, Meta::DepositRequest { .. }) => ObjectKind::DepositRequest,

        (Asset::Ent, Meta::DepositEntitlement { .. }) => ObjectKind::DepositEntitlement,

        (Asset::DistCtl, Meta::DistributionControl { .. }) => ObjectKind::DistributionControl,

        (Asset::U, Meta::DistributionVault { .. }) => ObjectKind::DistributionVault,

        (Asset::U, Meta::Ash) => ObjectKind::Ash,

        (Asset::Lbtc, Meta::PlainLbtc { .. }) => ObjectKind::PlainLbtc,

        (Asset::Lbtc, Meta::CpfpAnchor) => ObjectKind::CpfpAnchor,

        _ => ObjectKind::Foreign,
    }
}

// ´def:verification:shape-policy´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapePolicy {
    pub allowed_inputs: BTreeSet<ObjectKind>,

    pub allowed_outputs: BTreeSet<ObjectKind>,

    pub min_inputs: BTreeMap<ObjectKind, usize>,

    pub max_inputs: BTreeMap<ObjectKind, usize>,

    pub min_outputs: BTreeMap<ObjectKind, usize>,

    pub max_outputs: BTreeMap<ObjectKind, usize>,
}

// ´rule:verification:shape-policy´
//
// One may assume naively that the cardinality tables must be
// hand-maintained per branch; instead the shape policy is derived from
// the typed manifest's operation input/output declarations, with
// finite bounds resolved through the calibrated runtime constants.

fn resolve_max_count(constants: &Constants, maximum: architecture::MaxCount) -> usize {
    match maximum {
        architecture::MaxCount::Exact(value) => usize::from(value),
        architecture::MaxCount::Bound(bound) => crate::manifest::bound_value(constants, bound),
    }
}

pub(crate) fn shape_policy(constants: &Constants, branch: BranchKind) -> ShapePolicy {
    let spec = crate::manifest::operation_spec(branch);

    let mut allowed_inputs = BTreeSet::new();
    let mut allowed_outputs = BTreeSet::new();

    let mut min_inputs = BTreeMap::new();
    let mut max_inputs = BTreeMap::new();

    let mut min_outputs = BTreeMap::new();
    let mut max_outputs = BTreeMap::new();

    for input in spec.inputs {
        let kind = crate::manifest::object_kind_of(input.object);

        allowed_inputs.insert(kind);
        min_inputs.insert(kind, usize::from(input.minimum));
        max_inputs.insert(kind, resolve_max_count(constants, input.maximum));
    }

    for output in spec.outputs {
        let kind = crate::manifest::object_kind_of(output.object);

        allowed_outputs.insert(kind);
        min_outputs.insert(kind, usize::from(output.minimum));
        max_outputs.insert(kind, resolve_max_count(constants, output.maximum));
    }

    ShapePolicy {
        allowed_inputs,
        allowed_outputs,
        min_inputs,
        max_inputs,
        min_outputs,
        max_outputs,
    }
}

// ´rule:verification:branch-shape-validation´

pub(crate) fn validate_branch_shape_precommit(
    world: &World,
    branch: BranchKind,
    consumed: &BTreeMap<OutPoint, Utxo>,
    outputs: &[PendingOutput],
    data_outputs: &[DataOutput],
) -> Result<(), Guard> {
    let policy = shape_policy(&world.constants, branch);

    let mut input_counts: BTreeMap<ObjectKind, usize> = BTreeMap::new();

    let mut output_counts: BTreeMap<ObjectKind, usize> = BTreeMap::new();

    for utxo in consumed.values() {
        let kind = object_kind(utxo);

        if !policy.allowed_inputs.contains(&kind) {
            return Err(Guard::WrongShape);
        }

        *input_counts.entry(kind).or_insert(0) += 1;
    }

    for output in outputs {
        let kind = object_kind(&Utxo {
            asset: output.asset,
            value: output.value,
            meta: output.meta,
        });

        if !policy.allowed_outputs.contains(&kind) {
            return Err(Guard::WrongShape);
        }

        *output_counts.entry(kind).or_insert(0) += 1;
    }

    // Tag-family faults report before cardinality faults so that a
    // destruction under the wrong branch family surfaces as a
    // BadAuthorization, not as a missing-output shape error.
    validate_data_output_family(branch, data_outputs)?;

    validate_declared_cardinalities(&policy, &input_counts, &output_counts)?;

    validate_dynamic_cardinality(branch, &input_counts, &output_counts)?;

    Ok(())
}

// ´rule:verification:declared-cardinality´
//
// One may assume naively that per-branch minima must be restated in a
// hand-written switch; instead every static minimum and maximum is
// enforced directly from the manifest-derived shape policy. Only
// condition-dependent relationships remain in
// `validate_dynamic_cardinality`.

fn validate_declared_cardinalities(
    policy: &ShapePolicy,
    input_counts: &BTreeMap<ObjectKind, usize>,
    output_counts: &BTreeMap<ObjectKind, usize>,
) -> Result<(), Guard> {
    for (kind, minimum) in &policy.min_inputs {
        if count_of(input_counts, *kind) < *minimum {
            return Err(Guard::WrongShape);
        }
    }

    for (kind, count) in input_counts {
        let maximum = policy.max_inputs.get(kind).copied().unwrap_or(0);

        if *count > maximum {
            return Err(Guard::Domain);
        }
    }

    for (kind, minimum) in &policy.min_outputs {
        if count_of(output_counts, *kind) < *minimum {
            return Err(Guard::WrongShape);
        }
    }

    for (kind, count) in output_counts {
        let maximum = policy.max_outputs.get(kind).copied().unwrap_or(0);

        if *count > maximum {
            return Err(Guard::Domain);
        }
    }

    Ok(())
}

// ´rule:verification:required-cardinality´
//
// Only relationships that cannot be expressed as independent static
// per-object minima and maxima remain here; everything static is
// enforced by `validate_declared_cardinalities` from the manifest.
// Condition-dependent output rules (empty-cycle control/vault,
// terminal-settlement successors, sealing redemption RESV omission,
// clear residual ASH, maturity anchors) are enforced by the postcommit
// branch-semantic validators.

fn count_of(counts: &BTreeMap<ObjectKind, usize>, kind: ObjectKind) -> usize {
    counts.get(&kind).copied().unwrap_or(0)
}

fn validate_dynamic_cardinality(
    branch: BranchKind,
    inputs: &BTreeMap<ObjectKind, usize>,
    outputs: &BTreeMap<ObjectKind, usize>,
) -> Result<(), Guard> {
    use ObjectKind::*;

    match branch {
        BranchKind::AdmitDeposits => {
            // One entitlement per admitted request.
            if count_of(outputs, DepositEntitlement) != count_of(inputs, DepositRequest) {
                return Err(Guard::WrongShape);
            }
        }

        BranchKind::ReceiptRelabel => {
            // One live receipt per relabeled time-locked receipt; the
            // owner/value bijection is a postcommit check.
            if count_of(outputs, ReceiptLive) != count_of(inputs, ReceiptTimeLocked) {
                return Err(Guard::WrongShape);
            }
        }

        _ => {}
    }

    Ok(())
}

// ´rule:verification:data-output-family´
//
// Open tag-shaped junk may exist elsewhere in the global environment,
// but normative branch constructors emit only their declared tag
// families. The permitted families are derived from the typed
// manifest's per-operation data-output declarations; branch-specific
// amount/condition arithmetic remains in the semantic postconditions.

fn validate_data_output_family(
    branch: BranchKind,
    data_outputs: &[DataOutput],
) -> Result<(), Guard> {
    let spec = crate::manifest::operation_spec(branch);

    for output in data_outputs {
        let permitted = match output {
            DataOutput::BurnRecord { .. } => spec
                .data_outputs
                .iter()
                .any(|declared| declared.kind == architecture::DataOutputKind::BurnRecord),

            DataOutput::Destruction { tag, asset, .. } => {
                spec.data_outputs.iter().any(|declared| {
                    declared.kind == architecture::DataOutputKind::Destruction
                        && crate::manifest::tag_of(declared.tag) == *tag
                        && declared.asset.map(crate::manifest::asset_of) == Some(*asset)
                })
            }
        };

        if !permitted {
            return Err(Guard::BadAuthorization);
        }
    }

    Ok(())
}

// ´rule:verification:postcommit-shape´

pub(crate) fn validate_branch_shape_postcommit(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    match certificate.branch {
        BranchKind::CreateRequest
        | BranchKind::CancelRequest
        | BranchKind::TransferLive
        | BranchKind::TransferTimeLocked
        | BranchKind::Burn
        | BranchKind::CompactAsh
        | BranchKind::SettleDistribution => {
            if certificate.state_edge.is_some()
                || certificate.resv_edge.is_some()
                || certificate.pace_edge.is_some()
            {
                return Err(Guard::RootSuccession);
            }
        }

        BranchKind::AdmitDeposits => {
            require_succ_edge(certificate.state_edge)?;

            require_succ_edge(certificate.resv_edge)?;

            require_succ_edge(certificate.entitlement_authority_edge)?;
        }

        BranchKind::Cycle => {
            require_succ_edge(certificate.state_edge)?;

            require_succ_edge(certificate.resv_edge)?;

            require_succ_edge(certificate.pace_edge)?;

            require_succ_edge(certificate.distribution_authority_edge)?;
        }

        BranchKind::Redeem => {
            require_succ_edge(certificate.state_edge)?;

            match certificate.resv_edge {
                Some(RootEdge::Succ { .. }) => {}

                Some(RootEdge::Term { .. }) => {
                    let (_state_outpoint, state) = after.state()?;

                    if !state.is_sealed()? {
                        return Err(Guard::ResvWeld);
                    }
                }

                None => {
                    return Err(Guard::ResvWeld);
                }
            }
        }

        BranchKind::ReceiptRelabel | BranchKind::Clear | BranchKind::AnnounceMaturity => {
            require_succ_edge(certificate.state_edge)?;

            if certificate.resv_edge.is_some() {
                return Err(Guard::ResvWeld);
            }
        }
    }

    validate_branch_state_delta(before, after, certificate)?;

    validate_branch_event_projections(before, after, certificate)?;

    validate_branch_semantics(before, after, certificate)?;

    // (´rule:verification:manifest-delta-conformance´): the actual
    // canonical-delta set must equal the manifest's active expected
    // set for this operation.
    crate::manifest::validate_manifest_delta_conformance(before, after, certificate)?;

    Ok(())
}

fn require_succ_edge(edge: Option<RootEdge>) -> Result<(), Guard> {
    match edge {
        Some(RootEdge::Succ { .. }) => Ok(()),

        _ => Err(Guard::RootSuccession),
    }
}

// ´rule:verification:branch-state-delta´

fn state_from_edge_input(world: &World, edge: RootEdge) -> Result<PoolState, Guard> {
    let input = match edge {
        RootEdge::Succ { input, .. } | RootEdge::Term { input } => input,
    };

    match world.utxo(input)?.meta {
        Meta::State(state) => Ok(state),
        _ => Err(Guard::WrongShape),
    }
}

fn state_from_edge_output(world: &World, edge: RootEdge) -> Result<PoolState, Guard> {
    let output = match edge {
        RootEdge::Succ { output, .. } => output,

        RootEdge::Term { .. } => {
            return Err(Guard::RootSuccession);
        }
    };

    match world.utxo(output)?.meta {
        Meta::State(state) => Ok(state),
        _ => Err(Guard::WrongShape),
    }
}

fn validate_branch_state_delta(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let Some(state_edge) = certificate.state_edge else {
        return Ok(());
    };

    let old = state_from_edge_input(before, state_edge)?;

    let new = state_from_edge_output(after, state_edge)?;

    match certificate.branch {
        BranchKind::AdmitDeposits => {
            if old.omega != new.omega
                || old.y_l != new.y_l
                || old.y_t != new.y_t
                || old.cycle != new.cycle
                || old.maturity != new.maturity
                || new.q < old.q
            {
                return Err(Guard::ValuePin);
            }
        }

        BranchKind::Cycle => {
            if new.q != Sat::ZERO
                || new.cycle != old.next_cycle()?
                || new.omega != old.omega.checked_add(old.q)?
            {
                return Err(Guard::ValuePin);
            }

            validate_cycle_maturity_delta(old, new)?;
        }

        BranchKind::Redeem => {
            if new.q != old.q
                || new.cycle != old.cycle
                || new.maturity != old.maturity
                || new.omega > old.omega
                || new.y_l > old.y_l
                || new.y_t != old.y_t
            {
                return Err(Guard::ValuePin);
            }
        }

        BranchKind::ReceiptRelabel => {
            if new != old {
                return Err(Guard::ValuePin);
            }
        }

        BranchKind::Clear => {
            if new.omega != old.omega
                || new.q != old.q
                || new.y_t != old.y_t
                || new.cycle != old.cycle
                || new.maturity != old.maturity
                || new.y_l >= old.y_l
            {
                return Err(Guard::ValuePin);
            }
        }

        BranchKind::AnnounceMaturity => {
            if new.omega != old.omega
                || new.q != old.q
                || new.y_l != old.y_l
                || new.y_t != old.y_t
                || new.cycle != old.cycle
                || !matches!(
                    (old.maturity, new.maturity),
                    (Maturity::Unannounced, Maturity::Announced { .. }),
                )
            {
                return Err(Guard::ValuePin);
            }
        }

        _ => {}
    }

    Ok(())
}

// ´rule:verification:cycle-maturity-delta´
//
// This check supplements the branch's issuance/class arithmetic; it
// forbids an announced-but-overdue state.

fn validate_cycle_maturity_delta(old: PoolState, new: PoolState) -> Result<(), Guard> {
    match old.maturity {
        Maturity::Unannounced => {
            if new.maturity != Maturity::Unannounced {
                return Err(Guard::ValuePin);
            }
        }

        Maturity::Announced { cycle } => {
            if new.cycle == cycle {
                if new.maturity != Maturity::Complete || !new.y_t.is_zero() {
                    return Err(Guard::ValuePin);
                }
            } else if new.cycle < cycle {
                if new.maturity != old.maturity {
                    return Err(Guard::ValuePin);
                }
            } else {
                return Err(Guard::ValuePin);
            }
        }

        Maturity::Complete => {
            if new.maturity != Maturity::Complete || !new.y_t.is_zero() {
                return Err(Guard::ValuePin);
            }
        }
    }

    Ok(())
}

// ´rule:verification:branch-event-projections´
//
// A nonterminal settlement has no residue projection. A terminal
// settlement has exactly one.

fn validate_branch_event_projections(
    _before: &World,
    _after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    match certificate.branch {
        BranchKind::Burn => {
            if certificate.burn.is_none()
                || certificate.clear.is_some()
                || certificate.distribution_residue.is_some()
            {
                return Err(Guard::BadAuthorization);
            }
        }

        BranchKind::Clear => {
            if certificate.clear.is_none()
                || certificate.burn.is_some()
                || certificate.distribution_residue.is_some()
            {
                return Err(Guard::BadAuthorization);
            }
        }

        BranchKind::SettleDistribution => {
            if certificate.burn.is_some() || certificate.clear.is_some() {
                return Err(Guard::BadAuthorization);
            }
        }

        _ => {
            if certificate.burn.is_some()
                || certificate.clear.is_some()
                || certificate.distribution_residue.is_some()
            {
                return Err(Guard::BadAuthorization);
            }
        }
    }

    Ok(())
}

// ´rule:verification:transaction-object-helpers´

fn consumed_objects_of_kind(
    consumed: &BTreeMap<OutPoint, Utxo>,
    kind: ObjectKind,
) -> Vec<(OutPoint, &Utxo)> {
    consumed
        .iter()
        .filter_map(|(outpoint, utxo)| {
            if object_kind(utxo) == kind {
                Some((*outpoint, utxo))
            } else {
                None
            }
        })
        .collect()
}

fn created_objects_of_kind<'a>(
    after: &'a World,
    created: &BTreeSet<OutPoint>,
    kind: ObjectKind,
) -> Vec<(OutPoint, &'a Utxo)> {
    created
        .iter()
        .filter_map(|outpoint| {
            after
                .utxos
                .get(outpoint)
                .filter(|utxo| object_kind(utxo) == kind)
                .map(|utxo| (*outpoint, utxo))
        })
        .collect()
}

fn sum_created_values(
    after: &World,
    created: &BTreeSet<OutPoint>,
    kind: ObjectKind,
) -> Result<Sat, Guard> {
    created_objects_of_kind(after, created, kind)
        .into_iter()
        .map(|(_, utxo)| utxo.value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
}

/// Reserved for the remaining branch-semantic validators (request,
/// transfer, redemption, burn, ASH, clear, maturity, sponsor
/// envelopes) arriving in the next installment.
#[allow(dead_code)]
fn sum_consumed_values(
    consumed: &BTreeMap<OutPoint, Utxo>,
    kind: ObjectKind,
) -> Result<Sat, Guard> {
    consumed_objects_of_kind(consumed, kind)
        .into_iter()
        .map(|(_, utxo)| utxo.value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
}

// ´rule:verification:state-edge-pair´

fn state_pair_from_certificate(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(PoolState, PoolState), Guard> {
    let edge = certificate.state_edge.ok_or(Guard::RootSuccession)?;

    let (input, output) = match edge {
        RootEdge::Succ { input, output } => (input, output),

        RootEdge::Term { .. } => {
            return Err(Guard::RootSuccession);
        }
    };

    let old = match before.utxo(input)?.meta {
        Meta::State(state) => state,
        _ => return Err(Guard::WrongShape),
    };

    let new = match after.utxo(output)?.meta {
        Meta::State(state) => state,
        _ => return Err(Guard::WrongShape),
    };

    Ok((old, new))
}

// ´rule:verification:issuance-lookup´

pub(crate) fn issuance_projection(
    certificate: &TransitionCertificate,
    asset: Asset,
) -> Result<Option<&CertifiedIssuance>, Guard> {
    let matches = certificate
        .canonical_partition
        .issuances
        .iter()
        .filter(|issuance| issuance.asset == asset)
        .collect::<Vec<_>>();

    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches[0])),
        _ => Err(Guard::BadIssuance),
    }
}

// ´rule:verification:admission-postconditions´
//
// Admission allows at most one plain L-BTC reward output through the
// branch shape. There is no generic sponsor envelope in the core
// admission transaction.

fn validate_admission_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let (old_state, new_state) = state_pair_from_certificate(before, after, certificate)?;

    if old_state.y()?.is_zero() {
        return Err(Guard::Sealed);
    }

    let consumed_requests = certificate
        .consumed
        .iter()
        .map(|outpoint| {
            before
                .utxos
                .get(outpoint)
                .map(|utxo| (*outpoint, utxo))
                .ok_or(Guard::NoSuch)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(_, utxo)| object_kind(utxo) == ObjectKind::DepositRequest)
        .collect::<Vec<_>>();

    if consumed_requests.is_empty()
        || consumed_requests.len() > before.constants.admission_batch_max
    {
        return Err(Guard::ZeroProgress);
    }

    let target_cycle = old_state.next_cycle()?;

    let mut expected_entitlements: BTreeMap<(OwnerKey, Cycle, Sat), usize> = BTreeMap::new();

    let mut principal_total = Sat::ZERO;
    let mut budget_total = Sat::ZERO;

    for (_outpoint, request) in &consumed_requests {
        let view = validate_request_for_admission(&before.constants, request)?;

        principal_total = principal_total.checked_add(view.deposit_principal)?;

        budget_total = budget_total.checked_add(view.admission_budget)?;

        *expected_entitlements
            .entry((view.receipt_owner, target_cycle, view.deposit_principal))
            .or_insert(0) += 1;
    }

    let created_entitlements =
        created_objects_of_kind(after, &certificate.created, ObjectKind::DepositEntitlement);

    let mut actual_entitlements: BTreeMap<(OwnerKey, Cycle, Sat), usize> = BTreeMap::new();

    for (_outpoint, entitlement) in created_entitlements {
        let view = read_entitlement(entitlement)?;

        *actual_entitlements
            .entry((view.owner, view.target_cycle, view.value))
            .or_insert(0) += 1;
    }

    if actual_entitlements != expected_entitlements {
        return Err(Guard::RecipientPin);
    }

    if new_state.omega != old_state.omega
        || new_state.y_l != old_state.y_l
        || new_state.y_t != old_state.y_t
        || new_state.cycle != old_state.cycle
        || new_state.maturity != old_state.maturity
        || new_state.q != old_state.q.checked_add(principal_total)?
    {
        return Err(Guard::ValuePin);
    }

    let resv_edge = certificate.resv_edge.ok_or(Guard::ResvWeld)?;

    let (resv_input, resv_output) = match resv_edge {
        RootEdge::Succ { input, output } => (input, output),

        RootEdge::Term { .. } => {
            return Err(Guard::ResvWeld);
        }
    };

    let old_resv = before.utxo(resv_input)?;
    let new_resv = after.utxo(resv_output)?;

    if new_resv.value != old_resv.value.checked_add(principal_total)?
        || new_resv.value != new_state.omega.checked_add(new_state.q)?
    {
        return Err(Guard::ResvWeld);
    }

    let issuance = issuance_projection(certificate, Asset::Ent)?.ok_or(Guard::BadIssuance)?;

    if issuance.amount != principal_total {
        return Err(Guard::BadIssuance);
    }

    let reward_total = sum_created_values(after, &certificate.created, ObjectKind::PlainLbtc)?;

    if budget_total != reward_total.checked_add(certificate.chain_fee)? {
        return Err(Guard::PartitionPin);
    }

    Ok(())
}

// ´def:verification:cycle-arithmetic´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CycleArithmetic {
    pub issuance: Sat,

    pub live_issuance: Sat,
    pub time_locked_issuance: Sat,

    pub operator_live: Sat,
    pub operator_time_locked: Sat,

    pub contributor_live: Sat,
    pub contributor_time_locked: Sat,

    pub next_cycle: Cycle,
    pub at_maturity: bool,
    pub normal_phase: bool,
}

// ´rule:verification:derive-cycle-arithmetic´

fn derive_cycle_arithmetic(
    constants: &Constants,
    state: PoolState,
) -> Result<CycleArithmetic, Guard> {
    let y = state.y()?;

    if y.is_zero() {
        return Err(Guard::Sealed);
    }

    let next_cycle = state.next_cycle()?;

    let at_maturity = matches!(
        state.maturity,
        Maturity::Announced {
            cycle,
        } if cycle == next_cycle
    );

    let normal_phase = at_maturity || matches!(state.maturity, Maturity::Complete);

    let issuance = if state.q.is_zero() {
        Sat::ZERO
    } else {
        floor_mul_div(state.q, y, state.omega)?
    };

    let live_issuance = if normal_phase {
        issuance
    } else {
        floor_ratio(issuance, constants.zeta)?
    };

    let time_locked_issuance = issuance.checked_sub(live_issuance)?;

    let operator_live = floor_ratio(live_issuance, constants.mint_fee)?;

    let operator_time_locked = floor_ratio(time_locked_issuance, constants.mint_fee)?;

    let contributor_live = live_issuance.checked_sub(operator_live)?;

    let contributor_time_locked = time_locked_issuance.checked_sub(operator_time_locked)?;

    Ok(CycleArithmetic {
        issuance,

        live_issuance,
        time_locked_issuance,

        operator_live,
        operator_time_locked,

        contributor_live,
        contributor_time_locked,

        next_cycle,
        at_maturity,
        normal_phase,
    })
}

// ´rule:verification:cycle-postconditions´

fn validate_cycle_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let (old_state, new_state) = state_pair_from_certificate(before, after, certificate)?;

    let arithmetic = derive_cycle_arithmetic(&before.constants, old_state)?;

    let expected_omega = old_state.omega.checked_add(old_state.q)?;

    let (expected_y_l, expected_y_t, expected_maturity) = if arithmetic.at_maturity {
        (
            old_state
                .y_l
                .checked_add(arithmetic.live_issuance)?
                .checked_add(old_state.y_t)?,
            Sat::ZERO,
            Maturity::Complete,
        )
    } else {
        (
            old_state.y_l.checked_add(arithmetic.live_issuance)?,
            old_state.y_t.checked_add(arithmetic.time_locked_issuance)?,
            old_state.maturity,
        )
    };

    if new_state.omega != expected_omega
        || new_state.q != Sat::ZERO
        || new_state.cycle != arithmetic.next_cycle
        || new_state.y_l != expected_y_l
        || new_state.y_t != expected_y_t
        || new_state.maturity != expected_maturity
    {
        return Err(Guard::ValuePin);
    }

    let old_resv = before.active_resv()?.1;
    let new_resv_outpoint = match certificate.resv_edge.ok_or(Guard::ResvWeld)? {
        RootEdge::Succ { output, .. } => output,

        RootEdge::Term { .. } => {
            return Err(Guard::ResvWeld);
        }
    };

    let new_resv = after.utxo(new_resv_outpoint)?;

    if old_resv.value != new_resv.value
        || new_resv.value != new_state.omega.checked_add(new_state.q)?
    {
        return Err(Guard::ResvWeld);
    }

    match issuance_projection(certificate, Asset::U)? {
        Some(delta) if delta.amount == arithmetic.issuance => {}

        None if arithmetic.issuance.is_zero() => {}

        _ => {
            return Err(Guard::BadIssuance);
        }
    }

    let controls =
        created_objects_of_kind(after, &certificate.created, ObjectKind::DistributionControl);

    let vaults =
        created_objects_of_kind(after, &certificate.created, ObjectKind::DistributionVault);

    if old_state.q.is_zero() {
        if !controls.is_empty() || !vaults.is_empty() {
            return Err(Guard::WrongShape);
        }

        if issuance_projection(certificate, Asset::DistCtl)?.is_some() {
            return Err(Guard::BadIssuance);
        }
    } else {
        if controls.len() != 1 {
            return Err(Guard::WrongShape);
        }

        let control = read_distribution_control(controls[0].1)?;

        if control.cycle != arithmetic.next_cycle
            || control.principal != old_state.q
            || control.live_allocation != arithmetic.contributor_live
            || control.time_locked_allocation != arithmetic.contributor_time_locked
            || control.remaining_principal != old_state.q
            || control.remaining_live_value != arithmetic.contributor_live
            || control.remaining_time_locked_value != arithmetic.contributor_time_locked
        {
            return Err(Guard::ValuePin);
        }

        let expected_vault_value = arithmetic
            .contributor_live
            .checked_add(arithmetic.contributor_time_locked)?;

        match (expected_vault_value.is_zero(), vaults.len()) {
            (true, 0) => {}

            (false, 1) => {
                if vaults[0].1.value != expected_vault_value {
                    return Err(Guard::ControlVaultWeld);
                }
            }

            _ => {
                return Err(Guard::ControlVaultWeld);
            }
        }

        let control_issuance =
            issuance_projection(certificate, Asset::DistCtl)?.ok_or(Guard::BadIssuance)?;

        if control_issuance.amount != Sat::ONE {
            return Err(Guard::BadIssuance);
        }
    }

    let operator_live_total =
        created_objects_of_kind(after, &certificate.created, ObjectKind::ReceiptLive)
            .into_iter()
            .filter_map(|(_, utxo)| match utxo.meta {
                Meta::Receipt {
                    owner,
                    class: ReceiptClass::Live,
                } if owner == OPERATOR_KEY => Some(utxo.value),

                _ => None,
            })
            .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let operator_time_locked_total =
        created_objects_of_kind(after, &certificate.created, ObjectKind::ReceiptTimeLocked)
            .into_iter()
            .filter_map(|(_, utxo)| match utxo.meta {
                Meta::Receipt {
                    owner,
                    class: ReceiptClass::TimeLocked,
                } if owner == OPERATOR_KEY => Some(utxo.value),

                _ => None,
            })
            .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    // normal_phase ⟹ time_locked_issuance = 0 ⟹ operator_time_locked = 0, so the
    // sum form below is arithmetic.operator_live plus an always-zero term; it is
    // kept in sum form so the pin mirrors the fee split rather than assuming it.
    let expected_operator_live = if arithmetic.normal_phase {
        arithmetic
            .operator_live
            .checked_add(arithmetic.operator_time_locked)?
    } else {
        arithmetic.operator_live
    };

    let expected_operator_time_locked = if arithmetic.normal_phase {
        Sat::ZERO
    } else {
        arithmetic.operator_time_locked
    };

    if operator_live_total != expected_operator_live
        || operator_time_locked_total != expected_operator_time_locked
    {
        return Err(Guard::RecipientPin);
    }

    let anchor_count =
        created_objects_of_kind(after, &certificate.created, ObjectKind::CpfpAnchor).len();

    if arithmetic.at_maturity {
        if anchor_count != 1 {
            return Err(Guard::WrongShape);
        }
    } else if anchor_count != 0 {
        return Err(Guard::WrongShape);
    }

    Ok(())
}

// ´rule:verification:settlement-postconditions´
//
// The consumed objects are read back from the predecessor world, which
// still contains every certificate input; this is equivalent to
// receiving the builder's consumed map directly.

fn validate_settlement_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let consumed = certificate
        .consumed
        .iter()
        .map(|outpoint| Ok((*outpoint, *before.utxo(*outpoint)?)))
        .collect::<Result<BTreeMap<OutPoint, Utxo>, Guard>>()?;

    let controls = consumed_objects_of_kind(&consumed, ObjectKind::DistributionControl);

    if controls.len() != 1 {
        return Err(Guard::WrongShape);
    }

    let (control_outpoint, control_utxo) = controls[0];

    let control = read_distribution_control(control_utxo)?;

    let entitlement_inputs = consumed
        .iter()
        .filter_map(|(outpoint, utxo)| match read_entitlement(utxo) {
            Ok(view) => Some((*outpoint, view)),

            Err(_) => None,
        })
        .collect::<Vec<_>>();

    if entitlement_inputs.is_empty() {
        return Err(Guard::ZeroProgress);
    }

    let mut expected_receipts: BTreeMap<(OwnerKey, ReceiptClass), Sat> = BTreeMap::new();

    let mut settled_principal = Sat::ZERO;

    let mut live_total = Sat::ZERO;

    let mut time_locked_total = Sat::ZERO;

    for (_outpoint, entitlement) in &entitlement_inputs {
        if entitlement.target_cycle != control.cycle {
            return Err(Guard::WrongTarget);
        }

        let live = floor_mul_div(
            entitlement.value,
            control.live_allocation,
            control.principal,
        )?;

        let time_locked = floor_mul_div(
            entitlement.value,
            control.time_locked_allocation,
            control.principal,
        )?;

        settled_principal = settled_principal.checked_add(entitlement.value)?;

        live_total = live_total.checked_add(live)?;

        time_locked_total = time_locked_total.checked_add(time_locked)?;

        if !live.is_zero() {
            checked_add_to_map(
                &mut expected_receipts,
                (entitlement.owner, ReceiptClass::Live),
                live,
            )?;
        }

        if !time_locked.is_zero() {
            checked_add_to_map(
                &mut expected_receipts,
                (entitlement.owner, ReceiptClass::TimeLocked),
                time_locked,
            )?;
        }
    }

    let mut actual_receipts: BTreeMap<(OwnerKey, ReceiptClass), Sat> = BTreeMap::new();

    for outpoint in &certificate.created {
        let utxo = after.utxo(*outpoint)?;

        if let Ok(receipt) = read_receipt(utxo) {
            checked_add_to_map(
                &mut actual_receipts,
                (receipt.owner, receipt.class),
                receipt.value,
            )?;
        }
    }

    if actual_receipts != expected_receipts {
        return Err(Guard::RecipientPin);
    }

    let remaining_principal = control.remaining_principal.checked_sub(settled_principal)?;

    let remaining_live = control.remaining_live_value.checked_sub(live_total)?;

    let remaining_time_locked = control
        .remaining_time_locked_value
        .checked_sub(time_locked_total)?;

    let successor_controls =
        created_objects_of_kind(after, &certificate.created, ObjectKind::DistributionControl);

    let successor_vaults =
        created_objects_of_kind(after, &certificate.created, ObjectKind::DistributionVault);

    if remaining_principal.is_zero() {
        if !successor_controls.is_empty()
            || !successor_vaults.is_empty()
            || certificate.distribution_residue.is_none()
        {
            return Err(Guard::WrongShape);
        }

        let residue = certificate
            .distribution_residue
            .as_ref()
            .ok_or(Guard::BadDestruction)?;

        if residue.control_input != control_outpoint
            || residue.live_residue != remaining_live
            || residue.time_locked_residue != remaining_time_locked
        {
            return Err(Guard::BadDestruction);
        }
    } else {
        if successor_controls.len() != 1 {
            return Err(Guard::WrongShape);
        }

        let successor = read_distribution_control(successor_controls[0].1)?;

        if successor.cycle != control.cycle
            || successor.principal != control.principal
            || successor.live_allocation != control.live_allocation
            || successor.time_locked_allocation != control.time_locked_allocation
            || successor.remaining_principal != remaining_principal
            || successor.remaining_live_value != remaining_live
            || successor.remaining_time_locked_value != remaining_time_locked
        {
            return Err(Guard::ValuePin);
        }

        if certificate.distribution_residue.is_some() {
            return Err(Guard::BadDestruction);
        }

        let expected_vault = remaining_live.checked_add(remaining_time_locked)?;

        match (expected_vault.is_zero(), successor_vaults.len()) {
            (true, 0) => {}

            (false, 1) => {
                if successor_vaults[0].1.value != expected_vault {
                    return Err(Guard::ControlVaultWeld);
                }
            }

            _ => {
                return Err(Guard::ControlVaultWeld);
            }
        }
    }

    Ok(())
}

// ´rule:verification:relabel-positional-bijection´
//
// The model proves the owner/value multiset bijection: input
// time-locked receipts and output live receipts must have equal counts
// and identical (owner, value) multisets, so relabel can never
// redistribute value or owners. Exact output *positions* (STATE
// successor first; one live receipt per listed input in input order;
// then optional sponsor change) are a compiler layout obligation
// pinned by the emitted script, not a model-level check.

fn validate_relabel_positional_bijection(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let state_edge = certificate.state_edge.ok_or(Guard::RootSuccession)?;

    let (state_input, state_output) = match state_edge {
        RootEdge::Succ { input, output } => (input, output),

        RootEdge::Term { .. } => {
            return Err(Guard::RootSuccession);
        }
    };

    let old_state = match before.utxo(state_input)?.meta {
        Meta::State(state) => state,
        _ => return Err(Guard::WrongShape),
    };

    let new_state = match after.utxo(state_output)?.meta {
        Meta::State(state) => state,
        _ => return Err(Guard::WrongShape),
    };

    if old_state != new_state || old_state.maturity != Maturity::Complete {
        return Err(Guard::ValuePin);
    }

    let input_receipts = certificate
        .consumed
        .iter()
        .filter_map(|outpoint| {
            before.utxos.get(outpoint).and_then(|utxo| {
                read_receipt(utxo)
                    .ok()
                    .filter(|receipt| receipt.class == ReceiptClass::TimeLocked)
                    .map(|receipt| (*outpoint, receipt))
            })
        })
        .collect::<Vec<_>>();

    let output_receipts = certificate
        .created
        .iter()
        .filter_map(|outpoint| {
            after.utxos.get(outpoint).and_then(|utxo| {
                read_receipt(utxo)
                    .ok()
                    .filter(|receipt| receipt.class == ReceiptClass::Live)
                    .map(|receipt| (*outpoint, receipt))
            })
        })
        .collect::<Vec<_>>();

    if input_receipts.len() != output_receipts.len() || input_receipts.is_empty() {
        return Err(Guard::RecipientPin);
    }

    let mut input_multiset: BTreeMap<(OwnerKey, Sat), usize> = BTreeMap::new();

    let mut output_multiset: BTreeMap<(OwnerKey, Sat), usize> = BTreeMap::new();

    for (_outpoint, receipt) in input_receipts {
        *input_multiset
            .entry((receipt.owner, receipt.value))
            .or_insert(0) += 1;
    }

    for (_outpoint, receipt) in output_receipts {
        *output_multiset
            .entry((receipt.owner, receipt.value))
            .or_insert(0) += 1;
    }

    if input_multiset != output_multiset {
        return Err(Guard::RecipientPin);
    }

    Ok(())
}

// ´rule:verification:open-flow-object-helpers´

fn flow_sources<'a>(
    before: &'a World,
    flow: &OpenFlowProjection,
) -> Result<Vec<(OutPoint, &'a Utxo)>, Guard> {
    flow.source_inputs
        .iter()
        .map(|outpoint| Ok((*outpoint, before.utxo(*outpoint)?)))
        .collect()
}

fn flow_destinations<'a>(
    after: &'a World,
    flow: &OpenFlowProjection,
) -> Result<Vec<(OutPoint, &'a Utxo)>, Guard> {
    flow.destination_outputs
        .iter()
        .map(|outpoint| Ok((*outpoint, after.utxo(*outpoint)?)))
        .collect()
}

fn open_flows_of_kind(
    certificate: &TransitionCertificate,
    kind: OpenFlowKind,
) -> Vec<&OpenFlowProjection> {
    certificate
        .open_flows
        .iter()
        .filter(|flow| flow.kind == kind)
        .collect()
}

fn require_one_open_flow(
    certificate: &TransitionCertificate,
    kind: OpenFlowKind,
) -> Result<&OpenFlowProjection, Guard> {
    let matches = open_flows_of_kind(certificate, kind);

    if matches.len() == 1 {
        Ok(matches[0])
    } else {
        Err(Guard::OpenFlowMismatch)
    }
}

// ´rule:verification:open-flow-semantics´
//
// Recipient and amount checks that depend on branch arithmetic remain
// in the branch-specific validators.

fn validate_open_flow_semantics(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    for flow in &certificate.open_flows {
        if !branch_allows_open_flow(certificate.branch, flow.kind) {
            return Err(Guard::BadAuthorization);
        }

        let sources = flow_sources(before, flow)?;

        let destinations = flow_destinations(after, flow)?;

        match flow.kind {
            OpenFlowKind::RequestCreation => {
                if destinations
                    .iter()
                    .filter(|(_, utxo)| matches!(utxo.meta, Meta::DepositRequest { .. }))
                    .count()
                    != 1
                {
                    return Err(Guard::WrongShape);
                }

                if sources.iter().any(|(_, utxo)| {
                    !matches!(
                        (utxo.asset, utxo.meta),
                        (Asset::Lbtc, Meta::PlainLbtc { .. }),
                    )
                }) {
                    return Err(Guard::WrongShape);
                }
            }

            OpenFlowKind::RequestRefund => {
                if sources.len() != 1 || destinations.len() != 1 || !flow.fee.is_zero() {
                    return Err(Guard::OpenFlowMismatch);
                }

                let source = sources[0].1;
                let destination = destinations[0].1;

                let refund_key = match source.meta {
                    Meta::DepositRequest { refund_key, .. } => refund_key,

                    _ => {
                        return Err(Guard::WrongShape);
                    }
                };

                match destination.meta {
                    Meta::PlainLbtc { owner }
                        if owner == refund_key && destination.value == source.value => {}

                    _ => {
                        return Err(Guard::RecipientPin);
                    }
                }
            }

            OpenFlowKind::DepositAdmission => {
                if sources
                    .iter()
                    .filter(|(_, utxo)| matches!(utxo.meta, Meta::Resv))
                    .count()
                    != 1
                    || destinations
                        .iter()
                        .filter(|(_, utxo)| matches!(utxo.meta, Meta::Resv))
                        .count()
                        != 1
                {
                    return Err(Guard::ResvWeld);
                }

                if sources
                    .iter()
                    .any(|(_, utxo)| !matches!(utxo.meta, Meta::Resv | Meta::DepositRequest { .. }))
                {
                    return Err(Guard::WrongShape);
                }

                if destinations
                    .iter()
                    .any(|(_, utxo)| !matches!(utxo.meta, Meta::Resv | Meta::PlainLbtc { .. }))
                {
                    return Err(Guard::WrongShape);
                }

                if destinations
                    .iter()
                    .filter(|(_, utxo)| matches!(utxo.meta, Meta::PlainLbtc { .. }))
                    .count()
                    > 1
                {
                    return Err(Guard::WrongShape);
                }
            }

            OpenFlowKind::ReserveCarry => {
                if sources.len() != 1 || destinations.len() != 1 || !flow.fee.is_zero() {
                    return Err(Guard::OpenFlowMismatch);
                }

                if !matches!(sources[0].1.meta, Meta::Resv)
                    || !matches!(destinations[0].1.meta, Meta::Resv)
                    || sources[0].1.value != destinations[0].1.value
                {
                    return Err(Guard::ResvWeld);
                }
            }

            OpenFlowKind::Redemption => {
                if sources.len() != 1
                    || !flow.fee.is_zero()
                    || !matches!(sources[0].1.meta, Meta::Resv)
                {
                    return Err(Guard::OpenFlowMismatch);
                }

                if destinations
                    .iter()
                    .filter(|(_, utxo)| matches!(utxo.meta, Meta::Resv))
                    .count()
                    > 1
                    || destinations
                        .iter()
                        .filter(|(_, utxo)| matches!(utxo.meta, Meta::PlainLbtc { .. }))
                        .count()
                        != 1
                {
                    return Err(Guard::WrongShape);
                }
            }

            OpenFlowKind::FeeSponsor => {
                if sources.iter().any(|(_, utxo)| {
                    !matches!(
                        (utxo.asset, utxo.meta),
                        (Asset::Lbtc, Meta::PlainLbtc { .. }),
                    )
                }) {
                    return Err(Guard::SponsorMismatch);
                }

                if destinations.iter().any(|(_, utxo)| {
                    !matches!(
                        (utxo.asset, utxo.meta),
                        (Asset::Lbtc, Meta::PlainLbtc { .. }),
                    )
                }) || destinations.len() > 1
                {
                    return Err(Guard::SponsorMismatch);
                }
            }
        }
    }

    let sponsor_count = open_flows_of_kind(certificate, OpenFlowKind::FeeSponsor).len();

    if sponsor_count > 1 {
        return Err(Guard::SponsorMismatch);
    }

    Ok(())
}

// ´rule:verification:create-request-postconditions´

fn validate_create_request_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let flow = require_one_open_flow(certificate, OpenFlowKind::RequestCreation)?;

    let destinations = flow_destinations(after, flow)?;

    let requests = destinations
        .iter()
        .filter(|(_, utxo)| matches!(utxo.meta, Meta::DepositRequest { .. }))
        .collect::<Vec<_>>();

    if requests.len() != 1 {
        return Err(Guard::WrongShape);
    }

    let request = requests[0].1;

    match request.meta {
        Meta::DepositRequest {
            pool_id,
            deposit_principal,
            ..
        } => {
            if pool_id != before.constants.pool_id
                || deposit_principal.is_zero()
                || deposit_principal >= request.value
            {
                return Err(Guard::PartitionPin);
            }
        }

        _ => {
            return Err(Guard::WrongShape);
        }
    }

    if certificate.state_edge.is_some()
        || certificate.resv_edge.is_some()
        || certificate.pace_edge.is_some()
    {
        return Err(Guard::RootSuccession);
    }

    Ok(())
}

// ´rule:verification:cancel-request-postconditions´
//
// Authorization of `refund_key` remains checked by the transition
// constructor's signer set.

fn validate_cancel_request_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let refund_flow = require_one_open_flow(certificate, OpenFlowKind::RequestRefund)?;

    let sources = flow_sources(before, refund_flow)?;

    let destinations = flow_destinations(after, refund_flow)?;

    if sources.len() != 1 || destinations.len() != 1 {
        return Err(Guard::WrongShape);
    }

    let request = sources[0].1;

    let refund_key = match request.meta {
        Meta::DepositRequest { refund_key, .. } => refund_key,

        _ => return Err(Guard::WrongShape),
    };

    match destinations[0].1.meta {
        Meta::PlainLbtc { owner }
            if owner == refund_key && destinations[0].1.value == request.value => {}

        _ => {
            return Err(Guard::RecipientPin);
        }
    }

    Ok(())
}

// ´rule:verification:transfer-postconditions´
//
// Per-owner authorization remains part of the constructor and the
// signer-set evidence boundary.

fn validate_transfer_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
    expected_class: ReceiptClass,
) -> Result<(), Guard> {
    let mut input_total = Sat::ZERO;
    let mut output_total = Sat::ZERO;

    for outpoint in &certificate.consumed {
        let utxo = before.utxo(*outpoint)?;

        if utxo.asset == Asset::U {
            let receipt = read_receipt(utxo)?;

            if receipt.class != expected_class {
                return Err(Guard::ClassCross);
            }

            input_total = input_total.checked_add(receipt.value)?;
        }
    }

    for outpoint in &certificate.created {
        let utxo = after.utxo(*outpoint)?;

        if utxo.asset == Asset::U {
            let receipt = read_receipt(utxo)?;

            if receipt.class != expected_class {
                return Err(Guard::ClassCross);
            }

            output_total = output_total.checked_add(receipt.value)?;
        }
    }

    if input_total.is_zero() || output_total.is_zero() || input_total != output_total {
        return Err(Guard::ValuePin);
    }

    Ok(())
}

// ´rule:verification:redemption-postconditions´

fn validate_redemption_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let (old_state, new_state) = state_pair_from_certificate(before, after, certificate)?;

    let receipt_inputs = certificate
        .consumed
        .iter()
        .filter_map(|outpoint| {
            before
                .utxos
                .get(outpoint)
                .and_then(|utxo| read_receipt(utxo).ok().map(|receipt| (*outpoint, receipt)))
        })
        .collect::<Vec<_>>();

    if receipt_inputs.len() != 1 {
        return Err(Guard::WrongShape);
    }

    let (_receipt_outpoint, receipt) = receipt_inputs[0];

    if receipt.class != ReceiptClass::Live {
        return Err(Guard::ClassCross);
    }

    let y = old_state.y()?;

    let expected_payout = floor_mul_div(receipt.value, old_state.omega, y)?;

    let redemption_flow = require_one_open_flow(certificate, OpenFlowKind::Redemption)?;

    let destinations = flow_destinations(after, redemption_flow)?;

    let payout_outputs = destinations
        .iter()
        .filter(|(_, utxo)| matches!(utxo.meta, Meta::PlainLbtc { .. }))
        .collect::<Vec<_>>();

    if payout_outputs.len() != 1 {
        return Err(Guard::WrongShape);
    }

    match payout_outputs[0].1.meta {
        Meta::PlainLbtc { owner }
            if owner == receipt.owner && payout_outputs[0].1.value == expected_payout => {}

        _ => {
            return Err(Guard::RecipientPin);
        }
    }

    let sealing = receipt.value == y && old_state.q.is_zero();

    if sealing {
        if !new_state.is_sealed()?
            || certificate.resv_edge
                != Some(RootEdge::Term {
                    input: before.roots.resv.ok_or(Guard::Sealed)?,
                })
        {
            return Err(Guard::ResvWeld);
        }
    } else {
        let expected_omega = old_state.omega.checked_sub(expected_payout)?;

        let expected_y_l = old_state.y_l.checked_sub(receipt.value)?;

        if new_state.omega != expected_omega
            || new_state.y_l != expected_y_l
            || new_state.y_t != old_state.y_t
            || new_state.q != old_state.q
            || new_state.cycle != old_state.cycle
            || new_state.maturity != old_state.maturity
        {
            return Err(Guard::ValuePin);
        }
    }

    Ok(())
}

// ´rule:verification:burn-postconditions´
//
// The transition remains valid regardless of the off-chain attestation
// gate result.

fn validate_burn_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let burn = certificate.burn.as_ref().ok_or(Guard::BadAuthorization)?;

    let mut input_total = Sat::ZERO;

    for outpoint in &certificate.consumed {
        let utxo = before.utxo(*outpoint)?;

        if utxo.asset == Asset::U {
            let receipt = read_receipt(utxo)?;

            if receipt.class != ReceiptClass::Live {
                return Err(Guard::ClassCross);
            }

            input_total = input_total.checked_add(receipt.value)?;
        }
    }

    let mut change_total = Sat::ZERO;
    let mut ash_count = 0_usize;

    for outpoint in &certificate.created {
        let utxo = after.utxo(*outpoint)?;

        if utxo.asset != Asset::U {
            continue;
        }

        match utxo.meta {
            Meta::Ash => {
                ash_count += 1;

                if *outpoint != burn.ash_output || utxo.value != burn.ash_value {
                    return Err(Guard::ValuePin);
                }
            }

            Meta::Receipt {
                class: ReceiptClass::Live,
                ..
            } => {
                change_total = change_total.checked_add(utxo.value)?;
            }

            _ => {
                return Err(Guard::ClassCross);
            }
        }
    }

    if ash_count != 1 || burn.ash_value.checked_add(change_total)? != input_total {
        return Err(Guard::ValuePin);
    }

    // A burn transaction remains valid however much its records
    // over-claim. The `Σ records <= ash_value` credit gate is a per-query
    // verdict applied off-chain by the reference indexer
    // (`BurnTransaction::records_accepted`), which voids all records of an
    // over-claiming burn rather than invalidating the burn itself. Each
    // record amount is already a positive `Sat` and the record count is
    // bounded (`burn_record_max`), so there is no aggregate condition to
    // enforce here — and summing in `Sat` would wrongly reject a
    // legitimate over-claim whose aggregate exceeds the `Sat` maximum
    // (up to `burn_record_max * (2^51 - 1)`).

    Ok(())
}

// ´rule:verification:ash-compaction-postconditions´

fn validate_ash_compaction_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let input_total = certificate
        .consumed
        .iter()
        .filter_map(|outpoint| {
            before
                .utxos
                .get(outpoint)
                .and_then(|utxo| read_ash(utxo).ok())
        })
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let outputs = created_objects_of_kind(after, &certificate.created, ObjectKind::Ash);

    if input_total.is_zero()
        || outputs.len() != 1
        || outputs[0].1.value != input_total
        || certificate.burn.is_some()
    {
        return Err(Guard::ValuePin);
    }

    Ok(())
}

// ´rule:verification:clear-postconditions´

fn validate_clear_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let (old_state, new_state) = state_pair_from_certificate(before, after, certificate)?;

    let ash_total = certificate
        .consumed
        .iter()
        .filter_map(|outpoint| {
            before
                .utxos
                .get(outpoint)
                .and_then(|utxo| read_ash(utxo).ok())
        })
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let y = old_state.y()?;

    let clear_amount = ash_total.min(old_state.y_l).min(y.checked_sub(Sat::ONE)?);

    if clear_amount.is_zero() {
        return Err(Guard::ZeroProgress);
    }

    if new_state.y_l != old_state.y_l.checked_sub(clear_amount)?
        || new_state.omega != old_state.omega
        || new_state.q != old_state.q
        || new_state.y_t != old_state.y_t
        || new_state.cycle != old_state.cycle
        || new_state.maturity != old_state.maturity
    {
        return Err(Guard::ValuePin);
    }

    let residual = ash_total.checked_sub(clear_amount)?;

    let ash_outputs = created_objects_of_kind(after, &certificate.created, ObjectKind::Ash);

    match (residual.is_zero(), ash_outputs.len()) {
        (true, 0) => {}

        (false, 1) if ash_outputs[0].1.value == residual => {}

        _ => {
            return Err(Guard::ValuePin);
        }
    }

    let clear = certificate.clear.ok_or(Guard::BadAuthorization)?;

    if clear.omega != new_state.omega || clear.y != new_state.y()? || certificate.burn.is_some() {
        return Err(Guard::BadAuthorization);
    }

    Ok(())
}

// ´rule:verification:maturity-announcement-postconditions´
//
// Operator authorization remains a transition-constructor check.

fn validate_maturity_announcement_postconditions(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let (old_state, new_state) = state_pair_from_certificate(before, after, certificate)?;

    let announced_cycle = match (old_state.maturity, new_state.maturity) {
        (Maturity::Unannounced, Maturity::Announced { cycle }) => cycle,

        _ => {
            return Err(Guard::MaturityAlreadyAnnounced);
        }
    };

    let earliest = old_state
        .cycle
        .checked_add(before.constants.min_maturity_lead)
        .ok_or(Guard::CycleOverflow)?;

    let latest = old_state
        .cycle
        .checked_add(before.constants.max_maturity_lead)
        .ok_or(Guard::CycleOverflow)?;

    if announced_cycle < earliest {
        return Err(Guard::MaturityLeadTooShort);
    }

    if announced_cycle > latest {
        return Err(Guard::MaturityLeadTooLong);
    }

    if old_state.omega != new_state.omega
        || old_state.y_l != new_state.y_l
        || old_state.y_t != new_state.y_t
        || old_state.q != new_state.q
        || old_state.cycle != new_state.cycle
    {
        return Err(Guard::ValuePin);
    }

    Ok(())
}

// ´rule:verification:branch-semantic-postconditions´
//
// `validate_branch_shape_postcommit()` invokes this dispatcher after
// root and projection checks.

fn validate_branch_semantics(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    validate_open_flow_semantics(before, after, certificate)?;

    match certificate.branch {
        BranchKind::CreateRequest => {
            validate_create_request_postconditions(before, after, certificate)
        }

        BranchKind::CancelRequest => {
            validate_cancel_request_postconditions(before, after, certificate)
        }

        BranchKind::AdmitDeposits => validate_admission_postconditions(before, after, certificate),

        BranchKind::Cycle => validate_cycle_postconditions(before, after, certificate),

        BranchKind::SettleDistribution => {
            validate_settlement_postconditions(before, after, certificate)
        }

        BranchKind::TransferLive => {
            validate_transfer_postconditions(before, after, certificate, ReceiptClass::Live)
        }

        BranchKind::TransferTimeLocked => {
            validate_transfer_postconditions(before, after, certificate, ReceiptClass::TimeLocked)
        }

        BranchKind::Redeem => validate_redemption_postconditions(before, after, certificate),

        BranchKind::ReceiptRelabel => {
            validate_relabel_positional_bijection(before, after, certificate)
        }

        BranchKind::Burn => validate_burn_postconditions(before, after, certificate),

        BranchKind::CompactAsh => {
            validate_ash_compaction_postconditions(before, after, certificate)
        }

        BranchKind::Clear => validate_clear_postconditions(before, after, certificate),

        BranchKind::AnnounceMaturity => {
            validate_maturity_announcement_postconditions(before, after, certificate)
        }
    }
}
