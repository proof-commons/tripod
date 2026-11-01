//! Scripted confidential route contracts and the separately selected native capture.

pub mod common;
use linker::CandidateDeploymentIdentity;
use serde_json::{Value, json};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::OnceLock;
use target_elements_conformance::executor::{
    ExecutorTrust, NativeOperationCapture, OperationStep, PlanRefused, TargetOperationPlanner,
};
use target_elements_conformance::protocol::{
    ExecutorCapability, FundedOutput, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, WireOutpoint,
};
use transaction::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, OutputWitness, TargetInput,
    TargetOutput, TargetTransaction, Txid, ValueField,
};
use transaction::operator_right::BranchContext;
use vectors::maturity_native::confidential::{FAMILY, Planner, STEPS, replay_confidential_route};

const RUN_TEST_NAME: &str = "the_confidential_maturity_route_runs_against_a_real_target";
fn branch() -> BranchContext {
    BranchContext::new([0x41; 32], 7).expect("branch")
}
fn identity() -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity")
}
fn required(name: &str) -> String {
    common::environment(name).unwrap_or_else(|| panic!("{name} required"))
}
fn write_json(out: &mut String, name: &str, value: &Value) {
    common::write_bytes_field(out, name, &serde_json::to_vec(value).expect("JSON"));
}
fn payload(plan: &Planner, identity: &CandidateDeploymentIdentity, preflight: &Value) -> String {
    let mut out = String::from("maturity-confidential-evidence-schema 1\n");
    common::write_bytes_field(&mut out, "deployment-id", identity.network_id());
    common::write_bytes_field(&mut out, "genesis-id", identity.genesis_id());
    common::write_bytes_field(&mut out, "branch-identifier", branch().identifier());
    writeln!(out, "branch-checkpoint {}", branch().checkpoint()).expect("String");
    if let Some(policy) = plan.policy() {
        write_json(&mut out, "confidential-policy", &policy.0);
    }
    write_json(&mut out, "confidential-families", &plan.families_json());
    write_json(
        &mut out,
        "confidential-stopped",
        &if plan.families_json()[0]["stopped"] == true {
            json!([FAMILY])
        } else {
            json!([])
        },
    );
    write_json(&mut out, "confidential-preflight", preflight);
    out.push_str("campaign-manifest-schema 1\ncampaign-manifest-count 1\n");
    for row in &plan.manifest.rows {
        write_json(&mut out, "campaign-row", row);
    }
    for (step, response) in plan.exchanges() {
        write_json(
            &mut out,
            "exchange",
            &json!((step.case(), step.subject(), response)),
        );
    }
    if let Some(refusal) = plan.refusal() {
        common::write_bytes_field(&mut out, "refusal", refusal.field.as_bytes());
    }
    out
}
struct Probe;
impl TargetOperationPlanner for Probe {
    fn next_step(
        &mut self,
        _: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        Ok(None)
    }
}
fn handshake_reading(capture: &NativeOperationCapture) -> Value {
    let profiles = vectors::confidential_predecessor::selected_profiles();
    let handshake = capture.handshake();
    let funding = handshake.is_some_and(|h| {
        h.capabilities
            .contains(&ExecutorCapability::ConfidentialValueTestFunding)
    });
    let signing = handshake.is_some_and(|h| {
        h.capabilities
            .contains(&ExecutorCapability::ConfidentialValueSponsorAuthorization)
    });
    let matched = handshake
        .and_then(|h| h.confidential_funding.as_ref())
        .is_some_and(|advertisement| {
            target_elements_conformance::confidential_funding::check_selected_profiles(
                advertisement,
                &profiles,
            )
            .is_ok()
        });
    json!({"confidential_value_test_funding":funding,"confidential_value_sponsor_authorization":signing,"profiles":profiles,"advertisement_matches":matched})
}
fn reading_valid(reading: &Value) -> bool {
    [
        "confidential_value_test_funding",
        "confidential_value_sponsor_authorization",
        "advertisement_matches",
    ]
    .iter()
    .all(|key| reading[key] == true)
}

fn execute_family(
    adapter: &Path,
    directory: &Path,
    identity: &CandidateDeploymentIdentity,
    trust: ExecutorTrust,
    plan: &mut Planner,
    capture: &mut NativeOperationCapture,
) -> Result<
    target_elements_conformance::executor::ExecutionTranscript,
    target_elements_conformance::error::NativeConformanceError,
> {
    // A failed second handshake must not inherit the successful probe's reading.
    *capture = NativeOperationCapture::default();
    common::execute(adapter, directory, identity, trust, plan, capture)
}

