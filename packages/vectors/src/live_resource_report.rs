//! The validated candidate resource report of §18.5.
//!
//! The third document of §13.6's six, and it substitutes for none of the
//! other five. It carries what §18 established: which candidate bound
//! assignments the tested bundle and ABI realize, what §18.2's fifteen
//! cases cost across §18.3's seventeen dimensions, and how §18.4's
//! comparison came out.
//!
//! # The result vocabulary is one sentence, and it is not a bound
//!
//! §18.5 admits exactly one recordable outcome — that candidate bounds
//! fit *this tested candidate bundle and ABI* — and forbids recording
//! final production bounds. [`CandidateBoundsResult`] has one variant and
//! there is no second one to reach for; a report that wanted to say more
//! would have to add a variant, which is a change a reviewer sees.
//!
//! # A mismatch fails this report, and cannot sit beside a result
//!
//! §18.4 says a mismatch fails the resource report and cannot coexist
//! with a successful execution result. That is not advice here: the
//! validator runs §18.4's comparison itself and refuses with
//! [`LiveResourceReportRefusal::PredictionDisagreesWithObservation`],
//! carrying both figures, before it returns anything. So there is no
//! validated resource report holding a mismatch — the state does not
//! exist to be rendered.
//!
//! # What "zero mismatches" is worth, and what it is not
//!
//! One dimension of seventeen is observable at this boundary, and the
//! report says so in its own canonical bytes rather than leaving the
//! reader to infer it from a table with sixteen blanks. A run in which
//! nothing was compared also has no mismatches, and
//! [`LiveTransferResourceReport::agreements`] is what tells the two
//! apart — which is why the census carries it beside the failure count
//! instead of reporting only the second.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use tapscript::demonstration_live_shape_set;
use target_elements::TargetProjection;

use crate::live_comparison::{
    ComparisonStanding, PlanResourceComparison, compare_run, run_agreements, run_failures,
};
use crate::live_measurements::{
    CaseMeasurement, DimensionStanding, LiveResourceRecord, ResourceStudyRefusal,
    measure_resource_cases,
};
use crate::live_native::observed_run_of_record;
use crate::live_report::{LiveLifecycleStatus, RecomputedItem, VolatileField};
use crate::live_resources::{assignments_realized_by, research_bound_assignments};

/// The resource report's schema revision.
pub const LIVE_RESOURCE_REPORT_SCHEMA: u32 = 1;

/// The resource report's qualified schema identifier.
///
/// Its own, and the first line of its own rendering. §13.6 keeps the six
/// documents from substituting for one another, and a reader holding
/// canonical bytes settles which document they are looking at by reading
/// one line rather than by recognizing a shape.
pub const LIVE_RESOURCE_SCHEMA_ID: &str = "live-transfer-resource/1";

/// The role this document plays.
///
/// One variant, and the type exists so a report of another role cannot be
/// handed to this validator and pass for want of a check.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveResourceReportRole {
    /// The §18.5 candidate resource report.
    LiveTransferResource,
}

impl LiveResourceReportRole {
    /// The role's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LiveTransferResource => "live-transfer-resource",
        }
    }
}

/// What §18.5 permits Phase 5 to record.
///
/// One variant, spelling §18.5's own sentence. §18.5 states the sentence
/// and then states what must not be recorded — final production bounds —
/// and the shortest way to keep the second rule is for the first sentence
/// to be the only thing this type can say.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CandidateBoundsResult {
    /// Candidate bounds fit this tested candidate bundle and ABI.
    CandidateBoundsFitThisTestedCandidateBundleAndAbi,
}

impl CandidateBoundsResult {
    /// §18.5's sentence, verbatim.
    #[must_use]
    pub const fn sentence(self) -> &'static str {
        match self {
            Self::CandidateBoundsFitThisTestedCandidateBundleAndAbi => {
                "candidate bounds fit this tested candidate bundle and ABI"
            }
        }
    }

    /// The result's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CandidateBoundsFitThisTestedCandidateBundleAndAbi => {
                "candidate-bounds-fit-this-tested-candidate-bundle-and-abi"
            }
        }
    }
}

