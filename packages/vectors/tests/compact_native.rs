//! Compact native staging; malformed transaction assembly stays in this test target.

#[allow(
    dead_code,
    reason = "shared archive helpers include ceremonies outside compact"
)]
pub mod common;

use std::io::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, NativeOperationCapture,
    execute_operations_captured,
};
use transaction::ValueField;

const TEST_NAME: &str = "compact_campaign_runs_against_the_pinned_target";

use vectors as candidate;
include!("../src/compact_campaign_planner.rs");

fn with_value(output: &TargetOutput, amount: u64) -> TargetOutput {
    TargetOutput::new(
        output.asset(),
        ValueField::Explicit(amount),
        output.nonce(),
        output.program().to_vec(),
    )
}

fn amount(output: &TargetOutput) -> Result<u64, String> {
    if let ValueField::Explicit(value) = output.value() {
        Ok(value)
    } else {
        Err("nonexplicit amount".into())
    }
}

fn mutate(
    bytes: &[u8],
    recipe: ByteRecipe,
    extra: Option<Outpoint>,
    ash: u8,
) -> Result<Vec<u8>, String> {
    let tx = TargetTransaction::decode(bytes).map_err(|e| format!("{e:?}"))?;
    if tx.encode() != bytes {
        return Err("roundtrip drift".into());
    }
    let mut inputs = tx.inputs().to_vec();
    let mut outputs = tx.outputs().to_vec();
    let mut witnesses = tx.witnesses().to_vec();
    match recipe {
        ByteRecipe::OneInput => {
            inputs.remove(1);
            witnesses.remove(1);
            outputs[0] = with_value(&outputs[0], amount(&outputs[0])? - 100);
        }
        ByteRecipe::FiveInputs => {
            inputs.insert(
                usize::from(ash),
                TargetInput::new(extra.ok_or("extra coin missing")?, inputs[0].sequence()),
            );
            witnesses.insert(usize::from(ash), witnesses[1].clone());
            outputs[0] = with_value(&outputs[0], amount(&outputs[0])? + 100);
        }
        ByteRecipe::RedirectSuccessor => {
            let output = &outputs[0];
            outputs[0] =
                TargetOutput::new(output.asset(), output.value(), output.nonce(), vec![0x51]);
        }
        ByteRecipe::SplitSuccessor | ByteRecipe::SplitChange => {
            let index = usize::from(recipe == ByteRecipe::SplitChange);
            let original = outputs[index].clone();
            let value = amount(&original)?;
            outputs[index] = with_value(&original, value / 2);
            outputs.insert(index + 1, with_value(&original, value - value / 2));
        }
        ByteRecipe::DivertUnit => {
            let original = outputs[0].clone();
            outputs[0] = with_value(&original, amount(&original)? - 1);
            outputs.insert(
                1,
                TargetOutput::new(
                    original.asset(),
                    ValueField::Explicit(1),
                    original.nonce(),
                    vec![0x51],
                ),
            );
        }
        ByteRecipe::MissingAuthorization => {
            *witnesses.last_mut().ok_or("sponsor witness absent")? = InputWitness::new(Vec::new());
        }
    }
    let changed = rebuild(&tx, inputs, outputs, witnesses)?;
    if changed == bytes {
        return Err("mutation changed no bytes".into());
    }
    Ok(changed)
}

fn required(name: &str) -> String {
    std::env::var(name).expect(name)
}

fn refusal(step: &OperationStep, layer: ObservedOutcomeLayer) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA,
        case: step.case().clone(),
        observed_layer: layer,
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
        resources: target_elements_conformance::protocol::NativeResourceObservation::default(),
    }
}

#[test]
fn failed_control_suspends_its_family_without_spending_unrelated_packets() {
    let mut planner = Planner::new(Box::new(mutate));
    let failed = planner.jobs[0].family;
    planner.stage = Stage::SubmitControl;
    let step = planner.step().expect("step").expect("control");
    planner
        .settle(
            &step,
            &refusal(&step, ObservedOutcomeLayer::ScriptPathRejection),
        )
        .expect("settle");
    assert_eq!(planner.suspended, BTreeSet::from([failed]));
    assert!(planner.accepted_families.is_empty());
    let sibling = planner
        .jobs
        .iter()
        .position(|job| job.row.is_some() && job.family == failed)
        .expect("sibling");
    planner.index = sibling;
    planner.stage = Stage::Fund;
    let step = planner.step().expect("next").expect("unrelated funding");
    assert!(matches!(step.subject(), OperationSubject::Funding(_)));
    assert_ne!(planner.jobs[planner.index].family, failed);
    assert_eq!(planner.sponsor_index, 0);
}

