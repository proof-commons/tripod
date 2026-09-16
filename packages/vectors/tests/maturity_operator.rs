//! Native carrier for one synthetic operator spend.
//!
//! Run with all seven variables set in the invoking shell:
//! ```sh
//! TRIPOD_LIVE_EXECUTOR=/path/to/adapter TRIPOD_LIVE_NETWORK_ID=NETWORK_HEX \
//! TRIPOD_LIVE_GENESIS_ID=GENESIS_HEX TRIPOD_LIVE_REPORT_DIR=/path/to/new-report \
//! TRIPOD_LIVE_SUITE_SHORT_SHA=SUITE_SHORT_ID TRIPOD_LIVE_SUITE_COMMIT=SUITE_COMMIT_ID \
//! TRIPOD_LIVE_SUITE_TREE=SUITE_TREE_ID \
//! cargo test -p tripod-vectors --test maturity_operator -- --ignored --test-threads=1
//! ```
//! Network and genesis are printed-order 32-byte hex; suite commit and tree are 40-digit hex.
//! The fixture converts the genesis to internal order only at the signing boundary.
//! The report directory must exist and its derived capture destination must be new.
//! The adapter must implement revision 8 funding, script-path signing and submission.
//! The shared writer uses its generic `Report` ceremony and required Rust test name.
//! Use a report directory dedicated to this operator run; its payload identifies the spend.
//! Native execution and subsequent strict capture admission happen outside this lane.

pub mod common;

use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

use linker::CandidateDeploymentIdentity;
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    ReproducibilityContract, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    ExecutionTranscript, ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust,
    NativeOperationCapture, OperationStep, PlanRefused, TargetOperationPlanner,
    execute_operations_captured,
};
use target_elements_conformance::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundingBinding,
    ConfidentialFundingDestination, ConfidentialFundingProfiles, FundedOutput,
    FundingCustodyProfile, FundingMaterializerProfile, FundingRepresentationProfile,
    MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse,
    NativeResourceObservation, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetConfidentialFundingSubject, TargetConfidentialSponsorFundingSubject,
    TargetFundingSubject, TargetSponsorFundingSubject, TargetSubmissionSubject, WireOutpoint,
    WireSighashProfile,
};
use target_elements_conformance::test_material::PublicTestSignerHandle;
use vectors::maturity_operator::{
    EVIDENCE_ROWS, NATIVE_STEPS, OPERATOR_HANDLE, OperatorEvidence, OperatorPlanner, expected_layer,
};

fn identity() -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new(common::identifier(&"11".repeat(32)), [0x22; 32])
        .expect("deployment")
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

fn answer(planner: &OperatorPlanner, step: &OperationStep) -> NativeOperationResponse {
    let mut response = blank_response(step);
    match step.subject() {
        OperationSubject::Funding(subject) => {
            if subject.issue_asset {
                response.issued_asset = Some("55".repeat(32));
            }
            response.funded_outputs.push(FundedOutput {
                outpoint: WireOutpoint {
                    txid: "33".repeat(32),
                    vout: 0,
                },
                asset: "55".repeat(32),
                amount_satoshis: subject.amount_per_output,
                script: common::hex_bytes(&subject.output_program),
            });
        }
        OperationSubject::ScriptPathSigning(subject) => {
            let request = planner.fixture().expect("funded").freeze().expect("frozen");
            response.script_path_witness = vec![
                subject
                    .signer
                    .material()
                    .expect("public material")
                    .sign(request.message().with_vector_grown(), &[0; 32])
                    .expect("signature")
                    .to_vec(),
            ];
            response.signer_public_key =
                Some(subject.signer.x_only_public_key().expect("public key"));
            response.signed_profile = Some(subject.sighash_profile);
            response.signing_genesis = Some([0x22; 32]);
            response.signature_bound_to = Some(subject.finalized_transaction.clone());
        }
        OperationSubject::Submission(subject) => {
            if step.case().step == "operator-positive" {
                let candidate =
                    transaction::bytes::TargetTransaction::decode(&subject.transaction_bytes)
                        .expect("candidate decodes");
                let txid = display_hash(&candidate.encode_without_witness());
                response.accepted_txid = Some(txid.clone());
                response.mined_readback = Some(MinedFundingReadback {
                    transaction_id: txid,
                    witness_transaction_id: display_hash(&subject.transaction_bytes),
                    block_hash: "44".repeat(32),
                    block_height: 1,
                    raw_transaction: subject.transaction_bytes.clone(),
                });
            } else {
                response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
                response.observed_detail =
                    Some("scripted signature or commitment refusal".to_owned());
            }
        }
        _ => panic!("unexpected operation"),
    }
    response.validate_shape().expect("protocol shape");
    response
}