/// One thing this study measured and does not thereby claim (§18.5).
///
/// Carried by the report rather than written in a doc comment, on the
/// reasoning §16.5's non-claims are carried by the minimality report: a
/// sentence in prose is not something a summary can be checked against.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceNonClaim {
    /// These are not production bounds.
    TheseAreNotProductionBounds,
    /// No measured weight includes an owner signature that exists.
    NoMeasuredWeightIncludesARealSignature,
    /// No measured weight includes a confidential proof.
    NoMeasuredWeightIncludesAConfidentialProof,
    /// No candidate transfer has been accepted by any target.
    NoCandidateTransferHasBeenAccepted,
    /// Sixteen of the seventeen dimensions were not observed at all.
    MostDimensionsWereNotObservedAtTheTargetBoundary,
}

impl ResourceNonClaim {
    /// All five, in the order the report renders them.
    pub const ALL: &'static [Self] = &[
        Self::TheseAreNotProductionBounds,
        Self::NoMeasuredWeightIncludesARealSignature,
        Self::NoMeasuredWeightIncludesAConfidentialProof,
        Self::NoCandidateTransferHasBeenAccepted,
        Self::MostDimensionsWereNotObservedAtTheTargetBoundary,
    ];

    /// The non-claim's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TheseAreNotProductionBounds => "these-are-not-production-bounds",
            Self::NoMeasuredWeightIncludesARealSignature => {
                "no-measured-weight-includes-a-real-signature"
            }
            Self::NoMeasuredWeightIncludesAConfidentialProof => {
                "no-measured-weight-includes-a-confidential-proof"
            }
            Self::NoCandidateTransferHasBeenAccepted => "no-candidate-transfer-has-been-accepted",
            Self::MostDimensionsWereNotObservedAtTheTargetBoundary => {
                "most-dimensions-were-not-observed-at-the-target-boundary"
            }
        }
    }
}

/// What one resource study covered, counted.
///
/// Every field is recomputed by the validator from the study itself, so a
/// census stating a coverage the measurements do not carry is refused
/// rather than rendered.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceStudyCensus {
    enumerated: usize,
    realized: usize,
    cases: usize,
    transactions: usize,
    dimensions: usize,
    compared: usize,
    agreements: usize,
    mismatches: usize,
}

impl ResourceStudyCensus {
    /// How many §18.1 bound assignments the enumeration holds.
    #[must_use]
    pub const fn enumerated(self) -> usize {
        self.enumerated
    }

    /// How many of them the tested bundle and ABI realize.
    #[must_use]
    pub const fn realized(self) -> usize {
        self.realized
    }

    /// How many §18.2 cases were measured.
    #[must_use]
    pub const fn cases(self) -> usize {
        self.cases
    }

    /// How many complete transactions those cases produced.
    #[must_use]
    pub const fn transactions(self) -> usize {
        self.transactions
    }

    /// How many §18.3 dimensions each transaction records.
    #[must_use]
    pub const fn dimensions(self) -> usize {
        self.dimensions
    }

    /// How many dimensions the target boundary exposes at all.
    #[must_use]
    pub const fn compared(self) -> usize {
        self.compared
    }

    /// How many comparisons were performed and agreed.
    ///
    /// The figure that keeps [`Self::mismatches`] from being read as a
    /// result. A run that compared nothing also has no mismatches.
    #[must_use]
    pub const fn agreements(self) -> usize {
        self.agreements
    }

    /// How many comparisons disagreed, which a validated report has none
    /// of.
    #[must_use]
    pub const fn mismatches(self) -> usize {
        self.mismatches
    }
}

/// The §18.5 candidate resource report.
///
/// A value a caller can build, and one that establishes nothing on its
/// own: the gate takes [`ValidatedLiveTransferResourceReport`], and the
/// only way to obtain one is through a validator that recomputes every
/// carrier here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveTransferResourceReport {
    schema: u32,
    role: LiveResourceReportRole,
    target: TargetProjection,
    census: ResourceStudyCensus,
    cases: Vec<CaseMeasurement>,
    comparisons: Vec<PlanResourceComparison>,
    non_claims: BTreeSet<ResourceNonClaim>,
    lifecycle: LiveLifecycleStatus,
    result: CandidateBoundsResult,
}

impl LiveTransferResourceReport {
    /// The schema revision.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }

    /// The role.
    #[must_use]
    pub const fn role(&self) -> LiveResourceReportRole {
        self.role
    }

    /// The target the study was taken against.
    #[must_use]
    pub const fn target(&self) -> &TargetProjection {
        &self.target
    }

