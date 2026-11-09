//! Synthetic responses prove construction and replay, not node acceptance.
use crate::keyed_birth::{coin, response};
use serde_json::{Value, json};
use std::sync::OnceLock;
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::executor::{NativeOperationCapture, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    MAXIMUM_REQUEST_BYTES, NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, OperationSubject,
    ProtocolLimits,
};
use transaction::bytes::{TargetTransaction, ValueField};
use vectors::keyed_scaling::*;

const KEY: [u8; 32] = [0x47; 32];
const RUN_TEST_NAME: &str = "keyed_scaling::keyed_scaling_campaign_runs_against_a_real_target";

fn synthetic() -> &'static ScalingPlanner {
    static PLANNER: OnceLock<ScalingPlanner> = OnceLock::new();
    PLANNER.get_or_init(|| {
        let target = reviewed_elements_tapscript().expect("target");
        let mut p = ScalingPlanner::new(&target, KEY).expect("recipe");
        let mut next = p.next_step(None).expect("first");
        let mut batch = 0;
        while let Some(step) = next {
            let mut reply = response(&step, 0);
            match step.subject() {
                OperationSubject::SponsorFunding(request) => {
                    reply.funded_outputs = (0..request.sponsor_outputs)
                        .map(|i| coin(batch, u32::from(i)))
                        .collect();
                    batch += 1;
                }
                OperationSubject::SponsorSigning(_) => {
                    reply.sponsor_witness[0] = vec![0x30; SPONSOR_SIGNATURE_BYTES];
                    *reply.sponsor_witness[0].last_mut().expect("sighash") = 1;
                }
                OperationSubject::Submission(request) => {
                    let tx =
                        TargetTransaction::decode(&request.transaction_bytes).expect("transaction");
                    reply.resources.transaction_weight = Some(tx.weight());
                    if tx.inputs().len() == PROPOSED_MAXIMUM + 2 {
                        let mut d: Value =
                            serde_json::from_str(reply.observed_detail.as_deref().expect("detail"))
                                .expect("JSON");
                        d["route"] = json!("direct-block");
                        d["relay_allowed"] = json!(false);
                        d["relay_detail"] = json!("tx-size");
                        reply.observed_detail = Some(d.to_string());
                    }
                }
                _ => panic!("finite subject types"),
            }
            next = p
                .next_step(Some((step.case(), &reply)))
                .expect("synthetic response");
        }
        p.complete().expect("census");
        p
    })
}
fn journal() -> Value {
    let p = synthetic();
    let mut j = scaling_capture_json(&NativeOperationCapture::default(), p.descriptor());
    j["terminal"] = json!("complete");
    j["operations"] = json!(p.exchanges().iter().enumerate().map(|(n,(step,response))| json!({
        "operation_id":format!("operation-{n}"),"request_id":format!("request-{n}"),"response_id":format!("response-{n}"),"response_request_id":format!("request-{n}"),"response_operation_id":format!("operation-{n}"),
        "request":NativeOperationRequest { schema:NATIVE_PROTOCOL_SCHEMA,case:step.case().clone(),subject:step.subject().clone() },"response":response
    })).collect::<Vec<_>>());
    j
}
fn submitted(r: &ScalingRecord) -> TargetTransaction {
    let OperationSubject::Submission(request) = r.submission.0.subject() else {
        panic!("submission");
    };
    TargetTransaction::decode(&request.transaction_bytes).expect("carrier")
}

#[test]
fn frozen_arithmetic_and_boundary_weights_include_worst_sponsor_and_compact_size() {
    assert_eq!((INTERIOR, PROPOSED_MAXIMUM), (782, 1564));
    let p = synthetic();
    assert!(p.eligible());
    let merges: Vec<_> = p
        .records()
        .iter()
        .filter(|r| r.preparation.is_none())
        .collect();
    assert_eq!(merges.len(), 5);
    for r in &merges {
        let n = COUNTS[r.row];
        let expected = 1077 + INPUT_WEIGHT * n as u64 + if n + 1 >= 253 { 8 } else { 0 };
        assert_eq!(submitted(r).weight(), expected);
        assert_eq!(
            r.signing.1.sponsor_witness[0].len(),
            SPONSOR_SIGNATURE_BYTES
        );
    }
    let neighbour = submitted(merges[3]);
    let control = submitted(merges[4]);
    assert_eq!(control.weight(), 399_905);
    assert_eq!(neighbour.weight(), 400_160);
    assert!(control.weight() < STANDARD_WEIGHT && neighbour.weight() > STANDARD_WEIGHT);
    assert_eq!(neighbour.inputs().len(), control.inputs().len() + 1);
    assert_eq!(
        p.classification(merges[3]),
        ScalingClass::WeightPolicyRefusedBlockMined
    );
    // The existing constructor exposes only a consensus weight check, no keyed N.
    let target = reviewed_elements_tapscript().expect("target");
    assert!(transaction::construct::check_weight(&target, &control).is_ok());
    assert!(transaction::construct::check_weight(&target, &neighbour).is_ok());
}

