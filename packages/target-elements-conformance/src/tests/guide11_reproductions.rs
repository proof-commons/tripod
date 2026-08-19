//! Guide-11 reproductions and, where a wave has landed, guarantees.
//!
//! # Two kinds of test live here
//!
//! Every test began as a reproduction: it demonstrated a finding from the
//! Guide-11 preflight register by *passing* while the defect was present,
//! asserting that the wrong thing happened. As each wave repairs its
//! finding it flips the assertions of that finding's tests, which then
//! stand as the guarantee that the repair holds.
//!
//! - `G11-R02` and `G11-R03` are **CLOSED** by Wave 1. Their tests below
//!   assert the safe behaviour: a caller-authored subject cannot reach an
//!   evidence gate, and a report cannot claim provenance it does not
//!   have.
//! - `G11-R01` is **CLOSED** by Wave 2. Its tests assert that a
//!   transcript is bound to the contract, the binding, and the exact
//!   subjects it was produced under, so a run cannot be rebound to
//!   fixtures or a deployment it never touched.
//! - `G11-R04`, `G11-R05`, and `G11-R06` are **CLOSED** by Wave 3.
//!   Resource evidence follows the enforcement layer its fixture states,
//!   the gate reads every case status and the report's own completeness,
//!   and the run's provenance is compared against an explicit
//!   expectation before any row is consulted.
//! - `G11-R14` is **CLOSED** by Wave 4. One shared predicate says which
//!   defects a different representation nonce could repair, and both
//!   retry implementations consult it, so a permanent defect is
//!   reported after one attempt instead of being ground against every
//!   nonce in the bound and then misreported as exhaustion.
//!
//! Each test names its finding identifier in its own documentation. No
//! test here touches a production code path: they are constructions over
//! the public and crate-visible surfaces exactly as an external caller or
//! the existing suites reach them.
//!
//! # Two findings whose reproductions are prose rather than tests
//!
//! `G11-R07` is a build-system default, so its regressions live where
//! Meson can fail: configuring an executor path without a class, and a
//! reviewed class without its ADR-018 provenance, are configuration
//! errors, checked by configuring. `G11-R13` is a process-lifetime
//! property, whose regressions are in `executor.rs` and
//! `tests/executor_supervision.rs`.
//!
//! # What Wave 1 changed about the reproductions themselves
//!
//! Closing `G11-R02` removed the arbitrary-census route to the evidence
//! path, and two open findings had reproductions built on that route. The
//! `G11-R01` script-substitution reproduction moved to the experimental
//! path, where Wave 2 has now flipped it: the binding is a property of
//! the transcript rather than of the trust state, so the substitution is
//! refused there too. The `G11-R05` reproduction cannot be expressed at
//! all any more — its construction added a fixture to the canonical
//! census — so what stands in its place records that the route is closed
//! and that the gate defect it named is still open and still Wave 3's.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    TargetEvidenceRequirementId, validate_reviewed_development_binding,
};

use crate::claim::{ClaimRegistry, NativeEvidenceClaim, claim_registry, claims_of};
use crate::constructor::canonical::{CanonicalOrderDefect, construct_canonically_ordered};
use crate::constructor::curve::{FIELD_ELEMENT_BYTES, PointDecodingDefect};
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::metadata::PrototypeMetadata;
use crate::constructor::metadata_leaf::metadata_leaf_script;
use crate::constructor::totality::{TotalityDefect, TweakTotalityPolicy, construct_under_policy};
use crate::constructor::tree::{
    ConstructionDefect, FixtureTapTree, TreeDefect, TweakDefect, construct, retryable,
};
use crate::error::NativeConformanceError;
use crate::executor::{
    ExecutionTranscript, ExecutorTrust, PrototypeTranscriptParts, TranscriptParts,
};
use crate::fixture::{
    CanonicalPrimitiveFixtureSet, EnforcementLayer, ExpectedPrimitiveOutcome,
    ExpectedResourceObservation, FixtureScript, FixtureScriptSource, FixtureStatement,
    LeafVersionStatus, NativeCaseGroup, NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet,
    ResourceExpectation, canonical_fixture_set,
};
use crate::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeExecutionResponse, NativePrototypeResponse,
    NativeResourceObservation, NativeVerdict, ObservedFailureClass,
};
use crate::prototype::{
    CanonicalPrototypeMatrix, CompoundPrototypeFixture, ConstructorPrototypeMatrix,
    ExpectedPrototypeOutcome, PrototypeCaseId, PrototypeClaim, PrototypeConstruction,
    PrototypeRelation, WideFloorPrototypeMatrix, constructor_case_matrix, wide_floor_case_matrix,
};
use crate::prototype_report::PrototypeReportCompleteness;
use crate::prototype_validate::{
    PrototypeReportValidationInputs, evaluate_experimental_prototypes, evaluate_prototypes,
    prototype_gate, validate_prototype_report,
};
use crate::provenance::{ProvenanceDefect, ProvenanceSyntaxDefect};
use crate::report::{
    CaseStatus, EvidenceDisposition, EvidencePlanClass, PrototypeReportRole, ReportCompleteness,
};
use crate::validate::{
    NativeReportValidationInputs, ValidatedNativeConformanceReport, evaluate,
    evaluate_experimental, gate, guide_nine_evidence_plan, validate_native_report,
};

use super::support::{
    TEST_BINARY_REVISION, TEST_INTENDED_TIP, development_binding, expected_provenance,
    nonmock_handshake, observed_environment, prototype_subjects_of, reviewed_target, subjects_of,
};

/// A second development binding, on a different network and genesis.
///
/// Used to show that a transcript observed under one binding evaluates
/// happily under another.
fn other_binding(target: &ReviewedElementsTapscriptDefinition) -> ReviewedDevelopmentBinding {
    let binding = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        DeploymentEnvironment::Development,
        [0x33; 32],
        [0x44; 32],
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    validate_reviewed_development_binding(target, binding).expect("the binding validates")
}

/// What an honest executor reporting no interpreter stack would answer.
fn contract_answer(case: NativeCaseId, fixture: &PrimitiveFixture) -> NativeExecutionResponse {
    let resources = NativeResourceObservation {
        script_bytes: fixture.script().len() as u64,
        initial_stack_items: fixture.initial_stack().len() as u64,
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

/// Every response an honest run over one census would produce.
///
/// Taken as an iterator of fixtures rather than as one census type, so
/// that the canonical wrapper and a bare census can both be answered.
fn answers<'a>(
    fixtures: impl IntoIterator<Item = &'a PrimitiveFixture>,
) -> BTreeMap<NativeCaseId, NativeExecutionResponse> {
    fixtures
        .into_iter()
        .map(|fixture| (fixture.case(), contract_answer(fixture.case(), fixture)))
        .collect()
}

/// The evidence plan and the claim census, which every run below needs.
fn plan_and_registry() -> (crate::validate::EvidencePlan, ClaimRegistry) {
    (
        guide_nine_evidence_plan().expect("the plan is a partition"),
        claim_registry().expect("the claim census is coherent"),
    )
}

/// Every resource figure recorded and none of them fixed.
const fn recorded_only() -> ExpectedResourceObservation {
    ExpectedResourceObservation {
        script_bytes: ResourceExpectation::RecordedOnly,
        initial_stack_items: ResourceExpectation::RecordedOnly,
        peak_stack_items: ResourceExpectation::RecordedOnly,
        peak_altstack_items: ResourceExpectation::RecordedOnly,
        maximum_element_bytes: ResourceExpectation::RecordedOnly,
        validation_budget_used: ResourceExpectation::RecordedOnly,
        transaction_weight: ResourceExpectation::RecordedOnly,
    }
}

/// One prototype metadata object, at its first nonce.
const fn metadata() -> PrototypeMetadata {
    PrototypeMetadata {
        schema: 1,
        object_kind: 1,
        counter: 0,
        flags: 0,
        nonce: 0,
    }
}

/// A one-instruction program that pushes a nonempty literal.
fn pushes(target: &ReviewedElementsTapscriptDefinition, byte: u8) -> TapscriptProgram {
    let item = StackItem::new(target, vec![byte]).expect("a one-byte literal is admitted");
    TapscriptProgram::new(vec![TapscriptInstruction::Push(item)])
        .expect("a one-instruction program is admitted")
}

// -- G11-R01 (CLOSED by Wave 2) ---------------------------------------

/// `G11-R01` **CLOSED**: a transcript observed under one deployment
/// binding cannot be evaluated under a different one.
///
/// The transcript retains the deployment projection the run was requested
/// under, and evaluation compares it against the binding the report is
/// being stated against. A report whose declared network and observed
/// network disagree is therefore not a report that gets built and then
/// gated; it is a report that does not exist.
#[test]
fn g11_r01_a_transcript_cannot_rebind_to_a_deployment_it_never_ran_on() {
    let target = reviewed_target();
    let requested = development_binding(&target);
    let substituted = other_binding(&target);

    // The run: an honest executor answering the census stated against the
    // requested binding, and reporting the environment of that binding.
    let executed = canonical_fixture_set(&target, &requested).expect("the census states");
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &requested,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&executed),
        responses: answers(&executed),
    });

    // The evaluation: the same transcript, against the census and binding
    // of a network the executor never saw.
    let rebound = canonical_fixture_set(&target, &substituted).expect("the census states");
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let refusal = evaluate(
        &target,
        &substituted,
        &rebound,
        &transcript,
        &plan,
        &registry,
    )
    .expect_err("a cross-binding evaluation is refused");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::TranscriptDeploymentRebinding,
        ),
        "expected a deployment rebinding refusal, got {refusal:?}",
    );

    // And again at validation, which is the second of the two places the
    // guide requires the environment to be compared. A caller reaching
    // the validator directly with a report from elsewhere meets the same
    // refusal, by a path that does not depend on the recomputation being
    // the one that catches it.
    let honest = evaluate(
        &target,
        &requested,
        &executed,
        &transcript,
        &plan,
        &registry,
    )
    .expect("the run under its own binding evaluates");
    let refusal = validate_native_report(
        honest,
        NativeReportValidationInputs {
            target: &target,
            binding: &substituted,
            fixtures: &rebound,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )
    .expect_err("a cross-binding validation is refused");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::EnvironmentBindingMismatch
                | NativeConformanceError::GenesisObservationMismatch,
        ),
        "expected an environment refusal at validation, got {refusal:?}",
    );
}

