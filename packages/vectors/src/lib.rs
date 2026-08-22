#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abi_validation;
pub mod bundle;
pub mod comparison;
pub mod divergence;
pub mod error;
pub mod first_party;
pub mod fixture;
pub mod live_capability;
pub mod live_evidence;
pub mod live_first_party;
pub mod live_plan;
pub mod live_report;
pub mod live_safety;
pub mod materialize;
pub mod matrix;
pub mod mutation;
pub mod operation;
pub mod plan;
pub mod projection;
pub mod render;
pub mod report;
pub mod resource_study;
pub mod subject;
pub mod violation;

pub use abi_validation::{
    AbiValidationOutcome, AbiValidationRow, index_abi_validation, precedes_the_target,
};
pub use comparison::{
    ObservedProjection, ProjectionRefusal, ProjectionTerm, compare, read_accepted,
};
pub use divergence::{
    AmountBeyondTargetBound, StatedAmountPlace, TargetAmountStanding, target_amount_standing,
};
pub use error::{FixtureBundleRefusal, VectorError};
pub use first_party::{
    FirstPartyEvidenceRefusal, FirstPartyNegativeCase, FirstPartyRefusal, FirstPartyValidator,
    ValidatedFirstPartyNegativeEvidence, validate_first_party_negative,
};
pub use live_capability::{OracleFixtureValues, OracleLiveCurve};
pub use live_evidence::{
    FirstPartyGap, LiveEvidenceCensus, LiveEvidenceRow, LiveInfrastructureBlocker, LiveRowStanding,
    LiveTransferEvidencePlan, MinimalityRegistryStanding, blocker_census, carried_residuals,
    derive_live_evidence_plan,
};
pub use live_first_party::{
    LiveFirstPartyCase, LiveFirstPartyRefusal, LiveFirstPartyValidator, LiveOwnerScenario,
    LiveResponseMalformation, ValidatedLiveFirstPartyEvidence, discharge_live_first_party,
    live_first_party_cases, validate_live_first_party,
};
pub use live_plan::{
    demonstration_live_abi, demonstration_live_bundle, live_transfer_plan, published_owner,
};
pub use live_report::{
    LIVE_SAFETY_REPORT_SCHEMA, LiveLifecycleStatus, LiveRunStanding, LiveSafetyCompleteness,
    LiveSafetyDiagnostics, LiveSafetyReportRefusal, LiveSafetyReportRole, LiveTransferSafetyReport,
    RecomputedItem, ValidatedLiveTransferSafetyReport, VolatileField, assemble_live_safety_report,
    canonical_bytes_publish_no_sponsor_value, render_live_safety_report, section_scoreboard,
    validate_live_safety_report,
};
pub use live_safety::{
    LiveRelationStanding, LiveRowLink, LiveSafetyPolarity, LiveSafetyRow, LiveSafetySection,
    LiveUnlinkedReason, required_safety_matrix, resolve_row, row_count, section_census,
};
pub use materialize::{
    SponsorCoin, SponsorSigningTask, has_candidate_program, materialize_sponsored,
    needs_authorization, sponsor_signing_requests,
};
pub use matrix::{
    EvidenceBoundary, MutationLayer, VectorClass, VectorFamily, VectorPolarity, all_classes,
    class_count, family_census,
};
pub use plan::{
    CompactAshEvidencePlan, NegativeObservability, PlanCensus, RequiredTargetWork,
    derive_evidence_plan,
};
pub use render::{render_refused_run, render_validated_report};
pub use report::{
    OPERATION_REPORT_SCHEMA, ProjectionVerdict, ReportValidationRefusal,
    ValidatedCompactAshOperationReport, validate_operation_report,
};
pub use subject::{CanonicalSubject, ExperimentalSubject, SubjectStanding};
pub use violation::{
    ContradictedExpectation, DeclarationLink, FirstPartyEvidence, IntendedViolation,
    NegativeVectorDeclaration, SemanticChange, SourceFixtureRequirement, TargetField,
    UnlinkedReason, first_party_evidence, matching_requirement, resolve_declaration,
};
