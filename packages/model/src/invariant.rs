//! Invariant checker over \((\Gamma, H)\).
//!
//! Implements `´def:verification:accounting-folds´`,
//! `´rule:verification:derive-residue-history´`,
//! `´rule:verification:root-exactness´`,
//! `´rule:verification:state-projection-fold´`,
//! `´def:verification:replayed-roots´`,
//! `´rule:verification:certificate-set-integrity´`,
//! `´rule:verification:root-edge-membership´`,
//! `´rule:verification:replay-nonterminating-root´`,
//! `´rule:verification:replay-resv-root´`,
//! `´rule:verification:full-root-history-replay´`,
//! `´rule:verification:replayed-root-cursor-match´`,
//! `´rule:verification:distribution-bijection´`,
//! `´rule:verification:entitlement-lifecycle-check´`,
//! `´rule:verification:receipt-accounting-check´`,
//! `´rule:verification:maturity-coherence´`, and
//! `´def:verification:invariant-checker´`.
//!
//! History is audit-only. No branch reads the historical residue
//! values derived here.

use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigUint;
use num_traits::Zero;

use crate::asset::{Asset, Maturity, ReceiptClass};
use crate::guard::{Guard, InvariantError};
use crate::history::{History, RootEdge, TransitionCertificate};
use crate::policy::{RootUse, branch_policy};
use crate::pool::PoolState;
use crate::recognition::{
    CanonicalObject, DistributionControlView, StateClass, classify_state_object,
    read_distribution_control,
};
use crate::scalar::{Cycle, OutPoint, Sat, checked_active_backing, floor_mul_div};
use crate::world::World;

// ´rule:verification:clause-weld´

use architecture::InvariantClauseId;

/// Maps each failure reason onto the invariant clause it violates.
///
/// Many-to-one: reasons are finer than clauses; the oracle binds to
/// reasons, the document to clauses. The match is exhaustive, so a new
/// reason without a clause is a compile failure.
///
/// `ConsensusValueAuthority` is declared but never produced: in this
/// model a committed-vs-consensus value divergence is unrepresentable
/// (`Utxo.value` is the one value), so the consensus-value clause's
/// on-chain discharge is the `ExplicitValueIntrospection` dependency plus
/// introspection — structural evidence, not a runtime check. Do not
/// "add the missing check"; there is nothing for it to compare.
pub const fn clause_of(error: InvariantError) -> InvariantClauseId {
    match error {
        // 𝗜₁ absorbs canonical closure.
        InvariantError::IdentityAuthority | InvariantError::CanonicalClosure => {
            InvariantClauseId::Identity
        }

        InvariantError::Domains | InvariantError::ActiveBackingCap => InvariantClauseId::Domains,

        InvariantError::Floor => InvariantClauseId::RateFloor,

        InvariantError::SealedTerminal => InvariantClauseId::NoTrap,

        InvariantError::Backing => InvariantClauseId::Backing,

        InvariantError::DistributionPayability => InvariantClauseId::NoStarve,

        InvariantError::EntitlementLifecycle => InvariantClauseId::EscrowReceipts,

        InvariantError::ReceiptAccountingPreMaturity
        | InvariantError::ReceiptAccountingPostMaturity => InvariantClauseId::Accounting,

        InvariantError::ConsensusValueAuthority => InvariantClauseId::ConsensusValue,

        InvariantError::StateSuccession
        | InvariantError::ResvSuccession
        | InvariantError::HistoryProjection => InvariantClauseId::Succession,

        InvariantError::MaturityCoherence => InvariantClauseId::Maturity,
    }
}

// ´def:verification:accounting-folds´

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountingFold {
    pub circulating_live: BigUint,
    pub circulating_time_locked: BigUint,

    pub ash: BigUint,

    pub distribution_live: BigUint,
    pub distribution_time_locked: BigUint,

    pub outstanding_entitlement: BigUint,

    pub entitlement_by_target: BTreeMap<Cycle, BigUint>,

    pub controls: BTreeMap<Cycle, DistributionControlView>,

    pub vault_count: BTreeMap<Cycle, usize>,

    pub vault_value: BTreeMap<Cycle, BigUint>,

    pub historical_live_residue: BigUint,
    pub historical_time_locked_residue: BigUint,
}

