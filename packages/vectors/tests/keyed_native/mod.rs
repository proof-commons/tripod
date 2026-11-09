//! Offline carriers and replay; synthetic replies do not prove native outcomes.

use crate::keyed_birth::{coin, response};
use serde_json::{Value, json};
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::executor::{
    NativeOperationCapture, OperationStep, TargetOperationPlanner,
};
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, ObservedOutcomeLayer, OperationSubject,
};
use transaction::bytes::{TargetTransaction, ValueField};
use vectors::keyed_birth_native::{BIRTH_VALUE, BirthNativeRefusal, FIXTURE_FEE, SPONSOR_VALUE};
use vectors::keyed_native::*;

// Deployment handles belong to this test campaign, never to the constructor.
const KEYS: [[u8; 32]; 2] = [[0x31; 32], [0x32; 32]];
const RUN_TEST_NAME: &str = "keyed_native::keyed_campaign_runs_against_a_real_target";

fn synthetic() -> KeyedCampaignPlanner {
    let target = reviewed_elements_tapscript().expect("target");
    let mut planner = KeyedCampaignPlanner::new(&target, KEYS).expect("planner");
    let mut next = planner.next_step(None).expect("funding");
    let mut n = 0;
    while let Some(step) = next {
        let mut reply = response(&step, 0);
        if n == 0 {
            reply.funded_outputs = (0..7).map(|i| coin(0, i)).collect();
        }
        if let Some(mined) = &mut reply.mined_readback {
            mined.block_height = 100 + u32::try_from(n).expect("exchange count");
        }
        if n >= 10 && n % 2 == 0 {
            let id = KeyedSubjectId::SUBMISSION_ORDER[(n - 10) / 2];
            let error = match id {
                KeyedSubjectId::WrongControlParity | KeyedSubjectId::UncommittedLeaf => {
                    Some("Witness program hash mismatch")
                }
                KeyedSubjectId::KeyPathAttempt => Some("Invalid Schnorr signature"),
                KeyedSubjectId::WrongKey | KeyedSubjectId::MixedKeys => {
                    Some("Script failed an OP_EQUALVERIFY operation")
                }
                _ => None,
            };
            if let Some(error) = error {
                let reason = format!("mandatory-script-verify-flag-failed ({error})");
                let mut observation: Value =
                    serde_json::from_str(reply.observed_detail.as_deref().expect("detail"))
                        .expect("json");
                observation["route"] = json!("refused");
                observation["relay_allowed"] = json!(false);
                observation["relay_detail"] = json!(reason);
                observation["detail"] =
                    json!({"txid": reply.accepted_txid, "verdict_reason": reason});
                reply.observed_detail = Some(observation.to_string());
                reply.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
                reply.accepted_txid = None;
                reply.mined_readback = None;
            }
        }
        next = planner
            .next_step(Some((step.case(), &reply)))
            .expect("synthetic response");
        n += 1;
    }
    planner.complete().expect("complete");
    planner
}

fn journal(planner: &KeyedCampaignPlanner) -> Value {
    let mut journal = keyed_capture_json(&NativeOperationCapture::default(), planner.recipes());
    journal["terminal"] = json!("complete");
    journal["operations"] = json!(planner.exchanges().iter().enumerate().map(|(n, (step, response))| json!({
        "operation_id": format!("operation-{n}"), "request_id": format!("request-{n}"),
        "response_id": format!("response-{n}"), "response_request_id": format!("request-{n}"),
        "response_operation_id": format!("operation-{n}"),
        "request": NativeOperationRequest { schema: NATIVE_PROTOCOL_SCHEMA, case: step.case().clone(), subject: step.subject().clone() }, "response": response
    })).collect::<Vec<_>>());
    journal
}
fn classes(planner: &KeyedCampaignPlanner) -> Vec<KeyedObservationClass> {
    KeyedSubjectId::ALL
        .into_iter()
        .map(|id| classify_keyed_subject(planner, id))
        .collect()
}
fn submitted(planner: &KeyedCampaignPlanner, id: KeyedSubjectId) -> TargetTransaction {
    let row = planner.records().iter().find(|r| r.id == id).expect("row");
    let OperationSubject::Submission(request) = row.submission.0.subject() else {
        panic!("submission");
    };
    TargetTransaction::decode(&request.transaction_bytes).expect("decode")
}