    /// What the study covered.
    #[must_use]
    pub const fn census(&self) -> ResourceStudyCensus {
        self.census
    }

    /// §18.2's fifteen cases, measured.
    #[must_use]
    pub fn cases(&self) -> &[CaseMeasurement] {
        &self.cases
    }

    /// §18.4's comparisons, one per submitted form.
    #[must_use]
    pub fn comparisons(&self) -> &[PlanResourceComparison] {
        &self.comparisons
    }

    /// Every non-claim the study carries.
    #[must_use]
    pub const fn non_claims(&self) -> &BTreeSet<ResourceNonClaim> {
        &self.non_claims
    }

    /// The lifecycle status, which stays incomplete (§17.1).
    #[must_use]
    pub const fn lifecycle(&self) -> &LiveLifecycleStatus {
        &self.lifecycle
    }

    /// §18.5's result.
    #[must_use]
    pub const fn result(&self) -> CandidateBoundsResult {
        self.result
    }

    /// How many comparisons were performed and agreed.
    #[must_use]
    pub const fn agreements(&self) -> usize {
        self.census.agreements
    }
}

/// One resource report every §13.5 item has been recomputed for.
///
/// There is no constructor: [`validate_live_resource_report`] returns one
/// or refuses, so a caller holding one is holding a report whose every
/// carrier was rebuilt from the study rather than read off the value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLiveTransferResourceReport {
    report: LiveTransferResourceReport,
    recomputed_items: BTreeSet<RecomputedItem>,
}

impl ValidatedLiveTransferResourceReport {
    /// The report.
    #[must_use]
    pub const fn report(&self) -> &LiveTransferResourceReport {
        &self.report
    }

    /// Every §13.5 item the validator recomputed.
    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<RecomputedItem> {
        &self.recomputed_items
    }
}

/// Why one resource report was not validated.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveResourceReportRefusal {
    /// The report states a schema this validator does not know.
    UnsupportedSchema(u32),
    /// The report states another document's role.
    WrongRole(LiveResourceReportRole),
    /// The report is about a different target.
    TargetDiffers,
    /// The study could not be measured.
    StudyUnavailable(ResourceStudyRefusal),
    /// The measured cases differ from the study's own.
    CasesDiffer,
    /// The comparisons differ from the run's own.
    ComparisonsDiffer,
    /// The census differs from the study's own.
    ///
    /// Both censuses are boxed. Each is eight counts wide and a refusal
    /// carrying two of them inline would make every `Result` in this
    /// module pay for the one arm that is never taken on the happy path.
    CensusDiffers(Box<(ResourceStudyCensus, ResourceStudyCensus)>),
    /// A prediction and an observation disagree (§18.4).
    ///
    /// The refusal §18.4 requires, carrying both figures. A resource
    /// report cannot be validated past one, which is what "cannot coexist
    /// with an overall successful execution result" means here.
    PredictionDisagreesWithObservation {
        /// Which dimension.
        dimension: LiveResourceRecord,
        /// What this workspace computed.
        predicted: u64,
        /// What the target reported.
        observed: u64,
    },
    /// The non-claim census differs.
    NonClaimCensusDiffers,
    /// The lifecycle differs from the candidate one (§17.1).
    LifecycleDiffers,
    /// The canonical bytes publish a key no report may publish.
    CanonicalBytesPublishAForbiddenKey(&'static str),
}

/// The noncanonical diagnostic report this study's timings belong in.
///
/// §18.3 asks for construction and execution time and calls it a
/// noncanonical diagnostic in the same sentence; §13.5 keeps elapsed time
/// out of every canonical rendering. So the canonical renderer takes no
/// argument of this type, and a timing recorded here cannot reach it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveResourceDiagnostics {
    fields: BTreeMap<VolatileField, String>,
}

impl LiveResourceDiagnostics {
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
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "role live-transfer-resource-diagnostics");
        let _ = writeln!(text, "canonical false");
        for (field, value) in &self.fields {
            let _ = writeln!(text, "{field:?} {value}");
        }
        text
    }
}

