use std::{collections::BTreeSet, mem::size_of};

use proptest::prelude::*;

use crate::{
    Cycle, EncodedStateMetadata, Maturity, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE, ProtocolAmount,
    STATE_METADATA_BYTES, STATE_METADATA_CONSTANT_BYTES, STATE_METADATA_DOMAIN,
    STATE_METADATA_LAYOUT, STATE_METADATA_VARIABLE_BYTES, STATE_METADATA_VARIABLE_RANGE,
    StateMetadata, StateMetadataRefusal, StateMetadataRegionClass, StateRepresentationNonce,
    decode_state_metadata, encode_state_metadata, rebuild_state_metadata,
    state_metadata_variable_region,
};

const SCHEMA_OFFSET: usize = 21;
const OMEGA_OFFSET: usize = 25;
const Y_L_OFFSET: usize = 33;
const Y_T_OFFSET: usize = 41;
const Q_OFFSET: usize = 49;
const CYCLE_OFFSET: usize = 57;
const MATURITY_TAG_OFFSET: usize = 65;
const MATURITY_PAYLOAD_OFFSET: usize = 66;
const NONCE_OFFSET: usize = 74;
const RESERVED_OFFSET: usize = 78;

#[derive(Clone, Copy, Debug)]
enum StateField {
    Omega,
    Yl,
    Yt,
    Q,
    Cycle,
    Maturity,
    Nonce,
}

const STATE_FIELD_SLICES: [(StateField, usize, usize); 7] = [
    (StateField::Omega, OMEGA_OFFSET, size_of::<u64>()),
    (StateField::Yl, Y_L_OFFSET, size_of::<u64>()),
    (StateField::Yt, Y_T_OFFSET, size_of::<u64>()),
    (StateField::Q, Q_OFFSET, size_of::<u64>()),
    (StateField::Cycle, CYCLE_OFFSET, size_of::<u64>()),
    (
        StateField::Maturity,
        MATURITY_TAG_OFFSET,
        size_of::<u8>() + size_of::<u64>(),
    ),
    (StateField::Nonce, NONCE_OFFSET, size_of::<u32>()),
];

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

fn field_bytes(
    field: StateField,
    semantic: &StateMetadata,
    representation: StateRepresentationNonce,
) -> Vec<u8> {
    match field {
        StateField::Omega => semantic.omega.get().to_be_bytes().to_vec(),
        StateField::Yl => semantic.y_l.get().to_be_bytes().to_vec(),
        StateField::Yt => semantic.y_t.get().to_be_bytes().to_vec(),
        StateField::Q => semantic.q.get().to_be_bytes().to_vec(),
        StateField::Cycle => semantic.cycle.get().to_be_bytes().to_vec(),
        StateField::Maturity => {
            let (tag, payload) = match semantic.maturity {
                Maturity::Unannounced => (0, Cycle::ZERO),
                Maturity::Announced { cycle } => (1, cycle),
                Maturity::Complete => (2, Cycle::ZERO),
            };
            let mut bytes = [0_u8; 9];
            bytes[0] = tag;
            bytes[1..].copy_from_slice(&payload.get().to_be_bytes());
            bytes.to_vec()
        }
        StateField::Nonce => representation.get().to_be_bytes().to_vec(),
    }
}

fn expected_after_replacement(
    mut semantic: StateMetadata,
    mut representation: StateRepresentationNonce,
    field: StateField,
    replacement: &StateMetadata,
    replacement_representation: StateRepresentationNonce,
) -> EncodedStateMetadata {
    match field {
        StateField::Omega => semantic.omega = replacement.omega,
        StateField::Yl => semantic.y_l = replacement.y_l,
        StateField::Yt => semantic.y_t = replacement.y_t,
        StateField::Q => semantic.q = replacement.q,
        StateField::Cycle => semantic.cycle = replacement.cycle,
        StateField::Maturity => semantic.maturity = replacement.maturity,
        StateField::Nonce => representation = replacement_representation,
    }
    EncodedStateMetadata {
        semantic,
        representation,
    }
}

#[test]
fn layout_rows_tile_the_canonical_encoding() {
    let mut cursor = 0;
    let mut constant_bytes = 0;
    for row in STATE_METADATA_LAYOUT {
        assert_eq!(
            row.range.start, cursor,
            "gap or overlap before {}",
            row.name
        );
        assert!(row.range.end > row.range.start);
        if let StateMetadataRegionClass::Constant(bytes) = row.class {
            assert_eq!(bytes.len(), row.range.len());
            constant_bytes += bytes.len();
        }
        cursor = row.range.end;
    }

    assert_eq!(cursor, STATE_METADATA_BYTES);
    assert_eq!(constant_bytes, 33);
    assert_eq!(STATE_METADATA_CONSTANT_BYTES, constant_bytes);
    assert_eq!(STATE_METADATA_VARIABLE_RANGE, 25..78);
    assert_eq!(STATE_METADATA_VARIABLE_BYTES, 53);
    assert_eq!(constant_bytes + STATE_METADATA_VARIABLE_BYTES, cursor);
}

