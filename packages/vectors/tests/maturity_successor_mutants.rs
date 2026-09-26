//! Native successor-constructor mutant offers and their scripted capture form.

pub mod common;

use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

use linker::CandidateDeploymentIdentity;
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    ExecutionTranscript, ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust,
    NativeOperationCapture, OperationStep, TargetOperationPlanner, execute_operations_captured,
};
use target_elements_conformance::protocol::{
    FundedOutput, MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse,
    NativeResourceObservation, ObservedOutcomeLayer, OperationSubject, WireOutpoint,
};
use transaction::bytes::{TargetTransaction, Txid};
use transaction::operator_right::BranchContext;
use vectors::maturity_mutant_ceremony::{
    MaturityMutantCeremonyPlanner, MaturityMutantEvidence, SUCCESSOR_MUTANT_STEPS, declared_layer,
    roster,
};
use vectors::maturity_mutants::MaturityMutantTable;
use vectors::maturity_native::MaturityNativePlanRefusal;

const RUN_TEST_NAME: &str =
    "the_maturity_successor_mutants_are_refused_before_their_control_is_accepted";
const TABLE: MaturityMutantTable = MaturityMutantTable::SuccessorConstructor;
const RUN_BRANCH: ([u8; 32], u64) = ([0x41; 32], 7);

fn branch() -> BranchContext {
    BranchContext::new(RUN_BRANCH.0, RUN_BRANCH.1).expect("nonzero caller branch")
}

fn execute(
    adapter: &Path,
    directory: &Path,
    deployment: &CandidateDeploymentIdentity,
    trust: ExecutorTrust,
    planner: &mut dyn TargetOperationPlanner,
    capture: &mut NativeOperationCapture,
) -> Result<ExecutionTranscript, target_elements_conformance::error::NativeConformanceError> {
    let target = reviewed_elements_tapscript().expect("reviewed target");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            *deployment.network_id(),
            *deployment.genesis_id(),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("development binding");
    let configuration = ExecutorConfiguration::new(
        adapter,
        trust,
        Duration::from_secs(300),
        ExecutorDiagnostics::in_directory(&directory.join("diagnostics")),
    );
    execute_operations_captured(&target, &binding, &configuration, planner, capture)
}

fn evidence_payload(
    capture: &NativeOperationCapture,
    identity: &CandidateDeploymentIdentity,
    context: BranchContext,
    evidence: &Result<MaturityMutantEvidence, MaturityNativePlanRefusal>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "maturity-mutant-evidence-schema 1");
    common::write_bytes_field(&mut out, "deployment-network-id", identity.network_id());
    common::write_bytes_field(&mut out, "deployment-genesis-id", identity.genesis_id());
    common::write_bytes_field(&mut out, "branch-identifier", context.identifier());
    let _ = writeln!(out, "branch-checkpoint {}", context.checkpoint());
    for operation in capture.operations() {
        let _ = writeln!(out, "maturity-step {}", operation.request().case.step);
        if let Some(response) = operation.response() {
            common::write_bytes_field(
                &mut out,
                "validated-response",
                &serde_json::to_vec(response).expect("protocol response serialization"),
            );
        }
    }
    match evidence {
        Ok(proof) => {
            for offer in proof.offers() {
                let _ = writeln!(
                    out,
                    "mutant-record {} declared {:?} observed {:?} carrier {:?} binding {:?}",
                    offer.step(),
                    offer.declared_boundary(),
                    offer.observed_layer(),
                    offer.carrier().execution(),
                    proof.binding(offer),
                );
            }
            match proof.control() {
                Some(control) => {
                    let _ = writeln!(
                        out,
                        "mutant-control {}",
                        control.identity().to_target_display()
                    );
                }
                None => {
                    let _ = writeln!(out, "mutant-control none");
                }
            }
        }
        Err(refusal) => {
            let _ = writeln!(out, "maturity-mutant-evidence unavailable");
            common::write_bytes_field(
                &mut out,
                "evidence-refusal",
                format!("{refusal:?}").as_bytes(),
            );
            let _ = writeln!(out, "mutant-control none");
        }
    }
    out
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

