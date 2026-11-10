use std::fmt::Debug;

use architecture::{ARCHITECTURE, OperationId};
use tracing::Level;

use super::{
    announcement_input, bound_input, phase1_realization, test_policy,
    tracing_capture::{CapturedRecord, FieldValue, capture, isolate_capture_test},
};
use crate::{
    AnalysisPolicy, BoundCompilerInput, CompilationScope, CompileError, ProofSearchLimits,
    bind_input, live_transfer_plan::plan_live_transfer_target_operation,
    maturity_announcement_plan::plan_maturity_announcement_target_operation,
    operation_plan::plan_compact_ash_target_operation, placement::PlacementSearchLimits,
    target::analyze_target_requirements,
};

#[test]
fn compile_error_tracing_kinds_are_canonical_snake_case() {
    let table = include_str!("../error.rs")
        .split_once("pub(crate) const fn tracing_kind")
        .expect("tracing kind table")
        .1
        .split_once("    /// Whether a returned error")
        .expect("end of tracing kind table")
        .0;
    let entries = table.split("Self::").skip(1).collect::<Vec<_>>();
    // Includes the typed refusal for a missing owner reverse-registry entry.
    assert_eq!(entries.len(), 142);

    for entry in entries {
        let variant = entry
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>();
        let name = entry.split('"').nth(1).expect("static tracing kind name");
        let mut expected = String::new();
        for (index, letter) in variant.chars().enumerate() {
            if letter.is_ascii_uppercase() {
                if index != 0 {
                    expected.push('_');
                }
                expected.push(letter.to_ascii_lowercase());
            } else {
                expected.push(letter);
            }
        }
        assert_eq!(name, expected, "{variant}");

        let bytes = name.as_bytes();
        assert!(bytes.first().is_some_and(u8::is_ascii_lowercase), "{name}");
        assert!(bytes.last().is_none_or(|byte| *byte != b'_'), "{name}");
        assert!(
            bytes.windows(2).all(|pair| pair != b"__")
                && bytes.iter().all(|byte| byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || *byte == b'_'),
            "{name}"
        );
    }
}

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        10_000_000_u64.try_into().unwrap(),
        1_000_000_u64.try_into().unwrap(),
    )
}

fn tiny_proof_limit(input: &BoundCompilerInput) -> BoundCompilerInput {
    bind_input(
        &ARCHITECTURE,
        input.realization().clone(),
        input.scope().clone(),
        AnalysisPolicy::strict(ProofSearchLimits::new(
            1_u64.try_into().unwrap(),
            input.policy().proof_search_limits.maximum_candidates,
        )),
    )
    .unwrap()
}

fn completion<'a>(
    records: &'a [CapturedRecord],
    target: &str,
    message: &str,
    outcome: &str,
    level: Level,
) -> &'a CapturedRecord {
    let matching = records
        .iter()
        .filter(|record| {
            !record.is_span
                && record.target == target
                && record.fields.get("message") == Some(&FieldValue::Str(message.into()))
        })
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "one completion at {target}");
    let record = matching[0];
    assert_eq!(record.level, level);
    assert_eq!(
        record.fields.get("outcome"),
        Some(&FieldValue::Str(outcome.into()))
    );
    assert!(
        records
            .iter()
            .all(|record| record.level != Level::WARN && record.level != Level::ERROR)
    );
    record
}

fn assert_analysis_boundary<T: Debug + PartialEq>(
    input: &BoundCompilerInput,
    operation: fn(&BoundCompilerInput, PlacementSearchLimits) -> Result<T, CompileError>,
    target: &str,
    message: &str,
) {
    let plain = operation(input, limits()).unwrap();
    let (captured, records) = capture(|| operation(input, limits()).unwrap());
    assert_eq!(plain, captured);
    assert_eq!(
        format!("{plain:?}").into_bytes(),
        format!("{captured:?}").into_bytes()
    );
    let complete = completion(&records, target, message, "completed", Level::INFO);
    assert!(complete.fields.contains_key("proof_plan_count"));
    let span_name = if target == "compiler::target" {
        "analyze_target_requirements"
    } else {
        "plan_target_operation"
    };
    assert_eq!(
        records
            .iter()
            .filter(|record| record.is_span && record.target == target && record.name == span_name)
            .count(),
        1
    );

    let refused_input = tiny_proof_limit(input);
    let plain = operation(&refused_input, limits()).unwrap_err();
    let (captured, records) = capture(|| operation(&refused_input, limits()).unwrap_err());
    assert_eq!(
        plain,
        CompileError::ProofSearchStateLimitExceeded { maximum: 1 }
    );
    assert_eq!(plain, captured);
    assert_eq!(
        format!("{plain:?}").into_bytes(),
        format!("{captured:?}").into_bytes()
    );
    let refusal = completion(&records, target, message, "refused", Level::DEBUG);
    assert_eq!(
        refusal.fields.get("refusal_kind"),
        Some(&FieldValue::Str("proof_search_state_limit_exceeded".into()))
    );
}

