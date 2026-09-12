//! Projection of independently executed model transitions into realization observations.
//!
//! This module never participates in model transition acceptance. Operation
//! constructors and the kernel execute first; only after a successful transition
//! does this module project primitive facts for realization conformance checks.

use std::collections::{BTreeMap, BTreeSet};

use realization::{
    Count, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition,
    ObservedDestructionLeg, ObservedIssuance, ObservedObject, ObservedObjectKind,
    ObservedObjectRef, ObservedOpenFlow, ObservedRootEffect, ObservedRootEffectKind, ObservedSide,
    ObservedValue, OperationObservation, OwnerId, ProtocolAmount, RepresentationMode,
};

use crate::{
    AnnounceMaturity, Asset, BranchKind, CompactAsh, ExecutedTransition, Meta, OutPoint,
    ReceiptClass, RootEdge, Sat, SignerSet, Tag, TransferReceipts, TransitionCertificate, Utxo, World,
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
    SponsorProtocolRegionOverlap(u64),
}

/// A realization observation plus the external premises the model side
/// establishes for it.
///
/// Producible only by the conformance adapters from a bound execution
/// ([`ExecutedTransition`]), whose invariant-wrapped replay ran the
/// model kernel's exact conservation checks on the exact transition
/// being observed. The established set is therefore **model-side
/// evidence only** — never target, deployment, or independent
/// evidence; a target discharges its copy of the premise separately
/// through the later evidence envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelConformanceObservation {
    observation: OperationObservation,
    established_evidence: BTreeSet<realization::ExternalEvidenceRequirement>,
}

impl ModelConformanceObservation {
    pub fn observation(&self) -> &OperationObservation {
        &self.observation
    }

    pub fn established_evidence(
        &self,
    ) -> impl Iterator<Item = &realization::ExternalEvidenceRequirement> {
        self.established_evidence.iter()
    }
}

/// External-evidence requirements of `report` that `observation`'s
/// model-side evidence does not discharge.
///
/// The model pilot harnesses require this to be empty; each entry left
/// is a premise only target evidence can close.
pub fn unresolved_model_evidence(
    report: &realization::ConformanceReport,
    observation: &ModelConformanceObservation,
) -> BTreeSet<realization::ExternalEvidenceRequirement> {
    report
        .required_external_evidence()
        .filter(|requirement| !observation.established_evidence.contains(requirement))
        .cloned()
        .collect()
}

/// Observe a bound announcement after the model has checked operator authorization.
pub fn observe_announce_maturity(
    executed: &ExecutedTransition<AnnounceMaturity>,
) -> Result<ModelConformanceObservation, ConformanceProjectionError> {
    let request = executed.request();
    let observation = observe_transition(
        executed.before(),
        executed.after(),
        BranchKind::AnnounceMaturity,
        owner_ids(&request.signers),
        owner_ids(&request.fee_envelope.signers),
        RepresentationMode::Explicit,
    )?;
    let mut observation = bind_model_evidence(observation);

    // The bound request passed the model's operator-key require_signer check
    // on this exact transition; realization did not establish this premise.
    observation.established_evidence.insert(
        realization::ExternalEvidenceRequirement::OperatorAuthorization {
            operation: observation.observation.operation,
        },
    );

    Ok(observation)
}

pub fn observe_compact_ash(
    execution: &ExecutedTransition<CompactAsh>,
) -> Result<ModelConformanceObservation, ConformanceProjectionError> {
    let observation = observe_transition(
        execution.before(),
        execution.after(),
        BranchKind::CompactAsh,
        BTreeSet::new(),
        owner_ids(&execution.request().fee_envelope.signers),
        RepresentationMode::Explicit,
    )?;

    Ok(bind_model_evidence(observation))
}

pub fn observe_live_transfer(
    execution: &ExecutedTransition<TransferReceipts>,
) -> Result<ModelConformanceObservation, ConformanceProjectionError> {
    let request = execution.request();

    if request.class != ReceiptClass::Live {
        return Err(ConformanceProjectionError::WrongBranch {
            expected: BranchKind::TransferLive,
            actual: BranchKind::TransferTimeLocked,
        });
    }

    let observation = observe_transition(
        execution.before(),
        execution.after(),
        BranchKind::TransferLive,
        owner_ids(&request.signers),
        owner_ids(&request.fee_envelope.signers),
        RepresentationMode::Explicit,
    )?;

    Ok(bind_model_evidence(observation))
}