/// `G11-R01` **CLOSED**: a report cannot name a script the executor was
/// never handed.
///
/// The transcript retains the exact subject sent for each case, and
/// evaluation compares it against the fixture being reported. A fixture
/// with the same case identity, the same encoded width, and a compatible
/// expectation no longer takes the executed fixture's place.
///
/// # The width is the point
///
/// The two scripts here are the same length deliberately. The only thing
/// that used to stand between a substitution and a report was the exact
/// `script_bytes` resource expectation, which compares *widths* — so a
/// substitution that preserved the width sailed through it. Exact typed
/// comparison of the subject is what makes the width irrelevant.
///
/// # Both paths, not only the evidence one
///
/// This runs on the experimental path, where Wave 1 left it. That is not
/// a weaker statement: the binding is a property of the transcript rather
/// than of the trust state, so an experimental report cannot name an
/// unexecuted script either. What the experimental path keeps is
/// describing a run that did happen over a census the caller chose.
#[test]
fn g11_r01_a_report_cannot_name_a_script_the_executor_never_ran() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let case = NativeCaseId::new(NativeCaseGroup::Comparison, None, 0);

    let executed_program = pushes(&target, 0x01);
    let substituted_program = pushes(&target, 0x02);
    let state = |program: &TapscriptProgram| {
        PrimitiveFixture::state(
            &target,
            &binding,
            FixtureStatement {
                case,
                script: FixtureScript::Typed(program),
                initial_stack: &[],
                context: None,
                expected: ExpectedPrimitiveOutcome::accept(None),
                leaf_version: LeafVersionStatus::Reviewed,
                unreviewed_leaf_version: None,
                enforcement_layer: EnforcementLayer::Consensus,
            },
        )
        .expect("the fixture states")
    };
    let executed = state(&executed_program);
    let substituted = state(&substituted_program);
    assert_eq!(
        executed.script().len(),
        substituted.script().len(),
        "the two scripts have the same encoded width",
    );
    assert_ne!(executed.script(), substituted.script());
    // The width-only defence would have passed this substitution: the one
    // figure that used to be compared agrees exactly.
    assert_eq!(
        executed.expected_resources().script_bytes,
        substituted.expected_resources().script_bytes,
        "the substitution is invisible to the resource comparison",
    );

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of([&executed]),
        responses: BTreeMap::from([(case, contract_answer(case, &executed))]),
    });

    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let reported = PrimitiveFixtureSet::new([substituted]).expect("one fixture is a census");
    let refusal =
        evaluate_experimental(&target, &binding, &reported, &transcript, &plan, &registry)
            .expect_err("the substituted census is refused against another run's answers");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::TranscriptSubjectMismatch(reported) if reported == case,
        ),
        "expected a subject mismatch for the substituted case, got {refusal:?}",
    );

    // The executed fixture still evaluates, so the refusal is a binding
    // and not a blanket one.
    let honest = PrimitiveFixtureSet::new([executed]).expect("one fixture is a census");
    let report = evaluate_experimental(&target, &binding, &honest, &transcript, &plan, &registry)
        .expect("the executed census evaluates against its own run")
        .into_report();
    assert_eq!(report.cases.len(), 1);
    assert_eq!(report.cases[0].status, CaseStatus::Passed);
}

/// `G11-R01` **CLOSED**: a substituted initial stack is refused too.
///
/// The script is one member of the subject and the stack is another. A
/// repair that compared only the program would leave the same
/// substitution available one field along.
#[test]
fn g11_r01_a_report_cannot_name_an_initial_stack_the_executor_never_ran() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let case = NativeCaseId::new(NativeCaseGroup::Comparison, None, 0);
    let program = pushes(&target, 0x01);

    let state = |stack: &[StackItem]| {
        PrimitiveFixture::state(
            &target,
            &binding,
            FixtureStatement {
                case,
                script: FixtureScript::Typed(&program),
                initial_stack: stack,
                context: None,
                expected: ExpectedPrimitiveOutcome::accept(None),
                leaf_version: LeafVersionStatus::Reviewed,
                unreviewed_leaf_version: None,
                enforcement_layer: EnforcementLayer::Consensus,
            },
        )
        .expect("the fixture states")
    };
    let item = |byte: u8| StackItem::new(&target, vec![byte]).expect("a one-byte item is admitted");
    let executed = state(&[item(0x07)]);
    let substituted = state(&[item(0x09)]);
    // One item either way, so the exact `initial_stack_items` expectation
    // agrees and sees nothing.
    assert_eq!(
        executed.expected_resources().initial_stack_items,
        substituted.expected_resources().initial_stack_items,
    );

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of([&executed]),
        responses: BTreeMap::from([(case, contract_answer(case, &executed))]),
    });

    let (plan, registry) = plan_and_registry();
    let reported = PrimitiveFixtureSet::new([substituted]).expect("one fixture is a census");
    let refusal =
        evaluate_experimental(&target, &binding, &reported, &transcript, &plan, &registry)
            .expect_err("the substituted stack is refused");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::TranscriptSubjectMismatch(reported) if reported == case,
        ),
        "expected a subject mismatch, got {refusal:?}",
    );
}

/// `G11-R01` **CLOSED**: a transcript answering a case it never asked
/// about is refused.
///
/// A transcript whose two halves do not correspond describes no run. This
/// is the half the per-case comparison cannot reach: a response with no
/// request is not a fixture being reported wrongly, it is an answer that
/// belongs to some other exchange.
#[test]
fn g11_r01_a_response_for_a_case_never_sent_is_refused() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let case = NativeCaseId::new(NativeCaseGroup::Comparison, None, 0);
    let stray = NativeCaseId::new(NativeCaseGroup::Comparison, None, 1);

    let program = pushes(&target, 0x01);
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        case,
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::accept(None),
    )
    .expect("the fixture states");

    let mut responses = BTreeMap::from([(case, contract_answer(case, &fixture))]);
    responses.insert(stray, contract_answer(stray, &fixture));
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of([&fixture]),
        responses,
    });

    let (plan, registry) = plan_and_registry();
    let fixtures = PrimitiveFixtureSet::new([fixture]).expect("one fixture is a census");
    let refusal =
        evaluate_experimental(&target, &binding, &fixtures, &transcript, &plan, &registry)
            .expect_err("an answer with no question is refused");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::UnrequestedCaseResponse(answered) if answered == stray,
        ),
        "expected an unrequested-response refusal naming the stray case, got {refusal:?}",
    );
}

/// `G11-R01` **CLOSED**: a case with no retained request is refused
/// before its answer is read.
///
/// The pre-existing missing-response refusal is a different statement: it
/// says the executor did not answer. This one says the executor was never
/// asked, which is what a report built from another run's transcript
/// looks like when the case identities happen to line up.
#[test]
fn g11_r01_a_case_that_was_never_sent_is_refused() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let case = NativeCaseId::new(NativeCaseGroup::Comparison, None, 0);

    let program = pushes(&target, 0x01);
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        case,
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::accept(None),
    )
    .expect("the fixture states");

    // An answer for the case, and no record of ever having asked.
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: BTreeMap::new(),
        responses: BTreeMap::from([(case, contract_answer(case, &fixture))]),
    });

    let (plan, registry) = plan_and_registry();
    let fixtures = PrimitiveFixtureSet::new([fixture]).expect("one fixture is a census");
    let refusal =
        evaluate_experimental(&target, &binding, &fixtures, &transcript, &plan, &registry)
            .expect_err("a case that was never sent is refused");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::UnrequestedCaseResponse(answered) if answered == case,
        ),
        "expected the unanswered half to be named, got {refusal:?}",
    );
}

/// `G11-R01` **CLOSED**: a case sent and never answered is still refused.
///
/// Checked because the repair added a request map beside the response
/// map, and a repair that read only the new one would have lost the old
/// refusal.
#[test]
fn g11_r01_a_case_sent_and_never_answered_is_refused() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let case = NativeCaseId::new(NativeCaseGroup::Comparison, None, 0);

    let program = pushes(&target, 0x01);
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        case,
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::accept(None),
    )
    .expect("the fixture states");

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of([&fixture]),
        responses: BTreeMap::new(),
    });

    let (plan, registry) = plan_and_registry();
    let fixtures = PrimitiveFixtureSet::new([fixture]).expect("one fixture is a census");
    let refusal =
        evaluate_experimental(&target, &binding, &fixtures, &transcript, &plan, &registry)
            .expect_err("an unanswered case is refused");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::MissingCaseResponse(unanswered) if unanswered == case,
        ),
        "expected a missing-response refusal, got {refusal:?}",
    );
}

