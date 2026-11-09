//! Offline cost dimensions and a deliberately opt-in focused capture.

use crate::keyed_birth::{coin, response};
use serde_json::{Value, json};
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::executor::{NativeOperationCapture, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, ObservedOutcomeLayer, OperationSubject,
};
use transaction::bytes::{InputWitness, TargetTransaction};
use vectors::keyed_cost::*;
use vectors::keyed_cost_native::*;

const RUN_TEST_NAME: &str = "keyed_cost::keyed_cost_campaign_runs_against_a_real_target";

#[test]
fn packet_projects_exact_programs_and_keeps_native_dimensions_absent() {
    let packet = cost_packet().expect("admitted archives");
    assert_eq!(packet.bindings.len(), 3);
    for binding in &packet.bindings {
        assert_eq!(binding.executor().protocol_schema, 8);
        assert_eq!(binding.executor().adapter_version, "2.1.0");
    }
    assert_eq!(
        [
            packet.birth.instructions,
            packet.continuation.instructions,
            packet.keyfree_composed.instructions,
            packet.keyed.instructions
        ],
        [7, 8, 14, 16]
    );
    assert_eq!(
        [
            packet.birth.script_bytes,
            packet.continuation.script_bytes,
            packet.keyfree_composed.script_bytes,
            packet.keyed.script_bytes
        ],
        [11, 8, 18, 52]
    );
    assert_eq!(
        [
            packet.birth.peak_main,
            packet.continuation.peak_main,
            packet.keyfree_composed.peak_main,
            packet.keyed.peak_main
        ],
        [3, 4, 4, 4]
    );
    for cost in [
        &packet.birth,
        &packet.continuation,
        &packet.keyfree_composed,
        &packet.keyed,
    ] {
        assert_eq!(cost.peak_alternate, 0);
        assert_eq!(cost.script.len() as u64, cost.script_bytes);
        assert_eq!(cost.validation_budget, 0);
    }
    assert_eq!(packet.absent.len(), 6);
    assert!(packet.absent[0].contains("isolated continuation"));
    for row in &packet.carriers {
        assert_eq!(row.reported.peak_stack_items, None);
        assert_eq!(row.reported.peak_altstack_items, None);
        assert_eq!(row.reported.maximum_element_bytes, None);
        assert_eq!(row.reported.validation_budget_used, None);
        assert_eq!(row.reported.script_bytes, None);
        assert_eq!(row.reported.initial_stack_items, None);
    }
}

#[test]
fn admitted_weights_match_exact_carriers_without_clause_subtraction() {
    let packet = cost_packet().expect("packet");
    assert_eq!(packet.carriers.len(), 16);
    let weights: Vec<_> = packet
        .carriers
        .iter()
        .map(|r| r.node_weight.expect("node weight"))
        .collect();
    assert_eq!(
        weights,
        [
            1290, 1291, 1290, 1291, 1346, 1346, 1345, 1346, 1331, 1331, 1309, 1331, 1331, 1587,
            1586, 1331
        ]
    );
    for row in &packet.carriers {
        assert_eq!(
            row.weight_matches,
            Some(true),
            "{} {}",
            row.archive,
            row.step
        );
        let tx = TargetTransaction::decode(&row.transaction).expect("exact carrier");
        assert_eq!(tx.encode(), row.transaction);
        assert_eq!(
            row.witness_bytes,
            row.witnesses
                .iter()
                .map(|w| w.input_witness_bytes)
                .sum::<usize>()
                + tx.output_witness_bytes().len()
        );
    }
}

