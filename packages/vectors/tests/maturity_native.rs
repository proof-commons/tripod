//! Native capture carrier for the sponsorless maturity announcement.
//!
//! The loop invokes `scripts/live-native-maturity-capture.sh` with its documented adapter, deployment and toolchain environment. That driver supplies the clean suite identity and new report directory and selects only the ignored test. The ordinary tests below drive scripted responses in process and require no node.
//! The caller's branch context is retained for replay and establishes no current-root freshness. The whole-metadata schedule declares the archived relay refusal; the variable-metadata schedule declares acceptance. Exact scripted replay can establish acceptance bindings without authenticating a native run or promoting a row standing.

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
use vectors::maturity_native::{
    ANNOUNCEMENT_STEPS, MaturityAcceptanceObligation, MaturityAcceptanceRoute,
    MaturityAnnouncementPlanner, MaturityNativeEvidence, MaturityNativePlanRefusal,
    MaturityNativeStanding, expected_layer, matches_boundary, observed_boundary,
};

/// Caller-supplied identifier and checkpoint for reproducible replay; these values carry no chain or freshness claim.
const RUN_BRANCH: ([u8; 32], u64) = ([0x41; 32], 7);

/// The ignored run's name retained by the shared generic report ceremony.
const RUN_TEST_NAME: &str = "the_maturity_announcement_runs_against_a_real_target";

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
    evidence: &Result<MaturityNativeEvidence, MaturityNativePlanRefusal>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "maturity-evidence-schema 1");
    common::write_bytes_field(&mut out, "deployment-network-id", identity.network_id());
    common::write_bytes_field(&mut out, "deployment-genesis-id", identity.genesis_id());
    common::write_bytes_field(&mut out, "branch-identifier", context.identifier());
    let _ = writeln!(out, "branch-checkpoint {}", context.checkpoint());
    let _ = writeln!(out, "current-root-freshness unestablished");
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
        Ok(evidence) => render_evidence(&mut out, evidence),
        Err(refusal) => {
            let _ = writeln!(out, "maturity-evidence unavailable");
            common::write_bytes_field(
                &mut out,
                "evidence-refusal",
                format!("{refusal:?}").as_bytes(),
            );
        }
    }
    out
}

fn render_evidence(out: &mut String, evidence: &MaturityNativeEvidence) {
    for observation in evidence.observations() {
        let _ = writeln!(
            out,
            "row {} declared {:?} observed {:?}",
            observation.subject(),
            observation.declared_layer(),
            observation.observed_layer()
        );
    }
    let _ = writeln!(out, "sponsorless-standing {:?}", evidence.standing());
    match evidence.acceptance_obligation() {
        MaturityAcceptanceObligation::Outstanding { routes } => {
            let _ = writeln!(out, "sponsorless-acceptance outstanding");
            for route in routes {
                let _ = writeln!(out, "acceptance-route {route:?}");
            }
        }
        MaturityAcceptanceObligation::Established {
            schedule,
            identity,
            readback,
        } => {
            let _ = writeln!(out, "sponsorless-acceptance established");
            let _ = writeln!(out, "accepted-schedule {}", schedule.name());
            let _ = writeln!(out, "accepted-identity {}", identity.to_target_display());
            let _ = writeln!(
                out,
                "accepted-witness-identity {}",
                readback.witness_identity().to_target_display()
            );
            let _ = writeln!(
                out,
                "accepted-block {} {}",
                common::hex_bytes(readback.block_hash()),
                readback.block_height()
            );
        }
    }
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
fn the_maturity_announcement_runs_against_a_real_target() {
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
    let mut planner = MaturityAnnouncementPlanner::new(
        deployment.clone(),
        context,
        vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
    )
    .expect("public announcement planner");
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
        MaturityNativeEvidence::from_transcript(
            deployment.clone(),
            context,
            vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
            exchanges,
        )
    });
    let mut payload = evidence_payload(&capture, &deployment, context, &derived);
    if let Err(error) = &outcome {
        common::write_bytes_field(&mut payload, "executor-error", error.to_string().as_bytes());
    }
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    let transcript = outcome.expect("native exchange completed");
    let evidence = derived.expect("exact completed transcript replay");
    assert_eq!(
        transcript.operation_requests().len(),
        ANNOUNCEMENT_STEPS.len()
    );
    assert_eq!(
        transcript.operation_responses().len(),
        ANNOUNCEMENT_STEPS.len()
    );
    assert_completed(&planner, &evidence);
}

