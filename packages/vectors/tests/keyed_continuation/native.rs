//! Offline native-driver checks and one ignored capture; synthetic replies do
//! not establish ECDSA validity, native acceptance or constructor key separation.

use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::constructor::{
    UNSPENDABLE_INTERNAL_KEY,
    tagged::sha256,
    tree::{FixtureTapTree, construct},
};
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, NativeOperationCapture,
    OperationStep, TargetOperationPlanner, execute_operations_captured,
};
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, NativeOperationResponse, ObservedOutcomeLayer,
    OperationSubject,
};
use transaction::bytes::{TargetTransaction, ValueField};
use vectors::keyed_birth_native::{
    BIRTH_VALUE, BirthNativeRefusal, FIXTURE_FEE, SPONSOR_VALUE, apply_birth_signature,
};
use vectors::keyed_continuation::ContinuationSubjectId;
use vectors::keyed_continuation_native::{
    CONTINUATION_EQUALVERIFY_OFFSETS, ContinuationCampaignPlanner, ContinuationLeaves,
    ContinuationObservationClass, build_continuation_parent, build_continuation_spend,
    classify_continuation_subject, continuation_capture_json, replay_continuation_campaign,
    replay_continuation_capture,
};

use crate::{
    common,
    keyed_birth::{coin, executable_provenance, required, response, write_run_files},
};

const EQUALITY_ERROR: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";
const RUN_TEST_NAME: &str =
    "keyed_continuation::native::keyed_continuation_campaign_runs_against_a_real_target";

fn synthetic() -> ContinuationCampaignPlanner {
    let target = reviewed_elements_tapscript().expect("target");
    let mut planner = ContinuationCampaignPlanner::new(&target).expect("planner");
    let mut next = planner.next_step(None).expect("funding");
    let mut ordinal = 0;
    while let Some(step) = next {
        let mut reply = response(&step, 0);
        if [4, 6, 8].contains(&ordinal) {
            let mut observation: Value =
                serde_json::from_str(reply.observed_detail.as_deref().expect("detail"))
                    .expect("json");
            observation["route"] = json!("refused");
            observation["relay_allowed"] = json!(false);
            observation["relay_detail"] = json!(EQUALITY_ERROR);
            observation["detail"] =
                json!({"txid": reply.accepted_txid, "verdict_reason": EQUALITY_ERROR});
            reply.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
            reply.observed_detail = Some(observation.to_string());
            reply.accepted_txid = None;
            reply.mined_readback = None;
        }
        next = planner
            .next_step(Some((step.case(), &reply)))
            .expect("synthetic response");
        ordinal += 1;
    }
    planner.complete().expect("eleven exchanges");
    planner
}

fn journal(planner: &ContinuationCampaignPlanner) -> Value {
    let operations: Vec<_> = planner.exchanges().iter().enumerate().map(|(n, (step, response))| json!({
        "operation_id": format!("operation-{n}"), "request_id": format!("request-{n}"),
        "response_id": format!("response-{n}"), "response_request_id": format!("request-{n}"),
        "response_operation_id": format!("operation-{n}"),
        "request": NativeOperationRequest { schema: NATIVE_PROTOCOL_SCHEMA, case: step.case().clone(), subject: step.subject().clone() },
        "response": response,
    })).collect();
    json!({"continuation_wire_schema": 1, "terminal": "complete", "handshake": null, "environment": null, "operations": operations})
}

fn classes(planner: &ContinuationCampaignPlanner) -> Vec<ContinuationObservationClass> {
    ContinuationSubjectId::ALL
        .into_iter()
        .map(|id| classify_continuation_subject(planner, id))
        .collect()
}

