//! Scoped trace assertions over representative public boundaries.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

use super::{ash_view, linked_bundle, outpoint, pin, reviewed_target, view};
use crate::abi::derive_candidate_abi;
use crate::bytes::TargetTransaction;
use crate::construct::construct;
use crate::error::TransactionRefusal;
use crate::live_abi::derive_live_transfer_abi;
use crate::request::CompactAshRequest;
use crate::taproot::{AshInstanceOrigin, OutputKeyParity, PinnedAshInstance};

#[derive(Clone, Debug)]
pub(super) struct TraceRecord {
    kind: &'static str,
    name: &'static str,
    target: &'static str,
    level: Level,
    fields: BTreeMap<String, String>,
}

#[derive(Default)]
struct Fields(BTreeMap<String, String>);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0.insert(field.name().to_owned(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_owned(), value.to_owned());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), value.to_string());
    }
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<TraceRecord>>>);

static CAPTURE_INSTALL_LOCK: Mutex<()> = Mutex::new(());

pub(super) fn run_isolated_capture_test(module_path: &str, test_name: &str) -> bool {
    const CHILD_MARKER: &str = "TRIPOD_TRANSACTION_CAPTURE_CHILD";
    if std::env::var_os(CHILD_MARKER).is_some() {
        return false;
    }
    let exact = module_path.split_once("::").map_or_else(
        || test_name.to_owned(),
        |(_, module)| format!("{module}::{test_name}"),
    );
    let output = std::process::Command::new(std::env::current_exe().expect("test executable path"))
        .arg("--exact")
        .arg(&exact)
        .env(CHILD_MARKER, "1")
        .output()
        .expect("run isolated capture test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let passed = stdout
        .lines()
        .filter(|line| line.starts_with("test result: ok. 1 passed; 0 failed;"))
        .count();
    assert!(
        output.status.success() && passed == 1,
        "isolated capture test {exact} did not pass exactly once\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    true
}

impl<S> Layer<S> for Capture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        _id: &tracing::span::Id,
        _ctx: Context<'_, S>,
    ) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let meta = attrs.metadata();
        self.0.lock().expect("capture lock").push(TraceRecord {
            kind: "span",
            name: meta.name(),
            target: meta.target(),
            level: *meta.level(),
            fields: fields.0,
        });
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let meta = event.metadata();
        self.0.lock().expect("capture lock").push(TraceRecord {
            kind: "event",
            name: meta.name(),
            target: meta.target(),
            level: *meta.level(),
            fields: fields.0,
        });
    }

    fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let meta = span.metadata();
            self.0.lock().expect("capture lock").push(TraceRecord {
                kind: "close",
                name: meta.name(),
                target: meta.target(),
                level: *meta.level(),
                fields: BTreeMap::new(),
            });
        }
    }
}

pub(super) fn capture<T>(run: impl FnOnce() -> T) -> (T, Vec<TraceRecord>) {
    let _install_guard = CAPTURE_INSTALL_LOCK.lock().expect("capture install lock");
    let layer = Capture::default();
    let subscriber = tracing_subscriber::registry().with(layer.clone());
    let result = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        run()
    });
    let records = layer.0.lock().expect("capture lock").clone();
    (result, records)
}

pub(super) fn trace_kind_table(source: &str) -> BTreeMap<String, String> {
    let mut kinds = BTreeMap::new();
    let mut found_table = false;
    for implementation in source
        .split("impl crate::trace_support::TraceRefusal for ")
        .skip(1)
    {
        let mut variant = None;
        let mut arms = 0;
        let mut values = 0;
        for line in implementation
            .lines()
            .take_while(|line| !line.starts_with('}'))
        {
            if let Some((_, after)) = line.split_once("Self::") {
                let name: String = after
                    .chars()
                    .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == '_')
                    .collect();
                assert!(!name.is_empty(), "trace refusal arm has no variant name");
                variant = Some(name);
                arms += 1;
            }
            if let Some((_, after)) = line.split_once('"') {
                let (kind, _) = after.split_once('"').expect("closed trace kind literal");
                let name = variant.as_ref().expect("trace kind belongs to a variant");
                assert!(
                    kinds.insert(name.clone(), kind.to_owned()).is_none(),
                    "duplicate trace refusal variant {name}"
                );
                assert!(
                    kind.bytes()
                        .next()
                        .is_some_and(|byte| byte.is_ascii_lowercase())
                        && !kind.ends_with('_')
                        && !kind.contains("__")
                        && kind.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                        }),
                    "trace refusal kind {kind} does not match ^[a-z][a-z0-9]*(_[a-z0-9]+)*$"
                );
                let mut canonical = String::new();
                for letter in name.chars() {
                    if letter.is_ascii_uppercase() {
                        if !canonical.is_empty() {
                            canonical.push('_');
                        }
                        canonical.push(letter.to_ascii_lowercase());
                    } else {
                        canonical.push(letter);
                    }
                }
                assert_eq!(
                    kind, canonical,
                    "trace kind for {name} is not variant snake case"
                );
                values += 1;
            }
        }
        if arms > 0 {
            found_table = true;
            assert_eq!(values, arms, "every trace refusal arm has one name");
        }
    }
    assert!(found_table, "source has no trace refusal table");
    kinds
}

