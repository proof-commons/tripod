//! Target-independent typed semantic realization of the attestation contract.
//!
//! This crate sits between the finite architecture manifest and two
//! independent consumers:
//!
//! - executable-model conformance;
//! - future target-independent compiler analysis.
//!
//! The crate consumes validated typed architecture values. It does not
//! parse generated publications, model source, tests, documentation,
//! plans, target descriptions, or deployment artifacts.
//!
//! Phase 1 deliberately covers only:
//!
//! - compact ASH;
//! - live receipt transfer.
//!
//! A Phase-1 value is explicitly partial and cannot be accepted as a
//! complete protocol realization.

#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod binding;
pub mod domain;
pub mod error;
pub mod identity;
pub mod scope;

pub use binding::ArchitectureBinding;
pub use domain::{Count, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE, ProtocolAmount, RepresentationMode};
pub use error::RealizationError;
pub use identity::{
    FactId, ProofAlternativeId, ProofKind, RelationId, RelationKind, RelationSubject,
    TransactionSide,
};
pub use scope::{CompleteRealizationScope, RealizationScope};

#[cfg(test)]
mod tests;
