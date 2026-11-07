//! ADR-010 control-plane subprocess coverage for the labels binaries.
//!
//! These run the shipped executables as real subprocesses and pin the uniform
//! control-plane contract: `--help`/`--version` exit 0 with no stdout noise, a
//! missing or unknown argument is usage class 2, and every control-plane
//! record is one JSON object on stderr while stdout stays empty.
//!
//! TTY-refusal behaviour is covered by the PTY harness below (F1-024): a
//! direct-mode checker refuses a terminal stdout before doing any semantic
//! work, while build mode (with an explicit report/stamp destination) does not.

use std::process::{Command, Output, Stdio};

const BINARIES: [&str; 7] = [
    env!("CARGO_BIN_EXE_check-labels"),
    env!("CARGO_BIN_EXE_check-plans"),
    env!("CARGO_BIN_EXE_census-audit"),
    env!("CARGO_BIN_EXE_check-forbidden-text"),
    env!("CARGO_BIN_EXE_check-hash-citations"),
    env!("CARGO_BIN_EXE_generate-label-registers"),
    env!("CARGO_BIN_EXE_generate-attestation-register"),
];

/// A stub written by one test can meet another test's child in the script exec window.
/// Between fork and exec, a child inherits the test binary's open descriptors, so a
/// writable stub can remain open when a checker execs it and cause an executable-busy error.
/// One lock per test binary keeps every stub write and close separate from every spawn;
/// repeating a failed spawn would hide the race instead of removing it.
static STUB_PROCESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(unix)]
fn write_executable(path: &std::path::Path, contents: &str) -> std::path::PathBuf {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let _guard = STUB_PROCESS_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut file = std::fs::File::create(path).expect("create script");
    file.write_all(contents.as_bytes()).expect("write script");
    let mut permissions = file.metadata().expect("script metadata").permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions)
        .expect("set script permissions");
    drop(file);
    path.to_path_buf()
}

trait SerializedCommand {
    fn spawn_serialized(&mut self) -> std::io::Result<std::process::Child>;
    fn output_serialized(&mut self) -> std::io::Result<std::process::Output>;
}

impl SerializedCommand for Command {
    fn spawn_serialized(&mut self) -> std::io::Result<std::process::Child> {
        let _guard = STUB_PROCESS_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.spawn()
    }

    fn output_serialized(&mut self) -> std::io::Result<std::process::Output> {
        self.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        self.spawn_serialized()?.wait_with_output()
    }
}

fn run(binary: &str, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .output_serialized()
        .expect("binary runs")
}

fn run_with_filter(binary: &str, args: &[&str], filter: &str) -> Output {
    Command::new(binary)
        .args(args)
        .env("RUST_LOG", filter)
        .output_serialized()
        .expect("binary runs")
}

fn records(output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8(output.stderr.clone())
        .expect("stderr is utf8")
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).expect("stderr record is JSON"))
        .collect()
}

fn assert_success_terminal(output: &Output, output_mode: &str, result_published: bool) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "success records: {:?}",
        records(output)
    );
    let events = records(output);
    let terminal: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command completed"
        })
        .collect();
    assert_eq!(terminal.len(), 1, "success terminal: {events:?}");
    assert_eq!(terminal[0]["fields"]["exit_class"], "success");
    assert_eq!(terminal[0]["fields"]["output_mode"], output_mode);
    assert_eq!(terminal[0]["fields"]["result_published"], result_published);
    assert_eq!(
        events
            .iter()
            .filter(|record| record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command failed")
            .count(),
        0
    );
}

fn assert_json_only_stderr(output: &Output) {
    assert!(
        output.stdout.is_empty(),
        "a control-plane result must not write stdout"
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    let mut saw_record = false;
    for line in stderr.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(line).expect("every stderr line is one JSON object");
        assert!(value.is_object(), "each control-plane record is an object");
        saw_record = true;
    }
    assert!(
        saw_record,
        "expected at least one JSON control-plane record"
    );
}

#[test]
fn missing_arguments_are_usage_errors() {
    for binary in BINARIES {
        let output = run(binary, &[]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{binary}: no arguments must be usage class 2",
        );
        assert_json_only_stderr(&output);
    }
}

#[test]
fn check_plans_without_a_subject_is_a_usage_error() {
    // ADR-014 (F3-004): the build system states census membership by
    // argument. An invocation that names a repository root but no
    // subject must be refused as usage class 2, never run with the
    // on-disk walk as the effective census source.
    let output = run(
        env!("CARGO_BIN_EXE_check-plans"),
        &["--repository-root", "."],
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "check-plans with no --subject must be usage class 2",
    );
    assert_json_only_stderr(&output);
}

#[test]
fn unknown_flag_is_a_usage_error() {
    for binary in BINARIES {
        let output = run(binary, &["--definitely-not-a-real-flag"]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{binary}: an unknown flag must be usage class 2",
        );
        assert_json_only_stderr(&output);
    }
}

