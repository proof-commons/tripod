use std::{
    collections::BTreeMap,
    env,
    ffi::OsStr,
    process::Command,
    sync::{Arc, Mutex},
};

use tracing::{
    Event, Level, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id},
};
use tracing_subscriber::{
    Layer,
    layer::{Context, SubscriberExt},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum FieldValue {
    U64(u64),
    I64(i64),
    Bool(bool),
    Str(String),
    Debug(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CapturedRecord {
    pub is_span: bool,
    pub target: &'static str,
    pub name: &'static str,
    pub id: Option<String>,
    pub parent: Option<String>,
    pub level: Level,
    pub fields: BTreeMap<&'static str, FieldValue>,
}

#[derive(Default)]
struct FieldVisitor(BTreeMap<&'static str, FieldValue>);

impl Visit for FieldVisitor {
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name(), FieldValue::U64(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name(), FieldValue::I64(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name(), FieldValue::Bool(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0
            .insert(field.name(), FieldValue::Str(value.to_owned()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name(), FieldValue::Debug(format!("{value:?}")));
    }
}

struct CaptureLayer(Arc<Mutex<Vec<CapturedRecord>>>);

static CAPTURE_LOCK: Mutex<()> = Mutex::new(());

impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = FieldVisitor::default();
        attrs.record(&mut fields);
        let metadata = attrs.metadata();
        let parent = if attrs.is_contextual() {
            ctx.current_span().id().map(|parent| format!("{parent:?}"))
        } else {
            attrs.parent().map(|parent| format!("{parent:?}"))
        };
        self.0.lock().expect("capture lock").push(CapturedRecord {
            is_span: true,
            target: metadata.target(),
            name: metadata.name(),
            id: Some(format!("{id:?}")),
            parent,
            level: *metadata.level(),
            fields: fields.0,
        });
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut fields = FieldVisitor::default();
        event.record(&mut fields);
        let metadata = event.metadata();
        let parent = if event.is_contextual() {
            ctx.current_span().id().map(|parent| format!("{parent:?}"))
        } else {
            event.parent().map(|parent| format!("{parent:?}"))
        };
        self.0.lock().expect("capture lock").push(CapturedRecord {
            is_span: false,
            target: metadata.target(),
            name: metadata.name(),
            id: None,
            parent,
            level: *metadata.level(),
            fields: fields.0,
        });
    }
}

pub(super) fn capture<T>(operation: impl FnOnce() -> T) -> (T, Vec<CapturedRecord>) {
    let _serial = CAPTURE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let records = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(CaptureLayer(Arc::clone(&records)));
    let value = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        operation()
    });
    let records = Arc::try_unwrap(records)
        .expect("subscriber dropped after scoped capture")
        .into_inner()
        .expect("capture lock");
    (value, records)
}

pub(super) fn isolate_capture_test(test_path: &'static str) -> bool {
    const CHILD_MARKER: &str = "TRIPOD_COMPILER_CAPTURE_CHILD";
    if let Some(selected) = env::var_os(CHILD_MARKER) {
        assert_eq!(selected.as_os_str(), OsStr::new(test_path));
        return false;
    }

    let output = Command::new(env::current_exe().expect("current test executable"))
        .args(["--exact", test_path])
        .env(CHILD_MARKER, test_path)
        .output()
        .expect("run isolated capture test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let passed_once = stdout
        .lines()
        .filter(|line| line.starts_with("test result: ok. 1 passed; 0 failed;"))
        .count()
        == 1;
    assert!(
        output.status.success() && passed_once,
        "isolated {test_path} failed or selected the wrong number of tests (status: {}):\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
    true
}

fn emit_capture_probe() {
    let span = tracing::debug_span!(target: "compiler::proof", "capture_probe", state_limit = 3u64);
    let _entered = span.enter();
    tracing::debug!(target: "compiler::proof", outcome = "completed", message = "capture probe completed");
}

#[test]
fn a_previously_unobserved_callsite_is_captured_from_two_threads() {
    if isolate_capture_test(
        "tests::tracing_capture::a_previously_unobserved_callsite_is_captured_from_two_threads",
    ) {
        return;
    }
    emit_capture_probe();
    let captures = std::thread::scope(|scope| {
        let first = scope.spawn(|| capture(emit_capture_probe).1);
        let second = scope.spawn(|| capture(emit_capture_probe).1);
        [first.join().unwrap(), second.join().unwrap()]
    });
    for records in captures {
        assert_eq!(
            records
                .iter()
                .filter(|record| record.is_span && record.name == "capture_probe")
                .count(),
            1,
            "captured records: {records:#?}"
        );
    }
}

#[test]
fn scoped_capture_preserves_field_types_and_return_value() {
    if isolate_capture_test(
        "tests::tracing_capture::scoped_capture_preserves_field_types_and_return_value",
    ) {
        return;
    }
    let (value, records) = capture(|| {
        let span =
            tracing::debug_span!(target: "compiler::proof", "capture_probe", state_limit = 3u64);
        let _entered = span.enter();
        tracing::debug!(
            target: "compiler::proof",
            outcome = "refused",
            visited_state_count = 1u64,
            message = "proof search completed"
        );
        7u8
    });
    assert_eq!(value, 7);
    assert_eq!(records.len(), 2);
    assert!(records[0].is_span);
    assert_eq!(records[0].target, "compiler::proof");
    assert_eq!(records[0].name, "capture_probe");
    assert_eq!(records[0].level, Level::DEBUG);
    assert_eq!(
        records[0].fields.get("state_limit"),
        Some(&FieldValue::U64(3))
    );
    assert!(!records[1].is_span);
    assert_eq!(
        records[1].fields.get("outcome"),
        Some(&FieldValue::Str("refused".into()))
    );
    assert_eq!(
        records[1].fields.get("visited_state_count"),
        Some(&FieldValue::U64(1))
    );
    assert_eq!(
        records[1].fields.get("message"),
        Some(&FieldValue::Str("proof search completed".into()))
    );
}

#[test]
fn binding_control_and_one_operation_refusal_keep_result_bytes() {
    use architecture::{ARCHITECTURE, OperationId};

    use crate::{CompilationScope, CompileError, bind_input};

    if isolate_capture_test(
        "tests::tracing_capture::binding_control_and_one_operation_refusal_keep_result_bytes",
    ) {
        return;
    }

    let realization = super::phase1_realization();
    let control_scope = CompilationScope::from_operations([OperationId::CompactAsh]).unwrap();
    let plain = bind_input(
        &ARCHITECTURE,
        realization.clone(),
        control_scope.clone(),
        super::test_policy(),
    )
    .unwrap();
    let (captured, _) = capture(|| {
        bind_input(
            &ARCHITECTURE,
            realization.clone(),
            control_scope,
            super::test_policy(),
        )
        .unwrap()
    });
    assert_eq!(
        format!("{plain:?}").into_bytes(),
        format!("{captured:?}").into_bytes()
    );

    let refused_scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::AnnounceMaturity])
            .unwrap();
    let plain = bind_input(
        &ARCHITECTURE,
        realization.clone(),
        refused_scope.clone(),
        super::test_policy(),
    )
    .unwrap_err();
    let (captured, _) = capture(|| {
        bind_input(
            &ARCHITECTURE,
            realization,
            refused_scope,
            super::test_policy(),
        )
        .unwrap_err()
    });
    assert_eq!(
        plain,
        CompileError::IncompleteRealizationScope {
            operation: OperationId::AnnounceMaturity
        }
    );
    assert_eq!(plain, captured);
    assert_eq!(
        format!("{plain:?}").into_bytes(),
        format!("{captured:?}").into_bytes()
    );
}
