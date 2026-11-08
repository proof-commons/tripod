//! Offline staging checks and one opt-in native capture. Synthetic signatures
//! exercise carriers only; they are not ECDSA or native acceptance evidence.

use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::common;

use serde_json::{Value, json};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, NativeOperationCapture,
    OperationStep, TargetOperationPlanner, execute_operations_captured,
};
use target_elements_conformance::protocol::{
    FundedOutput, MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse,
    NativeResourceObservation, ObservedOutcomeLayer, OperationSubject, WireOutpoint,
};
use transaction::bytes::{TargetTransaction, Txid, ValueField};
use vectors::keyed_birth::{
    BirthFeeFloor, BirthPolicy, BirthSubjectId, PolicyFactOrigin, Wave1ExecutableProvenance,
    Wave1ExecutorBinding, Wave1TargetPacket,
};
use vectors::keyed_birth_native::{
    BIRTH_EQUALVERIFY_OFFSET, BIRTH_VALUE, BirthCampaignPlanner, BirthLeaf, BirthNativeRefusal,
    BirthObservationClass, FIXTURE_FEE, SPONSOR_VALUE, apply_birth_signature, build_birth_parent,
    build_birth_spend, classify_birth_subject, replay_birth_campaign, replay_birth_capture,
};

const RUN_TEST_NAME: &str = "keyed_birth::keyed_birth_campaign_runs_against_a_real_target";
const EQUALITY_ERROR: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";

fn public_key() -> Vec<u8> {
    let mut key = vec![2];
    key.extend(common::identifier(
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
    ));
    key
}

pub fn coin(pair: u8, index: u32) -> FundedOutput {
    FundedOutput {
        outpoint: WireOutpoint {
            txid: common::hex_bytes(&[pair + 1; 32]),
            vout: index,
        },
        asset: common::hex_bytes(&[5; 32]),
        amount_satoshis: SPONSOR_VALUE,
        script: "0014751e76e8199196d454941c45d1b3a323f1433bd6".into(),
    }
}

fn policy() -> Value {
    json!({
        "relay": "0.00000100", "mempool": "0.00000100", "block": "0.00000100",
        "unit": "BTC/kvB", "block_identifier": "DEFAULT_BLOCK_MIN_TX_FEE",
        "relay_provenance": "reported by the node",
        "block_provenance": "declared by the pinned source, no override",
        "start_arguments": ["-daemon=0", "-listen=0", "-dnsseed=0", "-connect=0"]
    })
}

pub fn response(step: &OperationStep, ordinal: usize) -> NativeOperationResponse {
    let mut response = NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: step.case().clone(),
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: vec![],
        confidential_funded_outputs: vec![],
        mined_readback: None,
        accepted_txid: None,
        sponsor_witness: vec![],
        script_path_witness: vec![],
        signer_public_key: None,
        signed_profile: None,
        signing_genesis: None,
        signature_bound_to: None,
        resources: NativeResourceObservation::default(),
    };
    match step.subject() {
        OperationSubject::SponsorFunding(_) => {
            let pair = u8::try_from(ordinal / 7).expect("two pairs");
            response.funded_outputs = vec![coin(pair, 0), coin(pair, 1)];
        }
        OperationSubject::SponsorSigning(request) => {
            response.signature_bound_to = Some(request.finalized_transaction.clone());
            response.sponsor_witness = vec![vec![0x30, 6, 2, 1, 1, 2, 1, 1, 1], public_key()];
        }
        OperationSubject::Submission(request) => {
            let bytes = &request.transaction_bytes;
            let tx = TargetTransaction::decode(bytes).expect("locally constructed bytes");
            let txid = Txid::from_internal(sha256(&sha256(&tx.encode_without_witness())))
                .to_target_display();
            if ordinal % 7 == 4 {
                response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
                response.observed_detail = Some(
                    json!({
                    "schema": 1, "policy": policy(), "route": "refused",
                    "relay_allowed": false, "relay_detail": EQUALITY_ERROR,
                    "detail": {"txid": txid, "verdict_reason": EQUALITY_ERROR}
                    })
                    .to_string(),
                );
            } else {
                let block = common::hex_bytes(&[9; 32]);
                response.accepted_txid = Some(txid.clone());
                response.mined_readback = Some(MinedFundingReadback {
                    transaction_id: txid.clone(),
                    witness_transaction_id: Txid::from_internal(sha256(&sha256(bytes)))
                        .to_target_display(),
                    block_hash: block.clone(),
                    block_height: 101,
                    raw_transaction: bytes.clone(),
                });
                response.observed_detail = Some(
                    json!({
                    "schema": 1, "policy": policy(), "route": "relay-and-template",
                    "relay_allowed": true, "relay_detail": null,
                    "detail": {"txid": txid, "included_block": block,
                        "produced_block": block, "inclusion_in_produced": true,
                        "load_five_minutes": 0.0, "mempool_size": 0,
                        "mempool_minimum": "0.00000100",
                        "template": {"member": true, "weight": 1000,
                            "txids": [txid], "source": "assembly-read"}}
                    })
                    .to_string(),
                );
            }
        }
        _ => panic!("only sponsor funding, signing and submission are staged"),
    }
    response
}

