//! Evaluation of typed semantic relations over primitive observations.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, DeltaKind, ObjectId, ProjectionId, ProjectionRule, RootId, RootUse};
use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef as _,
};

use crate::{
    CardinalityMaximum, ConstructibilityClass, Count, ExpectedCanonicalDelta, ObservedAsset,
    ObservedObject, ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow, ObservedSide,
    OperationObservation, ProtocolAmount, RealizationError, Relation, RelationDeclaration,
    RelationEdge, RelationId,
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
    RootPolicy,
    ProjectionPolicy,
    Constructibility,
    Representation,
    CanonicalDeltaPolicy,
    OpenFlowPolicy,
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
pub fn evaluate_operation(
    relation_graph: &DiGraph<RelationDeclaration, RelationEdge, u32>,
    relation_node_by_id: &BTreeMap<RelationId, NodeIndex<u32>>,
    relation_evaluation_order: &[RelationId],
    observation: &OperationObservation,
) -> Result<ConformanceReport, RealizationError> {
    let observation = observation.clone().validate_and_normalize()?;
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
            evaluate_relation(&declaration.relation, &observation)?
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
        Relation::LifecycleExit { .. } | Relation::ExpressionPredicate { .. } => {
            Ok(RelationStatus::StaticallyValidated)
        }
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
    let actual = observation
        .canonical_deltas
        .iter()
        .map(|delta| ExpectedCanonicalDelta {
            asset: delta.asset,
            kind: delta.kind,
            destruction_tag: delta.destruction_tag,
        })
        .collect::<BTreeSet<_>>();

    if &actual != expected {
        return Ok(false);
    }

    for delta in &observation.canonical_deltas {
        let Some(source_total) = sum_delta_side(
            observation,
            &delta.sources,
            delta.asset,
            ObservedSide::Input,
        )?
        else {
            return Ok(false);
        };
        let Some(destination_total) = sum_delta_side(
            observation,
            &delta.destinations,
            delta.asset,
            ObservedSide::Output,
        )?
        else {
            return Ok(false);
        };

        let amount_is_consistent = match delta.kind {
            DeltaKind::Lateral | DeltaKind::OwnerlessLateral => {
                source_total == delta.amount && destination_total == delta.amount
            }
            DeltaKind::Issuance => delta.sources.is_empty() && destination_total == delta.amount,
            DeltaKind::Destruction => {
                destination_total.is_zero()
                    && source_total == delta.amount
                    && delta.destruction_tag.is_some()
            }
        };

        if !amount_is_consistent {
            return Ok(false);
        }
    }

    Ok(expected.iter().all(|delta| match delta.kind {
        DeltaKind::OwnerlessLateral => canonical_delta_membership_holds(
            observation,
            ObjectId::Ash,
            DeltaKind::OwnerlessLateral,
        ),
        DeltaKind::Lateral => {
            canonical_delta_membership_holds(observation, ObjectId::ReceiptLive, DeltaKind::Lateral)
        }
        _ => true,
    }))
}

fn sum_delta_side(
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

fn canonical_delta_membership_holds(
    observation: &OperationObservation,
    object: ObjectId,
    expected_kind: DeltaKind,
) -> bool {
    let protocol_refs = observation
        .objects
        .iter()
        .filter(|observed| observed.kind == ObservedObjectKind::Declared(object))
        .map(|observed| observed.reference)
        .collect::<BTreeSet<_>>();
    let source_refs = observation
        .canonical_deltas
        .iter()
        .filter(|delta| delta.kind == expected_kind)
        .flat_map(|delta| delta.sources.iter().copied())
        .collect::<BTreeSet<_>>();
    let destination_refs = observation
        .canonical_deltas
        .iter()
        .filter(|delta| delta.kind == expected_kind)
        .flat_map(|delta| delta.destinations.iter().copied())
        .collect::<BTreeSet<_>>();
    let expected_sources = protocol_refs
        .iter()
        .copied()
        .filter(|reference| reference.side == ObservedSide::Input)
        .collect::<BTreeSet<_>>();
    let expected_destinations = protocol_refs
        .iter()
        .copied()
        .filter(|reference| reference.side == ObservedSide::Output)
        .collect::<BTreeSet<_>>();

    source_refs == expected_sources && destination_refs == expected_destinations
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
        ObjectId::ReceiptLive
        | ObjectId::ReceiptTimeLocked
        | ObjectId::DepositRequest
        | ObjectId::DepositEntitlement
        | ObjectId::PlainLbtc => observed.owner.is_some(),
        ObjectId::State
        | ObjectId::Resv
        | ObjectId::Pace
        | ObjectId::EntitlementAuthority
        | ObjectId::DistributionAuthority
        | ObjectId::DistributionControl
        | ObjectId::DistributionVault
        | ObjectId::Ash
        | ObjectId::CpfpAnchor => true,
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

    for object in &observation.objects {
        if object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc)
            && !object.value.is_zero()
        {
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