// ´rule:verification:derive-residue-history´

fn derive_historical_residue(history: &History) -> Result<(BigUint, BigUint), InvariantError> {
    let mut live = BigUint::zero();
    let mut time_locked = BigUint::zero();

    for (_certificate, residue) in history.residue_projections() {
        live += BigUint::from(residue.live_residue.get());

        time_locked += BigUint::from(residue.time_locked_residue.get());
    }

    Ok((live, time_locked))
}

// ´rule:verification:root-exactness´

fn check_root_exactness(world: &World) -> Result<(), InvariantError> {
    let expected = [
        (Asset::Pid, world.roots.state),
        (Asset::Pace, world.roots.pace),
        (Asset::EntAuth, world.roots.entitlement_authority),
        (Asset::DistAuth, world.roots.distribution_authority),
    ];

    for (asset, expected_outpoint) in expected {
        let all = world
            .utxos
            .iter()
            .filter(|(_, utxo)| utxo.asset == asset)
            .collect::<Vec<_>>();

        if all.len() != 1 {
            return Err(InvariantError::IdentityAuthority);
        }

        let (outpoint, utxo) = all[0];

        if *outpoint != expected_outpoint || utxo.value != Sat::ONE {
            return Err(InvariantError::IdentityAuthority);
        }
    }

    Ok(())
}

// ´rule:verification:state-projection-fold´

pub(crate) fn fold_accounting(world: &World) -> Result<AccountingFold, InvariantError> {
    let mut fold = AccountingFold::default();

    for (outpoint, utxo) in &world.utxos {
        match classify_state_object(world, *outpoint, utxo) {
            StateClass::InertExternal => {}

            StateClass::CanonicalViolation => {
                return Err(InvariantError::CanonicalClosure);
            }

            StateClass::ActiveResv => {}

            StateClass::Canonical(canonical) => match canonical {
                CanonicalObject::State
                | CanonicalObject::Pace
                | CanonicalObject::EntitlementAuthority
                | CanonicalObject::DistributionAuthority => {}

                CanonicalObject::Receipt {
                    class: ReceiptClass::Live,
                    ..
                } => {
                    fold.circulating_live += BigUint::from(utxo.value.get());
                }

                CanonicalObject::Receipt {
                    class: ReceiptClass::TimeLocked,
                    ..
                } => {
                    fold.circulating_time_locked += BigUint::from(utxo.value.get());
                }

                CanonicalObject::DepositEntitlement { target_cycle, .. } => {
                    fold.outstanding_entitlement += BigUint::from(utxo.value.get());

                    let entry = fold
                        .entitlement_by_target
                        .entry(target_cycle)
                        .or_insert_with(BigUint::zero);

                    *entry += BigUint::from(utxo.value.get());
                }

                CanonicalObject::DistributionControl => {
                    let view =
                        read_distribution_control(utxo).map_err(|_| InvariantError::Domains)?;

                    if fold.controls.insert(view.cycle, view).is_some() {
                        return Err(InvariantError::EntitlementLifecycle);
                    }

                    fold.distribution_live += BigUint::from(view.remaining_live_value.get());

                    fold.distribution_time_locked +=
                        BigUint::from(view.remaining_time_locked_value.get());
                }

                CanonicalObject::DistributionVault { cycle } => {
                    *fold.vault_count.entry(cycle).or_insert(0) += 1;

                    let entry = fold.vault_value.entry(cycle).or_insert_with(BigUint::zero);

                    *entry += BigUint::from(utxo.value.get());
                }

                CanonicalObject::Ash => {
                    fold.ash += BigUint::from(utxo.value.get());
                }
            },
        }
    }

    let (historical_live_residue, historical_time_locked_residue) =
        derive_historical_residue(&world.history)?;

    fold.historical_live_residue = historical_live_residue;

    fold.historical_time_locked_residue = historical_time_locked_residue;

    Ok(fold)
}

