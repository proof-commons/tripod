//! Publication tracing preserves envelope verdicts and exact artifact bytes.

use crate::{ARCHITECTURE, PublishedArchitecture};

use super::deployment_tests::release_architecture;
use super::tracing_capture::{Record, RecordKind, Value, capture, run_isolated};

fn assert_export(
    records: &[Record],
    span_name: &str,
    message: &str,
    outcome: &str,
    refusal: Option<&str>,
) {
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
    assert_eq!(spans[0].target, "architecture::export");
    assert_eq!(events[0].target, "architecture::export");
    assert_eq!(spans[0].level, tracing::Level::DEBUG);
    assert_eq!(events[0].level, tracing::Level::DEBUG);
    assert_eq!(
        events[0].fields.get("message"),
        Some(&Value::Str(message.to_owned()))
    );
    assert_eq!(
        events[0].fields.get("outcome"),
        Some(&Value::Str(outcome.to_owned()))
    );
    let expected_refusal = refusal.map(|kind| Value::Str(kind.to_owned()));
    assert_eq!(
        events[0].fields.get("refusal_kind").cloned(),
        expected_refusal
    );
}

fn draft_envelope() -> PublishedArchitecture {
    PublishedArchitecture::from_architecture(&super::validated(&ARCHITECTURE)).unwrap()
}

fn final_envelope() -> PublishedArchitecture {
    let architecture = release_architecture();
    PublishedArchitecture::from_architecture(&super::validated(&architecture)).unwrap()
}

#[test]
fn envelope_construction_and_rendering_preserve_values_and_bytes() {
    if run_isolated(
        "tests::export_tracing_tests::envelope_construction_and_rendering_preserve_values_and_bytes",
    ) {
        return;
    }
    let validated = super::validated(&ARCHITECTURE);
    let plain = PublishedArchitecture::from_architecture(&validated).unwrap();
    let (observed, records) = capture(|| PublishedArchitecture::from_architecture(&validated));
    assert_eq!(observed.unwrap(), plain);
    assert_export(
        &records,
        "build_envelope",
        "envelope construction completed",
        "completed",
        None,
    );

    let json = plain.to_artifact_json().unwrap();
    let (observed, records) = capture(|| plain.to_artifact_json());
    assert_eq!(observed.unwrap(), json);
    assert_export(
        &records,
        "render_artifact",
        "artifact rendering completed",
        "completed",
        None,
    );
    assert_eq!(
        records[0].fields.get("format"),
        Some(&Value::Str("json".to_owned()))
    );
    assert_eq!(
        records[1].fields.get("rendered_bytes"),
        Some(&Value::U64(json.len() as u64))
    );

    let toml = plain.to_artifact_toml().unwrap();
    let (observed, records) = capture(|| plain.to_artifact_toml());
    assert_eq!(observed.unwrap(), toml);
    assert_export(
        &records,
        "render_artifact",
        "artifact rendering completed",
        "completed",
        None,
    );
    assert_eq!(
        records[0].fields.get("format"),
        Some(&Value::Str("toml".to_owned()))
    );
    assert_eq!(
        records[1].fields.get("rendered_bytes"),
        Some(&Value::U64(toml.len() as u64))
    );
}

#[test]
fn hash_verification_has_one_outcome_for_control_and_one_coordinate_refusal() {
    if run_isolated(
        "tests::export_tracing_tests::hash_verification_has_one_outcome_for_control_and_one_coordinate_refusal",
    ) {
        return;
    }
    let envelope = draft_envelope();
    let plain = envelope.verify_hashes().map_err(|error| error.name());
    let (observed, records) = capture(|| envelope.verify_hashes().map_err(|error| error.name()));
    assert_eq!(observed, plain);
    assert!(plain.is_ok());
    assert_export(
        &records,
        "verify_hashes",
        "envelope validation completed",
        "completed",
        None,
    );
    assert_eq!(records[1].fields.get("defect_count"), Some(&Value::U64(0)));

    let mut invalid = envelope;
    invalid.semantic_hash.push('x');
    let plain = invalid.verify_hashes().map_err(|error| error.name());
    let (observed, records) = capture(|| invalid.verify_hashes().map_err(|error| error.name()));
    assert_eq!(observed, plain);
    assert_eq!(plain, Err("hash_mismatch"));
    assert_export(
        &records,
        "verify_hashes",
        "envelope validation completed",
        "refused",
        Some("hash_mismatch"),
    );
    assert_eq!(records[1].fields.get("defect_count"), Some(&Value::U64(1)));
}