#[test]
fn help_and_version_exit_zero_with_empty_stdout() {
    for binary in BINARIES {
        for flag in ["--help", "--version"] {
            let output = run(binary, &[flag]);
            assert_eq!(
                output.status.code(),
                Some(0),
                "{binary} {flag}: must exit 0",
            );
            assert!(
                output.stdout.is_empty(),
                "{binary} {flag}: help/version is JSON on stderr, not stdout",
            );
        }
    }
}

#[test]
fn control_records_survive_disabled_diagnostics() {
    for binary in BINARIES {
        for (args, kind, status) in [
            (vec!["--help"], "help", 0),
            (vec!["--version"], "version", 0),
            (vec!["--definitely-not-a-real-flag"], "usage_error", 2),
        ] {
            let output = run_with_filter(binary, &args, "off");
            assert_eq!(output.status.code(), Some(status), "{binary}: {kind}");
            assert!(output.stdout.is_empty(), "{binary}: {kind} stdout");
            let records = records(&output);
            assert_eq!(records.len(), 1, "{binary}: {kind} records: {records:?}");
            assert_eq!(records[0]["kind"], kind, "{binary}: {kind}");
            assert!(records[0]["command"].is_string(), "{binary}: {kind}");
        }
    }
}

#[test]
fn subscriber_panic_control_record_survives_disabled_diagnostics() {
    const CHILD: &str = "LABELS_PANIC_CONTROL_CHILD";
    if std::env::var_os(CHILD).is_some() {
        cli_common::install_command_subscriber("labels-panic-test");
        panic!("test panic after subscriber bootstrap");
    }
    let output = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "subscriber_panic_control_record_survives_disabled_diagnostics",
        ])
        .env(CHILD, "1")
        .env("RUST_LOG", "off")
        .output_serialized()
        .expect("panic child runs");
    assert_eq!(output.status.code(), Some(1));
    let records = records(&output);
    assert_eq!(records.len(), 1, "panic records: {records:?}");
    assert_eq!(records[0]["kind"], "panic");
    assert_eq!(records[0]["command"], "labels-panic-test");
}

#[cfg(unix)]
#[test]
fn command_span_closes_with_numeric_timing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let git = write_executable(&dir.path().join("empty-git.sh"), "#!/bin/sh\nexit 0\n");
    let args = [
        "--repository-root",
        dir.path().to_str().expect("utf8 path"),
        "--git",
        git.to_str().expect("utf8 path"),
        "--exclude-pattern",
        ".*",
    ];
    let output = run_with_filter(env!("CARGO_BIN_EXE_census-audit"), &args, "info");
    assert_eq!(output.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    assert_eq!(report["valid"], true);
    let records = records(&output);
    let closes: Vec<_> = records
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "span closed"
        })
        .collect();
    assert_eq!(closes.len(), 1, "command closes: {records:?}");
    assert_eq!(closes[0]["span"]["name"], "command");
    assert!(closes[0]["fields"]["busy_ns"].is_u64());
    assert!(closes[0]["fields"]["idle_ns"].is_u64());
    assert_eq!(
        records
            .iter()
            .filter(|record| record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command completed")
            .count(),
        1
    );
    assert_success_terminal(&output, "stdout", true);
    let report_path = dir.path().join("census.json");
    let stamp_path = dir.path().join("census.stamp");
    let build = Command::new(env!("CARGO_BIN_EXE_census-audit"))
        .args(args)
        .arg("--report")
        .arg(&report_path)
        .arg("--stamp")
        .arg(&stamp_path)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("build census checker");
    assert_success_terminal(&build, "asset", true);
    assert_eq!(build.stdout, [] as [u8; 0]);
    assert_eq!(
        std::fs::read(&report_path).expect("census report"),
        output.stdout
    );
    assert_eq!(
        std::fs::read(&stamp_path).expect("census stamp"),
        [] as [u8; 0]
    );
}

fn assert_operational_failure_contract(binary: &str, args: &[String], output_mode: &str) {
    let output = Command::new(binary)
        .args(args)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(1), "{binary}: operational exit");
    assert!(output.stdout.is_empty(), "{binary}: failure stdout");
    let events = records(&output);
    let terminal: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command failed"
        })
        .collect();
    assert_eq!(terminal.len(), 1, "{binary}: terminal events: {events:?}");
    assert_eq!(terminal[0]["fields"]["exit_class"], "failure");
    assert_eq!(
        events
            .iter()
            .filter(|record| record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command completed")
            .count(),
        0,
        "{binary}: no success terminal"
    );
    let closes: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "span closed"
        })
        .collect();
    assert_eq!(closes.len(), 1, "{binary}: command close: {events:?}");
    assert_eq!(closes[0]["span"]["output_mode"], output_mode);
}