#[test]
fn fixed_descriptor_and_repeated_leaf_control_are_distinct_dimensions() {
    let packet = cost_packet().expect("packet");
    assert_eq!(packet.constructor.key_commitment_script_bytes, 34);
    assert_eq!(packet.constructor.repeated_leaf_bytes, 52);
    assert_eq!(packet.constructor.repeated_control_bytes, 33);
    assert_eq!(packet.constructor.control_depth, 0);
    let descriptor = &packet.constructor.descriptor;
    assert_eq!(
        packet.script_paths[3].output_program,
        descriptor.output_program
    );
    assert_eq!(packet.script_paths[3].control, descriptor.control.bytes);
    assert_eq!(
        packet
            .script_paths
            .iter()
            .map(|p| p.witness.script_witness_bytes)
            .collect::<Vec<_>>(),
        [47, 44, 54, 88]
    );
    for path in &packet.script_paths {
        assert_eq!(path.control_depth, 0);
    }
    // Same-key two-input control repeats the leaf and control twice; no amortization.
    let row = &packet.carriers[14];
    assert_eq!(row.witnesses[0].item_bytes, [52, 33]);
    assert_eq!(row.witnesses[1].item_bytes, [52, 33]);
    assert_eq!(row.witnesses[0].input_witness_bytes, 91);
}

#[test]
fn compact_size_boundaries_are_counted_as_framing() {
    let witness = InputWitness::new(vec![vec![0; 252], vec![0; 253], vec![0; 65536]]);
    let cost = witness_cost(&witness);
    assert_eq!(cost.script_witness_bytes, 1 + 1 + 252 + 3 + 253 + 5 + 65536);
    assert_eq!(cost.input_witness_bytes, cost.script_witness_bytes + 3);
}

#[test]
fn altered_resource_report_refuses_beside_unchanged_archive() {
    let packet = cost_packet().expect("packet");
    let row = &packet.carriers[12];
    assert_eq!(validate_resource_report(row, row.reported), Ok(()));
    let mut altered = row.reported;
    altered.transaction_weight = altered.transaction_weight.map(|n| n + 1);
    assert_eq!(
        validate_resource_report(row, altered),
        Err(CostRefusal::ResourceReport)
    );
    let mut altered = row.reported;
    altered.peak_stack_items = Some(packet.keyed.peak_main);
    assert_eq!(
        validate_resource_report(row, altered),
        Err(CostRefusal::ResourceReport)
    );
    let mut altered = row.reported;
    altered.validation_budget_used = Some(0);
    assert_eq!(
        validate_resource_report(row, altered),
        Err(CostRefusal::ResourceReport)
    );
}

fn synthetic() -> CostCampaignPlanner {
    let target = reviewed_elements_tapscript().expect("target");
    let mut planner = CostCampaignPlanner::new(&target).expect("planner");
    let mut next = planner.next_step(None).expect("first");
    let mut n = 0;
    while let Some(step) = next {
        let mut reply = response(&step, usize::from(n >= 5));
        if n == 0 || n == 5 {
            reply.funded_outputs = (0..2).map(|i| coin(u8::from(n >= 5), i)).collect();
        }
        if let OperationSubject::Submission(_) = step.subject() {
            let cost = carrier_cost("synthetic", &step, &reply).expect("serialized cost");
            reply.resources.transaction_weight = Some(cost.serialized_weight);
        }
        if n == 9 {
            let reason = "mandatory-script-verify-flag-failed (Push value size limit exceeded)";
            reply.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
            reply.observed_detail = Some(json!({"route":"refused", "relay_allowed":false, "relay_detail":reason, "detail":{"verdict_reason":reason}}).to_string());
            reply.accepted_txid = None;
            reply.mined_readback = None;
        }
        next = planner
            .next_step(Some((step.case(), &reply)))
            .expect("synthetic response");
        n += 1;
    }
    planner.complete().expect("complete");
    planner
}

fn journal(planner: &CostCampaignPlanner) -> Value {
    let mut journal = cost_capture_json(&NativeOperationCapture::default());
    journal["terminal"] = json!("complete");
    journal["operations"] = json!(planner.exchanges().iter().enumerate().map(|(n,(step,response))| json!({"operation_id":format!("operation-{n}"), "request_id":format!("request-{n}"), "response_id":format!("response-{n}"), "response_request_id":format!("request-{n}"), "response_operation_id":format!("operation-{n}"), "request": NativeOperationRequest { schema:NATIVE_PROTOCOL_SCHEMA, case:step.case().clone(), subject:step.subject().clone() }, "response":response})).collect::<Vec<_>>());
    journal
}

