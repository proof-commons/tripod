use super::*;
use crate::keyed_plan::{Defect, Stage};
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};

pub(super) fn bound(scope: &[K]) -> BoundCompilerInput<KeyedOwner> {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("validation");
    crate::bind_input(
        &manifest,
        source,
        scope.iter().copied().map(O::Keyed).collect(),
        crate::AnalysisPolicy::strict(crate::ProofSearchLimits::new(
            std::num::NonZeroU64::new(1_000_000).unwrap(),
            std::num::NonZeroU64::new(100_000).unwrap(),
        )),
    )
    .expect("bound")
}
pub(super) fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).unwrap(),
        std::num::NonZeroU64::new(100_000).unwrap(),
    )
}
#[test]
fn production_keyed_publication_equals_record_oracle_and_revalidates() {
    for scope in [vec![K::Source], vec![K::Merge], vec![K::Source, K::Merge]] {
        let input = bound(&scope);
        let prepared = Prepared::new(&input).expect("preparation");
        let oracle =
            crate::analyzed::analyze_bound_records(&input, limits()).expect("record oracle");
        let plan = crate::keyed_plan::plan(&input, limits()).expect("production");
        assert_eq!(
            plan.project(),
            &prepared.publish(&oracle).expect("oracle publication")
        );
        assert!(crate::keyed_plan::revalidate(&input, limits(), &plan).is_ok());
        let mut missing = plan;
        missing.projection.evidence.pop_first();
        assert_eq!(
            crate::keyed_plan::revalidate(&input, limits(), &missing),
            Err(KeyedPlanError::PublicationMismatch)
        );
    }
}
fn alternative(records: &mut core::Records) -> &mut crate::engine_proof::Alternative {
    records
        .proofs
        .obligations
        .values_mut()
        .find_map(|row| match row {
            crate::engine_proof::Obligation::Static => None,
            crate::engine_proof::Obligation::Evidence(value) => Some(value),
            crate::engine_proof::Obligation::Required(values) => values.first_mut(),
        })
        .expect("alternative")
}
#[test]
fn production_keyed_source_and_join_refusals_are_owner_typed() {
    let input = bound(&[K::Source, K::Merge]);
    let control = Prepared::new(&input).expect("control");
    assert!(control.analyze(limits()).is_ok());
    for (mutate, defect) in [
        (
            (|r: &mut core::Records| {
                r.sources.tables[0].rows.clear();
            }) as fn(&mut core::Records),
            Defect::MissingOperand,
        ),
        (
            |r| {
                r.sources.tables.push(r.sources.tables[0].clone());
            },
            Defect::DuplicateSourceTable,
        ),
        (
            |r| {
                let mut extra = r.sources.tables[0].clone();
                extra.proof_kind = u16::MAX;
                r.sources.tables.push(extra);
            },
            Defect::UnnamedSourceTable,
        ),
        (
            |r| {
                alternative(r).constructible = false;
            },
            Defect::Constructibility,
        ),
        (
            |r| {
                alternative(r).sources.as_mut().unwrap()[0]
                    .availability
                    .kind ^= 1;
            },
            Defect::SourceRows,
        ),
        (
            |r| {
                alternative(r).id.namespace = 1;
            },
            Defect::ProofSource,
        ),
    ] {
        let mut subject = Prepared::new(&input).expect("fresh");
        mutate(&mut subject.records);
        assert!(
            matches!(subject.analyze(limits()), Err(KeyedPlanError::Refused { defect: actual, relations, .. }) if actual == defect && !relations.is_empty())
        );
    }
}
#[test]
fn production_keyed_unknown_and_cyclic_dependencies_refuse() {
    use crate::engine_records::{DependencyKind, DependencyRecord};
    let input = bound(&[K::Source]);
    let mut prepared = Prepared::new(&input).expect("control");
    assert!(prepared.analyze(limits()).is_ok());
    let first = prepared.records.relations.nodes[0].id.clone();
    let second = prepared.records.relations.nodes[1].id.clone();
    prepared.records.relations.edges.push(DependencyRecord {
        source: first.clone(),
        target: second.clone(),
        kind: DependencyKind {
            namespace: 2,
            code: 1,
        },
    });
    assert!(prepared.analyze(limits()).is_ok());
    prepared.records.relations.edges.push(DependencyRecord {
        source: second,
        target: first,
        kind: DependencyKind {
            namespace: 2,
            code: 1,
        },
    });
    assert!(matches!(
        prepared.analyze(limits()),
        Err(KeyedPlanError::Refused {
            defect: Defect::Cycle,
            ..
        })
    ));
    prepared.records.relations.edges.pop();
    prepared.records.relations.edges[0]
        .target
        .subject
        .push(u16::MAX);
    assert_eq!(prepared.analyze(limits()).err(), Some(missing()));
}
#[test]
fn production_keyed_local_rejection_remains_local() {
    use crate::{engine_proof as proof, proof::LocalProofRejection};
    let input = bound(&[K::Merge]);
    let mut prepared = Prepared::new(&input).expect("control");
    let control = prepared.analyze(limits()).expect("accepted");
    let (relation, alternatives) = prepared
        .records
        .proofs
        .obligations
        .iter_mut()
        .find_map(|(key, row)| {
            if let proof::Obligation::Required(values) = row {
                Some((key.clone(), values))
            } else {
                None
            }
        })
        .expect("runtime");
    let mut rejected = alternatives[0].clone();
    rejected.id.kind = u16::MAX;
    rejected.sources = Err(LocalProofRejection::SourceDerivation);
    rejected.constructible = false;
    alternatives.push(rejected);
    let result = prepared.analyze(limits()).expect("feasible sibling");
    assert_eq!(result.plans, control.plans);
    assert_eq!(result.proof_search.rejected_by_source, 1);
    if let proof::Obligation::Required(values) = prepared
        .records
        .proofs
        .obligations
        .get_mut(&relation)
        .unwrap()
    {
        values[0].sources = Err(LocalProofRejection::SourceDerivation);
        values[0].constructible = false;
    }
    assert_eq!(
        prepared.analyze(limits()).err(),
        Some(KeyedPlanError::NoFeasiblePlan(vec![
            prepared.relation(&relation).unwrap()
        ]))
    );
}
#[test]
fn production_keyed_missing_reverse_identity_is_class_three() {
    let input = bound(&[K::Merge]);
    let mut prepared = Prepared::new(&input).unwrap();
    let program = prepared.analyze(limits()).unwrap();
    assert!(prepared.publish(&program).is_ok());
    prepared.identities.carriers.clear();
    assert_eq!(prepared.publish(&program), Err(missing()));
}