/// `G11-R01` **CLOSED**: a changed expected verdict under a canonical
/// case identity is refused.
///
/// # Which weld catches this one, and why it is not the transcript's
///
/// The review lists a changed expectation beside the changed script and
/// stack, and the answer here is a different one on purpose. An
/// expectation is not part of the execution subject and does not cross
/// the wire at all under revision 3, so the transcript has nothing to
/// compare it against — by construction, which is the point. What refuses
/// it is the canonical comparison: the evidence path regenerates the
/// census and compares complete projections, and a projection whose
/// expectation differs is not the canonical case of that identity.
///
/// The two welds are therefore complementary rather than redundant. The
/// canonical comparison owns what the case *should* do; the transcript
/// owns what the executor was actually *asked*.
#[test]
fn g11_r01_a_changed_expected_verdict_is_refused() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = canonical_fixture_set(&target, &binding).expect("the census states");

    // One accepting static case, restated with the opposite verdict and
    // with every other member — program, stack, context, layer, leaf —
    // exactly as the canonical census states it.
    let victim = canonical
        .iter()
        .find(|fixture| {
            fixture.context().is_none()
                && fixture.expected().is_accepting()
                && fixture.leaf_version_status() == LeafVersionStatus::Reviewed
                && fixture.initial_stack().is_empty()
        })
        .expect("the census states a static accepting case with no initial stack");
    let case = victim.case();
    let flipped = PrimitiveFixture::state(
        &target,
        &binding,
        FixtureStatement {
            case,
            script: FixtureScript::DeliberatelyMalformed(victim.script().to_vec()),
            initial_stack: &[],
            context: None,
            expected: ExpectedPrimitiveOutcome::reject(
                [ObservedFailureClass::EvaluatedFalse],
                None,
            ),
            leaf_version: LeafVersionStatus::Reviewed,
            unreviewed_leaf_version: None,
            enforcement_layer: EnforcementLayer::Consensus,
        },
    )
    .expect("the flipped fixture states");

    let mutated = PrimitiveFixtureSet::new(
        canonical
            .iter()
            .filter(|fixture| fixture.case() != case)
            .cloned()
            .chain([flipped]),
    )
    .expect("the mutated census assembles");

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&mutated),
        responses: answers(&mutated),
    });
    let (plan, registry) = plan_and_registry();
    let refusal = evaluate(
        &target,
        &binding,
        &CanonicalPrimitiveFixtureSet::wrap_for_tests(mutated),
        &transcript,
        &plan,
        &registry,
    )
    .expect_err("a changed expectation removes canonical membership");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::NoncanonicalFixtureSubject(subject) if subject == case,
        ),
        "expected a canonical-subject refusal naming the case, got {refusal:?}",
    );
}

/// `G11-R01` **CLOSED**: a prototype construction cannot be substituted
/// under the same case name.
///
/// The compound counterpart, and the sharpest of the substitutions: a
/// row's whole claim is that one exact tree held together, so a report
/// naming a construction the executor never built would credit the
/// relation to a tree nobody materialized.
#[test]
fn g11_r01_a_prototype_construction_cannot_be_substituted() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    // The constructor matrix, whose rows state genuinely different trees:
    // the wide-floor rows share one construction and vary their witness,
    // so a construction substitution is not expressible there.
    let canonical = constructor_case_matrix(&target).expect("the constructor matrix is authored");

    // The run: the canonical matrix, honestly executed.
    let transcript = prototype_transcript(&target, &binding, canonical.rows());

    // The report: the same rows with one row's construction and program
    // replaced by another canonical row's. Both are real constructions
    // and the result is locally coherent, so nothing about the fixture
    // itself notices.
    let mut rows = canonical.rows().to_vec();
    let donor = rows
        .iter()
        .position(|row| row.construction != rows[0].construction)
        .expect("two constructor rows state different constructions");
    let substituted_case = rows[0].case.clone();
    rows[0].construction = rows[donor].construction.clone();
    rows[0].script = rows[donor].script.clone();

    let refusal = evaluate_experimental_prototypes(
        &target,
        &binding,
        PrototypeRelation::MetadataConstructorContinuity,
        &rows,
        &transcript,
    )
    .expect_err("a substituted construction is refused");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::PrototypeTranscriptSubjectMismatch(ref case)
                if *case == substituted_case,
        ),
        "expected a prototype subject mismatch, got {refusal:?}",
    );
}

// -- G11-R02 (CLOSED by Wave 1) ---------------------------------------

/// `G11-R02` **CLOSED**: a trivial true script labelled as a signature
/// case cannot satisfy signature evidence.
///
/// The label still produces the claim — `claims_of` reads the case's
/// declared group and primitive, and that derivation is what makes the
/// canonical census's claims meaningful — but the labelled fixture can no
/// longer be an evidence subject. The census a caller assembles is a
/// [`PrimitiveFixtureSet`], and the evidence path takes only a
/// [`CanonicalPrimitiveFixtureSet`], which has no constructor but the
/// canonical generator.
///
/// So the repair is not "inspect the script for the named primitive",
/// which a script containing an unreached opcode would defeat. It is that
/// membership in the evidence plan is the repository's to decide.
#[test]
fn g11_r02_a_trivial_true_script_cannot_bear_signature_evidence() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let signature_opcode = target_elements::OpcodeId::CheckSig;
    let case = NativeCaseId::new(NativeCaseGroup::Signature, Some(signature_opcode), 0);

    let program = pushes(&target, 0x01);
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        case,
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::accept(None),
    )
    .expect("an experimental fixture still states");

    // The label still derives the claim. That is deliberate: the claim
    // predicates are canonical policy over canonical cases, not a check
    // on a caller's honesty.
    let claims = claims_of(&fixture);
    assert!(claims.contains(&NativeEvidenceClaim::TransactionSignatureAccepted));
    assert!(
        !fixture
            .script()
            .contains(&target.definition().opcodes()[&signature_opcode].code()),
        "no signature primitive occurs in the script",
    );

    // What has closed is the route from that label to evidence.
    let fixtures = PrimitiveFixtureSet::new([fixture.clone()]).expect("a census assembles");
    let (plan, registry) = plan_and_registry();
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of([&fixture]),
        responses: BTreeMap::from([(case, contract_answer(case, &fixture))]),
    });

    // The only report an arbitrary census can produce is the experimental
    // one, and it says so in the document as well as in the type.
    let experimental =
        evaluate_experimental(&target, &binding, &fixtures, &transcript, &plan, &registry)
            .expect("an arbitrary census is reportable as an experiment");
    assert_eq!(
        experimental.report().role,
        PrototypeReportRole::ExperimentalPrimitive,
        "the report states that its subject was a caller's choice",
    );

    // And the second line of defence, reached here only because the
    // crate's own tests can wrap a census the generator did not state: the
    // evidence path regenerates the canonical census and refuses this one
    // on provenance.
    let forged = CanonicalPrimitiveFixtureSet::wrap_for_tests(fixtures);
    let refusal = evaluate(&target, &binding, &forged, &transcript, &plan, &registry)
        .expect_err("a census the generator did not state is not an evidence subject");
    assert!(
        matches!(refusal, NativeConformanceError::NoncanonicalFixtureCensus),
        "the refusal names the subject, not the coverage: {refusal:?}",
    );
}

/// `G11-R02` **CLOSED**: an arbitrary census is refused on *provenance*,
/// which is a different refusal from the one it used to get.
///
/// This is the distinction the Wave-0 reproduction recorded. A one-case
/// census used to reach `gate` and be refused there only because every
/// other required row was empty — a completeness refusal, which a caller
/// with enough labelled fixtures would have satisfied. It is now refused
/// before any row is computed, because of where the census came from.
#[test]
fn g11_r02_an_arbitrary_census_is_refused_on_provenance_not_completeness() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let case = NativeCaseId::new(
        NativeCaseGroup::Signature,
        Some(target_elements::OpcodeId::CheckSig),
        0,
    );
    let program = pushes(&target, 0x01);
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        case,
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::accept(None),
    )
    .expect("the fixture states");

    let fixtures = PrimitiveFixtureSet::new([fixture.clone()]).expect("a census assembles");
    let canonical = canonical_fixture_set(&target, &binding).expect("the census states");
    assert_ne!(
        &fixtures,
        canonical.fixtures(),
        "the arbitrary census is not the canonical one",
    );

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of([&fixture]),
        responses: BTreeMap::from([(case, contract_answer(case, &fixture))]),
    });
    let (plan, registry) = plan_and_registry();

    let refusal = evaluate(
        &target,
        &binding,
        &CanonicalPrimitiveFixtureSet::wrap_for_tests(fixtures),
        &transcript,
        &plan,
        &registry,
    )
    .expect_err("the evidence path refuses a census it did not state");
    assert!(
        matches!(refusal, NativeConformanceError::NoncanonicalFixtureCensus),
        "a subject refusal, not a completeness one: {refusal:?}",
    );

    // The old refusal is what the new one replaces: the canonical census
    // gates, so the harness is not simply refusing everything.
    let honest = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&canonical),
        responses: answers(&canonical),
    });
    let report = evaluate(&target, &binding, &canonical, &honest, &plan, &registry)
        .expect("the canonical census evaluates");
    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &binding,
            fixtures: &canonical,
            plan: &plan,
            registry: &registry,
            transcript: &honest,
        },
    )
    .expect("the canonical report validates");
    gate(&validated, Some(&expected_provenance()))
        .expect("the canonical census is the evidence subject");
}

