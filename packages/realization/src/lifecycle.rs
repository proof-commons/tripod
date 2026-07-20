//! Representation-specific lifecycle requirements stored in a direct Petgraph graph.

use std::collections::BTreeMap;

use architecture::{ObjectId, OperationId};
use petgraph::{
    algo::{has_path_connecting, kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{RealizationError, RepresentationMode};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LifecycleNodeId {
    Representation {
        object: ObjectId,
        mode: RepresentationMode,
    },
    RequiredExit {
        object: ObjectId,
        operation: OperationId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleNode {
    pub id: LifecycleNodeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LifecycleEdge {
    RequiresExit,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifecycleDependencyDeclaration {
    pub source: LifecycleNodeId,
    pub target: LifecycleNodeId,
    pub edge: LifecycleEdge,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifecycleDependencyProjection {
    pub source: LifecycleNodeId,
    pub target: LifecycleNodeId,
    pub edge: LifecycleEdge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleGraphProjection {
    pub nodes: Vec<LifecycleNode>,
    pub edges: Vec<LifecycleDependencyProjection>,
}

#[allow(clippy::type_complexity)]
pub fn build_lifecycle_graph(
    nodes: impl IntoIterator<Item = LifecycleNode>,
    edges: impl IntoIterator<Item = LifecycleDependencyDeclaration>,
) -> Result<
    (
        DiGraph<LifecycleNode, LifecycleEdge, u32>,
        BTreeMap<LifecycleNodeId, NodeIndex<u32>>,
        Vec<LifecycleNodeId>,
    ),
    RealizationError,
> {
    let mut nodes = nodes.into_iter().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    for pair in nodes.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(RealizationError::DuplicateLifecycleNode(pair[0].id.clone()));
        }
    }

    let mut edges = edges.into_iter().collect::<Vec<_>>();
    edges.sort();

    let mut graph =
        DiGraph::<LifecycleNode, LifecycleEdge, u32>::with_capacity(nodes.len(), edges.len());
    let mut node_by_id = BTreeMap::new();

    for node_weight in nodes {
        let id = node_weight.id.clone();
        let node = graph.add_node(node_weight);
        node_by_id.insert(id, node);
    }

    for edge in edges {
        let source = node_by_id
            .get(&edge.source)
            .copied()
            .ok_or_else(|| RealizationError::UnknownLifecycleNode(edge.source.clone()))?;
        let target = node_by_id
            .get(&edge.target)
            .copied()
            .ok_or_else(|| RealizationError::UnknownLifecycleNode(edge.target.clone()))?;
        graph.add_edge(source, target, edge.edge);
    }

    let order = toposort(&graph, None)
        .map_err(|_cycle| RealizationError::LifecycleCycle {
            components: canonical_lifecycle_sccs(&graph),
        })?
        .into_iter()
        .map(|node| graph[node].id.clone())
        .collect();

    Ok((graph, node_by_id, order))
}

pub fn require_lifecycle_exit(
    graph: &DiGraph<LifecycleNode, LifecycleEdge, u32>,
    node_by_id: &BTreeMap<LifecycleNodeId, NodeIndex<u32>>,
    object: ObjectId,
    mode: RepresentationMode,
    exit: OperationId,
) -> Result<(), RealizationError> {
    let source = node_by_id
        .get(&LifecycleNodeId::Representation { object, mode })
        .copied()
        .ok_or(RealizationError::MissingLifecycleRepresentation { object, mode })?;
    let target = node_by_id
        .get(&LifecycleNodeId::RequiredExit {
            object,
            operation: exit,
        })
        .copied()
        .ok_or(RealizationError::MissingLifecycleExitNode { object, exit })?;

    if !has_path_connecting(graph, source, target, None) {
        return Err(RealizationError::MissingLifecyclePath { object, mode, exit });
    }

    Ok(())
}

#[must_use]
pub fn project_lifecycle_graph(
    graph: &DiGraph<LifecycleNode, LifecycleEdge, u32>,
) -> LifecycleGraphProjection {
    let mut nodes = graph.node_weights().cloned().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    let mut edges = graph
        .edge_references()
        .map(|edge| LifecycleDependencyProjection {
            source: graph[edge.source()].id.clone(),
            target: graph[edge.target()].id.clone(),
            edge: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();

    LifecycleGraphProjection { nodes, edges }
}

fn canonical_lifecycle_sccs(
    graph: &DiGraph<LifecycleNode, LifecycleEdge, u32>,
) -> Vec<Vec<LifecycleNodeId>> {
    let mut components = kosaraju_scc(graph)
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || component
                    .first()
                    .is_some_and(|node| graph.find_edge(*node, *node).is_some())
        })
        .map(|component| {
            let mut ids = component
                .into_iter()
                .map(|node| graph[node].id.clone())
                .collect::<Vec<_>>();
            ids.sort();
            ids
        })
        .collect::<Vec<_>>();
    components.sort();
    components
}
