//! Paired corruptions of production plans at symbolic assignment boundaries.
use super::{
    Prepared,
    tests::{bound, limits},
};
use crate::{
    engine_placement as p,
    keyed_plan::{self, KeyedPlanError},
};
use architecture::semantic::KeyedOperationId as K;

#[test]
fn production_assignment_candidates_refuse_missing_duplicate_unreachable_and_mismatch() {
    let input = bound(&[K::Merge]);
    let prepared = Prepared::new(&input).unwrap();
    let program = prepared.analyze(limits()).unwrap();
    let published = keyed_plan::plan(&input, limits()).unwrap();
    assert!(keyed_plan::revalidate(&input, limits(), &published).is_ok());
    let records = super::super::lower_placements(input.owner());
    let accepted = &program
        .plans
        .values()
        .next()
        .unwrap()
        .operations
        .values()
        .next()
        .unwrap()
        .placements[0];
    assert_eq!(p::validate_placement(&records, accepted), Ok(()));
    let mut missing = accepted.clone();
    let removed = missing.assignments.remove(0);
    assert_eq!(
        p::validate_placement(&records, &missing),
        Err(p::Error::PlacementCensusMismatch {
            missing: vec![removed.key],
            unexpected: Vec::new()
        })
    );
    let mut duplicate = accepted.clone();
    duplicate.assignments.push(duplicate.assignments[0].clone());
    assert_eq!(
        p::validate_placement(&records, &duplicate),
        Err(p::Error::PlacementCensusMismatch {
            missing: Vec::new(),
            unexpected: Vec::new()
        })
    );
    let mut mismatch = accepted.clone();
    mismatch.assignments[0].carriers[0].quantification =
        crate::carrier::CarrierQuantification::Single;
    let assignment = &mismatch.assignments[0];
    assert_eq!(
        p::validate_placement(&records, &mismatch),
        Err(p::Error::UnpermittedCarrierPlacement {
            key: assignment.key.clone(),
            carrier: assignment.carriers[0].carrier.clone()
        })
    );
    let mut unreachable = records.clone();
    let key = &accepted.assignments[0].key;
    unreachable
        .eligibility
        .iter_mut()
        .find(|row| row.key == *key)
        .unwrap()
        .entries
        .clear();
    assert_eq!(
        p::validate_placement(&unreachable, accepted),
        Err(p::Error::UnpermittedCarrierPlacement {
            key: key.clone(),
            carrier: accepted.assignments[0].carriers[0].carrier.clone()
        })
    );
    let mut repeated = accepted.clone();
    let carrier = repeated.assignments[0].carriers[0].clone();
    repeated.assignments[0].carriers.push(carrier);
    assert_eq!(
        p::validate_placement(&records, &repeated),
        Err(p::Error::DuplicatePlacedCarrier {
            key: key.clone(),
            carrier: accepted.assignments[0].carriers[0].carrier.clone()
        })
    );
}

#[test]
fn keyed_revalidation_refuses_weaker_output_and_missing_runtime_clauses() {
    let input = bound(&[K::Merge]);
    let control = keyed_plan::plan(&input, limits()).unwrap();
    assert_eq!(keyed_plan::revalidate(&input, limits(), &control), Ok(()));
    for mutation in 0..5 {
        let mut subject = control.clone();
        let operation = subject.projection.plans[0]
            .operations
            .values_mut()
            .next()
            .unwrap();
        let clause = operation
            .relations
            .values_mut()
            .find(|row| !row.runtime.is_empty())
            .unwrap();
        match mutation {
            0 => clause.runtime[0].multiplicity = crate::placement::CarrierMultiplicity::ExactlyOne,
            1 => clause.runtime[0].multiplicity = crate::placement::CarrierMultiplicity::AtLeastOne,
            2 => clause.runtime[0].side = realization::TransactionSide::Output,
            3 => clause.runtime.clear(),
            _ => {
                operation.placements[0].pop_first();
            }
        }
        assert_eq!(
            keyed_plan::revalidate(&input, limits(), &subject),
            Err(KeyedPlanError::PublicationMismatch),
            "mutation {mutation}"
        );
    }
}

#[test]
fn keyed_revalidation_refuses_missing_evidence_without_fabricating_a_witness() {
    let input = bound(&[K::Source]);
    let control = keyed_plan::plan(&input, limits()).unwrap();
    assert_eq!(control.project().evidence.len(), 4);
    assert!(keyed_plan::revalidate(&input, limits(), &control).is_ok());
    let mut subject = control;
    subject.projection.evidence.pop_first();
    assert_eq!(
        keyed_plan::revalidate(&input, limits(), &subject),
        Err(KeyedPlanError::PublicationMismatch)
    );
}
