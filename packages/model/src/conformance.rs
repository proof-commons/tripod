//! Projection of independently executed model transitions into realization observations.
//!
//! This module never participates in model transition acceptance. Operation
//! constructors and the kernel execute first; only after a successful transition
//! does this module project primitive facts for realization conformance checks.

use std::collections::{BTreeMap, BTreeSet};

use realization::{
    Count, ObservedAsset, ObservedCanonicalDelta, ObservedObject, ObservedObjectKind,
    ObservedObjectRef, ObservedOpenFlow, ObservedRootEffect, ObservedSide, OperationObservation,
    OwnerId, ProtocolAmount, RepresentationMode,
};

use crate::{
    Asset, BranchKind, CompactAsh, Meta, OutPoint, ReceiptClass, RootEdge, Sat, SignerSet, Tag,
    TransferReceipts, TransitionCertificate, Utxo, World,
};

/// Failure while projecting a model transition into realization facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConformanceProjectionError {
    NoTransition,
    WrongBranch {
        expected: BranchKind,
        actual: BranchKind,
    },
    MissingConsumedObject(u64),
    MissingCreatedObject(u64),
    UnknownFlowSource(u64),
    UnknownFlowDestination(u64),
    UnknownDeltaSource(u64),
    UnknownDeltaDestination(u64),
    UndeclaredCanonicalAsset,
    UnexpectedIssuanceAuthority,
    HistoryLengthOverflow,
    NotOneTransitionExtension,
    HistoryPrefixChanged,
    NonIncreasingOrder,
    ConsumedCreatedOverlap,
    ConsumedObjectSurvived(u64),
    CreatedObjectAlreadyExisted(u64),
    InvalidObservation(realization::RealizationError),
    AmountOutOfDomain,
    BoundOutOfDomain,
}

pub fn observe_compact_ash(
    before: &World,
    request: &CompactAsh,
    after: &World,
) -> Result<OperationObservation, ConformanceProjectionError> {
    observe_transition(
        before,
        after,
        BranchKind::CompactAsh,
        BTreeSet::new(),
        owner_ids(&request.fee_envelope.signers),
        RepresentationMode::Explicit,
    )
}

pub fn observe_live_transfer(
    before: &World,
    request: &TransferReceipts,
    after: &World,
) -> Result<OperationObservation, ConformanceProjectionError> {
    if request.class != ReceiptClass::Live {
        return Err(ConformanceProjectionError::WrongBranch {
            expected: BranchKind::TransferLive,
            actual: BranchKind::TransferTimeLocked,
        });
    }

    observe_transition(
        before,
        after,
        BranchKind::TransferLive,
        owner_ids(&request.signers),
        owner_ids(&request.fee_envelope.signers),
        RepresentationMode::Explicit,
    )
}