fn synthetic() -> BirthCampaignPlanner {
    let target = reviewed_elements_tapscript().expect("target");
    let mut planner = BirthCampaignPlanner::new(&target).expect("planner");
    let mut next = planner.next_step(None).expect("funding");
    let mut ordinal = 0;
    while let Some(step) = next {
        let reply = response(&step, ordinal);
        next = planner
            .next_step(Some((step.case(), &reply)))
            .expect("synthetic step");
        ordinal += 1;
    }
    planner.complete().expect("fourteen exchanges");
    planner
}

fn packet() -> Wave1TargetPacket {
    let target = reviewed_elements_tapscript().expect("target");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            [1; 32],
            [2; 32],
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("synthetic binding");
    let floor = BirthFeeFloor {
        satoshis_per_kvb: 100,
        origin: PolicyFactOrigin::Declared,
        source: "synthetic".into(),
    };
    Wave1TargetPacket::bind(
        target,
        binding,
        Wave1ExecutableProvenance {
            node_name: "synthetic".into(),
            node_version: "synthetic".into(),
            binary_revision: "a".repeat(12),
            intended_tip: "a".repeat(40),
            framework_revision: "a".repeat(40),
            executable_path: "/synthetic/elementsd".into(),
            executable_sha256: [3; 32],
            provenance_record: "synthetic".into(),
        },
        Wave1ExecutorBinding {
            chain: "elementsregtest".into(),
            protocol_schema: 8,
            adapter_name: "elements-native-executor".into(),
            adapter_version: "2.1.0".into(),
        },
        BirthPolicy {
            representation: "explicit policy asset".into(),
            fee_policy: "node-defaults".into(),
            startup_arguments: vec!["-daemon=0".into()],
            relay: floor.clone(),
            mempool: floor.clone(),
            block: floor,
        },
    )
    .expect("synthetic packet")
}

#[test]
fn leaf_pins_the_reviewed_birth_site_and_unspendable_commitment() {
    let leaf = BirthLeaf::new(&reviewed_elements_tapscript().expect("target")).expect("leaf");
    assert_eq!(
        leaf.script,
        [0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0x51]
    );
    assert_eq!(leaf.script[BIRTH_EQUALVERIFY_OFFSET], 0x88);
    assert_eq!(leaf.program.len(), 34);
    assert_eq!(leaf.control_block.len(), 33);
    assert_eq!(
        &leaf.control_block[1..],
        target_elements_conformance::constructor::UNSPENDABLE_INTERNAL_KEY
    );
}

#[test]
fn parent_round_trip_preserves_equal_outputs_change_and_explicit_fee() {
    let leaf = BirthLeaf::new(&reviewed_elements_tapscript().expect("target")).expect("leaf");
    let parent = build_birth_parent(&leaf, &coin(0, 0)).expect("parent");
    assert_eq!(
        TargetTransaction::decode(&parent.encode()),
        Ok(parent.clone())
    );
    assert_eq!(parent.outputs()[0], parent.outputs()[1]);
    assert_eq!(
        parent.outputs()[0].value(),
        ValueField::Explicit(BIRTH_VALUE)
    );
    assert_eq!(
        parent.outputs()[2].value(),
        ValueField::Explicit(SPONSOR_VALUE - 2 * BIRTH_VALUE - FIXTURE_FEE)
    );
    assert_eq!(parent.outputs()[3].program(), [0_u8; 0]);
    assert_eq!(
        parent.outputs()[3].value(),
        ValueField::Explicit(FIXTURE_FEE)
    );
}

