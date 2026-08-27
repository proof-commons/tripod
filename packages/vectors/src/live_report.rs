//! The validated live-transfer safety report (§13.2, §13.5, §13.6).
//!
//! §13.2 asks one question — did every valid transfer preserve the exact
//! semantic relation, and did every required invalid transfer fail at its
//! owning boundary — and names twelve typed carriers the answer travels
//! in. §13.5 says the gate accepts a validated wrapper rather than a raw
//! report, names fourteen possible recomputations, and excludes ten
//! volatile fields from the canonical bytes. This module is all three.
//!
//! # The wrapper is the whole point
//!
//! [`LiveTransferSafetyReport`] is a value a caller can build, and it
//! establishes nothing. [`ValidatedLiveTransferSafetyReport`] has private
//! fields and one constructor, [`validate_live_safety_report`], which
//! records an item only after its comparison ran. A report whose summary
//! disagreed with its own rows is refused rather than published, and an
//! absent run binding cannot manufacture six passing run comparisons.
//!
//! # Timing is not in here, and that is structural
//!
//! §13.6 puts timing in a separate noncanonical diagnostic report, and
//! [`LiveSafetyDiagnostics`] is it. The canonical renderer takes a
//! validated report and nothing else, so the ten volatile fields are not
//! *filtered* out of the bytes — they are not reachable from the value
//! the renderer reads. `the_canonical_bytes_carry_no_volatile_field`
//! checks that no other carrier smuggles one in anyway.
//!
//! # A report is not a verdict about the target
//!
//! A report every one of whose rows is blocked is as valid as one whose
//! every row is discharged. What changes is the completeness token, and
//! §13.5's bar is that a report may not call itself complete while a
//! required row is unanswered. That is the honest state today, and the
//! report says so in a typed field rather than by the absence of
//! failures.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use target_elements::TargetProjection;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use transaction::Txid;

use crate::live_evidence::{
    LiveEvidenceCensus, LiveInfrastructureBlocker, LiveRowStanding, LiveTransferEvidencePlan,
    RecordedObservation, blocker_census,
};
use crate::live_safety::LiveSafetySection;
use crate::matrix::EvidenceBoundary;

/// The schema of the canonical rendered safety report.
///
/// Stated in the bytes so a reader never has to infer which revision a
/// file is: a report whose field set changed under a reader that assumed
/// the old one would be read wrong rather than refused.
///
/// Revision 2 adds the `operation_vocabulary_closed` census line and the
/// `operation-vocabulary-closed` outstanding spelling. A revision-1
/// reader summing the census lines it knows would find them short of the
/// row count, which is exactly the misreading a stated schema exists to
/// turn into a refusal.
///
/// Revision 3 adds the evidence corpus ledger, the
/// `recorded_observation_unbound` census, every observation bucket, and
/// the `native_refusal_at_unexpected_boundary` census and outstanding
/// spelling. It also makes `failed` reachable. Schema 2 is hard-rejected:
/// it has neither the ledger nor the boundary distinction and is never
/// silently reinterpreted as evidence-bearing schema 3.
pub const LIVE_SAFETY_REPORT_SCHEMA: u32 = 3;

/// What a safety report is, said in the bytes.
///
/// One variant, and it is named rather than assumed: §13.6 forbids the
/// safety and minimality reports substituting for one another, and a
/// reader holding only the bytes needs to be able to tell which it has.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveSafetyReportRole {
    /// The live-transfer safety report of §13.2.
    LiveTransferSafety,
}

impl LiveSafetyReportRole {
    /// The role's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LiveTransferSafety => "live-transfer-safety",
        }
    }
}

/// Whether a report may call itself complete (§13.5).
///
/// Three states and no inference. A report never implies completeness by
/// the absence of failures, so the token is explicit and the middle state
/// exists to keep "nothing failed" and "everything ran" apart.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveSafetyCompleteness {
    /// Every required row of §15 is answered at its own boundary.
    CompleteForTheRequiredMatrix,
    /// Some required row is unanswered, and the report names which.
    PartialRequiredRowsOutstanding,
    /// A required row was answered at a boundary other than its own.
    Failed,
}

impl LiveSafetyCompleteness {
    /// The token's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CompleteForTheRequiredMatrix => "complete-for-the-required-matrix",
            Self::PartialRequiredRowsOutstanding => "partial-required-rows-outstanding",
            Self::Failed => "failed",
        }
    }
}

/// The exact deployment fields one validated target run binds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveDeploymentBinding {
    environment: String,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    target_contract: String,
}

impl LiveDeploymentBinding {
    /// The deployment environment.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// The network identity in target display order.
    #[must_use]
    pub const fn network_id(&self) -> &[u8; 32] {
        &self.network_id
    }

    /// The genesis identity in target display order.
    #[must_use]
    pub const fn genesis_id(&self) -> &[u8; 32] {
        &self.genesis_id
    }

    /// The target contract revision recorded by the run.
    #[must_use]
    pub fn target_contract(&self) -> &str {
        &self.target_contract
    }
}

/// The stable executor self-description one validated target run binds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveExecutorProvenance {
    adapter_name: String,
    adapter_version: String,
    node_name: String,
    node_version: String,
    executed_source_tip: String,
}

impl LiveExecutorProvenance {
    /// The adapter name.
    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// The adapter version.
    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    /// The node name.
    #[must_use]
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    /// The node version.
    #[must_use]
    pub fn node_version(&self) -> &str {
        &self.node_version
    }

    /// The executed source identity.
    #[must_use]
    pub fn executed_source_tip(&self) -> &str {
        &self.executed_source_tip
    }
}

/// One exact target response retained by a validated run binding.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveTargetResponse {
    /// The target accepted the request and assigned this identity.
    Accepted {
        /// The typed target identity.
        identity: Txid,
    },
    /// The target refused the request.
    Refused {
        /// The observed refusal layer.
        observed_layer: ObservedOutcomeLayer,
        /// The target's exact detail.
        detail: String,
        /// The accepted control identity the refusal is attributable against.
        control_identity: Txid,
    },
}

/// One transcript-grade target run in the cumulative corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRunBinding {
    run_id: String,
    deployment: LiveDeploymentBinding,
    executor: LiveExecutorProvenance,
    requests: BTreeMap<String, Vec<u8>>,
    responses: BTreeMap<String, LiveTargetResponse>,
}

impl LiveRunBinding {
    /// The deterministic corpus identity of this run.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// The exact deployment binding.
    #[must_use]
    pub const fn deployment(&self) -> &LiveDeploymentBinding {
        &self.deployment
    }

    /// The validated executor provenance.
    #[must_use]
    pub const fn executor(&self) -> &LiveExecutorProvenance {
        &self.executor
    }

    /// Exact sent request bytes, keyed by deterministic request identity.
    #[must_use]
    pub const fn requests(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.requests
    }

    /// Exact responses, keyed by the same request identity.
    #[must_use]
    pub const fn responses(&self) -> &BTreeMap<String, LiveTargetResponse> {
        &self.responses
    }
}

/// A typed historical observation whose run binding is absent.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveRecordedObservation {
    /// A recorded acceptance.
    NativeAcceptance {
        /// The target-computed identity.
        identity: Txid,
    },
    /// A recorded refusal and accepted control.
    NativeRefusal {
        /// The row's declared boundary.
        declared_boundary: EvidenceBoundary,
        /// The recorded observed layer.
        observed_layer: ObservedOutcomeLayer,
        /// The accepted control identity.
        control_identity: Txid,
        /// The target's exact refusal detail.
        detail: &'static str,
    },
    /// A recorded relation over two accepted identities.
    PairedRelation {
        /// The explicit identity.
        explicit_identity: Txid,
        /// The private identity.
        private_identity: Txid,
        /// The recorded relation.
        relation: &'static str,
    },
}

