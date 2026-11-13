//! Offline bridge children; mock acceptance is never native evidence.
mod fixture;
use std::io::Write as _;
use std::process::Command;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{NativeOperationResponse, OperationCaseId};

fn child(test: &str, guard: &str) -> Command {
    let mut command = Command::new("/usr/bin/timeout");
    command
        .args(["--kill-after=1", guard])
        .arg(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", test, "--nocapture"]);
    command
}
fn run(command: &mut Command, name: &str) {
    let start = std::time::Instant::now();
    let output = command.output().unwrap();
    let wall = start.elapsed().as_secs_f64();
    writeln!(
        std::io::stdout().lock(),
        "bridge {name} wall={wall:.6}s exit={} split_or_report={}",
        output.status,
        wall > if name == "family" { 75.0 } else { 10.0 }
    )
    .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        writeln!(std::io::stdout().lock(), "bridge child: {line}").unwrap();
    }
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout.lines().filter(|l| *l == "running 1 test").count(), 1);
    assert_eq!(
        stdout
            .lines()
            .filter(|l| l.starts_with("test result: ok. 1 passed; 0 failed;"))
            .count(),
        1
    );
}
#[test]
fn public_construction_has_bounded_executor_bridge() {
    run(&mut child("keyed_bridge::family_child", "300"), "family");
}
#[test]
#[ignore = "bounded parent"]
fn family_child() {
    for name in [
        "control",
        "second",
        "refusals",
        "provenance",
        "authority",
        "decoder",
        "pairing",
        "recorded-revision",
        "tracing",
    ] {
        run(
            child("keyed_bridge::subject_child", "40").env("TRIPOD_BRIDGE_SUBJECT", name),
            name,
        );
    }
}
#[test]
#[ignore = "bounded family"]
fn subject_child() {
    match std::env::var("TRIPOD_BRIDGE_SUBJECT").unwrap().as_str() {
        "control" => replay(&fixture::history([42; 32])),
        "second" => replay(&fixture::history(vectors::keyed_final_manifest::SECOND_KEY)),
        "refusals" => fixture::refusals(),
        "provenance" => provenance(),
        "authority" => fixture::authority_controls(),
        "decoder" => fixture::decoder_boundary(),
        "pairing" => fixture::pairing_controls(),
        "recorded-revision" => fixture::recorded_revision_control(),
        "tracing" => tracing_control(),
        _ => panic!("unknown subject"),
    }
}
struct Replay {
    rows: Vec<(OperationStep, NativeOperationResponse)>,
    next: usize,
}
impl Replay {
    fn checked(
        rows: Vec<(OperationStep, NativeOperationResponse)>,
        expected: &vectors::keyed_final_native::CaptureProvenance,
        observed: &vectors::keyed_final_native::CaptureProvenance,
    ) -> Result<Self, vectors::keyed_final_native::ProvenanceRefusal> {
        vectors::keyed_final_native::bind_capture_provenance(expected, observed)?;
        Ok(Self { rows, next: 0 })
    }
}
impl TargetOperationPlanner for Replay {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        match previous {
            None if self.next == 0 => {}
            Some((case, response)) if self.next > 0 => {
                let (step, expected) = &self.rows[self.next - 1];
                if case != step.case() || response != expected {
                    return Err(PlanRefused);
                }
            }
            _ => return Err(PlanRefused),
        }
        let result = self.rows.get(self.next).map(|r| r.0.clone());
        self.next += usize::from(result.is_some());
        Ok(result)
    }
}
fn replay(session: &fixture::Session) {
    use std::os::unix::fs::PermissionsExt as _;
    use target_elements::*;
    use target_elements_conformance::executor::*;
    assert_eq!(session.finals.len(), 4);
    assert_eq!(
        session.finals[1].frozen().inputs()[0]
            .parent()
            .outpoint()
            .index(),
        1
    );
    assert_eq!(session.finals[1].frozen().inputs()[1].index(), 1);
    let responses: Vec<_> = session
        .exchanges
        .iter()
        .map(|(_, r)| serde_json::to_string(r).unwrap())
        .collect();
    let directory = crate::common::test_directory("keyed-bridge-replay");
    let adapter = directory.join("adapter.sh");
    std::fs::write(
        &adapter,
        crate::common::scripted_adapter(
            &[
                "test_sponsor_authorization",
                "target_transaction_submission",
            ],
            &responses,
        ),
    )
    .unwrap();
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755)).unwrap();
    let target = reviewed_elements_tapscript().unwrap();
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            [0x11; 32],
            [0x22; 32],
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .unwrap();
    let configuration = ExecutorConfiguration::new(
        &adapter,
        ExecutorTrust::Mock,
        std::time::Duration::from_secs(30),
        ExecutorDiagnostics::in_directory(&directory.join("diagnostics")),
    );
    let expected = mock_provenance(*session.finals[0].frozen().key());
    let observed = mock_provenance(*session.finals[0].frozen().key());
    let mut planner = Replay::checked(session.exchanges.clone(), &expected, &observed).unwrap();
    let mut capture = NativeOperationCapture::default();
    let started = std::time::Instant::now();
    let transcript = execute_operations_captured(
        &target,
        &binding,
        &configuration,
        &mut planner,
        &mut capture,
    )
    .unwrap();
    writeln!(
        std::io::stdout().lock(),
        "bridge transport wall={:.6}s",
        started.elapsed().as_secs_f64()
    )
    .unwrap();
    assert_eq!(transcript.trust(), ExecutorTrust::Mock);
    assert_eq!(capture.operations().len(), session.exchanges.len());
    for (captured, (step, response)) in capture.operations().iter().zip(&session.exchanges) {
        assert_eq!(&captured.request().subject, step.subject());
        assert_eq!(captured.response(), Some(response));
    }
    let journal: Vec<_> = capture
        .operations()
        .iter()
        .map(|row| (row.request().clone(), row.response().unwrap().clone()))
        .collect();
    journal_controls(&observed, capture.handshake().unwrap(), &journal);
    std::fs::remove_dir_all(directory).unwrap();
}
fn provenance() {
    use vectors::keyed_final_native::*;
    type Mutate = fn(&mut CaptureProvenance);
    let control = mock_provenance([42; 32]);
    Replay::checked(vec![], &control, &control).unwrap();
    let cases: [(ProvenanceRefusal, Mutate); 14] = [
        (ProvenanceRefusal::Repository, |p| {
            p.repository_commit.as_mut().unwrap().push('x');
        }),
        (ProvenanceRefusal::Lock, |p| {
            p.dependency_lock.as_mut().unwrap()[0] ^= 1;
        }),
        (ProvenanceRefusal::Toolchain, |p| {
            p.toolchain.as_mut().unwrap().push('x');
        }),
        (ProvenanceRefusal::Executable, |p| {
            p.executable.as_mut().unwrap().push('x');
        }),
        (ProvenanceRefusal::Framework, |p| {
            p.framework.as_mut().unwrap().push('x');
        }),
        (ProvenanceRefusal::Executor, |p| {
            p.executor.as_mut().unwrap()[0] ^= 1;
        }),
        (ProvenanceRefusal::Protocol, |p| {
            p.protocol = Some(8);
        }),
        (ProvenanceRefusal::Node, |p| {
            p.node.as_mut().unwrap().push('x');
        }),
        (ProvenanceRefusal::Image, |p| {
            p.image.as_mut().unwrap().push('x');
        }),
        (ProvenanceRefusal::Freeze, |p| {
            p.freeze.as_mut().unwrap()[0] ^= 1;
        }),
        (ProvenanceRefusal::Key, |p| {
            p.key.as_mut().unwrap()[0] ^= 1;
        }),
        (ProvenanceRefusal::CaptureBinary, |p| {
            p.capture_binary.as_mut().unwrap()[0] ^= 1;
        }),
        (ProvenanceRefusal::Interpreter, |p| {
            p.interpreter.as_mut().unwrap().push('x');
        }),
        (ProvenanceRefusal::Daemon, |p| {
            p.daemon.as_mut().unwrap().push('x');
        }),
    ];
    for (expected, mutate) in cases {
        let mut subject = control.clone();
        mutate(&mut subject);
        assert_eq!(
            Replay::checked(vec![], &control, &subject).map(|_| ()),
            Err(expected)
        );
    }
    missing_provenance(control);
}
fn missing_provenance(control: vectors::keyed_final_native::CaptureProvenance) {
    use vectors::keyed_final_native::*;
    type Mutate = fn(&mut CaptureProvenance);
    let missing: [(ProvenanceField, Mutate); 14] = [
        (ProvenanceField::Repository, |p| p.repository_commit = None),
        (ProvenanceField::Lock, |p| p.dependency_lock = None),
        (ProvenanceField::Toolchain, |p| p.toolchain = None),
        (ProvenanceField::Executable, |p| p.executable = None),
        (ProvenanceField::Framework, |p| p.framework = None),
        (ProvenanceField::Executor, |p| p.executor = None),
        (ProvenanceField::Protocol, |p| p.protocol = None),
        (ProvenanceField::Node, |p| p.node = None),
        (ProvenanceField::Image, |p| p.image = None),
        (ProvenanceField::Freeze, |p| p.freeze = None),
        (ProvenanceField::Key, |p| p.key = None),
        (ProvenanceField::CaptureBinary, |p| p.capture_binary = None),
        (ProvenanceField::Interpreter, |p| p.interpreter = None),
        (ProvenanceField::Daemon, |p| p.daemon = None),
    ];
    for (field, erase) in missing {
        let mut absent = control.clone();
        erase(&mut absent);
        for (expected, observed) in [(&control, &absent), (&absent, &control), (&absent, &absent)] {
            assert_eq!(
                bind_capture_provenance(expected, observed),
                Err(ProvenanceRefusal::Missing(field))
            );
        }
    }
    let empty: [(ProvenanceField, Mutate); 8] = [
        (ProvenanceField::Repository, |p| {
            p.repository_commit = Some(String::new());
        }),
        (ProvenanceField::Toolchain, |p| {
            p.toolchain = Some(String::new());
        }),
        (ProvenanceField::Executable, |p| {
            p.executable = Some(String::new());
        }),
        (ProvenanceField::Framework, |p| {
            p.framework = Some(String::new());
        }),
        (ProvenanceField::Node, |p| p.node = Some(String::new())),
        (ProvenanceField::Image, |p| p.image = Some(String::new())),
        (ProvenanceField::Interpreter, |p| {
            p.interpreter = Some(String::new());
        }),
        (ProvenanceField::Daemon, |p| p.daemon = Some(String::new())),
    ];
    for (field, erase) in empty {
        let mut absent = control.clone();
        erase(&mut absent);
        assert_eq!(
            bind_capture_provenance(&absent, &absent),
            Err(ProvenanceRefusal::Missing(field))
        );
    }
    let mut unreviewed = control;
    unreviewed.executor = Some([0; 32]);
    assert_eq!(
        bind_capture_provenance(&unreviewed, &unreviewed),
        Err(ProvenanceRefusal::ExecutorPin)
    );
}

