//! Evaluation of typed semantic relations over primitive observations.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, DeltaKind, ObjectId, ProjectionId, ProjectionRule, RootId, RootUse};
use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef as _,
};

use crate::{
    CardinalityMaximum, ConstructibilityClass, Count, EvaluatedExpressions, ExpectedCanonicalDelta,
    ExprId, FactId, ObservedAsset, ObservedObject, ObservedObjectKind, ObservedObjectRef,
    ObservedOpenFlow, ObservedSide, OperationObservation, ProtocolAmount, RealizationError,
    Relation, RelationDeclaration, RelationEdge, RelationId, SemanticValue, TransactionSide,
    expression::{DependencyEdge, ExpressionDeclaration, FactValues},
};

/// Result class for one realization relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationStatus {
    Passed,
    Failed { reason: RelationFailure },
    Blocked { prerequisites: Vec<RelationId> },
    StaticallyValidated,
}

/// Focused runtime failure class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationFailure {
    CardinalityBelowMinimum,
    CardinalityAboveMaximum,
    UndeclaredObjectFamily,
    ObjectRecognition,
    AmountConservation,
    MissingOwnerAuthorization,
    UnexpectedProtocolAuthorization,
    SponsorIsolation,
    SponsorEnvelopeMultiplicity,
    RootPolicy,
    ProjectionPolicy,
    Constructibility,
    Representation,
    CanonicalDeltaPolicy,
    OpenFlowPolicy,
    ExpressionPredicate,
}

/// One stable relation verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationVerdict {
    pub relation: RelationId,
    pub status: RelationStatus,
}

/// Deterministically ordered conformance report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConformanceReport {
    pub operation: architecture::OperationId,
    pub verdicts: Vec<RelationVerdict>,
}

impl ConformanceReport {
    #[must_use]
    pub fn is_conformant(&self) -> bool {
        self.verdicts.iter().all(|verdict| {
            matches!(
                verdict.status,
                RelationStatus::Passed | RelationStatus::StaticallyValidated
            )
        })
    }

    pub fn failed_relations(&self) -> impl Iterator<Item = &RelationId> {
        self.verdicts.iter().filter_map(|verdict| {
            matches!(
                verdict.status,
                RelationStatus::Failed { .. } | RelationStatus::Blocked { .. }
            )
            .then_some(&verdict.relation)
        })
    }

    #[must_use]
    pub fn verdict(&self, relation: &RelationId) -> Option<&RelationVerdict> {
        self.verdicts
            .binary_search_by(|verdict| verdict.relation.cmp(relation))
            .ok()
            .map(|index| &self.verdicts[index])
    }
}

/// Evaluate all relations owned by one operation.
#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_operation(
    relation_graph: &DiGraph<RelationDeclaration, RelationEdge, u32>,
    relation_node_by_id: &BTreeMap<RelationId, NodeIndex<u32>>,
    relation_evaluation_order: &[RelationId],
    expression_graph: &DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
    expression_node_by_id: &BTreeMap<ExprId, NodeIndex<u32>>,
    expression_evaluation_order: &[ExprId],
    observation: &OperationObservation,
) -> Result<ConformanceReport, RealizationError> {
    let observation = observation.clone().validate_and_normalize()?;
    let evaluated = evaluate_operation_expressions(
        relation_graph,
        expression_graph,
        expression_node_by_id,
        expression_evaluation_order,
        &observation,
    )?;
    let mut status_by_relation = BTreeMap::new();

    for relation_id in relation_evaluation_order {
        let node = relation_node_by_id
            .get(relation_id)
            .copied()
            .ok_or_else(|| RealizationError::UnknownRelation(relation_id.clone()))?;
        let declaration = &relation_graph[node];

        if declaration.id.operation() != observation.operation {
            continue;
        }

        let mut blocking = relation_graph
            .edges_directed(node, Direction::Incoming)
            .filter_map(|edge| {
                let prerequisite = &relation_graph[edge.source()].id;

                match status_by_relation.get(prerequisite) {
                    Some(RelationStatus::Passed | RelationStatus::StaticallyValidated) => None,
                    Some(RelationStatus::Failed { .. } | RelationStatus::Blocked { .. }) => {
                        Some(prerequisite.clone())
                    }
                    None => Some(prerequisite.clone()),
                }
            })
            .collect::<Vec<_>>();
        blocking.sort();
        blocking.dedup();

        let status = if blocking.is_empty() {
            evaluate_relation(&declaration.relation, &observation, evaluated.as_ref())?
        } else {
            RelationStatus::Blocked {
                prerequisites: blocking,
            }
        };

        status_by_relation.insert(declaration.id.clone(), status);
    }

    let verdicts = status_by_relation
        .into_iter()
        .map(|(relation, status)| RelationVerdict { relation, status })
        .collect();

    Ok(ConformanceReport {
        operation: observation.operation,
        verdicts,
    })
}

