//! Exact target bytes, against hand computation.
//!
//! # Why the expectations are written out rather than built
//!
//! Every expectation below is a literal byte string assembled from the
//! reviewed target's own serializer, field group by field group, with
//! the group named beside it. Comparing the encoder against a second
//! call to the encoder would establish that it is deterministic and
//! nothing else; comparing it against bytes written down from the
//! source is what can actually catch a wrong byte order, a missing
//! flag, or a length prefix in the wrong place.
//!
//! The two shapes are the two the reviewed form census names: a
//! sponsorless transaction paying no fee, and a sponsored one paying a
//! positive fee and taking change.

use crate::bytes::{
    AssetField, AssetId, InputWitness, NonceField, TargetInput, TargetOutput, TargetTransaction,
    Txid, ValueField, compact_size,
};
use crate::error::TransactionRefusal;
use crate::tests::{CLOSED_ASSET, PINNED_PROGRAM, RESERVE_ASSET, SPONSOR_CHANGE_PROGRAM};

/// The fixture ASH input identifiers and the sponsor's.
const FIRST_TXID: [u8; 32] = [0xaa; 32];
const SECOND_TXID: [u8; 32] = [0xbb; 32];
const SPONSOR_TXID: [u8; 32] = [0xdd; 32];

/// The script-path witness stack of the fixture: a leaf program and a
/// control block, standing in for the real ones so that the encoding is
/// checked without dragging a link into an encoder test.
const LEAF_ITEM: [u8; 3] = [0x01, 0x02, 0x03];
const CONTROL_ITEM: [u8; 1] = [0xff];

/// The sponsor's returned stack: a signature and a public key.
const SIGNATURE_ITEM: [u8; 5] = [0x30, 0x44, 0x02, 0x20, 0x01];
const PUBKEY_ITEM: [u8; 3] = [0x02, 0x0a, 0x0b];

/// One ASH input of the fixture.
fn ash_input(txid: [u8; 32], index: u32) -> TargetInput {
    TargetInput::new(
        crate::bytes::Outpoint::new(Txid::from_internal(txid), index)
            .expect("the fixture index is in range"),
        0xffff_ffff,
    )
}

/// The successor output of the fixture.
fn successor(amount: u64) -> TargetOutput {
    let mut program = vec![0x51, 0x20];
    program.extend_from_slice(&PINNED_PROGRAM);
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(CLOSED_ASSET)),
        ValueField::Explicit(amount),
        NonceField::Null,
        program,
    )
}

/// The script-path witness of the fixture.
fn script_path_witness() -> InputWitness {
    InputWitness::new(vec![LEAF_ITEM.to_vec(), CONTROL_ITEM.to_vec()])
}

/// The sponsorless fixture transaction.
fn sponsorless() -> TargetTransaction {
    TargetTransaction::new(
        3,
        vec![ash_input(FIRST_TXID, 0), ash_input(SECOND_TXID, 1)],
        vec![successor(300)],
        0,
        vec![script_path_witness(), script_path_witness()],
    )
    .expect("the fixture transaction is well formed")
}

/// The sponsored fixture transaction.
fn sponsored() -> TargetTransaction {
    let mut change_program = vec![0x00, 0x20];
    change_program.extend_from_slice(&SPONSOR_CHANGE_PROGRAM);
    TargetTransaction::new(
        2,
        vec![
            ash_input(FIRST_TXID, 0),
            ash_input(SECOND_TXID, 1),
            ash_input(SPONSOR_TXID, 2),
        ],
        vec![
            successor(300),
            TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal(RESERVE_ASSET)),
                ValueField::Explicit(40),
                NonceField::Null,
                change_program,
            ),
            TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal(RESERVE_ASSET)),
                ValueField::Explicit(10),
                NonceField::Null,
                Vec::new(),
            ),
        ],
        0,
        vec![
            script_path_witness(),
            script_path_witness(),
            InputWitness::new(vec![SIGNATURE_ITEM.to_vec(), PUBKEY_ITEM.to_vec()]),
        ],
    )
    .expect("the fixture transaction is well formed")
}

