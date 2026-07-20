//! Bidirectional architecture/realization validation.

use std::collections::BTreeSet;

use architecture::{
    Architecture, BoundId, DeltaCondition, DeltaKind, InputAuthorization, MaxCount, ObjectId,
    OpenFlowKind, OperationId, PermissionClass, ProjectionId, ProjectionRule, RootId, RootUse,
    ValueFlowClass,
};

use crate::{ArchitectureMismatchField, RealizationError, ScopedRealizationSpec};

pub fn validate_scoped_realization(
    _architecture: &Architecture,
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
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

    if scoped == declared {
        Ok(())
    } else {
        Err(RealizationError::UnsupportedOperationDeclaration(
            scoped
                .symmetric_difference(&declared)
                .copied()
                .next()
                .unwrap_or(OperationId::CompactAsh),
        ))
    }
}

pub fn validate_compact_ash_architecture(
    architecture: &Architecture,
) -> Result<(), RealizationError> {
    let operation = architecture.operation(OperationId::CompactAsh).ok_or(
        RealizationError::MissingArchitectureOperation(OperationId::CompactAsh),
    )?;

    if operation.authorization != PermissionClass::Permissionless {
        return mismatch(ArchitectureMismatchField::Authorization);
    }

    let input_families = operation
        .inputs
        .iter()
        .map(|input| input.object)
        .collect::<BTreeSet<_>>();

    if input_families != BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]) {
        return mismatch(ArchitectureMismatchField::InputFamilies);
    }

    let output_families = operation
        .outputs
        .iter()
        .map(|output| output.object)
        .collect::<BTreeSet<_>>();

    if output_families != BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]) {
        return mismatch(ArchitectureMismatchField::OutputFamilies);
    }

    let ash_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::Ash)
        .ok_or(RealizationError::ArchitectureOperationMismatch {
            operation: OperationId::CompactAsh,
            field: ArchitectureMismatchField::InputFamilies,
        })?;

    if ash_input.minimum != 2
        || ash_input.maximum != MaxCount::Bound(BoundId::AshBatchMax)
        || ash_input.authorization != InputAuthorization::Permissionless
    {
        return mismatch(ArchitectureMismatchField::AshInput);
    }

    let ash_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::Ash)
        .ok_or(RealizationError::ArchitectureOperationMismatch {
            operation: OperationId::CompactAsh,
            field: ArchitectureMismatchField::OutputFamilies,
        })?;

    if ash_output.minimum != 1 || ash_output.maximum != MaxCount::Exact(1) {
        return mismatch(ArchitectureMismatchField::AshOutput);
    }

    if operation.canonical_deltas.len() != 1 {
        return mismatch(ArchitectureMismatchField::CanonicalDelta);
    }

    let delta = operation.canonical_deltas[0];
    if delta.asset != architecture::AssetId::U
        || delta.kind != DeltaKind::OwnerlessLateral
        || delta.condition != DeltaCondition::Always
        || delta.destruction_tag.is_some()
    {
        return mismatch(ArchitectureMismatchField::CanonicalDelta);
    }

    if operation.open_flows != [OpenFlowKind::FeeSponsor] {
        return mismatch(ArchitectureMismatchField::OpenFlows);
    }

    if !operation
        .value_flows
        .contains(&ValueFlowClass::OwnerlessBoundMovement)
        || !operation
            .value_flows
            .contains(&ValueFlowClass::SponsorEnvelope)
    {
        return mismatch(ArchitectureMismatchField::ValueFlows);
    }

    for root in RootId::ALL {
        if operation.root_use(*root) != RootUse::Forbidden {
            return mismatch(ArchitectureMismatchField::RootPolicy);
        }
    }

    if operation.projection_rule(ProjectionId::TransitionCertificate) != ProjectionRule::Required {
        return mismatch(ArchitectureMismatchField::ProjectionPolicy);
    }

    for projection in [
        ProjectionId::BurnEvent,
        ProjectionId::ClearEvent,
        ProjectionId::DistributionResidue,
    ] {
        if operation.projection_rule(projection) != ProjectionRule::Forbidden {
            return mismatch(ArchitectureMismatchField::ProjectionPolicy);
        }
    }

    Ok(())
}

fn mismatch(field: ArchitectureMismatchField) -> Result<(), RealizationError> {
    Err(RealizationError::ArchitectureOperationMismatch {
        operation: OperationId::CompactAsh,
        field,
    })
}
