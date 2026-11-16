//! Fixed-form Keyed Provenance Covenant scoped declaration owner. This is not a history oracle or reader.
//!
//! Exact owner re-derivation catches a coherently removed clause, even if all
//! references to it were also removed. Evidence obligations remain unresolved.

use crate::keyed_fixed_relation::{KeyedFixedRelationDeclaration, KeyedFixedRelationId};
use architecture::keyed_fixed::KeyedFixedOperationId;
use architecture::keyed_fixed::{
    KeyedFixedArchitectureBinding, KeyedFixedManifest, KeyedFixedManifestError,
};
use std::collections::{BTreeMap, BTreeSet};

/// Positive semantic value. Target ceilings are checked by construction at S6.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyedFixedValue(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedFixedValueRefusal {
    Zero,
}

impl KeyedFixedValue {
    /// Admits a positive value without selecting a target ceiling.
    ///
    /// # Errors
    /// Returns [`KeyedFixedValueRefusal::Zero`] if `value` is zero.
    pub fn new(value: u64) -> Result<Self, KeyedFixedValueRefusal> {
        let span = tracing::debug_span!(target: "realization::keyed_fixed", "admit_positive_value");
        let _entered = span.enter();
        let result = if value == 0 {
            Err(KeyedFixedValueRefusal::Zero)
        } else {
            Ok(Self(value))
        };
        tracing::debug!(target: "realization::keyed_fixed", outcome = if result.is_ok() { "completed" } else { "refused" }, refusal_kind = if value == 0 { "zero" } else { "none" }, "positive value admission completed");
        result
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Supplied public triple. Its values never enter semantic manifest identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedFixedParameters {
    pub key: [u8; 32],
    pub asset: [u8; 32],
    pub value: KeyedFixedValue,
}

/// Semantic field form, after the caller authenticates the output fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedFixedField<T> {
    Explicit(T),
    Other,
}

/// Membership over authenticated fields, not a byte reader or authentication receipt.
///
/// The caller supplies the program authenticated for this triple under the
/// realization premises. Binding `authenticated_program` to the same triple is
/// the caller's obligation: this predicate does not read `parameters.key` or
/// authenticate that binding. Nonce is deliberately absent from the predicate.
pub fn is_keyed_fixed_member(
    parameters: KeyedFixedParameters,
    authenticated_program: &[u8],
    output_program: &[u8],
    birth_position: u64,
    asset: KeyedFixedField<[u8; 32]>,
    value: KeyedFixedField<u64>,
) -> bool {
    let span = tracing::debug_span!(target: "realization::keyed_fixed", "classify_member");
    let _entered = span.enter();
    let member = birth_position == 0
        && authenticated_program == output_program
        && asset == KeyedFixedField::Explicit(parameters.asset)
        && value == KeyedFixedField::Explicit(parameters.value.get());
    tracing::debug!(target: "realization::keyed_fixed", member, "membership classified over supplied authenticated fields");
    member
}

pub type KeyedFixedOperationRealization =
    crate::OperationRealization<KeyedFixedOperationId, KeyedFixedRelationDeclaration>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyedFixedRealizationError {
    InvalidManifest(KeyedFixedManifestError),
    ArchitectureBindingMismatch,
    /// Representational arm: the validated manifest requires both variants of
    /// the closed operation type, so no well-typed foreign operation exists.
    ForeignOperation(KeyedFixedOperationId),
    OperationCensus,
    DuplicateRelation(KeyedFixedRelationId),
    RelationIdentity(KeyedFixedRelationId),
    DeclarationCensus(KeyedFixedOperationId),
}

impl KeyedFixedRealizationError {
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
pub struct KeyedFixedRealizationDraft {
    pub binding: KeyedFixedArchitectureBinding,
    pub operations: Vec<KeyedFixedOperationRealization>,
}

#[derive(Clone, Debug)]
pub struct KeyedFixedScopedRealization {
    draft: KeyedFixedRealizationDraft,
    graph: petgraph::graph::DiGraph<KeyedFixedRelationDeclaration, ()>,
}

impl KeyedFixedRealizationDraft {
    pub fn derive(manifest: &KeyedFixedManifest) -> Result<Self, KeyedFixedRealizationError> {
        let span = tracing::debug_span!(target: "realization::keyed_fixed", "derive_keyed_fixed", operation_count = crate::trace_count(manifest.operations.len()));
        let _entered = span.enter();
        let validated = architecture::validate_draft(manifest)
            .map_err(KeyedFixedRealizationError::InvalidManifest)?;
        let operations = manifest
            .operations
            .iter()
            .map(|operation| crate::declarations::derive_operation(manifest, *operation))
            .collect::<Result<Vec<_>, _>>()?;
        tracing::debug!(target: "realization::keyed_fixed", outcome = "completed", operation_count = crate::trace_count(operations.len()), "keyed declarations derived");
        Ok(Self {
            binding: validated.binding(),
            operations,
        })
    }

