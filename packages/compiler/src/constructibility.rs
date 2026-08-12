//! Authorization-case constructibility analysis (Guide-3 Tranche D).
//!
//! The compiler rebuilds the realization's stable constructibility
//! projection into its own direct Petgraph graph and revalidates,
//! per operation and per owner-derived authorization case, that every
//! required ancestor's availability is dischargeable, that sponsor-local
//! dependencies stay confined to optional sponsor-only subtrees, and
//! that no dependency crosses operations. Witness-bearing source
//! requirements are then cross-checked against the graph so a proof
//! can never assume a witness constructibility does not provide.

// One item-level allowance remains: the Petgraph storage is retained
// beside the derived analysis rather than read by it. The analysis
// answers its questions from the resolved maps.

use std::collections::{BTreeMap, BTreeSet};

use petgraph::{
    Direction,
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use realization::{
    AvailabilityClass, ConstructibilityAuthorization, ConstructibilityEdge,
    ConstructibilityEdgeRole, ConstructibilityNode, ConstructibilityNodeId, RequirementStrength,
    WitnessRole,
};

use crate::{
    CompileError,
    input::BoundCompilerInput,
    source::{RequiredSourceKind, SourceRequirement},
};

/// Constructibility analysis of one authorization case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructibilityCaseAnalysis {
    pub authorization: ConstructibilityAuthorization,
    pub required_nodes: Vec<ConstructibilityNodeId>,
    pub optional_nodes: Vec<ConstructibilityNodeId>,
    pub required_availability: BTreeSet<AvailabilityClass>,
}

/// Constructibility analysis of one operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationConstructibilityAnalysis {
    pub operation: architecture::OperationId,
    pub cases: Vec<ConstructibilityCaseAnalysis>,
}

/// Internal compiler constructibility analysis.
#[derive(Debug)]
pub struct CompilerConstructibilityAnalysis {
    #[allow(dead_code)]
    pub graph: DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    pub node_by_id: BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
    pub operations: Vec<OperationConstructibilityAnalysis>,
}

/// Build and validate the scoped constructibility analysis.
pub fn build_constructibility_analysis(
    input: &BoundCompilerInput,
) -> Result<CompilerConstructibilityAnalysis, CompileError> {
    let projection = input.realization().project().constructibility;

    let mut nodes = projection
        .nodes
        .iter()
        .filter(|node| {
            input
                .scope()
                .operations()
                .contains(&node_operation(&node.id))
        })
        .cloned()
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    for pair in nodes.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(CompileError::DuplicateConstructibilityNode {
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
            return Err(CompileError::DuplicateConstructibilityDependency {
                node: pair[0].source.clone(),
            });
        }
    }

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

    for edge in &edges {
        let source = node_by_id.get(&edge.source).copied().ok_or_else(|| {
            CompileError::UnknownConstructibilityNode {
                node: edge.source.clone(),
            }
        })?;
        let target = node_by_id.get(&edge.target).copied().ok_or_else(|| {
            CompileError::UnknownConstructibilityNode {
                node: edge.target.clone(),
            }
        })?;

        graph.add_edge(source, target, edge.edge);
    }

    if toposort(&graph, None).is_err() {
        return Err(CompileError::ConstructibilityCycle {
            components: canonical_sccs(&graph),
        });
    }

    let mut operations = Vec::new();

    for operation in input.scope().operations() {
        let cases = input
            .realization()
            .constructibility_authorizations(*operation)
            .map_err(|_| CompileError::InvalidRealization)?;

        operations.push(analyze_operation(&graph, &node_by_id, *operation, cases)?);
    }

    Ok(CompilerConstructibilityAnalysis {
        graph,
        node_by_id,
        operations,
    })
}

