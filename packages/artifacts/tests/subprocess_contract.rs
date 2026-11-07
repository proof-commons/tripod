//! ADR-010 control-plane subprocess coverage for the artifacts binaries.
//!
//! These run the shipped executables as real subprocesses and pin the uniform
//! control-plane contract: `--help`/`--version` exit 0 with no stdout noise, a
//! missing or unknown argument is usage class 2, and every control-plane
//! record is one JSON object on stderr while stdout stays empty.
//!
//! A PTY exercises the direct checker refusal before semantic work. Full
//! source arguments exercise both binary success routes, the report and
//! stamp mode, and the shared subscriber's command records.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

const BINARIES: [&str; 2] = [
    env!("CARGO_BIN_EXE_check-generated"),
    env!("CARGO_BIN_EXE_generate-all"),
];

fn run(binary: &str, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .env("RUST_LOG", "off")
        .output()
        .expect("binary runs")
}

fn stderr_records(output: &Output) -> Vec<Value> {
    let stderr = std::str::from_utf8(&output.stderr).expect("stderr is utf8");
    stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("every stderr line is one JSON object"))
        .collect()
}

fn assert_control(output: &Output, command: &str, kind: &str) {
    assert!(
        output.stdout.is_empty(),
        "a control-plane result must not write stdout"
    );
    let records = stderr_records(output);
    assert_eq!(records.len(), 1, "exactly one control record");
    assert_eq!(records[0]["schema"], cli_common::CONTROL_PLANE_SCHEMA);
    assert_eq!(records[0]["command"], command);
    assert_eq!(records[0]["kind"], kind);
}

#[test]
fn missing_arguments_are_usage_errors() {
    for (binary, name) in BINARIES
        .into_iter()
        .zip(["check-generated", "generate-all"])
    {
        let output = run(binary, &[]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{binary}: no arguments must be usage class 2",
        );
        assert_control(&output, name, "usage_error");
    }
}

#[test]
fn unknown_flag_is_a_usage_error() {
    for (binary, name) in BINARIES
        .into_iter()
        .zip(["check-generated", "generate-all"])
    {
        let output = run(binary, &["--definitely-not-a-real-flag"]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{binary}: an unknown flag must be usage class 2",
        );
        assert_control(&output, name, "usage_error");
    }
}

#[test]
fn help_and_version_exit_zero_with_empty_stdout() {
    for (binary, name) in BINARIES
        .into_iter()
        .zip(["check-generated", "generate-all"])
    {
        for flag in ["--help", "--version"] {
            let output = run(binary, &[flag]);
            assert_eq!(
                output.status.code(),
                Some(0),
                "{binary} {flag}: must exit 0",
            );
            assert_control(
                &output,
                name,
                if flag == "--help" { "help" } else { "version" },
            );
        }
    }
}

struct Fixture {
    root: tempfile::TempDir,
    generated: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("fixture root");
        for directory in [
            "papers/attestation/sections",
            "docs/attestation",
            "packages/model/src",
        ] {
            std::fs::create_dir_all(root.path().join(directory)).expect("fixture directory");
        }
        std::fs::write(
            root.path().join("papers/attestation/main.tex"),
            "\\label{def:model:known}\n",
        )
        .expect("attestation source");
        std::fs::write(
            root.path().join("docs/attestation/realization.md"),
            "# Realization\n`sec:fixture:division`\n",
        )
        .expect("realization source");
        std::fs::write(
            root.path().join("packages/model/src/fixture.rs"),
            "// \u{b4}test:fixture:defined\u{b4}\n",
        )
        .expect("model source");
        let generated = root.path().join("generated");
        Self { root, generated }
    }

    fn command(&self, binary: &str) -> Command {
        let mut command = Command::new(binary);
        command
            .arg("--repository-root")
            .arg(self.root.path())
            .arg("--attestation-main")
            .arg("papers/attestation/main.tex")
            .arg("--realization")
            .arg("docs/attestation/realization.md")
            .arg("--model-source")
            .arg("packages/model/src/fixture.rs");
        command
    }
}