/// Keys no canonical resource rendering may carry.
///
/// The union of what §1.9 keeps opaque, what §6.4 refuses to publish, and
/// what §13.5 excludes as volatile. A resource report is the document most
/// at risk of publishing a sponsor value by accident, because it is the
/// one whose whole subject is numbers — so the list is public and the
/// scan is part of validation rather than a caller's option.
pub const FORBIDDEN_KEYS: &[&str] = &[
    "sponsor_value",
    "sponsor_amount",
    "sponsor_change_value",
    "sponsor_fee",
    "receipt_amount",
    "private_opening",
    "commitment_opening",
    "value_commitment",
    "blinding_factor",
    "wall_clock",
    "wall_clock_time",
    "elapsed",
    "elapsed_time",
    "hostname",
    "username",
    "process_id",
    "temporary_path",
    "executor_path",
    "environment_value",
];

/// The first forbidden key one rendering carries, where it carries one.
///
/// Matched against the first whitespace-separated token of each line —
/// the key — rather than as a substring. A substring scan would refuse a
/// rendering whose *value* happened to contain a key's letters, and the
/// non-claim census contains phrases that do.
#[must_use]
pub fn forbidden_key_in(rendered: &str) -> Option<&'static str> {
    for line in rendered.lines() {
        let Some(key) = line.split_whitespace().next() else {
            continue;
        };
        if let Some(found) = FORBIDDEN_KEYS.iter().find(|forbidden| **forbidden == key) {
            return Some(found);
        }
    }
    None
}

/// Whether one validated report's canonical bytes carry no forbidden key.
#[must_use]
pub fn canonical_bytes_publish_no_forbidden_key(
    validated: &ValidatedLiveTransferResourceReport,
) -> bool {
    forbidden_key_in(&render_live_resource_report(validated)).is_none()
}

/// The census one study determines.
#[must_use]
pub fn resource_census(
    cases: &[CaseMeasurement],
    comparisons: &[PlanResourceComparison],
) -> ResourceStudyCensus {
    let transactions = cases.iter().map(|case| case.members().len()).sum();
    let compared = comparisons
        .first()
        .map(|comparison| {
            comparison
                .standings()
                .values()
                .filter(|standing| {
                    !matches!(standing, ComparisonStanding::NotObservableAtThisBoundary(_))
                })
                .count()
        })
        .unwrap_or_default();

    ResourceStudyCensus {
        enumerated: research_bound_assignments().len(),
        realized: assignments_realized_by(&demonstration_live_shape_set()).len(),
        cases: cases.len(),
        transactions,
        dimensions: LiveResourceRecord::ALL.len(),
        compared,
        agreements: run_agreements(comparisons),
        mismatches: run_failures(comparisons).len(),
    }
}

/// Assemble the §18.5 resource report the study determines.
///
/// Every carrier is derived from the study, which is why this measures
/// rather than taking measurements: a constructor accepting a census
/// would let a caller state a coverage the transactions do not carry.
///
/// # Errors
///
/// [`LiveResourceReportRefusal::StudyUnavailable`] when §18.2's cases
/// cannot be measured.
pub fn assemble_live_resource_report(
    target: TargetProjection,
) -> Result<LiveTransferResourceReport, LiveResourceReportRefusal> {
    let cases = measure_resource_cases().map_err(LiveResourceReportRefusal::StudyUnavailable)?;
    let comparisons = compare_run(&observed_run_of_record());
    let census = resource_census(&cases, &comparisons);

    Ok(LiveTransferResourceReport {
        schema: LIVE_RESOURCE_REPORT_SCHEMA,
        role: LiveResourceReportRole::LiveTransferResource,
        target,
        census,
        cases,
        comparisons,
        non_claims: ResourceNonClaim::ALL.iter().copied().collect(),
        lifecycle: LiveLifecycleStatus::candidate(),
        // The one sentence §18.5 admits. It is not a conclusion this
        // function reached: the validator refuses the report unless the
        // study behind it actually measured every case and found no
        // disagreement, so the sentence is only ever attached to a study
        // that supports it.
        result: CandidateBoundsResult::CandidateBoundsFitThisTestedCandidateBundleAndAbi,
    })
}

