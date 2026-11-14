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

struct RetentionWall {
    target: CaptureTarget,
    key: [u8; 32],
    started: Instant,
}
impl Drop for RetentionWall {
    fn drop(&mut self) {
        println!(
            "native capture {:?} key={} post-transport-retention wall={:.6}s",
            self.target,
            crate::common::hex_bytes(&self.key),
            self.started.elapsed().as_secs_f64()
        );
    }
}

const fn retains_debug(target: CaptureTarget) -> bool {
    !matches!(
        target,
        CaptureTarget::Funding(_) | CaptureTarget::Scaling(_)
    )
}

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
fn runtime_measurements(
    prepared: &serde_json::Value,
    running: &Path,
    read: impl Fn(&Path) -> Vec<u8>,
) -> serde_json::Value {
    assert_eq!(text(prepared, "trust"), "reviewed_non_mock");
    // The retained object labels every coordinate; carried preparation data is
    // not a runtime observation. The typed archive still uses the same reader.
    let mut observed = prepared.clone();
    for (path, field) in [
        ("/capture-build/maturity_native", "capture_binary"),
        ("/capture-build/Cargo.lock", "dependency_lock"),
        ("/src/Cargo.lock", "dependency_lock"),
        ("/src/scripts/elements-native-executor.py", "executor"),
        ("/usr/bin/python3", "interpreter_digest"),
        ("/elements/bin/elementsd", "daemon_digest"),
    ] {
        let digest = sha256(&read(Path::new(path)));
        assert_eq!(digest, hex32(text(prepared, field)), "runtime {path}");
        observed[field] = crate::common::hex_bytes(&digest).into();
    }
    let digest = sha256(&read(running));
    assert_eq!(digest, hex32(text(prepared, "capture_binary")));
    observed["capture_binary"] = crate::common::hex_bytes(&digest).into();
    let commit = String::from_utf8(read(Path::new("/src/.git/HEAD"))).unwrap();
    assert_eq!(commit.trim(), text(prepared, "repository_commit"));
    observed["repository_commit"] = commit.trim().into();
    for (field, digest) in [
        ("interpreter", "interpreter_digest"),
        ("daemon", "daemon_digest"),
    ] {
        let value = text(prepared, field).replace(text(prepared, digest), text(&observed, digest));
        observed[field] = value.into();
    }
    let mut origins = serde_json::Map::new();
    for field in prepared.as_object().unwrap().keys() {
        let origin = match field.as_str() {
            "repository_commit" | "capture_binary" | "dependency_lock" | "executor"
            | "interpreter_digest" | "daemon_digest" => "remeasured file bytes at runtime",
            "interpreter" | "daemon" => {
                "preparation version text with remeasured executable digest"
            }
            _ => "carried preparation measurement",
        };
        origins.insert(field.clone(), origin.into());
    }
    observed["origins"] = origins.into();
    observed
}

fn retained_provenance(observed: &CaptureProvenance) -> serde_json::Value {
    let mut value = serde_json::to_value(observed).unwrap();
    let mut origins = serde_json::Map::new();
    for field in value.as_object().unwrap().keys() {
        let origin = match field.as_str() {
            "repository_commit" | "dependency_lock" | "executor" | "capture_binary" => {
                "remeasured file bytes at runtime"
            }
            "executable" | "framework" | "protocol" => "session handshake",
            "interpreter" | "daemon" => {
                "preparation version text with remeasured executable digest"
            }
            "freeze" => "computed for supplied key",
            "key" => "supplied public input",
            _ => "carried preparation measurement",
        };
        origins.insert(field.clone(), origin.into());
    }
    value["origins"] = origins.into();
    value
}

