//! Architecture-manifest test suite.
//!
//! Organized by concern:
//!
//! - [`validation_tests`] — draft/release validation and structural
//!   declarations (roots, objects, decisions, complete id coverage).
//! - [`conformance_tests`] — cross-declaration coverage: destruction
//!   lifecycle, issuance/delta/data-output correspondence, input
//!   authorization, object lifecycle paths, root cardinality, quantity
//!   read bidirectionality, and bound resolution.

mod conformance_tests;
mod validation_tests;