/// One row-level observation in the corpus ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveReportObservation {
    /// A bound target acceptance.
    NativeAcceptance {
        /// The matrix row.
        row: &'static str,
        /// The bound run.
        run_id: String,
        /// The exact request within the run.
        request_id: String,
        /// The target-computed identity.
        identity: Txid,
    },
    /// A bound target refusal.
    NativeRefusal {
        /// The matrix row.
        row: &'static str,
        /// The bound run.
        run_id: String,
        /// The exact request within the run.
        request_id: String,
        /// The row's declared boundary.
        declared_boundary: EvidenceBoundary,
        /// The observed layer.
        observed_layer: ObservedOutcomeLayer,
        /// The accepted control identity.
        control_identity: Txid,
        /// The target's exact refusal detail.
        detail: String,
    },
    /// A bound relation over two accepted members.
    PairedRelation {
        /// The matrix row.
        row: &'static str,
        /// The explicit member's run.
        explicit_run_id: String,
        /// The explicit request within its run.
        explicit_request_id: String,
        /// The explicit identity.
        explicit_identity: Txid,
        /// The private member's run.
        private_run_id: String,
        /// The private request within its run.
        private_request_id: String,
        /// The private identity.
        private_identity: Txid,
        /// The validated relation.
        relation: String,
    },
    /// A preserved historical observation lacking a complete run binding.
    RecordedObservationUnbound {
        /// The matrix row.
        row: &'static str,
        /// The typed recorded fact.
        observation: LiveRecordedObservation,
    },
    /// A first-party determinism observation.
    Determinism {
        /// The matrix row.
        row: &'static str,
        /// What was recomputed.
        recomputed: &'static str,
        /// The site that recomputed it.
        observed_by: &'static str,
    },
    /// A first-party fact.
    FirstPartyFact {
        /// The matrix row.
        row: &'static str,
        /// The fact.
        fact: &'static str,
        /// The site establishing it.
        observed_by: &'static str,
    },
    /// A row answered by validation of the report layer itself.
    ReportLayer {
        /// The matrix row.
        row: &'static str,
    },
}

/// The lifecycle obligations §13.2 makes the report carry (§17.1).
///
/// Transfer is implemented; burn and redemption are not; the candidate is
/// therefore lifecycle-incomplete. Recorded in the report because §7.4
/// requires the status to travel with the candidate rather than being
/// looked up.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveLifecycleStatus {
    implemented: BTreeSet<&'static str>,
    outstanding: BTreeSet<&'static str>,
    release_complete: bool,
}

impl LiveLifecycleStatus {
    /// The Phase-5 status, which is the only one this candidate has.
    #[must_use]
    pub fn candidate() -> Self {
        Self {
            implemented: BTreeSet::from(["transfer-live-receipts"]),
            outstanding: BTreeSet::from(["burn", "redeem"]),
            release_complete: false,
        }
    }

    /// The exits this candidate implements.
    #[must_use]
    pub const fn implemented(&self) -> &BTreeSet<&'static str> {
        &self.implemented
    }

    /// The exits it does not.
    #[must_use]
    pub const fn outstanding(&self) -> &BTreeSet<&'static str> {
        &self.outstanding
    }

    /// Whether the live-receipt lifecycle is complete, which it is not.
    #[must_use]
    pub const fn release_complete(&self) -> bool {
        self.release_complete
    }
}

/// One volatile field §13.5 keeps out of the canonical bytes.
///
/// Enumerated so the exclusion is a census a test can walk rather than a
/// list in a comment. Every one of them lives on
/// [`LiveSafetyDiagnostics`], which the canonical renderer cannot reach.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VolatileField {
    /// The wall-clock time the run started.
    WallClockTime,
    /// How long the run took.
    ElapsedTime,
    /// The host the run happened on.
    Hostname,
    /// The account it ran as.
    Username,
    /// The process identifier.
    ProcessId,
    /// A temporary directory path.
    TemporaryPath,
    /// The executor program's path.
    ExecutorPath,
    /// Raw text a child process wrote.
    RawChildStandardError,
    /// An environment variable's value.
    EnvironmentValue,
    /// Test-only private fixture material the schema does not require.
    TestOnlyFixtureMaterial,
}

impl VolatileField {
    /// All ten, in §13.5's order.
    pub const ALL: &'static [Self] = &[
        Self::WallClockTime,
        Self::ElapsedTime,
        Self::Hostname,
        Self::Username,
        Self::ProcessId,
        Self::TemporaryPath,
        Self::ExecutorPath,
        Self::RawChildStandardError,
        Self::EnvironmentValue,
        Self::TestOnlyFixtureMaterial,
    ];
}

/// The noncanonical diagnostic report of §13.6.
///
/// Everything §13.5 excludes from the canonical bytes, in one value that
/// the canonical renderer takes no argument of. Duration is a property of
/// the machine a run happened on rather than of the run's result, and a
/// canonical stream carrying one differs from every other by
/// construction — so two runs of the same matrix could never be compared
/// byte for byte.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveSafetyDiagnostics {
    fields: BTreeMap<VolatileField, String>,
}

impl LiveSafetyDiagnostics {
    /// A diagnostic report with one field recorded.
    #[must_use]
    pub fn with(mut self, field: VolatileField, value: impl Into<String>) -> Self {
        self.fields.insert(field, value.into());
        self
    }

    /// One recorded field.
    #[must_use]
    pub fn field(&self, field: VolatileField) -> Option<&str> {
        self.fields.get(&field).map(String::as_str)
    }

    /// Every recorded field.
    #[must_use]
    pub const fn fields(&self) -> &BTreeMap<VolatileField, String> {
        &self.fields
    }

    /// The diagnostic report's own rendering.
    ///
    /// Deliberately a separate function from the canonical one, and
    /// deliberately not called by it.
    ///
    /// # Panics
    ///
    /// Never: the sink is a `String`, whose writes cannot fail.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "role live-transfer-safety-diagnostics");
        let _ = writeln!(text, "canonical false");
        for (field, value) in &self.fields {
            let _ = writeln!(text, "{field:?} {value}");
        }
        text
    }
}

/// The live-transfer safety report of §13.2.
///
/// A value a caller can build, and one that establishes nothing on its
/// own. The cumulative corpus carriers are here; the conclusions are
/// [`validate_live_safety_report`]'s.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveTransferSafetyReport {
    schema: u32,
    role: LiveSafetyReportRole,
    target: TargetProjection,
    representations: BTreeSet<LiveTransferRepresentationPlan>,
    runs: Vec<LiveRunBinding>,
    observations: Vec<LiveReportObservation>,
    lifecycle: LiveLifecycleStatus,
    census: LiveEvidenceCensus,
    completeness: LiveSafetyCompleteness,
}

impl LiveTransferSafetyReport {
    /// The report's schema.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }

    /// What kind of report this is.
    #[must_use]
    pub const fn role(&self) -> LiveSafetyReportRole {
        self.role
    }

    /// The reviewed target the matrix is stated against.
    #[must_use]
    pub const fn target(&self) -> &TargetProjection {
        &self.target
    }

    /// The transcript-grade run bindings in the cumulative corpus.
    #[must_use]
    pub fn runs(&self) -> &[LiveRunBinding] {
        &self.runs
    }

    /// The representation plans in scope.
    #[must_use]
    pub const fn representations(&self) -> &BTreeSet<LiveTransferRepresentationPlan> {
        &self.representations
    }

    /// Every row-level observation in matrix order.
    #[must_use]
    pub fn observations(&self) -> &[LiveReportObservation] {
        &self.observations
    }

    /// The lifecycle obligations.
    #[must_use]
    pub const fn lifecycle(&self) -> &LiveLifecycleStatus {
        &self.lifecycle
    }

    /// The row census.
    #[must_use]
    pub const fn census(&self) -> LiveEvidenceCensus {
        self.census
    }

    /// The completeness token.
    #[must_use]
    pub const fn completeness(&self) -> LiveSafetyCompleteness {
        self.completeness
    }
}

/// Why one report could not be validated.
///
/// Every variant is a disagreement between the report and what this
/// module recomputed from the evidence plan. None of them is a target
/// verdict: a report whose every row is blocked validates perfectly and
/// says so.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveSafetyReportRefusal {
    /// The report states a schema this validator does not read.
    UnsupportedSchema(u32),
    /// The report states a role that is not the safety one.
    ///
    /// §13.6's separation, enforced: a minimality report handed to the
    /// safety gate is refused rather than read as a safety one.
    WrongRole(LiveSafetyReportRole),
    /// The report's target is not the one the plan is stated against.
    TargetDiffers,
    /// The report's representation census is not the plan's.
    RepresentationCensusDiffers,
    /// The report's row census is not the one recomputed from the plan.
    ///
    /// Both censuses are boxed, as [`crate::live_resource_report`] boxes
    /// its own pair and for the same reason: each is eight counts wide
    /// and a refusal carrying two of them inline would make every
    /// `Result` in this module pay for the one arm the happy path never
    /// takes. The first member is what the report said and the second is
    /// what the plan recomputes to.
    CensusDiffers(Box<(LiveEvidenceCensus, LiveEvidenceCensus)>),
    /// The report claims a completeness its own census does not support.
    CompletenessDiffers {
        /// What the report said.
        reported: LiveSafetyCompleteness,
        /// What the census supports.
        recomputed: LiveSafetyCompleteness,
    },
    /// A recorded target identity is not valid target-display txid text.
    MalformedRecordedIdentity {
        /// The matrix row carrying it.
        row: &'static str,
    },
    /// The report and the derived corpus carry different run identities.
    RunCensusDiffers,
    /// One run's deployment differs from the validated ledger.
    RunDeploymentDiffers(String),
    /// One run's exact request census or bytes differ.
    RunRequestDiffers(String),
    /// One run's exact response census or facts differ.
    RunResponseDiffers(String),
    /// One run's executor provenance differs.
    RunExecutorProvenanceDiffers(String),
    /// The report's row-level observations differ from the derived ledger.
    ObservationsDiffer,
    /// The report's observation buckets do not cross-foot its census.
    ObservationCensusDiffers,
    /// A bound observation cites no matching run or request.
    BoundObservationUnbacked {
        /// The matrix row carrying the invalid reference.
        row: &'static str,
    },
    /// The report's lifecycle status is not the candidate's.
    LifecycleDiffers,
}

