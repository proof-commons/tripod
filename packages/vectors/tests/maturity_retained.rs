//! Retained native routes have their own capture target so legacy driver censuses stay fixed.
//! The manifest freezes claims before execution and binds funding before a submission leaves the planner.

pub mod common;

use std::fmt::Write as _;
use std::path::Path;

use linker::CandidateDeploymentIdentity;
use serde_json::{Value, json};
use target_elements_conformance::executor::{
    ExecutorTrust, NativeOperationCapture, OperationStep, PlanRefused, TargetOperationPlanner,
};
use target_elements_conformance::protocol::{
    FundedOutput, NativeOperationResponse, OperationCaseId, OperationSubject,
};
use transaction::bytes::TargetTransaction;
use transaction::operator_right::BranchContext;
use vectors::maturity_mutants::native_staging::{
    NATIVE_ROWS, NativeBoundary, NativeStanding, NativeSubject,
};
use vectors::maturity_native::{MaturityNativePlanRefusal, native};

const RUN_TEST_NAME: &str = "the_retained_maturity_routes_run_against_a_real_target";
const SUBJECTS: [NativeSubject; 2] = [
    NativeSubject::SuccessorFromWrongSemanticMetadata,
    NativeSubject::TwoStateSuccessors,
];
const FAMILIES: [&str; 2] = ["wrong-semantic-metadata", "two-state-successors"];

fn branch() -> BranchContext {
    BranchContext::new([0x41; 32], 7).expect("fixed caller branch")
}

fn required_environment(name: &str) -> String {
    common::environment(name).unwrap_or_else(|| panic!("{name} is required"))
}

fn native_identity() -> CandidateDeploymentIdentity {
    let network = required_environment("TRIPOD_LIVE_NETWORK_ID");
    let genesis = required_environment("TRIPOD_LIVE_GENESIS_ID");
    common::require_lower_hex(&network, 64, "network").expect("network identity");
    common::require_lower_hex(&genesis, 64, "genesis").expect("genesis identity");
    CandidateDeploymentIdentity::new(common::identifier(&network), common::identifier(&genesis))
        .expect("nonzero deployment identities")
}

struct ManifestPlanner {
    planner: native::RetainedRoutesPlanner,
    rows: Vec<Value>,
    funding: Vec<(String, FundedOutput)>,
    failure: Option<String>,
}

impl ManifestPlanner {
    fn new(identity: CandidateDeploymentIdentity, context: BranchContext) -> Self {
        let rows = SUBJECTS.iter().zip(FAMILIES).map(|(subject, family)| {
            let row = NATIVE_ROWS.iter().find(|row| row.standing().subject() == Some(*subject))
                .expect("retained subject has a typed row");
            let safety = vectors::maturity_safety::rows().iter().find(|candidate|
                candidate.section() == row.section() && candidate.name() == row.name())
                .expect("native row has a Safety claim");
            assert_eq!(safety.boundary(), vectors::maturity_safety::MaturityRowBoundary::NativeSubjectAwaitsConstructionRoute);
            assert_eq!(row.boundary(), NativeBoundary::ScriptPathRejection);
            let expected_layer = if matches!(row.standing(), NativeStanding::StagedAcceptanceExpected { .. }) {
                "accepted"
            } else {
                match row.boundary() {
                    NativeBoundary::ScriptPathRejection => "script_path_rejection",
                    NativeBoundary::ConsensusRejectionBeforeScript => "consensus_rejection_before_script",
                }
            };
            let second = *subject == NativeSubject::TwoStateSuccessors;
            json!({
                "claim":{"section":row.section().section(), "name":row.name()},
                "operation":"announce_maturity", "family":family,
                "candidate":{"schedule":tapscript::StateWitnessSchedule::VariableMetadata.name(),
                    "subject_pin":subject.pin()},
                "abi":"variable-metadata", "target":"elements-tapscript-v2",
                "deployment":{"network":common::hex_bytes(identity.network_id()),
                    "genesis":common::hex_bytes(identity.genesis_id())},
                "branch":{"identifier":common::hex_bytes(context.identifier()), "checkpoint":context.checkpoint()},
                "policy_requirement":"node-defaults", "policy":null,
                "constructor":if second { "TargetTransaction" } else { "link_for_metadata" },
                "mutation":if second { "second-same-asset-input-and-successor-at-amount-one" } else { "semantic-q-checked-plus-one" },
                "declared_boundary":"script_path_rejection",
                "expected_layer":expected_layer,
                "control_id":format!("{family}:control:{}", if second {5} else {3}),
                "compatibility":if second { "equivalent-coin" } else { "same-coin" },
                "control":null, "observation":null,
                "expected_consumer":"native-evidence-fold",
            })
        }).collect();
        Self {
            planner: native::RetainedRoutesPlanner::new(identity, context)
                .expect("retained planner"),
            rows,
            funding: Vec::new(),
            failure: None,
        }
    }

