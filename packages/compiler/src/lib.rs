#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod analyzed;
mod analyzed_operation;
mod analyzed_validate;
mod capability;
mod carrier;
mod case;
mod constructibility;
mod coverage;
mod coverage_graph;
mod disclosure;
mod engine_attestation;
mod engine_case_facts;
// Finite owner tables feed the production whole-program analysis.
mod engine_cases;
mod engine_constructibility;
mod engine_coverage;
mod engine_coverage_graph;
mod engine_coverage_join;
mod engine_foundation;
mod engine_identity;
#[cfg(any(test, feature = "keyed-experimental"))]
mod engine_keyed;
mod engine_lifecycle;
mod engine_operation;
mod engine_placement;
mod engine_program;
mod engine_proof;
mod engine_records;
mod engine_requirement;
mod engine_source;
pub mod error;
mod expression;
mod fold;
mod foundation;
pub mod input;
mod instrumentation;
mod layout;
mod lifecycle;
pub mod live_transfer_plan;
pub mod maturity_announcement_plan;
mod maturity_announcement_requirements;
pub mod operation_plan;
mod placement;
mod proof;
mod relation;
mod requirement;
mod search_counter;
mod source;
mod sponsor_region;
pub mod target;

pub use constructibility::validate_required_dependency;
pub use error::{CompileError, OwnerProgramDefect, OwnerProgramStage};
pub use expression::ExpressionCycleComponent;
pub use input::{
    AnalysisPolicy, BoundCompilerInput, CompilationScope, ProofSearchLimits, bind_input,
};
pub use lifecycle::validate_compact_lifecycle_projection;
pub use maturity_announcement_plan::validate_maturity_announcement_plan;
pub use maturity_announcement_requirements::{
    AnnouncementConstructorRole, AnnouncementDuty, AnnouncementMetadataRequirement,
    AnnouncementPolicyContinuity, AnnouncementPredecessorMaturity, AnnouncementPublicationRole,
    AnnouncementRecoveryInputRole, AnnouncementRecoveryStep, AnnouncementRequirementBoundary,
    AnnouncementRequirementMutation, AnnouncementRootHistoryCheck, AnnouncementStaticContinuity,
    ConstructorContinuityRequirement, PublicRecoveryRequirement, RootHistoryRequirement,
    StateFieldLaw, StateFieldLawKind, StateFieldRequirement, StateFieldRequirementError,
    StateLawOperand, StateSuccessionRequirement,
};
pub use relation::RelationCycleComponent;
pub use source::OperandId;
pub use sponsor_region::{
    SponsorEnvelopeMultiplicityError, validate_sponsor_envelope_multiplicity,
};

#[cfg(test)]
mod tests;
