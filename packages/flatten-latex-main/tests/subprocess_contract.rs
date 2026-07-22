//! ADR-010 control-plane subprocess coverage for `flatten-latex-main`.
//!
//! Runs the shipped executable as a real subprocess and pins the uniform
//! control-plane contract: `--help`/`--version` exit 0 with no stdout noise, a
//! missing or unknown argument is usage class 2, and every control-plane
//! record is one JSON object on stderr while stdout stays empty.
//!
//! Success-path and TTY-refusal coverage are tracked separately under F1-024;
//! TTY refusal needs a PTY harness the workspace does not yet have, so that
//! lane remains incomplete.

use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_flatten-latex-main");

fn run(args: &[&str]) -> Output {
    Command::new(BINARY)
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
fn missing_arguments_are_a_usage_error() {
    let output = run(&[]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "no arguments must be usage class 2",
    );
    assert_json_only_stderr(&output);
}

#[test]
fn unknown_flag_is_a_usage_error() {
    let output = run(&["--definitely-not-a-real-flag"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "an unknown flag must be usage class 2",
    );
    assert_json_only_stderr(&output);
}

#[test]
fn help_and_version_exit_zero_with_empty_stdout() {
    for flag in ["--help", "--version"] {
        let output = run(&[flag]);
        assert_eq!(output.status.code(), Some(0), "{flag}: must exit 0");
        assert!(
            output.stdout.is_empty(),
            "{flag}: help/version is JSON on stderr, not stdout",
        );
    }
}
