//! The relative timelock.
//!
//! # A successful check leaves the operand where it found it
//!
//! The primitive inspects its operand and pushes nothing, so a script
//! that offers exactly one operand and checks it finishes with exactly
//! that operand — one item, whose own truth is the script's. That is why
//! a satisfied lock over a nonzero age accepts and a satisfied lock over
//! a zero age completes false: the retention is the observation.
//!
//! # What the check compares
//!
//! The operand is compared with the *input's own sequence field*, which
//! the fixture states, and the transaction version is the prerequisite.
//! Nothing here depends on how old a funding output is, so every case is
//! materializable without waiting for anything (Guide-9 §13.12).
//!
//! # No cadence
//!
//! A relative timelock is a target fact about one input. No
//! attestation-contract cadence band, epoch, or schedule appears here
//! or may.

use tapscript::TapscriptInstruction;
use target_elements::OpcodeId;

use crate::census::author::{Case, CensusAuthor, falsity, op, push};
use crate::census::context::timelock_transaction;
use crate::fixture::NativeCaseGroup;
use crate::protocol::ObservedFailureClass;

/// The version below which the primitive refuses every lock.
const VERSION_BELOW_MINIMUM: u32 = 1;

/// The smallest version the primitive admits.
const MINIMUM_VERSION: u32 = 2;

/// A relative lock of ten blocks, as an input carries it.
const HEIGHT_SEQUENCE: u32 = 10;

/// The bit that selects the time-interval mode.
const TIME_MODE_FLAG: u32 = 0x0040_0000;

/// A relative lock of five intervals, as an input carries it.
const TIME_SEQUENCE: u32 = TIME_MODE_FLAG | 5;

/// The largest age the sequence field's own mask can express.
const MASK_BOUNDARY: u32 = 0x0000_ffff;

/// The sequence that disables the check.
const DISABLED_SEQUENCE: u32 = 0xffff_ffff;

/// Every relative-timelock case.
pub fn cases(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::CheckSequenceVerify;
    let group = NativeCaseGroup::RelativeTimelock;
    let script = [op(id)];

    // Satisfied: at the age the input carries, one below it, and at the
    // mask's boundary. The operand is retained, so it is the script's
    // whole final stack.
    for (sequence, operand) in [
        (HEIGHT_SEQUENCE, i64::from(HEIGHT_SEQUENCE)),
        (HEIGHT_SEQUENCE, i64::from(HEIGHT_SEQUENCE) - 1),
        (TIME_SEQUENCE, i64::from(TIME_SEQUENCE)),
        (MASK_BOUNDARY, i64::from(MASK_BOUNDARY)),
    ] {
        let stack = [author.number(operand)];
        let expected = vec![author.number_bytes(operand)];
        author.accept(
            timelock_case(group, id, &script, &stack, MINIMUM_VERSION, sequence),
            expected,
        );
    }

    // Satisfied by a zero age, whose retained operand is the target's
    // false: the check succeeded and the script's value did not.
    let stack = [author.number(0)];
    author.evaluated_false(
        timelock_case(group, id, &script, &stack, MINIMUM_VERSION, HEIGHT_SEQUENCE),
        vec![falsity()],
    );

    // Unsatisfied: one above the age, the wrong mode, a version below
    // the prerequisite, and an input whose sequence disables the check.
    for (version, sequence, operand) in [
        (
            MINIMUM_VERSION,
            HEIGHT_SEQUENCE,
            i64::from(HEIGHT_SEQUENCE) + 1,
        ),
        (MINIMUM_VERSION, HEIGHT_SEQUENCE, i64::from(TIME_SEQUENCE)),
        (MINIMUM_VERSION, TIME_SEQUENCE, i64::from(HEIGHT_SEQUENCE)),
        (
            VERSION_BELOW_MINIMUM,
            HEIGHT_SEQUENCE,
            i64::from(HEIGHT_SEQUENCE),
        ),
        (
            MINIMUM_VERSION,
            DISABLED_SEQUENCE,
            i64::from(HEIGHT_SEQUENCE),
        ),
    ] {
        let stack = [author.number(operand)];
        author.reject(
            timelock_case(group, id, &script, &stack, version, sequence),
            &[ObservedFailureClass::UnsatisfiedTimelock],
        );
    }

    // A negative operand is refused before the comparison.
    let stack = [author.number(-1)];
    author.reject(
        timelock_case(group, id, &script, &stack, MINIMUM_VERSION, HEIGHT_SEQUENCE),
        &[ObservedFailureClass::NegativeTimelock],
    );

    // A trailing zero byte is not a minimal script number.
    let stack = [author.item(vec![0x0a, 0x00])];
    author.reject(
        timelock_case(group, id, &script, &stack, MINIMUM_VERSION, HEIGHT_SEQUENCE),
        &[ObservedFailureClass::MalformedScriptNumber],
    );

    author.reject(
        timelock_case(group, id, &script, &[], MINIMUM_VERSION, HEIGHT_SEQUENCE),
        &[ObservedFailureClass::StackUnderflow],
    );

    // The check leaves the stack as it found it, which a following
    // primitive can then read: the operand is still a script number.
    let script = [
        op(id),
        op(OpcodeId::ScriptNumToLe64),
        push(author.le64(i64::from(HEIGHT_SEQUENCE) - 1)),
        op(OpcodeId::GreaterThan64),
    ];
    let stack = [author.number(i64::from(HEIGHT_SEQUENCE))];
    author.accept(
        timelock_case(group, id, &script, &stack, MINIMUM_VERSION, HEIGHT_SEQUENCE),
        vec![crate::census::author::truth()],
    );
}

/// One case against a transaction at one version and one sequence.
fn timelock_case<'a>(
    group: NativeCaseGroup,
    id: OpcodeId,
    script: &'a [TapscriptInstruction],
    stack: &'a [tapscript::StackItem],
    version: u32,
    sequence: u32,
) -> Case<'a> {
    Case {
        group,
        opcode: Some(id),
        script,
        stack,
        context: Some(timelock_transaction(version, sequence)),
    }
}