#[test]
fn both_leaf_recipes_reconstruct_under_the_fixed_internal_key() {
    let target = reviewed_elements_tapscript().expect("target");
    let leaves = ContinuationLeaves::new(&target).expect("leaves");
    assert_eq!(
        leaves.primary.script,
        [
            0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0, 0xd1, 0x7b, 0x88, 0x88,
            0x51
        ]
    );
    let mut alternative = leaves.primary.script[..17].to_vec();
    alternative.extend([0, 0x75, 0x51]);
    assert_eq!(leaves.alternative.script, alternative);
    for leaf in [&leaves.primary, &leaves.alternative] {
        let tree = FixtureTapTree::Leaf {
            version: target.definition().leaf_version().get(),
            script: leaf.script.clone(),
        };
        let rebuilt =
            construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &tree).expect("independent reconstruction");
        assert_eq!(leaf.program, rebuilt.output_program());
        assert_eq!(leaf.control_block, rebuilt.control_block());
        assert_eq!(&leaf.control_block[1..], UNSPENDABLE_INTERNAL_KEY);
        for offset in CONTINUATION_EQUALVERIFY_OFFSETS {
            assert_eq!(leaf.script[offset], 0x88);
        }
    }
    assert_ne!(leaves.primary.program, leaves.alternative.program);
}

#[test]
fn continuation_parent_round_trip_balances_both_programs_change_and_fee() {
    let leaves =
        ContinuationLeaves::new(&reviewed_elements_tapscript().expect("target")).expect("leaves");
    let parent = build_continuation_parent(&leaves, &coin(0, 0)).expect("parent");
    assert_eq!(
        TargetTransaction::decode(&parent.encode()),
        Ok(parent.clone())
    );
    assert_eq!(parent.outputs()[0].program(), leaves.primary.program);
    assert_eq!(parent.outputs()[1].program(), leaves.alternative.program);
    assert_eq!(
        parent.outputs()[0].value(),
        ValueField::Explicit(BIRTH_VALUE)
    );
    assert_eq!(
        parent.outputs()[1].value(),
        ValueField::Explicit(BIRTH_VALUE)
    );
    assert_eq!(
        parent.outputs()[2].value(),
        ValueField::Explicit(SPONSOR_VALUE - 2 * BIRTH_VALUE - FIXTURE_FEE)
    );
    assert_eq!(
        parent.outputs()[3].value(),
        ValueField::Explicit(FIXTURE_FEE)
    );
    assert_eq!(parent.outputs()[3].program(), [0_u8; 0]);
}

#[test]
fn all_spends_decode_with_fixed_birth_funding_and_one_successor_mutation() {
    let target = reviewed_elements_tapscript().expect("target");
    let leaves = ContinuationLeaves::new(&target).expect("leaves");
    let parent = build_continuation_parent(&leaves, &coin(0, 0)).expect("parent");
    let control = build_continuation_spend(
        &leaves,
        &parent,
        &coin(0, 1),
        ContinuationSubjectId::Control,
    )
    .expect("control");
    for id in ContinuationSubjectId::ALL {
        let tx = build_continuation_spend(&leaves, &parent, &coin(0, 1), id).expect("spend");
        assert_eq!(TargetTransaction::decode(&tx.encode()), Ok(tx.clone()));
        assert_eq!(tx.inputs(), control.inputs());
        assert_eq!(tx.witnesses(), control.witnesses());
        assert_eq!(tx.inputs()[0].outpoint().index(), 0);
        assert_eq!(tx.inputs()[1].outpoint().index(), 1);
        assert_eq!(tx.outputs()[2], control.outputs()[2]);
        if id == ContinuationSubjectId::DisplacedSuccessor {
            assert_eq!(tx.outputs()[0], control.outputs()[1]);
            assert_eq!(tx.outputs()[1], control.outputs()[0]);
        } else {
            assert_eq!(tx.outputs()[0].value(), control.outputs()[0].value());
            assert_eq!(tx.outputs()[0].asset(), control.outputs()[0].asset());
            assert_eq!(tx.outputs()[1], control.outputs()[1]);
        }
        if id == ContinuationSubjectId::WrongVersion {
            assert_eq!(tx.outputs()[0].program()[0], 0);
            assert_eq!(
                &tx.outputs()[0].program()[1..],
                &control.outputs()[0].program()[1..]
            );
        }
    }
    synthetic()
        .manifest(super::packet())
        .expect("canonical manifest");
}

