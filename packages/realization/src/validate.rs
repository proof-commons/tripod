//! Bidirectional architecture/realization validation.

use std::collections::BTreeSet;

use architecture::{
    Architecture, AssetId, BoundId, DeltaCondition, DeltaKind, InputAuthorization, MaxCount,
    ObjectId, OpenFlowKind, OperationId, PermissionClass, ProjectionId, ProjectionRule, RootId,
    RootUse, ValueFlowClass, WitnessId,
};

use crate::{
    ArchitectureMismatchField, ConstructibilityAuthorization, RealizationError, RepresentationMode,
    ScopedRealizationSpec, require_lifecycle_exit, validate_constructibility,
};

pub fn validate_scoped_realization(
    architecture: &Architecture,
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
    realization.scope.validate_against(architecture)?;

    let scoped = realization
        .scope
        .operations()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let declared = realization
        .operations
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();

    if scoped != declared {
        return Err(RealizationError::UnsupportedOperationDeclaration(
            scoped
                .symmetric_difference(&declared)
                .copied()
                .next()
                .unwrap_or(OperationId::CompactAsh),
        ));
    }

    for operation_id in realization.scope.operations() {
        let operation = architecture.operation(*operation_id).ok_or(
            RealizationError::MissingArchitectureOperation(*operation_id),
        )?;

        for authorization in constructibility_authorizations(operation)? {
            validate_constructibility(
                &realization.constructibility_graph,
                &realization.constructibility_node_by_id,
                *operation_id,
                &authorization,
            )?;
        }
    }

    validate_pilot_lifecycle(realization)?;

    Ok(())
}

/// Derives the constructibility authorization cases from one typed
/// architecture operation row. The operation name is never inspected:
/// the primary `PermissionClass` selects the case shape, and owner/refund
/// cases read the operation's input authorizations. A cadence-band
/// operation yields both its operator and delayed-permissionless windows.
pub fn constructibility_authorizations(
    operation: &architecture::OperationSpec,
) -> Result<Vec<ConstructibilityAuthorization>, RealizationError> {
    let cases = match operation.authorization {
        PermissionClass::Permissionless => vec![ConstructibilityAuthorization::Permissionless],

        PermissionClass::ReceiptOwners => vec![ConstructibilityAuthorization::InputOwners {
            objects: input_owner_objects(operation),
        }],

        PermissionClass::ClientAuthorized => {
            vec![ConstructibilityAuthorization::ClientAuthorized {
                objects: input_owner_objects(operation),
            }]
        }

        PermissionClass::RefundKey => {
            let object = operation
                .inputs
                .iter()
                .find(|input| input.authorization == InputAuthorization::RefundKey)
                .map(|input| input.object)
                .ok_or(RealizationError::ConstructibilityAuthorizationMismatch(
                    operation.id,
                ))?;

            vec![ConstructibilityAuthorization::RefundKey { object }]
        }

        PermissionClass::Operator => vec![ConstructibilityAuthorization::Operator],

        PermissionClass::CadenceBand => vec![
            ConstructibilityAuthorization::CadenceOperator,
            ConstructibilityAuthorization::CadencePermissionless,
        ],
    };

    Ok(cases)
}

/// The set of objects whose consumed owners authorize an operation.
fn input_owner_objects(operation: &architecture::OperationSpec) -> BTreeSet<ObjectId> {
    operation
        .inputs
        .iter()
        .filter(|input| input.authorization == InputAuthorization::InputOwner)
        .map(|input| input.object)
        .collect()
}

