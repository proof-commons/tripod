//! The target package's single error root.
//!
//! Every validation failure in this crate is reported as a
//! [`TargetError`]. The enum is deliberately hand-written rather than
//! derived: the package carries no dependencies at all, so neither a
//! derive macro nor a `Display` helper crate is available, and none is
//! wanted for a vocabulary this small.
//!
//! # Admission rule
//!
//! A variant exists only when some validation branch in this crate
//! actually constructs it. A variant reserved for a failure no code
//! path can reach would be a claim about validation that has not been
//! implemented, so the vocabulary grows with the validators rather
//! than ahead of them.

use core::fmt;

/// A typed target-contract failure.
///
/// Validation of a target definition reports *all* diagnostics rather
/// than the first, so this type is normally seen inside a
/// `Vec<TargetError>`. The deployment-binding and combination
/// validators reject on a single typed reason and return one value.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetError {
    /// A target-contract version was offered that this crate does not
    /// implement. The version is an explicit compatibility decision,
    /// not an identity digest: an adapter that cannot interpret a
    /// revision must refuse it rather than guess.
    UnsupportedTargetContractVersion {
        /// The unsupported version number that was offered.
        offered: u32,
    },
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTargetContractVersion { offered } => {
                write!(f, "unsupported target contract version {offered}")
            }
        }
    }
}

impl core::error::Error for TargetError {}