/// Append a hand-written group to an expectation.
fn push(expected: &mut Vec<u8>, group: &[u8]) {
    expected.extend_from_slice(group);
}

/// The hand-written bytes of one explicit-asset, explicit-value,
/// null-nonce output.
fn hand_output(asset: [u8; 32], amount_big_endian: [u8; 8], program: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x01];
    bytes.extend_from_slice(&asset);
    bytes.push(0x01);
    bytes.extend_from_slice(&amount_big_endian);
    bytes.push(0x00);
    bytes.push(u8::try_from(program.len()).expect("the fixture programs are short"));
    bytes.extend_from_slice(program);
    bytes
}

/// The hand-written bytes of one input.
fn hand_input(txid: [u8; 32], index_little_endian: [u8; 4]) -> Vec<u8> {
    let mut bytes = txid.to_vec();
    bytes.extend_from_slice(&index_little_endian);
    bytes.push(0x00);
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    bytes
}

/// The hand-written witness of one input, over these stack items.
fn hand_witness(items: &[&[u8]]) -> Vec<u8> {
    let mut bytes = vec![0x00, 0x00];
    bytes.push(u8::try_from(items.len()).expect("the fixture stacks are short"));
    for item in items {
        bytes.push(u8::try_from(item.len()).expect("the fixture items are short"));
        bytes.extend_from_slice(item);
    }
    bytes.push(0x00);
    bytes
}

#[test]
fn the_sponsorless_form_encodes_to_exactly_these_bytes() {
    // Written out from `SerializeTransaction`: the version, then the
    // flag byte the target's own transaction mode writes
    // unconditionally, then the inputs, the outputs, the lock time, and
    // finally the whole witness — which is where the Elements
    // serialization differs from the upstream one, and the difference
    // this expectation would catch.
    let mut expected = Vec::new();
    push(&mut expected, &[0x03, 0x00, 0x00, 0x00]); // version 3, little-endian
    push(&mut expected, &[0x01]); // witness present
    push(&mut expected, &[0x02]); // two inputs
    push(
        &mut expected,
        &hand_input(FIRST_TXID, [0x00, 0x00, 0x00, 0x00]),
    );
    push(
        &mut expected,
        &hand_input(SECOND_TXID, [0x01, 0x00, 0x00, 0x00]),
    );
    push(&mut expected, &[0x01]); // one output
    let mut program = vec![0x51, 0x20];
    program.extend_from_slice(&PINNED_PROGRAM);
    push(
        &mut expected,
        // 300 as a big-endian eight-byte amount: this is the field
        // whose order differs from every other integer in the format.
        &hand_output(
            CLOSED_ASSET,
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2c],
            &program,
        ),
    );
    push(&mut expected, &[0x00, 0x00, 0x00, 0x00]); // lock time
    push(&mut expected, &hand_witness(&[&LEAF_ITEM, &CONTROL_ITEM]));
    push(&mut expected, &hand_witness(&[&LEAF_ITEM, &CONTROL_ITEM]));
    push(&mut expected, &[0x00, 0x00]); // the single output witness

    assert_eq!(sponsorless().encode(), expected);
    assert_eq!(expected.len(), 193);
}

