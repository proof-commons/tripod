//! ADR-010 control-plane coverage for the four shipped `emit-*` commands.
//!
//! # What this proves that `emit_tests.rs` cannot
//!
//! `G12-R03` split each command in two: the document is built by
//! `target_elements_conformance::emit`, where `src/tests/emit_tests.rs`
//! checks its content without spawning anything, and the binary keeps
//! only what a library test cannot reach — argument parsing, the shared
//! panic hook, the terminal refusal, the exit classes, and the write.
//! That second half is what runs here, as a real subprocess, because
//! every property it has is a property of a process: an exit status, a
//! stream, and a stream's absence.
//!
//! The four commands are covered together rather than one file each.
//! They sit on one `cli_common` runner, so a contract that held for one
//! and not another would be a difference between the commands, and a
//! table is the shape that shows it.
//!
//! # The terminal refusal is exercised, not inherited
//!
//! `run_stdout_json_command` refuses a terminal stdout before doing any
//! work, and until now no test had ever attached one. The `pty` module
//! below gives each command a real pseudo-terminal, following
//! `subprocess_contract.rs`, whose comments explain why the master is
//! drained on a detached thread in this sandbox.
//!
//! # The paths here are fixtures
//!
//! The run records these tests name are files this test creates, or
//! files that deliberately do not exist. Nothing here is target
//! evidence, and no record reaches a success path: what a well-formed
//! record produces is `emit_tests.rs`'s subject, and what a malformed or
//! absent one produces is this file's `(´[ADR015-rule:security:test-material]´)`.

#![cfg(unix)]

use std::path::Path;
use std::process::{Command, Output};

/// One shipped command, and the arguments it needs before any of its own
/// work begins.
struct Emitter {
    name: &'static str,
    binary: &'static str,
    /// Whether the command reads a run record named by `--run-record`.
    reads_a_run_record: bool,
}

/// Every shipped `emit-*` command.
///
/// The census is the point: a command added to this crate and left off
/// this list would have no contract coverage at all, and the meson
/// binary census is what makes that omission visible.
const EMITTERS: &[Emitter] = &[
    Emitter {
        name: "emit-conservation-matrix",
        binary: env!("CARGO_BIN_EXE_emit-conservation-matrix"),
        reads_a_run_record: false,
    },
    Emitter {
        name: "emit-normalization-matrix",
        binary: env!("CARGO_BIN_EXE_emit-normalization-matrix"),
        reads_a_run_record: false,
    },
    Emitter {
        name: "emit-normalization-report",
        binary: env!("CARGO_BIN_EXE_emit-normalization-report"),
        reads_a_run_record: true,
    },
    Emitter {
        name: "emit-lifecycle-report",
        binary: env!("CARGO_BIN_EXE_emit-lifecycle-report"),
        reads_a_run_record: true,
    },
];

/// A directory component chosen so that a diagnostic repeating the
/// caller's path could not do so by accident.
const PATH_MARKER: &str = "operator-private-directory";

fn run(binary: &str, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .output()
        .expect("binary runs")
}

fn run_with_log(binary: &str, args: &[&str], rust_log: &str) -> Output {
    Command::new(binary)
        .args(args)
        .env("RUST_LOG", rust_log)
        .output()
        .expect("binary runs")
}

fn stderr_records(output: &Output) -> Vec<serde_json::Value> {
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("stderr line is JSON"))
        .collect()
}

fn assert_command_completed(name: &str, output: &Output) {
    let records = stderr_records(output);
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["target"] == "cli_common::command"
                    && record["fields"]["message"] == "command completed"
                    && record["fields"]["result_published"] == true
            })
            .count(),
        1,
        "{name}: one terminal completion event: {records:?}"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["target"] == "cli_common::command"
                    && record["fields"]["message"] == "span closed"
                    && record["span"]["name"] == "command"
                    && record["fields"]["busy_ns"].is_u64()
                    && record["fields"]["idle_ns"].is_u64()
            })
            .count(),
        1,
        "{name}: one numeric command close record: {records:?}"
    );
}

/// Every stderr line is one JSON object, and stdout carries nothing.
fn assert_json_only_stderr(name: &str, output: &Output) {
    assert!(
        output.stdout.is_empty(),
        "{name}: a control-plane result must not write stdout",
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    let mut saw_record = false;
    for line in stderr.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|_| panic!("{name}: every stderr line is one JSON object: {line}"));
        assert!(
            value.is_object(),
            "{name}: each control-plane record is an object",
        );
        saw_record = true;
    }
    assert!(saw_record, "{name}: expected at least one JSON record");
}

/// No stream names the caller's own path.
fn assert_no_path_interpolation(name: &str, output: &Output, path: &Path) {
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    for forbidden in [PATH_MARKER, &*path.to_string_lossy()] {
        assert!(
            !stderr.contains(forbidden),
            "{name}: a diagnostic must not interpolate the caller's path: {stderr}",
        );
    }
}

