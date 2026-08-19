//! Guide-12 preflight reproductions owned by this crate.
//!
//! Every test here demonstrates a row of the Guide-12 preflight register
//! by *passing* while the defect is present: it asserts the wrong thing
//! happens and names the row it belongs to. Wave 0 reproduces and does
//! not repair, so the wave that fixes a row flips that row's assertions,
//! which then stand as the guarantee that the repair holds. Nothing here
//! reaches a crate-private constructor an external caller could not use.
//!
//! - `G12-R11` — the resource projection omits every byte a literal
//!   occupies.
//! - `G12-R12` — a pushed literal's exact bytes are not retained, so a
//!   verifying primitive over a byte the target reads as false keeps a
//!   success state.
//! - `G12-R13` — decoding runs its parse loop over the whole input and
//!   applies the instruction limit only in the constructor it finally
//!   calls.

use target_elements::{OpcodeId, ResourceDimension};

use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::{MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptProgram};
use crate::stack::{AbstractLimits, AbstractStackState, resource_projection, validate_program};

use super::reviewed_target;

/// `G12-R11`: a literal's bytes do not reach the projected script size.
///
/// The projection walks the instruction sequence and skips every
/// instruction that is not a primitive, so the push opcode, any width
/// prefix, and the whole payload contribute nothing. What remains is a
/// per-primitive tally, and the gap against the program's own encoded
/// length is the size of the payloads.
///
/// The assertion is the defect: `ScriptBytes` is the primitive count
/// while the encoded program is two payloads longer. A wave that derives
/// script bytes from the exact encoded length flips it.
#[test]
fn a_pushed_payload_contributes_no_projected_script_bytes() {
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

    let projected = resource_projection(&target, &program)
        .get(&ResourceDimension::ScriptBytes)
        .copied()
        .expect("the projection states the dimension");
    let encoded = u64::try_from(program.encode(&target).len()).expect("the program is small");

    // One primitive is priced, and the ninety-eight bytes the two
    // literals occupy are not.
    assert_eq!(projected, 1);
    assert_eq!(encoded, 99);
    assert!(
        projected < encoded,
        "G12-R11: the projection is expected to undercount while the row is open",
    );
}

/// `G12-R12`: a pushed zero byte leaves an unreachable success state.
///
/// A literal enters the abstract state as a width range, and its value is
/// carried only where the payload is a minimal script number. A single
/// zero byte is not one — zero's minimal form is the empty push — so the
/// walk records no constant for the position. The verifying primitive
/// then has to admit both outcomes, and the success branch survives.
///
/// On the target the byte is read as false and the primitive always
/// aborts, so no success state exists. The assertion is the defect: the
/// success set is non-empty. A wave that retains exact known bytes, or
/// that narrows the API so no liveness conclusion may be drawn from the
/// success set, flips it.
#[test]
fn a_pushed_zero_byte_leaves_a_success_state_the_target_cannot_reach() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        TapscriptInstruction::Push(
            StackItem::new(&target, vec![0x00]).expect("one byte is within the bound"),
        ),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ])
    .expect("two instructions are within the limit");

    let result = validate_program(
        &target,
        &program,
        &AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(&target),
    )
    .expect("the fixture validates");

    assert!(
        !result.success().is_empty(),
        "G12-R12: the success set is expected to hold an unreachable state while the row is open",
    );
}

/// `G12-R13`: decoding passes the work bound before it notices the limit.
///
/// The parse loop runs to the end of the input and the instruction limit
/// is applied only by the constructor the loop finally calls, so a script
/// far above the limit is parsed and allocated in full before it is
/// refused. The witness is a script whose bytes pass the limit long
/// before an unreadable byte arrives: a parser that stopped at the work
/// bound would report the limit, and this one reports the byte, which it
/// can only have reached by continuing.
///
/// The assertion is the defect. A wave that stops at the bound flips it
/// to [`TapscriptError::InstructionLimitExceeded`].
#[test]
fn decoding_passes_the_work_bound_before_it_notices_the_limit() {
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
        Err(TapscriptError::UnknownOpcodeByte(unreadable)),
        "G12-R13: the parser is expected to run past the limit while the row is open",
    );
}
