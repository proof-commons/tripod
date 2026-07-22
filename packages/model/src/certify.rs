//! Transition-certificate derivation.
//!
//! Implements `(´rule:verification:root-shape´)`,
//! `(´rule:verification:created-output-selectors´)`,
//! `(´rule:verification:derive-simple-root-edge´)`,
//! `(´rule:verification:derive-state-edge´)`,
//! `(´rule:verification:derive-resv-edge´)`,
//! `(´rule:verification:derive-authority-edges´)`,
//! `(´rule:verification:resolve-output-refs´)`,
//! `(´rule:verification:collect-created-outputs´)`,
//! `(´rule:verification:derive-canonical-deltas´)`,
//! `(´rule:verification:derive-burn-projection´)`,
//! `(´rule:verification:derive-clear-projection´)`,
//! `(´rule:verification:derive-distribution-residue´)`,
//! `(´rule:verification:derive-transition-certificate´)`, and
//! `(´rule:verification:data-output-validation´)`.
//!
//! Branches construct consumed inputs and outputs. The generic kernel
//! derives certificates and event projections. Branches do not author
//! the facts used to verify themselves.

use std::collections::{BTreeMap, BTreeSet};

use crate::asset::{Asset, ReceiptClass};
use crate::guard::Guard;
use crate::history::{
    BranchKind, BurnProjection, BurnRecord, CertifiedCanonicalFlow, CertifiedCanonicalPartition,
    CertifiedDestructionLeg, CertifiedIssuance, ClearProjection, DistributionResidueProjection,
    OpenFlowProjection, RootEdge, TransitionCertificate,
};
use crate::kernel::{
    CanonicalFlow, DestructionDeclaration, IssuanceDeclaration, OpenFlow, OutputRef, PendingOutput,
    flows_to_destructions,
};
use crate::object::{DataOutput, Meta, Tag, Utxo};
use crate::policy::{RootUse, branch_policy};
use crate::recognition::read_distribution_control;
use crate::scalar::{CanonicalOrder, OutPoint, Sat, TxId};
use crate::world::World;

// ´rule:verification:derive-transition-certificate´
//
// `TxBuilder::finish()` passes `fee` into this function.

#[allow(clippy::too_many_arguments)]
pub fn derive_transition_certificate(
    before: &World,
    after: &World,
    txid: TxId,
    order: CanonicalOrder,
    branch: BranchKind,
    consumed: &BTreeMap<OutPoint, Utxo>,
    output_map: &BTreeMap<OutputRef, OutPoint>,
    data_outputs: &[DataOutput],
    issuances: &[IssuanceDeclaration],
    flows: &[CanonicalFlow],
    open_flows: &[OpenFlow],
    chain_fee: Sat,
) -> Result<TransitionCertificate, Guard> {
    let policy = branch_policy(branch);

    if order <= before.history.last_order() {
        return Err(Guard::RootSuccession);
    }

    let destructions = flows_to_destructions(flows);

    let consumed_set = consumed.keys().copied().collect::<BTreeSet<_>>();

    let created_set = output_map.values().copied().collect::<BTreeSet<_>>();

    let state_edge = derive_state_edge(before, after, consumed, &created_set, policy.state)?;

    let resv_edge = derive_resv_edge(
        before,
        after,
        consumed,
        &created_set,
        policy.resv,
        state_edge,
    )?;

    let pace_edge = derive_pace_edge(before, after, consumed, &created_set, policy.pace)?;

    let entitlement_authority_edge = derive_entitlement_authority_edge(
        before,
        after,
        consumed,
        &created_set,
        policy.entitlement_authority,
    )?;

    let distribution_authority_edge = derive_distribution_authority_edge(
        before,
        after,
        consumed,
        &created_set,
        policy.distribution_authority,
    )?;

    let created_outputs = collect_created_outputs(after, output_map)?;

    let canonical_partition =
        derive_canonical_partition(consumed, &created_outputs, output_map, issuances, flows)?;

    let open_flow_projections = derive_open_flow_projections(output_map, open_flows)?;

    let burn = derive_burn_projection(
        branch,
        consumed,
        &created_set,
        after,
        data_outputs,
        policy.allows_burn_projection,
    )?;

    let clear = derive_clear_projection(
        branch,
        before,
        after,
        state_edge,
        &destructions,
        policy.allows_clear_projection,
    )?;

    let distribution_residue = derive_distribution_residue_projection(
        branch,
        consumed,
        &created_set,
        after,
        &destructions,
        policy.allows_distribution_residue_projection,
    )?;

    validate_data_outputs(
        after,
        branch,
        data_outputs,
        burn.as_ref(),
        clear.as_ref(),
        distribution_residue.as_ref(),
    )?;

    Ok(TransitionCertificate {
        txid,
        order,
        branch,

        consumed: consumed_set,
        created: created_set,

        state_edge,
        resv_edge,
        pace_edge,
        entitlement_authority_edge,
        distribution_authority_edge,

        canonical_partition,
        open_flows: open_flow_projections,

        chain_fee,

        burn,
        clear,
        distribution_residue,
    })
}

