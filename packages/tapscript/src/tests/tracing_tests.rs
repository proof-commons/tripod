//! Scoped observations at the public instrumentation boundaries.

use std::num::NonZeroU64;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;

use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::TapscriptProgram;
use crate::stack::{AbstractLimits, AbstractStackState, validate_program};

use super::tracing_capture::{Record, capture, run_isolated};
use super::{
    compact_ash_plan, live_transfer_plan, live_transfer_symbols, pattern_symbols, reviewed_target,
};

fn owned_events<'a>(records: &'a [Record], target: &str) -> Vec<&'a Record> {
    records
        .iter()
        .filter(|record| record.kind == "event" && record.target == target)
        .collect()
}

fn assert_no_high_severity(records: &[Record]) {
    assert!(records.iter().all(|record| {
        record.kind != "event"
            || (record.level != tracing::Level::WARN && record.level != tracing::Level::ERROR)
    }));
}

#[test]
fn validation_control_and_one_instruction_over_limit_keep_results_and_report_refusal() {
    run_isolated(
        "tests::tracing_tests::validation_control_and_one_instruction_over_limit_keep_results_and_report_refusal",
        || {
            let target = reviewed_target();
            let push = TapscriptInstruction::Push(StackItem::new(&target, vec![1]).unwrap());
            let control = TapscriptProgram::new(vec![push.clone()]).unwrap();
            let subject = TapscriptProgram::new(vec![push.clone(), push]).unwrap();
            let initial = AbstractStackState::from_main(Vec::new());
            let limits = AbstractLimits::for_target(&target)
                .with_maximum_instructions(NonZeroU64::new(1).unwrap());
            let plain_control = validate_program(&target, &control, &initial, limits);
            let (observed_control, control_records) =
                capture(|| validate_program(&target, &control, &initial, limits));
            assert_eq!(plain_control, observed_control);
            let control_events = owned_events(&control_records, "tapscript::stack");
            assert_eq!(control_events.len(), 1);
            assert_eq!(control_events[0].level, tracing::Level::DEBUG);
            assert_eq!(control_events[0].parent, Some("validate_program"));
            assert_eq!(control_events[0].fields["outcome"], "completed");
            assert!(control_events[0].fields.contains_key("visited_state_count"));
            assert!(
                control_records
                    .iter()
                    .any(|record| record.kind == "close" && record.name == "validate_program")
            );

            let plain_subject = validate_program(&target, &subject, &initial, limits);
            let (observed_subject, subject_records) =
                capture(|| validate_program(&target, &subject, &initial, limits));
            assert_eq!(plain_subject, observed_subject);
            assert!(plain_subject.is_err());
            let refused = owned_events(&subject_records, "tapscript::stack");
            assert_eq!(refused.len(), 1);
            assert_eq!(refused[0].level, tracing::Level::DEBUG);
            assert_eq!(refused[0].fields["outcome"], "refused");
            assert_eq!(
                refused[0].fields["refusal_kind"],
                "instruction_limit_exceeded"
            );
            assert_no_high_severity(&control_records);
            assert_no_high_severity(&subject_records);
        },
    );
}

#[test]
fn operation_assessment_has_one_owned_event_and_preserves_verdicts() {
    run_isolated(
        "tests::tracing_tests::operation_assessment_has_one_owned_event_and_preserves_verdicts",
        || {
            let target = reviewed_target();
            let plan = compact_ash_plan();
            let baseline = crate::operation_assessment::assess_operation_plan(&target, &plan);
            let (observed, records) =
                capture(|| crate::operation_assessment::assess_operation_plan(&target, &plan));
            assert_eq!(baseline, observed);
            let events = owned_events(&records, "tapscript::operation_assessment");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields["outcome"], "completed");
            assert_eq!(events[0].parent, Some("assess_operation_plan"));
            assert_no_high_severity(&records);
        },
    );
}

