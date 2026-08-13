//! The typed, reviewed Elements tapscript target compatibility
//! contract.
//!
//! This crate states target facts. It states nothing about the attestation contract
//! .
//!
//! # Boundary
//!
//! The package owns the tapscript execution domain and leaf version,
//! reviewed opcode identities and their stack and failure contracts,
//! field-specific operand and result encodings, sighash and
//! relative-timelock dimensions, confidential-value and issuance
//! capability descriptions, consensus and policy resource interfaces,
//! and the registry of target evidence that a future deployment must
//! produce.
//!
//! It owns no attestation-contract operation, object, relation, proof
//! plan, authorization policy, batch bound, or transaction layout.
//! Nothing here maps a target fact onto a protocol meaning; that
//! mapping belongs to a downstream adapter, which may depend on this
//! crate while this crate depends on nothing.
//!
//! # Dependencies
//!
//! None. Not first-party, not third-party. Every declaration is a
//! standard-library type. The crate serializes nothing, hashes
//! nothing, parses nothing, and opens no file.
//!
//! # Review provenance is not target identity
//!
//! The typed facts here were transcribed from a reviewed reading of
//! upstream Elements interpreter source. The upstream repository, the
//! revision consulted, the source paths, the node version, and the
//! review date are *review provenance*: they are recorded in the human
//! reference under `plans/reference/elements-tapscript.md` and they
//! never enter this crate's types or its stable projections. The
//! contract identifies a typed compatibility surface, not one
//! implementation revision.
//!
//! # Identity
//!
//! The crate mints no digest. There is no target-definition hash, no
//! deployment-instance hash, and no field reserved for one. Direct
//! typed comparison of validated values is the whole comparison
//! mechanism, and a stable contract version carries the one
//! compatibility decision a consumer actually makes.
//!
//! # State
//!
//! Implemented: the crate boundary, the typed error root, the
//! target-contract version, the tapscript execution domain and leaf
//! version, the reviewed primitive registry with complete stack,
//! failure, and resource contracts, the encoding and
//! evidence-requirement keys those primitives name, the target
//! validator, and the stable semantic projection.
//!
//! Not implemented, and not claimed: the encoding specification
//! registry, authorization and timelock contracts, confidential-value
//! and issuance contracts, resource interfaces, capability contracts,
//! evidence-requirement contents, and the development deployment
//! binding.
//!
//! Every evidence requirement this crate names is unresolved. No
//! target-native deployment evidence has been produced, and production
//! target support is not claimed.

#![forbid(unsafe_code)]

pub mod definition;
pub mod encoding;
pub mod error;
pub mod evidence;
pub mod opcode;

pub use definition::{
    TargetContractVersion, TargetDefinition, TargetProjection, ValidatedTargetDefinition,
    reviewed_elements_tapscript, validate_target_definition,
};
pub use encoding::{ByteOrder, EncodingClass, PayloadWidth};
pub use error::TargetError;
pub use evidence::TargetEvidenceRequirementId;
pub use opcode::{
    ExecutionDomain, FailureCause, FailureContract, FailureEffect, FailureOutcome, LeafVersion,
    OpcodeId, OpcodeResourceCost, OpcodeSpec, StackContract, StackValueType,
};

#[cfg(test)]
mod tests;
