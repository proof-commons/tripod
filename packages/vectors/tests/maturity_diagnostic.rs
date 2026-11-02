//! Diagnostic native routes have their own capture target so legacy driver censuses stay fixed.
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
use vectors::maturity_native::{MaturityNativePlanRefusal, native};

const RUN_TEST_NAME: &str = "the_diagnostic_maturity_routes_run_against_a_real_target";

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
    planner: native::DiagnosticRoutesPlanner,
    manifest: native::DiagnosticCampaignManifest,
    failure: Option<String>,
}

impl ManifestPlanner {
    fn new(identity: CandidateDeploymentIdentity, context: BranchContext) -> Self {
        Self {
            manifest: native::DiagnosticCampaignManifest::new(&identity, context),
            planner: native::DiagnosticRoutesPlanner::new(identity, context)
                .expect("diagnostic planner"),
            failure: None,
        }
    }

    fn bind_submission(&mut self, step: &OperationStep) -> Result<(), String> {
        self.manifest
            .bind_submission(step, self.planner.policy())
            .map_err(|error| error.field.to_owned())
    }
}

impl TargetOperationPlanner for ManifestPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let next = self.planner.next_step(previous)?;
        if let Some((case, response)) = previous {
            self.manifest.retain_funding(case, response);
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
    let mut out = String::from("maturity-diagnostic-evidence-schema 1\n");
    common::write_bytes_field(&mut out, "deployment-id", identity.network_id());
    common::write_bytes_field(&mut out, "genesis-id", identity.genesis_id());
    common::write_bytes_field(&mut out, "branch-identifier", context.identifier());
    writeln!(out, "branch-checkpoint {}", context.checkpoint()).expect("String write");
    if let Some(policy) = plan.planner.policy() {
        write_json(&mut out, "diagnostic-policy", &policy.0);
    }
    write_json(
        &mut out,
        "diagnostic-families",
        &json!(plan.planner.families_json()),
    );
    write_json(
        &mut out,
        "diagnostic-stopped",
        &json!(plan.planner.stopped()),
    );
    out.push_str("campaign-manifest-schema 1\ncampaign-manifest-count 23\n");
    for row in &plan.manifest.rows {
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
    for (_, response) in plan.planner.exchanges() {
        if matches!(
            response.observed_layer,
            target_elements_conformance::protocol::ObservedOutcomeLayer::ScriptPathRejection
                | target_elements_conformance::protocol::ObservedOutcomeLayer::ConsensusRejectionBeforeScript
        ) {
            // Read the vocabulary without treating a class mismatch as ineligibility.
            vectors::maturity_native::recorded_block_script_error(&response)
                .expect("recorded diagnostic vocabulary");
        }
    }
    assert!(
        plan.failure.is_none(),
        "manifest validation: {:?}",
        plan.failure
    );
    native::DiagnosticCampaignManifest::validate(&plan.planner, &plan.manifest.rows)
        .expect("manifest replay");
    let completed = plan.planner.complete();
    assert!(
        matches!(
            &completed,
            Ok(()) | Err(MaturityNativePlanRefusal::RetainedControlsRefused { .. })
        ),
        "completion: {completed:?}"
    );
    let replay = native::replay_diagnostic_routes(identity, context, &plan.planner.exchanges());
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
fn the_diagnostic_maturity_routes_run_against_a_real_target() {
    assert_eq!(
        required_environment("TRIPOD_MATURITY_CAPTURE_PROFILE"),
        "diagnostic"
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
    outcome.expect("diagnostic operations completed");
    assert_replay(&plan, identity, context);
}

#[test]
fn manifest_freezes_twenty_three_subjects_and_twenty_five_claims() {
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
    let plan = ManifestPlanner::new(identity, branch());
    assert_eq!(plan.manifest.rows.len(), 23);
    let mut count = 0;
    let mut shared = 0;
    for row in &plan.manifest.rows {
        let claims = row["claims"].as_array().expect("claims");
        count += claims.len();
        shared += usize::from(claims.len() == 2);
        assert!(row["control"].is_null());
        assert!(row["observation"].is_null());
        assert!(row["policy"].is_null());
        assert_ne!(row["subject"], "wrong-sequence");
        assert_eq!(row["expected_consumer"], "diagnostic-attribution-ledger");
    }
    assert_eq!((count, shared), (25, 2));
    assert_eq!(plan.planner.families_json().len(), 7);
}

fn historical_exchanges() -> (
    CandidateDeploymentIdentity,
    Vec<(OperationStep, NativeOperationResponse)>,
) {
    let native = vectors::maturity_corpus::maturity_native_run_of_record().expect("native");
    let retained = vectors::maturity_corpus::maturity_retained_run_of_record().expect("retained");
    (
        vectors::maturity_corpus::maturity_variable_run_of_record()
            .expect("identity archive")
            .evidence()
            .identity()
            .clone(),
        native.evidence.exchanges()[..44]
            .iter()
            .chain(&retained.evidence.exchanges()[..4])
            .cloned()
            .collect(),
    )
}

#[test]
fn both_historical_phases_replay_unchanged_and_stop_before_extra_funding() {
    let (identity, exchanges) = historical_exchanges();
    let plan =
        native::replay_diagnostic_routes(identity, branch(), &exchanges).expect("phase replay");
    assert_eq!(plan.exchanges(), exchanges);
    assert_eq!(plan.families_json().len(), 7);
    assert!(
        plan.families_json().iter().all(
            |family| family["expectation"] == "agreement" && family["control_accepted"] == true
        )
    );
    // Separate historical start arguments cannot serve as one live policy.
    assert_eq!(
        native::DiagnosticCampaignManifest::validate(&plan, &[])
            .expect_err("different historical policies")
            .field,
        "common policy"
    );
}

#[test]
fn replay_refuses_changed_requests_and_missing_or_surplus_responses() {
    let (identity, exchanges) = historical_exchanges();
    assert!(
        native::replay_diagnostic_routes(identity.clone(), branch(), &exchanges[..47]).is_err()
    );
    let mut changed = exchanges.clone();
    changed[2].0 = OperationStep::new(
        "window:successor-remains-unannounced:99",
        changed[2].0.subject().clone(),
    );
    assert!(native::replay_diagnostic_routes(identity.clone(), branch(), &changed).is_err());
    let mut extra = exchanges.clone();
    extra.push(exchanges[0].clone());
    assert!(native::replay_diagnostic_routes(identity, branch(), &extra).is_err());
}

#[test]
fn manifest_refuses_unretained_funding_before_submission() {
    let (identity, exchanges) = historical_exchanges();
    let mut manifest = native::DiagnosticCampaignManifest::new(&identity, branch());
    assert_eq!(
        manifest
            .bind_submission(&exchanges[2].0, None)
            .expect_err("unretained")
            .field,
        "submission spends an unretained outpoint"
    );
}

#[test]
fn native_phase_binds_shared_claims_and_typed_funding_compatibility() {
    let (identity, exchanges) = historical_exchanges();
    let mut plan = ManifestPlanner::new(identity, branch());
    let mut next = plan.next_step(None).expect("start");
    for (step, response) in &exchanges[..44] {
        assert_eq!(next.as_ref(), Some(step));
        next = plan
            .next_step(Some((step.case(), response)))
            .expect("native phase");
    }
    assert!(
        next.expect("retained bootstrap")
            .case()
            .step
            .starts_with("wrong-semantic-metadata:")
    );
    assert!(
        plan.manifest.rows[..22]
            .iter()
            .all(|row| !row["control"].is_null() && !row["observation"].is_null())
    );
}

#[test]
fn manifest_refuses_duplicate_claims_and_foreign_sections() {
    for duplicate in [true, false] {
        let (identity, exchanges) = historical_exchanges();
        let mut plan = ManifestPlanner::new(identity, branch());
        plan.next_step(None).expect("start");
        plan.next_step(Some((exchanges[0].0.case(), &exchanges[0].1)))
            .expect("fund");
        if duplicate {
            let claim = plan.manifest.rows[0]["claims"][0].clone();
            plan.manifest.rows[0]["claims"]
                .as_array_mut()
                .expect("claims")
                .push(claim);
        } else {
            plan.manifest.rows[0]["claims"][0]["section"] = json!("16.11");
        }
        assert!(
            plan.next_step(Some((exchanges[1].0.case(), &exchanges[1].1)))
                .is_err()
        );
        assert_eq!(plan.failure.as_deref(), Some("subject claim mapping"));
    }
}

fn scripted_response(
    step: &OperationStep,
    ordinal: u32,
    scenario: &str,
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
    let mut facts = json!({});
    match step.subject() {
        OperationSubject::Funding(subject) => {
            let asset = subject
                .asset
                .clone()
                .unwrap_or_else(|| format!("{ordinal:064x}"));
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
            let control = step.case().step.contains(":control:");
            let accept_negative = (scenario == "accepted-extra-coin"
                && step
                    .case()
                    .step
                    .starts_with("predecessor:wrong-state-asset:"))
                || scenario == "accepted-negative"
                    && step
                        .case()
                        .step
                        .starts_with("window:successor-remains-unannounced:");
            let refuse_control =
                scenario == "refused-control" && step.case().step.starts_with("window:control:");
            if (!control && !accept_negative) || refuse_control {
                response.observed_layer = if step.case().step.contains(":wrong-sequence:") {
                    ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                } else {
                    ObservedOutcomeLayer::ScriptPathRejection
                };
                facts = json!({"block_script_error":"unclassified"});
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
        _ => panic!("diagnostic schedule funds and submits"),
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
fn stopped_families_preserve_real_responses_and_continue_the_other_families() {
    for scenario in [
        "expected",
        "refused-control",
        "accepted-negative",
        "accepted-extra-coin",
    ] {
        let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
        let mut plan = ManifestPlanner::new(identity.clone(), branch());
        let mut next = plan.next_step(None).expect("start");
        let mut ordinal = 1;
        while let Some(step) = next {
            let response = scripted_response(&step, ordinal, scenario);
            next = plan
                .next_step(Some((step.case(), &response)))
                .unwrap_or_else(|_| {
                    panic!(
                        "manifest {:?}, planner {:?}",
                        plan.failure,
                        plan.planner.refusal()
                    )
                });
            ordinal += 1;
        }
        #[cfg(unix)]
        if scenario == "expected" {
            assert_scripted_capture(&plan, &identity);
        }
        assert_replay(&plan, identity, branch());
        let families = plan.planner.families_json();
        assert!(families[1..].iter().all(|family| {
            let expectation =
                if scenario == "accepted-extra-coin" && family["family"] == "predecessor" {
                    "contradiction"
                } else {
                    "agreement"
                };
            family["control_accepted"] == true && family["expectation"] == expectation
        }));
        if matches!(scenario, "expected" | "accepted-extra-coin") {
            assert_eq!(plan.planner.exchanges().len(), 48);
            assert_eq!(plan.planner.complete(), Ok(()));
        } else {
            assert_eq!(plan.planner.stopped(), vec!["window"]);
            assert_eq!(families[0]["stopped"], true);
            assert_eq!(plan.manifest.rows.len(), 23);
            assert!(plan.planner.complete().is_err());
        }
        if scenario == "accepted-negative" {
            assert_eq!(plan.planner.exchanges().len(), 46);
            assert!(plan.manifest.rows[0]["control"].is_null());
            assert!(plan.manifest.rows[1]["observation"].is_null());
        }
    }
}

#[test]
fn manifest_validation_refuses_changed_frozen_intent_and_control_bindings() {
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
    let mut plan = ManifestPlanner::new(identity, branch());
    let mut next = plan.next_step(None).expect("start");
    let mut ordinal = 1;
    while let Some(step) = next {
        let response = scripted_response(&step, ordinal, "expected");
        next = plan
            .next_step(Some((step.case(), &response)))
            .expect("step");
        ordinal += 1;
    }
    native::DiagnosticCampaignManifest::validate(&plan.planner, &plan.manifest.rows)
        .expect("valid");
    for field in [
        "family",
        "control_id",
        "expected_class",
        "expectation_reason",
        "constructor",
        "compatibility",
    ] {
        let mut changed = plan.manifest.rows.clone();
        changed[0][field] = json!("changed");
        assert!(
            native::DiagnosticCampaignManifest::validate(&plan.planner, &changed).is_err(),
            "{field}"
        );
    }
    let mut changed = plan.manifest.rows.clone();
    changed[0]["control"]["funded_outpoints"][0]["output"]["amount_satoshis"] = json!(2);
    assert!(native::DiagnosticCampaignManifest::validate(&plan.planner, &changed).is_err());
}

#[test]
fn manifest_refuses_funding_mutations_duplicate_submission_and_input_census() {
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
    let mut plan = ManifestPlanner::new(identity.clone(), branch());
    let mut next = plan.next_step(None).expect("start");
    let mut ordinal = 1;
    while let Some(step) = next {
        let response = scripted_response(&step, ordinal, "expected");
        next = plan
            .next_step(Some((step.case(), &response)))
            .expect("step");
        ordinal += 1;
    }
    for (scenario, target, expected) in [
        (
            "amount-plus-two",
            "wrong-singleton-amount",
            "control compatibility",
        ),
        (
            "asset-and-amount",
            "wrong-state-asset",
            "control compatibility",
        ),
        (
            "fresh-outpoint",
            "successor-remains-unannounced",
            "control outpoint relationship",
        ),
        (
            "duplicate",
            "successor-remains-unannounced",
            "duplicate submission",
        ),
        (
            "two-inputs",
            "successor-remains-unannounced",
            "funded input census",
        ),
    ] {
        let mut manifest = native::DiagnosticCampaignManifest::new(&identity, branch());
        let mut refusal = None;
        for (step, response) in plan.planner.exchanges() {
            let changed = if step.case().step.split(':').nth(1) == Some(target) {
                mutate_manifest_submission(&mut manifest, &step, scenario)
            } else {
                step.clone()
            };
            let result = manifest.bind_submission(&changed, plan.planner.policy());
            let result = if result.is_ok()
                && scenario == "duplicate"
                && step.case().step.split(':').nth(1) == Some(target)
            {
                manifest.bind_submission(&changed, plan.planner.policy())
            } else {
                result
            };
            if let Err(error) = result {
                refusal = Some(error.field);
                break;
            }
            manifest.retain_funding(step.case(), &response);
        }
        assert_eq!(refusal, Some(expected), "{scenario}");
    }
}

fn mutate_manifest_submission(
    manifest: &mut native::DiagnosticCampaignManifest,
    step: &OperationStep,
    scenario: &str,
) -> OperationStep {
    use transaction::bytes::{Outpoint, TargetInput, Txid};
    let OperationSubject::Submission(mut subject) = step.subject().clone() else {
        panic!("submission subject");
    };
    let transaction = TargetTransaction::decode(&subject.transaction_bytes).expect("transaction");
    let point = transaction.inputs()[0].outpoint();
    let (_, coin) = manifest
        .funding
        .iter_mut()
        .find(|(_, coin)| {
            coin.outpoint.txid == point.txid().to_target_display()
                && coin.outpoint.vout == point.index()
        })
        .expect("retained coin");
    let mut inputs = transaction.inputs().to_vec();
    let mut witnesses = transaction.witnesses().to_vec();
    match scenario {
        "amount-plus-two" | "asset-and-amount" => coin.amount_satoshis += 1,
        "fresh-outpoint" => {
            let mut fresh = coin.clone();
            fresh.outpoint.vout = 1000;
            inputs[0] = TargetInput::new(
                Outpoint::new(
                    Txid::from_target_display(&fresh.outpoint.txid).expect("txid"),
                    1000,
                )
                .expect("outpoint"),
                inputs[0].sequence(),
            );
            manifest.funding.push(("fresh-funding".into(), fresh));
        }
        "two-inputs" => {
            inputs.push(inputs[0].clone());
            witnesses.push(witnesses[0].clone());
        }
        "duplicate" => {}
        _ => panic!("known scenario"),
    }
    subject.transaction_bytes = TargetTransaction::new(
        transaction.version(),
        inputs,
        transaction.outputs().to_vec(),
        transaction.lock_time(),
        witnesses,
    )
    .expect("changed transaction")
    .encode();
    OperationStep::new(&step.case().step, OperationSubject::Submission(subject))
}

#[test]
fn diagnostic_planner_refuses_policy_change_mid_run() {
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity");
    let mut plan = native::DiagnosticRoutesPlanner::new(identity, branch()).expect("plan");
    let first = plan.next_step(None).expect("start").expect("funding");
    let response = scripted_response(&first, 1, "expected");
    let second = plan
        .next_step(Some((first.case(), &response)))
        .expect("first funding")
        .expect("next");
    let mut changed = scripted_response(&second, 2, "expected");
    let mut detail: Value =
        serde_json::from_str(changed.observed_detail.as_deref().expect("detail")).expect("JSON");
    detail["policy"]["start_arguments"]
        .as_array_mut()
        .expect("arguments")
        .push(json!("-diagnostic-policy-change"));
    changed.observed_detail = Some(detail.to_string());
    assert!(plan.next_step(Some((second.case(), &changed))).is_err());
    assert_eq!(
        plan.refusal(),
        Some(&MaturityNativePlanRefusal::SponsoredPolicy)
    );
}

#[cfg(unix)]
fn assert_scripted_capture(scripted: &ManifestPlanner, identity: &CandidateDeploymentIdentity) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = common::test_directory("diagnostic-capture");
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
            "diagnostic-manifest",
            Some(common::sha256(
                &serde_json::to_vec(&plan.manifest.rows).expect("rows"),
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
        .expect("capture diagnostic before gates");
    outcome.expect("scripted execution");
    assert_replay(&plan, identity.clone(), branch());
    assert_eq!(plan.manifest.rows, scripted.manifest.rows);
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
