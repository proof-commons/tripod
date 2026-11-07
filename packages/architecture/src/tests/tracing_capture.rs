//! Scoped capture of this package's tracing calls.

use std::collections::BTreeMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::{Event, Id, Level, Subscriber, span::Attributes};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Value {
    Str(String),
    U64(u64),
    Bool(bool),
    Debug(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RecordKind {
    Span,
    Event,
    SpanClose,
}

#[derive(Clone, Debug)]
pub(super) struct Record {
    pub kind: RecordKind,
    pub name: String,
    pub target: String,
    pub level: Level,
    pub fields: BTreeMap<String, Value>,
}

#[derive(Default)]
struct Fields(BTreeMap<String, Value>);

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0
            .insert(field.name().to_owned(), Value::Str(value.to_owned()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), Value::U64(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), Value::Bool(value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        let value = if field.name() == "message" {
            Value::Str(rendered)
        } else {
            Value::Debug(rendered)
        };
        self.0.insert(field.name().to_owned(), value);
    }
}

struct Capture(Arc<Mutex<Vec<Record>>>);

// Scoped subscribers still share callsite-interest caching across test threads.
static CAPTURE_LOCK: Mutex<()> = Mutex::new(());

const CHILD_ENV: &str = "TRIPOD_ARCHITECTURE_CAPTURE_CHILD";

pub(super) fn run_isolated(full_name: &str) -> bool {
    if std::env::var(CHILD_ENV).ok().as_deref() == Some(full_name) {
        return false;
    }

    let executable = std::env::current_exe().expect("test executable is available");
    let output = Command::new(executable)
        .args(["--exact", full_name])
        .env(CHILD_ENV, full_name)
        .output()
        .expect("isolated capture test can start");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stdout.contains("test result: ok. 1 passed; 0 failed; 0 ignored;"),
        "isolated capture test {full_name} failed or selected no test\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    true
}

impl<S> Layer<S> for Capture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let metadata = attrs.metadata();
        self.0.lock().unwrap().push(Record {
            kind: RecordKind::Span,
            name: metadata.name().to_owned(),
            target: metadata.target().to_owned(),
            level: *metadata.level(),
            fields: fields.0,
        });
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let metadata = event.metadata();
        self.0.lock().unwrap().push(Record {
            kind: RecordKind::Event,
            name: metadata.name().to_owned(),
            target: metadata.target().to_owned(),
            level: *metadata.level(),
            fields: fields.0,
        });
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let metadata = span.metadata();
            self.0.lock().unwrap().push(Record {
                kind: RecordKind::SpanClose,
                name: metadata.name().to_owned(),
                target: metadata.target().to_owned(),
                level: *metadata.level(),
                fields: BTreeMap::new(),
            });
        }
    }
}

pub(super) fn capture<T>(f: impl FnOnce() -> T) -> (T, Vec<Record>) {
    let _guard = CAPTURE_LOCK.lock().unwrap();
    let records = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Capture(Arc::clone(&records)));
    let result = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        f()
    });
    let captured = records.lock().unwrap().clone();
    (result, captured)
}

#[test]
fn capture_rebuilds_interest_after_uncaptured_call_on_two_threads() {
    if run_isolated(
        "tests::tracing_capture::capture_rebuilds_interest_after_uncaptured_call_on_two_threads",
    ) {
        return;
    }
    let architecture = crate::ARCHITECTURE;
    assert!(crate::validate_draft(&architecture).is_ok());

    std::thread::scope(|scope| {
        let handles = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    let (result, records) = capture(|| crate::validate_draft(&architecture));
                    assert!(result.is_ok());
                    assert_eq!(
                        records
                            .iter()
                            .filter(|record| {
                                record.kind == RecordKind::Span && record.name == "validate_draft"
                            })
                            .count(),
                        1
                    );
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}
