//! Reorg reprojection tests.
//!
//! Implements `´test:verification:attestation-reorg-reprojection´`.

use num_bigint::BigUint;
use num_traits::One;

use super::attestation_history_fixture::*;
use super::scenario_fixtures::ADDRESS_A;
use super::test_fixtures::{block_hash, context};
use crate::*;

#[test]
fn same_raw_burn_reprojects_across_clear_boundary() {
    let burn = accepted_burn(10, 5, 2, 100, ADDRESS_A);

    let mut first = ReferenceIndexer::empty_for_test(context(5, block_hash(5)));

    first.insert_clear_for_test(genesis_clear()).unwrap();

    let clear_before = clear_entry(1, 5, 1, 1_200, 1_000);

    first.insert_clear_for_test(clear_before).unwrap();

    first.insert_burn_for_test(burn.clone()).unwrap();

    let first_terms = first.query(ADDRESS_A).unwrap();

    let mut second = ReferenceIndexer::empty_for_test(context(5, block_hash(6)));

    second.insert_clear_for_test(genesis_clear()).unwrap();

    let clear_after = clear_entry(2, 5, 3, 1_200, 1_000);

    second.insert_clear_for_test(clear_after).unwrap();

    second.insert_burn_for_test(burn.clone()).unwrap();

    let second_terms = second.query(ADDRESS_A).unwrap();

    assert_eq!(
        burn.records,
        second.burns().get(&burn.txid).unwrap().records,
    );

    assert_ne!(
        first_terms.terms[0].clear_id,
        second_terms.terms[0].clear_id,
    );

    assert_ne!(first_terms.reduce(), second_terms.reduce());
}

#[test]
fn same_checkpoint_produces_identical_bytes() {
    let mut indexer = ReferenceIndexer::empty_for_test(context(5, block_hash(5)));

    indexer.insert_clear_for_test(genesis_clear()).unwrap();

    let clear = clear_entry(1, 5, 1, 1_200, 1_000);

    indexer.insert_clear_for_test(clear).unwrap();

    let burn = accepted_burn(10, 5, 2, 100, ADDRESS_A);

    indexer.insert_burn_for_test(burn).unwrap();

    let a = indexer.query(ADDRESS_A).unwrap();

    let b = indexer.query(ADDRESS_A).unwrap();

    assert_eq!(serialize_query(&a).unwrap(), serialize_query(&b).unwrap(),);
}

#[test]
fn genesis_clear_is_used_before_any_other_clear() {
    let burn = accepted_burn(10, 1, 0, 25, ADDRESS_A);

    let mut indexer = ReferenceIndexer::empty_for_test(context(1, block_hash(1)));

    indexer.insert_clear_for_test(genesis_clear()).unwrap();

    indexer.insert_burn_for_test(burn).unwrap();

    let query = indexer.query(ADDRESS_A).unwrap();

    assert_eq!(query.terms.len(), 1);

    assert!(matches!(query.terms[0].clear_id, ClearId::Genesis(_)));

    assert_eq!(
        query.reduce(),
        ExactRational {
            numerator: BigUint::from(25_u32),
            denominator: BigUint::one(),
        },
    );
}
