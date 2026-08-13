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
    ObservedOpenFlow, ObservedSide, ObservedValue, OperationObservation, ProtocolAmount,
    RealizationError, Relation, RelationDeclaration, RelationEdge, RelationId, SemanticValue,
    TransactionSide,
    expression::{DependencyEdge, ExpressionDeclaration, FactValues},
};

/// Typed premise the realization evaluator cannot establish itself.
///
/// Carrying the requirement in the report keeps the missing proof
/// visible: a runtime pass over the sponsor-erased observation is not
/// evidence that the substrate accepted whole-transaction value
/// conservation. The model kernel or the target discharges it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExternalEvidenceRequirement {
    SubstrateConservation {
        operation: architecture::OperationId,
        asset: AssetId,
    },
}

/// Result class for one realization relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationStatus {
    Passed,
    Failed {
        reason: RelationFailure,
    },
    Blocked {
        prerequisites: Vec<RelationId>,
    },
    StaticallyValidated,
    /// The relation is well-formed but only external evidence can
    /// discharge it; it never becomes `Passed` at this boundary.
    EvidenceRequired {
        requirement: ExternalEvidenceRequirement,
    },
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
    /// No semantic relation failed or was blocked.
    ///
    /// This does **not** imply external evidence completion: a report
    /// can be conformant while still carrying `EvidenceRequired`
    /// relations whose proof obligation belongs to the model kernel or
    /// the target. Use [`Self::is_evidence_complete`] for that claim.
    #[must_use]
    pub fn is_conformant(&self) -> bool {
        self.verdicts.iter().all(|verdict| {
            matches!(
                verdict.status,
                RelationStatus::Passed
                    | RelationStatus::StaticallyValidated
                    | RelationStatus::EvidenceRequired { .. }
            )
        })
    }

    /// True when any relation failed or was blocked.
    #[must_use]
    pub fn has_semantic_failure(&self) -> bool {
        self.verdicts.iter().any(|verdict| {
            matches!(
                verdict.status,
                RelationStatus::Failed { .. } | RelationStatus::Blocked { .. }
            )
        })
    }

    /// True when no relation still requires external evidence.
    #[must_use]
    pub fn is_evidence_complete(&self) -> bool {
        self.required_external_evidence().next().is_none()
    }

    /// Every external-evidence requirement this report still carries.
    pub fn required_external_evidence(&self) -> impl Iterator<Item = &ExternalEvidenceRequirement> {
        self.verdicts
            .iter()
            .filter_map(|verdict| match &verdict.status {
                RelationStatus::EvidenceRequired { requirement } => Some(requirement),
                _ => None,
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

                // Everything except an established prerequisite blocks
                // its dependents — including an undischarged external
                // premise: nothing downstream may build on evidence
                // this boundary has not seen.
                match status_by_relation.get(prerequisite) {
                    Some(RelationStatus::Passed | RelationStatus::StaticallyValidated) => None,
                    _ => Some(prerequisite.clone()),
                }
            })
            .collect::<Vec<_>>();
        blocking.sort();
        blocking.dedup();

        let status = if blocking.is_empty() {
            evaluate_relation(
                &declaration.id,
                &declaration.relation,
                &observation,
                evaluated.as_ref(),
            )?
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
    id: &RelationId,
    relation: &Relation,
    observation: &OperationObservation,
    evaluated: Option<&EvaluatedExpressions>,
) -> Result<RelationStatus, RealizationError> {
    match relation {
        Relation::SubstrateConservation { asset } => Ok(RelationStatus::EvidenceRequired {
            requirement: ExternalEvidenceRequirement::SubstrateConservation {
                operation: id.operation(),
                asset: *asset,
            },
        }),
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
                    && observed_object_shape_holds(observation, *object, observed)
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
            // The opacity guard forbids a PLAIN_LBTC family amount
            // from being declared at all, so every object reaching
            // here must carry a readable protocol amount. An erased
            // one is a malformed observation, never a zero.
            let mut amounts = Vec::new();
            for observed in declared_objects(observation, observed_side(*side), *object) {
                let Some(amount) = observed.value.protocol() else {
                    return Err(RealizationError::SponsorValueRead);
                };
                amounts.push(amount);
            }
            let total = ProtocolAmount::checked_sum(amounts)?;

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
                            && observed_object_shape_holds(observation, *object, observed)
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

        // A sponsor-erased value cannot participate in a protocol
        // total. Reaching one here means a sponsor object was routed
        // into a protocol flow, which is a defect rather than a zero.
        let Some(amount) = object.value.protocol() else {
            return Ok(None);
        };
        total = total.checked_add(amount)?;
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

/// Whether one observed object has the shape its declared family
/// requires.
///
/// Ordinary L-BTC is the one family whose required shape depends on
/// *where* the reference sits (S2-01): the fee-sponsor region carries
/// no readable amount, and a protocol open flow carries one. The whole
/// observation is therefore in scope, because the region is a property
/// of the open flows rather than of the object.
fn observed_object_shape_holds(
    observation: &OperationObservation,
    object: ObjectId,
    observed: &ObservedObject,
) -> bool {
    match object {
        ObjectId::State
        | ObjectId::Pace
        | ObjectId::EntitlementAuthority
        | ObjectId::DistributionAuthority
        | ObjectId::DistributionControl => {
            observed.value.is(ProtocolAmount::ONE) && observed.owner.is_none()
        }
        ObjectId::Resv => observed.owner.is_none(),
        ObjectId::ReceiptLive
        | ObjectId::ReceiptTimeLocked
        | ObjectId::DepositRequest
        | ObjectId::DepositEntitlement => {
            observed
                .value
                .protocol()
                .is_some_and(|amount| !amount.is_zero())
                && observed.owner.is_some()
        }
        // Sponsor-value opacity (F2-006, S3): a sponsor-region
        // L-BTC member is authenticated by asset, family, owner, and
        // exact membership — never by its amount, which the projection
        // does not carry at all. Requiring erasure here makes an
        // observation that smuggles a sponsor amount fail recognition
        // rather than pass unnoticed. A zero-valued sponsor member is
        // an ordinary member like any other; it cannot satisfy the
        // anchor, which is recognized by its own declared family
        // below, not by testing an ordinary output for zero.
        //
        // In a protocol open flow the same family is protocol value
        // (S2-01) — request funding, refund, admission reward,
        // redemption payout — and its owning relation has to be able
        // to read it, so an erased amount there is the malformed case
        // instead.
        //
        // A reference claimed by no flow at all is neither, and this
        // relation says nothing about its value: exact membership is
        // sponsor isolation's obligation, and an unclaimed ordinary
        // L-BTC member fails there. Duplicating the verdict here would
        // only move the reason.
        ObjectId::PlainLbtc => {
            observed.owner.is_some()
                && match observation.flow_role(observed.reference) {
                    crate::ObservedFlowRole::Sponsor => {
                        observed.value == ObservedValue::SponsorOpaque
                    }
                    crate::ObservedFlowRole::Protocol(_) => observed.value.protocol().is_some(),
                    crate::ObservedFlowRole::Unclaimed => true,
                }
        }
        ObjectId::DistributionVault | ObjectId::Ash => {
            observed
                .value
                .protocol()
                .is_some_and(|amount| !amount.is_zero())
                && observed.owner.is_none()
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
    // Selection is by declared protocol object kind, so every match
    // carries a readable amount; a sponsor-erased value here means a
    // reference from the fee-sponsor region was selected as protocol
    // value. That is a malformed observation, never a zero — reading
    // an erased sponsor amount as zero is exactly the read the
    // projection removes.
    let mut amounts = Vec::new();
    for observed in observation.objects.iter().filter(|observed| {
        observed.reference.side == side
            && observed.asset == ObservedAsset::Declared(asset)
            && matches!(observed.kind, ObservedObjectKind::Declared(kind) if objects.contains(&kind))
    }) {
        let Some(amount) = observed.value.protocol() else {
            return Err(RealizationError::SponsorValueRead);
        };
        amounts.push(amount);
    }
    ProtocolAmount::checked_sum(amounts)
}

fn sponsor_is_isolated(observation: &OperationObservation) -> Result<bool, RealizationError> {
    let mut used_sources = BTreeSet::new();
    let mut used_destinations = BTreeSet::new();

    // The sponsor region is exactly the fee-sponsor flows (S2-01).
    // Which flow kinds an operation may declare at all is
    // `OpenFlowPolicy`'s judgement, and the relation graph orders it
    // before this one, so a protocol flow observed where the manifest
    // forbids it is already rejected under its own name rather than
    // reported as a sponsor-isolation failure.
    for flow in observation
        .open_flows
        .iter()
        .filter(|flow| flow.kind == architecture::OpenFlowKind::FeeSponsor)
    {
        // Role structure only (S3). Sponsor conservation is the
        // substrate's: Elements validates that every transaction's
        // inputs and outputs balance, so re-deriving that here would
        // duplicate the base layer's own job while reading exactly the
        // amounts sponsor erasure removes from the protocol read-set.
        // This layer concerns itself only with the proofs that bear on
        // its own security — and the trap box is explicit that a
        // sponsor value read detects nothing the exact protocol
        // relations do not already pin.
        if !flow_role_is_exact(observation, flow, ObservedSide::Input, &mut used_sources)? {
            return Ok(false);
        }
        if !flow_role_is_exact(
            observation,
            flow,
            ObservedSide::Output,
            &mut used_destinations,
        )? {
            return Ok(false);
        }
    }

    // Opacity must not become omission: every ordinary L-BTC member —
    // zero-valued ones included — is claimed exactly once, by the
    // sponsor region or by a protocol open flow. Membership is decided
    // by the declared family, never by the amount; which region claims
    // it is decided by the flow, never by the family (S2-01).
    for object in &observation.objects {
        if object.kind != ObservedObjectKind::Declared(ObjectId::PlainLbtc) {
            continue;
        }

        let in_sponsor_region = match object.reference.side {
            ObservedSide::Input => used_sources.contains(&object.reference),
            ObservedSide::Output => used_destinations.contains(&object.reference),
        };

        if !in_sponsor_region
            && observation.flow_role(object.reference) == crate::ObservedFlowRole::Unclaimed
        {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Validate one side of a fee-sponsor flow as *role structure*: exact
/// membership, source and destination uniqueness, declared family and
/// asset, erasure, and owner authorization on the input side.
///
/// Deliberately value-blind. Sponsor conservation belongs to the
/// substrate — Elements validates that a transaction balances — so
/// this layer authenticates only what bears on its own security. The
/// one thing checked about the value is that there is none: a member
/// of this region carrying a readable amount is a projection that
/// failed to erase, and rejecting it reads no amount.
fn flow_role_is_exact(
    observation: &OperationObservation,
    flow: &ObservedOpenFlow,
    side: ObservedSide,
    used: &mut BTreeSet<ObservedObjectRef>,
) -> Result<bool, RealizationError> {
    let refs = match side {
        ObservedSide::Input => &flow.sources,
        ObservedSide::Output => &flow.destinations,
    };

    for reference in refs {
        if !used.insert(*reference) {
            return Ok(false);
        }

        let object = observation
            .object(*reference)
            .ok_or(RealizationError::UnknownObservedObject(*reference))?;

        if object.reference.side != side
            || object.kind != ObservedObjectKind::Declared(ObjectId::PlainLbtc)
            || object.asset != ObservedAsset::Declared(AssetId::Lbtc)
            || object.value != ObservedValue::SponsorOpaque
        {
            return Ok(false);
        }

        if side == ObservedSide::Input {
            let Some(owner) = object.owner else {
                return Ok(false);
            };

            if !observation.sponsor_signers.contains(&owner) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Check each root's *actual* effect against its declared policy.
///
/// The policy is a set of admitted effects, not a value to compare
/// against: `Succession` admits a succession, `SuccessionOrTermination`
/// admits either, and `Forbidden` admits none. A policy that is not
/// `Forbidden` still *requires* an effect, so a missing succession
/// fails exactly as it did before.
pub(crate) fn root_policy_holds(
    expected: &BTreeMap<RootId, RootUse>,
    observation: &OperationObservation,
) -> bool {
    for root in RootId::ALL {
        let actual = observation
            .root_effects
            .iter()
            .filter(|effect| effect.root == *root)
            .map(|effect| effect.effect)
            .collect::<Vec<_>>();

        if actual.len() > 1 {
            return false;
        }

        let policy = expected.get(root).copied().unwrap_or(RootUse::Forbidden);
        let holds = actual.first().map_or_else(
            || policy == RootUse::Forbidden,
            |effect| effect.permitted_by(policy),
        );

        if !holds {
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