pub(super) fn runtime_control() {
    for target in [
        CaptureTarget::Birth,
        CaptureTarget::Continuation,
        CaptureTarget::Commitment,
        CaptureTarget::Cost,
    ] {
        assert!(retains_debug(target));
    }
    for target in [
        CaptureTarget::Funding(0),
        CaptureTarget::Funding(23),
        CaptureTarget::Scaling(0),
        CaptureTarget::Scaling(1565),
    ] {
        assert!(!retains_debug(target));
    }
    let cleanup = DEFAULT_EXECUTOR_CLEANUP_GRACE.as_secs();
    assert_eq!(
        2 * (0..=23)
            .map(|p| session_timeout(CaptureTarget::Funding(p)).as_secs() + cleanup)
            .sum::<u64>(),
        1680
    );
    assert_eq!(
        2 * [0, 1, 2, 252, 253, 782, 1564, 1565]
            .into_iter()
            .map(|n| session_timeout(CaptureTarget::Scaling(n)).as_secs() + cleanup)
            .sum::<u64>(),
        3500
    );
    let bytes = b"measured artifact".to_vec();
    let digest = crate::common::hex_bytes(&sha256(&bytes));
    let prepared = serde_json::json!({
        "trust":"reviewed_non_mock", "capture_binary":digest, "dependency_lock":digest,
        "executor":digest, "interpreter_digest":digest, "daemon_digest":digest,
        "repository_commit":"synthetic commit", "interpreter":"once-only interpreter version",
        "daemon":"once-only daemon version",
    });
    let read = |path: &Path| {
        if path == Path::new("/src/.git/HEAD") {
            b"synthetic commit\n".to_vec()
        } else {
            bytes.clone()
        }
    };
    let mut runtime = runtime_measurements(&prepared, Path::new("/running"), read);
    let origins = runtime.as_object_mut().unwrap().remove("origins").unwrap();
    assert_eq!(runtime, prepared);
    assert_eq!(
        origins.as_object().unwrap().len(),
        prepared.as_object().unwrap().len()
    );
    assert_eq!(origins["trust"], "carried preparation measurement");
    assert_eq!(
        origins["dependency_lock"],
        "remeasured file bytes at runtime"
    );
    for changed in [
        "/src/Cargo.lock",
        "/capture-build/Cargo.lock",
        "/running",
        "/src/.git/HEAD",
    ] {
        assert!(
            std::panic::catch_unwind(|| runtime_measurements(
                &prepared,
                Path::new("/running"),
                |path| {
                    if path == Path::new(changed) {
                        b"changed".to_vec()
                    } else {
                        read(path)
                    }
                }
            ))
            .is_err(),
            "runtime drift {changed}"
        );
    }
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

const fn session_timeout(target: CaptureTarget) -> Duration {
    Duration::from_secs(match target {
        CaptureTarget::Funding(_) | CaptureTarget::Scaling(0..=2) => 30,
        CaptureTarget::Scaling(252 | 253) => 90,
        CaptureTarget::Scaling(782) => 240,
        _ => 600,
    })
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
    runtime: &serde_json::Value,
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
        session_timeout(target),
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
    let retention_wall = RetentionWall {
        target,
        key,
        started: Instant::now(),
    };
    drop(planner);
    // Retain partial evidence before interpreting any success or failure.
    // Envelope captures keep the complete journals and transactions without
    // the much larger duplicate Debug rendering. Existing targets keep it.
    if retains_debug(target) {
        std::fs::write(
            directory.join("capture.debug"),
            format!(
                "{capture:#?}\nresult={result:?}\nwall={:?}\n",
                started.elapsed()
            ),
        )
        .unwrap();
    }
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
    let mut observed = measurement(runtime, key);
    observed.executable = Some(handshake.node_version.clone());
    observed.framework.clone_from(&handshake.framework_revision);
    observed.protocol = Some(handshake.protocol_schema);
    bind_capture_provenance(&expected, &observed).unwrap();
    write_json(
        &directory.join("observed-provenance.json"),
        &retained_provenance(&observed),
    );
    let rows = std::fs::read_to_string(directory.join("subject-rows.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    drop(retention_wall);
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
    let runtime = runtime_measurements(&prepared, &std::env::current_exe().unwrap(), |path| {
        std::fs::read(path).unwrap()
    });
    write_json(Path::new("/capture/runtime-measurements.json"), &runtime);
    run_at(target, Path::new("/capture"), &prepared, &runtime);
}

/// Independent fresh chains per envelope coordinate, with one retained manifest per case.
/// # Panics
/// Panics on preparation, any session, or exact offline admission failure.
pub(super) fn run_envelope(funding: bool) {
    let _span =
        tracing::info_span!(target:"vectors::native_test","keyed_final_envelope",funding).entered();
    assert_eq!(
        std::env::var("TRIPOD_FINAL_CAPTURE_DIR").unwrap(),
        "/capture"
    );
    let prepared: serde_json::Value =
        serde_json::from_slice(&std::fs::read("/capture-build/prepared.json").unwrap()).unwrap();
    let runtime = runtime_measurements(&prepared, &std::env::current_exe().unwrap(), |p| {
        std::fs::read(p).unwrap()
    });
    write_json(Path::new("/capture/runtime-measurements.json"), &runtime);
    let targets: Vec<_> = if funding {
        (0..=23).map(CaptureTarget::Funding).collect()
    } else {
        [0, 1, 2, 252, 253, 782, 1564, 1565]
            .into_iter()
            .map(CaptureTarget::Scaling)
            .collect()
    };
    for (index, target) in targets.iter().enumerate() {
        let root = PathBuf::from(format!("/capture/case-{index}"));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(root.join("work")).unwrap();
        run_at(*target, &root, &prepared, &runtime);
    }
    write_json(Path::new("/capture/cases.json"), &targets);
    std::fs::write(
        "/capture/admitted",
        "all per-case offline admissions passed; no deployment premises discharged\n",
    )
    .unwrap();
}
fn run_at(
    target: CaptureTarget,
    root: &Path,
    prepared: &serde_json::Value,
    runtime: &serde_json::Value,
) {
    let mut sessions = vec![];
    let mut rows = vec![];
    let mut expected = vec![];
    for (i, key) in [[42; 32], vectors::keyed_final_manifest::SECOND_KEY]
        .into_iter()
        .enumerate()
    {
        let directory = root.join(format!("work/key-{i}"));
        let (session, subjects) = run_session(target, key, &directory, prepared, runtime);
        expected.push(measurement(prepared, key));
        sessions.push(session);
        rows.extend(subjects);
    }
    let writing = Instant::now();
    let manifest = CaptureManifest {
        schema: 1,
        target,
        rows,
    };
    let costs = (target == CaptureTarget::Cost).then(|| super::cost::derive(&sessions));
    let observed: Vec<_> = sessions
        .iter()
        .map(|session| session.provenance.clone())
        .collect();
    let archive = retained::Archive {
        manifest_digest: manifest_digest(&manifest).unwrap(),
        manifest,
        sessions,
        native_clause_dimensions: None,
        costs,
    };
    write_json(&root.join("subject-manifest.json"), &archive.manifest);
    write_json(
        &root.join("run-provenance.json"),
        &serde_json::json!({"manifest_digest":archive.manifest_digest,"sessions":observed}),
    );
    write_json(&root.join("capture.json"), &archive);
    println!(
        "native capture {target:?} case={} archive-writing wall={:.6}s",
        root.display(),
        writing.elapsed().as_secs_f64()
    );
    let admission = Instant::now();
    let result = retained::read(
        target,
        &expected,
        &std::fs::read(root.join("capture.json")).unwrap(),
    );
    println!(
        "native capture {target:?} case={} admission wall={:.6}s",
        root.display(),
        admission.elapsed().as_secs_f64()
    );
    result.expect("same offline admission as the typed mock");
    std::fs::write(
        root.join("admitted"),
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