#[test]
fn transaction_refusal_trace_kinds_are_snake_case() {
    let kinds = trace_kind_table(include_str!("../error.rs"));
    for (variant, expected) in [
        ("BundleIsNotACandidate", "bundle_is_not_a_candidate"),
        (
            "LiveBundleIsNotACandidate",
            "live_bundle_is_not_a_candidate",
        ),
        (
            "ReceiptInputIsNotALiveReceipt",
            "receipt_input_is_not_a_live_receipt",
        ),
    ] {
        assert_eq!(kinds.get(variant).map(String::as_str), Some(expected));
    }
}

#[test]
fn authorization_and_right_refusal_trace_kinds_are_snake_case() {
    let kinds = trace_kind_table(include_str!("../operator_signing.rs"));
    assert_eq!(
        kinds.get("OperatorKeyIsNotACurvePoint").map(String::as_str),
        Some("operator_key_is_not_a_curve_point")
    );
    trace_kind_table(include_str!("../operator_right.rs"));
    trace_kind_table(include_str!("../live_handoff.rs"));
}

#[test]
fn capture_rebuilds_interest_after_uncaptured_call_on_each_test_thread() {
    isolated_capture_test!(capture_rebuilds_interest_after_uncaptured_call_on_each_test_thread);
    drop(derive_candidate_abi(
        &reviewed_target(),
        &linked_bundle(),
        pin(),
    ));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            std::thread::spawn(|| {
                let (result, records) =
                    capture(|| derive_candidate_abi(&reviewed_target(), &linked_bundle(), pin()));
                assert!(result.is_ok());
                assert_eq!(
                    records
                        .iter()
                        .filter(|record| {
                            record.kind == "span"
                                && record.target == "transaction::abi"
                                && record.name == "derive_abi"
                        })
                        .count(),
                    1,
                );
            })
        })
        .collect();
    for thread in threads {
        thread.join().expect("capture test thread");
    }
}

pub(super) fn assert_boundary(
    records: &[TraceRecord],
    target: &str,
    span: &str,
    outcome: &str,
    refusal: Option<&str>,
) {
    assert_boundary_at_level(records, target, span, outcome, refusal, Level::INFO);
}

pub(super) fn assert_boundary_at_level(
    records: &[TraceRecord],
    target: &str,
    span: &str,
    outcome: &str,
    refusal: Option<&str>,
    success_level: Level,
) {
    assert_eq!(
        records
            .iter()
            .filter(|r| r.kind == "span" && r.target == target && r.name == span)
            .count(),
        1
    );
    assert_eq!(
        records
            .iter()
            .filter(|r| r.kind == "close" && r.target == target && r.name == span)
            .count(),
        1
    );
    let events: Vec<_> = records
        .iter()
        .filter(|r| r.kind == "event" && r.target == target)
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].fields.get("outcome").map(String::as_str),
        Some(outcome)
    );
    assert_eq!(
        events[0].fields.get("refusal_kind").map(String::as_str),
        refusal
    );
    if target == "transaction::operator_right" {
        let message = match span {
            "issue_right" => "operator right issued",
            "replace_right" => "operator right replaced",
            "consume_right" => "operator right consumed",
            "retry_right" => "operator right retried",
            "dispatch_right" => "operator right dispatched",
            "timeout_right" => "operator right timed out",
            _ => panic!("unrecognized operator right span {span}"),
        };
        assert_eq!(
            events[0]
                .fields
                .get("message")
                .map(|value| value.trim_matches('"')),
            Some(message)
        );
    }
    assert_eq!(
        events[0].level,
        if refusal.is_some() {
            Level::DEBUG
        } else {
            success_level
        }
    );
    assert!(
        !records
            .iter()
            .any(|r| r.kind == "event" && matches!(r.level, Level::WARN | Level::ERROR))
    );
}

pub(super) fn assert_event_field(records: &[TraceRecord], target: &str, field: &str, value: &str) {
    assert_eq!(
        records
            .iter()
            .find(|record| record.kind == "event" && record.target == target)
            .and_then(|record| record.fields.get(field))
            .map(String::as_str),
        Some(value)
    );
}

pub(super) fn assert_span_count(
    records: &[TraceRecord],
    target: &str,
    span: &str,
    level: Level,
    count: usize,
) {
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record.kind == "span"
                    && record.target == target
                    && record.name == span
                    && record.level == level
            })
            .count(),
        count,
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "close"
                && record.target == target
                && record.name == span)
            .count(),
        count,
    );
}