#[test]
fn all_three_negatives_are_signed_and_submitted_before_the_control() {
    let planner = synthetic();
    assert_eq!(planner.exchanges().len(), 11);
    assert_eq!(
        planner
            .records()
            .iter()
            .map(|r| r.subject.id)
            .collect::<Vec<_>>(),
        ContinuationSubjectId::SUBMISSION_ORDER
    );
    for (n, row) in planner.records().iter().enumerate() {
        assert_eq!(row.signing, planner.exchanges()[3 + 2 * n]);
        assert_eq!(row.submission, planner.exchanges()[4 + 2 * n]);
        assert_eq!(row.subject.context, planner.records()[0].subject.context);
    }
}

#[test]
fn continuation_reuses_signing_echo_and_sponsor_equality_checks() {
    let target = reviewed_elements_tapscript().expect("target");
    let planner = synthetic();
    let (step, response) = &planner.exchanges()[3];
    let OperationSubject::SponsorSigning(request) = step.subject() else {
        panic!("signing");
    };
    assert!(apply_birth_signature(request, response, &coin(0, 1)).is_ok());
    let mutations: [fn(&mut NativeOperationResponse); 3] = [
        |r| r.signature_bound_to.as_mut().expect("echo").push(0),
        |r| r.sponsor_witness[1][1] ^= 1,
        |r| *r.sponsor_witness[0].last_mut().expect("sighash") = 0x81,
    ];
    for mutate in mutations {
        let mut exchanges = planner.exchanges().to_vec();
        mutate(&mut exchanges[3].1);
        assert!(matches!(
            replay_continuation_campaign(&target, &exchanges),
            Err(BirthNativeRefusal::Signing)
        ));
    }
}

#[test]
fn complete_wire_json_alone_replays_all_responses_and_classifications() {
    let target = reviewed_elements_tapscript().expect("target");
    let planner = synthetic();
    let expected = planner.records().to_vec();
    let wire = serde_json::to_vec(&journal(&planner)).expect("wire JSON");
    drop(planner);
    let decoded: Value = serde_json::from_slice(&wire).expect("only JSON retained");
    let replay = replay_continuation_capture(&target, &decoded).expect("JSON-only replay");
    assert_eq!(replay.records(), expected);
    assert_eq!(
        classes(&replay),
        [
            ContinuationObservationClass::MinedControl,
            ContinuationObservationClass::ProgramEqualityRefusal,
            ContinuationObservationClass::VersionEqualityRefusal,
            ContinuationObservationClass::ProgramEqualityRefusal
        ]
    );
    for row in replay.records() {
        assert!(row.capture_json()["submission_response"].is_object());
    }
}

#[test]
fn journal_missing_response_field_link_and_changed_request_refuse() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = journal(&synthetic());
    let mutations: [fn(&mut Value); 6] = [
        |j| j["operations"][3]["response"] = Value::Null,
        |j| {
            j["operations"][3]["response"]
                .as_object_mut()
                .expect("response")
                .remove("sponsor_witness");
        },
        |j| j["operations"][4]["response_request_id"] = json!("request-3"),
        |j| j["operations"][4]["request"]["case"]["step"] = json!("changed"),
        |j| {
            j["operations"].as_array_mut().expect("operations").pop();
        },
        |j| j["terminal"] = json!("timeout"),
    ];
    for mutate in mutations {
        let mut changed = original.clone();
        mutate(&mut changed);
        assert!(replay_continuation_capture(&target, &changed).is_err());
    }
    let empty = continuation_capture_json(&NativeOperationCapture::default());
    assert!(replay_continuation_capture(&target, &empty).is_err());
}