fn missing_file_args(root: &str, missing: &str, one: &str, two: &str, name: &str) -> Vec<String> {
    let suffix: &[&str] = match name {
        "check-labels" => &[
            "--attestation-main",
            missing,
            "--realization",
            missing,
            "--specification-register",
            missing,
            "--realization-register",
            missing,
            "--attestation-register",
            missing,
            "--model-labels-json",
            missing,
        ],
        "check-plans" => &["--subject", missing],
        "check-hash-citations" => &["--git", missing, "--families", missing],
        "check-forbidden-text" => &["--git", missing],
        "census-audit" => &["--git", missing, "--exclude-pattern", ".*"],
        "generate-label-registers" => &[
            "--attestation-main",
            missing,
            "--realization",
            missing,
            "--specification-register-output",
            one,
            "--realization-register-output",
            two,
        ],
        "generate-attestation-register" => &[
            "--attestation-main",
            missing,
            "--realization",
            missing,
            "--specification-register",
            missing,
            "--realization-register",
            missing,
            "--model-labels-json",
            missing,
            "--attestation-register",
            one,
        ],
        _ => unreachable!("known binary"),
    };
    let mut args = vec!["--repository-root".to_owned(), root.to_owned()];
    args.extend(suffix.iter().map(|arg| (*arg).to_owned()));
    args
}

#[test]
fn every_binary_failure_has_structured_stderr_and_empty_stdout() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().expect("utf8 root");
    let missing_path = dir.path().join("missing");
    let missing = missing_path.to_str().expect("utf8 missing");
    let one_path = dir.path().join("one.md");
    let one = one_path.to_str().expect("utf8 output");
    let two_path = dir.path().join("two.md");
    let two = two_path.to_str().expect("utf8 output");
    let cases = [
        (env!("CARGO_BIN_EXE_check-labels"), "check-labels", true),
        (env!("CARGO_BIN_EXE_check-plans"), "check-plans", true),
        (
            env!("CARGO_BIN_EXE_check-hash-citations"),
            "check-hash-citations",
            true,
        ),
        (
            env!("CARGO_BIN_EXE_check-forbidden-text"),
            "check-forbidden-text",
            true,
        ),
        (env!("CARGO_BIN_EXE_census-audit"), "census-audit", true),
        (
            env!("CARGO_BIN_EXE_generate-label-registers"),
            "generate-label-registers",
            false,
        ),
        (
            env!("CARGO_BIN_EXE_generate-attestation-register"),
            "generate-attestation-register",
            false,
        ),
    ];
    for (binary, name, checker) in cases {
        let args = missing_file_args(root, missing, one, two, name);
        assert_operational_failure_contract(binary, &args, if checker { "stdout" } else { "none" });
        if checker {
            let report = dir.path().join(format!("{name}.json"));
            let stamp = dir.path().join(format!("{name}.stamp"));
            let mut build_args = args;
            build_args.extend([
                "--report".to_owned(),
                report.to_string_lossy().into_owned(),
                "--stamp".to_owned(),
                stamp.to_string_lossy().into_owned(),
            ]);
            assert_operational_failure_contract(binary, &build_args, "asset");
            assert!(
                !report.exists(),
                "{binary}: failure must not publish a report"
            );
            assert!(
                !stamp.exists(),
                "{binary}: failure must not touch the stamp"
            );
        }
    }
    assert!(!one_path.exists(), "failed generation leaves no asset");
    assert!(!two_path.exists(), "failed generation leaves no asset");
}

#[test]
fn label_register_generator_publishes_explicit_assets_without_stdout() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let attestation = root.join("papers/attestation/main.tex");
    let realization = root.join("docs/attestation/realization.md");
    std::fs::create_dir_all(attestation.parent().expect("attestation parent"))
        .expect("attestation directory");
    std::fs::create_dir_all(realization.parent().expect("realization parent"))
        .expect("realization directory");
    std::fs::write(&attestation, "\\label{def:model:known}\n").expect("attestation source");
    std::fs::write(
        &realization,
        "# Realization\n`sec:fixture:division`\nBody cite (`[A-def:model:known]`).\n",
    )
    .expect("realization source");
    let specification_output = root.join("specification.md");
    let realization_output = root.join("realization.md");
    let output = Command::new(env!("CARGO_BIN_EXE_generate-label-registers"))
        .args([
            "--repository-root",
            root.to_str().expect("utf8 root"),
            "--attestation-main",
            attestation.to_str().expect("utf8 source"),
            "--realization",
            realization.to_str().expect("utf8 source"),
            "--specification-register-output",
            specification_output.to_str().expect("utf8 output"),
            "--realization-register-output",
            realization_output.to_str().expect("utf8 output"),
        ])
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("generator runs");
    assert_eq!(
        output.status.code(),
        Some(0),
        "generator diagnostics: {:?}",
        records(&output)
    );
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_ne!(
        std::fs::read(&specification_output).expect("specification asset"),
        [] as [u8; 0]
    );
    assert_ne!(
        std::fs::read(&realization_output).expect("realization asset"),
        [] as [u8; 0]
    );
    let events = records(&output);
    let terminal: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command completed"
        })
        .collect();
    assert_eq!(terminal.len(), 1, "generator terminal: {events:?}");
    assert_eq!(terminal[0]["fields"]["output_mode"], "none");
    assert_eq!(terminal[0]["fields"]["result_published"], false);
    assert_success_terminal(&output, "none", false);
}

