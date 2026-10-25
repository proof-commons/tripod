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
        _ => return Err("recipe belongs to the second schedule".into()),
    }
    let changed = rebuild(&tx, inputs, outputs, witnesses)?;
    if changed == bytes {
        return Err("mutation changed no bytes".into());
    }
    Ok(changed)
}

fn mutate_additional(
    bytes: &[u8],
    recipe: ByteRecipe,
    coin: &FundedOutput,
    ash: u8,
) -> Result<Vec<u8>, String> {
    use transaction::{AssetField, AssetId};
    let tx = TargetTransaction::decode(bytes).map_err(|e| format!("{e:?}"))?;
    let mut inputs = tx.inputs().to_vec();
    let mut outputs = tx.outputs().to_vec();
    let mut witnesses = tx.witnesses().to_vec();
    let mut asset = hex32(&coin.asset);
    asset.reverse();
    let foreign = AssetField::Explicit(AssetId::from_internal(asset));
    let new_input = TargetInput::new(outpoint(coin), inputs[0].sequence());
    match recipe {
        ByteRecipe::AshInputAsset => {
            inputs[1] = new_input;
            outputs[0] = with_value(&outputs[0], amount(&outputs[0])? - coin.amount_satoshis);
            outputs.push(TargetOutput::new(
                foreign,
                ValueField::Explicit(coin.amount_satoshis),
                outputs[0].nonce(),
                vec![0x51],
            ));
        }
        ByteRecipe::AshInputObject => {
            inputs[1] = new_input;
            witnesses[1] = InputWitness::new(Vec::new());
        }
        ByteRecipe::SponsorInputAsset => {
            let index = usize::from(ash);
            inputs.push(inputs[index].clone());
            witnesses.push(witnesses[index].clone());
            inputs[index] = new_input;
            outputs.push(TargetOutput::new(
                foreign,
                ValueField::Explicit(coin.amount_satoshis),
                outputs[0].nonce(),
                vec![0x51],
            ));
        }
        ByteRecipe::SuccessorAsset | ByteRecipe::ChangeAsset => {
            let index = usize::from(recipe == ByteRecipe::ChangeAsset);
            let original = outputs[index].clone();
            inputs.push(new_input);
            witnesses.push(InputWitness::new(Vec::new()));
            outputs[index] = TargetOutput::new(
                foreign,
                original.value(),
                original.nonce(),
                original.program().to_vec(),
            );
            outputs.push(TargetOutput::new(
                original.asset(),
                original.value(),
                original.nonce(),
                vec![0x51],
            ));
        }
        ByteRecipe::TwoSponsors => {
            inputs.push(new_input);
            witnesses.push(InputWitness::new(vec![Vec::new(), Vec::new()]));
            let fee = outputs.last_mut().ok_or("fee missing")?;
            *fee = with_value(fee, amount(fee)? + coin.amount_satoshis);
        }
        _ => return Err("not an additional recipe".into()),
    }
    rebuild(&tx, inputs, outputs, witnesses)
}

fn required(name: &str) -> String {
    std::env::var(name).expect(name)
}

