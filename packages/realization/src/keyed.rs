//! Experimental scoped declaration owner. This is not a history oracle or reader.
//!
//! Exact owner re-derivation catches a coherently removed clause, even if all
//! references to it were also removed. Evidence obligations remain unresolved.

use crate::relation::{KeyedRelationDeclaration, KeyedRelationId};
use architecture::keyed::{KeyedArchitectureBinding, KeyedManifest, KeyedManifestError};
use architecture::semantic::SemanticOperationId;
use std::collections::{BTreeMap, BTreeSet};

pub type KeyedOperationRealization =
    crate::OperationRealization<SemanticOperationId, KeyedRelationDeclaration>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyedRealizationError {
    InvalidManifest(KeyedManifestError),
    ArchitectureBindingMismatch,
    ForeignOperation(SemanticOperationId),
    OperationCensus,
    DuplicateRelation(KeyedRelationId),
    RelationIdentity(KeyedRelationId),
    DeclarationCensus(SemanticOperationId),
}

impl KeyedRealizationError {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidManifest(_) => "invalid_manifest",
            Self::ArchitectureBindingMismatch => "architecture_binding_mismatch",
            Self::ForeignOperation(_) => "foreign_operation",
            Self::OperationCensus => "operation_census",
            Self::DuplicateRelation(_) => "duplicate_relation",
            Self::RelationIdentity(_) => "relation_identity",
            Self::DeclarationCensus(_) => "declaration_census",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedRealizationDraft {
    pub binding: KeyedArchitectureBinding,
    pub operations: Vec<KeyedOperationRealization>,
}

#[derive(Clone, Debug)]
pub struct KeyedScopedRealization {
    draft: KeyedRealizationDraft,
    graph: petgraph::graph::DiGraph<KeyedRelationDeclaration, ()>,
}

impl KeyedRealizationDraft {
    pub fn derive(manifest: &KeyedManifest) -> Result<Self, KeyedRealizationError> {
        let span = tracing::debug_span!(target: "realization::keyed", "derive_keyed", operation_count = crate::trace_count(manifest.operations.len()));
        let _entered = span.enter();
        let validated = architecture::validate_draft(manifest)
            .map_err(KeyedRealizationError::InvalidManifest)?;
        let operations = manifest
            .operations
            .iter()
            .map(|operation| crate::declarations::derive_operation(manifest, *operation))
            .collect::<Result<Vec<_>, _>>()?;
        tracing::debug!(target: "realization::keyed", outcome = "completed", operation_count = crate::trace_count(operations.len()), "keyed declarations derived");
        Ok(Self {
            binding: validated.binding(),
            operations,
        })
    }

    pub fn validate(
        self,
        manifest: &KeyedManifest,
    ) -> Result<KeyedScopedRealization, KeyedRealizationError> {
        let span = tracing::debug_span!(target: "realization::keyed", "validate_keyed", operation_count = crate::trace_count(self.operations.len()));
        let _entered = span.enter();
        let result = self.validate_core(manifest);
        tracing::debug!(target: "realization::keyed", outcome = if result.is_ok() { "completed" } else { "refused" }, refusal_kind = result.as_ref().err().map(KeyedRealizationError::kind), "keyed declaration validation completed");
        result
    }

    fn validate_core(
        mut self,
        manifest: &KeyedManifest,
    ) -> Result<KeyedScopedRealization, KeyedRealizationError> {
        let expected = Self::derive(manifest)?;
        if self.binding != expected.binding {
            return Err(KeyedRealizationError::ArchitectureBindingMismatch);
        }
        let mut seen_operations = BTreeSet::new();
        let expected_operations = expected
            .operations
            .iter()
            .map(|operation| (operation.operation, operation))
            .collect::<BTreeMap<_, _>>();
        let mut graph = petgraph::graph::DiGraph::new();
        for operation in &mut self.operations {
            let Some(required) = expected_operations.get(&operation.operation) else {
                return Err(KeyedRealizationError::ForeignOperation(operation.operation));
            };
            if !seen_operations.insert(operation.operation) {
                return Err(KeyedRealizationError::OperationCensus);
            }
            let mut seen_relations = BTreeSet::new();
            for declaration in &operation.relations {
                if !seen_relations.insert(declaration.id) {
                    return Err(KeyedRealizationError::DuplicateRelation(declaration.id));
                }
                if declaration.id.operation != operation.operation
                    || declaration.id.kind != declaration.relation.kind()
                {
                    return Err(KeyedRealizationError::RelationIdentity(declaration.id));
                }
            }
            operation.relations.sort();
            let mut required = (*required).clone();
            required.relations.sort();
            if *operation != required {
                return Err(KeyedRealizationError::DeclarationCensus(
                    operation.operation,
                ));
            }
            for declaration in &operation.relations {
                graph.add_node(*declaration);
            }
        }
        if seen_operations != expected_operations.keys().copied().collect() {
            return Err(KeyedRealizationError::OperationCensus);
        }
        self.operations.sort_by_key(|operation| operation.operation);
        Ok(KeyedScopedRealization { draft: self, graph })
    }
}

impl KeyedScopedRealization {
    pub const fn binding(&self) -> &KeyedArchitectureBinding {
        &self.draft.binding
    }
    pub fn draft(&self) -> KeyedRealizationDraft {
        self.draft.clone()
    }
    pub const fn graph(&self) -> &petgraph::graph::DiGraph<KeyedRelationDeclaration, ()> {
        &self.graph
    }
    pub fn validate_against(&self, manifest: &KeyedManifest) -> Result<(), KeyedRealizationError> {
        self.draft.clone().validate(manifest).map(|_| ())
    }
    /// Canonical relation rows; no graph indices or traversal schedule.
    pub fn project(&self) -> Vec<KeyedRelationDeclaration> {
        let mut rows = self.graph.node_weights().copied().collect::<Vec<_>>();
        rows.sort();
        rows
    }
}