fn observe_transition(
    before: &World,
    after: &World,
    expected_branch: BranchKind,
    protocol_signers: BTreeSet<OwnerId>,
    sponsor_signers: BTreeSet<OwnerId>,
    representation: RepresentationMode,
) -> Result<OperationObservation, ConformanceProjectionError> {
    let certificate = appended_certificate(before, after)?;
    validate_certificate_membership(before, after, certificate)?;

    if certificate.branch != expected_branch {
        return Err(ConformanceProjectionError::WrongBranch {
            expected: expected_branch,
            actual: certificate.branch,
        });
    }

    let mut objects = Vec::new();
    let mut reference_by_outpoint = BTreeMap::new();

    for (ordinal, outpoint) in certificate.consumed.iter().enumerate() {
        let ordinal =
            u32::try_from(ordinal).map_err(|_| ConformanceProjectionError::BoundOutOfDomain)?;
        let reference = ObservedObjectRef {
            side: ObservedSide::Input,
            ordinal,
        };
        let utxo = before
            .utxos
            .get(outpoint)
            .ok_or(ConformanceProjectionError::MissingConsumedObject(*outpoint))?;

        objects.push(observe_utxo(reference, utxo, representation)?);
        reference_by_outpoint.insert(*outpoint, reference);
    }

    for (ordinal, outpoint) in certificate.created.iter().enumerate() {
        let ordinal =
            u32::try_from(ordinal).map_err(|_| ConformanceProjectionError::BoundOutOfDomain)?;
        let reference = ObservedObjectRef {
            side: ObservedSide::Output,
            ordinal,
        };
        let utxo = after
            .utxos
            .get(outpoint)
            .ok_or(ConformanceProjectionError::MissingCreatedObject(*outpoint))?;

        objects.push(observe_utxo(reference, utxo, representation)?);
        reference_by_outpoint.insert(*outpoint, reference);
    }

    let open_flows = certificate
        .open_flows
        .iter()
        .map(|flow| {
            let sources = flow
                .source_inputs
                .iter()
                .map(|outpoint| {
                    reference_by_outpoint
                        .get(outpoint)
                        .copied()
                        .ok_or(ConformanceProjectionError::UnknownFlowSource(*outpoint))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let destinations = flow
                .destination_outputs
                .iter()
                .map(|outpoint| {
                    reference_by_outpoint.get(outpoint).copied().ok_or(
                        ConformanceProjectionError::UnknownFlowDestination(*outpoint),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(ObservedOpenFlow {
                kind: architecture_open_flow(flow.kind),
                sources,
                destinations,
                fee: observed_amount(flow.fee)?,
            })
        })
        .collect::<Result<Vec<_>, ConformanceProjectionError>>()?;

    let observation = OperationObservation {
        operation: crate::manifest::branch_operation(certificate.branch),
        objects,
        protocol_signers,
        sponsor_signers,
        canonical_deltas: observe_canonical_deltas(certificate, &reference_by_outpoint)?,
        open_flows,
        root_effects: observe_root_effects(certificate),
        projections: observe_projections(certificate),
        bounds: observe_bounds(&before.constants)?,
    };

    realization::validate_observation(observation)
        .map_err(ConformanceProjectionError::InvalidObservation)
}

fn appended_certificate<'a>(
    before: &World,
    after: &'a World,
) -> Result<&'a TransitionCertificate, ConformanceProjectionError> {
    let expected_length = before
        .history
        .transitions
        .len()
        .checked_add(1)
        .ok_or(ConformanceProjectionError::HistoryLengthOverflow)?;

    if after.history.transitions.len() != expected_length {
        return Err(ConformanceProjectionError::NotOneTransitionExtension);
    }

    if !after
        .history
        .transitions
        .starts_with(&before.history.transitions)
    {
        return Err(ConformanceProjectionError::HistoryPrefixChanged);
    }

    after
        .history
        .transitions
        .last()
        .ok_or(ConformanceProjectionError::NoTransition)
}

fn validate_certificate_membership(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), ConformanceProjectionError> {
    if certificate.order <= before.history.last_order() {
        return Err(ConformanceProjectionError::NonIncreasingOrder);
    }

    if !certificate.consumed.is_disjoint(&certificate.created) {
        return Err(ConformanceProjectionError::ConsumedCreatedOverlap);
    }

    for outpoint in &certificate.consumed {
        if !before.utxos.contains_key(outpoint) {
            return Err(ConformanceProjectionError::MissingConsumedObject(*outpoint));
        }

        if after.utxos.contains_key(outpoint) {
            return Err(ConformanceProjectionError::ConsumedObjectSurvived(
                *outpoint,
            ));
        }
    }

    for outpoint in &certificate.created {
        if before.utxos.contains_key(outpoint) {
            return Err(ConformanceProjectionError::CreatedObjectAlreadyExisted(
                *outpoint,
            ));
        }

        if !after.utxos.contains_key(outpoint) {
            return Err(ConformanceProjectionError::MissingCreatedObject(*outpoint));
        }
    }

    Ok(())
}

fn observe_utxo(
    reference: ObservedObjectRef,
    utxo: &Utxo,
    representation: RepresentationMode,
) -> Result<ObservedObject, ConformanceProjectionError> {
    Ok(ObservedObject {
        reference,
        kind: observed_object_kind(utxo),
        asset: observed_asset(utxo.asset),
        value: observed_amount(utxo.value)?,
        owner: observed_owner(utxo),
        representation,
    })
}

fn observed_asset(asset: Asset) -> ObservedAsset {
    match crate::manifest::declared_asset(asset) {
        Some(asset) => ObservedAsset::Declared(asset),
        None => match asset {
            Asset::Foreign(id) => ObservedAsset::Foreign(id),
            _ => unreachable!("every non-foreign model asset has an architecture ID"),
        },
    }
}

fn observed_object_kind(utxo: &Utxo) -> ObservedObjectKind {
    use architecture::ObjectId;

    let kind = match (utxo.asset, utxo.meta) {
        (Asset::Pid, Meta::State(_)) => ObjectId::State,
        (Asset::Lbtc, Meta::Resv) => ObjectId::Resv,
        (Asset::Pace, Meta::Pace) => ObjectId::Pace,
        (Asset::EntAuth, Meta::EntitlementAuthority) => ObjectId::EntitlementAuthority,
        (Asset::DistAuth, Meta::DistributionAuthority) => ObjectId::DistributionAuthority,
        (
            Asset::U,
            Meta::Receipt {
                class: ReceiptClass::Live,
                ..
            },
        ) => ObjectId::ReceiptLive,
        (
            Asset::U,
            Meta::Receipt {
                class: ReceiptClass::TimeLocked,
                ..
            },
        ) => ObjectId::ReceiptTimeLocked,
        (Asset::Lbtc, Meta::DepositRequest { .. }) => ObjectId::DepositRequest,
        (Asset::Ent, Meta::DepositEntitlement { .. }) => ObjectId::DepositEntitlement,
        (Asset::DistCtl, Meta::DistributionControl { .. }) => ObjectId::DistributionControl,
        (Asset::U, Meta::DistributionVault { .. }) => ObjectId::DistributionVault,
        (Asset::U, Meta::Ash) => ObjectId::Ash,
        (Asset::Lbtc, Meta::PlainLbtc { .. }) => ObjectId::PlainLbtc,
        (Asset::Lbtc, Meta::CpfpAnchor) => ObjectId::CpfpAnchor,
        _ => return ObservedObjectKind::Unrecognized,
    };

    ObservedObjectKind::Declared(kind)
}

fn observed_owner(utxo: &Utxo) -> Option<OwnerId> {
    match utxo.meta {
        Meta::Receipt { owner, .. }
        | Meta::DepositEntitlement { owner, .. }
        | Meta::PlainLbtc { owner } => Some(OwnerId(owner.0)),
        Meta::DepositRequest { refund_key, .. } => Some(OwnerId(refund_key.0)),
        _ => None,
    }
}

fn observed_amount(amount: Sat) -> Result<ProtocolAmount, ConformanceProjectionError> {
    ProtocolAmount::new(amount.get()).map_err(|_| ConformanceProjectionError::AmountOutOfDomain)
}

fn observe_canonical_deltas(
    certificate: &TransitionCertificate,
    reference_by_outpoint: &BTreeMap<OutPoint, ObservedObjectRef>,
) -> Result<Vec<ObservedCanonicalDelta>, ConformanceProjectionError> {
    certificate
        .canonical_deltas
        .iter()
        .map(|delta| {
            if delta.authority_input.is_some() {
                return Err(ConformanceProjectionError::UnexpectedIssuanceAuthority);
            }

            let asset = crate::manifest::declared_asset(delta.asset)
                .ok_or(ConformanceProjectionError::UndeclaredCanonicalAsset)?;
            let sources = delta
                .source_inputs
                .iter()
                .map(|outpoint| {
                    reference_by_outpoint
                        .get(outpoint)
                        .copied()
                        .ok_or(ConformanceProjectionError::UnknownDeltaSource(*outpoint))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let destinations = delta
                .destination_outputs
                .iter()
                .map(|outpoint| {
                    reference_by_outpoint.get(outpoint).copied().ok_or(
                        ConformanceProjectionError::UnknownDeltaDestination(*outpoint),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(ObservedCanonicalDelta {
                asset,
                kind: architecture_delta_kind(delta.kind),
                amount: observed_amount(delta.amount)?,
                sources,
                destinations,
                destruction_tag: delta.destruction_tag.map(architecture_tag),
            })
        })
        .collect()
}

fn architecture_delta_kind(kind: crate::DeltaKind) -> architecture::DeltaKind {
    match kind {
        crate::DeltaKind::Issuance => architecture::DeltaKind::Issuance,
        crate::DeltaKind::Destruction => architecture::DeltaKind::Destruction,
        crate::DeltaKind::Lateral => architecture::DeltaKind::Lateral,
        crate::DeltaKind::OwnerlessLateral => architecture::DeltaKind::OwnerlessLateral,
    }
}

fn architecture_tag(tag: Tag) -> architecture::TagId {
    match tag {
        Tag::Burn => architecture::TagId::Burn,
        Tag::Recon => architecture::TagId::Recon,
        Tag::Redeem => architecture::TagId::Redeem,
        Tag::Entitlement => architecture::TagId::Entitlement,
        Tag::DistributionControlClose => architecture::TagId::DistributionControlClose,
        Tag::DistributionResidue => architecture::TagId::DistributionResidue,
    }
}

fn observe_root_effects(certificate: &TransitionCertificate) -> Vec<ObservedRootEffect> {
    let rows = [
        (architecture::RootId::State, certificate.state_edge),
        (architecture::RootId::Resv, certificate.resv_edge),
        (architecture::RootId::Pace, certificate.pace_edge),
        (
            architecture::RootId::EntAuth,
            certificate.entitlement_authority_edge,
        ),
        (
            architecture::RootId::DistAuth,
            certificate.distribution_authority_edge,
        ),
    ];
    let mut effects = Vec::new();

    for (root, edge) in rows {
        let Some(edge) = edge else {
            continue;
        };
        let use_kind = match edge {
            RootEdge::Succ { .. } => architecture::RootUse::Succession,
            RootEdge::Term { .. } => architecture::RootUse::SuccessionOrTermination,
        };

        effects.push(ObservedRootEffect { root, use_kind });
    }

    effects.sort_by_key(|effect| effect.root.code());
    effects
}

fn observe_projections(
    certificate: &TransitionCertificate,
) -> BTreeSet<architecture::ProjectionId> {
    let mut projections = BTreeSet::from([architecture::ProjectionId::TransitionCertificate]);

    if certificate.burn.is_some() {
        projections.insert(architecture::ProjectionId::BurnEvent);
    }

    if certificate.clear.is_some() {
        projections.insert(architecture::ProjectionId::ClearEvent);
    }

    if certificate.distribution_residue.is_some() {
        projections.insert(architecture::ProjectionId::DistributionResidue);
    }

    projections
}

fn observe_bounds(
    constants: &crate::Constants,
) -> Result<BTreeMap<architecture::BoundId, Count>, ConformanceProjectionError> {
    architecture::BoundId::ALL
        .iter()
        .map(|bound| {
            let value = crate::manifest::bound_value(constants, *bound);
            let value =
                u64::try_from(value).map_err(|_| ConformanceProjectionError::BoundOutOfDomain)?;

            Ok((*bound, Count::new(value)))
        })
        .collect()
}

fn architecture_open_flow(kind: crate::OpenFlowKind) -> architecture::OpenFlowKind {
    match kind {
        crate::OpenFlowKind::RequestCreation => architecture::OpenFlowKind::RequestCreation,
        crate::OpenFlowKind::RequestRefund => architecture::OpenFlowKind::RequestRefund,
        crate::OpenFlowKind::DepositAdmission => architecture::OpenFlowKind::DepositAdmission,
        crate::OpenFlowKind::ReserveCarry => architecture::OpenFlowKind::ReserveCarry,
        crate::OpenFlowKind::Redemption => architecture::OpenFlowKind::Redemption,
        crate::OpenFlowKind::FeeSponsor => architecture::OpenFlowKind::FeeSponsor,
    }
}

fn owner_ids(signers: &SignerSet) -> BTreeSet<OwnerId> {
    signers.iter().map(|owner| OwnerId(owner.0)).collect()
}