// ´def:verification:replayed-roots´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReplayedRoots {
    state: OutPoint,
    resv: Option<OutPoint>,
    pace: OutPoint,
    entitlement_authority: OutPoint,
    distribution_authority: OutPoint,

    /// Set once the RESV chain terminates through sealing redemption.
    ///
    /// After this point no further STATE-spending pool transition is
    /// valid. Off-pool transitions such as request creation,
    /// cancellation, transfer, burn, settlement, and ASH compaction may
    /// still appear in history because they do not mutate the sealed
    /// pool state.
    sealed_state: Option<OutPoint>,
}

// ´rule:verification:certificate-set-integrity´
//
// This does not replay all non-root UTXO semantics; the transition
// kernel remains the authority for full input/output correctness. It
// ensures the root replay is not operating over obviously inconsistent
// certificate sets.

fn check_certificate_sets(
    certificate: &TransitionCertificate,
    seen_consumed: &mut BTreeSet<OutPoint>,
    seen_created: &mut BTreeSet<OutPoint>,
) -> Result<(), InvariantError> {
    if !certificate.consumed.is_disjoint(&certificate.created) {
        return Err(InvariantError::HistoryProjection);
    }

    for input in &certificate.consumed {
        if !seen_consumed.insert(*input) {
            // One abstract outpoint may be consumed only once.
            return Err(InvariantError::HistoryProjection);
        }
    }

    for output in &certificate.created {
        if !seen_created.insert(*output) {
            // One abstract output identity may be created only once.
            return Err(InvariantError::HistoryProjection);
        }

        if seen_consumed.contains(output) {
            // An output may not be created after it was already
            // consumed by an earlier transition.
            return Err(InvariantError::HistoryProjection);
        }
    }

    Ok(())
}

// ´rule:verification:root-edge-membership´

fn check_succ_edge_membership(
    certificate: &TransitionCertificate,
    input: OutPoint,
    output: OutPoint,
) -> Result<(), InvariantError> {
    if input == output {
        return Err(InvariantError::HistoryProjection);
    }

    if !certificate.consumed.contains(&input) {
        return Err(InvariantError::HistoryProjection);
    }

    if !certificate.created.contains(&output) {
        return Err(InvariantError::HistoryProjection);
    }

    Ok(())
}

fn check_term_edge_membership(
    certificate: &TransitionCertificate,
    input: OutPoint,
) -> Result<(), InvariantError> {
    if !certificate.consumed.contains(&input) {
        return Err(InvariantError::HistoryProjection);
    }

    Ok(())
}

// ´rule:verification:replay-nonterminating-root´
//
// Used for STATE, PACE, ENT_AUTH, and DIST_AUTH. The error variant is
// generic; the correctness property does not depend on distinguishing
// PACE/authority succession failures from STATE succession failures.

fn replay_nonterminating_root(
    current: OutPoint,
    edge: Option<RootEdge>,
    use_policy: RootUse,
    certificate: &TransitionCertificate,
) -> Result<OutPoint, InvariantError> {
    match use_policy {
        RootUse::Forbidden => {
            if edge.is_some() || certificate.consumed.contains(&current) {
                return Err(InvariantError::StateSuccession);
            }

            Ok(current)
        }

        RootUse::Succession => {
            let Some(RootEdge::Succ { input, output }) = edge else {
                return Err(InvariantError::StateSuccession);
            };

            if input != current {
                return Err(InvariantError::StateSuccession);
            }

            check_succ_edge_membership(certificate, input, output)?;

            Ok(output)
        }

        RootUse::SuccessionOrTermination => {
            // STATE, PACE, and authority roots never terminate.
            Err(InvariantError::StateSuccession)
        }
    }
}

// ´rule:verification:replay-resv-root´

