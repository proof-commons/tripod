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
//! the fixture states, and the transaction version is the prerequisite
//! (Guide-9 §13.12).
//!
//! # A satisfied lock is still a lock
//!
//! What the primitive compares does not depend on how old a funding
//! output is — but the transaction carrying the lock does. A relative
//! lock is enforced by the chain as well as read by the script, so an
//! input younger than its own sequence is refused before the
//! interpreter is reached, and the executor has to age the input to
//! what the fixture declared. That is cheap in blocks and free in
//! seconds, and it is why the ages stated here are the ages they are.
//!
//! # No cadence
//!
//! A relative timelock is a target fact about one input. No
//! attestation-contract cadence band, epoch, or schedule appears here
//! or may.

use tapscript::TapscriptInstruction;
use target_elements::{EncodingClass, OpcodeId};

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

/// The mask's largest age, counted in intervals rather than blocks.
///
/// The mask is the same sixteen bits in both modes, so a satisfied lock
/// at its top is the same fact either way — but only one of the two can
/// be materialized. Aging an input by sixty-five thousand blocks is a
/// chain a run cannot build; aging it by the same count of intervals is
/// a clock the executor moves forward at no cost. The boundary is
/// therefore stated in the mode that can be reached, and the height
/// mode's own boundary is a recorded residual rather than a case that
/// answers with infrastructure trouble.
const TIME_MASK_BOUNDARY: u32 = TIME_MODE_FLAG | MASK_BOUNDARY;

/// The sequence that disables the check.
const DISABLED_SEQUENCE: u32 = 0xffff_ffff;

/// The operand bit that makes the check do nothing at all.
///
/// Set on the *operand* rather than the input, it is the target's
/// forward-compatibility escape: the primitive returns before it
/// compares anything, so neither the age nor the mode is consulted. The
/// bit sits above the thirty-two bit field the operand is compared
/// against, which is why stating it at all needs the lock-time
/// script-number width and why no case here could be written until the
/// operand was typed at it.
const OPERAND_DISABLE_FLAG: u32 = 0x8000_0000;

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
        (TIME_MASK_BOUNDARY, i64::from(TIME_MASK_BOUNDARY)),
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

    // An operand carrying the disable flag is not a lock at all: the
    // primitive returns before comparing anything, so the operand is
    // retained and its own truth — it is a large positive number — is
    // the script's. Both cases would be rejections without the flag:
    // the first names an age far above the input's, and the second
    // names the interval mode against an input counting blocks.
    for operand in [
        i64::from(OPERAND_DISABLE_FLAG),
        i64::from(OPERAND_DISABLE_FLAG | TIME_SEQUENCE),
    ] {
        let bytes = lock_time_number_bytes(operand);
        let stack = [author.encoded(EncodingClass::LockTimeScriptNumber, bytes.clone())];
        author.accept(
            timelock_case(group, id, &script, &stack, MINIMUM_VERSION, HEIGHT_SEQUENCE),
            vec![bytes],
        );
    }

    // A negative operand is refused before the comparison.
    let stack = [author.number(-1)];
    author.reject(
        timelock_case(group, id, &script, &stack, MINIMUM_VERSION, HEIGHT_SEQUENCE),
        &[ObservedFailureClass::NegativeTimelock],
    );

    // A trailing zero byte is not a minimal script number — and
    // minimality is a relay rule, not the target's own: at consensus the
    // operand is read as the age it encodes and the lock is satisfied.
    let stack = [author.item(vec![0x0a, 0x00])];
    author.reject_at_relay(
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

/// The target's minimal signed encoding of one lock-time operand.
///
/// Stated here rather than taken from the typed script-number
/// constructor, which is bounded to the ordinary four-byte width: an
/// operand carrying the disable flag is exactly the value that does not
/// fit there, and the whole point of the case is that the target reads
/// this operand one byte wider.
fn lock_time_number_bytes(value: i64) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut remaining = value.unsigned_abs();
    while remaining > 0 {
        bytes.push(u8::try_from(remaining & 0xff).unwrap_or(0));
        remaining >>= 8;
    }
    // A leading byte with its high bit set would read as negative, so a
    // positive number takes one more byte rather than one fewer.
    if bytes.last().is_some_and(|byte| byte & 0x80 != 0) {
        bytes.push(0x00);
    }
    bytes
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