#[test]
fn requirement_assessment_keeps_projection_and_reports_aggregate_counts() {
    run_isolated(
        "tests::tracing_tests::requirement_assessment_keeps_projection_and_reports_aggregate_counts",
        || {
            use architecture::{ARCHITECTURE, OperationId};
            use compiler::input::{
                AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input,
            };
            use compiler::operation_plan::PlacementSearchLimits;
            use compiler::target::analyze_target_requirements;
            use realization::{RealizationScope, derive};

            let limit = |value: u64| NonZeroU64::new(value).unwrap();
            let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();
            let scope = CompilationScope::from_operations([OperationId::CompactAsh]).unwrap();
            let policy =
                AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
            let input = bind_input(&ARCHITECTURE, realization, scope, policy).unwrap();
            let requirements = analyze_target_requirements(
                &input,
                PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
            )
            .unwrap();
            let target = reviewed_target();
            let baseline = crate::capability::assess_requirements(&target, &requirements);
            let (observed, records) =
                capture(|| crate::capability::assess_requirements(&target, &requirements));
            assert_eq!(baseline, observed);
            let events = owned_events(&records, "tapscript::capability");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields["outcome"], "completed");
            assert!(events[0].fields.contains_key("supported_count"));
            assert!(events[0].fields.contains_key("unsupported_count"));
            assert_no_high_severity(&records);
        },
    );
}

#[test]
fn compact_bundle_reports_one_event_and_one_preference_refusal() {
    run_isolated(
        "tests::tracing_tests::compact_bundle_reports_one_event_and_one_preference_refusal",
        || {
            use crate::bundle::emit_candidate_bundle;
            use crate::policy::{CompactAshBackendPolicy, demonstration_policy};

            let target = reviewed_target();
            let plan = compact_ash_plan();
            let policy = demonstration_policy();
            let baseline =
                emit_candidate_bundle(&target, &plan, policy.clone(), pattern_symbols(&target));
            let (observed, records) = capture(|| {
                emit_candidate_bundle(&target, &plan, policy.clone(), pattern_symbols(&target))
            });
            assert_eq!(baseline, observed);
            assert!(baseline.is_ok());
            let events = owned_events(&records, "tapscript::bundle");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields["outcome"], "completed");
            assert!(events[0].fields.contains_key("script_bytes"));
            assert_eq!(
                records
                    .iter()
                    .filter(
                        |record| record.kind == "event" && record.target.starts_with("tapscript::")
                    )
                    .count(),
                1
            );

            let mut preference = policy.pattern_preference().to_vec();
            let _ = preference.pop();
            let subject = CompactAshBackendPolicy::new(
                policy.projection().clone(),
                policy.representation(),
                policy.cardinality().clone(),
                policy.layout(),
                preference,
                policy.objective(),
                policy.tie_break(),
            );
            let plain_refusal =
                emit_candidate_bundle(&target, &plan, subject.clone(), pattern_symbols(&target));
            let (observed_refusal, refused_records) = capture(|| {
                emit_candidate_bundle(&target, &plan, subject, pattern_symbols(&target))
            });
            assert_eq!(plain_refusal, observed_refusal);
            assert!(plain_refusal.is_err());
            let refused = owned_events(&refused_records, "tapscript::bundle");
            assert_eq!(refused.len(), 1);
            assert_eq!(refused[0].level, tracing::Level::DEBUG);
            assert_eq!(refused[0].fields["outcome"], "refused");
            assert_eq!(
                refused[0].fields["refusal_kind"],
                "pattern_selection_mismatch"
            );
            assert_no_high_severity(&records);
            assert_no_high_severity(&refused_records);
        },
    );
}