fn display_hash(bytes: &[u8]) -> String {
    let mut digest = common::sha256(&common::sha256(bytes));
    digest.reverse();
    common::hex_bytes(&digest)
}

fn json_optional<T: std::fmt::Debug>(value: Option<&T>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| format!("{value:?}"))
}

fn json_text(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "null".to_owned();
    };
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            control if control.is_control() => {
                let _ = write!(out, "\\u{:04x}", u32::from(control));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn response_json(response: &NativeOperationResponse) -> String {
    let layer = match response.observed_layer {
        ObservedOutcomeLayer::Accepted => "accepted",
        ObservedOutcomeLayer::ScriptPathRejection => "script_path_rejection",
        ObservedOutcomeLayer::FixtureConstructionFailure => "fixture_construction_failure",
        ObservedOutcomeLayer::ExecutorInfrastructureFailure => "executor_infrastructure_failure",
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript => "consensus_rejection_before_script",
        ObservedOutcomeLayer::KeyPathRejection => "key_path_rejection",
        ObservedOutcomeLayer::RelayPolicyRejection => "relay_policy_rejection",
        _ => panic!("unrecognized protocol layer"),
    };
    let funded = response.funded_outputs.iter().map(|coin| format!(
        "{{\"outpoint\":{{\"txid\":\"{}\",\"vout\":{}}},\"asset\":\"{}\",\"amount_satoshis\":{},\"script\":\"{}\"}}",
        coin.outpoint.txid, coin.outpoint.vout, coin.asset, coin.amount_satoshis, coin.script,
    )).collect::<Vec<_>>().join(",");
    let readback = response.mined_readback.as_ref().map_or_else(|| "null".to_owned(), |value| format!(
        "{{\"transaction_id\":\"{}\",\"witness_transaction_id\":\"{}\",\"block_hash\":\"{}\",\"block_height\":{},\"raw_transaction\":{:?}}}",
        value.transaction_id, value.witness_transaction_id, value.block_hash, value.block_height, value.raw_transaction,
    ));
    let profile = if response.signed_profile.is_some() {
        "\"all_inputs_all_outputs\""
    } else {
        "null"
    };
    format!(
        "{{\"schema\":8,\"case\":{{\"operation\":\"{}\",\"step\":\"{}\"}},\"observed_layer\":\"{layer}\",\"observed_detail\":{},\"issued_asset\":{},\"funded_outputs\":[{funded}],\"confidential_funded_outputs\":[],\"mined_readback\":{readback},\"accepted_txid\":{},\"sponsor_witness\":[],\"script_path_witness\":{:?},\"signer_public_key\":{},\"signed_profile\":{profile},\"signing_genesis\":{},\"signature_bound_to\":{},\"resources\":{{\"script_bytes\":{},\"initial_stack_items\":{},\"peak_stack_items\":{},\"peak_altstack_items\":{},\"maximum_element_bytes\":{},\"validation_budget_used\":{},\"transaction_weight\":{}}}}}",
        response.case.operation,
        response.case.step,
        json_text(response.observed_detail.as_deref()),
        json_optional(response.issued_asset.as_ref()),
        json_optional(response.accepted_txid.as_ref()),
        response.script_path_witness,
        json_optional(response.signer_public_key.as_ref()),
        json_optional(response.signing_genesis.as_ref()),
        json_optional(response.signature_bound_to.as_ref()),
        json_optional(response.resources.script_bytes.as_ref()),
        json_optional(response.resources.initial_stack_items.as_ref()),
        json_optional(response.resources.peak_stack_items.as_ref()),
        json_optional(response.resources.peak_altstack_items.as_ref()),
        json_optional(response.resources.maximum_element_bytes.as_ref()),
        json_optional(response.resources.validation_budget_used.as_ref()),
        json_optional(response.resources.transaction_weight.as_ref()),
    )
}

fn scripted_responses() -> (Vec<OperationStep>, Vec<String>) {
    let mut planner = OperatorPlanner::new(identity());
    let mut current = planner.next_step(None).expect("first step");
    let mut steps = Vec::new();
    let mut responses = Vec::new();
    while let Some(step) = current {
        let response = answer(&planner, &step);
        responses.push(response_json(&response));
        current = planner
            .next_step(Some((step.case(), &response)))
            .expect("next step");
        steps.push(step);
    }
    (steps, responses)
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

#[derive(Clone)]
struct RetainedSubjectPlan {
    steps: Vec<OperationStep>,
    next: usize,
}

impl TargetOperationPlanner for RetainedSubjectPlan {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if previous.is_some() {
            self.next += 1;
        }
        Ok(self.steps.get(self.next).cloned())
    }
}

fn retained_subjects() -> Vec<OperationSubject> {
    let binding = ConfidentialFundingBinding {
        fixture_handle: ConfidentialFixtureHandle::new("request-subject-fixture".to_owned()),
        fixture_digest: ConfidentialFixtureDigest::new([0x44; 32]),
        profiles: ConfidentialFundingProfiles {
            representation: FundingRepresentationProfile::ExplicitAssetConfidentialValue,
            custody: FundingCustodyProfile::CentralPublicFixtures,
            materializer: FundingMaterializerProfile::GuideCtfDeterministicV1,
            reproducibility_contract: ReproducibilityContract::ByteIdentity,
        },
    };
    vec![
        OperationSubject::Funding(Box::new(TargetFundingSubject {
            issue_asset: true,
            asset: None,
            output_program: vec![0x51],
            outputs: 1,
            amount_per_output: 7,
        })),
        OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
            sponsor_outputs: 1,
            amount_per_sponsor_output: 11,
        })),
        OperationSubject::ConfidentialFunding(Box::new(TargetConfidentialFundingSubject {
            issue_asset: false,
            asset: Some("11".repeat(32)),
            destinations: vec![ConfidentialFundingDestination {
                output_program: vec![0x51, 0x20],
            }],
            binding: binding.clone(),
        })),
        OperationSubject::ConfidentialSponsorFunding(Box::new(
            TargetConfidentialSponsorFundingSubject {
                destinations: vec![ConfidentialFundingDestination {
                    output_program: vec![0x51, 0x21],
                }],
                binding,
            },
        )),
        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
            transaction_bytes: vec![0x02, 0x00, 0x01, 0xff],
        })),
    ]
}