fn tx(planner: &CostCampaignPlanner, id: CostSubjectId) -> TargetTransaction {
    let (step, _) = planner.subject(id).expect("subject");
    let OperationSubject::Submission(request) = step.subject() else {
        panic!("submission");
    };
    TargetTransaction::decode(&request.transaction_bytes).expect("decode")
}

#[test]
fn staged_resource_pair_changes_only_argument_length_before_same_coin_control() {
    let planner = synthetic();
    assert_eq!(planner.exchanges().len(), 12);
    let negative = tx(&planner, CostSubjectId::OversizedWitness);
    let control = tx(&planner, CostSubjectId::ResourceControl);
    assert_eq!(
        negative.encode_without_witness(),
        control.encode_without_witness()
    );
    assert_eq!(negative.witnesses()[1], control.witnesses()[1]);
    assert_eq!(
        &negative.witnesses()[0].stack()[1..],
        &control.witnesses()[0].stack()[1..]
    );
    assert_eq!(negative.witnesses()[0].stack()[0], vec![0; 521]);
    assert_eq!(control.witnesses()[0].stack()[0], [0]);
    assert_eq!(control.witnesses()[0].stack()[1], [0x75, 0x51]);
    let isolated = tx(&planner, CostSubjectId::IsolatedContinuation);
    assert_eq!(
        isolated.witnesses()[0].stack()[0],
        [0xcd, 0xca, 0, 0xd1, 0x7b, 0x88, 0x88, 0x51]
    );
    assert_ne!(
        isolated.inputs()[0].outpoint(),
        control.inputs()[0].outpoint()
    );
}

#[test]
fn focused_capture_replays_complete_wire_json_without_debug_text() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = synthetic();
    let bytes = serde_json::to_vec(&journal(&original)).expect("wire JSON");
    let replay = replay_cost_capture(&target, &serde_json::from_slice(&bytes).expect("JSON only"))
        .expect("replay");
    assert_eq!(replay.exchanges(), original.exchanges());
    assert!(cost_control_mined(
        &replay,
        CostSubjectId::IsolatedContinuation
    ));
    assert!(cost_control_mined(&replay, CostSubjectId::ResourceControl));
    assert!(oversized_resource_refused(&replay));
}

#[test]
fn malformed_links_missing_responses_and_changed_signing_echo_refuse() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = journal(&synthetic());
    let mutations: [fn(&mut Value); 3] = [
        |j| j["operations"][3]["response_request_id"] = json!("other"),
        |j| {
            j["operations"][3]["response"]
                .as_object_mut()
                .expect("object")
                .remove("script_path_witness");
        },
        |j| {
            j["operations"][3]["response"]["signature_bound_to"]
                .as_array_mut()
                .expect("bytes")
                .push(json!(0));
        },
    ];
    for mutate in mutations {
        let mut changed = original.clone();
        mutate(&mut changed);
        assert!(replay_cost_capture(&target, &changed).is_err());
    }
}

#[test]
fn ambiguous_resource_error_and_missing_control_remain_open() {
    let target = reviewed_elements_tapscript().expect("target");
    let original = journal(&synthetic());
    let mut changed = original.clone();
    changed["operations"][9]["response"]["observed_detail"] = json!(json!({"route":"refused","relay_allowed":false,"relay_detail":"script failed","detail":{"verdict_reason":"script failed"}}).to_string());
    assert!(!oversized_resource_refused(
        &replay_cost_capture(&target, &changed).expect("retain ambiguity")
    ));
    let mut changed = original;
    changed["operations"][11]["response"]["mined_readback"]["raw_transaction"]
        .as_array_mut()
        .expect("bytes")
        .push(json!(0));
    assert!(!oversized_resource_refused(
        &replay_cost_capture(&target, &changed).expect("retain bad readback")
    ));
}

