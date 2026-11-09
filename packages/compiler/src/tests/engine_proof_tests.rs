//! Both vocabularies use the exact finite proof search, including evidence-only SOURCE.
use crate::{
    capability::CapabilityView,
    engine_proof as core,
    engine_records::{self, RelationOwner},
};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};

fn bound(scope: Vec<KeyedOperationId>) -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::probe().expect("binding");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("declaration")
        .validate(&manifest)
        .expect("owner validation");
    crate::bind_input(
        &manifest,
        source,
        scope.into_iter().map(SemanticOperationId::Keyed).collect(),
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("binding")
}

#[test]
fn fixed_source_refusal_restores_the_attestation_relation() {
    let input = super::announcement_input();
    let lowered = crate::engine_attestation::proof::lower(&input).expect("owner lowering");
    let limits = input.policy().proof_search_limits;
    assert!(
        core::enumerate(
            &lowered.records,
            &CapabilityView::Unconstrained,
            limits,
            &mut crate::proof::ProofSearchReport::default()
        )
        .is_ok()
    );
    let mut subject = lowered.records.clone();
    let alternative = subject
        .obligations
        .values_mut()
        .find_map(|obligation| match obligation {
            core::Obligation::Evidence(value) => Some(value),
            _ => None,
        })
        .expect("fixed evidence");
    let key = alternative.id.relation.clone();
    alternative.sources = Err(crate::proof::LocalProofRejection::SourceDerivation);
    let refusal = core::enumerate(
        &subject,
        &CapabilityView::Unconstrained,
        limits,
        &mut crate::proof::ProofSearchReport::default(),
    )
    .expect_err("changed source");
    assert_eq!(refusal, core::Error::FixedSource(key.clone()));
    let relation = input
        .realization()
        .relations()
        .find(|row| crate::engine_attestation::relation_key(&row.id) == key)
        .expect("restored relation")
        .id
        .clone();
    assert_eq!(
        lowered.refusal(refusal),
        crate::CompileError::FixedProofSource { relation }
    );
}

#[test]
fn shared_proof_tracing_reports_every_return_without_changing_results() {
    use super::tracing_capture::{capture, isolate_capture_test};
    use std::num::NonZeroU64;
    if isolate_capture_test(
        "tests::engine_proof_tests::shared_proof_tracing_reports_every_return_without_changing_results",
    ) {
        return;
    }
    let input = bound(vec![KeyedOperationId::Source, KeyedOperationId::Merge]);
    let control = crate::input::KeyedOwner::lower_proofs(&input).expect("records");
    let limits = input.policy().proof_search_limits;
    let mut unavailable = control.clone();
    for obligation in unavailable.obligations.values_mut() {
        if let core::Obligation::Required(alternatives) = obligation {
            alternatives.clear();
        }
    }
    let mut fixed = control.clone();
    for obligation in fixed.obligations.values_mut() {
        if let core::Obligation::Evidence(alternative) = obligation {
            alternative.constructible = false;
        }
    }
    let mut multiple = control.clone();
    for obligation in multiple.obligations.values_mut() {
        if let core::Obligation::Required(alternatives) = obligation {
            alternatives.push(alternatives[0].clone());
            break;
        }
    }
    let one = NonZeroU64::new(1).unwrap();
    for (records, limits, refusal) in [
        (&control, limits, None),
        (&unavailable, limits, Some("no_feasible_proof_plan")),
        (&fixed, limits, Some("fixed_source")),
        (
            &control,
            crate::ProofSearchLimits::new(one, limits.maximum_candidates),
            Some("proof_search_state_limit_exceeded"),
        ),
        (
            &multiple,
            crate::ProofSearchLimits::new(limits.maximum_states, one),
            Some("proof_candidate_limit_exceeded"),
        ),
    ] {
        let mut plain_report = crate::proof::ProofSearchReport::default();
        let plain = core::enumerate(
            records,
            &CapabilityView::Unconstrained,
            limits,
            &mut plain_report,
        );
        assert_eq!(plain.is_err(), refusal.is_some());
        let mut report = crate::proof::ProofSearchReport::default();
        let (observed, events) = capture(|| {
            core::enumerate(records, &CapabilityView::Unconstrained, limits, &mut report)
        });
        assert_eq!(observed, plain);
        assert_eq!(report, plain_report);
        assert_proof_trace(&events, limits, refusal, &report);
    }
}
fn assert_proof_trace(
    events: &[super::tracing_capture::CapturedRecord],
    limits: crate::ProofSearchLimits,
    refusal: Option<&str>,
    report: &crate::proof::ProofSearchReport,
) {
    use super::tracing_capture::FieldValue;
    let events = events
        .iter()
        .filter(|event| event.target == "compiler::engine_proof")
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_span);
    assert_eq!(
        events[0].fields.get("state_limit"),
        Some(&FieldValue::U64(limits.maximum_states.get()))
    );
    assert_eq!(
        events[0].fields.get("candidate_limit"),
        Some(&FieldValue::U64(limits.maximum_candidates.get()))
    );
    let event = events[1];
    assert!(!event.is_span);
    assert_eq!(
        event.fields.get("message"),
        Some(&FieldValue::Str("proof search completed".into()))
    );
    assert_eq!(
        event.fields.get("outcome"),
        Some(&FieldValue::Str(
            if refusal.is_some() {
                "refused"
            } else {
                "completed"
            }
            .into()
        ))
    );
    assert_eq!(
        event.fields.get("refusal_kind"),
        refusal.map(|kind| FieldValue::Str(kind.into())).as_ref()
    );
    for (field, value) in [
        ("visited_state_count", report.states_visited),
        ("complete_assignment_count", report.complete_assignments),
        ("feasible_assignment_count", report.feasible_assignments),
        ("capability_rejection_count", report.rejected_by_capability),
        ("source_rejection_count", report.rejected_by_source),
        (
            "constructibility_rejection_count",
            report.rejected_by_constructibility,
        ),
        (
            "representation_rejection_count",
            report.rejected_by_representation,
        ),
        ("disclosure_rejection_count", report.rejected_by_disclosure),
    ] {
        assert_eq!(event.fields.get(field), Some(&FieldValue::U64(value)));
    }
    assert!(!event.fields.contains_key("states"));
    assert!(!event.fields.contains_key("candidates"));
}

