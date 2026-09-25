//! The maturity candidate's separate §14.8 resource report assembles the archived comparison, six measured candidates and bounded nonce corpus.
//!
//! Its canonical bytes carry the twenty-one resource-field standings for each archived submission, the target roster, fourteen cases, six built candidates, nonce selections, one observed result, the seven present candidate parameters and a census of absence reasons. Construction time, nonce attempts and archived sidecar status travel in a separate diagnostics rendering. The result covers transaction weight over two submissions; it establishes no final calibration, relayability or production property, and its timing measures host construction only.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use target_elements::ResourceDimension;

use crate::live_resource_report::forbidden_key_in;
use crate::maturity_measurements::{
    MaturityCaseStanding, MaturityMeasuredCandidate, MaturityResourceCase, measure_maturity_cases,
};
use crate::maturity_native::MaturityLead;
use crate::maturity_nonce_corpus::{
    MaturityConstructorSide, MaturityNonceCorpus, maturity_nonce_corpus,
};
use crate::maturity_resources::{
    MaturityObservationStanding, MaturityPrediction, MaturityPredictionAbsence,
    MaturityResourceFailure, MaturityResourceField, MaturitySubmission,
    MaturitySubmissionResources, MaturityUnobservedReason, archived_submission_resources,
};

/// The separate candidate resource document's schema revision.
pub const MATURITY_RESOURCE_REPORT_SCHEMA: u32 = 1;

/// The only role this report can claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityResourceReportRole {
    /// The §14.8 candidate resource report.
    MaturityResource,
}

impl MaturityResourceReportRole {
    /// The role's stable rendering.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MaturityResource => "maturity-resource",
        }
    }
}

/// The sole target comparison supported by the admitted operation boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityResourceResult {
    /// Host and node weights agree for both archived submissions.
    TargetObservedWeightAgreesForTheArchivedSubmissions,
}

impl MaturityResourceResult {
    /// Name the observed dimension and both submissions, without widening the claim.
    #[must_use]
    pub const fn sentence(self) -> &'static str {
        match self {
            Self::TargetObservedWeightAgreesForTheArchivedSubmissions => {
                "Target-observed transaction weight agrees with the host prediction for the historical whole-metadata and accepted variable-metadata archived submissions."
            }
        }
    }
}

/// The one permitted calibration disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityCalibrationResult {
    /// The candidate values below are not final calibration.
    NoCandidateParameterIsFinalCalibration,
}

impl MaturityCalibrationResult {
    /// The exact §18.1 sentence.
    #[must_use]
    pub const fn sentence(self) -> &'static str {
        match self {
            Self::NoCandidateParameterIsFinalCalibration => {
                "No candidate parameter is final calibration"
            }
        }
    }
}

/// §18.1's seven candidate parameters in the guide's order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityCandidateParameter {
    /// The encoded metadata schema width.
    MetadataSchemaSize,
    /// The reviewed representation search bound.
    RepresentationNonceSearchBound,
    /// The static committed leaf set.
    StaticLeafSet,
    /// The reviewed control-path cap.
    TreeDepthPolicy,
    /// The available sponsor input shape.
    SponsorInputBound,
    /// Relay-item and consensus-element limits.
    WitnessLimits,
    /// The starting validation-budget formula.
    TargetValidationBudget,
}

impl MaturityCandidateParameter {
    /// Every §18.1 parameter in its published order.
    pub const ALL: [Self; 7] = [
        Self::MetadataSchemaSize,
        Self::RepresentationNonceSearchBound,
        Self::StaticLeafSet,
        Self::TreeDepthPolicy,
        Self::SponsorInputBound,
        Self::WitnessLimits,
        Self::TargetValidationBudget,
    ];

    /// The parameter's stable key.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MetadataSchemaSize => "metadata-schema-size",
            Self::RepresentationNonceSearchBound => "representation-nonce-search-bound",
            Self::StaticLeafSet => "static-leaf-set",
            Self::TreeDepthPolicy => "tree-depth-policy",
            Self::SponsorInputBound => "sponsor-input-bound",
            Self::WitnessLimits => "witness-limits",
            Self::TargetValidationBudget => "target-validation-budget",
        }
    }

    /// The value fixed by the present tree, stated without calibration.
    #[must_use]
    pub const fn present_value(self) -> &'static str {
        match self {
            Self::MetadataSchemaSize => "86-byte metadata schema with a 53-byte variable region",
            Self::RepresentationNonceSearchBound => "4,096 attempts",
            Self::StaticLeafSet => "singleton announcement leaf",
            Self::TreeDepthPolicy => "control-path depth cap 128",
            Self::SponsorInputBound => "no sponsor input in this generation",
            Self::WitnessLimits => "80-byte relay item bound and 520-byte element bound",
            Self::TargetValidationBudget => "witness size plus 50",
        }
    }
}