    fn bind_submission(&mut self, step: &OperationStep) -> Result<(), String> {
        let OperationSubject::Submission(subject) = step.subject() else {
            return Ok(());
        };
        let parts = step.case().step.split(':').collect::<Vec<_>>();
        let [family, label, _] = parts.as_slice() else {
            return Err("submission step shape".into());
        };
        let index = FAMILIES
            .iter()
            .position(|value| value == family)
            .ok_or("unknown family")?;
        let control = *label == "control";
        let row = &mut self.rows[index];
        if control {
            if row["control_id"] != step.case().step {
                return Err("control identity".into());
            }
        } else {
            let claims = native::submission_rows(step);
            if claims.len() != 1
                || claims[0].0.section() != row["claim"]["section"]
                || claims[0].1 != row["claim"]["name"]
            {
                return Err("subject claim mapping".into());
            }
        }
        let transaction = TargetTransaction::decode(&subject.transaction_bytes)
            .map_err(|error| format!("{error:?}"))?;
        let inputs = transaction
            .inputs()
            .iter()
            .map(|input| {
                let point = input.outpoint();
                let (funding_step, coin) = self
                    .funding
                    .iter()
                    .find(|(_, coin)| {
                        coin.outpoint.txid == point.txid().to_target_display()
                            && coin.outpoint.vout == point.index()
                    })
                    .ok_or("submission spends an unretained outpoint")?;
                Ok(json!({"funding_step":funding_step, "output":coin}))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let expected_inputs = if !control && index == 1 { 2 } else { 1 };
        if inputs.len() != expected_inputs {
            return Err("funded input census".into());
        }
        let policy = self
            .planner
            .policy()
            .ok_or("submission has no validated policy")?;
        row["policy"] = policy.0.clone();
        let slot = if control { "control" } else { "observation" };
        if !row[slot].is_null() {
            return Err("duplicate submission".into());
        }
        row[slot] = json!({"step":step.case().step,
            "digest":common::hex_bytes(&common::sha256(&subject.transaction_bytes)),
            "funded_outpoints":inputs, "claims":[row["claim"].clone()]});
        if !row["control"].is_null() && !row["observation"].is_null() {
            let control_coin = &row["control"]["funded_outpoints"][0]["output"];
            let subject_coin = &row["observation"]["funded_outpoints"][0]["output"];
            for key in ["asset", "amount_satoshis", "script"] {
                if control_coin[key] != subject_coin[key] {
                    return Err("control compatibility".into());
                }
            }
            if (control_coin["outpoint"] == subject_coin["outpoint"]) != (index == 0) {
                return Err("control outpoint relationship".into());
            }
        }
        Ok(())
    }
}

impl TargetOperationPlanner for ManifestPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let next = self.planner.next_step(previous)?;
        if let Some((case, response)) = previous {
            self.funding.extend(
                response
                    .funded_outputs
                    .iter()
                    .cloned()
                    .map(|coin| (case.step.clone(), coin)),
            );
        }
        if let Some(step) = &next
            && let Err(error) = self.bind_submission(step)
        {
            self.failure = Some(error);
            return Err(PlanRefused);
        }
        Ok(next)
    }
}

fn payload(
    plan: &ManifestPlanner,
    identity: &CandidateDeploymentIdentity,
    context: BranchContext,
) -> String {
    let mut out = String::from("maturity-retained-evidence-schema 1\n");
    common::write_bytes_field(&mut out, "deployment-id", identity.network_id());
    common::write_bytes_field(&mut out, "genesis-id", identity.genesis_id());
    common::write_bytes_field(&mut out, "branch-identifier", context.identifier());
    writeln!(out, "branch-checkpoint {}", context.checkpoint()).expect("String write");
    if let Some(policy) = plan.planner.policy() {
        write_json(&mut out, "retained-policy", &policy.0);
    }
    write_json(
        &mut out,
        "retained-families",
        &json!(plan.planner.families_json()),
    );
    write_json(&mut out, "retained-stopped", &json!(plan.planner.stopped()));
    out.push_str("campaign-manifest-schema 1\ncampaign-manifest-count 2\n");
    for row in &plan.rows {
        write_json(&mut out, "campaign-row", row);
    }
    for (step, response) in plan.planner.exchanges() {
        write_json(
            &mut out,
            "exchange",
            &json!((step.case(), step.subject(), response)),
        );
    }
    if let Some(refusal) = plan.planner.refusal() {
        common::write_bytes_field(&mut out, "refusal", format!("{refusal:?}").as_bytes());
    }
    if let Some(failure) = &plan.failure {
        common::write_bytes_field(&mut out, "manifest-refusal", failure.as_bytes());
    }
    out
}

