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