/// Every absence reason in the report, with its evidence provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityAbsenceCensus {
    reasons: BTreeMap<&'static str, &'static str>,
}

impl MaturityAbsenceCensus {
    /// The exact reason-to-provenance map validated with the report.
    #[must_use]
    pub const fn reasons(&self) -> &BTreeMap<&'static str, &'static str> {
        &self.reasons
    }
}

/// The candidate's resources before comparison and content validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateMaturityResourceReport {
    schema: u32,
    role: MaturityResourceReportRole,
    submissions: [MaturitySubmissionResources; 2],
    candidates: Vec<MaturityMeasuredCandidate>,
    cases: Vec<(MaturityResourceCase, MaturityCaseStanding)>,
    nonce_corpus: MaturityNonceCorpus,
    result: MaturityResourceResult,
    calibration: MaturityCalibrationResult,
    absences: MaturityAbsenceCensus,
}

/// A resource report whose comparison, coverage and canonical bytes were accepted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedCandidateMaturityResourceReport {
    report: CandidateMaturityResourceReport,
}

impl ValidatedCandidateMaturityResourceReport {
    /// The accepted report.
    #[must_use]
    pub const fn report(&self) -> &CandidateMaturityResourceReport {
        &self.report
    }
}

/// Host-only construction timing, nonce attempts and archived sidecar standings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityResourceDiagnostics {
    construction_time: Option<Duration>,
    nonce_attempts: Vec<(
        MaturitySubmission,
        MaturityConstructorSide,
        Option<MaturityLead>,
        u32,
    )>,
    archive_sidecars: [(MaturitySubmission, &'static str, &'static str, &'static str); 2],
}

impl MaturityResourceDiagnostics {
    /// Render this separate, explicitly noncanonical diagnostic document.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "role maturity-resource-diagnostics");
        let _ = writeln!(text, "canonical false");
        match self.construction_time {
            Some(duration) => {
                let _ = writeln!(text, "host-construction-ms {}", duration.as_millis());
            }
            None => {
                let _ = writeln!(text, "host-construction-ms NoHostConstructionTimeProvided");
            }
        }
        for (submission, side, lead, attempts) in &self.nonce_attempts {
            let _ = writeln!(
                text,
                "nonce-attempts {} {side:?} {lead:?} {attempts}",
                submission_name(*submission)
            );
        }
        for (submission, name, status, duration) in &self.archive_sidecars {
            let _ = writeln!(
                text,
                "archive-sidecar {} {name} status {status} duration {duration}",
                submission_name(*submission)
            );
        }
        text
    }
}

/// A typed refusal before a successful resource result can be rendered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityResourceReportRefusal {
    /// Host and node figures for one archived submission differ.
    PredictionDisagreesWithObservation {
        /// The archived submission.
        submission: MaturitySubmission,
        /// The measured field.
        field: MaturityResourceField,
        /// The host's figure.
        predicted: u64,
        /// The target's figure.
        observed: u64,
    },
    /// An archive could not supply its resources.
    ArchivedResourcesUnavailable(String),
    /// The six candidates could not be measured.
    MeasurementsUnavailable(String),
    /// The eight derivations could not be built.
    NonceCorpusUnavailable(String),
    /// A sidecar has a shape other than the archived three-field status.
    MalformedArchivedTimingSidecar(MaturitySubmission),
    /// The report states another schema.
    UnsupportedSchema(u32),
    /// A submission changed from the admitted comparison.
    SubmissionsDiffer,
    /// A measured candidate changed from the built candidates.
    CandidatesDiffer,
    /// The case roster changed.
    CasesDiffer,
    /// The finite derivation corpus changed.
    NonceCorpusDiffers,
    /// The result or calibration sentence changed.
    DispositionDiffers,
    /// The absence census changed.
    AbsenceCensusDiffers,
    /// The §18.2 case count differs.
    WrongCaseCount(usize),
    /// The nonce derivation count differs.
    WrongNonceCorpusSize(usize),
    /// An absent prediction was replaced by a numeric figure.
    AbsentFigureRenderedAsNumeral {
        /// The submission whose absence changed.
        submission: MaturitySubmission,
        /// The field changed to a figure.
        field: MaturityResourceField,
    },
    /// A forbidden key reached the canonical rendering.
    CanonicalBytesPublishAForbiddenKey(&'static str),
}

