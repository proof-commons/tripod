//! The push census, against explicit expected byte vectors.
//!
//! Every expected encoding below is written out by hand, prefix bytes
//! included. The production encoder never supplies its own
//! expectation, so a serializer that chose the wrong form or wrote a
//! width the wrong way round fails here rather than agreeing with
//! itself.

use target_elements::EncodingClass;

use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::TapscriptProgram;

use super::reviewed_target;

/// The one-instruction program pushing `payload`.
fn push(payload: Vec<u8>) -> TapscriptProgram {
    let target = reviewed_target();
    let item = StackItem::new(&target, payload).expect("the payload is within the bound");
    TapscriptProgram::new(vec![TapscriptInstruction::Push(item)])
        .expect("one instruction is within the limit")
}

/// A payload of `width` bytes that no numeric form ever carries.
fn filler(width: usize) -> Vec<u8> {
    vec![0xab; width]
}

#[test]
fn the_empty_push_is_the_single_canonical_opcode() {
    let target = reviewed_target();

    assert_eq!(push(Vec::new()).encode(&target), vec![0x00]);
}

#[test]
fn a_single_small_value_uses_its_own_opcode_rather_than_a_direct_push() {
    let target = reviewed_target();

    assert_eq!(push(vec![0x01]).encode(&target), vec![0x51]);
    assert_eq!(push(vec![0x08]).encode(&target), vec![0x58]);
    assert_eq!(push(vec![0x10]).encode(&target), vec![0x60]);
    assert_eq!(push(vec![0x81]).encode(&target), vec![0x4f]);
}

#[test]
fn a_single_byte_outside_the_literal_opcodes_uses_a_direct_push() {
    let target = reviewed_target();

    assert_eq!(push(vec![0x00]).encode(&target), vec![0x01, 0x00]);
    assert_eq!(push(vec![0x11]).encode(&target), vec![0x01, 0x11]);
    assert_eq!(push(vec![0x80]).encode(&target), vec![0x01, 0x80]);
    assert_eq!(push(vec![0x82]).encode(&target), vec![0x01, 0x82]);
}

#[test]
fn the_direct_push_boundary_is_exact() {
    let target = reviewed_target();

    let mut expected = vec![0x4b_u8];
    expected.extend_from_slice(&filler(75));
    assert_eq!(push(filler(75)).encode(&target), expected);

    // One byte more moves to the one-byte width form, prefix included.
    let mut expected = vec![0x4c_u8, 76];
    expected.extend_from_slice(&filler(76));
    assert_eq!(push(filler(76)).encode(&target), expected);
}

#[test]
fn the_one_byte_width_boundary_is_exact() {
    let target = reviewed_target();

    let mut expected = vec![0x4c_u8, 0xff];
    expected.extend_from_slice(&filler(255));
    assert_eq!(push(filler(255)).encode(&target), expected);

    // The two-byte width is written least significant byte first, so a
    // payload of 256 states its width as 0x00 0x01 and not 0x01 0x00.
    let mut expected = vec![0x4d_u8, 0x00, 0x01];
    expected.extend_from_slice(&filler(256));
    assert_eq!(push(filler(256)).encode(&target), expected);
}

#[test]
fn the_largest_admissible_literal_encodes_and_one_more_does_not_exist() {
    let target = reviewed_target();

    let mut expected = vec![0x4d_u8, 0x08, 0x02];
    expected.extend_from_slice(&filler(520));
    assert_eq!(push(filler(520)).encode(&target), expected);

    assert_eq!(
        StackItem::new(&target, filler(521)),
        Err(TapscriptError::OversizedStackItem {
            offered: 521,
            maximum: 520,
        }),
    );
}

#[test]
fn every_boundary_payload_round_trips_through_the_parser() {
    let target = reviewed_target();

    for payload in [
        Vec::new(),
        vec![0x00],
        vec![0x01],
        vec![0x10],
        vec![0x11],
        vec![0x81],
        filler(2),
        filler(75),
        filler(76),
        filler(255),
        filler(256),
        filler(520),
    ] {
        let program = push(payload.clone());
        let bytes = program.encode(&target);
        assert_eq!(
            TapscriptProgram::decode(&target, &bytes).expect("the minimal form parses"),
            program,
            "payload of {} bytes",
            payload.len(),
        );
    }
}

