//! The adapter from compiler-owned abstract target requirements to
//! Elements target obligations.
//!
//! # Boundary
//!
//! This crate is the join, and only the join. The compiler owns what an
//! approved analysis requires of *some* target, abstractly; the target
//! package owns what one reviewed Elements tapscript contract offers,
//! concretely; neither may name the other. The join has to live
//! somewhere, and it lives here because this is the one package whose
//! contract admits both.
//!
//! It owns no target program, no instruction, no stack schedule, no
//! transaction layout, and no bundle. It emits nothing.
//!
//! # Dependencies
//!
//! `compiler` and `target-elements`, and nothing else — not first-party
//! and not third-party. The crate serializes nothing, hashes nothing,
//! parses nothing, and opens no file.
//!
//! # State
//!
//! Implemented: the package boundary, the typed instruction core — a
//! typed instruction, a checked stack item, an exact serializer, and a
//! parser over the reviewed subset — the abstract stack validator, which
//! keeps every successful alternative and every non-aborting failure
//! state apart from the aborting causes — the static capability adapter
//! —
//! a multi-state assessment of each compiler capability against the
//! reviewed static target contract — and the external-evidence-role
//! adapter, with exact census equality in both directions against the
//! two censuses the analysis published.
//!
//! Not assessed here: anything a deployment declares. The static
//! contract, the development binding, and target-native evidence are
//! three different values, and nothing in this crate accepts one while
//! answering for another. A deployment-aware assessment is deferred
//! until a consumer for one exists.
//!
//! Not implemented: the stack scheduler, backend proof patterns,
//! constructors, and the relocatable bundle.
//!
//! Not claimed: anything about a real node. No target program has been
//! emitted, no transaction has been built, every evidence requirement
//! the target contract names remains unresolved, and no complete
//! backend proof pattern exists — the last of those is enforced by an
//! uninhabited pattern identity rather than by convention.

#![forbid(unsafe_code)]

pub mod capability;
pub mod error;
pub mod instruction;
pub mod program;
pub mod stack;

pub use capability::{
    AssessmentDisposition, AssessmentProjection, BackendFoundationRequirement, BackendPatternId,
    EvidenceAssessmentDisposition, EvidenceAssessmentProjection, ExternalEvidenceAssessment,
    StaticCapabilityAssessment, TargetAssessmentSet, UnsupportedReason, assess_complete_census,
    assess_evidence_role, assess_requirements, assess_static_capability,
};
pub use error::TapscriptError;
pub use instruction::{StackItem, TapscriptInstruction};
pub use program::{MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptProgram};
pub use stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, resource_projection,
    validate_program,
};

#[cfg(test)]
mod tests;
