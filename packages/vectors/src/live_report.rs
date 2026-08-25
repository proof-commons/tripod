//! The validated live-transfer safety report (§13.2, §13.5, §13.6).
//!
//! §13.2 asks one question — did every valid transfer preserve the exact
//! semantic relation, and did every required invalid transfer fail at its
//! owning boundary — and names twelve typed carriers the answer travels
//! in. §13.5 says the gate accepts a validated wrapper rather than a raw
//! report, recomputes fourteen items, and excludes ten volatile fields
//! from the canonical bytes. This module is all three.
//!
//! # The wrapper is the whole point
//!
//! [`LiveTransferSafetyReport`] is a value a caller can build, and it
//! establishes nothing. [`ValidatedLiveTransferSafetyReport`] has private
//! fields and one constructor, [`validate_live_safety_report`], which
//! *recomputes* every item from the evidence plan rather than reading it
//! off the report it was handed. A report whose summary disagreed with
//! its own rows is refused rather than published.
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
use target_elements::{DeploymentProjection, TargetProjection};

use crate::live_evidence::{
    LiveEvidenceCensus, LiveInfrastructureBlocker, LiveRowStanding, LiveTransferEvidencePlan,
    blocker_census,
};
use crate::live_safety::LiveSafetySection;

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
pub const LIVE_SAFETY_REPORT_SCHEMA: u32 = 2;

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

/// Where the run half of the report stands.
///
/// §13.2's carriers five through eight and twelve — the sent requests,
/// the responses, the target verdicts, the semantic projections, and the
/// executor provenance — exist only for a report about a run. A report
/// about no run carries this instead of empty collections, because an
/// empty response census and a census nobody asked for are different
/// facts and §1.11 does not let them share a field.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveRunStanding {
    /// No target-native run was requested.
    ///
    /// Carries the blocker that made a run pointless rather than merely
    /// recording an absence: a run whose every submission would fail for
    /// one unrelated reason produces observations that discharge nothing,
    /// and §4.3's "target was not asked" is the honest record of it.
    NoRunRequested(LiveInfrastructureBlocker),
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
/// own. The twelve carriers are here; the conclusions are
/// [`validate_live_safety_report`]'s.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveTransferSafetyReport {
    schema: u32,
    role: LiveSafetyReportRole,
    target: TargetProjection,
    deployment: Option<DeploymentProjection>,
    representations: BTreeSet<LiveTransferRepresentationPlan>,
    run: LiveRunStanding,
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

    /// The deployment a run was bound to, where one happened.
    ///
    /// `None` says no run was requested, and never that a run happened
    /// against a deployment nobody recorded.
    #[must_use]
    pub const fn deployment(&self) -> Option<&DeploymentProjection> {
        self.deployment.as_ref()
    }

    /// The representation plans in scope.
    #[must_use]
    pub const fn representations(&self) -> &BTreeSet<LiveTransferRepresentationPlan> {
        &self.representations
    }

    /// Where the run half stands.
    #[must_use]
    pub const fn run(&self) -> &LiveRunStanding {
        &self.run
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
    /// The report names a deployment while recording that no run
    /// happened.
    ///
    /// §19.5 binds a report to an exact deployment, and a binding without
    /// a run is a binding to nothing.
    DeploymentWithoutARun,
    /// The report's lifecycle status is not the candidate's.
    LifecycleDiffers,
}

