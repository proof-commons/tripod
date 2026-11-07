//! Local capture of tracing records for model instrumentation assertions.

use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tracing_subscriber::fmt::format::FmtSpan;

static CAPTURE_LOCK: Mutex<()> = Mutex::new(());
const CAPTURE_CHILD: &str = "TRIPOD_MODEL_CAPTURE_CHILD";

struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl Write for BufferWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn capture<R>(operation: impl FnOnce() -> R) -> (R, Vec<Value>) {
    let _capture_guard = CAPTURE_LOCK.lock().unwrap();
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let writer_bytes = Arc::clone(&bytes);
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(move || BufferWriter(Arc::clone(&writer_bytes)))
        .finish();

    let result = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        operation()
    });
    let captured = bytes.lock().unwrap().clone();
    let records = captured
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    (result, records)
}

pub fn events<'a>(records: &'a [Value], message: &str) -> Vec<&'a Value> {
    records
        .iter()
        .filter(|record| record["fields"]["message"] == message)
        .collect()
}

pub fn no_warn_or_error(records: &[Value]) -> bool {
    records
        .iter()
        .all(|record| record["level"] != "WARN" && record["level"] != "ERROR")
}

pub fn isolated_capture_child() -> bool {
    let current_thread = std::thread::current();
    let test_path = current_thread.name().unwrap();
    if std::env::var(CAPTURE_CHILD).ok().as_deref() == Some(test_path) {
        return true;
    }

    let output = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg(test_path)
        .env(CAPTURE_CHILD, test_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success()
            && stdout
                .lines()
                .any(|line| line.starts_with("test result: ok. 1 passed; 0 failed;")),
        "isolated capture test {test_path} failed or selected the wrong number of tests; status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    false
}

#[test]
fn capture_rebuilds_interest_after_unobserved_call_on_two_threads() {
    if !isolated_capture_child() {
        return;
    }
    fn probe() {
        let _span = tracing::debug_span!(target: "model::test", "capture_probe").entered();
    }

    probe();

    std::thread::scope(|scope| {
        let first = scope.spawn(|| capture(probe));
        let second = scope.spawn(|| capture(probe));
        for handle in [first, second] {
            let ((), records) = handle.join().unwrap();
            let closes: Vec<_> = records
                .iter()
                .filter(|record| {
                    record["fields"]["message"] == "close"
                        && record["span"]["name"] == "capture_probe"
                })
                .collect();
            assert_eq!(closes.len(), 1);
            assert_eq!(closes[0]["target"], "model::test");
        }
    });
}

#[test]
fn traced_variant_names_are_snake_case() {
    use crate::{BranchKind, ConformanceProjectionError, DifferentialError, ExecutionBindingError};

    fn assert_name(actual: &str, expected: &str) {
        assert_eq!(actual, expected);
        let mut parts = actual.split('_');
        let first = parts.next().unwrap();
        assert!(!first.is_empty() && first.bytes().all(|byte| byte.is_ascii_lowercase()));
        assert!(parts.all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        }));
    }

    let projection_names = [
        (ConformanceProjectionError::NoTransition, "no_transition"),
        (
            ConformanceProjectionError::WrongBranch {
                expected: BranchKind::CreateRequest,
                actual: BranchKind::CancelRequest,
            },
            "wrong_branch",
        ),
        (
            ConformanceProjectionError::MissingConsumedObject(0),
            "missing_consumed_object",
        ),
        (
            ConformanceProjectionError::MissingCreatedObject(0),
            "missing_created_object",
        ),
        (
            ConformanceProjectionError::UnknownFlowSource(0),
            "unknown_flow_source",
        ),
        (
            ConformanceProjectionError::UnknownFlowDestination(0),
            "unknown_flow_destination",
        ),
        (
            ConformanceProjectionError::UnknownDeltaSource(0),
            "unknown_delta_source",
        ),
        (
            ConformanceProjectionError::UnknownDeltaDestination(0),
            "unknown_delta_destination",
        ),
        (
            ConformanceProjectionError::UndeclaredCanonicalAsset,
            "undeclared_canonical_asset",
        ),
        (
            ConformanceProjectionError::HistoryLengthOverflow,
            "history_length_overflow",
        ),
        (
            ConformanceProjectionError::NotOneTransitionExtension,
            "not_one_transition_extension",
        ),
        (
            ConformanceProjectionError::HistoryPrefixChanged,
            "history_prefix_changed",
        ),
        (
            ConformanceProjectionError::NonIncreasingOrder,
            "non_increasing_order",
        ),
        (
            ConformanceProjectionError::ConsumedCreatedOverlap,
            "consumed_created_overlap",
        ),
        (
            ConformanceProjectionError::ConsumedObjectSurvived(0),
            "consumed_object_survived",
        ),
        (
            ConformanceProjectionError::CreatedObjectAlreadyExisted(0),
            "created_object_already_existed",
        ),
        (
            ConformanceProjectionError::InvalidObservation(
                realization::RealizationError::ArchitectureHashUnavailable,
            ),
            "invalid_observation",
        ),
        (
            ConformanceProjectionError::AmountOutOfDomain,
            "amount_out_of_domain",
        ),
        (
            ConformanceProjectionError::BoundOutOfDomain,
            "bound_out_of_domain",
        ),
        (
            ConformanceProjectionError::SponsorProtocolRegionOverlap(0),
            "sponsor_protocol_region_overlap",
        ),
    ];
    for (error, expected) in projection_names {
        assert_name(error.name(), expected);
    }

    let differential_names = [
        (DifferentialError::ContextMismatch, "context_mismatch"),
        (
            DifferentialError::EmptyAddressCensus,
            "empty_address_census",
        ),
        (
            DifferentialError::EventSnapshotFailure,
            "event_snapshot_failure",
        ),
        (
            DifferentialError::EventCountMismatch,
            "event_count_mismatch",
        ),
        (
            DifferentialError::EventOrderMismatch,
            "event_order_mismatch",
        ),
        (DifferentialError::EventKindMismatch, "event_kind_mismatch"),
        (
            DifferentialError::EventPayloadMismatch,
            "event_payload_mismatch",
        ),
        (
            DifferentialError::QueryValidationFailure,
            "query_validation_failure",
        ),
        (DifferentialError::QueryMismatch, "query_mismatch"),
        (
            DifferentialError::ExpectedZeroAddressNotZero,
            "expected_zero_address_not_zero",
        ),
        (
            DifferentialError::ReceiptAccountingProjectionMismatch,
            "receipt_accounting_projection_mismatch",
        ),
    ];
    for (error, expected) in differential_names {
        assert_name(error.name(), expected);
    }

    for (error, expected) in [
        (
            ExecutionBindingError::InvalidSuccessorExtension,
            "invalid_successor_extension",
        ),
        (
            ExecutionBindingError::RequestBindingMismatch,
            "request_binding_mismatch",
        ),
    ] {
        assert_name(error.name(), expected);
    }
}
