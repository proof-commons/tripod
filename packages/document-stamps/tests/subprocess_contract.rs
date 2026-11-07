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

fn run_git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", root.join("no-global-gitconfig"))
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("fixture Git runs");
    assert!(
        output.status.success(),
        "fixture Git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn committed_repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("fixture repository");
    let paper = dir.path().join("papers/attestation");
    std::fs::create_dir_all(&paper).expect("paper directory");
    std::fs::write(paper.join("main.tex"), b"main\n").expect("paper input");
    run_git(dir.path(), &["init", "-q", "--object-format=sha1"]);
    run_git(dir.path(), &["add", "--", "papers/attestation/main.tex"]);
    run_git(
        dir.path(),
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            "fixture",
        ],
    );
    dir
}

fn stderr_records(output: &Output) -> Vec<Value> {
    let stderr = std::str::from_utf8(&output.stderr).expect("stderr is utf8");
    stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("every stderr line is JSON"))
        .collect()
}

fn assert_command_success(records: &[Value], output_mode: &str, result_published: bool) {
    let completed: Vec<_> = records
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command completed"
        })
        .collect();
    assert_eq!(completed.len(), 1, "one terminal event: {records:?}");
    assert_eq!(completed[0]["level"], "INFO");
    assert_eq!(completed[0]["fields"]["exit_class"], "success");
    assert_eq!(completed[0]["fields"]["output_mode"], output_mode);
    assert_eq!(completed[0]["fields"]["result_published"], result_published);
}

fn assert_command_close(records: &[Value], output_mode: &str) {
    let closes: Vec<_> = records
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "span closed"
                && record["span"]["name"] == "command"
        })
        .collect();
    assert_eq!(closes.len(), 1, "one command close: {records:?}");
    let close = closes[0];
    assert_eq!(close["level"], "INFO");
    assert_eq!(close["span"]["command"], "attestation-stamps");
    assert_eq!(close["span"]["output_mode"], output_mode);
    assert!(close["fields"]["busy_ns"].is_u64());
    assert!(close["fields"]["idle_ns"].is_u64());
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

#[test]
fn successful_modes_keep_stdout_contract_and_emit_command_records() {
    let dir = committed_repository();
    let args = base_args(dir.path());
    let output = attestation_stamps()
        .args(&args)
        .env("RUST_LOG", "info")
        .output()
        .expect("JSON mode runs");
    assert_eq!(output.status.code(), Some(0));
    let stdout = std::str::from_utf8(&output.stdout).expect("stdout is utf8");
    assert!(stdout.ends_with('\n'));
    assert_eq!(stdout.lines().count(), 1, "one JSON result line");
    let result: Value = serde_json::from_str(stdout).expect("result is JSON");
    assert!(result["date"].is_string());
    assert!(result["timestamp"]["epoch"].is_i64());
    assert!(result["document_uuid"].is_string());
    assert!(result["instance_uuid"].is_string());
    let records = stderr_records(&output);
    assert_command_success(&records, "stdout", true);
    assert_command_close(&records, "stdout");

    let filtered = attestation_stamps()
        .args(&args)
        .env("RUST_LOG", "off")
        .output()
        .expect("filtered JSON mode runs");
    assert_eq!(filtered.status.code(), Some(0));
    assert_eq!(filtered.stdout, output.stdout);
    assert!(
        filtered.stderr.is_empty(),
        "filtered diagnostics are silent"
    );

    let paths = RenderPaths::new(dir.path());
    std::fs::write(&paths.template, "date=@ATTESTATION_DATE@\n").expect("render template");
    let mut render_args = args;
    push_path_arg(&mut render_args, "--template", &paths.template);
    push_path_arg(&mut render_args, "--stamps-output", &paths.stamps_output);
    push_path_arg(&mut render_args, "--epoch-output", &paths.epoch_output);
    let output = attestation_stamps()
        .args(render_args)
        .env("RUST_LOG", "info")
        .output()
        .expect("render mode runs");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty(), "render mode leaves stdout empty");
    assert_eq!(
        std::fs::read_to_string(&paths.stamps_output).expect("rendered stamps"),
        format!("date={}\n", result["date"].as_str().expect("date"))
    );
    assert_eq!(
        std::fs::read_to_string(&paths.epoch_output).expect("rendered epoch"),
        format!(
            "{}\n",
            result["timestamp"]["epoch"].as_i64().expect("epoch")
        )
    );
    let records = stderr_records(&output);
    assert_command_success(&records, "none", false);
    assert_command_close(&records, "none");
}

#[test]
fn help_version_and_usage_control_records_ignore_diagnostic_filter() {
    for (flag, kind, status) in [("--help", "help", 0), ("--version", "version", 0)] {
        let output = attestation_stamps()
            .arg(flag)
            .env("RUST_LOG", "off")
            .output()
            .expect("control invocation runs");
        assert_eq!(output.status.code(), Some(status));
        assert_eq!(output.stdout.as_slice(), b"");
        let records = stderr_records(&output);
        assert_eq!(records.len(), 1, "one {kind} control record");
        assert_eq!(records[0]["command"], "attestation-stamps");
        assert_eq!(records[0]["kind"], kind);
        assert!(records[0]["schema"].is_number());
    }

    let output = attestation_stamps()
        .arg("--unknown-secret=do-not-echo")
        .env("RUST_LOG", "off")
        .output()
        .expect("usage invocation runs");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout.as_slice(), b"");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("do-not-echo"));
    let records = stderr_records(&output);
    assert_eq!(records.len(), 1, "one usage control record");
    assert_eq!(records[0]["command"], "attestation-stamps");
    assert_eq!(records[0]["kind"], "usage_error");
}

