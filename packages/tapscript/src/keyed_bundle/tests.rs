//! Synthetic copy defects stay behind the sealed public plan boundary.
use super::*;
use architecture::{keyed::KeyedManifest, semantic::KeyedOperationId as K};
use compiler::keyed_plan::{PlacementSearchLimits, plan};
use compiler::{AnalysisPolicy, ProofSearchLimits, bind_input};

fn fixture() -> KeyedPlan {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let states = std::num::NonZeroU64::new(1_000_000).expect("positive");
    let candidates = std::num::NonZeroU64::new(100_000).expect("positive");
    let input = bind_input(
        &manifest,
        source,
        vec![
            SemanticOperationId::Keyed(K::Source),
            SemanticOperationId::Keyed(K::Merge),
        ],
        AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
    )
    .expect("bind");
    plan(&input, PlacementSearchLimits::new(states, candidates)).expect("plan")
}

fn rows(proof: &ProofPlan) -> Vec<(KeyedRelationId, RelationPlan)> {
    proof
        .operations
        .values()
        .flat_map(|o| o.relations.iter().map(|(id, row)| (*id, row.clone())))
        .collect()
}

#[test]
fn copied_relation_census_refuses_missing_duplicate_and_removed_continuation_row() {
    let plan = fixture();
    let proof = &plan.project().plans[0];
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let control = rows(proof);
    assert_eq!(
        lower_rows(&target, proof, &control)
            .expect("accepted")
            .len(),
        2
    );
    let mut missing = control.clone();
    let (id, _) = missing.remove(0);
    assert_eq!(
        lower_rows(&target, proof, &missing),
        Err(KeyedBundleRefusal::MissingRelation(id))
    );
    let mut duplicate = control.clone();
    duplicate.push(control[0].clone());
    assert_eq!(
        lower_rows(&target, proof, &duplicate),
        Err(KeyedBundleRefusal::DuplicateRelation(control[0].0))
    );
    let mut omitted = control;
    let position = omitted
        .iter()
        .position(|(_, row)| row.declaration == KeyedRelation::ContinueFirst)
        .expect("clause");
    let (id, _) = omitted.remove(position);
    assert_eq!(
        lower_rows(&target, proof, &omitted),
        Err(KeyedBundleRefusal::MissingRelation(id))
    );
}

#[test]
fn retained_census_and_fragment_census_refuse_changed_artifacts() {
    let plan = fixture();
    let proof = &plan.project().plans[0];
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let control = lower_selected(&target, proof, proof).expect("control");
    let mut source = proof.clone();
    source
        .operations
        .remove(&SemanticOperationId::Keyed(K::Merge));
    assert!(lower_selected(&target, &source, &source).is_ok());
    let mut unexpected = source.clone();
    let merge = proof.operations[&SemanticOperationId::Keyed(K::Merge)]
        .relations
        .iter()
        .next()
        .expect("merge row");
    unexpected
        .operations
        .get_mut(&SemanticOperationId::Keyed(K::Source))
        .expect("source")
        .relations
        .insert(*merge.0, merge.1.clone());
    assert_eq!(
        lower_selected(&target, &source, &unexpected),
        Err(KeyedBundleRefusal::UnexpectedRelation(*merge.0))
    );
    let mut missing = proof.clone();
    let id = *missing.operations[&SemanticOperationId::Keyed(K::Merge)]
        .relations
        .keys()
        .next()
        .expect("row");
    missing
        .operations
        .get_mut(&SemanticOperationId::Keyed(K::Merge))
        .expect("merge")
        .relations
        .remove(&id);
    assert_eq!(
        lower_selected(&target, proof, &missing),
        Err(KeyedBundleRefusal::MissingRelation(id))
    );
    assert_eq!(validate_fragment_census(proof, &control), Ok(()));
    let mut fragments = control.clone();
    let runtime = *fragments.keys().next().expect("runtime");
    fragments.remove(&runtime);
    assert_eq!(
        validate_fragment_census(proof, &fragments),
        Err(KeyedBundleRefusal::MissingRelation(runtime))
    );
    let mut fragments = control.clone();
    let evidence = *source
        .operations
        .values()
        .next()
        .expect("source")
        .relations
        .keys()
        .next()
        .expect("evidence");
    fragments.insert(evidence, control.values().next().expect("fragment").clone());
    assert_eq!(
        validate_fragment_census(proof, &fragments),
        Err(KeyedBundleRefusal::UnexpectedRelation(evidence))
    );
}

#[test]
fn placement_restriction_selects_the_second_candidate_and_retains_other_metadata() {
    let plan = fixture();
    let mut proof = plan.project().plans[0].clone();
    let merge = SemanticOperationId::Keyed(K::Merge);
    let operation = proof.operations.get_mut(&merge).expect("merge");
    let first = operation.placements[0].clone();
    let mut second = first.clone();
    second.pop_first(); // Synthetic candidate identity, not a new compiler plan.
    assert_ne!(first, second);
    operation.placements.push(second.clone());
    let mut choices = proof
        .operations
        .keys()
        .map(|id| (*id, 0))
        .collect::<BTreeMap<_, _>>();
    let control = restrict_placements(&proof, &choices).expect("first");
    assert_eq!(control.operations[&merge].placements, vec![first]);
    choices.insert(merge, 1);
    let selected = restrict_placements(&proof, &choices).expect("second");
    let mut expected = proof;
    expected
        .operations
        .get_mut(&merge)
        .expect("merge")
        .placements = vec![second];
    assert_eq!(selected, expected);
}

