//! The parser mutation matrix.
//!
//! Each mutation below takes a script the parser accepts and changes
//! exactly one thing about it: the opcode byte, the stated width, the
//! payload, where the script ends. Every one must fail, and must fail
//! with the reason that names what was changed — a parser that reported
//! one reason for all of them would be telling a reader nothing.

use target_elements::OpcodeId;

use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::{MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptProgram};

use super::reviewed_target;

/// A payload of `width` bytes that no numeric form ever carries.
fn filler(width: usize) -> Vec<u8> {
    vec![0xab; width]
}

/// A script the parser accepts: a literal, then a primitive.
fn accepted() -> Vec<u8> {
    let target = reviewed_target();
    let item = StackItem::new(&target, filler(80)).expect("the payload is within the bound");
    TapscriptProgram::new(vec![
        TapscriptInstruction::Push(item),
        TapscriptInstruction::Opcode(OpcodeId::Sha256Initialize),
    ])
    .expect("two instructions are within the limit")
    .encode(&target)
}

#[test]
fn the_unmutated_script_parses() {
    let target = reviewed_target();
    let script = accepted();

    // A width-prefixed push of eighty bytes, then one primitive byte.
    assert_eq!(script.len(), 1 + 1 + 80 + 1);
    let program = TapscriptProgram::decode(&target, &script).expect("the script is reviewed");
    assert_eq!(program.len(), 2);
}

#[test]
fn mutating_the_opcode_byte_is_refused() {
    let target = reviewed_target();
    let mut script = accepted();
    let last = script.len() - 1;
    // One byte below the lowest reviewed extension byte.
    script[last] = 0xc3;

    assert_eq!(
        TapscriptProgram::decode(&target, &script),
        Err(TapscriptError::UnknownOpcodeByte(0xc3)),
    );
}

#[test]
fn mutating_the_stated_width_is_refused() {
    let target = reviewed_target();

    // Narrower than the payload present: the width now describes a
    // shorter literal, and the bytes after it decode as instructions
    // that are not there.
    let mut narrower = accepted();
    narrower[1] = 79;
    assert!(TapscriptProgram::decode(&target, &narrower).is_err());

    // One byte wider swallows the primitive that followed, which is
    // still a well-formed script and a different program: a stated
    // width is not self-describing, which is exactly why a caller must
    // not treat a parsed program as evidence about the bytes it did not
    // write.
    let mut swallowing = accepted();
    swallowing[1] = 81;
    let swallowed =
        TapscriptProgram::decode(&target, &swallowing).expect("the wider push is well formed");
    assert_eq!(swallowed.len(), 1);

    // Wider than the script itself: it now ends inside the push.
    let mut wider = accepted();
    wider[1] = 82;
    assert_eq!(
        TapscriptProgram::decode(&target, &wider),
        Err(TapscriptError::TruncatedInstruction),
    );
}

#[test]
fn mutating_the_payload_leaves_a_different_program() {
    // A payload byte carries no structure, so changing one must not
    // fail: it must produce a different, equally valid program. A
    // parser that refused here would be reading meaning into a literal.
    let target = reviewed_target();
    let script = accepted();
    let mut mutated = script.clone();
    mutated[5] ^= 0xff;

    let original = TapscriptProgram::decode(&target, &script).expect("the script is reviewed");
    let changed =
        TapscriptProgram::decode(&target, &mutated).expect("a literal is still a literal");

    assert_ne!(original, changed);
    assert_eq!(changed.encode(&target), mutated);
}

#[test]
fn truncating_the_script_anywhere_inside_the_push_is_refused() {
    let target = reviewed_target();
    let script = accepted();

    // Every proper prefix that ends inside the push fails, and the one
    // that ends exactly after it parses as a shorter program.
    for end in 1..script.len() - 1 {
        assert_eq!(
            TapscriptProgram::decode(&target, &script[..end]),
            Err(TapscriptError::TruncatedInstruction),
            "prefix of {end} bytes",
        );
    }
    assert!(TapscriptProgram::decode(&target, &script[..script.len() - 1]).is_ok());
}

#[test]
fn a_trailing_partial_instruction_is_refused() {
    // The push form's own byte with nothing after it: the script ends
    // in the middle of the instruction that byte began.
    let target = reviewed_target();

    for opening in [
        vec![0x4c_u8],
        vec![0x4d],
        vec![0x4d, 0x02],
        vec![0x4e, 0x00],
    ] {
        assert_eq!(
            TapscriptProgram::decode(&target, &opening),
            Err(TapscriptError::TruncatedInstruction),
            "{opening:02x?}",
        );
    }

    // The same, appended to a script that is otherwise complete.
    let mut script = accepted();
    script.push(0x02);
    script.push(0xab);
    assert_eq!(
        TapscriptProgram::decode(&target, &script),
        Err(TapscriptError::TruncatedInstruction),
    );
}

#[test]
fn an_unknown_extension_byte_is_refused() {
    let target = reviewed_target();

    // Immediately above the reviewed extension range, and the byte the
    // upstream enumeration reserves for an invalid opcode.
    for byte in [0xe5_u8, 0xff] {
        assert_eq!(
            TapscriptProgram::decode(&target, &[byte]),
            Err(TapscriptError::UnknownOpcodeByte(byte)),
        );
    }
}

#[test]
fn a_script_of_too_many_instructions_is_refused() {
    let target = reviewed_target();
    let count = usize::try_from(MAXIMUM_PROGRAM_INSTRUCTIONS).expect("the limit fits a usize");

    let at_limit = vec![0x51_u8; count];
    assert!(TapscriptProgram::decode(&target, &at_limit).is_ok());

    let past_limit = vec![0x51_u8; count + 1];
    assert_eq!(
        TapscriptProgram::decode(&target, &past_limit),
        Err(TapscriptError::InstructionLimitExceeded {
            maximum: MAXIMUM_PROGRAM_INSTRUCTIONS,
        }),
    );
    assert_eq!(
        TapscriptProgram::new(vec![
            TapscriptInstruction::Opcode(OpcodeId::TxWeight);
            count + 1
        ]),
        Err(TapscriptError::InstructionLimitExceeded {
            maximum: MAXIMUM_PROGRAM_INSTRUCTIONS,
        }),
    );
}

#[test]
fn an_empty_script_is_an_empty_program() {
    let target = reviewed_target();
    let program = TapscriptProgram::decode(&target, &[]).expect("an empty script is well formed");

    assert!(program.is_empty());
    assert_eq!(program.encode(&target), [] as [u8; 0]);
}