/// `G11-R02` **CLOSED**: changing any member of a canonical fixture
/// removes gate eligibility.
///
/// The invariant is that a canonical case's subject, its exact program and
/// context, its exact expected outcome, and its exact claim set stand or
/// fall together. Here one case keeps its identity and loses its program,
/// and the complete-projection comparison refuses it by name.
#[test]
fn g11_r02_changing_a_canonical_fixture_member_removes_gate_eligibility() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = canonical_fixture_set(&target, &binding).expect("the census states");

    // A case with no transaction context, so the replacement needs none
    // either and the only difference is the program.
    let victim = canonical
        .iter()
        .find(|fixture| {
            fixture.context().is_none()
                && fixture.expected().is_accepting()
                && fixture.leaf_version_status() == LeafVersionStatus::Reviewed
        })
        .expect("the census states a static accepting case")
        .case();

    let program = pushes(&target, 0x01);
    let replacement = PrimitiveFixture::new(
        &target,
        &binding,
        victim,
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::accept(None),
    )
    .expect("the replacement states");

    let mutated = PrimitiveFixtureSet::new(
        canonical
            .iter()
            .filter(|fixture| fixture.case() != victim)
            .cloned()
            .chain([replacement]),
    )
    .expect("the mutated census assembles");
    assert_eq!(
        mutated.len(),
        canonical.len(),
        "exactly one member changed, and the census is the same size",
    );

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&mutated),
        responses: answers(&mutated),
    });
    let (plan, registry) = plan_and_registry();
    let refusal = evaluate(
        &target,
        &binding,
        &CanonicalPrimitiveFixtureSet::wrap_for_tests(mutated),
        &transcript,
        &plan,
        &registry,
    )
    .expect_err("a mutated canonical case is not a canonical case");
    assert_eq!(
        format!("{refusal:?}"),
        format!(
            "{:?}",
            NativeConformanceError::NoncanonicalFixtureSubject(victim)
        ),
        "the refusal names the case whose member changed",
    );
}

/// `G11-R02` **CLOSED**: a canonical case whose claim set changes is
/// refused, even under its own identity.
///
/// A primitive fixture carries no claim field: its claims are derived
/// from the case it answers for, the outcome the reviewed contract
/// requires, the enforcement layer, and the context. So "one added or
/// removed claim" is reached by changing one of those — here the required
/// outcome, which decides between the primitive-success and
/// primitive-abort claims — while the case identity stays exactly what it
/// was. The complete projection carries the derived claims, so the
/// substitution is refused by name.
#[test]
fn g11_r02_a_canonical_case_whose_claims_change_is_refused() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = canonical_fixture_set(&target, &binding).expect("the census states");

    let victim = canonical
        .iter()
        .find(|fixture| {
            fixture.context().is_none()
                && fixture.expected().is_accepting()
                && fixture.case().opcode().is_some()
                && fixture.leaf_version_status() == LeafVersionStatus::Reviewed
        })
        .expect("the census states a static accepting case naming a primitive")
        .clone();

    // The same case, required to abort rather than to succeed. An
    // aborting class is what moves the claim set: a case that completes
    // and is rejected anyway still ran its primitive, so it keeps the
    // primitive-success claim.
    let program = pushes(&target, 0x01);
    let inverted = PrimitiveFixture::new(
        &target,
        &binding,
        victim.case(),
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::reject([ObservedFailureClass::StackUnderflow], None),
    )
    .expect("the inverted fixture states");
    assert_ne!(
        claims_of(&inverted),
        claims_of(&victim),
        "the changed outcome changes the claim set the case bears",
    );

    let mutated = PrimitiveFixtureSet::new(
        canonical
            .iter()
            .filter(|fixture| fixture.case() != victim.case())
            .cloned()
            .chain([inverted]),
    )
    .expect("the mutated census assembles");
    assert_eq!(
        mutated.len(),
        canonical.len(),
        "the case identity is unchanged, so the census is the same size",
    );

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&mutated),
        responses: answers(&mutated),
    });
    let (plan, registry) = plan_and_registry();
    let refusal = evaluate(
        &target,
        &binding,
        &CanonicalPrimitiveFixtureSet::wrap_for_tests(mutated),
        &transcript,
        &plan,
        &registry,
    )
    .expect_err("a census whose claim sets are not the canonical ones is not evidence");
    assert_eq!(
        format!("{refusal:?}"),
        format!(
            "{:?}",
            NativeConformanceError::NoncanonicalFixtureSubject(victim.case())
        ),
        "the refusal names the case whose claim set changed",
    );
}

/// `G11-R02` **CLOSED**: permuting the canonical declaration order stays
/// harmless.
///
/// The census is keyed by typed case identity, so declaration order is not
/// part of the value at all. Reversing it produces the same census, which
/// still evaluates, validates, and gates.
#[test]
fn g11_r02_permuting_canonical_declaration_order_is_harmless() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = canonical_fixture_set(&target, &binding).expect("the census states");

    let mut reversed: Vec<PrimitiveFixture> = canonical.iter().cloned().collect();
    reversed.reverse();
    let permuted = PrimitiveFixtureSet::new(reversed).expect("the permuted census assembles");
    assert_eq!(
        &permuted,
        canonical.fixtures(),
        "declaration order is not a member of the census",
    );

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&permuted),
        responses: answers(&permuted),
    });
    let (plan, registry) = plan_and_registry();
    let wrapped = CanonicalPrimitiveFixtureSet::wrap_for_tests(permuted);
    let report = evaluate(&target, &binding, &wrapped, &transcript, &plan, &registry)
        .expect("a permutation of the canonical census is the canonical census");
    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &binding,
            fixtures: &wrapped,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )
    .expect("the report validates");
    gate(&validated, Some(&expected_provenance()))
        .expect("a permuted declaration order is still evidence");
}

// -- G11-R04 (CLOSED by Wave 3) ----------------------------------------

/// `G11-R04` **CLOSED**: consensus resource cases are not credited to
/// the policy resource row.
///
/// `bearing_requirements` receives the complete fixture, so it reads the
/// enforcement layer the case is stated at: a consensus case bears on
/// the consensus row and on nothing else. The canonical census states
/// every resource case at the consensus layer, so the policy row has no
/// case at all — and the plan now says so, classifying it unresolved by
/// design rather than required, which is the honest state until a real
/// relay-policy matrix exists.
#[test]
fn g11_r04_consensus_resource_cases_do_not_reach_the_policy_resource_row() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&fixtures),
        responses: answers(&fixtures),
    });
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");

    assert!(
        fixtures
            .iter()
            .filter(|fixture| fixture.case().group() == NativeCaseGroup::Resource)
            .all(|fixture| fixture.enforcement_layer() == EnforcementLayer::Consensus),
        "every canonical resource case is stated at the consensus layer",
    );

    let consensus = report
        .evidence
        .iter()
        .find(|row| row.requirement == "consensus_resource_limits")
        .expect("the consensus resource row exists");
    assert_eq!(consensus.plan, EvidencePlanClass::Required);
    assert_eq!(consensus.disposition, EvidenceDisposition::Passed);
    assert_ne!(
        consensus.cases, 0,
        "the consensus cases are counted under the consensus row",
    );

    let policy = report
        .evidence
        .iter()
        .find(|row| row.requirement == "policy_resource_limits")
        .expect("the policy resource row exists");
    assert_eq!(
        policy.plan,
        EvidencePlanClass::UnresolvedByDesign,
        "the row is honestly unresolved until relay-policy cases exist",
    );
    assert_eq!(
        policy.cases, 0,
        "no consensus case is counted under the policy row",
    );
    assert_eq!(
        policy.disposition,
        EvidenceDisposition::UnresolvedByDesign,
        "an unattempted row is unresolved, which is neither pass nor failure",
    );

    let claim = report
        .claims
        .iter()
        .find(|row| row.claim == NativeEvidenceClaim::PolicyResourceBoundObserved)
        .expect("the policy resource claim exists");
    assert_eq!(
        claim.disposition,
        EvidenceDisposition::UnresolvedByDesign,
        "the claim and the row it defines now agree",
    );
    assert!(
        claim.bearing_cases.is_empty(),
        "no case bears on the policy claim",
    );

    // The row and its defining claim no longer contradict each other,
    // and the report says the run left a dimension unattempted rather
    // than claiming it established one.
    assert_eq!(
        report.summary.completeness,
        ReportCompleteness::PartialUnresolvedClaims,
    );
}

// -- G11-R05 (CLOSED by Wave 3) ----------------------------------------

