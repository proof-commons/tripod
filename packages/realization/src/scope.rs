//! Explicit operation scope.
//!
//! A scoped Phase-1 realization is not a complete realization. The
//! complete wrapper can be constructed only after every operation in
//! the supplied validated architecture is present.

use architecture::{Architecture, OperationId};

use crate::RealizationError;

/// Canonically ordered operation scope of one realization derivation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealizationScope {
    operations: Vec<OperationId>,
}

impl RealizationScope {
    /// Construct a nonempty scope in stable operation-code order.
    pub fn from_operations(
        operations: impl IntoIterator<Item = OperationId>,
    ) -> Result<Self, RealizationError> {
        let mut operations = operations.into_iter().collect::<Vec<_>>();

        operations.sort_by_key(|operation| operation.code());

        if operations.is_empty() {
            return Err(RealizationError::EmptyScope);
        }

        if let Some(duplicate) = operations
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(RealizationError::DuplicateScopeOperation(duplicate));
        }

        Ok(Self { operations })
    }

    /// The canonical two-operation Phase-1 scope.
    #[must_use]
    pub fn phase1_pilots() -> Self {
        Self {
            operations: vec![OperationId::TransferLive, OperationId::CompactAsh],
        }
    }

    /// Operations in stable architecture-code order.
    #[must_use]
    pub fn operations(&self) -> &[OperationId] {
        &self.operations
    }

    /// Return whether the scope contains `operation`.
    #[must_use]
    pub fn contains(&self, operation: OperationId) -> bool {
        self.operations
            .binary_search_by_key(&operation.code(), |candidate| candidate.code())
            .is_ok()
    }

    /// Validate that every scoped operation exists in `architecture`.
    pub fn validate_against(&self, architecture: &Architecture) -> Result<(), RealizationError> {
        for operation in &self.operations {
            if architecture.operation(*operation).is_none() {
                return Err(RealizationError::OperationOutsideArchitecture(*operation));
            }
        }

        Ok(())
    }

    /// Upgrade this scope only when it covers the complete validated
    /// architecture operation set.
    pub fn try_complete(
        &self,
        architecture: &Architecture,
    ) -> Result<CompleteRealizationScope, RealizationError> {
        if let Err(errors) = architecture::validate_draft(architecture) {
            return Err(RealizationError::ArchitectureValidationFailed { errors });
        }

        self.validate_against(architecture)?;

        let mut expected = architecture
            .operations
            .iter()
            .map(|operation| operation.id)
            .collect::<Vec<_>>();

        expected.sort_by_key(|operation| operation.code());

        let missing = expected
            .into_iter()
            .filter(|operation| !self.contains(*operation))
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            return Err(RealizationError::IncompleteScope { missing });
        }

        Ok(CompleteRealizationScope {
            scope: self.clone(),
        })
    }
}

/// Operation scope proven to cover the complete validated architecture.
///
/// Construction is private to [`RealizationScope::try_complete`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompleteRealizationScope {
    scope: RealizationScope,
}

impl CompleteRealizationScope {
    /// Read the complete operation scope.
    #[must_use]
    pub const fn as_scope(&self) -> &RealizationScope {
        &self.scope
    }
}