fn tracing_control() {
    use std::sync::{Arc, Mutex};
    #[derive(Clone)]
    struct Writer(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Writer {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
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
        let session = fixture::history([42; 32]);
        provenance();
        replay(&session);
        fixture::pairing_controls();
    });
    let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    let rows: Vec<_> = text
        .lines()
        .filter(|r| r.contains("vectors::keyed_final_native"))
        .collect();
    for row in &rows {
        assert!(
            row.contains("DEBUG") && !row.contains("WARN") && !row.contains("ERROR"),
            "{row}"
        );
    }
    for (boundary, completed, refused) in [
        ("freeze_final_subject", 14, 16),
        ("bind_signing_request", 18, 3),
        ("complete_final_subject", 13, 2),
        ("validate_final_request", 12, 48),
    ] {
        let selected: Vec<_> = rows.iter().filter(|r| r.contains(boundary)).collect();
        assert_eq!(
            selected
                .iter()
                .filter(|r| r.contains("final bridge completed"))
                .count(),
            completed,
            "{text}"
        );
        assert_eq!(
            selected
                .iter()
                .filter(|r| r.contains("final bridge refused"))
                .count(),
            refused,
            "{text}"
        );
    }
    assert_eq!(
        rows.iter()
            .filter(|r| r.contains("capture provenance completed"))
            .count(),
        2
    );
    assert_eq!(
        rows.iter()
            .filter(|r| r.contains("capture provenance refused"))
            .count(),
        65
    );
    assert_eq!(
        rows.iter()
            .filter(|r| r.contains("capture freeze completed"))
            .count(),
        3
    );
    assert_eq!(
        rows.iter()
            .filter(|r| r.contains("capture journal completed"))
            .count(),
        1
    );
    assert_eq!(
        rows.iter()
            .filter(|r| r.contains("capture journal refused"))
            .count(),
        10
    );
    assert!(
        rows.iter()
            .filter(|r| r.contains("refused"))
            .all(|r| r.contains("refusal_kind"))
    );
}