fn find_event<'a>(records: &'a [Value], target: &str, message: &str) -> &'a Value {
    let events = records
        .iter()
        .filter(|record| record["target"] == target && record["fields"]["message"] == message)
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 1, "exactly one {target} {message}");
    events[0]
}

fn assert_command_completed(output: &Output, mode: &str, published: bool) {
    let records = stderr_records(output);
    let event = find_event(&records, "cli_common::command", "command completed");
    assert_eq!(event["level"], "INFO");
    assert_eq!(event["fields"]["exit_class"], "success");
    assert_eq!(event["fields"]["output_mode"], mode);
    assert_eq!(event["fields"]["result_published"], published);
    let close = find_event(&records, "cli_common::command", "span closed");
    assert_eq!(close["span"]["name"], "command");
    assert_eq!(close["span"]["output_mode"], mode);
    assert!(close["fields"]["busy_ns"].is_u64());
    assert!(close["fields"]["idle_ns"].is_u64());
}

fn assert_missing_artifact_fails_without_publication(fixture: &Fixture) {
    std::fs::remove_file(fixture.generated.join("model_labels.json"))
        .expect("remove one generated artifact");
    let failed_report = fixture.root.path().join("failed-report.json");
    let failed_stamp = fixture.root.path().join("failed.stamp");
    let failed = fixture
        .command(BINARIES[0])
        .arg("--generated-dir")
        .arg(&fixture.generated)
        .arg("--report")
        .arg(&failed_report)
        .arg("--stamp")
        .arg(&failed_stamp)
        .env("RUST_LOG", "info")
        .output()
        .expect("failing checker runs");
    assert_eq!(failed.status.code(), Some(1));
    assert!(failed.stdout.is_empty(), "failure stdout is empty");
    assert!(!failed_report.exists(), "failed report absent");
    assert!(!failed_stamp.exists(), "failed stamp absent");
    let records = stderr_records(&failed);
    let terminal = find_event(&records, "cli_common::command", "command failed");
    assert_eq!(terminal["fields"]["phase"], "execute");
    assert_eq!(terminal["fields"]["exit_class"], "failure");
    let defect = find_event(
        &records,
        "artifacts::check",
        "generated artifact not current",
    );
    assert_eq!(defect["level"], "ERROR");
    assert_eq!(defect["fields"]["diagnostic_code"], "missing");
    assert_eq!(defect["fields"]["artifact"], "model_labels.json");
    assert_eq!(
        defect["fields"]["detail"],
        "run `meson compile -C <builddir> generate-artifacts` and commit the diff"
    );
    assert!(defect["fields"].get("path").is_none());
}

