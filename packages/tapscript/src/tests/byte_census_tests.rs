//! The instruction byte census, against an independent table.
//!
//! The expected bytes below were transcribed from the reviewed upstream
//! opcode enumeration, one line at a time. They are deliberately *not*
//! read out of the target registry: a test that asked the registry
//! which byte it declares would agree with any registry at all,
//! including one whose transcription slipped by one.

use std::collections::{BTreeMap, BTreeSet};

use target_elements::OpcodeId;

use crate::instruction::TapscriptInstruction;
use crate::program::TapscriptProgram;

use super::reviewed_target;

/// Every reviewed primitive and the byte it is expected to serialize
/// to.
const EXPECTED_BYTES: &[(OpcodeId, u8)] = &[
    (OpcodeId::Verify, 0x69),
    (OpcodeId::DropTwo, 0x6d),
    (OpcodeId::DuplicateTwo, 0x6e),
    (OpcodeId::Drop, 0x75),
    (OpcodeId::Duplicate, 0x76),
    (OpcodeId::RemoveSecond, 0x77),
    (OpcodeId::CopyOver, 0x78),
    (OpcodeId::Rotate, 0x7b),
    (OpcodeId::Swap, 0x7c),
    (OpcodeId::Tuck, 0x7d),
    (OpcodeId::Concatenate, 0x7e),
    (OpcodeId::Substring, 0x7f),
    (OpcodeId::Size, 0x82),
    (OpcodeId::BitwiseAnd, 0x84),
    (OpcodeId::BitwiseXor, 0x86),
    (OpcodeId::Equal, 0x87),
    (OpcodeId::EqualVerify, 0x88),
    (OpcodeId::CheckSig, 0xac),
    (OpcodeId::CheckSigVerify, 0xad),
    (OpcodeId::CheckSequenceVerify, 0xb2),
    (OpcodeId::CheckSigFromStack, 0xc1),
    (OpcodeId::CheckSigFromStackVerify, 0xc2),
    (OpcodeId::Sha256Initialize, 0xc4),
    (OpcodeId::Sha256Update, 0xc5),
    (OpcodeId::Sha256Finalize, 0xc6),
    (OpcodeId::InspectInputOutpoint, 0xc7),
    (OpcodeId::InspectInputAsset, 0xc8),
    (OpcodeId::InspectInputValue, 0xc9),
    (OpcodeId::InspectInputScriptPubKey, 0xca),
    (OpcodeId::InspectInputSequence, 0xcb),
    (OpcodeId::InspectInputIssuance, 0xcc),
    (OpcodeId::PushCurrentInputIndex, 0xcd),
    (OpcodeId::InspectOutputAsset, 0xce),
    (OpcodeId::InspectOutputValue, 0xcf),
    (OpcodeId::InspectOutputNonce, 0xd0),
    (OpcodeId::InspectOutputScriptPubKey, 0xd1),
    (OpcodeId::InspectVersion, 0xd2),
    (OpcodeId::InspectLockTime, 0xd3),
    (OpcodeId::InspectNumInputs, 0xd4),
    (OpcodeId::InspectNumOutputs, 0xd5),
    (OpcodeId::TxWeight, 0xd6),
    (OpcodeId::Add64, 0xd7),
    (OpcodeId::Sub64, 0xd8),
    (OpcodeId::Mul64, 0xd9),
    (OpcodeId::Div64, 0xda),
    (OpcodeId::Neg64, 0xdb),
    (OpcodeId::LessThan64, 0xdc),
    (OpcodeId::LessThanOrEqual64, 0xdd),
    (OpcodeId::GreaterThan64, 0xde),
    (OpcodeId::GreaterThanOrEqual64, 0xdf),
    (OpcodeId::ScriptNumToLe64, 0xe0),
    (OpcodeId::Le64ToScriptNum, 0xe1),
    (OpcodeId::Le32ToLe64, 0xe2),
    (OpcodeId::EcMulScalarVerify, 0xe3),
    (OpcodeId::TweakVerify, 0xe4),
];

#[test]
fn the_expectation_covers_the_whole_census_exactly_once() {
    let expected: BTreeSet<OpcodeId> = EXPECTED_BYTES.iter().map(|(id, _)| *id).collect();
    let census: BTreeSet<OpcodeId> = OpcodeId::ALL.iter().copied().collect();

    assert_eq!(expected, census);
    assert_eq!(EXPECTED_BYTES.len(), OpcodeId::ALL.len());
}

#[test]
fn every_primitive_serializes_to_its_expected_byte() {
    let target = reviewed_target();

    for (id, expected) in EXPECTED_BYTES {
        let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(*id)])
            .expect("one instruction is within the limit");
        assert_eq!(program.encode(&target), vec![*expected], "{id:?}");
    }
}

#[test]
fn every_expected_byte_parses_back_to_its_own_primitive() {
    let target = reviewed_target();

    for (id, expected) in EXPECTED_BYTES {
        let program =
            TapscriptProgram::decode(&target, &[*expected]).expect("the byte is reviewed");
        assert_eq!(
            program.instructions(),
            [TapscriptInstruction::Opcode(*id)],
            "{expected:#04x}",
        );
    }
}

#[test]
fn no_two_primitives_share_a_byte_and_no_byte_names_two_primitives() {
    let mut by_byte: BTreeMap<u8, OpcodeId> = BTreeMap::new();

    for (id, expected) in EXPECTED_BYTES {
        assert!(
            by_byte.insert(*expected, *id).is_none(),
            "byte {expected:#04x} is claimed twice",
        );
    }
    assert_eq!(by_byte.len(), OpcodeId::ALL.len());
}

#[test]
fn the_whole_census_round_trips_as_one_program() {
    // Determinism in the shape that matters: the bytes follow the
    // instruction sequence, so a program holding every primitive
    // encodes and decodes back to itself.
    let target = reviewed_target();
    let instructions: Vec<TapscriptInstruction> = OpcodeId::ALL
        .iter()
        .map(|id| TapscriptInstruction::Opcode(*id))
        .collect();
    let program = TapscriptProgram::new(instructions).expect("the census is within the limit");

    let bytes = program.encode(&target);
    assert_eq!(bytes.len(), OpcodeId::ALL.len());
    assert_eq!(
        TapscriptProgram::decode(&target, &bytes).expect("the census parses"),
        program,
    );
    assert_eq!(
        program.encode(&target),
        bytes,
        "encoding twice gives the same bytes",
    );
}

#[test]
fn the_bytes_between_the_reviewed_ones_are_refused() {
    // Every byte that is neither a push form nor a reviewed primitive
    // must fail, including the ones that sit between reviewed
    // primitives and the extension bytes above the reviewed range.
    let target = reviewed_target();
    let reviewed: BTreeSet<u8> = EXPECTED_BYTES.iter().map(|(_, byte)| *byte).collect();
    let pushes = target.definition().pushes().occupied_opcodes();

    let mut refused = 0_usize;
    for byte in 0x00_u8..=0xff {
        if reviewed.contains(&byte) || pushes.contains(&byte) {
            continue;
        }
        assert!(
            TapscriptProgram::decode(&target, &[byte]).is_err(),
            "byte {byte:#04x} parses",
        );
        refused += 1;
    }
    assert!(refused > 100, "the sweep covered {refused} bytes");
}
