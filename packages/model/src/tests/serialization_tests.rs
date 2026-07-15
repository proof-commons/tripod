//! Canonical serialization tests.
//!
//! Implements `´test:verification:canonical-attestation-serialization´`.

use num_bigint::BigUint;
use num_traits::Zero;

use super::test_fixtures::*;
use crate::*;

pub fn sample_query() -> AttestationQueryResult {
    AttestationQueryResult {
        context: AttestationContext {
            network_id: TEST_NETWORK_ID,
            genesis_id: TEST_GENESIS_ID,
            architecture_manifest_hash: expected_architecture_manifest_hash().unwrap(),
            checkpoint_block_hash: block_hash(3),
            checkpoint_height: 100,
            schema_version: ATTESTATION_SCHEMA_VERSION,
        },

        address: ADDRESS_A,

        terms: vec![
            AttestationTerm {
                clear_id: ClearId::Genesis([0_u8; 32]),
                clear_order: CanonicalOrder {
                    height: 0,
                    tx_index: 0,
                },
                aggregate_burn_amount: BigUint::from(2_u32),
                omega: BigUint::from(1_000_u32),
                y: BigUint::from(1_000_u32),
            },
            AttestationTerm {
                clear_id: ClearId::Transaction(txid(9)),
                clear_order: CanonicalOrder {
                    height: 5,
                    tx_index: 1,
                },
                aggregate_burn_amount: BigUint::from(3_u32),
                omega: BigUint::from(1_100_u32),
                y: BigUint::from(1_000_u32),
            },
        ],
    }
}

#[test]
fn query_round_trip_is_canonical() {
    let query = sample_query();

    let bytes = serialize_query(&query).unwrap();

    let decoded = deserialize_query(&bytes).unwrap();

    assert_eq!(decoded, query);

    assert_eq!(serialize_query(&decoded).unwrap(), bytes);
}

#[test]
fn exact_rational_reduction_is_correct() {
    let query = sample_query();

    let reduced = query.reduce();

    // 2*(1000/1000)
    // + 3*(1100/1000)
    // = 2 + 33/10
    // = 53/10.
    assert_eq!(reduced.numerator, BigUint::from(53_u32));

    assert_eq!(reduced.denominator, BigUint::from(10_u32));
}

#[test]
fn nonminimal_varint_is_rejected() {
    let bytes = [0x80_u8, 0x00_u8];

    let mut cursor = 0_usize;

    assert_eq!(
        decode_varint(&bytes, &mut cursor),
        Err(DecodeError::NonMinimalVarint),
    );
}

#[test]
fn overflowing_tenth_varint_byte_is_rejected() {
    let bytes = [
        0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x02_u8,
    ];

    let mut cursor = 0_usize;

    assert_eq!(
        decode_varint(&bytes, &mut cursor),
        Err(DecodeError::NonMinimalVarint),
    );
}

#[test]
fn nonminimal_biguint_is_rejected() {
    let bytes = [0x01_u8, 0x00_u8];

    let mut cursor = 0_usize;

    assert_eq!(
        decode_biguint(&bytes, &mut cursor),
        Err(DecodeError::NonMinimalInteger),
    );
}

#[test]
fn duplicate_clear_term_is_rejected() {
    let mut query = sample_query();

    query.terms.push(query.terms[1].clone());

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::DuplicateTerm,
        )),
    );

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::DuplicateTerm));
}

#[test]
fn out_of_order_terms_are_rejected() {
    let mut query = sample_query();

    query.terms.swap(0, 1);

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::OutOfOrderTerm,
        )),
    );

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::OutOfOrderTerm));
}

#[test]
fn encoder_rejects_wrong_architecture_hash() {
    let mut query = sample_query();

    query.context.architecture_manifest_hash = [99_u8; 32];

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::ArchitectureManifestMismatch,
        )),
    );
}

#[test]
fn decoder_rejects_wrong_architecture_hash() {
    let mut query = sample_query();

    query.context.architecture_manifest_hash = [99_u8; 32];

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(
        deserialize_query(&bytes),
        Err(DecodeError::ArchitectureManifestMismatch),
    );
}

#[test]
fn encoder_rejects_unsupported_schema() {
    let mut query = sample_query();

    query.context.schema_version = ATTESTATION_SCHEMA_VERSION + 1;

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::UnsupportedSchema,
        )),
    );
}

#[test]
fn decoder_rejects_unsupported_schema() {
    let mut query = sample_query();

    query.context.schema_version = ATTESTATION_SCHEMA_VERSION + 1;

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(
        deserialize_query(&bytes),
        Err(DecodeError::UnsupportedSchema),
    );
}

#[test]
fn encoder_rejects_term_after_checkpoint() {
    let mut query = sample_query();

    query.context.checkpoint_height = 4;

    query.terms[1].clear_order.height = 5;

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::TermAfterCheckpoint,
        )),
    );
}

#[test]
fn decoder_rejects_term_after_checkpoint() {
    let mut query = sample_query();

    query.context.checkpoint_height = 4;

    query.terms[1].clear_order.height = 5;

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(
        deserialize_query(&bytes),
        Err(DecodeError::TermAfterCheckpoint),
    );
}

#[test]
fn encoder_rejects_zero_clear_omega() {
    let mut query = sample_query();

    query.terms[0].omega = BigUint::zero();

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::ZeroClearOmega,
        )),
    );
}

#[test]
fn decoder_rejects_zero_clear_omega() {
    let mut query = sample_query();

    query.terms[0].omega = BigUint::zero();

    let bytes = crate::ledger::serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::ZeroClearOmega));
}
