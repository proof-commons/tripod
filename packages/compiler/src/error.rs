//! The compiler's single error root.
//!
//! Analysis failures are typed and never silently weakened: an
//! analysis that cannot complete returns an error rather than dropping
//! a relation, weakening an authorization, or falling back to a
//! partial result.
//!
//! Only the input-boundary variants exist so far. They are the
//! failures the analyzed input boundary must be able to report, and
//! each one is exercised by the public-API test. The enum is
//! deliberately `non_exhaustive`, so the later analysis stages extend
//! this vocabulary without a breaking change and without this file
//! having to guess their shapes now.

use architecture::OperationId;
use thiserror::Error;

/// A typed compilation failure.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompileError {
    /// The realization was built against a realization schema this
    /// compiler does not support.
    #[error("unsupported realization schema {schema}")]
    UnsupportedRealizationSchema {
        /// The schema version carried by the offered realization.
        schema: u32,
    },

    /// The offered realization did not validate under its owner's
    /// validator. The compiler re-runs that validator rather than
    /// trusting the value it was handed.
    #[error("realization failed its owner's validation")]
    InvalidRealization,

    /// The realization binds an architecture identity other than the
    /// one the compilation was requested against.
    #[error("realization architecture binding does not match the requested architecture")]
    ArchitectureBindingMismatch,

    /// The requested compilation scope names an operation the offered
    /// realization does not declare.
    #[error("requested scope operation {operation:?} is absent from the realization")]
    IncompleteRealizationScope {
        /// The operation named by the scope but missing from the input.
        operation: OperationId,
    },

    /// The requested compilation scope names one operation more than
    /// once. Scope is a set; a repeated member is a caller defect, not
    /// a value to normalize away silently.
    #[error("requested scope names operation {operation:?} more than once")]
    DuplicateScopeOperation {
        /// The repeated operation.
        operation: OperationId,
    },
}