#[test]
fn runtime_failure_has_empty_stdout_and_subscriber_error() {
    let dir = committed_repository();
    let mut args = base_args(dir.path());
    let missing_git = dir.path().join("missing-private-git");
    args[1] = missing_git.to_str().expect("utf8 temp path").to_owned();
    let output = attestation_stamps()
        .args(args)
        .env("RUST_LOG", "info")
        .output()
        .expect("failing command runs");
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "failed result leaves stdout empty"
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("missing-private-git"));
    let records = stderr_records(&output);
    let failures: Vec<_> = records
        .iter()
        .filter(|record| {
            record["target"] == "cli_common::command"
                && record["fields"]["message"] == "command failed"
        })
        .collect();
    assert_eq!(failures.len(), 1, "one terminal failure: {records:?}");
    assert_eq!(failures[0]["level"], "ERROR");
    assert_eq!(failures[0]["fields"]["exit_class"], "failure");
    assert_eq!(failures[0]["fields"]["phase"], "execute");
    assert!(failures[0]["fields"]["error"].is_string());
    assert_command_close(&records, "stdout");
}

#[test]
fn early_panic_is_an_unfiltered_subscriber_control_record() {
    const CHILD: &str = "TRIPOD_DOCUMENT_STAMPS_PANIC_TEST_CHILD";
    if std::env::var_os(CHILD).is_some() {
        cli_common::install_command_subscriber("attestation-stamps");
        panic!("private-panic-marker");
    }

    let output = Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg("early_panic_is_an_unfiltered_subscriber_control_record")
        .env(CHILD, "1")
        .env("RUST_LOG", "off")
        .output()
        .expect("panic child runs");
    assert_eq!(output.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private-panic-marker"));
    let records = stderr_records(&output);
    assert_eq!(records.len(), 1, "one panic control record");
    assert_eq!(records[0]["command"], "attestation-stamps");
    assert_eq!(records[0]["kind"], "panic");
    assert_eq!(records[0]["fields"]["exit_code"], 1);
    assert!(records[0]["fields"]["payload"].is_null());
}

// ---------------------------------------------------------------------------
// PTY-based TTY-refusal coverage (F1-024).
//
// JSON mode writes a result to stdout and refuses a terminal (exit 2, one
// tty_refusal record) before any Git work. Render mode is a side-effect
// build output with no stdout result and never refuses a terminal. This
// pins that refusal depends on output mode, not binary identity.
// ---------------------------------------------------------------------------

#[cfg(unix)]
mod pty {
    use std::fs::File;
    use std::io::{Read, Seek};
    use std::path::Path;
    use std::process::{Command, ExitStatus, Stdio};

    use serde_json::Value;

    fn attestation_stamps() -> Command {
        Command::new(env!("CARGO_BIN_EXE_attestation-stamps"))
    }

    fn json_mode_args(root: &Path) -> Vec<String> {
        // A parseable JSON-mode invocation (no render arguments). The Git
        // program is never invoked: JSON mode refuses a terminal stdout
        // before doing any work.
        vec![
            "--git".to_string(),
            "/nonexistent/git".to_string(),
            "--repository-root".to_string(),
            root.to_str().unwrap().to_string(),
            "--tree".to_string(),
            "papers/attestation".to_string(),
            "--input".to_string(),
            "papers/attestation/main.tex".to_string(),
        ]
    }

    /// Run `command` with its stdout attached to a real pseudo-terminal.
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
            .spawn()
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

    fn find_record(stderr: &[u8], kind: &str) -> Option<Value> {
        let text = String::from_utf8(stderr.to_vec()).expect("stderr is utf8");
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value =
                serde_json::from_str(line).expect("every stderr line is one JSON object");
            if value["kind"] == kind {
                return Some(value);
            }
        }
        None
    }

    #[test]
    fn json_mode_refuses_terminal_stdout() {
        let dir = tempfile::tempdir().expect("tempdir");

        let mut command = attestation_stamps();
        command.args(json_mode_args(dir.path()));
        command.env("RUST_LOG", "off");

        let (status, pty_stdout, stderr) = run_with_terminal_stdout(command);

        assert_eq!(status.code(), Some(2), "terminal stdout must be refused");
        assert!(pty_stdout.is_empty(), "refusal must not write result data");

        let record = find_record(&stderr, "tty_refusal").expect("one tty_refusal record");
        assert_eq!(record["fields"]["stream"], "stdout");
        assert_eq!(record["fields"]["exit_code"], 2);
        assert_eq!(
            String::from_utf8(stderr)
                .expect("stderr is utf8")
                .lines()
                .count(),
            1,
            "control route is the only stderr record with RUST_LOG=off"
        );
    }

    #[test]
    fn render_mode_does_not_refuse_terminal_stdout() {
        let dir = tempfile::tempdir().expect("tempdir");

        let mut args = json_mode_args(dir.path());
        for (flag, name) in [
            ("--template", "stamps.tex.in"),
            ("--stamps-output", "stamps.tex"),
            ("--epoch-output", "source-date-epoch"),
        ] {
            args.push(flag.to_string());
            args.push(dir.path().join(name).to_str().unwrap().to_string());
        }

        let mut command = attestation_stamps();
        command.args(args);

        let (status, pty_stdout, stderr) = run_with_terminal_stdout(command);

        // Render mode is a side-effect build output: it never refuses a
        // terminal. Here it fails for an unrelated reason (the Git program
        // does not exist), which is a runtime failure (1), never the usage
        // refusal class (2), and it emits no tty_refusal record.
        assert_ne!(
            status.code(),
            Some(2),
            "render mode must not refuse a terminal"
        );
        assert!(pty_stdout.is_empty(), "render mode leaves stdout empty");
        assert!(
            find_record(&stderr, "tty_refusal").is_none(),
            "render mode must not emit a tty_refusal record",
        );
    }
}