#[cfg(unix)]
fn rendered_request_subject_capture() -> (NativeOperationCapture, String) {
    use std::os::unix::fs::PermissionsExt as _;

    let directory = common::test_directory("request-subject");
    let adapter = directory.join("adapter.sh");
    let steps = retained_subjects()
        .into_iter()
        .enumerate()
        .map(|(index, subject)| OperationStep::new(&format!("subject-{index}"), subject))
        .collect::<Vec<_>>();
    let responses = steps
        .iter()
        .map(|step| {
            let mut response = blank_response(step);
            response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
            response.observed_detail = Some("scripted subject refusal".to_owned());
            response_json(&response)
        })
        .collect::<Vec<_>>();
    std::fs::write(
        &adapter,
        common::scripted_adapter(
            &[
                "test_funding_ceremony",
                "target_transaction_submission",
                "test_sponsor_authorization",
                "confidential_value_test_funding",
                "confidential_value_sponsor_authorization",
            ],
            &responses,
        ),
    )
    .expect("scripted subject adapter");
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755))
        .expect("subject adapter permissions");
    let mut planner = RetainedSubjectPlan { steps, next: 0 };
    let mut capture = NativeOperationCapture::default();
    execute(
        &adapter,
        &directory,
        &identity(),
        ExecutorTrust::Mock,
        &mut planner,
        &mut capture,
    )
    .expect("scripted subject exchange");
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    let mut rendered = String::new();
    common::render_operations(&mut rendered, &capture, &facts).expect("operation block");
    std::fs::remove_dir_all(directory).expect("subject directory removed");
    (capture, rendered)
}

#[cfg(unix)]
#[test]
fn every_supported_funding_and_submission_subject_round_trips_through_the_writer() {
    let (capture, rendered) = rendered_request_subject_capture();
    let subject_lines = rendered
        .lines()
        .filter(|line| line.starts_with("request-subject "));
    assert_eq!(capture.operations().len(), 5);
    for (operation, line) in capture.operations().iter().zip(subject_lines) {
        assert_eq!(
            common::decode_request_subject_line(line).expect("retained subject decodes"),
            operation.request().subject,
        );
    }
}

#[cfg(unix)]
#[test]
fn every_operation_block_has_one_request_subject_in_protocol_order() {
    let (capture, rendered) = rendered_request_subject_capture();
    let lines = rendered.lines().collect::<Vec<_>>();
    let ordered = lines
        .windows(3)
        .filter(|window| window[0].starts_with("request-bytes "))
        .inspect(|window| {
            assert!(window[1].starts_with("request-subject "));
            assert!(window[2].starts_with("response-id "));
        })
        .count();
    assert_eq!(ordered, capture.operations().len());
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("request-subject "))
            .count(),
        capture.operations().len(),
    );
}

