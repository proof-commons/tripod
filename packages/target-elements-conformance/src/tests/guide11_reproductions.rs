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
//! - `G11-R01`, `G11-R04`, `G11-R05`, `G11-R06`, and `G11-R14` are still
//!   open. Their tests still assert the defect, so one of them failing
//!   after a later wave is that wave working rather than a regression.
//!
//! Each test names its finding identifier in its own documentation. No
//! test here touches a production code path: they are constructions over
//! the public and crate-visible surfaces exactly as an external caller or
//! the existing suites reach them.
//!
//! # What Wave 1 changed about the reproductions themselves
//!
//! Closing `G11-R02` removed the arbitrary-census route to the evidence
//! path, and two open findings had reproductions built on that route. The
//! `G11-R01` script-substitution reproduction now runs on the
//! experimental path, where it still shows exactly what it showed: the
//! transcript retains no request, so a report can name a script the
//! executor was never handed. The `G11-R05` reproduction cannot be
//! expressed at all any more — its construction added a fixture to the
//! canonical census — so what stands in its place records that the route
//! is closed and that the gate defect it named is still open and still
//! Wave 3's. Neither adaptation repairs the finding it belongs to.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    TargetEvidenceRequirementId, validate_reviewed_development_binding,
};

use crate::claim::{ClaimRegistry, NativeEvidenceClaim, claim_registry, claims_of};
use crate::constructor::canonical::{CanonicalOrderDefect, construct_canonically_ordered};
use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::metadata::PrototypeMetadata;
use crate::constructor::totality::{TotalityDefect, TweakTotalityPolicy, construct_under_policy};
use crate::constructor::tree::{FixtureTapTree, construct};
use crate::error::NativeConformanceError;
use crate::executor::{ExecutionTranscript, ExecutorTrust};
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
use crate::report::{
    CaseStatus, EvidenceDisposition, EvidencePlanClass, PrototypeReportRole, ReportCompleteness,
};
use crate::validate::{
    NativeReportValidationInputs, evaluate, evaluate_experimental, gate, guide_nine_evidence_plan,
    validate_native_report,
};

use super::support::{
    development_binding, nonmock_handshake, observed_environment, reviewed_target,
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

// -- G11-R01 -----------------------------------------------------------

/// `G11-R01`: a transcript observed under one deployment binding
/// evaluates, validates, and gates under a different one.
///
/// The transcript retains the executor's environment observation but not
/// the binding the run was requested under, and `evaluate` never compares
/// the two. The report therefore states one network and genesis as
/// declared while carrying the other as observed, and the gate accepts
/// it.
#[test]
fn g11_r01_a_transcript_rebinds_to_a_deployment_it_never_ran_on() {
    let target = reviewed_target();
    let requested = development_binding(&target);
    let substituted = other_binding(&target);

    // The run: an honest executor answering the census stated against the
    // requested binding, and reporting the environment of that binding.
    let executed = canonical_fixture_set(&target, &requested).expect("the census states");
    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        answers(&executed),
    );

    // The evaluation: the same transcript, against the census and binding
    // of a network the executor never saw.
    let rebound = canonical_fixture_set(&target, &substituted).expect("the census states");
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let report = evaluate(
        &target,
        &substituted,
        &rebound,
        &transcript,
        &plan,
        &registry,
    )
    .expect("the defect: a cross-binding evaluation is not refused");

    assert_ne!(
        report.network_id, report.observed_environment.network_id,
        "the defect: declared and observed networks differ in one report",
    );
    assert_ne!(
        report.genesis_id, report.observed_environment.genesis_id,
        "the defect: declared and observed genesis differ in one report",
    );

    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &substituted,
            fixtures: &rebound,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )
    .expect("the defect: the cross-binding report revalidates");
    gate(&validated).expect("the defect: the cross-binding report gates");
}

