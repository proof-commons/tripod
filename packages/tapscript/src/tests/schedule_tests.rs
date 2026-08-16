//! Machine-checked symbolic stack schedules for the Guide-10
//! prototypes.
//!
//! # What a schedule proves, and what it does not
//!
//! A schedule here is a typed program, a typed initial stack, and the
//! complete set of states the reviewed contracts say it can reach. The
//! abstract validator computes those states from the primitive
//! contracts alone; nothing in this file executes anything, and a
//! passing schedule is a statement about the contracts rather than
//! about a node (Guide-10 `rule:guide10:stack-schedule`).
//!
//! What each schedule establishes is exactly the list §7.3 asks for:
//! that every operand exists and is admitted, that every successful
//! alternative is accounted for, that no state reached through a
//! failure rejoins the successful ones, that every arithmetic success
//! flag is consumed, and that the surviving depth is the intended one.
//!
//! # Failure states are separated by the validator, not by assertion
//!
//! The validator carries a reached-through-failure bit alongside each
//! state, so a state that arrived by way of a non-aborting failure
//! stays in its own set even when its stack shape coincides with a
//! successful one. That is what makes "no failure state rejoins
//! success" a checked property here rather than a claim.
//!
//! # Why the arithmetic flags are what force the substrate
//!
//! Every fixed-width arithmetic primitive pushes a success flag above
//! its result, and on overflow it leaves both operands in place and
//! pushes a false. Before the compound-proof substrate was reviewed
//! there was no primitive that could consume that flag, so no
//! multi-step arithmetic schedule could be written at all: each step
//! left a flag behind and the stack grew without bound. Boolean
//! verification is the primitive that closes it, and the schedules
//! below are the demonstration.

use std::collections::BTreeSet;

use target_elements::{FailureCause, OpcodeId, StackValueType};

use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::TapscriptProgram;
use crate::stack::{AbstractExecutionResult, AbstractLimits, AbstractStackState, validate_program};
use crate::tests::reviewed_target;

/// The wide-floor base, `2^26`.
///
/// The limb base Guide-10 §11.3 fixes for the derived-limb candidate.
/// Every intermediate the candidate computes stays below `2^53`, well
/// inside the target's signed 64-bit arithmetic, which is what makes
/// the decomposition exact.
const LIMB_BASE: i64 = 1 << 26;

/// The abstract type a literal of `width` bytes carries.
const fn literal(width: usize) -> StackValueType {
    StackValueType::Bytes {
        minimum: width,
        maximum: width,
    }
}

/// The abstract type of the target's signed fixed-width integer.
fn signed64() -> StackValueType {
    let target = reviewed_target();
    let spec = target
        .definition()
        .opcodes()
        .get(&OpcodeId::Add64)
        .expect("the reviewed contract states a contract for addition");
    // Read from the contract rather than restated, so a schedule cannot
    // drift away from the type the primitive actually pushes.
    spec.stack()
        .success()
        .cases()
        .first()
        .expect("addition has a successful form")
        .effect()
        .computed_types()
        .first()
        .cloned()
        .expect("addition pushes a result")
}

/// One reviewed primitive, as an instruction.
const fn op(id: OpcodeId) -> TapscriptInstruction {
    TapscriptInstruction::Opcode(id)
}

/// Runs one schedule against the reviewed contracts.
fn schedule(
    instructions: Vec<TapscriptInstruction>,
    initial: &AbstractStackState,
) -> AbstractExecutionResult {
    let target = reviewed_target();
    let program = TapscriptProgram::new(instructions).expect("the schedule is within the limit");
    let limits = AbstractLimits::for_target(&target);
    validate_program(&target, &program, initial, limits).expect("the schedule validates")
}

/// A signed fixed-width literal, as an instruction.
fn value(number: i64) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(StackItem::signed_le64(&target, number))
}

/// A script-number literal, as an instruction.
fn number(value: i64) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(
        StackItem::script_number(&target, value).expect("a small script number is admissible"),
    )
}

/// The states a set of stacks describes.
fn states(stacks: &[Vec<StackValueType>]) -> BTreeSet<AbstractStackState> {
    stacks
        .iter()
        .map(|main| AbstractStackState::from_main(main.clone()))
        .collect()
}

