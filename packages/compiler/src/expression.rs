//! The canonical scoped expression graph (P2-005) and its fold slots.
//!
//! Constant folding needs a canonical source expression graph. The
//! compiler retains realization-owned [`realization::ExprId`]s directly
//! — no compiler-local expression identity exists — and preserves
//! every source dependency edge as provenance even once a node's value
//! is statically known. The optional folded value is derived analysis,
//! not a new identity.
//!
//! Scope closure is exact: every expression owned by an in-scope
//! operation enters, together with the operation-independent ancestors
//! it transitively requires — and nothing else. A dependency on an
//! expression owned by an out-of-scope operation fails closed.

use std::collections::{BTreeMap, BTreeSet};

use petgraph::{
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{CompileError, input::BoundCompilerInput};

/// One expression node: the complete source declaration plus the
/// optional conservatively folded value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerExpressionNode {
    pub source: realization::ExpressionDeclaration,
    pub folded_value: Option<realization::SemanticValue>,
}

/// One expression dependency, preserving the source role exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilerExpressionEdge {
    SourceDependency(realization::DependencyEdge),
}

/// Stable projection of one expression node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerExpressionNodeProjection {
    pub id: realization::ExprId,
    pub ty: realization::SemanticType,
    pub source: realization::ExpressionNode,
    pub folded_value: Option<realization::SemanticValue>,
}

/// Stable projection of one expression dependency edge.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompilerExpressionDependencyProjection {
    pub source: realization::ExprId,
    pub target: realization::ExprId,
    pub edge: CompilerExpressionEdge,
}

/// Canonical typed projection of the compiler expression graph.
///
/// Nodes sort by expression ID; edges by source, target, and edge
/// role. No local handle, topological order, or float appears here;
/// the source node is retained even when folded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerExpressionGraphProjection {
    pub nodes: Vec<CompilerExpressionNodeProjection>,
    pub edges: Vec<CompilerExpressionDependencyProjection>,
}

/// One canonically ordered cyclic strongly connected component.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpressionCycleComponent {
    pub members: Vec<realization::ExprId>,
    pub internal_edges: Vec<realization::ExpressionDependencyProjection>,
}

/// Internal expression analysis: a direct Petgraph graph plus typed
/// lookup metadata. Local indices and the topological schedule never
/// leave this structure.
#[derive(Debug)]
pub struct CompilerExpressionAnalysis {
    pub graph: DiGraph<CompilerExpressionNode, CompilerExpressionEdge, u32>,
    pub node_by_id: BTreeMap<realization::ExprId, NodeIndex<u32>>,
    pub evaluation_order: Vec<realization::ExprId>,
}

