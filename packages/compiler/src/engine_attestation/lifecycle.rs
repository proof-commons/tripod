//! Attestation lifecycle vocabulary lowering and compatibility projection.

use crate::{
    CompileError,
    engine_lifecycle::{
        self, ChoiceRecord, LifecycleDependency, LifecycleNodeKey, LifecycleRecordError,
        LifecycleRecords, ModeKey, ObjectKey,
    },
    engine_records::{DependencyKind, OperationKey},
    lifecycle::{
        CompilerLifecycleAnalysis, LifecycleRequirement, RepresentationChoice,
        RepresentationChoiceId,
    },
};
use architecture::{ObjectId, OperationId};
use realization::{
    LifecycleEdge, LifecycleGraphProjection, LifecycleNode, LifecycleNodeId, Relation,
    RelationDeclaration, RepresentationMode,
};
use std::collections::BTreeMap;

pub struct LoweredLifecycle {
    pub records: LifecycleRecords,
    objects: BTreeMap<ObjectKey, ObjectId>,
    operations: BTreeMap<OperationKey, OperationId>,
    modes: BTreeMap<ModeKey, RepresentationMode>,
    nodes: BTreeMap<LifecycleNodeKey, LifecycleNodeId>,
}

pub fn lower<'a>(
    scope: &[OperationId],
    declarations: impl Iterator<Item = &'a RelationDeclaration>,
    projection: &LifecycleGraphProjection,
) -> LoweredLifecycle {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_lifecycle", source_node_count = crate::instrumentation::count(projection.nodes.len()), source_edge_count = crate::instrumentation::count(projection.edges.len()));
    let _entered = span.enter();
    let declarations = declarations.collect::<Vec<_>>();
    let mut records =
        LifecycleRecords::empty(scope.iter().copied().map(super::operation_key).collect());
    for declaration in &declarations {
        if let Relation::Representation { object, allowed } = &declaration.relation {
            let mut candidates = allowed.iter().copied().map(mode_key).collect::<Vec<_>>();
            candidates.sort();
            records.choices.push(ChoiceRecord {
                operation: super::operation_key(declaration.id.operation()),
                object: object_key(*object),
                candidates,
            });
        }
    }
    for declaration in &declarations {
        if let Relation::LifecycleExit { object, .. } = &declaration.relation {
            records.required_choices.push((
                super::operation_key(declaration.id.operation()),
                object_key(*object),
            ));
        }
    }
    records.nodes = projection
        .nodes
        .iter()
        .map(|node| node_key(&node.id))
        .collect();
    records.edges = projection
        .edges
        .iter()
        .map(|edge| LifecycleDependency {
            source: node_key(&edge.source),
            target: node_key(&edge.target),
            kind: edge_key(edge.edge),
        })
        .collect();
    tracing::debug!(target: "compiler::engine_attestation", outcome = "completed", choice_count = crate::instrumentation::count(records.choices.len()), "lifecycle records lowered");
    LoweredLifecycle {
        records,
        objects: ObjectId::ALL
            .iter()
            .copied()
            .map(|object| (object_key(object), object))
            .collect(),
        operations: OperationId::ALL
            .iter()
            .copied()
            .map(|operation| (super::operation_key(operation), operation))
            .collect(),
        modes: [
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
            RepresentationMode::PublicCommitted,
        ]
        .into_iter()
        .map(|mode| (mode_key(mode), mode))
        .collect(),
        nodes: projection
            .nodes
            .iter()
            .map(|node| (node_key(&node.id), node.id.clone()))
            .collect(),
    }
}

impl LoweredLifecycle {
    pub fn analyze(&self) -> Result<CompilerLifecycleAnalysis, CompileError> {
        self.compatibility_view(engine_lifecycle::analyze_lifecycle(&self.records))
    }

    pub fn compatibility_view(
        &self,
        result: Result<engine_lifecycle::LifecycleAnalysis, LifecycleRecordError>,
    ) -> Result<CompilerLifecycleAnalysis, CompileError> {
        let analysis = result.map_err(|error| match error {
            LifecycleRecordError::MissingRepresentationChoice { operation, object } => {
                CompileError::MissingRepresentationChoice {
                    operation: self.operations[&operation],
                    object: self.objects[&object],
                }
            }
            LifecycleRecordError::MissingLifecyclePath {
                object,
                representation,
                exit,
            } => CompileError::MissingLifecyclePath {
                object: self.objects[&object],
                representation: self.modes[&representation],
                exit: self.operations[&exit],
            },
        })?;
        Ok(CompilerLifecycleAnalysis {
            graph: analysis.graph.map(
                |_, id| LifecycleNode {
                    id: self.nodes[id].clone(),
                },
                // This fixed owner's edge vocabulary has one member. The
                // shared traversal carries its encoded identity unchanged;
                // this compatibility constructor never serves a keyed owner.
                |_, _| LifecycleEdge::RequiresExit,
            ),
            node_by_id: analysis
                .node_by_id
                .into_iter()
                .map(|(id, index)| (self.nodes[&id].clone(), index))
                .collect(),
            choices: analysis
                .choices
                .into_iter()
                .map(|choice| RepresentationChoice {
                    id: RepresentationChoiceId {
                        operation: self.operations[&choice.operation],
                        object: self.objects[&choice.object],
                    },
                    candidates: choice
                        .candidates
                        .into_iter()
                        .map(|mode| self.modes[&mode])
                        .collect(),
                })
                .collect(),
            requirements: analysis
                .requirements
                .into_iter()
                .map(|requirement| LifecycleRequirement {
                    object: self.objects[&requirement.object],
                    representation: self.modes[&requirement.representation],
                    exit: self.operations[&requirement.exit],
                    status: requirement.status,
                })
                .collect(),
        })
    }
}

const fn object_key(object: ObjectId) -> ObjectKey {
    ObjectKey {
        namespace: super::ATTESTATION,
        code: object.code(),
    }
}

const fn mode_key(mode: RepresentationMode) -> ModeKey {
    let code = match mode {
        RepresentationMode::Explicit => 0,
        RepresentationMode::PrivateCommitted => 1,
        RepresentationMode::PublicCommitted => 2,
    };
    ModeKey {
        namespace: super::ATTESTATION,
        code,
    }
}

const fn node_key(id: &LifecycleNodeId) -> LifecycleNodeKey {
    match id {
        LifecycleNodeId::Representation { object, mode } => LifecycleNodeKey::Representation {
            object: object_key(*object),
            mode: mode_key(*mode),
        },
        LifecycleNodeId::RequiredExit { object, operation } => LifecycleNodeKey::RequiredExit {
            object: object_key(*object),
            operation: super::operation_key(*operation),
        },
    }
}

const fn edge_key(edge: LifecycleEdge) -> DependencyKind {
    let code = match edge {
        LifecycleEdge::RequiresExit => 0,
    };
    DependencyKind {
        namespace: super::ATTESTATION,
        code,
    }
}
