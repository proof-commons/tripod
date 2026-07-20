//! Typed semantic relations and direct Petgraph relation dependencies.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, BoundId, ObjectId, OperationId, ProjectionId, ProjectionRule, RootId, RootUse,
};
use petgraph::{
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{
    Count, ExprId, ObservedSide, ProofAlternativeId, RealizationError, RelationId,
    RepresentationMode,
};

/// Semantic constructibility class for one operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructibilityClass {
    PublicPermissionless,
    OwnersOf { object: ObjectId },
}

/// Cardinality maximum retained in architecture-owned form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardinalityMaximum {
    Exact(Count),
    Bound(BoundId),
}

/// Runtime-evaluable or declaration-level semantic relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Relation {
    Cardinality {
        side: ObservedSide,
        object: ObjectId,
        minimum: Count,
        maximum: CardinalityMaximum,
    },
    AllowedObjectFamilies {
        side: ObservedSide,
        allowed: BTreeSet<ObjectId>,
    },
    Recognition {
        side: ObservedSide,
        object: ObjectId,
        asset: AssetId,
    },
    AmountConservation {
        asset: AssetId,
        input_objects: BTreeSet<ObjectId>,
        output_objects: BTreeSet<ObjectId>,
    },
    OwnerAuthorization {
        object: ObjectId,
    },
    PermissionlessAuthorization,
    SponsorIsolation,
    RootPolicy {
        expected: BTreeMap<RootId, RootUse>,
    },
    ProjectionPolicy {
        expected: BTreeMap<ProjectionId, ProjectionRule>,
    },
    Constructibility {
        class: ConstructibilityClass,
    },
    Representation {
        object: ObjectId,
        allowed: BTreeSet<RepresentationMode>,
    },
    LifecycleExit {
        object: ObjectId,
        exit: OperationId,
    },
    ExpressionPredicate {
        expression: ExprId,
    },
}

/// Why one relation depends on another.
///
/// Direction is:
///
/// ```text
/// prerequisite -> dependent
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationEdge {
    RecognitionBeforeCardinality,
    RecognitionBeforeValue,
    CardinalityBeforeValue,
    AuthorizationBeforeClosure,
    AuthorizationBeforeConstructibility,
    SponsorBeforeConstructibility,
    SponsorBeforeOperation,
    RepresentationBeforeLifecycle,
    ProjectionPolicyBeforeOperation,
    RootPolicyBeforeOperation,
    StaticRequirement,
}

/// Stable relation declaration stored directly as a Petgraph node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationDeclaration {
    pub id: RelationId,
    pub relation: Relation,
    pub proof_alternatives: BTreeSet<ProofAlternativeId>,
}

/// One explicit stable relation dependency-edge declaration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationDependencyDeclaration {
    pub prerequisite: RelationId,
    pub dependent: RelationId,
    pub edge: RelationEdge,
}

/// Stable typed projection of one relation dependency edge.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationDependencyProjection {
    pub source: RelationId,
    pub target: RelationId,
    pub edge: RelationEdge,
}

/// Canonical typed projection of one relation graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationGraphProjection {
    pub nodes: Vec<RelationDeclaration>,
    pub edges: Vec<RelationDependencyProjection>,
}

/// Build a direct Petgraph relation dependency graph.
#[allow(clippy::type_complexity)]
pub fn build_relation_graph(
    declarations: impl IntoIterator<Item = RelationDeclaration>,
    dependencies: impl IntoIterator<Item = RelationDependencyDeclaration>,
) -> Result<
    (
        DiGraph<RelationDeclaration, RelationEdge, u32>,
        BTreeMap<RelationId, NodeIndex<u32>>,
        Vec<RelationId>,
    ),
    RealizationError,
> {
    let mut by_id = BTreeMap::new();

    for declaration in declarations {
        let id = declaration.id.clone();

        if by_id.insert(id.clone(), declaration).is_some() {
            return Err(RealizationError::DuplicateRelation(id));
        }
    }

    let mut pending_edges = dependencies.into_iter().collect::<Vec<_>>();
    pending_edges.sort();

    for dependency in &pending_edges {
        if !by_id.contains_key(&dependency.prerequisite) {
            return Err(RealizationError::UnknownRelationDependency {
                relation: dependency.dependent.clone(),
                dependency: dependency.prerequisite.clone(),
            });
        }

        if !by_id.contains_key(&dependency.dependent) {
            return Err(RealizationError::UnknownRelationDependency {
                relation: dependency.dependent.clone(),
                dependency: dependency.dependent.clone(),
            });
        }
    }

    let mut graph = DiGraph::<RelationDeclaration, RelationEdge, u32>::with_capacity(
        by_id.len(),
        pending_edges.len(),
    );
    let mut node_by_id = BTreeMap::new();

    for declaration in by_id.values() {
        let node = graph.add_node(declaration.clone());
        node_by_id.insert(declaration.id.clone(), node);
    }

    for dependency in pending_edges {
        let source = node_by_id[&dependency.prerequisite];
        let target = node_by_id[&dependency.dependent];
        graph.add_edge(source, target, dependency.edge);
    }

    let evaluation_order = toposort(&graph, None)
        .map_err(|_cycle| RealizationError::RelationDependencyCycle {
            components: cyclic_relation_components(&graph),
        })?
        .into_iter()
        .map(|node| graph[node].id.clone())
        .collect();

    Ok((graph, node_by_id, evaluation_order))
}

/// Project a direct Petgraph relation graph into stable typed values.
#[must_use]
pub fn project_relation_graph(
    graph: &DiGraph<RelationDeclaration, RelationEdge, u32>,
) -> RelationGraphProjection {
    let mut nodes = graph.node_weights().cloned().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    let mut edges = graph
        .edge_references()
        .map(|edge| RelationDependencyProjection {
            source: graph[edge.source()].id.clone(),
            target: graph[edge.target()].id.clone(),
            edge: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();

    RelationGraphProjection { nodes, edges }
}

fn cyclic_relation_components(
    graph: &DiGraph<RelationDeclaration, RelationEdge, u32>,
) -> Vec<Vec<RelationId>> {
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
