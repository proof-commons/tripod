//! Guide-12 preflight reproductions owned by this crate.
//!
//! Each test here belongs to a row of the Guide-12 preflight register.
//! While a row is open its test passes by asserting the wrong thing
//! happens, which is what Wave 0 recorded: it reproduces and does not
//! repair. The wave that fixes a row flips that row's assertions, and
//! they then stand as the guarantee that the repair holds. A row marked
//! CLOSED below is one whose test has been flipped. Nothing here reaches
//! a crate-private constructor an external caller could not use.
//!
//! - `G12-R11` — CLOSED: script bytes are the program's exact encoded
//!   length, so every byte a literal occupies is projected.
//! - `G12-R12` — CLOSED: a pushed literal's exact bytes are retained, so
//!   the target's own reading of them decides both the truth a verifying
//!   primitive checks and the equality a comparing one checks.
//! - `G12-R13` — CLOSED: the parse loop applies the instruction limit
//!   itself, so a script above the bound is refused by the bound rather
//!   than by whatever its later bytes happen to be.

use std::collections::BTreeSet;

use target_elements::{
    FailureCause, OpcodeId, ResourceDimension, ReviewedElementsTapscriptDefinition,
};

use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::{MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptProgram};
use crate::stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, resource_projection,
    validate_program,
};

use super::reviewed_target;

/// `G12-R11`: a literal's bytes reach the projected script size.
///
/// The projection used to walk the instruction sequence and skip every
/// instruction that was not a primitive, so the push opcode, any width
/// prefix, and the whole payload contributed nothing; what remained was
/// a per-primitive tally, and the gap against the program's own encoded
/// length was the size of the payloads.
///
/// The assertion is now the guarantee, in two independent ways. The
/// first program's expected length is arithmetic done here rather than
/// read from the encoder — a sixty-four-byte and a thirty-two-byte
/// literal each cost their payload and one opcode byte, and the
/// signature check costs one — so a projection agreeing with a broken
/// encoder would still fail. The rest are a census of push forms whose
/// projection is compared with the bytes the program actually
/// serializes to.
#[test]
fn every_pushed_payload_reaches_the_projected_script_bytes() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        TapscriptInstruction::Push(
            StackItem::new(&target, vec![0xab; 64]).expect("the payload is within the bound"),
        ),
        TapscriptInstruction::Push(
            StackItem::new(&target, vec![0xcd; 32]).expect("the payload is within the bound"),
        ),
        TapscriptInstruction::Opcode(OpcodeId::CheckSig),
    ])
    .expect("three instructions are within the limit");

    // (64 + 1) + (32 + 1) + 1, computed here and not asked of the
    // encoder.
    assert_eq!(projected_script_bytes(&target, &program), 99);
    assert_eq!(program.encoded_length(&target), 99);

    // Every push form the reviewed rule can choose: a payload carried in
    // the opcode, a direct push, and each width-prefixed form. Widths
    // are chosen either side of the form boundaries rather than at round
    // numbers, so a projection that priced one form's prefix wrongly
    // could not hide behind another's.
    for width in [0, 1, 32, 64, 75, 76, 77, 254, 255, 256, 520] {
        let program = TapscriptProgram::new(vec![
            TapscriptInstruction::Push(
                StackItem::new(&target, vec![0x5a; width])
                    .expect("every width here is within the literal bound"),
            ),
            TapscriptInstruction::Opcode(OpcodeId::Verify),
        ])
        .expect("two instructions are within the limit");

        let encoded = u64::try_from(program.encode(&target).len()).expect("the program is small");
        assert_eq!(
            projected_script_bytes(&target, &program),
            encoded,
            "G12-R11: a {width}-byte literal must be projected at the bytes it serializes to",
        );
    }
}

