//! Reorg reprojection tests.
//!
//! Implements `´test:verification:attestation-reorg-reprojection´`.

use num_bigint::BigUint;
use num_traits::One;

use super::attestation_history_fixture::*;
use super::scenario_fixtures::ADDRESS_A;
use super::test_fixtures::{block_hash, context, txid};
use crate::*;

/// A reorg is a change of chronology (`postc:interface:monotonicity`,
/// `sec:ledger:reorg`): the same immutable burn record is revalued
/// upward or downward depending on which settlement precedes it in the
/// selected context. Neither direction is privileged; only the raw
/// record is invariant.
///
/// Chronology with the higher-floor settlement *before* the burn
/// (φ = 1.2), so the burn samples it.
fn context_with_clear_before_burn() -> ReferenceIndexer {
    let mut indexer = ReferenceIndexer::empty_for_test(context(5, block_hash(5)));
    indexer.insert_clear_for_test(genesis_clear()).unwrap();
    indexer
        .insert_clear_for_test(clear_entry(1, 5, 1, 1_200, 1_000))
        .unwrap();
    indexer
        .insert_burn_for_test(accepted_burn(10, 5, 2, 100, ADDRESS_A))
        .unwrap();
    indexer
}

/// The same burn, but the higher-floor settlement now *follows* it, so
/// the burn samples the genesis floor (φ = 1.0) instead.
fn context_with_clear_after_burn() -> ReferenceIndexer {
    let mut indexer = ReferenceIndexer::empty_for_test(context(5, block_hash(6)));
    indexer.insert_clear_for_test(genesis_clear()).unwrap();
    indexer
        .insert_clear_for_test(clear_entry(2, 5, 3, 1_200, 1_000))
        .unwrap();
    indexer
        .insert_burn_for_test(accepted_burn(10, 5, 2, 100, ADDRESS_A))
        .unwrap();
    indexer
}

fn burn_value(indexer: &ReferenceIndexer) -> ExactRational {
    indexer.query(ADDRESS_A).unwrap().try_reduce().unwrap()
}

/// Exact rational comparison by cross-multiplication (no `Ord` assumed).
fn strictly_less(left: &ExactRational, right: &ExactRational) -> bool {
    left.numerator() * right.denominator() < right.numerator() * left.denominator()
}

#[test]
fn reorg_can_revalue_the_same_record_downward() {
    // Chronology C has the settlement before the burn; C' has it after.
    // Moving C -> C' revalues the identical record downward.
    let high = burn_value(&context_with_clear_before_burn());
    let low = burn_value(&context_with_clear_after_burn());
    assert!(
        strictly_less(&low, &high),
        "moving the settlement after the burn must lower the valuation",
    );
}

#[test]
fn reorg_can_revalue_the_same_record_upward() {
    // The mirror direction: C' -> C revalues the identical record upward.
    let low = burn_value(&context_with_clear_after_burn());
    let high = burn_value(&context_with_clear_before_burn());
    assert!(
        strictly_less(&low, &high),
        "moving the settlement before the burn must raise the valuation",
    );
}

#[test]
fn reorg_leaves_the_raw_burn_record_unchanged_in_both_directions() {
    let before = context_with_clear_before_burn();
    let after = context_with_clear_after_burn();

    let burn_before = before.burns().get(&txid(10)).unwrap();
    let burn_after = after.burns().get(&txid(10)).unwrap();

    // Only the sampled settlement differs between the two chronologies;
    // the raw record — txid, ash value, canonical order, and records
    // (address, amount, ordinal) — is identical.
    assert_eq!(burn_before.txid, burn_after.txid);
    assert_eq!(burn_before.ash_value, burn_after.ash_value);
    assert_eq!(burn_before.order, burn_after.order);
    assert_eq!(burn_before.records, burn_after.records);
}

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

    assert_ne!(
        first_terms.try_reduce().unwrap(),
        second_terms.try_reduce().unwrap()
    );
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

    let reduced = query.try_reduce().unwrap();

    assert_eq!(reduced.numerator(), &BigUint::from(25_u32));
    assert_eq!(reduced.denominator(), &BigUint::one());
}
