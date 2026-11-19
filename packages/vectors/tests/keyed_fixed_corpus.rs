//! Five focused offline checks of the sealed fixed-form campaign projection.

#[path = "../src/keyed_fixed_corpus/fixture.rs"]
mod keyed_fixed_corpus;

use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_fixed_plan::{PlacementSearchLimits, plan},
};
use elements::hashes::sha256d;
use keyed_fixed_corpus::{CorpusRefusal, Packet};
use linker::{
    keyed_fixed_bundle::link_keyed_fixed_bundle,
    keyed_fixed_reference::keyed_fixed_parameter_offers,
};
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use std::io::Read;
use std::time::Duration;
use std::time::Instant;
use tapscript::{
    keyed_fixed_bundle::{KeyedFixedBundleSelection, emit_keyed_fixed_bundle},
    keyed_fixed_constructor::KeyedFixedLeafRecipe,
};
use transaction::keyed_fixed_construct::{
    KeyedFixedConstructionError, KeyedFixedConstructionRequest, KeyedFixedOutputOffer,
    construct_keyed_fixed,
};
use transaction::keyed_fixed_inputs::{KeyedFixedInputOffer, KeyedFixedInputRole};
use transaction::keyed_fixed_request::keyed_fixed_merge_request;
use transaction::keyed_fixed_taproot::commit_keyed_fixed_bundle;
use transaction::{
    AssetField, InputWitness, NonceField, Outpoint, SponsorCapability, SponsorOffer,
    SponsorSignature, SponsorSigningRequest, TargetOutput, TargetTransaction, Txid, ValueField,
};
use vectors::keyed_fixed_capture::{CaptureFamily, CaptureSubject};
use vectors::keyed_fixed_manifest::{FreezeManifest, FreezeRefusal};

fn report(name: &str, started: Instant, records: usize) {
    tracing::info!(target: "vectors::keyed_fixed_corpus", name, records, wall_ms = started.elapsed().as_millis(), outcome = "completed", "focused offline test completed");
}

fn trace() {
    drop(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .without_time()
            .with_ansi(false)
            .try_init(),
    );
}

struct ReplaySponsor {
    point: Outpoint,
    program: Vec<u8>,
    fee: u64,
    change: u64,
}

impl SponsorCapability for ReplaySponsor {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new(
            [self.point],
            self.fee,
            Some(ValueField::Explicit(self.change)),
        )
        .unwrap()
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, self.program[2..].to_vec()))
    }

    fn sign(&self, _request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        None
    }
}

