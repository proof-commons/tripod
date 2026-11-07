//! Scoped in-memory capture for library instrumentation tests.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use tracing::field::{Field, Visit};
use tracing::{Event, Id, Subscriber};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};

static INSTALL_LOCK: Mutex<()> = Mutex::new(());
static INTEREST_ANCHOR: OnceLock<tracing::Dispatch> = OnceLock::new();
const CHILD_MARKER: &str = "TRIPOD_TAPSCRIPT_CAPTURE_CHILD";

pub(super) fn run_isolated(test_name: &str, work: impl FnOnce()) {
    if std::env::var_os(CHILD_MARKER).is_some() {
        work();
        return;
    }

    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name])
        .env(CHILD_MARKER, "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stdout.contains("test result: ok. 1 passed; 0 failed;"),
        "isolated test {test_name} failed or selected no test: status={}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Record {
    pub kind: &'static str,
    pub target: &'static str,
    pub level: tracing::Level,
    pub name: &'static str,
    pub parent: Option<&'static str>,
    pub fields: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
struct CaptureLayer(Arc<Mutex<Vec<Record>>>);

#[derive(Default)]
struct Fields(BTreeMap<String, String>);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().into(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().into(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().into(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().into(), value.to_string());
    }
}

impl<S> Layer<S> for CaptureLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attributes: &tracing::span::Attributes<'_>, _: &Id, _: Context<'_, S>) {
        let mut fields = Fields::default();
        attributes.record(&mut fields);
        let meta = attributes.metadata();
        self.0.lock().unwrap().push(Record {
            kind: "span",
            target: meta.target(),
            level: *meta.level(),
            name: meta.name(),
            parent: None,
            fields: fields.0,
        });
    }

    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let meta = event.metadata();
        self.0.lock().unwrap().push(Record {
            kind: "event",
            target: meta.target(),
            level: *meta.level(),
            name: meta.name(),
            parent: context.lookup_current().map(|span| span.metadata().name()),
            fields: fields.0,
        });
    }

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        if let Some(span) = context.span(&id) {
            let meta = span.metadata();
            self.0.lock().unwrap().push(Record {
                kind: "close",
                target: meta.target(),
                level: *meta.level(),
                name: meta.name(),
                parent: None,
                fields: BTreeMap::new(),
            });
        }
    }
}

pub(super) fn capture<R>(work: impl FnOnce() -> R) -> (R, Vec<Record>) {
    let _install = INSTALL_LOCK.lock().unwrap();
    let _anchor = INTEREST_ANCHOR
        .get_or_init(|| tracing::Dispatch::new(Registry::default().with(CaptureLayer::default())));
    let layer = CaptureLayer::default();
    let records = Arc::clone(&layer.0);
    let result = tracing::subscriber::with_default(Registry::default().with(layer), || {
        tracing::callsite::rebuild_interest_cache();
        work()
    });
    let snapshot = records.lock().unwrap().clone();
    (result, snapshot)
}

fn cached_probe() {
    let span = tracing::debug_span!(target: "tapscript::test", "cached_probe");
    let _entered = span.enter();
}

#[test]
fn scoped_capture_rebuilds_interest_after_uncaptured_use_on_two_threads() {
    run_isolated(
        "tests::tracing_capture::scoped_capture_rebuilds_interest_after_uncaptured_use_on_two_threads",
        || {
            {
                let _install = INSTALL_LOCK.lock().unwrap();
                cached_probe();
            }
            let threads: Vec<_> = (0..2)
                .map(|_| {
                    std::thread::spawn(|| {
                        let ((), records) = capture(cached_probe);
                        records
                            .iter()
                            .filter(|record| record.kind == "span" && record.name == "cached_probe")
                            .count()
                    })
                })
                .collect();
            for thread in threads {
                assert_eq!(thread.join().unwrap(), 1);
            }
        },
    );
}

#[test]
fn scoped_capture_records_an_event_and_close() {
    run_isolated(
        "tests::tracing_capture::scoped_capture_records_an_event_and_close",
        || {
            let ((), records) = capture(|| {
                let span = tracing::debug_span!(target: "tapscript::test", "probe", count = 1_u64);
                let _entered = span.enter();
                tracing::debug!(target: "tapscript::test", outcome = "completed", "probe completed");
            });
            assert_eq!(records.len(), 3);
            assert_eq!(records[0].kind, "span");
            assert_eq!(records[1].kind, "event");
            assert_eq!(records[1].parent, Some("probe"));
            assert_eq!(records[2].kind, "close");
        },
    );
}

#[test]
fn refusal_name_omits_payload() {
    let error = crate::TapscriptError::InstructionLimitExceeded { maximum: 7 };
    assert_eq!(error.refusal_kind(), "instruction_limit_exceeded");
}

#[test]
fn all_tracing_refusal_names_are_snake_case_with_single_letter_words() {
    let tables = [
        ("TapscriptError", include_str!("../error.rs"), 21),
        ("BundleRefusal", include_str!("../bundle.rs"), 19),
        ("LiveBundleRefusal", include_str!("../live_bundle.rs"), 11),
        (
            "LiveConstructorRefusal",
            include_str!("../live_constructor.rs"),
            12,
        ),
        (
            "StateConstructorRefusal",
            include_str!("../state_constructor.rs"),
            20,
        ),
    ];
    let mut all_names = Vec::new();
    for (table, source, expected_count) in tables {
        let body = source
            .split_once("const fn refusal_kind(&self) -> &'static str {")
            .unwrap()
            .1
            .split_once("\n}\n")
            .unwrap()
            .0;
        let names: Vec<_> = body.split('"').skip(1).step_by(2).collect();
        assert_eq!(names.len(), expected_count, "{table} tracing names");
        for name in &names {
            let mut bytes = name.bytes();
            let valid = matches!(bytes.next(), Some(b'a'..=b'z'))
                && bytes
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
                && !name.contains("__")
                && !name.ends_with('_');
            assert!(valid, "{table} returned {name:?} outside snake_case");
        }
        all_names.extend(names);
    }
    assert!(all_names.contains(&"internal_key_not_a_point"));
    assert!(all_names.contains(&"relocation_site_is_not_a_push"));
}
