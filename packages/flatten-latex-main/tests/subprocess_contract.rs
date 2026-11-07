//! ADR-010 subprocess coverage for the `flatten-latex-main` binary.
//!
//! The binary owns no stdout result: control and diagnostic records are JSON
//! on stderr, while the flattened TeX is an explicit asset.

// TeX include literals such as `\input{section}` are source text, not format strings.
#![allow(clippy::literal_string_with_formatting_args)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};
use tempfile::tempdir;

const BINARY: &str = env!("CARGO_BIN_EXE_flatten-latex-main");
const COMMAND_NAME: &str = "flatten-latex-main";

fn run(args: &[&str], rust_log: &str) -> Output {
    Command::new(BINARY)
        .args(args)
        .env("RUST_LOG", rust_log)
        .output()
        .expect("binary runs")
}

fn run_flatten(main: &Path, file: &Path, output: &Path, rust_log: &str, debug: bool) -> Output {
    let mut command = Command::new(BINARY);
    command
        .arg("--main")
        .arg(main)
        .arg("--file")
        .arg(file)
        .arg("--output")
        .arg(output)
        .env("RUST_LOG", rust_log);
    if debug {
        command.arg("--debug");
    }
    command.output().expect("binary runs")
}

fn stderr_records(output: &Output) -> Vec<Value> {
    assert!(
        output.stdout.is_empty(),
        "this command has no stdout result"
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("stderr is utf8");
    assert!(stderr.ends_with('\n'), "stderr records end in newlines");
    let records: Vec<_> = stderr
        .lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line).expect("stderr line is JSON");
            assert!(value.is_object(), "stderr record is a JSON object");
            value
        })
        .collect();
    assert!(!records.is_empty(), "expected at least one stderr record");
    records
}

fn assert_control_record(output: &Output, kind: &str) -> Value {
    let records = stderr_records(output);
    assert_eq!(records.len(), 1, "one unfiltered control record");
    let record = records.into_iter().next().expect("control record");
    assert_eq!(record["schema"], json!(1));
    assert_eq!(record["command"], json!(COMMAND_NAME));
    assert_eq!(record["kind"], json!(kind));
    assert!(record["message"].is_string());
    record
}

fn assert_close_record(records: &[Value], target: &str, name: &str) {
    let closes: Vec<_> = records
        .iter()
        .filter(|record| {
            record["target"] == target
                && record["fields"]["message"] == "span closed"
                && record["span"]["name"] == name
        })
        .collect();
    assert_eq!(closes.len(), 1, "one {name} close record: {records:?}");
    assert!(closes[0]["fields"]["busy_ns"].is_u64());
    assert!(closes[0]["fields"]["idle_ns"].is_u64());
}

#[test]
fn missing_arguments_are_a_usage_error() {
    let output = run(&[], "off");
    assert_eq!(output.status.code(), Some(2));
    let record = assert_control_record(&output, "usage_error");
    assert_eq!(record["fields"]["exit_code"], json!(2));
}

#[test]
fn unknown_flag_is_a_usage_error() {
    let flag = "--definitely-not-a-real-flag";
    let output = run(&[flag], "off");
    assert_eq!(output.status.code(), Some(2));
    let record = assert_control_record(&output, "usage_error");
    assert_eq!(record["fields"]["exit_code"], json!(2));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(flag));
}

#[test]
fn help_and_version_exit_zero_with_empty_stdout() {
    for (flag, kind) in [("--help", "help"), ("--version", "version")] {
        let output = run(&[flag], "off");
        assert_eq!(output.status.code(), Some(0), "{flag}: exit 0");
        let record = assert_control_record(&output, kind);
        assert_eq!(record["fields"]["type"], json!(kind));
    }
}

#[test]
fn successful_flatten_has_empty_stdout_and_terminal_and_close_records() {
    let dir = tempdir().expect("tempdir");
    let main = dir.path().join("main.tex");
    let section = dir.path().join("section.tex");
    let plain = dir.path().join("plain.tex");
    let traced = dir.path().join("traced.tex");
    fs::write(&main, "\\input{section}\n").expect("main source");
    fs::write(&section, "Section text\n").expect("section source");

    let baseline = run_flatten(&main, &section, &plain, "off", false);
    assert_eq!(baseline.status.code(), Some(0));
    assert_eq!(baseline.stdout, Vec::<u8>::new());
    assert!(
        baseline.stderr.is_empty(),
        "disabled diagnostics stay quiet"
    );
    let enabled = run_flatten(&main, &section, &traced, "info", false);
    assert_eq!(enabled.status.code(), Some(0));
    assert_eq!(
        fs::read(&plain).expect("plain asset"),
        fs::read(&traced).expect("traced asset")
    );

    let records = stderr_records(&enabled);
    let completed: Vec<_> = records
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command completed"
        })
        .collect();
    assert_eq!(completed.len(), 1, "one terminal event");
    assert_eq!(completed[0]["fields"]["exit_class"], json!("success"));
    assert_eq!(completed[0]["fields"]["output_mode"], json!("none"));
    assert_eq!(completed[0]["fields"]["result_published"], json!(false));
    assert_close_record(&records, "cli_common::command", "command");
    assert_close_record(&records, "flatten_latex_main::flatten", "flatten");
    assert!(!String::from_utf8_lossy(&enabled.stderr).contains("flattened paper written"));
}

#[test]
fn refused_include_keeps_asset_and_emits_failure_records() {
    let dir = tempdir().expect("tempdir");
    let main = dir.path().join("main.tex");
    let section = dir.path().join("section.tex");
    let output = dir.path().join("flat.tex");
    fs::write(&main, "\\input{section}\n").expect("main source");
    fs::write(&section, "Section text\n").expect("section source");
    let control = run_flatten(&main, &section, &output, "off", false);
    assert_eq!(control.status.code(), Some(0));
    let prior = fs::read(&output).expect("control asset");

    fs::write(&main, "\\input{missing}\n").expect("refused source");
    let refused = run_flatten(&main, &section, &output, "", true);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(fs::read(&output).expect("prior asset"), prior);
    let records = stderr_records(&refused);
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["target"] == "flatten_latex_main::flatten"
                    && record["fields"]["outcome"] == "refused"
                    && record["fields"]["refusal_kind"] == "missing_include"
                    && record["level"] == "DEBUG"
            })
            .count(),
        1,
        "one DEBUG library refusal"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["target"] == "cli_common::command"
                    && record["fields"]["message"] == "command failed"
                    && record["fields"]["exit_class"] == "failure"
                    && record["level"] == "ERROR"
            })
            .count(),
        1,
        "one ERROR terminal failure"
    );
    assert_close_record(&records, "cli_common::command", "command");
}
