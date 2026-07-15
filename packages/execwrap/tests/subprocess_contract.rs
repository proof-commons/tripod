//! End-to-end subprocess tests for the `execwrap` binary: real child
//! processes, real pipes, real log files. These pin the behaviors the
//! paper build depends on — byte-exact raw capture, per-stream
//! routing, exit-code propagation (including signals), ambiguous
//! routing refusal, and the data-loss failure policy.

use std::path::Path;
use std::process::{Command, Output};

fn execwrap() -> Command {
    Command::new(env!("CARGO_BIN_EXE_execwrap"))
}

fn run(args: &[&str]) -> Output {
    execwrap().args(args).output().expect("execwrap runs")
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).expect("log file readable")
}

/// `--redirect` (raw, merged) preserves arbitrary bytes: invalid
/// UTF-8, NUL, ANSI escapes, and a large binary chunk.
#[test]
fn raw_redirect_is_byte_exact() {
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("raw.log");

    // 64 KiB of 0xA5 plus invalid UTF-8, NUL, and an ANSI escape.
    let script = r"
        printf '\377\376\000\033[31m'
        head -c 65536 /dev/zero | tr '\000' '\245'
        printf '\n'
    ";

    let output = run(&[
        "--redirect",
        log.to_str().expect("utf8 path"),
        "--",
        "sh",
        "-c",
        script,
    ]);
    assert!(output.status.success());

    let written = read(&log);
    let mut expected: Vec<u8> = vec![0xff, 0xfe, 0x00, 0x1b, b'[', b'3', b'1', b'm'];
    expected.extend(std::iter::repeat_n(0xa5_u8, 65536));
    expected.push(b'\n');
    assert_eq!(written, expected, "raw redirection altered the byte stream");
}

/// Simultaneous stdout/stderr routing to separate files keeps each
/// stream's bytes exact and unmixed.
#[test]
fn split_streams_route_to_their_own_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_log = dir.path().join("out.log");
    let err_log = dir.path().join("err.log");

    let output = run(&[
        "--redirect-output",
        out_log.to_str().expect("utf8 path"),
        "--redirect-error",
        err_log.to_str().expect("utf8 path"),
        "--",
        "sh",
        "-c",
        "printf 'to-stdout\\n'; printf 'to-stderr\\n' >&2",
    ]);
    assert!(output.status.success());

    assert_eq!(read(&out_log), b"to-stdout\n");
    assert_eq!(read(&err_log), b"to-stderr\n");
}

/// A child's nonzero exit code is propagated unchanged.
#[test]
fn child_exit_code_is_propagated() {
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("raw.log");

    let output = run(&[
        "--redirect",
        log.to_str().expect("utf8 path"),
        "--",
        "sh",
        "-c",
        "exit 7",
    ]);
    assert_eq!(output.status.code(), Some(7));
}

/// Signal termination is reported as 128 + signal.
#[test]
fn signal_termination_maps_to_128_plus_signal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("raw.log");

    let output = run(&[
        "--redirect",
        log.to_str().expect("utf8 path"),
        "--",
        "sh",
        "-c",
        "kill -TERM $$",
    ]);
    // SIGTERM = 15.
    assert_eq!(output.status.code(), Some(128 + 15));
}

/// The same path configured for two different routing kinds is an
/// invalid argument combination: usage class 2, no side effects.
#[test]
fn ambiguous_same_path_routing_is_a_usage_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("both.log");
    let path = log.to_str().expect("utf8 path");

    let output = run(&["--redirect", path, "--redirect-output", path, "--", "true"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!log.exists(), "usage failure must not open log files");
}

/// Child argv is never logged: secrets on the command line must not
/// appear on either wrapper stream (ADR-010 redaction).
#[test]
fn child_argv_secrets_never_appear_in_diagnostics() {
    let secret = "SHOULD_NOT_APPEAR_hunter2";
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("raw.log");

    let output = execwrap()
        .args([
            "--debug",
            "--redirect",
            log.to_str().expect("utf8 path"),
            "--",
            "sh",
            "-c",
            "true",
            "sh",
            "--api-token",
            secret,
        ])
        .output()
        .expect("execwrap runs");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(secret), "secret leaked to stdout");
    assert!(
        !stderr.contains(secret),
        "secret leaked to stderr diagnostics"
    );
}

/// A credential-bearing URL in child argv must not appear on either
/// wrapper stream (B0-001: URLs with userinfo are secrets too).
#[test]
fn credential_bearing_url_never_appears_in_diagnostics() {
    let url = "https://user:hunter2-secret@example.invalid/private?api_key=SHOULD_NOT_APPEAR";
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("raw.log");

    let output = execwrap()
        .args([
            "--debug",
            "--redirect",
            log.to_str().expect("utf8 path"),
            "--",
            "sh",
            "-c",
            "true",
            "sh",
            "--endpoint",
            url,
        ])
        .output()
        .expect("execwrap runs");
    assert!(output.status.success());

    for (name, bytes) in [("stdout", &output.stdout), ("stderr", &output.stderr)] {
        let text = String::from_utf8_lossy(bytes);
        assert!(
            !text.contains("hunter2-secret") && !text.contains("SHOULD_NOT_APPEAR"),
            "credential URL leaked to {name}",
        );
    }
}

