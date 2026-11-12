//! Refusals are evidence/rule/framing failures, never covenant execution refusals.
use super::Dimension;
use crate::keyed_history_reader::{EventId, Handle, Missing};
use std::collections::{BTreeMap, BTreeSet};
/// All incomplete keyed handles, not only the first failure.
pub type MissingHandles = BTreeMap<Handle, BTreeSet<Missing>>;
/// All supplied unexamined events with missing evidence.
pub type MissingEvents = BTreeMap<EventId, BTreeSet<Missing>>;
/// Layer at which a reading/report offer is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    ReaderEvidence,
    ReaderRule,
    Annotation,
    ReportFraming,
    Quantity,
}
/// Required report component that is absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Subject,
    Provenance,
    Trace,
    Evidence,
    Assumptions,
}
/// Empty external provenance component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceField {
    Deployment,
    PolicyVersion,
    HistorySource,
    PolicySource,
    Inclusion,
    Unspentness,
    Retrieval,
    Finality,
}
/// One condition per variant; owner errors retain their own identity and boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// incomplete mismatch at the `ReportFraming` boundary.
    IncompleteMismatch,
    /// body mismatch at the `ReportFraming` boundary.
    BodyMismatch,
    /// subject missing at the `ReportFraming` boundary.
    SubjectMissing(Handle),
    /// finality hypothetical at the `ReaderEvidence` boundary.
    FinalityHypothetical,
    /// finality inclusion mismatch at the `ReaderEvidence` boundary.
    FinalityInclusionMismatch,
    /// finality retrieval mismatch at the `ReaderEvidence` boundary.
    FinalityRetrievalMismatch,

    /// Changed observation epoch or boundary.
    ObservationMismatch,
    /// A closure report concealed or changed another supplied event's missing evidence.
    UnexaminedMismatch,
    /// No named external acceptance of the declaration.
    FinalityAcceptanceSource,
    /// Reader at the `ReaderEvidence` boundary.
    Reader(crate::keyed_history_reader::Refusal),
    /// Query at the `ReaderEvidence` boundary.
    Query(crate::keyed_history_reader::Query),
    /// Incomplete at the `ReaderEvidence` boundary.
    Incomplete {
        handle: Handle,
        missing: BTreeSet<Missing>,
    },
    /// Every failure in the supplied history, after the queried closure check.
    WholeHistory {
        handles: MissingHandles,
        events: MissingEvents,
    },
    /// Annotation missing at the Annotation boundary.
    AnnotationMissing(Handle),
    /// Annotation decode at the Annotation boundary.
    AnnotationDecode(transaction::TransactionRefusal),
    /// Same epoch at the `ReaderRule` boundary.
    SameEpoch,
    /// Unknown epoch at the `ReaderRule` boundary.
    UnknownEpoch,
    /// Empty provenance at the `ReaderEvidence` boundary.
    EmptyProvenance(ProvenanceField),
    /// Finality missing at the `ReaderEvidence` boundary.
    FinalityMissing,
    /// Finality deployment at the `ReaderEvidence` boundary.
    FinalityDeployment,
    /// Finality anchor at the `ReaderEvidence` boundary.
    FinalityAnchor,
    /// Finality policy at the `ReaderEvidence` boundary.
    FinalityPolicy,
    /// Finality inclusion source at the `ReaderEvidence` boundary.
    FinalityInclusionSource,
    /// Finality retrieval source at the `ReaderEvidence` boundary.
    FinalityRetrievalSource,
    /// Finality unspentness scope at the `ReaderEvidence` boundary.
    FinalityUnspentnessScope,
    /// Finality rule at the `ReaderEvidence` boundary.
    FinalityRule,
    /// Missing field at the `ReportFraming` boundary.
    MissingField(Field),
    /// Schema mismatch at the `ReportFraming` boundary.
    SchemaMismatch,
    /// Family mismatch at the `ReportFraming` boundary.
    FamilyMismatch,
    /// Subject bytes mismatch at the `ReportFraming` boundary.
    SubjectBytesMismatch,
    /// Subject identity mismatch at the `ReportFraming` boundary.
    SubjectIdentityMismatch,
    /// Subject handle mismatch at the `ReportFraming` boundary.
    SubjectHandleMismatch,
    /// Provenance mismatch at the `ReportFraming` boundary.
    ProvenanceMismatch,
    /// Trace mismatch at the `ReportFraming` boundary.
    TraceMismatch,
    /// Rule mismatch at the `ReportFraming` boundary.
    RuleMismatch,
    /// Evidence mismatch at the `ReportFraming` boundary.
    EvidenceMismatch,
    /// Assumptions mismatch at the `ReportFraming` boundary.
    AssumptionsMismatch,
    /// Outcome mismatch at the `ReportFraming` boundary.
    OutcomeMismatch,
    /// Membership mismatch at the `ReportFraming` boundary.
    MembershipMismatch,
    /// Accepted mismatch at the `ReportFraming` boundary.
    AcceptedMismatch,
    /// Annotations mismatch at the `ReportFraming` boundary.
    AnnotationsMismatch,
    /// Quantity missing at the Quantity boundary.
    QuantityMissing(Dimension),
    /// Quantity unit at the Quantity boundary.
    QuantityUnit(Dimension),
    /// Quantity value at the Quantity boundary.
    QuantityValue(Dimension),
    /// Quantity availability at the Quantity boundary.
    QuantityAvailability(Dimension),
    /// Quantity overflow at the Quantity boundary.
    QuantityOverflow(Dimension),
}
impl Refusal {
    /// Exhaustive layer mapping.
    #[must_use]
    pub const fn boundary(&self) -> Boundary {
        match self {
            Self::FinalityHypothetical
            | Self::FinalityInclusionMismatch
            | Self::FinalityRetrievalMismatch
            | Self::FinalityAcceptanceSource
            | Self::Reader(..)
            | Self::Query(..)
            | Self::Incomplete { .. }
            | Self::WholeHistory { .. }
            | Self::EmptyProvenance(..)
            | Self::FinalityMissing
            | Self::FinalityDeployment
            | Self::FinalityAnchor
            | Self::FinalityPolicy
            | Self::FinalityInclusionSource
            | Self::FinalityRetrievalSource
            | Self::FinalityUnspentnessScope
            | Self::FinalityRule => Boundary::ReaderEvidence,
            Self::AnnotationMissing(..) | Self::AnnotationDecode(..) => Boundary::Annotation,
            Self::SameEpoch | Self::UnknownEpoch => Boundary::ReaderRule,
            Self::IncompleteMismatch
            | Self::BodyMismatch
            | Self::SubjectMissing(..)
            | Self::ObservationMismatch
            | Self::MissingField(..)
            | Self::UnexaminedMismatch
            | Self::SchemaMismatch
            | Self::FamilyMismatch
            | Self::SubjectBytesMismatch
            | Self::SubjectIdentityMismatch
            | Self::SubjectHandleMismatch
            | Self::ProvenanceMismatch
            | Self::TraceMismatch
            | Self::RuleMismatch
            | Self::EvidenceMismatch
            | Self::AssumptionsMismatch
            | Self::OutcomeMismatch
            | Self::MembershipMismatch
            | Self::AcceptedMismatch
            | Self::AnnotationsMismatch => Boundary::ReportFraming,
            Self::QuantityMissing(..)
            | Self::QuantityUnit(..)
            | Self::QuantityValue(..)
            | Self::QuantityAvailability(..)
            | Self::QuantityOverflow(..) => Boundary::Quantity,
        }
    }
    /// Stable tracing vocabulary.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::FinalityRetrievalMismatch => "finality_retrieval_mismatch",
            Self::FinalityInclusionMismatch => "finality_inclusion_mismatch",
            Self::FinalityHypothetical => "finality_hypothetical",
            Self::SubjectMissing(..) => "subject_missing",
            Self::BodyMismatch => "body_mismatch",
            Self::IncompleteMismatch => "incomplete_mismatch",
            Self::ObservationMismatch => "observation_mismatch",
            Self::UnexaminedMismatch => "unexamined_mismatch",
            Self::FinalityAcceptanceSource => "finality_acceptance_source",
            Self::Reader(..) => "reader",
            Self::Query(..) => "query",
            Self::Incomplete { .. } => "incomplete",
            Self::WholeHistory { .. } => "whole_history",
            Self::AnnotationMissing(..) => "annotation_missing",
            Self::AnnotationDecode(..) => "annotation_decode",
            Self::SameEpoch => "same_epoch",
            Self::UnknownEpoch => "unknown_epoch",
            Self::EmptyProvenance(..) => "empty_provenance",
            Self::FinalityMissing => "finality_missing",
            Self::FinalityDeployment => "finality_deployment",
            Self::FinalityAnchor => "finality_anchor",
            Self::FinalityPolicy => "finality_policy",
            Self::FinalityInclusionSource => "finality_inclusion_source",
            Self::FinalityRetrievalSource => "finality_retrieval_source",
            Self::FinalityUnspentnessScope => "finality_unspentness_scope",
            Self::FinalityRule => "finality_rule",
            Self::MissingField(..) => "missing_field",
            Self::SchemaMismatch => "schema_mismatch",
            Self::FamilyMismatch => "family_mismatch",
            Self::SubjectBytesMismatch => "subject_bytes_mismatch",
            Self::SubjectIdentityMismatch => "subject_identity_mismatch",
            Self::SubjectHandleMismatch => "subject_handle_mismatch",
            Self::ProvenanceMismatch => "provenance_mismatch",
            Self::TraceMismatch => "trace_mismatch",
            Self::RuleMismatch => "rule_mismatch",
            Self::EvidenceMismatch => "evidence_mismatch",
            Self::AssumptionsMismatch => "assumptions_mismatch",
            Self::OutcomeMismatch => "outcome_mismatch",
            Self::MembershipMismatch => "membership_mismatch",
            Self::AcceptedMismatch => "accepted_mismatch",
            Self::AnnotationsMismatch => "annotations_mismatch",
            Self::QuantityMissing(..) => "quantity_missing",
            Self::QuantityUnit(..) => "quantity_unit",
            Self::QuantityValue(..) => "quantity_value",
            Self::QuantityAvailability(..) => "quantity_availability",
            Self::QuantityOverflow(..) => "quantity_overflow",
        }
    }
}
