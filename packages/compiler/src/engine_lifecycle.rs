//! Shared representation-choice and lifecycle traversal over owner records.

use crate::{
    engine_records::{DependencyKind, OperationKey},
    lifecycle::LifecycleExitStatus,
};
use petgraph::visit::EdgeRef;
use petgraph::{
    algo::has_path_connecting,
    graph::{DiGraph, NodeIndex},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectKey {
    pub namespace: u16,
    pub code: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModeKey {
    pub namespace: u16,
    pub code: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LifecycleNodeKey {
    Representation {
        object: ObjectKey,
        mode: ModeKey,
    },
    RequiredExit {
        object: ObjectKey,
        operation: OperationKey,
    },
}

impl LifecycleNodeKey {
    const fn object(&self) -> ObjectKey {
        match self {
            Self::Representation { object, .. } | Self::RequiredExit { object, .. } => *object,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceRecord {
    pub operation: OperationKey,
    pub object: ObjectKey,
    pub candidates: Vec<ModeKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifecycleDependency {
    pub source: LifecycleNodeKey,
    pub target: LifecycleNodeKey,
    pub kind: DependencyKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleRecords {
    pub scope: Vec<OperationKey>,
    pub choices: Vec<ChoiceRecord>,
    pub required_choices: Vec<(OperationKey, ObjectKey)>,
    pub nodes: Vec<LifecycleNodeKey>,
    pub edges: Vec<LifecycleDependency>,
}

impl LifecycleRecords {
    pub const fn empty(scope: Vec<OperationKey>) -> Self {
        Self {
            scope,
            choices: Vec::new(),
            required_choices: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifecycleRequirementRecord {
    pub object: ObjectKey,
    pub representation: ModeKey,
    pub exit: OperationKey,
    pub status: LifecycleExitStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleRecordError {
    MissingRepresentationChoice {
        operation: OperationKey,
        object: ObjectKey,
    },
    MissingLifecyclePath {
        object: ObjectKey,
        representation: ModeKey,
        exit: OperationKey,
    },
}

#[derive(Debug)]
pub struct LifecycleAnalysis {
    pub graph: DiGraph<LifecycleNodeKey, DependencyKind, u32>,
    pub node_by_id: BTreeMap<LifecycleNodeKey, NodeIndex<u32>>,
    pub choices: Vec<ChoiceRecord>,
    pub requirements: Vec<LifecycleRequirementRecord>,
}

/// Typed stage projection, including a genuine empty result. Like the existing
/// whole-program projection this is exercised by the probe before publication.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct LifecycleProjection {
    pub nodes: Vec<LifecycleNodeKey>,
    pub edges: Vec<LifecycleDependency>,
    pub choices: Vec<ChoiceRecord>,
    pub requirements: Vec<LifecycleRequirementRecord>,
}

impl LifecycleAnalysis {
    #[allow(dead_code)]
    pub fn project(&self) -> LifecycleProjection {
        let mut nodes = self.graph.node_weights().cloned().collect::<Vec<_>>();
        nodes.sort();
        let mut edges = self
            .graph
            .edge_references()
            .map(|edge| LifecycleDependency {
                source: self.graph[edge.source()].clone(),
                target: self.graph[edge.target()].clone(),
                kind: *edge.weight(),
            })
            .collect::<Vec<_>>();
        edges.sort();
        LifecycleProjection {
            nodes,
            edges,
            choices: self.choices.clone(),
            requirements: self.requirements.clone(),
        }
    }
}

pub fn analyze_lifecycle(
    records: &LifecycleRecords,
) -> Result<LifecycleAnalysis, LifecycleRecordError> {
    let started = tracing::enabled!(target: "compiler::engine_lifecycle", tracing::Level::DEBUG)
        .then(std::time::Instant::now);
    let span = tracing::debug_span!(target: "compiler::engine_lifecycle", "analyze_lifecycle",
        choice_count = crate::instrumentation::count(records.choices.len()), node_count = crate::instrumentation::count(records.nodes.len()), edge_count = crate::instrumentation::count(records.edges.len()));
    let _entered = span.enter();
    let result = analyze_core(records);
    if let Some(started) = started {
        tracing::debug!(target: "compiler::engine_lifecycle", outcome = if result.is_ok() { "completed" } else { "refused" }, elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX), "lifecycle records analyzed");
    }
    result
}

fn analyze_core(records: &LifecycleRecords) -> Result<LifecycleAnalysis, LifecycleRecordError> {
    let mut choices = records.choices.clone();
    choices.sort_by_key(|choice| (choice.operation, choice.object));
    let choice_objects = choices
        .iter()
        .map(|choice| choice.object)
        .collect::<BTreeSet<_>>();
    for (operation, object) in &records.required_choices {
        if !choice_objects.contains(object) {
            return Err(LifecycleRecordError::MissingRepresentationChoice {
                operation: *operation,
                object: *object,
            });
        }
    }
    let nodes = records
        .nodes
        .iter()
        .filter(|node| choice_objects.contains(&node.object()))
        .cloned()
        .collect::<Vec<_>>();
    let mut graph = DiGraph::with_capacity(nodes.len(), records.edges.len());
    let mut node_by_id = BTreeMap::new();
    for id in nodes {
        let node = graph.add_node(id.clone());
        node_by_id.insert(id, node);
    }
    for edge in &records.edges {
        let (Some(source), Some(target)) = (
            node_by_id.get(&edge.source).copied(),
            node_by_id.get(&edge.target).copied(),
        ) else {
            continue;
        };
        graph.add_edge(source, target, edge.kind);
    }
    let mut requirements = Vec::new();
    for (id, node) in &node_by_id {
        let LifecycleNodeKey::Representation { object, mode } = id else {
            continue;
        };
        for (exit_id, exit_node) in &node_by_id {
            let LifecycleNodeKey::RequiredExit {
                object: exit_object,
                operation,
            } = exit_id
            else {
                continue;
            };
            if exit_object != object {
                continue;
            }
            if !has_path_connecting(&graph, *node, *exit_node, None) {
                return Err(LifecycleRecordError::MissingLifecyclePath {
                    object: *object,
                    representation: *mode,
                    exit: *operation,
                });
            }
            let status = if records.scope.contains(operation) {
                LifecycleExitStatus::AvailableInCompilerScope
            } else {
                LifecycleExitStatus::DeclaredOutsideCompilerScope
            };
            requirements.push(LifecycleRequirementRecord {
                object: *object,
                representation: *mode,
                exit: *operation,
                status,
            });
        }
    }
    requirements.sort();
    Ok(LifecycleAnalysis {
        graph,
        node_by_id,
        choices,
        requirements,
    })
}
