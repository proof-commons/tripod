//! Constructibility traversal over finite owner-produced records.
use crate::engine_records::OperationKey;
use petgraph::{
    Direction,
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Availability {
    pub namespace: u16,
    pub kind: u16,
    pub object: u16,
    pub sponsor_local: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeKey {
    pub kind: u16,
    pub operation: OperationKey,
    pub subject: Vec<u16>,
    pub availability: Option<Availability>,
    pub independent: bool,
}
impl NodeKey {
    pub const fn operation(operation: OperationKey) -> Self {
        Self {
            kind: 0,
            operation,
            subject: Vec::new(),
            availability: None,
            independent: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: NodeKey,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub kind: u16,
    pub required: bool,
    pub sponsor_only: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    pub source: NodeKey,
    pub target: NodeKey,
    pub edge: Edge,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authorization {
    pub key: Vec<u16>,
    pub permissionless: bool,
    pub allowed: BTreeSet<Availability>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Records {
    pub scope: Vec<OperationKey>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Dependency>,
    pub operations: Vec<(OperationKey, Vec<Authorization>)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseAnalysis {
    pub authorization: Authorization,
    pub required_nodes: Vec<NodeKey>,
    pub optional_nodes: Vec<NodeKey>,
    pub required_availability: BTreeSet<Availability>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationAnalysis {
    pub operation: OperationKey,
    pub cases: Vec<CaseAnalysis>,
}
#[derive(Clone, Debug)]
pub struct Analysis {
    pub graph: DiGraph<Node, Edge, u32>,
    pub node_by_id: BTreeMap<NodeKey, NodeIndex<u32>>,
    pub operations: Vec<OperationAnalysis>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordError {
    DuplicateConstructibilityNode {
        node: NodeKey,
    },
    DuplicateConstructibilityDependency {
        node: NodeKey,
    },
    UnknownConstructibilityNode {
        node: NodeKey,
    },
    ConstructibilityCycle {
        components: Vec<Vec<NodeKey>>,
    },
    ConstructibilityCrossOperationDependency {
        operation: OperationKey,
        node: NodeKey,
    },
    PermissionlessPrivateDependency {
        operation: OperationKey,
        node: NodeKey,
        path: Vec<NodeKey>,
    },
    ConstructibilityWitnessUnavailable {
        operation: OperationKey,
        authorization: Authorization,
        node: NodeKey,
        path: Vec<NodeKey>,
    },
    SponsorDependencyEscaped {
        operation: OperationKey,
        node: NodeKey,
        path: Vec<NodeKey>,
    },
}
pub fn analyze_constructibility(records: &Records) -> Result<Analysis, RecordError> {
    let span = tracing::debug_span!(target: "compiler::engine_constructibility", "analyze_constructibility", node_count = crate::instrumentation::count(records.nodes.len()), case_count = crate::instrumentation::count(records.operations.len()));
    let _entered = span.enter();
    let projection = records;

    let mut nodes = projection
        .nodes
        .iter()
        .filter(|node| records.scope.contains(&node_operation(&node.id)))
        .cloned()
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    for pair in nodes.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(RecordError::DuplicateConstructibilityNode {
                node: pair[0].id.clone(),
            });
        }
    }

    let selected = nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    let mut edges = projection
        .edges
        .iter()
        .filter(|edge| selected.contains(&edge.source) || selected.contains(&edge.target))
        .cloned()
        .collect::<Vec<_>>();
    edges.sort();

    for pair in edges.windows(2) {
        if pair[0] == pair[1] {
            return Err(RecordError::DuplicateConstructibilityDependency {
                node: pair[0].source.clone(),
            });
        }
    }

    let mut graph = DiGraph::<Node, Edge, u32>::with_capacity(nodes.len(), edges.len());
    let mut node_by_id = BTreeMap::new();

    for node_weight in nodes {
        let id = node_weight.id.clone();
        let node = graph.add_node(node_weight);
        node_by_id.insert(id, node);
    }

    for edge in &edges {
        let source = node_by_id.get(&edge.source).copied().ok_or_else(|| {
            RecordError::UnknownConstructibilityNode {
                node: edge.source.clone(),
            }
        })?;
        let target = node_by_id.get(&edge.target).copied().ok_or_else(|| {
            RecordError::UnknownConstructibilityNode {
                node: edge.target.clone(),
            }
        })?;

        graph.add_edge(source, target, edge.edge);
    }

    if toposort(&graph, None).is_err() {
        return Err(RecordError::ConstructibilityCycle {
            components: canonical_sccs(&graph),
        });
    }

    let mut operations = Vec::new();

    for (operation, cases) in &records.operations {
        operations.push(analyze_operation(&graph, &node_by_id, *operation, cases)?);
    }

    Ok(Analysis {
        graph,
        node_by_id,
        operations,
    })
}

fn analyze_operation(
    graph: &DiGraph<Node, Edge, u32>,
    node_by_id: &BTreeMap<NodeKey, NodeIndex<u32>>,
    operation: OperationKey,
    authorizations: &[Authorization],
) -> Result<OperationAnalysis, RecordError> {
    let operation_id = NodeKey::operation(operation);
    let operation_node = node_by_id.get(&operation_id).copied().ok_or_else(|| {
        RecordError::UnknownConstructibilityNode {
            node: operation_id.clone(),
        }
    })?;

    let required = required_ancestors(graph, operation_node);
    let ancestors = all_ancestors(graph, operation_node);

    // Cross-operation dependencies fail regardless of the case.
    for ancestor in &ancestors {
        let id = &graph[*ancestor].id;

        if node_operation(id) != operation && !operation_independent_fact(id) {
            return Err(RecordError::ConstructibilityCrossOperationDependency {
                operation,
                node: id.clone(),
            });
        }
    }

    let mut cases = Vec::new();

    for authorization in authorizations {
        let mut required_nodes = Vec::new();
        let mut optional_nodes = Vec::new();
        let mut required_availability = BTreeSet::new();

        for ancestor in &ancestors {
            let id = &graph[*ancestor].id;
            let availability = node_availability(id);
            let is_required = required.contains(ancestor);

            if let Some(availability) = availability {
                if is_required {
                    required_availability.insert(availability);
                    validate_required_dependency(operation, authorization, id, || {
                        path_to_operation(graph, *ancestor, operation_node)
                    })?;
                }

                // A sponsor-local dependency must stay confined to an
                // optional sponsor-only subtree, required or not.
                if availability.sponsor_local {
                    if is_required {
                        return Err(RecordError::SponsorDependencyEscaped {
                            operation,
                            node: id.clone(),
                            path: path_to_operation(graph, *ancestor, operation_node),
                        });
                    }

                    if let Some(path) = sponsor_escape_path(graph, *ancestor, operation_node) {
                        return Err(RecordError::SponsorDependencyEscaped {
                            operation,
                            node: id.clone(),
                            path,
                        });
                    }
                }
            }

            if is_required {
                required_nodes.push(id.clone());
            } else {
                optional_nodes.push(id.clone());
            }
        }

        required_nodes.sort();
        optional_nodes.sort();

        cases.push(CaseAnalysis {
            authorization: authorization.clone(),
            required_nodes,
            optional_nodes,
            required_availability,
        });
    }

    Ok(OperationAnalysis { operation, cases })
}

pub fn validate_required_dependency(
    operation: OperationKey,
    authorization: &Authorization,
    node: &NodeKey,
    path: impl FnOnce() -> Vec<NodeKey>,
) -> Result<(), RecordError> {
    if node
        .availability
        .is_none_or(|value| authorization.allowed.contains(&value))
    {
        return Ok(());
    }
    let node = node.clone();
    let path = path();
    if authorization.permissionless {
        Err(RecordError::PermissionlessPrivateDependency {
            operation,
            node,
            path,
        })
    } else {
        Err(RecordError::ConstructibilityWitnessUnavailable {
            operation,
            authorization: authorization.clone(),
            node,
            path,
        })
    }
}
const fn node_operation(node: &NodeKey) -> OperationKey {
    node.operation
}
const fn operation_independent_fact(node: &NodeKey) -> bool {
    node.independent
}
const fn node_availability(node: &NodeKey) -> Option<Availability> {
    node.availability
}
fn required_ancestors(
    graph: &DiGraph<Node, Edge, u32>,
    operation_node: NodeIndex<u32>,
) -> BTreeSet<NodeIndex<u32>> {
    let mut seen = BTreeSet::new();
    let mut stack = graph
        .edges_directed(operation_node, Direction::Incoming)
        .filter(|edge| edge.weight().required)
        .map(|edge| edge.source())
        .collect::<Vec<_>>();

    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }

        stack.extend(
            graph
                .edges_directed(node, Direction::Incoming)
                .filter(|edge| edge.weight().required)
                .map(|edge| edge.source()),
        );
    }

    seen
}

fn all_ancestors(
    graph: &DiGraph<Node, Edge, u32>,
    operation_node: NodeIndex<u32>,
) -> Vec<NodeIndex<u32>> {
    let mut seen = BTreeSet::new();
    let mut stack = graph
        .edges_directed(operation_node, Direction::Incoming)
        .map(|edge| edge.source())
        .collect::<Vec<_>>();

    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }

        stack.extend(
            graph
                .edges_directed(node, Direction::Incoming)
                .map(|edge| edge.source()),
        );
    }

    let mut ancestors = seen.into_iter().collect::<Vec<_>>();
    ancestors.sort_by(|left, right| graph[*left].id.cmp(&graph[*right].id));
    ancestors
}

/// One deterministic path from `source` to the operation, in stable
/// node IDs — for diagnostics only, insertion-order independent
/// because successors are visited in sorted-ID order.
fn path_to_operation(
    graph: &DiGraph<Node, Edge, u32>,
    source: NodeIndex<u32>,
    operation_node: NodeIndex<u32>,
) -> Vec<NodeKey> {
    fn visit(
        graph: &DiGraph<Node, Edge, u32>,
        node: NodeIndex<u32>,
        operation_node: NodeIndex<u32>,
        path: &mut Vec<NodeKey>,
    ) -> bool {
        if node == operation_node {
            return true;
        }

        for successor in sorted_successors(graph, node) {
            path.push(graph[successor].id.clone());

            if visit(graph, successor, operation_node, path) {
                return true;
            }

            path.pop();
        }

        false
    }

    let mut path = vec![graph[source].id.clone()];
    visit(graph, source, operation_node, &mut path);
    path
}

/// A path proving a sponsor-local subtree escapes confinement, if any:
/// a step over a non-sponsor-only edge or through a non-sponsor node
/// before reaching the operation.
fn sponsor_escape_path(
    graph: &DiGraph<Node, Edge, u32>,
    source: NodeIndex<u32>,
    operation_node: NodeIndex<u32>,
) -> Option<Vec<NodeKey>> {
    let mut edges = graph
        .edges_directed(source, Direction::Outgoing)
        .map(|edge| (edge.target(), *edge.weight()))
        .collect::<Vec<_>>();
    edges.sort_by(|(left, _), (right, _)| graph[*left].id.cmp(&graph[*right].id));

    for (successor, edge) in edges {
        let mut path = vec![graph[source].id.clone(), graph[successor].id.clone()];

        let edge_escapes = !edge.sponsor_only;
        let node_escapes = successor != operation_node
            && !node_availability(&graph[successor].id).is_some_and(|value| value.sponsor_local);

        if edge_escapes || node_escapes {
            return Some(path);
        }

        if successor != operation_node
            && let Some(deeper) = sponsor_escape_path(graph, successor, operation_node)
        {
            path.pop();
            path.extend(deeper);
            return Some(path);
        }
    }

    None
}

fn sorted_successors(
    graph: &DiGraph<Node, Edge, u32>,
    node: NodeIndex<u32>,
) -> Vec<NodeIndex<u32>> {
    let mut successors = graph
        .edges_directed(node, Direction::Outgoing)
        .map(|edge| edge.target())
        .collect::<Vec<_>>();
    successors.sort_by(|left, right| graph[*left].id.cmp(&graph[*right].id));
    successors.dedup();
    successors
}

fn canonical_sccs(graph: &DiGraph<Node, Edge, u32>) -> Vec<Vec<NodeKey>> {
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
