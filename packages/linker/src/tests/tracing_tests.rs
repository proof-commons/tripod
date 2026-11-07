//! Scoped tracing assertions for the three candidate entry points.

use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::sync::{Arc, Mutex};

use architecture::{ARCHITECTURE, AssetId};
use tapscript::upstream::{LiveTransferRepresentationPlan, StateSingletonDeclaration};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber, span};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};

use super::{
    ScriptedCurve, bridge, declaration, live_deployment, record, relocatable_bundle,
    reviewed_target, single_live_bundle, singleton, state_constructor, state_metadata,
};
use crate::tests::deployment;
use crate::{
    LinkRefusal, SelfCommitmentStrategy, StateLinkRefusal, StateLinkSources, link_candidate,
    link_live_candidate, link_state_candidate,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Text(String),
    Count(u64),
}

#[derive(Clone, Debug)]
enum TraceRecord {
    Span {
        id: u64,
        parent: Option<u64>,
        name: &'static str,
        target: &'static str,
        fields: BTreeMap<String, Value>,
    },
    Event {
        parent: Option<u64>,
        target: &'static str,
        level: Level,
        fields: BTreeMap<String, Value>,
    },
    Close(u64),
}

#[derive(Default)]
struct Fields(BTreeMap<String, Value>);

impl Visit for Fields {
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), Value::Count(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0
            .insert(field.name().to_owned(), Value::Text(value.to_owned()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0
            .insert(field.name().to_owned(), Value::Text(format!("{value:?}")));
    }
}

struct Capture(Arc<Mutex<Vec<TraceRecord>>>);

static CAPTURE_INSTALL_LOCK: Mutex<()> = Mutex::new(());

impl<S> Layer<S> for Capture
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let parent = attrs
            .parent()
            .map(|id| id.clone().into_u64())
            .or_else(|| ctx.current_span().id().map(|id| id.clone().into_u64()));
        self.0
            .lock()
            .expect("capture lock")
            .push(TraceRecord::Span {
                id: id.clone().into_u64(),
                parent,
                name: attrs.metadata().name(),
                target: attrs.metadata().target(),
                fields: fields.0,
            });
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        self.0
            .lock()
            .expect("capture lock")
            .push(TraceRecord::Event {
                parent: ctx.current_span().id().map(|id| id.clone().into_u64()),
                target: event.metadata().target(),
                level: *event.metadata().level(),
                fields: fields.0,
            });
    }

    fn on_close(&self, id: span::Id, _ctx: Context<'_, S>) {
        self.0
            .lock()
            .expect("capture lock")
            .push(TraceRecord::Close(id.into_u64()));
    }
}

fn capture<T>(f: impl FnOnce() -> T) -> (T, Vec<TraceRecord>) {
    let _install = CAPTURE_INSTALL_LOCK.lock().expect("capture install lock");
    let records = Arc::new(Mutex::new(Vec::new()));
    let subscriber = Registry::default().with(Capture(Arc::clone(&records)));
    let result = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        f()
    });
    let snapshot = records.lock().expect("capture lock").clone();
    (result, snapshot)
}

fn record_outline(records: &[TraceRecord]) -> String {
    let mut outline = String::new();
    for record in records {
        match record {
            TraceRecord::Span {
                id,
                parent,
                name,
                target,
                ..
            } => writeln!(
                &mut outline,
                "span name={name} target={target} id={id} parent={parent:?}"
            ),
            TraceRecord::Event {
                parent,
                target,
                level,
                ..
            } => writeln!(
                &mut outline,
                "event target={target} level={level:?} id=none parent={parent:?}"
            ),
            TraceRecord::Close(id) => writeln!(&mut outline, "close id={id}"),
        }
        .expect("writing to String");
    }
    outline
}