#[cfg(unix)]
#[test]
fn submission_subject_and_request_bytes_retain_the_same_transaction() {
    let (capture, rendered) = rendered_request_subject_capture();
    let (index, operation) = capture
        .operations()
        .iter()
        .enumerate()
        .find(|(_, operation)| {
            matches!(operation.request().subject, OperationSubject::Submission(_))
        })
        .expect("submission operation");
    let line = rendered
        .lines()
        .filter(|line| line.starts_with("request-subject "))
        .nth(index)
        .expect("submission subject line");
    let OperationSubject::Submission(subject) =
        common::decode_request_subject_line(line).expect("submission subject decodes")
    else {
        panic!("retained subject is not a submission");
    };
    let transaction_bytes = operation.transaction_bytes().expect("submission bytes");
    assert_eq!(subject.transaction_bytes, transaction_bytes);
    assert_eq!(
        rendered
            .lines()
            .filter(|line| line.starts_with("request-bytes "))
            .nth(index)
            .expect("submission request bytes line"),
        format!(
            "request-bytes {} {}",
            transaction_bytes.len(),
            common::hex_bytes(transaction_bytes),
        ),
    );
}

#[cfg(unix)]
fn scripted_exchange() -> (ExecutionTranscript, NativeOperationCapture) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = common::test_directory("operator");
    let adapter = directory.join("adapter.sh");
    let (_, responses) = scripted_responses();
    std::fs::write(
        &adapter,
        common::scripted_adapter(
            &[
                "test_funding_ceremony",
                "test_script_path_authorization",
                "target_transaction_submission",
            ],
            &responses,
        ),
    )
    .expect("scripted adapter");
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755))
        .expect("permissions");
    let mut planner = OperatorPlanner::new(identity());
    let mut capture = NativeOperationCapture::default();
    let transcript = execute(
        &adapter,
        &directory,
        &identity(),
        ExecutorTrust::Mock,
        &mut planner,
        &mut capture,
    )
    .expect("scripted revision-8 exchange");
    std::fs::remove_dir_all(directory).expect("scripted directory removed");
    (transcript, capture)
}

fn assert_evidence(evidence: &OperatorEvidence, native: bool) {
    assert!(evidence.public_context_verified);
    assert!(evidence.signature_verified);
    assert!(evidence.byte_identical_readback);
    assert_eq!(evidence.message.as_ref().expect("message").len(), 32);
    assert_eq!(evidence.rows.len(), EVIDENCE_ROWS.len());
    for row in &evidence.rows {
        if let Some(layer) = row.expected_layer {
            assert_eq!(row.observed_layer, Some(layer), "{}", row.subject);
            assert_eq!(row.standing.is_answered(), native, "{}", row.subject);
        } else {
            assert_eq!(
                row.standing.is_answered(),
                row.subject != "operator-membership",
                "{}",
                row.subject
            );
        }
    }
}

#[test]
fn the_exact_step_census_carries_only_public_signer_handles() {
    let (steps, _) = scripted_responses();
    assert_eq!(steps.len(), 11);
    for (index, step) in steps.iter().enumerate() {
        assert_eq!(step.case().step, NATIVE_STEPS[index]);
        match step.subject() {
            OperationSubject::Funding(subject) => {
                assert!(index < 2);
                assert_eq!(subject.issue_asset, index == 0);
                assert_eq!(subject.outputs, 1);
                assert_eq!(subject.amount_per_output, 5000);
                assert_eq!(subject.output_program.len(), 34);
                assert_eq!(subject.asset, (index == 1).then(|| "55".repeat(32)));
            }
            OperationSubject::ScriptPathSigning(subject) => {
                assert_eq!(
                    subject.signer,
                    if index == 2 {
                        PublicTestSignerHandle::First
                    } else {
                        OPERATOR_HANDLE
                    }
                );
                assert_eq!(subject.input_index, 0);
                assert_eq!(subject.spent_outputs.len(), 1);
                assert_eq!(subject.spent_outputs[0].asset_field.len(), 33);
                assert_eq!(subject.spent_outputs[0].value_field.len(), 9);
                assert_eq!(subject.spent_outputs[0].program.len(), 34);
                assert_eq!(subject.executing_leaf.script.len(), 34);
                assert_eq!(subject.executing_leaf.control_block.len(), 33);
                assert_eq!(
                    subject.sighash_profile,
                    WireSighashProfile::AllInputsAllOutputs
                );
            }
            OperationSubject::Submission(subject) => {
                assert!((4..11).contains(&index));
                let candidate =
                    transaction::bytes::TargetTransaction::decode(&subject.transaction_bytes)
                        .expect("candidate");
                assert_eq!(candidate.inputs().len(), 1);
                assert_eq!(candidate.outputs().len(), 2);
                assert_eq!(candidate.witnesses()[0].stack().len(), 3);
            }
            _ => panic!("unexpected subject"),
        }
    }
}