#[test]
fn keyed_proof_search_retains_source_evidence_without_a_runtime_variable() {
    for (scope, proofs, sources, evidence, states) in [
        (vec![KeyedOperationId::Source], 0, 4, 4, 1),
        (vec![KeyedOperationId::Merge], 2, 5, 3, 3),
        (
            vec![KeyedOperationId::Source, KeyedOperationId::Merge],
            2,
            9,
            7,
            3,
        ),
    ] {
        let input = bound(scope);
        let (candidates, report) =
            engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
                .expect("shared search");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].proofs.len(), proofs);
        assert_eq!(candidates[0].sources.len(), sources);
        assert_eq!(candidates[0].evidence.len(), evidence);
        assert_eq!(report.states_visited, states);
        assert_eq!(report.complete_assignments, 1);
        assert_eq!(report.feasible_assignments, 1);
        assert_eq!(candidates[0].representations.len(), 0);
    }
    let input = super::bound_input(&[OperationId::CompactAsh]);
    let (shared, report) =
        engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
            .expect("attestation records");
    let old = crate::proof::enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("compatibility");
    assert_eq!(shared.len(), old.candidates.len());
    assert_eq!(report, old.search);
}

#[test]
fn required_alternatives_and_fixed_evidence_cannot_be_dropped_by_search() {
    let input = bound(vec![KeyedOperationId::Source, KeyedOperationId::Merge]);
    let control = crate::input::KeyedOwner::lower_proofs(&input).expect("records");
    let limits = input.policy().proof_search_limits;
    assert!(
        core::enumerate(
            &control,
            &CapabilityView::Unconstrained,
            limits,
            &mut crate::proof::ProofSearchReport::default()
        )
        .is_ok()
    );
    let mut unavailable = control.clone();
    let (relation, alternatives) = unavailable
        .obligations
        .iter_mut()
        .find_map(|(key, value)| match value {
            core::Obligation::Required(alternatives) => Some((key.clone(), alternatives)),
            _ => None,
        })
        .expect("clause");
    alternatives[0].constructible = false;
    assert_eq!(
        core::enumerate(
            &unavailable,
            &CapabilityView::Unconstrained,
            limits,
            &mut crate::proof::ProofSearchReport::default()
        )
        .err(),
        Some(core::Error::NoFeasiblePlan(vec![relation]))
    );
    let mut evidence = control;
    let (relation, alternative) = evidence
        .obligations
        .iter_mut()
        .find_map(|(key, value)| match value {
            core::Obligation::Evidence(alternative) => Some((key.clone(), alternative)),
            _ => None,
        })
        .expect("evidence");
    alternative.constructible = false;
    assert_eq!(
        core::enumerate(
            &evidence,
            &CapabilityView::Unconstrained,
            limits,
            &mut crate::proof::ProofSearchReport::default()
        )
        .err(),
        Some(core::Error::FixedSource(relation))
    );
}