#[test]
fn layout_variable_rows_match_field_slices() {
    let offsets = [
        ("domain", 0),
        ("schema", SCHEMA_OFFSET),
        ("omega", OMEGA_OFFSET),
        ("y_l", Y_L_OFFSET),
        ("y_t", Y_T_OFFSET),
        ("q", Q_OFFSET),
        ("cycle", CYCLE_OFFSET),
        ("maturity", MATURITY_TAG_OFFSET),
        ("nonce", NONCE_OFFSET),
        ("reserved", RESERVED_OFFSET),
    ];
    assert_eq!(STATE_METADATA_LAYOUT.len(), offsets.len());
    for (row, (name, offset)) in STATE_METADATA_LAYOUT.iter().zip(offsets) {
        assert_eq!(row.name, name);
        assert_eq!(row.range.start, offset);
    }
    assert_eq!(
        MATURITY_PAYLOAD_OFFSET,
        STATE_METADATA_LAYOUT[7].range.start + 1
    );

    let variable_rows = STATE_METADATA_LAYOUT
        .iter()
        .filter(|row| row.class == StateMetadataRegionClass::Variable)
        .map(|row| (row.name, row.range.clone()))
        .collect::<Vec<_>>();
    let fields = STATE_FIELD_SLICES
        .iter()
        .map(|&(field, offset, width)| {
            let name = match field {
                StateField::Omega => "omega",
                StateField::Yl => "y_l",
                StateField::Yt => "y_t",
                StateField::Q => "q",
                StateField::Cycle => "cycle",
                StateField::Maturity => "maturity",
                StateField::Nonce => "nonce",
            };
            (name, offset..offset + width)
        })
        .collect::<Vec<_>>();

    assert_eq!(variable_rows, fields);
    assert_eq!(
        variable_rows
            .iter()
            .map(|(_, range)| range.clone())
            .collect::<Vec<_>>(),
        [25..33, 33..41, 41..49, 49..57, 57..65, 65..74, 74..78],
    );
    assert_eq!(
        variable_rows.first().unwrap().1.start,
        STATE_METADATA_VARIABLE_RANGE.start
    );
    assert_eq!(
        variable_rows.last().unwrap().1.end,
        STATE_METADATA_VARIABLE_RANGE.end
    );
    for pair in variable_rows.windows(2) {
        assert_eq!(pair[0].1.end, pair[1].1.start);
    }
}

#[test]
fn field_slices_tile_the_canonical_encoding() {
    assert_eq!(SCHEMA_OFFSET, STATE_METADATA_DOMAIN.len());
    assert_eq!(OMEGA_OFFSET, SCHEMA_OFFSET + size_of::<u32>());

    let mut cursor = OMEGA_OFFSET;
    for &(field, offset, width) in &STATE_FIELD_SLICES {
        assert_eq!(offset, cursor, "gap or overlap before {field:?}");
        cursor += width;
    }

    assert_eq!(cursor, RESERVED_OFFSET);
    assert_eq!(RESERVED_OFFSET + size_of::<u64>(), STATE_METADATA_BYTES);
}