fn write_json(out: &mut String, name: &str, value: &Value) {
    common::write_bytes_field(
        out,
        name,
        &serde_json::to_vec(value).expect("JSON serialization"),
    );
}

fn assert_replay(
    plan: &ManifestPlanner,
    identity: CandidateDeploymentIdentity,
    context: BranchContext,
) {
    assert!(
        plan.failure.is_none(),
        "manifest validation: {:?}",
        plan.failure
    );
    let completed = plan.planner.complete();
    assert!(
        matches!(
            &completed,
            Ok(()) | Err(MaturityNativePlanRefusal::RetainedControlsRefused { .. })
        ),
        "completion: {completed:?}"
    );
    let replay = native::replay_retained_routes(identity, context, plan.planner.exchanges());
    match replay {
        Ok(replay) => {
            assert_eq!(completed, Ok(()));
            assert_eq!(replay.families_json(), plan.planner.families_json());
            assert_eq!(replay.policy(), plan.planner.policy());
        }
        Err(error) => assert_eq!(Err(error), completed),
    }
}

#[test]
#[ignore = "requires the revision-8 native adapter and capture environment"]
fn the_retained_maturity_routes_run_against_a_real_target() {
    assert_eq!(
        required_environment("TRIPOD_MATURITY_CAPTURE_PROFILE"),
        "retained"
    );
    let executor = required_environment("TRIPOD_LIVE_EXECUTOR");
    let directory = required_environment("TRIPOD_LIVE_REPORT_DIR");
    let mut destination = common::CaptureDestination::new(
        directory.clone().into(),
        required_environment("TRIPOD_LIVE_SUITE_SHORT_SHA"),
        required_environment("TRIPOD_LIVE_SUITE_COMMIT"),
        required_environment("TRIPOD_LIVE_SUITE_TREE"),
    )
    .expect("capture provenance");
    destination.report_test_name = RUN_TEST_NAME;
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    let identity = native_identity();
    let context = branch();
    let mut plan = ManifestPlanner::new(identity.clone(), context);
    let mut capture = NativeOperationCapture::default();
    let outcome = common::execute(
        Path::new(&executor),
        Path::new(&directory),
        &identity,
        ExecutorTrust::ReviewedNonMock,
        &mut plan,
        &mut capture,
    );
    let evidence = payload(&plan, &identity, context);
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &evidence);
    outcome.expect("retained operations completed");
    assert_replay(&plan, identity, context);
}

#[test]
fn manifest_freezes_two_distinct_typed_claims_without_observations() {
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
    let plan = ManifestPlanner::new(identity.clone(), branch());
    assert_eq!(plan.rows.len(), 2);
    assert_eq!(plan.rows[0]["expected_layer"], "script_path_rejection");
    assert_eq!(plan.rows[1]["expected_layer"], "accepted");
    assert_ne!(
        plan.rows[0]["candidate"]["subject_pin"],
        plan.rows[1]["candidate"]["subject_pin"]
    );
    for row in &plan.rows {
        assert_eq!(row["claim"]["section"], "16.6");
        assert!(row["control"].is_null());
        assert!(row["observation"].is_null());
        assert!(row["policy"].is_null());
    }
    let text = payload(&plan, &identity, branch());
    assert_eq!(
        text.lines()
            .filter(|line| line.starts_with("campaign-row "))
            .count(),
        2
    );
    assert!(!text.contains("exchange "));
    assert!(text.contains("retained-stopped 2 5b5d\n"));
}

