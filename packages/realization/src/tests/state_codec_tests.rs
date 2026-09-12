use std::{collections::BTreeSet, mem::size_of};

use proptest::prelude::*;

use crate::{
    Cycle, EncodedStateMetadata, Maturity, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE, ProtocolAmount,
    STATE_METADATA_BYTES, STATE_METADATA_DOMAIN, StateMetadata, StateMetadataRefusal,
    StateRepresentationNonce, decode_state_metadata, encode_state_metadata,
};

const SCHEMA_OFFSET: usize = 21;
const OMEGA_OFFSET: usize = 25;
const MATURITY_TAG_OFFSET: usize = 65;
const MATURITY_PAYLOAD_OFFSET: usize = 66;
const RESERVED_OFFSET: usize = 78;

fn amount(value: u64) -> ProtocolAmount {
    ProtocolAmount::new(value).unwrap()
}

fn metadata(maturity: Maturity) -> StateMetadata {
    StateMetadata {
        omega: amount(1),
        y_l: amount(2),
        y_t: amount(3),
        q: amount(4),
        cycle: Cycle::new(5),
        maturity,
    }
}

fn canonical_bytes(maturity: Maturity) -> Vec<u8> {
    encode_state_metadata(&metadata(maturity), StateRepresentationNonce::new(7))
}

fn metadata_strategy() -> impl Strategy<Value = StateMetadata> {
    let amount = 0_u64..PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE;
    let maturity = prop_oneof![
        Just(Maturity::Unannounced),
        any::<u64>().prop_map(|cycle| Maturity::Announced {
            cycle: Cycle::new(cycle),
        }),
        Just(Maturity::Complete),
    ];

    (
        amount.clone(),
        amount.clone(),
        amount.clone(),
        amount,
        any::<u64>(),
        maturity,
    )
        .prop_map(|(omega, y_l, y_t, q, cycle, maturity)| StateMetadata {
            omega: ProtocolAmount::new(omega).unwrap(),
            y_l: ProtocolAmount::new(y_l).unwrap(),
            y_t: ProtocolAmount::new(y_t).unwrap(),
            q: ProtocolAmount::new(q).unwrap(),
            cycle: Cycle::new(cycle),
            maturity,
        })
}

#[test]
fn encoding_width_is_fixed() {
    let recomputed = STATE_METADATA_DOMAIN.len()
        + size_of::<u32>()
        + 5 * size_of::<u64>()
        + size_of::<u8>()
        + size_of::<u64>()
        + size_of::<u32>()
        + 8;

    assert_eq!(recomputed, 86);
    assert_eq!(STATE_METADATA_BYTES, recomputed);

    for maturity in [
        Maturity::Unannounced,
        Maturity::Announced {
            cycle: Cycle::new(6),
        },
        Maturity::Complete,
    ] {
        assert_eq!(canonical_bytes(maturity).len(), recomputed);
    }
}

