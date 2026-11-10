//! Immutable owner storage fixed by `bind_input`.
//!
//! Constructors are crate-only; owner-specific lowering stays in the private
//! engine owner modules. Accessors never expose mutable state or engine records.

use architecture::OperationId;
use architecture::{keyed::KeyedManifest, semantic::SemanticOperationId};
use realization::ScopedRealizationSpec;
use realization::keyed::KeyedScopedRealization;

/// The statically selected attestation owner retained by the input binder.
#[derive(Clone, Debug)]
pub struct AttestationOwner {
    source: ScopedRealizationSpec,
    scope: super::CompilationScope,
    architecture_operations: std::collections::BTreeSet<OperationId>,
}

impl AttestationOwner {
    pub(crate) const fn new(
        source: ScopedRealizationSpec,
        scope: super::CompilationScope,
        architecture_operations: std::collections::BTreeSet<OperationId>,
    ) -> Self {
        Self {
            source,
            scope,
            architecture_operations,
        }
    }

    /// The validated declaration source retained for independent re-derivation.
    #[must_use]
    pub const fn source(&self) -> &ScopedRealizationSpec {
        &self.source
    }

    pub(crate) const fn scope(&self) -> &super::CompilationScope {
        &self.scope
    }

    pub(crate) const fn architecture_operations(&self) -> &std::collections::BTreeSet<OperationId> {
        &self.architecture_operations
    }
}

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