#[test]
fn binding_completion_and_one_operation_refusal_are_debug() {
    if isolate_capture_test(
        "tests::tracing_boundary_tests::binding_completion_and_one_operation_refusal_are_debug",
    ) {
        return;
    }
    let realization = phase1_realization();
    let scope = CompilationScope::from_operations([OperationId::CompactAsh]).unwrap();
    let plain = bind_input(
        &ARCHITECTURE,
        realization.clone(),
        scope.clone(),
        test_policy(),
    )
    .unwrap();
    let (captured, records) =
        capture(|| bind_input(&ARCHITECTURE, realization.clone(), scope, test_policy()).unwrap());
    assert_eq!(
        format!("{plain:?}").into_bytes(),
        format!("{captured:?}").into_bytes()
    );
    let record = completion(
        &records,
        "compiler::input",
        "input binding completed",
        "completed",
        Level::DEBUG,
    );
    assert_eq!(
        record.fields.get("scope_operation_count"),
        Some(&FieldValue::U64(1))
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.is_span && record.name == "bind_input")
            .count(),
        1
    );

    let refused_scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::AnnounceMaturity])
            .unwrap();
    let plain = bind_input(
        &ARCHITECTURE,
        realization.clone(),
        refused_scope.clone(),
        test_policy(),
    )
    .unwrap_err();
    let (captured, records) = capture(|| {
        bind_input(&ARCHITECTURE, realization, refused_scope, test_policy()).unwrap_err()
    });
    assert_eq!(plain, captured);
    assert_eq!(
        format!("{plain:?}").into_bytes(),
        format!("{captured:?}").into_bytes()
    );
    let record = completion(
        &records,
        "compiler::input",
        "input binding completed",
        "refused",
        Level::DEBUG,
    );
    assert_eq!(
        record.fields.get("refusal_kind"),
        Some(&FieldValue::Str("incomplete_realization_scope".into()))
    );
}

#[test]
fn requirements_completion_and_proof_limit_refusal() {
    if isolate_capture_test(
        "tests::tracing_boundary_tests::requirements_completion_and_proof_limit_refusal",
    ) {
        return;
    }
    assert_analysis_boundary(
        &bound_input(&[OperationId::CompactAsh]),
        analyze_target_requirements,
        "compiler::target",
        "target requirements analyzed",
    );
}

#[test]
fn compact_plan_completion_and_proof_limit_refusal() {
    if isolate_capture_test(
        "tests::tracing_boundary_tests::compact_plan_completion_and_proof_limit_refusal",
    ) {
        return;
    }
    assert_analysis_boundary(
        &bound_input(&[OperationId::CompactAsh]),
        plan_compact_ash_target_operation,
        "compiler::plan",
        "target operation planned",
    );
}

#[test]
fn live_plan_completion_and_proof_limit_refusal() {
    if isolate_capture_test(
        "tests::tracing_boundary_tests::live_plan_completion_and_proof_limit_refusal",
    ) {
        return;
    }
    assert_analysis_boundary(
        &bound_input(&[OperationId::TransferLive]),
        plan_live_transfer_target_operation,
        "compiler::plan",
        "target operation planned",
    );
}

#[test]
fn state_plan_completion_and_proof_limit_refusal() {
    if isolate_capture_test(
        "tests::tracing_boundary_tests::state_plan_completion_and_proof_limit_refusal",
    ) {
        return;
    }
    assert_analysis_boundary(
        &announcement_input(),
        plan_maturity_announcement_target_operation,
        "compiler::plan",
        "target operation planned",
    );
}

#[test]
fn keyed_production_rederives_every_stage_without_a_producer_cache() {
    if isolate_capture_test(
        "tests::tracing_boundary_tests::keyed_production_rederives_every_stage_without_a_producer_cache",
    ) {
        return;
    }
    let input = super::engine_coverage_tests::bound(vec![
        architecture::semantic::KeyedOperationId::Source,
        architecture::semantic::KeyedOperationId::Merge,
    ]);
    let (result, events) = capture(|| crate::keyed_plan::plan(&input, limits()));
    assert!(result.is_ok());
    for (target, name) in [
        ("compiler::engine_keyed", "prepare"),
        ("compiler::engine_records", "analyze_relations"),
        ("compiler::engine_source", "analyze_sources"),
        (
            "compiler::engine_constructibility",
            "analyze_constructibility",
        ),
        ("compiler::engine_lifecycle", "analyze_lifecycle"),
        ("compiler::engine_proof", "enumerate"),
    ] {
        assert_eq!(
            events
                .iter()
                .filter(|event| event.is_span && event.target == target && event.name == name)
                .count(),
            2,
            "{target}::{name}"
        );
    }
    assert_eq!(
        events
            .iter()
            .filter(|event| event.is_span
                && event.target == "compiler::engine_placement"
                && event.name == "enumerate")
            .count(),
        4
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.is_span
                && event.target == "compiler::engine_coverage"
                && event.name == "validate_census")
            .count(),
        4
    );
    let plan = result.unwrap();
    let (result, events) = capture(|| crate::keyed_plan::revalidate(&input, limits(), &plan));
    assert!(result.is_ok());
    assert_eq!(
        events
            .iter()
            .filter(|event| event.is_span
                && event.target == "compiler::engine_keyed"
                && event.name == "prepare")
            .count(),
        1
    );
}