#[test]
fn attestation_register_generator_publishes_explicit_asset_without_stdout() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("packages directory")
        .parent()
        .expect("repository root");
    let census = labels::RepositoryCensus::discover(root);
    let dir = tempfile::tempdir().expect("output directory");
    let output_path = dir.path().join("attestation-register.md");
    let mut command = Command::new(env!("CARGO_BIN_EXE_generate-attestation-register"));
    command
        .arg("--repository-root")
        .arg(root)
        .arg("--attestation-main")
        .arg(&census.attestation_main)
        .arg("--realization")
        .arg(&census.realization)
        .arg("--specification-register")
        .arg(&census.specification_register)
        .arg("--realization-register")
        .arg(&census.realization_register)
        .arg("--model-labels-json")
        .arg(&census.model_labels_json)
        .arg("--attestation-register")
        .arg(&output_path)
        .env("RUST_LOG", "info");
    for path in &census.attestation_sections {
        command.arg("--attestation-section").arg(path);
    }
    for path in &census.adrs {
        command.arg("--adr").arg(path);
    }
    for path in &census.plans {
        command.arg("--plan").arg(path);
    }
    for path in &census.docs {
        command.arg("--doc").arg(path);
    }
    for path in &census.model_sources {
        command.arg("--model-source").arg(path);
    }
    for sources in census.crate_sources.values() {
        for path in sources {
            command.arg("--crate-source").arg(path);
        }
    }
    for path in &census.scripts {
        command.arg("--script-source").arg(path);
    }
    let output = command.output_serialized().expect("generator runs");
    assert_eq!(
        output.status.code(),
        Some(0),
        "generator diagnostics: {:?}",
        records(&output)
    );
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_ne!(
        std::fs::read(&output_path).expect("attestation asset"),
        [] as [u8; 0]
    );
    let events = records(&output);
    let terminal: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command completed"
        })
        .collect();
    assert_eq!(terminal.len(), 1, "generator terminal: {events:?}");
    assert_eq!(terminal[0]["fields"]["output_mode"], "none");
    assert_eq!(terminal[0]["fields"]["result_published"], false);
    assert_success_terminal(&output, "none", false);
}

#[test]
fn plan_checker_keeps_direct_and_build_result_bytes() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("packages directory")
        .parent()
        .expect("repository root");
    let census = labels::RepositoryCensus::discover(root);
    let mut args = vec![
        "--repository-root".to_owned(),
        root.to_string_lossy().into_owned(),
    ];
    for path in census.adrs.iter().chain(census.plans.iter()) {
        args.push("--subject".to_owned());
        args.push(path.to_string_lossy().into_owned());
    }
    for path in std::iter::once("adr/README.md").chain(labels::plans::GENERATED_REGISTERS) {
        args.push("--subject".to_owned());
        args.push(root.join(path).to_string_lossy().into_owned());
    }
    let direct = Command::new(env!("CARGO_BIN_EXE_check-plans"))
        .args(&args)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("direct checker runs");
    assert_success_terminal(&direct, "stdout", true);
    assert!(direct.stdout.ends_with(b"\n"));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&direct.stdout).expect("direct report")["valid"],
        true
    );
    let output_dir = tempfile::tempdir().expect("output directory");
    let report = output_dir.path().join("plans.json");
    let stamp = output_dir.path().join("plans.stamp");
    let build = Command::new(env!("CARGO_BIN_EXE_check-plans"))
        .args(&args)
        .arg("--report")
        .arg(&report)
        .arg("--stamp")
        .arg(&stamp)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("build checker runs");
    assert_success_terminal(&build, "asset", true);
    assert_eq!(build.stdout, [] as [u8; 0]);
    assert_eq!(std::fs::read(&report).expect("build report"), direct.stdout);
    assert_eq!(std::fs::read(&stamp).expect("stamp bytes"), [] as [u8; 0]);
    let mut adjacent = args.clone();
    adjacent.push("--subject".to_owned());
    adjacent.push(root.join("README.md").to_string_lossy().into_owned());
    let negative = Command::new(env!("CARGO_BIN_EXE_check-plans"))
        .args(&adjacent)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("adjacent plan checker runs");
    assert_eq!(negative.status.code(), Some(1));
    assert_eq!(negative.stdout, [] as [u8; 0]);
    let events = records(&negative);
    assert!(
        events
            .iter()
            .any(|record| record["target"] == "labels::plans"
                && record["fields"]["message"] == "plan check completed"
                && record["fields"]["valid"] == false)
    );
    let defect = events
        .iter()
        .find(|record| {
            record["target"] == "labels::plans"
                && record["fields"]["diagnostic_code"] == "plan_failure"
        })
        .expect("plan defect");
    assert!(defect["fields"]["detail"].is_string());
    assert!(defect["fields"].get("path").is_none());
}

