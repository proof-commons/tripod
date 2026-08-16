//! What the compound-prototype report validator and gate refuse.
//!
//! Every test here is a way a description of a run could be offered in
//! place of the run: a row deleted, a status relabelled, a claim
//! invented, a summary edited, a matrix filed under the other relation's
//! role. The validator recomputes rather than trusts, so each one is a
//! typed refusal rather than a report that reads green.

use std::collections::BTreeMap;

use crate::error::NativeConformanceError;
use crate::executor::{ExecutionTranscript, ExecutorTrust};
use crate::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativePrototypeResponse, NativeResourceObservation, NativeVerdict,
    ObservedFailureClass,
};
use crate::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeCaseId, PrototypeRelation,
    constructor_case_matrix, wide_floor_case_matrix,
};
use crate::prototype_report::{PrototypeConformanceReport, PrototypeReportCompleteness};
use crate::prototype_validate::{
    PrototypeReportValidationInputs, evaluate_prototypes, prototype_gate, validate_prototype_report,
};
use crate::report::{CaseStatus, EvidenceDisposition, PrototypeReportRole};

/// The wide-floor matrix, which every test here runs.
///
/// Chosen because every one of its rows is answerable from the fixture
/// alone: a test that had to reason about which constructor rows accept
/// would be testing the matrix rather than the validator.
fn matrix(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
) -> Vec<CompoundPrototypeFixture> {
    wide_floor_case_matrix(target).expect("the wide-floor matrix is authored")
}

/// A transcript in which the executor answered every row exactly as its
/// fixture requires.
///
/// This is what a run against an honest target looks like. It is not
/// evidence of anything about a target — it is assembled here — and the
/// tests below use it only as the fixed input the validator recomputes
/// from.
fn agreeing_transcript(
    matrix: &[CompoundPrototypeFixture],
    trust: ExecutorTrust,
) -> ExecutionTranscript {
    let mut responses: BTreeMap<PrototypeCaseId, NativePrototypeResponse> = BTreeMap::new();
    for fixture in matrix {
        let (verdict, observed_failure) = match fixture.expected {
            ExpectedPrototypeOutcome::Accepted => (NativeVerdict::Accepted, None),
            ExpectedPrototypeOutcome::Rejected => (
                NativeVerdict::Rejected,
                // A compound fixture admits no class set, so any class
                // an executor names is recorded and compared against
                // nothing. One is named here for exactly that reason:
                // the row must pass regardless of which it is.
                Some(ObservedFailureClass::FalseVerification),
            ),
        };
        responses.insert(
            fixture.case.clone(),
            NativePrototypeResponse {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case: fixture.case.clone(),
                verdict,
                final_stack: None,
                final_altstack: None,
                observed_failure,
                resources: NativeResourceObservation::default(),
            },
        );
    }
    ExecutionTranscript::prototypes_for_tests(
        crate::tests::support::nonmock_handshake(),
        crate::tests::support::observed_environment(),
        trust,
        responses,
    )
}

/// One evaluated report over the wide-floor matrix.
fn report_over(
    trust: ExecutorTrust,
) -> (
    target_elements::ReviewedElementsTapscriptDefinition,
    Vec<CompoundPrototypeFixture>,
    ExecutionTranscript,
    PrototypeConformanceReport,
) {
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let matrix = matrix(&target);
    let transcript = agreeing_transcript(&matrix, trust);
    let report = evaluate_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &matrix,
        &transcript,
    )
    .expect("an agreeing run evaluates");
    (target, matrix, transcript, report)
}

/// Offers one report back to the validator.
fn revalidate(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
    matrix: &[CompoundPrototypeFixture],
    transcript: &ExecutionTranscript,
    report: PrototypeConformanceReport,
) -> Result<crate::prototype_validate::ValidatedPrototypeReport, NativeConformanceError> {
    let binding = crate::tests::support::development_binding(target);
    validate_prototype_report(
        report,
        PrototypeReportValidationInputs {
            target,
            binding: &binding,
            relation: PrototypeRelation::WideFloorRelation,
            matrix,
            transcript,
        },
    )
}

#[test]
fn an_agreeing_run_is_complete_for_its_own_relation() {
    let (target, matrix, transcript, report) = report_over(ExecutorTrust::ReviewedNonMock);

    assert_eq!(report.role, PrototypeReportRole::WideFloor);
    assert_eq!(report.relation, PrototypeRelation::WideFloorRelation);
    assert_eq!(report.cases.len(), matrix.len());
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::CompleteForWideFloorPrototype,
    );
    assert_eq!(report.summary.cases_failed, 0);
    assert_eq!(report.summary.claims_unresolved, 0);
    assert_eq!(
        report.summary.required_claims_passed,
        report.summary.required_claims_total,
    );
    assert!(
        report
            .claims
            .iter()
            .all(|row| row.relation == PrototypeRelation::WideFloorRelation
                && row.disposition == EvidenceDisposition::Passed),
        "every wide-floor claim is answered by the run",
    );

    let validated =
        revalidate(&target, &matrix, &transcript, report).expect("its own report validates");
    prototype_gate(&validated).expect("a complete nonmock run is evidence");
}

