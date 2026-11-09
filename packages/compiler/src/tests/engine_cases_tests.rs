//! Candidate selection never calls the semantic owner back.
use crate::{capability::CapabilityView, engine_cases as core, engine_records::RelationOwner};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};

fn keyed() -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::probe().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("validation");
    crate::bind_input(
        &manifest,
        source,
        vec![
            SemanticOperationId::Keyed(KeyedOperationId::Source),
            SemanticOperationId::Keyed(KeyedOperationId::Merge),
        ],
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("binding")
}
#[test]
fn keyed_case_tables_select_all_evidence_and_runtime_rows() {
    let input = keyed();
    let tables = crate::input::KeyedOwner::lower_cases(&input).expect("tables before search");
    assert_eq!(tables.cases.len(), 2);
    assert_eq!(tables.rows.len(), 9);
    let (proofs, _) =
        crate::engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
            .expect("proof");
    let selected = core::select(&tables, &proofs[0]).expect("select");
    assert_eq!(selected.placement.plans.len(), 9);
    assert_eq!(selected.placement.eligibility.len(), 2);
    assert_eq!(selected.coverage.len(), 9);
    assert_eq!(selected.facts.len(), 9);
    assert_eq!(
        selected
            .facts
            .iter()
            .map(|facts| facts.sources.len())
            .sum::<usize>(),
        9
    );
    assert_eq!(
        selected
            .facts
            .iter()
            .map(|facts| facts.runtime.len())
            .sum::<usize>(),
        2
    );
    let mut mismatched = tables.clone();
    mismatched.rows[0]
        .facts
        .as_mut()
        .expect("case facts")
        .case_operation
        .namespace = 1;
    assert_eq!(
        core::select(&mismatched, &proofs[0]).err(),
        Some(core::Error::CaseFactsMismatch(
            mismatched.rows[0].key.clone()
        ))
    );
    let required = selected
        .placement
        .plans
        .iter()
        .map(|(key, _)| key.clone())
        .collect();
    assert!(crate::engine_coverage::validate_census(&required, &selected.coverage).is_ok());
    let mut removed = tables.clone();
    let row = removed.rows.remove(0);
    assert_eq!(
        core::select(&removed, &proofs[0]).err(),
        Some(core::Error::CaseCensusMismatch {
            missing: vec![row.key],
            unexpected: Vec::new()
        })
    );
    let mut duplicate = tables;
    let row = duplicate.rows[0].clone();
    let key = row.key.clone();
    duplicate.rows.push(row);
    assert_eq!(
        core::select(&duplicate, &proofs[0]).err(),
        Some(core::Error::DuplicateCase(key))
    );
}
#[test]
fn attestation_tables_match_dynamic_case_discharge_for_each_representation() {
    let input = super::bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let tables = crate::input::AttestationOwner::lower_cases(&input).expect("tables before search");
    assert_eq!(tables.cases.len(), 8);
    let (proofs, _) =
        crate::engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
            .expect("proofs");
    let lowered = crate::engine_attestation::proof::lower(&input).expect("compatibility");
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let mut seen = std::collections::BTreeSet::new();
    for proof in proofs {
        if !seen.insert(proof.representations.clone()) {
            continue;
        }
        let selected = core::select(&tables, &proof).expect("table selection");
        let candidate = lowered.candidate(proof);
        let cases =
            crate::case::execution_cases(&relations, &candidate).expect("compatibility cases");
        let plans = crate::placement::classify_relation_cases(&relations, &cases)
            .expect("compatibility discharge");
        let eligible = crate::carrier::relation_case_eligibility(&relations, &plans)
            .expect("compatibility eligibility");
        compare_case_facts(&relations, &cases, &plans, &eligible, &selected.facts);
        let expected = crate::engine_attestation::placement::lower(&plans, &eligible, None);
        assert_eq!(
            selected
                .placement
                .plans
                .iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            expected.records.plans.into_iter().collect()
        );
        assert_eq!(
            selected.placement.eligibility.len(),
            expected.records.eligibility.len()
        );
        let direct = crate::coverage::analyze_plan_coverage(&relations, &plans)
            .expect("compatibility coverage");
        let rows = direct
            .plans()
            .map(crate::engine_attestation::coverage::lowered)
            .map(|plan| (plan.key.clone(), plan))
            .collect::<std::collections::BTreeMap<_, _>>();
        for row in selected.coverage {
            assert_eq!(row, rows[&row.key]);
        }
    }
    assert_eq!(seen.len(), 4);
}

fn compare_case_facts(
    relations: &crate::relation::CompilerRelationAnalysis,
    cases: &[crate::case::ExecutionCase],
    plans: &[crate::placement::RelationCasePlan],
    eligibility: &[crate::carrier::CarrierEligibility],
    facts: &[crate::engine_case_facts::Facts],
) {
    for plan in plans {
        let execution = cases
            .iter()
            .find(|case| case.id == plan.case)
            .expect("case");
        let sources = execution
            .active_sources
            .iter()
            .filter(|row| row.operand.relation() == &plan.relation)
            .cloned()
            .collect();
        let eligible = eligibility
            .iter()
            .filter(|row| row.relation == plan.relation && row.case == plan.case)
            .cloned()
            .collect::<Vec<_>>();
        let layout =
            crate::layout::layout_requirements(relations, std::slice::from_ref(plan), &eligible)
                .expect("layout");
        let expected = crate::engine_attestation::case_facts::lower(plan, &sources, &layout);
        let actual = facts
            .iter()
            .find(|facts| facts.key == expected.key)
            .expect("retained case");
        assert_eq!(actual, &expected);
    }
}