fn replay_resv_root(
    current: Option<OutPoint>,
    edge: Option<RootEdge>,
    use_policy: RootUse,
    certificate: &TransitionCertificate,
) -> Result<Option<OutPoint>, InvariantError> {
    match use_policy {
        RootUse::Forbidden => {
            if edge.is_some() {
                return Err(InvariantError::ResvSuccession);
            }

            if let Some(current) = current
                && certificate.consumed.contains(&current)
            {
                return Err(InvariantError::ResvSuccession);
            }

            Ok(current)
        }

        RootUse::Succession => {
            let current = current.ok_or(InvariantError::ResvSuccession)?;

            let Some(RootEdge::Succ { input, output }) = edge else {
                return Err(InvariantError::ResvSuccession);
            };

            if input != current {
                return Err(InvariantError::ResvSuccession);
            }

            check_succ_edge_membership(certificate, input, output)?;

            Ok(Some(output))
        }

        RootUse::SuccessionOrTermination => {
            let current = current.ok_or(InvariantError::ResvSuccession)?;

            match edge {
                Some(RootEdge::Succ { input, output }) => {
                    if input != current {
                        return Err(InvariantError::ResvSuccession);
                    }

                    check_succ_edge_membership(certificate, input, output)?;

                    Ok(Some(output))
                }

                Some(RootEdge::Term { input }) => {
                    if input != current {
                        return Err(InvariantError::ResvSuccession);
                    }

                    check_term_edge_membership(certificate, input)?;

                    Ok(None)
                }

                None => Err(InvariantError::ResvSuccession),
            }
        }
    }
}

// ´rule:verification:full-root-history-replay´
//
// Every root edge is validated against the root active immediately
// before that transition, so an intermediate corruption is detected
// even when a later edge restores the expected final cursor.

fn replay_root_history(world: &World) -> Result<ReplayedRoots, InvariantError> {
    let mut replayed = ReplayedRoots {
        state: world.history.genesis.state_out,
        resv: Some(world.history.genesis.resv_out),
        pace: world.history.genesis.pace_out,
        entitlement_authority: world.history.genesis.entitlement_authority_out,
        distribution_authority: world.history.genesis.distribution_authority_out,
        sealed_state: None,
    };

    let mut previous_order = world.history.genesis.order;

    let mut seen_txids = BTreeSet::new();
    seen_txids.insert(world.history.genesis.txid);

    let mut seen_consumed = BTreeSet::new();
    let mut seen_created = BTreeSet::new();

    // Every genesis-created outpoint — the five roots and the two
    // genesis receipts — is already created; a later certificate
    // claiming one is rejected as a duplicate creation. Outpoints
    // allocated by external open-object injection remain outside this
    // census by design: history replays the protocol's own creations,
    // and the kernel's monotonic allocator is the authority for
    // globally fresh identities.
    for created in [
        world.history.genesis.state_out,
        world.history.genesis.resv_out,
        world.history.genesis.pace_out,
        world.history.genesis.entitlement_authority_out,
        world.history.genesis.distribution_authority_out,
        world.history.genesis.live_receipt_out,
        world.history.genesis.time_locked_receipt_out,
    ] {
        seen_created.insert(created);
    }

    for certificate in &world.history.transitions {
        if certificate.order <= previous_order {
            return Err(InvariantError::HistoryProjection);
        }

        previous_order = certificate.order;

        if !seen_txids.insert(certificate.txid) {
            return Err(InvariantError::HistoryProjection);
        }

        check_certificate_sets(certificate, &mut seen_consumed, &mut seen_created)?;

        let policy = branch_policy(certificate.branch);

        if replayed.sealed_state.is_some() && policy.state != RootUse::Forbidden {
            // Once the RESV chain terminates, the sealed STATE is a
            // tombstone. No later pool-state transition is valid.
            return Err(InvariantError::StateSuccession);
        }

        if replayed.sealed_state.is_some() && certificate.resv_edge.is_some() {
            // Redundant with the forbidden-branch check above, but it
            // documents the terminal nature of RESV termination
            // explicitly: no transition after termination may recreate
            // RESV.
            return Err(InvariantError::ResvSuccession);
        }

        let old_state = replayed.state;

        replayed.state = replay_nonterminating_root(
            replayed.state,
            certificate.state_edge,
            policy.state,
            certificate,
        )?;

        replayed.resv = replay_resv_root(
            replayed.resv,
            certificate.resv_edge,
            policy.resv,
            certificate,
        )?;

        replayed.pace = replay_nonterminating_root(
            replayed.pace,
            certificate.pace_edge,
            policy.pace,
            certificate,
        )?;

        replayed.entitlement_authority = replay_nonterminating_root(
            replayed.entitlement_authority,
            certificate.entitlement_authority_edge,
            policy.entitlement_authority,
            certificate,
        )?;

        replayed.distribution_authority = replay_nonterminating_root(
            replayed.distribution_authority,
            certificate.distribution_authority_edge,
            policy.distribution_authority,
            certificate,
        )?;

        if matches!(certificate.resv_edge, Some(RootEdge::Term { .. })) {
            let Some(RootEdge::Succ { input, output }) = certificate.state_edge else {
                return Err(InvariantError::ResvSuccession);
            };

            if input != old_state || output != replayed.state {
                return Err(InvariantError::ResvSuccession);
            }

            replayed.sealed_state = Some(output);
        }
    }

    Ok(replayed)
}