#[test]
fn generated_assets_and_checker_result_keep_their_output_routes() {
    let fixture = Fixture::new();
    let generated = fixture
        .command(BINARIES[1])
        .arg("--output")
        .arg(&fixture.generated)
        .env("RUST_LOG", "info")
        .output()
        .expect("generate-all runs");
    assert_eq!(generated.status.code(), Some(0));
    assert!(generated.stdout.is_empty(), "generator stdout is empty");
    assert_command_completed(&generated, "none", false);
    let generated_records = stderr_records(&generated);
    let publication = find_event(
        &generated_records,
        "cli_common::publication",
        "asset batch published",
    );
    assert_eq!(publication["fields"]["outcome"], "completed");
    assert_eq!(publication["fields"]["asset_count"], 4);
    assert!(
        !generated_records
            .iter()
            .any(|record| { record["fields"]["message"] == "artifact published" })
    );
    let root = fixture.root.path().to_path_buf();
    let census = labels::RepositoryCensus {
        root: root.clone(),
        attestation_main: root.join("papers/attestation/main.tex"),
        realization: root.join("docs/attestation/realization.md"),
        model_sources: vec![root.join("packages/model/src/fixture.rs")],
        ..labels::RepositoryCensus::default()
    };
    for artifact in artifacts::expected_artifacts(&census).expect("expected generated bytes") {
        assert_eq!(
            std::fs::read(fixture.generated.join(artifact.name)).expect("generated asset"),
            artifact.bytes,
            "{} differs from expected bytes",
            artifact.name
        );
    }

    let direct = fixture
        .command(BINARIES[0])
        .arg("--generated-dir")
        .arg(&fixture.generated)
        .env("RUST_LOG", "info")
        .output()
        .expect("direct checker runs");
    assert_eq!(direct.status.code(), Some(0));
    assert_eq!(direct.stdout.last(), Some(&b'\n'));
    assert_eq!(direct.stdout.split(|byte| *byte == b'\n').count(), 2);
    let report: Value = serde_json::from_slice(&direct.stdout).expect("direct JSON report");
    assert_eq!(report["current"], true);
    assert_command_completed(&direct, "stdout", true);

    let report_path = fixture.root.path().join("report.json");
    let stamp_path = fixture.root.path().join("success.stamp");
    let build = fixture
        .command(BINARIES[0])
        .arg("--generated-dir")
        .arg(&fixture.generated)
        .arg("--report")
        .arg(&report_path)
        .arg("--stamp")
        .arg(&stamp_path)
        .env("RUST_LOG", "info")
        .output()
        .expect("build checker runs");
    assert_eq!(build.status.code(), Some(0));
    assert!(build.stdout.is_empty(), "build checker stdout is empty");
    assert_eq!(
        std::fs::read(&report_path).expect("report asset"),
        direct.stdout
    );
    assert!(stamp_path.is_file(), "success stamp exists");
    assert_eq!(
        std::fs::read(&stamp_path).expect("stamp bytes"),
        [] as [u8; 0]
    );
    assert_command_completed(&build, "asset", true);

    assert_missing_artifact_fails_without_publication(&fixture);
}

#[test]
fn generator_warning_and_checker_defects_are_bounded_and_located() {
    let fixture = Fixture::new();
    let initial = fixture
        .command(BINARIES[1])
        .arg("--output")
        .arg(&fixture.generated)
        .env("RUST_LOG", "info")
        .output()
        .expect("initial generation runs");
    assert_eq!(initial.status.code(), Some(0));
    std::fs::write(fixture.generated.join("stray.txt"), b"stray").expect("add one unowned file");
    let regenerated = fixture
        .command(BINARIES[1])
        .arg("--output")
        .arg(&fixture.generated)
        .env("RUST_LOG", "info")
        .output()
        .expect("regeneration runs");
    assert_eq!(regenerated.status.code(), Some(0));
    assert_eq!(regenerated.stdout, [] as [u8; 0]);
    assert_command_completed(&regenerated, "none", false);
    let records = stderr_records(&regenerated);
    let warning = find_event(
        &records,
        "artifacts::render",
        "unowned files left in output directory",
    );
    assert_eq!(warning["level"], "WARN");
    assert_eq!(warning["fields"]["unowned_file_count"], 1);
    assert!(warning["fields"].get("path").is_none());
    assert!(warning["fields"].get("file").is_none());
    assert!(
        !records
            .iter()
            .any(|record| record["fields"]["message"] == "artifact published")
    );
    find_event(&records, "cli_common::publication", "asset batch published");

    std::fs::write(fixture.generated.join("architecture.json"), b"stale")
        .expect("stale one owned artifact");
    let checked = fixture
        .command(BINARIES[0])
        .arg("--generated-dir")
        .arg(&fixture.generated)
        .env("RUST_LOG", "info")
        .output()
        .expect("checker runs");
    assert_eq!(checked.status.code(), Some(1));
    assert_eq!(checked.stdout, [] as [u8; 0]);
    let records = stderr_records(&checked);
    let stale = find_event(
        &records,
        "artifacts::check",
        "generated artifact not current",
    );
    assert_eq!(stale["level"], "ERROR");
    assert_eq!(stale["fields"]["diagnostic_code"], "stale");
    assert_eq!(stale["fields"]["artifact"], "architecture.json");
    assert!(stale["fields"].get("path").is_none());
    let unexpected = find_event(
        &records,
        "artifacts::check",
        "unexpected file in generated directory",
    );
    assert_eq!(unexpected["level"], "ERROR");
    assert_eq!(unexpected["fields"]["diagnostic_code"], "unexpected");
    assert_eq!(unexpected["fields"]["path"], "stray.txt");
    assert_eq!(
        unexpected["fields"]["detail"],
        "remove the file or add it to the generated-artifact census"
    );
    find_event(&records, "cli_common::command", "command failed");
}

