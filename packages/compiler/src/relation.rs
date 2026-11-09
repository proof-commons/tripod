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

// The item-level allowances below cover the Petgraph index map and the
// evaluation order, retained beside the graph as diagnostic structure
// the canonical analysis does not read.

use std::collections::BTreeMap;

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{
    CompileError, input::BoundCompilerInput, sponsor_region::validate_decidable_sponsor_regions,
};

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
    #[allow(dead_code)]
    pub node_by_id: BTreeMap<AnalysisNodeId, NodeIndex<u32>>,
    #[allow(dead_code)]
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
    let analysis = input.analyze_relation_records()?;

    // The single choke point every analysis path passes through, which
    // is why the sponsor-region gate sits here. Stages below this one
    // ask whether an ordinary-L-BTC reference is erased; for an
    // operation whose protocol flows claim that family the declarations
    // do not answer, and the answer is not one to guess. Refusing the
    // scope here is what makes "ordinary L-BTC is the sponsor region" a
    // property of every analysis the compiler accepts rather than an
    // assumption those stages quietly rest on.
    validate_decidable_sponsor_regions(&analysis, input.scope().operations())?;

    Ok(analysis)
}

/// Build one scoped relation graph from explicit source declarations.
///
/// Separated from [`build_relation_analysis`] so tests can exercise
/// every rejection path with synthetic declarations a validated
/// realization could never carry.
#[cfg(test)]
pub fn build_relation_graph(
    scope: &[architecture::OperationId],
    source_nodes: &[realization::RelationDeclaration],
    source_edges: &[realization::RelationDependencyProjection],
) -> Result<CompilerRelationAnalysis, CompileError> {
    crate::engine_attestation::lower_relations(scope, source_nodes, source_edges).analyze()
}