fn validate_pilot_lifecycle(realization: &ScopedRealizationSpec) -> Result<(), RealizationError> {
    if realization.scope.contains(OperationId::CompactAsh) {
        for mode in [
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ] {
            for exit in [OperationId::CompactAsh, OperationId::Clear] {
                require_lifecycle_exit(
                    &realization.lifecycle_graph,
                    &realization.lifecycle_node_by_id,
                    ObjectId::Ash,
                    mode,
                    exit,
                )?;
            }
        }
    }

    if realization.scope.contains(OperationId::TransferLive) {
        for mode in [
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
        ] {
            for exit in [
                OperationId::TransferLive,
                OperationId::Burn,
                OperationId::Redeem,
            ] {
                require_lifecycle_exit(
                    &realization.lifecycle_graph,
                    &realization.lifecycle_node_by_id,
                    ObjectId::ReceiptLive,
                    mode,
                    exit,
                )?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub fn validate_compact_ash_architecture(
    architecture: &Architecture,
) -> Result<(), RealizationError> {
    let operation = architecture.operation(OperationId::CompactAsh).ok_or(
        RealizationError::MissingArchitectureOperation(OperationId::CompactAsh),
    )?;

    if operation.authorization != PermissionClass::Permissionless {
        return mismatch_compact(ArchitectureMismatchField::Authorization);
    }

    let input_families = operation
        .inputs
        .iter()
        .map(|input| input.object)
        .collect::<BTreeSet<_>>();

    if input_families != BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]) {
        return mismatch_compact(ArchitectureMismatchField::InputFamilies);
    }

    let output_families = operation
        .outputs
        .iter()
        .map(|output| output.object)
        .collect::<BTreeSet<_>>();

    if output_families != BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]) {
        return mismatch_compact(ArchitectureMismatchField::OutputFamilies);
    }

    let ash_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::Ash)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::InputFamilies))?;

    if ash_input.minimum != 2
        || ash_input.maximum != MaxCount::Bound(BoundId::AshBatchMax)
        || ash_input.authorization != InputAuthorization::Permissionless
    {
        return mismatch_compact(ArchitectureMismatchField::AshInput);
    }

    let ash_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::Ash)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::OutputFamilies))?;

    if ash_output.minimum != 1 || ash_output.maximum != MaxCount::Exact(1) {
        return mismatch_compact(ArchitectureMismatchField::AshOutput);
    }

    let sponsor_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::PlainLbtc)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::SponsorInput))?;

    if sponsor_input.minimum != 0
        || sponsor_input.maximum != MaxCount::Bound(BoundId::FeeSponsorInputMax)
        || sponsor_input.authorization != InputAuthorization::SponsorOwner
    {
        return mismatch_compact(ArchitectureMismatchField::SponsorInput);
    }

    let sponsor_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::PlainLbtc)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::SponsorOutput))?;

    if sponsor_output.minimum != 0 || sponsor_output.maximum != MaxCount::Exact(1) {
        return mismatch_compact(ArchitectureMismatchField::SponsorOutput);
    }

    if operation.bounds.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([BoundId::AshBatchMax, BoundId::FeeSponsorInputMax])
    {
        return mismatch_compact(ArchitectureMismatchField::Bounds);
    }

    if operation.canonical_deltas.len() != 1 {
        return mismatch_compact(ArchitectureMismatchField::CanonicalDelta);
    }

    let delta = operation.canonical_deltas[0];
    if delta.asset != architecture::AssetId::U
        || delta.kind != DeltaKind::OwnerlessLateral
        || delta.condition != DeltaCondition::Always
        || delta.destruction_tag.is_some()
    {
        return mismatch_compact(ArchitectureMismatchField::CanonicalDelta);
    }

    if operation
        .open_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([OpenFlowKind::FeeSponsor])
    {
        return mismatch_compact(ArchitectureMismatchField::OpenFlows);
    }

    if operation
        .value_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            ValueFlowClass::OwnerlessBoundMovement,
            ValueFlowClass::SponsorEnvelope,
        ])
    {
        return mismatch_compact(ArchitectureMismatchField::ValueFlows);
    }

    for root in RootId::ALL {
        if operation.root_use(*root) != RootUse::Forbidden {
            return mismatch_compact(ArchitectureMismatchField::RootPolicy);
        }
    }

    if operation.projection_rule(ProjectionId::TransitionCertificate) != ProjectionRule::Required {
        return mismatch_compact(ArchitectureMismatchField::ProjectionPolicy);
    }

    for projection in [
        ProjectionId::BurnEvent,
        ProjectionId::ClearEvent,
        ProjectionId::DistributionResidue,
    ] {
        if operation.projection_rule(projection) != ProjectionRule::Forbidden {
            return mismatch_compact(ArchitectureMismatchField::ProjectionPolicy);
        }
    }

    if !operation.data_outputs.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::DataOutputs);
    }

    if operation.witnesses.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([
            WitnessId::CanonicalDelta,
            WitnessId::AshLineage,
            WitnessId::ValueFlowClosure,
            WitnessId::UtxoLifecycle,
        ])
    {
        return mismatch_compact(ArchitectureMismatchField::Witnesses);
    }

    Ok(())
}

fn compact_error(field: ArchitectureMismatchField) -> RealizationError {
    RealizationError::ArchitectureOperationMismatch {
        operation: OperationId::CompactAsh,
        field,
    }
}

fn mismatch_compact(field: ArchitectureMismatchField) -> Result<(), RealizationError> {
    Err(compact_error(field))
}