#[test]
#[ignore = "requires the native adapter and confidential capture environment"]
fn the_confidential_maturity_route_runs_against_a_real_target() {
    let executor = required("TRIPOD_LIVE_EXECUTOR");
    let directory = required("TRIPOD_LIVE_REPORT_DIR");
    let network = required("TRIPOD_LIVE_NETWORK_ID");
    let genesis = required("TRIPOD_LIVE_GENESIS_ID");
    common::require_lower_hex(&network, 64, "network").expect("network");
    common::require_lower_hex(&genesis, 64, "genesis").expect("genesis");
    let identity = CandidateDeploymentIdentity::new(
        common::identifier(&network),
        common::identifier(&genesis),
    )
    .expect("identity");
    let mut destination = common::CaptureDestination::new(
        directory.clone().into(),
        required("TRIPOD_LIVE_SUITE_SHORT_SHA"),
        required("TRIPOD_LIVE_SUITE_COMMIT"),
        required("TRIPOD_LIVE_SUITE_TREE"),
    )
    .expect("provenance");
    destination.report_test_name = RUN_TEST_NAME;
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    let mut plan = Planner::new(identity.clone(), branch()).expect("typed route");
    let mut capture = NativeOperationCapture::default();
    let library = common::environment("ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY");
    let bytes = library.as_ref().and_then(|path| std::fs::read(path).ok());
    let configured = library.as_ref().is_some_and(|path| !path.is_empty());
    let library_digest = bytes
        .as_ref()
        .map(|bytes| common::hex_bytes(&common::sha256(bytes)));
    let probe = common::execute(
        Path::new(&executor),
        Path::new(&directory),
        &identity,
        ExecutorTrust::ReviewedNonMock,
        &mut Probe,
        &mut capture,
    );
    let probe_reading = handshake_reading(&capture);
    let preconditions = configured
        && bytes.is_some()
        && reading_valid(&probe_reading)
        && common::environment("TRIPOD_MATURITY_CAPTURE_PROFILE").as_deref()
            == Some("confidential");
    let outcome = if probe.is_ok() && preconditions {
        execute_family(
            Path::new(&executor),
            Path::new(&directory),
            &identity,
            ExecutorTrust::ReviewedNonMock,
            &mut plan,
            &mut capture,
        )
    } else {
        probe
    };
    let real_reading = handshake_reading(&capture);
    let preflight = json!({"library_configured":configured,"library_readable":bytes.is_some(),"library_sha256":library_digest,"probe":probe_reading,"capture":real_reading});
    let evidence = payload(&plan, &identity, &preflight);
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    common::write_capture_before_gates(&mut guard, &capture, &facts, &evidence);
    assert!(preconditions, "confidential preflight");
    assert_eq!(
        preflight["probe"], preflight["capture"],
        "handshake changed"
    );
    outcome.expect("capture operations");
    vectors::maturity_native::confidential::ConfidentialCampaignFields {
        policy: plan.policy().expect("policy").0.clone(),
        families: plan.families_json(),
        stopped: if plan.families_json()[0]["stopped"] == true {
            json!([FAMILY])
        } else {
            json!([])
        },
        preflight,
        rows: plan.manifest.rows.clone(),
    }
    .validate(&plan)
    .expect("reported campaign records");
    let replay =
        replay_confidential_route(identity, branch(), plan.exchanges()).expect("exact replay");
    assert_eq!(replay.manifest.rows, plan.manifest.rows);
    assert!(plan.eligible(), "accepted-control boundary");
}