#[test]
fn the_sponsored_form_encodes_to_exactly_these_bytes() {
    let mut expected = Vec::new();
    push(&mut expected, &[0x02, 0x00, 0x00, 0x00]);
    push(&mut expected, &[0x01]);
    push(&mut expected, &[0x03]); // three inputs
    push(
        &mut expected,
        &hand_input(FIRST_TXID, [0x00, 0x00, 0x00, 0x00]),
    );
    push(
        &mut expected,
        &hand_input(SECOND_TXID, [0x01, 0x00, 0x00, 0x00]),
    );
    push(
        &mut expected,
        &hand_input(SPONSOR_TXID, [0x02, 0x00, 0x00, 0x00]),
    );
    push(&mut expected, &[0x03]); // three outputs
    let mut program = vec![0x51, 0x20];
    program.extend_from_slice(&PINNED_PROGRAM);
    push(
        &mut expected,
        &hand_output(
            CLOSED_ASSET,
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2c],
            &program,
        ),
    );
    let mut change = vec![0x00, 0x20];
    change.extend_from_slice(&SPONSOR_CHANGE_PROGRAM);
    push(
        &mut expected,
        &hand_output(
            RESERVE_ASSET,
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28],
            &change,
        ),
    );
    push(
        &mut expected,
        // The fee role: an empty program, an explicit value, an
        // explicit asset. Its length prefix is a zero, and that zero is
        // the whole of what makes it a fee output rather than an
        // ordinary one.
        &hand_output(
            RESERVE_ASSET,
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a],
            &[],
        ),
    );
    push(&mut expected, &[0x00, 0x00, 0x00, 0x00]);
    push(&mut expected, &hand_witness(&[&LEAF_ITEM, &CONTROL_ITEM]));
    push(&mut expected, &hand_witness(&[&LEAF_ITEM, &CONTROL_ITEM]));
    push(
        &mut expected,
        &hand_witness(&[&SIGNATURE_ITEM, &PUBKEY_ITEM]),
    );
    push(&mut expected, &[0x00, 0x00]); // successor's output witness
    push(&mut expected, &[0x00, 0x00]); // change's
    push(&mut expected, &[0x00, 0x00]); // the fee role's

    assert_eq!(sponsored().encode(), expected);
    assert_eq!(expected.len(), 374);
}

#[test]
fn the_stripped_encoding_is_not_a_prefix_of_the_witness_one() {
    // The flag byte differs, so the two diverge at their fifth byte.
    // Worth asserting because a reader who knows the upstream Bitcoin
    // format expects the stripped form to be a prefix, and a weight
    // computed on that assumption would be wrong.
    let transaction = sponsorless();
    let full = transaction.encode();
    let stripped = transaction.encode_without_witness();
    assert_eq!(stripped.len(), 171);
    assert_eq!(full[4], 0x01);
    assert_eq!(stripped[4], 0x00);
    assert!(!full.starts_with(&stripped));
}

#[test]
fn the_settled_figures_are_the_targets_own_arithmetic() {
    // stripped 171, total 193, so the witness is 22 bytes; the weight
    // is 171 * 3 + 193 = 706; the virtual size is that over four,
    // rounded up, which is 177 rather than 176.
    let transaction = sponsorless();
    assert_eq!(transaction.witness_bytes(), 22);
    assert_eq!(transaction.weight(), 706);
    assert_eq!(transaction.virtual_size(), 177);

    // stripped 334, total 374, witness 40, weight 334 * 3 + 374 = 1376,
    // virtual size 344 exactly.
    let sponsored = sponsored();
    assert_eq!(sponsored.witness_bytes(), 40);
    assert_eq!(sponsored.weight(), 1376);
    assert_eq!(sponsored.virtual_size(), 344);
}

#[test]
fn both_forms_survive_the_round_trip() {
    for transaction in [sponsorless(), sponsored()] {
        let bytes = transaction.encode();
        let decoded = TargetTransaction::decode(&bytes).expect("the fixture bytes decode");
        assert_eq!(decoded, transaction);
        assert_eq!(decoded.encode(), bytes);
    }
}

#[test]
fn compact_size_uses_the_narrowest_form() {
    // The four widths and their boundaries, written out. A count of
    // 0xfd is the first that cannot be a single byte, and a decoder
    // that accepted the wider spelling of a small count would accept
    // two byte strings for one transaction.
    assert_eq!(compact_size(0), vec![0x00]);
    assert_eq!(compact_size(0xfc), vec![0xfc]);
    assert_eq!(compact_size(0xfd), vec![0xfd, 0xfd, 0x00]);
    assert_eq!(compact_size(0xffff), vec![0xfd, 0xff, 0xff]);
    assert_eq!(compact_size(0x1_0000), vec![0xfe, 0x00, 0x00, 0x01, 0x00]);
    assert_eq!(
        compact_size(0x1_0000_0000),
        vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
    );
}