#[test]
fn label_checker_keeps_direct_and_build_result_bytes() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("packages directory")
        .parent()
        .expect("repository root");
    let census = labels::RepositoryCensus::discover(root);
    let mut args = vec![
        "--repository-root".to_owned(),
        root.to_string_lossy().into_owned(),
    ];
    for (flag, path) in [
        ("--attestation-main", &census.attestation_main),
        ("--realization", &census.realization),
        ("--specification-register", &census.specification_register),
        ("--realization-register", &census.realization_register),
        ("--attestation-register", &census.attestation_register),
        ("--model-labels-json", &census.model_labels_json),
    ] {
        args.push(flag.to_owned());
        args.push(path.to_string_lossy().into_owned());
    }
    for (flag, paths) in [
        ("--attestation-section", &census.attestation_sections),
        ("--adr", &census.adrs),
        ("--plan", &census.plans),
        ("--doc", &census.docs),
        ("--model-source", &census.model_sources),
        ("--script-source", &census.scripts),
    ] {
        for path in paths {
            args.push(flag.to_owned());
            args.push(path.to_string_lossy().into_owned());
        }
    }
    for sources in census.crate_sources.values() {
        for path in sources {
            args.push("--crate-source".to_owned());
            args.push(path.to_string_lossy().into_owned());
        }
    }
    let direct = Command::new(env!("CARGO_BIN_EXE_check-labels"))
        .args(&args)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("direct checker runs");
    assert_success_terminal(&direct, "stdout", true);
    assert!(direct.stdout.ends_with(b"\n"));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&direct.stdout).expect("direct report")["valid"],
        true
    );
    let output_dir = tempfile::tempdir().expect("output directory");
    let report = output_dir.path().join("labels.json");
    let stamp = output_dir.path().join("labels.stamp");
    let build = Command::new(env!("CARGO_BIN_EXE_check-labels"))
        .args(&args)
        .arg("--report")
        .arg(&report)
        .arg("--stamp")
        .arg(&stamp)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("build checker runs");
    assert_success_terminal(&build, "asset", true);
    assert_eq!(build.stdout, [] as [u8; 0]);
    assert_eq!(std::fs::read(&report).expect("build report"), direct.stdout);
    assert_eq!(std::fs::read(&stamp).expect("stamp bytes"), [] as [u8; 0]);
    let mut adjacent = args.clone();
    adjacent.push("--attestation-section".to_owned());
    adjacent.push(
        census
            .attestation_sections
            .first()
            .expect("attestation section")
            .to_string_lossy()
            .into_owned(),
    );
    let negative = Command::new(env!("CARGO_BIN_EXE_check-labels"))
        .args(&adjacent)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("adjacent label checker runs");
    assert_eq!(negative.status.code(), Some(1));
    assert_eq!(negative.stdout, [] as [u8; 0]);
    let events = records(&negative);
    let defect = events
        .iter()
        .find(|record| {
            record["target"] == "labels::check" && record["fields"]["diagnostic_code"].is_string()
        })
        .expect("located label defect");
    assert!(defect["fields"]["path"].is_string());
    assert!(defect["fields"]["line"].is_u64());
    assert!(defect["fields"]["column"].is_u64());
    assert!(defect["fields"]["detail"].is_string());
}

/// The git program is argument-supplied, so its stderr is arbitrary
/// child output. A failing git must surface only the process status and
/// a generic message — never the raw child stderr (F1-032, ADR-010).
#[cfg(unix)]
#[test]
fn census_audit_omits_untrusted_git_stderr() {
    let secret = "SHOULD_NOT_APPEAR_git_stderr_secret";

    let dir = tempfile::tempdir().expect("tempdir");
    let script = write_executable(
        &dir.path().join("fake-git.sh"),
        &format!("#!/bin/sh\necho '{secret}' >&2\nexit 17\n"),
    );

    let output = run(
        env!("CARGO_BIN_EXE_census-audit"),
        &[
            "--repository-root",
            dir.path().to_str().expect("utf8 path"),
            "--git",
            script.to_str().expect("utf8 path"),
            "--exclude-pattern",
            ".*",
        ],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "a failing git program is a runtime failure, not usage",
    );
    assert_eq!(output.stdout, [] as [u8; 0]);

    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    assert!(
        !stderr.contains(secret),
        "untrusted git stderr leaked into diagnostics: {stderr}",
    );

    // Every emitted record is still one JSON object, and the fixed
    // failure message is present.
    assert_json_only_stderr(&output);
    assert!(
        stderr.contains("git ls-files failed"),
        "expected the generic git-failure message: {stderr}",
    );
    let events = records(&output);
    let failed: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "labels::census"
                && record["fields"]["message"] == "tracked audit completed"
        })
        .collect();
    assert_eq!(failed.len(), 1, "census failure completion: {events:?}");
    assert_eq!(failed[0]["fields"]["outcome"], "failed");
    assert_eq!(failed[0]["fields"]["error_kind"], "git");
    assert_eq!(failed[0]["fields"]["child_exit_code"], 17);
}