/// `G11-R01`: a report names a script the executor was never handed.
///
/// The transcript keys responses by case identity alone and retains no
/// request, so a fixture with the same case identity, the same encoded
/// width, and a compatible expectation takes the executed fixture's place
/// in the report. The row then presents the substituted script as the
/// complete subject the executor was handed.
///
/// The equal width matters: the fixture's exact `script_bytes`
/// expectation is compared against the executor's reported figure, so a
/// substitution of a *differently sized* script is caught by that
/// comparison. That is a width check, not a subject binding.
///
/// # Why this runs on the experimental path now
///
/// Wave 1 closed the arbitrary-census route to the evidence path, so a
/// two-fixture substitution can no longer be evaluated as evidence. The
/// defect this test names is not that route: it is that the transcript
/// retains no request, so nothing anywhere compares what was sent with
/// what is reported. That is unchanged, and it is what Wave 2 repairs.
#[test]
fn g11_r01_a_report_names_a_script_the_executor_never_ran() {
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

    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        BTreeMap::from([(case, contract_answer(case, &executed))]),
    );

    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let reported =
        PrimitiveFixtureSet::new([substituted.clone()]).expect("one fixture is a census");
    let report = evaluate_experimental(&target, &binding, &reported, &transcript, &plan, &registry)
        .expect("the defect: the substituted census evaluates against another run's answers")
        .into_report();

    assert_eq!(report.cases.len(), 1);
    assert_eq!(
        report.cases[0].status,
        CaseStatus::Passed,
        "the defect: the substituted subject passes",
    );
    assert_eq!(
        report.cases[0].fixture.script,
        substituted.script().to_vec(),
        "the defect: the report names the script that was never run",
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
    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        BTreeMap::from([(case, contract_answer(case, &fixture))]),
    );

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

    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        BTreeMap::from([(case, contract_answer(case, &fixture))]),
    );
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
    let honest = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        answers(&canonical),
    );
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
    gate(&validated).expect("the canonical census is the evidence subject");
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

    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        answers(&mutated),
    );
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

    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        answers(&mutated),
    );
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

    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        answers(&permuted),
    );
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
    gate(&validated).expect("a permuted declaration order is still evidence");
}

// -- G11-R04 -----------------------------------------------------------

/// `G11-R04`: consensus resource cases are credited to the policy
/// resource row.
///
/// `bearing_requirements` receives a case identity, not a fixture, so it
/// cannot read the enforcement layer. The canonical census states every
/// resource case at the consensus layer, and the policy row nevertheless
/// passes on their statuses while the policy claim it owns is recorded
/// unresolved.
#[test]
fn g11_r04_consensus_resource_cases_pass_the_policy_resource_row() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        answers(&fixtures),
    );
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");

    assert!(
        fixtures
            .iter()
            .filter(|fixture| fixture.case().group() == NativeCaseGroup::Resource)
            .all(|fixture| fixture.enforcement_layer() == EnforcementLayer::Consensus),
        "every canonical resource case is stated at the consensus layer",
    );

    let policy = report
        .evidence
        .iter()
        .find(|row| row.requirement == "policy_resource_limits")
        .expect("the policy resource row exists");
    assert_eq!(policy.plan, EvidencePlanClass::Required);
    assert_eq!(
        policy.disposition,
        EvidenceDisposition::Passed,
        "the defect: a required policy row passes on consensus-layer cases",
    );
    assert_ne!(
        policy.cases, 0,
        "the defect: consensus cases are counted under the policy row",
    );

    let claim = report
        .claims
        .iter()
        .find(|row| row.claim == NativeEvidenceClaim::PolicyResourceBoundObserved)
        .expect("the policy resource claim exists");
    assert_eq!(
        claim.disposition,
        EvidenceDisposition::UnresolvedByDesign,
        "the defect: the row passes while its defining claim is unresolved",
    );
    assert!(
        !claim.required,
        "the defect: a required row owns no required claim, so completeness cannot notice",
    );
}

// -- G11-R05 -----------------------------------------------------------

/// `G11-R05`: the route this finding was reproduced through is closed,
/// and the finding itself is not.
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
/// is refused before any row is computed. That is the `G11-R02` repair
/// doing its work, and it is *not* a repair of this finding: `gate` still
/// reads neither `summary.completeness` nor the case statuses. What has
/// changed is only that this particular construction can no longer reach
/// it, because every canonical case bears on a required row.
///
/// So this test records both halves — the closed route, and the summary
/// that is still failed while nothing at the gate consults it — and Wave
/// 3 owns finding a route that reaches the gate itself.
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
    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        responses,
    );

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

    // Half two: the report the run produces is still failed, and the gate
    // still has no field that would notice. The finding stands.
    let report = evaluate_experimental(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates as an experiment")
        .into_report();
    assert_eq!(
        report.summary.cases_failed, 1,
        "exactly the extra case failed",
    );
    assert_eq!(
        report.summary.completeness,
        ReportCompleteness::Failed,
        "the defect: the summary calls the report failed, and no gate reads that",
    );
    let sighash = report
        .evidence
        .iter()
        .find(|row| row.requirement == "sighash_semantics")
        .expect("the sighash row exists");
    assert_ne!(
        sighash.plan,
        EvidencePlanClass::Required,
        "the failing case touches no required row",
    );
}

// -- G11-R06 -----------------------------------------------------------

