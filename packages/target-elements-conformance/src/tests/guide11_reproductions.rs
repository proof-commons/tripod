//! Guide-11 Wave-0 reproductions of the sixth static review.
//!
//! # These tests assert the defect, not the repair
//!
//! Every test here demonstrates a finding from the Guide-11 preflight
//! register by *passing* while the defect is present: it asserts that the
//! wrong thing happens. Waves 1 to 4 flip each assertion as they repair
//! the finding, so a test here failing after a repair is the repair
//! working rather than a regression.
//!
//! Each test names its finding identifier in its own documentation. No
//! test here touches a production code path: they are constructions over
//! the public and crate-visible surfaces exactly as an external caller or
//! the existing suites reach them.

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
use crate::executor::{ExecutionTranscript, ExecutorTrust};
use crate::fixture::{
    EnforcementLayer, ExpectedPrimitiveOutcome, ExpectedResourceObservation, FixtureScript,
    FixtureScriptSource, FixtureStatement, LeafVersionStatus, NativeCaseGroup, NativeCaseId,
    PrimitiveFixture, PrimitiveFixtureSet, ResourceExpectation, canonical_fixture_set,
};
use crate::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeExecutionResponse, NativePrototypeResponse,
    NativeResourceObservation, NativeVerdict, ObservedFailureClass,
};
use crate::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeCaseId, PrototypeClaim,
    PrototypeConstruction, PrototypeRelation, wide_floor_case_matrix,
};
use crate::prototype_report::PrototypeReportCompleteness;
use crate::prototype_validate::{
    PrototypeReportValidationInputs, evaluate_prototypes, prototype_gate, validate_prototype_report,
};
use crate::report::{CaseStatus, EvidenceDisposition, EvidencePlanClass, ReportCompleteness};
use crate::validate::{
    NativeReportValidationInputs, evaluate, gate, guide_nine_evidence_plan, validate_native_report,
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
fn answers(fixtures: &PrimitiveFixtureSet) -> BTreeMap<NativeCaseId, NativeExecutionResponse> {
    fixtures
        .iter()
        .map(|fixture| (fixture.case(), contract_answer(fixture.case(), fixture)))
        .collect()
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
    let report = evaluate(&target, &binding, &reported, &transcript, &plan, &registry)
        .expect("the defect: the substituted census evaluates against another run's answers");

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

// -- G11-R02 -----------------------------------------------------------

/// `G11-R02`: signature evidence is derived from case metadata rather
/// than from the executed script.
///
/// A program that only pushes a true literal, filed under a case whose
/// group is the signature group and whose named primitive is a signature
/// primitive, produces the signature-accepted claim and the signature
/// evidence requirement. Nothing inspects the script.
#[test]
fn g11_r02_a_trivial_true_script_bears_signature_claims() {
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
    .expect("the defect: nothing checks that the script bears on the case");

    let claims = claims_of(&fixture);
    assert!(
        claims.contains(&NativeEvidenceClaim::TransactionSignatureAccepted),
        "the defect: a literal push is credited with an accepted transaction signature",
    );
    assert!(
        claims.contains(&NativeEvidenceClaim::PrimitiveSuccessObserved),
        "the defect: a literal push is credited with a reviewed primitive succeeding",
    );
    assert!(
        !fixture
            .script()
            .contains(&target.definition().opcodes()[&signature_opcode].code()),
        "no signature primitive occurs in the script",
    );
}

/// `G11-R02`: an arbitrary caller-built census is an evaluation and
/// validation subject exactly like the canonical one.
///
/// `PrimitiveFixtureSet::new` is public and checks only that no case
/// identity repeats. There is no canonical trust state, so the evidence
/// path admits any set a caller assembles.
#[test]
fn g11_r02_an_arbitrary_census_reaches_the_evidence_path() {
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

    let fixtures = PrimitiveFixtureSet::new([fixture.clone()])
        .expect("the defect: an arbitrary census assembles");
    let canonical = canonical_fixture_set(&target, &binding).expect("the census states");
    assert_ne!(
        fixtures, canonical,
        "the arbitrary census is not the canonical one",
    );

    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        BTreeMap::from([(case, contract_answer(case, &fixture))]),
    );
    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the defect: an arbitrary census produces an evidence report");

    let signature = report
        .evidence
        .iter()
        .find(|row| row.requirement == "signature_semantics")
        .expect("the signature row exists");
    assert_ne!(
        signature.cases, 0,
        "the defect: the signature row counts a case that ran no signature primitive",
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
    .expect("the defect: the arbitrary-census report validates");
    // The gate refuses this particular run only because a one-case census
    // leaves every other required row empty, which is a completeness
    // refusal and not a subject refusal.
    assert!(gate(&validated).is_err());
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

/// `G11-R05`: the primitive gate accepts a report whose completeness is
/// failed.
///
/// A case in a group whose only evidence requirement sits outside the
/// required plan, and whose only claim is not required, can fail without
/// touching a required row or a required claim. `summarize` calls the
/// report failed; `gate` reads neither the completeness nor the case
/// statuses and returns success.
#[test]
fn g11_r05_the_gate_accepts_a_failed_report() {
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

    let plan = guide_nine_evidence_plan().expect("the plan is a partition");
    let registry = claim_registry().expect("the claim census is coherent");
    let transcript = ExecutionTranscript::for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        responses,
    );
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .expect("the run evaluates");

    assert_eq!(
        report.summary.cases_failed, 1,
        "exactly the extra case failed",
    );
    assert_eq!(
        report.summary.completeness,
        ReportCompleteness::Failed,
        "the summary calls the report failed",
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
    .expect("the honest report validates");
    gate(&validated).expect("the defect: a failed report satisfies the native gate");
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

// -- G11-R03 -----------------------------------------------------------

/// `G11-R03`: a trivial true leaf carrying every wide-floor claim is
/// gate-eligible prototype evidence.
///
/// `CompoundPrototypeFixture` has public fields, `defect` checks only
/// local coherence, and the claim set is copied from the fixture. A bare
/// leaf whose script pushes one true literal satisfies every coherence
/// rule the wide-floor relation has — it needs no output of any role —
/// and the report then credits all eleven wide-floor claims to a program
/// that computes nothing.
#[test]
fn g11_r03_a_trivial_leaf_certifies_the_whole_wide_floor_relation() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let program = pushes(&target, 0x01);
    let script = program.encode(&target);

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

    let case = PrototypeCaseId {
        relation: PrototypeRelation::WideFloorRelation,
        name: "forged".to_owned(),
    };
    let forgery = CompoundPrototypeFixture {
        case: case.clone(),
        claims: claims.clone(),
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
    };
    assert!(
        forgery.defect(&target).is_none(),
        "the defect: the forgery states a coherent case",
    );

    let matrix = vec![forgery];
    let transcript = ExecutionTranscript::prototypes_for_tests(
        nonmock_handshake(),
        observed_environment(),
        ExecutorTrust::ReviewedNonMock,
        BTreeMap::from([(
            case.clone(),
            NativePrototypeResponse {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case,
                verdict: NativeVerdict::Accepted,
                final_stack: None,
                final_altstack: None,
                observed_failure: None,
                resources: NativeResourceObservation::default(),
            },
        )]),
    );

    let report = evaluate_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &matrix,
        &transcript,
    )
    .expect("the defect: an arbitrary matrix evaluates as prototype evidence");
    assert_eq!(
        report.summary.required_claims_passed, 11,
        "the defect: every wide-floor claim passes on a literal push",
    );
    assert_eq!(
        report.summary.completeness,
        PrototypeReportCompleteness::CompleteForWideFloorPrototype,
        "the defect: the run reads as a complete wide-floor prototype",
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
    .expect("the defect: the forged report revalidates against its own matrix");
    prototype_gate(&validated).expect("the defect: the forged report satisfies the prototype gate");

    // The canonical matrix is a different value entirely, and nothing on
    // the evidence path compares the two.
    let canonical = wide_floor_case_matrix(&target).expect("the canonical matrix states");
    assert_ne!(matrix, canonical);
}

/// `G11-R03`: a prototype report claims typed-program provenance for raw
/// caller-supplied bytes.
///
/// The projection stamps every compound row `TypedProgram`, though
/// `CompoundPrototypeFixture::script` is a public byte vector and
/// `defect` never establishes that the bytes came from a typed program.
#[test]
fn g11_r03_raw_bytes_are_reported_as_a_typed_program() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    // Bytes no typed program encodes: a lone push prefix with no payload.
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
    let report = evaluate_prototypes(
        &target,
        &binding,
        PrototypeRelation::WideFloorRelation,
        &matrix,
        &transcript,
    )
    .expect("the matrix evaluates");
    assert_eq!(
        report.cases[0].fixture.script_source,
        FixtureScriptSource::TypedProgram,
        "the defect: raw bytes are reported as a typed program",
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
