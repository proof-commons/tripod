//! ADR-010 control-plane coverage for `check-target-elements-prototypes`.
//!
//! The command runs as a real subprocess here: `--help` and `--version`
//! exit 0 writing nothing on stdout, a missing or unknown argument is
//! usage class 2, a relation this command does not run is a usage error
//! rather than a silently chosen matrix, a refused gate publishes no
//! report and dates no stamp, and no credential argument exists.
//!
//! The executor these tests select is the mock, so nothing here is
//! target evidence, and the gate says so.

#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_check-target-elements-prototypes");

/// A public development identifier, as the command spells them.
const NETWORK_ID: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const GENESIS_ID: &str = "2222222222222222222222222222222222222222222222222222222222222222";

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

#[test]
fn missing_arguments_are_usage_errors() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
}

#[test]
fn an_unknown_flag_is_a_usage_error() {
    let output = run(&["--definitely-not-a-real-flag"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
}

#[test]
fn a_relation_this_command_does_not_run_is_a_usage_error() {
    // The relation selects the matrix, and a name outside the two is
    // refused rather than resolved to a default. A default here would
    // run one relation's rows and file them under whichever role the
    // caller thought they had asked for.
    let output = run(&[
        "--relation",
        "not-a-relation",
        "--executor",
        "/nonexistent",
        "--executor-class",
        "mock",
        "--network-id",
        NETWORK_ID,
        "--genesis-id",
        GENESIS_ID,
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
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
fn a_declared_mock_run_cannot_satisfy_the_prototype_gate() {
    for relation in ["constructor-continuity", "wide-floor"] {
        let directory = tempfile::tempdir().expect("tempdir");
        let executor = wrapper(directory.path(), "echo-expected");
        let output = run(&[
            "--relation",
            relation,
            "--executor",
            executor.to_str().expect("utf8 path"),
            "--executor-class",
            "mock",
            "--network-id",
            NETWORK_ID,
            "--genesis-id",
            GENESIS_ID,
        ]);

        assert_eq!(
            output.status.code(),
            Some(1),
            "{relation}: a mock run is a runtime failure, not a usage error",
        );
        assert!(
            output.stdout.is_empty(),
            "{relation}: a failed check writes no result",
        );
        let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
        assert!(
            stderr.contains("mock executor cannot satisfy"),
            "{relation}: expected the mock refusal: {stderr}",
        );
    }
}

#[test]
fn a_refused_gate_publishes_no_report_and_dates_no_stamp() {
    let directory = tempfile::tempdir().expect("tempdir");
    let executor = wrapper(directory.path(), "echo-expected");
    let report = directory.path().join("report.json");
    let stamp = directory.path().join("stamp.ok");

    let output = run(&[
        "--relation",
        "wide-floor",
        "--executor",
        executor.to_str().expect("utf8 path"),
        "--executor-class",
        "mock",
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
    assert!(!report.exists(), "a refused run publishes no report asset");
    assert!(!stamp.exists(), "a refused run dates no stamp");
}
