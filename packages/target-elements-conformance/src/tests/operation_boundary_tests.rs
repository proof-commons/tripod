//! The §16.2 target-generic operation boundary.
//!
//! # Why the exchange is scripted rather than spawned
//!
//! Every property here is about the harness's own half of the exchange:
//! which step it sends, what it refuses, and what it records. A spawned
//! executor would answer those questions too, and would answer them
//! through a process whose scheduling is not the property under test. So
//! the executor's side is a written script and the harness's side is the
//! real [`run_protocol`], which is the same arrangement the handshake
//! regression in `executor.rs` uses.
//!
//! # What none of this establishes
//!
//! Nothing about a target. There is no node here, no transaction is
//! built, and no answer below came from anything that executed anything.
//! What these tests hold is the boundary's shape: that a plan can
//! interleave, that an expectation cannot cross, and that a contradictory
//! answer is refused before a caller can read a fact out of it.

use std::collections::BTreeSet;

use crate::error::NativeConformanceError;
use crate::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, NativeWorkload, OperationStep,
    PlanRefused, TargetOperationPlanner, run_protocol,
};
use crate::protocol::{
    ExecutorCapability, ExecutorHandshake, FundedOutput, NATIVE_PROTOCOL_SCHEMA,
    NativeOperationRequest, NativeOperationResponse, NativeResourceObservation,
    ObservedOutcomeLayer, OperationCaseId, OperationStepKind, OperationSubject,
    TargetFundingSubject, TargetSubmissionSubject, WireOutpoint,
};

use super::support::{
    development_binding, nonmock_handshake, observed_environment, reviewed_target,
};

/// A witness program the funding steps below pay to.
///
/// An arbitrary public development value. It names nothing and
/// authorizes nothing; no key is derived from it and none exists.
const TEST_PROGRAM: [u8; 4] = [0x51, 0x20, 0xaa, 0xbb];

/// A handshake from an executor that does both kinds of operation work.
fn operating_handshake() -> ExecutorHandshake {
    let mut handshake = nonmock_handshake();
    handshake
        .capabilities
        .insert(ExecutorCapability::TestFundingCeremony);
    handshake
        .capabilities
        .insert(ExecutorCapability::TargetTransactionSubmission);
    handshake
}

/// A handshake from an executor that submits but holds no funds.
fn submitting_handshake() -> ExecutorHandshake {
    let mut handshake = nonmock_handshake();
    handshake
        .capabilities
        .insert(ExecutorCapability::TargetTransactionSubmission);
    handshake
}

/// One funding step's subject.
fn funding(outputs: u8) -> OperationSubject {
    OperationSubject::Funding(Box::new(TargetFundingSubject {
        issue_asset: true,
        asset: None,
        output_program: TEST_PROGRAM.to_vec(),
        outputs,
        amount_per_output: 100_000,
    }))
}

/// One submission step's subject.
fn submission(bytes: Vec<u8>) -> OperationSubject {
    OperationSubject::Submission(Box::new(TargetSubmissionSubject {
        transaction_bytes: bytes,
    }))
}

/// An accepted funding answer creating one output.
fn funded(step: &str, txid: &str) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: OperationStepKind::Fund,
            step: step.to_owned(),
        },
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        issued_asset: Some("test-asset".to_owned()),
        funded_outputs: vec![FundedOutput {
            outpoint: WireOutpoint {
                txid: txid.to_owned(),
                vout: 0,
            },
            asset: "test-asset".to_owned(),
            amount_satoshis: 100_000,
            script: "5120aabb".to_owned(),
        }],
        accepted_txid: None,
        sponsor_witness: Vec::new(),
        signature_bound_to: None,
        resources: NativeResourceObservation::default(),
    }
}

/// An accepted submission answer.
fn submitted(step: &str, txid: &str) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: OperationStepKind::Submit,
            step: step.to_owned(),
        },
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: Vec::new(),
        accepted_txid: Some(txid.to_owned()),
        sponsor_witness: Vec::new(),
        signature_bound_to: None,
        resources: NativeResourceObservation::default(),
    }
}

