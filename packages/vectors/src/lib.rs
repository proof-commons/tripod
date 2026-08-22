#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod bundle;
pub mod comparison;
pub mod divergence;
pub mod error;
pub mod fixture;
pub mod materialize;
pub mod matrix;
pub mod mutation;
pub mod operation;
pub mod plan;
pub mod projection;
pub mod report;
pub mod resource_study;
pub mod subject;
pub mod violation;

pub use comparison::{
    ObservedProjection, ProjectionRefusal, ProjectionTerm, compare, read_accepted,
};
pub use divergence::{
    AmountBeyondTargetBound, StatedAmountPlace, TargetAmountStanding, target_amount_standing,
};
pub use error::{FixtureBundleRefusal, VectorError};
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
pub use report::{
    OPERATION_REPORT_SCHEMA, ProjectionVerdict, ReportValidationRefusal,
    ValidatedCompactAshOperationReport, validate_operation_report,
};
pub use subject::{CanonicalSubject, ExperimentalSubject, SubjectStanding};
pub use violation::{
    FirstPartyEvidence, IntendedViolation, UnlinkedReason, first_party_evidence,
    matching_requirement,
};