const fn submission_name(submission: MaturitySubmission) -> &'static str {
    match submission {
        MaturitySubmission::HistoricalWholeMetadata => "historical-whole-metadata",
        MaturitySubmission::AcceptedVariableMetadata => "accepted-variable-metadata",
    }
}

const fn dimension_name(dimension: ResourceDimension) -> &'static str {
    match dimension {
        ResourceDimension::TransactionWeight => "transaction-weight",
        ResourceDimension::WitnessBytes => "witness-bytes",
        ResourceDimension::ScriptBytes => "script-bytes",
        ResourceDimension::InitialStackItems => "initial-stack-items",
        ResourceDimension::InitialWitnessItemBytes => "initial-witness-item-bytes",
        ResourceDimension::PeakStackItems => "peak-stack-items",
        ResourceDimension::StackElementBytes => "stack-element-bytes",
        ResourceDimension::ValidationBudget => "validation-budget",
        ResourceDimension::OperationCost => "operation-cost",
        ResourceDimension::ControlPathDepth => "control-path-depth",
        ResourceDimension::PackageLimit => "package-limit",
        _ => "unknown-resource-dimension",
    }
}

const fn prediction_absence_name(reason: MaturityPredictionAbsence) -> &'static str {
    match reason {
        MaturityPredictionAbsence::NoEmitterTotalsIt => "NoEmitterTotalsIt",
        MaturityPredictionAbsence::NoPackageInOneTransaction => "NoPackageInOneTransaction",
        MaturityPredictionAbsence::TheFieldIsATargetVerdict => "TheFieldIsATargetVerdict",
    }
}

const fn unobserved_name(reason: MaturityUnobservedReason) -> &'static str {
    match reason {
        MaturityUnobservedReason::TheNodeExposesNoInterpreterAccounting => {
            "TheNodeExposesNoInterpreterAccounting"
        }
        MaturityUnobservedReason::TheFigureIsNotATargetObservation => {
            "TheFigureIsNotATargetObservation"
        }
        MaturityUnobservedReason::TheFieldIsANoncanonicalDiagnostic => {
            "TheFieldIsANoncanonicalDiagnostic"
        }
    }
}

fn insert_reason(reasons: &mut BTreeMap<&'static str, &'static str>, reason: &'static str) {
    let provenance = if reason == "TheNodeExposesNoInterpreterAccounting" {
        "upstream obs:upstream:eg-011"
    } else {
        "lane-local"
    };
    reasons.insert(reason, provenance);
}

fn census(
    submissions: &[MaturitySubmissionResources; 2],
    candidates: &[MaturityMeasuredCandidate],
    cases: &[(MaturityResourceCase, MaturityCaseStanding)],
) -> MaturityAbsenceCensus {
    let mut reasons = BTreeMap::new();
    for resources in submissions
        .iter()
        .chain(candidates.iter().map(|item| &item.resources))
    {
        for prediction in resources.fields.values().chain(resources.roster.values()) {
            if let MaturityPrediction::Absent(reason) = prediction {
                insert_reason(&mut reasons, prediction_absence_name(*reason));
            }
        }
        for standing in resources.standings.values() {
            match standing {
                MaturityObservationStanding::NotObservableAtThisBoundary(reason) => {
                    insert_reason(&mut reasons, unobserved_name(*reason));
                }
                MaturityObservationStanding::NoObservationInThisRun => {
                    insert_reason(&mut reasons, "NoObservationInThisRun");
                }
                _ => {}
            }
        }
    }
    for (_, standing) in cases {
        match standing {
            MaturityCaseStanding::SponsoredFormRefusedAtConstruction => {
                insert_reason(&mut reasons, "SponsoredFormRefusedAtConstruction");
            }
            MaturityCaseStanding::NoSponsorShapeInThisGeneration => {
                insert_reason(&mut reasons, "NoSponsorShapeInThisGeneration");
            }
            _ => {}
        }
    }
    insert_reason(&mut reasons, "NoDurationIsArchived");
    MaturityAbsenceCensus { reasons }
}

