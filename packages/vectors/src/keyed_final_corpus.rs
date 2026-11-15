//! Immutable compact projections of the sealed final native captures.
//! Acquisition checks precede semantic interpretation; this is conformance only.
use crate::keyed_final_capture::{CaptureManifest, manifest_digest};
use serde_json::Value;
use std::io::{Read, Write};
use target_elements_conformance::constructor::tagged::sha256;
#[path = "keyed_final_corpus/history.rs"]
mod history;
#[path = "keyed_final_corpus/reproduction.rs"]
mod reproduction;

// These target-local bounds end hangs and are sized at ten times the largest measured unit or more.
const CHILD_HANG_SECONDS: u32 = 120;
const FAMILY_HANG_SECONDS: u32 = 900;

const CENSUS: &str = include_str!("../fixtures/keyed-final-native/CENSUS.json");
const CENSUS_PIN: &str = "05101e74826f1291348b06f25efe06fb97ff7090bbaaae40d184456a398fb614";
const FILES: [(&str, &[u8]); 6] = [
    (
        "birth",
        include_bytes!("../fixtures/keyed-final-native/birth.json.lzma"),
    ),
    (
        "continuation",
        include_bytes!("../fixtures/keyed-final-native/continuation.json.lzma"),
    ),
    (
        "commitment",
        include_bytes!("../fixtures/keyed-final-native/commitment.json.lzma"),
    ),
    (
        "cost",
        include_bytes!("../fixtures/keyed-final-native/cost.json.lzma"),
    ),
    (
        "funding",
        include_bytes!("../fixtures/keyed-final-native/funding.json.lzma"),
    ),
    (
        "scaling",
        include_bytes!("../fixtures/keyed-final-native/scaling.json.lzma"),
    ),
];

#[derive(Clone, Debug, PartialEq, Eq)]
enum Refusal {
    Encoding,
    Bound,
    FixtureDigest,
    ManifestDigest,
    SubjectDigest,
    Census,
}

fn hex(bytes: &[u8]) -> String {
    crate::maturity_observer::encode_hex(bytes)
}
fn bytes(value: &Value) -> Result<Vec<u8>, Refusal> {
    crate::maturity_observer::decode_hex(value.as_str().ok_or(Refusal::Encoding)?)
        .map_err(|_| Refusal::Encoding)
}
fn check_digest(data: &[u8], expected: &Value) -> Result<(), Refusal> {
    if Some(hex(&sha256(data)).as_str()) != expected.as_str() {
        return Err(Refusal::FixtureDigest);
    }
    Ok(())
}
fn load(target: usize, case: usize) -> Result<Value, Refusal> {
    let _span = tracing::info_span!("load_final_corpus", target).entered();
    if hex(&sha256(CENSUS.as_bytes())) != CENSUS_PIN {
        return Err(Refusal::Census);
    }
    let census: Value = serde_json::from_str(CENSUS).map_err(|_| Refusal::Encoding)?;
    let row = &census["files"][target];
    let (name, compressed) = FILES[target];
    if row["name"] != format!("{name}.json.lzma") || row["bytes"] != compressed.len() {
        return Err(Refusal::Census);
    }
    check_digest(compressed, &row["sha256"])?;
    let member = &row["cases"][case];
    let offset = usize::try_from(member["offset"].as_u64().ok_or(Refusal::Bound)?)
        .map_err(|_| Refusal::Bound)?;
    let length = usize::try_from(member["bytes"].as_u64().ok_or(Refusal::Bound)?)
        .map_err(|_| Refusal::Bound)?;
    let compressed = compressed
        .get(offset..offset.checked_add(length).ok_or(Refusal::Bound)?)
        .ok_or(Refusal::Bound)?;
    let bound = member["decoded_bytes"].as_u64().ok_or(Refusal::Bound)?;
    if bound > 32_000_000 {
        return Err(Refusal::Bound);
    }
    let mut decoder = lzma_rust2::LzmaReader::new_mem_limit(compressed, 16 * 1024, None)
        .map_err(|_| Refusal::Encoding)?;
    decoder.set_relaxed_end_condition(false);
    let mut raw = Vec::new();
    decoder
        .take(bound + 1)
        .read_to_end(&mut raw)
        .map_err(|_| Refusal::Encoding)?;
    if raw.len() as u64 != bound {
        return Err(Refusal::Bound);
    }
    check_digest(&raw, &member["decoded_sha256"])?;
    let corpus: Value = serde_json::from_slice(&raw).map_err(|_| Refusal::Encoding)?;
    Ok(corpus)
}

