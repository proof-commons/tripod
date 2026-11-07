use std::sync::{Arc, Mutex};

use serde_json::{Map, Value, json};
use tracing::field::{Field, Visit};
use tracing::span::Attributes;
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

use crate::{CommandExit, PublicationAsset, publish_batch, run_no_stdout_command};

#[derive(Clone, Debug)]
struct Observation {
    target: String,
    level: String,
    name: Option<String>,
    fields: Map<String, Value>,
}

struct Capture(Arc<Mutex<Vec<Observation>>>);

static CAPTURE_INSTALL_LOCK: Mutex<()> = Mutex::new(());
const CAPTURE_CHILD: &str = "CLI_COMMON_CAPTURE_CHILD";

fn run_in_isolated_capture_process(test_name: &str) -> bool {
    if std::env::var_os(CAPTURE_CHILD).is_some() {
        return true;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg(format!("tests::instrumentation::{test_name}"))
        .env(CAPTURE_CHILD, "1")
        .output()
        .expect("capture child starts");
    let stdout = String::from_utf8(output.stdout).expect("test output is UTF-8");
    assert!(
        output.status.success(),
        "isolated capture test failed: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("1 passed; 0 failed"),
        "capture test was not selected: {stdout}"
    );
    false
}

fn capture<R>(subject: impl FnOnce() -> R) -> (R, Arc<Mutex<Vec<Observation>>>) {
    let install_guard = CAPTURE_INSTALL_LOCK.lock().expect("capture install lock");
    let records = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Capture(Arc::clone(&records)));
    let result = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        subject()
    });
    drop(install_guard);
    (result, records)
}

impl<S: Subscriber> Layer<S> for Capture {
    fn on_new_span(
        &self,
        attributes: &Attributes<'_>,
        _id: &tracing::span::Id,
        _context: Context<'_, S>,
    ) {
        let mut fields = Fields::default();
        attributes.record(&mut fields);
        self.0.lock().expect("capture lock").push(Observation {
            target: attributes.metadata().target().to_owned(),
            level: attributes.metadata().level().as_str().to_owned(),
            name: Some(attributes.metadata().name().to_owned()),
            fields: fields.0,
        });
    }

    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        self.0.lock().expect("capture lock").push(Observation {
            target: event.metadata().target().to_owned(),
            level: event.metadata().level().as_str().to_owned(),
            name: None,
            fields: fields.0,
        });
    }
}

#[derive(Default)]
struct Fields(Map<String, Value>);

impl Visit for Fields {
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), json!(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_owned(), json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), json!(value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_owned(), json!(format!("{value:?}")));
    }
}

#[test]
fn batch_success_has_stage_spans_and_one_count_event_without_changing_bytes() {
    if !run_in_isolated_capture_process(
        "batch_success_has_stage_spans_and_one_count_event_without_changing_bytes",
    ) {
        return;
    }
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = directory.path().join("first.json");
    let second = directory.path().join("second.json");
    let assets = [
        PublicationAsset {
            role: "first",
            path: &first,
            bytes: b"one",
        },
        PublicationAsset {
            role: "second",
            path: &second,
            bytes: b"two",
        },
    ];
    let plain = publish_batch(&assets).expect("plain publication");
    assert_eq!(std::fs::read(&first).expect("first bytes"), b"one");
    assert_eq!(std::fs::read(&second).expect("second bytes"), b"two");
    std::fs::remove_file(&first).expect("remove first");
    std::fs::remove_file(&second).expect("remove second");
    let (captured, records) = capture(|| publish_batch(&assets));
    let captured = captured.expect("captured publication");
    assert_eq!(captured, plain);
    assert_eq!(std::fs::read(&first).expect("first bytes"), b"one");
    assert_eq!(std::fs::read(&second).expect("second bytes"), b"two");
    let records = records.lock().expect("capture lock");
    assert_eq!(
        records
            .iter()
            .filter(|record| record.name.as_deref() == Some("publish_batch"))
            .count(),
        1
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.name.as_deref() == Some("stage_asset"))
            .count(),
        2
    );
    let events = records
        .iter()
        .filter(|record| record.name.is_none())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].fields["outcome"], "completed");
    assert_eq!(events[0].fields["bytes_changed_count"], 2);
    drop(events);
    drop(records);
}

#[test]
fn staging_failure_has_a_failed_message_and_same_return() {
    if !run_in_isolated_capture_process("staging_failure_has_a_failed_message_and_same_return") {
        return;
    }
    let directory = tempfile::tempdir().expect("temporary directory");
    let blocker = directory.path().join("blocker");
    std::fs::write(&blocker, b"file").expect("create blocker");
    let destination = blocker.join("report.json");
    let assets = [PublicationAsset {
        role: "report",
        path: &destination,
        bytes: b"result",
    }];
    let plain = publish_batch(&assets);
    let (captured, records) = capture(|| publish_batch(&assets));
    assert!(matches!(
        &plain,
        Err(crate::BatchPublicationError::Stage { .. })
    ));
    assert_eq!(format!("{plain:?}"), format!("{captured:?}"));
    let records = records.lock().expect("capture lock");
    let events = records
        .iter()
        .filter(|record| record.name.is_none())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].target, "cli_common::publication");
    assert_eq!(events[0].level, "INFO");
    assert_eq!(events[0].fields["message"], "asset batch failed");
    assert_eq!(events[0].fields["outcome"], "failed");
    assert_eq!(events[0].fields["phase"], "stage");
    drop(events);
    drop(records);
}