/// A path under a distinctively named directory, created or not.
fn marked_path(directory: &Path, leaf: &str) -> std::path::PathBuf {
    let marked = directory.join(PATH_MARKER);
    std::fs::create_dir_all(&marked).expect("create marked directory");
    marked.join(leaf)
}

#[test]
fn a_matrix_command_writes_one_compact_json_object_and_nothing_else() {
    for emitter in EMITTERS.iter().filter(|one| !one.reads_a_run_record) {
        let output = run(emitter.binary, &[]);
        let name = emitter.name;

        assert_eq!(output.status.code(), Some(0), "{name}: a clean run exits 0");
        assert_command_completed(name, &output);

        // ADR-010's stdout result data: one compact JSON line, not a
        // pretty-printed block, so a reader can take it a line at a time.
        let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
        assert!(
            stdout.ends_with('\n'),
            "{name}: the result line is terminated",
        );
        assert_eq!(
            stdout.lines().count(),
            1,
            "{name}: the result is exactly one line",
        );
        let value: serde_json::Value =
            serde_json::from_str(stdout.trim_end()).expect("the result line is JSON");
        assert!(value.is_object(), "{name}: the result is one JSON object");
    }
}

#[test]
fn matrix_stdout_bytes_ignore_the_diagnostic_filter() {
    for emitter in EMITTERS.iter().filter(|one| !one.reads_a_run_record) {
        let normal = run_with_log(emitter.binary, &[], "info");
        let quiet = run_with_log(emitter.binary, &[], "off");
        assert_eq!(normal.status.code(), Some(0), "{}", emitter.name);
        assert_eq!(quiet.status.code(), Some(0), "{}", emitter.name);
        assert_eq!(normal.stdout, quiet.stdout, "{}", emitter.name);
        assert!(
            quiet.stderr.is_empty(),
            "{}: diagnostics filtered off",
            emitter.name
        );
    }
}

#[test]
fn panic_control_record_survives_disabled_diagnostics() {
    const CHILD: &str = "TRIPOD_TARGET_ELEMENTS_PANIC_PROBE_CHILD";
    if std::env::var_os(CHILD).is_some() {
        cli_common::install_command_subscriber("emit-panic-probe");
        panic!("private panic payload");
    }
    let output = Command::new(std::env::current_exe().expect("current test binary"))
        .args([
            "--exact",
            "panic_control_record_survives_disabled_diagnostics",
        ])
        .env(CHILD, "1")
        .env("RUST_LOG", "off")
        .output()
        .expect("panic probe runs");
    assert_eq!(output.status.code(), Some(1));
    let records = stderr_records(&output);
    assert_eq!(records.len(), 1, "one panic control record: {records:?}");
    assert_eq!(records[0]["kind"], "panic");
    assert_eq!(records[0]["command"], "emit-panic-probe");
    assert!(records[0]["fields"].get("payload").is_none());
}

#[test]
fn every_command_answers_help_and_version_as_control_plane_records() {
    for emitter in EMITTERS {
        for flag in ["--help", "--version"] {
            let output = run_with_log(emitter.binary, &[flag], "off");
            assert_eq!(
                output.status.code(),
                Some(0),
                "{}: {flag} exits 0",
                emitter.name,
            );
            assert_json_only_stderr(emitter.name, &output);
            let records = stderr_records(&output);
            assert_eq!(records.len(), 1, "{}: one control record", emitter.name);
            assert_eq!(
                records[0]["kind"],
                if flag == "--help" { "help" } else { "version" }
            );
        }
    }
}

#[test]
fn an_unknown_flag_is_a_usage_error_for_every_command() {
    for emitter in EMITTERS {
        let output = run_with_log(emitter.binary, &["--definitely-not-a-real-flag"], "off");
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}: an unknown flag is usage class 2",
            emitter.name,
        );
        assert_json_only_stderr(emitter.name, &output);
        let records = stderr_records(&output);
        assert_eq!(
            records.len(),
            1,
            "{}: one usage control record",
            emitter.name
        );
        assert_eq!(records[0]["kind"], "usage_error");
    }
}

#[test]
fn a_report_command_without_its_run_record_is_a_usage_error() {
    for emitter in EMITTERS.iter().filter(|one| one.reads_a_run_record) {
        let output = run(emitter.binary, &[]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}: a missing required argument is usage class 2",
            emitter.name,
        );
        assert_json_only_stderr(emitter.name, &output);
    }
}

