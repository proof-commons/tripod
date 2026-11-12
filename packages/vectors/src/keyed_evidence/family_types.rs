//! Family-specific claims; these internal offers are never producer receipts.
use super::{Family, Measurement, ReadingBody, Trace, Unit};
use crate::keyed_history_reader::{self as reader, Handle, Membership, RelationEvidence};
use std::collections::{BTreeMap, BTreeSet};

/// Birth evidence for first outputs and every consumed member's born-first clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BirthBody {
    pub trace: Trace,
    pub first_outputs: BTreeSet<Handle>,
    pub relations: Vec<RelationEvidence>,
}
/// Continuing members and byte-derived successor edges, never a producer assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContinuationBody {
    pub trace: Trace,
    pub relations: Vec<RelationEvidence>,
    pub successors: BTreeMap<Handle, Handle>,
}
/// Complete recurrence claims; incomplete outcomes cannot carry a union.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recurrence {
    pub source: Option<Handle>,
    pub consumed: BTreeMap<Handle, Membership>,
    pub records: BTreeSet<Handle>,
}
/// Recurrence has no acceptance rule or amount annotation.
pub type RecurrenceBody = ReadingBody<Recurrence>;
/// Schedule identity separate from the subject/member identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Birth,
    Continuation,
    Leaf,
}
/// Three distinct readings of a resource dimension; absent does not mean zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceComparison {
    pub predicted: Measurement,
    pub model: Measurement,
    pub direct: Measurement,
}
/// One member's exact fragment and its observations in a serialized context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleResources {
    pub event: reader::EventId,
    pub input: usize,
    pub member: reader::Member,
    pub schedule: Schedule,
    pub script: Vec<u8>,
    /// The authenticated depth-zero control item, never an execution argument.
    pub control: Vec<u8>,
    pub execution_arguments: usize,
    pub execution_arguments_unit: Unit,
    /// Full projector inventory, including unfilled deployment/package terms.
    pub dimensions:
        BTreeMap<target_elements::ResourceDimension, tapscript::keyed_resources::KeyedResourceTerm>,
    pub script_bytes: ResourceComparison,
    pub operations: ResourceComparison,
    pub peak: ResourceComparison,
    /// Concrete width is bounded by the projector's full-domain width, not equal to it.
    pub element_width: ResourceComparison,
}
/// Digest identity of archive bytes, not an event address or a semantic key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveDigest(pub [u8; 32]);
/// A relevant admitted row; full bytes and all observations stay in the context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchivedObservation {
    pub archive: String,
    pub step: String,
    pub digest: ArchiveDigest,
}
/// Exact serialized terms and model comparisons; no new native metrics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcesBody {
    pub trace: Trace,
    pub schedules: Vec<ScheduleResources>,
    pub members: u64,
    pub members_unit: Unit,
    pub script_stack_bytes: ResourceComparison,
    pub member_witness_bytes: ResourceComparison,
    pub member_weight: ResourceComparison,
    pub base_bytes: u64,
    pub base_bytes_unit: Unit,
    pub non_member_witness_bytes: u64,
    pub non_member_witness_bytes_unit: Unit,
    /// Projector formula evaluated on measured terms; elements is the independent comparison.
    pub transaction_weight: ResourceComparison,
}
/// Typed preparation or family-validation boundary; observation failures retain theirs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamilyRefusal {
    pub family: Option<Family>,
    pub(super) outer: FamilyBoundary,
    pub condition: FamilyCondition,
}
/// One discrepancy per condition, with owner refusals preserved verbatim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FamilyCondition {
    Reader(reader::Refusal),
    Model {
        event: reader::EventId,
        input: usize,
        schedule: Schedule,
        kind: &'static str,
    },
    /// Failed observation/request derivation, not report comparison.
    Observation(super::Refusal),
    Framing(super::Refusal),
    Decode(transaction::TransactionRefusal),
    MissingSubject(reader::EventId),
    Body,
    Trace,
    FirstOutputs,
    Relations,
    RelationEvent,
    RelationInput,
    RelationMember,
    RelationKind,
    RelationLeaf,
    RelationControl,
    RelationCounters,
    RecurrenceSource,
    RecurrenceInputs,
    RecurrenceRecords,
    Control,
    ExecutionArguments,

    Successors,
    Recurrence,
    ScheduleIdentity,
    Script,
    ResourceCount,
    ResourceUnit,
    ResourceAvailability,
    ResourceTerms,
    PredictionDisagreement,
    ProjectionShape,
    IndexDisagreement,
    ReplayDisagreement,
    RecurrenceShape,
    MissingConsumedMembership,
    MemberFormDisagreement,
    GrammarDisagreement,
    FinalityUnsupported,
    ProjectionTerms,
    Archive(crate::keyed_cost::CostRefusal),
    Arithmetic,
    Target(Vec<target_elements::TargetError>),
    Projection(tapscript::TapscriptError),
}
/// Preserved observation layer beneath the family-report boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamilyObservationBoundary {
    Reader(reader::Boundary),
    Evidence(super::Boundary),
}
/// Preparation is distinct from validating a requested family's claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamilyBoundary {
    Preparation,
    FamilyReportValidation,
}
impl FamilyRefusal {
    /// Typed outer operation boundary.
    #[must_use]
    pub const fn boundary(&self) -> FamilyBoundary {
        self.outer
    }
    /// Only observation/derivation failures carry an underlying observation boundary.
    #[must_use]
    pub const fn observation_boundary(&self) -> Option<FamilyObservationBoundary> {
        match &self.condition {
            FamilyCondition::Reader(e)
            | FamilyCondition::Observation(super::Refusal::Reader(e)) => {
                Some(FamilyObservationBoundary::Reader(e.boundary()))
            }
            FamilyCondition::Observation(e) => {
                Some(FamilyObservationBoundary::Evidence(e.boundary()))
            }
            FamilyCondition::Model { .. } => Some(FamilyObservationBoundary::Reader(
                reader::Boundary::ModelEvaluation,
            )),
            FamilyCondition::Framing(_)
            | FamilyCondition::Decode(_)
            | FamilyCondition::MissingSubject(_)
            | FamilyCondition::Body
            | FamilyCondition::Trace
            | FamilyCondition::FirstOutputs
            | FamilyCondition::Relations
            | FamilyCondition::RelationEvent
            | FamilyCondition::RelationInput
            | FamilyCondition::RelationMember
            | FamilyCondition::RelationKind
            | FamilyCondition::RelationLeaf
            | FamilyCondition::RelationControl
            | FamilyCondition::RelationCounters
            | FamilyCondition::RecurrenceSource
            | FamilyCondition::RecurrenceInputs
            | FamilyCondition::RecurrenceRecords
            | FamilyCondition::Control
            | FamilyCondition::ExecutionArguments
            | FamilyCondition::Successors
            | FamilyCondition::Recurrence
            | FamilyCondition::ScheduleIdentity
            | FamilyCondition::Script
            | FamilyCondition::ResourceCount
            | FamilyCondition::ResourceUnit
            | FamilyCondition::ResourceAvailability
            | FamilyCondition::ResourceTerms
            | FamilyCondition::PredictionDisagreement
            | FamilyCondition::ProjectionTerms
            | FamilyCondition::Archive(_)
            | FamilyCondition::Arithmetic
            | FamilyCondition::Target(_)
            | FamilyCondition::Projection(_)
            | FamilyCondition::ProjectionShape
            | FamilyCondition::IndexDisagreement
            | FamilyCondition::ReplayDisagreement
            | FamilyCondition::RecurrenceShape
            | FamilyCondition::MissingConsumedMembership
            | FamilyCondition::MemberFormDisagreement
            | FamilyCondition::GrammarDisagreement
            | FamilyCondition::FinalityUnsupported => None,
        }
    }
    /// Stable structured condition name.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match &self.condition {
            FamilyCondition::Reader(_) => "reader",
            FamilyCondition::Model { .. } => "model",
            FamilyCondition::Observation(_) => "observation",
            FamilyCondition::Framing(_) => "framing",
            FamilyCondition::Decode(_) => "decode",
            FamilyCondition::MissingSubject(_) => "missing_subject",
            FamilyCondition::Body => "body",
            FamilyCondition::Trace => "trace",
            FamilyCondition::FirstOutputs => "first_outputs",
            FamilyCondition::Relations => "relations",
            FamilyCondition::RelationEvent => "relation_event",
            FamilyCondition::RelationInput => "relation_input",
            FamilyCondition::RelationMember => "relation_member",
            FamilyCondition::RelationKind => "relation_kind",
            FamilyCondition::RelationLeaf => "relation_leaf",
            FamilyCondition::RelationControl => "relation_control",
            FamilyCondition::RelationCounters => "relation_counters",
            FamilyCondition::RecurrenceSource => "recurrence_source",
            FamilyCondition::RecurrenceInputs => "recurrence_inputs",
            FamilyCondition::RecurrenceRecords => "recurrence_records",
            FamilyCondition::Control => "control",
            FamilyCondition::ExecutionArguments => "execution_arguments",
            FamilyCondition::Successors => "successors",
            FamilyCondition::Recurrence => "recurrence",
            FamilyCondition::ScheduleIdentity => "schedule_identity",
            FamilyCondition::Script => "script",
            FamilyCondition::ResourceCount => "resource_count",
            FamilyCondition::ResourceUnit => "resource_unit",
            FamilyCondition::ResourceAvailability => "resource_availability",
            FamilyCondition::ResourceTerms => "resource_terms",
            FamilyCondition::PredictionDisagreement => "prediction_disagreement",
            FamilyCondition::ProjectionTerms => "projection_terms",
            FamilyCondition::Archive(_) => "archive",
            FamilyCondition::Arithmetic => "arithmetic",
            FamilyCondition::Target(_) => "target",
            FamilyCondition::Projection(_) => "projection",
            FamilyCondition::ProjectionShape => "projection_shape",
            FamilyCondition::IndexDisagreement => "index_disagreement",
            FamilyCondition::ReplayDisagreement => "replay_disagreement",
            FamilyCondition::RecurrenceShape => "recurrence_shape",
            FamilyCondition::MissingConsumedMembership => "missing_consumed_membership",
            FamilyCondition::MemberFormDisagreement => "member_form_disagreement",
            FamilyCondition::GrammarDisagreement => "grammar_disagreement",
            FamilyCondition::FinalityUnsupported => "finality_unsupported",
        }
    }
}
pub(super) const fn present(value: u64, unit: Unit) -> Measurement {
    Measurement::Present { value, unit }
}