#[test]
#[cfg(unix)]
fn scripted_signing_verifies_the_positive_and_every_declared_layer() {
    let (transcript, _) = scripted_exchange();
    let evidence =
        OperatorEvidence::from_transcript(identity(), &transcript).expect("evidence replay");
    assert_evidence(&evidence, false);
}

#[test]
#[cfg(unix)]
fn another_deployment_cannot_admit_the_scripted_transcript() {
    let (transcript, _) = scripted_exchange();
    let other = CandidateDeploymentIdentity::new([0x11; 32], [0x23; 32]).expect("other deployment");
    assert!(OperatorEvidence::from_transcript(other, &transcript).is_err());
}

#[test]
fn missing_evidence_answers_no_row() {
    let evidence = OperatorEvidence::incomplete();
    assert_eq!(evidence.rows.len(), 11);
    assert!(evidence.rows.iter().all(|row| !row.standing.is_answered()));
    assert!(!evidence.signature_verified);
    assert!(!evidence.byte_identical_readback);
    assert_eq!(
        expected_layer("wrong-key"),
        Some(ObservedOutcomeLayer::ScriptPathRejection)
    );
}

fn control_block(script: &[u8]) -> Vec<u8> {
    use target_elements_conformance::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
    use transaction::live_taproot::LiveCurveCapability as _;
    let target = reviewed_elements_tapscript().expect("target");
    let leaf = transaction::taproot::leaf_hash(LeafVersion::TAPSCRIPT, script);
    let output = vectors::OracleLiveCurve::new(target)
        .output_key(&UNSPENDABLE_INTERNAL_KEY, &leaf)
        .expect("output key");
    let mut control = vec![LeafVersion::TAPSCRIPT.get() | output.parity().bit()];
    control.extend_from_slice(&UNSPENDABLE_INTERNAL_KEY);
    control
}

#[test]
fn every_submission_changes_only_its_declared_subject() {
    use transaction::bytes::TargetTransaction;
    let (steps, _) = scripted_responses();
    let candidates: Vec<_> = steps
        .iter()
        .filter_map(|step| match step.subject() {
            OperationSubject::Submission(subject) => {
                Some(TargetTransaction::decode(&subject.transaction_bytes).expect("submission"))
            }
            _ => None,
        })
        .collect();
    let positive = &candidates[6];
    let positive_stack = positive.witnesses()[0].stack();
    for (index, candidate) in candidates.iter().enumerate() {
        let stack = candidate.witnesses()[0].stack();
        assert_eq!(candidate.version(), positive.version());
        assert_eq!(candidate.lock_time(), positive.lock_time());
        assert_eq!(candidate.output_witnesses(), positive.output_witnesses());
        if index == 1 {
            assert_eq!(
                candidate.inputs()[0].outpoint(),
                positive.inputs()[0].outpoint()
            );
            assert_eq!(
                candidate.inputs()[0].sequence(),
                positive.inputs()[0].sequence() ^ 1
            );
        } else {
            assert_eq!(candidate.inputs(), positive.inputs());
        }
        if index == 5 {
            assert_eq!(
                candidate.outputs(),
                &[positive.outputs()[1].clone(), positive.outputs()[0].clone()]
            );
        } else {
            assert_eq!(candidate.outputs(), positive.outputs());
        }
        match index {
            0 => {
                assert_ne!(stack[0], positive_stack[0]);
                assert_eq!(&stack[1..], &positive_stack[1..]);
            }
            2 => {
                let mut script = vec![32];
                script.extend_from_slice(
                    &PublicTestSignerHandle::First
                        .x_only_public_key()
                        .expect("first key"),
                );
                script.push(0xac);
                assert_eq!(stack[0], positive_stack[0]);
                assert_eq!(stack[1], script);
                assert_eq!(stack[2], control_block(&script));
                assert_eq!(&stack[2][1..], &positive_stack[2][1..]);
            }
            3 => {
                assert_eq!(stack[0], positive_stack[0][..63]);
                assert_eq!(&stack[1..], &positive_stack[1..]);
            }
            4 => {
                let mut signature = positive_stack[0].clone();
                signature.push(1);
                assert_eq!(stack[0], signature);
                assert_eq!(&stack[1..], &positive_stack[1..]);
            }
            _ => assert_eq!(stack, positive_stack),
        }
    }
    let OperationSubject::ScriptPathSigning(first) = steps[2].subject() else {
        panic!("first signer")
    };
    let OperationSubject::ScriptPathSigning(third) = steps[3].subject() else {
        panic!("operator signer")
    };
    let mut same = first.clone();
    same.signer = OPERATOR_HANDLE;
    assert_eq!(&same, third);
}

