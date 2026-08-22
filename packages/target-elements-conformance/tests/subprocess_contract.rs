//! ADR-010 control-plane coverage for `check-target-elements-native`.
//!
//! The command runs as a real subprocess here: `--help` and `--version`
//! exit 0 writing nothing on stdout, a missing or unknown argument is
//! usage class 2, a refused gate publishes no report and dates no stamp,
//! and direct mode refuses a terminal stdout before doing any work.
//!
//! The executor these tests select is the mock, so nothing here is
//! target evidence, and the gate says so.

#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_check-target-elements-native");

/// A public development identifier, as the command spells them.
const NETWORK_ID: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const GENESIS_ID: &str = "2222222222222222222222222222222222222222222222222222222222222222";

/// An ADR-018 provenance expectation, as the command spells one.
///
/// Forty lowercase hexadecimal digits each. They name no object in any
/// repository and do not need to: what these tests exercise is the
/// command's own argument handling.
const INTENDED_TIP: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678";
const UPSTREAM_BASE: &str = "0123456789abcdef0123456789abcdef01234567";

fn run(args: &[&str]) -> Output {
    Command::new(BINARY)
        .args(args)
        .output()
        .expect("binary runs")
}

/// A wrapper script selecting one mock behaviour.
fn wrapper(directory: &Path, behavior: &str) -> PathBuf {
    let path = directory.join(format!("executor-{behavior}.sh"));
    let mut file = std::fs::File::create(&path).expect("create wrapper");
    writeln!(file, "#!/bin/sh").expect("write wrapper");
    writeln!(
        file,
        "exec {} --behavior {behavior} \"$@\"",
        env!("CARGO_BIN_EXE_mock-native-executor"),
    )
    .expect("write wrapper");
    let mut permissions = file.metadata().expect("metadata").permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions).expect("set mode");
    drop(file);
    path
}

fn assert_json_only_stderr(output: &Output) {
    assert!(
        output.stdout.is_empty(),
        "a control-plane result must not write stdout",
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    let mut saw_record = false;
    for line in stderr.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value =
            serde_json::from_str(line).expect("every stderr line is one JSON object");
        assert!(value.is_object(), "each control-plane record is an object");
        saw_record = true;
    }
    assert!(saw_record, "expected at least one JSON record");
}

#[test]
fn missing_arguments_are_usage_errors() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert_json_only_stderr(&output);
}

#[test]
fn an_unknown_flag_is_a_usage_error() {
    let output = run(&["--definitely-not-a-real-flag"]);
    assert_eq!(output.status.code(), Some(2));
    assert_json_only_stderr(&output);
}

