#![doc = include_str!("../README.md")]
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