fn evidence_payload(capture: &NativeOperationCapture, evidence: &OperatorEvidence) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "operator-evidence-schema 1");
    let _ = writeln!(out, "state-construction incomplete");
    let _ = writeln!(out, "operator-membership external-evidence-required");
    if let Some(message) = evidence.message.as_deref() {
        common::write_bytes_field(&mut out, "recomputed-message", message);
    }
    for operation in capture.operations() {
        let _ = writeln!(out, "operator-step {}", operation.request().case.step);
        if let OperationSubject::ScriptPathSigning(subject) = &operation.request().subject {
            let _ = writeln!(out, "signer-handle {:?}", subject.signer);
            let _ = writeln!(out, "input-index {}", subject.input_index);
            let _ = writeln!(out, "profile {:?}", subject.sighash_profile);
            common::write_bytes_field(
                &mut out,
                "finalized-transaction",
                &subject.finalized_transaction,
            );
            let _ = writeln!(out, "spent-output-count {}", subject.spent_outputs.len());
            for spent in &subject.spent_outputs {
                common::write_bytes_field(&mut out, "spent-asset", &spent.asset_field);
                common::write_bytes_field(&mut out, "spent-value", &spent.value_field);
                common::write_bytes_field(&mut out, "spent-program", &spent.program);
            }
            let _ = writeln!(out, "leaf-version {}", subject.executing_leaf.leaf_version);
            common::write_bytes_field(&mut out, "leaf-script", &subject.executing_leaf.script);
            common::write_bytes_field(
                &mut out,
                "control-block",
                &subject.executing_leaf.control_block,
            );
        }
        if let Some(response) = operation.response() {
            // The complete validated response retains funding, signature and readback operands.
            common::write_bytes_field(
                &mut out,
                "validated-response",
                response_json(response).as_bytes(),
            );
        }
    }
    for row in &evidence.rows {
        let _ = writeln!(
            out,
            "row {} expected {:?} observed {:?} standing {:?}",
            row.subject, row.expected_layer, row.observed_layer, row.standing
        );
    }
    out
}

#[test]
fn membership_and_complete_state_construction_stay_explicitly_open() {
    let evidence = OperatorEvidence::incomplete();
    assert_eq!(
        evidence.membership_requirement,
        realization::ExternalEvidenceRequirement::OperatorAuthorization {
            operation: architecture::OperationId::AnnounceMaturity,
        }
    );
    assert_eq!(evidence.unconstructed_state.len(), 12);
    for side in [
        realization::TransactionSide::Input,
        realization::TransactionSide::Output,
    ] {
        for field in realization::StateField::ALL {
            assert!(
                evidence
                    .unconstructed_state
                    .contains(&realization::FactId::StateField {
                        operation: architecture::OperationId::AnnounceMaturity,
                        side,
                        field: *field,
                    })
            );
        }
    }
    assert!(
        !evidence
            .rows
            .iter()
            .find(|row| row.subject == "operator-membership")
            .expect("membership row")
            .standing
            .is_answered()
    );
}

#[test]
#[cfg(unix)]
fn enhanced_capture_retains_the_exact_signing_and_readback_operands() {
    let (transcript, capture) = scripted_exchange();
    let evidence = OperatorEvidence::from_transcript(identity(), &transcript).expect("replay");
    let payload = evidence_payload(&capture, &evidence);
    assert!(payload.contains("signer-handle Third"));
    assert!(payload.contains("spent-output-count 1"));
    assert!(payload.contains("recomputed-message 32 "));
    assert_eq!(payload.matches("validated-response ").count(), 11);
    let directory = common::test_directory("operator-payload");
    let destination = common::test_capture_destination(&directory);
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture)
        .with_digest(
            "operator-message",
            Some(
                evidence
                    .message
                    .as_ref()
                    .expect("message")
                    .as_slice()
                    .try_into()
                    .expect("32 bytes"),
            ),
        );
    let rendered = common::render_enhanced_capture(
        &destination,
        common::CeremonyId::Report,
        &capture,
        &facts,
        &payload,
    )
    .expect("shared enhanced format");
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    assert_eq!(
        std::fs::read_to_string(guard.capture_path().expect("capture path")).expect("capture"),
        rendered
    );
    drop(guard);
    assert!(rendered.contains(&common::hex_bytes(payload.as_bytes())));
    assert!(rendered.contains("terminal-state complete"));
    std::fs::remove_dir_all(directory).expect("capture directory removed");
}