#[test]
fn every_merge_consumes_distinct_mined_output_zero_and_balances_explicit_asset() {
    let p = synthetic();
    let mut handles = std::collections::BTreeMap::new();
    for r in p.records() {
        let tx = submitted(r);
        if r.preparation.is_some() {
            let mined = r
                .submission
                .1
                .mined_readback
                .as_ref()
                .expect("mined SOURCE");
            assert!(
                handles
                    .insert(mined.transaction_id.clone(), r.row)
                    .is_none()
            );
            assert_eq!(tx.inputs().len(), 1);
        } else {
            let n = COUNTS[r.row];
            assert_eq!(tx.inputs().len(), n + 1);
            for (input, witness) in tx.inputs()[..n].iter().zip(&tx.witnesses()[..n]) {
                assert_eq!(input.outpoint().index(), 0);
                assert_eq!(
                    handles.remove(&input.outpoint().txid().to_target_display()),
                    Some(r.row)
                );
                assert_eq!(
                    witness.stack(),
                    [
                        p.descriptor().leaf_bytes.clone(),
                        p.descriptor().control.bytes.clone()
                    ]
                );
            }
        }
        let amounts: Vec<_> = tx
            .outputs()
            .iter()
            .map(|o| match o.value() {
                ValueField::Explicit(n) => n,
                _ => panic!("explicit"),
            })
            .collect();
        let n = tx.inputs().len() - 1;
        assert_eq!(amounts.iter().sum::<u64>(), 100_000 + 10_000 * n as u64);
        assert_eq!(amounts[0], 10_000);
        assert_eq!(amounts[2], if n == 0 { 1000 } else { MERGE_FEE });
        assert_eq!(tx.outputs()[0].program(), p.descriptor().output_program);
        assert_eq!(tx.outputs()[2].program(), [0_u8; 0]);
        assert!(
            tx.outputs()
                .iter()
                .all(|o| o.asset() == tx.outputs()[0].asset())
        );
    }
    assert_eq!(handles.len(), 1, "standalone SOURCE remains unspent");
}

#[test]
fn exchange_census_order_and_protocol_record_sizes_fit_without_sibling_handles() {
    let p = synthetic();
    assert_eq!(SOURCE_COUNT, 3915);
    assert_eq!(EXCHANGE_COUNT, 7861);
    assert_eq!(p.exchanges().len(), EXCHANGE_COUNT);
    assert_eq!(p.records().len(), SOURCE_COUNT + 5);
    assert_eq!(
        p.exchanges()
            .iter()
            .filter(|(s, _)| matches!(s.subject(), OperationSubject::SponsorFunding(_)))
            .count(),
        FUNDING_COUNT
    );
    let mut max_request = 0;
    let mut max_response = 0;
    for (s, r) in p.exchanges() {
        max_request = max_request.max(
            serde_json::to_vec(&NativeOperationRequest {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case: s.case().clone(),
                subject: s.subject().clone(),
            })
            .expect("request")
            .len(),
        );
        max_response = max_response.max(serde_json::to_vec(r).expect("response").len());
    }
    // Reserve 64 KiB for envelope identities and native details/template data.
    assert!(max_request + 65_536 < MAXIMUM_REQUEST_BYTES);
    assert!(max_response + 65_536 < ProtocolLimits::DEFAULT.maximum_response_bytes);
    assert_eq!(p.records().last().expect("last").row, 5);
}

#[test]
fn complete_capture_replays_from_wire_json_without_debug_text() {
    let j: Value =
        serde_json::from_slice(&serde_json::to_vec(&journal()).expect("JSON")).expect("wire");
    let replay = replay_scaling_capture(&reviewed_elements_tapscript().expect("target"), &j)
        .expect("replay");
    assert_eq!(replay.report_json(), synthetic().report_json());
    assert!(replay.eligible());
}

#[test]
fn changed_counts_descriptor_signing_echo_and_missing_response_refuse() {
    let target = reviewed_elements_tapscript().expect("target");
    for mutation in 0..4 {
        let mut j = journal();
        match mutation {
            0 => j["counts"][3] = json!(783),
            1 => j["descriptor"]["semantic_key"][0] = json!(0),
            2 => j["operations"][1]["response"]["signature_bound_to"] = json!([]),
            _ => {
                j["operations"][1]["response"]
                    .as_object_mut()
                    .expect("response")
                    .remove("sponsor_witness");
            }
        }
        assert!(replay_scaling_capture(&target, &j).is_err());
    }
}