#[test]
fn the_report_bytes_are_a_function_of_the_run() {
    // Determinism, checked as bytes rather than as typed equality: the
    // report is compared by both, and a collection ordered by insertion
    // somewhere inside would satisfy one and not the other
    // (Guide-10 `rule:guide10:report-determinism`).
    let (_target, _matrix, _transcript, first) = report_over(ExecutorTrust::ReviewedNonMock);
    let (_target, _matrix, _transcript, second) = report_over(ExecutorTrust::ReviewedNonMock);
    assert_eq!(
        serde_json::to_vec(&first).expect("a report serializes"),
        serde_json::to_vec(&second).expect("a report serializes"),
    );
}

#[test]
fn a_declared_mock_run_cannot_satisfy_the_prototype_gate() {
    let (target, matrix, transcript, report) = report_over(ExecutorTrust::Mock);
    let validated =
        revalidate(&target, &matrix, &transcript, report).expect("a mock report still validates");
    assert!(
        matches!(
            prototype_gate(&validated),
            Err(NativeConformanceError::MockExecutorCannotSatisfyNativeGate),
        ),
        "a mock's answers are the matrix's own expectations read back",
    );
}

#[test]
fn a_run_of_no_cases_is_not_a_passing_run() {
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let empty: Vec<CompoundPrototypeFixture> = Vec::new();
    let transcript = agreeing_transcript(&empty, ExecutorTrust::ReviewedNonMock);
    let report = evaluate_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &empty,
        &transcript,
    )
    .expect("an empty run evaluates");

    // Every claim is required and none has a bearing case, so the
    // completeness is a failure before the gate is ever asked.
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::Failed,
    );
    let validated = validate_prototype_report(
        report,
        PrototypeReportValidationInputs {
            target: &target,
            binding: &binding,
            relation: PrototypeRelation::WideFloorRelation,
            matrix: &empty,
            transcript: &transcript,
        },
    )
    .expect("an empty report validates against its own empty matrix");
    assert!(matches!(
        prototype_gate(&validated),
        Err(NativeConformanceError::RequiredPrototypeClaimMissing(_)
            | NativeConformanceError::EmptyPrototypeMatrix),
    ));
}

#[test]
fn a_matrix_of_the_other_relation_is_refused() {
    // One report answers one relation. A constructor matrix evaluated
    // under the wide-floor relation would file its coverage under the
    // wrong role, where a reader counts it as coverage of a relation
    // nothing established.
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let constructor = constructor_case_matrix(&target).expect("the constructor matrix is authored");
    let transcript = agreeing_transcript(&constructor, ExecutorTrust::ReviewedNonMock);

    assert!(matches!(
        evaluate_prototypes(
            &target,
            &binding,
            PrototypeRelation::WideFloorRelation,
            &constructor,
            &transcript,
        ),
        Err(NativeConformanceError::PrototypeMatrixRelationMismatch { .. }),
    ));
}

#[test]
fn a_role_that_is_not_the_relations_own_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.role = PrototypeReportRole::ConstructorContinuity;
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::PrototypeReportRoleMismatch),
    ));
}

#[test]
fn a_deleted_case_row_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.cases.pop().expect("the matrix has rows");
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::PrototypeReportCaseCensusMismatch),
    ));
}

#[test]
fn a_duplicated_case_row_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    let first = report.cases.first().expect("the matrix has rows").clone();
    report.cases.push(first);
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::DuplicatePrototypeCase(_)),
    ));
}

#[test]
fn a_relabelled_case_status_is_refused() {
    // The status is recomputed from the fixture and the transcript, so
    // editing it is editing a description of a comparison that was made
    // rather than the comparison.
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.cases[0].status = CaseStatus::Failed;
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::PrototypeCaseOutcomeMismatch(_)),
    ));
}

#[test]
fn an_edited_fixture_projection_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.cases[0].fixture.script.push(0x51);
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::PrototypeProjectionMismatch(_)),
    ));
}

#[test]
fn an_edited_observation_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.cases[0].observed.resources.script_bytes += 1;
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::PrototypeCaseOutcomeMismatch(_)),
    ));
}

#[test]
fn a_cleared_claim_census_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.claims.clear();
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::MissingPrototypeClaim(_)),
    ));
}

#[test]
fn a_duplicated_claim_row_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    let first = report
        .claims
        .first()
        .expect("the relation has claims")
        .clone();
    report.claims.push(first);
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::DuplicatePrototypeClaim(_)),
    ));
}