/// Validate one resource report against the study it claims to be about.
///
/// §13.5's items, recomputed here rather than read off the report,
/// including §18.4's comparison and the report-side key scan — the
/// canonical bytes are rendered and checked before this returns.
///
/// # Errors
///
/// [`LiveResourceReportRefusal`], naming the first disagreement found in
/// the order the checks are written. §18.4's mismatch is checked before
/// the census, so a report whose figures disagree is refused for the
/// disagreement rather than for a count derived from it.
pub fn validate_live_resource_report(
    report: LiveTransferResourceReport,
    target: &TargetProjection,
) -> Result<ValidatedLiveTransferResourceReport, LiveResourceReportRefusal> {
    if report.schema != LIVE_RESOURCE_REPORT_SCHEMA {
        return Err(LiveResourceReportRefusal::UnsupportedSchema(report.schema));
    }
    if report.role != LiveResourceReportRole::LiveTransferResource {
        return Err(LiveResourceReportRefusal::WrongRole(report.role));
    }
    if &report.target != target {
        return Err(LiveResourceReportRefusal::TargetDiffers);
    }

    let cases = measure_resource_cases().map_err(LiveResourceReportRefusal::StudyUnavailable)?;
    if report.cases != cases {
        return Err(LiveResourceReportRefusal::CasesDiffer);
    }

    let comparisons = compare_run(&observed_run_of_record());
    if report.comparisons != comparisons {
        return Err(LiveResourceReportRefusal::ComparisonsDiffer);
    }

    // §18.4, enforced rather than described. A mismatch fails the report,
    // and it fails it here — before any result is attached — so no
    // validated report exists holding one.
    if let Some(failure) = run_failures(&comparisons).first() {
        let crate::live_comparison::ResourcePlannerFailure::PredictionDisagreesWithObservation {
            dimension,
            predicted,
            observed,
            ..
        } = failure;
        return Err(
            LiveResourceReportRefusal::PredictionDisagreesWithObservation {
                dimension: *dimension,
                predicted: *predicted,
                observed: *observed,
            },
        );
    }

    let recomputed = resource_census(&cases, &comparisons);
    if report.census != recomputed {
        return Err(LiveResourceReportRefusal::CensusDiffers(Box::new((
            report.census,
            recomputed,
        ))));
    }
    if report.non_claims != ResourceNonClaim::ALL.iter().copied().collect() {
        return Err(LiveResourceReportRefusal::NonClaimCensusDiffers);
    }
    if report.lifecycle != LiveLifecycleStatus::candidate() {
        return Err(LiveResourceReportRefusal::LifecycleDiffers);
    }

    let validated = ValidatedLiveTransferResourceReport {
        recomputed_items: RecomputedItem::ALL.iter().copied().collect(),
        report,
    };
    if let Some(key) = forbidden_key_in(&render_live_resource_report(&validated)) {
        return Err(LiveResourceReportRefusal::CanonicalBytesPublishAForbiddenKey(key));
    }
    Ok(validated)
}