#[test]
fn golden_vector_pins_layout_and_byte_order() {
    let semantic = metadata(Maturity::Announced {
        cycle: Cycle::new(6),
    });
    let encoded = encode_state_metadata(&semantic, StateRepresentationNonce::new(7));
    let expected = [
        116, 114, 105, 112, 111, 100, 47, 115, 116, 97, 116, 101, 45, 109, 101, 116, 97, 100,
        97, 116, 97, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0,
        0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5, 1, 0, 0, 0, 0, 0,
        0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(encoded.as_slice(), expected.as_slice());
}

#[test]
fn nonce_is_erased_by_semantic_projection() {
    let semantic = metadata(Maturity::Unannounced);
    let first = encode_state_metadata(&semantic, StateRepresentationNonce::new(7));
    let second = encode_state_metadata(&semantic, StateRepresentationNonce::new(8));
    let first_decoded = decode_state_metadata(&first).unwrap();
    let second_decoded = decode_state_metadata(&second).unwrap();

    assert_ne!(first, second);
    assert_eq!(first_decoded.semantic, second_decoded.semantic);
}

#[test]
fn nonce_advances_deterministically_from_zero() {
    let first = StateRepresentationNonce::ZERO.next().unwrap();
    let second = first.next().unwrap();

    assert_eq!(first, StateRepresentationNonce::new(1));
    assert_eq!(second, StateRepresentationNonce::new(2));
    assert_eq!(StateRepresentationNonce::new(u32::MAX).next(), None);
}

#[test]
fn wrong_domain_refuses() {
    let mut bytes = canonical_bytes(Maturity::Unannounced);
    bytes[0] ^= 1;

    assert_eq!(
        decode_state_metadata(&bytes),
        Err(StateMetadataRefusal::WrongDomain),
    );
}

#[test]
fn unsupported_schema_refuses() {
    let mut bytes = canonical_bytes(Maturity::Unannounced);
    bytes[SCHEMA_OFFSET..OMEGA_OFFSET].copy_from_slice(&2_u32.to_be_bytes());

    assert_eq!(
        decode_state_metadata(&bytes),
        Err(StateMetadataRefusal::UnsupportedSchema),
    );
}

#[test]
fn short_input_refuses_wrong_length() {
    let bytes = canonical_bytes(Maturity::Unannounced);

    assert_eq!(
        decode_state_metadata(&[]),
        Err(StateMetadataRefusal::WrongLength),
    );
    assert_eq!(
        decode_state_metadata(&STATE_METADATA_DOMAIN[..STATE_METADATA_DOMAIN.len() - 1]),
        Err(StateMetadataRefusal::WrongLength),
    );
    assert_eq!(
        decode_state_metadata(&bytes[..STATE_METADATA_BYTES - 1]),
        Err(StateMetadataRefusal::WrongLength),
    );
}

#[test]
fn amount_out_of_domain_refuses() {
    let mut bytes = canonical_bytes(Maturity::Unannounced);
    bytes[OMEGA_OFFSET..OMEGA_OFFSET + size_of::<u64>()]
        .copy_from_slice(&PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE.to_be_bytes());

    assert_eq!(
        decode_state_metadata(&bytes),
        Err(StateMetadataRefusal::AmountOutOfDomain),
    );
}

#[test]
fn unknown_maturity_discriminant_refuses() {
    let mut bytes = canonical_bytes(Maturity::Unannounced);
    bytes[MATURITY_TAG_OFFSET] = 3;

    assert_eq!(
        decode_state_metadata(&bytes),
        Err(StateMetadataRefusal::UnknownMaturityDiscriminant),
    );
}

#[test]
fn malformed_maturity_payload_refuses() {
    let mut unannounced = canonical_bytes(Maturity::Unannounced);
    unannounced[MATURITY_PAYLOAD_OFFSET + size_of::<u64>() - 1] = 1;

    let mut complete = canonical_bytes(Maturity::Complete);
    complete[MATURITY_PAYLOAD_OFFSET + size_of::<u64>() - 1] = 1;

    assert_eq!(
        decode_state_metadata(&unannounced),
        Err(StateMetadataRefusal::MaturityPayloadMalformed),
    );
    assert_eq!(
        decode_state_metadata(&complete),
        Err(StateMetadataRefusal::MaturityPayloadMalformed),
    );
}

#[test]
fn reserved_nonzero_refuses() {
    let mut bytes = canonical_bytes(Maturity::Unannounced);
    bytes[RESERVED_OFFSET] = 1;

    assert_eq!(
        decode_state_metadata(&bytes),
        Err(StateMetadataRefusal::ReservedFieldNonzero),
    );
}

#[test]
fn trailing_byte_refuses() {
    let mut bytes = canonical_bytes(Maturity::Unannounced);
    bytes.push(0);

    assert_eq!(
        decode_state_metadata(&bytes),
        Err(StateMetadataRefusal::TrailingBytes),
    );
}

#[test]
fn every_refusal_variant_is_reached() {
    let mut wrong_domain = canonical_bytes(Maturity::Unannounced);
    wrong_domain[0] ^= 1;

    let mut unsupported_schema = canonical_bytes(Maturity::Unannounced);
    unsupported_schema[SCHEMA_OFFSET..OMEGA_OFFSET].copy_from_slice(&2_u32.to_be_bytes());

    let wrong_length = Vec::new();

    let mut amount_out_of_domain = canonical_bytes(Maturity::Unannounced);
    amount_out_of_domain[OMEGA_OFFSET..OMEGA_OFFSET + size_of::<u64>()]
        .copy_from_slice(&PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE.to_be_bytes());

    let mut unknown_discriminant = canonical_bytes(Maturity::Unannounced);
    unknown_discriminant[MATURITY_TAG_OFFSET] = 3;

    let mut malformed_payload = canonical_bytes(Maturity::Unannounced);
    malformed_payload[MATURITY_PAYLOAD_OFFSET + size_of::<u64>() - 1] = 1;

    let mut reserved_nonzero = canonical_bytes(Maturity::Unannounced);
    reserved_nonzero[RESERVED_OFFSET] = 1;

    let mut trailing_bytes = canonical_bytes(Maturity::Unannounced);
    trailing_bytes.push(0);

    let inputs = [
        wrong_domain,
        unsupported_schema,
        wrong_length,
        amount_out_of_domain,
        unknown_discriminant,
        malformed_payload,
        reserved_nonzero,
        trailing_bytes,
    ];
    let observed = inputs
        .iter()
        .map(|bytes| decode_state_metadata(bytes).unwrap_err())
        .collect::<Vec<_>>();

    assert_eq!(StateMetadataRefusal::ALL.len(), 8);
    assert_eq!(observed.as_slice(), StateMetadataRefusal::ALL);

    for refusal in StateMetadataRefusal::ALL {
        assert!(observed.contains(refusal), "unreached refusal: {refusal:?}");
    }
}

#[test]
fn refusal_names_are_distinct() {
    let mut names = Vec::new();
    let mut unique = BTreeSet::new();

    for refusal in StateMetadataRefusal::ALL {
        names.push(refusal.name());
        unique.insert(refusal.name());
    }

    assert_eq!(names.len(), 8);
    assert_eq!(unique.len(), names.len());

    for name in names {
        let kebab = name.chars().all(|c| c.is_ascii_lowercase() || c == '-');

        assert!(kebab, "name is not kebab-case: {name}");
    }
}

proptest! {
    #[test]
    fn decode_after_encode_is_identity(
        semantic in metadata_strategy(),
        nonce in any::<u32>(),
    ) {
        let representation = StateRepresentationNonce::new(nonce);
        let encoded = encode_state_metadata(&semantic, representation);
        let decoded = decode_state_metadata(&encoded).unwrap();

        prop_assert_eq!(
            decoded,
            EncodedStateMetadata {
                semantic,
                representation,
            },
        );
    }

    #[test]
    fn encode_after_decode_is_byte_identical(
        semantic in metadata_strategy(),
        nonce in any::<u32>(),
        index in 0_usize..STATE_METADATA_BYTES,
        mutation in 1_u8..=u8::MAX,
    ) {
        let mut bytes = encode_state_metadata(
            &semantic,
            StateRepresentationNonce::new(nonce),
        );
        bytes[index] ^= mutation;

        if let Ok(decoded) = decode_state_metadata(&bytes) {
            let reencoded = encode_state_metadata(&decoded.semantic, decoded.representation);

            prop_assert_eq!(reencoded, bytes);
        }
    }

    #[test]
    fn distinct_pairs_encode_distinctly(
        left_semantic in metadata_strategy(),
        left_nonce in any::<u32>(),
        right_semantic in metadata_strategy(),
        right_nonce in any::<u32>(),
    ) {
        let left = (
            left_semantic,
            StateRepresentationNonce::new(left_nonce),
        );
        let right = (
            right_semantic,
            StateRepresentationNonce::new(right_nonce),
        );

        prop_assume!(left != right);

        let left_bytes = encode_state_metadata(&left.0, left.1);
        let right_bytes = encode_state_metadata(&right.0, right.1);

        prop_assert_ne!(left_bytes, right_bytes);
    }
}