fn policy() -> Value {
    vectors::maturity_corpus::maturity_native_run_of_record()
        .expect("archive")
        .evidence
        .policy()
        .expect("policy")
        .0
        .clone()
}
fn sponsor_program() -> Vec<u8> {
    // The published explicit sponsor witness supplies the public signing identity.
    let seed =
        vectors::maturity_corpus::maturity_sponsored_run_of_record().expect("sponsored archive");
    seed.evidence
        .exchanges()
        .iter()
        .find_map(|(_, r)| {
            r.funded_outputs
                .iter()
                .find(|coin| coin.amount_satoshis == 1_000_000)
                .map(|coin| {
                    coin.script
                        .as_bytes()
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|pair| {
                            u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16)
                                .expect("hex")
                        })
                        .collect()
                })
        })
        .expect("sponsor program")
}
fn readback(raw: Vec<u8>) -> target_elements_conformance::protocol::MinedFundingReadback {
    let tx = TargetTransaction::decode(&raw).expect("transaction");
    target_elements_conformance::protocol::MinedFundingReadback {
        transaction_id: Txid::from_internal(common::sha256(&common::sha256(
            &tx.encode_without_witness(),
        )))
        .to_target_display(),
        witness_transaction_id: Txid::from_internal(common::sha256(&common::sha256(&raw)))
            .to_target_display(),
        block_hash: "77".repeat(32),
        block_height: 10,
        raw_transaction: raw,
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "The scripted responder mirrors the fixed operation vocabulary in one match."
)]
fn scripted_response(step: &OperationStep, n: usize) -> NativeOperationResponse {
    let seed = vectors::maturity_corpus::maturity_variable_run_of_record().expect("archive");
    let mut response = seed.exchanges()[0].1.clone();
    response.case = step.case().clone();
    response.issued_asset = None;
    response.funded_outputs.clear();
    response.confidential_funded_outputs.clear();
    response.mined_readback = None;
    response.accepted_txid = None;
    response.sponsor_witness.clear();
    response.signature_bound_to = None;
    response.observed_layer = ObservedOutcomeLayer::Accepted;
    let mut route = "setup";
    let mut facts = Value::Null;
    match step.subject() {
        OperationSubject::Funding(subject) => {
            let asset = "55".repeat(32);
            if subject.issue_asset {
                response.issued_asset = Some(asset.clone());
            }
            response.funded_outputs.push(FundedOutput {
                outpoint: WireOutpoint {
                    txid: "66".repeat(32),
                    vout: u32::try_from(n).expect("ordinal"),
                },
                asset,
                amount_satoshis: subject.amount_per_output,
                script: common::hex_bytes(&subject.output_program),
            });
        }
        OperationSubject::SponsorFunding(subject) => response.funded_outputs.push(FundedOutput {
            outpoint: WireOutpoint {
                txid: "88".repeat(32),
                vout: u32::try_from(n).expect("ordinal"),
            },
            asset: "99".repeat(32),
            amount_satoshis: subject.amount_per_sponsor_output,
            script: common::hex_bytes(&sponsor_program()),
        }),
        OperationSubject::SponsorSigning(subject) => {
            let seed = vectors::maturity_corpus::maturity_sponsored_run_of_record()
                .expect("sponsor archive");
            response.sponsor_witness = seed
                .evidence
                .exchanges()
                .iter()
                .find_map(|(_, r)| {
                    (!r.sponsor_witness.is_empty()).then(|| r.sponsor_witness.clone())
                })
                .expect("witness");
            response.signature_bound_to = Some(subject.finalized_transaction.clone());
            response.resources.transaction_weight = Some(
                TargetTransaction::decode(&subject.finalized_transaction)
                    .expect("signed candidate")
                    .weight(),
            );
        }
        OperationSubject::ConfidentialSponsorFunding(subject) => {
            use target_elements_conformance::confidential_fixture::FixtureOpenings;
            use target_elements_conformance::confidential_oracles::ReferenceProofMaterial;
            use vectors::confidential_sponsor_reserve::frozen_sponsor_reserve_registry;
            let program = sponsor_program();
            let registry = frozen_sponsor_reserve_registry([0x99; 32], &program).expect("registry");
            let resolved = registry
                .resolve(
                    &subject.binding.fixture_handle,
                    &subject.binding.fixture_digest,
                )
                .expect("fixture binding");
            let FixtureOpenings::Derived { openings, .. } = resolved.openings() else {
                panic!("derived")
            };
            let material = ReferenceProofMaterial::new();
            let mut outputs = Vec::new();
            let mut witnesses = Vec::new();
            for opening in openings {
                let opening = opening.as_ref().expect("opening");
                outputs.push(TargetOutput::new(
                    AssetField::Explicit(AssetId::from_internal([0x99; 32])),
                    ValueField::Commitment(opening.value_commitment),
                    NonceField::Commitment(
                        material
                            .nonce_commitment(&opening.nonce_input)
                            .expect("nonce"),
                    ),
                    program.clone(),
                ));
                witnesses.push(OutputWitness::new(
                    Vec::new(),
                    material
                        .range_proof(
                            &[0x99; 32],
                            1250,
                            &opening.value_blinder,
                            &opening.rangeproof_seed,
                            &opening.value_commitment,
                            &program,
                        )
                        .expect("proof"),
                ));
            }
            let tx = TargetTransaction::with_output_witnesses(
                2,
                vec![TargetInput::new(
                    Outpoint::new(Txid::from_internal([0xaa; 32]), 0).expect("outpoint"),
                    u32::MAX,
                )],
                outputs,
                0,
                vec![InputWitness::new(Vec::new())],
                witnesses,
            )
            .expect("funding");
            let mined = readback(tx.encode());
            for (index, (output, witness)) in
                tx.outputs().iter().zip(tx.output_witnesses()).enumerate()
            {
                let ValueField::Commitment(commitment) = output.value() else {
                    panic!("commitment")
                };
                let NonceField::Commitment(nonce) = output.nonce() else {
                    panic!("nonce")
                };
                let index = u32::try_from(index).expect("index");
                response.confidential_funded_outputs.push(
                    target_elements_conformance::protocol::ConfidentialFundedOutput {
                        outpoint: WireOutpoint {
                            txid: mined.transaction_id.clone(),
                            vout: index,
                        },
                        explicit_asset: "99".repeat(32),
                        value_commitment: commitment.to_vec(),
                        nonce: nonce.to_vec(),
                        script: common::hex_bytes(&program),
                        output_witness_index: index,
                        surjection_proof: witness.surjection_proof().to_vec(),
                        rangeproof: witness.range_proof().to_vec(),
                    },
                );
            }
            response.mined_readback = Some(mined);
        }
        OperationSubject::Submission(subject) => {
            let mined = readback(subject.transaction_bytes.clone());
            facts = json!({"txid":mined.transaction_id,"included_block":mined.block_hash,"produced_block":mined.block_hash,"inclusion_in_produced":true,"load_five_minutes":0,"mempool_size":0,"mempool_minimum":"0.00000100","template":{"member":true,"weight":TargetTransaction::decode(&subject.transaction_bytes).expect("tx").weight(),"txids":[mined.transaction_id],"source":"assembly-read"}});
            response.accepted_txid = Some(mined.transaction_id.clone());
            response.mined_readback = Some(mined);
            route = "relay-and-template";
        }
        _ => panic!("route operation"),
    }
    response.observed_detail=Some(json!({"schema":1,"policy":policy(),"route":route,"relay_allowed":if route=="setup" {Value::Null}else{json!(true)},"relay_detail":null,"detail":facts}).to_string());
    response.validate_shape().expect("response shape");
    response
}
fn exchanges() -> &'static [(OperationStep, NativeOperationResponse)] {
    static EXCHANGES: OnceLock<Vec<(OperationStep, NativeOperationResponse)>> = OnceLock::new();
    EXCHANGES.get_or_init(|| {
        let mut plan = Planner::new(identity(), branch()).expect("planner");
        let mut next = plan.next_step(None).expect("first");
        let mut n = 0;
        while let Some(step) = next {
            let response = scripted_response(&step, n);
            next = plan
                .next_step(Some((step.case(), &response)))
                .unwrap_or_else(|_| panic!("step {n}: {:?}", plan.refusal()));
            n += 1;
        }
        assert_eq!(n, 14);
        plan.exchanges().to_vec()
    })
}
fn replay(exchanges: &[(OperationStep, NativeOperationResponse)]) -> Planner {
    replay_confidential_route(identity(), branch(), exchanges).expect("replay")
}
fn refuse(response: &mut NativeOperationResponse) {
    response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
    response.accepted_txid = None;
    response.mined_readback = None;
    let mut detail: Value =
        serde_json::from_str(response.observed_detail.as_ref().expect("detail")).expect("JSON");
    detail["route"] = json!("refused");
    detail["relay_allowed"] = json!(false);
    detail["relay_detail"] = json!("script refusal");
    detail["detail"] = Value::Null;
    response.observed_detail = Some(detail.to_string());
}
#[test]
fn confidential_agreement_requires_control_and_subject_readback() {
    let plan = replay(exchanges());
    #[cfg(unix)]
    assert_scripted_capture(&plan);
    assert_eq!(plan.label(), "agreement");
    assert!(plan.eligible());
    assert!(plan.admit().is_ok());
    assert_eq!(
        plan.exchanges()
            .iter()
            .map(|(s, _)| s.case().step.split(':').nth(1).expect("step"))
            .collect::<Vec<_>>(),
        STEPS
    );
}
#[test]
fn confidential_refused_subject_records_contradiction() {
    let mut ex = exchanges().to_vec();
    refuse(&mut ex[13].1);
    let plan = replay(&ex);
    assert_eq!(plan.label(), "contradiction");
    assert!(plan.eligible());
    assert!(plan.admit().is_err());
}
#[test]
fn confidential_direct_block_subject_records_contradiction() {
    let mut ex = exchanges().to_vec();
    let response = &mut ex[13].1;
    let readback = response.mined_readback.clone();
    let mut detail: Value =
        serde_json::from_str(response.observed_detail.as_ref().expect("detail")).expect("JSON");
    detail["route"] = json!("direct-block");
    detail["relay_allowed"] = json!(false);
    detail["relay_detail"] = json!("relay refusal");
    response.observed_detail = Some(detail.to_string());
    let plan = replay(&ex);
    assert_eq!(plan.label(), "contradiction");
    assert!(plan.eligible());
    assert!(plan.admit().is_err());
    assert_eq!(plan.families_json()[0]["control_accepted"], true);
    assert_eq!(plan.families_json()[0]["stopped"], false);
    assert_eq!(
        plan.manifest.rows[0]["observation"]["readback"],
        json!(readback)
    );
}

