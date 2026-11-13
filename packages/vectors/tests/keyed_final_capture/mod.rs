//! Offline controls and explicitly ignored native capture entries.
#![allow(
    clippy::print_stdout,
    reason = "Bounded children print retained wall measurements even when stopped."
)]
#[allow(
    dead_code,
    reason = "reuse unchanged typed mock support without copying its fixture bytes"
)]
#[allow(
    clippy::duplicate_mod,
    reason = "The unchanged fixture is shared test support, not an independent oracle."
)]
#[path = "../keyed_bridge/fixture.rs"]
mod bridge_mock;
mod cost;
mod driver;
mod mock;
mod native;
pub mod retained;

use std::io::Write as _;
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::*;
use vectors::keyed_final_capture::*;
use vectors::keyed_final_native::*;

struct Recorded {
    pending: std::collections::VecDeque<(OperationStep, NativeOperationResponse)>,
    journal: Vec<(NativeOperationRequest, NativeOperationResponse)>,
    rows: Vec<CaptureRow>,
}
impl driver::Exchange for Recorded {
    fn exchange(&mut self, step: OperationStep) -> NativeOperationResponse {
        let (expected, response) = self.pending.pop_front().expect("recorded response");
        assert_eq!(
            step, expected,
            "public reconstruction must reproduce its request"
        );
        self.journal.push((
            NativeOperationRequest {
                schema: 9,
                case: step.case().clone(),
                subject: step.subject().clone(),
            },
            response.clone(),
        ));
        response
    }
    fn before_submission(&mut self, row: CaptureRow) {
        self.rows.push(row);
    }
}

fn packet() -> CaptureProvenance {
    CaptureProvenance {
        repository_commit: Some("mock build identity".into()),
        dependency_lock: Some([1; 32]),
        toolchain: Some("mock compiler and cargo".into()),
        executable: Some("mock node version".into()),
        framework: Some("1".repeat(40)),
        executor: Some(target_elements_conformance::constructor::tagged::sha256(
            include_bytes!("../../../../scripts/elements-native-executor.py"),
        )),
        protocol: Some(9),
        node: Some("mock compute node".into()),
        image: Some("mock image".into()),
        freeze: Some(capture_freeze_digest([42; 32]).unwrap()),
        key: Some([42; 32]),
        capture_binary: Some([3; 32]),
        interpreter: Some("mock Python".into()),
        daemon: Some("mock daemon".into()),
    }
}
fn handshake() -> ExecutorHandshake {
    ExecutorHandshake {
        protocol_schema: 9,
        adapter_name: "mock".into(),
        adapter_version: "mock".into(),
        framework_revision: Some("1".repeat(40)),
        node_name: "mock".into(),
        node_version: "mock node version".into(),
        binary_reported_revision: Some("1".repeat(12)),
        intended_executed_tip: Some("1".repeat(40)),
        upstream_base: Some("1".repeat(40)),
        included_local_topics: std::collections::BTreeSet::new(),
        supported_domains: [WireExecutionDomain::Tapscript].into(),
        supported_leaf_versions: [196].into(),
        capabilities: [
            ExecutorCapability::TestSponsorAuthorization,
            ExecutorCapability::TargetTransactionSubmission,
        ]
        .into(),
        confidential_funding: None,
    }
}

fn cost_control() -> (
    CaptureManifest,
    Vec<(NativeOperationRequest, NativeOperationResponse)>,
) {
    let mut rows = vec![];
    let mut journal = vec![];
    for key in [[42; 32], vectors::keyed_final_manifest::SECOND_KEY] {
        let started = std::time::Instant::now();
        let old = bridge_mock::history(key); // All eight unchanged literal pins are asserted here.
        let replay = Recorded {
            pending: old.exchanges.into(),
            journal: vec![],
            rows: vec![],
        };
        let replay = driver::controls(replay, key);
        assert!(replay.pending.is_empty());
        rows.extend(replay.rows);
        journal.extend(replay.journal);
        println!(
            "capture mock-pin-replay key={} wall={:.6}s",
            crate::common::hex_bytes(&key),
            started.elapsed().as_secs_f64()
        );
        assert!(started.elapsed().as_secs() < 10, "split/report threshold");
    }
    (
        CaptureManifest {
            schema: 1,
            target: CaptureTarget::Cost,
            rows,
        },
        journal,
    )
}

fn check(
    manifest: &CaptureManifest,
    journal: &[(NativeOperationRequest, NativeOperationResponse)],
) -> Result<(), CaptureRefusal> {
    let p = packet();
    bind_capture_manifest(
        CaptureTarget::Cost,
        &p,
        &p,
        &handshake(),
        manifest,
        manifest_digest(manifest).unwrap(),
        journal,
    )
}