#[test]
#[ignore = "requires the revision-8 native adapter and capture environment"]
fn the_maturity_successor_mutants_are_refused_before_their_control_is_accepted() {
    let executor = required_environment("TRIPOD_LIVE_EXECUTOR");
    let directory = required_environment("TRIPOD_LIVE_REPORT_DIR");
    let mut destination = common::CaptureDestination::new(
        directory.clone().into(),
        required_environment("TRIPOD_LIVE_SUITE_SHORT_SHA"),
        required_environment("TRIPOD_LIVE_SUITE_COMMIT"),
        required_environment("TRIPOD_LIVE_SUITE_TREE"),
    )
    .expect("enhanced capture provenance");
    destination.report_test_name = RUN_TEST_NAME;
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    let deployment = native_identity();
    let context = branch();
    let mut planner =
        MaturityMutantCeremonyPlanner::new(deployment.clone(), context, TABLE).expect("planner");
    let mut capture = NativeOperationCapture::default();
    let outcome = execute(
        Path::new(&executor),
        Path::new(&directory),
        &deployment,
        ExecutorTrust::ReviewedNonMock,
        &mut planner,
        &mut capture,
    );
    let derived = planner.completed_transcript().and_then(|exchanges| {
        MaturityMutantEvidence::from_transcript(deployment.clone(), context, TABLE, exchanges)
    });
    let mut payload = evidence_payload(&capture, &deployment, context, &derived);
    if let Err(error) = &outcome {
        common::write_bytes_field(&mut payload, "executor-error", error.to_string().as_bytes());
    }
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture)
        .with_digest(
            "maturity-submission",
            planner.inner().submission_bytes().map(common::sha256),
        );
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    let transcript = outcome.expect("native exchange completed");
    let proof = derived.expect("exact completed transcript replay");
    assert_eq!(
        transcript.operation_requests().len(),
        SUCCESSOR_MUTANT_STEPS.len()
    );
    assert_eq!(
        transcript.operation_responses().len(),
        SUCCESSOR_MUTANT_STEPS.len()
    );
    assert_eq!(proof.offers().len(), TABLE.rows().len());
}

fn identity() -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("scripted identity")
}

fn blank_response(step: &OperationStep) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: step.case().clone(),
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: Vec::new(),
        confidential_funded_outputs: Vec::new(),
        mined_readback: None,
        accepted_txid: None,
        sponsor_witness: Vec::new(),
        script_path_witness: Vec::new(),
        signer_public_key: None,
        signed_profile: None,
        signing_genesis: None,
        signature_bound_to: None,
        resources: NativeResourceObservation::default(),
    }
}

fn answer(step: &OperationStep) -> NativeOperationResponse {
    let mut response = blank_response(step);
    match step.subject() {
        OperationSubject::Funding(subject) => {
            let asset = format!("01{}fe", "55".repeat(30));
            if subject.issue_asset {
                response.issued_asset = Some(asset.clone());
            }
            response.funded_outputs.push(FundedOutput {
                outpoint: WireOutpoint {
                    txid: format!("02{}fd", "66".repeat(30)),
                    vout: if subject.issue_asset { 3 } else { 7 },
                },
                asset,
                amount_satoshis: subject.amount_per_output,
                script: common::hex_bytes(&subject.output_program),
            });
        }
        OperationSubject::Submission(subject) => {
            response.observed_layer =
                declared_layer(&step.case().step).expect("submission declaration");
            if response.observed_layer == ObservedOutcomeLayer::Accepted {
                let candidate =
                    TargetTransaction::decode(&subject.transaction_bytes).expect("decode");
                let txid = display_hash(&candidate.encode_without_witness());
                response.accepted_txid = Some(txid.clone());
                response.mined_readback = Some(MinedFundingReadback {
                    transaction_id: txid,
                    witness_transaction_id: display_hash(&subject.transaction_bytes),
                    block_hash: "77".repeat(32),
                    block_height: 11,
                    raw_transaction: subject.transaction_bytes.clone(),
                });
            } else {
                response.observed_detail = Some("EC scalar mult verify fail".to_owned());
            }
        }
        _ => panic!("only funding and submission"),
    }
    response.validate_shape().expect("scripted response shape");
    response
}

fn display_hash(bytes: &[u8]) -> String {
    Txid::from_internal(common::sha256(&common::sha256(bytes))).to_target_display()
}

fn scripted_run() -> MaturityMutantCeremonyPlanner {
    let mut planner =
        MaturityMutantCeremonyPlanner::new(identity(), branch(), TABLE).expect("planner");
    let mut next = planner.next_step(None).expect("initial step");
    while let Some(step) = next {
        let response = answer(&step);
        next = planner
            .next_step(Some((step.case(), &response)))
            .expect("settled response");
    }
    planner
}

fn derive(planner: &MaturityMutantCeremonyPlanner) -> MaturityMutantEvidence {
    MaturityMutantEvidence::from_transcript(
        identity(),
        branch(),
        TABLE,
        planner.completed_transcript().expect("complete exchanges"),
    )
    .expect("replayed evidence")
}

#[test]
fn scripted_ceremony_settles_its_roster() {
    let planner = scripted_run();
    let exchanges = planner.completed_transcript().expect("complete");
    assert_eq!(exchanges.len(), roster(TABLE).len());
    for ((step, response), name) in exchanges.iter().zip(roster(TABLE)) {
        assert_eq!(&step.case().step, name);
        assert_eq!(response.case, *step.case());
    }
    let proof = derive(&planner);
    assert_eq!(proof.offers().len(), TABLE.rows().len());
    assert!(proof.control().is_some());
}