#[test]
fn confidential_campaign_projection_refuses_a_changed_closed_field() {
    let plan = replay(exchanges());
    let mut rows = plan.manifest.rows.clone();
    vectors::maturity_native::confidential::CampaignManifest::validate(&plan, &rows)
        .expect("valid projection");
    rows[0]["constructor"] = json!("changed");
    assert_eq!(
        vectors::maturity_native::confidential::CampaignManifest::validate(&plan, &rows)
            .expect_err("changed projection")
            .field,
        "campaign projection"
    );
}
#[test]
fn confidential_refused_control_records_stopped_without_subject() {
    let mut ex = exchanges()[..8].to_vec();
    refuse(&mut ex[7].1);
    let plan = replay(&ex);
    assert_eq!(plan.label(), "stopped");
    assert!(!plan.eligible());
    assert_eq!(plan.exchanges().len(), 8);
}
#[test]
fn confidential_readback_mismatch_is_ineligible() {
    for failed_step in [7, 13] {
        let mut plan = Planner::new(identity(), branch()).expect("planner");
        let mut next = plan.next_step(None).expect("first");
        for (n, (step, response)) in exchanges()[..=failed_step].iter().enumerate() {
            assert_eq!(next.as_ref(), Some(step));
            let mut response = response.clone();
            if n == failed_step {
                response
                    .mined_readback
                    .as_mut()
                    .expect("readback")
                    .raw_transaction
                    .push(0);
            }
            next = plan
                .next_step(Some((step.case(), &response)))
                .unwrap_or_default();
        }
        assert!(!plan.eligible());
        assert_eq!(plan.refusal().expect("refusal").field, "readback mismatch");
        assert_eq!(plan.exchanges().len(), failed_step + 1);
        let family = &plan.families_json()[0];
        assert_eq!(family["stopped"], failed_step == 7);
        assert_eq!(family["control_accepted"], failed_step == 13);
        assert_eq!(family["refusal"], "readback mismatch");
        if failed_step == 13 {
            assert_eq!(family["expectation"], "pending");
            let record = payload(&plan, &identity(), &json!({}));
            assert!(record.contains("confidential-stopped 2 5b5d\n"));
        }
    }
}
#[test]
fn confidential_direct_block_control_stops_the_family() {
    let mut ex = exchanges()[..8].to_vec();
    let response = &mut ex[7].1;
    let mut detail: Value =
        serde_json::from_str(response.observed_detail.as_ref().expect("detail")).expect("JSON");
    detail["route"] = json!("direct-block");
    detail["relay_allowed"] = json!(false);
    detail["relay_detail"] = json!("relay refusal");
    response.observed_detail = Some(detail.to_string());
    let plan = replay(&ex);
    assert_eq!(plan.label(), "stopped");
    assert!(!plan.eligible());
}
#[test]
#[should_panic(expected = "accepted-control boundary")]
fn confidential_acceptance_boundary_panics_on_refused_control() {
    let mut ex = exchanges()[..8].to_vec();
    refuse(&mut ex[7].1);
    assert!(replay(&ex).eligible(), "accepted-control boundary");
}
#[test]
fn confidential_replay_refuses_changed_requests_funding_and_controls() {
    for index in 0..14 {
        let mut ex = exchanges().to_vec();
        ex[index].0 = OperationStep::new("changed", ex[index].0.subject().clone());
        assert!(replay_confidential_route(identity(), branch(), &ex).is_err());
    }
    for index in [0, 1, 2, 5, 8] {
        let mut ex = exchanges().to_vec();
        ex[index].1.funded_outputs[0].outpoint.vout += 1;
        assert!(replay_confidential_route(identity(), branch(), &ex).is_err());
    }
    for field in [
        "value_commitment",
        "nonce",
        "rangeproof",
        "surjection_proof",
        "explicit_asset",
        "script",
        "output_witness_index",
        "outpoint",
    ] {
        let mut ex = exchanges().to_vec();
        let mut coin = json!(ex[9].1.confidential_funded_outputs[0]);
        coin[field] = match field {
            "explicit_asset" | "script" => json!("00"),
            "output_witness_index" => json!(1),
            "outpoint" => json!({"txid":"00".repeat(32),"vout":0}),
            _ => json!([0]),
        };
        ex[9].1.confidential_funded_outputs[0] = serde_json::from_value(coin).expect("coin");
        assert!(
            replay_confidential_route(identity(), branch(), &ex).is_err(),
            "{field}"
        );
    }
    let mut ex = exchanges().to_vec();
    refuse(&mut ex[7].1);
    assert!(replay_confidential_route(identity(), branch(), &ex).is_err());
    assert!(replay_confidential_route(identity(), branch(), &exchanges()[..13]).is_err());
    let mut ex = exchanges().to_vec();
    ex.swap(7, 8);
    assert!(replay_confidential_route(identity(), branch(), &ex).is_err());
}
#[test]
fn confidential_payload_excludes_construction_material() {
    let plan = replay(exchanges());
    let reading = handshake_reading(&NativeOperationCapture::default());
    let preflight = json!({"library_configured":true,"library_readable":true,"library_sha256":"public-digest","probe":reading,"capture":reading});
    let text = payload(&plan, &identity(), &preflight);
    let fields = decoded_evidence_fields(&text);
    let rows = serde_json::to_string(&plan.manifest.rows).expect("rows");
    let exchanges = serde_json::to_string(
        &plan
            .exchanges()
            .iter()
            .map(|(step, response)| json!((step.case(), step.subject(), response)))
            .collect::<Vec<_>>(),
    )
    .expect("public exchanges");
    for name in [
        "value_blinder",
        "input_blinder_sum",
        "nonce_input",
        "rangeproof_seed",
        "openings",
    ] {
        assert!(!text.contains(name));
        assert!(!rows.contains(name));
        assert!(!exchanges.contains(name));
        assert!(fields.iter().all(|field| !field.to_string().contains(name)));
    }
    let registry = vectors::confidential_sponsor_reserve::frozen_sponsor_reserve_registry(
        [0x99; 32],
        &sponsor_program(),
    )
    .expect("registry");
    let handle = target_elements_conformance::confidential_fixture::sponsor_reserve_handle();
    let resolved = registry
        .resolve(
            &handle,
            registry.registered_digest(&handle).expect("digest"),
        )
        .expect("resolved");
    if let target_elements_conformance::confidential_fixture::FixtureOpenings::Derived {
        openings,
        ..
    } = resolved.openings()
    {
        for opening in openings.iter().flatten() {
            for material in [
                &opening.value_blinder,
                &opening.nonce_input,
                &opening.rangeproof_seed,
            ] {
                assert!(
                    fields
                        .iter()
                        .all(|field| !contains_material(field, material))
                );
                for public in [&rows, &exchanges] {
                    let public: Value = serde_json::from_str(public).expect("public JSON");
                    assert!(!contains_material(&public, material));
                }
            }
        }
        let opening = openings[0].as_ref().expect("consumed opening");
        for (domain, input) in [
            ("STATE-change/nonce", &opening.nonce_input),
            ("STATE-change/rangeproof", &opening.rangeproof_seed),
        ] {
            let mut input = input.to_vec();
            input.extend(1_u16.to_be_bytes());
            let material = transaction::taproot::tagged_hash(domain, &input);
            assert!(
                fields
                    .iter()
                    .all(|field| !contains_material(field, &material))
            );
            for public in [&rows, &exchanges] {
                let public: Value = serde_json::from_str(public).expect("public JSON");
                assert!(!contains_material(&public, &material));
            }
        }
    }
}
fn decoded_evidence_fields(text: &str) -> Vec<Value> {
    text.lines()
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() != 3 {
                return None;
            }
            let width: usize = parts[1].parse().expect("encoded width");
            let (pairs, remainder) = parts[2].as_bytes().as_chunks::<2>();
            assert_eq!(remainder, b"");
            let bytes = pairs
                .iter()
                .map(|pair| {
                    u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16)
                        .expect("hex byte")
                })
                .collect::<Vec<_>>();
            assert_eq!(bytes.len(), width);
            Some(serde_json::from_slice(&bytes).unwrap_or_else(|_| {
                String::from_utf8(bytes.clone()).map_or_else(|_| json!(bytes), Value::String)
            }))
        })
        .collect()
}
fn contains_material(public: &Value, material: &[u8; 32]) -> bool {
    match public {
        Value::String(value) => value.as_bytes().windows(64).any(|window| {
            window
                .as_chunks::<2>()
                .0
                .iter()
                .zip(material)
                .all(|(pair, expected)| {
                    let high = (pair[0] as char).to_digit(16);
                    let low = (pair[1] as char).to_digit(16);
                    high.zip(low)
                        .is_some_and(|(high, low)| high * 16 + low == u32::from(*expected))
                })
        }),
        Value::Array(values) => {
            values.windows(32).any(|window| {
                window
                    .iter()
                    .zip(material)
                    .all(|(value, expected)| value.as_u64() == Some(u64::from(*expected)))
            }) || values
                .iter()
                .any(|value| contains_material(value, material))
        }
        Value::Object(values) => values
            .values()
            .any(|value| contains_material(value, material)),
        _ => false,
    }
}
#[test]
fn confidential_final_weight_outgrowing_the_measured_fee_refuses() {
    let mut plan = Planner::new(identity(), branch()).expect("planner");
    let mut next = plan.next_step(None).expect("first");
    for (n, (step, response)) in exchanges()[..=12].iter().enumerate() {
        assert_eq!(next.as_ref(), Some(step));
        let mut response = response.clone();
        if n == 12 {
            // Only the final witness grows; its binding and sighash byte stay fixed.
            response.sponsor_witness[0].splice(0..0, vec![0; 6000]);
            assert!(plan.next_step(Some((step.case(), &response))).is_err());
            assert_eq!(plan.refusal().expect("final fit refusal").field, "fee fit");
            assert_eq!(plan.exchanges().len(), 13);
            assert!(!plan.eligible());
            assert_eq!(plan.families_json()[0]["stopped"], false);
            assert!(plan.admit().is_err());
        } else {
            next = plan
                .next_step(Some((step.case(), &response)))
                .expect("template and control qualify");
        }
    }
}

