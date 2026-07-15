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
    VerificationStatus, deployment_profile_hash, deployment_profile_hash_hex,
    manifest_minimum_for_bound, validate_deployment_release,
};
pub use export::{ArchitectureExport, EnvelopeError, PublishedArchitecture};
pub use ids::*;
pub use spec::*;
pub use validate::{ManifestError, validate_architecture_release, validate_draft};