#[test]
fn maturity_slice_is_canonical_and_strict() {
    let announced_cycle = Cycle::new(13);
    for (maturity, tag, payload) in [
        (Maturity::Unannounced, 0, Cycle::ZERO),
        (
            Maturity::Announced {
                cycle: announced_cycle,
            },
            1,
            announced_cycle,
        ),
        (Maturity::Complete, 2, Cycle::ZERO),
    ] {
        let bytes = canonical_bytes(maturity);
        assert_eq!(bytes[MATURITY_TAG_OFFSET], tag);
        assert_eq!(
            &bytes[MATURITY_PAYLOAD_OFFSET..NONCE_OFFSET],
            &payload.get().to_be_bytes(),
        );
    }

    for tag in [0, 2] {
        let mut malformed = canonical_bytes(Maturity::Unannounced);
        malformed[MATURITY_TAG_OFFSET] = tag;
        malformed[NONCE_OFFSET - 1] = 1;
        assert_eq!(
            decode_state_metadata(&malformed),
            Err(StateMetadataRefusal::MaturityPayloadMalformed),
        );
    }

    let mut unknown = canonical_bytes(Maturity::Unannounced);
    unknown[MATURITY_TAG_OFFSET] = 3;
    assert_eq!(
        decode_state_metadata(&unknown),
        Err(StateMetadataRefusal::UnknownMaturityDiscriminant),
    );
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
        116, 114, 105, 112, 111, 100, 47, 115, 116, 97, 116, 101, 45, 109, 101, 116, 97, 100, 97,
        116, 97, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
        3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5, 1, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 7,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(encoded.as_slice(), expected.as_slice());
    for row in STATE_METADATA_LAYOUT {
        if let StateMetadataRegionClass::Constant(bytes) = row.class {
            assert_eq!(&expected[row.range], bytes, "constant row {}", row.name);
        }
    }
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
    fn layout_constants_match_encoded_metadata(
        semantic in metadata_strategy(),
        nonce in any::<u32>(),
    ) {
        let encoded = encode_state_metadata(&semantic, StateRepresentationNonce::new(nonce));
        for row in STATE_METADATA_LAYOUT {
            if let StateMetadataRegionClass::Constant(bytes) = row.class {
                prop_assert_eq!(&encoded[row.range], bytes);
            }
        }
    }

    #[test]
    fn variable_region_round_trips_encoded_metadata(
        semantic in metadata_strategy(),
        nonce in any::<u32>(),
    ) {
        let encoded = encode_state_metadata(&semantic, StateRepresentationNonce::new(nonce));
        let canonical: [u8; STATE_METADATA_BYTES] = encoded.as_slice().try_into().unwrap();
        let variable = state_metadata_variable_region(&canonical);

        prop_assert_eq!(variable.as_slice(), &encoded[25..78]);
        prop_assert_eq!(rebuild_state_metadata(&variable), canonical);
    }

    #[test]
    fn rebuilt_metadata_decodes_identically(
        semantic in metadata_strategy(),
        nonce in any::<u32>(),
    ) {
        let representation = StateRepresentationNonce::new(nonce);
        let encoded = encode_state_metadata(&semantic, representation);
        let canonical: [u8; STATE_METADATA_BYTES] = encoded.as_slice().try_into().unwrap();
        let rebuilt = rebuild_state_metadata(&state_metadata_variable_region(&canonical));
        let decoded = decode_state_metadata(&rebuilt);

        prop_assert_eq!(&decoded, &decode_state_metadata(&encoded));
        prop_assert_eq!(decoded, Ok(EncodedStateMetadata { semantic, representation }));
    }

    #[test]
    fn arbitrary_variable_regions_round_trip(
        bytes in prop::collection::vec(any::<u8>(), STATE_METADATA_VARIABLE_BYTES),
    ) {
        let variable: [u8; STATE_METADATA_VARIABLE_BYTES] = bytes.as_slice().try_into().unwrap();
        let rebuilt = rebuild_state_metadata(&variable);

        prop_assert_eq!(state_metadata_variable_region(&rebuilt), variable);
        prop_assert_eq!(&rebuilt[25..78], variable.as_slice());
        for row in STATE_METADATA_LAYOUT {
            if let StateMetadataRegionClass::Constant(bytes) = row.class {
                prop_assert_eq!(&rebuilt[row.range], bytes);
            }
        }
    }

    #[test]
    fn field_slices_match_canonical_bytes_and_exact_differences(
        left_semantic in metadata_strategy(),
        left_nonce in any::<u32>(),
        right_semantic in metadata_strategy(),
        right_nonce in any::<u32>(),
    ) {
        let left_representation = StateRepresentationNonce::new(left_nonce);
        let right_representation = StateRepresentationNonce::new(right_nonce);
        let left = encode_state_metadata(&left_semantic, left_representation);
        let right = encode_state_metadata(&right_semantic, right_representation);
        let differing_values = [
            left_semantic.omega != right_semantic.omega,
            left_semantic.y_l != right_semantic.y_l,
            left_semantic.y_t != right_semantic.y_t,
            left_semantic.q != right_semantic.q,
            left_semantic.cycle != right_semantic.cycle,
            left_semantic.maturity != right_semantic.maturity,
            left_representation != right_representation,
        ];

        for (&(field, offset, width), values_differ) in
            STATE_FIELD_SLICES.iter().zip(differing_values)
        {
            let left_field = field_bytes(field, &left_semantic, left_representation);
            let right_field = field_bytes(field, &right_semantic, right_representation);
            let end = offset + width;

            prop_assert_eq!(&left[offset..end], left_field.as_slice());
            prop_assert_eq!(&right[offset..end], right_field.as_slice());
            prop_assert_eq!(left[offset..end] != right[offset..end], values_differ);
        }
    }

    #[test]
    fn replacing_field_slice_changes_only_its_projection(
        semantic in metadata_strategy(),
        nonce in any::<u32>(),
        replacement in metadata_strategy(),
        replacement_nonce in any::<u32>(),
    ) {
        let representation = StateRepresentationNonce::new(nonce);
        let replacement_representation = StateRepresentationNonce::new(replacement_nonce);
        let encoded = encode_state_metadata(&semantic, representation);

        for &(field, offset, width) in &STATE_FIELD_SLICES {
            let mut replaced = encoded.clone();
            let replacement_bytes = field_bytes(field, &replacement, replacement_representation);
            replaced[offset..offset + width].copy_from_slice(&replacement_bytes);
            let expected = expected_after_replacement(
                semantic,
                representation,
                field,
                &replacement,
                replacement_representation,
            );

            prop_assert_eq!(decode_state_metadata(&replaced), Ok(expected));
        }
    }

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