fn manifest(case: &Value) -> Result<CaptureManifest, Refusal> {
    let manifest = decoded_manifest(case)?;
    if hex(&manifest_digest(&manifest).map_err(|_| Refusal::Encoding)?) != case["manifest_digest"] {
        return Err(Refusal::ManifestDigest);
    }
    Ok(manifest)
}

// Semantic children already authenticate the complete compact case in `load`.
// Keep canonical manifest reserialization in the dedicated acquisition child.
fn decoded_manifest(case: &Value) -> Result<CaptureManifest, Refusal> {
    let mut value = case["manifest"].clone();
    let mut submitted = Vec::new();
    for row in value["rows"].as_array_mut().ok_or(Refusal::Encoding)? {
        submitted.push(bytes(&row["submitted"])?);
        row["submitted"] = Value::Array(Vec::new());
    }
    let mut manifest: CaptureManifest =
        serde_json::from_value(value).map_err(|_| Refusal::Encoding)?;
    for (row, raw) in manifest.rows.iter_mut().zip(submitted) {
        row.submitted = raw;
        if sha256(&row.submitted) != row.digest {
            return Err(Refusal::SubjectDigest);
        }
    }
    Ok(manifest)
}

fn verify_case(target: usize, index: usize) {
    let corpus = load(target, index).unwrap();
    let census: Value = serde_json::from_str(CENSUS).unwrap();
    let case = &corpus;
    let expected = &census["files"][target]["cases"][index];
    assert_eq!(case["archive_digest"], expected["archive_digest"]);
    assert_eq!(case["manifest_digest"], expected["manifest_digest"]);
    let accepted = manifest(case).unwrap();
    manifest_coordinates(&accepted);
    assert_eq!(
        accepted.rows.len() as u64,
        expected["rows"].as_u64().unwrap()
    );
    joins(case, &accepted, target);
    if target != 0 {
        return;
    }
    // Each refusal has its own accepted control and exactly one changed coordinate.
    let mut wrong_subject = case.clone();
    let mut changed = bytes(&wrong_subject["manifest"]["rows"][0]["submitted"]).unwrap();
    changed[0] ^= 1;
    wrong_subject["manifest"]["rows"][0]["submitted"] = hex(&changed).into();
    assert_eq!(manifest(&wrong_subject), Err(Refusal::SubjectDigest));
    let mut wrong_evidence = case.clone();
    wrong_evidence["manifest_digest"] = "00".repeat(32).into();
    assert_eq!(manifest(&wrong_evidence), Err(Refusal::ManifestDigest));
}

