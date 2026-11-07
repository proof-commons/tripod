//! Tracing does not change architecture admission or its returned defects.

use crate::{
    ARCHITECTURE, DeploymentError, ManifestError, validate_architecture_release,
    validate_deployment_profile, validate_deployment_profile_structure, validate_draft,
    validate_production_deployment_release,
};

use super::deployment_tests::{release_architecture, release_profile};
use super::tracing_capture::{Record, RecordKind, Value, capture, run_isolated};

fn assert_single_validation(
    records: &[Record],
    span: &str,
    schema_version: u64,
    message: &str,
    outcome: &str,
    count: u64,
    refusal: Option<&str>,
) {
    let spans = records
        .iter()
        .filter(|record| record.kind == RecordKind::Span)
        .collect::<Vec<_>>();
    let closes = records
        .iter()
        .filter(|record| record.kind == RecordKind::SpanClose)
        .collect::<Vec<_>>();
    let events = records
        .iter()
        .filter(|record| record.kind == RecordKind::Event)
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), 1);
    assert_eq!(closes.len(), 1);
    assert_eq!(events.len(), 1);
    assert_eq!(spans[0].name, span);
    assert_eq!(closes[0].name, span);
    assert_eq!(spans[0].target, "architecture::validate");
    assert_eq!(events[0].target, "architecture::validate");
    assert_eq!(spans[0].level, tracing::Level::DEBUG);
    assert_eq!(events[0].level, tracing::Level::DEBUG);
    assert_eq!(
        spans[0].fields.get("schema_version"),
        Some(&Value::U64(schema_version))
    );
    if span == "validate_draft" {
        assert_eq!(
            spans[0].fields.get("operation_count"),
            Some(&Value::U64(ARCHITECTURE.operations.len() as u64))
        );
    }
    assert_eq!(
        events[0].fields.get("message"),
        Some(&Value::Str(message.to_owned()))
    );
    assert_eq!(
        events[0].fields.get("outcome"),
        Some(&Value::Str(outcome.to_owned()))
    );
    assert_eq!(
        events[0].fields.get("defect_count"),
        Some(&Value::U64(count))
    );
    let expected_refusal = refusal.map(|kind| Value::Str(kind.to_owned()));
    assert_eq!(
        events[0].fields.get("refusal_kind").cloned(),
        expected_refusal
    );
}

#[test]
fn draft_tracing_preserves_control_and_one_coordinate_refusal() {
    if run_isolated(
        "tests::tracing_tests::draft_tracing_preserves_control_and_one_coordinate_refusal",
    ) {
        return;
    }
    let plain = validate_draft(&ARCHITECTURE).unwrap();
    let (observed, records) = capture(|| validate_draft(&ARCHITECTURE));
    assert_eq!(plain.architecture(), observed.unwrap().architecture());
    assert_single_validation(
        &records,
        "validate_draft",
        18,
        "draft validation completed",
        "completed",
        0,
        None,
    );

    let mut invalid = ARCHITECTURE;
    invalid.document.architecture_schema_version = 0;
    let plain = validate_draft(&invalid).unwrap_err();
    let (observed, records) = capture(|| validate_draft(&invalid));
    assert_eq!(observed.unwrap_err(), plain);
    assert_eq!(plain.len(), 1);
    assert_eq!(plain[0].name(), "invalid_document");
    assert_single_validation(
        &records,
        "validate_draft",
        0,
        "draft validation completed",
        "refused",
        1,
        Some("invalid_document"),
    );
}

#[test]
fn release_tracing_preserves_control_and_one_coordinate_refusal_without_nested_event() {
    if run_isolated(
        "tests::tracing_tests::release_tracing_preserves_control_and_one_coordinate_refusal_without_nested_event",
    ) {
        return;
    }
    let plain = validate_architecture_release(&ARCHITECTURE).unwrap();
    let (observed, records) = capture(|| validate_architecture_release(&ARCHITECTURE));
    assert_eq!(plain.architecture(), observed.unwrap().architecture());
    assert_single_validation(
        &records,
        "validate_architecture_release",
        18,
        "release validation completed",
        "completed",
        0,
        None,
    );

    let mut invalid = ARCHITECTURE;
    invalid.document.specification.anchor_set_hash = None;
    let plain = validate_architecture_release(&invalid).unwrap_err();
    let (observed, records) = capture(|| validate_architecture_release(&invalid));
    assert_eq!(observed.unwrap_err(), plain);
    assert_eq!(plain, vec![ManifestError::UnpinnedSpecification]);
    assert_single_validation(
        &records,
        "validate_architecture_release",
        18,
        "release validation completed",
        "refused",
        1,
        Some("unpinned_specification"),
    );
}