#[test]
fn production_keyed_binding_and_limits_are_revalidated_from_retained_source() {
    let source = bound(&[K::Source]);
    let merge = bound(&[K::Merge]);
    let plan = crate::keyed_plan::plan(&source, limits()).unwrap();
    assert_eq!(
        crate::keyed_plan::revalidate(&merge, limits(), &plan),
        Err(KeyedPlanError::PublicationMismatch)
    );
    let mut subject = plan;
    subject.projection.binding.schema_version ^= 1;
    assert_eq!(
        crate::keyed_plan::revalidate(&source, limits(), &subject),
        Err(KeyedPlanError::PublicationMismatch)
    );
    let tight = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1).unwrap(),
        std::num::NonZeroU64::new(1).unwrap(),
    );
    assert_eq!(
        crate::keyed_plan::plan(&merge, tight).err(),
        Some(KeyedPlanError::SearchLimit {
            stage: Stage::Placement,
            candidates: false,
            limit: 1
        })
    );
}

#[test]
fn production_keyed_fixed_source_refusal_restores_its_relation() {
    let input = bound(&[K::Source]);
    let mut subject = Prepared::new(&input).unwrap();
    assert!(subject.analyze(limits()).is_ok());
    let alternative = alternative(&mut subject.records);
    let key = alternative.id.relation.clone();
    alternative.sources = Err(crate::proof::LocalProofRejection::SourceDerivation);
    alternative.constructible = false;
    assert_eq!(
        subject.analyze(limits()).err(),
        Some(KeyedPlanError::FixedSource(subject.relation(&key).unwrap()))
    );
}
