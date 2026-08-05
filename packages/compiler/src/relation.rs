//! The canonical compiler relation DAG (C1-005 / P2-005).
//!
//! The compiler's fixed semantic census: every realization relation in
//! compiler scope appears exactly once, with its complete source
//! declaration, and every in-scope source dependency appears exactly
//! once. Nothing is omitted, duplicated, merged, reassigned, or
//! weakened — a relation the compiler cannot yet discharge (the T2
//! substrate-conservation requirement included) is preserved, not
//! dropped.
//!
//! Petgraph indices are local handles only: they never enter stable
//! projections, diagnostics, or identities. The internal topological
//! schedule is an implementation order, not a semantic fact, and is
//! excluded from the stable projection for the same reason the
//! realization excludes it.

// The analysis stages have no non-test consumer until proof
// planning (P2-007) and the P2-012 analyzed program; unit tests
// exercise them until then. Remove with the first real consumer.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use petgraph::{
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{CompileError, input::BoundCompilerInput};

/// Stable identity of one compiler analysis node.
///
/// The wrapped relation ID remains realization-owned; this type names
/// the compiler-graph role in which it appears. Later analysis stages
/// extend the enum with compiler-owned roles; no speculative variant
/// exists now.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnalysisNodeId {
    SourceRelation(realization::RelationId),
}

/// One relation node: the complete source declaration, preserved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerRelationNode {
    pub id: AnalysisNodeId,
    pub source: realization::RelationDeclaration,
}

/// One relation dependency, preserving the source edge kind exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilerRelationEdge {
    SourceDependency(realization::RelationEdge),
}

/// Stable projection of one relation node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerRelationNodeProjection {
    pub id: AnalysisNodeId,
    pub source: realization::RelationDeclaration,
}

/// Stable projection of one relation dependency edge.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompilerRelationDependencyProjection {
    pub source: AnalysisNodeId,
    pub target: AnalysisNodeId,
    pub edge: CompilerRelationEdge,
}

/// Canonical typed projection of the compiler relation graph.
///
/// Nodes sort by [`AnalysisNodeId`]; edges by source, target, and edge
/// kind. No Petgraph index, evaluation order, insertion sequence,
/// path, or hash appears here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerRelationGraphProjection {
    pub nodes: Vec<CompilerRelationNodeProjection>,
    pub edges: Vec<CompilerRelationDependencyProjection>,
}

/// One canonically ordered cyclic strongly connected component.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationCycleComponent {
    pub members: Vec<realization::RelationId>,
    pub internal_edges: Vec<realization::RelationDependencyProjection>,
}

/// Internal relation analysis: a direct Petgraph graph plus typed
/// lookup metadata. Local indices and the topological schedule never
/// leave this structure.
#[derive(Debug)]
pub struct CompilerRelationAnalysis {
    pub graph: DiGraph<CompilerRelationNode, CompilerRelationEdge, u32>,
    pub node_by_id: BTreeMap<AnalysisNodeId, NodeIndex<u32>>,
    pub evaluation_order: Vec<AnalysisNodeId>,
}