#[test]
fn every_longer_equivalent_form_is_refused() {
    // The same payload, carried by a form that is not its minimal one.
    // Each of these is a valid target script that a node would relay
    // only under looser rules than the ones this parser applies.
    let target = reviewed_target();
    let nonminimal: Vec<Vec<u8>> = vec![
        // The empty payload as a direct push of nothing is impossible,
        // so the nonminimal empties are the width-prefixed ones.
        vec![0x4c, 0x00],
        vec![0x4d, 0x00, 0x00],
        vec![0x4e, 0x00, 0x00, 0x00, 0x00],
        // A small value as a direct push instead of its own opcode.
        vec![0x01, 0x05],
        vec![0x01, 0x81],
        // A two-byte payload through each wider width.
        vec![0x4c, 0x02, 0xab, 0xab],
        vec![0x4d, 0x02, 0x00, 0xab, 0xab],
        vec![0x4e, 0x02, 0x00, 0x00, 0x00, 0xab, 0xab],
    ];

    for script in nonminimal {
        assert_eq!(
            TapscriptProgram::decode(&target, &script),
            Err(TapscriptError::NonMinimalPush),
            "{script:02x?}",
        );
    }
}

#[test]
fn the_widest_width_form_is_refused_even_at_a_width_it_alone_could_state() {
    // A four-byte width stating a payload above the literal bound is
    // refused for being oversized rather than for being nonminimal, and
    // the two are different findings about the same script.
    let target = reviewed_target();
    let mut script = vec![0x4e_u8, 0x09, 0x02, 0x00, 0x00];
    script.extend_from_slice(&filler(521));

    assert_eq!(
        TapscriptProgram::decode(&target, &script),
        Err(TapscriptError::OversizedStackItem {
            offered: 521,
            maximum: 520,
        }),
    );
}

#[test]
fn a_typed_script_number_pushes_its_minimal_encoding() {
    let target = reviewed_target();
    let cases: Vec<(i64, Vec<u8>)> = vec![
        (0, vec![0x00]),
        (1, vec![0x51]),
        (16, vec![0x60]),
        (17, vec![0x01, 0x11]),
        (-1, vec![0x4f]),
        (127, vec![0x01, 0x7f]),
        // A magnitude whose own top bit is set needs a further byte to
        // carry the sign, which is why 128 is two bytes and not one.
        (128, vec![0x02, 0x80, 0x00]),
        (-128, vec![0x02, 0x80, 0x80]),
        (255, vec![0x02, 0xff, 0x00]),
        (2_147_483_647, vec![0x04, 0xff, 0xff, 0xff, 0x7f]),
    ];

    for (value, expected) in cases {
        let item = StackItem::script_number(&target, value).expect("the value is representable");
        let program = TapscriptProgram::new(vec![TapscriptInstruction::Push(item)])
            .expect("one instruction is within the limit");
        assert_eq!(program.encode(&target), expected, "{value}");
    }
}

#[test]
fn a_script_number_outside_the_target_range_has_no_item() {
    let target = reviewed_target();

    for value in [2_147_483_648_i64, -2_147_483_648, i64::MAX, i64::MIN] {
        assert_eq!(
            StackItem::script_number(&target, value),
            Err(TapscriptError::ScriptNumberOutOfRange { offered: value }),
            "{value}",
        );
    }
}

#[test]
fn the_fixed_width_constructors_produce_the_widths_the_contract_states() {
    // The constructors resolve their width and order from the reviewed
    // contract; this states independently what those turn out to be.
    let target = reviewed_target();

    assert_eq!(
        StackItem::signed_le64(&target, -2).bytes(),
        [0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
    );
    assert_eq!(StackItem::unsigned_le32(&target, 1).bytes(), [1, 0, 0, 0]);
    assert_eq!(
        StackItem::unsigned_le64(&target, 258).bytes(),
        [2, 1, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn an_encoded_item_is_checked_against_its_class() {
    let target = reviewed_target();

    let digest = StackItem::encoded(&target, EncodingClass::Sha256Digest, filler(32))
        .expect("a digest is thirty-two bytes");
    assert_eq!(digest.len(), 32);

    assert_eq!(
        StackItem::encoded(&target, EncodingClass::Sha256Digest, filler(31)),
        Err(TapscriptError::MalformedEncodedItem {
            class: EncodingClass::Sha256Digest,
        }),
    );
    assert_eq!(
        StackItem::encoded(&target, EncodingClass::SchnorrSignature, filler(65)),
        Err(TapscriptError::MalformedEncodedItem {
            class: EncodingClass::SchnorrSignature,
        }),
    );
}

#[test]
fn an_encoded_script_number_must_already_be_minimal() {
    let target = reviewed_target();

    assert!(StackItem::encoded(&target, EncodingClass::ScriptNumber, vec![0x7f]).is_ok());
    assert!(StackItem::encoded(&target, EncodingClass::ScriptNumber, Vec::new()).is_ok());
    assert!(StackItem::encoded(&target, EncodingClass::ScriptNumber, vec![0x80, 0x00]).is_ok());

    for nonminimal in [vec![0x00], vec![0x80], vec![0x01, 0x00], vec![0x01, 0x80]] {
        assert_eq!(
            StackItem::encoded(&target, EncodingClass::ScriptNumber, nonminimal.clone()),
            Err(TapscriptError::NonMinimalScriptNumber),
            "{nonminimal:02x?}",
        );
    }
}