/// `G11-R05` **CLOSED**: both routes to the gate are shut.
///
/// # What the reproduction used to do
///
/// It added one fixture to the canonical census — a case in a group whose
/// only evidence requirement sits outside the required plan, and whose
/// only claim is not required — and answered it with a rejection. The
/// case failed without touching a required row or a required claim, so
/// `summarize` called the report failed and `gate`, which reads neither
/// the completeness nor the case statuses, returned success.
///
/// # Why it cannot do that any more
///
/// Wave 1 made the census a canonical trust state, so an augmented census
/// is refused before any row is computed. That was the `G11-R02` repair
/// doing its work rather than a repair of this finding, and it left the
/// gate itself unrepaired: a construction reaching `gate` with a failed
/// case would still have passed.
///
/// # What Wave 3 changed
///
/// The gate now reads every case status and the summary's own
/// completeness. This test therefore shows both halves closed: the
/// census route is refused at evaluation, and the report that route used
/// to produce is refused at the gate when it is put there directly.
///
/// # Why the report is wrapped rather than validated
///
/// The subject under test is `gate`, and the report it must refuse is
/// one `validate_native_report` would never produce — that is the point
/// of the Wave-1 repair. So the report is built by the experimental path
/// over the very census the old reproduction used, and asserted into the
/// validated state through the crate-visible test constructor. The two
/// rules are tested separately because they are two rules.
#[test]
fn g11_r05_the_augmented_census_route_to_the_gate_is_closed() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = canonical_fixture_set(&target, &binding).expect("the census states");

    // A case outside every required row: the sighash group is unresolved
    // by design, and the case names no primitive, so no opcode row is
    // added either.
    let case = NativeCaseId::new(NativeCaseGroup::Sighash, None, 0);
    assert!(
        !canonical.contains(case),
        "the census states no sighash case"
    );
    let program = pushes(&target, 0x01);
    let extra = PrimitiveFixture::new(
        &target,
        &binding,
        case,
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::accept(None),
    )
    .expect("the fixture states");

    let fixtures = PrimitiveFixtureSet::new(canonical.iter().cloned().chain([extra.clone()]))
        .expect("the census assembles");

    // Every case answered as the contract requires, except the extra one,
    // which the executor rejects.
    let mut responses = answers(&fixtures);
    responses.insert(
        case,
        NativeExecutionResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case,
            verdict: NativeVerdict::Rejected,
            final_stack: None,
            final_altstack: None,
            observed_failure: Some(ObservedFailureClass::EvaluatedFalse),
            resources: NativeResourceObservation {
                script_bytes: extra.script().len() as u64,
                initial_stack_items: 0,
                ..NativeResourceObservation::default()
            },
        },
    );

    let (plan, registry) = plan_and_registry();
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&fixtures),
        responses,
    });

    // Half one: the augmented census is no longer an evidence subject.
    let refusal = evaluate(
        &target,
        &binding,
        &CanonicalPrimitiveFixtureSet::wrap_for_tests(fixtures.clone()),
        &transcript,
        &plan,
        &registry,
    )
    .expect_err("an augmented census is not the canonical census");
    assert!(matches!(
        refusal,
        NativeConformanceError::NoncanonicalFixtureCensus
    ));

    // Half two: the report that route produced is refused at the gate.
    let report = evaluate_experimental(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates as an experiment")
        .into_report();
    assert_eq!(
        report.summary.cases_failed, 1,
        "exactly the extra case failed",
    );
    assert_eq!(report.summary.completeness, ReportCompleteness::Failed);
    let sighash = report
        .evidence
        .iter()
        .find(|row| row.requirement == "sighash_semantics")
        .expect("the sighash row exists");
    assert_ne!(
        sighash.plan,
        EvidencePlanClass::Required,
        "the failing case still touches no required row",
    );

    let refusal = gate(
        &ValidatedNativeConformanceReport::wrap_for_tests(report),
        Some(&expected_provenance()),
    )
    .expect_err("a failed case is refused whatever it bears on");
    assert!(
        matches!(refusal, NativeConformanceError::NativeCaseFailed(failed) if failed == case),
        "the gate names the case that failed",
    );
}

/// `G11-R05` **CLOSED**: a report whose only defect is its own summary
/// is refused.
///
/// The case loop and the required-row loop between them catch every
/// failure this harness can compute, so the completeness check is the
/// arm that catches anything they do not enumerate. It is reachable
/// only by presenting a report whose summary disagrees with its rows,
/// which is why the report is wrapped rather than validated: the
/// validator recomputes the summary and would refuse this document
/// first.
#[test]
fn g11_r05_a_report_whose_summary_says_failed_is_refused() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let (plan, registry) = plan_and_registry();
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&fixtures),
        responses: answers(&fixtures),
    });
    let mut report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");

    // The run itself passed: every case, every required claim, every
    // required row.
    gate(
        &ValidatedNativeConformanceReport::wrap_for_tests(report.clone()),
        Some(&expected_provenance()),
    )
    .expect("an honest complete run is evidence");

    report.summary.completeness = ReportCompleteness::Failed;
    let refusal = gate(
        &ValidatedNativeConformanceReport::wrap_for_tests(report),
        Some(&expected_provenance()),
    )
    .expect_err("no gate returns success for a report whose summary says failed");
    assert!(matches!(
        refusal,
        NativeConformanceError::ReportSummaryFailed
    ));
}

/// `G11-R05` **CLOSED**: a failed canonical case is refused on the
/// ordinary evidence path, with no test constructor involved.
#[test]
fn g11_r05_a_failed_canonical_case_never_reaches_evidence() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let (plan, registry) = plan_and_registry();

    // One case answered with the opposite verdict, and nothing else
    // touched.
    let mut responses = answers(&fixtures);
    let victim = fixtures
        .iter()
        .next()
        .expect("the canonical census is nonempty")
        .case();
    let answer = responses.get_mut(&victim).expect("the case was answered");
    answer.verdict = match answer.verdict {
        NativeVerdict::Accepted => NativeVerdict::Rejected,
        _ => NativeVerdict::Accepted,
    };
    answer.observed_failure = None;

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&fixtures),
        responses,
    });
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");
    assert_eq!(report.summary.cases_failed, 1);

    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &binding,
            fixtures: &fixtures,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )
    .expect("the report faithfully describes the run");
    gate(&validated, Some(&expected_provenance()))
        .expect_err("a run with a failed case is not evidence");
}

// -- G11-R06 (CLOSED by Wave 3) ----------------------------------------

/// One run whose handshake has been altered, taken as far as the gate.
///
/// The alteration is applied to the provenance the executor reports;
/// everything else is the honest canonical run. So a refusal here is the
/// gate's provenance comparison and nothing else — the census, the
/// subjects, the answers, and the environment are all the ones the
/// passing case uses.
fn gate_with_handshake(
    alter: impl FnOnce(&mut crate::protocol::ExecutorHandshake),
) -> Result<(), NativeConformanceError> {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let (plan, registry) = plan_and_registry();

    let mut handshake = nonmock_handshake();
    alter(&mut handshake);

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake,
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&fixtures),
        responses: answers(&fixtures),
    });
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");
    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &binding,
            fixtures: &fixtures,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )
    .expect("the report faithfully describes the run");
    gate(&validated, Some(&expected_provenance()))
}

/// Whether a refusal is the provenance comparison refusing.
fn is_provenance_refusal(result: &Result<(), NativeConformanceError>, expected: ProvenanceDefect) {
    match result {
        Err(NativeConformanceError::ExecutorProvenanceUnestablished(defect)) => {
            assert_eq!(*defect, expected);
        }
        other => panic!("expected a provenance refusal naming {expected:?}, got {other:?}"),
    }
}

/// `G11-R06` **CLOSED**: a run that records no binary provenance is
/// refused, whether the fields are absent or blank.
///
/// `establishes_workspace_provenance` used to be the whole of the
/// harness's opinion here, and no gate called it. It was also too weak
/// to be worth calling: three `Some` values satisfied it whatever they
/// held, so `Some("")` passed. The gate now compares typed values
/// against an explicit expectation, and both shapes fail — the absent
/// one as a missing field, the blank one as a field whose contents are
/// not a revision.
#[test]
fn g11_r06_a_run_with_no_workspace_provenance_is_refused() {
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.binary_reported_revision = None;
            handshake.intended_executed_tip = None;
            handshake.upstream_base = None;
            handshake.included_local_topics = BTreeSet::new();
        }),
        ProvenanceDefect::MissingBinaryRevision,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.binary_reported_revision = Some(String::new());
            handshake.intended_executed_tip = Some(String::new());
            handshake.upstream_base = Some(String::new());
            handshake.included_local_topics = BTreeSet::new();
        }),
        ProvenanceDefect::MalformedBinaryRevision(ProvenanceSyntaxDefect::RevisionTooShort),
    );
}

/// `G11-R06` **CLOSED**: each provenance field is required on its own.
///
/// The guide's regression list, one field at a time: a run missing any
/// one of them is refused naming that one, rather than passing because
/// the others were present.
#[test]
fn g11_r06_each_missing_provenance_field_is_refused_by_name() {
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.binary_reported_revision = None),
        ProvenanceDefect::MissingBinaryRevision,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.intended_executed_tip = None),
        ProvenanceDefect::MissingIntendedTip,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.upstream_base = None),
        ProvenanceDefect::MissingUpstreamBase,
    );
}

/// `G11-R06` **CLOSED**: a blank name or version is refused.
///
/// Five provenance roles, and a blank string in any of the four naming
/// ones is an executor that did not answer the question.
#[test]
fn g11_r06_a_blank_name_or_version_is_refused() {
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.adapter_name = String::new()),
        ProvenanceDefect::BlankAdapterName,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.adapter_version = "   ".to_owned()),
        ProvenanceDefect::BlankAdapterVersion,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.node_name = String::new()),
        ProvenanceDefect::BlankNodeName,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.node_version = "\t".to_owned()),
        ProvenanceDefect::BlankNodeVersion,
    );
}

/// `G11-R06` **CLOSED**: a binary revision contradicting the intended
/// tip is refused.
///
/// This is the case ADR-018 names outright: a binary whose embedded
/// revision does not match the intended tip is refused for evidence.
#[test]
fn g11_r06_a_contradictory_revision_pair_is_refused() {
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.binary_reported_revision =
                Some("0000000000000000000000000000000000000000".to_owned());
        }),
        ProvenanceDefect::BinaryRevisionIsNotTheIntendedTip,
    );

    // And a prefix that is *nearly* the tip: one digit different at the
    // last position of the abbreviation, which arbitrary text equality
    // would also have caught but which a "starts with something" rule
    // would not.
    let mut near = TEST_BINARY_REVISION.to_owned();
    near.pop();
    near.push('f');
    assert_ne!(near, TEST_BINARY_REVISION);
    is_provenance_refusal(
        &gate_with_handshake(|handshake| handshake.binary_reported_revision = Some(near)),
        ProvenanceDefect::BinaryRevisionIsNotTheIntendedTip,
    );
}

