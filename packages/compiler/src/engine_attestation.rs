//! Attestation-owned lowering and lossless compatibility for relation records.
//!
//! The binder fixes this owner once. Re-derivation lowers its retained source
//! again; neither the graph algorithm nor its compatibility view selects an owner.

use std::collections::BTreeMap;

use architecture::OperationId;
use realization::{
    RelationDeclaration, RelationDependencyProjection, RelationEdge, RelationId, RelationKind,
    RelationSubject,
};

use crate::{
    CompileError,
    engine_records::{
        self, DependencyKind, DependencyRecord, OperationKey, RelationKey, RelationRecord,
        RelationRecordError, RelationRecords,
    },
    relation::{
        AnalysisNodeId, CompilerRelationAnalysis, CompilerRelationEdge, CompilerRelationNode,
        RelationCycleComponent,
    },
};

use crate::engine_identity::{ATTESTATION, side_code};
use crate::input::AttestationOwner;

pub mod case_facts;
pub mod cases;
pub mod constructibility;
pub mod coverage;
pub mod coverage_join;
pub mod foundation;
pub mod identity;
pub mod lifecycle;
pub mod operation;
pub mod placement;
pub mod program;
pub mod proof;
pub mod requirement;
pub mod source;

impl engine_records::RelationOwner for AttestationOwner {
    type Lowered = AttestationRelations;
    type Error = CompileError;
    fn source_binding(&self) -> crate::engine_program::Source {
        crate::engine_program::Source::Attestation(Box::new(crate::analyzed::AnalyzedSource {
            architecture: self.source().project().architecture,
            realization: self.source().project(),
            compilation_scope: self.scope().clone(),
        }))
    }
    #[cfg(test)]
    fn lower_requirements(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<Vec<crate::engine_requirement::Table>, CompileError> {
        requirement::tables(input)
    }
    #[cfg(test)]
    fn lower_predicates(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_foundation::Records, CompileError> {
        let relations = crate::relation::build_relation_analysis(input)?;
        let expressions = crate::expression::build_expression_analysis(input)?;
        Ok(foundation::lower(&relations, &expressions).records)
    }
    #[cfg(test)]
    fn lower_proofs(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_proof::Records, CompileError> {
        Ok(proof::lower(input)?.records)
    }
    #[cfg(test)]
    fn lower_cases(
        input: &crate::input::BoundCompilerInput<Self>,
    ) -> Result<crate::engine_cases::Tables, CompileError> {
        cases::lower(input)
    }
    fn lower_sources(&self) -> Result<crate::engine_source::SourceRecords, CompileError> {
        source::lower(self)
    }
    #[cfg(test)]
    fn lower_constructibility(
        &self,
    ) -> Result<crate::engine_constructibility::Records, CompileError> {
        Ok(constructibility::lower(self)?.records)
    }
    fn lower_relations(&self) -> Result<Self::Lowered, CompileError> {
        let source = self.source().project();
        Ok(lower_relations(
            self.scope().operations(),
            &source.relations.nodes,
            &source.relations.edges,
            &source.lifecycle,
        ))
    }
}

pub struct AttestationRelations {
    records: RelationRecords,
    lifecycle: lifecycle::LoweredLifecycle,
    identities: BTreeMap<RelationKey, RelationId>,
    declarations: BTreeMap<RelationKey, RelationDeclaration>,
    dependency_kinds: BTreeMap<DependencyKind, RelationEdge>,
}

impl engine_records::LoweredRelations for AttestationRelations {
    fn records(&self) -> &RelationRecords {
        &self.records
    }

    fn lifecycle(&self) -> &crate::engine_lifecycle::LifecycleRecords {
        &self.lifecycle.records
    }
}

/// Retain the entire source declaration beside its graph-facing records.
/// No vocabulary decision is delegated back from the graph to this owner.
pub fn lower_relations(
    scope: &[OperationId],
    nodes: &[RelationDeclaration],
    edges: &[RelationDependencyProjection],
    lifecycle_projection: &realization::LifecycleGraphProjection,
) -> AttestationRelations {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_relations", source_node_count = crate::instrumentation::count(nodes.len()), source_edge_count = crate::instrumentation::count(edges.len()));
    let _entered = span.enter();
    let lifecycle = lifecycle::lower(
        scope,
        nodes
            .iter()
            .filter(|node| scope.contains(&node.id.operation())),
        lifecycle_projection,
    );
    let mut identities = BTreeMap::new();
    let mut register = |id: &RelationId| {
        let key = relation_key(id);
        identities.insert(key.clone(), id.clone());
        key
    };
    let mut declarations = BTreeMap::new();
    let nodes = nodes
        .iter()
        .map(|declaration| {
            let id = register(&declaration.id);
            declarations.insert(id.clone(), declaration.clone());
            RelationRecord {
                id,
                proof_owners: declaration
                    .proof_alternatives
                    .iter()
                    .map(|alternative| register(alternative.relation()))
                    .collect(),
            }
        })
        .collect();
    let mut dependency_kinds = BTreeMap::new();
    let edges = edges
        .iter()
        .map(|edge| {
            let kind = dependency_kind(edge.edge);
            dependency_kinds.insert(kind, edge.edge);
            DependencyRecord {
                source: register(&edge.source),
                target: register(&edge.target),
                kind,
            }
        })
        .collect();
    tracing::debug!(target: "compiler::engine_attestation", outcome = "completed", identity_count = crate::instrumentation::count(identities.len()), "relation records lowered");
    AttestationRelations {
        lifecycle,
        records: RelationRecords {
            scope: scope.iter().copied().map(operation_key).collect(),
            nodes,
            edges,
        },
        identities,
        declarations,
        dependency_kinds,
    }
}

impl AttestationRelations {
    #[cfg(test)]
    pub fn lifecycle_compatibility_view(
        &self,
        result: Result<
            crate::engine_lifecycle::LifecycleAnalysis,
            crate::engine_lifecycle::LifecycleRecordError,
        >,
    ) -> Result<crate::lifecycle::CompilerLifecycleAnalysis, CompileError> {
        self.lifecycle.compatibility_view(result)
    }
    #[cfg(test)]
    pub fn analyze(&self) -> Result<CompilerRelationAnalysis, CompileError> {
        self.compatibility_view(engine_records::analyze_relations(&self.records))
    }

    /// Map the analyzed graph without repeating its construction or validation.
    /// Every returned identity originated in the retained lowering registry.
    pub fn compatibility_view(
        &self,
        result: Result<engine_records::RelationGraph, RelationRecordError>,
    ) -> Result<CompilerRelationAnalysis, CompileError> {
        let analysis = result.map_err(|error| self.refusal(error))?;
        Ok(CompilerRelationAnalysis {
            graph: analysis.graph.map(
                |_, record| CompilerRelationNode {
                    id: AnalysisNodeId::SourceRelation(self.identities[&record.id].clone()),
                    source: self.declarations[&record.id].clone(),
                },
                |_, kind| CompilerRelationEdge::SourceDependency(self.dependency_kinds[kind]),
            ),
            node_by_id: analysis
                .node_by_id
                .into_iter()
                .map(|(key, node)| {
                    (
                        AnalysisNodeId::SourceRelation(self.identities[&key].clone()),
                        node,
                    )
                })
                .collect(),
            evaluation_order: analysis
                .evaluation_order
                .into_iter()
                .map(|key| AnalysisNodeId::SourceRelation(self.identities[&key].clone()))
                .collect(),
        })
    }

    pub(super) fn refusal(&self, error: RelationRecordError) -> CompileError {
        let id = |key: &RelationKey| self.identities[key].clone();
        match error {
            RelationRecordError::ForeignProofOwner {
                relation,
                alternative_relation,
            } => CompileError::ForeignProofAlternative {
                relation: id(&relation),
                alternative_relation: id(&alternative_relation),
            },
            RelationRecordError::DuplicateRelation { relation } => {
                CompileError::DuplicateRelation {
                    relation: id(&relation),
                }
            }
            RelationRecordError::UnknownEndpoint { relation } => {
                CompileError::UnknownRelationDependencyEndpoint {
                    relation: id(&relation),
                }
            }
            RelationRecordError::EscapesScope {
                prerequisite,
                dependent,
            } => CompileError::RelationDependencyEscapesScope {
                prerequisite: id(&prerequisite),
                dependent: id(&dependent),
            },
            RelationRecordError::DuplicateDependency {
                prerequisite,
                dependent,
                kind,
            } => CompileError::DuplicateRelationDependency {
                prerequisite: id(&prerequisite),
                dependent: id(&dependent),
                edge: self.dependency_kinds[&kind],
            },
            RelationRecordError::Cycle { components } => CompileError::RelationDependencyCycle {
                components: components
                    .into_iter()
                    .map(|component| RelationCycleComponent {
                        members: component.members.iter().map(id).collect(),
                        internal_edges: component
                            .internal_edges
                            .into_iter()
                            .map(|edge| RelationDependencyProjection {
                                source: id(&edge.source),
                                target: id(&edge.target),
                                edge: self.dependency_kinds[&edge.kind],
                            })
                            .collect(),
                    })
                    .collect(),
            },
            RelationRecordError::CensusMismatch {
                missing,
                unexpected,
            } => CompileError::RelationCensusMismatch {
                missing: missing.iter().map(id).collect(),
                unexpected: unexpected.iter().map(id).collect(),
            },
        }
    }
}

pub const fn operation_key(operation: OperationId) -> OperationKey {
    OperationKey {
        namespace: ATTESTATION,
        code: operation.code(),
    }
}

// These are identifier encodings, not semantic classifiers. The order of
// each code follows the existing derived ordering, including subject tags.
pub fn relation_key(id: &RelationId) -> RelationKey {
    let kind = match id.kind() {
        RelationKind::Cardinality => 0,
        RelationKind::Recognition => 1,
        RelationKind::Authorization => 2,
        RelationKind::Conservation => 3,
        RelationKind::AllowedObjectFamilies => 4,
        RelationKind::CanonicalDeltaPolicy => 5,
        RelationKind::OpenFlowPolicy => 6,
        RelationKind::SponsorEnvelopeMultiplicity => 7,
        RelationKind::SponsorIsolation => 8,
        RelationKind::RootPolicy => 9,
        RelationKind::ProjectionPolicy => 10,
        RelationKind::Constructibility => 11,
        RelationKind::Lifecycle => 12,
        RelationKind::Representation => 13,
        RelationKind::SubstrateConservation => 14,
        RelationKind::ExpressionPredicate => 15,
    };
    let subject = match id.subject() {
        RelationSubject::Operation => vec![0],
        RelationSubject::ObjectFamily { side, object } => vec![1, side_code(*side), object.code()],
        RelationSubject::TransactionSide { side } => vec![2, side_code(*side)],
        RelationSubject::Asset { asset } => vec![3, asset.code()],
        RelationSubject::Root { root } => vec![4, root.code()],
        RelationSubject::Projection { projection } => vec![5, projection.code()],
        RelationSubject::Sponsor => vec![6],
        RelationSubject::LifecycleExit { object, exit } => vec![7, object.code(), exit.code()],
        RelationSubject::Representation { object } => vec![8, object.code()],
    };
    RelationKey {
        operation: operation_key(id.operation()),
        kind,
        subject,
    }
}

const fn dependency_kind(edge: RelationEdge) -> DependencyKind {
    let code = match edge {
        RelationEdge::RecognitionBeforeCardinality => 0,
        RelationEdge::RecognitionBeforeValue => 1,
        RelationEdge::CardinalityBeforeValue => 2,
        RelationEdge::AuthorizationBeforeClosure => 3,
        RelationEdge::AuthorizationBeforeConstructibility => 4,
        RelationEdge::SponsorBeforeConstructibility => 5,
        RelationEdge::SponsorBeforeOperation => 6,
        RelationEdge::OpenFlowPolicyBeforeSponsor => 7,
        RelationEdge::CanonicalDeltaBeforeValue => 8,
        RelationEdge::RepresentationBeforeLifecycle => 9,
        RelationEdge::ProjectionPolicyBeforeOperation => 10,
        RelationEdge::RootPolicyBeforeOperation => 11,
        RelationEdge::StaticRequirement => 12,
    };
    DependencyKind {
        namespace: ATTESTATION,
        code,
    }
}