#[test]
fn manifest_refuses_unretained_funding_before_submission() {
    let seed = vectors::maturity_corpus::maturity_variable_run_of_record().expect("archive");
    let mut plan = ManifestPlanner::new(seed.evidence().identity().clone(), branch());
    let (_, response) = seed.exchanges().last().expect("submission");
    let bytes = response
        .mined_readback
        .as_ref()
        .expect("accepted readback")
        .raw_transaction
        .clone();
    let step = OperationStep::new(
        "wrong-semantic-metadata:successor-from-wrong-semantic-metadata:2",
        OperationSubject::Submission(Box::new(
            target_elements_conformance::protocol::TargetSubmissionSubject {
                transaction_bytes: bytes,
            },
        )),
    );
    assert_eq!(
        plan.bind_submission(&step),
        Err("submission spends an unretained outpoint".into())
    );
    assert!(plan.rows[0]["observation"].is_null());
}

// The wrapper has validated earlier steps; remove only the requested binding so
// each guard test can offer one changed step or funding response in isolation.
fn manifest_at_submission(name: &str) -> (ManifestPlanner, OperationStep) {
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
    let mut plan = ManifestPlanner::new(identity, branch());
    let mut next = plan.next_step(None).expect("bootstrap");
    let mut ordinal = 0;
    while let Some(step) = next {
        if step.case().step == name {
            let index = usize::from(name.starts_with("two-state-successors:"));
            let slot = if name.contains(":control:") {
                "control"
            } else {
                "observation"
            };
            plan.rows[index][slot] = Value::Null;
            return (plan, step);
        }
        let response = scripted_response(&step, ordinal, false, false);
        next = plan
            .next_step(Some((step.case(), &response)))
            .expect("scripted step");
        ordinal += 1;
    }
    panic!("missing scripted submission {name}");
}

const SEMANTIC_CONTROL: &str = "wrong-semantic-metadata:control:3";
const SUCCESSOR_SUBJECT: &str = "two-state-successors:two-state-successors:8";

#[test]
fn manifest_refuses_changed_control_identity() {
    let (mut plan, step) = manifest_at_submission(SEMANTIC_CONTROL);
    let changed = OperationStep::new("wrong-semantic-metadata:control:4", step.subject().clone());
    assert_eq!(
        plan.bind_submission(&changed),
        Err("control identity".into())
    );
}

#[test]
fn manifest_refuses_changed_subject_claim_mapping() {
    let (mut plan, step) = manifest_at_submission(SUCCESSOR_SUBJECT);
    let changed = OperationStep::new(
        "two-state-successors:successor-from-wrong-semantic-metadata:8",
        step.subject().clone(),
    );
    assert_eq!(
        plan.bind_submission(&changed),
        Err("subject claim mapping".into())
    );
}

#[test]
fn manifest_refuses_changed_funded_input_census() {
    let (mut plan, step) = manifest_at_submission(SUCCESSOR_SUBJECT);
    let changed = OperationStep::new(
        "wrong-semantic-metadata:successor-from-wrong-semantic-metadata:2",
        step.subject().clone(),
    );
    assert_eq!(
        plan.bind_submission(&changed),
        Err("funded input census".into())
    );
}

#[test]
fn manifest_refuses_incompatible_control_funding_response() {
    let (mut plan, step) = manifest_at_submission(SEMANTIC_CONTROL);
    let (funding_step, original) = plan
        .planner
        .exchanges()
        .iter()
        .find(|(step, _)| {
            step.case().step == plan.rows[0]["observation"]["funded_outpoints"][0]["funding_step"]
        })
        .expect("funding response");
    let mut changed = original.clone();
    changed.funded_outputs[0].amount_satoshis += 1;
    let (_, retained) = plan
        .funding
        .iter_mut()
        .find(|(name, _)| name == &funding_step.case().step)
        .expect("retained response");
    *retained = changed.funded_outputs[0].clone();
    assert_eq!(
        plan.bind_submission(&step),
        Err("control compatibility".into())
    );
}

#[test]
fn manifest_refuses_changed_control_outpoint_relationship() {
    let (mut plan, step) = manifest_at_submission(SUCCESSOR_SUBJECT);
    let OperationSubject::Submission(subject) = step.subject() else {
        panic!("submission")
    };
    let transaction = TargetTransaction::decode(&subject.transaction_bytes).expect("transaction");
    let (control, _) = plan
        .planner
        .exchanges()
        .iter()
        .find(|(step, _)| step.case().step == "two-state-successors:control:5")
        .expect("control");
    let OperationSubject::Submission(control) = control.subject() else {
        panic!("control submission")
    };
    let control =
        TargetTransaction::decode(&control.transaction_bytes).expect("control transaction");
    let mut inputs = transaction.inputs().to_vec();
    inputs[0] = control.inputs()[0].clone();
    let changed = TargetTransaction::new(
        transaction.version(),
        inputs,
        transaction.outputs().to_vec(),
        transaction.lock_time(),
        transaction.witnesses().to_vec(),
    )
    .expect("changed transaction");
    let step = OperationStep::new(
        &step.case().step,
        OperationSubject::Submission(Box::new(
            target_elements_conformance::protocol::TargetSubmissionSubject {
                transaction_bytes: changed.encode(),
            },
        )),
    );
    assert_eq!(
        plan.bind_submission(&step),
        Err("control outpoint relationship".into())
    );
}