fn linked_merge(q: KeyedFixedParameters) -> linker::keyed_fixed_bundle::LinkedKeyedFixedBundle {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let manifest = KeyedFixedManifest::contract().unwrap();
    let declaration = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let limit = std::num::NonZeroU64::new(100_000).unwrap();
    let input = bind_input(
        &manifest,
        declaration,
        vec![KeyedFixedOperationId::Merge],
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .unwrap();
    let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
    let selection = KeyedFixedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedFixedLeafRecipe::new(q);
    let emission = emit_keyed_fixed_bundle(&target, &plan, selection, recipe).unwrap();
    link_keyed_fixed_bundle(
        &target,
        &emission,
        &keyed_fixed_parameter_offers(&target, recipe),
    )
    .unwrap()
}

#[allow(clippy::too_many_lines)] // Both replay sizes use the same authenticated parent and policy route.
fn replay_typed_resources(
    packet: &Packet,
    members: usize,
    record_name: &str,
    sponsor_exchange: Option<usize>,
) -> Result<(), KeyedFixedConstructionError> {
    let row: serde_json::Value =
        serde_json::from_slice(packet.file(&format!("scaling/{record_name}.json")).unwrap())
            .unwrap();
    let mut parents: Vec<Vec<u8>> = serde_json::from_value(row["parents"].clone()).unwrap();
    assert_eq!(parents.len(), members + 1);
    let sponsor_index = members;
    let sponsor_vout = if let Some(exchange) = sponsor_exchange {
        let response: serde_json::Value = serde_json::from_slice(
            packet
                .file(&format!("scaling/exchange-{exchange:05}-response.json"))
                .unwrap(),
        )
        .unwrap();
        parents[sponsor_index] =
            serde_json::from_value(response["mined_readback"]["raw_transaction"].clone()).unwrap();
        u32::try_from(
            response["funded_outputs"][0]["outpoint"]["vout"]
                .as_u64()
                .unwrap(),
        )
        .unwrap()
    } else {
        let tx = TargetTransaction::decode(
            &row["submitted"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| u8::try_from(b.as_u64().unwrap()).unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        tx.inputs()[sponsor_index].outpoint().index()
    };
    let raw: Vec<u8> = serde_json::from_value(row["submitted"].clone()).unwrap();
    let original = TargetTransaction::decode(&raw).unwrap();
    let ValueField::Explicit(change) = original.outputs()[1].value() else {
        panic!("explicit change")
    };
    let ValueField::Explicit(fee) = original.outputs().last().unwrap().value() else {
        panic!("explicit fee")
    };
    let asset: [u8; 32] = serde_json::from_value::<Vec<u8>>(row["asset"].clone())
        .unwrap()
        .try_into()
        .unwrap();
    let q = KeyedFixedParameters {
        key: [42; 32],
        asset,
        value: KeyedFixedValue::new(60).unwrap(),
    };
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let linked = linked_merge(q);
    let commitment = commit_keyed_fixed_bundle(&target, &linked).unwrap();
    let sponsor_parent = TargetTransaction::decode(&parents[sponsor_index]).unwrap();
    let sponsor_spent = &sponsor_parent.outputs()[usize::try_from(sponsor_vout).unwrap()];
    let AssetField::Explicit(policy_asset) = sponsor_spent.asset() else {
        panic!("explicit sponsor asset")
    };
    let sponsor_txid = Txid::from_internal(
        sha256d::Hash::hash(&sponsor_parent.encode_without_witness()).to_byte_array(),
    );
    let sponsor = ReplaySponsor {
        point: Outpoint::new(sponsor_txid, sponsor_vout).unwrap(),
        program: sponsor_spent.program().to_vec(),
        fee,
        change,
    };
    let offers = parents
        .iter()
        .enumerate()
        .map(|(index, bytes)| {
            let parent = TargetTransaction::decode(bytes).unwrap();
            let vout = if index == sponsor_index {
                sponsor_vout
            } else {
                original.inputs()[index].outpoint().index()
            };
            let id = Txid::from_internal(
                sha256d::Hash::hash(&parent.encode_without_witness()).to_byte_array(),
            );
            KeyedFixedInputOffer {
                input_index: u32::try_from(index).unwrap(),
                parent: bytes,
                outpoint: Outpoint::new(id, vout).unwrap(),
                sequence: original.inputs()[index].sequence(),
                flags: 0,
                role: if index == sponsor_index {
                    KeyedFixedInputRole::Collateral
                } else {
                    KeyedFixedInputRole::Member
                },
                capability: if index == sponsor_index {
                    Some(&sponsor as &dyn SponsorCapability)
                } else {
                    None
                },
            }
        })
        .collect::<Vec<_>>();
    let request = keyed_fixed_merge_request(&target, q, &commitment, &offers).unwrap();
    let offer = KeyedFixedOutputOffer {
        policy_asset,
        successor_value: 60,
        additional_outputs: vec![TargetOutput::new(
            AssetField::Explicit(policy_asset),
            ValueField::Explicit(change),
            NonceField::Null,
            sponsor.program.clone(),
        )],
        fee: Some(fee),
    };
    construct_keyed_fixed(
        &target,
        KeyedFixedConstructionRequest::Merge(&request),
        offer,
    )
    .map(|_| ())
}

#[test]
#[allow(clippy::too_many_lines)] // One focused decoder test keeps each control beside its changed byte.
fn fixed_corpus_seal_and_identity() {
    trace();
    let started = Instant::now();
    let clauses = Packet::load(CaptureFamily::Clauses).unwrap();
    let policy = Packet::load(CaptureFamily::Policy).unwrap();
    let sources = [
        (
            "packages/vectors/src/keyed_fixed_manifest.rs",
            clauses
                .source("packages/vectors/src/keyed_fixed_manifest.rs")
                .unwrap(),
        ),
        (
            "packages/vectors/src/keyed_fixed_manifest/encoding.rs",
            clauses
                .source("packages/vectors/src/keyed_fixed_manifest/encoding.rs")
                .unwrap(),
        ),
    ];
    for (packet, path) in [
        (&clauses, "clauses/freeze-42-60.bin"),
        (&clauses, "clauses/freeze-43-1000.bin"),
        (&policy, "policy/freeze-42-14.bin"),
        (&policy, "policy/freeze-42-15.bin"),
    ] {
        let encoded = packet.file(path).unwrap();
        let historical = FreezeManifest::reconstruct_campaign_freeze(encoded, &sources).unwrap();
        let parameters: KeyedFixedParameters = historical.manifest().descriptor.parameters;
        assert_eq!(historical.encoding(), encoded);
        assert_eq!(
            historical.manifest().clone().validate(parameters),
            Err(FreezeRefusal::Inventory)
        );
        assert!(
            FreezeManifest::offer(parameters)
                .unwrap()
                .validate(parameters)
                .is_ok()
        );
    }
    let original = clauses.file("clauses/freeze-42-60.bin").unwrap();
    let mut changed = original.to_vec();
    let source_path = b"packages/vectors/src/keyed_fixed_manifest.rs";
    let position = changed
        .windows(source_path.len())
        .position(|window| window == source_path)
        .unwrap();
    changed[position + source_path.len()] ^= 1;
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&changed, &sources),
        Err(FreezeRefusal::Inventory)
    );
    let mut domain = original.to_vec();
    domain[0] ^= 1;
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&domain, &sources),
        Err(FreezeRefusal::Recipe)
    );
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&original[..original.len() - 1], &sources),
        Err(FreezeRefusal::Framing)
    );
    let mut boolean = original.to_vec();
    *boolean.last_mut().unwrap() = 2;
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&boolean, &sources),
        Err(FreezeRefusal::Framing)
    );
    let mut envelope = original.to_vec();
    *envelope.last_mut().unwrap() = 0;
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&envelope, &sources),
        Err(FreezeRefusal::Envelope)
    );
    let codes = [0, 0, 0, 15, 0, 1, 0, 2];
    let code_offset = original
        .windows(codes.len())
        .position(|window| window == codes)
        .unwrap();
    let mut code = original.to_vec();
    code[code_offset + 5] = 0;
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&code, &sources),
        Err(FreezeRefusal::Inventory)
    );
    let key_offset = code_offset + 4 + 15 * 2;
    let mut triple = original.to_vec();
    triple[key_offset] ^= 1;
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&triple, &sources),
        Err(FreezeRefusal::Descriptor)
    );
    let leaf_offset = key_offset + 32 + 32 + 8 + 42 + 32 + 4;
    let mut descriptor = original.to_vec();
    descriptor[leaf_offset] ^= 1;
    assert_eq!(
        FreezeManifest::reconstruct_campaign_freeze(&descriptor, &sources),
        Err(FreezeRefusal::Descriptor)
    );
    let mut compressed =
        include_bytes!("../fixtures/keyed-fixed-native/clauses.json.lzma").to_vec();
    compressed[15] ^= 1;
    assert_eq!(
        Packet::check_fixture_candidate("clauses", &compressed),
        Err(CorpusRefusal::Digest)
    );
    report("fixed_corpus_seal_and_identity", started, 4);
}