#[test]
fn changed_node_weight_is_retained_as_a_finding_not_a_new_prediction() {
    let planner = synthetic();
    let (step, response) = planner
        .subject(CostSubjectId::IsolatedContinuation)
        .expect("control");
    let mut changed = response.clone();
    changed.resources.transaction_weight = Some(1);
    let row = carrier_cost("synthetic-mutant", step, &changed).expect("retain mismatch");
    assert_eq!(row.weight_matches, Some(false));
    changed.resources.transaction_weight = None;
    assert_eq!(
        carrier_cost("synthetic-absent", step, &changed)
            .expect("retain absence")
            .weight_matches,
        None
    );
}

#[test]
fn witness_policy_refusal_does_not_claim_a_consensus_size_site() {
    let target = reviewed_elements_tapscript().expect("target");
    let mut changed = journal(&synthetic());
    changed["operations"][9]["response"]["observed_detail"] = json!(json!({"route":"refused", "relay_allowed":false, "relay_detail":"bad-witness-nonstandard", "detail":{"verdict_reason":"block-validation-failed"}}).to_string());
    let replay = replay_cost_capture(&target, &changed).expect("retain policy observation");
    assert!(oversized_policy_refused(&replay));
    assert!(!oversized_resource_refused(&replay));
}

fn cost_rows(planner: &CostCampaignPlanner) -> Vec<Value> {
    [CostSubjectId::IsolatedContinuation, CostSubjectId::OversizedWitness, CostSubjectId::ResourceControl].into_iter().filter_map(|id| planner.subject(id).map(|(step, response)| json!({"subject": format!("{id:?}"), "request":step.subject(), "response":response, "carrier":carrier_cost("focused-cost-capture", step, response).ok(), "mined_control":cost_control_mined(planner,id), "size_refusal": id == CostSubjectId::OversizedWitness && oversized_resource_refused(planner), "witness_policy_refusal": id == CostSubjectId::OversizedWitness && oversized_policy_refused(planner)}))).collect()
}

fn eligible_cost_capture(planner: &CostCampaignPlanner) -> bool {
    planner.complete().is_ok()
        && cost_control_mined(planner, CostSubjectId::IsolatedContinuation)
        && cost_control_mined(planner, CostSubjectId::ResourceControl)
        && (oversized_resource_refused(planner) || oversized_policy_refused(planner))
        && [
            CostSubjectId::IsolatedContinuation,
            CostSubjectId::ResourceControl,
        ]
        .into_iter()
        .all(|id| {
            planner
                .subject(id)
                .is_some_and(|(_, response)| response.resources.transaction_weight.is_some())
        })
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
fn keyed_cost_campaign_runs_against_a_real_target() {
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
    let mut planner = CostCampaignPlanner::new(&target).expect("construction");
    let mut capture = NativeOperationCapture::default();
    let outcome = common::trace_ceremony("keyed_cost", "cost", &mut capture, |capture| {
        execute_operations_captured(&target, &binding, &config, &mut planner, capture)
    });
    let wire = cost_capture_json(&capture);
    // Round-trip the exact wire form that admission will read, not Debug text.
    let wire: Value = serde_json::from_slice(&serde_json::to_vec(&wire).expect("wire encoding"))
        .expect("wire decoding");
    let replay = replay_cost_capture(&target, &wire);
    let eligible = outcome.is_ok() && replay.as_ref().is_ok_and(eligible_cost_capture);
    let rows = cost_rows(&planner);
    let payload = json!({"keyed_cost_capture_schema": 1, "wire_journal": wire, "rows": rows, "eligible_for_review": eligible, "admitted": false, "executable_provenance": provenance, "construction_refusal": format!("{:?}", planner.refusal()), "replay_refusal": format!("{:?}", replay.as_ref().err())}).to_string();
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &payload);
    drop(guard);
    let report = format!(
        "keyed-cost-run-schema 1\nwall-seconds {:.3}\neligible-for-review {eligible}\nadmitted false\nexecutor-completed {}\nreplay-refusal {:?}\nrows {:?}\n",
        started.elapsed().as_secs_f64(),
        outcome.is_ok(),
        replay.as_ref().err(),
        cost_rows(&planner)
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