fn sidecar_status(
    submission: MaturitySubmission,
    contents: &'static str,
) -> Result<&'static str, MaturityResourceReportRefusal> {
    let mut lines = contents.lines();
    if lines.next() != Some("timing-schema 1") || lines.next() != Some("ceremony-id report") {
        return Err(MaturityResourceReportRefusal::MalformedArchivedTimingSidecar(submission));
    }
    let status = lines.next().and_then(|line| line.strip_prefix("status "));
    if lines.next().is_some() || status != Some("passed") {
        return Err(MaturityResourceReportRefusal::MalformedArchivedTimingSidecar(submission));
    }
    Ok("passed")
}

/// Assemble the archived comparison, six cases' candidates and eight derivations.
///
/// The caller supplies only host construction time; the two archive sidecars carry status but no duration.
///
/// # Errors
/// Returns a typed archive, measurement, corpus or sidecar refusal.
pub fn assemble_maturity_resource_report(
    diagnostics_time: Option<Duration>,
) -> Result<
    (CandidateMaturityResourceReport, MaturityResourceDiagnostics),
    MaturityResourceReportRefusal,
> {
    let submissions = archived_submission_resources().map_err(|error| {
        MaturityResourceReportRefusal::ArchivedResourcesUnavailable(format!("{error:?}"))
    })?;
    let candidates = measure_maturity_cases().map_err(|error| {
        MaturityResourceReportRefusal::MeasurementsUnavailable(format!("{error:?}"))
    })?;
    let nonce_corpus = maturity_nonce_corpus().map_err(|error| {
        MaturityResourceReportRefusal::NonceCorpusUnavailable(format!("{error:?}"))
    })?;
    let cases = MaturityResourceCase::ALL
        .iter()
        .map(|&case| (case, case.standing()))
        .collect::<Vec<_>>();
    let absences = census(&submissions, &candidates, &cases);
    let nonce_attempts = nonce_corpus
        .derivations
        .iter()
        .map(|entry| (entry.submission, entry.side, entry.lead, entry.attempts))
        .collect();
    let archive_sidecars = [
        (
            MaturitySubmission::HistoricalWholeMetadata,
            "3a690139.report.capture.timing",
            sidecar_status(
                MaturitySubmission::HistoricalWholeMetadata,
                include_str!("../fixtures/maturity-run-of-record/3a690139.report.capture.timing"),
            )?,
            "NoDurationIsArchived",
        ),
        (
            MaturitySubmission::AcceptedVariableMetadata,
            "cae3bd7c.report.capture.timing",
            sidecar_status(
                MaturitySubmission::AcceptedVariableMetadata,
                include_str!(
                    "../fixtures/maturity-variable-run-of-record/cae3bd7c.report.capture.timing"
                ),
            )?,
            "NoDurationIsArchived",
        ),
    ];
    Ok((
        CandidateMaturityResourceReport {
            schema: MATURITY_RESOURCE_REPORT_SCHEMA,
            role: MaturityResourceReportRole::MaturityResource,
            submissions,
            candidates,
            cases,
            nonce_corpus,
            result: MaturityResourceResult::TargetObservedWeightAgreesForTheArchivedSubmissions,
            calibration: MaturityCalibrationResult::NoCandidateParameterIsFinalCalibration,
            absences,
        },
        MaturityResourceDiagnostics {
            construction_time: diagnostics_time,
            nonce_attempts,
            archive_sidecars,
        },
    ))
}

fn first_failure(resources: &MaturitySubmissionResources) -> Option<MaturityResourceReportRefusal> {
    resources.failures().first().map(|failure| match failure {
        MaturityResourceFailure::PredictionDisagreesWithObservation {
            submission,
            field,
            predicted,
            observed,
        } => MaturityResourceReportRefusal::PredictionDisagreesWithObservation {
            submission: *submission,
            field: *field,
            predicted: *predicted,
            observed: *observed,
        },
    })
}