#[test]
#[allow(clippy::too_many_lines)] // One full admission shares the acquired packet with small inner-boundary changes.
fn fixed_corpus_exact_native_pairs() {
    trace();
    if std::env::var_os("TRIPOD_S8B_ADMISSION_CHILD").is_none() {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "fixed_corpus_exact_native_pairs", "--nocapture"])
            .env("TRIPOD_S8B_ADMISSION_CHILD", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let stdout_pipe = child.stdout.take().unwrap();
        let stderr_pipe = child.stderr.take().unwrap();
        let stdout_reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let mut reader = stdout_pipe;
            reader.read_to_end(&mut bytes).unwrap();
            bytes
        });
        let stderr_reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let mut reader = stderr_pipe;
            reader.read_to_end(&mut bytes).unwrap();
            bytes
        });
        let started = Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if started.elapsed() > Duration::from_secs(900) {
                child.kill().unwrap();
                child.wait().unwrap();
                let stdout = stdout_reader.join().unwrap();
                let stderr = stderr_reader.join().unwrap();
                panic!(
                    "full admission exceeded 900 s: {} {}",
                    String::from_utf8_lossy(&stdout),
                    String::from_utf8_lossy(&stderr)
                );
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let status = child.wait().unwrap();
        let stdout_bytes = stdout_reader.join().unwrap();
        let stderr_bytes = stderr_reader.join().unwrap();
        let stdout = String::from_utf8_lossy(&stdout_bytes);
        let stderr = String::from_utf8_lossy(&stderr_bytes);
        assert!(status.success(), "{stdout} {stderr}");
        assert!(
            stdout.contains("running 1 test"),
            "filtered command ran zero or multiple tests: {stdout}"
        );
        assert!(
            stdout.contains("S8B_RECORDS") && stdout.contains("records=220"),
            "full admission count absent: {stdout}"
        );
        assert!(
            stdout.contains("S8B_PARENT_ONLY") && stdout.contains("parent_only=4000"),
            "parent-only count absent: {stdout}"
        );
        for line in stdout
            .lines()
            .filter(|line| line.contains("S8B_ATTRIBUTION"))
        {
            tracing::info!(target: "vectors::keyed_fixed_corpus", line, "retained model attribution result");
        }
        report("fixed_corpus_exact_native_pairs", started, 220);
        return;
    }
    let started = Instant::now();
    let mut total = 0;
    let mut parent_only = 0;
    let mut clauses = None;
    for family in [
        CaptureFamily::Clauses,
        CaptureFamily::Policy,
        CaptureFamily::Scaling,
        CaptureFamily::Histories,
    ] {
        let packet = Packet::load(family).unwrap();
        total += packet.admit_all().unwrap();
        if family == CaptureFamily::Scaling {
            parent_only = packet.authenticate_scaling_parents().unwrap();
        }
        if family == CaptureFamily::Clauses {
            clauses = Some(packet);
        }
    }
    assert_eq!(total, 220);
    assert_eq!(parent_only, 4000);
    let clauses = clauses.unwrap();
    let sites = clauses.attribute_sites().unwrap();
    assert_eq!(sites.len(), 7);
    let observed = sites
        .iter()
        .map(|site| (site.subject, site.instruction, site.reason))
        .collect::<Vec<_>>();
    let expected = [
        (CaptureSubject::ValueMinus, Some(12), "unique equality"),
        (CaptureSubject::ValuePlus, Some(12), "unique equality"),
        (CaptureSubject::WrongProgram, Some(34), "unique equality"),
        (CaptureSubject::WrongKey, Some(34), "unique equality"),
        (
            CaptureSubject::DisplacedSuccessor,
            Some(34),
            "unique equality",
        ),
        (
            CaptureSubject::MalformedPaymentValue,
            Some(21),
            "unique equality",
        ),
        (CaptureSubject::LaterPosition, Some(26), "unique equality"),
    ];
    assert_eq!(observed.as_slice(), expected.as_slice());
    for site in &sites {
        tracing::info!(target: "vectors::keyed_fixed_corpus", subject = ?site.subject, instruction = ?site.instruction, reason = site.reason, "S8B_ATTRIBUTION");
    }
    let mut changed = clauses.clone();
    let path = "clauses/subject-00036.json";
    let mut row: serde_json::Value = serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    row["parents"][0][0] = serde_json::json!(3);
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&row).unwrap())
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Record));
    let mut changed = clauses.clone();
    let mut row: serde_json::Value = serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    row["control"]["request"]["case"]["step"] = serde_json::json!("fixed-submit-3");
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&row).unwrap())
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Control));
    let mut changed = clauses.clone();
    let path = "clauses/observed-provenance.json";
    let mut observed: serde_json::Value =
        serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    observed["daemon_digest"][0] = serde_json::json!(0);
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&observed).unwrap())
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Provenance));
    let mut changed = clauses.clone();
    let path = "clauses/container-final-state.json";
    let mut state: serde_json::Value = serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    state[0]["Image"] = serde_json::json!("changed-image");
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&state).unwrap())
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Provenance));
    let mut changed = clauses.clone();
    let sign_path = "clauses/exchange-00035-response.json";
    let mut sign: serde_json::Value =
        serde_json::from_slice(changed.file(sign_path).unwrap()).unwrap();
    sign["sponsor_witness"][0][0] = serde_json::json!(0);
    changed
        .replace_after_acquisition(sign_path, serde_json::to_vec(&sign).unwrap())
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Record));
    let mut changed = clauses.clone();
    let path = "clauses/subject-00084.json";
    let mut row: serde_json::Value = serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    let mut detail: serde_json::Value =
        serde_json::from_str(row["response"]["observed_detail"].as_str().unwrap()).unwrap();
    let txid = detail["detail"]["txid"].as_str().unwrap();
    let first = if txid.starts_with('0') { '1' } else { '0' };
    detail["detail"]["txid"] = serde_json::json!(format!("{first}{}", &txid[1..]));
    row["response"]["observed_detail"] = serde_json::json!(serde_json::to_string(&detail).unwrap());
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&row).unwrap())
        .unwrap();
    changed
        .replace_after_acquisition(
            "clauses/exchange-00084-response.json",
            serde_json::to_vec(&row["response"]).unwrap(),
        )
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Record));
    let mut changed = clauses.clone();
    let path = "clauses/exchange-00087-response.json";
    let mut response: serde_json::Value =
        serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    let mut detail: serde_json::Value =
        serde_json::from_str(response["observed_detail"].as_str().unwrap()).unwrap();
    detail["relay_allowed"] = serde_json::json!(false);
    response["observed_detail"] = serde_json::json!(serde_json::to_string(&detail).unwrap());
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&response).unwrap())
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Record));
    let mut changed = clauses.clone();
    let path = "clauses/subject-00084.json";
    let mut row: serde_json::Value = serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    let original: Vec<u8> = serde_json::from_value(row["submitted"].clone()).unwrap();
    let tx = TargetTransaction::decode(&original).unwrap();
    let mut witnesses = tx.witnesses().to_vec();
    let mut member = witnesses[0].stack().to_vec();
    assert_eq!(member.len(), 1);
    member.push(vec![1]);
    witnesses[0] = InputWitness::new(member);
    let changed_tx = TargetTransaction::with_output_witnesses(
        tx.version(),
        tx.inputs().to_vec(),
        tx.outputs().to_vec(),
        tx.lock_time(),
        witnesses,
        tx.output_witnesses().to_vec(),
    )
    .unwrap()
    .encode();
    row["submitted"] = serde_json::to_value(&changed_tx).unwrap();
    row["request"]["subject"]["transaction_bytes"] = row["submitted"].clone();
    let changed_weight = elements::encode::deserialize::<elements::Transaction>(&changed_tx)
        .unwrap()
        .weight();
    row["response"]["resources"]["transaction_weight"] = serde_json::json!(changed_weight);
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&row).unwrap())
        .unwrap();
    changed
        .replace_after_acquisition("clauses/subject-00084.transaction", changed_tx)
        .unwrap();
    changed
        .replace_after_acquisition(
            "clauses/exchange-00084-request.json",
            serde_json::to_vec(&row["request"]).unwrap(),
        )
        .unwrap();
    changed
        .replace_after_acquisition(
            "clauses/exchange-00084-response.json",
            serde_json::to_vec(&row["response"]).unwrap(),
        )
        .unwrap();
    assert_eq!(changed.admit_all(), Err(CorpusRefusal::Witness));
    let mut changed = clauses;
    let path = "clauses/CENSUS.json";
    let mut census: serde_json::Value =
        serde_json::from_slice(changed.file(path).unwrap()).unwrap();
    census["subjects"][0]["subject"] = serde_json::json!("value_minus");
    changed
        .replace_after_acquisition(path, serde_json::to_vec(&census).unwrap())
        .unwrap();
    assert_eq!(changed.recheck_census(), Err(CorpusRefusal::Census));
    tracing::info!(target: "vectors::keyed_fixed_corpus", records = total, "S8B_RECORDS");
    tracing::info!(target: "vectors::keyed_fixed_corpus", parent_only, "S8B_PARENT_ONLY");
    report("fixed_corpus_exact_native_pairs", started, total);
}

