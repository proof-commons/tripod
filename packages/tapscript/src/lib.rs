#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod authorization;
pub mod bundle;
pub mod capability;
pub mod error;
pub mod instruction;
pub mod live_constructor;
pub mod live_pattern;
pub mod live_shape;
pub mod operation_assessment;
pub mod pattern;
pub mod policy;
pub mod program;
pub mod shape;
pub mod stack;
pub mod upstream;

pub use authorization::{
    DimensionRefusal, DimensionRole, OwnerKeyEncodingClosure, OwnerKeyNegative, OwnerKeyObligation,
    OwnerProfileDisposition, OwnerSighashProfile, ProtectedDatum, owner_key_encoding_closure,
    profile_classifies_every_offered_dimension, selected_owner_profile,
};
pub use bundle::{
    BackendArtifactStatus, BundleRefusal, BundleSymbol, CandidateRelocatableTapscriptBundle,
    ConcreteCarrierSite, ConcreteLayout, ConcreteRelationPlacement, ConstructorAssumption,
    ExplicitValuePolicy, FieldSide, InputPlacement, InputRole, InternalKeyPolicy,
    IntrospectionReference, KeyPathPolicy, LeafProgram, LeafRole, OutputPlacement, OutputRole,
    OutstandingLifecycle, ProgramResources, ProgramRole, Relocation, RelocationEncoding,
    RelocationSite, ResourceModel, ResourceObligation, ShapeResourceFormula, SharedLeafProof,
    SharingGround, StaticAshConstructor, SubstitutionMode, SymbolBinding, SymbolEntry, SymbolWidth,
    TargetRole, WitnessComponent, WitnessRole, emit_candidate_bundle, fit_shape_model,
    predict_shape_model,
};
pub use capability::{
    AssessmentDisposition, AssessmentProjection, BackendFoundationRequirement, BackendPatternId,
    EvidenceAssessmentDisposition, EvidenceAssessmentProjection, ExternalEvidenceAssessment,
    StaticCapabilityAssessment, TargetAssessmentSet, UnsupportedReason, assess_complete_census,
    assess_evidence_role, assess_requirements, assess_static_capability,
};
pub use error::TapscriptError;
pub use instruction::{StackItem, TapscriptInstruction};
pub use live_constructor::{
    BindingStatus, CandidateTransferLifecycle, ConstructorBinding, ConstructorDisposition,
    ConstructorFacet, ConstructorMutationCase, ConstructorMutationCaseId, KeyPathClosure,
    LiveConstructorRefusal, LiveProgramRole, LiveSpendingRoute, LiveTransferLeafRole,
    MutationCensusDefect, MutationResidual, OwnerKey, OwnerKeyEstablishment, OwnerKeyRejection,
    OwnerKeyResidual, PendingBinding, StaticLiveReceiptConstructor, constructor_mutation_cases,
    derive_live_receipt_constructor, key_path_closure, mutation_census_defects,
    static_transfer_leaf_set,
};
pub use live_pattern::{
    CoordinatorGlobalCheck, FinalStackDefect, GlobalCheckPlacement, GlobalCheckStatus,
    LiveConstructibility, LiveDisclosure, LiveFragmentId, LivePatternOwner, LiveProgramRefusal,
    LiveTransferPattern, LiveTransferPatternId, LiveTransferSymbols, LiveWitnessRole,
    NegativeDisposition, OutstandingGlobalPattern, OwnerAssignmentRejection, OwnerKeyMutation,
    OwnerKeyMutationGate, OwnerKeyMutationOutcome, OwnerKeyOracle, PlacementDefect,
    ReceiptOwnerAssignment, RecognitionCarrier, RecognitionEstablishment, RecognitionResidual,
    RecognizedFact, build_live_pattern, coordinator_placements, emitted_fragments,
    every_emitted_fragment, final_stack_defects, has_member_position, live_cardinality_fragment,
    live_coordinator_program, live_member_program, live_member_role_fragment,
    live_owner_profile_disposition, live_program_precondition, live_transfer_patterns,
    local_recognition_fragment, mutated_owner_authorization_fragment, mutation_gate,
    negative_disposition, owner_authorization_fragment, owner_authorization_precondition,
    owner_key_mutation_outcome, owner_key_obligation, patterns_for, recognition_establishments,
    validate_coordinator_placements,
};
pub use live_shape::{
    LiveShapeRejection, LiveTransferShape, LiveTransferShapeBounds, LiveTransferShapeSet,
    MINIMUM_TRANSFER_RECEIPT_INPUTS, MINIMUM_TRANSFER_RECEIPT_OUTPUTS,
    demonstration_live_shape_set, dense_live_shape_set,
};
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
    SponsorChangePresence, UsefulCandidateCondition, demonstration_shape_set, dense_shape_set,
};
pub use stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, SignatureSuccessForm,
    resource_projection, validate_program,
};

#[cfg(test)]
mod tests;