fn analyze_operation(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    node_by_id: &BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
    operation: architecture::OperationId,
    authorizations: &[ConstructibilityAuthorization],
) -> Result<OperationConstructibilityAnalysis, CompileError> {
    let operation_id = ConstructibilityNodeId::Operation(operation);
    let operation_node = node_by_id.get(&operation_id).copied().ok_or_else(|| {
        CompileError::UnknownConstructibilityNode {
            node: operation_id.clone(),
        }
    })?;

    let required = required_ancestors(graph, operation_node);
    let ancestors = all_ancestors(graph, operation_node);

    // Cross-operation dependencies fail regardless of the case.
    for ancestor in &ancestors {
        let id = &graph[*ancestor].id;

        if node_operation(id) != operation && !operation_independent_fact(id) {
            return Err(CompileError::ConstructibilityCrossOperationDependency {
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

                    if !authorization.discharges(availability) {
                        return Err(unavailable_error(
                            operation,
                            authorization,
                            id.clone(),
                            path_to_operation(graph, *ancestor, operation_node),
                        ));
                    }
                }

                // A sponsor-local dependency must stay confined to an
                // optional sponsor-only subtree, required or not.
                if availability == AvailabilityClass::SponsorLocal {
                    if is_required {
                        return Err(CompileError::SponsorDependencyEscaped {
                            operation,
                            node: id.clone(),
                            path: path_to_operation(graph, *ancestor, operation_node),
                        });
                    }

                    if let Some(path) = sponsor_escape_path(graph, *ancestor, operation_node) {
                        return Err(CompileError::SponsorDependencyEscaped {
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

        cases.push(ConstructibilityCaseAnalysis {
            authorization: authorization.clone(),
            required_nodes,
            optional_nodes,
            required_availability,
        });
    }

    Ok(OperationConstructibilityAnalysis { operation, cases })
}

/// Cross-check witness-bearing source requirements against the graph:
/// a proof may not assume a witness constructibility does not provide.
pub fn validate_source_constructibility(
    analysis: &CompilerConstructibilityAnalysis,
    relation: &realization::RelationId,
    rows: &[SourceRequirement],
) -> Result<(), CompileError> {
    let operation = relation.operation();
    let optional = optional_witnesses(analysis, operation);

    for row in rows {
        let expected = match row.source {
            RequiredSourceKind::InputOwnerWitness => Some(WitnessRole::ProtocolOwnerAuthorization),
            RequiredSourceKind::OperatorWitness => Some(WitnessRole::OperatorAuthorization),
            RequiredSourceKind::SponsorLocalWitness => Some(WitnessRole::SponsorAuthorization),
            _ => None,
        };
        let Some(role) = expected else {
            continue;
        };

        let witness = ConstructibilityNodeId::Witness {
            operation,
            role,
            availability: row.availability,
        };

        if !analysis.node_by_id.contains_key(&witness) {
            return Err(CompileError::SourceConstructibilityMismatch {
                relation: relation.clone(),
                operand: row.operand.clone(),
            });
        }

        // A sponsor-local witness must belong to an optional subtree.
        if row.availability == AvailabilityClass::SponsorLocal && !optional.contains(&witness) {
            return Err(CompileError::SourceConstructibilityMismatch {
                relation: relation.clone(),
                operand: row.operand.clone(),
            });
        }
    }

    Ok(())
}

fn optional_witnesses(
    analysis: &CompilerConstructibilityAnalysis,
    operation: architecture::OperationId,
) -> BTreeSet<ConstructibilityNodeId> {
    analysis
        .operations
        .iter()
        .filter(|row| row.operation == operation)
        .flat_map(|row| row.cases.iter())
        .flat_map(|case| case.optional_nodes.iter().cloned())
        .collect()
}

fn unavailable_error(
    operation: architecture::OperationId,
    authorization: &ConstructibilityAuthorization,
    node: ConstructibilityNodeId,
    path: Vec<ConstructibilityNodeId>,
) -> CompileError {
    match authorization {
        ConstructibilityAuthorization::Permissionless
        | ConstructibilityAuthorization::CadencePermissionless => {
            CompileError::PermissionlessPrivateDependency {
                operation,
                node,
                path,
            }
        }
        _ => CompileError::ConstructibilityWitnessUnavailable {
            operation,
            authorization: authorization.clone(),
            node,
            path,
        },
    }
}

const fn node_operation(node: &ConstructibilityNodeId) -> architecture::OperationId {
    match node {
        ConstructibilityNodeId::Operation(operation)
        | ConstructibilityNodeId::Fact { operation, .. }
        | ConstructibilityNodeId::Witness { operation, .. } => *operation,
    }
}

/// True for a fact node whose underlying fact is operation-independent
/// (an architecture bound cited by the operation's own node ID).
const fn operation_independent_fact(node: &ConstructibilityNodeId) -> bool {
    matches!(
        node,
        ConstructibilityNodeId::Fact {
            fact: realization::FactId::BoundValue { .. },
            ..
        }
    )
}

const fn node_availability(node: &ConstructibilityNodeId) -> Option<AvailabilityClass> {
    match node {
        ConstructibilityNodeId::Fact { availability, .. }
        | ConstructibilityNodeId::Witness { availability, .. } => Some(*availability),
        ConstructibilityNodeId::Operation(_) => None,
    }
}

fn required_ancestors(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    operation_node: NodeIndex<u32>,
) -> BTreeSet<NodeIndex<u32>> {
    let mut seen = BTreeSet::new();
    let mut stack = graph
        .edges_directed(operation_node, Direction::Incoming)
        .filter(|edge| edge.weight().strength == RequirementStrength::Required)
        .map(|edge| edge.source())
        .collect::<Vec<_>>();

    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }

        stack.extend(
            graph
                .edges_directed(node, Direction::Incoming)
                .filter(|edge| edge.weight().strength == RequirementStrength::Required)
                .map(|edge| edge.source()),
        );
    }

    seen
}

fn all_ancestors(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
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
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    source: NodeIndex<u32>,
    operation_node: NodeIndex<u32>,
) -> Vec<ConstructibilityNodeId> {
    fn visit(
        graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
        node: NodeIndex<u32>,
        operation_node: NodeIndex<u32>,
        path: &mut Vec<ConstructibilityNodeId>,
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
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    source: NodeIndex<u32>,
    operation_node: NodeIndex<u32>,
) -> Option<Vec<ConstructibilityNodeId>> {
    let mut edges = graph
        .edges_directed(source, Direction::Outgoing)
        .map(|edge| (edge.target(), *edge.weight()))
        .collect::<Vec<_>>();
    edges.sort_by(|(left, _), (right, _)| graph[*left].id.cmp(&graph[*right].id));

    for (successor, edge) in edges {
        let mut path = vec![graph[source].id.clone(), graph[successor].id.clone()];

        let edge_escapes = edge.role != ConstructibilityEdgeRole::SponsorOnly;
        let node_escapes = successor != operation_node
            && node_availability(&graph[successor].id) != Some(AvailabilityClass::SponsorLocal);

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
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
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

fn canonical_sccs(
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