#[cfg(unix)]
#[test]
fn hash_scan_completes_a_negative_then_runner_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let git = write_executable(
        &dir.path().join("fake-git.sh"),
        "#!/bin/sh\nprintf 'note.md\\0'\n",
    );
    let families = dir.path().join("families.tsv");
    std::fs::write(&families, "id\tmatch_kind\tmatch_value\tgroup\tverification\treferent\tgeneration_program\tnotes\nrecords\tpathglob\tplans/*\tINTERNAL\tRECOMPUTABLE\tA record.\tThe record.\tRecords.\n").expect("write rules");
    let subject = dir.path().join("note.md");
    std::fs::write(&subject, "plain text\n").expect("write control");
    let args = [
        "--repository-root",
        dir.path().to_str().expect("utf8 root"),
        "--git",
        git.to_str().expect("utf8 git"),
        "--families",
        families.to_str().expect("utf8 rules"),
    ];
    let control = run_with_filter(env!("CARGO_BIN_EXE_check-hash-citations"), &args, "info");
    assert_eq!(control.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&control.stdout).expect("control report")["valid"],
        true
    );
    assert_success_terminal(&control, "stdout", true);
    let build_report = dir.path().join("hash.json");
    let build_stamp = dir.path().join("hash.stamp");
    let build = Command::new(env!("CARGO_BIN_EXE_check-hash-citations"))
        .args(args)
        .arg("--report")
        .arg(&build_report)
        .arg("--stamp")
        .arg(&build_stamp)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("build hash checker");
    assert_success_terminal(&build, "asset", true);
    assert_eq!(build.stdout, [] as [u8; 0]);
    assert_eq!(
        std::fs::read(&build_report).expect("hash report"),
        control.stdout
    );
    assert_eq!(
        std::fs::read(&build_stamp).expect("hash stamp"),
        [] as [u8; 0]
    );
    assert!(
        records(&control)
            .iter()
            .any(|record| record["target"] == "labels::hashcite"
                && record["fields"]["outcome"] == "completed"
                && record["fields"]["valid"] == true)
    );
    std::fs::write(&subject, "digest deadbeef1 unexplained\n").expect("write subject");
    let negative = run_with_filter(env!("CARGO_BIN_EXE_check-hash-citations"), &args, "info");
    assert_eq!(negative.status.code(), Some(1));
    assert_eq!(negative.stdout, [] as [u8; 0]);
    let events = records(&negative);
    let completion: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "labels::hashcite"
                && record["fields"]["message"] == "tracked audit completed"
        })
        .collect();
    assert_eq!(completion.len(), 1, "hash completion: {events:?}");
    assert_eq!(completion[0]["fields"]["outcome"], "completed");
    assert_eq!(completion[0]["fields"]["valid"], false);
    assert_eq!(completion[0]["fields"]["refusal_count"], 1);
    assert_eq!(
        events
            .iter()
            .filter(|record| record["target"] == "labels::hashcite"
                && record["fields"]["message"] == "span closed"
                && record["span"]["name"] == "scan_hash_citations")
            .count(),
        1
    );
    let defect = events
        .iter()
        .find(|record| record["fields"]["diagnostic_code"] == "unclaimed_hash_citation")
        .expect("located refusal");
    assert_eq!(defect["fields"]["path"], "note.md");
    assert_eq!(defect["fields"]["line"], 1);
    assert!(defect["fields"].get("value").is_none());
    assert!(!String::from_utf8_lossy(&negative.stderr).contains("deadbeef1"));
}

#[cfg(unix)]
#[test]
fn forbidden_scan_completes_a_negative_then_runner_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let git_path = dir.path().join("fake-git.sh");
    let git = write_executable(&git_path, "#!/bin/sh\nexit 1\n");
    let args = [
        "--repository-root",
        dir.path().to_str().expect("utf8 root"),
        "--git",
        git.to_str().expect("utf8 git"),
    ];
    let control = run_with_filter(env!("CARGO_BIN_EXE_check-forbidden-text"), &args, "info");
    assert_eq!(control.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&control.stdout).expect("control report")["valid"],
        true
    );
    assert_success_terminal(&control, "stdout", true);
    let build_report = dir.path().join("forbidden.json");
    let build_stamp = dir.path().join("forbidden.stamp");
    let build = Command::new(env!("CARGO_BIN_EXE_check-forbidden-text"))
        .args(args)
        .arg("--report")
        .arg(&build_report)
        .arg("--stamp")
        .arg(&build_stamp)
        .env("RUST_LOG", "info")
        .output_serialized()
        .expect("build forbidden checker");
    assert_success_terminal(&build, "asset", true);
    assert_eq!(build.stdout, [] as [u8; 0]);
    assert_eq!(
        std::fs::read(&build_report).expect("forbidden report"),
        control.stdout
    );
    assert_eq!(
        std::fs::read(&build_stamp).expect("forbidden stamp"),
        [] as [u8; 0]
    );
    assert!(
        records(&control)
            .iter()
            .any(|record| record["target"] == "labels::forbidden"
                && record["fields"]["outcome"] == "completed"
                && record["fields"]["valid"] == true)
    );
    write_executable(
        &git_path,
        &format!("#!/bin/sh\nprintf 'note.md:1:<{}>\\n'\n", "char"),
    );
    let negative = run_with_filter(env!("CARGO_BIN_EXE_check-forbidden-text"), &args, "info");
    assert_eq!(negative.status.code(), Some(1));
    assert_eq!(negative.stdout, [] as [u8; 0]);
    let events = records(&negative);
    let completion: Vec<_> = events
        .iter()
        .filter(|record| {
            record["target"] == "labels::forbidden"
                && record["fields"]["message"] == "tracked audit completed"
        })
        .collect();
    assert_eq!(completion.len(), 1, "forbidden completion: {events:?}");
    assert_eq!(completion[0]["fields"]["outcome"], "completed");
    assert_eq!(completion[0]["fields"]["valid"], false);
    assert_eq!(completion[0]["fields"]["refusal_count"], 1);
    assert_eq!(
        events
            .iter()
            .filter(|record| record["target"] == "labels::forbidden"
                && record["fields"]["message"] == "span closed"
                && record["span"]["name"] == "scan_forbidden_text")
            .count(),
        1
    );
    let defect = events
        .iter()
        .find(|record| record["fields"]["diagnostic_code"] == "forbidden_token")
        .expect("located refusal");
    assert_eq!(defect["fields"]["path"], "note.md");
    assert_eq!(defect["fields"]["line"], 1);
}