#[test]
fn scripted_payload_renders_every_record_in_order() {
    let planner = scripted_run();
    let proof = derive(&planner);
    let payload = evidence_payload(
        &NativeOperationCapture::default(),
        &identity(),
        branch(),
        &Ok(proof.clone()),
    );
    let lines = payload
        .lines()
        .filter(|line| line.starts_with("mutant-record "))
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), proof.offers().len());
    for (line, offer) in lines.iter().zip(proof.offers()) {
        assert_eq!(
            *line,
            format!(
                "mutant-record {} declared {:?} observed {:?} carrier {:?} binding {:?}",
                offer.step(),
                offer.declared_boundary(),
                offer.observed_layer(),
                offer.carrier().execution(),
                proof.binding(offer)
            )
        );
    }
}

#[cfg(unix)]
fn scripted_exchange() -> (
    MaturityMutantCeremonyPlanner,
    NativeOperationCapture,
    std::path::PathBuf,
) {
    use std::os::unix::fs::PermissionsExt as _;
    let scripted = scripted_run();
    let responses = scripted
        .completed_transcript()
        .expect("scripted exchanges")
        .iter()
        .map(|(_, response)| serde_json::to_string(response).expect("response JSON"))
        .collect::<Vec<_>>();
    let directory = common::test_directory("successor-mutant-scripted");
    let adapter = directory.join("scripted-adapter.sh");
    std::fs::write(
        &adapter,
        common::scripted_adapter(
            &["test_funding_ceremony", "target_transaction_submission"],
            &responses,
        ),
    )
    .expect("scripted protocol peer");
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755))
        .expect("scripted peer permissions");
    let mut planner =
        MaturityMutantCeremonyPlanner::new(identity(), branch(), TABLE).expect("planner");
    let mut capture = NativeOperationCapture::default();
    let transcript = execute(
        &adapter,
        &directory,
        &identity(),
        ExecutorTrust::Mock,
        &mut planner,
        &mut capture,
    )
    .expect("scripted captured exchange");
    assert_eq!(transcript.operation_requests().len(), roster(TABLE).len());
    (planner, capture, directory)
}

#[test]
#[cfg(unix)]
fn scripted_capture_is_written_before_the_gates() {
    let (planner, capture, directory) = scripted_exchange();
    let proof = derive(&planner);
    let payload = evidence_payload(&capture, &identity(), branch(), &Ok(proof));
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture)
        .with_digest(
            "maturity-submission",
            planner.inner().submission_bytes().map(common::sha256),
        );
    let mut destination = common::test_capture_destination(&directory);
    destination.report_test_name = RUN_TEST_NAME;
    let rendered = common::render_enhanced_capture(
        &destination,
        common::CeremonyId::Report,
        &capture,
        &facts,
        &payload,
    )
    .expect("enhanced capture");
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    assert_eq!(
        std::fs::read_to_string(guard.capture_path().expect("capture path")).expect("capture"),
        rendered
    );
    assert!(rendered.contains(&common::hex_bytes(payload.as_bytes())));
    assert_eq!(
        payload.matches("validated-response ").count(),
        roster(TABLE).len()
    );
    let subjects = rendered
        .lines()
        .filter(|line| line.starts_with("request-subject "))
        .map(|line| common::decode_request_subject_line(line).expect("recorded subject"))
        .collect::<Vec<_>>();
    let exchanges = planner.completed_transcript().expect("complete");
    assert_eq!(subjects.len(), exchanges.len());
    for (subject, (step, _)) in subjects.iter().zip(exchanges) {
        assert_eq!(subject, step.subject());
    }
    drop(guard);
    std::fs::remove_dir_all(directory).expect("capture directory removed");
}

#[test]
fn no_mutant_step_is_read_as_a_live_mutation() {
    for step in roster(TABLE) {
        assert_eq!(common::mutation_fact(step), (None, None));
    }
}

#[test]
fn shared_capture_names_preserve_the_legacy_roster() {
    let directory = common::test_directory("successor-mutant-capture-names");
    let destination = common::test_capture_destination(&directory);
    for ceremony in common::CeremonyId::ALL {
        assert!(
            destination
                .capture_path(ceremony, ceremony.rust_test_name())
                .is_ok()
        );
        assert!(
            destination
                .capture_path(ceremony, "unrelated-test")
                .is_err()
        );
    }
    let mut named = destination.clone();
    named.report_test_name = RUN_TEST_NAME;
    let report = common::CeremonyId::Report;
    assert!(named.capture_path(report, RUN_TEST_NAME).is_ok());
    assert!(named.capture_path(report, report.rust_test_name()).is_err());
    assert!(destination.capture_path(report, RUN_TEST_NAME).is_err());
    for ceremony in common::CeremonyId::ALL {
        if ceremony != report {
            assert_eq!(named.rust_test_name(ceremony), ceremony.rust_test_name());
        }
    }
    std::fs::remove_dir_all(directory).expect("naming directory removed");
}