#[test]
fn four_spends_round_trip_with_one_changed_outpoint_per_pair() {
    let leaf = BirthLeaf::new(&reviewed_elements_tapscript().expect("target")).expect("leaf");
    let parent = build_birth_parent(&leaf, &coin(0, 0)).expect("parent");
    for id in BirthSubjectId::ALL {
        let spend = build_birth_spend(&leaf, &parent, &coin(0, 1), id).expect("spend");
        assert_eq!(
            TargetTransaction::decode(&spend.encode()),
            Ok(spend.clone())
        );
        let current = usize::from(id.positions().1);
        assert_eq!(spend.inputs()[current].outpoint().index(), id.positions().0);
        assert_eq!(spend.inputs()[1 - current].outpoint().index(), 1);
        assert_eq!(
            spend.witnesses()[current].stack(),
            [leaf.script.clone(), leaf.control_block.clone()]
        );
    }
    let planner = synthetic();
    let manifest = planner.manifest(packet()).expect("pair census");
    for pair in manifest.subjects().as_chunks::<2>().0 {
        assert_eq!(pair[0].context, pair[1].context);
        assert_eq!(pair[0].context.funding_outpoint.output_index, 1);
    }
    assert_ne!(
        manifest.subjects()[0].context.parent_id,
        manifest.subjects()[2].context.parent_id
    );
}

#[test]
fn negative_signing_and_submission_precede_each_control() {
    let planner = synthetic();
    assert_eq!(planner.exchanges().len(), 14);
    assert_eq!(
        planner
            .records()
            .iter()
            .map(|r| r.subject.id)
            .collect::<Vec<_>>(),
        BirthSubjectId::SUBMISSION_ORDER
    );
    for row in planner.records() {
        let signing = planner
            .exchanges()
            .iter()
            .position(|s| s == &row.signing)
            .expect("sign");
        let submit = planner
            .exchanges()
            .iter()
            .position(|s| s == &row.submission)
            .expect("submit");
        assert_eq!(signing + 1, submit);
    }
}

#[test]
fn signing_echo_key_hash_profile_and_input_mismatches_refuse() {
    let planner = synthetic();
    let (step, response) = &planner.exchanges()[3];
    let OperationSubject::SponsorSigning(request) = step.subject() else {
        panic!("signing");
    };
    assert!(apply_birth_signature(request, response, &coin(0, 1)).is_ok());
    let mutations: [fn(&mut NativeOperationResponse); 3] = [
        |r| r.signature_bound_to.as_mut().expect("echo").push(0),
        |r| r.sponsor_witness[1][1] ^= 1,
        |r| *r.sponsor_witness[0].last_mut().expect("hash type") = 0x81,
    ];
    for mutate in mutations {
        let mut changed = response.clone();
        mutate(&mut changed);
        assert_eq!(
            apply_birth_signature(request, &changed, &coin(0, 1)),
            Err(BirthNativeRefusal::Signing)
        );
    }
    let mut changed = request.as_ref().clone();
    changed.sponsor_input_index = 0;
    assert_eq!(
        apply_birth_signature(&changed, response, &coin(0, 1)),
        Err(BirthNativeRefusal::Signing)
    );
}

#[test]
fn synthetic_capture_replays_exactly_and_exposes_all_four_records() {
    let planner = synthetic();
    let replay = replay_birth_campaign(
        &reviewed_elements_tapscript().expect("target"),
        planner.exchanges(),
    )
    .expect("replay");
    assert_eq!(replay.records(), planner.records());
    for id in BirthSubjectId::ALL {
        let expected = if id.control().is_some() {
            BirthObservationClass::BornFirstEqualVerify
        } else {
            BirthObservationClass::MinedControl
        };
        assert_eq!(classify_birth_subject(&replay, id), expected);
    }
    for row in replay.records() {
        assert_eq!(row.capture_json()["birth_equality_byte_offset"], 8);
        assert!(row.capture_json()["submission_response"].is_object());
    }
}