/// `G11-R06` **CLOSED**: an intended tip or upstream base that is not
/// the expected one is refused.
#[test]
fn g11_r06_a_run_against_another_tip_or_base_is_refused() {
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.intended_executed_tip =
                Some("ffffffffffffffffffffffffffffffffffffffff".to_owned());
        }),
        ProvenanceDefect::IntendedTipIsNotTheExpectedTip,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.upstream_base = Some("ffffffffffffffffffffffffffffffffffffffff".to_owned());
        }),
        ProvenanceDefect::UpstreamBaseIsNotTheExpectedBase,
    );

    // An abbreviated intended tip is refused as well. The expectation is
    // a full identifier and the operator's own declaration is compared
    // against it exactly: the prefix rule exists for what the *binary*
    // reports, which is the one field that cannot carry more.
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.intended_executed_tip = Some(TEST_BINARY_REVISION.to_owned());
        }),
        ProvenanceDefect::IntendedTipIsNotTheExpectedTip,
    );
}

/// `G11-R06` **CLOSED**: a topic census that is not the expected one is
/// refused, in either direction.
///
/// Equality rather than containment. A tip that folded in an extra local
/// branch is not the tip that was reviewed, and one that folded in fewer
/// is not it either.
#[test]
fn g11_r06_a_wrong_topic_census_is_refused() {
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.included_local_topics = BTreeSet::new();
        }),
        ProvenanceDefect::TopicCensusIsNotTheExpectedCensus,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake
                .included_local_topics
                .insert("fix/unexpected".to_owned());
        }),
        ProvenanceDefect::TopicCensusIsNotTheExpectedCensus,
    );
    is_provenance_refusal(
        &gate_with_handshake(|handshake| {
            handshake.included_local_topics = BTreeSet::from(["fix/other".to_owned()]);
        }),
        ProvenanceDefect::TopicCensusIsNotTheExpectedCensus,
    );
}

/// `G11-R06` **CLOSED**: the honest run passes, by prefix and in full.
///
/// The positive half of the regression list. The default handshake
/// reports the narrowest admitted abbreviation, which is the shape a
/// real node binary embeds; the same run reporting the full identifier
/// passes too, because equality is the width-forty case of the one
/// prefix rule rather than a second rule.
#[test]
fn g11_r06_a_correct_revision_passes_abbreviated_and_in_full() {
    gate_with_handshake(|_| {}).expect("the honest run is evidence");
    gate_with_handshake(|handshake| {
        handshake.binary_reported_revision = Some(TEST_INTENDED_TIP.to_owned());
    })
    .expect("a binary reporting the full identifier is evidence");

    // And an abbreviation wider than the minimum, which is compared
    // against more digits rather than fewer.
    gate_with_handshake(|handshake| {
        handshake.binary_reported_revision = Some(TEST_INTENDED_TIP[..12].to_owned());
    })
    .expect("a wider abbreviation is evidence");
}

/// `G11-R06` **CLOSED**: a run with no configured expectation cannot be
/// gated at all.
///
/// Fail-closed. A caller who configured no expectation has not satisfied
/// the comparison; they have skipped it, and the gate says so rather
/// than treating an absent operand as an agreeing one.
#[test]
fn g11_r06_a_run_with_no_expectation_is_refused() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let (plan, registry) = plan_and_registry();
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&fixtures),
        responses: answers(&fixtures),
    });
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");
    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &binding,
            fixtures: &fixtures,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )
    .expect("the report validates");

    assert!(matches!(
        gate(&validated, None).expect_err("an unstated expectation is not a satisfied one"),
        NativeConformanceError::ExpectedProvenanceUnavailable,
    ));
}

/// `G11-R06` **CLOSED**: an experimental run keeps incomplete provenance
/// and still reaches no gate.
///
/// The last item of the guide's regression list. An experiment may
/// record whatever provenance it has — that is what makes it useful —
/// and the type system is what keeps it away from the gate: the
/// experimental path yields a report with no validator, so there is no
/// value the gate would accept.
#[test]
fn g11_r06_an_experimental_run_may_keep_incomplete_provenance() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let (plan, registry) = plan_and_registry();

    let mut handshake = nonmock_handshake();
    handshake.binary_reported_revision = None;
    handshake.intended_executed_tip = None;
    handshake.upstream_base = None;

    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake,
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: subjects_of(&fixtures),
        responses: answers(&fixtures),
    });
    let experimental = evaluate_experimental(
        &target,
        &binding,
        fixtures.fixtures(),
        &transcript,
        &plan,
        &registry,
    )
    .expect("the experiment evaluates");
    let report = experimental.report();
    assert!(
        !report.executor.establishes_workspace_provenance(),
        "the experiment records that its provenance is incomplete",
    );
    assert_eq!(report.role, PrototypeReportRole::ExperimentalPrimitive);
}

// -- G11-R03 (CLOSED by Wave 1) ---------------------------------------

/// One forged wide-floor row: a bare true leaf carrying every claim of
/// the relation.
///
/// Locally coherent in every way the fixture language can check — the
/// tree contains the executing leaf, the predecessor program is the one
/// that tree determines, the wide-floor relation requires no output of
/// any role, and all eleven claims belong to the relation.
fn forged_wide_floor_row(target: &ReviewedElementsTapscriptDefinition) -> CompoundPrototypeFixture {
    let program = pushes(target, 0x01);
    let script = program.encode(target);
    let leaf = FixtureTapTree::leaf(script.clone());
    let built =
        construct(&UNSPENDABLE_INTERNAL_KEY, &leaf, &leaf).expect("the bare leaf is constructible");
    let claims: BTreeSet<PrototypeClaim> = PrototypeClaim::ALL
        .iter()
        .copied()
        .filter(|claim| claim.relation() == PrototypeRelation::WideFloorRelation)
        .collect();
    assert_eq!(
        claims.len(),
        11,
        "the wide-floor relation owns eleven claims"
    );

    CompoundPrototypeFixture {
        case: PrototypeCaseId {
            relation: PrototypeRelation::WideFloorRelation,
            name: "forged".to_owned(),
        },
        claims,
        target_contract_version: target.definition().version().get(),
        script,
        initial_stack: Vec::new(),
        construction: PrototypeConstruction {
            internal_key: UNSPENDABLE_INTERNAL_KEY,
            tree: leaf.clone(),
            executing_leaf: leaf,
            control: Some(built.control_block().to_vec()),
            predecessor_program: built.output_program().to_vec(),
            outputs: Vec::new(),
        },
        expected: ExpectedPrototypeOutcome::Accepted,
        expected_resources: recorded_only(),
    }
}

/// A transcript answering every row of one matrix as its fixture requires.
fn prototype_answers(
    matrix: &[CompoundPrototypeFixture],
) -> BTreeMap<PrototypeCaseId, NativePrototypeResponse> {
    matrix
        .iter()
        .map(|fixture| {
            let verdict = match fixture.expected {
                ExpectedPrototypeOutcome::Accepted => NativeVerdict::Accepted,
                ExpectedPrototypeOutcome::Rejected => NativeVerdict::Rejected,
            };
            (
                fixture.case.clone(),
                NativePrototypeResponse {
                    schema: NATIVE_PROTOCOL_SCHEMA,
                    case: fixture.case.clone(),
                    verdict,
                    final_stack: None,
                    final_altstack: None,
                    observed_failure: None,
                    resources: NativeResourceObservation::default(),
                },
            )
        })
        .collect()
}

/// The run over one matrix, as a transcript.
fn prototype_transcript(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    matrix: &[CompoundPrototypeFixture],
) -> ExecutionTranscript {
    ExecutionTranscript::prototypes_for_tests(PrototypeTranscriptParts {
        target,
        binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: prototype_subjects_of(matrix),
        responses: prototype_answers(matrix),
    })
}

/// `G11-R03` **CLOSED**: a trivial true leaf cannot certify the
/// wide-floor relation.
///
/// The forgery is still coherent, and still evaluable — as an experiment.
/// What it cannot be is a matrix: the gate's input is a
/// [`WideFloorPrototypeMatrix`] whose rows are private and whose only
/// constructor is the canonical generator, and the evidence path
/// regenerates that matrix and compares it row for row.
#[test]
fn g11_r03_a_trivial_leaf_cannot_certify_the_wide_floor_relation() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let forgery = forged_wide_floor_row(&target);
    assert!(
        forgery.defect(&target).is_none(),
        "the forgery is locally coherent, which is exactly why coherence was never enough",
    );

    let matrix = vec![forgery];
    let transcript = prototype_transcript(&target, &binding, &matrix);

    // The experimental path still describes the run, and says what it is.
    let experimental = evaluate_experimental_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &matrix,
        &transcript,
    )
    .expect("an ad hoc matrix is reportable as an experiment");
    assert_eq!(
        experimental.report().role,
        PrototypeReportRole::ExperimentalPrototype,
        "the report states that its subject was a caller's choice",
    );

    // And there is no route from it to the gate: the only report
    // `prototype_gate` reads is a validated one, and the only matrix the
    // validator accepts is the canonical wrapper, which refuses this.
    let forged = WideFloorPrototypeMatrix::wrap_for_tests(matrix.clone());
    let refusal = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&forged),
        &transcript,
    )
    .expect_err("a matrix the generator did not state is not an evidence subject");
    assert!(
        matches!(
            refusal,
            NativeConformanceError::NoncanonicalPrototypeMatrix
                | NativeConformanceError::NoncanonicalPrototypeCase(_)
        ),
        "the refusal names the subject: {refusal:?}",
    );

    // The canonical matrix is a different value entirely, and now the
    // evidence path is the thing that compares the two.
    let canonical = wide_floor_case_matrix(&target).expect("the canonical matrix states");
    assert_ne!(matrix.as_slice(), canonical.rows());
}