#[allow(clippy::too_many_lines)]
fn evaluate_relation(
    relation: &Relation,
    observation: &OperationObservation,
    evaluated: Option<&EvaluatedExpressions>,
) -> Result<RelationStatus, RealizationError> {
    match relation {
        Relation::Cardinality {
            side,
            object,
            minimum,
            maximum,
        } => {
            let count = Count::new(
                u64::try_from(declared_objects(observation, *side, *object).count())
                    .map_err(|_| RealizationError::CountOverflow)?,
            );
            let maximum = match maximum {
                CardinalityMaximum::Exact(value) => *value,
                CardinalityMaximum::Bound(bound) => observation
                    .bounds
                    .get(bound)
                    .copied()
                    .ok_or(RealizationError::MissingBoundValue(*bound))?,
            };

            if count < *minimum {
                status(false, RelationFailure::CardinalityBelowMinimum)
            } else if count > maximum {
                status(false, RelationFailure::CardinalityAboveMaximum)
            } else {
                status(true, RelationFailure::CardinalityBelowMinimum)
            }
        }
        Relation::AllowedObjectFamilies { side, allowed } => status(
            observation
                .objects
                .iter()
                .filter(|object| object.reference.side == *side)
                .all(|object| matches!(object.kind, ObservedObjectKind::Declared(kind) if allowed.contains(&kind))),
            RelationFailure::UndeclaredObjectFamily,
        ),
        Relation::Recognition {
            side,
            object,
            asset,
        } => status(
            declared_objects(observation, *side, *object).all(|observed| {
                observed.asset == ObservedAsset::Declared(*asset)
                    && observed_object_shape_holds(*object, observed)
            }),
            RelationFailure::ObjectRecognition,
        ),
        Relation::AmountConservation {
            asset,
            input_objects,
            output_objects,
        } => {
            let input = sum_selected_amounts(observation, ObservedSide::Input, *asset, input_objects)?;
            let output = sum_selected_amounts(observation, ObservedSide::Output, *asset, output_objects)?;

            status(input == output, RelationFailure::AmountConservation)
        }
        Relation::OwnerAuthorization { object } => {
            let Some(required) = required_owners(observation, *object) else {
                return status(false, RelationFailure::MissingOwnerAuthorization);
            };

            status(
                required.is_subset(&observation.protocol_signers),
                RelationFailure::MissingOwnerAuthorization,
            )
        }
        Relation::PermissionlessAuthorization => status(
            observation.protocol_signers.is_empty(),
            RelationFailure::UnexpectedProtocolAuthorization,
        ),
        Relation::SponsorIsolation => status(
            sponsor_is_isolated(observation)?,
            RelationFailure::SponsorIsolation,
        ),
        Relation::SponsorEnvelopeMultiplicity { maximum } => {
            let count = Count::new(
                u64::try_from(
                    observation
                        .open_flows
                        .iter()
                        .filter(|flow| flow.kind == architecture::OpenFlowKind::FeeSponsor)
                        .count(),
                )
                .map_err(|_| RealizationError::CountOverflow)?,
            );
            status(
                count <= *maximum,
                RelationFailure::SponsorEnvelopeMultiplicity,
            )
        }
        Relation::RootPolicy { expected } => {
            status(root_policy_holds(expected, observation), RelationFailure::RootPolicy)
        }
        Relation::ProjectionPolicy { expected } => status(
            projection_policy_holds(expected, observation),
            RelationFailure::ProjectionPolicy,
        ),
        Relation::CanonicalDeltaPolicy { expected } => status(
            canonical_delta_policy_holds(expected, observation)?,
            RelationFailure::CanonicalDeltaPolicy,
        ),
        Relation::OpenFlowPolicy { allowed } => status(
            observation
                .open_flows
                .iter()
                .all(|flow| allowed.contains(&flow.kind)),
            RelationFailure::OpenFlowPolicy,
        ),
        Relation::Constructibility { class } => match class {
            ConstructibilityClass::PublicPermissionless => status(
                observation.protocol_signers.is_empty(),
                RelationFailure::Constructibility,
            ),
            ConstructibilityClass::OwnersOf { object } => {
                let Some(required) = required_owners(observation, *object) else {
                    return status(false, RelationFailure::Constructibility);
                };

                status(
                    required.is_subset(&observation.protocol_signers),
                    RelationFailure::Constructibility,
                )
            }
        },
        Relation::Representation { object, allowed } => status(
            observation.objects.iter().all(|observed| {
                observed.kind != ObservedObjectKind::Declared(*object)
                    || allowed.contains(&observed.representation)
            }),
            RelationFailure::Representation,
        ),
        Relation::LifecycleExit { .. } => Ok(RelationStatus::StaticallyValidated),
        Relation::ExpressionPredicate { expression } => {
            let evaluated =
                evaluated.ok_or_else(|| RealizationError::UnknownEvaluatedExpression {
                    expression: expression.clone(),
                })?;

            status(
                evaluated.bool(expression)?,
                RelationFailure::ExpressionPredicate,
            )
        }
    }
}