fn manifest_controls(
    control: &CaptureManifest,
    journal: &[(NativeOperationRequest, NativeOperationResponse)],
) {
    type Change = fn(&mut CaptureManifest);
    let cases: &[(Change, CaptureRefusal)] = &[
        (|m| m.schema = 2, CaptureRefusal::Version),
        (|m| m.target = CaptureTarget::Birth, CaptureRefusal::Target),
        (
            |m| {
                m.rows.pop();
            },
            CaptureRefusal::MissingSchema,
        ),
        (
            |m| m.rows.push(m.rows[0].clone()),
            CaptureRefusal::DuplicateSchema,
        ),
        (
            |m| m.rows[0].schema = CaptureSchema::LaterBorn,
            CaptureRefusal::Schema,
        ),
        (|m| m.rows[0].key = [4; 32], CaptureRefusal::Key),
        (|m| m.rows[0].freeze[0] ^= 1, CaptureRefusal::Freeze),
        (
            |m| m.rows[0].boundary = CaptureBoundary::Birth,
            CaptureRefusal::Boundary,
        ),
        (
            |m| m.rows[0].expected = ObservedOutcomeLayer::ScriptPathRejection,
            CaptureRefusal::ExpectedOutcome,
        ),
        (|m| m.rows[0].sealed_control = false, CaptureRefusal::Origin),
        (
            |m| m.rows[0].submitted[0] ^= 1,
            CaptureRefusal::SubmittedDigest,
        ),
        (
            |m| {
                m.rows[1].submitted = m.rows[0].submitted.clone();
                m.rows[1].digest = m.rows[0].digest;
            },
            CaptureRefusal::DuplicateDigest,
        ),
    ];
    assert_eq!(check(control, journal), Ok(()));
    for (change, expected) in cases {
        let mut subject = control.clone();
        change(&mut subject);
        assert_eq!(check(&subject, journal), Err(expected.clone()));
    }
    let p = packet();
    let mut digest = manifest_digest(control).unwrap();
    digest[0] ^= 1;
    assert_eq!(
        bind_capture_manifest(
            CaptureTarget::Cost,
            &p,
            &p,
            &handshake(),
            control,
            digest,
            journal
        ),
        Err(CaptureRefusal::ManifestDigest)
    );
}

fn observation_controls(
    manifest: &CaptureManifest,
    control: &[(NativeOperationRequest, NativeOperationResponse)],
) {
    let submission = control
        .iter()
        .position(|(r, _)| matches!(r.subject, OperationSubject::Submission(_)))
        .unwrap();
    let mut subject = control.to_vec();
    subject.remove(submission);
    assert_eq!(
        check(manifest, &subject),
        Err(CaptureRefusal::RowWithoutObservation)
    );
    let mut subject = control.to_vec();
    subject.push(subject[submission].clone());
    assert_eq!(
        check(manifest, &subject),
        Err(CaptureRefusal::DuplicateObservation)
    );
    let mut subject = control.to_vec();
    if let OperationSubject::Submission(s) = &mut subject[submission].0.subject {
        s.transaction_bytes[0] ^= 1;
    }
    assert_eq!(
        check(manifest, &subject),
        Err(CaptureRefusal::ObservationWithoutRow)
    );
    let mut subject = control.to_vec();
    let response = &mut subject[submission].1;
    response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
    response.accepted_txid = None;
    response.mined_readback = None;
    assert_eq!(
        check(manifest, &subject),
        Err(CaptureRefusal::ObservedOutcome)
    );
    let mut subject = control.to_vec();
    subject[submission]
        .1
        .mined_readback
        .as_mut()
        .unwrap()
        .raw_transaction[0] ^= 1;
    assert_eq!(
        check(manifest, &subject),
        Err(CaptureRefusal::ReadbackBytes)
    );
    let mut subject = control.to_vec();
    subject[submission].1.mined_readback = None;
    assert!(matches!(
        check(manifest, &subject),
        Err(CaptureRefusal::Journal(JournalRefusal::Shape {
            refusal: ResponseShapeDefect::AcceptedOperationOmitsObservation,
            ..
        }))
    ));
    let mut p = packet();
    p.dependency_lock = None;
    assert!(matches!(
        bind_capture_manifest(
            CaptureTarget::Cost,
            &packet(),
            &p,
            &handshake(),
            manifest,
            manifest_digest(manifest).unwrap(),
            control
        ),
        Err(CaptureRefusal::Provenance(ProvenanceRefusal::Missing(_)))
    ));
}