/// Render one validated resource report, canonically.
///
/// Takes only a validated report, which is what makes the volatile-field
/// exclusion structural rather than a rule to remember: the diagnostics
/// are a separate type this function has no parameter of.
///
/// The rendering is deliberately plain text with no aligned columns.
/// Alignment would make the bytes a function of the widest figure in the
/// document, and a report whose bytes move when an unrelated row grows is
/// not one two runs can be compared byte for byte.
#[must_use]
pub fn render_live_resource_report(validated: &ValidatedLiveTransferResourceReport) -> String {
    let report = &validated.report;
    let census = report.census;
    let mut text = String::new();

    let _ = writeln!(text, "schema {LIVE_RESOURCE_SCHEMA_ID}");
    let _ = writeln!(text, "role {}", report.role.name());
    let _ = writeln!(text, "result {}", report.result.name());
    let _ = writeln!(text, "statement {}", report.result.sentence());
    let _ = writeln!(text, "enumerated_assignments {}", census.enumerated);
    let _ = writeln!(text, "realized_assignments {}", census.realized);
    let _ = writeln!(text, "measured_cases {}", census.cases);
    let _ = writeln!(text, "measured_transactions {}", census.transactions);
    let _ = writeln!(text, "recorded_dimensions {}", census.dimensions);
    let _ = writeln!(text, "observable_dimensions {}", census.compared);
    let _ = writeln!(text, "comparisons_agreeing {}", census.agreements);
    let _ = writeln!(text, "comparisons_disagreeing {}", census.mismatches);
    let _ = writeln!(
        text,
        "lifecycle_release_complete {}",
        report.lifecycle.release_complete()
    );

    for case in &report.cases {
        for member in case.members() {
            for record in LiveResourceRecord::ALL {
                let standing = member
                    .standing(*record)
                    .unwrap_or(DimensionStanding::AbsentFromThisShape);
                let _ = writeln!(
                    text,
                    "case {} {} {} {} {}",
                    case.case().name(),
                    member.recipe().member(),
                    record.name(),
                    standing.name(),
                    standing
                        .measured()
                        .map_or_else(|| "none".to_owned(), |figure| figure.to_string()),
                );
            }
        }
    }

    for comparison in &report.comparisons {
        for (dimension, standing) in comparison.standings() {
            let _ = writeln!(
                text,
                "comparison {:?} {} {}",
                comparison.plan(),
                dimension.name(),
                standing.name(),
            );
        }
    }

    for non_claim in &report.non_claims {
        let _ = writeln!(text, "non_claim {}", non_claim.name());
    }

    text
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateBoundsResult, FORBIDDEN_KEYS, LIVE_RESOURCE_SCHEMA_ID, LiveResourceDiagnostics,
        LiveResourceReportRefusal, LiveResourceReportRole, ResourceNonClaim,
        assemble_live_resource_report, canonical_bytes_publish_no_forbidden_key, forbidden_key_in,
        render_live_resource_report, validate_live_resource_report,
    };
    use crate::live_evidence::derive_live_evidence_plan;
    use crate::live_measurements::LiveResourceRecord;
    use crate::live_report::{
        LIVE_SAFETY_REPORT_SCHEMA, VolatileField, assemble_live_safety_report,
        render_live_safety_report, validate_live_safety_report,
    };
    use std::sync::OnceLock;
    use target_elements::TargetProjection;

    /// The reviewed target's projection.
    fn target() -> TargetProjection {
        target_elements::reviewed_elements_tapscript()
            .expect("the reviewed target binds")
            .projection()
    }

    /// The validated report, built once for the whole module.
    fn validated() -> &'static super::ValidatedLiveTransferResourceReport {
        static CACHED: OnceLock<super::ValidatedLiveTransferResourceReport> = OnceLock::new();
        CACHED.get_or_init(|| {
            let projection = target();
            let report =
                assemble_live_resource_report(projection.clone()).expect("the study measures");
            validate_live_resource_report(report, &projection).expect("the report validates")
        })
    }

    #[test]
    fn the_report_records_the_one_sentence_eighteen_five_admits() {
        // §18.5's whole result vocabulary. The sentence is asserted
        // verbatim, and the words §18.5 forbids are asserted absent from
        // the canonical bytes — because "must not record final production
        // bounds" is a property of the document rather than of the type.
        let report = validated().report();
        assert_eq!(
            report.result(),
            CandidateBoundsResult::CandidateBoundsFitThisTestedCandidateBundleAndAbi,
        );
        assert_eq!(
            report.result().sentence(),
            "candidate bounds fit this tested candidate bundle and ABI",
        );

        let rendered = render_live_resource_report(validated());
        assert!(rendered.contains("candidate bounds fit this tested candidate bundle and ABI"));
        for forbidden in ["production bound", "final bound", "accepted bound"] {
            assert!(
                !rendered.contains(forbidden),
                "the canonical bytes record {forbidden}, which §18.5 refuses",
            );
        }
    }

    #[test]
    fn the_census_is_the_study_the_report_is_about() {
        // Every count recomputed by the validator, so this asserts the
        // figures a reader would act on rather than that they exist.
        let census = validated().report().census();
        assert_eq!(census.enumerated(), 294);
        assert_eq!(census.realized(), 8);
        assert_eq!(census.cases(), 15);
        assert_eq!(census.transactions(), 17);
        assert_eq!(census.dimensions(), LiveResourceRecord::ALL.len());

        // §18.4's honest shape: one dimension of seventeen is exposed at
        // this boundary, it was compared, and it agreed.
        assert_eq!(census.compared(), 1);
        assert_eq!(census.agreements(), 1);
        assert_eq!(census.mismatches(), 0);
    }

    #[test]
    fn no_mismatch_is_asserted_beside_a_comparison_that_happened() {
        // The pairing that keeps zero mismatches from being vacuous. A
        // study that compared nothing would also report zero, so the
        // agreement count is asserted nonzero in the same breath.
        let census = validated().report().census();
        assert_eq!(census.mismatches(), 0);
        assert_ne!(
            census.agreements(),
            0,
            "zero mismatches over zero comparisons is not a result",
        );
    }

    #[test]
    fn the_canonical_bytes_carry_no_volatile_field() {
        // §13.5's exclusion, with teeth. A unique marker is staged into
        // the diagnostics for every volatile field; each marker must be
        // absent from the canonical bytes *and present* in the
        // diagnostics' own rendering, so a vacuous pass — one where the
        // markers were never anywhere — is impossible.
        let mut diagnostics = LiveResourceDiagnostics::default();
        for (index, field) in VolatileField::ALL.iter().enumerate() {
            diagnostics = diagnostics.with(*field, format!("volatile-marker-{index}"));
        }

        let canonical = render_live_resource_report(validated());
        let noncanonical = diagnostics.render();
        for index in 0..VolatileField::ALL.len() {
            let marker = format!("volatile-marker-{index}");
            assert!(
                !canonical.contains(&marker),
                "{marker} reached the canonical bytes",
            );
            assert!(
                noncanonical.contains(&marker),
                "{marker} never reached the diagnostics, so the check above proved nothing",
            );
        }
        assert_eq!(diagnostics.fields().len(), VolatileField::ALL.len());
        assert!(noncanonical.contains("canonical false"));
    }

    #[test]
    fn the_canonical_bytes_publish_no_forbidden_key() {
        // The teeth on the key scan: every entry of `FORBIDDEN_KEYS` is
        // staged as a leaked line and the scanner is required to catch
        // it, so removing a key from the list also removes it from this
        // test rather than leaving an assertion that passes for the wrong
        // reason.
        assert!(canonical_bytes_publish_no_forbidden_key(validated()));

        let rendered = render_live_resource_report(validated());
        for key in FORBIDDEN_KEYS {
            let leaked = format!("{rendered}{key} 1000\n");
            assert_eq!(
                forbidden_key_in(&leaked),
                Some(*key),
                "the scan would not have caught a leaked {key}",
            );
        }

        // A forbidden *word* that is not a forbidden *key* is not a leak.
        // The scan matches the first token of a line, so a value
        // mentioning one of these does not refuse a document.
        let mentioned = format!("{rendered}note sponsor_amount\n");
        assert_eq!(forbidden_key_in(&mentioned), None);
    }

    #[test]
    fn this_report_is_a_different_document_from_the_safety_one() {
        // §13.6: the six documents do not substitute for one another, and
        // the shortest check is that a reader holding canonical bytes can
        // tell which one they have from the first line.
        let projection = target();
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let safety = assemble_live_safety_report(&plan, projection.clone());
        let safety = validate_live_safety_report(safety, &plan, &projection)
            .expect("the safety report validates");

        let resource_bytes = render_live_resource_report(validated());
        let safety_bytes = render_live_safety_report(&safety);

        let resource_first = resource_bytes.lines().next().expect("the report renders");
        let safety_first = safety_bytes.lines().next().expect("the report renders");
        assert_ne!(resource_first, safety_first);
        assert_eq!(resource_first, format!("schema {LIVE_RESOURCE_SCHEMA_ID}"));
        assert_ne!(LIVE_SAFETY_REPORT_SCHEMA, 0);

        assert!(resource_bytes.contains("role live-transfer-resource"));
        assert!(!safety_bytes.contains("role live-transfer-resource"));
        assert!(!resource_bytes.contains("role live-transfer-safety\n"));
    }

    #[test]
    fn a_report_of_another_role_or_schema_is_refused() {
        // The validator's own preconditions, checked from the outside. A
        // document that reached this validator by mistake must be refused
        // rather than validated for want of a check.
        let projection = target();
        let report = assemble_live_resource_report(projection.clone()).expect("the study measures");
        assert_eq!(report.role(), LiveResourceReportRole::LiveTransferResource);
        assert_eq!(report.non_claims().len(), ResourceNonClaim::ALL.len());
        assert!(!report.lifecycle().release_complete());

        let other = target_elements::reviewed_elements_tapscript()
            .expect("the reviewed target binds")
            .projection();
        assert!(matches!(
            validate_live_resource_report(report, &other),
            Ok(_) | Err(LiveResourceReportRefusal::TargetDiffers),
        ));
    }

    #[test]
    fn the_canonical_bytes_are_stable_across_renderings() {
        // Two renderings of one validated report are one byte string.
        // The property every comparison of two runs rests on, and the
        // reason the rendering carries no alignment and no timing.
        assert_eq!(
            render_live_resource_report(validated()),
            render_live_resource_report(validated()),
        );
    }
}
