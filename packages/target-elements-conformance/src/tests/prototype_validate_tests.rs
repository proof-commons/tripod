//! What the compound-prototype report validator and gate refuse.
//!
//! Every test here is a way a description of a run could be offered in
//! place of the run: a row deleted, a status relabelled, a claim
//! invented, a summary edited, a matrix filed under the other relation's
//! role. The validator recomputes rather than trusts, so each one is a
//! typed refusal rather than a report that reads green.

use std::collections::BTreeMap;

use crate::error::NativeConformanceError;
use crate::executor::{ExecutionTranscript, ExecutorTrust, PrototypeTranscriptParts};
use crate::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativePrototypeResponse, NativeResourceObservation, NativeVerdict,
    ObservedFailureClass,
};
use crate::prototype::{
    CanonicalPrototypeMatrix, CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeCaseId,
    PrototypeRelation, WideFloorPrototypeMatrix, constructor_case_matrix, wide_floor_case_matrix,
};
use crate::prototype_report::{PrototypeConformanceReport, PrototypeReportCompleteness};
use crate::prototype_validate::{
    PrototypeReportValidationInputs, evaluate_experimental_prototypes, evaluate_prototypes,
    prototype_gate, validate_prototype_report,
};
use crate::report::{CaseStatus, EvidenceDisposition, PrototypeReportRole};

/// The wide-floor matrix, which every test here runs.
///
/// Chosen because every one of its rows is answerable from the fixture
/// alone: a test that had to reason about which constructor rows accept
/// would be testing the matrix rather than the validator.
fn matrix(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
) -> WideFloorPrototypeMatrix {
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
    target: &target_elements::ReviewedElementsTapscriptDefinition,
    binding: &target_elements::ReviewedDevelopmentBinding,
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
    ExecutionTranscript::prototypes_for_tests(PrototypeTranscriptParts {
        target,
        binding,
        handshake: crate::tests::support::nonmock_handshake(),
        environment: crate::tests::support::observed_environment(),
        trust,
        requests: crate::tests::support::prototype_subjects_of(matrix),
        responses,
    })
}

/// One evaluated report over the wide-floor matrix.
fn report_over(
    trust: ExecutorTrust,
) -> (
    target_elements::ReviewedElementsTapscriptDefinition,
    WideFloorPrototypeMatrix,
    ExecutionTranscript,
    PrototypeConformanceReport,
) {
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let matrix = matrix(&target);
    let transcript = agreeing_transcript(&target, &binding, matrix.rows(), trust);
    let report = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&matrix),
        &transcript,
    )
    .expect("an agreeing run evaluates");
    (target, matrix, transcript, report)
}

/// Offers one report back to the validator.
fn revalidate(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
    matrix: &WideFloorPrototypeMatrix,
    transcript: &ExecutionTranscript,
    report: PrototypeConformanceReport,
) -> Result<crate::prototype_validate::ValidatedPrototypeReport, NativeConformanceError> {
    let binding = crate::tests::support::development_binding(target);
    validate_prototype_report(
        report,
        PrototypeReportValidationInputs {
            target,
            binding: &binding,
            matrix: CanonicalPrototypeMatrix::WideFloor(matrix),
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
    prototype_gate(
        &validated,
        Some(&crate::tests::support::expected_provenance()),
    )
    .expect("a complete nonmock run is evidence");
}

#[test]
fn the_report_bytes_are_a_function_of_the_run() {
    // Determinism, checked as bytes rather than as typed equality: the
    // report is compared by both, and a collection ordered by insertion
    // somewhere inside would satisfy one and not the other
    // (´[PLAN-rule:guide10:report-determinism]´).
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
            prototype_gate(
                &validated,
                Some(&crate::tests::support::expected_provenance())
            ),
            Err(NativeConformanceError::MockExecutorCannotSatisfyNativeGate),
        ),
        "a mock's answers come from the matrix's own expectations",
    );
}