/// `G11-R03` **CLOSED**: the canonical wide-floor matrix is what gates.
///
/// The companion to the refusals below: without this, a harness that
/// refused every matrix would pass them all.
#[test]
fn g11_r03_the_canonical_wide_floor_matrix_is_the_evidence_subject() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = wide_floor_case_matrix(&target).expect("the canonical matrix states");
    let transcript = prototype_transcript(&target, &binding, canonical.rows());

    let report = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&canonical),
        &transcript,
    )
    .expect("the canonical matrix evaluates");
    assert_eq!(report.role, PrototypeReportRole::WideFloor);
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::CompleteForWideFloorPrototype,
    );

    let validated = validate_prototype_report(
        report,
        PrototypeReportValidationInputs {
            target: &target,
            binding: &binding,
            matrix: CanonicalPrototypeMatrix::WideFloor(&canonical),
            transcript: &transcript,
        },
    )
    .expect("the canonical report validates");
    prototype_gate(&validated, Some(&expected_provenance()))
        .expect("the canonical matrix is prototype evidence");
}

/// `G11-R03` **CLOSED**: a canonical case with one added claim fails.
#[test]
fn g11_r03_a_canonical_case_with_one_added_claim_fails() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = wide_floor_case_matrix(&target).expect("the canonical matrix states");

    let mut rows = canonical.rows().to_vec();
    let extra = PrototypeClaim::ALL
        .iter()
        .copied()
        .find(|claim| {
            claim.relation() == PrototypeRelation::WideFloorRelation
                && !rows[0].claims.contains(claim)
        })
        .expect("some wide-floor claim the first row does not already carry");
    rows[0].claims.insert(extra);
    assert!(
        rows[0].defect(&target).is_none(),
        "the added claim belongs to the relation, so coherence still holds",
    );

    let transcript = prototype_transcript(&target, &binding, &rows);
    let case = rows[0].case.clone();
    let forged = WideFloorPrototypeMatrix::wrap_for_tests(rows);
    let refusal = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&forged),
        &transcript,
    )
    .expect_err("a case bearing a claim the canonical case does not is not that case");
    assert_eq!(
        format!("{refusal:?}"),
        format!(
            "{:?}",
            NativeConformanceError::NoncanonicalPrototypeCase(case)
        ),
    );
}

/// `G11-R03` **CLOSED**: a canonical case with one removed claim fails.
#[test]
fn g11_r03_a_canonical_case_with_one_removed_claim_fails() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = wide_floor_case_matrix(&target).expect("the canonical matrix states");

    let mut rows = canonical.rows().to_vec();
    let index = rows
        .iter()
        .position(|row| row.claims.len() > 1)
        .expect("some canonical row bears on more than one claim");
    let dropped = *rows[index]
        .claims
        .iter()
        .next()
        .expect("the row bears on a claim");
    rows[index].claims.remove(&dropped);

    let transcript = prototype_transcript(&target, &binding, &rows);
    let case = rows[index].case.clone();
    let forged = WideFloorPrototypeMatrix::wrap_for_tests(rows);
    let refusal = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&forged),
        &transcript,
    )
    .expect_err("a case bearing fewer claims than the canonical case is not that case");
    assert_eq!(
        format!("{refusal:?}"),
        format!(
            "{:?}",
            NativeConformanceError::NoncanonicalPrototypeCase(case)
        ),
    );
}

/// `G11-R03` **CLOSED**: replacing the canonical prototype program while
/// retaining the case name fails.
///
/// The case name was never the identity — the report binds the complete
/// fixture — and this is what makes that true of the gate as well.
#[test]
fn g11_r03_replacing_the_canonical_program_under_the_same_case_name_fails() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = wide_floor_case_matrix(&target).expect("the canonical matrix states");

    let mut rows = canonical.rows().to_vec();
    let case = rows[0].case.clone();
    let substitute = forged_wide_floor_row(&target);
    rows[0].script = substitute.script.clone();
    rows[0].construction = substitute.construction;
    assert!(
        rows[0].defect(&target).is_none(),
        "the substituted construction is internally coherent",
    );

    let transcript = prototype_transcript(&target, &binding, &rows);
    let forged = WideFloorPrototypeMatrix::wrap_for_tests(rows);
    let refusal = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&forged),
        &transcript,
    )
    .expect_err("a case name is not a subject");
    assert_eq!(
        format!("{refusal:?}"),
        format!(
            "{:?}",
            NativeConformanceError::NoncanonicalPrototypeCase(case)
        ),
    );
}

/// `G11-R03` **CLOSED**: replacing the constructor successor program with
/// an arbitrary nonempty program fails canonical-matrix validation.
///
/// The successor's program is the one field a constructor fixture cannot
/// derive from its own tree — it belongs to the successor's construction —
/// so `defect` deliberately leaves it free. That is exactly why it has to
/// be pinned by the canonical comparison instead.
#[test]
fn g11_r03_replacing_the_constructor_successor_program_fails() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let canonical = constructor_case_matrix(&target).expect("the canonical matrix states");

    let mut rows = canonical.rows().to_vec();
    let index = rows
        .iter()
        .position(|row| !row.construction.outputs.is_empty())
        .expect("a constructor row states a successor output");
    let case = rows[index].case.clone();
    rows[index].construction.outputs[0].program = vec![0x51];
    assert!(
        rows[index].defect(&target).is_none(),
        "an arbitrary nonempty successor program is locally coherent",
    );

    let transcript = prototype_transcript(&target, &binding, &rows);
    let forged = ConstructorPrototypeMatrix::wrap_for_tests(rows);
    let refusal = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::Constructor(&forged),
        &transcript,
    )
    .expect_err("an arbitrary successor program is not the canonical one");
    assert_eq!(
        format!("{refusal:?}"),
        format!(
            "{:?}",
            NativeConformanceError::NoncanonicalPrototypeCase(case)
        ),
    );
}

/// `G11-R03` **CLOSED**: a prototype report does not claim typed-program
/// provenance for raw caller-supplied bytes.
///
/// The projection used to stamp every compound row `TypedProgram`. It now
/// decodes the bytes through the reviewed contract and re-encodes them,
/// so the claim holds exactly when the bytes are some typed program's own
/// encoding. These are not: a lone push prefix with no payload is a
/// truncated instruction.
#[test]
fn g11_r03_raw_bytes_are_not_reported_as_a_typed_program() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    // Bytes no typed program encodes: a push prefix with no payload.
    let script = vec![0x02, 0x01];
    let leaf = FixtureTapTree::leaf(script.clone());
    let built =
        construct(&UNSPENDABLE_INTERNAL_KEY, &leaf, &leaf).expect("the bare leaf is constructible");
    let case = PrototypeCaseId {
        relation: PrototypeRelation::WideFloorRelation,
        name: "raw-bytes".to_owned(),
    };
    let fixture = CompoundPrototypeFixture {
        case: case.clone(),
        claims: BTreeSet::from([PrototypeClaim::WideFloorExactDivisionObserved]),
        target_contract_version: target.definition().version().get(),
        script,
        initial_stack: Vec::new(),
        construction: PrototypeConstruction {
            internal_key: UNSPENDABLE_INTERNAL_KEY,
            tree: leaf.clone(),
            executing_leaf: leaf,
            control: Some(built.control_block().to_vec()),
            predecessor_program: built.output_program().to_vec(),
            outputs: Vec::new(),
        },
        expected: ExpectedPrototypeOutcome::Rejected,
        expected_resources: recorded_only(),
    };
    let matrix = vec![fixture];
    let transcript = ExecutionTranscript::prototypes_for_tests(PrototypeTranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::ReviewedNonMock,
        requests: prototype_subjects_of(&matrix),
        responses: BTreeMap::from([(
            case.clone(),
            NativePrototypeResponse {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case,
                verdict: NativeVerdict::Rejected,
                final_stack: None,
                final_altstack: None,
                observed_failure: None,
                resources: NativeResourceObservation::default(),
            },
        )]),
    });
    let report = evaluate_experimental_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &matrix,
        &transcript,
    )
    .expect("the matrix evaluates as an experiment");
    assert_eq!(
        report.report().cases[0].fixture.script_source,
        FixtureScriptSource::DeliberatelyMalformed,
        "bytes no typed program encodes are not reported as a typed program",
    );

    // The canonical rows, whose scripts are emitted programs' own
    // encodings, still report the provenance they actually have.
    let canonical = wide_floor_case_matrix(&target).expect("the canonical matrix states");
    let honest = prototype_transcript(&target, &binding, canonical.rows());
    let canonical_report = evaluate_prototypes(
        &target,
        &binding,
        CanonicalPrototypeMatrix::WideFloor(&canonical),
        &honest,
    )
    .expect("the canonical matrix evaluates");
    assert!(
        canonical_report
            .cases
            .iter()
            .all(|case| case.fixture.script_source == FixtureScriptSource::TypedProgram),
        "every canonical row's bytes are one typed program's own encoding",
    );
}

// -- G11-R14 -----------------------------------------------------------

/// The static subtree the byte-comparison below constructs over.
fn r14_static_subtree() -> FixtureTapTree {
    FixtureTapTree::leaf(vec![0x51])
}

