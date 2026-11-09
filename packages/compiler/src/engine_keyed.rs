//! Keyed owner lowering into the same concrete relation-graph record domain.
//! This module supplies records; it contains no graph algorithm or analyzer.

use architecture::{keyed::KeyedManifest, semantic::SemanticOperationId};
use realization::{
    keyed::{KeyedRealizationError, KeyedScopedRealization},
    relation::{KeyedRelationId, KeyedRelationKind},
};

use crate::engine_records::{
    LoweredRelations, OperationKey, RelationKey, RelationOwner, RelationRecord, RelationRecords,
};

/// The statically selected keyed owner retained by the input binder.
#[derive(Clone, Debug)]
pub struct KeyedOwner {
    manifest: KeyedManifest,
    source: KeyedScopedRealization,
    scope: Vec<SemanticOperationId>,
}

impl KeyedOwner {
    pub(crate) const fn new(
        manifest: KeyedManifest,
        source: KeyedScopedRealization,
        scope: Vec<SemanticOperationId>,
    ) -> Self {
        Self {
            manifest,
            source,
            scope,
        }
    }

    /// The declaration owner and paper binding retained at binding.
    #[must_use]
    pub const fn manifest(&self) -> &KeyedManifest {
        &self.manifest
    }

    /// The validated declaration source, with no mutable accessor.
    #[must_use]
    pub const fn source(&self) -> &KeyedScopedRealization {
        &self.source
    }

    /// The explicit compiler scope, distinct from the realization scope.
    #[must_use]
    pub fn scope(&self) -> &[SemanticOperationId] {
        &self.scope
    }
}

/// Refusals of the keyed input binding entry, before any analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyedBindingError {
    InvalidRealization(KeyedRealizationError),
    EmptyScope,
    DuplicateOperation(SemanticOperationId),
    ForeignOperation(SemanticOperationId),
}

impl KeyedBindingError {
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidRealization(_) => "invalid_realization",
            Self::EmptyScope => "empty_scope",
            Self::DuplicateOperation(_) => "duplicate_operation",
            Self::ForeignOperation(_) => "foreign_operation",
        }
    }
}

pub struct KeyedRelations {
    records: RelationRecords,
}

impl LoweredRelations for KeyedRelations {
    fn records(&self) -> &RelationRecords {
        &self.records
    }
}

impl RelationOwner for KeyedOwner {
    type Lowered = KeyedRelations;

    fn lower_relations(&self) -> Self::Lowered {
        let span = tracing::debug_span!(target: "compiler::engine_keyed", "lower_relations", scope_operation_count = crate::instrumentation::count(self.scope.len()));
        let _entered = span.enter();
        let nodes = self
            .source
            .project()
            .into_iter()
            .map(|declaration| RelationRecord {
                id: relation_key(declaration.id),
                proof_owners: Vec::new(),
            })
            .collect::<Vec<_>>();
        tracing::debug!(target: "compiler::engine_keyed", outcome = "completed", source_node_count = crate::instrumentation::count(nodes.len()), "relation records lowered");
        // The exact declaration owner validates the absence of relation
        // dependencies. No graph, proof strategy or execution carrier is invented.
        KeyedRelations {
            records: RelationRecords {
                scope: self.scope.iter().copied().map(operation_key).collect(),
                nodes,
                edges: Vec::new(),
            },
        }
    }
}

pub const fn operation_key(operation: SemanticOperationId) -> OperationKey {
    let [namespace, code] = operation.encoding();
    OperationKey { namespace, code }
}

pub fn relation_key(id: KeyedRelationId) -> RelationKey {
    let kind = match id.kind {
        KeyedRelationKind::InputFamily => 0,
        KeyedRelationKind::OutputFamily => 1,
        KeyedRelationKind::BornFirst => 2,
        KeyedRelationKind::ContinueFirst => 3,
        KeyedRelationKind::SourceClassification => 4,
        KeyedRelationKind::MembershipReading => 5,
    };
    RelationKey {
        operation: operation_key(id.operation),
        kind,
        subject: id.object.encoding().to_vec(),
    }
}