#[test]
fn a_positional_run_record_is_a_usage_error() {
    // `G12-R03` replaced `std::env::args().nth(1)` with `--run-record
    // PATH`. A command that still accepted the positional form would
    // accept a caller that had not been migrated, silently.
    let directory = tempfile::tempdir().expect("tempdir");
    let record = marked_path(directory.path(), "record.json");
    std::fs::write(&record, b"{}").expect("write record");

    for emitter in EMITTERS.iter().filter(|one| one.reads_a_run_record) {
        let output = run(emitter.binary, &[record.to_str().expect("utf8 path")]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}: a positional run record is not an argument this command has",
            emitter.name,
        );
        assert_json_only_stderr(emitter.name, &output);
        assert_no_path_interpolation(emitter.name, &output, &record);
    }
}

#[test]
fn an_absent_run_record_is_a_runtime_failure_naming_no_path() {
    let directory = tempfile::tempdir().expect("tempdir");
    let record = marked_path(directory.path(), "there-is-no-such-record.json");

    for emitter in EMITTERS.iter().filter(|one| one.reads_a_run_record) {
        let output = run(
            emitter.binary,
            &["--run-record", record.to_str().expect("utf8 path")],
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}: an unreadable input is a runtime failure, not a usage error",
            emitter.name,
        );
        assert_json_only_stderr(emitter.name, &output);
        // `G12-R04`: an operator's directory layout is not part of this
        // command's diagnostic vocabulary.
        assert_no_path_interpolation(emitter.name, &output, &record);
    }
}

#[test]
fn a_malformed_run_record_is_a_runtime_failure_naming_no_path() {
    let directory = tempfile::tempdir().expect("tempdir");
    let record = marked_path(directory.path(), "record.json");
    std::fs::write(&record, b"this is not JSON").expect("write record");

    for emitter in EMITTERS.iter().filter(|one| one.reads_a_run_record) {
        let output = run(
            emitter.binary,
            &["--run-record", record.to_str().expect("utf8 path")],
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}: a malformed input is a runtime failure",
            emitter.name,
        );
        assert_json_only_stderr(emitter.name, &output);
        assert_no_path_interpolation(emitter.name, &output, &record);
    }
}

#[test]
fn a_well_formed_record_that_is_not_a_run_is_refused_rather_than_asserted() {
    // The gates these two commands apply moved into the library with
    // `G12-R03`, where an assertion became a refusal. The contract half
    // of that change is the exit class: a record that parses but says
    // nothing must leave the process by the failure path and publish no
    // result.
    let directory = tempfile::tempdir().expect("tempdir");
    let record = marked_path(directory.path(), "record.json");
    std::fs::write(&record, b"{}").expect("write record");

    for emitter in EMITTERS.iter().filter(|one| one.reads_a_run_record) {
        let output = run(
            emitter.binary,
            &["--run-record", record.to_str().expect("utf8 path")],
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}: an empty record is refused, not asserted on",
            emitter.name,
        );
        assert_json_only_stderr(emitter.name, &output);
        assert_no_path_interpolation(emitter.name, &output, &record);
    }
}

#[test]
fn no_command_defines_a_credential_argument() {
    for emitter in EMITTERS {
        let output = run(emitter.binary, &["--help"]);
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
                "{}: the command must define no credential argument: {forbidden}",
                emitter.name,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// PTY-based TTY-refusal coverage.
//
// Every one of these commands writes its result to stdout, so every one
// of them refuses a terminal there, before any of its own work. The
// refusal was inherited from `cli_common` and never exercised; it is
// exercised here.
// ---------------------------------------------------------------------------

mod pty {
    use std::fs::File;
    use std::io::{Read, Seek};
    use std::process::{Command, ExitStatus, Stdio};

    use super::EMITTERS;

    /// Runs `command` with its stdout attached to a real pseudo-terminal.
    ///
    /// The master is drained on a detached helper thread and the bytes
    /// arrive over a channel with a grace period, for the reason
    /// `subprocess_contract.rs` records: in this sandbox the master does
    /// not reliably report end of stream once the slave closes, so a
    /// joined thread could block forever.
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
    fn every_command_refuses_a_terminal_stdout() {
        for emitter in EMITTERS {
            let mut command = Command::new(emitter.binary);
            if emitter.reads_a_run_record {
                // The refusal happens before the record is read, so a
                // path that does not exist is the sharper fixture: a
                // command that reached the file would report the read
                // failure instead, and status 1 would say so.
                command.args(["--run-record", "/nonexistent-run-record.json"]);
            }
            let (status, stdout, stderr) = run_with_terminal_stdout(command);
            let name = emitter.name;

            assert_eq!(
                status.code(),
                Some(2),
                "{name}: terminal stdout is usage class 2 (ADR-010)",
            );
            assert!(
                stdout.is_empty(),
                "{name}: a refused command writes nothing to the terminal",
            );
            let stderr = String::from_utf8(stderr).expect("stderr is utf8");
            assert!(
                stderr.contains("tty_refusal"),
                "{name}: expected one tty_refusal record: {stderr}",
            );
        }
    }
}