#[test]
fn manifest_refuses_duplicate_submission() {
    let (mut plan, step) = manifest_at_submission(SEMANTIC_CONTROL);
    assert_eq!(plan.bind_submission(&step), Ok(()));
    assert_eq!(
        plan.bind_submission(&step),
        Err("duplicate submission".into())
    );
}

#[test]
#[should_panic(expected = "completion: Err(IncompleteTranscript)")]
fn replay_rejects_incomplete_completion() {
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
    let plan = ManifestPlanner::new(identity.clone(), branch());
    assert_eq!(
        plan.planner.complete(),
        Err(MaturityNativePlanRefusal::IncompleteTranscript)
    );
    assert_replay(&plan, identity, branch());
}

fn scripted_response(
    step: &OperationStep,
    ordinal: u32,
    semantic_accepts: bool,
    successor_refuses: bool,
) -> NativeOperationResponse {
    use target_elements_conformance::protocol::{ObservedOutcomeLayer, WireOutpoint};
    let seed = vectors::maturity_corpus::maturity_variable_run_of_record().expect("archive");
    let mut response = seed.exchanges()[0].1.clone();
    response.case = step.case().clone();
    response.issued_asset = None;
    response.funded_outputs.clear();
    response.mined_readback = None;
    response.accepted_txid = None;
    response.observed_layer = ObservedOutcomeLayer::Accepted;
    let mut route = "setup";
    let mut facts = Value::Null;
    match step.subject() {
        OperationSubject::Funding(subject) => {
            let asset = format!("01{}fe", "55".repeat(30));
            if subject.issue_asset {
                response.issued_asset = Some(asset.clone());
            }
            response.funded_outputs.push(FundedOutput {
                outpoint: WireOutpoint {
                    txid: "66".repeat(32),
                    vout: ordinal,
                },
                asset,
                amount_satoshis: subject.amount_per_output,
                script: common::hex_bytes(&subject.output_program),
            });
        }
        OperationSubject::Submission(subject) => {
            let semantic = step.case().step.starts_with("wrong-semantic-metadata:");
            let control = step.case().step.contains(":control:");
            if (semantic && control == semantic_accepts)
                || (!semantic && !control && successor_refuses)
            {
                response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
                route = "refused";
            } else {
                let transaction =
                    TargetTransaction::decode(&subject.transaction_bytes).expect("subject");
                let txid = transaction::bytes::Txid::from_internal(common::sha256(
                    &common::sha256(&transaction.encode_without_witness()),
                ))
                .to_target_display();
                let wtxid = transaction::bytes::Txid::from_internal(common::sha256(
                    &common::sha256(&subject.transaction_bytes),
                ))
                .to_target_display();
                let mut readback = seed
                    .exchanges()
                    .last()
                    .expect("submission")
                    .1
                    .mined_readback
                    .clone()
                    .expect("readback");
                readback.transaction_id.clone_from(&txid);
                readback.witness_transaction_id = wtxid;
                readback
                    .raw_transaction
                    .clone_from(&subject.transaction_bytes);
                facts = json!({"txid":txid, "included_block":readback.block_hash,
                    "produced_block":readback.block_hash, "inclusion_in_produced":true,
                    "load_five_minutes":0, "mempool_size":0, "mempool_minimum":"0.00000100"});
                response.accepted_txid = Some(txid);
                response.mined_readback = Some(readback);
                route = "direct-block";
            }
        }
        _ => panic!("retained schedule funds and submits"),
    }
    let policy = vectors::maturity_corpus::maturity_native_run_of_record()
        .expect("native archive")
        .evidence
        .policy()
        .expect("policy")
        .0
        .clone();
    let setup = route == "setup";
    response.observed_detail = Some(json!({"schema":1, "policy":policy, "route":route,
        "relay_allowed":if setup {Value::Null} else {json!(false)},
        "relay_detail":if setup {Value::Null} else {json!("min relay fee not met")}, "detail":facts}).to_string());
    response.validate_shape().expect("scripted shape");
    response
}