/// One run's report, validated: the only thing a safety gate reads.
///
/// # What holding one of these establishes
///
/// That the corpus carriers agree with what this module recomputed from
/// the evidence plan. Its recomputation witness names only comparisons
/// that actually ran; with no transcript-grade target run in the tree it
/// establishes the preserved observations as unbound, not as target
/// verdict evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLiveTransferSafetyReport {
    report: LiveTransferSafetyReport,
    recomputed_items: BTreeSet<RecomputedItem>,
    blockers: BTreeMap<LiveInfrastructureBlocker, usize>,
    outstanding: Vec<(&'static str, LiveRowStanding)>,
}

/// One of §13.5's fourteen recomputed items.
///
/// A census rather than prose, so a validation that stopped performing
/// one has to remove it here and fail the count.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RecomputedItem {
    /// The report's role.
    Role,
    /// The report's schema.
    Schema,
    /// The target and deployment binding.
    TargetDeploymentBinding,
    /// The exact request and response census.
    RequestResponseCensus,
    /// The response shape.
    ResponseShape,
    /// The case census.
    CaseCensus,
    /// The relation census.
    RelationCensus,
    /// The mutation links.
    MutationLinks,
    /// The target verdict comparison.
    TargetVerdictComparison,
    /// The semantic projection.
    SemanticProjection,
    /// The disclosure comparison.
    DisclosureComparison,
    /// The resource comparison.
    ResourceComparison,
    /// The executor provenance.
    ExecutorProvenance,
    /// The summary.
    Summary,
}

impl RecomputedItem {
    /// All fourteen, in §13.5's order.
    pub const ALL: &'static [Self] = &[
        Self::Role,
        Self::Schema,
        Self::TargetDeploymentBinding,
        Self::RequestResponseCensus,
        Self::ResponseShape,
        Self::CaseCensus,
        Self::RelationCensus,
        Self::MutationLinks,
        Self::TargetVerdictComparison,
        Self::SemanticProjection,
        Self::DisclosureComparison,
        Self::ResourceComparison,
        Self::ExecutorProvenance,
        Self::Summary,
    ];

    /// Whether this item can only be recomputed for a report about a run.
    ///
    /// Six of the fourteen are about things an executor produced. They do
    /// not enter the validated witness until at least one bound run made
    /// the corresponding comparison possible.
    #[must_use]
    pub const fn needs_a_run(self) -> bool {
        matches!(
            self,
            Self::RequestResponseCensus
                | Self::ResponseShape
                | Self::TargetVerdictComparison
                | Self::SemanticProjection
                | Self::ResourceComparison
                | Self::ExecutorProvenance
        )
    }
}

impl ValidatedLiveTransferSafetyReport {
    /// The report every carrier of which has been recomputed.
    #[must_use]
    pub const fn report(&self) -> &LiveTransferSafetyReport {
        &self.report
    }

    /// The §13.5 items this validation recomputed.
    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<RecomputedItem> {
        &self.recomputed_items
    }

    /// Every blocker, with how many rows carry it.
    #[must_use]
    pub const fn blockers(&self) -> &BTreeMap<LiveInfrastructureBlocker, usize> {
        &self.blockers
    }

