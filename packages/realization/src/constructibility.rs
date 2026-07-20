//! Typed constructibility requirements stored in a direct Petgraph graph.

use std::collections::BTreeMap;

use architecture::{ObjectId, OperationId};
use petgraph::{
    Direction,
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{FactId, RealizationError};

/// Availability class of one fact or witness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AvailabilityClass {
    Public,
    InputOwners { object: ObjectId },
    Operator,
    SponsorLocal,
}

/// Witness role distinct from target witness encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessRole {
    ProtocolOwnerAuthorization,
    OperatorAuthorization,
    SponsorAuthorization,
}

/// Stable identity of one constructibility-graph node.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstructibilityNodeId {
    Operation(OperationId),
    Fact {
        operation: OperationId,
        fact: FactId,
        availability: AvailabilityClass,
    },
    Witness {
        operation: OperationId,
        role: WitnessRole,
        availability: AvailabilityClass,
    },
}

/// Direct Petgraph node weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructibilityNode {
    pub id: ConstructibilityNodeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequirementStrength {
    Required,
    Optional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstructibilityEdgeRole {
    RequiredFact,
    RequiredWitness,
    SponsorOnly,
}

/// Direct Petgraph edge weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstructibilityEdge {
    pub role: ConstructibilityEdgeRole,
    pub strength: RequirementStrength,
}

/// Canonical graph-construction input.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstructibilityDependencyDeclaration {
    pub source: ConstructibilityNodeId,
    pub target: ConstructibilityNodeId,
    pub edge: ConstructibilityEdge,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstructibilityDependencyProjection {
    pub source: ConstructibilityNodeId,
    pub target: ConstructibilityNodeId,
    pub edge: ConstructibilityEdge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructibilityGraphProjection {
    pub nodes: Vec<ConstructibilityNode>,
    pub edges: Vec<ConstructibilityDependencyProjection>,
}

#[allow(clippy::type_complexity)]
pub fn build_constructibility_graph(
    nodes: impl IntoIterator<Item = ConstructibilityNode>,
    edges: impl IntoIterator<Item = ConstructibilityDependencyDeclaration>,
) -> Result<
    (
        DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
        BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
        Vec<ConstructibilityNodeId>,
    ),
    RealizationError,
> {
    let mut nodes = nodes.into_iter().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    for pair in nodes.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(RealizationError::DuplicateConstructibilityNode(
                pair[0].id.clone(),
            ));
        }
    }

    let mut edges = edges.into_iter().collect::<Vec<_>>();
    edges.sort();

    let mut graph = DiGraph::<ConstructibilityNode, ConstructibilityEdge, u32>::with_capacity(
        nodes.len(),
        edges.len(),
    );
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
            .ok_or_else(|| RealizationError::UnknownConstructibilityNode(edge.source.clone()))?;
        let target = node_by_id
            .get(&edge.target)
            .copied()
            .ok_or_else(|| RealizationError::UnknownConstructibilityNode(edge.target.clone()))?;

        graph.add_edge(source, target, edge.edge);
    }

    let order = toposort(&graph, None)
        .map_err(|_cycle| RealizationError::ConstructibilityCycle {
            components: canonical_constructibility_sccs(&graph),
        })?
        .into_iter()
        .map(|node| graph[node].id.clone())
        .collect();

    Ok((graph, node_by_id, order))
}

pub fn validate_constructibility(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    node_by_id: &BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
    operation: OperationId,
    permissionless: bool,
) -> Result<(), RealizationError> {
    let operation_id = ConstructibilityNodeId::Operation(operation);
    let operation_node = node_by_id.get(&operation_id).copied().ok_or(
        RealizationError::MissingConstructibilityOperation(operation),
    )?;

    for edge in graph.edges_directed(operation_node, Direction::Incoming) {
        let source = &graph[edge.source()].id;
        let availability = match source {
            ConstructibilityNodeId::Fact { availability, .. }
            | ConstructibilityNodeId::Witness { availability, .. } => *availability,
            ConstructibilityNodeId::Operation(_) => continue,
        };

        if permissionless
            && matches!(
                availability,
                AvailabilityClass::InputOwners { .. } | AvailabilityClass::Operator
            )
        {
            return Err(RealizationError::PermissionlessPrivateDependency {
                operation,
                source_node: source.clone(),
            });
        }

        if availability == AvailabilityClass::SponsorLocal
            && edge.weight().role != ConstructibilityEdgeRole::SponsorOnly
        {
            return Err(RealizationError::SponsorDependencyEscaped {
                operation,
                source_node: source.clone(),
            });
        }
    }

    Ok(())
}

#[must_use]
pub fn project_constructibility_graph(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
) -> ConstructibilityGraphProjection {
    let mut nodes = graph.node_weights().cloned().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    let mut edges = graph
        .edge_references()
        .map(|edge| ConstructibilityDependencyProjection {
            source: graph[edge.source()].id.clone(),
            target: graph[edge.target()].id.clone(),
            edge: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();

    ConstructibilityGraphProjection { nodes, edges }
}

fn canonical_constructibility_sccs(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
) -> Vec<Vec<ConstructibilityNodeId>> {
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
