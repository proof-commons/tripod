//! Evidence-plan, comparison, claim, report-validation, and gate tests.

use std::collections::BTreeMap;

use target_elements::{
    CapabilityContract, ElementsCapability, StaticCapabilityStatus, TargetEvidenceRequirementId,
};

use crate::claim::{ClaimRegistry, NativeEvidenceClaim, claim_registry};
use crate::error::NativeConformanceError;
use crate::executor::{ExecutionTranscript, ExecutorTrust, TranscriptParts};
use crate::fixture::{
    CanonicalPrimitiveFixtureSet, EnforcementLayer, ExpectedPrimitiveOutcome,
    ExpectedResourceObservation, NativeCaseGroup, NativeCaseId, ResourceExpectation,
    canonical_fixture_set,
};
use crate::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeExecutionResponse, NativeResourceObservation, NativeVerdict,
};
use crate::report::{
    CaseStatus, EvidenceDisposition, EvidencePlanClass, NATIVE_REPORT_SCHEMA,
    NativeConformanceReport, ReportCompleteness,
};
use crate::validate::{
    EvidencePlan, NativeReportValidationInputs, evaluate, gate, guide_nine_evidence_plan,
    resources_agree, validate_native_report,
};

use super::support::{
    development_binding, expected_provenance, nonmock_handshake, observed_environment,
    reviewed_target, subjects_of,
};

/// Everything one run needs, assembled once per test.
struct Run {
    target: target_elements::ReviewedElementsTapscriptDefinition,
    binding: target_elements::ReviewedDevelopmentBinding,
    fixtures: CanonicalPrimitiveFixtureSet,
    plan: EvidencePlan,
    registry: ClaimRegistry,
    transcript: ExecutionTranscript,
}

impl Run {
    /// A run whose executor answered every case as the contract requires.
    ///
    /// The answers are the census's own expectations, which is exactly
    /// what makes this a *harness* test and not evidence: the gate
    /// refuses the mock declaration for the same reason.
    fn passing() -> Self {
        Self::new(ExecutorTrust::ReviewedNonMock, |_case, _status| None)
    }

    /// The same run with one case answered differently.
    fn with_override(
        change: impl Fn(NativeCaseId, &NativeExecutionResponse) -> Option<NativeExecutionResponse>,
    ) -> Self {
        Self::new(ExecutorTrust::ReviewedNonMock, change)
    }

    fn new(
        trust: ExecutorTrust,
        change: impl Fn(NativeCaseId, &NativeExecutionResponse) -> Option<NativeExecutionResponse>,
    ) -> Self {
        let target = reviewed_target();
        let binding = development_binding(&target);
        let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
        let plan = guide_nine_evidence_plan().expect("the plan is a partition");
        let registry = claim_registry().expect("the claim census is coherent");

        let mut responses = BTreeMap::new();
        for fixture in &fixtures {
            let case = fixture.case();
            let contract_answer = expected_answer(case, fixture);
            let answer = change(case, &contract_answer).unwrap_or(contract_answer);
            responses.insert(case, answer);
        }
        let transcript = ExecutionTranscript::for_tests(TranscriptParts {
            target: &target,
            binding: &binding,
            handshake: nonmock_handshake(),
            environment: observed_environment(),
            trust,
            requests: subjects_of(&fixtures),
            responses,
        });

        Self {
            target,
            binding,
            fixtures,
            plan,
            registry,
            transcript,
        }
    }

    fn report(&self) -> NativeConformanceReport {
        evaluate(
            &self.target,
            &self.binding,
            &self.fixtures,
            &self.transcript,
            &self.plan,
            &self.registry,
        )
        .expect("the run evaluates")
    }

    fn inputs(&self) -> NativeReportValidationInputs<'_> {
        NativeReportValidationInputs {
            target: &self.target,
            binding: &self.binding,
            fixtures: &self.fixtures,
            plan: &self.plan,
            registry: &self.registry,
            transcript: &self.transcript,
        }
    }

    /// Validates a possibly-damaged report and returns the refusal.
    fn refuse(&self, report: NativeConformanceReport) -> NativeConformanceError {
        validate_native_report(report, self.inputs()).expect_err("a damaged report must be refused")
    }
}

