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
//! Implemented: the package boundary and the capability adapter — a
//! multi-state assessment of each compiler capability against one
//! validated target and development binding, with exact census equality
//! against the analysis that published the requirements.
//!
//! Not implemented: the target program type, the instruction builder,
//! the stack scheduler, backend proof patterns, constructors, and the
//! relocatable bundle.
//!
//! Not claimed: anything about a real node. No target program has been
//! emitted, no transaction has been built, every evidence requirement
//! the target contract names remains unresolved, and no complete
//! backend proof pattern exists — the last of those is enforced by an
//! uninhabited pattern identity rather than by convention.

#![forbid(unsafe_code)]

pub mod capability;
pub mod error;

pub use capability::{
    AssessmentDisposition, AssessmentProjection, BackendFoundationRequirement, BackendPatternId,
    CapabilityAssessment, CapabilityAssessmentSet, UnsupportedReason, assess_capability,
    assess_complete_census, assess_requirements,
};
pub use error::TapscriptError;

#[cfg(test)]
mod tests;