fn in_isolated_capture_test(test_name: &str) -> bool {
    if std::env::var("TRIPOD_LINKER_CAPTURE_CHILD").as_deref() == Ok(test_name) {
        return true;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg(test_name)
        .arg("--test-threads=1")
        .env("TRIPOD_LINKER_CAPTURE_CHILD", test_name)
        .output()
        .expect("run isolated capture test");
    assert!(
        output.status.success(),
        "isolated capture test {test_name} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.starts_with("test result: ok. 1 passed; 0 failed;")),
        "isolated capture test {test_name} did not run exactly one test:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    false
}

fn kind_table_names(source: &str) -> Vec<&str> {
    let table = source
        .split_once("pub(crate) const fn kind(&self) -> &'static str {")
        .expect("refusal kind table")
        .1
        .split_once("\n    }\n}")
        .expect("end of refusal kind table")
        .0;
    let arm_count = table.lines().filter(|line| line.contains("=>")).count();
    let names: Vec<_> = table
        .lines()
        .filter_map(|line| {
            line.split_once('"')?
                .1
                .split_once('"')
                .map(|(name, _)| name)
        })
        .collect();
    let delegated_count = usize::from(table.contains("=> refusal.kind()"));
    assert_eq!(names.len() + delegated_count, arm_count);
    names
}

#[test]
fn refusal_kind_tables_are_snake_case_with_single_letter_words() {
    let link_names = kind_table_names(include_str!("../error.rs"));
    let state_names = kind_table_names(include_str!("../state_error.rs"));
    assert!(link_names.contains(&"bundle_is_not_a_candidate"));
    assert!(state_names.contains(&"site_is_not_a_push"));
    for name in link_names.iter().chain(state_names.iter()) {
        assert!(
            name.bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase())
                && name.split('_').all(|word| {
                    !word.is_empty()
                        && word
                            .bytes()
                            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                }),
            "invalid refusal kind: {name}"
        );
    }
}

fn assert_outer(
    records: &[TraceRecord],
    target: &str,
    name: &str,
    family: &str,
    outcome: &str,
    level: Level,
    refusal_kind: Option<&str>,
) {
    let spans: Vec<_> = records
        .iter()
        .filter_map(|record| match record {
            TraceRecord::Span {
                id,
                parent,
                name: span_name,
                target: span_target,
                fields,
            } if *span_name == name && *span_target == target => {
                assert_eq!(*parent, None);
                assert_eq!(fields.get("family"), Some(&Value::Text(family.to_owned())));
                Some(*id)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        spans.len(),
        1,
        "captured records:\n{}",
        record_outline(records)
    );
    let event_count = records
        .iter()
        .filter_map(|record| match record {
            TraceRecord::Event {
                parent,
                target: event_target,
                level: event_level,
                fields,
            } if *event_target == target => {
                assert_eq!(*parent, Some(spans[0]));
                assert_eq!(*event_level, level);
                assert_eq!(fields.get("family"), Some(&Value::Text(family.to_owned())));
                assert_eq!(
                    fields.get("outcome"),
                    Some(&Value::Text(outcome.to_owned()))
                );
                assert_eq!(
                    fields.get("refusal_kind"),
                    refusal_kind
                        .map(|kind| Value::Text(kind.to_owned()))
                        .as_ref()
                );
                if outcome == "completed" {
                    assert!(matches!(fields.get("leaf_count"), Some(Value::Count(_))));
                    assert!(matches!(
                        fields.get("outstanding_obligation_count"),
                        Some(Value::Count(_))
                    ));
                }
                Some(fields)
            }
            _ => None,
        })
        .count();
    assert_eq!(
        event_count,
        1,
        "captured records:\n{}",
        record_outline(records)
    );
    assert!(
        records
            .iter()
            .any(|record| matches!(record, TraceRecord::Close(id) if *id == spans[0]))
    );
    assert!(!records.iter().any(|record| matches!(
        record,
        TraceRecord::Event {
            level: Level::WARN | Level::ERROR,
            ..
        }
    )));
}

fn assert_child(records: &[TraceRecord], parent_name: &str, name: &str, target: &str) {
    let parent = records
        .iter()
        .find_map(|record| match record {
            TraceRecord::Span { id, name, .. } if *name == parent_name => Some(*id),
            _ => None,
        })
        .expect("parent span");
    let children: Vec<_> = records
        .iter()
        .filter_map(|record| match record {
            TraceRecord::Span {
                id,
                parent: Some(child_parent),
                name: child_name,
                target: child_target,
                fields,
            } if *child_parent == parent && *child_name == name && *child_target == target => {
                assert!(matches!(fields.get("family"), Some(Value::Text(_))));
                Some(*id)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        children.len(),
        1,
        "expected {name} at {target} beneath {parent_name}; captured records:\n{}",
        record_outline(records)
    );
    assert!(
        records
            .iter()
            .any(|record| matches!(record, TraceRecord::Close(id) if *id == children[0]))
    );
}

#[test]
fn scoped_capture_survives_uncaptured_call_and_two_threads() {
    if !in_isolated_capture_test(
        "tests::tracing_tests::scoped_capture_survives_uncaptured_call_and_two_threads",
    ) {
        return;
    }
    let target = reviewed_target();
    let bundle = relocatable_bundle();
    let accepted = deployment(&target, SelfCommitmentStrategy::IdentityIntrospection);
    assert!(link_candidate(&target, &bundle, &accepted).is_ok());

    let captures: Vec<_> = (0..2)
        .map(|_| {
            std::thread::spawn(|| {
                let target = reviewed_target();
                let bundle = relocatable_bundle();
                let accepted = deployment(&target, SelfCommitmentStrategy::IdentityIntrospection);
                let (linked, records) = capture(|| link_candidate(&target, &bundle, &accepted));
                assert!(linked.is_ok());
                records
            })
        })
        .collect();
    for thread in captures {
        let records = thread.join().expect("capture thread");
        assert_outer(
            &records,
            "linker::bundle",
            "link_candidate",
            "compact",
            "completed",
            Level::INFO,
            None,
        );
    }
}

#[test]
fn compact_entry_records_control_and_one_strategy_refusal() {
    if !in_isolated_capture_test(
        "tests::tracing_tests::compact_entry_records_control_and_one_strategy_refusal",
    ) {
        return;
    }
    let target = reviewed_target();
    let bundle = relocatable_bundle();
    let accepted = deployment(&target, SelfCommitmentStrategy::IdentityIntrospection);
    let refused = deployment(&target, SelfCommitmentStrategy::NotStated);
    let plain = link_candidate(&target, &bundle, &accepted);
    let (observed, records) = capture(|| link_candidate(&target, &bundle, &accepted));
    assert_eq!(observed, plain);
    assert_outer(
        &records,
        "linker::bundle",
        "link_candidate",
        "compact",
        "completed",
        Level::INFO,
        None,
    );
    for (name, target) in [
        ("resolve_graph", "linker::graph"),
        ("substitute_relocations", "linker::relocate"),
        ("assemble_taptree", "linker::taptree"),
        ("close_carriers", "linker::carrier"),
    ] {
        assert_child(&records, "link_candidate", name, target);
    }
    let plain = link_candidate(&target, &bundle, &refused);
    let (observed, records) = capture(|| link_candidate(&target, &bundle, &refused));
    assert_eq!(observed, plain);
    assert!(matches!(
        observed,
        Err(LinkRefusal::ImpossibleStaticFixedPoint { .. })
    ));
    assert_outer(
        &records,
        "linker::bundle",
        "link_candidate",
        "compact",
        "refused",
        Level::DEBUG,
        Some("impossible_static_fixed_point"),
    );
}

#[test]
fn live_entry_records_control_and_one_offering_refusal() {
    if !in_isolated_capture_test(
        "tests::tracing_tests::live_entry_records_control_and_one_offering_refusal",
    ) {
        return;
    }
    let target = reviewed_target();
    let deployment = live_deployment(&target);
    let bundles = [single_live_bundle(
        LiveTransferRepresentationPlan::Explicit,
        &[0x11; 32],
    )];
    let plain = link_live_candidate(&target, &bundles, &deployment);
    let (observed, records) = capture(|| link_live_candidate(&target, &bundles, &deployment));
    assert_eq!(observed, plain);
    assert_outer(
        &records,
        "linker::live_bundle",
        "link_live_candidate",
        "live",
        "completed",
        Level::INFO,
        None,
    );
    assert_child(
        &records,
        "link_live_candidate",
        "link_live_bundle",
        "linker::live_bundle",
    );
    for (name, target) in [
        ("substitute_relocations", "linker::live_relocate"),
        ("assemble_taptree", "linker::live_taptree"),
    ] {
        assert_child(&records, "link_live_bundle", name, target);
    }
    assert_child(
        &records,
        "link_live_candidate",
        "close_carriers",
        "linker::live_carrier",
    );
    let plain = link_live_candidate(&target, &[], &deployment);
    let (observed, records) = capture(|| link_live_candidate(&target, &[], &deployment));
    assert_eq!(observed, plain);
    assert_eq!(observed, Err(LinkRefusal::NoLiveBundleOffered));
    assert_outer(
        &records,
        "linker::live_bundle",
        "link_live_candidate",
        "live",
        "refused",
        Level::DEBUG,
        Some("no_live_bundle_offered"),
    );
}

#[test]
fn state_entry_records_control_and_one_declaration_refusal() {
    if !in_isolated_capture_test(
        "tests::tracing_tests::state_entry_records_control_and_one_declaration_refusal",
    ) {
        return;
    }
    let target = reviewed_target();
    let record = record();
    let bridge = bridge();
    let constructor = state_constructor();
    let singleton = singleton();
    let declaration = declaration();
    let metadata = state_metadata();
    let curve = ScriptedCurve;
    let sources = StateLinkSources::new(
        &record,
        &bridge,
        &constructor,
        &singleton,
        &declaration,
        &metadata,
        &curve,
    );
    let plain = link_state_candidate(&target, &sources);
    let (observed, records) = capture(|| link_state_candidate(&target, &sources));
    assert_eq!(observed, plain);
    assert_outer(
        &records,
        "linker::state_bundle",
        "link_state_candidate",
        "state",
        "completed",
        Level::INFO,
        None,
    );
    for (name, target) in [
        ("substitute_relocations", "linker::state_relocate"),
        ("resolve_graph", "linker::state_graph"),
        ("assemble_taptree", "linker::state_taptree"),
        ("measure_state_resources", "linker::state_resource"),
        ("close_carriers", "linker::state_carrier"),
    ] {
        assert_child(&records, "link_state_candidate", name, target);
    }
    let other = ARCHITECTURE.asset(AssetId::Pace).expect("pace asset");
    let mismatched =
        StateSingletonDeclaration::from_architecture_asset(other).expect("other declaration");
    let sources = StateLinkSources::new(
        &record,
        &bridge,
        &constructor,
        &singleton,
        &mismatched,
        &metadata,
        &curve,
    );
    let plain = link_state_candidate(&target, &sources);
    let (observed, records) = capture(|| link_state_candidate(&target, &sources));
    assert_eq!(observed, plain);
    assert_eq!(
        observed,
        Err(LinkRefusal::StateLink(
            StateLinkRefusal::SingletonDeclarationMismatch
        ))
    );
    assert_outer(
        &records,
        "linker::state_bundle",
        "link_state_candidate",
        "state",
        "refused",
        Level::DEBUG,
        Some("singleton_declaration_mismatch"),
    );
}