#[test]
fn replay_refuses_missing_reordered_changed_and_surplus_requests() {
    let target = reviewed_elements_tapscript().expect("target");
    let planner = synthetic();
    let original = planner.exchanges().to_vec();
    let mut missing = original.clone();
    missing.pop();
    let mut reordered = original.clone();
    reordered.swap(3, 4);
    let mut surplus = original.clone();
    surplus.push(original[0].clone());
    let mut changed = original;
    let mut subject = changed[4].0.subject().clone();
    let OperationSubject::Submission(request) = &mut subject else {
        panic!("submit");
    };
    request.transaction_bytes.push(0);
    changed[4].0 = OperationStep::new(&changed[4].0.case().step, subject);
    for exchanges in [missing, reordered, surplus, changed] {
        assert!(replay_birth_campaign(&target, &exchanges).is_err());
    }
}

#[test]
fn ambiguous_error_and_earlier_layer_leave_the_subject_open() {
    let target = reviewed_elements_tapscript().expect("target");
    for reason in [
        "script failed",
        "OP_EQUALVERIFY or OP_CHECKSIG",
        "sponsor OP_EQUALVERIFY",
    ] {
        let mut exchanges = synthetic().exchanges().to_vec();
        let mut detail: Value =
            serde_json::from_str(exchanges[4].1.observed_detail.as_deref().expect("detail"))
                .expect("json");
        detail["relay_detail"] = json!(reason);
        exchanges[4].1.observed_detail = Some(detail.to_string());
        let replay =
            replay_birth_campaign(&target, &exchanges).expect("retain ambiguous observation");
        assert_eq!(
            classify_birth_subject(&replay, BirthSubjectId::A1),
            BirthObservationClass::Open
        );
    }
    let mut exchanges = synthetic().exchanges().to_vec();
    exchanges[4].1.observed_layer = ObservedOutcomeLayer::RelayPolicyRejection;
    let replay = replay_birth_campaign(&target, &exchanges).expect("retain earlier refusal");
    assert_eq!(
        classify_birth_subject(&replay, BirthSubjectId::A1),
        BirthObservationClass::Open
    );
}

#[test]
fn sponsor_equality_site_error_cannot_be_attributed_to_birth() {
    let target = reviewed_elements_tapscript().expect("target");
    let mut exchanges = synthetic().exchanges().to_vec();
    exchanges[3].1.sponsor_witness[1][1] ^= 1;
    assert!(matches!(
        replay_birth_campaign(&target, &exchanges),
        Err(BirthNativeRefusal::Signing)
    ));
}

#[test]
fn wrong_parent_readback_and_reused_pair_funding_refuse() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = synthetic().exchanges().to_vec();
    let mut changed = original.clone();
    changed[2]
        .1
        .mined_readback
        .as_mut()
        .expect("readback")
        .raw_transaction
        .push(0);
    assert!(matches!(
        replay_birth_campaign(&target, &changed),
        Err(BirthNativeRefusal::Readback)
    ));
    let mut changed = original.clone();
    changed[7]
        .1
        .funded_outputs
        .clone_from(&original[0].1.funded_outputs);
    assert!(matches!(
        replay_birth_campaign(&target, &changed),
        Err(BirthNativeRefusal::Funding)
    ));
}

pub fn required(name: &str) -> String {
    common::environment(name).unwrap_or_else(|| panic!("{name} is required"))
}

pub fn executable_provenance() -> Value {
    let binary = required("ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD");
    let path = Path::new(&binary)
        .parent()
        .and_then(Path::parent)
        .expect("toolchain bin/elementsd layout")
        .join("PROVENANCE.txt");
    json!({
        "executable_path": binary,
        "executable_sha256": common::hex_bytes(&sha256(&std::fs::read(&binary).expect("binary"))),
        "provenance_path": path,
        "provenance_record": std::fs::read_to_string(&path).expect("toolchain provenance"),
        "intended_tip": required("ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP"),
        "representation": "explicit policy asset",
        "fee_policy": required("ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY")
    })
}