/// `G11-R06`: the gate accepts a report that records no binary
/// provenance.
///
/// `ExecutorProvenance::establishes_workspace_provenance` exists and is
/// never called by either gate. A handshake with no binary revision, no
/// intended tip, and no upstream base still produces a gate-eligible
/// report; so does one whose revision strings are blank.
#[test]
fn g11_r06_the_gate_accepts_a_run_with_no_workspace_provenance() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");

    for blank in [false, true] {
        let mut handshake = nonmock_handshake();
        if blank {
            handshake.binary_reported_revision = Some(String::new());
            handshake.intended_executed_tip = Some(String::new());
            handshake.upstream_base = Some(String::new());
        } else {
            handshake.binary_reported_revision = None;
            handshake.intended_executed_tip = None;
            handshake.upstream_base = None;
        }
        handshake.included_local_topics = BTreeSet::new();

        let transcript = ExecutionTranscript::for_tests(
            handshake,
            observed_environment(),
            ExecutorTrust::ReviewedNonMock,
            answers(&fixtures),
        );
        let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
            .expect("the run evaluates");
        assert_eq!(
            report.executor.establishes_workspace_provenance(),
            blank,
            "blank strings satisfy the predicate; absent ones do not",
        );

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
        gate(&validated).expect("the defect: provenance is never a gate condition");
    }
}

/// `G11-R06`: a report whose binary revision contradicts its intended tip
/// is gate-eligible.
///
/// The predicate compares nothing: three `Some` values satisfy it
/// whatever they say.
#[test]
fn g11_r06_a_contradictory_revision_pair_still_establishes_provenance() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = canonical_fixture_set(&target, &binding).expect("the census states");
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");

    let mut handshake = nonmock_handshake();
    handshake.binary_reported_revision =
        Some("0000000000000000000000000000000000000000".to_owned());
    handshake.intended_executed_tip = Some("ffffffffffffffffffffffffffffffffffffffff".to_owned());

    let transcript = ExecutionTranscript::for_tests(
        handshake,
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        answers(&fixtures),
    );
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");
    assert_ne!(
        report.executor.binary_reported_revision,
        report.executor.intended_executed_tip,
    );
    assert!(
        report.executor.establishes_workspace_provenance(),
        "the defect: two contradictory revisions establish workspace provenance",
    );

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
    gate(&validated).expect("the defect: the gate does not compare the two revisions");
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
fn prototype_transcript(matrix: &[CompoundPrototypeFixture]) -> ExecutionTranscript {
    ExecutionTranscript::prototypes_for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        prototype_answers(matrix),
    )
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
    let transcript = prototype_transcript(&matrix);

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
    let transcript = prototype_transcript(canonical.rows());

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
    prototype_gate(&validated).expect("the canonical matrix is prototype evidence");
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

    let transcript = prototype_transcript(&rows);
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

    let transcript = prototype_transcript(&rows);
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

    let transcript = prototype_transcript(&rows);
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

    let transcript = prototype_transcript(&rows);
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
    let transcript = ExecutionTranscript::prototypes_for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        BTreeMap::from([(
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
    );
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
    let honest = prototype_transcript(canonical.rows());
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

/// `G11-R14`: nonce retry retries an internal key no nonce can repair.
///
/// Only a tree defect short-circuits. An internal key that is not a curve
/// point is invariant under every metadata nonce, and the search runs to
/// its bound and reports exhaustion rather than the permanent
/// classification the type documents.
#[test]
fn g11_r14_an_invalid_internal_key_is_retried_to_exhaustion() {
    // Above the field prime, so no x-only lift exists for any nonce.
    let invalid_key = [0xff_u8; FIELD_ELEMENT_BYTES];
    let leaf = FixtureTapTree::leaf(vec![0x51]);
    let attempts = 8;

    let defect = construct_under_policy(
        &invalid_key,
        metadata(),
        &leaf,
        TweakTotalityPolicy::CanonicalNonceRetry {
            maximum_attempts: attempts,
        },
        |_metadata| leaf.clone(),
    )
    .expect_err("an invalid internal key determines no output key");

    assert_eq!(
        defect,
        TotalityDefect::RetryExhausted { attempts },
        "the defect: a permanent failure is reported as an exhausted search",
    );
    assert!(
        !matches!(defect, TotalityDefect::NotRepairableByRetry(_)),
        "the defect: the documented permanent classification is not used",
    );
}

/// `G11-R14`: the canonical ordered constructor has the same gap.
///
/// `construct_canonically_ordered` short-circuits on a tree defect and on
/// nothing else, so an invalid internal key exhausts the search there
/// too.
#[test]
fn g11_r14_the_canonical_constructor_retries_the_same_permanent_defect() {
    let target = reviewed_target();
    let invalid_key = [0xff_u8; FIELD_ELEMENT_BYTES];
    let static_subtree = FixtureTapTree::leaf(vec![0x51]);
    let attempts = 4;

    let defect = construct_canonically_ordered(
        &target,
        &invalid_key,
        &metadata(),
        &static_subtree,
        &static_subtree,
        attempts,
    )
    .expect_err("an invalid internal key determines no output key");

    assert!(
        matches!(defect, CanonicalOrderDefect::SearchExhausted { .. }),
        "the defect: a permanent failure is reported as an exhausted search, got {defect:?}",
    );
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
