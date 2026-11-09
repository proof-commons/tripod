//! Crate-private relation records consumed without consulting an owner vocabulary.
//!
//! Fixed-owner lowering supplies all shared stages before search. Keys are
//! complete namespace-qualified owner identities, never positions or hashes.

use std::collections::{BTreeMap, BTreeSet};

use petgraph::{
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

/// Typed owner refusal retained by the binding-fixed producer.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LoweringError {
    #[error("attestation lowering refused: {0}")]
    Attestation(#[from] crate::CompileError),
    #[cfg(any(test, feature = "keyed-experimental"))]
    #[error("keyed lowering refused: {0:?}")]
    Keyed(#[from] crate::engine_keyed::KeyedLoweringError),
}

/// Static lowering authority retained by the input binder. This parameter is
/// not an engine identity parameter and provides no per-case policy callbacks.
pub trait RelationOwner {
    type Lowered: LoweredRelations;
    type Error: Into<LoweringError>;
    fn source_binding(&self) -> crate::engine_program::Source;
    #[cfg(test)]
    fn lower_requirements(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<Vec<crate::engine_requirement::Table>, Self::Error>
    where
        Self: Sized;
    #[cfg(test)]
    fn lower_predicates(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_foundation::Records, Self::Error>
    where
        Self: Sized;
    fn lower_relations(&self) -> Result<Self::Lowered, Self::Error>;
    #[cfg(test)]
    fn lower_proofs(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_proof::Records, Self::Error>
    where
        Self: Sized;
    #[cfg(test)]
    fn lower_cases(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_cases::Tables, Self::Error>
    where
        Self: Sized;
    fn lower_sources(&self) -> Result<crate::engine_source::SourceRecords, Self::Error>;
    #[cfg(test)]
    fn lower_constructibility(
        &self,
    ) -> Result<crate::engine_constructibility::Records, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
#[cfg(test)]
pub enum ProofFailure {
    #[error("proof lowering refused: {0:?}")]
    Lowering(LoweringError),
    #[error("proof analysis refused: {0:?}")]
    Analysis(crate::engine_proof::Error),
}
#[cfg(test)]
pub fn analyze_bound_proofs<O: RelationOwner>(
    input: &crate::input::BoundCompilerInput<O>,
    view: &crate::capability::CapabilityView,
) -> Result<
    (
        Vec<crate::engine_proof::Candidate>,
        crate::proof::ProofSearchReport,
    ),
    ProofFailure,
> {
    let records = O::lower_proofs(input).map_err(|error| ProofFailure::Lowering(error.into()))?;
    let mut report = crate::proof::ProofSearchReport::default();
    let candidates = crate::engine_proof::enumerate(
        &records,
        view,
        input.policy().proof_search_limits,
        &mut report,
    )
    .map_err(ProofFailure::Analysis)?;
    Ok((candidates, report))
}

#[derive(Debug, thiserror::Error)]
#[cfg(test)]
pub enum SourceFailure {
    #[error("source lowering refused: {0:?}")]
    Lowering(LoweringError),
    #[error("source analysis refused: {0:?}")]
    Analysis(Box<crate::engine_source::SourceError>),
}
#[cfg(test)]
pub fn analyze_bound_sources<O: RelationOwner>(
    input: &crate::input::BoundCompilerInput<O>,
) -> Result<Vec<Vec<crate::engine_source::SourceRow>>, SourceFailure> {
    let records = input
        .owner()
        .lower_sources()
        .map_err(|error| SourceFailure::Lowering(error.into()))?;
    crate::engine_source::analyze_sources(&records)
        .map_err(|error| SourceFailure::Analysis(Box::new(error)))
}

#[derive(Debug, thiserror::Error)]
#[cfg(test)]
pub enum ConstructibilityFailure {
    #[error("constructibility lowering refused: {0:?}")]
    Lowering(LoweringError),
    #[error("constructibility analysis refused: {0:?}")]
    Analysis(Box<crate::engine_constructibility::RecordError>),
}

#[cfg(test)]
pub fn analyze_bound_constructibility<O: RelationOwner>(
    input: &crate::input::BoundCompilerInput<O>,
) -> Result<crate::engine_constructibility::Analysis, ConstructibilityFailure> {
    let records = input
        .owner()
        .lower_constructibility()
        .map_err(|error| ConstructibilityFailure::Lowering(error.into()))?;
    crate::engine_constructibility::analyze_constructibility(&records)
        .map_err(|error| ConstructibilityFailure::Analysis(Box::new(error)))
}

pub trait LoweredRelations {
    fn records(&self) -> &RelationRecords;
    fn lifecycle(&self) -> &crate::engine_lifecycle::LifecycleRecords;
}

/// The lowering remains beside the outcome solely for lossless owner-specific
/// compatibility views and typed refusal payloads.
pub struct BoundRelationAnalysis<L> {
    pub lowering: L,
    pub result: Result<RelationGraph, RelationRecordError>,
}

pub fn analyze_bound_relations<O: RelationOwner>(
    input: &crate::input::BoundCompilerInput<O>,
) -> Result<BoundRelationAnalysis<O::Lowered>, O::Error> {
    let lowering = input.owner().lower_relations()?;
    let result = analyze_relations(lowering.records());
    Ok(BoundRelationAnalysis { lowering, result })
}

// The probe exercises this bound entry for both owners. The existing
// attestation offered-projection API also uses the same concrete traversal.
#[cfg(test)]
pub struct BoundLifecycleAnalysis<L> {
    pub lowering: L,
    pub result: Result<
        crate::engine_lifecycle::LifecycleAnalysis,
        crate::engine_lifecycle::LifecycleRecordError,
    >,
}

#[cfg(test)]
pub fn analyze_bound_lifecycle<O: RelationOwner>(
    input: &crate::input::BoundCompilerInput<O>,
) -> Result<BoundLifecycleAnalysis<O::Lowered>, O::Error> {
    let lowering = input.owner().lower_relations()?;
    let result = crate::engine_lifecycle::analyze_lifecycle(lowering.lifecycle());
    Ok(BoundLifecycleAnalysis { lowering, result })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationKey {
    pub namespace: u16,
    pub code: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationKey {
    pub operation: OperationKey,
    pub kind: u16,
    pub subject: Vec<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DependencyKind {
    pub namespace: u16,
    pub code: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationRecord {
    pub id: RelationKey,
    pub proof_owners: Vec<RelationKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DependencyRecord {
    pub source: RelationKey,
    pub target: RelationKey,
    pub kind: DependencyKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationRecords {
    pub scope: Vec<OperationKey>,
    pub nodes: Vec<RelationRecord>,
    pub edges: Vec<DependencyRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CycleComponent {
    pub members: Vec<RelationKey>,
    pub internal_edges: Vec<DependencyRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationRecordError {
    ForeignProofOwner {
        relation: RelationKey,
        alternative_relation: RelationKey,
    },
    DuplicateRelation {
        relation: RelationKey,
    },
    UnknownEndpoint {
        relation: RelationKey,
    },
    EscapesScope {
        prerequisite: RelationKey,
        dependent: RelationKey,
    },
    DuplicateDependency {
        prerequisite: RelationKey,
        dependent: RelationKey,
        kind: DependencyKind,
    },
    Cycle {
        components: Vec<CycleComponent>,
    },
    CensusMismatch {
        missing: Vec<RelationKey>,
        unexpected: Vec<RelationKey>,
    },
}

#[derive(Clone, Debug)]
pub struct RelationGraph {
    pub graph: DiGraph<RelationRecord, DependencyKind, u32>,
    pub node_by_id: BTreeMap<RelationKey, NodeIndex<u32>>,
    pub evaluation_order: Vec<RelationKey>,
}

/// The existing relation-DAG algorithm over owner-produced records.
/// Namespace and local codes participate only in identity comparisons/order.
pub fn analyze_relations(records: &RelationRecords) -> Result<RelationGraph, RelationRecordError> {
    let span = tracing::debug_span!(target: "compiler::engine_records", "analyze_relations",
        source_node_count = crate::instrumentation::count(records.nodes.len()), source_edge_count = crate::instrumentation::count(records.edges.len()),
        scope_operation_count = crate::instrumentation::count(records.scope.len()));
    let _entered = span.enter();
    build_graph(records)
}

fn build_graph(records: &RelationRecords) -> Result<RelationGraph, RelationRecordError> {
    let in_scope = |operation| records.scope.contains(&operation);
    let mut selected = BTreeMap::new();
    let mut expected_ids = BTreeSet::new();
    for node in &records.nodes {
        if !in_scope(node.id.operation) {
            continue;
        }
        for alternative_relation in &node.proof_owners {
            if alternative_relation != &node.id {
                return Err(RelationRecordError::ForeignProofOwner {
                    relation: node.id.clone(),
                    alternative_relation: alternative_relation.clone(),
                });
            }
        }
        expected_ids.insert(node.id.clone());
        if selected.insert(node.id.clone(), node.clone()).is_some() {
            return Err(RelationRecordError::DuplicateRelation {
                relation: node.id.clone(),
            });
        }
    }
    let mut pending_edges = Vec::new();
    for dependency in &records.edges {
        let prerequisite_in = expected_ids.contains(&dependency.source);
        let dependent_in = expected_ids.contains(&dependency.target);
        match (
            in_scope(dependency.source.operation),
            in_scope(dependency.target.operation),
        ) {
            (false, false) => {}
            (true, true) => {
                if !prerequisite_in || !dependent_in {
                    return Err(RelationRecordError::UnknownEndpoint {
                        relation: if prerequisite_in {
                            dependency.target.clone()
                        } else {
                            dependency.source.clone()
                        },
                    });
                }
                pending_edges.push(dependency.clone());
            }
            _ => {
                return Err(RelationRecordError::EscapesScope {
                    prerequisite: dependency.source.clone(),
                    dependent: dependency.target.clone(),
                });
            }
        }
    }
    pending_edges.sort();
    for pair in pending_edges.windows(2) {
        if pair[0] == pair[1] {
            return Err(RelationRecordError::DuplicateDependency {
                prerequisite: pair[0].source.clone(),
                dependent: pair[0].target.clone(),
                kind: pair[0].kind,
            });
        }
    }
    let mut graph = DiGraph::with_capacity(selected.len(), pending_edges.len());
    let mut node_by_id = BTreeMap::new();
    for declaration in selected.values() {
        let node = graph.add_node(declaration.clone());
        node_by_id.insert(declaration.id.clone(), node);
    }
    for dependency in &pending_edges {
        graph.add_edge(
            node_by_id[&dependency.source],
            node_by_id[&dependency.target],
            dependency.kind,
        );
    }
    let evaluation_order = toposort(&graph, None)
        .map_err(|_| RelationRecordError::Cycle {
            components: cyclic_components(&graph),
        })?
        .into_iter()
        .map(|node| graph[node].id.clone())
        .collect();
    let graph_ids = graph
        .node_weights()
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    if expected_ids != graph_ids {
        return Err(RelationRecordError::CensusMismatch {
            missing: expected_ids.difference(&graph_ids).cloned().collect(),
            unexpected: graph_ids.difference(&expected_ids).cloned().collect(),
        });
    }
    Ok(RelationGraph {
        graph,
        node_by_id,
        evaluation_order,
    })
}

fn cyclic_components(graph: &DiGraph<RelationRecord, DependencyKind, u32>) -> Vec<CycleComponent> {
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
                .map(|node| graph[*node].id.clone())
                .collect::<Vec<_>>();
            members.sort();
            let mut internal_edges = graph
                .edge_references()
                .filter(|edge| {
                    component_nodes.contains(&edge.source())
                        && component_nodes.contains(&edge.target())
                })
                .map(|edge| DependencyRecord {
                    source: graph[edge.source()].id.clone(),
                    target: graph[edge.target()].id.clone(),
                    kind: *edge.weight(),
                })
                .collect::<Vec<_>>();
            internal_edges.sort();
            CycleComponent {
                members,
                internal_edges,
            }
        })
        .collect::<Vec<_>>();
    components.sort();
    components
}