/// The script bytes one program is projected to occupy.
fn projected_script_bytes(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> u64 {
    resource_projection(target, program)
        .get(&ResourceDimension::ScriptBytes)
        .copied()
        .expect("the projection states the dimension")
}

/// `G12-R12`: an exact pushed literal settles the verified truth value.
///
/// A literal used to enter the abstract state as a width range, its value
/// carried only where the payload was a minimal script number. A single
/// zero byte is not one — zero's minimal form is the empty push — so the
/// walk recorded nothing for the position, the verifying primitive had to
/// admit both outcomes, and the success branch survived a program the
/// target always aborts.
///
/// The walk now keeps the item's bytes, so the target's own reading
/// decides both directions. The first four rows are the required
/// regressions of `(´[PLAN-tab:guide12-exec:tapscript-tests]´)` — the
/// empty item,
/// the zero byte, the negative zero, and a true one — and the rest are
/// their siblings: the wider all-zero payloads and the trailing sign
/// bytes that are equally false and equally not minimal numbers, and the
/// near neighbours of each that are true. Every expected reading is
/// written out rather than recomputed from the same rule the crate
/// applies, so a rule that is wrong in both places still fails here.
///
/// A false literal reaches no success state and does abort; a true one
/// reaches exactly one and can then end only on the domain gate, which
/// is a claim the previous behaviour could not make in either direction.
#[test]
fn an_exact_pushed_literal_settles_the_verified_truth_value() {
    let target = reviewed_target();

    for (payload, verifies) in [
        (vec![], false),
        (vec![0x00], false),
        (vec![0x80], false),
        (vec![0x01], true),
        (vec![0x00, 0x00], false),
        (vec![0x00, 0x80], false),
        (vec![0x00; 32], false),
        (vec![0x80, 0x00], true),
        (vec![0x00, 0x01], true),
        (vec![0x81], true),
    ] {
        let result = verified(&target, payload.clone());

        assert_eq!(
            !result.success().is_empty(),
            verifies,
            "G12-R12: a success state must survive {payload:02x?} exactly when the target's \
             reading of it is true",
        );
        assert_eq!(
            result.aborts().contains(&FailureCause::FalseVerification),
            !verifies,
            "G12-R12: the false-verification abort must survive {payload:02x?} exactly when \
             the target's reading of it is false",
        );
        if verifies {
            assert_eq!(
                result.success(),
                &BTreeSet::from([AbstractStackState::from_main(Vec::new())]),
                "a verified literal is consumed and leaves the stack empty",
            );
            assert_eq!(
                result.aborts(),
                &BTreeSet::from([FailureCause::UnsupportedExecutionDomain]),
                "the only remaining way to end this program's evaluation is the domain \
                 gate every primitive carries",
            );
        }
    }
}

/// `G12-R12`: two exact pushed literals settle the compared equality.
///
/// The other half of the row, and the remaining two required regressions
/// of `(´[PLAN-tab:guide12-exec:tapscript-tests]´)`. Both operands are
/// known byte
/// for byte, so the comparison the verifying primitive makes is one the
/// walk can make: operands that differ reach no success state and do
/// abort, and operands that agree reach one and cannot abort.
#[test]
fn two_exact_pushed_literals_settle_the_compared_equality() {
    let target = reviewed_target();

    for (left, right) in [(0x01_u8, 0x02_u8), (0x01, 0x01)] {
        let equal = left == right;
        let program = TapscriptProgram::new(vec![
            TapscriptInstruction::Push(
                StackItem::new(&target, vec![left]).expect("one byte is within the bound"),
            ),
            TapscriptInstruction::Push(
                StackItem::new(&target, vec![right]).expect("one byte is within the bound"),
            ),
            TapscriptInstruction::Opcode(OpcodeId::EqualVerify),
        ])
        .expect("three instructions are within the limit");
        let result = validate(&target, &program);

        assert_eq!(
            !result.success().is_empty(),
            equal,
            "G12-R12: {left:#04x} against {right:#04x} must reach a success state exactly \
             when the two are the same bytes",
        );
        assert_eq!(
            result.aborts().contains(&FailureCause::UnequalOperands),
            !equal,
            "G12-R12: the inequality abort must survive {left:#04x} against {right:#04x} \
             exactly when the two differ",
        );
    }
}

/// The result of pushing one literal and verifying it.
fn verified(
    target: &ReviewedElementsTapscriptDefinition,
    payload: Vec<u8>,
) -> AbstractExecutionResult {
    let program = TapscriptProgram::new(vec![
        TapscriptInstruction::Push(
            StackItem::new(target, payload).expect("every payload here is within the bound"),
        ),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ])
    .expect("two instructions are within the limit");
    validate(target, &program)
}

/// One program's abstract result from an empty initial stack.
fn validate(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> AbstractExecutionResult {
    validate_program(
        target,
        program,
        &AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(target),
    )
    .expect("the fixture validates")
}

/// `G12-R13`: decoding stops at the work bound before it reads further.
///
/// The parse loop used to run to the end of the input, the instruction
/// limit being applied only by the constructor the loop finally called,
/// so a script far above the limit was parsed and allocated in full
/// before it was refused. The witness is a script whose bytes pass the
/// limit long before an unreadable byte arrives: a parser that stops at
/// the work bound reports the limit, and one that reports the byte can
/// only have reached it by continuing past the bound.
///
/// The assertion is now the guarantee: the limit is what comes back, so
/// the unreadable byte at the end was never read.
#[test]
fn decoding_stops_at_the_work_bound_before_it_reads_further() {
    let target = reviewed_target();

    // Resolved from the contract rather than restated: one primitive's
    // own byte, and the first byte the parser reads as neither a
    // primitive nor a push form.
    let primitive = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Verify)])
        .expect("one instruction is within the limit")
        .encode(&target);
    let [primitive_byte] = primitive[..] else {
        panic!("a primitive encodes as exactly one byte")
    };
    let unreadable = (0..=u8::MAX)
        .find(|byte| {
            matches!(
                TapscriptProgram::decode(&target, &[*byte]),
                Err(TapscriptError::UnknownOpcodeByte(_))
            )
        })
        .expect("some byte is neither a primitive nor a push form");

    let beyond = usize::try_from(MAXIMUM_PROGRAM_INSTRUCTIONS).expect("the limit is small") + 1;
    let mut script = vec![primitive_byte; beyond];
    script.push(unreadable);

    assert_eq!(
        TapscriptProgram::decode(&target, &script),
        Err(TapscriptError::InstructionLimitExceeded {
            maximum: MAXIMUM_PROGRAM_INSTRUCTIONS,
        }),
        "G12-R13: the parse stops at the work bound, so the trailing byte is never read",
    );
}