#[test]
fn confidential_fee_requires_positive_change_and_final_weight_fit() {
    let plan = replay(exchanges());
    for slot in ["control", "observation"] {
        let row = &plan.manifest.rows[0][slot];
        let fee = row["fee"].as_u64().expect("fee");
        assert_eq!(fee + row["change"].as_u64().expect("change"), 1250);
        assert!(row["change"].as_u64().expect("change") > 0);
        assert!(
            plan.policy()
                .expect("policy")
                .fee(row["weight"].as_u64().expect("weight"))
                .expect("fee")
                <= fee
        );
    }
    let mut ex = exchanges().to_vec();
    ex[4].1.resources.transaction_weight = Some(u64::MAX);
    assert!(replay_confidential_route(identity(), branch(), &ex).is_err());
    let mut plan = Planner::new(identity(), branch()).expect("planner");
    let mut next = plan.next_step(None).expect("first");
    for (step, response) in &exchanges()[..5] {
        assert_eq!(next.as_ref(), Some(step));
        let mut response = response.clone();
        let mut detail: Value =
            serde_json::from_str(response.observed_detail.as_ref().expect("detail")).expect("JSON");
        detail["policy"]["relay"] = json!("1.00000000");
        response.observed_detail = Some(detail.to_string());
        next = plan
            .next_step(Some((step.case(), &response)))
            .unwrap_or_default();
    }
    assert_eq!(plan.refusal().expect("fee refusal").field, "fee fit");
    assert!(!plan.eligible());
}

