#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod capability;
pub mod error;
pub mod instruction;
pub mod operation_assessment;
pub mod pattern;
pub mod policy;
pub mod program;
pub mod shape;
pub mod stack;

pub use capability::{
    AssessmentDisposition, AssessmentProjection, BackendFoundationRequirement, BackendPatternId,
    EvidenceAssessmentDisposition, EvidenceAssessmentProjection, ExternalEvidenceAssessment,
    StaticCapabilityAssessment, TargetAssessmentSet, UnsupportedReason, assess_complete_census,
    assess_evidence_role, assess_requirements, assess_static_capability,
};
pub use error::TapscriptError;
pub use instruction::{StackItem, TapscriptInstruction};
pub use operation_assessment::{
    EmissionRefusal, OperationAssessmentSet, OperationRequirement, OperationVerdict, VerdictGround,
    assess_operation_plan,
};
pub use pattern::{
    AUTHORIZATION_PRIMITIVES, AbiAssumption, BackendPattern, CompactAshSymbols, MutationOutcome,
    PatternFailure, PatternMutation, PatternOwner, PatternResources, PatternStackContract,
    build_pattern, carries_authorization, coordinator_program, member_program, operation_patterns,
};
pub use policy::{
    AshRepresentationSelection, CompactAshBackendPolicy, ConcreteCandidate, ExactTargetProjection,
    LayoutFamily, SelectionObjective, SelectionRefusal, SemanticEquivalence, TieBreak,
    demonstration_policy, reviewed_target_projection,
};
pub use program::{MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptProgram};
pub use shape::{
    CandidateShapeSet, CompactAshShape, CompactAshShapeBounds, MINIMUM_ASH_INPUTS, ShapeRejection,
    SponsorChangePresence, UsefulCandidateCondition, demonstration_shape_set,
};
pub use stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, resource_projection,
    validate_program,
};

#[cfg(test)]
mod tests;