pub fn write_run_files(directory: &Path, short: &str, report: &str, elapsed: f64, eligible: bool) {
    std::fs::write(directory.join("RUN-REPORT"), report).expect("run report");
    let timing = format!(
        "timing-schema 1\nceremony-id report\nstatus {}\nwall-seconds {elapsed:.3}\n",
        if eligible {
            "passed"
        } else {
            "failed-after-transcript"
        }
    );
    std::fs::write(directory.join(format!("{short}.report.timing")), timing).expect("timing");
    let mut manifest = String::new();
    for name in [
        format!("{short}.report.capture"),
        format!("{short}.report.timing"),
        "RUN-REPORT".into(),
    ] {
        let bytes = std::fs::read(directory.join(&name)).expect("capture artifact");
        writeln!(manifest, "{}  {name}", common::hex_bytes(&sha256(&bytes))).expect("manifest");
    }
    std::fs::write(directory.join("MANIFEST.sha256"), manifest).expect("manifest file");
}

#[test]
#[ignore = "requires the pinned native image, adapter and fresh capture environment"]
fn keyed_birth_campaign_runs_against_a_real_target() {
    let started = Instant::now();
    let directory = required("TRIPOD_LIVE_REPORT_DIR");
    let directory = Path::new(&directory);
    let short = required("TRIPOD_LIVE_SUITE_SHORT_SHA");
    let mut destination = common::CaptureDestination::new(
        directory.into(),
        short.clone(),
        required("TRIPOD_LIVE_SUITE_COMMIT"),
        required("TRIPOD_LIVE_SUITE_TREE"),
    )
    .expect("capture provenance");
    destination.report_test_name = RUN_TEST_NAME;
    std::fs::create_dir_all(directory).expect("report directory");
    for name in [
        format!("{short}.report.capture"),
        format!("{short}.report.timing"),
        "RUN-REPORT".into(),
        "MANIFEST.sha256".into(),
    ] {
        assert!(!directory.join(name).exists(), "fresh report destination");
    }
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    let target = reviewed_elements_tapscript().expect("target");
    let network = required("TRIPOD_LIVE_NETWORK_ID");
    let genesis = required("TRIPOD_LIVE_GENESIS_ID");
    common::require_lower_hex(&network, 64, "network").expect("network");
    common::require_lower_hex(&genesis, 64, "genesis").expect("genesis");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            common::identifier(&network),
            common::identifier(&genesis),
            ActivationDeclaration::new(true, target.definition().leaf_version(), []),
            None,
        ),
    )
    .expect("binding");
    let provenance = executable_provenance();
    let config = ExecutorConfiguration::new(
        Path::new(&required("TRIPOD_LIVE_EXECUTOR")),
        ExecutorTrust::ReviewedNonMock,
        Duration::from_secs(900),
        ExecutorDiagnostics::in_directory(&directory.join("diagnostics")),
    );
    let mut planner = BirthCampaignPlanner::new(&target).expect("construction");
    let mut capture = NativeOperationCapture::default();
    let outcome = common::trace_ceremony("keyed_birth", "birth", &mut capture, |capture| {
        execute_operations_captured(&target, &binding, &config, &mut planner, capture)
    });
    let replay = replay_birth_capture(&target, &capture);
    let eligible = outcome.is_ok()
        && replay.as_ref().is_ok_and(|p| {
            BirthSubjectId::ALL
                .into_iter()
                .all(|id| classify_birth_subject(p, id) != BirthObservationClass::Open)
        });
    let rows: Vec<_> = planner
        .records()
        .iter()
        .map(|row| {
            json!({
                "record": row.capture_json(),
                "class": format!("{:?}", classify_birth_subject(&planner, row.subject.id))
            })
        })
        .collect();
    let payload = json!({"keyed_birth_capture_schema": 1, "rows": rows,
        "eligible_for_review": eligible, "admitted": false, "executable_provenance": provenance,
        "construction_refusal": format!("{:?}", planner.refusal()),
        "replay_refusal": format!("{:?}", replay.as_ref().err())})
    .to_string();
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    drop(guard);
    let report = format!(
        "keyed-birth-run-schema 1\nwall-seconds {:.3}\neligible-for-review {eligible}\nadmitted false\nexecutor-result {outcome:?}\nreplay-result {:?}\n",
        started.elapsed().as_secs_f64(),
        replay.as_ref().err()
    );
    write_run_files(
        directory,
        &short,
        &report,
        started.elapsed().as_secs_f64(),
        eligible,
    );
    assert!(eligible, "capture retained but ineligible; see RUN-REPORT");
}
