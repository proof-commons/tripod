use std::process::Command;

use cli_common::{
    ControlPlaneRecord, emit_control_plane_record, install_command_subscriber,
    run_no_stdout_command,
};

const CHILD_MODE: &str = "CLI_COMMON_BOOTSTRAP_CHILD";

#[test]
fn subscriber_bootstrap_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    if mode == "runner" {
        install_command_subscriber("bootstrap-test");
        let exit = run_no_stdout_command("bootstrap-test", false, tracing::Level::INFO, || {
            Ok::<(), &'static str>(())
        });
        assert_eq!(exit, std::process::ExitCode::SUCCESS);
        return;
    }
    install_command_subscriber("bootstrap-test");
    let record = ControlPlaneRecord::help("bootstrap-test", "help text");
    emit_control_plane_record(&record).expect("serializable control record");
    if mode == "span" {
        let span = tracing::info_span!(target: "cli_common::command", "command", command = "bootstrap-test", output_mode = "none");
        let entered = span.enter();
        drop(entered);
        drop(span);
    }
    if mode == "diagnostic" {
        tracing::warn!(target: "cli_common::command", "child diagnostic");
    }
}

fn child_stderr(mode: &str, rust_log: &str) -> String {
    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .arg("--exact")
        .arg("subscriber_bootstrap_child")
        .arg("--nocapture")
        .env(CHILD_MODE, mode)
        .env("RUST_LOG", rust_log)
        .output()
        .expect("child process starts");
    assert!(output.status.success());
    String::from_utf8(output.stderr).expect("JSON stderr is UTF-8")
}

#[test]
fn subscriber_bootstrap_control_route_ignores_off_filter() {
    let stderr = child_stderr("control", "off");
    let expected = serde_json::to_string(&ControlPlaneRecord::help("bootstrap-test", "help text"))
        .expect("serializable control record");
    assert_eq!(stderr, format!("{expected}\n"));
}

#[test]
fn subscriber_bootstrap_invalid_filter_falls_back_without_echoing_value() {
    let stderr = child_stderr("diagnostic", "[invalid");
    assert!(!stderr.contains("[invalid"));
    let lines = stderr.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 3);
    let warning: serde_json::Value = serde_json::from_str(lines[0]).expect("JSON warning");
    assert_eq!(warning["fields"]["message"], "filter directive rejected");
    let expected = serde_json::to_string(&ControlPlaneRecord::help("bootstrap-test", "help text"))
        .expect("serializable control record");
    assert_eq!(lines[1], expected);
    let diagnostic: serde_json::Value = serde_json::from_str(lines[2]).expect("JSON diagnostic");
    assert_eq!(diagnostic["fields"]["message"], "child diagnostic");
}

#[test]
fn subscriber_bootstrap_enabled_span_has_one_numeric_close_record() {
    let stderr = child_stderr("span", "info");
    let lines = stderr.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    let close: serde_json::Value = serde_json::from_str(lines[1]).expect("JSON close record");
    assert_eq!(close["target"], "cli_common::command");
    assert_eq!(close["level"], "INFO");
    assert_eq!(close["span"]["name"], "command");
    assert!(close["fields"]["busy_ns"].is_u64());
    assert!(close["fields"]["idle_ns"].is_u64());
}

#[test]
fn subscriber_bootstrap_runner_success_has_event_and_close() {
    let stderr = child_stderr("runner", "info");
    let lines = stderr.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    let completed: serde_json::Value = serde_json::from_str(lines[0]).expect("JSON outcome");
    assert_eq!(completed["target"], "cli_common::command");
    assert_eq!(completed["level"], "INFO");
    assert_eq!(completed["fields"]["message"], "command completed");
    assert_eq!(completed["fields"]["exit_class"], "success");
    let close: serde_json::Value = serde_json::from_str(lines[1]).expect("JSON close record");
    assert_eq!(close["span"]["name"], "command");
    assert!(close["fields"]["busy_ns"].is_u64());
}
