//! Six focused offline checks of the sealed fixed-form campaign projection and epochs.

#[path = "../src/keyed_fixed_corpus/fixture.rs"]
mod keyed_fixed_corpus;
#[path = "keyed_fixed_capture/native.rs"]
#[allow(dead_code)] // The packet control reuses the actual producer's local campaign route.
mod native;

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
use vectors::keyed_fixed_manifest::{CaptureEpoch, FreezeManifest, FreezeRefusal, SourceVersion};

const POLICY_ASSET: [u8; 32] = [
    0x23, 0x0f, 0x4f, 0x5d, 0x4b, 0x7c, 0x6f, 0xa8, 0x45, 0x80, 0x6e, 0xe4, 0xf6, 0x77, 0x13, 0x45,
    0x9e, 0x1b, 0x69, 0xe8, 0xe6, 0x0f, 0xce, 0xe2, 0xe4, 0x94, 0x0c, 0x7a, 0x0d, 0x5d, 0xe1, 0xb2,
];

fn parameters(key: u8, value: u64) -> KeyedFixedParameters {
    KeyedFixedParameters {
        key: [key; 32],
        asset: POLICY_ASSET,
        value: KeyedFixedValue::new(value).unwrap(),
    }
}

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

fn mock_hex(bytes: &[u8]) -> String {
    vectors::maturity_observer::encode_hex(bytes)
}

fn mock_digest(bytes: &[u8]) -> String {
    mock_hex(&target_elements_conformance::constructor::tagged::sha256(
        bytes,
    ))
}

fn mock_listing(files: &std::collections::BTreeMap<String, Vec<u8>>, prefix: &str) -> Vec<u8> {
    use std::fmt::Write as _;
    let mut listing = String::new();
    for (path, bytes) in files {
        if let Some(relative) = path.strip_prefix(prefix) {
            let _ = writeln!(listing, "{}  {relative}", mock_digest(bytes));
        }
    }
    listing.into_bytes()
}