#[test]
fn equal_published_and_retained_rows_still_refuse_unsupported_shapes() {
    let plan = fixture();
    let control = &plan.project().plans[0];
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    for declaration in [KeyedRelation::BornFirst, KeyedRelation::ContinueFirst] {
        for (active, runtime) in [
            (
                false,
                vec![RuntimeClause {
                    side: TransactionSide::Input,
                    multiplicity: CarrierMultiplicity::EveryMember,
                }],
            ),
            (
                true,
                vec![RuntimeClause {
                    side: TransactionSide::Output,
                    multiplicity: CarrierMultiplicity::EveryMember,
                }],
            ),
            (
                true,
                vec![RuntimeClause {
                    side: TransactionSide::Input,
                    multiplicity: CarrierMultiplicity::ExactlyOne,
                }],
            ),
            (
                true,
                vec![RuntimeClause {
                    side: TransactionSide::Input,
                    multiplicity: CarrierMultiplicity::AtLeastOne,
                }],
            ),
            (true, vec![]),
        ] {
            assert!(lower_selected(&target, control, control).is_ok());
            let mut subject = control.clone();
            let (id, row) = subject
                .operations
                .values_mut()
                .flat_map(|o| &mut o.relations)
                .find(|(_, r)| r.declaration == declaration)
                .expect("clause");
            let id = *id;
            row.active = active;
            row.runtime = runtime;
            assert_eq!(
                lower_selected(&target, &subject, &subject),
                Err(KeyedBundleRefusal::RuntimeClauseMismatch(id))
            );
        }
    }
    for (id, row) in control
        .operations
        .values()
        .flat_map(|o| &o.relations)
        .filter(|(_, r)| r.runtime.is_empty())
    {
        assert!(matches!(
            row.declaration,
            KeyedRelation::HandleFamily { .. }
                | KeyedRelation::SourceClassification
                | KeyedRelation::MembershipReading { .. }
        ));
        assert!(lower_selected(&target, control, control).is_ok());
        let mut subject = control.clone();
        subject
            .operations
            .get_mut(&id.operation)
            .expect("operation")
            .relations
            .get_mut(id)
            .expect("row")
            .runtime
            .push(RuntimeClause {
                side: TransactionSide::Input,
                multiplicity: CarrierMultiplicity::EveryMember,
            });
        assert_eq!(
            lower_selected(&target, &subject, &subject),
            Err(KeyedBundleRefusal::EvidenceRuntimeCarrier(*id))
        );
    }
}

#[test]
fn copied_runtime_rows_refuse_reader_output_weaker_and_empty_clauses() {
    let plan = fixture();
    let proof = &plan.project().plans[0];
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let control = rows(proof);
    assert!(lower_rows(&target, proof, &control).is_ok());
    let mut reader = control.clone();
    let (id, row) = reader
        .iter_mut()
        .find(|(_, r)| matches!(r.declaration, KeyedRelation::MembershipReading { .. }))
        .expect("reader");
    row.runtime.push(RuntimeClause {
        side: TransactionSide::Input,
        multiplicity: CarrierMultiplicity::EveryMember,
    });
    let id = *id;
    assert_eq!(
        lower_rows(&target, proof, &reader),
        Err(KeyedBundleRefusal::EvidenceRuntimeCarrier(id))
    );
    for runtime in [
        vec![RuntimeClause {
            side: TransactionSide::Output,
            multiplicity: CarrierMultiplicity::EveryMember,
        }],
        vec![RuntimeClause {
            side: TransactionSide::Input,
            multiplicity: CarrierMultiplicity::ExactlyOne,
        }],
        vec![],
    ] {
        assert!(lower_rows(&target, proof, &control).is_ok());
        let mut subject = control.clone();
        let (id, row) = subject
            .iter_mut()
            .find(|(_, r)| r.declaration == KeyedRelation::BornFirst)
            .expect("birth");
        row.runtime = runtime;
        let id = *id;
        assert_eq!(
            lower_rows(&target, proof, &subject),
            Err(KeyedBundleRefusal::RuntimeClauseMismatch(id))
        );
    }
}

#[test]
fn copied_coverage_is_compared_in_full() {
    let plan = fixture();
    let proof = &plan.project().plans[0];
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let mut subject = rows(proof);
    assert!(lower_rows(&target, proof, &subject).is_ok());
    let id = subject[0].0;
    subject[0].1.positive.clear();
    assert_eq!(
        lower_rows(&target, proof, &subject),
        Err(KeyedBundleRefusal::RelationMismatch(id))
    );
}