#[test]
fn a_relabelled_claim_disposition_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.claims[0].required = false;
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::PrototypeReportClaimCensusMismatch),
    ));
}

#[test]
fn an_edited_summary_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.summary.cases_passed += 1;
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::PrototypeReportSummaryMismatch),
    ));
}

#[test]
fn a_report_of_an_unvalidatable_schema_is_refused() {
    let (target, matrix, transcript, mut report) = report_over(ExecutorTrust::ReviewedNonMock);
    report.schema += 1;
    assert!(matches!(
        revalidate(&target, &matrix, &transcript, report),
        Err(NativeConformanceError::UnsupportedReportSchema { .. }),
    ));
}

#[test]
fn a_disagreeing_verdict_fails_its_case_and_its_claims() {
    // The comparison compares. An executor that accepted a spend the
    // matrix requires rejected has not produced weak evidence: it has
    // produced a finding, and the report says so on the row, on every
    // claim the row bears on, and in the completeness.
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let matrix = matrix(&target);
    let refusing = matrix
        .iter()
        .position(|fixture| fixture.expected == ExpectedPrototypeOutcome::Rejected)
        .expect("the matrix has refusing rows");

    let mut responses: BTreeMap<PrototypeCaseId, NativePrototypeResponse> = BTreeMap::new();
    for (index, fixture) in matrix.iter().enumerate() {
        let accepted = index == refusing || fixture.expected == ExpectedPrototypeOutcome::Accepted;
        responses.insert(
            fixture.case.clone(),
            NativePrototypeResponse {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case: fixture.case.clone(),
                verdict: if accepted {
                    NativeVerdict::Accepted
                } else {
                    NativeVerdict::Rejected
                },
                final_stack: None,
                final_altstack: None,
                observed_failure: if accepted {
                    None
                } else {
                    Some(ObservedFailureClass::FalseVerification)
                },
                resources: NativeResourceObservation::default(),
            },
        );
    }
    let transcript = ExecutionTranscript::prototypes_for_tests(
        crate::tests::support::nonmock_handshake(),
        crate::tests::support::observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        responses,
    );

    let report = evaluate_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &matrix,
        &transcript,
    )
    .expect("a disagreeing run still evaluates");

    assert_eq!(report.summary.cases_failed, 1);
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::Failed,
    );
    let failed = &matrix[refusing].case;
    assert!(
        report
            .claims
            .iter()
            .any(|row| row.bearing_cases.contains(failed)
                && row.disposition == EvidenceDisposition::Failed),
        "the claims the failing row bears on fail with it",
    );

    let validated = validate_prototype_report(
        report,
        PrototypeReportValidationInputs {
            target: &target,
            binding: &binding,
            relation: PrototypeRelation::WideFloorRelation,
            matrix: &matrix,
            transcript: &transcript,
        },
    )
    .expect("a report of a failing run is still an exact report");
    assert!(matches!(
        prototype_gate(&validated),
        Err(NativeConformanceError::RequiredPrototypeClaimFailed(_)),
    ));
}

#[test]
fn an_executor_that_could_not_run_a_case_is_not_a_rejection() {
    // Infrastructure trouble is never a target verdict. A row the
    // executor could not run is recorded as trouble, and the gate
    // refuses it as trouble rather than counting it as a refusal the
    // fixture happened to want.
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let matrix = matrix(&target);

    let mut responses: BTreeMap<PrototypeCaseId, NativePrototypeResponse> = BTreeMap::new();
    for (index, fixture) in matrix.iter().enumerate() {
        let broken = index == 0;
        responses.insert(
            fixture.case.clone(),
            NativePrototypeResponse {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case: fixture.case.clone(),
                verdict: if broken {
                    NativeVerdict::InfrastructureError
                } else {
                    match fixture.expected {
                        ExpectedPrototypeOutcome::Accepted => NativeVerdict::Accepted,
                        ExpectedPrototypeOutcome::Rejected => NativeVerdict::Rejected,
                    }
                },
                final_stack: None,
                final_altstack: None,
                observed_failure: if !broken
                    && fixture.expected == ExpectedPrototypeOutcome::Rejected
                {
                    Some(ObservedFailureClass::FalseVerification)
                } else {
                    None
                },
                resources: NativeResourceObservation::default(),
            },
        );
    }
    let transcript = ExecutionTranscript::prototypes_for_tests(
        crate::tests::support::nonmock_handshake(),
        crate::tests::support::observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        responses,
    );

    let report = evaluate_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &matrix,
        &transcript,
    )
    .expect("a troubled run still evaluates");
    assert_eq!(report.summary.cases_infrastructure_error, 1);
    assert_eq!(report.summary.cases_failed, 0);
    assert_eq!(
        report.cases[0].status,
        CaseStatus::InfrastructureError,
        "trouble is its own status and not a failure",
    );
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::Failed,
    );
}
