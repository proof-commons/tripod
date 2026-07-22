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

/// A child status in the relay range (3..=255) is propagated unchanged.
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

/// Reserved workspace exit classes survive child-status relay: 0 stays
/// success, a child 1 or 2 becomes wrapper runtime failure 1 (so code 2 stays
/// reserved for wrapper usage), and 3..=255 relay unchanged.
#[test]
fn reserved_wrapper_exit_classes_are_preserved() {
    let dir = tempfile::tempdir().expect("tempdir");

    for (child, wrapper) in [(0, 0), (1, 1), (2, 1), (3, 3), (42, 42)] {
        let log = dir.path().join(format!("relay-{child}.log"));
        let exit_command = format!("exit {child}");
        let output = run(&[
            "--redirect",
            log.to_str().expect("utf8 path"),
            "--",
            "sh",
            "-c",
            exit_command.as_str(),
        ]);
        assert_eq!(
            output.status.code(),
            Some(wrapper),
            "child {child} should map to wrapper {wrapper}",
        );
    }
}

/// A child that cannot be spawned is a wrapper runtime failure (1), not a
/// relayed child status.
#[test]
fn unspawnable_child_is_wrapper_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let log = dir.path().join("raw.log");

    let output = run(&[
        "--redirect",
        log.to_str().expect("utf8 path"),
        "--",
        "definitely-not-a-real-program-b3f1",
    ]);
    assert_eq!(output.status.code(), Some(1));
}

/// A spawn failure must not echo the caller-controlled program path:
/// `argv[0]` is raw child argv under ADR-010, so a secret embedded in
/// the executable path must stay out of the JSON diagnostics.
#[test]
fn spawn_failure_does_not_echo_child_program_text() {
    let secret = "SHOULD_NOT_APPEAR_spawn-program-secret";
    let program = format!("/nonexistent/{secret}");

    let output = run(&["--", program.as_str()]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, [] as [u8; 0]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(secret),
        "child program text leaked to diagnostics: {stderr}",
    );

    for line in stderr.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value = serde_json::from_str(line).expect("diagnostic is JSON");
        assert!(value.is_object());
    }
}

/// A credential-bearing URL supplied as the program path must not appear
/// in diagnostics on spawn failure either.
#[test]
fn spawn_failure_does_not_echo_credential_bearing_program_url() {
    let program = "https://user:hunter2-secret@example.invalid/SHOULD_NOT_APPEAR";

    let output = run(&["--", program]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, [] as [u8; 0]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("hunter2-secret") && !stderr.contains("SHOULD_NOT_APPEAR"),
        "credential program URL leaked to diagnostics: {stderr}",
    );
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
    assert_eq!(output.stdout, [] as [u8; 0]);
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
    assert_eq!(output.stdout, [] as [u8; 0]);
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

/// The production wrapper no longer knows any mock flags: test simulation is
/// unreachable through it. Each former flag is now an unknown argument (usage
/// class 2), so the wrapper cannot be made to fabricate outputs (F1-027).
#[test]
fn production_wrapper_rejects_mock_flags() {
    for flag in [
        &["--mock-child", "xelatex", "--", "true"][..],
        &["--mock-outdir", "/tmp", "--", "true"][..],
        &["--mock-fail", "--", "true"][..],
    ] {
        let output = run(flag);
        assert_eq!(
            output.status.code(),
            Some(2),
            "production execwrap must reject the mock flag {flag:?}",
        );
        assert!(output.stdout.is_empty(), "usage errors write no stdout");
    }
}

fn mock_tex() -> Command {
    Command::new(env!("CARGO_BIN_EXE_execwrap-mock-tex"))
}

/// The dedicated mock helper fabricates each child's deterministic outputs
/// under `--outdir`, emitting no stdout.
#[test]
fn mock_tex_writes_each_child_outputs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path();

    for (child, files) in [
        ("xelatex", &["main.bcf", "main.aux"][..]),
        ("biber", &["main.bbl"][..]),
        ("latexmk", &["main.pdf", "main.aux"][..]),
    ] {
        let output = mock_tex()
            .args([
                "--child",
                child,
                "--outdir",
                out.to_str().expect("utf8 path"),
            ])
            .output()
            .expect("mock tex runs");
        assert!(output.status.success(), "mock {child} should succeed");
        assert!(output.stdout.is_empty(), "mock {child} writes no stdout");
        for name in files {
            assert!(out.join(name).exists(), "mock {child} must write {name}");
        }
    }
}

/// `--fail` injects a render failure: exit 1 and no output written.
#[test]
fn mock_tex_fail_writes_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path();

    let output = mock_tex()
        .args([
            "--child",
            "latexmk",
            "--outdir",
            out.to_str().expect("utf8 path"),
            "--fail",
        ])
        .output()
        .expect("mock tex runs");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        !out.join("main.pdf").exists(),
        "--fail must write no output"
    );
}
