//! Ignored transport driver. Only the orchestrator's prepared unit invokes it.
use super::{driver, retained};
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// Two sessions plus both five-second cleanup graces consume at most 1210 s;
// the smallest unit guard is 1800 s, leaving 590 s for construction and retention.
const SESSION_TIMEOUT: Duration = Duration::from_secs(600);
use target_elements::*;
use target_elements_conformance::NativeConformanceError;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::*;
use target_elements_conformance::protocol::*;
use target_elements_conformance::provenance::ExpectedExecutorProvenance;
use vectors::keyed_final_capture::*;
use vectors::keyed_final_native::*;

struct Wire {
    steps: mpsc::Sender<Option<OperationStep>>,
    answers: mpsc::Receiver<NativeOperationResponse>,
    directory: PathBuf,
}
impl driver::Exchange for Wire {
    fn exchange(&mut self, step: OperationStep) -> NativeOperationResponse {
        let request = NativeOperationRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            subject: step.subject().clone(),
        };
        let started = Instant::now();
        self.steps.send(Some(step)).expect("live planner");
        let response = self.answers.recv().expect("transport response or refusal");
        let completed_epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let elapsed_seconds = started.elapsed().as_secs_f64();
        let mut journal = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.directory.join("completed-exchanges.jsonl"))
            .unwrap();
        serde_json::to_writer(
            &mut journal,
            &serde_json::json!({
                "request": request, "response": response,
                "completed_epoch_ms": completed_epoch_ms, "elapsed_seconds": elapsed_seconds,
            }),
        )
        .unwrap();
        writeln!(journal).unwrap();
        journal.sync_all().unwrap();
        sync_directory(&self.directory);
        println!(
            "capture exchange completed epoch_ms={completed_epoch_ms} step={} wall={elapsed_seconds:.6}s",
            request.case.step
        );
        response
    }
    fn before_submission(&mut self, row: CaptureRow) {
        // Durable before yielding the request to the transport, even if it dies.
        let file = self.directory.join(format!(
            "{}.transaction",
            crate::common::hex_bytes(&row.digest)
        ));
        let mut bytes = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file)
            .unwrap();
        bytes.write_all(&row.submitted).unwrap();
        bytes.sync_all().unwrap();
        sync_directory(&self.directory);
        let mut manifest = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.directory.join("subject-rows.jsonl"))
            .unwrap();
        serde_json::to_writer(&mut manifest, &row).unwrap();
        writeln!(manifest).unwrap();
        manifest.sync_all().unwrap();
        sync_directory(&self.directory);
    }
}
struct Planner {
    steps: mpsc::Receiver<Option<OperationStep>>,
    answers: Option<mpsc::Sender<NativeOperationResponse>>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Planner {
    fn new(key: [u8; 32], target: CaptureTarget, directory: &Path) -> Self {
        let (send_step, steps) = mpsc::channel();
        let (answers, receive_answer) = mpsc::channel();
        let directory = directory.to_owned();
        let dispatch = tracing::dispatcher::get_default(Clone::clone);
        let parent = tracing::Span::current();
        let worker = std::thread::spawn(move || {
            tracing::dispatcher::with_default(&dispatch, || {
                let _entered = parent.enter();
                let wire = Wire {
                    steps: send_step,
                    answers: receive_answer,
                    directory,
                };
                let wire = driver::campaign(wire, key, target);
                drop(wire.steps.send(None));
            });
        });
        Self {
            steps,
            answers: Some(answers),
            worker: Some(worker),
        }
    }
}
impl TargetOperationPlanner for Planner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_, response)) = previous {
            self.answers
                .as_ref()
                .ok_or(PlanRefused)?
                .send(response.clone())
                .map_err(|_| PlanRefused)?;
        }
        self.steps.recv().map_err(|_| PlanRefused)
    }
}
impl Drop for Planner {
    fn drop(&mut self) {
        self.answers.take();
        if let Some(worker) = self.worker.take() {
            drop(worker.join());
        }
    }
}