#[test]
fn compact_abi_control_and_one_byte_pin_refusal_trace_without_changing_results() {
    isolated_capture_test!(
        compact_abi_control_and_one_byte_pin_refusal_trace_without_changing_results
    );
    let target = reviewed_target();
    let bundle = linked_bundle();
    let plain = derive_candidate_abi(&target, &bundle, pin());
    let (traced, records) = capture(|| derive_candidate_abi(&target, &bundle, pin()));
    assert_eq!(plain, traced);
    assert_boundary(
        &records,
        "transaction::abi",
        "derive_abi",
        "completed",
        None,
    );

    let mut key = super::INTERNAL_KEY;
    key[0] ^= 0xff;
    let wrong = || {
        PinnedAshInstance::new(
            &super::PINNED_PROGRAM,
            OutputKeyParity::Even,
            target_elements::LeafVersion::TAPSCRIPT,
            key.to_vec(),
            AshInstanceOrigin::SyntheticTestFunding,
        )
        .expect("the offered pin has the reviewed width")
    };
    let plain = derive_candidate_abi(&target, &bundle, wrong());
    let (traced, records) = capture(|| derive_candidate_abi(&target, &bundle, wrong()));
    assert_eq!(plain, traced);
    assert_eq!(traced, Err(TransactionRefusal::PinnedInternalKeyMismatch));
    assert_boundary(
        &records,
        "transaction::abi",
        "derive_abi",
        "refused",
        Some("pinned_internal_key_mismatch"),
    );
}

#[test]
fn compact_construction_control_and_one_flag_refusal_preserve_bytes() {
    isolated_capture_test!(compact_construction_control_and_one_flag_refusal_preserve_bytes);
    let target = reviewed_target();
    let abi = super::candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let offered = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("two inputs");
    let plain = construct(&target, &abi, &request, &offered, None)
        .expect("control constructs")
        .bytes();
    let (traced, records) = capture(|| construct(&target, &abi, &request, &offered, None));
    assert_eq!(plain, traced.expect("traced control constructs").bytes());
    assert_boundary(
        &records,
        "transaction::construct",
        "construct",
        "completed",
        None,
    );

    let wrong = CompactAshRequest::new([first, second], true)
        .expect("sponsor flag is a request coordinate");
    let plain = construct(&target, &abi, &wrong, &offered, None);
    let (traced, records) = capture(|| construct(&target, &abi, &wrong, &offered, None));
    assert_eq!(plain, traced);
    assert_eq!(
        traced,
        Err(TransactionRefusal::SponsorRequestedWithoutCapability)
    );
    assert_boundary(
        &records,
        "transaction::construct",
        "construct",
        "refused",
        Some("sponsor_requested_without_capability"),
    );
}

#[test]
fn live_abi_control_and_one_curve_refusal_preserve_results() {
    use super::live_support::{FixtureCurve, RefusingCurve, linked_live_bundle};
    isolated_capture_test!(live_abi_control_and_one_curve_refusal_preserve_results);

    let target = reviewed_target();
    let bundle = linked_live_bundle();
    let plain = derive_live_transfer_abi(&target, &bundle, &FixtureCurve);
    let (traced, records) = capture(|| derive_live_transfer_abi(&target, &bundle, &FixtureCurve));
    assert_eq!(plain, traced);
    assert_boundary(
        &records,
        "transaction::live_abi",
        "derive_abi",
        "completed",
        None,
    );

    let plain = derive_live_transfer_abi(&target, &bundle, &RefusingCurve);
    let (traced, records) = capture(|| derive_live_transfer_abi(&target, &bundle, &RefusingCurve));
    assert_eq!(plain, traced);
    assert!(matches!(
        traced,
        Err(TransactionRefusal::OwnerKeyIsNotACurvePoint { .. })
    ));
    assert_boundary(
        &records,
        "transaction::live_abi",
        "derive_abi",
        "refused",
        Some("owner_key_is_not_a_curve_point"),
    );
}

#[test]
fn decoding_one_truncated_byte_refuses_at_debug_without_changing_bytes() {
    isolated_capture_test!(decoding_one_truncated_byte_refuses_at_debug_without_changing_bytes);
    let target = reviewed_target();
    let abi = super::candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let offered = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("two inputs");
    let bytes = construct(&target, &abi, &request, &offered, None)
        .expect("control constructs")
        .bytes();
    let plain = TargetTransaction::decode(&bytes);
    let (traced, records) = capture(|| TargetTransaction::decode(&bytes));
    assert_eq!(plain, traced);
    assert_boundary_at_level(
        &records,
        "transaction::bytes",
        "decode_transaction",
        "completed",
        None,
        Level::DEBUG,
    );
    assert_event_field(
        &records,
        "transaction::bytes",
        "input_bytes",
        &bytes.len().to_string(),
    );

    let mut truncated = bytes;
    truncated.pop();
    let plain = TargetTransaction::decode(&truncated);
    let (traced, records) = capture(|| TargetTransaction::decode(&truncated));
    assert_eq!(plain, traced);
    assert!(matches!(
        traced,
        Err(TransactionRefusal::TruncatedTargetBytes { .. })
    ));
    assert_boundary_at_level(
        &records,
        "transaction::bytes",
        "decode_transaction",
        "refused",
        Some("truncated_target_bytes"),
        Level::DEBUG,
    );
}
