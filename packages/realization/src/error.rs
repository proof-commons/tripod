//! Typed failures owned by the realization package.

use core::fmt;

use architecture::{ManifestError, OperationId};

/// Failure while constructing or validating target-independent
/// realization values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RealizationError {
    /// The architecture failed its own structural validation.
    ArchitectureValidationFailed { errors: Vec<ManifestError> },

    /// The architecture semantic hash could not be derived.
    ArchitectureHashUnavailable,

    /// A realization scope contained no operations.
    EmptyScope,

    /// One operation occurred more than once in a scope declaration.
    DuplicateScopeOperation(OperationId),

    /// A scoped operation is absent from the supplied architecture.
    OperationOutsideArchitecture(OperationId),

    /// A complete realization was requested from a partial scope.
    IncompleteScope { missing: Vec<OperationId> },

    /// A protocol amount was outside the realization amount domain.
    AmountOutOfDomain { value: u64 },

    /// Checked protocol-amount addition overflowed.
    AmountOverflow,

    /// Checked protocol-amount subtraction underflowed.
    AmountUnderflow,

    /// Checked count arithmetic overflowed.
    CountOverflow,
}

impl fmt::Display for RealizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArchitectureValidationFailed { errors } => {
                write!(
                    formatter,
                    "architecture validation failed with {} error(s)",
                    errors.len(),
                )
            }

            Self::ArchitectureHashUnavailable => {
                formatter.write_str("architecture semantic hash is unavailable")
            }

            Self::EmptyScope => formatter.write_str("realization scope is empty"),

            Self::DuplicateScopeOperation(operation) => {
                write!(
                    formatter,
                    "operation {operation} occurs more than once in the realization scope",
                )
            }

            Self::OperationOutsideArchitecture(operation) => {
                write!(
                    formatter,
                    "operation {operation} is outside the supplied architecture",
                )
            }

            Self::IncompleteScope { missing } => {
                write!(
                    formatter,
                    "realization scope is incomplete; {} architecture operation(s) are missing",
                    missing.len(),
                )
            }

            Self::AmountOutOfDomain { value } => {
                write!(
                    formatter,
                    "protocol amount {value} is outside the domain 0 <= v < 2^51",
                )
            }

            Self::AmountOverflow => {
                formatter.write_str("checked protocol-amount arithmetic overflowed")
            }

            Self::AmountUnderflow => {
                formatter.write_str("checked protocol-amount arithmetic underflowed")
            }

            Self::CountOverflow => formatter.write_str("checked count arithmetic overflowed"),
        }
    }
}

impl std::error::Error for RealizationError {}