/// What an honest executor reporting no interpreter stack would answer.
fn expected_answer(
    case: NativeCaseId,
    fixture: &crate::fixture::PrimitiveFixture,
) -> NativeExecutionResponse {
    let resources = NativeResourceObservation {
        script_bytes: Some(fixture.script().len() as u64),
        initial_stack_items: Some(fixture.initial_stack().len() as u64),
        ..NativeResourceObservation::default()
    };
    match fixture.expected() {
        ExpectedPrimitiveOutcome::Accept { .. } => NativeExecutionResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case,
            verdict: NativeVerdict::Accepted,
            final_stack: None,
            final_altstack: None,
            observed_failure: None,
            resources,
        },
        ExpectedPrimitiveOutcome::Reject { classes, .. } => NativeExecutionResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case,
            verdict: NativeVerdict::Rejected,
            final_stack: None,
            final_altstack: None,
            observed_failure: classes.iter().next().copied(),
            resources,
        },
    }
}

#[test]
fn an_exact_resource_expectation_never_agrees_with_absence() {
    let recorded = ResourceExpectation::RecordedOnly;
    let expected = ExpectedResourceObservation {
        script_bytes: ResourceExpectation::Exact(33),
        initial_stack_items: recorded,
        peak_stack_items: recorded,
        peak_altstack_items: recorded,
        maximum_element_bytes: recorded,
        validation_budget_used: recorded,
        transaction_weight: recorded,
    };
    assert!(!resources_agree(
        expected,
        &NativeResourceObservation::default(),
    ));
}

#[test]
fn the_plan_partitions_the_evidence_census_exactly() {
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    for id in TargetEvidenceRequirementId::ALL {
        assert!(
            plan.class(*id).is_some(),
            "every requirement needs a plan class",
        );
    }
    assert_eq!(plan.iter().count(), TargetEvidenceRequirementId::ALL.len());
}

#[test]
fn the_required_plan_is_the_guide_nine_one() {
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::PushEncodingSemantics),
        Some(EvidencePlanClass::Required),
        "the push-encoding row is required, not inherited from opcode semantics",
    );
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::SighashSemantics),
        Some(EvidencePlanClass::UnresolvedByDesign),
    );
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::CommitmentEquality),
        Some(EvidencePlanClass::UnsupportedByStaticContract),
    );
    assert_eq!(
        plan.class(TargetEvidenceRequirementId::ConfidentialValueConservation),
        Some(EvidencePlanClass::DeferredToTransactionEvidence),
    );
}

#[test]
fn the_unsupported_classification_agrees_with_the_reviewed_contract() {
    // The plan calls commitment equality unsupported. That is only
    // honest while the contract says so too, which is what this checks:
    // a contract that later gains a reviewed mechanism must force the
    // plan to be revisited rather than leaving a stale classification.
    let target = reviewed_target();
    let status = target
        .definition()
        .capabilities()
        .get(&ElementsCapability::CommitmentEquality)
        .map(CapabilityContract::status);
    assert_eq!(
        status,
        Some(StaticCapabilityStatus::Unsupported),
        "the plan's unsupported classification must match the contract",
    );
}

#[test]
fn every_fixture_group_bears_on_a_requirement() {
    // A group that bears on nothing would let a case run, pass, and
    // establish no evidence at all.
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    for group in NativeCaseGroup::ALL {
        for layer in [EnforcementLayer::Consensus, EnforcementLayer::RelayPolicy] {
            let requirements = crate::validate::requirements_for_tests(*group, layer);
            assert!(
                !requirements.is_empty(),
                "the {} group bears on no requirement at {layer:?}",
                group.wire_name(),
            );
            for requirement in requirements {
                assert!(
                    plan.class(*requirement).is_some(),
                    "a group bears on a requirement the plan does not classify",
                );
            }
        }
    }
}

/// `G11-R04`: every required row is defined by at least one required
/// claim, so no broad row can pass on case aggregation alone.
#[test]
fn every_required_evidence_row_owns_a_required_claim() {
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    crate::validate::check_required_rows_own_required_claims(&plan, &registry)
        .expect("every required row owns a required claim");
}

