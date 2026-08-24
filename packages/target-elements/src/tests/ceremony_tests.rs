//! The candidate ceremony vocabulary this crate carries beside its
//! reviewed facts.
//!
//! Nothing here reads a target. What is held is that the one widening
//! the crate contract admits stays closed, stays single-spelled, and
//! never acquires a default.

use std::collections::BTreeSet;

use crate::ceremony::ReproducibilityContract;

#[test]
fn the_vocabulary_is_closed_and_its_census_is_complete() {
    // The census constant and the variants are held together: a variant
    // added without being placed in `ALL` is a member no census walks.
    let census: BTreeSet<ReproducibilityContract> =
        ReproducibilityContract::ALL.into_iter().collect();
    assert_eq!(
        census.len(),
        2,
        "the vocabulary states exactly two contracts"
    );
    assert!(census.contains(&ReproducibilityContract::ByteIdentity));
    assert!(census.contains(&ReproducibilityContract::RecordedRandomness));
}

#[test]
fn every_contract_has_one_spelling() {
    // The code and the rendering are the same word for every member. A
    // second authored spelling is harmless while a reader is human and
    // is not harmless the moment anything compares the two.
    let codes: BTreeSet<&str> = ReproducibilityContract::ALL
        .iter()
        .map(|contract| contract.code())
        .collect();
    assert_eq!(codes.len(), ReproducibilityContract::ALL.len());
    for contract in ReproducibilityContract::ALL {
        assert_eq!(contract.to_string(), contract.code());
        assert_eq!(
            ReproducibilityContract::from_code(contract.code()),
            Some(contract)
        );
    }
}

#[test]
fn an_unknown_code_is_unknown_rather_than_the_reference_contract() {
    // Answering an unheld code with the reference contract would move a
    // run onto guarantees nobody selected.
    assert_eq!(ReproducibilityContract::from_code("byte-identity"), None);
    assert_eq!(ReproducibilityContract::from_code(""), None);
    assert_eq!(ReproducibilityContract::from_code("semantic"), None);
}

#[test]
fn byte_identity_is_the_reference_and_the_two_contracts_differ_where_they_should() {
    assert_eq!(
        ReproducibilityContract::REFERENCE,
        ReproducibilityContract::ByteIdentity,
    );
    // The two properties that actually distinguish the contracts, held
    // apart: which opening sources are admissible, and whether cross-run
    // byte equality may be claimed at all.
    assert!(!ReproducibilityContract::ByteIdentity.admits_run_produced_openings());
    assert!(ReproducibilityContract::RecordedRandomness.admits_run_produced_openings());
    assert!(ReproducibilityContract::ByteIdentity.claims_cross_run_byte_equality());
    assert!(!ReproducibilityContract::RecordedRandomness.claims_cross_run_byte_equality());
}
