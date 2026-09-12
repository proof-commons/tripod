//! Typed constructibility requirements stored in a direct Petgraph graph.

use std::collections::{BTreeMap, BTreeSet};

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
    RefundKey { object: ObjectId },
    ClientOwners { object: ObjectId },
}

/// Realization-owned analysis of how one operation is authorized.
///
/// This is not architecture identity: it is the derived case set that
/// [`constructibility_authorizations`](crate::ScopedRealizationSpec::constructibility_authorizations)
/// computes from the typed operation row, and against which every
/// constructibility dependency's [`AvailabilityClass`] is checked. A
/// cadence-band operation yields two cases — an operator window and a
/// delayed permissionless window — so both are validated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstructibilityAuthorization {
    Permissionless,
    InputOwners { objects: BTreeSet<ObjectId> },
    RefundKey { object: ObjectId },
    ClientAuthorized { objects: BTreeSet<ObjectId> },
    Operator,
    CadenceOperator,
    CadencePermissionless,
}

impl ConstructibilityAuthorization {
    /// Whether this authorization case can discharge a dependency of
    /// the given availability class.
    ///
    /// `Public` is always dischargeable. `SponsorLocal` is
    /// dischargeable here because sponsor confinement — the rule that
    /// a sponsor-local dependency lives only in an optional
    /// sponsor-only subtree — is validated separately; availability is
    /// not the confinement check. Every private class must be named by
    /// the case.
    #[must_use]
    pub fn discharges(&self, availability: AvailabilityClass) -> bool {
        availability_allowed(availability, self)
    }
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

/// Whether one constructibility dependency must be dischargeable in
/// every validated authorization case.
///
/// A `Required` dependency's availability must be discharged by each
/// authorization case. An `Optional` dependency marks an omissible
/// subtree — the sponsor envelope, for example — that a case simply
/// omits when it cannot discharge the availability; when the subtree is
/// present it still obeys the sponsor-confinement and cross-operation
/// rules.
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
pub(crate) fn build_constructibility_graph(
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

    for pair in edges.windows(2) {
        if pair[0] == pair[1] {
            return Err(RealizationError::DuplicateConstructibilityDependency(
                pair[0].clone(),
            ));
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

pub(crate) fn validate_constructibility(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    node_by_id: &BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
    operation: OperationId,
    authorization: &ConstructibilityAuthorization,
) -> Result<(), RealizationError> {
    let operation_id = ConstructibilityNodeId::Operation(operation);
    let operation_node = node_by_id.get(&operation_id).copied().ok_or(
        RealizationError::MissingConstructibilityOperation(operation),
    )?;
    let required = required_ancestors(graph, operation_node);

    for ancestor in reverse_reachable_ancestors(graph, operation_node) {
        let source = &graph[ancestor].id;
        let path = path_to_operation(graph, ancestor, operation_node)
            .unwrap_or_else(|| vec![source.clone(), ConstructibilityNodeId::Operation(operation)]);

        if !node_belongs_to_operation(source, operation) {
            return Err(RealizationError::CrossOperationConstructibilityDependency {
                operation,
                source_node: source.clone(),
                path,
            });
        }

        let Some(availability) = node_availability(source) else {
            continue;
        };

        // An ancestor whose every path to the operation crosses an
        // Optional edge is an omissible subtree: an authorization case
        // that cannot discharge its availability simply omits it, so
        // unavailability is not a defect for that case.
        if !availability_allowed(availability, authorization) && required.contains(&ancestor) {
            return Err(unavailable_dependency_error(
                operation,
                authorization,
                source.clone(),
                path,
            ));
        }

        if availability == AvailabilityClass::SponsorLocal
            && let Some(path) = sponsor_escape_path(graph, ancestor, operation_node)
        {
            return Err(RealizationError::SponsorDependencyEscaped {
                operation,
                source_node: source.clone(),
                path,
            });
        }
    }

    Ok(())
}

/// Whether one dependency's availability class can be discharged under
/// the given operation authorization. Public and sponsor-local
/// dependencies are always allowed here (sponsor-local dependencies are
/// separately confined to sponsor-only edges); every private class must
/// be named by the authorization.
fn availability_allowed(
    availability: AvailabilityClass,
    authorization: &ConstructibilityAuthorization,
) -> bool {
    use ConstructibilityAuthorization as Auth;

    match availability {
        AvailabilityClass::Public | AvailabilityClass::SponsorLocal => true,

        AvailabilityClass::InputOwners { object } => matches!(
            authorization,
            Auth::InputOwners { objects } | Auth::ClientAuthorized { objects }
                if objects.contains(&object)
        ),

        AvailabilityClass::Operator => {
            matches!(authorization, Auth::Operator | Auth::CadenceOperator)
        }

        AvailabilityClass::RefundKey { object } => matches!(
            authorization,
            Auth::RefundKey { object: expected } if *expected == object
        ),

        AvailabilityClass::ClientOwners { object } => matches!(
            authorization,
            Auth::ClientAuthorized { objects } if objects.contains(&object)
        ),
    }
}

/// Selects the focused error for a dependency the authorization cannot
/// discharge. Permissionless windows keep the historic
/// `PermissionlessPrivateDependency`; every other authorization reports
/// the typed unavailable-witness error.
fn unavailable_dependency_error(
    operation: OperationId,
    authorization: &ConstructibilityAuthorization,
    source_node: ConstructibilityNodeId,
    path: Vec<ConstructibilityNodeId>,
) -> RealizationError {
    match authorization {
        ConstructibilityAuthorization::Permissionless
        | ConstructibilityAuthorization::CadencePermissionless => {
            RealizationError::PermissionlessPrivateDependency {
                operation,
                source_node,
                path,
            }
        }

        _ => RealizationError::ConstructibilityWitnessUnavailable {
            operation,
            authorization: authorization.clone(),
            source_node,
            path,
        },
    }
}

/// Ancestors required in every authorization case: nodes with at least
/// one path to the operation using only `Required`-strength edges. A
/// node whose every path crosses an `Optional` edge belongs to an
/// omissible subtree instead.
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

fn reverse_reachable_ancestors(
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

fn node_availability(node: &ConstructibilityNodeId) -> Option<AvailabilityClass> {
    match node {
        ConstructibilityNodeId::Fact { availability, .. }
        | ConstructibilityNodeId::Witness { availability, .. } => Some(*availability),
        ConstructibilityNodeId::Operation(_) => None,
    }
}

fn node_belongs_to_operation(node: &ConstructibilityNodeId, operation: OperationId) -> bool {
    match node {
        ConstructibilityNodeId::Operation(node_operation)
        | ConstructibilityNodeId::Witness {
            operation: node_operation,
            ..
        } => *node_operation == operation,
        ConstructibilityNodeId::Fact {
            operation: node_operation,
            fact,
            ..
        } => {
            *node_operation == operation
                && fact_operation(fact).is_none_or(|fact_operation| fact_operation == operation)
        }
    }
}

fn fact_operation(fact: &FactId) -> Option<OperationId> {
    match fact {
        FactId::FamilyCount { operation, .. }
        | FactId::FamilyAmount { operation, .. }
        | FactId::StateField { operation, .. }
        | FactId::RequestedAnnouncementCycle { operation }
        | FactId::AnnouncementLead { operation, .. }
        | FactId::InputOwners { operation, .. }
        | FactId::Signers { operation }
        | FactId::ProjectionPresent { operation, .. }
        | FactId::FamilyRecognized { operation, .. }
        | FactId::SponsorIsolated { operation }
        | FactId::ProtocolSecretUsed { operation } => Some(*operation),
        FactId::BoundValue { .. } => None,
    }
}

fn path_to_operation(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    source: NodeIndex<u32>,
    operation_node: NodeIndex<u32>,
) -> Option<Vec<ConstructibilityNodeId>> {
    if source == operation_node {
        return Some(vec![graph[source].id.clone()]);
    }

    let successors = sorted_successors(graph, source);

    for successor in successors {
        if let Some(mut path) = path_to_operation(graph, successor, operation_node) {
            path.insert(0, graph[source].id.clone());
            return Some(path);
        }
    }

    None
}

fn sponsor_escape_path(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    source: NodeIndex<u32>,
    operation_node: NodeIndex<u32>,
) -> Option<Vec<ConstructibilityNodeId>> {
    fn visit(
        graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
        node: NodeIndex<u32>,
        operation_node: NodeIndex<u32>,
        path: &mut Vec<ConstructibilityNodeId>,
    ) -> Option<Vec<ConstructibilityNodeId>> {
        if node == operation_node {
            return None;
        }

        for (successor, edge) in sorted_outgoing_edges(graph, node) {
            path.push(graph[successor].id.clone());

            let edge_escapes = edge.role != ConstructibilityEdgeRole::SponsorOnly;
            let node_escapes = successor != operation_node
                && node_availability(&graph[successor].id) != Some(AvailabilityClass::SponsorLocal);

            if edge_escapes || node_escapes {
                return Some(path.clone());
            }

            if let Some(path) = visit(graph, successor, operation_node, path) {
                return Some(path);
            }

            path.pop();
        }

        None
    }

    let mut path = vec![graph[source].id.clone()];
    visit(graph, source, operation_node, &mut path)
}

fn sorted_successors(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    node: NodeIndex<u32>,
) -> Vec<NodeIndex<u32>> {
    sorted_outgoing_edges(graph, node)
        .into_iter()
        .map(|(successor, _edge)| successor)
        .collect()
}

fn sorted_outgoing_edges(
    graph: &DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    node: NodeIndex<u32>,
) -> Vec<(NodeIndex<u32>, ConstructibilityEdge)> {
    let mut edges = graph
        .edges_directed(node, Direction::Outgoing)
        .map(|edge| (edge.target(), *edge.weight()))
        .collect::<Vec<_>>();
    edges.sort_by(|(left_node, left_edge), (right_node, right_edge)| {
        graph[*left_node]
            .id
            .cmp(&graph[*right_node].id)
            .then_with(|| left_edge.cmp(right_edge))
    });
    edges
}

#[must_use]
pub(crate) fn project_constructibility_graph(
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