#[test]
fn wrong_source_readback_and_reused_funding_never_become_handles() {
    let target = reviewed_elements_tapscript().expect("target");
    let mut j = journal();
    j["operations"][2]["response"]["mined_readback"]["raw_transaction"] = json!([]);
    assert!(replay_scaling_capture(&target, &j).is_err());
    let mut j = journal();
    j["operations"][3]["response"]["funded_outputs"][0]["outpoint"] =
        j["operations"][0]["response"]["funded_outputs"][0]["outpoint"].clone();
    assert!(replay_scaling_capture(&target, &j).is_err());
}

#[test]
fn ambiguous_boundary_and_false_weight_or_fee_evidence_stay_open() {
    let p = synthetic();
    let row = p
        .records()
        .iter()
        .find(|r| r.preparation.is_none() && COUNTS[r.row] == PROPOSED_MAXIMUM + 1)
        .expect("neighbour");
    let mut changed = row.clone();
    let mut d: Value = serde_json::from_str(
        changed
            .submission
            .1
            .observed_detail
            .as_deref()
            .expect("detail"),
    )
    .expect("JSON");
    d["relay_detail"] = json!("unspecified refusal");
    changed.submission.1.observed_detail = Some(d.to_string());
    assert_eq!(p.classification(&changed), ScalingClass::Open);
    let mut changed = row.clone();
    changed.submission.1.resources.transaction_weight = Some(1);
    assert_eq!(p.classification(&changed), ScalingClass::Open);
    let mut changed = row.clone();
    let mut d: Value = serde_json::from_str(
        changed
            .submission
            .1
            .observed_detail
            .as_deref()
            .expect("detail"),
    )
    .expect("JSON");
    d["policy"]["relay"] = json!("0.10000000");
    changed.submission.1.observed_detail = Some(d.to_string());
    assert_eq!(p.classification(&changed), ScalingClass::Open);
}

use crate::{
    common,
    keyed_birth::{executable_provenance, required, write_run_files},
};
use std::{
    fmt::Write as _,
    path::Path,
    time::{Duration, Instant},
};
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
fn keyed_scaling_campaign_runs_against_a_real_target() {
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
    .expect("provenance");
    destination.report_test_name = RUN_TEST_NAME;
    std::fs::create_dir_all(directory).expect("directory");
    for name in [
        format!("{short}.report.capture"),
        format!("{short}.report.timing"),
        format!("{short}.report.capture.timing"),
        "RUN-REPORT".into(),
        "MANIFEST.sha256".into(),
    ] {
        assert!(!directory.join(name).exists(), "fresh destination");
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
        Duration::from_secs(3600),
        ExecutorDiagnostics::in_directory(&directory.join("diagnostics")),
    );
    let mut planner = ScalingPlanner::new(&target, KEY).expect("recipe");
    let mut capture = NativeOperationCapture::default();
    let outcome = common::trace_ceremony("keyed_scaling", "scaling", &mut capture, |capture| {
        execute_operations_captured(&target, &binding, &config, &mut planner, capture)
    });
    let wire = scaling_capture_json(&capture, planner.descriptor());
    let wire: Value = serde_json::from_slice(&serde_json::to_vec(&wire).expect("wire encoding"))
        .expect("wire decoding");
    let replay = replay_scaling_capture(&target, &wire);
    let eligible = outcome.is_ok() && replay.as_ref().is_ok_and(ScalingPlanner::eligible);
    let payload=json!({"keyed_scaling_capture_schema":1,"wire_journal":wire,"report":planner.report_json(),"eligible_for_review":eligible,"admitted":false,"executable_provenance":provenance,"construction_refusal":format!("{:?}",planner.refusal()),"replay_refusal":format!("{:?}",replay.as_ref().err())}).to_string();
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    drop(guard);
    let report = format!(
        "keyed-scaling-run-schema 1\nwall-seconds {:.3}\neligible-for-review {eligible}\nadmitted false\nexecutor-completed {}\nreplay-refusal {:?}\nexchange-count {}\nsubject-count {}\n",
        started.elapsed().as_secs_f64(),
        outcome.is_ok(),
        replay.as_ref().err(),
        planner.exchanges().len(),
        planner.records().len()
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
            &std::fs::read(directory.join(&timing_name)).expect("timing")
        ))
    )
    .expect("manifest line");
    std::fs::write(directory.join("MANIFEST.sha256"), manifest).expect("manifest");
    assert!(
        eligible,
        "capture retained but ineligible; see wire journal and RUN-REPORT"
    );
}
