use architecture::{ARCHITECTURE, OperationId};

use crate::{
    ObservedOperatorMembership, ObservedValue, OperatorMembershipDisposition, ProtocolAmount,
    RealizationScope, derive, project_scoped_realization, validate_observation,
};

use super::{
    compact_ash_tests::valid_observation,
    tracing_tests::{Record, assert_no_warning_or_error, capture, in_capture_child},
};

fn event<'a>(records: &'a [Record], target: &str, message: &str, outcome: &str) -> &'a Record {
    let matches = records
        .iter()
        .filter(|record| {
            record.target == target
                && record.fields.get("message").map(String::as_str) == Some(message)
                && record.fields.get("outcome").map(String::as_str) == Some(outcome)
        })
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    matches[0]
}

fn assert_bool_field(record: &Record, field: &str, expected: bool) {
    assert_eq!(
        record.fields.get(field).map(String::as_str),
        Some(if expected { "true" } else { "false" })
    );
    assert_eq!(record.types.get(field), Some(&"bool"));
}

#[test]
fn direct_normalization_has_one_completion_or_refusal() {
    if !in_capture_child(
        "tests::tracing_evaluation_tests::direct_normalization_has_one_completion_or_refusal",
    ) {
        return;
    }
    let observation = valid_observation();
    let plain = validate_observation(observation.clone());
    let (captured, records) = capture(|| validate_observation(observation.clone()));
    assert_eq!(plain, captured);
    assert!(records.iter().any(|record| {
        record.target == "realization::observation"
            && record.name == "normalize_observation"
            && record.level == tracing::Level::DEBUG
    }));
    event(
        &records,
        "realization::observation",
        "observation normalized",
        "completed",
    );
    assert_no_warning_or_error(&records);

    let mut malformed = observation;
    malformed.objects.push(malformed.objects[0].clone());
    let plain = malformed.clone().validate_and_normalize();
    let (captured, records) = capture(|| malformed.validate_and_normalize());
    assert_eq!(plain, captured);
    let refusal = event(
        &records,
        "realization::observation",
        "observation normalized",
        "refused",
    );
    assert_eq!(refusal.level, tracing::Level::DEBUG);
    assert_eq!(
        refusal.fields.get("refusal_kind").map(String::as_str),
        Some("duplicate_observed_object")
    );
    assert_no_warning_or_error(&records);
}

#[test]
fn evaluation_preserves_reports_and_separates_negative_verdict_from_refusal() {
    if !in_capture_child(
        "tests::tracing_evaluation_tests::evaluation_preserves_reports_and_separates_negative_verdict_from_refusal",
    ) {
        return;
    }
    let scope = RealizationScope::from_operations([OperationId::CompactAsh]).unwrap();
    let spec = derive(&ARCHITECTURE, scope).unwrap();
    let control = valid_observation();
    let mut negative = control.clone();
    negative.objects[2].value = ObservedValue::Protocol(ProtocolAmount::new(99).unwrap());
    let membership = ObservedOperatorMembership::new(
        OperationId::CompactAsh,
        OperatorMembershipDisposition::Member,
        "test",
    )
    .unwrap();
    for observation in [&control, &negative] {
        let plain = spec.evaluate_operation(observation);
        let (captured, records) = capture(|| spec.evaluate_operation(observation));
        assert_eq!(plain, captured);
        let report = captured.unwrap();
        let expected_conformant = observation == &control;
        assert_eq!(report.is_conformant(), expected_conformant);
        assert!(records.iter().any(|record| {
            record.target == "realization::evaluate"
                && record.name == "evaluate_operation"
                && record.level == tracing::Level::DEBUG
                && record
                    .fields
                    .get("operator_membership_supplied")
                    .map(String::as_str)
                    == Some("false")
        }));
        let completion = event(
            &records,
            "realization::evaluate",
            "operation evaluated",
            "completed",
        );
        assert_eq!(
            completion.fields.get("conformant").map(String::as_str),
            Some(if expected_conformant { "true" } else { "false" })
        );
        assert_bool_field(completion, "conformant", report.is_conformant());
        assert_bool_field(
            completion,
            "evidence_complete",
            report.is_evidence_complete(),
        );
        assert_eq!(completion.types.get("failed_count"), Some(&"u64"));
        assert_eq!(completion.types.get("operation_code"), Some(&"u64"));
        assert_no_warning_or_error(&records);
    }

    let plain = spec.evaluate_operation_with_operator_membership(&control, &membership);
    let (captured, records) =
        capture(|| spec.evaluate_operation_with_operator_membership(&control, &membership));
    assert_eq!(plain, captured);
    assert!(records.iter().any(|record| {
        record.target == "realization::evaluate"
            && record.name == "evaluate_operation"
            && record
                .fields
                .get("operator_membership_supplied")
                .map(String::as_str)
                == Some("true")
    }));
    event(
        &records,
        "realization::evaluate",
        "operation evaluated",
        "completed",
    );
    assert_no_warning_or_error(&records);

    let mut malformed = control;
    malformed.objects.push(malformed.objects[0].clone());
    let plain = spec.evaluate_operation(&malformed);
    let (captured, records) = capture(|| spec.evaluate_operation(&malformed));
    assert_eq!(plain, captured);
    let refusal = event(
        &records,
        "realization::evaluate",
        "operation evaluated",
        "refused",
    );
    assert_eq!(refusal.level, tracing::Level::DEBUG);
    assert_eq!(
        refusal.fields.get("refusal_kind").map(String::as_str),
        Some("duplicate_observed_object")
    );
    assert!(
        !records
            .iter()
            .any(|record| record.target == "realization::observation")
    );
    assert_no_warning_or_error(&records);
}

#[test]
fn projection_aliases_each_emit_one_event_and_preserve_value() {
    if !in_capture_child(
        "tests::tracing_evaluation_tests::projection_aliases_each_emit_one_event_and_preserve_value",
    ) {
        return;
    }
    let scope = RealizationScope::from_operations([OperationId::CompactAsh]).unwrap();
    let spec = derive(&ARCHITECTURE, scope).unwrap();
    let plain = project_scoped_realization(&spec);
    let (captured, records) = capture(|| project_scoped_realization(&spec));
    assert_eq!(plain, captured);
    assert!(records.iter().any(|record| {
        record.target == "realization::derive"
            && record.name == "project_spec"
            && record.level == tracing::Level::DEBUG
    }));
    event(
        &records,
        "realization::derive",
        "realization projected",
        "completed",
    );
    let (captured_method, records) = capture(|| spec.project());
    assert_eq!(plain, captured_method);
    event(
        &records,
        "realization::derive",
        "realization projected",
        "completed",
    );
    assert_no_warning_or_error(&records);
}