/// The causes every reviewed primitive can abort on regardless of what
/// it is handed.
fn domain_abort() -> BTreeSet<FailureCause> {
    BTreeSet::from([FailureCause::UnsupportedExecutionDomain])
}

// -- The wide-floor prototype, Candidate A ------------------------

#[test]
fn a_limb_decomposition_leaves_exactly_two_limbs() {
    // The candidate derives limbs by dividing rather than trusting a
    // caller to supply them (Guide-10 `candidate:guide10:derived-limbs`).
    //
    // Division pushes a remainder, a quotient, and a success flag. The
    // flag must be consumed before anything else can be scheduled on
    // top, and Boolean verification is what consumes it — the whole
    // reason the substrate had to be reviewed before this schedule
    // could exist.
    let result = schedule(
        vec![
            value(7),
            value(LIMB_BASE),
            op(OpcodeId::Div64),
            op(OpcodeId::Verify),
        ],
        &AbstractStackState::from_main(Vec::new()),
    );

    // The low limb and the high limb, and nothing else. The flag is
    // gone, and no proof-local value survives.
    assert_eq!(result.success(), &states(&[vec![signed64(), signed64()]]));

    // The divide-by-zero and overflow paths retain both operands and
    // push a false, so verification finds a false beneath a stack of
    // literals. The shape differs from the successful one, and the
    // validator keeps it apart regardless.
    assert_eq!(
        result.nonaborting_failure(),
        &states(&[vec![literal(8), literal(8)]])
    );

    // No successful state is also a failure state: the two sets are
    // disjoint, which is the property §7.3 asks a schedule to prove.
    assert!(
        result
            .success()
            .intersection(result.nonaborting_failure())
            .next()
            .is_none()
    );

    // Verification aborts on a false operand. That is the whole point
    // of using it: the flag cannot be ignored.
    let mut expected = domain_abort();
    expected.insert(FailureCause::FalseVerification);
    assert_eq!(result.aborts(), &expected);
}

#[test]
fn a_partial_product_normalizes_to_a_digit_and_a_carry() {
    // One coefficient of the schoolbook product, normalized: multiply
    // two limbs, then split the product into a digit below the base and
    // a carry above it (Guide-10 §11.3).
    //
    // Two arithmetic flags arise and both are consumed immediately,
    // which is what keeps the depth constant across the step.
    let initial = AbstractStackState::from_main(vec![signed64(), signed64()]);
    let result = schedule(
        vec![
            op(OpcodeId::Mul64),
            op(OpcodeId::Verify),
            value(LIMB_BASE),
            op(OpcodeId::Div64),
            op(OpcodeId::Verify),
        ],
        &initial,
    );

    // The digit and the carry. Two operands in, two results out.
    assert_eq!(result.success(), &states(&[vec![signed64(), signed64()]]));

    assert!(
        result
            .success()
            .intersection(result.nonaborting_failure())
            .next()
            .is_none()
    );

    let mut expected = domain_abort();
    expected.insert(FailureCause::FalseVerification);
    assert_eq!(result.aborts(), &expected);
}

#[test]
fn an_unconsumed_arithmetic_flag_is_visible_as_a_deeper_stack() {
    // The negative control for the two schedules above. Without the
    // verification the flag stays, and the step leaves three items
    // rather than two — which is exactly the mis-scheduling the
    // reviewed contract's asymmetric failure behaviour causes when a
    // caller assumes a primitive pushes only its result.
    let initial = AbstractStackState::from_main(vec![signed64(), signed64()]);
    let result = schedule(vec![op(OpcodeId::Mul64)], &initial);

    let states_left: Vec<usize> = result.success().iter().map(|s| s.main().len()).collect();
    assert_eq!(states_left, vec![2]);

    // And the overflow path is deeper still: both operands retained,
    // with a false above them.
    let failure_depths: Vec<usize> = result
        .nonaborting_failure()
        .iter()
        .map(|s| s.main().len())
        .collect();
    assert_eq!(failure_depths, vec![3]);
}