impl CompilerRelationAnalysis {
    /// Project into stable typed values for comparison and tests.
    #[must_use]
    pub fn project(&self) -> CompilerRelationGraphProjection {
        let mut nodes = self
            .graph
            .node_weights()
            .map(|node| CompilerRelationNodeProjection {
                id: node.id.clone(),
                source: node.source.clone(),
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.id.cmp(&right.id));

        let mut edges = self
            .graph
            .edge_references()
            .map(|edge| CompilerRelationDependencyProjection {
                source: self.graph[edge.source()].id.clone(),
                target: self.graph[edge.target()].id.clone(),
                edge: *edge.weight(),
            })
            .collect::<Vec<_>>();
        edges.sort();

        CompilerRelationGraphProjection { nodes, edges }
    }
}

/// Build the exact scoped relation DAG from a validated compiler input.
pub fn build_relation_analysis(
    input: &BoundCompilerInput,
) -> Result<CompilerRelationAnalysis, CompileError> {
    let source = input.realization().project();

    build_relation_graph(
        input.scope().operations(),
        &source.relations.nodes,
        &source.relations.edges,
    )
}

/// Build one scoped relation graph from explicit source declarations.
///
/// Separated from [`build_relation_analysis`] so tests can exercise
/// every rejection path with synthetic declarations a validated
/// realization could never carry.
pub fn build_relation_graph(
    scope: &[architecture::OperationId],
    source_nodes: &[realization::RelationDeclaration],
    source_edges: &[realization::RelationDependencyProjection],
) -> Result<CompilerRelationAnalysis, CompileError> {
    let in_scope = |operation: architecture::OperationId| scope.contains(&operation);

    // Select by explicit compiler scope — never by reachability.
    let mut selected = BTreeMap::new();
    let mut expected_ids = BTreeSet::new();

    for node in source_nodes {
        if !in_scope(node.id.operation()) {
            continue;
        }

        validate_relation_ownership(node)?;
        expected_ids.insert(node.id.clone());

        if selected.insert(node.id.clone(), node.clone()).is_some() {
            return Err(CompileError::DuplicateRelation {
                relation: node.id.clone(),
            });
        }
    }

    // Classify every source dependency against the compiler boundary.
    let mut pending_edges = Vec::new();

    for dependency in source_edges {
        let prerequisite_in = expected_ids.contains(&dependency.source);
        let dependent_in = expected_ids.contains(&dependency.target);
        let prerequisite_scoped = in_scope(dependency.source.operation());
        let dependent_scoped = in_scope(dependency.target.operation());

        match (prerequisite_scoped, dependent_scoped) {
            (false, false) => {}
            (true, true) => {
                if !prerequisite_in || !dependent_in {
                    let relation = if prerequisite_in {
                        dependency.target.clone()
                    } else {
                        dependency.source.clone()
                    };

                    return Err(CompileError::UnknownRelationDependencyEndpoint { relation });
                }

                pending_edges.push(dependency.clone());
            }
            _ => {
                return Err(CompileError::RelationDependencyEscapesScope {
                    prerequisite: dependency.source.clone(),
                    dependent: dependency.target.clone(),
                });
            }
        }
    }

    pending_edges.sort();

    for pair in pending_edges.windows(2) {
        if pair[0] == pair[1] {
            return Err(CompileError::DuplicateRelationDependency {
                prerequisite: pair[0].source.clone(),
                dependent: pair[0].target.clone(),
                edge: pair[0].edge,
            });
        }
    }

    // Canonical insertion: nodes in stable relation-ID order (the
    // BTreeMap iteration order), then edges in sorted order.
    let mut graph = DiGraph::<CompilerRelationNode, CompilerRelationEdge, u32>::with_capacity(
        selected.len(),
        pending_edges.len(),
    );
    let mut node_by_id = BTreeMap::new();

    for declaration in selected.values() {
        let id = AnalysisNodeId::SourceRelation(declaration.id.clone());
        let node = graph.add_node(CompilerRelationNode {
            id: id.clone(),
            source: declaration.clone(),
        });

        node_by_id.insert(id, node);
    }

    for dependency in &pending_edges {
        let source_node = node_by_id[&AnalysisNodeId::SourceRelation(dependency.source.clone())];
        let target_node = node_by_id[&AnalysisNodeId::SourceRelation(dependency.target.clone())];

        graph.add_edge(
            source_node,
            target_node,
            CompilerRelationEdge::SourceDependency(dependency.edge),
        );
    }

    let evaluation_order = toposort(&graph, None)
        .map_err(|_cycle| CompileError::RelationDependencyCycle {
            components: cyclic_relation_components(&graph),
        })?
        .into_iter()
        .map(|node| graph[node].id.clone())
        .collect();

    // Exact census: source scope, graph, and expectation must agree.
    let graph_ids = graph
        .node_weights()
        .map(|node| node.source.id.clone())
        .collect::<BTreeSet<_>>();

    if expected_ids != graph_ids {
        return Err(CompileError::RelationCensusMismatch {
            missing: expected_ids.difference(&graph_ids).cloned().collect(),
            unexpected: graph_ids.difference(&expected_ids).cloned().collect(),
        });
    }

    Ok(CompilerRelationAnalysis {
        graph,
        node_by_id,
        evaluation_order,
    })
}

/// Compiler-side ownership revalidation: input is owner-controlled,
/// so the compiler does not depend on an earlier caller's validation
/// having run.
fn validate_relation_ownership(
    node: &realization::RelationDeclaration,
) -> Result<(), CompileError> {
    for alternative in &node.proof_alternatives {
        if alternative.relation() != &node.id {
            return Err(CompileError::ForeignProofAlternative {
                relation: node.id.clone(),
                alternative_relation: alternative.relation().clone(),
            });
        }
    }

    Ok(())
}

fn cyclic_relation_components(
    graph: &DiGraph<CompilerRelationNode, CompilerRelationEdge, u32>,
) -> Vec<RelationCycleComponent> {
    let mut components = kosaraju_scc(graph)
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || component
                    .first()
                    .is_some_and(|node| graph.find_edge(*node, *node).is_some())
        })
        .map(|component| {
            let component_nodes = component.iter().copied().collect::<BTreeSet<_>>();
            let mut members = component
                .iter()
                .map(|node| graph[*node].source.id.clone())
                .collect::<Vec<_>>();
            members.sort();

            let mut internal_edges = graph
                .edge_references()
                .filter(|edge| {
                    component_nodes.contains(&edge.source())
                        && component_nodes.contains(&edge.target())
                })
                .map(|edge| {
                    let CompilerRelationEdge::SourceDependency(kind) = *edge.weight();

                    realization::RelationDependencyProjection {
                        source: graph[edge.source()].source.id.clone(),
                        target: graph[edge.target()].source.id.clone(),
                        edge: kind,
                    }
                })
                .collect::<Vec<_>>();
            internal_edges.sort();

            RelationCycleComponent {
                members,
                internal_edges,
            }
        })
        .collect::<Vec<_>>();

    components.sort();
    components
}