/// Recompute the comparison and refuse any mismatch before a result is rendered.
///
/// # Errors
/// Returns the first typed mismatch, then any coverage, absence or canonical-key refusal.
pub fn validate_maturity_resource_report(
    report: &CandidateMaturityResourceReport,
) -> Result<ValidatedCandidateMaturityResourceReport, MaturityResourceReportRefusal> {
    for submission in &report.submissions {
        if let Some(failure) = first_failure(submission) {
            return Err(failure);
        }
    }
    let expected = archived_submission_resources().map_err(|error| {
        MaturityResourceReportRefusal::ArchivedResourcesUnavailable(format!("{error:?}"))
    })?;
    for submission in &expected {
        if let Some(failure) = first_failure(submission) {
            return Err(failure);
        }
    }
    for (submitted, rebuilt) in report.submissions.iter().zip(&expected) {
        for (&field, original) in &rebuilt.fields {
            if matches!(original, MaturityPrediction::Absent(_))
                && matches!(
                    submitted.fields.get(&field),
                    Some(MaturityPrediction::Figure(_))
                )
            {
                return Err(
                    MaturityResourceReportRefusal::AbsentFigureRenderedAsNumeral {
                        submission: submitted.submission,
                        field,
                    },
                );
            }
        }
    }
    if report.schema != MATURITY_RESOURCE_REPORT_SCHEMA {
        return Err(MaturityResourceReportRefusal::UnsupportedSchema(
            report.schema,
        ));
    }
    if report.submissions != expected {
        return Err(MaturityResourceReportRefusal::SubmissionsDiffer);
    }
    if report.cases.len() != 14 {
        return Err(MaturityResourceReportRefusal::WrongCaseCount(
            report.cases.len(),
        ));
    }
    if report.nonce_corpus.derivations.len() != 8 {
        return Err(MaturityResourceReportRefusal::WrongNonceCorpusSize(
            report.nonce_corpus.derivations.len(),
        ));
    }
    let candidates = measure_maturity_cases().map_err(|error| {
        MaturityResourceReportRefusal::MeasurementsUnavailable(format!("{error:?}"))
    })?;
    if report.candidates != candidates {
        return Err(MaturityResourceReportRefusal::CandidatesDiffer);
    }
    let cases = MaturityResourceCase::ALL
        .iter()
        .map(|&case| (case, case.standing()))
        .collect::<Vec<_>>();
    if report.cases != cases {
        return Err(MaturityResourceReportRefusal::CasesDiffer);
    }
    let corpus = maturity_nonce_corpus().map_err(|error| {
        MaturityResourceReportRefusal::NonceCorpusUnavailable(format!("{error:?}"))
    })?;
    if report.nonce_corpus != corpus {
        return Err(MaturityResourceReportRefusal::NonceCorpusDiffers);
    }
    if report.role != MaturityResourceReportRole::MaturityResource
        || report.result
            != MaturityResourceResult::TargetObservedWeightAgreesForTheArchivedSubmissions
        || report.calibration != MaturityCalibrationResult::NoCandidateParameterIsFinalCalibration
    {
        return Err(MaturityResourceReportRefusal::DispositionDiffers);
    }
    if report.absences != census(&expected, &candidates, &cases) {
        return Err(MaturityResourceReportRefusal::AbsenceCensusDiffers);
    }
    let validated = ValidatedCandidateMaturityResourceReport {
        report: report.clone(),
    };
    if let Some(key) = forbidden_key_in(&render_maturity_resource_report(&validated)) {
        return Err(MaturityResourceReportRefusal::CanonicalBytesPublishAForbiddenKey(key));
    }
    Ok(validated)
}

fn write_prediction(text: &mut String, prediction: &MaturityPrediction) {
    match prediction {
        MaturityPrediction::Figure(value) => {
            let _ = write!(text, "{value}");
        }
        MaturityPrediction::ByPosition(widths) => {
            for (index, (position, width)) in widths.iter().enumerate() {
                if index != 0 {
                    let _ = write!(text, ",");
                }
                let _ = write!(text, "{position:?}:{width}");
            }
        }
        MaturityPrediction::ZeroByTheReviewedContract => {
            let _ = write!(text, "0");
        }
        MaturityPrediction::NoncanonicalDiagnostic => {
            let _ = write!(text, "TheFieldIsANoncanonicalDiagnostic");
        }
        MaturityPrediction::Absent(reason) => {
            let _ = write!(text, "{}", prediction_absence_name(*reason));
        }
    }
}