/// `G11-R04`: every exception names a required row and states a reason.
///
/// An exception naming a row the plan does not require, or carrying a
/// blank reason, would be an exemption that documents nothing.
#[test]
fn the_claim_decomposition_exceptions_are_stated() {
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let exceptions = crate::validate::claim_decomposition_exceptions_for_tests();
    for exception in exceptions {
        assert_eq!(
            plan.class(exception.requirement),
            Some(EvidencePlanClass::Required),
            "an exception names a row the plan does not require",
        );
        assert!(
            !exception.reason.trim().is_empty(),
            "an exception states no reason",
        );
    }
    // Exactly the rows the census justifies. A fifth exemption arriving
    // silently is the invariant being widened rather than satisfied.
    let exempt: Vec<TargetEvidenceRequirementId> = exceptions
        .iter()
        .map(|exception| exception.requirement)
        .collect();
    assert_eq!(
        exempt,
        vec![
            TargetEvidenceRequirementId::EncodingSemantics,
            TargetEvidenceRequirementId::StackRearrangementSemantics,
            TargetEvidenceRequirementId::ByteStringSemantics,
            TargetEvidenceRequirementId::VerificationSemantics,
        ],
    );

    // And the reasons they state are checkable: each exempt row owns no
    // claim of its own, and the rows its cases are filed under do state
    // required claims.
    let registry = claim_registry().expect("the claim census is coherent");
    for exception in exceptions {
        assert!(
            registry.owned_by(exception.requirement).is_empty(),
            "an exempt row owns claims, so it is not exempt from decomposition",
        );
    }
    for owner in [
        TargetEvidenceRequirementId::InputIntrospectionSemantics,
        TargetEvidenceRequirementId::OutputIntrospectionSemantics,
        TargetEvidenceRequirementId::OpcodeSemantics,
    ] {
        assert!(
            !registry.required_claims(owner).is_empty(),
            "the rows the exempt rows' cases are filed under state required claims",
        );
    }
}

#[test]
fn the_report_schema_is_stated() {
    assert_eq!(NATIVE_REPORT_SCHEMA, 2);
}

#[test]
fn a_schema_one_report_is_not_reinterpreted_as_a_schema_two_report() {
    let run = Run::passing();
    let mut report = run.report();
    report.schema = 1;
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::UnsupportedReportSchema { offered: 1 },
    ));
}

#[test]
fn the_claim_census_is_coherent_and_owned() {
    let registry = claim_registry().expect("the claim census is coherent");
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    assert!(!registry.is_empty());
    for record in registry.iter() {
        assert!(
            plan.class(record.requirement()).is_some(),
            "every claim's owner must be a classified requirement",
        );
        // A claim is unresolved only with a stated reason. An absence
        // nobody explained is how an uncovered corner becomes invisible.
        assert_eq!(
            record.is_required(),
            record.unresolved_reason().is_none(),
            "an unresolved claim states why",
        );
    }
}

#[test]
fn a_complete_run_passes_the_gate_with_its_unresolved_claims_explicit() {
    let run = Run::passing();
    let report = run.report();
    let validated =
        validate_native_report(report, run.inputs()).expect("an unmutated report validates");
    gate(&validated, Some(&expected_provenance())).expect("a complete run is evidence");

    let report = validated.report();
    assert_eq!(report.summary.cases_failed, 0);
    assert_eq!(report.summary.cases_infrastructure_error, 0);
    assert_eq!(
        report.summary.completeness,
        ReportCompleteness::PartialUnresolvedClaims,
        "the honest state of this census is partial, not a clean sweep",
    );
    assert!(
        report.summary.claims_unresolved > 0,
        "the uncovered corners must be enumerable, not inferred from silence",
    );
    // Every case row carries its complete subject rather than an ordinal.
    for row in &report.cases {
        assert_eq!(row.status, CaseStatus::Passed);
        assert_eq!(row.fixture.case, row.case());
        assert_ne!(row.fixture.script, [] as [u8; 0]);
    }
}