#[test]
fn shared_capture_names_preserve_the_legacy_roster() {
    let directory = common::test_directory("operator-legacy-names");
    let destination = common::test_capture_destination(&directory);
    assert_eq!(common::CeremonyId::ALL.len(), 43);
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
    std::fs::remove_dir_all(directory).expect("naming directory removed");
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
fn the_live_transfer_candidate_runs_against_a_real_target() {
    let executor = required_environment("TRIPOD_LIVE_EXECUTOR");
    let directory = required_environment("TRIPOD_LIVE_REPORT_DIR");
    let destination = common::CaptureDestination::new(
        directory.clone().into(),
        required_environment("TRIPOD_LIVE_SUITE_SHORT_SHA"),
        required_environment("TRIPOD_LIVE_SUITE_COMMIT"),
        required_environment("TRIPOD_LIVE_SUITE_TREE"),
    )
    .expect("enhanced capture provenance");
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    let deployment = native_identity();
    let mut planner = OperatorPlanner::new(deployment.clone());
    let mut capture = NativeOperationCapture::default();
    let outcome = execute(
        Path::new(&executor),
        Path::new(&directory),
        &deployment,
        ExecutorTrust::ReviewedNonMock,
        &mut planner,
        &mut capture,
    );
    let derived = outcome
        .as_ref()
        .ok()
        .map(|transcript| OperatorEvidence::from_transcript(deployment, transcript));
    let evidence = derived
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_else(OperatorEvidence::incomplete);
    let mut payload = evidence_payload(&capture, &evidence);
    if let Err(error) = &outcome {
        common::write_bytes_field(&mut payload, "executor-error", error.to_string().as_bytes());
    }
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    outcome.expect("native exchange completed");
    derived
        .expect("completed transcript")
        .expect("exact evidence replay");
    let fixture = planner.fixture().expect("funded fixture");
    assert_eq!(fixture.candidate().inputs().len(), 1);
    assert_eq!(fixture.candidate().outputs().len(), 2);
    assert_eq!(
        fixture.signing_subject().expect("signing subject").signer,
        OPERATOR_HANDLE
    );
    assert_evidence(&evidence, true);
}

fn admitted_operator() -> &'static vectors::ValidatedNativeOperatorCorpus {
    vectors::native_operator_run_of_record().expect("reviewed operator corpus admits")
}

fn admitted_row(name: &str) -> &vectors::maturity_operator::OperatorEvidenceRow {
    admitted_operator()
        .evidence()
        .rows
        .iter()
        .find(|row| row.subject == name)
        .expect("row")
}

fn native_control(name: &str, detail: &str) {
    use vectors::live_evidence::{LiveRowStanding, NativeObservedAcceptance};
    let LiveRowStanding::NativeRunObserved {
        acceptance: NativeObservedAcceptance::Single(identity),
    } = admitted_row("operator-positive").standing
    else {
        panic!("native positive")
    };
    assert_eq!(
        admitted_row(name).standing,
        LiveRowStanding::NativeRefusalObserved {
            declared_boundary: vectors::EvidenceBoundary::ScriptPathRejection,
            observed_layer: ObservedOutcomeLayer::ScriptPathRejection,
            control_identity: identity,
            refusal_detail: detail.to_owned(),
        }
    );
}

#[test]
fn admitted_positive_identity_equals_readback_and_recomputed_txid() {
    use target_elements_conformance::constructor::tagged;
    use vectors::live_evidence::{LiveRowStanding, NativeObservedAcceptance};
    let readback = admitted_operator().exchanges()[10]
        .1
        .mined_readback
        .as_ref()
        .expect("readback");
    let transaction =
        transaction::TargetTransaction::decode(&readback.raw_transaction).expect("transaction");
    let first = tagged::sha256(&transaction.encode_without_witness());
    let identity = transaction::Txid::from_internal(tagged::sha256(&first));
    assert_eq!(identity.to_target_display(), readback.transaction_id);
    assert_eq!(
        admitted_operator().exchanges()[10]
            .1
            .accepted_txid
            .as_deref(),
        Some(readback.transaction_id.as_str())
    );
    assert_eq!(
        admitted_row("operator-positive").standing,
        LiveRowStanding::NativeRunObserved {
            acceptance: NativeObservedAcceptance::Single(identity)
        }
    );
    assert!(admitted_operator().evidence().byte_identical_readback);
}

#[test]
fn admitted_wrong_key_is_native_script_refusal() {
    native_control(
        "wrong-key",
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)",
    );
}

#[test]
fn admitted_wrong_candidate_is_native_script_refusal() {
    native_control(
        "wrong-candidate",
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)",
    );
}