#[test]
fn compact_schedule_summary() {
    let Ok(path) = std::env::var("TRIPOD_COMPACT_SUMMARY") else {
        return;
    };
    let bundle = fixture_bundle().expect("candidate");
    let mode = required("TRIPOD_COMPACT_MODE");
    let attempt = required("TRIPOD_COMPACT_ATTEMPT");
    let second = required("TRIPOD_COMPACT_SCHEDULE") == "2";
    let (rows, census, staged, address_input) = if second {
        let rows = candidate::compact_campaign_second::roster(&bundle).expect("roster");
        let census = candidate::compact_campaign_second::operation_census(&rows);
        let staged = candidate::compact_campaign_second::staged(&rows).len();
        let address = candidate::compact_campaign_second::address_input(&rows, &mode, &attempt);
        (rows, census, staged, address)
    } else {
        let rows = roster(&bundle).expect("roster");
        let census = candidate::compact_campaign::operation_census(&rows);
        let staged = rows.values().filter(|row| row.recipe.is_ok()).count();
        let address =
            std::str::from_utf8(candidate::compact_corpus::first_specification().address_input)
                .expect("address text")
                .replace(
                    "mode record\nattempt 2\n",
                    &format!("mode {mode}\nattempt {attempt}\n"),
                );
        (rows, census, staged, address)
    };
    std::fs::write(format!("{path}.address"), address_input).expect("address inputs");
    std::fs::write(path, format!("summary-schema compact-schedule-1\nroster {}\ncontrols {}\nstaged {staged}\nissuance {}\nconstructor {}\nsponsor-batches {}\nfunding-batches {}\nsignings {}\nsubmissions {}\noperations {}\n",
        rows.len(), ControlFamily::ALL.len(), census.issuance, census.constructor_probes,
        census.sponsor_batches, census.ash_batches, census.signings, census.submissions, census.total())).expect("summary");
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

type Ledger = BTreeMap<Outpoint, (String, String)>;

fn display(mut asset: [u8; 32]) -> String {
    asset.reverse();
    hex_bytes(&asset)
}

// An in-memory executor: controls are mined, mutants refused at the script path.
fn simulated(
    step: &OperationStep,
    bundle: &FixtureBundle,
    ledger: &mut Ledger,
    serial: &mut u8,
) -> NativeOperationResponse {
    use target_elements_conformance::constructor::tagged::sha256;
    *serial += 1;
    let txid = Txid::from_internal([*serial; 32]).to_target_display();
    let mut response = refusal(step, ObservedOutcomeLayer::Accepted);
    let mut fund = |asset: &str, program: &[u8], count: u8, amount: u64| {
        (0..count)
            .map(|vout| {
                let coin = FundedOutput {
                    outpoint: WireOutpoint {
                        txid: txid.clone(),
                        vout: u32::from(vout),
                    },
                    asset: asset.to_owned(),
                    amount_satoshis: amount,
                    script: hex_bytes(program),
                };
                ledger.insert(outpoint(&coin), (coin.asset.clone(), coin.script.clone()));
                coin
            })
            .collect()
    };
    match step.subject() {
        OperationSubject::Funding(subject) => {
            let asset = if subject.issue_asset {
                let issued = if *serial == 1 {
                    display(bundle.closed_asset())
                } else {
                    display([0x77; 32])
                };
                response.issued_asset = Some(issued.clone());
                issued
            } else {
                subject.asset.clone().expect("funding asset")
            };
            response.funded_outputs = fund(
                &asset,
                &subject.output_program,
                subject.outputs,
                subject.amount_per_output,
            );
        }
        OperationSubject::SponsorFunding(subject) => {
            let mut program = vec![0, 20];
            program.extend([0x24; 20]);
            response.funded_outputs = fund(
                &display(bundle.reserve_asset()),
                &program,
                subject.sponsor_outputs,
                subject.amount_per_sponsor_output,
            );
        }
        OperationSubject::SponsorSigning(subject) => {
            response.signature_bound_to = Some(subject.finalized_transaction.clone());
            response.sponsor_witness = vec![vec![0x30; 71], vec![0x02; 33]];
        }
        OperationSubject::Submission(subject) => {
            if step.case().step.as_str().ends_with("/submit-mutant") {
                response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
            } else {
                let bytes = &subject.transaction_bytes;
                let tx = TargetTransaction::decode(bytes).expect("control");
                let identity = Txid::from_internal(sha256(&sha256(&tx.encode_without_witness())))
                    .to_target_display();
                response.accepted_txid = Some(identity.clone());
                response.mined_readback = Some(
                    target_elements_conformance::protocol::MinedFundingReadback {
                        transaction_id: identity,
                        witness_transaction_id: Txid::from_internal(sha256(&sha256(bytes)))
                            .to_target_display(),
                        block_hash: "00".repeat(32),
                        block_height: u32::from(*serial),
                        raw_transaction: bytes.clone(),
                    },
                );
            }
        }
        _ => panic!("the compact schedule names no other subject"),
    }
    response
}

// The row's own coordinate differs between the mutant and its paired control.
fn own_coordinate_changes(
    recipe: ByteRecipe,
    ash: u8,
    mutant: &[u8],
    control: &[u8],
    ledger: &Ledger,
) {
    let mutant = TargetTransaction::decode(mutant).expect("mutant");
    let control = TargetTransaction::decode(control).expect("control");
    let held =
        |tx: &TargetTransaction, index: usize| ledger[&tx.inputs()[index].outpoint()].clone();
    let ash = usize::from(ash);
    match recipe {
        ByteRecipe::AshInputAsset => {
            assert_ne!(held(&mutant, 1).0, held(&control, 1).0);
            assert_eq!(held(&mutant, 1).1, held(&control, 1).1);
        }
        ByteRecipe::AshInputObject => {
            assert_eq!(held(&mutant, 1).0, held(&control, 1).0);
            assert_ne!(held(&mutant, 1).1, held(&control, 1).1);
        }
        ByteRecipe::SponsorInputAsset => {
            assert_ne!(held(&mutant, ash).0, held(&control, ash).0);
            assert_eq!(held(&mutant, ash).1, held(&control, ash).1);
        }
        ByteRecipe::SuccessorAsset | ByteRecipe::ChangeAsset => {
            let index = usize::from(recipe == ByteRecipe::ChangeAsset);
            assert_ne!(
                mutant.outputs()[index].asset(),
                control.outputs()[index].asset()
            );
            assert_eq!(
                mutant.outputs()[index].value(),
                control.outputs()[index].value()
            );
            assert_eq!(
                mutant.outputs()[index].program(),
                control.outputs()[index].program()
            );
        }
        ByteRecipe::TwoSponsors => {
            let sponsor = held(&control, ash);
            let count = |tx: &TargetTransaction| {
                (0..tx.inputs().len())
                    .filter(|index| held(tx, *index) == sponsor)
                    .count()
            };
            assert_eq!((count(&mutant), count(&control)), (2, 1));
        }
        _ => panic!("not a second-schedule recipe"),
    }
}

#[test]
fn second_schedule_settles_offline_to_its_derived_census() {
    let bundle = fixture_bundle().expect("candidate");
    let rows = candidate::compact_campaign_second::roster(&bundle).expect("roster");
    let census = candidate::compact_campaign_second::operation_census(&rows);
    let mut planner = Planner::second(Box::new(mutate_additional));
    let (mut ledger, mut serial, mut counts) = (Ledger::new(), 0, [0; 6]);
    let (mut previous, mut pending) = (None::<(OperationStep, NativeOperationResponse)>, None);
    let mut recipes = BTreeSet::new();
    while let Some(step) = planner
        .next_step(
            previous
                .as_ref()
                .map(|(step, response)| (step.case(), response)),
        )
        .unwrap_or_else(|_| panic!("{:?}", planner.failure))
    {
        counts[match step.subject() {
            OperationSubject::Funding(subject) if subject.issue_asset => 0,
            OperationSubject::Funding(_) if step.case().step == "constructor" => 1,
            OperationSubject::SponsorFunding(_) => 2,
            OperationSubject::Funding(_) => 3,
            OperationSubject::SponsorSigning(_) => 4,
            _ => 5,
        }] += 1;
        if let OperationSubject::Submission(subject) = step.subject() {
            let job = &planner.jobs[planner.index];
            if step.case().step.as_str().ends_with("/submit-mutant") {
                let recipe = job.row.as_ref().expect("row").recipe.expect("recipe");
                pending = Some((recipe, job.family.ash, subject.transaction_bytes.clone()));
            } else if let Some((recipe, ash, mutant)) = pending.take() {
                own_coordinate_changes(recipe, ash, &mutant, &subject.transaction_bytes, &ledger);
                recipes.insert(format!("{recipe:?}/{}", job.family.name()));
            }
        }
        let response = simulated(&step, &bundle, &mut ledger, &mut serial);
        previous = Some((step, response));
    }
    assert!(planner.failure.is_none(), "{:?}", planner.failure);
    assert_eq!(
        counts,
        [
            census.issuance,
            census.constructor_probes,
            census.sponsor_batches,
            census.ash_batches,
            census.signings,
            census.submissions
        ]
    );
    assert_eq!(counts, [2, 1, 1, 30, 22, 31]);
    assert_eq!(planner.accepted_families.len(), 9);
    assert!(planner.suspended.is_empty());
    assert_eq!(
        recipes,
        [
            "AshInputAsset/A2",
            "AshInputAsset/N2",
            "AshInputObject/A2",
            "AshInputObject/N2",
            "ChangeAsset/C2",
            "SponsorInputAsset/N2",
            "SuccessorAsset/A2",
            "SuccessorAsset/N2",
            "TwoSponsors/N2"
        ]
        .map(String::from)
        .into_iter()
        .collect()
    );
    assert_eq!(
        planner
            .findings
            .iter()
            .filter(|finding| finding.starts_with("mutant ") && finding.ends_with(" true"))
            .count(),
        11
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
    let second = std::env::var("TRIPOD_COMPACT_SCHEDULE").is_ok_and(|version| version == "2");
    let mut planner = if second {
        Planner::second(Box::new(mutate_additional))
    } else {
        Planner::staged(
            std::env::var("TRIPOD_COMPACT_MODE").is_ok_and(|mode| mode == "probe"),
            Box::new(mutate),
        )
    };
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