#[test]
fn fixed_corpus_history_closure() {
    trace();
    let started = Instant::now();
    let packet = Packet::load(CaptureFamily::Histories).unwrap();
    assert_eq!(packet.rebuild_histories(), Ok(6));
    report("fixed_corpus_history_closure", started, 168);
}

#[test]
fn fixed_corpus_resources() {
    trace();
    let started = Instant::now();
    let policy = Packet::load(CaptureFamily::Policy).unwrap();
    let floor_15: serde_json::Value =
        serde_json::from_slice(policy.file("policy/subject-00003.json").unwrap()).unwrap();
    let floor_14: serde_json::Value =
        serde_json::from_slice(policy.file("policy/subject-00006.json").unwrap()).unwrap();
    assert_eq!(floor_15["subject"], serde_json::json!({"floor":15}));
    assert_eq!(floor_14["subject"], serde_json::json!({"floor":14}));
    assert_eq!(floor_15["response"]["observed_layer"], "accepted");
    let refused: serde_json::Value =
        serde_json::from_str(floor_14["response"]["observed_detail"].as_str().unwrap()).unwrap();
    assert_eq!(refused["route"], "direct-block");
    assert_eq!(refused["relay_allowed"], false);
    let scaling = Packet::load(CaptureFamily::Scaling).unwrap();
    assert_eq!(
        replay_typed_resources(&scaling, 1299, "subject-08148", None),
        Ok(())
    );
    assert_eq!(
        replay_typed_resources(&scaling, 1300, "subject-12052", Some(12049)),
        Err(KeyedFixedConstructionError::HandleCountExceedsPolicy {
            offered: 1300,
            maximum: 1299,
        })
    );
    let typed: serde_json::Value =
        serde_json::from_slice(scaling.file("scaling/typed-weight-1300.json").unwrap()).unwrap();
    assert_eq!(typed["members"], 1300);
    assert_eq!(typed["maximum"], 1299);
    assert_eq!(typed["submitted"], false);
    let structural: serde_json::Value =
        serde_json::from_slice(scaling.file("scaling/subject-12052.json").unwrap()).unwrap();
    assert_eq!(structural["subject"], "structural_weight1300");
    let refused: serde_json::Value =
        serde_json::from_str(structural["response"]["observed_detail"].as_str().unwrap()).unwrap();
    assert_eq!(refused["route"], "direct-block");
    assert_eq!(refused["relay_allowed"], false);
    for (ordinal, members) in [
        (9, 1_u64),
        (18, 2),
        (774, 251),
        (1533, 252),
        (2295, 253),
        (4248, 650),
        (8148, 1299),
        (12052, 1300),
    ] {
        let row: serde_json::Value = serde_json::from_slice(
            scaling
                .file(&format!("scaling/subject-{ordinal:05}.json"))
                .unwrap(),
        )
        .unwrap();
        let raw: Vec<u8> = serde_json::from_value(row["submitted"].clone()).unwrap();
        let tx = TargetTransaction::decode(&raw).unwrap();
        let weight = u64::try_from(tx.encode_without_witness().len()).unwrap() * 3
            + u64::try_from(raw.len()).unwrap();
        assert_eq!(row["response"]["resources"]["transaction_weight"], weight);
        let worst = if members < 252 { 1077 } else { 1085 } + 307 * members;
        assert!(weight <= worst && worst - weight <= 2);
    }
    report("fixed_corpus_resources", started, 13);
}

#[test]
fn fixed_corpus_reading_measurements() {
    trace();
    let started = Instant::now();
    let packet = Packet::load(CaptureFamily::Histories).unwrap();
    assert_eq!(packet.check_retained_measurements(), Ok(504));
    let timing = packet.file("histories/phase-timing.jsonl").unwrap();
    let first_line = timing.iter().position(|byte| *byte == b'\n').unwrap();
    assert_eq!(
        packet.check_measurements(&timing[first_line + 1..]),
        Err(CorpusRefusal::Measurement)
    );
    let scaling = Packet::load(CaptureFamily::Scaling).unwrap();
    assert_eq!(scaling.check_retained_measurements(), Ok(12052));
    report("fixed_corpus_reading_measurements", started, 12556);
}
