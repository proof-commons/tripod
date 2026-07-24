//! Bidirectional architecture/realization validation.

use std::collections::BTreeSet;

use architecture::{
    Architecture, AssetId, BoundId, DeltaCondition, DeltaKind, InputAuthorization, MaxCount,
    ObjectId, OpenFlowKind, OperationId, OperationKind, PermissionClass, ProjectionId,
    ProjectionRule, RootId, RootUse, ValueFlowClass, WitnessId,
};

use crate::{
    ArchitectureMismatchField, ConstructibilityAuthorization, ConstructibilityNodeId,
    DisclosureNode, ExpressionNode, FactId, RealizationError, RepresentationMode,
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

    validate_sponsor_value_opacity(realization)?;

    Ok(())
}

/// Sponsor-value opacity (F2-006): an ordinary sponsor L-BTC
/// amount — individual or aggregate — is sponsor-local data, never a
/// protocol-readable fact.
///
/// The structural read-set guard: no expression, disclosure node,
/// declassification entry, or constructibility fact may name a
/// `PLAIN_LBTC` family amount, on either side. Sponsor safety is
/// discharged by asset authentication, family recognition, exact
/// membership, owner authorization, isolation, and conservation — a
/// backend must not add an amount read merely because its target
/// exposes an introspection primitive.
fn validate_sponsor_value_opacity(
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
    fn is_sponsor_amount(fact: &FactId) -> bool {
        matches!(
            fact,
            FactId::FamilyAmount {
                object: ObjectId::PlainLbtc,
                ..
            }
        )
    }

    let expression_read = realization
        .expression_graph
        .node_weights()
        .any(|declaration| {
            matches!(&declaration.node, ExpressionNode::Fact(fact) if is_sponsor_amount(fact))
        });

    let disclosure_read = realization
        .disclosure_graph
        .node_weights()
        .any(|node| matches!(node, DisclosureNode::Fact { id, .. } if is_sponsor_amount(id)));

    let constructibility_read = realization.constructibility_node_by_id.keys().any(
        |node| matches!(node, ConstructibilityNodeId::Fact { fact, .. } if is_sponsor_amount(fact)),
    );

    let declassification = &realization.declassification;
    let declassification_read = declassification
        .required_public
        .keys()
        .chain(declassification.newly_disclosed.keys())
        .chain(declassification.retained_private.iter())
        .any(is_sponsor_amount);

    if expression_read || disclosure_read || constructibility_read || declassification_read {
        return Err(RealizationError::SponsorValueRead);
    }

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

    if operation.kind != OperationKind::CovenantBranch {
        return mismatch_compact(ArchitectureMismatchField::OperationKind);
    }

    if operation.authorization != PermissionClass::Permissionless {
        return mismatch_compact(ArchitectureMismatchField::Authorization);
    }

    if !operation.issuances.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::Issuances);
    }

    if !operation.reads.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::Reads);
    }

    if !operation.writes.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::Writes);
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

    if operation.kind != OperationKind::CovenantBranch {
        return mismatch_live(ArchitectureMismatchField::OperationKind);
    }

    if operation.authorization != PermissionClass::ReceiptOwners {
        return mismatch_live(ArchitectureMismatchField::Authorization);
    }

    if !operation.issuances.is_empty() {
        return mismatch_live(ArchitectureMismatchField::Issuances);
    }

    if !operation.reads.is_empty() {
        return mismatch_live(ArchitectureMismatchField::Reads);
    }

    if !operation.writes.is_empty() {
        return mismatch_live(ArchitectureMismatchField::Writes);
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

/// Generic per-operation ownership validation (F3-006).
///
/// Runs before graph assembly so a mistyped declaration fails with a
/// focused ownership error rather than a graph-shaped one, and so no
/// invariant rests on helper-constructor correctness. Checked per
/// operation: the declaration's own identity, expression IDs and
/// operands, relation IDs, relation-dependency endpoints, proof
/// alternative bindings, constructibility nodes and edge endpoints,
/// and disclosure nodes, edge endpoints, and seeds.
///
/// Deliberate cross-operation meanings stay valid because they are
/// typed payloads, not owners: a lifecycle `RequiredExit` (and the
/// `LifecycleExit` relation subject) names its exit operation as
/// semantic content while the declaring relation remains owned by the
/// declaring operation, and an architecture-owned `BoundValue` fact
/// has no owning operation at all.
pub fn validate_operation_ownership(
    requested: OperationId,
    declaration: &crate::OperationRealization,
) -> Result<(), RealizationError> {
    if declaration.operation != requested {
        return Err(RealizationError::OperationDeclarationIdentityMismatch {
            requested,
            declared: declaration.operation,
        });
    }

    for expression in &declaration.expressions {
        ensure_expression_owner(requested, &expression.id)?;
        for operand in expression_operand_ids(&expression.node) {
            ensure_expression_owner(requested, operand)?;
        }
        if let ExpressionNode::Fact(fact) = &expression.node
            && fact_owner(fact).is_some_and(|owner| owner != requested)
        {
            return Err(RealizationError::ForeignExpressionOwnership {
                operation: requested,
                expression: crate::ExprId::fact(fact.clone()),
            });
        }
    }

    for relation in &declaration.relations {
        if relation.id.operation() != requested {
            return Err(RealizationError::ForeignRelationOwnership {
                operation: requested,
                relation: relation.id.clone(),
            });
        }
        for alternative in &relation.proof_alternatives {
            if alternative.relation() != &relation.id {
                return Err(RealizationError::ForeignProofAlternativeBinding {
                    relation: relation.id.clone(),
                    foreign: alternative.relation().clone(),
                });
            }
        }
    }

    for dependency in &declaration.relation_dependencies {
        for endpoint in [&dependency.prerequisite, &dependency.dependent] {
            if endpoint.operation() != requested {
                return Err(RealizationError::ForeignRelationDependency {
                    operation: requested,
                    relation: endpoint.clone(),
                });
            }
        }
    }

    for node in &declaration.constructibility_nodes {
        ensure_constructibility_owner(requested, &node.id)?;
    }
    for edge in &declaration.constructibility_edges {
        ensure_constructibility_owner(requested, &edge.source)?;
        ensure_constructibility_owner(requested, &edge.target)?;
    }

    for node in &declaration.disclosure_nodes {
        ensure_disclosure_owner(requested, &node.id())?;
    }
    for edge in &declaration.disclosure_edges {
        ensure_disclosure_owner(requested, &edge.source)?;
        ensure_disclosure_owner(requested, &edge.target)?;
    }
    for seed in &declaration.disclosure_seeds {
        ensure_disclosure_owner(requested, &seed.node)?;
        for relation in seed_reason_relations(&seed.reason) {
            if relation.operation() != requested {
                return Err(RealizationError::ForeignDisclosureOwnership {
                    operation: requested,
                    node: crate::DisclosureNodeId::Relation(relation.clone()),
                });
            }
        }
        if let crate::DisclosureReason::PermissionlessConstructibility { operation, .. } =
            &seed.reason
            && *operation != requested
        {
            return Err(RealizationError::ForeignDisclosureOwnership {
                operation: requested,
                node: seed.node.clone(),
            });
        }
    }

    Ok(())
}

/// Operation owning one primitive fact, if any.
///
/// `BoundValue` is architecture-owned and belongs to no operation.
fn fact_owner(fact: &FactId) -> Option<OperationId> {
    match fact {
        FactId::FamilyCount { operation, .. }
        | FactId::FamilyAmount { operation, .. }
        | FactId::InputOwners { operation, .. }
        | FactId::Signers { operation }
        | FactId::ProjectionPresent { operation, .. }
        | FactId::FamilyRecognized { operation, .. }
        | FactId::SponsorIsolated { operation }
        | FactId::ProtocolSecretUsed { operation } => Some(*operation),
        FactId::BoundValue { .. } => None,
    }
}

fn expression_owner(id: &crate::ExprId) -> Option<OperationId> {
    match id {
        crate::ExprId::Fact(fact) => fact_owner(fact),
        crate::ExprId::Relation { relation, .. } => Some(relation.operation()),
    }
}

fn ensure_expression_owner(
    requested: OperationId,
    id: &crate::ExprId,
) -> Result<(), RealizationError> {
    if expression_owner(id).is_some_and(|owner| owner != requested) {
        return Err(RealizationError::ForeignExpressionOwnership {
            operation: requested,
            expression: id.clone(),
        });
    }
    Ok(())
}

fn expression_operand_ids(node: &ExpressionNode) -> Vec<&crate::ExprId> {
    match node {
        ExpressionNode::Fact(_)
        | ExpressionNode::Bool(_)
        | ExpressionNode::Count(_)
        | ExpressionNode::Amount(_) => Vec::new(),
        ExpressionNode::CheckedSum { terms, .. } | ExpressionNode::All { terms } => {
            terms.iter().collect()
        }
        ExpressionNode::Equal { left, right } | ExpressionNode::LessOrEqual { left, right } => {
            vec![left, right]
        }
        ExpressionNode::OwnerSubset {
            required,
            presented,
        } => vec![required, presented],
    }
}

fn ensure_constructibility_owner(
    requested: OperationId,
    node: &ConstructibilityNodeId,
) -> Result<(), RealizationError> {
    let owner = match node {
        ConstructibilityNodeId::Operation(operation)
        | ConstructibilityNodeId::Fact { operation, .. }
        | ConstructibilityNodeId::Witness { operation, .. } => *operation,
    };
    if owner != requested {
        return Err(RealizationError::ForeignConstructibilityOwnership {
            operation: requested,
            node: node.clone(),
        });
    }
    // A constructibility fact payload must agree with its declared
    // operation envelope.
    if let ConstructibilityNodeId::Fact { fact, .. } = node
        && fact_owner(fact).is_some_and(|owner| owner != requested)
    {
        return Err(RealizationError::ForeignConstructibilityOwnership {
            operation: requested,
            node: node.clone(),
        });
    }
    Ok(())
}

fn ensure_disclosure_owner(
    requested: OperationId,
    node: &crate::DisclosureNodeId,
) -> Result<(), RealizationError> {
    let owner = match node {
        crate::DisclosureNodeId::Fact(fact) => fact_owner(fact),
        crate::DisclosureNodeId::Relation(relation) => Some(relation.operation()),
    };
    if owner.is_some_and(|owner| owner != requested) {
        return Err(RealizationError::ForeignDisclosureOwnership {
            operation: requested,
            node: node.clone(),
        });
    }
    Ok(())
}

/// Relations named inside one disclosure-seed reason.
fn seed_reason_relations(reason: &crate::DisclosureReason) -> Vec<&crate::RelationId> {
    match reason {
        crate::DisclosureReason::PermissionlessConstructibility { relation, .. }
        | crate::DisclosureReason::TargetSafety { relation } => vec![relation],
        crate::DisclosureReason::PublicState
        | crate::DisclosureReason::PublicEvent
        | crate::DisclosureReason::PublicInterface
        | crate::DisclosureReason::DeploymentPolicy { .. } => Vec::new(),
    }
}