#[test]
fn an_exact_limb_comparison_consumes_both_limbs() {
    // The candidate finishes by comparing all four normalized limbs of
    // one product against the other's. Byte equality is exact for these
    // operands: a signed fixed-width item is a canonical eight-byte
    // encoding, so two of them are equal as numbers exactly when they
    // are equal as bytes.
    //
    // The verifying form is what a proof wants — an unequal pair must
    // end the spend, not leave a false to be reduced later.
    let initial = AbstractStackState::from_main(vec![signed64(), signed64()]);
    let result = schedule(vec![op(OpcodeId::EqualVerify)], &initial);

    assert_eq!(result.success(), &states(&[Vec::new()]));
    assert!(result.nonaborting_failure().is_empty());

    let mut expected = domain_abort();
    expected.insert(FailureCause::UnequalOperands);
    assert_eq!(result.aborts(), &expected);
}

#[test]
fn the_candidate_a_inner_step_holds_its_depth_across_repetition() {
    // The property that makes the full candidate schedulable at all: the
    // normalize-and-carry step neither grows nor shrinks the working
    // stack, so repeating it for each coefficient stays within the
    // target's depth bound rather than accumulating flags.
    let initial = AbstractStackState::from_main(vec![signed64(), signed64()]);

    let mut instructions = Vec::new();
    for _ in 0..4 {
        instructions.extend([
            op(OpcodeId::Mul64),
            op(OpcodeId::Verify),
            value(LIMB_BASE),
            op(OpcodeId::Div64),
            op(OpcodeId::Verify),
            // Restore the pair the next iteration multiplies, so the
            // step is measured in isolation rather than against a stack
            // the previous one happened to leave.
            op(OpcodeId::Drop),
            value(3),
        ]);
    }

    let result = schedule(instructions, &initial);
    for state in result.success() {
        assert_eq!(state.main().len(), 2, "the step holds its depth");
    }
    assert!(!result.success().is_empty());
}

// -- The constructor prototype's canonical ordering ---------------

#[test]
fn a_reviewed_primitive_does_not_order_two_digests() {
    // The finding that shapes the constructor candidate
    // (Guide-10 `rule:guide10:tapbranch-order`).
    //
    // Branch construction needs the two children in canonical
    // byte-lexicographic order. No reviewed primitive orders byte
    // strings: the fixed-width comparisons take eight-byte signed
    // integers and refuse anything else, so handing them two
    // thirty-two byte digests is a typed refusal rather than an
    // ordering.
    let target = reviewed_target();
    let digest = literal(32);
    let program = TapscriptProgram::new(vec![op(OpcodeId::LessThan64)])
        .expect("one instruction is within the limit");
    let initial = AbstractStackState::from_main(vec![digest.clone(), digest]);

    let outcome = validate_program(
        &target,
        &program,
        &initial,
        AbstractLimits::for_target(&target),
    );
    assert!(
        outcome.is_err(),
        "no reviewed comparison accepts two digests"
    );
}