fn mock_compressed(
    files: &std::collections::BTreeMap<String, Vec<u8>>,
) -> (Vec<u8>, serde_json::Value) {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let pairs = files
        .iter()
        .map(|(path, bytes)| (path, vectors::maturity_observer::encode_hex(bytes)))
        .collect::<Vec<_>>();
    let decoded = serde_json::to_vec(&pairs).unwrap();
    let decoded_bytes = decoded.len();
    let decoded_digest = mock_digest(&decoded);
    let mut process = Command::new("python3")
        .args(["-c", "import lzma,sys; sys.stdout.buffer.write(lzma.compress(sys.stdin.buffer.read(),format=lzma.FORMAT_ALONE,preset=6))"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = process.stdin.take().unwrap();
    let writer = std::thread::spawn(move || input.write_all(&decoded));
    let output = process.wait_with_output().unwrap();
    writer.join().unwrap().unwrap();
    assert!(output.status.success());
    let encoded = output.stdout;
    let digest = mock_digest(&encoded);
    let entry = serde_json::json!({"bytes":encoded.len(),"sha256":digest,"paths":pairs.len(),
        "members":[{"offset":0,"bytes":encoded.len(),"sha256":digest,"paths":pairs.len(),
            "decoded_bytes":decoded_bytes,"decoded_sha256":decoded_digest}]});
    (encoded, entry)
}

#[allow(clippy::too_many_lines)] // One mock packet is projected through the same three indexed fixture members.
fn mocked_s8c_packet() -> (Packet, Packet) {
    use std::collections::BTreeMap;
    let temp = tempfile::tempdir().unwrap();
    let run = "mock-s8c";
    let mut family_files = BTreeMap::new();
    for (name, family) in [
        ("sponsor", CaptureFamily::Sponsor),
        ("legacy-copy", CaptureFamily::LegacyCopy),
    ] {
        let directory = temp.path().join(name);
        std::fs::create_dir(&directory).unwrap();
        native::mocked_campaign(family, &directory);
        let census = native::s8c_census(family);
        std::fs::write(
            directory.join("CENSUS.json"),
            serde_json::to_vec(&census).unwrap(),
        )
        .unwrap();
        native::check_s8c_census(&directory, family);
        if family == CaptureFamily::Sponsor {
            let mut stale = census.clone();
            stale["primary"] = serde_json::json!(5);
            std::fs::write(
                directory.join("CENSUS.json"),
                serde_json::to_vec(&stale).unwrap(),
            )
            .unwrap();
            assert!(
                std::panic::catch_unwind(|| native::check_s8c_census(&directory, family)).is_err(),
                "the producer refuses a stale sponsor primary count"
            );
            std::fs::write(
                directory.join("CENSUS.json"),
                serde_json::to_vec(&census).unwrap(),
            )
            .unwrap();
        }
        let mut files = BTreeMap::new();
        for entry in std::fs::read_dir(&directory).unwrap() {
            let entry = entry.unwrap();
            files.insert(
                entry.file_name().to_string_lossy().into_owned(),
                std::fs::read(entry.path()).unwrap(),
            );
        }
        if family == CaptureFamily::LegacyCopy {
            let value: serde_json::Value =
                serde_json::from_slice(&files["legacy-copy-process.json"]).unwrap();
            let mut process = value;
            process["command"] = serde_json::json!([
                "/usr/bin/timeout",
                "--kill-after=5",
                "60",
                "/usr/bin/cp",
                "/capture/legacy-public-input.transaction",
                "/capture/legacy-public-copy.transaction"
            ]);
            files.insert(
                "legacy-copy-process.json".into(),
                serde_json::to_vec(&process).unwrap(),
            );
        }
        files.insert("observed-provenance.json".into(), serde_json::to_vec(&serde_json::json!({
            "executable_digest":vec![2_u8;32],"executor_digest":vec![3_u8;32],"dependency_lock_digest":vec![4_u8;32],
            "daemon_digest":vec![5_u8;32],"zk_library_digest":vec![6_u8;32],"zk_library_loading":"present-not-loaded"
        })).unwrap());
        files.insert("executor-handshake.json".into(), serde_json::to_vec(&serde_json::json!({
            "protocol_schema":9,"adapter_name":"elements-native-executor","framework_revision":"mock",
            "intended_executed_tip":"mock"
        })).unwrap());
        files.insert(
            "container-final-state.json".into(),
            serde_json::to_vec(&serde_json::json!([
                {"Image":"mock-image"}
            ]))
            .unwrap(),
        );
        let prefixed = files
            .into_iter()
            .map(|(path, bytes)| (format!("{run}/{name}/{path}"), bytes))
            .collect::<BTreeMap<_, _>>();
        family_files.insert(name, prefixed);
    }
    let mut preparation = BTreeMap::new();
    preparation.insert(
        format!("{run}/build/prepared.json"),
        serde_json::to_vec(&serde_json::json!({
            "capture_binary":mock_hex(&[2;32]),"executor":mock_hex(&[3;32]),
            "dependency_lock":mock_hex(&[4;32]),"daemon_digest":mock_hex(&[5;32]),
            "framework":"mock","zk_library_digest":mock_hex(&[6;32]),"image":"mock-image"
        }))
        .unwrap(),
    );
    preparation.insert(
        format!("{run}/prepare.log"),
        format!("KFF_PREPARE_DONE run={run} exit=0\n").into_bytes(),
    );
    preparation.insert(
        format!("{run}/campaign.log"),
        format!("KFF_CAMPAIGN_DONE run={run} exit=0\n").into_bytes(),
    );
    let build_listing = mock_listing(&preparation, &format!("{run}/build/"));
    preparation.insert(format!("{run}/build/retention.sha256"), build_listing);
    let root_listing = mock_listing(
        &preparation
            .iter()
            .filter(|(path, _)| {
                path.as_str() == format!("{run}/prepare.log")
                    || path.as_str() == format!("{run}/campaign.log")
            })
            .map(|(path, bytes)| (path.clone(), bytes.clone()))
            .collect(),
        &format!("{run}/"),
    );
    preparation.insert(format!("{run}/retention.sha256"), root_listing);
    for (name, files) in &mut family_files {
        let listing = mock_listing(files, &format!("{run}/{name}/"));
        files.insert(format!("{run}/{name}/retention.sha256"), listing);
    }
    let seal = [
        "retention.sha256",
        "build/retention.sha256",
        "sponsor/retention.sha256",
        "legacy-copy/retention.sha256",
    ]
    .map(|path| {
        let bytes = preparation
            .get(&format!("{run}/{path}"))
            .or_else(|| {
                family_files
                    .values()
                    .find_map(|files| files.get(&format!("{run}/{path}")))
            })
            .unwrap();
        format!("{}  {path}\n", mock_digest(bytes))
    })
    .concat();
    preparation.insert(
        format!("{run}/campaign-retention.sha256"),
        seal.into_bytes(),
    );
    let mut fixtures = serde_json::Map::new();
    let mut blobs = Vec::new();
    for (name, files) in [
        ("preparation", &preparation),
        ("sponsor", &family_files["sponsor"]),
        ("legacy-copy", &family_files["legacy-copy"]),
    ] {
        let (blob, entry) = mock_compressed(files);
        fixtures.insert(name.to_string(), entry);
        blobs.push(blob);
    }
    let index = serde_json::to_vec(&serde_json::json!({"schema":1,"preset":6,"run":run,
        "fixtures":fixtures,"archive_addressed":[]}))
    .unwrap();
    let pin = mock_digest(&index);
    Packet::load_s8c(&index, &pin, [&blobs[0], &blobs[1], &blobs[2]]).unwrap()
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

fn linked(
    q: KeyedFixedParameters,
    operation: KeyedFixedOperationId,
) -> linker::keyed_fixed_bundle::LinkedKeyedFixedBundle {
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
        vec![operation],
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

fn linked_merge(q: KeyedFixedParameters) -> linker::keyed_fixed_bundle::LinkedKeyedFixedBundle {
    linked(q, KeyedFixedOperationId::Merge)
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
    let sources = clauses.epoch_sources().unwrap();
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
#[allow(clippy::too_many_lines)] // The four earlier encodings and three new identities share one authenticated source catalog.
fn fixed_corpus_epoch_sources() {
    trace();
    let started = Instant::now();
    let clauses = Packet::load(CaptureFamily::Clauses).unwrap();
    let policy = Packet::load(CaptureFamily::Policy).unwrap();
    let sources = clauses.epoch_sources().unwrap();
    let encodings = [
        clauses.file("clauses/freeze-42-60.bin").unwrap(),
        clauses.file("clauses/freeze-43-1000.bin").unwrap(),
        policy.file("policy/freeze-42-14.bin").unwrap(),
        policy.file("policy/freeze-42-15.bin").unwrap(),
    ];
    for encoding in encodings {
        let historical = FreezeManifest::reconstruct_epoch_freeze(
            CaptureEpoch::CampaignThree,
            encoding,
            &sources,
        )
        .unwrap();
        let q = historical.manifest().descriptor.parameters;
        assert_eq!(
            historical.manifest().clone().validate(q),
            Err(FreezeRefusal::Inventory)
        );
        assert_eq!(
            FreezeManifest::reconstruct_epoch_freeze(CaptureEpoch::Sponsor, encoding, &sources),
            Err(FreezeRefusal::Inventory)
        );
    }
    let original = encodings[0];
    let sponsor_index = sources
        .iter()
        .position(|version| version.path == "packages/transaction/src/keyed_fixed_sponsor.rs")
        .unwrap();
    assert_eq!(
        clauses
            .source(sources[sponsor_index].path, sources[sponsor_index].digest)
            .unwrap(),
        sources[sponsor_index].bytes,
    );
    let mut missing = sources.clone();
    missing.remove(sponsor_index);
    assert_eq!(
        FreezeManifest::reconstruct_epoch_freeze(CaptureEpoch::CampaignThree, original, &missing),
        Err(FreezeRefusal::Inventory)
    );
    let mut changed_bytes = sources[sponsor_index].bytes.to_vec();
    changed_bytes[0] ^= 1;
    let mut changed = sources.clone();
    changed[sponsor_index].bytes = &changed_bytes;
    assert_eq!(
        FreezeManifest::reconstruct_epoch_freeze(CaptureEpoch::CampaignThree, original, &changed),
        Err(FreezeRefusal::Inventory)
    );
    let unknown_bytes = b"unknown snapshot";
    let mut unknown = sources.clone();
    unknown.push(SourceVersion {
        path: "packages/unknown.rs",
        digest: target_elements_conformance::constructor::tagged::sha256(unknown_bytes),
        bytes: unknown_bytes,
    });
    assert_eq!(
        FreezeManifest::reconstruct_epoch_freeze(CaptureEpoch::CampaignThree, original, &unknown),
        Err(FreezeRefusal::Inventory)
    );
    let mut unreviewed_count = original.to_vec();
    unreviewed_count[vectors::keyed_fixed_manifest::RECIPE.len() + 4] += 1;
    assert_eq!(
        FreezeManifest::reconstruct_epoch_freeze(
            CaptureEpoch::CampaignThree,
            &unreviewed_count,
            &sources
        ),
        Err(FreezeRefusal::Inventory)
    );
    let campaign =
        FreezeManifest::reconstruct_epoch_freeze(CaptureEpoch::CampaignThree, original, &sources)
            .unwrap();
    let q60 = campaign.manifest().descriptor.parameters;
    let corrected = FreezeManifest::offer(q60).unwrap().validate(q60).unwrap();
    assert_eq!(
        vectors::maturity_observer::encode_hex(&corrected.identity()),
        "bdeb5788ff13893ddb100ec5c24a19e234adbb594678a43442ec5d349c563c85"
    );
    assert_eq!(
        FreezeManifest::reconstruct_epoch_freeze(
            CaptureEpoch::Sponsor,
            corrected.encoding(),
            &sources
        ),
        Err(FreezeRefusal::Inventory)
    );
    assert_eq!(
        FreezeManifest::reconstruct_epoch_freeze(
            CaptureEpoch::SponsorCorrected,
            corrected.encoding(),
            &sources
        ),
        Ok(corrected)
    );
    let prior = [
        (
            "packages/vectors/src/keyed_fixed_capture.rs",
            "f2f41ba3367950ea16e7477a829a27fed6c0ecff81abab0c947b71318e02057c",
        ),
        (
            "packages/vectors/src/keyed_fixed_manifest.rs",
            "f9686eb4392a941ac06cc93d16674738355e4b5aeac08a26bc465900421e839b",
        ),
        (
            "packages/vectors/tests/keyed_fixed_capture.rs",
            "92abb4f41880b82f71a3cca77f74d4f3bd931cb29cb2f8c360df1c3efafc79a9",
        ),
        (
            "packages/vectors/tests/keyed_fixed_capture/native.rs",
            "bb3ada1c5b2c34c03660447ff46fc67f3bad0dcf53b4f21cda053e10368c7b83",
        ),
    ];
    for (value, expected) in [
        (
            60,
            "17a7b1da3385f79d2ec5a3587e8f35c25bba87cc760aae2c35d35e8e40c94480",
        ),
        (
            17,
            "3353d21e195b40379ec6d4923f1393e8cee785ad25587c01987da07d7775f331",
        ),
        (
            18,
            "5980c072960539d7b0c28dbab308987548fd51fc9e9514b1265aed603990a9e1",
        ),
    ] {
        let q = KeyedFixedParameters {
            value: KeyedFixedValue::new(value).unwrap(),
            ..q60
        };
        let current = FreezeManifest::offer(q).unwrap().validate(q).unwrap();
        let mut failed_encoding = current.encoding().to_vec();
        for (path, old_hex) in prior {
            let current_digest = current
                .manifest()
                .sources
                .iter()
                .find(|entry| entry.path == path)
                .unwrap()
                .digest;
            let previous: [u8; 32] = vectors::maturity_observer::decode_hex(old_hex)
                .unwrap()
                .try_into()
                .unwrap();
            let positions = failed_encoding
                .windows(32)
                .enumerate()
                .filter_map(|(index, bytes)| (bytes == current_digest).then_some(index))
                .collect::<Vec<_>>();
            assert_eq!(positions.len(), 1);
            failed_encoding[positions[0]..positions[0] + 32].copy_from_slice(&previous);
        }
        assert_eq!(
            vectors::maturity_observer::encode_hex(
                &target_elements_conformance::constructor::tagged::sha256(&failed_encoding)
            ),
            expected
        );
        assert!(
            FreezeManifest::reconstruct_epoch_freeze(
                CaptureEpoch::Sponsor,
                &failed_encoding,
                &sources
            )
            .is_ok()
        );
        assert_eq!(
            FreezeManifest::reconstruct_epoch_freeze(
                CaptureEpoch::SponsorCorrected,
                &failed_encoding,
                &sources
            ),
            Err(FreezeRefusal::Inventory)
        );
        if value == 60 {
            let missing_native = sources
                .iter()
                .copied()
                .filter(|entry| {
                    !(entry.path == prior[3].0
                        && vectors::maturity_observer::encode_hex(&entry.digest) == prior[3].1)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                FreezeManifest::reconstruct_epoch_freeze(
                    CaptureEpoch::Sponsor,
                    &failed_encoding,
                    &missing_native
                ),
                Err(FreezeRefusal::Inventory)
            );
        }
    }
    assert!(matches!(
        Packet::load_s8c(b"{}", "unreviewed-index", [&[], &[], &[]]),
        Err(CorpusRefusal::Index)
    ));
    assert_eq!(clauses.admit_s8c(), Err(CorpusRefusal::Census));
    let (sponsor, legacy) = mocked_s8c_packet();
    assert_eq!(sponsor.admit_s8c(), Ok(6));
    assert_eq!(legacy.admit_s8c(), Ok(3));
    let mut wrong_step = sponsor;
    let mut request: serde_json::Value = serde_json::from_slice(
        wrong_step
            .file("sponsor/exchange-00014-request.json")
            .unwrap(),
    )
    .unwrap();
    request["case"]["step"] = serde_json::json!("fixed-sign-14");
    wrong_step
        .replace_after_acquisition(
            "sponsor/exchange-00014-request.json",
            serde_json::to_vec(&request).unwrap(),
        )
        .unwrap();
    assert_eq!(wrong_step.admit_s8c(), Err(CorpusRefusal::Record));
    report("fixed_corpus_epoch_sources", started, 15);
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
