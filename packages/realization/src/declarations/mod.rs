//! Operation realization declarations.

use architecture::{Architecture, OperationId};

use crate::{OperationRealization, RealizationError};

pub mod compact_ash;

pub fn derive_operation(
    architecture: &Architecture,
    operation: OperationId,
) -> Result<OperationRealization, RealizationError> {
    match operation {
        OperationId::CompactAsh => compact_ash::derive(architecture),
        other => Err(RealizationError::UnsupportedOperationDeclaration(other)),
    }
}