#[test]
fn manifest_replays_expected_and_contradictory_outcomes_with_funding_bound() {
    for (scenario, semantic_accepts, successor_refuses) in [
        ("both-as-expected", false, false),
        ("semantic-accepted", true, false),
        ("two-successor-refused", false, true),
        ("both-departing", true, true),
    ] {
        let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32])
            .expect("scripted adapter deployment");
        let mut plan = ManifestPlanner::new(identity.clone(), branch());
        let mut next = plan.next_step(None).expect("issuance");
        let mut ordinal = 0;
        while let Some(step) = next {
            let response = scripted_response(&step, ordinal, semantic_accepts, successor_refuses);
            next = plan
                .next_step(Some((step.case(), &response)))
                .unwrap_or_else(|_| panic!("{:?}: {:?}", plan.failure, plan.planner.refusal()));
            ordinal += 1;
        }
        assert_eq!(ordinal, 9);
        #[cfg(unix)]
        if scenario == "both-as-expected" {
            assert_scripted_capture(&plan, &identity);
        }
        assert_replay(&plan, identity.clone(), branch());
        for (family, contradiction, stopped) in [
            (0, semantic_accepts, semantic_accepts),
            (1, successor_refuses, false),
        ] {
            let outcomes = plan.planner.families_json();
            assert_eq!(
                outcomes[family]["expectation"],
                if contradiction {
                    "contradiction"
                } else {
                    "agreement"
                },
                "{scenario}, family {family}"
            );
            assert_eq!(
                outcomes[family]["stopped"], stopped,
                "{scenario}, family {family}"
            );
        }
        for row in &plan.rows {
            assert_eq!(row["observation"]["claims"][0], row["claim"]);
            assert!(row["observation"]["funded_outpoints"].is_array());
            assert!(row["control"]["funded_outpoints"].is_array());
        }
        assert_eq!(
            payload(&plan, &identity, branch())
                .lines()
                .filter(|line| line.starts_with("exchange "))
                .count(),
            9
        );
    }
}

#[cfg(unix)]
fn assert_scripted_capture(scripted: &ManifestPlanner, identity: &CandidateDeploymentIdentity) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = common::test_directory("retained-capture");
    let adapter = directory.join("scripted-adapter.sh");
    let responses = scripted
        .planner
        .exchanges()
        .iter()
        .map(|(_, response)| serde_json::to_string(response).expect("response"))
        .collect::<Vec<_>>();
    std::fs::write(
        &adapter,
        common::scripted_adapter(
            &["test_funding_ceremony", "target_transaction_submission"],
            &responses,
        ),
    )
    .expect("scripted adapter");
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755))
        .expect("executable adapter");
    let mut plan = ManifestPlanner::new(identity.clone(), branch());
    let mut capture = NativeOperationCapture::default();
    let outcome = common::execute(
        &adapter,
        &directory,
        identity,
        ExecutorTrust::Mock,
        &mut plan,
        &mut capture,
    );
    let mut destination = common::test_capture_destination(&directory);
    for ceremony in common::CeremonyId::ALL {
        assert!(
            destination
                .capture_path(ceremony, ceremony.rust_test_name())
                .is_ok()
        );
    }
    destination.report_test_name = RUN_TEST_NAME;
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture)
        .with_digest(
            "retained-manifest",
            Some(common::sha256(
                &serde_json::to_vec(&plan.rows).expect("rows"),
            )),
        );
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    common::write_capture_before_gates(
        &mut guard,
        &capture,
        &facts,
        &payload(&plan, identity, branch()),
    );
    let rendered = std::fs::read_to_string(guard.capture_path().expect("capture path"))
        .expect("capture retained before gates");
    outcome.expect("scripted execution");
    assert_replay(&plan, identity.clone(), branch());
    assert_eq!(plan.rows, scripted.rows);
    let subjects = rendered
        .lines()
        .filter(|line| line.starts_with("request-subject "))
        .map(|line| common::decode_request_subject_line(line).expect("subject"))
        .collect::<Vec<_>>();
    assert_eq!(
        subjects,
        plan.planner
            .exchanges()
            .iter()
            .map(|(step, _)| step.subject().clone())
            .collect::<Vec<_>>()
    );
    drop(guard);
    std::fs::remove_dir_all(directory).expect("capture removed");
}