#[test]
fn the_named_uncovered_claims_are_unresolved_rather_than_passed() {
    // The four the fifth review named, plus the relay-policy resource
    // row. Each has cases in its family that pass; none of those cases
    // establishes the corner named here.
    let run = Run::passing();
    let report = run.report();
    let unresolved: Vec<NativeEvidenceClaim> = report
        .claims
        .iter()
        .filter(|row| row.disposition == EvidenceDisposition::UnresolvedByDesign)
        .map(|row| row.claim)
        .collect();

    for claim in [
        NativeEvidenceClaim::IssuancePresentObserved,
        NativeEvidenceClaim::ReissuancePresentObserved,
        NativeEvidenceClaim::TransactionSignatureAccepted,
        NativeEvidenceClaim::InputFieldConfidentialForm,
        NativeEvidenceClaim::OutputFieldConfidentialForm,
        NativeEvidenceClaim::PushNonminimalRelayRefused,
        NativeEvidenceClaim::PolicyResourceBoundObserved,
    ] {
        assert!(
            unresolved.contains(&claim),
            "{claim:?} must be unresolved rather than absorbed into a passing row",
        );
    }

    // And the families they sit in did run cases, which is exactly the
    // situation an existential aggregation would have called complete.
    let issuance = report
        .claims
        .iter()
        .find(|row| row.claim == NativeEvidenceClaim::IssuanceAbsentObserved)
        .expect("the issuance-absent claim exists");
    assert_eq!(issuance.disposition, EvidenceDisposition::Passed);
    assert!(!issuance.bearing_cases.is_empty());
}

#[test]
fn an_emptied_evidence_report_is_refused() {
    let run = Run::passing();
    let mut report = run.report();
    report.evidence.clear();
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportEvidenceCensusMismatch,
    ));

    let mut report = run.report();
    report.claims.clear();
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::MissingEvidenceClaim(_),
    ));

    let mut report = run.report();
    report.cases.clear();
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportCaseCensusMismatch,
    ));
}

#[test]
fn a_removed_or_duplicated_row_is_refused() {
    let run = Run::passing();

    let mut report = run.report();
    report.cases.pop();
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportCaseCensusMismatch,
    ));

    let mut report = run.report();
    let first = report.cases[0].clone();
    report.cases.push(first);
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::DuplicateReportCase(_),
    ));

    let mut report = run.report();
    report.evidence.pop();
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportEvidenceCensusMismatch,
    ));

    let mut report = run.report();
    let claim = report.claims[0].clone();
    report.claims.push(claim);
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::DuplicateEvidenceClaim(_),
    ));

    let mut report = run.report();
    report.claims.pop();
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::MissingEvidenceClaim(_),
    ));
}

#[test]
fn relabelling_a_row_or_editing_the_summary_is_refused() {
    let run = Run::passing();

    // A required row moved out of the required plan.
    let mut report = run.report();
    let row = report
        .evidence
        .iter_mut()
        .find(|row| row.plan == EvidencePlanClass::Required)
        .expect("some row is required");
    row.plan = EvidencePlanClass::UnresolvedByDesign;
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportEvidenceCensusMismatch,
    ));

    // A required claim relabelled as one nobody has to establish.
    let mut report = run.report();
    let row = report
        .claims
        .iter_mut()
        .find(|row| row.required)
        .expect("some claim is required");
    row.required = false;
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportClaimCensusMismatch,
    ));

    let mut report = run.report();
    report.summary.cases_passed += 1;
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportSummaryMismatch,
    ));

    let mut report = run.report();
    report.summary.completeness = ReportCompleteness::CompleteForPrimitivePlan;
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportSummaryMismatch,
    ));
}

#[test]
fn a_row_whose_subject_is_not_the_executed_fixture_is_refused() {
    let run = Run::passing();

    for damage in [0_u8, 1, 2, 3] {
        let mut report = run.report();
        let row = &mut report.cases[0];
        match damage {
            0 => row.fixture.script.push(0x51),
            1 => row.fixture.initial_stack.push(vec![0x01]),
            2 => {
                row.fixture.enforcement_layer = crate::fixture::EnforcementLayer::RelayPolicy;
            }
            _ => {
                row.fixture.expected_resources.script_bytes =
                    crate::fixture::ResourceExpectation::RecordedOnly;
            }
        }
        assert!(
            matches!(
                run.refuse(report),
                NativeConformanceError::FixtureProjectionMismatch(_),
            ),
            "damage {damage} must be caught",
        );
    }

    // And a row whose claims were widened beyond what the case bears on.
    let mut report = run.report();
    report.cases[0]
        .claims
        .insert(NativeEvidenceClaim::TransactionSignatureAccepted);
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::FixtureProjectionMismatch(_),
    ));
}