// ´rule:verification:derive-open-flow-projections´

fn derive_open_flow_projections(
    output_map: &BTreeMap<OutputRef, OutPoint>,
    open_flows: &[OpenFlow],
) -> Result<Vec<OpenFlowProjection>, Guard> {
    open_flows
        .iter()
        .map(|flow| {
            Ok(OpenFlowProjection {
                kind: flow.kind,

                source_inputs: flow.source_inputs.clone(),

                destination_outputs: resolve_output_refs(output_map, &flow.destination_outputs)?,

                fee: flow.fee,
            })
        })
        .collect()
}

// ´rule:verification:root-shape´

fn is_state_root(utxo: &Utxo) -> bool {
    matches!(
        (utxo.asset, utxo.meta, utxo.value),
        (Asset::Pid, Meta::State(_), Sat::ONE)
    )
}

fn is_resv_shape(utxo: &Utxo) -> bool {
    matches!((utxo.asset, utxo.meta), (Asset::Lbtc, Meta::Resv))
}

fn is_pace_root(utxo: &Utxo) -> bool {
    matches!(
        (utxo.asset, utxo.meta, utxo.value),
        (Asset::Pace, Meta::Pace, Sat::ONE)
    )
}

fn is_entitlement_authority_root(utxo: &Utxo) -> bool {
    matches!(
        (utxo.asset, utxo.meta, utxo.value),
        (Asset::EntAuth, Meta::EntitlementAuthority, Sat::ONE)
    )
}

fn is_distribution_authority_root(utxo: &Utxo) -> bool {
    matches!(
        (utxo.asset, utxo.meta, utxo.value),
        (Asset::DistAuth, Meta::DistributionAuthority, Sat::ONE)
    )
}

// ´rule:verification:created-output-selectors´

fn created_outpoints_matching<F>(
    world: &World,
    created: &BTreeSet<OutPoint>,
    predicate: F,
) -> Vec<OutPoint>
where
    F: Fn(&Utxo) -> bool,
{
    created
        .iter()
        .filter_map(|outpoint| {
            world
                .utxos
                .get(outpoint)
                .filter(|utxo| predicate(utxo))
                .map(|_| *outpoint)
        })
        .collect()
}

fn require_exactly_one(values: Vec<OutPoint>) -> Result<OutPoint, Guard> {
    if values.len() == 1 {
        Ok(values[0])
    } else {
        Err(Guard::RootSuccession)
    }
}

fn require_none(values: Vec<OutPoint>) -> Result<(), Guard> {
    if values.is_empty() {
        Ok(())
    } else {
        Err(Guard::RootSuccession)
    }
}

// ´rule:verification:derive-simple-root-edge´

fn derive_simple_root_edge<F>(
    current_root: OutPoint,
    before: &World,
    after: &World,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    use_policy: RootUse,
    successor_predicate: F,
) -> Result<Option<RootEdge>, Guard>
where
    F: Fn(&Utxo) -> bool,
{
    let consumed_current = consumed.contains_key(&current_root);

    let successors = created_outpoints_matching(after, created, successor_predicate);

    match use_policy {
        RootUse::Forbidden => {
            if consumed_current {
                return Err(Guard::RootSuccession);
            }

            require_none(successors)?;
            Ok(None)
        }

        RootUse::Succession => {
            if !consumed_current {
                return Err(Guard::RootSuccession);
            }

            let output = require_exactly_one(successors)?;

            let _ = before.utxo(current_root)?;

            Ok(Some(RootEdge::Succ {
                input: current_root,
                output,
            }))
        }

        RootUse::SuccessionOrTermination => {
            if !consumed_current {
                return Err(Guard::RootSuccession);
            }

            match successors.len() {
                0 => Ok(Some(RootEdge::Term {
                    input: current_root,
                })),

                1 => Ok(Some(RootEdge::Succ {
                    input: current_root,
                    output: successors[0],
                })),

                _ => Err(Guard::RootSuccession),
            }
        }
    }
}