#[test]
fn reconstructed_recipes_and_every_parent_bind_birth_zero() {
    let planner = synthetic();
    let descriptors = planner.recipes().descriptors();
    assert_ne!(descriptors[0].output_program, descriptors[1].output_program);
    for descriptor in descriptors {
        for offset in KEYED_EQUALVERIFY_OFFSETS {
            assert_eq!(descriptor.leaf_bytes[offset], 0x88);
        }
    }
    for (p, n) in [2, 4, 6, 8].into_iter().enumerate() {
        let readback = planner.exchanges()[n]
            .1
            .mined_readback
            .as_ref()
            .expect("parent");
        let tx = TargetTransaction::decode(&readback.raw_transaction).expect("parent bytes");
        assert_eq!(tx.encode(), readback.raw_transaction);
        assert_eq!(
            tx.outputs()[0].program(),
            descriptors[usize::from(p == 3)].output_program
        );
        assert_eq!(tx.outputs()[0], tx.outputs()[1]);
        assert_eq!(
            tx.outputs()[2].value(),
            ValueField::Explicit(SPONSOR_VALUE - 2 * BIRTH_VALUE - FIXTURE_FEE)
        );
    }
    assert!(
        KeyedRecipes::new(
            &reviewed_elements_tapscript().expect("target"),
            [KEYS[0]; 2]
        )
        .is_err()
    );
}

#[test]
fn single_coin_rows_change_only_the_declared_coordinate() {
    let planner = synthetic();
    let control = submitted(&planner, KeyedSubjectId::ControlK);
    for id in [
        KeyedSubjectId::WrongControlParity,
        KeyedSubjectId::UncommittedLeaf,
        KeyedSubjectId::KeyPathAttempt,
        KeyedSubjectId::WrongKey,
    ] {
        let tx = submitted(&planner, id);
        assert_eq!(tx.inputs(), control.inputs());
        assert_eq!(tx.inputs()[0].outpoint().index(), 0);
        assert_eq!(tx.inputs()[1].outpoint().index(), 4);
        if id == KeyedSubjectId::WrongKey {
            assert_eq!(
                tx.outputs()[0].program(),
                planner.recipes().descriptors()[1].output_program
            );
            assert_eq!(tx.outputs()[0].value(), control.outputs()[0].value());
            assert_eq!(tx.outputs()[0].asset(), control.outputs()[0].asset());
            assert_eq!(&tx.outputs()[1..], &control.outputs()[1..]);
            assert_eq!(tx.witnesses()[0], control.witnesses()[0]);
        } else {
            assert_eq!(
                tx.encode_without_witness(),
                control.encode_without_witness()
            );
            assert_eq!(tx.witnesses()[1], control.witnesses()[1]);
            let mut expected = control.witnesses()[0].stack().to_vec();
            match id {
                KeyedSubjectId::WrongControlParity => expected[1][0] ^= 1,
                KeyedSubjectId::UncommittedLeaf => expected[0][1] ^= 1,
                KeyedSubjectId::KeyPathAttempt => expected = vec![vec![0; 64]],
                _ => unreachable!(),
            }
            assert_eq!(tx.witnesses()[0].stack(), expected);
        }
        assert_eq!(TargetTransaction::decode(&tx.encode()), Ok(tx));
    }
}