#[test]
fn aliased_batch_is_debug_refusal_without_warning_or_error() {
    if !run_in_isolated_capture_process("aliased_batch_is_debug_refusal_without_warning_or_error") {
        return;
    }
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = directory.path().join("first.json");
    let second = directory.path().join("second.json");
    let control = [
        PublicationAsset {
            role: "first",
            path: &first,
            bytes: b"one",
        },
        PublicationAsset {
            role: "second",
            path: &second,
            bytes: b"two",
        },
    ];
    let subject = [
        PublicationAsset {
            role: "first",
            path: &first,
            bytes: b"one",
        },
        PublicationAsset {
            role: "second",
            path: &first,
            bytes: b"two",
        },
    ];
    let plain_refusal = publish_batch(&subject);
    let ((accepted, refused), records) =
        capture(|| (publish_batch(&control), publish_batch(&subject)));
    assert!(accepted.is_ok());
    assert!(matches!(
        &plain_refusal,
        Err(crate::BatchPublicationError::AliasedOutputs(_))
    ));
    assert!(matches!(
        &refused,
        Err(crate::BatchPublicationError::AliasedOutputs(_))
    ));
    assert_eq!(format!("{plain_refusal:?}"), format!("{refused:?}"));
    let records = records.lock().expect("capture lock");
    let events = records
        .iter()
        .filter(|record| record.name.is_none())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].fields["outcome"], "completed");
    assert_eq!(events[1].fields["outcome"], "refused");
    assert_eq!(events[1].fields["refusal_kind"], "aliased_outputs");
    assert_eq!(events[1].level, "DEBUG");
    assert!(
        !records
            .iter()
            .any(|record| record.level == "WARN" || record.level == "ERROR")
    );
    drop(events);
    drop(records);
}

#[test]
fn runner_success_has_one_terminal_event_and_same_return() {
    if !run_in_isolated_capture_process("runner_success_has_one_terminal_event_and_same_return") {
        return;
    }
    let plain = run_no_stdout_command("capture-test", false, tracing::Level::INFO, || {
        Ok::<(), &'static str>(())
    });
    let (captured, records) = capture(|| {
        run_no_stdout_command("capture-test", false, tracing::Level::INFO, || {
            Ok::<(), &'static str>(())
        })
    });
    assert_eq!(plain, CommandExit::Success.exit_code());
    assert_eq!(captured, plain);
    let records = records.lock().expect("capture lock");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].name.as_deref(), Some("command"));
    assert_eq!(records[0].fields["output_mode"], "none");
    assert_eq!(records[1].target, "cli_common::command");
    assert_eq!(records[1].level, "INFO");
    assert_eq!(records[1].fields["message"], "command completed");
    assert_eq!(records[1].fields["result_published"], false);
    drop(records);
}

#[test]
fn runner_operational_failure_has_one_terminal_error_and_same_return() {
    if !run_in_isolated_capture_process(
        "runner_operational_failure_has_one_terminal_error_and_same_return",
    ) {
        return;
    }
    let plain = run_no_stdout_command("capture-test", false, tracing::Level::INFO, || {
        Err::<(), _>("failure")
    });
    let (captured, records) = capture(|| {
        run_no_stdout_command("capture-test", false, tracing::Level::INFO, || {
            Err::<(), _>("failure")
        })
    });
    assert_eq!(plain, CommandExit::Failure.exit_code());
    assert_eq!(captured, plain);
    let records = records.lock().expect("capture lock");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].name.as_deref(), Some("command"));
    assert_eq!(records[1].target, "cli_common::command");
    assert_eq!(records[1].level, "ERROR");
    assert_eq!(records[1].fields["message"], "command failed");
    assert_eq!(records[1].fields["phase"], "execute");
    drop(records);
}

#[test]
fn cached_callsite_is_captured_once_on_each_test_thread() {
    if !run_in_isolated_capture_process("cached_callsite_is_captured_once_on_each_test_thread") {
        return;
    }
    let plain = run_no_stdout_command("cache-test", false, tracing::Level::INFO, || {
        Ok::<(), &'static str>(())
    });
    assert_eq!(plain, CommandExit::Success.exit_code());

    let run_thread = || {
        std::thread::spawn(|| {
            let (exit, records) = capture(|| {
                run_no_stdout_command("cache-test", false, tracing::Level::INFO, || {
                    Ok::<(), &'static str>(())
                })
            });
            assert_eq!(exit, CommandExit::Success.exit_code());
            let records = records.lock().expect("capture lock");
            let count = records
                .iter()
                .filter(|record| record.name.as_deref() == Some("command"))
                .count();
            drop(records);
            count
        })
    };
    let first = run_thread();
    let second = run_thread();
    assert_eq!(first.join().expect("first capture thread"), 1);
    assert_eq!(second.join().expect("second capture thread"), 1);
}
