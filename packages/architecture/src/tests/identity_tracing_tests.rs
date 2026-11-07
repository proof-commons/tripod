//! Hash tracing preserves canonical values and adds one DEBUG outcome per entry point.

use crate::{
    ARCHITECTURE, behavioural_hash, behavioural_hash_hex, canonical_json_bytes,
    deployment_profile_hash, deployment_profile_hash_hex, semantic_hash, semantic_hash_hex,
    validate_deployment_profile,
};

use super::deployment_tests::{release_architecture, release_profile};
use super::tracing_capture::{Record, RecordKind, Value, capture, run_isolated};

fn assert_identity(records: &[Record], target: &str, span_name: &str, kind: &str) {
    let spans = records
        .iter()
        .filter(|record| record.kind == RecordKind::Span)
        .collect::<Vec<_>>();
    let events = records
        .iter()
        .filter(|record| record.kind == RecordKind::Event)
        .collect::<Vec<_>>();
    let close_count = records
        .iter()
        .filter(|record| record.kind == RecordKind::SpanClose)
        .count();
    assert_eq!((spans.len(), events.len(), close_count), (1, 1, 1));
    assert_eq!(spans[0].name, span_name);
    assert_eq!(spans[0].target, target);
    assert_eq!(events[0].target, target);
    assert_eq!(spans[0].level, tracing::Level::DEBUG);
    assert_eq!(events[0].level, tracing::Level::DEBUG);
    assert_eq!(
        events[0].fields.get("message"),
        Some(&Value::Str("identity computation completed".to_owned()))
    );
    assert_eq!(
        events[0].fields.get("outcome"),
        Some(&Value::Str("completed".to_owned()))
    );
    assert_eq!(
        events[0].fields.get("identity_kind"),
        Some(&Value::Str(kind.to_owned()))
    );
    assert!(!events[0].fields.contains_key("error_kind"));
}

#[test]
fn manifest_identity_calls_preserve_digests_hex_and_canonical_bytes() {
    if run_isolated(
        "tests::identity_tracing_tests::manifest_identity_calls_preserve_digests_hex_and_canonical_bytes",
    ) {
        return;
    }
    let validated = super::validated(&ARCHITECTURE);
    let plain_bytes = canonical_json_bytes(&validated).unwrap();
    let (observed_bytes, bytes_records) = capture(|| canonical_json_bytes(&validated));
    assert_eq!(observed_bytes.unwrap(), plain_bytes);
    assert!(bytes_records.is_empty());

    let plain = semantic_hash(&validated).unwrap();
    let (observed, records) = capture(|| semantic_hash(&validated));
    assert_eq!(observed.unwrap(), plain);
    assert_identity(
        &records,
        "architecture::canonical",
        "semantic_hash",
        "semantic",
    );

    let plain = semantic_hash_hex(&validated).unwrap();
    let (observed, records) = capture(|| semantic_hash_hex(&validated));
    assert_eq!(observed.unwrap(), plain);
    assert_identity(
        &records,
        "architecture::canonical",
        "semantic_hash_hex",
        "semantic",
    );

    let plain = behavioural_hash(&validated).unwrap();
    let (observed, records) = capture(|| behavioural_hash(&validated));
    assert_eq!(observed.unwrap(), plain);
    assert_identity(
        &records,
        "architecture::canonical",
        "behavioural_hash",
        "behavioural",
    );

    let plain = behavioural_hash_hex(&validated).unwrap();
    let (observed, records) = capture(|| behavioural_hash_hex(&validated));
    assert_eq!(observed.unwrap(), plain);
    assert_identity(
        &records,
        "architecture::canonical",
        "behavioural_hash_hex",
        "behavioural",
    );
}

#[test]
fn profile_identity_calls_preserve_digests_and_hex() {
    if run_isolated(
        "tests::identity_tracing_tests::profile_identity_calls_preserve_digests_and_hex",
    ) {
        return;
    }
    let architecture = release_architecture();
    let profile = release_profile(&architecture);
    let validated = validate_deployment_profile(&architecture, &profile).unwrap();

    let plain = deployment_profile_hash(&validated).unwrap();
    let (observed, records) = capture(|| deployment_profile_hash(&validated));
    assert_eq!(observed.unwrap(), plain);
    assert_identity(
        &records,
        "architecture::deployment",
        "deployment_profile_hash",
        "profile",
    );

    let plain = deployment_profile_hash_hex(&validated).unwrap();
    let (observed, records) = capture(|| deployment_profile_hash_hex(&validated));
    assert_eq!(observed.unwrap(), plain);
    assert_identity(
        &records,
        "architecture::deployment",
        "deployment_profile_hash_hex",
        "profile",
    );
}