/// `check-forbidden-text` also takes the git program by argument; a git
/// error (exit > 1) must surface only the status, never the raw child
/// stderr (ADR-010/ADR-015).
#[cfg(unix)]
#[test]
fn check_forbidden_text_omits_untrusted_git_stderr() {
    let secret = "SHOULD_NOT_APPEAR_grep_stderr_secret";

    let dir = tempfile::tempdir().expect("tempdir");
    let script = write_executable(
        &dir.path().join("fake-git.sh"),
        &format!("#!/bin/sh\necho '{secret}' >&2\nexit 2\n"),
    );

    let output = run(
        env!("CARGO_BIN_EXE_check-forbidden-text"),
        &[
            "--repository-root",
            dir.path().to_str().expect("utf8 path"),
            "--git",
            script.to_str().expect("utf8 path"),
        ],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "a failing git program is a runtime failure, not usage",
    );
    assert_eq!(output.stdout, [] as [u8; 0]);

    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    assert!(
        !stderr.contains(secret),
        "untrusted git stderr leaked into diagnostics: {stderr}",
    );

    assert_json_only_stderr(&output);
    assert!(
        stderr.contains("git grep failed"),
        "expected the generic git-failure message: {stderr}",
    );
}

// ---------------------------------------------------------------------------
// PTY-based TTY-refusal coverage (F1-024).
//
// Refusal depends on output *mode*, not binary identity: a direct-mode
// checker with a terminal stdout must refuse (exit 2, one tty_refusal
// record) before touching the filesystem, whereas build mode publishes a
// report/stamp and never refuses. `census-audit` exercises both.
// ---------------------------------------------------------------------------

#[cfg(unix)]
mod pty {
    use super::{SerializedCommand, write_executable};
    use std::fs::File;
    use std::io::{Read, Seek};
    use std::process::{Command, ExitStatus, Stdio};

    /// Run `command` with its stdout attached to a real pseudo-terminal.
    /// Returns the child status, the bytes the child wrote to the PTY, and
    /// its captured stderr.
    fn run_with_terminal_stdout(mut command: Command) -> (ExitStatus, Vec<u8>, Vec<u8>) {
        let pty = nix::pty::openpty(None, None).expect("open pty");
        let mut master = File::from(pty.master);
        let slave = File::from(pty.slave);

        let stderr_file = tempfile::tempfile().expect("stderr tempfile");
        let mut stderr_reader = stderr_file.try_clone().expect("clone stderr");

        // The child gets its own dup of the slave; the parent then drops
        // its slave so the only remaining slave is the child's.
        let child_slave = slave.try_clone().expect("clone slave");
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::from(child_slave))
            .stderr(Stdio::from(stderr_file))
            .spawn_serialized()
            .expect("command spawns");
        drop(slave);