/// The executor's side of one run, written out in advance.
fn script(handshake: &ExecutorHandshake, answers: &[NativeOperationResponse]) -> String {
    let mut lines = vec![
        serde_json::to_string(handshake).expect("the handshake serializes"),
        serde_json::to_string(&observed_environment()).expect("the observation serializes"),
    ];
    for answer in answers {
        lines.push(serde_json::to_string(answer).expect("the answer serializes"));
    }
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

/// Everything the harness wrote, parsed back into requests.
fn requests_written(sent: &[u8]) -> Vec<NativeOperationRequest> {
    String::from_utf8(sent.to_vec())
        .expect("the harness writes text")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<NativeOperationRequest>(line).ok())
        .collect()
}

/// A plan whose steps are fixed, recording what it was told.
struct ScriptedPlan {
    steps: Vec<OperationStep>,
    /// What the harness handed back, in the order it handed it back.
    seen: Vec<(OperationCaseId, NativeOperationResponse)>,
    /// Whether to refuse instead of stating the next step.
    refuse_after: Option<usize>,
}

impl ScriptedPlan {
    fn new(steps: Vec<OperationStep>) -> Self {
        Self {
            steps,
            seen: Vec::new(),
            refuse_after: None,
        }
    }

    fn refusing_after(steps: Vec<OperationStep>, after: usize) -> Self {
        Self {
            steps,
            seen: Vec::new(),
            refuse_after: Some(after),
        }
    }
}

impl TargetOperationPlanner for ScriptedPlan {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((case, response)) = previous {
            self.seen.push((case.clone(), response.clone()));
        }
        if self.refuse_after == Some(self.seen.len()) {
            return Err(PlanRefused);
        }
        Ok(self.steps.get(self.seen.len()).cloned())
    }
}

/// A plan that builds its submission out of what the funding answered.
///
/// The property this exists to hold: the second step is not statable
/// before the first is answered. A boundary that took a fixed list could
/// not express it, and one that split the run in two would fund one
/// disposable chain and submit to another.
struct InterleavingPlan {
    submitted: Option<Vec<u8>>,
}

impl TargetOperationPlanner for InterleavingPlan {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let Some((_case, response)) = previous else {
            return Ok(Some(OperationStep::new("ceremony", funding(1))));
        };
        if self.submitted.is_some() {
            return Ok(None);
        }
        // The transaction the plan builds spends what the funding step
        // actually created. Nothing here is a real encoding: the point is
        // that the bytes are a function of the answer.
        let outpoint = response
            .funded_outputs
            .first()
            .ok_or(PlanRefused)?
            .outpoint
            .clone();
        let bytes = outpoint.txid.as_bytes().to_vec();
        self.submitted = Some(bytes.clone());
        Ok(Some(OperationStep::new("vector", submission(bytes))))
    }
}

/// Drives one scripted run and returns the harness's outcome and what it
/// wrote.
fn drive(
    planner: &mut dyn TargetOperationPlanner,
    handshake: &ExecutorHandshake,
    answers: &[NativeOperationResponse],
) -> (
    Result<crate::executor::ExecutionTranscript, NativeConformanceError>,
    Vec<u8>,
) {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let configuration = ExecutorConfiguration::new(
        std::path::Path::new("/nonexistent-executor"),
        ExecutorTrust::ReviewedNonMock,
        std::time::Duration::from_secs(1),
        // Nothing is spawned here: the protocol is driven over an
        // in-memory script, so the destinations are named and never
        // opened.
        ExecutorDiagnostics::in_directory(std::path::Path::new("/nonexistent-diagnostics")),
    );
    let text = script(handshake, answers);
    let mut reader = std::io::BufReader::new(std::io::Cursor::new(text.into_bytes()));
    let mut sent: Vec<u8> = Vec::new();
    let outcome = run_protocol(
        &target,
        &binding,
        &configuration,
        NativeWorkload::Operations(planner),
        &mut sent,
        &mut reader,
    );
    (outcome, sent)
}

#[test]
fn a_plan_states_its_second_step_out_of_the_first_answer() {
    let mut planner = InterleavingPlan { submitted: None };
    let (outcome, sent) = drive(
        &mut planner,
        &operating_handshake(),
        &[funded("ceremony", "aa00"), submitted("vector", "bb11")],
    );
    let transcript = outcome.expect("the run completes");

    let written = requests_written(&sent);
    assert_eq!(written.len(), 2, "one request per step");
    // The submission carries bytes derived from the funding answer, so
    // the second request could not have been written before the first was
    // answered.
    match &written[1].subject {
        OperationSubject::Submission(subject) => assert_eq!(
            subject.transaction_bytes,
            b"aa00".to_vec(),
            "the submission was not built from the funding answer",
        ),
        OperationSubject::Funding(_)
        | OperationSubject::SponsorFunding(_)
        | OperationSubject::SponsorSigning(_) => panic!("the second step is a submission"),
    }
    assert_eq!(transcript.operation_responses().len(), 2);
    assert_eq!(transcript.operation_requests().len(), 2);
}