#[test]
fn capture_support_has_bounded_offline_controls() {
    let started = std::time::Instant::now();
    let output = std::process::Command::new("/usr/bin/timeout")
        .args(["--kill-after=1", "300"])
        .arg(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "keyed_final_capture::family_child",
            "--nocapture",
        ])
        .output()
        .unwrap();
    writeln!(
        std::io::stdout().lock(),
        "final capture family process wall={:.6}s guard=300s exit={}",
        started.elapsed().as_secs_f64(),
        output.status
    )
    .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        writeln!(std::io::stdout().lock(), "capture-family| {line}").unwrap();
    }
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout
            .lines()
            .filter(|line| *line == "running 1 test")
            .count(),
        1
    );
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.starts_with("test result: ok. 1 passed; 0 failed; 0 ignored;"))
            .count(),
        1
    );
}

#[test]
#[ignore = "bounded family parent"]
fn family_child() {
    let family = std::time::Instant::now();
    for subject in [
        "pins",
        "watchdog",
        "birth",
        "continuation",
        "commitment",
        "cost",
    ] {
        let start = std::time::Instant::now();
        let output = std::process::Command::new("/usr/bin/timeout")
            .args(["--kill-after=1", "40"])
            .arg(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "keyed_final_capture::offline_child",
                "--nocapture",
            ])
            .env("TRIPOD_CAPTURE_CONTROL", subject)
            .output()
            .unwrap();
        let wall = start.elapsed().as_secs_f64();
        writeln!(
            std::io::stdout().lock(),
            "final capture subject={subject} wall={wall:.6}s guard=40s split_or_report={} exit={}",
            wall > 10.0,
            output.status
        )
        .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            writeln!(std::io::stdout().lock(), "capture-child| {line}").unwrap();
        }
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            stdout
                .lines()
                .filter(|line| *line == "running 1 test")
                .count(),
            1
        );
        assert_eq!(
            stdout
                .lines()
                .filter(|line| line.starts_with("test result: ok. 1 passed; 0 failed; 0 ignored;"))
                .count(),
            1
        );
    }
    let wall = family.elapsed().as_secs_f64();
    writeln!(
        std::io::stdout().lock(),
        "final capture family wall={wall:.6}s guard=300s"
    )
    .unwrap();
    assert!(wall < 300.0);
}

#[test]
#[ignore = "bounded offline parent"]
fn offline_child() {
    let target = match std::env::var("TRIPOD_CAPTURE_CONTROL").unwrap().as_str() {
        "pins" => {
            let (manifest, journal) = cost_control();
            manifest_controls(&manifest, &journal);
            observation_controls(&manifest, &journal);
            tracing_control(&manifest, &journal);
            return;
        }
        "watchdog" => {
            native::watchdog_control();
            return;
        }
        "birth" => CaptureTarget::Birth,
        "continuation" => CaptureTarget::Continuation,
        "commitment" => CaptureTarget::Commitment,
        "cost" => CaptureTarget::Cost,
        _ => panic!("closed offline subject census"),
    };
    retained::controls(target);
}

fn tracing_control(
    manifest: &CaptureManifest,
    journal: &[(NativeOperationRequest, NativeOperationResponse)],
) {
    use std::sync::{Arc, Mutex};
    #[derive(Clone)]
    struct Writer(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let bytes = Arc::new(Mutex::new(vec![]));
    let copy = bytes.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || Writer(copy.clone()))
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        assert_eq!(check(manifest, journal), Ok(()));
        let mut mutant = manifest.clone();
        mutant.schema = 2;
        assert_eq!(check(&mutant, journal), Err(CaptureRefusal::Version));
    });
    let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    let rows: Vec<_> = text
        .lines()
        .filter(|line| line.contains("vectors::keyed_final_capture"))
        .collect();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.contains("DEBUG")
        && row.contains("bind_capture_manifest")
        && !row.contains("WARN")
        && !row.contains("ERROR")));
    assert_eq!(
        rows.iter()
            .filter(|row| row.contains("capture admission completed"))
            .count(),
        1
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.contains("capture admission refused") && row.contains("version"))
            .count(),
        1
    );
}

#[test]
#[ignore = "orchestrator-only native pilot/campaign; prepared isolated unit required"]
fn birth() {
    native::run(CaptureTarget::Birth);
}
#[test]
#[ignore = "orchestrator-only native pilot/campaign; prepared isolated unit required"]
fn continuation() {
    native::run(CaptureTarget::Continuation);
}
#[test]
#[ignore = "orchestrator-only native pilot/campaign; prepared isolated unit required"]
fn commitment() {
    native::run(CaptureTarget::Commitment);
}
#[test]
#[ignore = "orchestrator-only native pilot/campaign; prepared isolated unit required"]
fn cost() {
    native::run(CaptureTarget::Cost);
}