fn assert_completed(planner: &MaturityAnnouncementPlanner, evidence: &MaturityNativeEvidence) {
    let exchanges = planner.completed_transcript().expect("completed exchanges");
    assert_eq!(exchanges.len(), ANNOUNCEMENT_STEPS.len());
    for ((step, response), name) in exchanges.iter().zip(ANNOUNCEMENT_STEPS) {
        assert_eq!(step.case().step, name);
        assert_eq!(response.case, *step.case());
        if name != "sponsorless" {
            assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
        }
    }
    let (step, response) = exchanges.last().expect("submission");
    let boundary =
        observed_boundary(expected_layer("sponsorless", planner.schedule()).expect("declaration"))
            .expect("target boundary");
    assert!(matches_boundary(boundary, response.observed_layer));
    assert_eq!(
        evidence.standing(),
        MaturityNativeStanding::AnsweredAtDeclaredBoundary
    );
    let OperationSubject::Submission(subject) = step.subject() else {
        panic!("submission subject")
    };
    assert_eq!(
        Some(subject.transaction_bytes.as_slice()),
        planner.submission_bytes()
    );
    let announcement = planner.announcement().expect("retained announcement");
    assert_eq!(
        planner.predecessor(),
        Some(announcement.protected().inputs()[0].outpoint())
    );
    assert_outstanding(evidence);
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

fn answer(step: &OperationStep, layer: ObservedOutcomeLayer) -> NativeOperationResponse {
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
            response.observed_layer = layer;
            if layer == ObservedOutcomeLayer::Accepted {
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
                response.observed_detail =
                    Some("scripted relay refusal; no node invoked".to_owned());
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

fn scripted_run(layer: ObservedOutcomeLayer) -> MaturityAnnouncementPlanner {
    scripted_run_for(tapscript::StateWitnessSchedule::WholeMetadata, layer)
}

fn scripted_run_for(
    schedule: tapscript::StateWitnessSchedule,
    layer: ObservedOutcomeLayer,
) -> MaturityAnnouncementPlanner {
    let mut planner = MaturityAnnouncementPlanner::new(
        identity(),
        branch(),
        vectors::maturity_closure::MaturityWitnessSelection::Retained(schedule),
    )
    .expect("planner");
    let mut current = planner.next_step(None).expect("initial step");
    while let Some(step) = current {
        let response = answer(&step, layer);
        let serialized = serde_json::to_vec(&response).expect("response JSON");
        assert_eq!(
            NativeOperationResponse::from_recorded_json(&serialized).expect("recorded response"),
            response
        );
        current = planner
            .next_step(Some((step.case(), &response)))
            .expect("settle scripted response");
    }
    planner
}

fn derive(
    planner: &MaturityAnnouncementPlanner,
) -> Result<MaturityNativeEvidence, MaturityNativePlanRefusal> {
    MaturityNativeEvidence::from_transcript(
        identity(),
        branch(),
        vectors::maturity_closure::MaturityWitnessSelection::Retained(planner.schedule()),
        planner.completed_transcript()?,
    )
}

fn assert_outstanding(evidence: &MaturityNativeEvidence) {
    assert_eq!(
        evidence.acceptance_obligation(),
        &MaturityAcceptanceObligation::Outstanding {
            routes: [
                MaturityAcceptanceRoute::RelayWitnessRestructure,
                MaturityAcceptanceRoute::BlockLayerSubmissionSubject
            ],
        }
    );
}

#[test]
fn scripted_relay_refusal_answers_the_declared_boundary() {
    let planner = scripted_run(ObservedOutcomeLayer::RelayPolicyRejection);
    let evidence = derive(&planner).expect("settled evidence");
    assert_completed(&planner, &evidence);
}

#[test]
fn whole_metadata_scripted_acceptance_is_off_declaration_and_outstanding() {
    let planner = scripted_run(ObservedOutcomeLayer::Accepted);
    let evidence = derive(&planner).expect("settled evidence");
    assert_eq!(
        evidence.standing(),
        MaturityNativeStanding::ObservedElsewhere
    );
    let submission = evidence.observations().last().expect("submission");
    assert_eq!(submission.observed_layer(), ObservedOutcomeLayer::Accepted);
    assert_eq!(
        submission.declared_layer(),
        expected_layer("sponsorless", planner.schedule())
    );
    assert_outstanding(&evidence);
}

#[test]
fn variable_scripted_acceptance_establishes_through_the_public_api() {
    let planner = scripted_run_for(
        tapscript::StateWitnessSchedule::VariableMetadata,
        ObservedOutcomeLayer::Accepted,
    );
    let evidence = derive(&planner).expect("accepted replay");
    assert_eq!(
        evidence.standing(),
        MaturityNativeStanding::AnsweredAtDeclaredBoundary
    );
    let MaturityAcceptanceObligation::Established {
        schedule,
        identity,
        readback,
    } = evidence.acceptance_obligation()
    else {
        panic!("variable acceptance must establish its checked readback")
    };
    assert_eq!(*schedule, planner.schedule());
    assert_eq!(*identity, readback.identity());
    assert_eq!(planner.readback(), Some(readback));
    assert_eq!(
        readback.bytes(),
        planner.submission_bytes().expect("submitted bytes")
    );
}

#[test]
fn established_payload_renders_five_lines_in_order() {
    let planner = scripted_run_for(
        tapscript::StateWitnessSchedule::VariableMetadata,
        ObservedOutcomeLayer::Accepted,
    );
    let evidence = derive(&planner).expect("accepted replay");
    let MaturityAcceptanceObligation::Established {
        schedule,
        identity,
        readback,
    } = evidence.acceptance_obligation()
    else {
        panic!("established value")
    };
    let mut text = String::new();
    render_evidence(&mut text, &evidence);
    let lines: Vec<_> = text.lines().collect();
    let expected = [
        "sponsorless-acceptance established".to_owned(),
        format!("accepted-schedule {}", schedule.name()),
        format!("accepted-identity {}", identity.to_target_display()),
        format!(
            "accepted-witness-identity {}",
            readback.witness_identity().to_target_display()
        ),
        format!(
            "accepted-block {} {}",
            common::hex_bytes(readback.block_hash()),
            readback.block_height()
        ),
    ];
    assert_eq!(&lines[lines.len() - expected.len()..], expected);
}

#[test]
fn incomplete_scripted_run_has_no_evidence() {
    let mut planner = MaturityAnnouncementPlanner::new(
        identity(),
        branch(),
        vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
    )
    .expect("planner");
    let issue = planner.next_step(None).expect("initial").expect("issue");
    let response = answer(&issue, ObservedOutcomeLayer::RelayPolicyRejection);
    planner
        .next_step(Some((issue.case(), &response)))
        .expect("funding step")
        .expect("next step");
    assert_eq!(
        derive(&planner),
        Err(MaturityNativePlanRefusal::IncompleteTranscript)
    );
    let payload = evidence_payload(
        &NativeOperationCapture::default(),
        &identity(),
        branch(),
        &derive(&planner),
    );
    assert!(payload.contains("maturity-evidence unavailable"));
    assert!(!payload.contains("sponsorless-standing"));
}

#[test]
fn payload_retains_standing_context_and_both_outstanding_acceptance_routes() {
    let planner = scripted_run(ObservedOutcomeLayer::RelayPolicyRejection);
    let payload = evidence_payload(
        &NativeOperationCapture::default(),
        &identity(),
        branch(),
        &derive(&planner),
    );
    assert!(payload.contains("sponsorless-standing AnsweredAtDeclaredBoundary\n"));
    assert!(payload.contains(
        "row sponsorless declared Some(RelayPolicyRejection) observed RelayPolicyRejection\n"
    ));
    assert!(payload.contains("sponsorless-acceptance outstanding\n"));
    assert!(payload.contains("acceptance-route RelayWitnessRestructure\n"));
    assert!(payload.contains("acceptance-route BlockLayerSubmissionSubject\n"));
    assert!(payload.contains(&format!(
        "deployment-network-id 32 {}\n",
        common::hex_bytes(identity().network_id())
    )));
    assert!(payload.contains(&format!(
        "deployment-genesis-id 32 {}\n",
        common::hex_bytes(identity().genesis_id())
    )));
    assert!(payload.contains(&format!(
        "branch-identifier 32 {}\n",
        common::hex_bytes(branch().identifier())
    )));
    assert!(payload.contains("branch-checkpoint 7\n"));
    assert!(payload.contains("current-root-freshness unestablished\n"));
}

#[test]
fn shared_capture_names_preserve_the_legacy_roster() {
    let directory = common::test_directory("maturity-capture-names");
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

#[test]
#[cfg(unix)]
fn scripted_capture_retains_replay_operands_before_gates() {
    use std::os::unix::fs::PermissionsExt as _;

    let scripted = scripted_run(ObservedOutcomeLayer::RelayPolicyRejection);
    let responses = scripted
        .completed_transcript()
        .expect("scripted exchanges")
        .iter()
        .map(|(_, response)| serde_json::to_string(response).expect("response JSON"))
        .collect::<Vec<_>>();
    let directory = common::test_directory("maturity-scripted-capture");
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
    let mut planner = MaturityAnnouncementPlanner::new(
        identity(),
        branch(),
        vectors::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
    )
    .expect("planner");
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
    assert_eq!(transcript.trust(), ExecutorTrust::Mock);
    let evidence = derive(&planner);
    let payload = evidence_payload(&capture, &identity(), branch(), &evidence);
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture)
        .with_digest(
            "maturity-submission",
            planner.submission_bytes().map(common::sha256),
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
    assert!(rendered.starts_with("native-capture-schema 2\n"));
    assert!(rendered.contains(&format!(
        "rust-test-name {} {}\n",
        RUN_TEST_NAME.len(),
        common::hex_bytes(RUN_TEST_NAME.as_bytes())
    )));
    assert!(rendered.contains("terminal-state complete\n"));
    assert!(rendered.contains(&common::hex_bytes(payload.as_bytes())));
    assert_eq!(
        payload.matches("validated-response ").count(),
        ANNOUNCEMENT_STEPS.len()
    );
    assert_capture_subjects(&rendered, &planner);
    assert_completed(&planner, &evidence.expect("replayed evidence"));
    drop(guard);
    std::fs::remove_dir_all(directory).expect("capture directory removed");
}

fn assert_capture_subjects(rendered: &str, planner: &MaturityAnnouncementPlanner) {
    let subjects = rendered
        .lines()
        .filter(|line| line.starts_with("request-subject "))
        .map(|line| common::decode_request_subject_line(line).expect("recorded subject"))
        .collect::<Vec<_>>();
    assert_eq!(subjects.len(), ANNOUNCEMENT_STEPS.len());
    let exchanges = planner.completed_transcript().expect("complete");
    for (subject, (step, _)) in subjects.iter().zip(exchanges) {
        assert_eq!(subject, step.subject());
    }
}
