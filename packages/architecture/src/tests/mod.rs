//! Architecture-manifest test suite.
//!
//! Organized by concern:
//!
//! - [`validation_tests`] — draft/release validation and structural
//!   declarations (roots, objects, decisions, complete id coverage).
//! - [`export_hash_tests`] — canonical export, semantic-hash
//!   stability, and generated-artifact verification.
//! - [`conformance_tests`] — cross-declaration coverage: destruction
//!   lifecycle, issuance/delta/data-output correspondence, input
//!   authorization, object lifecycle paths, root cardinality, quantity
//!   read bidirectionality, and bound resolution.
//! - [`mutation_tests`] — deliberate manifest corruptions that must
//!   each fail draft validation.
//! - [`deployment_tests`] — deployment-release validation: the
//!   synthetic final profile fixture and per-field rejection
//!   mutations.
//! - [`versioning_gate_tests`] — the denotation gate: the behavioural
//!   hash may move only with a `realization_version` bump.

use crate::spec::Architecture;
use crate::validate::{ValidatedDraftArchitecture, validate_draft};

/// The validated wrapper the public identity functions accept.
///
/// Identity is reachable only through validation (R2-N03), so tests
/// that need a hash or a publication of a *valid* architecture pass
/// through here. Tests that deliberately corrupt the manifest use the
/// crate-private unchecked projections instead.
fn validated(architecture: &Architecture) -> ValidatedDraftArchitecture<'_> {
    validate_draft(architecture).expect("fixture architecture is draft-valid")
}

mod conformance_tests;
mod deployment_tests;
mod export_hash_tests;
mod guide12_reproductions;
mod mutation_tests;
mod validation_tests;
mod versioning_gate_tests;