// ´rule:verification:derive-state-edge´
//
// The kernel derives the edge from the actual canonical PID input and
// output. Branch code cannot author it.

fn derive_state_edge(
    before: &World,
    after: &World,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    use_policy: RootUse,
) -> Result<Option<RootEdge>, Guard> {
    derive_simple_root_edge(
        before.roots.state,
        before,
        after,
        consumed,
        created,
        use_policy,
        is_state_root,
    )
}

// ´rule:verification:derive-resv-edge´
//
// A state-only branch forbids every `RESV`-shaped input, active or
// decoy.

fn derive_resv_edge(
    before: &World,
    after: &World,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    use_policy: RootUse,
    state_edge: Option<RootEdge>,
) -> Result<Option<RootEdge>, Guard> {
    let current_resv = before.roots.resv;

    match (current_resv, use_policy) {
        (None, RootUse::Forbidden) => {
            require_none(created_outpoints_matching(after, created, is_resv_shape))?;

            Ok(None)
        }

        (None, _) => Err(Guard::Sealed),

        (Some(_), RootUse::Forbidden) => {
            if consumed.values().any(is_resv_shape) {
                return Err(Guard::ResvWeld);
            }

            require_none(created_outpoints_matching(after, created, is_resv_shape))?;

            Ok(None)
        }

        (Some(resv), RootUse::Succession) => {
            if !consumed.contains_key(&resv) {
                return Err(Guard::ResvWeld);
            }

            let output =
                require_exactly_one(created_outpoints_matching(after, created, is_resv_shape))?;

            if !matches!(state_edge, Some(RootEdge::Succ { .. })) {
                return Err(Guard::ResvWeld);
            }

            Ok(Some(RootEdge::Succ {
                input: resv,
                output,
            }))
        }

        (Some(resv), RootUse::SuccessionOrTermination) => {
            if !consumed.contains_key(&resv) {
                return Err(Guard::ResvWeld);
            }

            let successors = created_outpoints_matching(after, created, is_resv_shape);

            match successors.len() {
                0 => {
                    let state_output = match state_edge {
                        Some(RootEdge::Succ { output, .. }) => output,

                        _ => {
                            return Err(Guard::ResvWeld);
                        }
                    };

                    let state_utxo = after.utxo(state_output)?;

                    let state = match state_utxo.meta {
                        Meta::State(state) => state,

                        _ => {
                            return Err(Guard::ResvWeld);
                        }
                    };

                    if !state.is_sealed()? {
                        return Err(Guard::ResvWeld);
                    }

                    Ok(Some(RootEdge::Term { input: resv }))
                }

                1 => Ok(Some(RootEdge::Succ {
                    input: resv,
                    output: successors[0],
                })),

                _ => Err(Guard::ResvWeld),
            }
        }
    }
}

// ´rule:verification:derive-authority-edges´

fn derive_pace_edge(
    before: &World,
    after: &World,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    use_policy: RootUse,
) -> Result<Option<RootEdge>, Guard> {
    derive_simple_root_edge(
        before.roots.pace,
        before,
        after,
        consumed,
        created,
        use_policy,
        is_pace_root,
    )
}

fn derive_entitlement_authority_edge(
    before: &World,
    after: &World,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    use_policy: RootUse,
) -> Result<Option<RootEdge>, Guard> {
    derive_simple_root_edge(
        before.roots.entitlement_authority,
        before,
        after,
        consumed,
        created,
        use_policy,
        is_entitlement_authority_root,
    )
}