#[test]
fn the_transcript_retains_the_exact_subject_of_every_step() {
    // §transcript-binding, for this workload: an evaluator compares its
    // own steps against what was sent, so what was sent is retained
    // rather than reconstructed from the identities.
    let mut planner = ScriptedPlan::new(vec![OperationStep::new("only", funding(3))]);
    let (outcome, _sent) = drive(
        &mut planner,
        &operating_handshake(),
        &[funded("only", "aa00")],
    );
    let transcript = outcome.expect("the run completes");
    let case = OperationCaseId {
        operation: OperationStepKind::Fund,
        step: "only".to_owned(),
    };
    match transcript
        .operation_requests()
        .get(&case)
        .expect("the step was recorded")
    {
        OperationSubject::Funding(subject) => assert_eq!(subject.outputs, 3),
        OperationSubject::Submission(_)
        | OperationSubject::SponsorFunding(_)
        | OperationSubject::SponsorSigning(_) => panic!("the step was a funding step"),
    }
}

#[test]
fn an_executor_that_holds_no_funds_is_refused_the_funding_step_only() {
    // The gate is per step, not per workload: a runner that submits
    // perfectly well should be refused the steps it cannot do and no
    // others.
    let mut planner = ScriptedPlan::new(vec![OperationStep::new("ceremony", funding(1))]);
    let (outcome, sent) = drive(&mut planner, &submitting_handshake(), &[]);
    let error = outcome.expect_err("a funding step is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::OperationStepUnsupported(OperationStepKind::Fund)
        ),
        "expected a refused funding step, got {error}",
    );
    assert!(
        requests_written(&sent).is_empty(),
        "a refused step must not have been sent",
    );

    let mut submitting = ScriptedPlan::new(vec![OperationStep::new("vector", submission(vec![7]))]);
    let (outcome, _sent) = drive(
        &mut submitting,
        &submitting_handshake(),
        &[submitted("vector", "bb11")],
    );
    assert!(
        outcome.is_ok(),
        "the same executor answers the step it advertised",
    );
}

#[test]
fn a_plan_that_reuses_a_step_identity_is_refused() {
    let mut planner = ScriptedPlan::new(vec![
        OperationStep::new("same", funding(1)),
        OperationStep::new("same", funding(1)),
    ]);
    let (outcome, _sent) = drive(
        &mut planner,
        &operating_handshake(),
        &[funded("same", "aa00"), funded("same", "aa01")],
    );
    let error = outcome.expect_err("a repeated step identity is refused");
    assert!(
        matches!(&error, NativeConformanceError::DuplicateOperationStep(case) if case.step == "same"),
        "expected a duplicate step, got {error}",
    );
}

#[test]
fn a_plan_that_cannot_state_its_next_step_refuses_the_run() {
    let mut planner =
        ScriptedPlan::refusing_after(vec![OperationStep::new("first", funding(1))], 1);
    let (outcome, _sent) = drive(
        &mut planner,
        &operating_handshake(),
        &[funded("first", "aa00")],
    );
    let error = outcome.expect_err("a refusing plan refuses the run");
    assert!(
        matches!(error, NativeConformanceError::OperationPlanRefused),
        "expected a refused plan, got {error}",
    );
}

#[test]
fn an_answer_to_a_step_that_was_never_asked_for_is_refused() {
    let mut planner = ScriptedPlan::new(vec![OperationStep::new("asked", funding(1))]);
    let (outcome, _sent) = drive(
        &mut planner,
        &operating_handshake(),
        &[funded("never-asked", "aa00")],
    );
    let error = outcome.expect_err("an unasked answer is refused");
    assert!(
        matches!(
            &error,
            NativeConformanceError::UnexpectedOperationResponse(case) if case.step == "never-asked"
        ),
        "expected an unexpected answer, got {error}",
    );
}