#[test]
fn live_constructor_reports_one_event_and_one_missing_leaf_refusal() {
    run_isolated(
        "tests::tracing_tests::live_constructor_reports_one_event_and_one_missing_leaf_refusal",
        || {
            use crate::authorization::owner_key_encoding_closure;
            use crate::live_constructor::{
                OwnerKey, derive_live_receipt_constructor, static_transfer_leaf_set,
            };
            use crate::live_shape::demonstration_live_shape_set;

            let target = reviewed_target();
            let plan = live_transfer_plan();
            let representation = LiveTransferRepresentationPlan::Explicit;
            let shapes = demonstration_live_shape_set();
            let leaves = static_transfer_leaf_set(representation, &shapes);
            let closure = owner_key_encoding_closure(target.definition().authorization());
            let owner = OwnerKey::new(&closure, closure.approved(), vec![0x11; 32]).unwrap();
            let baseline = derive_live_receipt_constructor(
                &target,
                &plan,
                representation,
                owner.clone(),
                shapes.clone(),
                leaves.clone(),
            );
            let (observed, records) = capture(|| {
                derive_live_receipt_constructor(
                    &target,
                    &plan,
                    representation,
                    owner.clone(),
                    shapes.clone(),
                    leaves.clone(),
                )
            });
            assert_eq!(baseline, observed);
            assert!(baseline.is_ok());
            let events = owned_events(&records, "tapscript::live_constructor");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields["outcome"], "completed");

            let mut missing_one = leaves;
            let _ = missing_one.pop_first();
            let plain_refusal = derive_live_receipt_constructor(
                &target,
                &plan,
                representation,
                owner.clone(),
                shapes.clone(),
                missing_one.clone(),
            );
            let (observed_refusal, refused_records) = capture(|| {
                derive_live_receipt_constructor(
                    &target,
                    &plan,
                    representation,
                    owner,
                    shapes,
                    missing_one,
                )
            });
            assert_eq!(plain_refusal, observed_refusal);
            assert!(plain_refusal.is_err());
            let refused = owned_events(&refused_records, "tapscript::live_constructor");
            assert_eq!(refused.len(), 1);
            assert_eq!(refused[0].level, tracing::Level::DEBUG);
            assert_eq!(refused[0].fields["refusal_kind"], "leaf_set_incomplete");
            assert_no_high_severity(&records);
            assert_no_high_severity(&refused_records);
        },
    );
}

#[test]
fn live_bundle_preserves_candidate_bytes_and_reports_one_owned_event() {
    run_isolated(
        "tests::tracing_tests::live_bundle_preserves_candidate_bytes_and_reports_one_owned_event",
        || {
            use crate::authorization::owner_key_encoding_closure;
            use crate::live_bundle::emit_candidate_live_bundle;
            use crate::live_constructor::{
                OwnerKey, derive_live_receipt_constructor, static_transfer_leaf_set,
            };
            use crate::live_shape::demonstration_live_shape_set;

            let target = reviewed_target();
            let plan = live_transfer_plan();
            let representation = LiveTransferRepresentationPlan::Explicit;
            let shapes = demonstration_live_shape_set();
            let leaves = static_transfer_leaf_set(representation, &shapes);
            let closure = owner_key_encoding_closure(target.definition().authorization());
            let owner = OwnerKey::new(&closure, closure.approved(), vec![0x11; 32]).unwrap();
            let constructor = derive_live_receipt_constructor(
                &target,
                &plan,
                representation,
                owner,
                shapes,
                leaves,
            )
            .unwrap();
            let baseline = emit_candidate_live_bundle(
                &target,
                &plan,
                &constructor,
                live_transfer_symbols(&target),
            );
            let (observed, records) = capture(|| {
                emit_candidate_live_bundle(
                    &target,
                    &plan,
                    &constructor,
                    live_transfer_symbols(&target),
                )
            });
            assert_eq!(baseline, observed);
            assert!(baseline.is_ok());
            let events = owned_events(&records, "tapscript::live_bundle");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields["outcome"], "completed");
            assert!(events[0].fields.contains_key("script_bytes"));
            assert_eq!(
                records
                    .iter()
                    .filter(
                        |record| record.kind == "event" && record.target.starts_with("tapscript::")
                    )
                    .count(),
                1
            );
            assert_no_high_severity(&records);
        },
    );
}