#[test]
fn admitted_wrong_leaf_is_native_script_refusal() {
    native_control(
        "wrong-leaf",
        "mandatory-script-verify-flag-failed (Witness program hash mismatch)",
    );
}

#[test]
fn admitted_signature_width_is_native_script_refusal() {
    native_control(
        "signature-width",
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature size)",
    );
}

#[test]
fn admitted_signature_type_is_native_script_refusal() {
    native_control(
        "signature-type",
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)",
    );
}

#[test]
fn admitted_protected_term_is_native_script_refusal() {
    native_control(
        "protected-term",
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)",
    );
}

fn admitted_first_party(name: &str, fact: &'static str) {
    assert_eq!(
        admitted_row(name).standing,
        vectors::LiveRowStanding::FirstPartyFactObserved {
            fact,
            observed_by: "OperatorEvidence::from_transcript",
        }
    );
}

#[test]
fn admitted_unknown_key_admission_is_first_party() {
    admitted_first_party(
        "unknown-key-admission",
        "deployment binding refused an unbound operator key",
    );
}

#[test]
fn admitted_duplicate_authorization_is_first_party() {
    admitted_first_party(
        "duplicate-authorization",
        "affine registry refused a second outstanding right before signing",
    );
}

#[test]
fn admitted_signature_verification_is_first_party() {
    admitted_first_party(
        "signature-verification",
        "public-data Schnorr verification bound the signature to the recomputed message",
    );
    assert!(admitted_operator().evidence().signature_verified);
    assert!(admitted_operator().evidence().public_context_verified);
}

#[test]
fn admitted_membership_and_twelve_state_facts_remain_external() {
    let evidence = admitted_operator().evidence();
    assert_eq!(
        evidence.membership_requirement,
        OperatorEvidence::incomplete().membership_requirement
    );
    assert_eq!(
        evidence.unconstructed_state,
        OperatorEvidence::incomplete().unconstructed_state
    );
    assert_eq!(evidence.unconstructed_state.len(), 12);
    assert_eq!(
        admitted_row("operator-membership").standing,
        vectors::LiveRowStanding::NativeRunRequired(None)
    );
    assert_eq!(
        evidence
            .rows
            .iter()
            .map(|row| row.subject)
            .collect::<Vec<_>>(),
        EVIDENCE_ROWS
    );
}

#[test]
fn admitted_message_equals_independent_recomputation() {
    assert_eq!(
        admitted_operator().evidence().message.as_deref(),
        Some(admitted_operator().recorded_message())
    );
}

#[test]
fn admitted_signing_keys_match_first_and_third_handles() {
    for (index, handle) in [
        (2, PublicTestSignerHandle::First),
        (3, PublicTestSignerHandle::Third),
    ] {
        assert_eq!(
            admitted_operator().exchanges()[index].1.signer_public_key,
            Some(handle.x_only_public_key().expect("public key"))
        );
    }
}

#[test]
fn admitted_environment_has_fourteen_capabilities_including_script_signing() {
    let capabilities = admitted_operator().capabilities();
    assert_eq!(capabilities.len(), 14);
    assert_eq!(capabilities[11], "test-script-path-authorization");
}

#[test]
fn admitted_suite_and_deployment_bindings_agree() {
    let corpus = admitted_operator();
    assert_eq!(
        corpus.report().suite_commit(),
        corpus.capture_suite_commit()
    );
    assert_eq!(corpus.report().suite_tree(), corpus.capture_suite_tree());
    for identity in [corpus.report().suite_commit(), corpus.report().suite_tree()] {
        assert_eq!(identity.len(), 40);
        assert!(
            identity
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        );
    }
    let prior = vectors::live_corpus_native_v2_r7::run_of_record().expect("revision seven corpus");
    assert_ne!(prior.runs(), []);
    for run in prior.runs() {
        assert_eq!(
            run.deployment().network_id(),
            &common::identifier(corpus.report().network_id())
        );
        assert_eq!(
            run.deployment().genesis_id(),
            &common::identifier(corpus.report().genesis_id())
        );
    }
}

#[test]
fn admitted_content_address_is_stable_and_report_bound() {
    let first = vectors::native_operator_run_of_record().expect("first admission");
    let second = vectors::native_operator_run_of_record().expect("second admission");
    assert_eq!(first.content_address(), second.content_address());
    assert_eq!(
        first.content_address(),
        vectors::NATIVE_OPERATOR_RUN_ADDRESS
    );
    assert_eq!(
        first.report().manifest_digest(),
        vectors::NATIVE_OPERATOR_MANIFEST_SHA256
    );
    assert_eq!(first.report().protocol_revision(), 8);
}