#[test]
fn a_repeated_answer_is_told_apart_from_an_unasked_one() {
    // Two faults, kept apart: an answer to a step already settled, and an
    // answer to a step never asked for. Collapsing them would report a
    // stuttering adapter as one that answered a question nobody put.
    let mut planner = ScriptedPlan::new(vec![
        OperationStep::new("first", funding(1)),
        OperationStep::new("second", funding(1)),
    ]);
    let (outcome, _sent) = drive(
        &mut planner,
        &operating_handshake(),
        &[funded("first", "aa00"), funded("first", "aa00")],
    );
    let error = outcome.expect_err("a repeated answer is refused");
    assert!(
        matches!(
            &error,
            NativeConformanceError::DuplicateOperationResponse(case) if case.step == "first"
        ),
        "expected a duplicate answer, got {error}",
    );
}

#[test]
fn an_answer_naming_the_other_kinds_observation_is_refused() {
    // A funding step that named an accepted transaction is answering a
    // question it was not asked, and a caller reading a submission fact
    // out of it would attach one kind of observation to the other kind's
    // obligation.
    let mut answer = funded("ceremony", "aa00");
    answer.accepted_txid = Some("bb11".to_owned());

    let mut planner = ScriptedPlan::new(vec![OperationStep::new("ceremony", funding(1))]);
    let (outcome, _sent) = drive(&mut planner, &operating_handshake(), &[answer]);
    let error = outcome.expect_err("a mismatched answer is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::MalformedOperationResponseShape { defect, .. }
                if defect == crate::protocol::ResponseShapeDefect::OperationResponseMismatchesStep
        ),
        "expected a mismatched step, got {error}",
    );
}

#[test]
fn an_answer_carrying_an_observation_for_a_step_that_never_ran_is_refused() {
    // The rule every other record in this protocol already keeps: a run
    // that reached no target verdict created no coin, chose no asset, and
    // computed no transaction identity.
    let mut answer = funded("ceremony", "aa00");
    answer.observed_layer = ObservedOutcomeLayer::ExecutorInfrastructureFailure;

    let mut planner = ScriptedPlan::new(vec![OperationStep::new("ceremony", funding(1))]);
    let (outcome, _sent) = drive(&mut planner, &operating_handshake(), &[answer]);
    let error = outcome.expect_err("an impossible observation is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::MalformedOperationResponseShape { defect, .. }
                if defect
                    == crate::protocol::ResponseShapeDefect::InfrastructureResponseCarriesObservation
        ),
        "expected an infrastructure contradiction, got {error}",
    );
}

#[test]
fn the_adapters_not_yet_implemented_refusal_is_a_declared_record() {
    // The Python seam's own answer, in the exact spelling
    // `answer_operation_step` writes it. Written out here rather than
    // derived, because what is being checked is that the two
    // implementations agree on one revision — a value this side produced
    // would only check this side against itself
    // `(´[PLAN-rule:guide12-exec:protocol-revision]´)`.
    //
    // The refusal is what an unimplemented step honestly is: the run did
    // not happen, so no member describing what a target did is present,
    // and the shape check passes for that reason rather than by
    // exemption.
    let refusal = serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "case": {"operation": "fund", "step": "ceremony"},
        "observed_layer": "executor_infrastructure_failure",
        "observed_detail": "this adapter recognizes operation steps and performs none: \
            the funding ceremony and transaction submission are not implemented",
        "issued_asset": serde_json::Value::Null,
        "funded_outputs": [],
        "accepted_txid": serde_json::Value::Null,
        "resources": {
            "script_bytes": 0,
            "initial_stack_items": 0,
            "peak_stack_items": serde_json::Value::Null,
            "peak_altstack_items": serde_json::Value::Null,
            "maximum_element_bytes": serde_json::Value::Null,
            "validation_budget_used": serde_json::Value::Null,
            "transaction_weight": serde_json::Value::Null,
        },
    });
    let response: NativeOperationResponse =
        serde_json::from_value(refusal).expect("the adapter's refusal is a declared record");
    assert_eq!(response.case.operation, OperationStepKind::Fund);
    assert!(
        !response.observed_layer.is_target_verdict(),
        "an unimplemented step must not be reported as anything the target did",
    );
    assert_eq!(
        response.validate_shape(),
        Ok(()),
        "the refusal must be a shape the protocol defines",
    );
}

