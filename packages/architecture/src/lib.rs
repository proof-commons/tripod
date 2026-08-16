//! Typed normative architecture manifest for Attestation
//! Realization.
//!
//! This crate contains stable semantic identifiers, static architecture
//! declarations, validation, canonical semantic hashing, and the Serde
//! export model used to produce the two derivative artifacts
//! `architecture.json` and `architecture.toml`.
//!
//! It deliberately contains no:
//!
//! - UTXO state;
//! - branch execution;
//! - script implementation;
//! - indexer database;
//! - property-test state machine.
//!
//! Those artifacts live in `tripod-model`, depend on this crate,
//! and are checked against it. The dependency direction is one-way:
//! this crate must never depend on the executable model.
//!
//! # Boundary
//!
//! The crate is a leaf: it reads no file, no environment variable, and
//! no network. Its whole input is the [`ARCHITECTURE`] constant compiled
//! into it, plus whatever typed [`Architecture`] value a caller hands a
//! function. Everything it produces — validation verdicts, canonical
//! bytes, hashes, export DTOs — is a deterministic function of that
//! typed value. The two published artifacts `architecture.json` and
//! `architecture.toml` are derivative renderings owned by the artifacts
//! generator, never inputs here.
//!
//! # Module map
//!
//! - [`ids`] — the finite stable identifier vocabulary
//!   ([`AssetId`], [`RootId`], [`ObjectId`], [`OperationId`],
//!   [`QuantityId`], [`WitnessId`], [`BoundId`], [`TagId`],
//!   [`InvariantClauseId`], and the classification enums). Explicit
//!   discriminants are part of the canonical encoding; variants are
//!   never reordered or renumbered after publication.
//! - [`spec`] — the declarative manifest itself: the [`Architecture`]
//!   struct, its per-category `*Spec` records, and the [`ARCHITECTURE`]
//!   constant every consumer starts from. Static data, not executable
//!   policy.
//! - [`validate`] — [`validate_draft`] and
//!   [`validate_architecture_release`], plus the [`ManifestError`]
//!   vocabulary and the two validated-wrapper types.
//! - [`canonical`] — canonical JSON projection and the semantic,
//!   behavioural, and Layer-0 anchor-set hashes.
//! - [`export`] — the Serde DTO tree ([`ArchitectureExport`]) and the
//!   publication envelope [`PublishedArchitecture`], with envelope
//!   validation ([`EnvelopeError`]).
//! - [`deployment`] — the separate deployment-evidence assurance
//!   boundary: [`DeploymentProfile`], its structural validator, and its
//!   own domain-separated hash.
//!
//! # Primary workflow
//!
//! Validation precedes identity (ADR-016), so the order is fixed:
//!
//! 1. start from [`spec::ARCHITECTURE`] (or your own [`spec::Architecture`]);
//! 2. call [`validate::validate_draft`] for internal consistency, or
//!    [`validate::validate_architecture_release`] when the Layer-0 pin
//!    and a final publication status are also required;
//! 3. pass the returned wrapper — never a raw `Architecture` — to
//!    [`canonical::semantic_hash_hex`],
//!    [`canonical::behavioural_hash_hex`],
//!    [`canonical::canonical_json_bytes`], or
//!    [`export::PublishedArchitecture::from_architecture`].
//!
//! ```
//! use architecture::{
//!     ARCHITECTURE, OperationId, PublishedArchitecture, semantic_hash_hex,
//!     validate_architecture_release,
//! };
//!
//! let release = validate_architecture_release(&ARCHITECTURE)
//!     .expect("the shipped manifest is release-valid");
//!
//! // Identity is defined only over the validated wrapper.
//! let digest = semantic_hash_hex(&release.draft()).unwrap();
//! assert_eq!(digest.len(), 64);
//!
//! // Lookups go through the typed manifest.
//! let compact_ash = release
//!     .architecture()
//!     .operation(OperationId::CompactAsh)
//!     .expect("declared operation");
//! assert_eq!(compact_ash.id, OperationId::CompactAsh);
//!
//! // The publication envelope re-derives and carries both hashes.
//! let published = PublishedArchitecture::from_architecture(&release.draft()).unwrap();
//! assert_eq!(published.semantic_hash, digest);
//! published.validate_release_envelope().unwrap();
//! ```
//!
//! The README carries the longer worked examples, the item-by-item API
//! tour, and the error-handling guide.

#![forbid(unsafe_code)]
// This crate is a transcription of the normative architecture manifest.
// Purely stylistic pedantic/nursery lints are allowed crate-wide;
// correctness, suspicious, and perf lints remain in force.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::use_self
)]

pub mod canonical;
pub mod deployment;
pub mod export;
pub mod ids;
pub mod spec;
pub mod validate;

pub use canonical::{
    BEHAVIOURAL_HASH_ALGORITHM, RETIRED_BEHAVIOURAL_HASH_ALGORITHMS, SEMANTIC_HASH_ALGORITHM,
    anchor_set_hash, behavioural_hash, behavioural_hash_hex, canonical_json_bytes, semantic_hash,
    semantic_hash_hex,
};
pub use deployment::{
    ArtifactHashes, BoundCalibration, DEPLOYMENT_HASH_ALGORITHM, DEPLOYMENT_PROFILE_SCHEMA_VERSION,
    DependencyEvidence, DeploymentError, DeploymentProfile, ScriptLimits, TestEvidence,
    ValidatedPreReleaseDeploymentProfile, VerificationStatus, deployment_profile_hash,
    deployment_profile_hash_hex, manifest_minimum_for_bound, validate_deployment_profile,
    validate_deployment_profile_structure, validate_production_deployment_release,
};
pub use export::{ArchitectureExport, EnvelopeError, PublishedArchitecture};
pub use ids::*;
pub use spec::*;
pub use validate::{
    ManifestError, ValidatedDraftArchitecture, ValidatedReleaseArchitecture,
    validate_architecture_release, validate_draft,
};

#[cfg(test)]
mod tests;
