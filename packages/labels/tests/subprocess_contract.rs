//! ADR-010 control-plane subprocess coverage for the labels binaries.
//!
//! These run the shipped executables as real subprocesses and pin the uniform
//! control-plane contract: `--help`/`--version` exit 0 with no stdout noise, a
//! missing or unknown argument is usage class 2, and every control-plane
//! record is one JSON object on stderr while stdout stays empty.
//!
//! Success-path (the full repository census argv) and TTY-refusal coverage are
//! tracked separately under F1-024; TTY refusal needs a PTY harness the
//! workspace does not yet have, so that lane remains incomplete.

use std::process::{Command, Output};

const BINARIES: [&str; 4] = [
    env!("CARGO_BIN_EXE_check-labels"),
    env!("CARGO_BIN_EXE_census-audit"),
    env!("CARGO_BIN_EXE_check-forbidden-text"),
    env!("CARGO_BIN_EXE_generate-label-registers"),
];

fn run(binary: &str, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .output()
        .expect("binary runs")
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

/// The git program is argument-supplied, so its stderr is arbitrary
/// child output. A failing git must surface only the process status and
/// a generic message — never the raw child stderr (F1-032, ADR-010).
#[cfg(unix)]
#[test]
fn census_audit_omits_untrusted_git_stderr() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let secret = "SHOULD_NOT_APPEAR_git_stderr_secret";

    let dir = tempfile::tempdir().expect("tempdir");
    let script = dir.path().join("fake-git.sh");
    let mut file = std::fs::File::create(&script).expect("create script");
    writeln!(file, "#!/bin/sh").unwrap();
    writeln!(file, "echo '{secret}' >&2").unwrap();
    writeln!(file, "exit 17").unwrap();
    let mut perms = file.metadata().unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).unwrap();
    drop(file);

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
    assert!(output.stdout.is_empty());

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
}
