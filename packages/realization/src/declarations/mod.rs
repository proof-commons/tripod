//! Operation realization declarations.

use architecture::{Architecture, OperationId};

use crate::{OperationRealization, RealizationError};

pub mod announce_maturity;
pub mod compact_ash;
pub mod keyed;
pub mod transfer_live;

fn derive_attestation_operation(
    architecture: &Architecture,
    operation: OperationId,
) -> Result<OperationRealization, RealizationError> {
    match operation {
        OperationId::AnnounceMaturity => announce_maturity::derive(architecture),
        OperationId::CompactAsh => compact_ash::derive(architecture),
        OperationId::TransferLive => transfer_live::derive(architecture),
        other => Err(RealizationError::UnsupportedOperationDeclaration(other)),
    }
}

/// Both declaration families enter through the existing semantic owner seam.
pub trait OperationDeclaration {
    type Id;
    type Declaration;
    type Error;
    fn declare_operation(&self, operation: Self::Id) -> Result<Self::Declaration, Self::Error>;
}

impl OperationDeclaration for Architecture {
    type Id = OperationId;
    type Declaration = OperationRealization;
    type Error = RealizationError;
    fn declare_operation(&self, operation: Self::Id) -> Result<Self::Declaration, Self::Error> {
        derive_attestation_operation(self, operation)
    }
}

impl OperationDeclaration for architecture::keyed::KeyedManifest {
    type Id = architecture::semantic::SemanticOperationId;
    type Declaration = crate::keyed::KeyedOperationRealization;
    type Error = crate::keyed::KeyedRealizationError;
    fn declare_operation(&self, operation: Self::Id) -> Result<Self::Declaration, Self::Error> {
        use architecture::semantic::{KeyedOperationId, SemanticOperationId};
        match operation {
            SemanticOperationId::Keyed(KeyedOperationId::Source) => Ok(keyed::source()),
            SemanticOperationId::Keyed(KeyedOperationId::Merge) => Ok(keyed::merge()),
            SemanticOperationId::Attestation(_) => Err(
                crate::keyed::KeyedRealizationError::ForeignOperation(operation),
            ),
        }
    }
}

pub fn derive_operation<A: OperationDeclaration>(
    architecture: &A,
    operation: A::Id,
) -> Result<A::Declaration, A::Error> {
    architecture.declare_operation(operation)
}