#[allow(clippy::too_many_lines)]
pub fn validate_live_transfer_architecture(
    architecture: &Architecture,
) -> Result<(), RealizationError> {
    let operation = architecture.operation(OperationId::TransferLive).ok_or(
        RealizationError::MissingArchitectureOperation(OperationId::TransferLive),
    )?;

    if operation.authorization != PermissionClass::ReceiptOwners {
        return mismatch_live(ArchitectureMismatchField::Authorization);
    }

    let input_families = operation
        .inputs
        .iter()
        .map(|input| input.object)
        .collect::<BTreeSet<_>>();

    if input_families != BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]) {
        return mismatch_live(ArchitectureMismatchField::InputFamilies);
    }

    let output_families = operation
        .outputs
        .iter()
        .map(|output| output.object)
        .collect::<BTreeSet<_>>();

    if output_families != BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]) {
        return mismatch_live(ArchitectureMismatchField::OutputFamilies);
    }

    let receipt_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::ReceiptLive)
        .ok_or_else(|| live_error(ArchitectureMismatchField::ReceiptInput))?;

    if receipt_input.minimum != 1
        || receipt_input.maximum != MaxCount::Bound(BoundId::TransferInputMax)
        || receipt_input.authorization != InputAuthorization::InputOwner
    {
        return mismatch_live(ArchitectureMismatchField::ReceiptInput);
    }

    let sponsor_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::PlainLbtc)
        .ok_or_else(|| live_error(ArchitectureMismatchField::SponsorInput))?;

    if sponsor_input.minimum != 0
        || sponsor_input.maximum != MaxCount::Bound(BoundId::FeeSponsorInputMax)
        || sponsor_input.authorization != InputAuthorization::SponsorOwner
    {
        return mismatch_live(ArchitectureMismatchField::SponsorInput);
    }

    let receipt_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::ReceiptLive)
        .ok_or_else(|| live_error(ArchitectureMismatchField::ReceiptOutput))?;

    if receipt_output.minimum != 1
        || receipt_output.maximum != MaxCount::Bound(BoundId::TransferOutputMax)
    {
        return mismatch_live(ArchitectureMismatchField::ReceiptOutput);
    }

    let sponsor_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::PlainLbtc)
        .ok_or_else(|| live_error(ArchitectureMismatchField::SponsorOutput))?;

    if sponsor_output.minimum != 0 || sponsor_output.maximum != MaxCount::Exact(1) {
        return mismatch_live(ArchitectureMismatchField::SponsorOutput);
    }

    if operation.bounds.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([
            BoundId::TransferInputMax,
            BoundId::TransferOutputMax,
            BoundId::FeeSponsorInputMax,
        ])
    {
        return mismatch_live(ArchitectureMismatchField::Bounds);
    }

    if operation
        .open_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([OpenFlowKind::FeeSponsor])
    {
        return mismatch_live(ArchitectureMismatchField::OpenFlows);
    }

    if operation
        .value_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::SponsorEnvelope,
        ])
    {
        return mismatch_live(ArchitectureMismatchField::ValueFlows);
    }

    for root in RootId::ALL {
        if operation.root_use(*root) != RootUse::Forbidden {
            return mismatch_live(ArchitectureMismatchField::RootPolicy);
        }
    }

    if operation.canonical_deltas.len() != 1 {
        return mismatch_live(ArchitectureMismatchField::CanonicalDeltas);
    }

    let delta = operation.canonical_deltas[0];
    if delta.asset != AssetId::U
        || delta.kind != DeltaKind::Lateral
        || delta.condition != DeltaCondition::Always
        || delta.destruction_tag.is_some()
    {
        return mismatch_live(ArchitectureMismatchField::CanonicalDeltas);
    }

    if !operation.data_outputs.is_empty() {
        return mismatch_live(ArchitectureMismatchField::DataOutputs);
    }

    if operation.projection_rule(ProjectionId::TransitionCertificate) != ProjectionRule::Required {
        return mismatch_live(ArchitectureMismatchField::ProjectionPolicy);
    }

    for projection in [
        ProjectionId::BurnEvent,
        ProjectionId::ClearEvent,
        ProjectionId::DistributionResidue,
    ] {
        if operation.projection_rule(projection) != ProjectionRule::Forbidden {
            return mismatch_live(ArchitectureMismatchField::ProjectionPolicy);
        }
    }

    if operation.witnesses.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([
            WitnessId::CanonicalDelta,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
            WitnessId::ValueFlowClosure,
        ])
    {
        return mismatch_live(ArchitectureMismatchField::Witnesses);
    }

    Ok(())
}

fn live_error(field: ArchitectureMismatchField) -> RealizationError {
    RealizationError::ArchitectureOperationMismatch {
        operation: OperationId::TransferLive,
        field,
    }
}

fn mismatch_live(field: ArchitectureMismatchField) -> Result<(), RealizationError> {
    Err(live_error(field))
}