const PANIC_CHILD_ENV: &str = "TRIPOD_ARTIFACTS_PANIC_CHILD";

#[test]
fn panic_subscriber_child() {
    if std::env::var_os(PANIC_CHILD_ENV).is_none() {
        return;
    }
    cli_common::install_command_subscriber("check-generated");
    panic!("private panic payload");
}

#[test]
fn panic_control_record_survives_off_filter() {
    let output = Command::new(std::env::current_exe().expect("integration test executable"))
        .arg("--exact")
        .arg("panic_subscriber_child")
        .arg("--nocapture")
        .env(PANIC_CHILD_ENV, "1")
        .env("RUST_LOG", "off")
        .output()
        .expect("panic child runs");
    assert_eq!(output.status.code(), Some(1));
    let records = stderr_records(&output);
    assert_eq!(records.len(), 1, "exactly one panic control record");
    assert_eq!(records[0]["schema"], cli_common::CONTROL_PLANE_SCHEMA);
    assert_eq!(records[0]["command"], "check-generated");
    assert_eq!(records[0]["kind"], "panic");
    assert!(
        !std::str::from_utf8(&output.stderr)
            .expect("utf8")
            .contains("private panic payload")
    );
}

#[cfg(unix)]
#[test]
fn checker_refuses_terminal_stdout_before_work() {
    use std::fs::File;
    use std::io::{Read, Seek};
    use std::process::Stdio;

    let fixture = Fixture::new();
    let pty = nix::pty::openpty(None, None).expect("open pty");
    let mut master = File::from(pty.master);
    let slave = File::from(pty.slave);
    let stderr_file = tempfile::tempfile().expect("stderr file");
    let mut stderr_reader = stderr_file.try_clone().expect("clone stderr");
    let child_slave = slave.try_clone().expect("clone slave");
    let mut command = fixture.command(BINARIES[0]);
    let mut child = command
        .arg("--generated-dir")
        .arg(&fixture.generated)
        .env("RUST_LOG", "off")
        .stdin(Stdio::null())
        .stdout(Stdio::from(child_slave))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .expect("checker starts");
    drop(slave);

    let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        loop {
            match master.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    if sender.send(buffer[..read].to_vec()).is_err() {
                        break;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
    });
    let status = child.wait().expect("checker exits");
    let mut stdout = Vec::new();
    while let Ok(chunk) = receiver.recv_timeout(std::time::Duration::from_millis(500)) {
        stdout.extend_from_slice(&chunk);
    }
    stderr_reader.rewind().expect("rewind stderr");
    let mut stderr = Vec::new();
    stderr_reader.read_to_end(&mut stderr).expect("read stderr");
    let output = Output {
        status,
        stdout,
        stderr,
    };
    assert_eq!(output.status.code(), Some(2));
    assert_control(&output, "check-generated", "tty_refusal");
    assert!(
        !fixture.generated.exists(),
        "refusal preceded semantic work"
    );
}