impl CompilerExpressionAnalysis {
    /// Project into stable typed values for comparison and tests.
    #[must_use]
    pub fn project(&self) -> CompilerExpressionGraphProjection {
        let mut nodes = self
            .graph
            .node_weights()
            .map(|node| CompilerExpressionNodeProjection {
                id: node.source.id.clone(),
                ty: node.source.ty,
                source: node.source.node.clone(),
                folded_value: node.folded_value.clone(),
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.id.cmp(&right.id));

        let mut edges = self
            .graph
            .edge_references()
            .map(|edge| CompilerExpressionDependencyProjection {
                source: self.graph[edge.source()].source.id.clone(),
                target: self.graph[edge.target()].source.id.clone(),
                edge: *edge.weight(),
            })
            .collect::<Vec<_>>();
        edges.sort();

        CompilerExpressionGraphProjection { nodes, edges }
    }
}

/// Owning operation of one expression, if any.
///
/// Relation-owned expressions belong to their relation's operation;
/// fact expressions belong to their fact's operation; architecture
/// bound-value facts are operation-independent.
pub const fn expression_operation(id: &realization::ExprId) -> Option<architecture::OperationId> {
    match id {
        realization::ExprId::Relation { relation, .. } => Some(relation.operation()),
        realization::ExprId::Fact(fact) => fact.operation(),
    }
}

/// Build the exact scoped expression graph from a validated input.
pub fn build_expression_analysis(
    input: &BoundCompilerInput,
) -> Result<CompilerExpressionAnalysis, CompileError> {
    let source = input.realization().project();

    build_expression_graph(
        input.scope().operations(),
        &source.expressions.nodes,
        &source.expressions.edges,
    )
}

/// Build one scoped expression graph from explicit source declarations.
///
/// Separated from [`build_expression_analysis`] so tests can exercise
/// every rejection path with synthetic declarations. The empty graph
/// is valid: the current pilots declare no expressions.
pub fn build_expression_graph(
    scope: &[architecture::OperationId],
    source_nodes: &[realization::ExpressionDeclaration],
    source_edges: &[realization::ExpressionDependencyProjection],
) -> Result<CompilerExpressionAnalysis, CompileError> {
    // Index and reject duplicates before any selection decision.
    let mut by_id = BTreeMap::new();

    for declaration in source_nodes {
        if by_id
            .insert(declaration.id.clone(), declaration.clone())
            .is_some()
        {
            return Err(CompileError::DuplicateExpression {
                expression: declaration.id.clone(),
            });
        }
    }

    // Dependency adjacency: consumer -> its dependency edges.
    let mut incoming: BTreeMap<
        realization::ExprId,
        Vec<&realization::ExpressionDependencyProjection>,
    > = BTreeMap::new();
    let mut sorted_edges = source_edges.iter().collect::<Vec<_>>();
    sorted_edges.sort();

    for pair in sorted_edges.windows(2) {
        if pair[0] == pair[1] {
            return Err(CompileError::DuplicateExpressionDependency {
                dependency: pair[0].source.clone(),
                consumer: pair[0].target.clone(),
            });
        }
    }

    for edge in &sorted_edges {
        incoming.entry(edge.target.clone()).or_default().push(edge);
    }

    let closure = select_scoped_closure(scope, &by_id, &incoming)?;

    // Canonical insertion: nodes in stable expression-ID order, then
    // the closure-internal edges in sorted order.
    let expected_ids = closure.clone();
    let selected_edges = sorted_edges
        .iter()
        .filter(|edge| closure.contains(&edge.source) && closure.contains(&edge.target))
        .collect::<Vec<_>>();

    let mut graph = DiGraph::<CompilerExpressionNode, CompilerExpressionEdge, u32>::with_capacity(
        closure.len(),
        selected_edges.len(),
    );
    let mut node_by_id = BTreeMap::new();

    for id in &closure {
        let node = graph.add_node(CompilerExpressionNode {
            source: by_id[id].clone(),
            folded_value: None,
        });

        node_by_id.insert(id.clone(), node);
    }

    for edge in selected_edges {
        graph.add_edge(
            node_by_id[&edge.source],
            node_by_id[&edge.target],
            CompilerExpressionEdge::SourceDependency(edge.edge),
        );
    }

    let evaluation_order = toposort(&graph, None)
        .map_err(|_cycle| CompileError::ExpressionDependencyCycle {
            components: cyclic_expression_components(&graph),
        })?
        .into_iter()
        .map(|node| graph[node].source.id.clone())
        .collect();

    let graph_ids = graph
        .node_weights()
        .map(|node| node.source.id.clone())
        .collect::<BTreeSet<_>>();

    if expected_ids != graph_ids {
        return Err(CompileError::ExpressionCensusMismatch {
            missing: expected_ids.difference(&graph_ids).cloned().collect(),
            unexpected: graph_ids.difference(&expected_ids).cloned().collect(),
        });
    }

    Ok(CompilerExpressionAnalysis {
        graph,
        node_by_id,
        evaluation_order,
    })
}

/// Seed with every expression owned by an in-scope operation, then
/// close over operation-independent ancestors; a dependency owned by
/// another operation fails closed.
fn select_scoped_closure(
    scope: &[architecture::OperationId],
    by_id: &BTreeMap<realization::ExprId, realization::ExpressionDeclaration>,
    incoming: &BTreeMap<realization::ExprId, Vec<&realization::ExpressionDependencyProjection>>,
) -> Result<BTreeSet<realization::ExprId>, CompileError> {
    let in_scope = |operation: architecture::OperationId| scope.contains(&operation);
    let mut closure = BTreeSet::new();
    let mut worklist = by_id
        .keys()
        .filter(|id| expression_operation(id).is_some_and(in_scope))
        .cloned()
        .collect::<Vec<_>>();

    while let Some(consumer) = worklist.pop() {
        if !closure.insert(consumer.clone()) {
            continue;
        }

        for edge in incoming.get(&consumer).into_iter().flatten() {
            let dependency = &edge.source;

            if !by_id.contains_key(dependency) {
                return Err(CompileError::UnknownExpressionDependencyEndpoint {
                    expression: dependency.clone(),
                });
            }

            match expression_operation(dependency) {
                Some(operation) if in_scope(operation) => worklist.push(dependency.clone()),
                Some(_) => {
                    return Err(CompileError::ExpressionDependencyEscapesScope {
                        dependency: dependency.clone(),
                        consumer: consumer.clone(),
                    });
                }
                None => worklist.push(dependency.clone()),
            }
        }
    }

    Ok(closure)
}

fn cyclic_expression_components(
    graph: &DiGraph<CompilerExpressionNode, CompilerExpressionEdge, u32>,
) -> Vec<ExpressionCycleComponent> {
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
                    let CompilerExpressionEdge::SourceDependency(role) = *edge.weight();

                    realization::ExpressionDependencyProjection {
                        source: graph[edge.source()].source.id.clone(),
                        target: graph[edge.target()].source.id.clone(),
                        edge: role,
                    }
                })
                .collect::<Vec<_>>();
            internal_edges.sort();

            ExpressionCycleComponent {
                members,
                internal_edges,
            }
        })
        .collect::<Vec<_>>();

    components.sort();
    components
}