#[test]
fn mixed_pair_has_fixed_first_coin_reserve_values_and_successor() {
    let planner = synthetic();
    let negative = submitted(&planner, KeyedSubjectId::MixedKeys);
    let control = submitted(&planner, KeyedSubjectId::SameKeyControl);
    assert_eq!(negative.inputs().len(), 3);
    assert_eq!(negative.inputs()[0], control.inputs()[0]);
    assert_eq!(negative.inputs()[2], control.inputs()[2]);
    assert_ne!(
        negative.inputs()[1].outpoint(),
        control.inputs()[1].outpoint()
    );
    assert_eq!(negative.inputs()[0].outpoint().index(), 0);
    assert_eq!(negative.inputs()[1].outpoint().index(), 0);
    assert_eq!(negative.inputs()[2].outpoint().index(), 5);
    assert_eq!(negative.outputs(), control.outputs());
    assert_eq!(
        negative.outputs()[0].value(),
        ValueField::Explicit(2 * BIRTH_VALUE)
    );
    assert_eq!(negative.witnesses()[0], control.witnesses()[0]);
    assert_eq!(
        negative.witnesses()[1].stack()[0],
        planner.recipes().descriptors()[1].leaf_bytes
    );
    assert_eq!(
        control.witnesses()[1].stack()[0],
        planner.recipes().descriptors()[0].leaf_bytes
    );
    for tx in [negative, control] {
        assert_eq!(TargetTransaction::decode(&tx.encode()), Ok(tx));
    }
}

#[test]
fn census_orders_all_negatives_before_their_controls_and_coexistence_is_separate() {
    let planner = synthetic();
    assert_eq!(planner.exchanges().len(), 25);
    assert_eq!(
        planner.records().iter().map(|r| r.id).collect::<Vec<_>>(),
        KeyedSubjectId::SUBMISSION_ORDER
    );
    for (i, row) in planner.records().iter().enumerate() {
        assert_eq!(row.signing, planner.exchanges()[9 + 2 * i]);
        assert_eq!(row.submission, planner.exchanges()[10 + 2 * i]);
        if let Some(control) = row.id.control() {
            assert!(
                planner
                    .records()
                    .iter()
                    .position(|r| r.id == control)
                    .expect("control")
                    > i
            );
        }
        assert_ne!(row.id.coordinate(), "");
        assert_ne!(row.id.collateral(), "");
        assert_ne!(row.id.boundary(), "");
    }
    let k = submitted(&planner, KeyedSubjectId::ControlK);
    let l = submitted(&planner, KeyedSubjectId::CoexistenceL);
    assert_ne!(k.inputs()[0].outpoint(), l.inputs()[0].outpoint());
    assert_eq!(l.inputs()[1].outpoint().index(), 6);
    assert_eq!(
        l.outputs()[0].program(),
        planner.recipes().descriptors()[1].output_program
    );
}

#[test]
fn complete_wire_json_alone_replays_all_keyed_classes() {
    use KeyedObservationClass::*;
    let target = reviewed_elements_tapscript().expect("target");
    let planner = synthetic();
    let expected = planner.records().to_vec();
    let bytes = serde_json::to_vec(&journal(&planner)).expect("wire");
    drop(planner);
    let replay = replay_keyed_capture(&target, &serde_json::from_slice(&bytes).expect("json only"))
        .expect("replay");
    assert_eq!(replay.records(), expected);
    assert_eq!(
        classes(&replay),
        [
            MinedControl,
            CommitmentRefusal,
            CommitmentRefusal,
            KeyPathSignatureRefusal,
            ProgramEqualityRefusal,
            MixedKeyEqualityRefusal,
            MinedControl,
            MinedControl
        ]
    );
    for row in replay.records() {
        assert!(row.capture_json()["submission_response"].is_object());
    }
}