#[test]
fn a_run_of_no_cases_is_not_a_passing_run() {
    // Two refusals stand between an empty run and the gate, and this
    // asserts both. The canonical comparison refuses an empty matrix as a
    // subject, because the relation's canonical matrix is not empty; and
    // the report an empty run produces is failed on its own terms, every
    // required claim lacking a bearing case. The gate's own empty-matrix
    // guard sits behind both and is now unreachable from here, which is
    // the canonical-subject repair doing its work rather than the guard
    // becoming unnecessary.
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let empty: Vec<CompoundPrototypeFixture> = Vec::new();
    let transcript = agreeing_transcript(&target, &binding, &empty, ExecutorTrust::ReviewedNonMock);

    let forged = WideFloorPrototypeMatrix::wrap_for_tests(empty.clone());
    assert!(matches!(
        evaluate_prototypes(
            &target,
            &binding,
            CanonicalPrototypeMatrix::WideFloor(&forged),
            &transcript,
        ),
        Err(NativeConformanceError::NoncanonicalPrototypeMatrix),
    ));

    let report = evaluate_experimental_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &empty,
        &transcript,
    )
    .expect("an empty run evaluates as an experiment")
    .into_report();
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::Failed,
        "a run that executed nothing established nothing",
    );
}

#[test]
fn a_matrix_of_the_other_relation_is_refused() {
    // One report answers one relation. A constructor matrix evaluated
    // under the wide-floor relation would file its coverage under the
    // wrong role, where a reader counts it as coverage of a relation
    // nothing established.
    //
    // On the evidence path this is no longer statable: the relation
    // travels inside the canonical matrix, so there is no second argument
    // to disagree with it. The refusal is still needed on the
    // experimental path, where a caller names the relation, and that is
    // what is asserted here.
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let constructor = constructor_case_matrix(&target).expect("the constructor matrix is authored");
    let transcript = agreeing_transcript(
        &target,
        &binding,
        constructor.rows(),
        ExecutorTrust::ReviewedNonMock,
    );

    assert!(matches!(
        evaluate_experimental_prototypes(
            &target,
            &binding,
            PrototypeRelation::WideFloorRelation,
            constructor.rows(),
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
        .rows()
        .iter()
        .position(|fixture| fixture.expected == ExpectedPrototypeOutcome::Rejected)
        .expect("the matrix has refusing rows");

    let mut responses: BTreeMap<PrototypeCaseId, NativePrototypeResponse> = BTreeMap::new();
    for (index, fixture) in matrix.rows().iter().enumerate() {
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
    let transcript = ExecutionTranscript::prototypes_for_tests(PrototypeTranscriptParts {
        target: &target,
        binding: &binding,
        handshake: crate::tests::support::nonmock_handshake(),
        environment: crate::tests::support::observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: crate::tests::support::prototype_subjects_of(matrix.rows()),
        responses,
    });

    let report = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&matrix),
        &transcript,
    )
    .expect("a disagreeing run still evaluates");

    assert_eq!(report.summary.cases_failed, 1);
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::Failed,
    );
    let failed = &matrix.rows()[refusing].case;
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
            matrix: CanonicalPrototypeMatrix::WideFloor(&matrix),
            transcript: &transcript,
        },
    )
    .expect("a report of a failing run is still an exact report");
    assert!(matches!(
        prototype_gate(
            &validated,
            Some(&crate::tests::support::expected_provenance())
        ),
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
    for (index, fixture) in matrix.rows().iter().enumerate() {
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
    let transcript = ExecutionTranscript::prototypes_for_tests(PrototypeTranscriptParts {
        target: &target,
        binding: &binding,
        handshake: crate::tests::support::nonmock_handshake(),
        environment: crate::tests::support::observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: crate::tests::support::prototype_subjects_of(matrix.rows()),
        responses,
    });

    let report = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&matrix),
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