fn hex32(value: &str) -> [u8; 32] {
    vectors::maturity_observer::decode_hex(value)
        .unwrap()
        .try_into()
        .unwrap()
}
fn text<'a>(value: &'a serde_json::Value, field: &str) -> &'a str {
    value[field].as_str().expect("measured preparation field")
}
fn sync_directory(path: &Path) {
    std::fs::File::open(path).unwrap().sync_all().unwrap();
}
fn write_json(path: &Path, value: &impl serde::Serialize) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, value).unwrap();
    writeln!(file).unwrap();
    file.sync_all().unwrap();
    sync_directory(path.parent().unwrap());
}
fn measurement(prepared: &serde_json::Value, key: [u8; 32]) -> CaptureProvenance {
    CaptureProvenance {
        repository_commit: Some(text(prepared, "repository_commit").into()),
        dependency_lock: Some(hex32(text(prepared, "dependency_lock"))),
        toolchain: Some(text(prepared, "toolchain").into()),
        executable: Some(text(prepared, "daemon_version").into()),
        framework: Some(text(prepared, "framework").into()),
        executor: Some(hex32(text(prepared, "executor"))),
        protocol: Some(9),
        node: Some(text(prepared, "node").into()),
        image: Some(text(prepared, "image").into()),
        freeze: Some(capture_freeze_digest(key).unwrap()),
        key: Some(key),
        capture_binary: Some(hex32(text(prepared, "capture_binary"))),
        interpreter: Some(text(prepared, "interpreter").into()),
        daemon: Some(text(prepared, "daemon").into()),
    }
}
fn prepare_runtime(prepared: &serde_json::Value) {
    assert_eq!(text(prepared, "trust"), "reviewed_non_mock");
    for (path, field) in [
        (
            PathBuf::from("/capture-build/maturity_native"),
            "capture_binary",
        ),
        (
            PathBuf::from("/capture-build/Cargo.lock"),
            "dependency_lock",
        ),
        (
            PathBuf::from("/src/scripts/elements-native-executor.py"),
            "executor",
        ),
        (PathBuf::from("/usr/bin/python3"), "interpreter_digest"),
        (PathBuf::from("/elements/bin/elementsd"), "daemon_digest"),
    ] {
        assert_eq!(
            sha256(&std::fs::read(path).unwrap()),
            hex32(text(prepared, field)),
            "runtime {field}"
        );
    }
    assert_eq!(
        sha256(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),
        hex32(text(prepared, "capture_binary"))
    );
    assert_eq!(
        std::fs::read_to_string("/src/.git/HEAD").unwrap().trim(),
        text(prepared, "repository_commit")
    );
}

fn binding() -> ReviewedDevelopmentBinding {
    let target = reviewed_elements_tapscript().unwrap();
    // Public development identities of the retained keyed run of record; the
    // transport independently compares the fresh node's environment response.
    validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            hex32("75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1"),
            hex32("209577bda6bf4b5804bd46f8621580dd6d4e8bfa2d190e1c50e932492baca07d"),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .unwrap()
}

