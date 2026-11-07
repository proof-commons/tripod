use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use architecture::{ARCHITECTURE, ManifestError, OperationId};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id},
};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

use crate::{RealizationError, RealizationScope, derive, project_scoped_realization};

#[test]
fn every_realization_trace_kind_is_snake_case() {
    let source = include_str!("../error.rs");
    let table = source
        .split_once("pub(crate) fn trace_kind(")
        .unwrap()
        .1
        .split_once("pub(crate) fn trace_failed(")
        .unwrap()
        .0;
    let names = table.split('"').skip(1).step_by(2).collect::<Vec<_>>();
    assert!(names.len() > 80, "trace kind table unexpectedly shrank");
    for name in &names {
        let bytes = name.as_bytes();
        assert!(
            bytes.first().is_some_and(u8::is_ascii_lowercase)
                && bytes.iter().all(|byte| byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || *byte == b'_')
                && !name.contains("__")
                && !name.ends_with('_'),
            "trace kind is not snake case: {name}"
        );
    }
    assert!(names.contains(&"cycle_bound_used_as_count"));
    assert!(names.contains(&"architecture_validation_failed"));

    let defect = ManifestError::DraftPublication;
    let error = RealizationError::ArchitectureValidationFailed {
        errors: vec![defect.clone()],
    };
    assert_eq!(error.trace_kind(), defect.name());
    let empty = RealizationError::ArchitectureValidationFailed { errors: vec![] };
    assert_eq!(empty.trace_kind(), "architecture_validation_failed");
}

#[derive(Clone, Debug)]
pub(super) struct Record {
    pub target: &'static str,
    pub name: &'static str,
    pub level: tracing::Level,
    pub fields: BTreeMap<String, String>,
    pub types: BTreeMap<String, &'static str>,
}

#[derive(Default)]
struct Fields {
    values: BTreeMap<String, String>,
    types: BTreeMap<String, &'static str>,
}

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.values
            .insert(field.name().to_owned(), format!("{value:?}"));
        self.types.insert(field.name().to_owned(), "debug");
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.values
            .insert(field.name().to_owned(), value.to_owned());
        self.types.insert(field.name().to_owned(), "str");
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.values
            .insert(field.name().to_owned(), value.to_string());
        self.types.insert(field.name().to_owned(), "u64");
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.values
            .insert(field.name().to_owned(), value.to_string());
        self.types.insert(field.name().to_owned(), "bool");
    }
}

struct Capture(Arc<Mutex<Vec<Record>>>);

static CAPTURE_INSTALL_LOCK: Mutex<()> = Mutex::new(());
const CAPTURE_CHILD_ENV: &str = "TRIPOD_REALIZATION_CAPTURE_CHILD";

impl<S: Subscriber> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let metadata = attrs.metadata();
        self.0.lock().unwrap().push(Record {
            target: metadata.target(),
            name: metadata.name(),
            level: *metadata.level(),
            fields: fields.values,
            types: fields.types,
        });
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let metadata = event.metadata();
        self.0.lock().unwrap().push(Record {
            target: metadata.target(),
            name: metadata.name(),
            level: *metadata.level(),
            fields: fields.values,
            types: fields.types,
        });
    }
}

pub(super) fn capture<R>(f: impl FnOnce() -> R) -> (R, Vec<Record>) {
    let _install_guard = CAPTURE_INSTALL_LOCK.lock().unwrap();
    let records = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Capture(Arc::clone(&records)));
    let result = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        f()
    });
    let records = records.lock().unwrap().clone();
    (result, records)
}

pub(super) fn in_capture_child(test_name: &str) -> bool {
    if std::env::var(CAPTURE_CHILD_ENV).is_ok_and(|value| value == test_name) {
        return true;
    }

    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name])
        .env(CAPTURE_CHILD_ENV, test_name)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stdout
                .lines()
                .any(|line| line.starts_with("test result: ok. 1 passed; 0 failed;")),
        "isolated capture test {test_name} failed or selected no test\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    false
}