#[test]
fn envelope_validation_has_one_outcome_for_control_and_schema_refusal() {
    if run_isolated(
        "tests::export_tracing_tests::envelope_validation_has_one_outcome_for_control_and_schema_refusal",
    ) {
        return;
    }
    let envelope = draft_envelope();
    let plain = envelope.validate_envelope().map_err(|error| error.name());
    let (observed, records) =
        capture(|| envelope.validate_envelope().map_err(|error| error.name()));
    assert_eq!(observed, plain);
    assert!(plain.is_ok());
    assert_export(
        &records,
        "validate_envelope",
        "envelope validation completed",
        "completed",
        None,
    );

    let mut invalid = envelope;
    invalid.architecture_schema_version += 1;
    let plain = invalid.validate_envelope().map_err(|error| error.name());
    let (observed, records) = capture(|| invalid.validate_envelope().map_err(|error| error.name()));
    assert_eq!(observed, plain);
    assert_eq!(plain, Err("unsupported_schema_version"));
    assert_export(
        &records,
        "validate_envelope",
        "envelope validation completed",
        "refused",
        Some("unsupported_schema_version"),
    );
}

#[test]
fn release_envelope_validation_has_one_outcome_for_final_and_draft() {
    if run_isolated(
        "tests::export_tracing_tests::release_envelope_validation_has_one_outcome_for_final_and_draft",
    ) {
        return;
    }
    let envelope = final_envelope();
    let plain = envelope
        .validate_release_envelope()
        .map_err(|error| error.name());
    let (observed, records) = capture(|| {
        envelope
            .validate_release_envelope()
            .map_err(|error| error.name())
    });
    assert_eq!(observed, plain);
    assert!(plain.is_ok());
    assert_export(
        &records,
        "validate_release_envelope",
        "envelope validation completed",
        "completed",
        None,
    );

    let mut invalid = envelope;
    invalid.publication_status = "draft".to_owned();
    let plain = invalid
        .validate_release_envelope()
        .map_err(|error| error.name());
    let (observed, records) = capture(|| {
        invalid
            .validate_release_envelope()
            .map_err(|error| error.name())
    });
    assert_eq!(observed, plain);
    assert_eq!(plain, Err("not_final"));
    assert_export(
        &records,
        "validate_release_envelope",
        "envelope validation completed",
        "refused",
        Some("not_final"),
    );
}

#[test]
fn expected_envelope_validation_has_one_outcome_for_equal_and_changed_version() {
    if run_isolated(
        "tests::export_tracing_tests::expected_envelope_validation_has_one_outcome_for_equal_and_changed_version",
    ) {
        return;
    }
    let expected = draft_envelope();
    let plain = expected
        .validate_against_expected(&expected)
        .map_err(|error| error.name());
    let (observed, records) = capture(|| {
        expected
            .validate_against_expected(&expected)
            .map_err(|error| error.name())
    });
    assert_eq!(observed, plain);
    assert!(plain.is_ok());
    assert_export(
        &records,
        "validate_against_expected",
        "envelope validation completed",
        "completed",
        None,
    );

    let mut invalid = expected.clone();
    invalid.realization_version = "0.8.0-dev".to_owned();
    let plain = invalid
        .validate_against_expected(&expected)
        .map_err(|error| error.name());
    let (observed, records) = capture(|| {
        invalid
            .validate_against_expected(&expected)
            .map_err(|error| error.name())
    });
    assert_eq!(observed, plain);
    assert_eq!(plain, Err("unexpected_publication"));
    assert_export(
        &records,
        "validate_against_expected",
        "envelope validation completed",
        "refused",
        Some("unexpected_publication"),
    );
}