// ´rule:verification:replayed-root-cursor-match´

fn check_replayed_roots(world: &World, state: PoolState) -> Result<(), InvariantError> {
    let replayed = replay_root_history(world)?;

    if replayed.state != world.roots.state {
        return Err(InvariantError::StateSuccession);
    }

    if replayed.resv != world.roots.resv {
        return Err(InvariantError::ResvSuccession);
    }

    if replayed.pace != world.roots.pace
        || replayed.entitlement_authority != world.roots.entitlement_authority
        || replayed.distribution_authority != world.roots.distribution_authority
    {
        return Err(InvariantError::IdentityAuthority);
    }

    let sealed = state.is_sealed().map_err(|_| InvariantError::Domains)?;

    match (sealed, replayed.resv, replayed.sealed_state) {
        (false, Some(_), None) => {}

        (true, None, Some(sealed_state)) if sealed_state == replayed.state => {}

        _ => {
            return Err(InvariantError::ResvSuccession);
        }
    }

    Ok(())
}

// ´rule:verification:distribution-bijection´

fn check_distributions(fold: &AccountingFold) -> Result<(), InvariantError> {
    for (cycle, control) in &fold.controls {
        let expected = control
            .remaining_live_value
            .checked_add(control.remaining_time_locked_value)
            .map_err(|_| InvariantError::Domains)?;

        let count = fold.vault_count.get(cycle).copied().unwrap_or(0);

        let actual = fold
            .vault_value
            .get(cycle)
            .cloned()
            .unwrap_or_else(BigUint::zero);

        if expected.is_zero() {
            if count != 0 || !actual.is_zero() {
                return Err(InvariantError::DistributionPayability);
            }
        } else if count != 1 || actual != BigUint::from(expected.get()) {
            return Err(InvariantError::DistributionPayability);
        }

        let max_live = floor_mul_div(
            control.remaining_principal,
            control.live_allocation,
            control.principal,
        )
        .map_err(|_| InvariantError::Domains)?;

        let max_time_locked = floor_mul_div(
            control.remaining_principal,
            control.time_locked_allocation,
            control.principal,
        )
        .map_err(|_| InvariantError::Domains)?;

        if control.remaining_live_value < max_live
            || control.remaining_time_locked_value < max_time_locked
        {
            return Err(InvariantError::DistributionPayability);
        }
    }

    for cycle in fold.vault_count.keys() {
        if !fold.controls.contains_key(cycle) {
            return Err(InvariantError::DistributionPayability);
        }
    }

    Ok(())
}

// ´rule:verification:entitlement-lifecycle-check´

fn check_entitlement_lifecycle(
    state: PoolState,
    fold: &AccountingFold,
) -> Result<(), InvariantError> {
    let pending_target = state.next_cycle().map_err(|_| InvariantError::Domains)?;

    let pending = fold
        .entitlement_by_target
        .get(&pending_target)
        .cloned()
        .unwrap_or_else(BigUint::zero);

    if pending != BigUint::from(state.q.get()) {
        return Err(InvariantError::EntitlementLifecycle);
    }

    for (target, amount) in &fold.entitlement_by_target {
        if *target == pending_target {
            continue;
        }

        let control = fold
            .controls
            .get(target)
            .ok_or(InvariantError::EntitlementLifecycle)?;

        if *amount != BigUint::from(control.remaining_principal.get()) {
            return Err(InvariantError::EntitlementLifecycle);
        }
    }

    let mut remaining_total = BigUint::from(state.q.get());

    for (cycle, control) in &fold.controls {
        if *cycle > state.cycle {
            return Err(InvariantError::EntitlementLifecycle);
        }

        let amount = fold
            .entitlement_by_target
            .get(cycle)
            .cloned()
            .unwrap_or_else(BigUint::zero);

        if amount != BigUint::from(control.remaining_principal.get()) {
            return Err(InvariantError::EntitlementLifecycle);
        }

        remaining_total += BigUint::from(control.remaining_principal.get());
    }

    if fold.outstanding_entitlement != remaining_total {
        return Err(InvariantError::EntitlementLifecycle);
    }

    Ok(())
}