#[test]
fn an_acceptance_with_nothing_to_show_for_it_is_refused() {
    // An adapter that returned the layer without doing the work is
    // indistinguishable, from the outside, from one that did it and
    // reported nothing. Neither is admitted.
    let mut answer = submitted("vector", "bb11");
    answer.accepted_txid = None;

    let mut planner = ScriptedPlan::new(vec![OperationStep::new("vector", submission(vec![7]))]);
    let (outcome, _sent) = drive(&mut planner, &operating_handshake(), &[answer]);
    let error = outcome.expect_err("an empty acceptance is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::MalformedOperationResponseShape { defect, .. }
                if defect
                    == crate::protocol::ResponseShapeDefect::AcceptedOperationOmitsObservation
        ),
        "expected an empty acceptance, got {error}",
    );
}

#[test]
fn a_refusal_that_reached_a_target_verdict_is_not_a_shape_defect() {
    // The other half of the rule above. A submission the target refused
    // reports no identity and creates no coin, and that is the ordinary
    // negative answer rather than a contradiction — the whole point of
    // the workload is to record it.
    let mut answer = submitted("vector", "bb11");
    answer.accepted_txid = None;
    answer.observed_layer = ObservedOutcomeLayer::ConsensusRejectionBeforeScript;
    answer.observed_detail = Some("the target refused it".to_owned());

    let mut planner = ScriptedPlan::new(vec![OperationStep::new("vector", submission(vec![7]))]);
    let (outcome, _sent) = drive(&mut planner, &operating_handshake(), &[answer]);
    let transcript = outcome.expect("a refused submission is a completed run");
    assert_eq!(transcript.operation_responses().len(), 1);
}

#[test]
fn no_operation_record_carries_an_expectation() {
    // §request-subject, held by the type rather than by a discipline.
    // Every member of a request is enumerated here, so a member added
    // later that could carry an expected layer, identity, or class fails
    // this test rather than passing unnoticed.
    let request = NativeOperationRequest {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: OperationStepKind::Submit,
            step: "vector".to_owned(),
        },
        subject: submission(vec![1, 2, 3]),
    };
    let value = serde_json::to_value(&request).expect("the request serializes");
    let members: BTreeSet<&str> = value
        .as_object()
        .expect("a request is an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        members,
        BTreeSet::from(["schema", "case", "subject"]),
        "an operation request carries the identity and the subject, and nothing else",
    );

    let subject = value
        .get("subject")
        .and_then(serde_json::Value::as_object)
        .expect("a subject is an object");
    assert_eq!(
        subject.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from(["transaction_bytes"]),
        "a submission subject carries the bytes and nothing else",
    );
}

#[test]
fn every_operation_record_round_trips_and_refuses_an_unknown_member() {
    // §16.3's cross-language requirement, held on this side: each record
    // round-trips, and a member neither side declared is refused rather
    // than ignored.
    let request = NativeOperationRequest {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: OperationStepKind::Fund,
            step: "ceremony".to_owned(),
        },
        subject: funding(2),
    };
    let text = serde_json::to_string(&request).expect("the request serializes");
    assert_eq!(
        serde_json::from_str::<NativeOperationRequest>(&text).expect("the request parses"),
        request,
    );

    let response = funded("ceremony", "aa00");
    let text = serde_json::to_string(&response).expect("the response serializes");
    assert_eq!(
        serde_json::from_str::<NativeOperationResponse>(&text).expect("the response parses"),
        response,
    );

    let mut value = serde_json::to_value(&response).expect("the response serializes");
    value
        .as_object_mut()
        .expect("a response is an object")
        .insert("commentary".to_owned(), serde_json::Value::from("extra"));
    assert!(
        serde_json::from_value::<NativeOperationResponse>(value).is_err(),
        "an undeclared member must be refused rather than ignored",
    );
}

#[test]
fn the_two_subject_shapes_are_distinguishable_without_a_tag() {
    // The untagged subject rests on the two variants sharing no member.
    // A member added to one that the other also has would make the wire
    // ambiguous, and this is what would notice.
    let funding_members: BTreeSet<String> = serde_json::to_value(funding(1))
        .expect("the subject serializes")
        .as_object()
        .expect("a subject is an object")
        .keys()
        .cloned()
        .collect();
    let submission_members: BTreeSet<String> = serde_json::to_value(submission(vec![1]))
        .expect("the subject serializes")
        .as_object()
        .expect("a subject is an object")
        .keys()
        .cloned()
        .collect();
    assert!(
        funding_members.is_disjoint(&submission_members),
        "the two operation subjects share a member, so the untagged wire form is ambiguous",
    );
}