    /// Every required row that is not answered, in §15 order.
    #[must_use]
    pub fn outstanding(&self) -> &[(&'static str, LiveRowStanding)] {
        &self.outstanding
    }
}

/// The completeness one census supports.
///
/// # The mismatch count is asked FIRST, and that ordering is the rule
///
/// A wrong-boundary refusal and a run that has not happened are different
/// conditions, and the enum has carried a distinct token for the first
/// since it was written — `Failed` says a required row was answered at a
/// boundary other than its own. Nothing could reach it, because nothing
/// compared the two boundaries. Now that something does, the mismatch
/// count is tested BEFORE the outstanding count: a matrix with both a
/// contradiction and an ordinary outstanding row is `Failed`, not
/// partial, because a report that renders a contradiction as "still
/// waiting" understates what it found.
const fn completeness_of(census: LiveEvidenceCensus) -> LiveSafetyCompleteness {
    if census.native_refusal_at_unexpected_boundary() > 0 {
        LiveSafetyCompleteness::Failed
    } else if census.every_required_row_is_answered() {
        LiveSafetyCompleteness::CompleteForTheRequiredMatrix
    } else {
        LiveSafetyCompleteness::PartialRequiredRowsOutstanding
    }
}

/// Assemble the safety report one evidence plan determines (§13.2).
///
/// Every carrier is derived from the plan, which is why this returns a
/// report rather than taking one: a constructor accepting the census
/// would let a caller state a conclusion the rows do not support, and
/// [`validate_live_safety_report`] would then be checking the caller
/// against themselves.
///
/// # Errors
///
/// [`LiveSafetyReportRefusal::MalformedRecordedIdentity`] if a historical
/// identity cannot be parsed through the transaction package's target-display
/// API, or [`LiveSafetyReportRefusal::BoundObservationUnbacked`] if a future
/// bound standing arrives without its validated run binding.
pub fn assemble_live_safety_report(
    plan: &LiveTransferEvidencePlan,
    target: TargetProjection,
) -> Result<LiveTransferSafetyReport, LiveSafetyReportRefusal> {
    let census = plan.census();
    Ok(LiveTransferSafetyReport {
        schema: LIVE_SAFETY_REPORT_SCHEMA,
        role: LiveSafetyReportRole::LiveTransferSafety,
        target,
        representations: BTreeSet::from([
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ]),
        // No historical target run in the tree retains all four binding
        // carriers. The observation ledger below preserves what was recorded
        // without manufacturing deployment or executor provenance.
        runs: Vec::new(),
        observations: observations_from_plan(plan)?,
        lifecycle: LiveLifecycleStatus::candidate(),
        census,
        completeness: completeness_of(census),
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ObservationCensus {
    accepted: usize,
    refused: usize,
    determinism: usize,
    paired_relation: usize,
    first_party_fact: usize,
    report_layer: usize,
    recorded_unbound: usize,
    unbound_acceptance: usize,
    unbound_refusal: usize,
    unbound_paired_relation: usize,
}

impl ObservationCensus {
    fn from_observations(observations: &[LiveReportObservation]) -> Self {
        let mut census = Self::default();
        for observation in observations {
            match observation {
                LiveReportObservation::NativeAcceptance { .. } => census.accepted += 1,
                LiveReportObservation::NativeRefusal { .. } => census.refused += 1,
                LiveReportObservation::PairedRelation { .. } => census.paired_relation += 1,
                LiveReportObservation::RecordedObservationUnbound { observation, .. } => {
                    census.recorded_unbound += 1;
                    match observation {
                        LiveRecordedObservation::NativeAcceptance { .. } => {
                            census.unbound_acceptance += 1;
                        }
                        LiveRecordedObservation::NativeRefusal { .. } => {
                            census.unbound_refusal += 1;
                        }
                        LiveRecordedObservation::PairedRelation { .. } => {
                            census.unbound_paired_relation += 1;
                        }
                    }
                }
                LiveReportObservation::Determinism { .. } => census.determinism += 1,
                LiveReportObservation::FirstPartyFact { .. } => census.first_party_fact += 1,
                LiveReportObservation::ReportLayer { .. } => census.report_layer += 1,
            }
        }
        census
    }

    const fn cross_foots(self, census: LiveEvidenceCensus) -> bool {
        self.accepted == census.native_run_observed()
            && self.refused == census.native_refusal_observed()
            && self.determinism == census.determinism_observed()
            && self.paired_relation == census.paired_relation_observed()
            && self.first_party_fact == census.first_party_fact_observed()
            && self.report_layer == census.report_layer()
            && self.recorded_unbound
                == census.recorded_observation_unbound()
                    + census.native_refusal_at_unexpected_boundary()
    }

    const fn total(self) -> usize {
        self.accepted
            + self.refused
            + self.determinism
            + self.paired_relation
            + self.first_party_fact
            + self.report_layer
            + self.recorded_unbound
    }
}

fn parse_recorded_identity(
    row: &'static str,
    identity: &str,
) -> Result<Txid, LiveSafetyReportRefusal> {
    Txid::from_target_display(identity)
        .map_err(|_| LiveSafetyReportRefusal::MalformedRecordedIdentity { row })
}

fn observations_from_plan(
    plan: &LiveTransferEvidencePlan,
) -> Result<Vec<LiveReportObservation>, LiveSafetyReportRefusal> {
    let mut observations = Vec::new();
    for evidence in plan.rows() {
        let row = evidence.row().name();
        let observation = match evidence.standing() {
            LiveRowStanding::RecordedObservationUnbound(recorded) => {
                let observation = match recorded {
                    RecordedObservation::NativeAcceptance { accepted_identity } => {
                        LiveRecordedObservation::NativeAcceptance {
                            identity: parse_recorded_identity(row, accepted_identity)?,
                        }
                    }
                    RecordedObservation::NativeRefusal {
                        declared_boundary,
                        observed_layer,
                        control_identity,
                        refusal_detail,
                    } => LiveRecordedObservation::NativeRefusal {
                        declared_boundary: *declared_boundary,
                        observed_layer: *observed_layer,
                        control_identity: parse_recorded_identity(row, control_identity)?,
                        detail: refusal_detail,
                    },
                    RecordedObservation::PairedRelation {
                        explicit_identity,
                        private_identity,
                        relation,
                    } => LiveRecordedObservation::PairedRelation {
                        explicit_identity: parse_recorded_identity(row, explicit_identity)?,
                        private_identity: parse_recorded_identity(row, private_identity)?,
                        relation,
                    },
                };
                Some(LiveReportObservation::RecordedObservationUnbound { row, observation })
            }
            LiveRowStanding::DeterminismObserved {
                recomputed,
                observed_by,
            } => Some(LiveReportObservation::Determinism {
                row,
                recomputed,
                observed_by,
            }),
            LiveRowStanding::FirstPartyFactObserved { fact, observed_by } => {
                Some(LiveReportObservation::FirstPartyFact {
                    row,
                    fact,
                    observed_by,
                })
            }
            LiveRowStanding::ReportLayerAnswerable => {
                Some(LiveReportObservation::ReportLayer { row })
            }
            // A future bound standing must arrive with a validated run binding;
            // silently inventing one from its row name would restore this defect.
            LiveRowStanding::NativeRunObserved { .. }
            | LiveRowStanding::NativeRefusalObserved { .. }
            | LiveRowStanding::PairedRelationObserved { .. } => {
                return Err(LiveSafetyReportRefusal::BoundObservationUnbacked { row });
            }
            LiveRowStanding::NativeRefusalAtUnexpectedBoundary {
                declared_boundary,
                observed_layer,
                control_identity,
                refusal_detail,
            } => Some(LiveReportObservation::RecordedObservationUnbound {
                row,
                observation: LiveRecordedObservation::NativeRefusal {
                    declared_boundary: *declared_boundary,
                    observed_layer: *observed_layer,
                    control_identity: parse_recorded_identity(row, control_identity)?,
                    detail: refusal_detail,
                },
            }),
            LiveRowStanding::FirstPartyDischarged { .. }
            | LiveRowStanding::FirstPartyUndischarged(_)
            | LiveRowStanding::NativeRunRequired(_)
            | LiveRowStanding::InfrastructureBlocked(_)
            | LiveRowStanding::OperationVocabularyClosed
            | LiveRowStanding::Experimental => None,
        };
        if let Some(observation) = observation {
            observations.push(observation);
        }
    }
    Ok(observations)
}

#[derive(Debug, Default)]
struct RecomputationProgress {
    completed: BTreeSet<RecomputedItem>,
}

impl RecomputationProgress {
    fn mark(&mut self, item: RecomputedItem) {
        self.completed.insert(item);
    }

    fn finish(self) -> BTreeSet<RecomputedItem> {
        self.completed
    }
}

fn compare_run_bindings(
    offered: &[LiveRunBinding],
    expected: &[LiveRunBinding],
) -> Result<(), LiveSafetyReportRefusal> {
    if offered.len() != expected.len() {
        return Err(LiveSafetyReportRefusal::RunCensusDiffers);
    }
    if offered
        .windows(2)
        .any(|pair| pair[0].run_id >= pair[1].run_id)
        || expected
            .windows(2)
            .any(|pair| pair[0].run_id >= pair[1].run_id)
    {
        return Err(LiveSafetyReportRefusal::RunCensusDiffers);
    }
    for (offered, expected) in offered.iter().zip(expected) {
        if offered.run_id != expected.run_id {
            return Err(LiveSafetyReportRefusal::RunCensusDiffers);
        }
        if offered.deployment != expected.deployment {
            return Err(LiveSafetyReportRefusal::RunDeploymentDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.requests != expected.requests {
            return Err(LiveSafetyReportRefusal::RunRequestDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.responses != expected.responses {
            return Err(LiveSafetyReportRefusal::RunResponseDiffers(
                offered.run_id.clone(),
            ));
        }
        if offered.executor != expected.executor {
            return Err(LiveSafetyReportRefusal::RunExecutorProvenanceDiffers(
                offered.run_id.clone(),
            ));
        }
    }
    Ok(())
}

fn response_for<'run>(
    runs: &'run [LiveRunBinding],
    run_id: &str,
    request_id: &str,
    row: &'static str,
) -> Result<&'run LiveTargetResponse, LiveSafetyReportRefusal> {
    let run = runs
        .iter()
        .find(|run| run.run_id == run_id)
        .ok_or(LiveSafetyReportRefusal::BoundObservationUnbacked { row })?;
    if !run.requests.contains_key(request_id) {
        return Err(LiveSafetyReportRefusal::BoundObservationUnbacked { row });
    }
    run.responses
        .get(request_id)
        .ok_or(LiveSafetyReportRefusal::BoundObservationUnbacked { row })
}

fn validate_bound_observations(
    observations: &[LiveReportObservation],
    runs: &[LiveRunBinding],
) -> Result<usize, LiveSafetyReportRefusal> {
    let mut compared = 0_usize;
    for observation in observations {
        match observation {
            LiveReportObservation::NativeAcceptance {
                row,
                run_id,
                request_id,
                identity,
            } => {
                let expected = LiveTargetResponse::Accepted {
                    identity: *identity,
                };
                if response_for(runs, run_id, request_id, row)? != &expected {
                    return Err(LiveSafetyReportRefusal::RunResponseDiffers(run_id.clone()));
                }
                compared += 1;
            }
            LiveReportObservation::NativeRefusal {
                row,
                run_id,
                request_id,
                observed_layer,
                control_identity,
                detail,
                ..
            } => {
                let expected = LiveTargetResponse::Refused {
                    observed_layer: *observed_layer,
                    detail: detail.clone(),
                    control_identity: *control_identity,
                };
                if response_for(runs, run_id, request_id, row)? != &expected {
                    return Err(LiveSafetyReportRefusal::RunResponseDiffers(run_id.clone()));
                }
                compared += 1;
            }
            LiveReportObservation::PairedRelation {
                row,
                explicit_run_id,
                explicit_request_id,
                explicit_identity,
                private_run_id,
                private_request_id,
                private_identity,
                ..
            } => {
                let explicit_expected = LiveTargetResponse::Accepted {
                    identity: *explicit_identity,
                };
                if response_for(runs, explicit_run_id, explicit_request_id, row)?
                    != &explicit_expected
                {
                    return Err(LiveSafetyReportRefusal::RunResponseDiffers(
                        explicit_run_id.clone(),
                    ));
                }
                let private_expected = LiveTargetResponse::Accepted {
                    identity: *private_identity,
                };
                if response_for(runs, private_run_id, private_request_id, row)? != &private_expected
                {
                    return Err(LiveSafetyReportRefusal::RunResponseDiffers(
                        private_run_id.clone(),
                    ));
                }
                compared += 1;
            }
            LiveReportObservation::RecordedObservationUnbound { .. }
            | LiveReportObservation::Determinism { .. }
            | LiveReportObservation::FirstPartyFact { .. }
            | LiveReportObservation::ReportLayer { .. } => {}
        }
    }
    Ok(compared)
}

/// Validate one safety report against the plan it claims to be about.
///
/// §13.5's possible items, each recorded only after its comparison ran.
/// Run-only items stay absent while the corpus has no transcript-grade
/// run binding.
///
/// # Errors
///
/// [`LiveSafetyReportRefusal`], naming the first disagreement found in
/// the order the checks are written.
pub fn validate_live_safety_report(
    report: LiveTransferSafetyReport,
    plan: &LiveTransferEvidencePlan,
    target: &TargetProjection,
) -> Result<ValidatedLiveTransferSafetyReport, LiveSafetyReportRefusal> {
    let mut progress = RecomputationProgress::default();
    if report.schema != LIVE_SAFETY_REPORT_SCHEMA {
        return Err(LiveSafetyReportRefusal::UnsupportedSchema(report.schema));
    }
    progress.mark(RecomputedItem::Schema);
    if report.role != LiveSafetyReportRole::LiveTransferSafety {
        return Err(LiveSafetyReportRefusal::WrongRole(report.role));
    }
    progress.mark(RecomputedItem::Role);
    if &report.target != target {
        return Err(LiveSafetyReportRefusal::TargetDiffers);
    }

    let representations: BTreeSet<_> = plan
        .operation_plan()
        .representations()
        .map(compiler::live_transfer_plan::LiveTransferRepresentationProjection::plan)
        .collect();
    if report.representations != representations {
        return Err(LiveSafetyReportRefusal::RepresentationCensusDiffers);
    }

    let expected_runs = Vec::new();
    compare_run_bindings(&report.runs, &expected_runs)?;
    if !report.runs.is_empty() {
        progress.mark(RecomputedItem::TargetDeploymentBinding);
        progress.mark(RecomputedItem::RequestResponseCensus);
        progress.mark(RecomputedItem::ExecutorProvenance);
    }

    let expected_observations = observations_from_plan(plan)?;
    if report.observations != expected_observations {
        return Err(LiveSafetyReportRefusal::ObservationsDiffer);
    }
    progress.mark(RecomputedItem::CaseCensus);
    progress.mark(RecomputedItem::RelationCensus);
    progress.mark(RecomputedItem::MutationLinks);

    let observed = ObservationCensus::from_observations(&report.observations);
    if !observed.cross_foots(plan.census()) {
        return Err(LiveSafetyReportRefusal::ObservationCensusDiffers);
    }

    let bound_compared = validate_bound_observations(&report.observations, &report.runs)?;
    if bound_compared > 0 {
        progress.mark(RecomputedItem::ResponseShape);
        progress.mark(RecomputedItem::TargetVerdictComparison);
    }

    let recomputed = plan.census();
    if report.census != recomputed {
        return Err(LiveSafetyReportRefusal::CensusDiffers(Box::new((
            report.census,
            recomputed,
        ))));
    }

    let completeness = completeness_of(recomputed);
    if report.completeness != completeness {
        return Err(LiveSafetyReportRefusal::CompletenessDiffers {
            reported: report.completeness,
            recomputed: completeness,
        });
    }

    if report.lifecycle != LiveLifecycleStatus::candidate() {
        return Err(LiveSafetyReportRefusal::LifecycleDiffers);
    }
    progress.mark(RecomputedItem::Summary);

    let outstanding = plan
        .rows()
        .iter()
        .filter(|row| {
            !row.standing().is_answered()
                && !matches!(
                    row.standing(),
                    LiveRowStanding::OperationVocabularyClosed | LiveRowStanding::Experimental
                )
        })
        .map(|row| (row.row().name(), row.standing().clone()))
        .collect();

    Ok(ValidatedLiveTransferSafetyReport {
        report,
        recomputed_items: progress.finish(),
        blockers: blocker_census(plan),
        outstanding,
    })
}

/// Render the canonical bytes of one validated safety report.
///
/// Takes a validated report and nothing else. The ten volatile fields are
/// not filtered here — they are not reachable from this function's one
/// argument, which is the difference between a rule and a habit.
///
/// # Panics
///
/// Never: the sink is a `String`, whose writes cannot fail.
#[must_use]
pub fn render_live_safety_report(validated: &ValidatedLiveTransferSafetyReport) -> String {
    let report = &validated.report;
    let mut text = String::new();
    let _ = writeln!(text, "schema {}", report.schema);
    let _ = writeln!(text, "role {}", report.role.name());
    let _ = writeln!(text, "operation transfer-live-receipts");
    let _ = writeln!(text, "candidate true");
    let _ = writeln!(text, "target_contract {:?}", report.target.version());
    let _ = writeln!(
        text,
        "execution_domain {:?}",
        report.target.execution_domain()
    );
    for representation in &report.representations {
        let _ = writeln!(text, "representation {representation:?}");
    }
    let observation_census = ObservationCensus::from_observations(&report.observations);
    let _ = writeln!(
        text,
        "target_evidence {}",
        target_evidence_name(observation_census)
    );
    let _ = writeln!(text, "run_bindings {}", report.runs.len());
    render_run_bindings(&mut text, &report.runs);

    let _ = writeln!(text, "rows {}", report.census.rows());
    let _ = writeln!(
        text,
        "first_party_discharged {}",
        report.census.first_party_discharged()
    );
    let _ = writeln!(
        text,
        "first_party_undischarged {}",
        report.census.first_party_undischarged()
    );
    let _ = writeln!(
        text,
        "native_run_required {}",
        report.census.native_run_required()
    );
    let _ = writeln!(
        text,
        "recorded_observation_unbound {}",
        report.census.recorded_observation_unbound()
    );
    let _ = writeln!(
        text,
        "native_run_observed {}",
        report.census.native_run_observed()
    );
    let _ = writeln!(
        text,
        "native_refusal_observed {}",
        report.census.native_refusal_observed()
    );
    let _ = writeln!(
        text,
        "native_refusal_at_unexpected_boundary {}",
        report.census.native_refusal_at_unexpected_boundary()
    );
    let _ = writeln!(
        text,
        "determinism_observed {}",
        report.census.determinism_observed()
    );
    let _ = writeln!(
        text,
        "paired_relation_observed {}",
        report.census.paired_relation_observed()
    );
    let _ = writeln!(
        text,
        "first_party_fact_observed {}",
        report.census.first_party_fact_observed()
    );
    let _ = writeln!(
        text,
        "infrastructure_blocked {}",
        report.census.infrastructure_blocked()
    );
    let _ = writeln!(text, "report_layer {}", report.census.report_layer());
    let _ = writeln!(
        text,
        "operation_vocabulary_closed {}",
        report.census.vocabulary_closed()
    );
    let _ = writeln!(text, "experimental {}", report.census.experimental());

    let _ = writeln!(text, "observations {}", observation_census.total());
    let _ = writeln!(text, "accepted {}", observation_census.accepted);
    let _ = writeln!(text, "refused {}", observation_census.refused);
    let _ = writeln!(text, "determinism {}", observation_census.determinism);
    let _ = writeln!(
        text,
        "paired_relation {}",
        observation_census.paired_relation
    );
    let _ = writeln!(
        text,
        "first_party_fact {}",
        observation_census.first_party_fact
    );
    let _ = writeln!(
        text,
        "report_layer_observations {}",
        observation_census.report_layer
    );
    let _ = writeln!(
        text,
        "recorded_unbound_native_acceptance {}",
        observation_census.unbound_acceptance
    );
    let _ = writeln!(
        text,
        "recorded_unbound_native_refusal {}",
        observation_census.unbound_refusal
    );
    let _ = writeln!(
        text,
        "recorded_unbound_paired_relation {}",
        observation_census.unbound_paired_relation
    );
    render_observations(&mut text, &report.observations);

    for (blocker, rows) in &validated.blockers {
        let _ = writeln!(text, "blocker {blocker:?} {rows}");
    }
    for (row, standing) in &validated.outstanding {
        let _ = writeln!(text, "outstanding {row} {}", standing_name(standing));
    }
    for item in &validated.recomputed_items {
        let _ = writeln!(text, "recomputed {item:?}");
    }

    for exit in report.lifecycle.implemented() {
        let _ = writeln!(text, "lifecycle_implemented {exit}");
    }
    for exit in report.lifecycle.outstanding() {
        let _ = writeln!(text, "lifecycle_outstanding {exit}");
    }
    let _ = writeln!(
        text,
        "release_complete {}",
        report.lifecycle.release_complete()
    );
    let _ = writeln!(text, "completeness {}", report.completeness.name());
    text
}

const fn target_evidence_name(census: ObservationCensus) -> &'static str {
    let bound = census.accepted + census.refused + census.paired_relation;
    let unbound = census.recorded_unbound;
    if bound == 0 && unbound == 0 {
        "none-recorded"
    } else if bound == 0 {
        "recorded-unbound"
    } else if unbound == 0 {
        "bound"
    } else {
        "mixed-bound-and-unbound"
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(rendered, "{byte:02x}");
    }
    rendered
}

fn render_run_bindings(text: &mut String, runs: &[LiveRunBinding]) {
    for run in runs {
        let _ = writeln!(text, "run {}", run.run_id);
        let _ = writeln!(
            text,
            "run_deployment_environment {:?}",
            run.deployment.environment
        );
        let _ = writeln!(
            text,
            "run_deployment_network {}",
            hex_bytes(&run.deployment.network_id)
        );
        let _ = writeln!(
            text,
            "run_deployment_genesis {}",
            hex_bytes(&run.deployment.genesis_id)
        );
        let _ = writeln!(
            text,
            "run_deployment_target {:?}",
            run.deployment.target_contract
        );
        let _ = writeln!(text, "run_executor_adapter {:?}", run.executor.adapter_name);
        let _ = writeln!(
            text,
            "run_executor_adapter_version {:?}",
            run.executor.adapter_version
        );
        let _ = writeln!(text, "run_executor_node {:?}", run.executor.node_name);
        let _ = writeln!(
            text,
            "run_executor_node_version {:?}",
            run.executor.node_version
        );
        let _ = writeln!(
            text,
            "run_executor_source_tip {:?}",
            run.executor.executed_source_tip
        );
        for (request_id, bytes) in &run.requests {
            let _ = writeln!(text, "run_request {request_id} {}", hex_bytes(bytes));
        }
        for (request_id, response) in &run.responses {
            match response {
                LiveTargetResponse::Accepted { identity } => {
                    let _ = writeln!(
                        text,
                        "run_response {request_id} accepted {}",
                        identity.to_target_display()
                    );
                }
                LiveTargetResponse::Refused {
                    observed_layer,
                    detail,
                    control_identity,
                } => {
                    let _ = writeln!(
                        text,
                        "run_response {request_id} refused {observed_layer:?} control {} detail {detail:?}",
                        control_identity.to_target_display()
                    );
                }
            }
        }
    }
}

fn render_observations(text: &mut String, observations: &[LiveReportObservation]) {
    for observation in observations {
        match observation {
            LiveReportObservation::NativeAcceptance {
                row,
                run_id,
                request_id,
                identity,
            } => {
                let _ = writeln!(
                    text,
                    "observation {row} accepted run {run_id} request {request_id} identity {}",
                    identity.to_target_display()
                );
            }
            LiveReportObservation::NativeRefusal {
                row,
                run_id,
                request_id,
                declared_boundary,
                observed_layer,
                control_identity,
                detail,
            } => {
                let _ = writeln!(
                    text,
                    "observation {row} refused run {run_id} request {request_id} declared {declared_boundary:?} observed {observed_layer:?} control {} detail {detail:?}",
                    control_identity.to_target_display()
                );
            }
            LiveReportObservation::PairedRelation {
                row,
                explicit_run_id,
                explicit_request_id,
                explicit_identity,
                private_run_id,
                private_request_id,
                private_identity,
                relation,
            } => {
                let _ = writeln!(
                    text,
                    "observation {row} paired-relation explicit-run {explicit_run_id} explicit-request {explicit_request_id} explicit-identity {} private-run {private_run_id} private-request {private_request_id} private-identity {} relation {relation:?}",
                    explicit_identity.to_target_display(),
                    private_identity.to_target_display()
                );
            }
            LiveReportObservation::RecordedObservationUnbound { row, observation } => {
                match observation {
                    LiveRecordedObservation::NativeAcceptance { identity } => {
                        let _ = writeln!(
                            text,
                            "observation {row} recorded-observation-unbound native-acceptance identity {}",
                            identity.to_target_display()
                        );
                    }
                    LiveRecordedObservation::NativeRefusal {
                        declared_boundary,
                        observed_layer,
                        control_identity,
                        detail,
                    } => {
                        let _ = writeln!(
                            text,
                            "observation {row} recorded-observation-unbound native-refusal declared {declared_boundary:?} observed {observed_layer:?} control {} detail {detail:?}",
                            control_identity.to_target_display()
                        );
                    }
                    LiveRecordedObservation::PairedRelation {
                        explicit_identity,
                        private_identity,
                        relation,
                    } => {
                        let _ = writeln!(
                            text,
                            "observation {row} recorded-observation-unbound paired-relation explicit-identity {} private-identity {} relation {relation:?}",
                            explicit_identity.to_target_display(),
                            private_identity.to_target_display()
                        );
                    }
                }
            }
            LiveReportObservation::Determinism {
                row,
                recomputed,
                observed_by,
            } => {
                let _ = writeln!(
                    text,
                    "observation {row} determinism recomputed {recomputed:?} observed-by {observed_by:?}"
                );
            }
            LiveReportObservation::FirstPartyFact {
                row,
                fact,
                observed_by,
            } => {
                let _ = writeln!(
                    text,
                    "observation {row} first-party-fact fact {fact:?} observed-by {observed_by:?}"
                );
            }
            LiveReportObservation::ReportLayer { row } => {
                let _ = writeln!(
                    text,
                    "observation {row} report-layer validated-by canonical-bytes"
                );
            }
        }
    }
}

/// One standing's wire spelling.
///
/// Explicit rather than derived from the type's own `Debug`, because a
/// standing carrying a requirement identity would otherwise print the
/// identity into canonical bytes that are meant to summarize.
const fn standing_name(standing: &LiveRowStanding) -> &'static str {
    match standing {
        LiveRowStanding::FirstPartyDischarged { .. } => "first-party-discharged",
        LiveRowStanding::FirstPartyUndischarged(_) => "first-party-undischarged",
        LiveRowStanding::NativeRunRequired(_) => "native-run-required",
        LiveRowStanding::RecordedObservationUnbound(_) => "recorded-observation-unbound",
        // The identity is deliberately not printed here. It is the
        // standing's own payload and belongs in the evidence plan, not
        // in bytes whose job is to summarize.
        LiveRowStanding::NativeRunObserved { .. } => "native-run-observed",
        // Its payload is withheld for the same reason and for one more:
        // the refusal detail is the target's own sentence, and a summary
        // that quoted a verdict would be carrying evidence in bytes
        // whose job is to count.
        LiveRowStanding::NativeRefusalObserved { .. } => "native-refusal-observed",
        // Withheld for the same two reasons as the standing above, and
        // spelled APART from it because the two are different facts.
        // This one says the target refused somewhere other than where
        // the row declared, which is a finding rather than an answer —
        // and it is spelled apart from `native-run-required` too,
        // because a contradicted row is not a row still waiting.
        LiveRowStanding::NativeRefusalAtUnexpectedBoundary { .. } => {
            "native-refusal-at-unexpected-boundary"
        }
        // Withheld for the first reason and not the second: there is no
        // target sentence here to carry, and what the payload names is
        // the first-party test that recomputed the fixture, which
        // belongs in the evidence plan beside the row rather than in
        // bytes whose job is to count.
        LiveRowStanding::DeterminismObserved { .. } => "determinism-observed",
        // TWO identities withheld rather than one, and for the first
        // reason twice over: they are the standing's own payload and
        // belong in the evidence plan beside the row. A summary that
        // printed a pair of identities would be carrying the evidence in
        // bytes whose job is to count it.
        LiveRowStanding::PairedRelationObserved { .. } => "paired-relation-observed",
        // Withheld for the same two reasons: no target sentence exists
        // to carry, and the payload names a first-party site, which
        // belongs beside the row in the evidence plan rather than in
        // bytes whose job is to count.
        LiveRowStanding::FirstPartyFactObserved { .. } => "first-party-fact-observed",
        LiveRowStanding::InfrastructureBlocked(_) => "infrastructure-blocked",
        LiveRowStanding::ReportLayerAnswerable => "report-layer-answerable",
        LiveRowStanding::OperationVocabularyClosed => "operation-vocabulary-closed",
        LiveRowStanding::Experimental => "experimental",
    }
}

/// Every key the canonical rendering is forbidden to emit.
///
/// §1.9's list, as keys rather than as substrings. The distinction is
/// load-bearing: §15.5 has a row *named* `wrong-private-blinding-balance`
/// and §15.6 has one named `report-publishes-sponsor-amount`, and a
/// search for the word "blinding" or "sponsor amount" anywhere in the
/// bytes would flag the matrix's own row names. What §1.9 forbids is a
/// sponsor amount being *emitted*, which is a statement about what a
/// field carries.
const FORBIDDEN_KEYS: &[&str] = &[
    "sponsor_amount",
    "sponsor_amounts",
    "sponsor_opening",
    "sponsor_openings",
    "sponsor_value",
    "sponsor_values",
    "sponsor_blinding",
    "sponsor_change_amount",
    "sponsor_total",
    "blinding_factor",
    "value_blinding",
    "fixture_opening",
];

/// The §15.6 report rows, answered against the canonical bytes.
///
/// Two rows ask whether the report publishes a sponsor amount or a
/// sponsor opening, and this is what answers them: the canonical
/// serialization is rendered and every line's *key* is compared against
/// this module's forbidden-key list. §1.9 forbids an individual sponsor
/// amount from
/// being emitted in a canonical report at all, so the answer must be
/// that no line carries one.
#[must_use]
pub fn canonical_bytes_publish_no_sponsor_value(
    validated: &ValidatedLiveTransferSafetyReport,
) -> bool {
    render_live_safety_report(validated)
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .all(|key| !FORBIDDEN_KEYS.contains(&key))
}

/// The per-section scoreboard a reader of a report wants first.
#[must_use]
pub fn section_scoreboard(
    plan: &LiveTransferEvidencePlan,
) -> BTreeMap<LiveSafetySection, (usize, usize, usize)> {
    let mut board: BTreeMap<LiveSafetySection, (usize, usize, usize)> = LiveSafetySection::ALL
        .iter()
        .map(|section| (*section, (0, 0, 0)))
        .collect();
    for row in plan.rows() {
        let entry = board.entry(row.row().section()).or_insert((0, 0, 0));
        entry.0 += 1;
        if row.standing().is_answered() {
            entry.1 += 1;
        }
        if row.standing().is_infrastructure_error() {
            entry.2 += 1;
        }
    }
    board
}

#[cfg(test)]
mod tests {
    use super::{
        LiveDeploymentBinding, LiveExecutorProvenance, LiveRecordedObservation,
        LiveReportObservation, LiveRunBinding, LiveSafetyCompleteness, LiveSafetyDiagnostics,
        LiveSafetyReportRefusal, LiveSafetyReportRole, LiveTargetResponse, RecomputedItem,
        VolatileField, assemble_live_safety_report, canonical_bytes_publish_no_sponsor_value,
        compare_run_bindings, render_live_safety_report, section_scoreboard, target_evidence_name,
        validate_live_safety_report,
    };
    use crate::live_evidence::derive_live_evidence_plan;
    use crate::live_plan::reviewed_target;
    use crate::live_safety::{LiveSafetyPolarity, LiveSafetySection};
    use target_elements::TargetProjection;

    fn projection() -> TargetProjection {
        reviewed_target()
            .expect("the reviewed contract validates")
            .projection()
    }

    fn parsed_identity(value: &str) -> transaction::Txid {
        transaction::Txid::from_target_display(value).expect("the test identity parses")
    }

    fn synthetic_run() -> LiveRunBinding {
        let identity =
            parsed_identity("40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029");
        LiveRunBinding {
            run_id: "synthetic-bound-run".to_owned(),
            deployment: LiveDeploymentBinding {
                environment: "development".to_owned(),
                network_id: [0x11; 32],
                genesis_id: [0x22; 32],
                target_contract: "elements-tapscript-v1".to_owned(),
            },
            executor: LiveExecutorProvenance {
                adapter_name: "native-adapter".to_owned(),
                adapter_version: "1".to_owned(),
                node_name: "elementsd".to_owned(),
                node_version: "28.99".to_owned(),
                executed_source_tip: "recorded-tip".to_owned(),
            },
            requests: std::collections::BTreeMap::from([(
                "request-a".to_owned(),
                vec![0x01, 0x02, 0x03],
            )]),
            responses: std::collections::BTreeMap::from([(
                "request-a".to_owned(),
                LiveTargetResponse::Accepted { identity },
            )]),
        }
    }

    #[test]
    fn an_unexpected_boundary_refusal_is_outstanding_and_fails_the_report() {
        use super::{completeness_of, standing_name};
        use crate::live_evidence::{LiveEvidenceCensus, LiveRowStanding};
        use crate::matrix::EvidenceBoundary;
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let standing = LiveRowStanding::NativeRefusalAtUnexpectedBoundary {
            declared_boundary: EvidenceBoundary::ScriptPathRejection,
            observed_layer: ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            control_identity: "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029",
            refusal_detail: "bad-txns-in-ne-out",
        };

        // It renders under its OWN spelling, so a reader can tell a
        // contradiction from a row that is merely waiting.
        assert_eq!(
            standing_name(&standing),
            "native-refusal-at-unexpected-boundary",
        );

        // It lands in `outstanding`. That list is built by filtering on
        // exactly this predicate, so asserting the predicate is
        // asserting the membership.
        assert!(!standing.is_answered());

        // And the census carrying one is FAILED rather than partial —
        // the mismatch taking precedence, with every other bucket clear
        // so the verdict can only have come from the mismatch.
        let census = LiveEvidenceCensus::one_unexpected_boundary_for_tests();
        assert!(!census.every_required_row_is_answered());
        assert_eq!(completeness_of(census), LiveSafetyCompleteness::Failed);
        assert_eq!(LiveSafetyCompleteness::Failed.name(), "failed");
    }

    #[test]
    fn a_report_validates_against_the_plan_it_is_about() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target)
            .expect("the assembled report validates");

        // Only comparisons that actually ran are recorded. The corpus has
        // no transcript-grade run bindings yet, so no run-only item can be
        // smuggled into the witness by inserting `ALL` wholesale.
        assert_eq!(
            validated.recomputed_items(),
            &std::collections::BTreeSet::from([
                RecomputedItem::Role,
                RecomputedItem::Schema,
                RecomputedItem::CaseCensus,
                RecomputedItem::RelationCensus,
                RecomputedItem::MutationLinks,
                RecomputedItem::Summary,
            ]),
        );
        assert_eq!(RecomputedItem::ALL.len(), 14);
        for item in RecomputedItem::ALL {
            if item.needs_a_run() {
                assert!(!validated.recomputed_items().contains(item));
            }
        }
    }

    #[test]
    fn the_assembled_report_is_a_truthful_unbound_corpus_ledger() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let rendered = render_live_safety_report(&validated);

        assert!(rendered.contains("target_evidence recorded-unbound\n"));
        assert!(rendered.contains("run_bindings 0\n"));
        assert!(!rendered.contains("deployment none"));
        assert!(!rendered.contains("run none"));
        assert!(!rendered.contains("NoRunRequested"));

        for exact in [
            "recorded_observation_unbound 42\n",
            "native_run_observed 0\n",
            "native_refusal_observed 0\n",
            "determinism_observed 1\n",
            "paired_relation_observed 0\n",
            "first_party_fact_observed 3\n",
            "observations 48\n",
            "accepted 0\n",
            "refused 0\n",
            "determinism 1\n",
            "paired_relation 0\n",
            "first_party_fact 3\n",
            "report_layer 2\n",
            "report_layer_observations 2\n",
            "recorded_unbound_native_acceptance 24\n",
            "recorded_unbound_native_refusal 17\n",
            "recorded_unbound_paired_relation 1\n",
        ] {
            assert!(rendered.contains(exact), "missing {exact:?}");
        }
        assert_eq!(validated.outstanding().len(), 67);
    }

    #[test]
    fn every_rendered_target_identity_is_typed_and_shared_identities_keep_their_rows() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let rendered = render_live_safety_report(&validated);
        let mut occurrences = std::collections::BTreeMap::new();

        for line in rendered
            .lines()
            .filter(|line| line.starts_with("observation "))
        {
            let words: Vec<_> = line.split_whitespace().collect();
            for pair in words.windows(2) {
                if matches!(
                    pair[0],
                    "identity" | "control" | "explicit-identity" | "private-identity"
                ) {
                    let parsed = transaction::Txid::from_target_display(pair[1])
                        .expect("every rendered identity parses");
                    assert_eq!(parsed.to_target_display(), pair[1]);
                    *occurrences.entry(pair[1]).or_insert(0_usize) += 1;
                }
            }
        }
        assert!(occurrences.values().any(|count| *count > 1));
        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("observation "))
                .count(),
            48,
        );
    }

    #[test]
    fn every_bound_run_fact_has_a_typed_refusal() {
        let expected = vec![synthetic_run()];

        let mut deployment = expected.clone();
        deployment[0].deployment.genesis_id[0] ^= 1;
        assert_eq!(
            compare_run_bindings(&deployment, &expected),
            Err(LiveSafetyReportRefusal::RunDeploymentDiffers(
                "synthetic-bound-run".to_owned(),
            )),
        );

        let mut request = expected.clone();
        request[0]
            .requests
            .insert("request-a".to_owned(), vec![0xff]);
        assert_eq!(
            compare_run_bindings(&request, &expected),
            Err(LiveSafetyReportRefusal::RunRequestDiffers(
                "synthetic-bound-run".to_owned(),
            )),
        );

        let mut response = expected.clone();
        response[0].responses.insert(
            "request-a".to_owned(),
            LiveTargetResponse::Refused {
                observed_layer:
                    target_elements_conformance::protocol::ObservedOutcomeLayer::ScriptPathRejection,
                detail: "mutated".to_owned(),
                control_identity: parsed_identity(
                    "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029",
                ),
            },
        );
        assert_eq!(
            compare_run_bindings(&response, &expected),
            Err(LiveSafetyReportRefusal::RunResponseDiffers(
                "synthetic-bound-run".to_owned(),
            )),
        );

        let mut executor = expected.clone();
        executor[0].executor.node_version = "another-version".to_owned();
        assert_eq!(
            compare_run_bindings(&executor, &expected),
            Err(LiveSafetyReportRefusal::RunExecutorProvenanceDiffers(
                "synthetic-bound-run".to_owned(),
            )),
        );
    }

    #[test]
    fn mutating_a_recorded_identity_refuses_the_report() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let observation = report
            .observations
            .iter_mut()
            .find_map(|observation| match observation {
                LiveReportObservation::RecordedObservationUnbound {
                    observation: LiveRecordedObservation::NativeAcceptance { identity },
                    ..
                } => Some(identity),
                _ => None,
            })
            .expect("the corpus preserves an acceptance");
        *observation =
            parsed_identity("75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1");
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::ObservationsDiffer),
        );
    }

    #[test]
    fn no_target_evidence_fixture_renders_none_recorded() {
        assert_eq!(
            target_evidence_name(super::ObservationCensus::default()),
            "none-recorded",
        );
        let one = [LiveReportObservation::RecordedObservationUnbound {
            row: "fixture-row",
            observation: LiveRecordedObservation::NativeAcceptance {
                identity: parsed_identity(
                    "75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1",
                ),
            },
        }];
        assert_eq!(
            target_evidence_name(super::ObservationCensus::from_observations(&one)),
            "recorded-unbound",
        );
    }

    #[test]
    fn the_private_restart_rows_are_preserved_but_unbound() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target).expect("assembles");
        let rows = report
            .observations()
            .iter()
            .filter_map(|observation| match observation {
                LiveReportObservation::RecordedObservationUnbound { row, .. }
                    if matches!(*row, "private-one-to-one" | "both-commitment-parity-forms") =>
                {
                    Some(*row)
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            rows,
            std::collections::BTreeSet::from([
                "both-commitment-parity-forms",
                "private-one-to-one",
            ]),
        );
    }

    #[test]
    fn the_report_refuses_to_call_itself_complete_and_names_what_is_outstanding() {
        // §13.5's bar and the honest answer to it. A report that claimed
        // completeness while §15.1 has never run would be the one thing
        // this whole module exists to prevent.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        assert_eq!(
            report.completeness(),
            LiveSafetyCompleteness::PartialRequiredRowsOutstanding,
        );
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        assert_ne!(validated.outstanding().len(), 0);
        // And NO blockers, where there used to be one. The report is
        // partial for the honest reasons: some rows need a future run and
        // historical target observations need transcript-grade binding,
        // not because a component they need is missing. The last blocker
        // left when the raw-bypass row was answered by a fact about
        // this workspace, its own gate being whether that path exists.
        //
        // Asserted as an equality rather than dropped, because "nothing
        // is blocked" is a claim this report makes and a reader is
        // entitled to see it checked rather than merely unstated.
        assert_eq!(validated.blockers().len(), 0);
    }

    #[test]
    fn a_report_claiming_a_completeness_its_census_does_not_support_is_refused() {
        // The recomputation made falsifiable: the validator does not read
        // the token, it derives one and compares.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        report.completeness = LiveSafetyCompleteness::CompleteForTheRequiredMatrix;
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::CompletenessDiffers {
                reported: LiveSafetyCompleteness::CompleteForTheRequiredMatrix,
                recomputed: LiveSafetyCompleteness::PartialRequiredRowsOutstanding,
            }),
        );
    }

    #[test]
    fn schema_two_is_hard_rejected() {
        // Schema 2 is historical and never a legacy-validated route into
        // the evidence-bearing schema 3 report.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        report.schema = 2;
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::UnsupportedSchema(2)),
        );
        assert_eq!(
            LiveSafetyReportRole::LiveTransferSafety.name(),
            "live-transfer-safety",
        );
    }

    #[test]
    fn the_canonical_bytes_carry_no_volatile_field() {
        // §13.5's exclusion, checked rather than argued. Each of the ten
        // fields is given a value nothing else in the workspace produces,
        // recorded in the diagnostic report, and the canonical bytes are
        // searched for every one of them. The renderer takes no
        // diagnostic argument, so a hit here would mean some *other*
        // carrier had smuggled a volatile value in.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        let canonical = render_live_safety_report(&validated);

        let mut diagnostics = LiveSafetyDiagnostics::default();
        let mut markers = Vec::with_capacity(VolatileField::ALL.len());
        for (index, field) in VolatileField::ALL.iter().enumerate() {
            let marker = format!("volatile-marker-{index}-{field:?}");
            diagnostics = diagnostics.with(*field, marker.clone());
            markers.push(marker);
        }
        assert_eq!(diagnostics.fields().len(), 10);

        for marker in &markers {
            assert!(
                !canonical.contains(marker.as_str()),
                "the canonical bytes carry {marker}",
            );
            // And the diagnostic report really does carry it, so the
            // check above is not passing because nothing was recorded.
            assert!(diagnostics.render().contains(marker.as_str()));
        }

        // The two renderings are different documents, and the canonical
        // one says nothing about being timed.
        assert!(!canonical.contains("wall"));
        assert!(!canonical.contains("elapsed"));
        assert!(diagnostics.render().contains("canonical false"));
    }

    #[test]
    fn the_canonical_bytes_are_stable_across_renderings() {
        // The property timing would have destroyed: two renderings of the
        // same validated report are the same bytes.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        assert_eq!(
            render_live_safety_report(&validated),
            render_live_safety_report(&validated),
        );
    }

    #[test]
    fn the_canonical_bytes_publish_no_sponsor_value() {
        // §15.6's two report rows, answered. §1.9 forbids an individual
        // sponsor amount from being emitted in a canonical report, and
        // this is the check that says it is not.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone()).expect("assembles");
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        assert!(canonical_bytes_publish_no_sponsor_value(&validated));

        // And the check has teeth: a rendering that really did emit one
        // of the forbidden keys is caught. Staged over the same key
        // vocabulary the production path reads, so a key removed from the
        // list stops being checked here too.
        let rendered = render_live_safety_report(&validated);
        for key in super::FORBIDDEN_KEYS {
            let leaked = format!("{rendered}{key} 1000\n");
            assert!(
                leaked
                    .lines()
                    .filter_map(|line| line.split_whitespace().next())
                    .any(|first| super::FORBIDDEN_KEYS.contains(&first)),
                "{key} would not have been caught",
            );
        }

        // The two §15.6 rows this answers are the ones the plan filed at
        // the report boundary, and there are exactly two of them.
        assert_eq!(plan.census().report_layer(), 2);
    }

    #[test]
    fn the_scoreboard_partitions_the_matrix_by_section() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let board = section_scoreboard(&plan);
        assert_eq!(board.len(), LiveSafetySection::ALL.len());
        let total: usize = board.values().map(|(rows, _, _)| rows).sum();
        assert_eq!(total, crate::live_safety::row_count());

        // Neither positive table waits on a component that does not
        // exist, and the scoreboard exists to say how far each one has
        // actually got rather than to round it away. Both numbers are
        // asserted rather than bounded, because a scoreboard that said
        // "some" would let the next row in without a run.
        //
        // All target-derived rows now preserve their historical observation
        // without counting it as validated evidence. The only positive row
        // still answered is the independently recomputed determinism row.
        let (explicit_rows, explicit_answered, explicit_blocked) =
            board[&LiveSafetySection::PositiveExplicit];
        assert_eq!(explicit_answered, 0, "the explicit table's answered count");
        assert_eq!(explicit_blocked, 0);
        assert_ne!(explicit_rows, 0);

        let (private_rows, private_answered, private_blocked) =
            board[&LiveSafetySection::PositivePrivate];
        assert_eq!(private_answered, 1, "the private table's answered count");
        assert_eq!(private_blocked, 0);
        assert_eq!(private_rows, 10);
        assert_eq!(LiveSafetyPolarity::ALL.len(), 2);
    }

    #[test]
    fn the_lifecycle_status_stays_incomplete() {
        // §17.1: transfer is implemented, burn and redemption are not,
        // and no report may say the candidate is release-complete.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target).expect("assembles");
        assert!(!report.lifecycle().release_complete());
        assert_eq!(report.lifecycle().outstanding().len(), 2);
        assert_eq!(report.lifecycle().implemented().len(), 1);
    }
}
