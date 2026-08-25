#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abi_validation;
pub mod bundle;
pub mod comparison;
pub mod confidential_materializer;
pub mod confidential_predecessor;
pub mod divergence;
pub mod error;
pub mod first_party;
pub mod fixture;
pub mod live_capability;
pub mod live_comparison;
pub mod live_disclosure;
pub mod live_evidence;
pub mod live_fault_discharge;
pub mod live_first_party;
pub mod live_measurements;
pub mod live_minimality_report;
pub mod live_native;
pub mod live_owner_observation;
pub mod live_pairs;
pub mod live_plan;
pub mod live_proof_bearing_observation;
pub mod live_report;
pub mod live_resource_report;
pub mod live_resources;
pub mod live_restart;
pub mod live_roles;
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
pub use live_comparison::{
    ComparisonStanding, PlanResourceComparison, ResourcePlannerFailure, UnobservedReason,
    compare_run, run_agreements, run_failures,
};
pub use live_disclosure::{
    AdditionalDisclosureReason, DisclosedItem, DisclosureClass, DisclosureRow, DisclosureStanding,
    additional_exact_amount_disclosures, disclosure_difference, disclosure_index, disclosure_table,
    recorded_classes, shape_leakage,
};
pub use live_evidence::{
    DischargingValidator, FirstPartyGap, LiveEvidenceCensus, LiveEvidenceRow,
    LiveInfrastructureBlocker, LiveRowStanding, LiveTransferEvidencePlan,
    MinimalityRegistryStanding, blocker_census, carried_residuals, derive_live_evidence_plan,
};
pub use live_fault_discharge::{
    FaultMutation, LiveFaultCase, LiveFaultRefusal, LiveFaultValidator, ObservedFaultRefusal,
    ValidatedLiveFaultEvidence, discharge_live_faults, live_fault_cases, validate_live_fault,
};
pub use live_first_party::{
    LiveFirstPartyCase, LiveFirstPartyRefusal, LiveFirstPartyValidator, LiveOwnerScenario,
    LiveResponseMalformation, ValidatedLiveFirstPartyEvidence, discharge_live_first_party,
    live_first_party_cases, validate_live_first_party,
};
pub use live_measurements::{
    CaseMeasurement, DimensionStanding, LiveResourceCase, LiveResourceNonClaim, LiveResourceRecord,
    MEASURED_DESTINATION_RANDOMNESS, MEASURED_PREDECESSOR_RANDOMNESS, MEASURED_RECEIPT_UNIT,
    MEASURED_SPONSOR_CHANGE, MEASURED_SPONSOR_FEE, MeasuredSponsorRole, MeasurementRecipe,
    ResourceStudyRefusal, TransactionMeasurement, deepest_committed_shape, measure_resource_cases,
    measurement_recipes,
};
pub use live_minimality_report::{
    FailureModeStanding, LIVE_MINIMALITY_REPORT_SCHEMA, LIVE_MINIMALITY_SCHEMA_ID,
    LifecycleConclusion, LiveMinimalityDiagnostics, LiveMinimalityReportRefusal,
    LiveMinimalityReportRole, LiveTransferMinimalityReport, MinimalityFailureMode,
    MinimalityPairCensus, MinimalityStanding, PlanDisclosure, PrivacyNonClaim,
    ValidatedLiveTransferMinimalityReport, assemble_live_minimality_report,
    canonical_bytes_publish_no_forbidden_key, disclosure_comparison, item_standings, pair_census,
    pair_standings, render_live_minimality_report, resolve_failure_modes,
    validate_live_minimality_report,
};
pub use live_native::{
    LiveFormNotSubmitted, LiveNativeObservation, LiveNativeRefusal, LiveNativeStep,
    LiveNativeTranscript, LiveTransferOperationPlanner, PredictedTransferResources,
    observed_run_of_record, render_live_native_run,
};
pub use live_pairs::{
    ExpectedTransferSemantics, MinimalityConditionStanding, MinimalityPair, MinimalityPairRefusal,
    MinimalityPairRow, PairAcceptanceCondition, PairMaterialization, PairShapeClaim,
    PairTargetVerdict, PredecessorAssumption, ResourceComparisonStanding, SemanticEndpoint,
    SemanticTransferFixture, SponsorPresence, UnclaimedPairReason, build_minimality_pairs,
    condition_scoreboard, minimality_fixtures,
};
pub use live_plan::{
    demonstration_live_abi, demonstration_live_bundle, link_live_bundle_for_asset,
    live_abi_for_asset, live_deployment_for_asset, live_transfer_plan, published_owner,
    relocatable_live_bundles,
};
pub use live_report::{
    LIVE_SAFETY_REPORT_SCHEMA, LiveLifecycleStatus, LiveRunStanding, LiveSafetyCompleteness,
    LiveSafetyDiagnostics, LiveSafetyReportRefusal, LiveSafetyReportRole, LiveTransferSafetyReport,
    RecomputedItem, ValidatedLiveTransferSafetyReport, VolatileField, assemble_live_safety_report,
    canonical_bytes_publish_no_sponsor_value, render_live_safety_report, section_scoreboard,
    validate_live_safety_report,
};
pub use live_resource_report::{
    CandidateBoundsResult, LIVE_RESOURCE_REPORT_SCHEMA, LIVE_RESOURCE_SCHEMA_ID,
    LiveResourceDiagnostics, LiveResourceReportRefusal, LiveResourceReportRole,
    LiveTransferResourceReport, ResourceNonClaim, ResourceStudyCensus,
    ValidatedLiveTransferResourceReport, assemble_live_resource_report,
    render_live_resource_report, resource_census, validate_live_resource_report,
};
pub use live_resources::{
    CandidateBoundCost, CandidatePositionDomain, CandidateTreeAdmission,
    RESEARCH_RECEIPT_INPUT_BOUNDS, RESEARCH_RECEIPT_OUTPUT_BOUNDS, RESEARCH_SPONSOR_INPUT_BOUNDS,
    admitted_shape_count, assignment_fits_tested_set, assignments_realized_by,
    bound_assignment_costs, committed_leaf_count, committed_leaves, cost_bound_assignment,
    research_bound_assignments, tree_admission,
};
pub use live_safety::{
    LiveRelationStanding, LiveRowBoundary, LiveRowLink, LiveSafetyPolarity, LiveSafetyRow,
    LiveSafetySection, LiveUnlinkedReason, required_safety_matrix, resolve_row, row_count,
    section_census,
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