#[test]
fn a_row_whose_outcome_is_not_the_observed_one_is_refused() {
    let run = Run::passing();
    let mut report = run.report();
    report.cases[0].status = CaseStatus::Failed;
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportCaseOutcomeMismatch(_),
    ));

    let mut report = run.report();
    report.cases[0].observed.verdict = NativeVerdict::InfrastructureError;
    assert!(matches!(
        run.refuse(report),
        NativeConformanceError::ReportCaseOutcomeMismatch(_),
    ));
}

#[test]
fn a_failed_case_cannot_be_removed_to_make_the_run_pass() {
    // The defect the validated wrapper exists for: a run with a failing
    // case, whose report is edited to drop the row and restate the
    // summary. Both the deletion and the restated summary are refused,
    // and the honest report fails the gate.
    let target = reviewed_target();
    let binding = development_binding(&target);
    let census = canonical_fixture_set(&target, &binding).expect("the census states");
    let accepting = census
        .iter()
        .find(|fixture| fixture.expected().is_accepting())
        .map(crate::fixture::PrimitiveFixture::case)
        .expect("the census has an accepting case");

    let run = Run::with_override(|case, answer| {
        (case == accepting).then(|| {
            let mut wrong = answer.clone();
            wrong.verdict = NativeVerdict::Rejected;
            wrong.observed_failure = Some(crate::protocol::ObservedFailureClass::EvaluatedFalse);
            wrong
        })
    });
    let honest = run.report();
    assert!(honest.summary.cases_failed > 0);
    let validated =
        validate_native_report(honest, run.inputs()).expect("the honest report validates");
    assert!(
        gate(&validated, Some(&expected_provenance())).is_err(),
        "a failing run is not evidence"
    );

    let mut edited = validated.into_report();
    let failed = edited
        .cases
        .iter()
        .position(|row| row.status == CaseStatus::Failed)
        .expect("a case failed");
    edited.cases.remove(failed);
    edited.summary.cases_total -= 1;
    edited.summary.cases_failed -= 1;
    edited.summary.completeness = ReportCompleteness::PartialUnresolvedClaims;
    assert!(matches!(
        run.refuse(edited),
        NativeConformanceError::ReportCaseCensusMismatch,
    ));
}

#[test]
fn equal_inputs_produce_equal_report_bytes() {
    // Determinism is the property that lets a report be compared at all.
    // Nothing here carries a clock, a host, a process identifier, or a
    // path, and every collection is ordered — so two runs over the same
    // contract, binding, census, registry, and answers are the same
    // bytes, not merely the same information.
    let first = Run::passing().report();
    let second = Run::passing().report();
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_vec(&first).expect("a report serializes"),
        serde_json::to_vec(&second).expect("a report serializes"),
    );
}

#[test]
fn an_executor_that_establishes_no_workspace_provenance_says_so() {
    // The support handshake states a binary revision, an intended tip,
    // and an upstream base, which is what ADR-018 asks a run to record.
    let run = Run::passing();
    let report = run.report();
    assert!(report.executor.establishes_workspace_provenance());
    assert_eq!(
        report.executor.node_name,
        run.transcript.handshake().node_name,
    );

    // Strip the binary's own revision and the run no longer establishes
    // it. The point is that the field goes empty rather than being filled
    // from a checkout that identifies intended source and not a binary.
    let mut provenance = report.executor;
    provenance.binary_reported_revision = None;
    assert!(!provenance.establishes_workspace_provenance());
    assert!(provenance.intended_executed_tip.is_some());
}

#[test]
fn a_declared_mock_run_can_never_satisfy_the_gate() {
    let run = Run::new(ExecutorTrust::Mock, |_case, _answer| None);
    let report = run.report();
    let validated = validate_native_report(report, run.inputs()).expect("the report validates");
    assert!(matches!(
        gate(&validated, Some(&expected_provenance())).expect_err("a mock is refused"),
        NativeConformanceError::MockExecutorCannotSatisfyNativeGate,
    ));
}
