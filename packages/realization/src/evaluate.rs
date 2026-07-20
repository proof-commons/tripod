//! Evaluation of typed semantic relations over primitive observations.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, ObjectId, ProjectionId, ProjectionRule, RootId, RootUse};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{
    CardinalityMaximum, Count, ObservedAsset, ObservedObject, ObservedObjectKind,
    ObservedObjectRef, ObservedOpenFlow, ObservedSide, OperationObservation, ProtocolAmount,
    RealizationError, Relation, RelationDeclaration, RelationEdge, RelationId,
};

/// Result class for one relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationStatus {
    Passed,
    Failed,
    DeclarationOnly,
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
        self.verdicts
            .iter()
            .all(|verdict| verdict.status != RelationStatus::Failed)
    }

    pub fn failed_relations(&self) -> impl Iterator<Item = &RelationId> {
        self.verdicts.iter().filter_map(|verdict| {
            (verdict.status == RelationStatus::Failed).then_some(&verdict.relation)
        })
    }
}

/// Evaluate all relations owned by one operation.
pub fn evaluate_operation(
    relation_graph: &DiGraph<RelationDeclaration, RelationEdge, u32>,
    relation_node_by_id: &BTreeMap<RelationId, NodeIndex<u32>>,
    relation_evaluation_order: &[RelationId],
    observation: &OperationObservation,
) -> Result<ConformanceReport, RealizationError> {
    let mut verdicts = Vec::new();

    for relation_id in relation_evaluation_order {
        let node = relation_node_by_id
            .get(relation_id)
            .copied()
            .ok_or_else(|| RealizationError::UnknownRelation(relation_id.clone()))?;
        let declaration = &relation_graph[node];

        if declaration.id.operation() != observation.operation {
            continue;
        }

        verdicts.push(RelationVerdict {
            relation: declaration.id.clone(),
            status: evaluate_relation(&declaration.relation, observation)?,
        });
    }

    verdicts.sort_by(|left, right| left.relation.cmp(&right.relation));

    Ok(ConformanceReport {
        operation: observation.operation,
        verdicts,
    })
}

fn evaluate_relation(
    relation: &Relation,
    observation: &OperationObservation,
) -> Result<RelationStatus, RealizationError> {
    let passed = match relation {
        Relation::Cardinality {
            side,
            object,
            minimum,
            maximum,
        } => {
            let count = Count::new(u64::try_from(declared_objects(observation, *side, *object).count()).map_err(|_| RealizationError::CountOverflow)?);
            let maximum = match maximum {
                CardinalityMaximum::Exact(value) => *value,
                CardinalityMaximum::Bound(bound) => observation
                    .bounds
                    .get(bound)
                    .copied()
                    .ok_or(RealizationError::MissingBoundValue(*bound))?,
            };

            count >= *minimum && count <= maximum
        }
        Relation::AllowedObjectFamilies { side, allowed } => observation
            .objects
            .iter()
            .filter(|object| object.reference.side == *side)
            .all(|object| matches!(object.kind, ObservedObjectKind::Declared(kind) if allowed.contains(&kind))),
        Relation::Recognition {
            side,
            object,
            asset,
        } => declared_objects(observation, *side, *object)
            .all(|observed| observed.asset == ObservedAsset::Declared(*asset)),
        Relation::AmountConservation {
            asset,
            input_objects,
            output_objects,
        } => {
            let input = sum_selected_amounts(observation, ObservedSide::Input, *asset, input_objects)?;
            let output = sum_selected_amounts(observation, ObservedSide::Output, *asset, output_objects)?;

            input == output
        }
        Relation::OwnerAuthorization { object } => {
            let required = declared_objects(observation, ObservedSide::Input, *object)
                .filter_map(|observed| observed.owner)
                .collect::<BTreeSet<_>>();

            required.is_subset(&observation.protocol_signers)
        }
        Relation::PermissionlessAuthorization => observation.protocol_signers.is_empty(),
        Relation::SponsorIsolation => sponsor_is_isolated(observation)?,
        Relation::RootPolicy { expected } => root_policy_holds(expected, observation),
        Relation::ProjectionPolicy { expected } => projection_policy_holds(expected, observation),
        Relation::PublicConstructibility => observation.protocol_signers.is_empty()
            && observation.objects.iter().all(|object| {
                object.reference.side != ObservedSide::Input
                    || object.owner.is_none()
                    || object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc)
            }),
        Relation::Representation { object, allowed } => observation.objects.iter().all(|observed| {
            observed.kind != ObservedObjectKind::Declared(*object)
                || allowed.contains(&observed.representation)
        }),
        Relation::LifecycleExit { .. } | Relation::ExpressionPredicate { .. } => {
            return Ok(RelationStatus::DeclarationOnly);
        }
    };

    Ok(if passed {
        RelationStatus::Passed
    } else {
        RelationStatus::Failed
    })
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
