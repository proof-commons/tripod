//! Coverage retains each typed evidence subject and independently checks census.
use crate::{capability::CapabilityView, engine_coverage as core};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};
use std::collections::BTreeSet;

pub(super) fn bound(
    scope: Vec<KeyedOperationId>,
) -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("validation");
    crate::bind_input(
        &manifest,
        source,
        scope.into_iter().map(SemanticOperationId::Keyed).collect(),
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("binding")
}
#[test]
fn source_evidence_reaches_proof_placement_and_coverage_without_execution() {
    for (scope, positive, negative, evidence) in [
        (vec![KeyedOperationId::Source], 4, 12, 4),
        (vec![KeyedOperationId::Merge], 5, 11, 3),
        (
            vec![KeyedOperationId::Source, KeyedOperationId::Merge],
            9,
            23,
            7,
        ),
    ] {
        let input = bound(scope);
        let (proof, _) =
            crate::engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
                .expect("proof");
        let placement = crate::engine_keyed::lower_placements(input.owner());
        let coverage = crate::engine_keyed::lower_coverage(input.owner());
        let planned = placement.plans.iter().map(|(key, _)| key.clone()).collect();
        assert!(core::validate_census(&planned, &coverage).is_ok());
        assert_eq!(
            coverage
                .iter()
                .map(|plan| plan.positive.len())
                .sum::<usize>(),
            positive
        );
        assert_eq!(
            coverage
                .iter()
                .map(|plan| plan.negative.len())
                .sum::<usize>(),
            negative
        );
        let covered_evidence = coverage
            .iter()
            .flat_map(|plan| plan.evidence.iter().cloned())
            .collect::<BTreeSet<_>>();
        assert_eq!(covered_evidence.len(), evidence);
        assert_eq!(covered_evidence, proof[0].evidence);
        for plan in &coverage {
            if !plan.evidence.is_empty() {
                assert!(!placement.eligibility.iter().any(|row| row.key == plan.key));
                assert_eq!(
                    plan.boundaries,
                    BTreeSet::from([crate::coverage::CoverageBoundary::ExternalEvidence])
                );
            }
        }
    }
}
#[test]
fn coverage_refuses_missing_duplicate_foreign_and_uncovered_boundary_subjects() {
    let input = bound(vec![KeyedOperationId::Source]);
    let control = crate::engine_keyed::lower_coverage(input.owner());
    let planned = crate::engine_keyed::lower_placements(input.owner())
        .plans
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    assert!(core::validate_census(&planned, &control).is_ok());
    let mut missing = control.clone();
    let row = missing.remove(0);
    assert_eq!(
        core::validate_census(&planned, &missing),
        Err(core::Error::CensusMismatch {
            missing: vec![row.key],
            unexpected: Vec::new()
        })
    );
    let mut duplicate = control.clone();
    let repeated = duplicate[0].positive[0].clone();
    duplicate[0].positive.push(repeated.clone());
    assert_eq!(
        core::validate_census(&planned, &duplicate),
        Err(core::Error::DuplicateRequirement(repeated))
    );
    let mut negative = control.clone();
    negative[0].negative.clear();
    assert_eq!(
        core::validate_census(&planned, &negative),
        Err(core::Error::MissingNegative {
            key: negative[0].key.clone(),
            boundary: crate::coverage::CoverageBoundary::ExternalEvidence
        })
    );
    let mut positive = control.clone();
    positive[0].positive.clear();
    assert_eq!(
        core::validate_census(&planned, &positive),
        Err(core::Error::MissingPositive {
            key: positive[0].key.clone(),
            boundary: crate::coverage::CoverageBoundary::ExternalEvidence
        })
    );
    let mut foreign = control;
    foreign[0].key.relation.operation.namespace = 1;
    assert!(matches!(
        core::validate_census(&planned, &foreign),
        Err(core::Error::CensusMismatch { .. })
    ));
}