fn write_standing(text: &mut String, standing: MaturityObservationStanding) {
    match standing {
        MaturityObservationStanding::Agree { .. } => {
            let _ = write!(text, "Agree");
        }
        MaturityObservationStanding::Mismatch { .. } => {
            let _ = write!(text, "Mismatch");
        }
        MaturityObservationStanding::Layer(layer) => {
            let _ = write!(text, "Layer:{layer:?}");
        }
        MaturityObservationStanding::NotObservableAtThisBoundary(reason) => {
            let _ = write!(
                text,
                "NotObservableAtThisBoundary:{}",
                unobserved_name(reason)
            );
        }
        MaturityObservationStanding::NoObservationInThisRun => {
            let _ = write!(text, "NoObservationInThisRun");
        }
    }
}

/// Render canonical bytes from a validated report alone.
#[must_use]
pub fn render_maturity_resource_report(
    validated: &ValidatedCandidateMaturityResourceReport,
) -> String {
    let report = &validated.report;
    let mut text = String::new();
    let _ = writeln!(text, "schema {}", report.schema);
    let _ = writeln!(text, "role {}", report.role.name());
    for resources in &report.submissions {
        for field in MaturityResourceField::ALL {
            let _ = write!(
                text,
                "field {} {} ",
                submission_name(resources.submission),
                field.name()
            );
            if let Some(prediction) = resources.fields.get(field) {
                write_prediction(&mut text, prediction);
            }
            let _ = write!(text, " ");
            if let Some(standing) = resources.standings.get(field) {
                write_standing(&mut text, *standing);
            }
            let _ = writeln!(text);
        }
        for (dimension, prediction) in &resources.roster {
            let _ = write!(
                text,
                "roster {} {} ",
                submission_name(resources.submission),
                dimension_name(*dimension)
            );
            write_prediction(&mut text, prediction);
            let _ = writeln!(
                text,
                " {}",
                if matches!(prediction, MaturityPrediction::ZeroByTheReviewedContract) {
                    "ZeroByTheReviewedContract"
                } else {
                    "Predicted"
                }
            );
        }
    }
    for candidate in &report.candidates {
        let _ = writeln!(
            text,
            "candidate {} {:?} cycle {} bytes {} observation NoObservationInThisRun",
            submission_name(candidate.submission),
            candidate.lead,
            candidate.announced_cycle.get(),
            candidate.bytes.len(),
        );
    }
    for (case, standing) in &report.cases {
        let _ = writeln!(text, "case {} {standing:?}", case.name());
    }
    let _ = writeln!(
        text,
        "nonce-budget {}",
        report.nonce_corpus.budget.attempts()
    );
    for derivation in &report.nonce_corpus.derivations {
        let _ = writeln!(
            text,
            "nonce-derivation {} {:?} {:?} selected {}",
            submission_name(derivation.submission),
            derivation.side,
            derivation.lead,
            derivation.selected.get(),
        );
    }
    let _ = writeln!(text, "result {}", report.result.sentence());
    let _ = writeln!(text, "calibration {}", report.calibration.sentence());
    for parameter in MaturityCandidateParameter::ALL {
        let _ = writeln!(
            text,
            "parameter {} {}",
            parameter.name(),
            parameter.present_value()
        );
    }
    for (reason, provenance) in report.absences.reasons() {
        let _ = writeln!(text, "absence {reason} {provenance}");
    }
    text
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::*;

    fn assembled() -> &'static (CandidateMaturityResourceReport, MaturityResourceDiagnostics) {
        static CACHED: OnceLock<(CandidateMaturityResourceReport, MaturityResourceDiagnostics)> =
            OnceLock::new();
        CACHED.get_or_init(|| {
            assemble_maturity_resource_report(Some(Duration::from_millis(424_242)))
                .expect("the admitted archives and six candidates assemble")
        })
    }

    fn validated() -> ValidatedCandidateMaturityResourceReport {
        validate_maturity_resource_report(&assembled().0)
            .expect("the assembled comparison and report validate")
    }

    #[test]
    fn the_report_states_one_result_and_its_non_calibration() {
        let validated = validated();
        let report = validated.report();
        let rendered = render_maturity_resource_report(&validated);
        assert_eq!(rendered.lines().next(), Some("schema 1"));
        assert_eq!(rendered.lines().nth(1), Some("role maturity-resource"));
        assert_eq!(report.submissions.len(), 2);
        assert_eq!(report.submissions[0].agreements(), 1);
        assert_eq!(report.submissions[1].agreements(), 1);
        assert_eq!(
            report.result,
            MaturityResourceResult::TargetObservedWeightAgreesForTheArchivedSubmissions
        );
        assert_eq!(
            report.result.sentence(),
            "Target-observed transaction weight agrees with the host prediction for the historical whole-metadata and accepted variable-metadata archived submissions."
        );
        assert_eq!(
            report.calibration.sentence(),
            "No candidate parameter is final calibration"
        );
        assert_eq!(MaturityCandidateParameter::ALL.len(), 7);
        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("parameter "))
                .collect::<Vec<_>>(),
            MaturityCandidateParameter::ALL
                .iter()
                .map(|parameter| {
                    let line = rendered
                        .lines()
                        .find(|line| {
                            line.starts_with(format!("parameter {} ", parameter.name()).as_str())
                        })
                        .expect("each candidate parameter has a canonical line");
                    assert_eq!(
                        line,
                        format!(
                            "parameter {} {}",
                            parameter.name(),
                            parameter.present_value()
                        )
                    );
                    line
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(report.candidates.len(), 6);
        assert_eq!(report.cases.len(), 14);
    }

    #[test]
    fn a_mismatch_fails_validation_before_any_result() {
        let mut report = assembled().0.clone();
        report.submissions[0].standings.insert(
            MaturityResourceField::TransactionWeight,
            MaturityObservationStanding::Mismatch {
                predicted: 2_084,
                observed: 2_085,
            },
        );
        assert_eq!(
            validate_maturity_resource_report(&report),
            Err(
                MaturityResourceReportRefusal::PredictionDisagreesWithObservation {
                    submission: MaturitySubmission::HistoricalWholeMetadata,
                    field: MaturityResourceField::TransactionWeight,
                    predicted: 2_084,
                    observed: 2_085,
                }
            )
        );
    }

    #[test]
    fn no_absent_figure_renders_as_zero() {
        let validated = validated();
        let rendered = render_maturity_resource_report(&validated);
        let zero_lines = rendered
            .lines()
            .filter(|line| {
                let parts = line.split_whitespace().collect::<Vec<_>>();
                matches!(parts.first().copied(), Some("field" | "roster"))
                    && parts.get(3) == Some(&"0")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            zero_lines,
            [
                "field historical-whole-metadata peak-alternate-stack 0 NotObservableAtThisBoundary:TheNodeExposesNoInterpreterAccounting",
                "roster historical-whole-metadata operation-cost 0 ZeroByTheReviewedContract",
                "field accepted-variable-metadata peak-alternate-stack 0 NotObservableAtThisBoundary:TheNodeExposesNoInterpreterAccounting",
                "roster accepted-variable-metadata operation-cost 0 ZeroByTheReviewedContract",
            ]
        );
        for submission in &validated.report().submissions {
            for (field, prediction) in &submission.fields {
                if let MaturityPrediction::Absent(reason) = prediction {
                    let prefix = format!(
                        "field {} {} ",
                        submission_name(submission.submission),
                        field.name()
                    );
                    let line = rendered
                        .lines()
                        .find(|line| line.starts_with(&prefix))
                        .expect("every resource field has one line");
                    assert_eq!(
                        line.split_whitespace().nth(3),
                        Some(prediction_absence_name(*reason))
                    );
                    assert_eq!(
                        line.split_whitespace()
                            .nth(3)
                            .and_then(|value| value.parse::<u64>().ok()),
                        None
                    );
                }
            }
            for (dimension, prediction) in &submission.roster {
                if let MaturityPrediction::Absent(reason) = prediction {
                    let prefix = format!(
                        "roster {} {} ",
                        submission_name(submission.submission),
                        dimension_name(*dimension)
                    );
                    let line = rendered
                        .lines()
                        .find(|line| line.starts_with(&prefix))
                        .expect("every roster dimension has one line");
                    assert_eq!(
                        line.split_whitespace().nth(3),
                        Some(prediction_absence_name(*reason))
                    );
                    assert_eq!(
                        line.split_whitespace()
                            .nth(3)
                            .and_then(|value| value.parse::<u64>().ok()),
                        None
                    );
                }
            }
        }
        let mut report = assembled().0.clone();
        report.submissions[0].fields.insert(
            MaturityResourceField::ConstructorBytes,
            MaturityPrediction::Figure(0),
        );
        assert_eq!(
            validate_maturity_resource_report(&report),
            Err(
                MaturityResourceReportRefusal::AbsentFigureRenderedAsNumeral {
                    submission: MaturitySubmission::HistoricalWholeMetadata,
                    field: MaturityResourceField::ConstructorBytes,
                }
            )
        );
    }

    #[test]
    fn the_canonical_bytes_carry_no_timing_or_forbidden_key() {
        let validated = validated();
        let canonical = render_maturity_resource_report(&validated);
        let diagnostics = assembled().1.render();
        assert_eq!(forbidden_key_in(&canonical), None);
        assert_eq!(
            diagnostics.lines().next(),
            Some("role maturity-resource-diagnostics")
        );
        assert_eq!(diagnostics.lines().nth(1), Some("canonical false"));
        assert!(diagnostics.contains("host-construction-ms 424242"));
        assert!(!canonical.contains("424242"));
        assert_eq!(
            diagnostics
                .lines()
                .filter(|line| line.starts_with("nonce-attempts "))
                .count(),
            8
        );
        assert_eq!(
            canonical
                .lines()
                .filter(|line| line.starts_with("nonce-attempts "))
                .count(),
            0
        );
        assert_eq!(
            diagnostics
                .lines()
                .filter(|line| line.starts_with("archive-sidecar "))
                .count(),
            2
        );
        assert!(diagnostics.contains("NoDurationIsArchived"));
        assert!(canonical.contains("NoDurationIsArchived"));
    }

    #[test]
    fn every_absence_names_a_live_reason_and_no_stale_blocker() {
        let census = assembled().0.absences.reasons();
        assert_eq!(
            census.keys().copied().collect::<Vec<_>>(),
            [
                "NoDurationIsArchived",
                "NoEmitterTotalsIt",
                "NoObservationInThisRun",
                "NoPackageInOneTransaction",
                "NoSponsorShapeInThisGeneration",
                "SponsoredFormRefusedAtConstruction",
                "TheFieldIsANoncanonicalDiagnostic",
                "TheFieldIsATargetVerdict",
                "TheFigureIsNotATargetObservation",
                "TheNodeExposesNoInterpreterAccounting",
            ]
        );
        for (reason, provenance) in census {
            assert_eq!(
                *provenance,
                if *reason == "TheNodeExposesNoInterpreterAccounting" {
                    "upstream obs:upstream:eg-011"
                } else {
                    "lane-local"
                }
            );
        }
        let sources = [
            include_str!("maturity_resource_report.rs"),
            include_str!("maturity_resources.rs"),
            include_str!("maturity_measurements.rs"),
            include_str!("maturity_nonce_corpus.rs"),
        ];
        let stale = [
            concat!("LiveInfrastructure", "Blocker"),
            concat!("OwnerSighash", "NotComputable"),
            concat!("NoAcceptingControl", "Exists"),
        ];
        for source in sources {
            for blocker in stale {
                assert!(!source.contains(blocker));
            }
        }
    }

    #[test]
    fn the_canonical_bytes_are_stable_across_renderings() {
        let validated = validated();
        let first = render_maturity_resource_report(&validated);
        let second = render_maturity_resource_report(&validated);
        assert_eq!(first, second);
    }

    #[test]
    fn independent_assemblies_render_identical_bytes() {
        let (first, _) = assemble_maturity_resource_report(Some(Duration::from_millis(424_242)))
            .expect("first assembly");
        let (second, _) = assemble_maturity_resource_report(Some(Duration::from_millis(424_242)))
            .expect("second assembly");

        let first = validate_maturity_resource_report(&first).expect("first report validates");
        let second = validate_maturity_resource_report(&second).expect("second report validates");

        let first = render_maturity_resource_report(&first);
        let second = render_maturity_resource_report(&second);

        assert_eq!(first, second);
        assert_eq!(first.lines().next(), Some("schema 1"));
    }
}
