//! Synthetic wire replies test carriers and replay, never native acceptance.
use crate::keyed_birth::{coin, response};
use serde_json::{Value, json};
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::executor::{NativeOperationCapture, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, ObservedOutcomeLayer, OperationSubject,
};
use transaction::bytes::{TargetTransaction, ValueField};
use vectors::keyed_funding::*;
use vectors::maturity_native::sponsored::Policy;

const KEY: [u8; 32] = [0x45; 32];
const RUN_TEST_NAME: &str = "keyed_funding::keyed_funding_campaign_runs_against_a_real_target";

fn synthetic() -> FundingPlanner {
    let target = reviewed_elements_tapscript().expect("target");
    let mut planner = FundingPlanner::new(&target, KEY).expect("planner");
    let mut next = planner.next_step(None).expect("funding");
    let mut funding = 0;
    while let Some(step) = next {
        let mut reply = response(&step, 0);
        match step.subject() {
            OperationSubject::SponsorFunding(request) => {
                reply.funded_outputs = (0..request.sponsor_outputs)
                    .map(|i| coin(funding, u32::from(i)))
                    .collect();
                funding += 1;
            }
            OperationSubject::Submission(request) => {
                let tx = TargetTransaction::decode(&request.transaction_bytes).expect("bytes");
                let ValueField::Explicit(handle) = tx.outputs()[0].value() else {
                    panic!("explicit");
                };
                let ValueField::Explicit(fee) = tx.outputs().last().expect("fee").value() else {
                    panic!("explicit");
                };
                reply.resources.transaction_weight = Some(1000);
                let mut detail: Value =
                    serde_json::from_str(reply.observed_detail.as_deref().expect("detail"))
                        .expect("json");
                if handle == 0 {
                    reply.observed_layer = ObservedOutcomeLayer::ConsensusRejectionBeforeScript;
                    reply.accepted_txid = None;
                    reply.mined_readback = None;
                    detail["route"] = json!("refused");
                    detail["relay_allowed"] = json!(false);
                    detail["relay_detail"] = json!("bad-txns-vout-zero");
                    detail["detail"]["verdict_reason"] = json!("bad-txns-vout-zero");
                    detail["detail"]["block_script_error"] = json!("unclassified");
                } else if handle < 300 || fee < 25 {
                    detail["route"] = json!("direct-block");
                    detail["relay_allowed"] = json!(false);
                    detail["relay_detail"] = json!(if handle < 300 {
                        "dust"
                    } else {
                        "min relay fee not met"
                    });
                }
                reply.observed_detail = Some(detail.to_string());
            }
            _ => {}
        }
        next = planner
            .next_step(Some((step.case(), &reply)))
            .expect("synthetic response");
    }
    planner.complete().expect("finite census");
    planner
}
fn journal(planner: &FundingPlanner) -> Value {
    let mut wire = funding_capture_json(&NativeOperationCapture::default(), planner.descriptor());
    wire["terminal"] = json!("complete");
    wire["operations"] = json!(planner.exchanges().iter().enumerate().map(|(n,(step,response))| json!({
        "operation_id":format!("operation-{n}"),"request_id":format!("request-{n}"),"response_id":format!("response-{n}"),"response_request_id":format!("request-{n}"),"response_operation_id":format!("operation-{n}"),
        "request":NativeOperationRequest { schema:NATIVE_PROTOCOL_SCHEMA,case:step.case().clone(),subject:step.subject().clone() },"response":response
    })).collect::<Vec<_>>());
    wire
}
fn submitted(row: &FundingRecord) -> TargetTransaction {
    let OperationSubject::Submission(request) = row.submission.0.subject() else {
        panic!("submission");
    };
    TargetTransaction::decode(&request.transaction_bytes).expect("exact bytes")
}
fn explicit(tx: &TargetTransaction) -> Vec<u64> {
    tx.outputs()
        .iter()
        .map(|o| {
            let ValueField::Explicit(n) = o.value() else {
                panic!("explicit");
            };
            n
        })
        .collect()
}
#[test]
fn fixed_grid_census_and_independent_funding_are_exact() {
    let p = synthetic();
    assert_eq!(p.records().len(), 36);
    assert_eq!(p.exchanges().len(), EXCHANGE_COUNT);
    assert_eq!(
        p.records().iter().map(|r| r.subject).collect::<Vec<_>>(),
        funding_subjects()
    );
    let mut n = 0;
    let mut outpoints = std::collections::BTreeSet::new();
    for row in p.records() {
        assert!(matches!(
            p.exchanges()[n].0.subject(),
            OperationSubject::SponsorFunding(_)
        ));
        let width = if row.subject.role == FundingRole::Source {
            3
        } else {
            5
        };
        assert_eq!(row.signing, p.exchanges()[n + width - 2]);
        assert_eq!(row.submission, p.exchanges()[n + width - 1]);
        for coin in &p.exchanges()[n].1.funded_outputs {
            assert!(outpoints.insert((coin.outpoint.txid.clone(), coin.outpoint.vout)));
        }
        n += width;
    }
    assert_eq!(n, 146);
    assert_eq!(outpoints.len(), 55);
}
#[test]
fn source_and_merge_balance_and_mined_birth_zero_are_bound() {
    let p = synthetic();
    let mut n = 0;
    for row in p.records() {
        let tx = submitted(row);
        let values = explicit(&tx);
        assert_eq!(tx.outputs()[0].program(), p.descriptor().output_program);
        assert_eq!(values[0], row.handle);
        assert_eq!(values.last(), Some(&row.fee));
        assert_eq!(tx.outputs().last().expect("fee").program(), [0_u8; 0]);
        let merge = row.subject.role == FundingRole::Merge;
        assert_eq!(tx.inputs().len(), if merge { 2 } else { 1 });
        assert_eq!(
            values.iter().sum::<u64>(),
            if merge { 110_000 } else { 100_000 }
        );
        if merge {
            let parent = &p.exchanges()[n + 2]
                .1
                .mined_readback
                .as_ref()
                .expect("mined SOURCE");
            assert_eq!(
                tx.inputs()[0].outpoint().txid().to_target_display(),
                parent.transaction_id
            );
            assert_eq!(tx.inputs()[0].outpoint().index(), 0);
            assert_eq!(
                tx.witnesses()[0].stack(),
                [
                    p.descriptor().leaf_bytes.clone(),
                    p.descriptor().control.bytes.clone()
                ]
            );
        }
        n += if merge { 5 } else { 3 };
    }
}
#[test]
fn open_outputs_and_redistribution_preserve_the_successor_program() {
    let p = synthetic();
    let ordinary = submitted(&p.records()[34]);
    let redistributed = submitted(&p.records()[35]);
    assert_eq!(explicit(&ordinary), [10_000, 1000, 98_000, 1000]);
    assert_eq!(explicit(&redistributed), [1000, 10_000, 98_000, 1000]);
    assert_ne!(
        ordinary.outputs()[1].program(),
        ordinary.outputs()[0].program()
    );
    assert_eq!(
        ordinary.outputs()[0].program(),
        redistributed.outputs()[0].program()
    );
    assert_eq!(p.outcome(34), FundingOutcome::RelayMined);
    assert_eq!(p.outcome(35), FundingOutcome::RelayMined);
}
#[test]
fn json_alone_replays_every_response_and_separates_relay_from_consensus() {
    let p = synthetic();
    let expected = p.report_json();
    let bytes = serde_json::to_vec(&journal(&p)).expect("wire");
    drop(p);
    let p = replay_funding_capture(
        &reviewed_elements_tapscript().expect("target"),
        &serde_json::from_slice(&bytes).expect("json"),
    )
    .expect("replay");
    assert_eq!(p.report_json(), expected);
    assert_eq!(p.outcome(2), FundingOutcome::ConsensusRefused);
    assert_eq!(p.outcome(4), FundingOutcome::RelayRefusedBlockMined);
    for role in [FundingRole::Source, FundingRole::Merge] {
        let amount = p.boundary(role, false);
        assert_eq!(amount["last_refused"], 299);
        assert_eq!(amount["first_accepted"], 300);
        assert_eq!(amount["adjacent_satoshis"], true);
        let fee = p.boundary(role, true);
        assert_eq!(fee["last_refused"], 24);
        assert_eq!(fee["first_accepted"], 25);
    }
}
#[test]
fn malformed_wire_recipe_grid_and_signing_echo_are_refused() {
    let p = synthetic();
    let original = journal(&p);
    let target = reviewed_elements_tapscript().expect("target");
    let mutations: [fn(&mut Value); 7] = [
        |j| j["handle_points"][1] = json!(3),
        |j| j["fee_offsets"][0] = json!(-3),
        |j| {
            j["descriptor"]["output_program"][2] =
                json!(j["descriptor"]["output_program"][2].as_u64().expect("byte") ^ 1);
        },
        |j| j["operations"][1]["response_request_id"] = json!("other"),
        |j| {
            j["operations"][1]["response"]["signature_bound_to"]
                .as_array_mut()
                .expect("echo")
                .push(json!(0));
        },
        |j| {
            j["operations"].as_array_mut().expect("operations").pop();
        },
        |j| j["terminal"] = json!("timeout"),
    ];
    for mutate in mutations {
        let mut j = original.clone();
        mutate(&mut j);
        assert!(replay_funding_capture(&target, &j).is_err());
    }
}
#[test]
fn absent_policy_weight_readback_or_consensus_evidence_stays_open() {
    let p = synthetic();
    let original = journal(&p);
    let target = reviewed_elements_tapscript().expect("target");
    // First handle-one SOURCE, not a required setup/control.
    let index = p
        .exchanges()
        .iter()
        .position(|(s, _)| s == &p.records()[4].submission.0)
        .expect("row");
    for field in ["policy", "route"] {
        let mut j = original.clone();
        let r = &mut j["operations"][index]["response"];
        let mut detail: Value =
            serde_json::from_str(r["observed_detail"].as_str().expect("detail")).expect("json");
        detail[field] = Value::Null;
        r["observed_detail"] = json!(detail.to_string());
        let replay = replay_funding_capture(&target, &j).expect("retain unknown outcome");
        assert_eq!(replay.outcome(4), FundingOutcome::Open);
    }
    let mut j = original.clone();
    j["operations"][index]["response"]["resources"]["transaction_weight"] = Value::Null;
    let replay = replay_funding_capture(&target, &j).expect("retain absent weight");
    assert_eq!(replay.outcome(4), FundingOutcome::Open);
    let mut j = original;
    j["operations"][index]["response"]["mined_readback"]["raw_transaction"]
        .as_array_mut()
        .expect("readback")
        .push(json!(0));
    let replay = replay_funding_capture(&target, &j).expect("retain bad point readback");
    assert_eq!(replay.outcome(4), FundingOutcome::Open);
}
#[test]
fn no_threshold_is_invented_when_all_positive_handles_relay() {
    let p = synthetic();
    let mut j = journal(&p);
    let control: Value = serde_json::from_str(
        p.records()[0]
            .submission
            .1
            .observed_detail
            .as_deref()
            .expect("detail"),
    )
    .expect("json");
    for (i, row) in p.records().iter().enumerate() {
        if matches!(row.subject.point, FundingPoint::Handle(1..))
            && p.outcome(i) == FundingOutcome::RelayRefusedBlockMined
        {
            let index = p
                .exchanges()
                .iter()
                .position(|(s, _)| s == &row.submission.0)
                .expect("row");
            let r = &mut j["operations"][index]["response"];
            let mut d: Value =
                serde_json::from_str(r["observed_detail"].as_str().expect("detail")).expect("json");
            d["route"] = json!("relay-and-template");
            d["relay_allowed"] = json!(true);
            d["relay_detail"] = Value::Null;
            d["detail"]["template"] = control["detail"]["template"].clone();
            d["detail"]["template"]["txids"] = json!([d["detail"]["txid"]]);
            r["observed_detail"] = json!(d.to_string());
        }
    }
    let replay = replay_funding_capture(&reviewed_elements_tapscript().expect("target"), &j)
        .expect("replay");
    assert_eq!(
        replay.boundary(FundingRole::Source, false)["reading"],
        "no discovered threshold in tested range"
    );
    assert_eq!(
        replay.boundary(FundingRole::Merge, false)["first_accepted"],
        1
    );
}
#[test]
fn exact_fee_arithmetic_rounds_up_and_rejects_missing_weight() {
    let p = synthetic();
    let d: Value = serde_json::from_str(
        p.records()[0]
            .submission
            .1
            .observed_detail
            .as_deref()
            .expect("detail"),
    )
    .expect("json");
    let policy = Policy::parse(d["policy"].clone()).expect("policy");
    assert_eq!(reported_floor(&policy, 1000), Ok(25));
    assert_eq!(reported_floor(&policy, 1001), Ok(26));
    assert!(reported_floor(&policy, 0).is_err());
    assert!(reported_floor(&policy, u64::MAX).is_err());
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
fn keyed_funding_campaign_runs_against_a_real_target() {
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
    let mut planner = FundingPlanner::new(&target, KEY).expect("recipe");
    let mut capture = NativeOperationCapture::default();
    let outcome = common::trace_ceremony("keyed_funding", "funding", &mut capture, |capture| {
        execute_operations_captured(&target, &binding, &config, &mut planner, capture)
    });
    let wire = funding_capture_json(&capture, planner.descriptor());
    let wire: Value = serde_json::from_slice(&serde_json::to_vec(&wire).expect("wire encoding"))
        .expect("wire decoding");
    let replay = replay_funding_capture(&target, &wire);
    let eligible = outcome.is_ok()
        && replay.as_ref().is_ok_and(|p| {
            (0..p.records().len()).all(|i| p.outcome(i) != FundingOutcome::Open)
                && [0, 1, 34, 35]
                    .into_iter()
                    .all(|i| p.outcome(i) == FundingOutcome::RelayMined)
        });
    let payload=json!({"keyed_funding_capture_schema":1,"wire_journal":wire,"report":planner.report_json(),"eligible_for_review":eligible,"admitted":false,"executable_provenance":provenance,"construction_refusal":format!("{:?}",planner.refusal()),"replay_refusal":format!("{:?}",replay.as_ref().err())}).to_string();
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    drop(guard);
    let report = format!(
        "keyed-funding-run-schema 1\nwall-seconds {:.3}\neligible-for-review {eligible}\nadmitted false\nexecutor-completed {}\nreplay-refusal {:?}\nexchange-count {}\nsubject-count {}\n",
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

#[test]
fn changed_setup_policy_parent_bytes_and_reused_funding_refuse() {
    let original = journal(&synthetic());
    let target = reviewed_elements_tapscript().expect("target");
    let mut changed = original.clone();
    // Second subject's SOURCE parent is exchange five.
    let r = &mut changed["operations"][5]["response"];
    let mut d: Value =
        serde_json::from_str(r["observed_detail"].as_str().expect("detail")).expect("json");
    d["policy"]["relay"] = json!("0.00000200");
    r["observed_detail"] = json!(d.to_string());
    assert!(replay_funding_capture(&target, &changed).is_err());
    let mut changed = original.clone();
    changed["operations"][5]["response"]["mined_readback"]["raw_transaction"]
        .as_array_mut()
        .expect("bytes")
        .push(json!(0));
    assert!(replay_funding_capture(&target, &changed).is_err());
    let mut changed = original.clone();
    changed["operations"][3]["response"]["funded_outputs"][0]["outpoint"] =
        original["operations"][0]["response"]["funded_outputs"][0]["outpoint"].clone();
    assert!(replay_funding_capture(&target, &changed).is_err());
}

#[test]
fn ambiguous_boundary_reasons_and_missing_block_verdict_remain_open() {
    let p = synthetic();
    let original = journal(&p);
    let target = reviewed_elements_tapscript().expect("target");
    for (record, field) in [(4, "relay_detail"), (2, "block_script_error")] {
        let index = p
            .exchanges()
            .iter()
            .position(|(s, _)| s == &p.records()[record].submission.0)
            .expect("row");
        let mut changed = original.clone();
        let r = &mut changed["operations"][index]["response"];
        let mut d: Value =
            serde_json::from_str(r["observed_detail"].as_str().expect("detail")).expect("json");
        if record == 4 {
            d[field] = json!("unspecified policy refusal");
        } else {
            d["detail"][field] = Value::Null;
        }
        r["observed_detail"] = json!(d.to_string());
        let replay = replay_funding_capture(&target, &changed).expect("retain ambiguous response");
        if record == 4 {
            assert_eq!(
                replay.boundary(FundingRole::Source, false)["reading"],
                "open: incomplete, nonmonotone or unattributed outcomes"
            );
        } else {
            assert_eq!(replay.outcome(record), FundingOutcome::Open);
        }
    }
}

#[test]
fn an_all_refused_range_has_no_invented_admission_endpoint() {
    let p = synthetic();
    let mut j = journal(&p);
    for row in p.records() {
        if matches!(row.subject.point, FundingPoint::Handle(1..)) {
            let index = p
                .exchanges()
                .iter()
                .position(|(s, _)| s == &row.submission.0)
                .expect("row");
            let r = &mut j["operations"][index]["response"];
            let mut d: Value =
                serde_json::from_str(r["observed_detail"].as_str().expect("detail")).expect("json");
            d["route"] = json!("direct-block");
            d["relay_allowed"] = json!(false);
            d["relay_detail"] = json!("dust");
            r["observed_detail"] = json!(d.to_string());
        }
    }
    let replay = replay_funding_capture(&reviewed_elements_tapscript().expect("target"), &j)
        .expect("replay");
    let boundary = replay.boundary(FundingRole::Source, false);
    assert_eq!(boundary["reading"], "no admitted point in tested range");
    assert_eq!(boundary["first_accepted"], Value::Null);
    assert_eq!(boundary["last_refused"], 10_000);
}