fn manifest_coordinates(manifest: &CaptureManifest) {
    use crate::keyed_final_capture::{CaptureBoundary, CaptureSchema, schemas};
    use std::collections::{BTreeMap, BTreeSet};
    assert_eq!(manifest.schema, 1);
    let keys = [[42; 32], crate::keyed_final_manifest::SECOND_KEY];
    // The retained corpus carries the freeze of its capture, not the current one.
    let freezes: BTreeMap<_, _> = keys
        .into_iter()
        .map(|key| {
            (
                key,
                crate::keyed_final_native::retained_capture_freeze_digest(key).unwrap(),
            )
        })
        .collect();
    let expected: BTreeSet<_> = schemas(manifest.target)
        .into_iter()
        .flat_map(|schema| keys.into_iter().map(move |key| (schema, key)))
        .collect();
    let mut identities = BTreeSet::new();
    let mut digests = BTreeSet::new();
    for row in &manifest.rows {
        assert!(identities.insert((row.schema, row.key)));
        assert!(digests.insert(row.digest));
        assert_eq!(row.freeze, freezes[&row.key]);
        assert_eq!(row.boundary, row.schema.boundary());
        assert_eq!(row.expected, row.schema.outcome());
        assert_eq!(
            row.sealed_control,
            row.boundary == CaptureBoundary::Acceptance
                && !matches!(
                    row.schema,
                    CaptureSchema::LaterParent
                        | CaptureSchema::KeyPathControl
                        | CaptureSchema::ScalingMerge(1565)
                        | CaptureSchema::FundingPoint(_)
                )
        );
    }
    assert_eq!(identities, expected);
}

#[allow(
    clippy::too_many_lines,
    reason = "Keep the independent observation mutations beside the same accepted native binding."
)]
fn joins(case: &Value, manifest: &CaptureManifest, target: usize) {
    use crate::final_native::{JoinRefusal, bind_native};
    use crate::keyed_final_native::{
        CaptureProvenance, ProvenanceRefusal, bind_capture_provenance, capture_freeze_digest,
        retained_capture_freeze_digest,
    };
    let rows: std::collections::BTreeMap<_, _> = manifest
        .rows
        .iter()
        .map(|row| (hex(&row.digest), row))
        .collect();
    let mut count = 0;
    let baseline = load(if target < 4 { 0 } else { 4 }, 0).unwrap();
    assert_eq!(case["run"], baseline["run"]);
    let run_provenance: CaptureProvenance =
        serde_json::from_value(baseline["sessions"][0]["provenance"].clone()).unwrap();
    for session in case["sessions"].as_array().unwrap() {
        let provenance: CaptureProvenance =
            serde_json::from_value(session["provenance"].clone()).unwrap();
        let key: [u8; 32] = serde_json::from_value(session["key"].clone()).unwrap();
        assert_eq!(provenance.key, Some(key));
        let mut expected = run_provenance.clone();
        expected.key = Some(key);
        expected.freeze = Some(retained_capture_freeze_digest(key).unwrap());
        expected.protocol = Some(target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA);
        // The binder checks field presence and the executor pin; other run-wide fields match the run's first session.
        bind_capture_provenance(&expected, &provenance).unwrap();
        // Paths re-frozen since this capture give the current freeze another digest,
        // which this retained session refuses at exactly that coordinate.
        let mut current = expected;
        current.freeze = Some(capture_freeze_digest(key).unwrap());
        assert_eq!(
            bind_capture_provenance(&current, &provenance),
            Err(ProvenanceRefusal::Freeze)
        );
        let handshake: target_elements_conformance::protocol::ExecutorHandshake =
            serde_json::from_value(session["handshake"].clone()).unwrap();
        assert_eq!(Some(handshake.protocol_schema), provenance.protocol);
        assert_eq!(Some(handshake.node_version), provenance.executable);
        assert_eq!(handshake.framework_revision, provenance.framework);
        assert_eq!(session["handshake"], baseline["sessions"][0]["handshake"]);
        let diagnostic = session["diagnostic"].as_str().unwrap();
        for observation in session["observations"].as_array().unwrap() {
            let row = rows[observation["digest"].as_str().unwrap()];
            let control = bind_native(row, observation, diagnostic).unwrap_or_else(|error| {
                panic!(
                    "{:?}: {error:?}; relay={} verdict={}",
                    row.schema, observation["relay_detail"], observation["verdict_reason"]
                )
            });
            assert_eq!(control.subject, row.digest);
            count += 1;
            if row.schema == crate::keyed_final_capture::CaptureSchema::ScalingMerge(1565) {
                assert!(control.structural);
                let mut changed = row.clone();
                changed.boundary = crate::keyed_final_capture::CaptureBoundary::Acceptance;
                assert_eq!(
                    bind_native(&changed, observation, diagnostic),
                    Err(JoinRefusal::Structural)
                );
                let mut changed = row.clone();
                changed.sealed_control = true;
                assert_eq!(
                    bind_native(&changed, observation, diagnostic),
                    Err(JoinRefusal::Structural)
                );
            }
            if target == 0 {
                for field in ["script_bytes", "initial_stack_items"] {
                    let mut changed = observation.clone();
                    changed["resources"][field] = 1.into();
                    assert_eq!(
                        bind_native(row, &changed, diagnostic),
                        Err(JoinRefusal::Dimension)
                    );
                }
                let mut changed = observation.clone();
                changed["digest"] = "00".repeat(32).into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Subject)
                );
                let mut changed = observation.clone();
                changed["route"] = "another-route".into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Route)
                );
                let mut changed = observation.clone();
                changed["template_member"] = (!control.relay_allowed).into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Route)
                );
                let mut changed = observation.clone();
                changed["inclusion_in_produced"] = false.into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Route)
                );
                let mut changed = observation.clone();
                changed["verdict_reason"] = "another-verdict".into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Reason)
                );
                let mut changed = observation.clone();
                changed["relay_detail"] = "another-class".into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Reason)
                );
                let mut changed = observation.clone();
                changed["resources"]["transaction_weight"] = 1.into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Weight)
                );
                let mut changed = observation.clone();
                changed["policy"]["unit"] = "bytes".into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Policy)
                );
            }
            if row.schema == crate::keyed_final_capture::CaptureSchema::LaterBorn {
                let mut changed = observation.clone();
                changed["block_script_error"] = "unexpected-block-error".into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Block)
                );
            }
            if matches!(
                row.schema,
                crate::keyed_final_capture::CaptureSchema::FundingPoint(1 | 13)
                    | crate::keyed_final_capture::CaptureSchema::Flagged
            ) {
                let tx: elements::Transaction =
                    elements::encode::deserialize(&row.submitted).unwrap();
                let changed = diagnostic.replace(&tx.txid().to_string(), &"00".repeat(32));
                assert_ne!(changed, diagnostic);
                assert_eq!(
                    bind_native(row, observation, &changed),
                    Err(JoinRefusal::Block)
                );
            }
            if matches!(
                row.schema,
                crate::keyed_final_capture::CaptureSchema::FundingPoint(1 | 2 | 13 | 14)
                    | crate::keyed_final_capture::CaptureSchema::Flagged
            ) {
                let mut changed = observation.clone();
                changed["block_script_error"] = "another-error".into();
                assert_eq!(
                    bind_native(row, &changed, diagnostic),
                    Err(JoinRefusal::Block)
                );
                assert_eq!(bind_native(row, observation, ""), Err(JoinRefusal::Block));
            }
        }
    }
    assert_eq!(count, manifest.rows.len());
}