fn manifest_fixture() -> (
    vectors::maturity_native::confidential::CampaignManifest,
    OperationStep,
    vectors::maturity_native::sponsored::Policy,
) {
    static MANIFEST: OnceLock<vectors::maturity_native::confidential::CampaignManifest> =
        OnceLock::new();
    let mut manifest = MANIFEST
        .get_or_init(|| replay(exchanges()).manifest)
        .clone();
    manifest.rows[0]["observation"] = Value::Null;
    (
        manifest,
        exchanges()[13].0.clone(),
        vectors::maturity_native::sponsored::Policy::parse(policy()).expect("policy"),
    )
}
#[expect(
    clippy::too_many_lines,
    reason = "Each named guard starts with the same valid campaign and changes one coordinate."
)]
fn manifest_guard(case: &str) {
    let (mut manifest, mut step, policy) = manifest_fixture();
    assert!(
        manifest
            .clone()
            .bind_submission(&step, Some(&policy))
            .is_ok(),
        "valid starting state"
    );
    let mut supplied_policy = Some(&policy);
    let expected = match case {
        "bad_step_shape" => {
            step = OperationStep::new("bad", step.subject().clone());
            "step shape"
        }
        "unknown_family" => {
            step = OperationStep::new("unknown:subject:13", step.subject().clone());
            "unknown family"
        }
        "wrong_row_census" => {
            manifest.rows.clear();
            "row census"
        }
        "changed_control_identity" => {
            step = exchanges()[7].0.clone();
            manifest.rows[0]["control"] = Value::Null;
            manifest.rows[0]["control_id"] = json!("changed");
            "control identity"
        }
        "changed_subject_claim_mapping" => {
            manifest.rows[0]["claim"]["name"] = json!("changed");
            "subject claim mapping"
        }
        "invalid_transaction_encoding" => {
            step = OperationStep::new(
                &step.case().step,
                OperationSubject::Submission(Box::new(
                    target_elements_conformance::protocol::TargetSubmissionSubject {
                        transaction_bytes: vec![0],
                    },
                )),
            );
            "transaction encoding"
        }
        "unretained_funding" => {
            manifest.funding.clear();
            "unretained funding"
        }
        "changed_input_census" => {
            let OperationSubject::Submission(subject) = step.subject() else {
                panic!("submission")
            };
            let tx = TargetTransaction::decode(&subject.transaction_bytes).expect("tx");
            let tx = TargetTransaction::with_output_witnesses(
                tx.version(),
                tx.inputs()[..1].to_vec(),
                tx.outputs().to_vec(),
                tx.lock_time(),
                tx.witnesses()[..1].to_vec(),
                tx.output_witnesses().to_vec(),
            )
            .expect("one input");
            step = OperationStep::new(
                &step.case().step,
                OperationSubject::Submission(Box::new(
                    target_elements_conformance::protocol::TargetSubmissionSubject {
                        transaction_bytes: tx.encode(),
                    },
                )),
            );
            "input census"
        }
        "missing_or_changed_policy" => {
            supplied_policy = None;
            "policy"
        }
        "changed_frozen_policy" => {
            manifest.rows[0]["policy"]["relay"] = json!("0.00000200");
            "policy"
        }
        "duplicate_submission" => {
            manifest.rows[0]["observation"] = json!({});
            "duplicate submission"
        }
        "incompatible_control" => {
            manifest.rows[0]["control"]["funded_outpoints"][0]["output"]["amount_satoshis"] =
                json!(2);
            "incompatible control"
        }
        "reused_control_outpoints" => {
            let subject = replay(exchanges()).manifest.rows[0]["observation"].clone();
            manifest.rows[0]["control"]["funded_outpoints"][0]["output"]["outpoint"] =
                subject["funded_outpoints"][0]["output"]["outpoint"].clone();
            "reused control outpoints"
        }
        "subject_before_accepted_control" => {
            let mut partial = Planner::new(identity(), branch()).expect("planner");
            let mut next = partial.next_step(None).expect("first");
            for (step, response) in &exchanges()[..7] {
                assert_eq!(next.as_ref(), Some(step));
                next = partial
                    .next_step(Some((step.case(), response)))
                    .expect("step");
            }
            partial.manifest.funding = manifest.funding;
            manifest = partial.manifest;
            "subject before accepted control"
        }
        "changed_confidential_funding_fields" => {
            let coin = manifest
                .funding
                .iter_mut()
                .find(|(_, coin)| coin.get("value_commitment").is_some())
                .expect("committed");
            coin.1["amount_satoshis"] = json!(1250);
            "confidential funding fields"
        }
        _ => panic!("named guard"),
    };
    assert_eq!(
        manifest
            .bind_submission(&step, supplied_policy)
            .expect_err("guard")
            .field,
        expected
    );
}
macro_rules! guard_test {
    ($name:ident,$case:literal) => {
        #[test]
        fn $name() {
            manifest_guard($case);
        }
    };
}
guard_test!(manifest_refuses_bad_step_shape, "bad_step_shape");
guard_test!(manifest_refuses_unknown_family, "unknown_family");
guard_test!(manifest_refuses_wrong_row_census, "wrong_row_census");
guard_test!(
    manifest_refuses_changed_control_identity,
    "changed_control_identity"
);
guard_test!(
    manifest_refuses_changed_subject_claim_mapping,
    "changed_subject_claim_mapping"
);
guard_test!(
    manifest_refuses_invalid_transaction_encoding,
    "invalid_transaction_encoding"
);
guard_test!(manifest_refuses_unretained_funding, "unretained_funding");
guard_test!(
    manifest_refuses_changed_input_census,
    "changed_input_census"
);
guard_test!(
    manifest_refuses_missing_or_changed_policy,
    "missing_or_changed_policy"
);
guard_test!(
    manifest_refuses_changed_frozen_policy,
    "changed_frozen_policy"
);
guard_test!(
    manifest_refuses_duplicate_submission,
    "duplicate_submission"
);
guard_test!(
    manifest_refuses_incompatible_control,
    "incompatible_control"
);
guard_test!(
    manifest_refuses_reused_control_outpoints,
    "reused_control_outpoints"
);
guard_test!(
    manifest_refuses_subject_before_accepted_control,
    "subject_before_accepted_control"
);
guard_test!(
    manifest_refuses_changed_confidential_funding_fields,
    "changed_confidential_funding_fields"
);