#[test]
fn keyed_resource_entries_report_one_owned_completion() {
    run_isolated(
        "tests::tracing_tests::keyed_resource_entries_report_one_owned_completion",
        || {
            use crate::keyed_resources::{
                KeyedSchedule, keyed_resource_projection, keyed_transaction_weight,
            };
            let target = reviewed_target();
            for schedule in [
                KeyedSchedule::Birth,
                KeyedSchedule::Continuation,
                KeyedSchedule::Leaf,
            ] {
                let baseline = keyed_resource_projection(&target, schedule);
                let (observed, records) = capture(|| keyed_resource_projection(&target, schedule));
                assert_eq!(observed, baseline);
                assert!(observed.is_ok());
                assert_eq!(
                    records
                        .iter()
                        .filter(|r| r.kind == "event" && r.target.starts_with("tapscript::"))
                        .count(),
                    1
                );
                let events = owned_events(&records, "tapscript::keyed_resources");
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].fields["outcome"], "completed");
                assert_no_high_severity(&records);
            }
            for base_bytes in [100, u64::MAX] {
                let baseline = keyed_transaction_weight(&target, 2, base_bytes, 7);
                let (observed, records) =
                    capture(|| keyed_transaction_weight(&target, 2, base_bytes, 7));
                assert_eq!(observed, baseline);
                assert_eq!(
                    records
                        .iter()
                        .filter(|r| r.kind == "event" && r.target.starts_with("tapscript::"))
                        .count(),
                    1
                );
                let events = owned_events(&records, "tapscript::keyed_resources");
                assert_eq!(events.len(), 1);
                assert_eq!(
                    events[0].fields["outcome"],
                    if observed.is_ok() {
                        "completed"
                    } else {
                        "refused"
                    }
                );
                assert_no_high_severity(&records);
            }
        },
    );
}

#[test]
fn keyed_bundle_and_direct_material_each_report_one_owned_completion() {
    run_isolated(
        "tests::tracing_tests::keyed_bundle_and_direct_material_each_report_one_owned_completion",
        || {
            use crate::keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle};
            use crate::keyed_constructor::KeyedLeafRecipe;
            use architecture::{
                keyed::KeyedManifest,
                semantic::{KeyedOperationId, SemanticOperationId},
            };
            use compiler::{
                AnalysisPolicy, ProofSearchLimits, bind_input,
                keyed_plan::{PlacementSearchLimits, plan},
            };
            let target = reviewed_target();
            let manifest = KeyedManifest::contract().unwrap();
            let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
                .unwrap()
                .validate(&manifest)
                .unwrap();
            let limit = NonZeroU64::new(1_000_000).unwrap();
            let input = bind_input(
                &manifest,
                source,
                vec![
                    SemanticOperationId::Keyed(KeyedOperationId::Source),
                    SemanticOperationId::Keyed(KeyedOperationId::Merge),
                ],
                AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
            )
            .unwrap();
            let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
            let selection = KeyedBundleSelection {
                binding: plan.project().binding.clone(),
                scope: plan.project().scope.clone(),
                proof: 0,
                placements: plan.project().plans[0]
                    .operations
                    .keys()
                    .map(|id| (*id, 0))
                    .collect(),
            };
            let recipe = KeyedLeafRecipe::new([42; 32]);
            let baseline = emit_keyed_bundle(&target, &plan, selection.clone(), recipe);
            let (observed, records) =
                capture(|| emit_keyed_bundle(&target, &plan, selection, recipe));
            assert_eq!(observed, baseline);
            assert!(observed.is_ok());
            let events = owned_events(&records, "tapscript::keyed_bundle");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields["outcome"], "completed");
            assert_eq!(
                records
                    .iter()
                    .filter(|r| r.kind == "event" && r.target.starts_with("tapscript::"))
                    .count(),
                1
            );
            let baseline = recipe.construct(&target);
            let (observed, records) = capture(|| recipe.construct(&target));
            assert_eq!(observed, baseline);
            assert!(observed.is_ok());
            let events = owned_events(&records, "tapscript::keyed_constructor");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].level, tracing::Level::DEBUG);
            assert_eq!(events[0].parent, Some("construct_keyed_material"));
            assert_eq!(events[0].fields["outcome"], "completed");
            assert_eq!(
                records
                    .iter()
                    .filter(|r| r.kind == "event" && r.target.starts_with("tapscript::"))
                    .count(),
                1
            );
            assert_no_high_severity(&records);
        },
    );
}