fn assert_single_deployment(
    records: &[Record],
    span: &str,
    message: &str,
    count: u64,
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
    assert_eq!(spans[0].name, span);
    assert_eq!(spans[0].target, "architecture::deployment");
    assert_eq!(events[0].target, "architecture::deployment");
    assert_eq!(spans[0].level, tracing::Level::DEBUG);
    assert_eq!(events[0].level, tracing::Level::DEBUG);
    let expected_schema = if refusal == Some("unsupported_schema_version") {
        1
    } else {
        2
    };
    assert_eq!(
        spans[0].fields.get("profile_schema_version"),
        Some(&Value::U64(expected_schema))
    );
    assert_eq!(
        events[0].fields.get("message"),
        Some(&Value::Str(message.to_owned()))
    );
    assert_eq!(
        events[0].fields.get("defect_count"),
        Some(&Value::U64(count))
    );
    let outcome = if refusal.is_some() {
        "refused"
    } else {
        "completed"
    };
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

#[test]
fn structure_tracing_preserves_control_and_schema_refusal() {
    if run_isolated("tests::tracing_tests::structure_tracing_preserves_control_and_schema_refusal")
    {
        return;
    }
    let architecture = release_architecture();
    let profile = release_profile(&architecture);
    let plain = validate_deployment_profile_structure(&architecture, &profile);
    let (observed, records) =
        capture(|| validate_deployment_profile_structure(&architecture, &profile));
    assert_eq!(observed, plain);
    assert!(plain.is_ok());
    assert_single_deployment(
        &records,
        "validate_deployment_profile_structure",
        "deployment structure validation completed",
        0,
        None,
    );

    let mut invalid = profile;
    invalid.schema_version = 1;
    let plain = validate_deployment_profile_structure(&architecture, &invalid);
    let (observed, records) =
        capture(|| validate_deployment_profile_structure(&architecture, &invalid));
    assert_eq!(observed, plain);
    assert_eq!(
        plain.unwrap_err(),
        vec![DeploymentError::UnsupportedSchemaVersion]
    );
    assert_single_deployment(
        &records,
        "validate_deployment_profile_structure",
        "deployment structure validation completed",
        1,
        Some("unsupported_schema_version"),
    );
}

#[test]
fn profile_wrapper_tracing_has_one_outcome_and_preserves_admission() {
    if run_isolated(
        "tests::tracing_tests::profile_wrapper_tracing_has_one_outcome_and_preserves_admission",
    ) {
        return;
    }
    let architecture = release_architecture();
    let profile = release_profile(&architecture);
    let plain = validate_deployment_profile(&architecture, &profile).unwrap();
    let (observed, records) = capture(|| validate_deployment_profile(&architecture, &profile));
    assert!(std::ptr::eq(plain.profile(), observed.unwrap().profile()));
    assert_single_deployment(
        &records,
        "validate_deployment_profile",
        "deployment structure validation completed",
        0,
        None,
    );

    let mut invalid = profile;
    invalid.schema_version = 1;
    let plain = validate_deployment_profile(&architecture, &invalid).unwrap_err();
    let (observed, records) = capture(|| validate_deployment_profile(&architecture, &invalid));
    assert_eq!(observed.unwrap_err(), plain);
    assert_eq!(plain, vec![DeploymentError::UnsupportedSchemaVersion]);
    assert_single_deployment(
        &records,
        "validate_deployment_profile",
        "deployment structure validation completed",
        1,
        Some("unsupported_schema_version"),
    );
}

#[test]
fn production_tracing_reports_policy_refusal_and_first_structural_defect() {
    if run_isolated(
        "tests::tracing_tests::production_tracing_reports_policy_refusal_and_first_structural_defect",
    ) {
        return;
    }
    let architecture = release_architecture();
    let profile = release_profile(&architecture);
    let plain = validate_production_deployment_release(&architecture, &profile);
    let (observed, records) =
        capture(|| validate_production_deployment_release(&architecture, &profile));
    assert_eq!(observed, plain);
    assert_eq!(
        plain.unwrap_err(),
        vec![DeploymentError::ProductionReleaseUnsupported]
    );
    assert_single_deployment(
        &records,
        "validate_production_deployment_release",
        "production release validation completed",
        1,
        Some("production_release_unsupported"),
    );

    let mut invalid = profile;
    invalid.schema_version = 1;
    let plain = validate_production_deployment_release(&architecture, &invalid);
    let (observed, records) =
        capture(|| validate_production_deployment_release(&architecture, &invalid));
    assert_eq!(observed, plain);
    assert_eq!(
        plain.unwrap_err(),
        vec![
            DeploymentError::UnsupportedSchemaVersion,
            DeploymentError::ProductionReleaseUnsupported
        ]
    );
    assert_single_deployment(
        &records,
        "validate_production_deployment_release",
        "production release validation completed",
        2,
        Some("unsupported_schema_version"),
    );
}