#[cfg(unix)]
fn assert_scripted_capture(scripted: &Planner) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = common::test_directory("confidential-capture");
    let adapter = directory.join("scripted-adapter.sh");
    let capabilities = [
        "test_funding_ceremony",
        "target_transaction_submission",
        "test_sponsor_authorization",
        "confidential_value_test_funding",
        "confidential_value_sponsor_authorization",
    ];
    std::fs::write(&adapter, common::scripted_adapter(&capabilities, &[])).expect("probe adapter");
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let mut capture = NativeOperationCapture::default();
    common::execute(
        &adapter,
        &directory,
        &identity(),
        ExecutorTrust::Mock,
        &mut Probe,
        &mut capture,
    )
    .expect("zero-operation probe");
    assert_eq!(capture.operations(), []);
    let probe = handshake_reading(&capture);
    assert!(reading_valid(&probe));
    for key in [
        "confidential_value_test_funding",
        "confidential_value_sponsor_authorization",
        "advertisement_matches",
    ] {
        let mut changed = probe.clone();
        changed[key] = json!(false);
        assert!(!reading_valid(&changed));
    }
    let responses = scripted
        .exchanges()
        .iter()
        .map(|(_, response)| serde_json::to_string(response).expect("response"))
        .collect::<Vec<_>>();
    std::fs::write(
        &adapter,
        common::scripted_adapter(&capabilities, &responses),
    )
    .expect("adapter");
    let mut plan = Planner::new(identity(), branch()).expect("planner");
    let outcome = execute_family(
        &adapter,
        &directory,
        &identity(),
        ExecutorTrust::Mock,
        &mut plan,
        &mut capture,
    );
    let real = handshake_reading(&capture);
    let preflight = json!({"library_configured":true,"library_readable":true,"library_sha256":common::hex_bytes(&common::sha256(b"scripted library bytes")),"probe":probe,"capture":real});
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
            "confidential-manifest",
            Some(common::sha256(
                &serde_json::to_vec(&plan.manifest.rows).expect("rows"),
            )),
        );
    let mut guard = common::CaptureGuard::for_test(common::CeremonyId::Report, destination);
    common::write_capture_before_gates(
        &mut guard,
        &capture,
        &facts,
        &payload(&plan, &identity(), &preflight),
    );
    let rendered = std::fs::read_to_string(guard.capture_path().expect("capture path"))
        .expect("capture before gates");
    outcome.expect("scripted operations");
    assert_eq!(probe, real);
    assert_eq!(plan.manifest.rows, scripted.manifest.rows);
    let subjects = rendered
        .lines()
        .filter(|line| line.starts_with("request-subject "))
        .map(|line| common::decode_request_subject_line(line).expect("subject"))
        .collect::<Vec<_>>();
    assert_eq!(
        subjects,
        plan.exchanges()
            .iter()
            .map(|(step, _)| step.subject().clone())
            .collect::<Vec<_>>()
    );
    drop(guard);
    assert_failed_second_handshake(&adapter, &directory, &mut capture);
    std::fs::remove_dir_all(directory).expect("cleanup");
}

#[cfg(unix)]
fn assert_failed_second_handshake(
    adapter: &Path,
    directory: &Path,
    capture: &mut NativeOperationCapture,
) {
    assert!(reading_valid(&handshake_reading(capture)));
    std::fs::write(
        adapter,
        "#!/bin/sh\nIFS= read -r request\nprintf '%s\\n' invalid-handshake\n",
    )
    .expect("broken adapter");
    let mut plan = Planner::new(identity(), branch()).expect("planner");
    assert!(
        execute_family(
            adapter,
            directory,
            &identity(),
            ExecutorTrust::Mock,
            &mut plan,
            capture
        )
        .is_err()
    );
    assert!(!reading_valid(&handshake_reading(capture)));
    assert_eq!(capture.operations(), []);
    assert!(!plan.eligible());
}