#[test]
fn malformed_links_missing_responses_and_changed_descriptors_refuse() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = journal(&synthetic());
    let mutations: [fn(&mut Value); 6] = [
        |j| {
            j["operations"][9]["response"]
                .as_object_mut()
                .expect("response")
                .remove("sponsor_witness");
        },
        |j| j["operations"][10]["response_request_id"] = json!("other"),
        |j| j["operations"][10]["request"]["case"]["step"] = json!("other"),
        |j| {
            j["descriptors"][0]["output_program"][2] = json!(
                j["descriptors"][0]["output_program"][2]
                    .as_u64()
                    .expect("byte")
                    ^ 1
            );
        },
        |j| {
            j["operations"].as_array_mut().expect("operations").pop();
        },
        |j| j["terminal"] = json!("timeout"),
    ];
    for mutate in mutations {
        let mut changed = original.clone();
        mutate(&mut changed);
        assert!(replay_keyed_capture(&target, &changed).is_err());
    }
}

#[test]
fn sponsor_echo_preimage_and_sighash_mutants_refuse_replay() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = synthetic();
    for index in [9, 19] {
        let mutations: [fn(&mut target_elements_conformance::protocol::NativeOperationResponse);
            3] = [
            |r| r.signature_bound_to.as_mut().expect("echo").push(0),
            |r| r.sponsor_witness[1][1] ^= 1,
            |r| *r.sponsor_witness[0].last_mut().expect("sighash") = 0x81,
        ];
        for mutate in mutations {
            let mut exchanges = original.exchanges().to_vec();
            mutate(&mut exchanges[index].1);
            assert!(matches!(
                replay_keyed_campaign(&target, KEYS, &exchanges),
                Err(BirthNativeRefusal::Signing)
            ));
        }
    }
}

#[test]
fn ambiguous_wrong_site_and_earlier_layer_refusals_stay_open() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = synthetic();
    for (index, id) in [
        (10, KeyedSubjectId::WrongControlParity),
        (12, KeyedSubjectId::UncommittedLeaf),
        (14, KeyedSubjectId::KeyPathAttempt),
        (16, KeyedSubjectId::WrongKey),
        (20, KeyedSubjectId::MixedKeys),
    ] {
        for reason in [
            "script failed",
            "sponsor OP_EQUALVERIFY",
            "mandatory-script-verify-flag-failed (Invalid Schnorr signature size)",
        ] {
            let mut exchanges = original.exchanges().to_vec();
            let response = &mut exchanges[index].1;
            let mut observation: Value =
                serde_json::from_str(response.observed_detail.as_deref().expect("detail"))
                    .expect("json");
            observation["relay_detail"] = json!(reason);
            observation["detail"]["verdict_reason"] = json!(reason);
            response.observed_detail = Some(observation.to_string());
            let replay =
                replay_keyed_campaign(&target, KEYS, &exchanges).expect("retain unknown error");
            assert_eq!(
                classify_keyed_subject(&replay, id),
                KeyedObservationClass::Open
            );
        }
        let mut exchanges = original.exchanges().to_vec();
        exchanges[index].1.observed_layer = ObservedOutcomeLayer::RelayPolicyRejection;
        let replay =
            replay_keyed_campaign(&target, KEYS, &exchanges).expect("retain earlier layer");
        assert_eq!(
            classify_keyed_subject(&replay, id),
            KeyedObservationClass::Open
        );
    }
}

#[test]
fn missing_exact_control_readback_opens_its_dependent_rows() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = synthetic();
    for (index, dependents) in [
        (
            18,
            vec![
                KeyedSubjectId::ControlK,
                KeyedSubjectId::WrongControlParity,
                KeyedSubjectId::UncommittedLeaf,
                KeyedSubjectId::KeyPathAttempt,
                KeyedSubjectId::WrongKey,
                KeyedSubjectId::CoexistenceL,
            ],
        ),
        (
            22,
            vec![KeyedSubjectId::SameKeyControl, KeyedSubjectId::MixedKeys],
        ),
    ] {
        let mut exchanges = original.exchanges().to_vec();
        exchanges[index]
            .1
            .mined_readback
            .as_mut()
            .expect("readback")
            .raw_transaction
            .push(0);
        let replay = replay_keyed_campaign(&target, KEYS, &exchanges).expect("retain bad control");
        for id in dependents {
            assert_eq!(
                classify_keyed_subject(&replay, id),
                KeyedObservationClass::Open
            );
        }
    }
    let mut exchanges = original.exchanges().to_vec();
    exchanges[8]
        .1
        .mined_readback
        .as_mut()
        .expect("parent")
        .raw_transaction
        .push(0);
    assert!(matches!(
        replay_keyed_campaign(&target, KEYS, &exchanges),
        Err(BirthNativeRefusal::Readback)
    ));
}