#[test]
fn a_non_minimal_count_is_refused() {
    let mut bytes = sponsorless().encode();
    // Respell the input count 0x02 as the two-byte form.
    bytes.splice(5..6, [0xfd, 0x02, 0x00]);
    assert!(matches!(
        TargetTransaction::decode(&bytes),
        Err(TransactionRefusal::NonMinimalCompactSize { value: 2, .. })
    ));
}

#[test]
fn issuance_and_pegin_markers_are_refused_rather_than_ignored() {
    for (flag, expected) in [
        (0x80_u8, TransactionRefusal::IssuanceInputRefused),
        (0x40_u8, TransactionRefusal::PeginInputRefused),
    ] {
        let mut bytes = sponsorless().encode();
        // The first input's index is the four bytes after its
        // identifier, and its high byte carries both markers.
        let high = 6 + 32 + 3;
        bytes[high] |= flag;
        assert_eq!(TargetTransaction::decode(&bytes), Err(expected));
    }
}

#[test]
fn trailing_bytes_are_refused() {
    let mut bytes = sponsorless().encode();
    bytes.push(0x00);
    assert!(matches!(
        TargetTransaction::decode(&bytes),
        Err(TransactionRefusal::TrailingTargetBytes { .. })
    ));
}

#[test]
fn an_unrecognized_field_prefix_is_refused() {
    let mut bytes = sponsorless().encode();
    // The single output begins after the version, flag, input count,
    // two inputs, and the output count.
    let output = 4 + 1 + 1 + 2 * 41 + 1;
    bytes[output] = 0x07;
    assert_eq!(
        TargetTransaction::decode(&bytes),
        Err(TransactionRefusal::UnrecognizedFieldPrefix { prefix: 0x07 })
    );
}

#[test]
fn the_fee_role_is_recognized_by_form_and_never_by_amount() {
    let sponsored = sponsored();
    let fee = sponsored.outputs().last().expect("the fee role is present");
    assert!(fee.is_fee());

    // The same output with a spendable program is not a fee output,
    // whatever its amount, and an ordinary output holding a small
    // number does not become one.
    let ordinary = TargetOutput::new(
        fee.asset(),
        ValueField::Explicit(0),
        NonceField::Null,
        vec![0x51, 0x20],
    );
    assert!(!ordinary.is_fee());

    // And an empty-programmed output whose value is a commitment is not
    // one either: the test is a conjunction, not a program check.
    let committed = TargetOutput::new(
        fee.asset(),
        ValueField::Commitment([0x08; 33]),
        NonceField::Null,
        Vec::new(),
    );
    assert!(!committed.is_fee());
}

#[test]
fn a_witness_census_that_is_not_one_per_input_is_refused() {
    let refusal = TargetTransaction::new(
        3,
        vec![ash_input(FIRST_TXID, 0), ash_input(SECOND_TXID, 1)],
        vec![successor(300)],
        0,
        vec![script_path_witness()],
    )
    .expect_err("a short witness census is refused");
    assert_eq!(
        refusal,
        TransactionRefusal::WitnessCensusMismatch {
            inputs: 2,
            witnesses: 1,
        }
    );
}

#[test]
fn a_transaction_with_no_witness_carries_the_other_flag() {
    let transaction = TargetTransaction::new(
        3,
        vec![ash_input(FIRST_TXID, 0)],
        vec![successor(300)],
        0,
        vec![InputWitness::default()],
    )
    .expect("an unsigned template is well formed");
    assert!(!transaction.has_witness());
    let bytes = transaction.encode();
    assert_eq!(bytes[4], 0x00);
    // Encoding an all-empty witness section is what the target asserts
    // against, so the two encodings coincide here rather than differing
    // by an empty section.
    assert_eq!(bytes, transaction.encode_without_witness());
    assert_eq!(transaction.witness_bytes(), 0);
}
