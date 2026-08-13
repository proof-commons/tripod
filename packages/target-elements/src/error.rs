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

use crate::opcode::{FailureCause, OpcodeId};

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

    /// A leaf version was offered that this package has not reviewed.
    /// A leaf version selects the semantics of everything executed
    /// beneath it, so an unreviewed byte describes semantics this
    /// contract cannot speak for.
    UnreviewedLeafVersion {
        /// The unreviewed leaf version byte that was offered.
        offered: u8,
    },

    /// A reviewed primitive identity has no contract in the registry.
    MissingOpcodeContract(OpcodeId),

    /// A registry entry claims one identity while being filed under
    /// another, which makes the key the registry is indexed by a lie.
    OpcodeIdMismatch {
        /// The identity the entry is filed under.
        key: OpcodeId,
        /// The identity the entry claims.
        declared: OpcodeId,
    },

    /// Two registry entries claim the same target byte. One of them
    /// would be unreachable, and which one is not determinable from
    /// the contract.
    DuplicateOpcodeCode(u8),

    /// A primitive declares no execution domain, or declares none that
    /// includes the domain the contract describes.
    UnsupportedOpcodeExecutionDomain(OpcodeId),

    /// A primitive's operand or result widths are incoherent, so the
    /// contract describes no admissible stack shape.
    InvalidOpcodeStackContract(OpcodeId),

    /// A primitive declares no failure behavior at all. Every reviewed
    /// primitive can fail, so an empty failure contract is an
    /// incomplete transcription.
    MissingOpcodeFailureContract(OpcodeId),

    /// A primitive declares one failure cause with two different
    /// effects, so the contract does not say what the target does.
    ContradictoryFailureCause {
        /// The primitive carrying the contradiction.
        opcode: OpcodeId,
        /// The cause declared twice.
        cause: FailureCause,
    },

    /// A primitive declares no resource cost where the target requires
    /// a positive one.
    MissingOpcodeResourceCost(OpcodeId),

    /// A primitive names no evidence requirement, so its semantics
    /// rest on this crate's assertion alone and no deployment is ever
    /// asked to demonstrate them.
    MissingOpcodeEvidence(OpcodeId),
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTargetContractVersion { offered } => {
                write!(f, "unsupported target contract version {offered}")
            }
            Self::UnreviewedLeafVersion { offered } => {
                write!(f, "unreviewed leaf version {offered:#04x}")
            }
            Self::MissingOpcodeContract(id) => {
                write!(f, "reviewed opcode {id:?} has no contract")
            }
            Self::OpcodeIdMismatch { key, declared } => {
                write!(f, "opcode entry {key:?} declares identity {declared:?}")
            }
            Self::DuplicateOpcodeCode(code) => {
                write!(f, "two opcodes claim target byte {code:#04x}")
            }
            Self::UnsupportedOpcodeExecutionDomain(id) => {
                write!(f, "opcode {id:?} declares no usable execution domain")
            }
            Self::InvalidOpcodeStackContract(id) => {
                write!(f, "opcode {id:?} has an incoherent stack contract")
            }
            Self::MissingOpcodeFailureContract(id) => {
                write!(f, "opcode {id:?} declares no failure behavior")
            }
            Self::ContradictoryFailureCause { opcode, cause } => {
                write!(
                    f,
                    "opcode {opcode:?} gives failure cause {cause:?} two different effects"
                )
            }
            Self::MissingOpcodeResourceCost(id) => {
                write!(f, "opcode {id:?} declares no resource cost")
            }
            Self::MissingOpcodeEvidence(id) => {
                write!(f, "opcode {id:?} names no evidence requirement")
            }
        }
    }
}

impl core::error::Error for TargetError {}