/// `G11-R14`: a permanent defect is classified as one, not ground.
///
/// Only a tree defect used to short-circuit. An internal key that is
/// not a curve point is invariant under every metadata nonce — it is
/// decoded before the root is consulted at all — so the search ran to
/// its bound and reported exhaustion, and a caller reading that
/// diagnostic would raise the retry limit against a failure no limit
/// can repair.
#[test]
fn g11_r14_an_invalid_internal_key_fails_after_one_attempt() {
    // Above the field prime, so no x-only lift exists for any nonce.
    let invalid_key = [0xff_u8; FIELD_ELEMENT_BYTES];
    let leaf = r14_static_subtree();
    let seen = std::cell::Cell::new(0_u32);

    let defect = construct_under_policy(
        &invalid_key,
        metadata(),
        &leaf,
        TweakTotalityPolicy::CanonicalNonceRetry {
            maximum_attempts: 8,
        },
        |_metadata| {
            seen.set(seen.get() + 1);
            leaf.clone()
        },
    )
    .expect_err("an invalid internal key determines no output key");

    assert!(
        matches!(
            defect,
            TotalityDefect::NotRepairableByRetry(ConstructionDefect::Tweak(
                TweakDefect::InternalKeyNotOnCurve(_),
            )),
        ),
        "a permanent defect is classified as permanent, got {defect:?}",
    );
    // The count is the point of the repair, not a detail of it: the old
    // behaviour did this work eight times over and then misreported it.
    assert_eq!(seen.get(), 1, "the search stops after the first attempt");
}

/// `G11-R14`: the canonical ordered constructor classifies it too.
///
/// The two retry implementations now consult one predicate, so this is
/// the same fact reached by the other route.
#[test]
fn g11_r14_the_canonical_constructor_classifies_the_same_permanent_defect() {
    let target = reviewed_target();
    let invalid_key = [0xff_u8; FIELD_ELEMENT_BYTES];
    let static_subtree = r14_static_subtree();

    let defect = construct_canonically_ordered(
        &target,
        &invalid_key,
        &metadata(),
        &static_subtree,
        &static_subtree,
        4,
    )
    .expect_err("an invalid internal key determines no output key");

    assert!(
        matches!(
            defect,
            CanonicalOrderDefect::NotRepairableByRetry(ConstructionDefect::Tweak(
                TweakDefect::InternalKeyNotOnCurve(_),
            )),
        ),
        "a permanent defect is classified as permanent, got {defect:?}",
    );
}

/// `G11-R14`: a tree the nonce cannot fix fails after one attempt.
///
/// A missing executing leaf and a repeated one are both properties of
/// the static subtree, which the nonce does not touch. Both were
/// already short-circuited; both are checked here so the shared
/// predicate cannot lose them while gaining the internal-key case.
#[test]
fn g11_r14_a_tree_defect_fails_after_one_attempt() {
    let present = FixtureTapTree::leaf(vec![0x51]);
    let absent = FixtureTapTree::leaf(vec![0x52]);
    let repeated = FixtureTapTree::branch(present.clone(), present.clone());

    for (tree, executing, expected) in [
        (
            present.clone(),
            absent.clone(),
            TreeDefect::ExecutingLeafAbsent,
        ),
        (
            repeated.clone(),
            present.clone(),
            TreeDefect::ExecutingLeafRepeated,
        ),
    ] {
        let seen = std::cell::Cell::new(0_u32);
        let defect = construct_under_policy(
            &UNSPENDABLE_INTERNAL_KEY,
            metadata(),
            &executing,
            TweakTotalityPolicy::CanonicalNonceRetry {
                maximum_attempts: 8,
            },
            |_metadata| {
                seen.set(seen.get() + 1);
                tree.clone()
            },
        )
        .expect_err("the tree determines no control path");

        assert_eq!(
            defect,
            TotalityDefect::NotRepairableByRetry(ConstructionDefect::Tree(expected)),
        );
        assert_eq!(seen.get(), 1, "the search stops after the first attempt");
    }
}

/// `G11-R14`: exactly the two nonce-movable tweak defects retry.
///
/// The classification stated directly, over every defect the
/// constructor can produce. Getting it wrong in the other direction
/// would be worse than the finding: marking every tweak failure
/// permanent would refuse instances a second nonce would have built,
/// since the tweak is a hash of a root the nonce moves.
#[test]
fn g11_r14_only_the_nonce_movable_tweak_defects_are_retryable() {
    for defect in [
        ConstructionDefect::Tweak(TweakDefect::TweakNotAScalar),
        ConstructionDefect::Tweak(TweakDefect::TweakedKeyIsIdentity),
    ] {
        assert!(retryable(defect), "{defect:?} moves with the nonce");
    }

    for defect in [
        ConstructionDefect::Tweak(TweakDefect::InternalKeyNotOnCurve(
            PointDecodingDefect::NotAFieldElement,
        )),
        ConstructionDefect::Tweak(TweakDefect::InternalKeyNotOnCurve(
            PointDecodingDefect::NotOnCurve,
        )),
        ConstructionDefect::Tree(TreeDefect::ExecutingLeafAbsent),
        ConstructionDefect::Tree(TreeDefect::ExecutingLeafIsNotALeaf),
        ConstructionDefect::Tree(TreeDefect::ExecutingLeafRepeated),
        ConstructionDefect::Tree(TreeDefect::PathTooDeep { needed: 129 }),
    ] {
        assert!(!retryable(defect), "{defect:?} is fixed under the nonce");
    }
}

/// `G11-R14`: an exhausted search still reports its configured bound.
///
/// The repair narrows what is retried, and must not turn a genuine
/// exhausted search into something else. The static subtree is chosen
/// here rather than assumed: a root every nonce in the bound hashes
/// above is searched for, so the exhaustion is deterministic instead of
/// being a coin flip the test happens to win.
#[test]
fn g11_r14_exhaustion_reports_the_configured_attempt_count() {
    let target = reviewed_target();
    let attempts = 3_u32;

    // The metadata leaf hashes the search will produce, in order.
    let leaves = (0..attempts)
        .map(|nonce| {
            let written = metadata().with_nonce(nonce);
            let script = metadata_leaf_script(&target, &written.encode())
                .expect("the metadata is expressible");
            FixtureTapTree::leaf(script).node_hash()
        })
        .collect::<Vec<_>>();

    // A static subtree whose root precedes every one of them, so the
    // MetadataFirst side never holds within the bound.
    let subtree = (0_u16..4096)
        .map(|discriminant| {
            FixtureTapTree::leaf(
                vec![0x51, 0x01, 0x02, 0x03, 0x04]
                    .into_iter()
                    .chain(discriminant.to_be_bytes())
                    .collect::<Vec<_>>(),
            )
        })
        .find(|candidate| {
            let root = candidate.node_hash();
            leaves.iter().all(|leaf| *leaf > root)
        })
        .expect("a root below every leaf hash in the bound exists");

    let defect = construct_canonically_ordered(
        &target,
        &UNSPENDABLE_INTERNAL_KEY,
        &metadata(),
        &subtree,
        &subtree,
        attempts,
    )
    .expect_err("no nonce in the bound is on the canonical side");

    assert_eq!(
        defect,
        CanonicalOrderDefect::SearchExhausted { attempts },
        "an exhausted search reports exactly the bound it was given",
    );
}

/// `G11-R14`: the canonical construction produces identical outputs.
///
/// The repair changes which defects are retried and must change nothing
/// about what a successful construction determines. These values were
/// read off the constructor before the repair and are compared against
/// it after: a byte comparison, not a re-derivation, since a value
/// recomputed by the code under test would agree with itself however it
/// had moved.
#[test]
fn g11_r14_the_canonical_construction_is_byte_identical() {
    let target = reviewed_target();
    let subtree = r14_static_subtree();

    let built = construct_canonically_ordered(
        &target,
        &UNSPENDABLE_INTERNAL_KEY,
        &metadata(),
        &subtree,
        &subtree,
        1_000,
    )
    .expect("the canonical search finds a nonce");
    let output = built.output();

    assert_eq!(built.attempts(), 1);
    assert_eq!(built.metadata().nonce, 0);
    assert_eq!(
        hex_of(output.merkle_root()),
        "36e51d9fba0544e54bf2d5adfbb2aa2f51da649a6f2619cef0413e7ca0233762",
    );
    assert_eq!(
        hex_of(output.tweak()),
        "f42dc12bf8e91866d7b2150d8e026bb1ce863487f30164aab51cd16496d7c9aa",
    );
    assert_eq!(
        hex_of(output.output_key()),
        "d3ea922e605c6bf7d0c4a43f863329067ed689003bc63e3964ff53ee45cf182e",
    );
    assert_eq!(output.parity(), 1);
    assert_eq!(
        hex_of(output.output_program()),
        "5120d3ea922e605c6bf7d0c4a43f863329067ed689003bc63e3964ff53ee45cf182e",
    );
    assert_eq!(
        hex_of(output.control_block()),
        "c550929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0\
         53118bf6314510facb28ffd10dc491df2046d6e59ba3b014be9b03ef1241a404",
    );
}

/// Lowercase hex, for comparing exact bytes against a written record.
fn hex_of(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// The registry and plan are stated once here so an unused-import warning
/// cannot hide a reproduction that stopped exercising them.
#[test]
fn the_reproduction_inputs_state() {
    let registry: ClaimRegistry = claim_registry().expect("the claim census is coherent");
    assert!(!registry.is_empty());
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    assert!(
        plan.class(TargetEvidenceRequirementId::SighashSemantics)
            == Some(EvidencePlanClass::UnresolvedByDesign),
    );
}