fn mock_provenance(key: [u8; 32]) -> vectors::keyed_final_native::CaptureProvenance {
    vectors::keyed_final_native::CaptureProvenance {
        repository_commit: Some("mock capture build commit".into()),
        dependency_lock: Some([1; 32]),
        toolchain: Some("mock toolchain".into()),
        executable: Some("mock executable name/version".into()),
        framework: Some("mock framework".into()),
        executor: Some(target_elements_conformance::constructor::tagged::sha256(
            include_bytes!("../../../../scripts/elements-native-executor.py"),
        )),
        protocol: Some(9),
        node: Some("mock compute node".into()),
        image: Some("mock image".into()),
        freeze: Some(vectors::keyed_final_native::capture_freeze_digest(key).unwrap()),
        key: Some(key),
        capture_binary: Some([3; 32]),
        interpreter: Some("mock interpreter".into()),
        daemon: Some("mock Elements daemon".into()),
    }
}

fn journal_controls(
    packet: &vectors::keyed_final_native::CaptureProvenance,
    handshake: &target_elements_conformance::protocol::ExecutorHandshake,
    rows: &[(
        target_elements_conformance::protocol::NativeOperationRequest,
        NativeOperationResponse,
    )],
) {
    use target_elements_conformance::protocol::{OperationStepKind, ResponseShapeDefect};
    use target_elements_conformance::sponsor_parent::SponsorParentRefusal;
    use vectors::keyed_final_native::*;
    let control = bind_capture_journal(packet, handshake, rows).unwrap();
    assert_eq!(control.revision(), 9);
    assert_eq!(control.exchanges(), rows);
    let fund = rows
        .iter()
        .rposition(|r| r.0.case.operation == OperationStepKind::FundSponsor)
        .unwrap();
    let mut subject = rows.to_vec();
    subject[fund].1.funded_outputs[0].amount_satoshis += 1;
    assert_eq!(
        bind_capture_journal(packet, handshake, &subject).unwrap_err(),
        JournalRefusal::SponsorParent {
            index: fund,
            output: 0,
            refusal: SponsorParentRefusal::Amount
        }
    );
    subject = rows.to_vec();
    subject[fund].1.mined_readback = None;
    assert_eq!(
        bind_capture_journal(packet, handshake, &subject).unwrap_err(),
        JournalRefusal::Shape {
            index: fund,
            refusal: ResponseShapeDefect::SponsorReadbackMissing
        }
    );
    journal_identity_controls(packet, handshake, rows, fund);
}
fn journal_identity_controls(
    packet: &vectors::keyed_final_native::CaptureProvenance,
    handshake: &target_elements_conformance::protocol::ExecutorHandshake,
    rows: &[(
        target_elements_conformance::protocol::NativeOperationRequest,
        NativeOperationResponse,
    )],
    fund: usize,
) {
    use target_elements_conformance::protocol::OperationStepKind;
    use vectors::keyed_final_native::*;
    let mut subject = rows.to_vec();
    subject[fund].0.schema = 8;
    assert_eq!(
        bind_capture_journal(packet, handshake, &subject).unwrap_err(),
        JournalRefusal::RequestRevision(fund)
    );
    subject = rows.to_vec();
    subject[fund].1.schema = 8;
    assert_eq!(
        bind_capture_journal(packet, handshake, &subject).unwrap_err(),
        JournalRefusal::ResponseRevision(fund)
    );
    subject = rows.to_vec();
    subject[fund].1.case.step.push('x');
    assert_eq!(
        bind_capture_journal(packet, handshake, &subject).unwrap_err(),
        JournalRefusal::Case(fund)
    );
    subject = rows.to_vec();
    subject[fund].0.case.operation = OperationStepKind::Submit;
    subject[fund].1.case.operation = OperationStepKind::Submit;
    assert_eq!(
        bind_capture_journal(packet, handshake, &subject).unwrap_err(),
        JournalRefusal::Kind(fund)
    );
    let mut wrong_handshake = handshake.clone();
    wrong_handshake.protocol_schema = 8;
    assert_eq!(
        bind_capture_journal(packet, &wrong_handshake, rows).unwrap_err(),
        JournalRefusal::HandshakeRevision
    );
    let mut wrong_packet = packet.clone();
    wrong_packet.protocol = Some(8);
    assert_eq!(
        bind_capture_journal(&wrong_packet, handshake, rows).unwrap_err(),
        JournalRefusal::HandshakeRevision
    );
    wrong_packet.protocol = Some(10);
    assert_eq!(
        bind_capture_journal(&wrong_packet, handshake, rows).unwrap_err(),
        JournalRefusal::UnknownRevision(10)
    );
    wrong_packet.protocol = None;
    assert_eq!(
        bind_capture_journal(&wrong_packet, handshake, rows).unwrap_err(),
        JournalRefusal::MissingRevision
    );
}

#[test]
fn capture_journal_surface_is_sealed() {
    let source = include_str!("../../src/keyed_final_native.rs");
    let body = source
        .split("pub struct BoundCaptureJournal {")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    assert!(!body.contains("pub "));
    assert!(source.contains("compile_fail,E0451"));
    // Explicit spelling lock, not macro or alias analysis.
    assert!(source.contains("pub fn bind_capture_journal("));
}