#[test]
fn a_four_byte_chunk_comparison_is_expressible() {
    // Canonical ordering is not blocked by primitive availability, and
    // this is why. Reading a four-byte chunk of a digest as an unsigned
    // number and comparing it is expressible in reviewed primitives:
    // the widening conversion reads the chunk unsigned and
    // zero-extends, so the result is never negative and the signed
    // comparison is a correct unsigned one.
    //
    // Ordering two whole digests is then this step, applied per chunk,
    // combined so that the first differing chunk decides. That
    // combination is arithmetic on values in {0,1} and needs no
    // conditional branch.
    //
    // What this schedule does *not* do is take the chunks out of the
    // digests, which is the part that is not yet schedulable — see the
    // test below.
    let initial = AbstractStackState::from_main(vec![literal(4), literal(4)]);
    let result = schedule(
        vec![
            op(OpcodeId::Le32ToLe64),
            op(OpcodeId::Swap),
            op(OpcodeId::Le32ToLe64),
            op(OpcodeId::Swap),
            op(OpcodeId::LessThan64),
        ],
        &initial,
    );

    // One truth value, and the operands are gone.
    assert_eq!(result.success().len(), 1);
    for state in result.success() {
        assert_eq!(state.main().len(), 1);
    }
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn a_sliced_chunk_schedules_into_a_width_constrained_operand() {
    // The step that used to block a complete constructor schedule.
    //
    // Slicing four bytes out of a digest produces a result whose width
    // the reviewed contract states as its length operand's value, so a
    // schedule that slices with a literal bound carries a settled
    // four-byte item into the widening conversion — which is exactly
    // what the target does. Before the width relation existed the
    // contract reported an unconstrained byte string here and the
    // composition was refused, even though no target ever refused it
    // (Guide-10 `rule:guide10:stack-schedule`).
    let result = schedule(
        vec![
            number(0),
            number(4),
            op(OpcodeId::Substring),
            op(OpcodeId::Le32ToLe64),
        ],
        &AbstractStackState::from_main(vec![literal(32)]),
    );

    // One widened chunk; the digest and both bounds are gone.
    assert_eq!(result.success(), &states(&[vec![signed64()]]));
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn a_slice_of_the_wrong_width_is_still_refused() {
    // The relation narrows a result; it does not admit one. A five-byte
    // slice is a five-byte item, and the widening conversion takes
    // four, so the composition is refused exactly where the target
    // would refuse it.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        number(0),
        number(5),
        op(OpcodeId::Substring),
        op(OpcodeId::Le32ToLe64),
    ])
    .expect("the schedule is within the limit");
    let initial = AbstractStackState::from_main(vec![literal(32)]);

    assert!(
        validate_program(
            &target,
            &program,
            &initial,
            AbstractLimits::for_target(&target),
        )
        .is_err(),
        "a five-byte slice does not satisfy a four-byte position"
    );
}

#[test]
fn a_slice_whose_length_the_program_did_not_fix_stays_unsettled() {
    // The negative control for the repair, and its honest floor.
    //
    // Where the length operand is not a literal the program pushed —
    // here it arrives on the initial stack — the walk settles nothing,
    // the result keeps the contract's unconstrained type, and the
    // composition is refused rather than assumed. The width relation is
    // a statement about bounds a program fixed, and it claims nothing
    // whatever about a bound it did not.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![op(OpcodeId::Substring), op(OpcodeId::Le32ToLe64)])
        .expect("the schedule is within the limit");
    let initial = AbstractStackState::from_main(vec![
        literal(32),
        StackValueType::ScriptNumber,
        StackValueType::ScriptNumber,
    ]);

    assert!(
        validate_program(
            &target,
            &program,
            &initial,
            AbstractLimits::for_target(&target),
        )
        .is_err(),
        "an unsettled length settles no width"
    );
}

#[test]
fn a_chunk_of_each_digest_is_compared_end_to_end() {
    // The Wave-3 finding, composed: read the same four-byte chunk out
    // of each of two digests, widen both unsigned, and compare them.
    //
    // This is the whole of canonical ordering except the combination
    // step: applied per chunk and combined so that the first differing
    // chunk decides, it orders two digests with no conditional branch
    // (Guide-10 `rule:guide10:tapbranch-order`).
    let digest = literal(32);
    let initial = AbstractStackState::from_main(vec![digest.clone(), digest]);
    let result = schedule(
        vec![
            // The deeper digest's chunk, widened.
            op(OpcodeId::CopyOver),
            number(0),
            number(4),
            op(OpcodeId::Substring),
            op(OpcodeId::Le32ToLe64),
            // The shallower digest's chunk, widened.
            op(OpcodeId::CopyOver),
            number(0),
            number(4),
            op(OpcodeId::Substring),
            op(OpcodeId::Le32ToLe64),
            op(OpcodeId::LessThan64),
        ],
        &initial,
    );

    // Both digests survive for the next chunk, with one truth value
    // above them: the step is repeatable, which is what makes the
    // per-chunk ordering schedulable at all.
    assert_eq!(
        result.success(),
        &states(&[vec![literal(32), literal(32), StackValueType::Bool]])
    );
    assert!(result.nonaborting_failure().is_empty());
}
