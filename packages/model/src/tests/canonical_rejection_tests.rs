//! Canonical serialization rejection tests.
//!
//! Implements `´test:verification:canonical-serialization-rejection´`.

use num_bigint::BigUint;
use num_traits::Zero;

use super::serialization_tests::sample_query;
use super::test_fixtures::block_hash;
use crate::*;

#[test]
fn trailing_bytes_are_rejected() {
    let query = sample_query();

    let mut bytes = serialize_query(&query).unwrap();

    bytes.push(0);

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::TrailingBytes));
}

#[test]
fn zero_denominator_is_rejected() {
    let mut query = sample_query();

    query.terms[0].y = BigUint::zero();

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::ZeroDenominator,
        )),
    );

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::ZeroDenominator),);
}

#[test]
fn zero_aggregate_term_is_rejected() {
    let mut query = sample_query();

    query.terms[0].aggregate_burn_amount = BigUint::zero();

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::ZeroAggregate,
        )),
    );

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::ZeroAggregate));
}

#[test]
fn try_reduce_rejects_zero_denominator() {
    let mut query = sample_query();
    query.terms[0].y = BigUint::zero();

    assert_eq!(
        query.try_reduce(),
        Err(QueryValidationError::ZeroDenominator)
    );
}

#[test]
fn try_reduce_rejects_zero_aggregate() {
    let mut query = sample_query();
    query.terms[0].aggregate_burn_amount = BigUint::zero();

    assert_eq!(query.try_reduce(), Err(QueryValidationError::ZeroAggregate));
}

#[test]
fn try_reduce_rejects_wrong_manifest_context() {
    let mut query = sample_query();
    query.context.architecture_manifest_hash = [99_u8; 32];

    assert_eq!(
        query.try_reduce(),
        Err(QueryValidationError::ArchitectureManifestMismatch)
    );
}

#[test]
fn fork_context_changes_serialized_query() {
    let query = sample_query();

    let mut forked = query.clone();

    forked.context.checkpoint_block_hash = block_hash(99);

    assert_ne!(
        serialize_query(&query).unwrap(),
        serialize_query(&forked).unwrap(),
    );
}