#[test]
fn changed_mixed_request_and_reordered_negatives_refuse() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = synthetic();
    let mut exchanges = original.exchanges().to_vec();
    exchanges.swap(9, 11);
    assert!(replay_keyed_campaign(&target, KEYS, &exchanges).is_err());
    let mut exchanges = original.exchanges().to_vec();
    let mut subject = exchanges[19].0.subject().clone();
    let OperationSubject::SponsorSigning(request) = &mut subject else {
        panic!("signing");
    };
    request.finalized_transaction.push(0);
    exchanges[19].0 = OperationStep::new(&exchanges[19].0.case().step, subject);
    assert!(replay_keyed_campaign(&target, KEYS, &exchanges).is_err());
}

use crate::{
    common,
    keyed_birth::{executable_provenance, required, write_run_files},
};
use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding,
    validate_reviewed_development_binding,
};
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, execute_operations_captured,
};

#[test]
#[ignore = "requires the pinned native image, adapter and fresh capture environment"]
fn keyed_campaign_runs_against_a_real_target() {
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
        format!("{short}.report.capture.timing"),
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
    let mut planner = KeyedCampaignPlanner::new(&target, KEYS).expect("construction");
    let mut capture = NativeOperationCapture::default();
    let outcome = common::trace_ceremony("keyed_native", "keyed", &mut capture, |capture| {
        execute_operations_captured(&target, &binding, &config, &mut planner, capture)
    });
    let wire = keyed_capture_json(&capture, planner.recipes());
    // Round-trip the exact wire form that admission will read, not Debug text.
    let wire: Value = serde_json::from_slice(&serde_json::to_vec(&wire).expect("wire encoding"))
        .expect("wire decoding");
    let replay = replay_keyed_capture(&target, &wire);
    let eligible = outcome.is_ok()
        && replay
            .as_ref()
            .is_ok_and(|p| classes(p).iter().all(|c| *c != KeyedObservationClass::Open));
    let rows: Vec<_> = planner.records().iter().map(|row| json!({"record": row.capture_json(), "class": format!("{:?}", classify_keyed_subject(&planner, row.id))})).collect();
    let payload = json!({"keyed_native_capture_schema": 1, "wire_journal": wire, "rows": rows, "eligible_for_review": eligible, "admitted": false, "executable_provenance": provenance, "construction_refusal": format!("{:?}", planner.refusal()), "replay_refusal": format!("{:?}", replay.as_ref().err())}).to_string();
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    drop(guard);
    let report = format!(
        "keyed-run-schema 1\nwall-seconds {:.3}\neligible-for-review {eligible}\nadmitted false\nexecutor-completed {}\nreplay-refusal {:?}\nclasses {:?}\n",
        started.elapsed().as_secs_f64(),
        outcome.is_ok(),
        replay.as_ref().err(),
        classes(&planner)
    );
    write_run_files(
        directory,
        &short,
        &report,
        started.elapsed().as_secs_f64(),
        eligible,
    );
    let timing_name = format!("{short}.report.capture.timing");
    let mut manifest =
        std::fs::read_to_string(directory.join("MANIFEST.sha256")).expect("manifest");
    writeln!(
        manifest,
        "{}  {timing_name}",
        common::hex_bytes(&sha256(
            &std::fs::read(directory.join(&timing_name)).expect("capture timing")
        ))
    )
    .expect("manifest line");
    std::fs::write(directory.join("MANIFEST.sha256"), manifest).expect("complete manifest");
    assert!(
        eligible,
        "capture retained but ineligible; see wire journal and RUN-REPORT"
    );
}