#[test]
fn sealed_final_manifest_digests() {
    family_process("digests", 0, 0);
}

fn verify_manifests() {
    let start = std::time::Instant::now();
    for (target, cases) in [1, 1, 1, 1, 24, 8].into_iter().enumerate() {
        for index in 0..cases {
            let started = std::time::Instant::now();
            let output = std::process::Command::new("/usr/bin/timeout")
                .arg("--kill-after=1")
                .arg(CHILD_HANG_SECONDS.to_string())
                .arg(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "keyed_final_corpus::manifest_child",
                    "--nocapture",
                ])
                .env("TRIPOD_FINAL_CASE", format!("{target}:{index}"))
                .output()
                .unwrap();
            let wall = started.elapsed().as_secs_f64();
            writeln!(
                std::io::stdout().lock(),
                "final-manifest target={target} case={index} wall={wall:.6}s guard={CHILD_HANG_SECONDS}s report={}",
                wall >= f64::from(CHILD_HANG_SECONDS) / 10.0
            )
            .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        }
    }
    writeln!(
        std::io::stdout().lock(),
        "final-manifests wall={:.6}s",
        start.elapsed().as_secs_f64()
    )
    .unwrap();
}

#[test]
#[ignore = "bounded parent selects one archive case"]
fn manifest_child() {
    let subject = std::env::var("TRIPOD_FINAL_CASE").unwrap();
    let (target, case) = subject.split_once(':').unwrap();
    verify_case(target.parse().unwrap(), case.parse().unwrap());
}