fn derive_distribution_authority_edge(
    before: &World,
    after: &World,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    use_policy: RootUse,
) -> Result<Option<RootEdge>, Guard> {
    derive_simple_root_edge(
        before.roots.distribution_authority,
        before,
        after,
        consumed,
        created,
        use_policy,
        is_distribution_authority_root,
    )
}

// ´rule:verification:resolve-output-refs´

fn resolve_output_refs(
    output_map: &BTreeMap<OutputRef, OutPoint>,
    references: &[OutputRef],
) -> Result<Vec<OutPoint>, Guard> {
    references
        .iter()
        .map(|reference| {
            output_map
                .get(reference)
                .copied()
                .ok_or(Guard::MissingOutputIndex)
        })
        .collect()
}

// ´rule:verification:collect-created-outputs´

fn collect_created_outputs(
    after: &World,
    output_map: &BTreeMap<OutputRef, OutPoint>,
) -> Result<Vec<PendingOutput>, Guard> {
    let mut by_index = output_map
        .iter()
        .map(|(output_ref, outpoint)| Ok((output_ref.0, after.utxo(*outpoint)?)))
        .collect::<Result<Vec<_>, Guard>>()?;

    by_index.sort_by_key(|(index, _)| *index);

    for (expected, (actual, _)) in by_index.iter().enumerate() {
        if expected != *actual {
            return Err(Guard::MissingOutputIndex);
        }
    }

    Ok(by_index
        .into_iter()
        .map(|(_, utxo)| PendingOutput {
            asset: utxo.asset,
            value: utxo.value,
            meta: utxo.meta,
        })
        .collect())
}

// ´rule:verification:derive-canonical-deltas´
//
// Derives the exact canonical partition (issuances + flows), preserving
// the kernel's grouping instead of flattening each flow into separate
// delta rows. Each flow stores its summed source/destination amounts as
// proof metadata; the per-flow equation and the movement-kind rule are
// re-checked here as defense in depth over the kernel's conservation.