/// The bound execution's kernel run validated open-asset conservation
/// on the complete transaction, so the model-side substrate premise is
/// established for exactly this operation and L-BTC.
fn bind_model_evidence(observation: OperationObservation) -> ModelConformanceObservation {
    let established_evidence = BTreeSet::from([
        realization::ExternalEvidenceRequirement::SubstrateConservation {
            operation: observation.operation,
            asset: architecture::AssetId::Lbtc,
        },
    ]);

    ModelConformanceObservation {
        observation,
        established_evidence,
    }
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

    let sponsor_region = sponsor_region(certificate)?;

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

        objects.push(observe_utxo(
            reference,
            utxo,
            representation,
            sponsor_region.contains(outpoint),
        )?);
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

        objects.push(observe_utxo(
            reference,
            utxo,
            representation,
            sponsor_region.contains(outpoint),
        )?);
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
        canonical_partition: observe_canonical_partition(certificate, &reference_by_outpoint)?,
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

/// The exact fee-sponsor region of one certificate: every outpoint
/// claimed by a fee-sponsor open flow, on either side.
///
/// The region is a *flow role*, never an object family (S2-01).
/// `PLAIN_LBTC` is one architecture family that the fee-sponsor region
/// happens to use; the same family also carries protocol-role value —
/// request funding, refund, admission reward, redemption payout — whose
/// amounts the owning protocol relation must be able to read. Deriving
/// the region here, before any value is projected, is what keeps the
/// two apart.
///
/// Overlap is rejected rather than assumed away: the kernel already
/// partitions open flows, but this projection is the erasure point, and
/// a reference that were both sponsor and protocol would have to be
/// erased and readable at once.
pub(crate) fn sponsor_region(
    certificate: &TransitionCertificate,
) -> Result<BTreeSet<OutPoint>, ConformanceProjectionError> {
    let region = |sponsor: bool| {
        certificate
            .open_flows
            .iter()
            .filter(move |flow| (flow.kind == crate::OpenFlowKind::FeeSponsor) == sponsor)
            .flat_map(|flow| flow.source_inputs.iter().chain(&flow.destination_outputs))
            .copied()
            .collect::<BTreeSet<OutPoint>>()
    };

    let sponsor = region(true);
    let protocol = region(false);

    if let Some(outpoint) = sponsor.intersection(&protocol).next() {
        return Err(ConformanceProjectionError::SponsorProtocolRegionOverlap(
            *outpoint,
        ));
    }

    Ok(sponsor)
}

fn observe_utxo(
    reference: ObservedObjectRef,
    utxo: &Utxo,
    representation: RepresentationMode,
    in_sponsor_region: bool,
) -> Result<ObservedObject, ConformanceProjectionError> {
    let kind = observed_object_kind(utxo);

    // This adapter is the erasure point (S3). The transparent model
    // knows every sponsor amount; the realization projection must not.
    // Sponsor L-BTC is therefore erased here, on the way out of the
    // model, rather than carried across the boundary and then guarded
    // against — a value that never crosses cannot be read.
    //
    // Erasure follows the fee-sponsor *flow role* (S2-01), not the
    // `PLAIN_LBTC` family: an ordinary L-BTC reference claimed by a
    // protocol open flow keeps its readable protocol amount, because
    // its owning relation is the thing that has to check it.
    let value = if in_sponsor_region {
        ObservedValue::SponsorOpaque
    } else {
        ObservedValue::Protocol(observed_amount(utxo.value)?)
    };

    Ok(ObservedObject {
        reference,
        kind,
        asset: observed_asset(utxo.asset),
        value,
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

fn observe_canonical_partition(
    certificate: &TransitionCertificate,
    reference_by_outpoint: &BTreeMap<OutPoint, ObservedObjectRef>,
) -> Result<ObservedCanonicalPartition, ConformanceProjectionError> {
    let source_ref = |outpoint: &OutPoint| {
        reference_by_outpoint
            .get(outpoint)
            .copied()
            .ok_or(ConformanceProjectionError::UnknownDeltaSource(*outpoint))
    };
    let destination_ref = |outpoint: &OutPoint| {
        reference_by_outpoint.get(outpoint).copied().ok_or(
            ConformanceProjectionError::UnknownDeltaDestination(*outpoint),
        )
    };

    let mut issuances = Vec::new();

    for issuance in &certificate.canonical_partition.issuances {
        let asset = crate::manifest::declared_asset(issuance.asset)
            .ok_or(ConformanceProjectionError::UndeclaredCanonicalAsset)?;
        let authority = crate::manifest::declared_asset(issuance.authority_asset)
            .ok_or(ConformanceProjectionError::UndeclaredCanonicalAsset)?;
        let authority_input = source_ref(&issuance.authority_input)?;
        let destinations = issuance
            .destination_outputs
            .iter()
            .map(&destination_ref)
            .collect::<Result<Vec<_>, _>>()?;

        issuances.push(ObservedIssuance {
            asset,
            authority,
            authority_input,
            amount: observed_amount(issuance.amount)?,
            destinations,
        });
    }

    let mut flows = Vec::new();

    for flow in &certificate.canonical_partition.flows {
        let asset = crate::manifest::declared_asset(flow.asset)
            .ok_or(ConformanceProjectionError::UndeclaredCanonicalAsset)?;
        let sources = flow
            .source_inputs
            .iter()
            .map(&source_ref)
            .collect::<Result<Vec<_>, _>>()?;
        let destinations = flow
            .destination_outputs
            .iter()
            .map(&destination_ref)
            .collect::<Result<Vec<_>, _>>()?;
        let destructions = flow
            .destructions
            .iter()
            .map(|leg| {
                Ok(ObservedDestructionLeg {
                    tag: architecture_tag(leg.tag),
                    amount: observed_amount(leg.amount)?,
                })
            })
            .collect::<Result<Vec<_>, ConformanceProjectionError>>()?;

        flows.push(ObservedCanonicalFlow {
            asset,
            sources,
            destinations,
            movement_kind: flow.movement_kind.map(architecture_delta_kind),
            destructions,
        });
    }

    Ok(ObservedCanonicalPartition { issuances, flows })
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
        // The certificate records what happened, so the projection is
        // an effect, never a policy: a terminating edge is a
        // termination, not the `SuccessionOrTermination` policy that
        // happens to permit one.
        let effect = match edge {
            RootEdge::Succ { .. } => ObservedRootEffectKind::Succession,
            RootEdge::Term { .. } => ObservedRootEffectKind::Termination,
        };

        effects.push(ObservedRootEffect { root, effect });
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