fn executor(directory: &Path, prepared: &serde_json::Value) -> PathBuf {
    let tip = text(prepared, "framework");
    assert_eq!(tip.len(), 40);
    assert!(tip.bytes().all(|b| b.is_ascii_hexdigit()));
    let script = format!(
        "#!/bin/sh\nset -eu\nexec /usr/bin/python3 /src/scripts/elements-native-executor.py --framework /elements/src/test/functional --elementsd /elements/bin/elementsd --elements-cli /elements/bin/elements-cli --fee-policy node-defaults --maturity-observation --enable-wallet --datadir '{}/regtest' --network-id 75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1 --intended-executed-tip {tip} --upstream-base {tip} \"$@\"\n",
        directory.display()
    );
    let path = directory.join("executor");
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn retain_exchanges(
    directory: &Path,
    capture: &NativeOperationCapture,
) -> Vec<(NativeOperationRequest, NativeOperationResponse)> {
    write_json(&directory.join("handshake.json"), &capture.handshake());
    write_json(&directory.join("environment.json"), &capture.environment());
    let exchanges: Vec<_> = capture
        .operations()
        .iter()
        .filter_map(|op| {
            op.response()
                .map(|answer| (op.request().clone(), answer.clone()))
        })
        .collect();
    write_json(&directory.join("journal.json"), &exchanges);
    exchanges
}

// Explain the transport's refusal after retention; this is not a preflight check.
fn unsupported_operation_message(
    error: &NativeConformanceError,
    capture: &NativeOperationCapture,
) -> Option<String> {
    if !matches!(error, NativeConformanceError::OperationStepUnsupported(_)) {
        return None;
    }
    let needed = [
        ExecutorCapability::TestFundingCeremony,
        ExecutorCapability::TargetTransactionSubmission,
        ExecutorCapability::TestSponsorAuthorization,
    ];
    let offered = capture.handshake().map(|handshake| &handshake.capabilities);
    let missing: Vec<_> = needed
        .iter()
        .filter(|capability| offered.is_none_or(|offered| !offered.contains(capability)))
        .collect();
    Some(format!(
        "native transport {error:?}: needed={} offered={} missing={}",
        serde_json::to_string(&needed).unwrap(),
        serde_json::to_string(&offered).unwrap(),
        serde_json::to_string(&missing).unwrap(),
    ))
}

pub(super) fn command_control() {
    let directory = tempfile::tempdir().unwrap();
    let tip = "1".repeat(40);
    let path = executor(directory.path(), &serde_json::json!({"framework":tip}));
    let actual = std::fs::read_to_string(path).unwrap();
    for required in [
        "exec /usr/bin/python3 /src/scripts/elements-native-executor.py",
        "--framework /elements/src/test/functional",
        "--elementsd /elements/bin/elementsd",
        "--elements-cli /elements/bin/elements-cli",
        "--fee-policy node-defaults",
        "--maturity-observation",
        "--enable-wallet",
        "--network-id 75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1",
        &format!("--datadir '{}/regtest'", directory.path().display()),
        &format!("--intended-executed-tip {tip}"),
        &format!("--upstream-base {tip}"),
        "\"$@\"",
    ] {
        assert_eq!(
            actual.matches(required).count(),
            1,
            "missing or repeated {required}"
        );
    }
}

fn run_session(
    target: CaptureTarget,
    key: [u8; 32],
    directory: &Path,
    prepared: &serde_json::Value,
) -> (retained::SessionArchive, Vec<CaptureRow>) {
    std::fs::create_dir(directory).unwrap();
    sync_directory(directory.parent().unwrap());
    let expected = measurement(prepared, key);
    bind_capture_provenance(&expected, &expected).unwrap(); // Presence and rendered executor pin before any launch.
    write_json(&directory.join("expected-provenance.json"), &expected);
    let adapter = executor(directory, prepared);
    let configuration = ExecutorConfiguration::new(
        &adapter,
        ExecutorTrust::ReviewedNonMock,
        SESSION_TIMEOUT,
        ExecutorDiagnostics::in_directory(&directory.join("diagnostics")),
    )
    .with_expected_provenance(
        ExpectedExecutorProvenance::new(
            text(prepared, "framework"),
            text(prepared, "framework"),
            [],
        )
        .unwrap(),
    );
    let mut planner = Planner::new(key, target, directory);
    let mut capture = NativeOperationCapture::default();
    let started = std::time::Instant::now();
    let result = execute_operations_captured(
        &reviewed_elements_tapscript().unwrap(),
        &binding(),
        &configuration,
        &mut planner,
        &mut capture,
    );
    drop(planner);
    // Retain partial evidence before interpreting any success or failure.
    std::fs::write(
        directory.join("capture.debug"),
        format!(
            "{capture:#?}\nresult={result:?}\nwall={:?}\n",
            started.elapsed()
        ),
    )
    .unwrap();
    let exchanges = retain_exchanges(directory, &capture);
    if let Err(error) = &result
        && let Some(message) = unsupported_operation_message(error, &capture)
    {
        panic!("{message}; partial evidence retained");
    }
    let transcript = result.expect("native transport; partial evidence retained");
    transcript
        .provenance_agreement()
        .expect("expected provenance supplied")
        .expect("handshake provenance agrees");
    let handshake = capture.handshake().unwrap().clone();
    let mut observed = expected.clone();
    observed.executable = Some(handshake.node_version.clone());
    observed.framework.clone_from(&handshake.framework_revision);
    observed.protocol = Some(handshake.protocol_schema);
    bind_capture_provenance(&expected, &observed).unwrap();
    write_json(&directory.join("observed-provenance.json"), &observed);
    let rows = std::fs::read_to_string(directory.join("subject-rows.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    println!(
        "native capture {target:?} key={} wall={:.6}s",
        crate::common::hex_bytes(&key),
        started.elapsed().as_secs_f64()
    );
    (
        retained::SessionArchive {
            key,
            provenance: observed,
            handshake,
            exchanges,
        },
        rows,
    )
}

/// Run one explicitly ignored target from its prepared, measured capture directory.
/// # Panics
/// Panics if preparation, transport, retention or offline admission refuses.
pub fn run(target: CaptureTarget) {
    let _span = tracing::info_span!(target: "vectors::native_test", "keyed_final_capture", ?target)
        .entered();
    assert_eq!(
        std::env::var("TRIPOD_FINAL_CAPTURE_DIR").unwrap(),
        "/capture"
    );
    let prepared: serde_json::Value =
        serde_json::from_slice(&std::fs::read("/capture-build/prepared.json").unwrap()).unwrap();
    prepare_runtime(&prepared);
    let mut sessions = vec![];
    let mut rows = vec![];
    let mut expected = vec![];
    for (i, key) in [[42; 32], vectors::keyed_final_manifest::SECOND_KEY]
        .into_iter()
        .enumerate()
    {
        let directory = PathBuf::from(format!("/capture/work/key-{i}"));
        let (session, subjects) = run_session(target, key, &directory, &prepared);
        expected.push(measurement(&prepared, key));
        sessions.push(session);
        rows.extend(subjects);
    }
    let manifest = CaptureManifest {
        schema: 1,
        target,
        rows,
    };
    let costs = (target == CaptureTarget::Cost).then(|| super::cost::derive(&sessions));
    let archive = retained::Archive {
        manifest_digest: manifest_digest(&manifest).unwrap(),
        manifest,
        sessions,
        native_clause_dimensions: None,
        costs,
    };
    write_json(
        Path::new("/capture/subject-manifest.json"),
        &archive.manifest,
    );
    write_json(
        Path::new("/capture/run-provenance.json"),
        &serde_json::json!({"manifest_digest":archive.manifest_digest,"sessions":expected}),
    );
    write_json(Path::new("/capture/capture.json"), &archive);
    retained::read(
        target,
        &expected,
        &std::fs::read("/capture/capture.json").unwrap(),
    )
    .expect("same offline admission as the typed mock");
    std::fs::write(
        "/capture/admitted",
        "offline capture evidence admitted; no finality or deployment premise discharged\n",
    )
    .unwrap();
}

pub(super) fn offline_transport(
    target: CaptureTarget,
    session: &retained::SessionArchive,
) -> ExecutorHandshake {
    offline_transport_control(target, session, false, false)
}

pub(super) fn watchdog_control() {
    let archive = retained::mock_archive(CaptureTarget::Cost);
    offline_transport_control(CaptureTarget::Cost, &archive.sessions[0], true, false);
    assert_eq!(
        2 * SESSION_TIMEOUT.as_secs() + 2 * DEFAULT_EXECUTOR_CLEANUP_GRACE.as_secs(),
        1210
    );
}

fn scripted_peer(
    directory: &Path,
    session: &retained::SessionArchive,
    stall: bool,
    missing: bool,
) -> PathBuf {
    let responses: Vec<_> = session
        .exchanges
        .iter()
        .map(|(_, response)| serde_json::to_string(response).unwrap())
        .collect();
    let mut peer = crate::common::scripted_adapter(
        if missing {
            &[]
        } else {
            &[
                "test_funding_ceremony",
                "test_sponsor_authorization",
                "target_transaction_submission",
            ]
        },
        if stall { &responses[..1] } else { &responses },
    )
    .replace("framework-tip", &"1".repeat(40))
    .replace("binary-tip", &"1".repeat(12))
    .replace("intended-tip", &"1".repeat(40))
    .replace("upstream-base", &"1".repeat(40))
    .replace("23.2.1", "mock node version")
    .replace("[\"topic-a\",\"topic-b\"]", "[]");
    if stall {
        peer.push_str("exec sleep 30\n");
    }
    let adapter = directory.join("peer");
    std::fs::write(&adapter, peer).unwrap();
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
    adapter
}

fn offline_transport_control(
    target: CaptureTarget,
    session: &retained::SessionArchive,
    stall: bool,
    missing: bool,
) -> ExecutorHandshake {
    let directory = tempfile::tempdir().unwrap();
    let adapter = scripted_peer(directory.path(), session, stall, missing);
    let target_definition = reviewed_elements_tapscript().unwrap();
    let deployment = validate_reviewed_development_binding(
        &target_definition,
        DevelopmentDeploymentBinding::new(
            target_definition.definition().version(),
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
        Duration::from_secs(if stall { 2 } else { 30 }),
        ExecutorDiagnostics::in_directory(&directory.path().join("diagnostics")),
    );
    let mut planner = Planner::new(session.key, target, directory.path());
    let mut capture = NativeOperationCapture::default();
    let started = std::time::Instant::now();
    let result = execute_operations_captured(
        &target_definition,
        &deployment,
        &configuration,
        &mut planner,
        &mut capture,
    );
    drop(planner);
    println!(
        "capture transport {target:?} key={} wall={:.6}s guard={}s",
        crate::common::hex_bytes(&session.key),
        started.elapsed().as_secs_f64(),
        if stall { 2 } else { 30 }
    );
    if missing {
        assert_unsupported(directory.path(), &capture, &result);
        return capture.handshake().unwrap().clone();
    }
    let durable: Vec<serde_json::Value> =
        std::fs::read_to_string(directory.path().join("completed-exchanges.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
    if stall {
        assert!(
            matches!(result, Err(NativeConformanceError::ExecutorTimeout)),
            "{result:?}"
        );
        assert_eq!(
            capture.terminal_state(),
            Some(CaptureTerminalState::Timeout)
        );
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "in-process watchdog before outer guard"
        );
        assert_eq!(durable.len(), 1);
        assert_eq!(
            durable[0]["request"],
            serde_json::to_value(&session.exchanges[0].0).unwrap()
        );
        assert_eq!(
            durable[0]["response"],
            serde_json::to_value(&session.exchanges[0].1).unwrap()
        );
        assert!(durable[0]["completed_epoch_ms"].as_u64().unwrap() > 0);
        assert!(durable[0]["elapsed_seconds"].as_f64().unwrap() >= 0.0);
        println!(
            "capture watchdog partial journal rows={} wall={:.6}s outer_guard=40s",
            durable.len(),
            started.elapsed().as_secs_f64()
        );
        return capture.handshake().unwrap().clone();
    }
    result.unwrap();
    assert_eq!(durable.len(), session.exchanges.len());
    for (row, (request, response)) in durable.iter().zip(&session.exchanges) {
        assert_eq!(row["request"], serde_json::to_value(request).unwrap());
        assert_eq!(row["response"], serde_json::to_value(response).unwrap());
    }
    assert_submissions(directory.path(), &capture, session);
    capture.handshake().unwrap().clone()
}

fn assert_submissions(
    directory: &Path,
    capture: &NativeOperationCapture,
    session: &retained::SessionArchive,
) {
    let actual: Vec<_> = capture
        .operations()
        .iter()
        .map(|row| (row.request().clone(), row.response().unwrap().clone()))
        .collect();
    assert_eq!(actual, session.exchanges);
    let rows: Vec<CaptureRow> = std::fs::read_to_string(directory.join("subject-rows.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let submitted: Vec<_> = actual
        .iter()
        .filter_map(|(request, _)| match &request.subject {
            OperationSubject::Submission(subject) => Some(&subject.transaction_bytes),
            _ => None,
        })
        .collect();
    assert_eq!(rows.len(), submitted.len());
    for (row, bytes) in rows.iter().zip(submitted) {
        assert_eq!(&row.submitted, bytes);
        assert_eq!(
            std::fs::read(directory.join(format!(
                "{}.transaction",
                crate::common::hex_bytes(&row.digest)
            )))
            .unwrap(),
            *bytes
        );
    }
}

pub(super) fn capability_control() {
    let archive = retained::mock_archive(CaptureTarget::Cost);
    offline_transport_control(CaptureTarget::Cost, &archive.sessions[0], false, false);
    offline_transport_control(CaptureTarget::Cost, &archive.sessions[0], false, true);
}

fn assert_unsupported(
    directory: &Path,
    capture: &NativeOperationCapture,
    result: &Result<ExecutionTranscript, NativeConformanceError>,
) {
    assert_eq!(retain_exchanges(directory, capture), []);
    let retained: Vec<(NativeOperationRequest, NativeOperationResponse)> =
        serde_json::from_slice(&std::fs::read(directory.join("journal.json")).unwrap()).unwrap();
    assert_eq!(retained, []);
    let handshake: Option<ExecutorHandshake> =
        serde_json::from_slice(&std::fs::read(directory.join("handshake.json")).unwrap()).unwrap();
    assert_eq!(handshake.as_ref(), capture.handshake());
    assert!(
        matches!(
            result,
            Err(NativeConformanceError::OperationStepUnsupported(
                OperationStepKind::FundSponsor
            ))
        ),
        "{result:?}"
    );
    assert_eq!(capture.operations(), []);
    assert!(!directory.join("completed-exchanges.jsonl").exists());
    assert!(!directory.join("subject-rows.jsonl").exists());
    let message = unsupported_operation_message(result.as_ref().unwrap_err(), capture).unwrap();
    let names = "[\"test_funding_ceremony\",\"target_transaction_submission\",\"test_sponsor_authorization\"]";
    assert!(message.contains(&format!("needed={names}")), "{message}");
    assert!(message.contains("offered=[]"), "{message}");
    assert!(message.contains(&format!("missing={names}")), "{message}");
    println!("{message}");
}