        // Drain the master on a helper thread. In this sandbox the master
        // does not reliably report EIO/EOF once the slave closes, so the
        // thread may block on a final read; we collect via a channel with
        // a grace period and never join it, so the test cannot hang.
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
                    // EIO once the slave closes, or any other error: done.
                    Err(_) => break,
                }
            }
        });

        let status = child.wait().expect("child exits");

        let mut pty_stdout = Vec::new();
        while let Ok(chunk) = receiver.recv_timeout(std::time::Duration::from_millis(500)) {
            pty_stdout.extend_from_slice(&chunk);
        }

        stderr_reader.rewind().expect("rewind stderr");
        let mut stderr = Vec::new();
        stderr_reader.read_to_end(&mut stderr).expect("read stderr");

        (status, pty_stdout, stderr)
    }

    /// Parse the single JSON control-plane record of the given kind, if any.
    fn find_record(stderr: &[u8], kind: &str) -> Option<serde_json::Value> {
        let text = String::from_utf8(stderr.to_vec()).expect("stderr is utf8");
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            let value: serde_json::Value =
                serde_json::from_str(line).expect("every stderr line is one JSON object");
            if value["kind"] == kind {
                return Some(value);
            }
        }
        None
    }

    /// A fake git that emits an empty `ls-files -z` listing and succeeds.
    fn empty_git(dir: &std::path::Path) -> std::path::PathBuf {
        write_executable(&dir.join("empty-git.sh"), "#!/bin/sh\nexit 0\n")
    }

    fn census_audit() -> Command {
        Command::new(env!("CARGO_BIN_EXE_census-audit"))
    }

    #[test]
    fn census_audit_refuses_terminal_stdout_in_direct_mode() {
        let dir = tempfile::tempdir().expect("tempdir");

        let mut command = census_audit();
        command.args([
            "--repository-root",
            dir.path().to_str().unwrap(),
            // Never invoked: direct-mode refusal precedes the semantic
            // work, so the git program is irrelevant here.
            "--git",
            "/nonexistent/git",
            "--exclude-pattern",
            ".*",
        ]);
        command.env("RUST_LOG", "off");

        let (status, pty_stdout, stderr) = run_with_terminal_stdout(command);

        assert_eq!(status.code(), Some(2), "terminal stdout must be refused");
        assert!(pty_stdout.is_empty(), "refusal must not write result data");

        let record = find_record(&stderr, "tty_refusal").expect("one tty_refusal record");
        assert_eq!(record["fields"]["stream"], "stdout");
        assert_eq!(record["fields"]["exit_code"], 2);
    }

    #[test]
    fn census_audit_build_mode_does_not_refuse_terminal_stdout() {
        let dir = tempfile::tempdir().expect("tempdir");
        let git = empty_git(dir.path());
        let report = dir.path().join("census.json");
        let stamp = dir.path().join("census.stamp");

        let mut command = census_audit();
        command.args([
            "--repository-root",
            dir.path().to_str().unwrap(),
            "--git",
            git.to_str().unwrap(),
            "--exclude-pattern",
            ".*",
            "--report",
            report.to_str().unwrap(),
            "--stamp",
            stamp.to_str().unwrap(),
        ]);

        let (status, pty_stdout, stderr) = run_with_terminal_stdout(command);

        // Empty tracked set + empty declared census = a valid audit, so
        // build mode publishes the report and stamp and exits 0 even
        // though stdout is a terminal.
        assert_eq!(
            status.code(),
            Some(0),
            "build mode must not refuse a terminal"
        );
        assert!(pty_stdout.is_empty(), "build mode leaves stdout empty");
        assert!(report.exists(), "build mode publishes the report");
        assert!(stamp.exists(), "build mode touches the stamp");
        let report_bytes = std::fs::read(&report).expect("build report bytes");
        assert!(
            report_bytes.ends_with(b"\n"),
            "report is newline terminated"
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&report_bytes).expect("JSON report")["valid"],
            true
        );
        assert_eq!(std::fs::read(&stamp).expect("stamp bytes"), [] as [u8; 0]);
        assert!(
            find_record(&stderr, "tty_refusal").is_none(),
            "build mode must not emit a tty_refusal record",
        );
    }

    /// A fake git emitting one mode-bearing record for a tracked
    /// symlink. Using a stub keeps the wiring test independent of the
    /// host git's symlink handling; the parsing and policy are unit
    /// tested separately.
    fn symlink_git(dir: &std::path::Path) -> std::path::PathBuf {
        write_executable(
            &dir.join("symlink-git.sh"),
            "#!/bin/sh\nprintf '120000 aaaa 0\\tplans/alias.md\\0'\n",
        )
    }

    #[test]
    fn census_audit_fails_on_a_tracked_symlink_naming_path_and_mode() {
        // ADR-017: repository shape has one central owner. The audit
        // rejects a non-blob entry even when the declared census
        // agrees, and names both the path and the rejected mode.
        let dir = tempfile::tempdir().expect("tempdir");
        let git = symlink_git(dir.path());
        let report = dir.path().join("census.json");
        let stamp = dir.path().join("census.stamp");

        let output = census_audit()
            .args([
                "--repository-root",
                dir.path().to_str().unwrap(),
                "--git",
                git.to_str().unwrap(),
                "--exclude-pattern",
                "^$",
                "--report",
                report.to_str().unwrap(),
                "--stamp",
                stamp.to_str().unwrap(),
                "plans/alias.md",
            ])
            .output_serialized()
            .expect("census-audit runs");

        assert_eq!(output.status.code(), Some(1), "a non-blob entry fails");
        assert!(!stamp.exists(), "a failed audit leaves no success stamp");

        let events = super::records(&output);
        let record = events
            .iter()
            .find(|value| value["fields"]["mode"] == "120000")
            .expect("one mode-defect record");
        assert_eq!(record["fields"]["path"], "plans/alias.md");
        assert_eq!(record["fields"]["diagnostic_code"], "disallowed_mode");
        assert_eq!(
            record["fields"]["detail"],
            "tracked entry is not an ordinary blob; the repository carries no symlinks, gitlinks, or submodules"
        );
        let completion: Vec<_> = events
            .iter()
            .filter(|value| {
                value["target"] == "labels::census"
                    && value["fields"]["message"] == "tracked audit completed"
            })
            .collect();
        assert_eq!(completion.len(), 1, "census completion: {events:?}");
        assert_eq!(completion[0]["fields"]["outcome"], "completed");
        assert_eq!(completion[0]["fields"]["valid"], false);
        assert_eq!(
            events
                .iter()
                .filter(|value| value["target"] == "labels::census"
                    && value["fields"]["message"] == "span closed"
                    && value["span"]["name"] == "audit_census")
                .count(),
            1
        );
    }
}