#[test]
fn ambiguous_sponsor_site_and_earlier_layer_refusals_remain_open() {
    let target = reviewed_elements_tapscript().expect("target");
    for index in [4, 6, 8] {
        for reason in [
            "script failed",
            "sponsor OP_EQUALVERIFY",
            "OP_EQUALVERIFY or OP_CHECKSIG",
        ] {
            let mut exchanges = synthetic().exchanges().to_vec();
            let mut observation: Value = serde_json::from_str(
                exchanges[index]
                    .1
                    .observed_detail
                    .as_deref()
                    .expect("detail"),
            )
            .expect("json");
            observation["relay_detail"] = json!(reason);
            observation["detail"]["verdict_reason"] = json!(reason);
            exchanges[index].1.observed_detail = Some(observation.to_string());
            let replay =
                replay_continuation_campaign(&target, &exchanges).expect("retain observation");
            assert_eq!(
                classify_continuation_subject(
                    &replay,
                    ContinuationSubjectId::SUBMISSION_ORDER[(index - 4) / 2]
                ),
                ContinuationObservationClass::Open
            );
        }
    }
    let mut exchanges = synthetic().exchanges().to_vec();
    exchanges[6].1.observed_layer = ObservedOutcomeLayer::RelayPolicyRejection;
    let replay = replay_continuation_campaign(&target, &exchanges).expect("retain earlier layer");
    assert_eq!(
        classify_continuation_subject(&replay, ContinuationSubjectId::WrongVersion),
        ContinuationObservationClass::Open
    );
}

#[test]
fn control_readback_is_required_for_every_negative_classification() {
    let target = reviewed_elements_tapscript().expect("target");
    let mut exchanges = synthetic().exchanges().to_vec();
    exchanges[10]
        .1
        .mined_readback
        .as_mut()
        .expect("mined control")
        .raw_transaction
        .push(0);
    let replay =
        replay_continuation_campaign(&target, &exchanges).expect("retain invalid readback");
    assert_eq!(classes(&replay), [ContinuationObservationClass::Open; 4]);
    let mut exchanges = synthetic().exchanges().to_vec();
    exchanges[2]
        .1
        .mined_readback
        .as_mut()
        .expect("mined parent")
        .raw_transaction
        .push(0);
    assert!(matches!(
        replay_continuation_campaign(&target, &exchanges),
        Err(BirthNativeRefusal::Readback)
    ));
}

#[test]
fn changed_coordinates_and_reordered_exchanges_cannot_replay() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = synthetic().exchanges().to_vec();
    let mut reordered = original.clone();
    reordered.swap(3, 5);
    assert!(replay_continuation_campaign(&target, &reordered).is_err());
    let mut changed = original;
    let mut subject = changed[4].0.subject().clone();
    let OperationSubject::Submission(request) = &mut subject else {
        panic!("submission");
    };
    request.transaction_bytes.push(0);
    changed[4].0 = OperationStep::new(&changed[4].0.case().step, subject);
    assert!(replay_continuation_campaign(&target, &changed).is_err());
}

#[test]
#[ignore = "requires the pinned native image, adapter and fresh capture environment"]
fn keyed_continuation_campaign_runs_against_a_real_target() {
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
    let mut planner = ContinuationCampaignPlanner::new(&target).expect("construction");
    let mut capture = NativeOperationCapture::default();
    let outcome = common::trace_ceremony(
        "keyed_continuation",
        "continuation",
        &mut capture,
        |capture| execute_operations_captured(&target, &binding, &config, &mut planner, capture),
    );
    let wire = continuation_capture_json(&capture);
    // Round-trip the exact wire form that admission will read, not Debug text.
    let wire: Value = serde_json::from_slice(&serde_json::to_vec(&wire).expect("wire encoding"))
        .expect("wire decoding");
    let replay = replay_continuation_capture(&target, &wire);
    let eligible = outcome.is_ok()
        && replay.as_ref().is_ok_and(|p| {
            classes(p)
                .iter()
                .all(|c| *c != ContinuationObservationClass::Open)
        });
    let rows: Vec<_> = planner.records().iter().map(|row| json!({"record": row.capture_json(), "class": format!("{:?}", classify_continuation_subject(&planner, row.subject.id))})).collect();
    let payload = json!({"keyed_continuation_capture_schema": 1, "wire_journal": wire, "rows": rows, "eligible_for_review": eligible, "admitted": false, "executable_provenance": provenance, "construction_refusal": format!("{:?}", planner.refusal()), "replay_refusal": format!("{:?}", replay.as_ref().err())}).to_string();
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    drop(guard);
    let report = format!(
        "keyed-continuation-run-schema 1\nwall-seconds {:.3}\neligible-for-review {eligible}\nadmitted false\nexecutor-completed {}\nreplay-refusal {:?}\nclasses {:?}\n",
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