    pub fn validate(
        self,
        manifest: &KeyedFixedManifest,
    ) -> Result<KeyedFixedScopedRealization, KeyedFixedRealizationError> {
        let span = tracing::debug_span!(target: "realization::keyed_fixed", "validate_keyed_fixed", operation_count = crate::trace_count(self.operations.len()));
        let _entered = span.enter();
        let result = self.validate_core(manifest);
        tracing::debug!(target: "realization::keyed_fixed", outcome = if result.is_ok() { "completed" } else { "refused" }, refusal_kind = result.as_ref().err().map(KeyedFixedRealizationError::kind), "keyed declaration validation completed");
        result
    }

    fn validate_core(
        mut self,
        manifest: &KeyedFixedManifest,
    ) -> Result<KeyedFixedScopedRealization, KeyedFixedRealizationError> {
        let expected = Self::derive(manifest)?;
        if self.binding != expected.binding {
            return Err(KeyedFixedRealizationError::ArchitectureBindingMismatch);
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
                return Err(KeyedFixedRealizationError::ForeignOperation(
                    operation.operation,
                ));
            };
            if !seen_operations.insert(operation.operation) {
                return Err(KeyedFixedRealizationError::OperationCensus);
            }
            let mut seen_relations = BTreeSet::new();
            for declaration in &operation.relations {
                if !seen_relations.insert(declaration.id) {
                    return Err(KeyedFixedRealizationError::DuplicateRelation(
                        declaration.id,
                    ));
                }
                if declaration.id.operation != operation.operation
                    || declaration.id.kind != declaration.relation.kind()
                {
                    return Err(KeyedFixedRealizationError::RelationIdentity(declaration.id));
                }
            }
            operation.relations.sort();
            let mut required = (*required).clone();
            required.relations.sort();
            if *operation != required {
                return Err(KeyedFixedRealizationError::DeclarationCensus(
                    operation.operation,
                ));
            }
            for declaration in &operation.relations {
                graph.add_node(*declaration);
            }
        }
        if seen_operations != expected_operations.keys().copied().collect() {
            return Err(KeyedFixedRealizationError::OperationCensus);
        }
        self.operations.sort_by_key(|operation| operation.operation);
        Ok(KeyedFixedScopedRealization { draft: self, graph })
    }
}

impl KeyedFixedScopedRealization {
    pub const fn binding(&self) -> &KeyedFixedArchitectureBinding {
        &self.draft.binding
    }
    pub fn draft(&self) -> KeyedFixedRealizationDraft {
        self.draft.clone()
    }
    pub const fn graph(&self) -> &petgraph::graph::DiGraph<KeyedFixedRelationDeclaration, ()> {
        &self.graph
    }
    pub fn validate_against(
        &self,
        manifest: &KeyedFixedManifest,
    ) -> Result<(), KeyedFixedRealizationError> {
        self.draft.clone().validate(manifest).map(|_| ())
    }
    /// Canonical relation rows; no graph indices or traversal schedule.
    pub fn project(&self) -> Vec<KeyedFixedRelationDeclaration> {
        let mut rows = self.graph.node_weights().copied().collect::<Vec<_>>();
        rows.sort();
        rows
    }
}
