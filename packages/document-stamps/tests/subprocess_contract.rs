//! Subprocess contract tests for the `attestation-stamps` binary.
//!
//! These tests exercise clap-level render-mode argument validation before
//! any Git or filesystem input is resolved.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

fn attestation_stamps() -> Command {
    Command::new(env!("CARGO_BIN_EXE_attestation-stamps"))
}

// Each flag selects whether one render-mode argument is present; the matrix of
// flags is the point of the fixture, so the bool count is intentional.
#[derive(Clone, Copy)]
struct RenderCase {
    name: &'static str,
    template: bool,
    stamps_output: bool,
    epoch_output: bool,
}

struct RenderPaths {
    template: PathBuf,
    stamps_output: PathBuf,
    epoch_output: PathBuf,
}

impl RenderPaths {
    fn new(root: &Path) -> Self {
        Self {
            template: root.join("stamps.tex.in"),
            stamps_output: root.join("stamps.tex"),
            epoch_output: root.join("source-date-epoch"),
        }
    }

    fn assert_absent(&self, name: &str) {
        assert!(!self.template.exists(), "{name}: template file was created");
        assert!(
            !self.stamps_output.exists(),
            "{name}: stamps output was created"
        );
        assert!(
            !self.epoch_output.exists(),
            "{name}: epoch output was created"
        );
    }
}

fn push_path_arg(args: &mut Vec<String>, flag: &str, path: &Path) {
    args.push(flag.to_string());
    args.push(path.to_str().expect("utf8 temp path").to_string());
}

fn base_args(root: &Path) -> Vec<String> {
    vec![
        "--git".to_string(),
        "git".to_string(),
        "--repository-root".to_string(),
        root.to_str().expect("utf8 temp path").to_string(),
        "--tree-ref".to_string(),
        "HEAD".to_string(),
        "--tree".to_string(),
        "papers/attestation".to_string(),
        "--input".to_string(),
        "papers/attestation/main.tex".to_string(),
    ]
}

fn run_case(root: &Path, paths: &RenderPaths, case: RenderCase) -> Output {
    let mut args = base_args(root);

    if case.template {
        push_path_arg(&mut args, "--template", &paths.template);
    }
    if case.stamps_output {
        push_path_arg(&mut args, "--stamps-output", &paths.stamps_output);
    }
    if case.epoch_output {
        push_path_arg(&mut args, "--epoch-output", &paths.epoch_output);
    }

    attestation_stamps()
        .args(args)
        .output()
        .expect("attestation-stamps runs")
}

fn assert_usage_error(name: &str, output: &Output) {
    assert_eq!(output.status.code(), Some(2), "{name}: exit code");
    assert!(output.stdout.is_empty(), "{name}: stdout must be empty");

    let stderr = std::str::from_utf8(&output.stderr).expect("stderr is utf8");
    let lines = stderr.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "{name}: expected one JSON record");

    let record: Value = serde_json::from_str(lines[0]).expect("usage record is JSON");
    assert_eq!(record["command"], json!("attestation-stamps"));
    assert_eq!(record["kind"], json!("usage_error"));
    assert_eq!(record["fields"]["type"], json!("usage_error"));
    assert_eq!(record["fields"]["exit_code"], json!(2));
    assert!(
        record["fields"]["clap_error_kind"].is_string(),
        "{name}: clap error kind must be present"
    );
}

#[test]
fn partial_render_argument_sets_are_usage_errors_without_side_effects() {
    // The complete render set is exactly {template, stamps-output,
    // epoch-output}; every proper non-empty subset must be a usage error.
    let cases = [
        RenderCase {
            name: "template only",
            template: true,
            stamps_output: false,
            epoch_output: false,
        },
        RenderCase {
            name: "stamps output only",
            template: false,
            stamps_output: true,
            epoch_output: false,
        },
        RenderCase {
            name: "epoch output only",
            template: false,
            stamps_output: false,
            epoch_output: true,
        },
        RenderCase {
            name: "template and stamps output",
            template: true,
            stamps_output: true,
            epoch_output: false,
        },
        RenderCase {
            name: "template and epoch output",
            template: true,
            stamps_output: false,
            epoch_output: true,
        },
        RenderCase {
            name: "stamps and epoch output without template",
            template: false,
            stamps_output: true,
            epoch_output: true,
        },
    ];

    for case in cases {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = RenderPaths::new(dir.path());
        let output = run_case(dir.path(), &paths, case);

        assert_usage_error(case.name, &output);
        paths.assert_absent(case.name);
    }
}

#[test]
fn the_removed_success_stamp_flag_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = RenderPaths::new(dir.path());

    let mut args = base_args(dir.path());
    push_path_arg(&mut args, "--template", &paths.template);
    push_path_arg(&mut args, "--stamps-output", &paths.stamps_output);
    push_path_arg(&mut args, "--epoch-output", &paths.epoch_output);
    push_path_arg(
        &mut args,
        "--stamp",
        &dir.path().join("attestation-stamps.ok"),
    );

    let output = attestation_stamps()
        .args(args)
        .output()
        .expect("attestation-stamps runs");

    assert_usage_error("removed --stamp flag", &output);
    paths.assert_absent("removed --stamp flag");
    assert!(
        !dir.path().join("attestation-stamps.ok").exists(),
        "no success stamp is written"
    );
}
