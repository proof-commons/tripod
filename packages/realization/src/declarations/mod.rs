//! Operation realization declarations.

use architecture::{Architecture, OperationId};

use crate::{OperationRealization, RealizationError};

pub mod announce_maturity;
pub mod burn;
pub mod clear;
pub mod compact_ash;
pub mod keyed;
pub mod keyed_fixed;
pub mod transfer_live;

fn derive_attestation_operation(
    architecture: &Architecture,
    operation: OperationId,
) -> Result<OperationRealization, RealizationError> {
    match operation {
        OperationId::AnnounceMaturity => announce_maturity::derive(architecture),
        OperationId::Burn => burn::derive(architecture),
        OperationId::Clear => clear::derive(architecture),
        OperationId::CompactAsh => compact_ash::derive(architecture),
        OperationId::TransferLive => transfer_live::derive(architecture),
        OperationId::CreateRequest
        | OperationId::CancelRequest
        | OperationId::AdmitDeposits
        | OperationId::Cycle
        | OperationId::SettleDistribution
        | OperationId::TransferTimeLocked
        | OperationId::Redeem
        | OperationId::ReceiptRelabel => {
            Err(RealizationError::UnsupportedOperationDeclaration(operation))
        }
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for architecture::Architecture {}
    impl Sealed for architecture::keyed::KeyedManifest {}
    impl Sealed for architecture::keyed_fixed::KeyedFixedManifest {}
}

/// Declaration families enter through the existing semantic owner seam.
pub trait OperationDeclaration: sealed::Sealed {
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

impl OperationDeclaration for architecture::keyed_fixed::KeyedFixedManifest {
    type Id = architecture::keyed_fixed::KeyedFixedOperationId;
    type Declaration = crate::keyed_fixed::KeyedFixedOperationRealization;
    type Error = crate::keyed_fixed::KeyedFixedRealizationError;
    fn declare_operation(&self, operation: Self::Id) -> Result<Self::Declaration, Self::Error> {
        architecture::validate_draft(self).map_err(Self::Error::InvalidManifest)?;
        match operation {
            Self::Id::Source => Ok(keyed_fixed::source()),
            Self::Id::Merge => Ok(keyed_fixed::merge()),
        }
    }
}