macro_rules! small_case {
    ($name:ident, $target:expr, $first:expr) => {
        #[test]
        fn $name() {
            family_process("small", $target, $first);
        }
    };
}
small_case!(final_history_birth, 0, 0);
small_case!(final_history_continuation, 1, 0);
small_case!(final_history_commitment, 2, 0);
small_case!(final_history_cost, 3, 0);
small_case!(final_history_funding_source, 4, 0);
small_case!(final_history_funding_merge, 4, 12);
fn small_cases(target: usize, first: usize) {
    let end = if target == 4 { first + 12 } else { 1 };
    for case in first..end {
        for key in 0..2 {
            semantic_process(target, case, key, "history", 0);
            semantic_process(target, case, key, "reproduction", 0);
        }
    }
}

macro_rules! wide_case {
    ($name:ident, $case:expr, $count:expr) => {
        mod $name {
            #[test]
            fn history() {
                super::family_process("history", $case, $count);
            }
            #[test]
            fn reproduction_key_0() {
                super::family_process("reproduction0", $case, $count);
            }
            #[test]
            fn reproduction_key_1() {
                super::family_process("reproduction1", $case, $count);
            }
            #[test]
            fn validators_key_0() {
                super::family_process("validators0", $case, $count);
            }
            #[test]
            fn validators_key_1() {
                super::family_process("validators1", $case, $count);
            }
            #[test]
            fn rules_key_0() {
                super::family_process("rules0", $case, $count);
            }
            #[test]
            fn rules_key_1() {
                super::family_process("rules1", $case, $count);
            }
        }
    };
}
wide_case!(final_history_zero, 0, 0);
wide_case!(final_history_one, 1, 1);
wide_case!(final_history_two, 2, 2);
wide_case!(final_history_252, 3, 252);
wide_case!(final_history_253, 4, 253);
wide_case!(final_history_782, 5, 782);
wide_case!(final_history_1564, 6, 1564);
wide_case!(final_history_structural_1565, 7, 1565);

static FAMILY_SLOTS: (std::sync::Mutex<usize>, std::sync::Condvar) =
    (std::sync::Mutex::new(8), std::sync::Condvar::new());
struct FamilySlot;
impl FamilySlot {
    fn acquire() -> Self {
        let (slots, available) = &FAMILY_SLOTS;
        let mut slots = slots.lock().unwrap();
        while *slots == 0 {
            slots = available.wait(slots).unwrap();
        }
        *slots -= 1;
        Self
    }
}
impl Drop for FamilySlot {
    fn drop(&mut self) {
        let (slots, available) = &FAMILY_SLOTS;
        *slots.lock().unwrap() += 1;
        available.notify_one();
    }
}

fn family_process(kind: &str, case: usize, count: usize) {
    let started = std::time::Instant::now();
    let _slot = FamilySlot::acquire();
    let queue = started.elapsed().as_secs_f64();
    tracing::info!(kind, case, count, queue, "final family slot acquired");
    let output = std::process::Command::new("/usr/bin/timeout")
        .arg("--kill-after=1")
        .arg(FAMILY_HANG_SECONDS.to_string())
        .arg(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "keyed_final_corpus::family_child",
            "--nocapture",
        ])
        .env("TRIPOD_FINAL_FAMILY", format!("{kind}:{case}:{count}"))
        .output()
        .unwrap();
    let wall = started.elapsed().as_secs_f64();
    writeln!(
        std::io::stdout().lock(),
        "final-family kind={kind} case={case} count={count} queue={queue:.6}s wall={wall:.6}s guard={FAMILY_HANG_SECONDS}s report={}\n{}",
        wall >= f64::from(FAMILY_HANG_SECONDS) / 10.0,
        String::from_utf8_lossy(&output.stdout)
    )
    .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "bounded parent selects one family"]