#[test]
fn capture_rebuilds_interest_after_uncaptured_call_on_each_thread() {
    if !in_capture_child(
        "tests::tracing_tests::capture_rebuilds_interest_after_uncaptured_call_on_each_thread",
    ) {
        return;
    }
    let scope = RealizationScope::from_operations([OperationId::CompactAsh]).unwrap();
    derive(&ARCHITECTURE, scope).unwrap();

    let handles = (0..2)
        .map(|_| {
            std::thread::spawn(|| {
                let scope = RealizationScope::from_operations([OperationId::CompactAsh]).unwrap();
                let (_, records) = capture(|| derive(&ARCHITECTURE, scope).unwrap());
                records
                    .iter()
                    .filter(|record| {
                        record.target == "realization::derive" && record.name == "derive"
                    })
                    .count()
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        assert_eq!(handle.join().unwrap(), 1);
    }
}

pub(super) fn assert_no_warning_or_error(records: &[Record]) {
    assert!(records.iter().all(
        |record| record.level != tracing::Level::WARN && record.level != tracing::Level::ERROR
    ));
}

#[test]
fn derivation_and_direct_validation_emit_one_outcome_per_public_call() {
    if !in_capture_child(
        "tests::tracing_tests::derivation_and_direct_validation_emit_one_outcome_per_public_call",
    ) {
        return;
    }
    let scope = RealizationScope::from_operations([OperationId::CompactAsh]).unwrap();
    let plain = derive(&ARCHITECTURE, scope.clone()).unwrap();
    let (observed, records) = capture(|| derive(&ARCHITECTURE, scope.clone()));
    assert_eq!(
        project_scoped_realization(&plain),
        project_scoped_realization(&observed.unwrap())
    );
    assert!(records.iter().any(|record| {
        record.target == "realization::derive"
            && record.name == "derive"
            && record.level == tracing::Level::INFO
            && record
                .fields
                .get("scope_operation_count")
                .map(String::as_str)
                == Some("1")
    }));
    assert!(
        records
            .iter()
            .any(|record| record.target == "realization::derive"
                && record.name == "assemble"
                && record.level == tracing::Level::DEBUG)
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.fields.get("message").map(String::as_str)
                == Some("realization derived")
                && record.fields.get("outcome").map(String::as_str) == Some("completed"))
            .count(),
        1
    );
    let derive_span = records
        .iter()
        .find(|record| record.name == "derive")
        .unwrap();
    assert_eq!(derive_span.types.get("scope_operation_count"), Some(&"u64"));
    let completion = records
        .iter()
        .find(|record| record.fields.contains_key("expression_count"))
        .unwrap();
    assert_eq!(completion.types.get("operation_count"), Some(&"u64"));
    assert_eq!(completion.types.get("outcome"), Some(&"str"));
    assert_no_warning_or_error(&records);

    let mut altered = ARCHITECTURE;
    altered.operations = &[];
    let plain_refusal = derive(&altered, scope.clone());
    let (captured_refusal, records) = capture(|| derive(&altered, scope));
    assert_eq!(plain_refusal.unwrap_err(), captured_refusal.unwrap_err());
    assert!(
        records
            .iter()
            .any(|record| record.target == "realization::derive"
                && record.level == tracing::Level::DEBUG
                && record.fields.get("outcome").map(String::as_str) == Some("refused")
                && record.fields.contains_key("refusal_kind")
                && record.fields.contains_key("phase"))
    );
    assert_no_warning_or_error(&records);

    let plain_validation = plain.validate_against(&ARCHITECTURE);
    let (captured_validation, records) = capture(|| plain.validate_against(&ARCHITECTURE));
    assert_eq!(plain_validation, captured_validation);
    assert!(
        records
            .iter()
            .any(|record| record.target == "realization::validate"
                && record.name == "validate_spec"
                && record.level == tracing::Level::DEBUG)
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.fields.get("message").map(String::as_str)
                == Some("realization validated")
                && record.fields.get("outcome").map(String::as_str) == Some("completed"))
            .count(),
        1
    );
    let plain_validation_refusal = plain.validate_against(&altered);
    let (captured_validation_refusal, records) = capture(|| plain.validate_against(&altered));
    assert_eq!(plain_validation_refusal, captured_validation_refusal);
    assert!(
        records
            .iter()
            .any(|record| record.target == "realization::validate"
                && record.level == tracing::Level::DEBUG
                && record.fields.get("outcome").map(String::as_str) == Some("refused"))
    );
    assert_no_warning_or_error(&records);
}
