//! Binding from one realization value to its typed architecture.

use architecture::Architecture;

use crate::RealizationError;

/// Architecture identity carried by a realization derivation.
///
/// This is an architecture binding, not a new realization identity.
/// Phase 1 deliberately does not publish a realization hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchitectureBinding {
    architecture_schema_version: u32,
    realization_version: String,
    semantic_hash: [u8; 32],
}

impl ArchitectureBinding {
    /// Validate and bind one typed architecture.
    pub fn from_architecture(architecture: &Architecture) -> Result<Self, RealizationError> {
        if let Err(errors) = architecture::validate_draft(architecture) {
            return Err(RealizationError::ArchitectureValidationFailed { errors });
        }

        let semantic_hash = architecture::semantic_hash(architecture)
            .map_err(|_| RealizationError::ArchitectureHashUnavailable)?;

        Ok(Self {
            architecture_schema_version: architecture.document.architecture_schema_version,
            realization_version: architecture.document.realization_version.to_owned(),
            semantic_hash,
        })
    }

    /// Bound architecture schema.
    #[must_use]
    pub const fn architecture_schema_version(&self) -> u32 {
        self.architecture_schema_version
    }

    /// Bound tracked realization version from the architecture envelope.
    #[must_use]
    pub fn realization_version(&self) -> &str {
        &self.realization_version
    }

    /// Bound full architecture semantic hash.
    #[must_use]
    pub const fn semantic_hash(&self) -> [u8; 32] {
        self.semantic_hash
    }
}