fn family_child() {
    let args = std::env::var("TRIPOD_FINAL_FAMILY").unwrap();
    let args: Vec<_> = args.split(':').collect();
    let case: usize = args[1].parse().unwrap();
    let count: usize = args[2].parse().unwrap();
    match args[0] {
        "digests" => verify_manifests(),
        "small" => small_cases(case, count),
        "history" => {
            history::check_wide(case);
        }
        "reproduction0" | "reproduction1" => {
            let key = usize::from(args[0] == "reproduction1");
            for chunk in 0..=(count / 128) {
                semantic_process(5, case, key, "reproduction", chunk);
            }
        }
        "validators0" | "validators1" => {
            let key = usize::from(args[0] == "validators1");
            history::check_families(case, key, false);
        }
        "rules0" | "rules1" => {
            let key = usize::from(args[0] == "rules1");
            history::check_families(case, key, true);
        }
        _ => panic!("named family"),
    }
}

fn semantic_worker<T: Send + 'static>(
    case: usize,
    key: usize,
    phase: &'static str,
    part: usize,
    work: impl FnOnce() -> T + Send + 'static,
) -> T {
    let started = std::time::Instant::now();
    let (send, receive) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || assert!(send.send(work()).is_ok()));
    let result = receive.recv_timeout(std::time::Duration::from_secs(u64::from(
        CHILD_HANG_SECONDS,
    )));
    let wall = started.elapsed().as_secs_f64();
    tracing::info!(
        case,
        key,
        phase,
        part,
        wall,
        "final bounded worker completed"
    );
    writeln!(std::io::stdout().lock(), "final-semantic case={case} key={key} phase={phase} part={part} worker=thread wall={wall:.6}s guard={CHILD_HANG_SECONDS}s report={}", wall >= f64::from(CHILD_HANG_SECONDS) / 10.0).unwrap();
    let result = result.expect("bounded semantic worker completes without panic or timeout");
    worker.join().unwrap();
    result
}

fn semantic_process(target: usize, case: usize, key: usize, phase: &str, part: usize) {
    let started = std::time::Instant::now();
    let output = std::process::Command::new("/usr/bin/timeout")
        .arg("--kill-after=1")
        .arg(CHILD_HANG_SECONDS.to_string())
        .arg(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "keyed_final_corpus::semantic_child",
            "--nocapture",
        ])
        .env_clear()
        .env(
            "TRIPOD_FINAL_SEMANTIC",
            format!("{target}:{case}:{key}:{phase}:{part}"),
        )
        .output()
        .unwrap();
    let wall = started.elapsed().as_secs_f64();
    writeln!(std::io::stdout().lock(), "final-semantic target={target} case={case} key={key} phase={phase} part={part} wall={wall:.6}s guard={CHILD_HANG_SECONDS}s report={}", wall >= f64::from(CHILD_HANG_SECONDS) / 10.0).unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}

#[test]
#[ignore = "bounded parent supplies a single immutable native case"]
fn semantic_child() {
    let args = std::env::var("TRIPOD_FINAL_SEMANTIC").unwrap();
    let args: Vec<_> = args.split(':').collect();
    let target: usize = args[0].parse().unwrap();
    let case: usize = args[1].parse().unwrap();
    let key: usize = args[2].parse().unwrap();
    let part: usize = args[4].parse().unwrap();
    let corpus = load(target, case).unwrap();
    let case = &corpus;
    match args[3] {
        "further-key" => history::further_key(),
        "history" => history::check(case, key),
        "reproduction" => reproduction::check(case, key, part),
        _ => panic!("named phase"),
    }
}

#[test]
fn further_supplied_key_requires_its_own_reading() {
    semantic_process(0, 0, 0, "further-key", 0);
}