/// Evaluate the expression closure this operation's expression-predicate
/// relations need, deriving every primitive fact from the observation.
///
/// Returns `None` when the operation declares no expression predicate,
/// so an expression-free operation never touches the expression graph.
fn evaluate_operation_expressions(
    relation_graph: &DiGraph<RelationDeclaration, RelationEdge, u32>,
    expression_graph: &DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
    expression_node_by_id: &BTreeMap<ExprId, NodeIndex<u32>>,
    expression_evaluation_order: &[ExprId],
    observation: &OperationObservation,
) -> Result<Option<EvaluatedExpressions>, RealizationError> {
    let predicates = relation_graph
        .node_weights()
        .filter(|declaration| declaration.id.operation() == observation.operation)
        .filter_map(|declaration| match &declaration.relation {
            Relation::ExpressionPredicate { expression } => Some(expression.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    if predicates.is_empty() {
        return Ok(None);
    }

    let scope =
        crate::expression::ancestor_closure(expression_graph, expression_node_by_id, predicates)?;
    let mut facts = FactValues::default();

    for id in &scope {
        if let ExprId::Fact(fact) = id {
            facts.insert(fact.clone(), derive_fact(observation, fact)?)?;
        }
    }

    crate::expression::evaluate_expressions_in(
        expression_graph,
        expression_node_by_id,
        expression_evaluation_order,
        &scope,
        &facts,
    )
    .map(Some)
}

/// Derive one primitive fact value from the observation.
///
/// Facts are derived from primitive observed structure — never accepted
/// as caller-authored assertions — and an operation-scoped fact of
/// another operation is a declaration defect, not an evaluation input.
fn derive_fact(
    observation: &OperationObservation,
    fact: &FactId,
) -> Result<SemanticValue, RealizationError> {
    if fact
        .operation()
        .is_some_and(|operation| operation != observation.operation)
    {
        return Err(RealizationError::ForeignExpressionFact {
            fact: fact.clone(),
            operation: observation.operation,
        });
    }

    match fact {
        FactId::FamilyCount { side, object, .. } => {
            let count = Count::new(
                u64::try_from(declared_objects(observation, observed_side(*side), *object).count())
                    .map_err(|_| RealizationError::CountOverflow)?,
            );

            Ok(SemanticValue::Count(count))
        }
        FactId::FamilyAmount { side, object, .. } => {
            let total = ProtocolAmount::checked_sum(
                declared_objects(observation, observed_side(*side), *object)
                    .map(|observed| observed.value),
            )?;

            Ok(SemanticValue::Amount(total))
        }
        FactId::InputOwners { object, .. } => required_owners(observation, *object)
            .map(SemanticValue::OwnerSet)
            .ok_or_else(|| RealizationError::UnderivableOwnerFact { fact: fact.clone() }),
        FactId::Signers { .. } => Ok(SemanticValue::OwnerSet(
            observation.protocol_signers.clone(),
        )),
        FactId::ProjectionPresent { projection, .. } => Ok(SemanticValue::Bool(
            observation.projections.contains(projection),
        )),
        FactId::BoundValue { bound } => observation
            .bounds
            .get(bound)
            .copied()
            .map(SemanticValue::Count)
            .ok_or(RealizationError::MissingBoundValue(*bound)),
        FactId::FamilyRecognized { side, object, .. } => {
            let recognized = architecture::ARCHITECTURE
                .object(*object)
                .is_some_and(|spec| {
                    declared_objects(observation, observed_side(*side), *object).all(|observed| {
                        observed.asset == ObservedAsset::Declared(spec.asset)
                            && observed_object_shape_holds(*object, observed)
                    })
                });

            Ok(SemanticValue::Bool(recognized))
        }
        FactId::SponsorIsolated { .. } => {
            Ok(SemanticValue::Bool(sponsor_is_isolated(observation)?))
        }
        FactId::ProtocolSecretUsed { .. } => Ok(SemanticValue::Bool(
            !observation.protocol_signers.is_empty(),
        )),
    }
}

const fn observed_side(side: TransactionSide) -> ObservedSide {
    match side {
        TransactionSide::Input => ObservedSide::Input,
        TransactionSide::Output => ObservedSide::Output,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn status(passed: bool, reason: RelationFailure) -> Result<RelationStatus, RealizationError> {
    Ok(if passed {
        RelationStatus::Passed
    } else {
        RelationStatus::Failed { reason }
    })
}

fn canonical_delta_policy_holds(
    expected: &BTreeSet<ExpectedCanonicalDelta>,
    observation: &OperationObservation,
) -> Result<bool, RealizationError> {
    let partition = &observation.canonical_partition;

    // 1. The active family set implied by the exact partition equals the
    //    manifest's expected family set.
    if &observed_active_families(partition) != expected {
        return Ok(false);
    }

    // 2. Each issuance issues its exact positive amount into a nonempty
    //    destination set.
    for issuance in &partition.issuances {
        if issuance.amount.is_zero() || issuance.destinations.is_empty() {
            return Ok(false);
        }

        let Some(destination_total) = sum_partition_side(
            observation,
            &issuance.destinations,
            issuance.asset,
            ObservedSide::Output,
        )?
        else {
            return Ok(false);
        };

        if destination_total != issuance.amount {
            return Ok(false);
        }
    }

    // 3. Each flow satisfies the exact source = destination + destruction
    //    equation and the movement-kind rule.
    for flow in &partition.flows {
        if !canonical_flow_holds(observation, flow)? {
            return Ok(false);
        }
    }

    // 4. Partition exactness: every observed canonical-value object is
    //    cited by exactly one witness on its side, and no witness cites
    //    a reference twice.
    Ok(partition_membership_holds(observation))
}

/// The active delta-family set implied by an observed partition.
fn observed_active_families(
    partition: &crate::ObservedCanonicalPartition,
) -> BTreeSet<ExpectedCanonicalDelta> {
    let mut families = BTreeSet::new();

    for issuance in &partition.issuances {
        families.insert(ExpectedCanonicalDelta {
            asset: issuance.asset,
            kind: DeltaKind::Issuance,
            destruction_tag: None,
        });
    }

    for flow in &partition.flows {
        if let Some(kind) = flow.movement_kind {
            families.insert(ExpectedCanonicalDelta {
                asset: flow.asset,
                kind,
                destruction_tag: None,
            });
        }

        for leg in &flow.destructions {
            families.insert(ExpectedCanonicalDelta {
                asset: flow.asset,
                kind: DeltaKind::Destruction,
                destruction_tag: Some(leg.tag),
            });
        }
    }

    families
}

/// Exact per-flow arithmetic: nonempty sources, `source = destination +
/// Σ destruction`, movement kind present iff the destination is
/// positive, and every destruction leg positive.
fn canonical_flow_holds(
    observation: &OperationObservation,
    flow: &crate::ObservedCanonicalFlow,
) -> Result<bool, RealizationError> {
    if flow.sources.is_empty() {
        return Ok(false);
    }

    let Some(source_total) =
        sum_partition_side(observation, &flow.sources, flow.asset, ObservedSide::Input)?
    else {
        return Ok(false);
    };

    let Some(destination_total) = sum_partition_side(
        observation,
        &flow.destinations,
        flow.asset,
        ObservedSide::Output,
    )?
    else {
        return Ok(false);
    };

    let destruction_total =
        ProtocolAmount::checked_sum(flow.destructions.iter().map(|leg| leg.amount))?;

    if source_total != destination_total.checked_add(destruction_total)? {
        return Ok(false);
    }

    if flow.movement_kind.is_some() == destination_total.is_zero() {
        return Ok(false);
    }

    if flow.destructions.iter().any(|leg| leg.amount.is_zero()) {
        return Ok(false);
    }

    Ok(true)
}

fn sum_partition_side(
    observation: &OperationObservation,
    references: &[ObservedObjectRef],
    asset: AssetId,
    side: ObservedSide,
) -> Result<Option<ProtocolAmount>, RealizationError> {
    let mut total = ProtocolAmount::ZERO;

    for reference in references {
        if reference.side != side {
            return Err(RealizationError::WrongObservedReferenceSide(*reference));
        }

        let object = observation
            .object(*reference)
            .ok_or(RealizationError::UnknownObservedObject(*reference))?;

        if object.asset != ObservedAsset::Declared(asset) {
            return Ok(None);
        }

        total = total.checked_add(object.value)?;
    }

    Ok(Some(total))
}

/// The exact-partition membership rule: every observed object of a
/// canonical value asset is cited by exactly one witness — a flow
/// source on the input side; a flow or issuance destination on the
/// output side — and no reference is cited by two witnesses. Membership
/// derives from the architecture object table, never from a
/// per-operation object-family map inside the evaluator.
fn partition_membership_holds(observation: &OperationObservation) -> bool {
    let partition = &observation.canonical_partition;
    let mut cited_sources = BTreeSet::new();
    let mut cited_destinations = BTreeSet::new();

    for flow in &partition.flows {
        for source in &flow.sources {
            if !cited_sources.insert(*source) {
                return false;
            }
        }

        for destination in &flow.destinations {
            if !cited_destinations.insert(*destination) {
                return false;
            }
        }
    }

    for issuance in &partition.issuances {
        for destination in &issuance.destinations {
            if !cited_destinations.insert(*destination) {
                return false;
            }
        }
    }

    let mut expected_sources = BTreeSet::new();
    let mut expected_destinations = BTreeSet::new();

    for observed in &observation.objects {
        let ObservedObjectKind::Declared(object) = observed.kind else {
            continue;
        };
        let canonical = architecture::ARCHITECTURE
            .object(object)
            .is_some_and(|spec| architecture::canonical_value_asset(spec.asset));

        if !canonical {
            continue;
        }

        match observed.reference.side {
            ObservedSide::Input => expected_sources.insert(observed.reference),
            ObservedSide::Output => expected_destinations.insert(observed.reference),
        };
    }

    cited_sources == expected_sources && cited_destinations == expected_destinations
}

fn declared_objects(
    observation: &OperationObservation,
    side: ObservedSide,
    object: ObjectId,
) -> impl Iterator<Item = &ObservedObject> {
    observation.objects.iter().filter(move |observed| {
        observed.reference.side == side && observed.kind == ObservedObjectKind::Declared(object)
    })
}

fn observed_object_shape_holds(object: ObjectId, observed: &ObservedObject) -> bool {
    match object {
        ObjectId::State
        | ObjectId::Pace
        | ObjectId::EntitlementAuthority
        | ObjectId::DistributionAuthority
        | ObjectId::DistributionControl => {
            observed.value == ProtocolAmount::ONE && observed.owner.is_none()
        }
        ObjectId::Resv => observed.owner.is_none(),
        ObjectId::ReceiptLive
        | ObjectId::ReceiptTimeLocked
        | ObjectId::DepositRequest
        | ObjectId::DepositEntitlement => !observed.value.is_zero() && observed.owner.is_some(),
        // Sponsor-value opacity (F2-006): ordinary sponsor L-BTC
        // is authenticated by asset, family, and owner — never by its
        // amount. A zero-valued PLAIN_LBTC member is an ordinary
        // sponsor object like any other; it cannot satisfy the anchor,
        // which is recognized by its own declared family below, not by
        // testing whether an ordinary output happens to be zero.
        ObjectId::PlainLbtc => observed.owner.is_some(),
        ObjectId::DistributionVault | ObjectId::Ash => {
            !observed.value.is_zero() && observed.owner.is_none()
        }
        ObjectId::CpfpAnchor => observed.value.is_zero() && observed.owner.is_none(),
    }
}

fn required_owners(
    observation: &OperationObservation,
    object: ObjectId,
) -> Option<BTreeSet<crate::OwnerId>> {
    declared_objects(observation, ObservedSide::Input, object)
        .map(|observed| observed.owner)
        .collect()
}

fn sum_selected_amounts(
    observation: &OperationObservation,
    side: ObservedSide,
    asset: AssetId,
    objects: &BTreeSet<ObjectId>,
) -> Result<ProtocolAmount, RealizationError> {
    ProtocolAmount::checked_sum(
        observation
            .objects
            .iter()
            .filter(|observed| {
                observed.reference.side == side
                    && observed.asset == ObservedAsset::Declared(asset)
                    && matches!(observed.kind, ObservedObjectKind::Declared(kind) if objects.contains(&kind))
            })
            .map(|observed| observed.value),
    )
}

fn sponsor_is_isolated(observation: &OperationObservation) -> Result<bool, RealizationError> {
    let mut used_sources = BTreeSet::new();
    let mut used_destinations = BTreeSet::new();

    for flow in &observation.open_flows {
        if flow.kind != architecture::OpenFlowKind::FeeSponsor {
            return Ok(false);
        }

        let Some(source_total) =
            flow_total(observation, flow, ObservedSide::Input, &mut used_sources)?
        else {
            return Ok(false);
        };
        let Some(destination_total) = flow_total(
            observation,
            flow,
            ObservedSide::Output,
            &mut used_destinations,
        )?
        else {
            return Ok(false);
        };

        if source_total != destination_total.checked_add(flow.fee)? {
            return Ok(false);
        }
    }

    // Opacity must not become omission: every ordinary sponsor member
    // — zero-valued ones included — is claimed exactly once by the
    // sponsor region. Membership is decided by the declared family,
    // never by the amount.
    for object in &observation.objects {
        if object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc) {
            let used = match object.reference.side {
                ObservedSide::Input => used_sources.contains(&object.reference),
                ObservedSide::Output => used_destinations.contains(&object.reference),
            };

            if !used {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn flow_total(
    observation: &OperationObservation,
    flow: &ObservedOpenFlow,
    side: ObservedSide,
    used: &mut BTreeSet<ObservedObjectRef>,
) -> Result<Option<ProtocolAmount>, RealizationError> {
    let refs = match side {
        ObservedSide::Input => &flow.sources,
        ObservedSide::Output => &flow.destinations,
    };
    let mut total = ProtocolAmount::ZERO;

    for reference in refs {
        if !used.insert(*reference) {
            return Ok(None);
        }

        let object = observation
            .object(*reference)
            .ok_or(RealizationError::UnknownObservedObject(*reference))?;

        if object.reference.side != side
            || object.kind != ObservedObjectKind::Declared(ObjectId::PlainLbtc)
            || object.asset != ObservedAsset::Declared(AssetId::Lbtc)
        {
            return Ok(None);
        }

        if side == ObservedSide::Input {
            let Some(owner) = object.owner else {
                return Ok(None);
            };

            if !observation.sponsor_signers.contains(&owner) {
                return Ok(None);
            }
        }

        total = total.checked_add(object.value)?;
    }

    Ok(Some(total))
}

fn root_policy_holds(
    expected: &BTreeMap<RootId, RootUse>,
    observation: &OperationObservation,
) -> bool {
    for root in RootId::ALL {
        let actual = observation
            .root_effects
            .iter()
            .filter(|effect| effect.root == *root)
            .map(|effect| effect.use_kind)
            .collect::<Vec<_>>();

        if actual.len() > 1 {
            return false;
        }

        let actual = actual.first().copied().unwrap_or(RootUse::Forbidden);
        let expected = expected.get(root).copied().unwrap_or(RootUse::Forbidden);

        if actual != expected {
            return false;
        }
    }

    true
}

fn projection_policy_holds(
    expected: &BTreeMap<ProjectionId, ProjectionRule>,
    observation: &OperationObservation,
) -> bool {
    ProjectionId::ALL.iter().all(|projection| {
        let rule = expected
            .get(projection)
            .copied()
            .unwrap_or(ProjectionRule::Forbidden);
        let present = observation.projections.contains(projection);

        match rule {
            ProjectionRule::Forbidden => !present,
            ProjectionRule::Optional => true,
            ProjectionRule::Required => present,
        }
    })
}