#[test]
fn keyed_assessment_and_member_binding_report_one_owned_completion() {
    run_isolated(
        "tests::tracing_tests::keyed_assessment_and_member_binding_report_one_owned_completion",
        || {
            use crate::keyed_assessment::assess_keyed_bundle;
            use crate::keyed_members::{KeyedMemberRefusal, bind_keyed_members};
            let target = reviewed_target();
            let (bundle, inputs, members) = crate::keyed_members::tests::context();
            let baseline = assess_keyed_bundle(&target, &bundle);
            let (observed, records) = capture(|| assess_keyed_bundle(&target, &bundle));
            assert_eq!(observed, baseline);
            assert!(observed.is_ok());
            assert_eq!(
                records
                    .iter()
                    .filter(|r| r.kind == "event" && r.target.starts_with("tapscript::"))
                    .count(),
                1
            );
            let events = owned_events(&records, "tapscript::keyed_assessment");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields["outcome"], "completed");
            assert!(!events[0].fields.contains_key("refusal_kind"));
            assert_no_high_severity(&records);
            for (offer, expected) in [
                (&members[..], None),
                (&members[..1], Some(KeyedMemberRefusal::MissingMember(2))),
            ] {
                let baseline = bind_keyed_members(&bundle, &inputs, offer);
                let (observed, records) = capture(|| bind_keyed_members(&bundle, &inputs, offer));
                assert_eq!(observed, baseline);
                assert_eq!(
                    records
                        .iter()
                        .filter(|r| r.kind == "event" && r.target.starts_with("tapscript::"))
                        .count(),
                    1
                );
                let events = owned_events(&records, "tapscript::keyed_members");
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].level, tracing::Level::DEBUG);
                assert_eq!(events[0].parent, Some("bind_keyed_members"));
                if let Some(error) = expected {
                    assert_eq!(observed, Err(error));
                    assert_eq!(events[0].fields["outcome"], "refused");
                    assert_eq!(events[0].fields["refusal_kind"], "missing_member");
                } else {
                    assert!(observed.is_ok());
                    assert_eq!(events[0].fields["outcome"], "completed");
                    assert!(!events[0].fields.contains_key("refusal_kind"));
                }
                assert_no_high_severity(&records);
            }
        },
    );
}

#[test]
fn keyed_capability_core_preserves_degraded_statuses() {
    use crate::keyed_assessment::{KeyedAssessmentRefusal, assess_capabilities};
    use target_elements::{ElementsCapability as C, StaticCapabilityStatus as S};
    let target = reviewed_target();
    let (bundle, _, _) = crate::keyed_members::tests::context();
    let program = bundle.project().successor.program();
    assert!(assess_capabilities(target.validated(), program).is_ok());
    for status in [S::Incomplete, S::Unsupported] {
        let subject = super::target_with_status(C::InputOutpointInspection, status);
        assert_eq!(
            assess_capabilities(&subject, program),
            Err(KeyedAssessmentRefusal::Capability {
                capability: C::InputOutpointInspection,
                status: Some(status)
            })
        );
    }
}