fn derive_canonical_partition(
    consumed: &BTreeMap<OutPoint, Utxo>,
    outputs: &[PendingOutput],
    output_map: &BTreeMap<OutputRef, OutPoint>,
    issuances: &[IssuanceDeclaration],
    flows: &[CanonicalFlow],
) -> Result<CertifiedCanonicalPartition, Guard> {
    let mut certified_issuances = Vec::new();

    for issuance in issuances {
        let mut destination_total = Sat::ZERO;

        for destination in &issuance.destination_outputs {
            let output = outputs
                .get(destination.0)
                .ok_or(Guard::MissingOutputIndex)?;

            destination_total = destination_total.checked_add(output.value)?;
        }

        if destination_total != issuance.amount {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        certified_issuances.push(CertifiedIssuance {
            asset: issuance.asset,
            authority_asset: issuance.authority_asset,
            authority_input: issuance.authority_input,
            amount: issuance.amount,
            destination_outputs: resolve_output_refs(output_map, &issuance.destination_outputs)?,
        });
    }

    let mut certified_flows = Vec::new();

    for flow in flows {
        let mut destination_total = Sat::ZERO;

        for destination in &flow.destination_outputs {
            let output = outputs
                .get(destination.0)
                .ok_or(Guard::MissingOutputIndex)?;

            destination_total = destination_total.checked_add(output.value)?;
        }

        // Movement kind present iff the destination amount is positive.
        if destination_total.is_zero() == flow.movement_kind.is_some() {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        let mut destruction_total = Sat::ZERO;
        let mut destructions = Vec::new();

        for leg in &flow.destructions {
            if leg.amount.is_zero() {
                return Err(Guard::CanonicalDeltaMismatch);
            }

            destruction_total = destruction_total.checked_add(leg.amount)?;

            destructions.push(CertifiedDestructionLeg {
                tag: leg.tag,
                amount: leg.amount,
            });
        }

        // Sum the actual consumed source object values and require the
        // exact per-flow equation source = destination + destruction.
        let mut source_total = Sat::ZERO;

        for source in &flow.source_inputs {
            let utxo = consumed.get(source).ok_or(Guard::MissingOutputIndex)?;

            source_total = source_total.checked_add(utxo.value)?;
        }

        if source_total != destination_total.checked_add(destruction_total)? {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        certified_flows.push(CertifiedCanonicalFlow {
            asset: flow.asset,
            source_inputs: flow.source_inputs.clone(),
            destination_outputs: resolve_output_refs(output_map, &flow.destination_outputs)?,
            source_amount: source_total,
            destination_amount: destination_total,
            destructions,
            movement_kind: flow.movement_kind,
        });
    }

    Ok(CertifiedCanonicalPartition {
        issuances: certified_issuances,
        flows: certified_flows,
    })
}

// ´rule:verification:derive-burn-records´
//
// Burn records are derived from the actual staged unspendable outputs.
// There is no parallel branch-authored record vector.

fn derive_burn_records(data_outputs: &[DataOutput]) -> Result<Vec<BurnRecord>, Guard> {
    let mut records = Vec::new();

    for output in data_outputs {
        let DataOutput::BurnRecord {
            record_index,
            address,
            amount,
        } = output
        else {
            continue;
        };

        if amount.is_zero() {
            return Err(Guard::Domain);
        }

        let expected_index = u32::try_from(records.len()).map_err(|_| Guard::Overflow)?;

        if *record_index != expected_index {
            return Err(Guard::WrongShape);
        }

        records.push(BurnRecord {
            record_index: *record_index,
            address: *address,
            amount: *amount,
        });
    }

    // The burn-record maximum is enforced by the manifest-driven
    // data-output cardinality check in `validate_data_outputs`.

    Ok(records)
}

// ´rule:verification:derive-burn-projection´
//
// Output-integrity signature verification remains outside the oracle's
// byte-level model and is represented by the signer-set abstraction
// plus the compiler/deployment sighash profile.

fn derive_burn_projection(
    branch: BranchKind,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    after: &World,
    data_outputs: &[DataOutput],
    allowed: bool,
) -> Result<Option<BurnProjection>, Guard> {
    let records = derive_burn_records(data_outputs)?;

    if branch != BranchKind::Burn {
        if !records.is_empty() {
            return Err(Guard::BadAuthorization);
        }

        return Ok(None);
    }

    if !allowed {
        return Err(Guard::BadAuthorization);
    }

    let canonical_u_inputs = consumed
        .iter()
        .filter(|(_, utxo)| utxo.asset == Asset::U)
        .collect::<Vec<_>>();

    if canonical_u_inputs.is_empty() {
        return Err(Guard::WrongShape);
    }

    for (_outpoint, utxo) in canonical_u_inputs {
        match utxo.meta {
            Meta::Receipt {
                class: ReceiptClass::Live,
                ..
            } => {}

            _ => {
                return Err(Guard::WrongClass);
            }
        }
    }

    // The redundant ASH-input check remains for clarity even though
    // the canonical-input loop rejects non-live receipt inputs. It
    // explicitly documents the event-type anchor.
    if consumed
        .values()
        .any(|utxo| utxo.asset == Asset::U && utxo.meta == Meta::Ash)
    {
        return Err(Guard::WrongShape);
    }

    let ash_outputs = created_outpoints_matching(after, created, |utxo| {
        utxo.asset == Asset::U && utxo.meta == Meta::Ash
    });

    let ash_output = require_exactly_one(ash_outputs)?;

    for outpoint in created {
        let utxo = after.utxo(*outpoint)?;

        if utxo.asset != Asset::U {
            continue;
        }

        match utxo.meta {
            Meta::Ash => {}

            Meta::Receipt {
                class: ReceiptClass::Live,
                ..
            } => {}

            _ => {
                return Err(Guard::ClassCross);
            }
        }
    }

    let ash_value = after.utxo(ash_output)?.value;

    Ok(Some(BurnProjection {
        ash_output,
        ash_value,
        records,
    }))
}

// ´rule:verification:derive-clear-projection´

fn derive_clear_projection(
    branch: BranchKind,
    before: &World,
    after: &World,
    state_edge: Option<RootEdge>,
    destructions: &[DestructionDeclaration],
    allowed: bool,
) -> Result<Option<ClearProjection>, Guard> {
    if branch != BranchKind::Clear {
        return Ok(None);
    }

    if !allowed {
        return Err(Guard::BadAuthorization);
    }

    let (state_input, state_output) = match state_edge {
        Some(RootEdge::Succ { input, output }) => (input, output),

        _ => {
            return Err(Guard::RootSuccession);
        }
    };

    let before_state = match before.utxo(state_input)?.meta {
        Meta::State(state) => state,
        _ => {
            return Err(Guard::WrongShape);
        }
    };

    let after_state = match after.utxo(state_output)?.meta {
        Meta::State(state) => state,
        _ => {
            return Err(Guard::WrongShape);
        }
    };

    let recon = destructions
        .iter()
        .filter(|destruction| destruction.asset == Asset::U && destruction.tag == Tag::Recon)
        .collect::<Vec<_>>();

    if recon.len() != 1 {
        return Err(Guard::BadDestruction);
    }

    let amount = recon[0].amount;

    if before_state.omega != after_state.omega
        || before_state.q != after_state.q
        || before_state.y_t != after_state.y_t
        || before_state.cycle != after_state.cycle
        || before_state.maturity != after_state.maturity
        || before_state.y_l.checked_sub(amount)? != after_state.y_l
    {
        return Err(Guard::ValuePin);
    }

    let y = after_state.y()?;

    Ok(Some(ClearProjection {
        omega: after_state.omega,
        y,
    }))
}

// ´rule:verification:derive-distribution-residue´

fn derive_distribution_residue_projection(
    branch: BranchKind,
    consumed: &BTreeMap<OutPoint, Utxo>,
    created: &BTreeSet<OutPoint>,
    after: &World,
    destructions: &[DestructionDeclaration],
    allowed: bool,
) -> Result<Option<DistributionResidueProjection>, Guard> {
    if branch != BranchKind::SettleDistribution {
        return Ok(None);
    }

    if !allowed {
        return Err(Guard::BadAuthorization);
    }

    let control_inputs = consumed
        .iter()
        .filter_map(|(outpoint, utxo)| {
            if utxo.asset == Asset::DistCtl {
                Some((*outpoint, *utxo))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if control_inputs.len() != 1 {
        return Err(Guard::WrongShape);
    }

    let (control_input, control_utxo) = control_inputs[0];

    let control = read_distribution_control(&control_utxo)?;

    let has_successor = created
        .iter()
        .filter_map(|outpoint| after.utxos.get(outpoint))
        .any(|utxo| {
            matches!(
                utxo.meta,
                Meta::DistributionControl {
                    cycle,
                    ..
                } if cycle == control.cycle
            )
        });

    if has_successor {
        return Ok(None);
    }

    let control_close = destructions
        .iter()
        .filter(|destruction| {
            destruction.asset == Asset::DistCtl && destruction.tag == Tag::DistributionControlClose
        })
        .collect::<Vec<_>>();

    if control_close.len() != 1
        || control_close[0].amount != Sat::ONE
        || control_close[0].source_inputs != vec![control_input]
    {
        return Err(Guard::BadDestruction);
    }

    let entitlement_destroyed = destructions
        .iter()
        .filter(|destruction| {
            destruction.asset == Asset::Ent && destruction.tag == Tag::Entitlement
        })
        .map(|destruction| destruction.amount)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let remaining_principal = control
        .remaining_principal
        .checked_sub(entitlement_destroyed)?;

    if !remaining_principal.is_zero() {
        return Err(Guard::OverDraw);
    }

    let live_receipt_outputs = created
        .iter()
        .filter_map(|outpoint| after.utxos.get(outpoint))
        .filter_map(|utxo| match (utxo.asset, utxo.meta) {
            (
                Asset::U,
                Meta::Receipt {
                    class: ReceiptClass::Live,
                    ..
                },
            ) => Some(utxo.value),

            _ => None,
        })
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let time_locked_receipt_outputs = created
        .iter()
        .filter_map(|outpoint| after.utxos.get(outpoint))
        .filter_map(|utxo| match (utxo.asset, utxo.meta) {
            (
                Asset::U,
                Meta::Receipt {
                    class: ReceiptClass::TimeLocked,
                    ..
                },
            ) => Some(utxo.value),

            _ => None,
        })
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let live_residue = control
        .remaining_live_value
        .checked_sub(live_receipt_outputs)?;

    let time_locked_residue = control
        .remaining_time_locked_value
        .checked_sub(time_locked_receipt_outputs)?;

    let residue_destroyed = destructions
        .iter()
        .filter(|destruction| {
            destruction.asset == Asset::U && destruction.tag == Tag::DistributionResidue
        })
        .map(|destruction| destruction.amount)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    if residue_destroyed != live_residue.checked_add(time_locked_residue)? {
        return Err(Guard::BadDestruction);
    }

    let vault_input = consumed
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (Asset::U, Meta::DistributionVault { cycle }) if cycle == control.cycle => {
                Some(*outpoint)
            }

            _ => None,
        });

    Ok(Some(DistributionResidueProjection {
        cycle: control.cycle,
        control_input,
        vault_input,
        live_residue,
        time_locked_residue,
    }))
}

// ´rule:verification:data-output-validation´
//
// The reference indexer does not authenticate event types merely from
// these tags. Tags are consistency aids; provenance comes from the
// derived certificate.
//
// One may assume naively that per-family counts must be hand-coded per
// branch; instead each declared data-output family's minimum and
// maximum come from the typed manifest, evaluated against the
// transition's actual activation conditions. An inactive family must
// be empty; an active family must satisfy its declared cardinality.

/// Whether a declared data-output family is active for this
/// transition. Only the conditions the manifest attaches to
/// data-output families are evaluated here.
fn data_output_condition_active(
    condition: architecture::DeltaCondition,
    residue: Option<&DistributionResidueProjection>,
) -> Result<bool, Guard> {
    match condition {
        architecture::DeltaCondition::Always => Ok(true),

        architecture::DeltaCondition::DistributionTerminates => Ok(residue.is_some()),

        architecture::DeltaCondition::PositiveDistributionResidue => match residue {
            Some(residue) => Ok(!residue
                .live_residue
                .checked_add(residue.time_locked_residue)?
                .is_zero()),

            None => Ok(false),
        },

        _ => Err(Guard::BadDestruction),
    }
}

fn validate_data_outputs(
    after: &World,
    branch: BranchKind,
    data_outputs: &[DataOutput],
    burn: Option<&BurnProjection>,
    clear: Option<&ClearProjection>,
    residue: Option<&DistributionResidueProjection>,
) -> Result<(), Guard> {
    let spec = crate::manifest::operation_spec(branch);

    for declared in spec.data_outputs {
        let count = data_outputs
            .iter()
            .filter(|output| match output {
                DataOutput::BurnRecord { .. } => {
                    declared.kind == architecture::DataOutputKind::BurnRecord
                }

                DataOutput::Destruction { tag, asset, .. } => {
                    declared.kind == architecture::DataOutputKind::Destruction
                        && crate::manifest::tag_of(declared.tag) == *tag
                        && declared.asset.map(crate::manifest::asset_of) == Some(*asset)
                }
            })
            .count();

        let maximum = match declared.maximum {
            architecture::MaxCount::Exact(value) => usize::from(value),
            architecture::MaxCount::Bound(bound) => {
                crate::manifest::bound_value(&after.constants, bound)
            }
        };

        if count > maximum {
            return Err(Guard::Domain);
        }

        if data_output_condition_active(declared.condition, residue)? {
            if count < usize::from(declared.minimum) {
                return Err(Guard::BadDestruction);
            }
        } else if count != 0 {
            return Err(Guard::BadDestruction);
        }
    }

    // Projection correspondence: family membership is enforced
    // precommit, and family cardinality above; the payloads staged
    // here must additionally agree with the derived events.
    let has_burn_records = data_outputs
        .iter()
        .any(|output| matches!(output, DataOutput::BurnRecord { .. }));

    if branch == BranchKind::Burn {
        burn.ok_or(Guard::BadAuthorization)?;
    } else if has_burn_records {
        // Normative non-burn branches emit no burn-record payloads.
        // Arbitrary external tag-shaped junk remains possible elsewhere
        // in the global environment but is not part of this transition.
        return Err(Guard::BadAuthorization);
    }

    let has_recon = data_outputs.iter().any(|output| {
        matches!(
            output,
            DataOutput::Destruction {
                tag: Tag::Recon,
                ..
            }
        )
    });

    if has_recon != clear.is_some() {
        return Err(Guard::BadDestruction);
    }

    Ok(())
}