#[test]
fn a_lone_report_destination_is_a_usage_error() {
    // `--report` and `--stamp` are all-or-nothing (ADR-014).
    let output = run(&[
        "--executor",
        "/nonexistent",
        "--executor-class",
        "mock",
        "--executor-diagnostics-directory",
        "/nonexistent-diagnostics",
        "--network-id",
        NETWORK_ID,
        "--genesis-id",
        GENESIS_ID,
        "--report",
        "/tmp/does-not-matter.json",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert_json_only_stderr(&output);
}

#[test]
fn help_and_version_exit_zero_with_empty_stdout() {
    for flag in ["--help", "--version"] {
        let output = run(&[flag]);
        assert_eq!(output.status.code(), Some(0), "{flag}: must exit 0");
        assert!(
            output.stdout.is_empty(),
            "{flag}: help and version are JSON on stderr",
        );
    }
}

#[test]
fn no_credential_argument_exists() {
    let output = run(&["--help"]);
    let help = String::from_utf8(output.stderr).expect("stderr is utf8");
    for forbidden in [
        "--rpc-user",
        "--rpc-password",
        "--cookie",
        "--token",
        "--wallet",
        "--private-key",
    ] {
        assert!(
            !help.contains(forbidden),
            "the command must define no credential argument: {forbidden}",
        );
    }
}

#[test]
fn a_declared_mock_run_cannot_satisfy_the_gate() {
    let directory = tempfile::tempdir().expect("tempdir");
    let executor = wrapper(directory.path(), "answer-from-census");
    // The run keeps its executor diagnostics beside the executor it
    // selected, which is where a temporary directory puts everything
    // else this test owns.
    let diagnostics = directory
        .path()
        .to_str()
        .expect("the temporary path is text");
    let output = run(&[
        "--executor",
        executor.to_str().expect("utf8 path"),
        "--executor-class",
        "mock",
        "--executor-diagnostics-directory",
        diagnostics,
        "--network-id",
        NETWORK_ID,
        "--genesis-id",
        GENESIS_ID,
    ]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "a mock run is a runtime failure, not a usage error",
    );
    assert!(output.stdout.is_empty(), "a failed check writes no result");
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(
        stderr.contains("mock executor cannot satisfy"),
        "expected the mock refusal: {stderr}",
    );
}

#[test]
fn declaring_a_mock_reviewed_no_longer_reaches_the_gate() {
    // `G11-R06`. The harness still cannot tell a mock from an
    // interpreter, so a dishonest declaration still gets past the mock
    // refusal — and the census such a wrapper "passes" is the census's
    // own expectations, which the mock holds a copy of. What stops it
    // now is the provenance comparison: the mock reports no revision of
    // its own, no intended tip, and no upstream base, so it cannot be
    // the executable the operator declared they built.
    let directory = tempfile::tempdir().expect("tempdir");
    let executor = wrapper(directory.path(), "answer-from-census");
    // The run keeps its executor diagnostics beside the executor it
    // selected, which is where a temporary directory puts everything
    // else this test owns.
    let diagnostics = directory
        .path()
        .to_str()
        .expect("the temporary path is text");
    let output = run(&[
        "--executor",
        executor.to_str().expect("utf8 path"),
        "--executor-class",
        "reviewed-non-mock",
        "--executor-diagnostics-directory",
        diagnostics,
        "--network-id",
        NETWORK_ID,
        "--genesis-id",
        GENESIS_ID,
        "--intended-tip",
        INTENDED_TIP,
        "--upstream-base",
        UPSTREAM_BASE,
    ]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "an unidentified executable is a runtime refusal, not a usage error",
    );
    assert!(output.stdout.is_empty(), "a refused run publishes nothing");
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(
        stderr.contains("provenance"),
        "expected the provenance refusal: {stderr}",
    );
}

#[test]
fn a_reviewed_declaration_without_provenance_is_a_configuration_failure() {
    // `G11-R07`. The class alone does not configure an evidence run:
    // ADR-018's binding values are required with it, and their absence
    // is caught before any executor starts.
    let output = run(&[
        "--executor",
        "/nonexistent-executor",
        "--executor-class",
        "reviewed-non-mock",
        "--executor-diagnostics-directory",
        "/nonexistent-diagnostics",
        "--network-id",
        NETWORK_ID,
        "--genesis-id",
        GENESIS_ID,
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "a refused run publishes nothing");
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(
        stderr.contains("intended executed tip"),
        "expected the missing-tip refusal: {stderr}",
    );
}

#[test]
fn a_refused_gate_publishes_no_report_and_dates_no_stamp() {
    let directory = tempfile::tempdir().expect("tempdir");
    let executor = wrapper(directory.path(), "answer-from-census");
    // The run keeps its executor diagnostics beside the executor it
    // selected, which is where a temporary directory puts everything
    // else this test owns.
    let diagnostics = directory
        .path()
        .to_str()
        .expect("the temporary path is text");
    let report = directory.path().join("report.json");
    let stamp = directory.path().join("stamp.ok");

    let output = run(&[
        "--executor",
        executor.to_str().expect("utf8 path"),
        "--executor-class",
        "mock",
        "--executor-diagnostics-directory",
        diagnostics,
        "--network-id",
        NETWORK_ID,
        "--genesis-id",
        GENESIS_ID,
        "--report",
        report.to_str().expect("utf8 path"),
        "--stamp",
        stamp.to_str().expect("utf8 path"),
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(!report.exists(), "a refused run publishes no report");
    assert!(!stamp.exists(), "a refused run touches no stamp");
}

#[test]
fn a_refused_gate_leaves_an_existing_stamp_untouched() {
    let directory = tempfile::tempdir().expect("tempdir");
    let executor = wrapper(directory.path(), "answer-from-census");
    // The run keeps its executor diagnostics beside the executor it
    // selected, which is where a temporary directory puts everything
    // else this test owns.
    let diagnostics = directory
        .path()
        .to_str()
        .expect("the temporary path is text");
    let report = directory.path().join("report.json");
    let stamp = directory.path().join("stamp.ok");
    std::fs::write(&stamp, b"earlier bytes").expect("write stamp");
    let before = std::fs::metadata(&stamp).expect("stamp metadata");

    let output = run(&[
        "--executor",
        executor.to_str().expect("utf8 path"),
        "--executor-class",
        "mock",
        "--executor-diagnostics-directory",
        diagnostics,
        "--network-id",
        NETWORK_ID,
        "--genesis-id",
        GENESIS_ID,
        "--report",
        report.to_str().expect("utf8 path"),
        "--stamp",
        stamp.to_str().expect("utf8 path"),
    ]);

    assert_eq!(output.status.code(), Some(1));
    let after = std::fs::metadata(&stamp).expect("stamp metadata");
    assert_eq!(
        std::fs::read(&stamp).expect("read stamp"),
        b"earlier bytes",
        "a failing run must not rewrite an existing stamp",
    );
    assert_eq!(
        before.modified().expect("mtime"),
        after.modified().expect("mtime"),
        "a failing run must not re-date an existing stamp",
    );
}

#[test]
fn a_malformed_identifier_is_a_runtime_failure() {
    let directory = tempfile::tempdir().expect("tempdir");
    let executor = wrapper(directory.path(), "answer-from-census");
    // The run keeps its executor diagnostics beside the executor it
    // selected, which is where a temporary directory puts everything
    // else this test owns.
    let diagnostics = directory
        .path()
        .to_str()
        .expect("the temporary path is text");
    let output = run(&[
        "--executor",
        executor.to_str().expect("utf8 path"),
        "--executor-class",
        "mock",
        "--executor-diagnostics-directory",
        diagnostics,
        "--network-id",
        "not-hex",
        "--genesis-id",
        GENESIS_ID,
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_json_only_stderr(&output);
}

// ---------------------------------------------------------------------------
// PTY-based TTY-refusal coverage.
//
// Direct mode refuses a terminal stdout before any semantic work, so the
// executor never runs; build mode has an explicit destination and does
// not refuse.
// ---------------------------------------------------------------------------

mod pty {
    use std::fs::File;
    use std::io::{Read, Seek};
    use std::process::{Command, ExitStatus, Stdio};

    use super::{BINARY, GENESIS_ID, NETWORK_ID};

    /// Runs `command` with its stdout attached to a real pseudo-terminal.
    fn run_with_terminal_stdout(mut command: Command) -> (ExitStatus, Vec<u8>, Vec<u8>) {
        let pty = nix::pty::openpty(None, None).expect("open pty");
        let mut master = File::from(pty.master);
        let slave = File::from(pty.slave);

        let stderr_file = tempfile::tempfile().expect("stderr tempfile");
        let mut stderr_reader = stderr_file.try_clone().expect("clone stderr");

        let child_slave = slave.try_clone().expect("clone slave");
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::from(child_slave))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .expect("command spawns");
        drop(slave);

        // The master is drained on a detached helper thread: in this
        // sandbox it does not reliably report end of stream once the
        // slave closes, so the thread may block on a final read. The
        // bytes arrive over a channel with a grace period and the thread
        // is never joined, so the test cannot hang.
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

        let status = child.wait().expect("child exits");

        let mut stdout = Vec::new();
        while let Ok(chunk) = receiver.recv_timeout(std::time::Duration::from_millis(500)) {
            stdout.extend_from_slice(&chunk);
        }

        stderr_reader.rewind().expect("rewind stderr");
        let mut stderr = Vec::new();
        stderr_reader.read_to_end(&mut stderr).expect("read stderr");

        (status, stdout, stderr)
    }

    #[test]
    fn direct_mode_refuses_a_terminal_stdout() {
        let mut command = Command::new(BINARY);
        command.args([
            "--executor",
            "/nonexistent-executor",
            "--executor-class",
            "reviewed-non-mock",
            "--executor-diagnostics-directory",
            "/nonexistent-diagnostics",
            "--network-id",
            NETWORK_ID,
            "--genesis-id",
            GENESIS_ID,
        ]);
        let (status, stdout, stderr) = run_with_terminal_stdout(command);

        assert_eq!(
            status.code(),
            Some(2),
            "terminal stdout is usage class 2 (ADR-010)",
        );
        assert!(
            stdout.is_empty(),
            "a refused command writes nothing to the terminal",
        );
        let stderr = String::from_utf8(stderr).expect("stderr is utf8");
        assert!(
            stderr.contains("tty_refusal"),
            "expected one tty_refusal record: {stderr}",
        );
    }
}