#[test]
fn every_staged_recipe_builds_distinct_bytes_without_relaxing_the_control() {
    let mut planner = Planner::new(Box::new(mutate));
    planner.bundle = Some(fixture_bundle().expect("bundle"));
    let mut asset = planner.bundle.as_ref().expect("bundle").closed_asset();
    asset.reverse();
    let mut sponsor_program = vec![0, 20];
    sponsor_program.extend(0..20);
    planner.sponsors = (0..15)
        .map(|index| {
            SponsorCoin::new(
                Outpoint::new(Txid::from_internal([0x41; 32]), index).expect("outpoint"),
                100_000,
                sponsor_program.clone(),
            )
        })
        .collect();
    let mut mutations = 0;
    for index in 0..planner.jobs.len() {
        planner.index = index;
        let job = &planner.jobs[index];
        let count = job.family.ash
            + u8::from(
                job.row
                    .as_ref()
                    .is_some_and(|row| row.recipe == Ok(ByteRecipe::FiveInputs)),
            );
        let coins = (0..count)
            .map(|vout| FundedOutput {
                outpoint: WireOutpoint {
                    txid: Txid::from_internal([0x42; 32]).to_target_display(),
                    vout: u32::from(vout),
                },
                asset: common::hex_bytes(&asset),
                amount_satoshis: 100,
                script: common::hex_bytes(&planner.program),
            })
            .collect::<Vec<_>>();
        planner
            .construct(&coins)
            .expect("safe control then test-local surgery");
        let control = TargetTransaction::decode(&planner.control).expect("control bytes");
        assert_eq!(
            control.inputs().len(),
            usize::from(planner.jobs[index].family.ash)
                + usize::from(planner.jobs[index].family.sponsored)
        );
        if planner.jobs[index].row.is_some() {
            assert_ne!(planner.control, planner.mutant);
            assert_eq!(
                TargetTransaction::decode(&planner.mutant)
                    .expect("mutant bytes")
                    .encode(),
                planner.mutant
            );
            mutations += 1;
        }
    }
    assert_eq!(mutations, 16);
    assert_eq!(planner.sponsor_index, 15);
}

#[test]
fn submission_readback_must_match_the_submitted_bytes() {
    let planner = Planner::new(Box::new(mutate));
    let step = planner.funding("identity", true, 1, 1);
    let response = refusal(&step, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        verify_readback(&[], &response),
        Err("mined readback absent".into())
    );
}

#[test]
#[ignore = "requires the pinned target and disposable funding"]
#[allow(
    clippy::too_many_lines,
    reason = "capture is written before checking completion"
)]
fn compact_campaign_runs_against_the_pinned_target() {
    let started = Instant::now();
    let directory = required("TRIPOD_LIVE_REPORT_DIR");
    let target = reviewed_elements_tapscript().expect("target");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            hex32(&required("TRIPOD_LIVE_NETWORK_ID")),
            hex32(&required("TRIPOD_LIVE_GENESIS_ID")),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("binding");
    let mut planner = Planner::staged(
        std::env::var("TRIPOD_COMPACT_MODE").is_ok_and(|mode| mode == "probe"),
        Box::new(mutate),
    );
    let staged_ms = started.elapsed().as_millis();
    let mut capture = NativeOperationCapture::default();
    let adapter = required("TRIPOD_LIVE_EXECUTOR");
    let configuration = ExecutorConfiguration::new(
        Path::new(&adapter),
        ExecutorTrust::ReviewedNonMock,
        Duration::from_secs(1800),
        ExecutorDiagnostics::in_directory(&Path::new(&directory).join("diagnostics")),
    );
    let executing = Instant::now();
    let outcome = execute_operations_captured(
        &target,
        &binding,
        &configuration,
        &mut planner,
        &mut capture,
    );
    let execution_ms = executing.elapsed().as_millis();
    let mut payload = String::from("compact-campaign-schema 1\n");
    let _ = writeln!(
        payload,
        "run-address {}",
        required("TRIPOD_COMPACT_RUN_ADDRESS")
    );
    for operation in capture.operations() {
        if let Some(response) = operation.response() {
            common::write_bytes_field(
                &mut payload,
                "compact-validated-response",
                &serde_json::to_vec(response).expect("response"),
            );
        }
    }
    for row in &planner.rows {
        let _ = writeln!(
            payload,
            "roster {} {} {:?} expected {:?}",
            requirement_name(&row.manifest.requirement),
            row.control.name(),
            row.recipe,
            row.expected
        );
    }
    for finding in &planner.findings {
        let _ = writeln!(payload, "{finding}");
    }
    let _ = writeln!(
        payload,
        "suspended {:?}\nfailure {:?}",
        planner.suspended, planner.failure
    );
    let mut destination = common::CaptureDestination::new(
        directory.into(),
        required("TRIPOD_COMPACT_RUN_ADDRESS"),
        required("TRIPOD_LIVE_SUITE_COMMIT"),
        required("TRIPOD_LIVE_SUITE_TREE"),
    )
    .expect("destination");
    destination.report_test_name = TEST_NAME;
    let facts = common::CeremonyCaptureFacts::from_capture(common::CeremonyId::Report, &capture);
    let rendered = common::render_enhanced_capture(
        &destination,
        common::CeremonyId::Report,
        &capture,
        &facts,
        &payload,
    )
    .expect("capture grammar");
    let path = destination
        .capture_path(common::CeremonyId::Report, TEST_NAME)
        .expect("new capture");
    std::fs::write(&path, rendered).expect("capture");
    let status = if outcome.is_ok() && planner.failure.is_none() {
        "passed"
    } else {
        "failed"
    };
    std::fs::write(path.with_extension("capture.timing"), format!("timing-schema compact-phases-1\nceremony-id report\nstatus {status}\nphase-staging-ms {staged_ms}\nphase-execution-ms {execution_ms}\ntotal-ms {}\n", started.elapsed().as_millis())).expect("timing");
    let mut output = std::io::stdout().lock();
    writeln!(
        output,
        "\nCOMPACT operations {} controls {} suspended {}",
        capture.operations().len(),
        planner.accepted_families.len(),
        planner.suspended.len()
    )
    .expect("summary");
    for finding in &planner.findings {
        writeln!(output, "{finding}").expect("finding");
    }
    for row in &planner.rows {
        writeln!(
            output,
            "roster {} {} {:?}",
            requirement_name(&row.manifest.requirement),
            row.control.name(),
            row.recipe
        )
        .expect("roster");
    }
    outcome.expect("executor completion");
    assert!(planner.failure.is_none(), "{:?}", planner.failure);
}