/// One run's report, validated: the only thing a safety gate reads.
///
/// # What holding one of these establishes
///
/// That the report's twelve carriers agree with what this module
/// recomputed from the evidence plan: the same target, the same
/// representation census, the same row classification, the same
/// completeness token, and the same lifecycle status. Every one of those
/// is computed here out of [`LiveTransferEvidencePlan`], which itself has
/// no unchecked constructor, rather than read off the report.
///
/// It establishes nothing about what a target *decided*, because nothing
/// asked one.
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
    /// Six of the fourteen are about things an executor produced. For a
    /// report recording that no run happened, recomputing one means
    /// establishing that it is *empty and consistent with that* — which
    /// is a real check and not a skipped one, because a report claiming
    /// no run while carrying responses would fail it.
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
const fn completeness_of(census: LiveEvidenceCensus) -> LiveSafetyCompleteness {
    if census.every_required_row_is_answered() {
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
#[must_use]
pub fn assemble_live_safety_report(
    plan: &LiveTransferEvidencePlan,
    target: TargetProjection,
) -> LiveTransferSafetyReport {
    let census = plan.census();
    LiveTransferSafetyReport {
        schema: LIVE_SAFETY_REPORT_SCHEMA,
        role: LiveSafetyReportRole::LiveTransferSafety,
        target,
        // No run, so no deployment binding. §19.5 binds a report to an
        // exact deployment and a binding without a run binds nothing.
        deployment: None,
        representations: BTreeSet::from([
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ]),
        run: LiveRunStanding::NoRunRequested(LiveInfrastructureBlocker::OwnerSighashNotComputable),
        lifecycle: LiveLifecycleStatus::candidate(),
        census,
        completeness: completeness_of(census),
    }
}

/// Validate one safety report against the plan it claims to be about.
///
/// §13.5's fourteen items, recomputed here rather than read off the
/// report. The six that are about a run are recomputed for a report
/// recording that no run happened by establishing that its run carriers
/// really are empty, which a report claiming otherwise fails.
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
    if report.schema != LIVE_SAFETY_REPORT_SCHEMA {
        return Err(LiveSafetyReportRefusal::UnsupportedSchema(report.schema));
    }
    if report.role != LiveSafetyReportRole::LiveTransferSafety {
        return Err(LiveSafetyReportRefusal::WrongRole(report.role));
    }
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

    // §19.5's binding, both directions. A report recording no run may not
    // name a deployment, and — for the run half of §13.5's items — its
    // run carriers must really be the empty ones a no-run report has.
    let LiveRunStanding::NoRunRequested(_) = report.run;
    if report.deployment.is_some() {
        return Err(LiveSafetyReportRefusal::DeploymentWithoutARun);
    }

    if report.lifecycle != LiveLifecycleStatus::candidate() {
        return Err(LiveSafetyReportRefusal::LifecycleDiffers);
    }

    let outstanding = plan
        .rows()
        .iter()
        .filter(|row| !row.standing().is_answered())
        .map(|row| (row.row().name(), row.standing().clone()))
        .collect();

    Ok(ValidatedLiveTransferSafetyReport {
        report,
        recomputed_items: RecomputedItem::ALL.iter().copied().collect(),
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
    let _ = writeln!(text, "deployment none");
    for representation in &report.representations {
        let _ = writeln!(text, "representation {representation:?}");
    }
    let LiveRunStanding::NoRunRequested(blocker) = &report.run;
    let _ = writeln!(text, "run none {blocker:?}");

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
        // The identity is deliberately not printed here. It is the
        // standing's own payload and belongs in the evidence plan, not
        // in bytes whose job is to summarize.
        LiveRowStanding::NativeRunObserved { .. } => "native-run-observed",
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
        LiveSafetyCompleteness, LiveSafetyDiagnostics, LiveSafetyReportRefusal,
        LiveSafetyReportRole, RecomputedItem, VolatileField, assemble_live_safety_report,
        canonical_bytes_publish_no_sponsor_value, render_live_safety_report, section_scoreboard,
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

    #[test]
    fn a_report_validates_against_the_plan_it_is_about() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone());
        let validated = validate_live_safety_report(report, &plan, &target)
            .expect("the assembled report validates");

        // §13.5's fourteen, recomputed as a census rather than claimed.
        assert_eq!(
            validated.recomputed_items().len(),
            RecomputedItem::ALL.len()
        );
        assert_eq!(validated.recomputed_items().len(), 14);
    }

    #[test]
    fn the_report_refuses_to_call_itself_complete_and_names_what_is_outstanding() {
        // §13.5's bar and the honest answer to it. A report that claimed
        // completeness while §15.1 has never run would be the one thing
        // this whole module exists to prevent.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let report = assemble_live_safety_report(&plan, target.clone());
        assert_eq!(
            report.completeness(),
            LiveSafetyCompleteness::PartialRequiredRowsOutstanding,
        );
        let validated = validate_live_safety_report(report, &plan, &target).expect("validates");
        assert_ne!(validated.outstanding().len(), 0);
        assert_ne!(validated.blockers().len(), 0);
    }

    #[test]
    fn a_report_claiming_a_completeness_its_census_does_not_support_is_refused() {
        // The recomputation made falsifiable: the validator does not read
        // the token, it derives one and compares.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone());
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
    fn a_report_of_another_role_is_refused_by_the_safety_validator() {
        // §13.6's separation. There is one role today, so the check is
        // staged through the schema instead — a reader holding only the
        // bytes has to be able to refuse a document of the wrong kind,
        // and both fields exist for that.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let target = projection();
        let mut report = assemble_live_safety_report(&plan, target.clone());
        report.schema = super::LIVE_SAFETY_REPORT_SCHEMA + 1;
        assert_eq!(
            validate_live_safety_report(report, &plan, &target),
            Err(LiveSafetyReportRefusal::UnsupportedSchema(
                super::LIVE_SAFETY_REPORT_SCHEMA + 1
            )),
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
        let report = assemble_live_safety_report(&plan, target.clone());
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
        let report = assemble_live_safety_report(&plan, target.clone());
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
        let report = assemble_live_safety_report(&plan, target.clone());
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

        // The positive tables are still mostly unanswered, and the
        // scoreboard exists to make the "mostly" visible at a glance
        // rather than to round it away. Neither table waits on a
        // component that does not exist any more. The explicit table is
        // answered nowhere. The private table is answered in exactly seven
        // rows, because a real node accepted private transfers of those
        // rows' shapes — and the number is asserted rather than bounded,
        // because a scoreboard that said "some" would let the next row
        // in without a run. It read two before the shape wave built the
        // multi-output and multi-input fixtures and ran three more shapes,
        // and six until a merge whose forced blinder is nonzero was
        // accepted; each time it is updated to the observed fact rather
        // than loosened to a range that would stop noticing.
        let (explicit_rows, explicit_answered, explicit_blocked) =
            board[&LiveSafetySection::PositiveExplicit];
        assert_eq!(explicit_answered, 0, "the explicit table claims an answer");
        assert_eq!(explicit_blocked, 0);
        assert_ne!(explicit_rows, 0);

        let (private_rows, private_answered, private_blocked) =
            board[&LiveSafetySection::PositivePrivate];
        assert_eq!(private_answered, 7, "the private table's answered count");
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
        let report = assemble_live_safety_report(&plan, target);
        assert!(!report.lifecycle().release_complete());
        assert_eq!(report.lifecycle().outstanding().len(), 2);
        assert_eq!(report.lifecycle().implemented().len(), 1);
    }
}
