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
//! - [`doc_conformance_tests`] — the register-1 → document weld:
//!   every manifest-carried document label appears verbatim in the
//!   realization markdown.

mod conformance_tests;
mod deployment_tests;
mod doc_conformance_tests;
mod export_hash_tests;
mod mutation_tests;
mod validation_tests;
mod versioning_gate_tests;