/// Redirecting only stderr must still relay stdout to the parent.
#[test]
fn redirect_error_only_still_forwards_stdout() {
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("err.log");

    let output = run(&[
        "--quiet",
        "--redirect-error",
        log.to_str().expect("utf8 path"),
        "--",
        "sh",
        "-c",
        "printf 'stdout-visible'; printf 'to-file' >&2",
    ]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"stdout-visible");
    assert_eq!(read(&log), b"to-file");
}

/// The minimal valid invocation: a separator and a child, no options.
#[test]
fn bare_separator_invocation_succeeds() {
    let output = run(&["--", "true"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
}

/// An unsubscribed child stream is relayed to the parent's stream even
/// when the parent stream is a pipe, never silently discarded.
#[test]
fn unredirected_streams_pass_through_pipes() {
    // No redirection at all: both streams relay.
    let output = run(&[
        "--quiet",
        "--",
        "sh",
        "-c",
        "printf 'out-bytes'; printf 'err-bytes' >&2",
    ]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"out-bytes");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("err-bytes"), "stderr bytes were discarded");

    // Only stdout redirected: stderr still relays.
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("out.log");
    let output = run(&[
        "--quiet",
        "--redirect-output",
        log.to_str().expect("utf8 path"),
        "--",
        "sh",
        "-c",
        "printf 'to-file'; printf 'still-visible' >&2",
    ]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty(), "redirected stdout must not relay");
    assert_eq!(read(&log), b"to-file");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("still-visible"), "stderr was discarded");
}

/// Help and version are control-plane paths: JSON on stderr, empty
/// stdout, exit 0. Usage failures exit 2 (ADR-010).
#[test]
fn help_version_and_usage_follow_the_output_contract() {
    for flag in ["--help", "--version"] {
        let output = run(&[flag]);
        assert_eq!(output.status.code(), Some(0), "{flag} must exit 0");
        assert!(output.stdout.is_empty(), "{flag} must not write stdout");

        let stderr = String::from_utf8(output.stderr).expect("stderr is text");
        let record: serde_json::Value =
            serde_json::from_str(stderr.lines().next().expect("one record"))
                .expect("stderr record is JSON");
        assert_eq!(record["command"], "execwrap");
    }

    // Unknown flag: usage class.
    let output = run(&["--no-such-flag", "--", "true"]);
    assert_eq!(output.status.code(), Some(2));

    // Missing `--` separator before the command: usage class.
    let output = run(&["true"]);
    assert_eq!(output.status.code(), Some(2));

    // No command at all: usage class.
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
}

/// Prefixed mode is a text presentation: streams are tagged and
/// control bytes are sanitized.
#[test]
fn prefixed_redirect_tags_streams() {
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("prefixed.log");

    let output = run(&[
        "--redirect-prefixed",
        log.to_str().expect("utf8 path"),
        "--",
        "sh",
        "-c",
        "printf 'out-line\\n'; printf 'err-line\\n' >&2",
    ]);
    assert!(output.status.success());

    let text = String::from_utf8(read(&log)).expect("prefixed log is text");
    assert!(text.contains(">>> out-line"));
    assert!(text.contains("*** err-line"));
}

/// Losing captured bytes fails the run even when the child exited 0:
/// writes to /dev/full fail with ENOSPC, so the wrapper must not
/// report success.
#[test]
fn data_loss_fails_a_successful_child() {
    if !Path::new("/dev/full").exists() {
        // Non-Linux fallback: nothing to exercise.
        return;
    }

    let output = run(&[
        "--redirect",
        "/dev/full",
        "--",
        "sh",
        "-c",
        "printf 'doomed bytes\\n'; exit 0",
    ]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "wrapper must fail when log bytes were lost",
    );
}

/// Diagnostics are JSON on stderr (ADR-010): a spawn failure produces
/// parseable JSON records and no stdout.
#[test]
fn spawn_failure_emits_json_diagnostics() {
    let output = run(&["--", "/nonexistent-binary-for-execwrap-test"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "stdout must stay empty");

    let stderr = String::from_utf8(output.stderr).expect("stderr is text");
    let mut saw_json = false;
    for line in stderr.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(line).expect("every stderr line is one JSON object");
        saw_json = true;
        assert!(value.is_object());
    }
    assert!(saw_json, "expected at least one JSON diagnostic");
}