// ´rule:verification:receipt-accounting-check´

fn check_receipt_accounting(state: PoolState, fold: &AccountingFold) -> Result<(), InvariantError> {
    match state.maturity {
        Maturity::Complete => {
            if !state.y_t.is_zero() {
                return Err(InvariantError::ReceiptAccountingPostMaturity);
            }

            let total = &fold.circulating_live
                + &fold.circulating_time_locked
                + &fold.ash
                + &fold.distribution_live
                + &fold.distribution_time_locked
                + &fold.historical_live_residue
                + &fold.historical_time_locked_residue;

            if total != BigUint::from(state.y_l.get()) {
                return Err(InvariantError::ReceiptAccountingPostMaturity);
            }
        }

        _ => {
            let live = &fold.circulating_live
                + &fold.ash
                + &fold.distribution_live
                + &fold.historical_live_residue;

            let time_locked = &fold.circulating_time_locked
                + &fold.distribution_time_locked
                + &fold.historical_time_locked_residue;

            if live != BigUint::from(state.y_l.get()) {
                return Err(InvariantError::ReceiptAccountingPreMaturity);
            }

            if time_locked != BigUint::from(state.y_t.get()) {
                return Err(InvariantError::ReceiptAccountingPreMaturity);
            }
        }
    }

    Ok(())
}

// ´rule:verification:maturity-coherence´

fn check_maturity_coherence(state: PoolState) -> Result<(), InvariantError> {
    match state.maturity {
        Maturity::Unannounced => {}

        Maturity::Announced { cycle } => {
            if cycle <= state.cycle {
                return Err(InvariantError::MaturityCoherence);
            }
        }

        Maturity::Complete => {
            if !state.y_t.is_zero() {
                return Err(InvariantError::MaturityCoherence);
            }
        }
    }

    Ok(())
}

// ´def:verification:invariant-checker´

pub fn check_invariant(world: &World) -> Result<(), InvariantError> {
    check_root_exactness(world)?;

    let (_state_outpoint, state) = world
        .state()
        .map_err(|_| InvariantError::IdentityAuthority)?;

    world
        .constants
        .validate()
        .map_err(|_| InvariantError::Domains)?;

    let y = state.y().map_err(|_| InvariantError::Domains)?;

    // 𝗜₂ — domains and active-backing cap: in addition to the < 2^51
    // arithmetic domain, active backing satisfies the stricter
    // ACTIVE_BACKING_MAX economic cap. It is a perpetual cap on simultaneous
    // active reserve plus escrow, not on cumulative deposit volume.
    let active_backing =
        checked_active_backing(state.omega, state.q).map_err(|error| match error {
            Guard::ActiveBackingCapExceeded => InvariantError::ActiveBackingCap,

            _ => InvariantError::Domains,
        })?;

    if y > state.omega {
        return Err(InvariantError::Floor);
    }

    if y.is_zero() && (!state.omega.is_zero() || !state.q.is_zero()) {
        return Err(InvariantError::SealedTerminal);
    }

    match (y.is_zero(), world.roots.resv) {
        (true, None) => {}

        (false, Some(_)) => {
            let (_resv_outpoint, resv) =
                world.active_resv().map_err(|_| InvariantError::Backing)?;

            if resv.value != active_backing {
                return Err(InvariantError::Backing);
            }
        }

        _ => {
            return Err(InvariantError::Backing);
        }
    }

    check_replayed_roots(world, state)?;

    check_maturity_coherence(state)?;

    let fold = fold_accounting(world)?;

    check_distributions(&fold)?;

    check_entitlement_lifecycle(state, &fold)?;

    check_receipt_accounting(state, &fold)?;

    Ok(())
}
