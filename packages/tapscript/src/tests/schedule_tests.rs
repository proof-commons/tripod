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

use target_elements::{EncodingClass, FailureCause, OpcodeId, StackValueType};

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
    // push a false — and the verification then finds that false and
    // ends evaluation. So no failure path survives this schedule at
    // all, which is the whole reason a proof verifies the flag instead
    // of dropping it.
    //
    // The validator establishes this rather than being told it: the
    // false is a literal the abstract state knows is the empty item,
    // and a primitive that aborts on a false operand has no successful
    // form there.
    assert!(result.nonaborting_failure().is_empty());

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

/// An arbitrary byte string, as an instruction.
fn raw(bytes: Vec<u8>) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(StackItem::new(&target, bytes).expect("within the literal bound"))
}

#[test]
fn a_boolean_verdict_does_not_enter_the_arithmetic_domain() {
    // The Wave-3 residual, stated exactly.
    //
    // A comparison pushes the target's truth value, and every
    // fixed-width arithmetic primitive takes eight-byte operands. So a
    // verdict cannot be added, multiplied, or otherwise combined with
    // another verdict: the composition is a typed refusal, not a
    // narrower result.
    //
    // Every scheme that orders two digests by comparing them chunk by
    // chunk and combining the per-chunk verdicts arithmetically dies
    // here. The repair below is not a widening primitive; it is a
    // construction that never produces a second verdict to combine.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![op(OpcodeId::LessThan64), op(OpcodeId::Add64)])
        .expect("the schedule is within the limit");
    let initial = AbstractStackState::from_main(vec![signed64(), signed64(), signed64()]);

    assert!(
        validate_program(
            &target,
            &program,
            &initial,
            AbstractLimits::for_target(&target),
        )
        .is_err(),
        "a truth value does not satisfy a fixed-width arithmetic operand"
    );
}

#[test]
fn one_verdict_is_consumed_where_it_is_produced() {
    // The shape the repair relies on. A single comparison feeding the
    // verification that consumes it needs no widening at all: the
    // verdict never becomes an operand of anything, so the domain it
    // cannot enter is never entered.
    let result = schedule(
        vec![op(OpcodeId::LessThan64), op(OpcodeId::Verify)],
        &AbstractStackState::from_main(vec![signed64(), signed64()]),
    );

    assert_eq!(result.success(), &states(&[Vec::new()]));
    assert!(result.nonaborting_failure().is_empty());

    let mut expected = domain_abort();
    expected.insert(FailureCause::FalseVerification);
    assert_eq!(result.aborts(), &expected);
}

#[test]
fn a_witnessed_number_is_pinned_to_one_digest_byte() {
    // The step that replaces the missing widening
    // (Guide-10 `rule:guide10:tapbranch-order`).
    //
    // A single byte of a digest cannot be read as a number: the
    // reviewed widening conversion takes four bytes, and nothing
    // narrows one. So the caller witnesses the byte's numeric form and
    // the program *proves* it: the script-number conversion produces a
    // canonical eight-byte value, its low byte must equal the digest
    // byte at the witnessed index, and its remaining seven bytes must
    // be zero.
    //
    // The zero tail is what makes the witness harmless. It pins the
    // number into `0..=255`, so a caller cannot offer a negative or
    // oversized value and win a comparison the bytes do not support.
    let result = schedule(
        vec![
            // The digest byte at the witnessed index. The length is a
            // literal, so the slice's width is settled even though its
            // offset is not.
            number(1),
            op(OpcodeId::Substring),
            op(OpcodeId::Swap),
            op(OpcodeId::ScriptNumToLe64),
            // Seven zero bytes above the low one.
            op(OpcodeId::Duplicate),
            number(1),
            number(7),
            op(OpcodeId::Substring),
            raw(vec![0; 7]),
            op(OpcodeId::EqualVerify),
            // The low byte is the digest byte.
            op(OpcodeId::Duplicate),
            number(0),
            number(1),
            op(OpcodeId::Substring),
            op(OpcodeId::Rotate),
            op(OpcodeId::EqualVerify),
        ],
        // Deepest first: the witnessed number, the digest, the
        // witnessed index.
        &AbstractStackState::from_main(vec![
            StackValueType::ScriptNumber,
            literal(32),
            StackValueType::ScriptNumber,
        ]),
    );

    // The pinned value, in the domain the comparison accepts.
    assert_eq!(result.success(), &states(&[vec![signed64()]]));
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn an_orientation_limb_is_pinned_to_the_two_admissible_masks() {
    // The other half of the repair: which way round the two children
    // go is a witness, and this is the check that leaves it exactly two
    // choices.
    //
    // The limb `m` must satisfy `m * (m + 1) = 0`, whose only integer
    // roots are `0` and `-1` — the all-zero and all-ones eight-byte
    // masks. A limb outside that pair either fails the equality or
    // overflows the multiplication, and the overflow path pushes a
    // false that the verification then aborts on, so neither survives.
    //
    // Repeated into a thirty-two byte mask, those two values select
    // between the two child orders with no conditional branch: one
    // leaves the pair alone and the other exchanges it.
    let result = schedule(
        vec![
            op(OpcodeId::Duplicate),
            op(OpcodeId::Duplicate),
            value(1),
            op(OpcodeId::Add64),
            op(OpcodeId::Verify),
            op(OpcodeId::Mul64),
            op(OpcodeId::Verify),
            value(0),
            op(OpcodeId::EqualVerify),
        ],
        &AbstractStackState::from_main(vec![signed64()]),
    );

    assert_eq!(result.success(), &states(&[vec![signed64()]]));
    assert!(result.nonaborting_failure().is_empty());

    let mut expected = domain_abort();
    expected.insert(FailureCause::FalseVerification);
    expected.insert(FailureCause::UnequalOperands);
    assert_eq!(result.aborts(), &expected);
}

#[test]
fn no_reviewed_primitive_reads_below_the_third_item() {
    // The constraint that governs every schedule in this file, stated
    // once and machine-checked.
    //
    // The reviewed census has no positional copy, no positional move,
    // and no alternate-stack transfer. So the deepest item any
    // primitive can read is the third, and a value a program must keep
    // while it consumes something beneath it has to be kept within
    // reach of that window. It is a property of the reviewed contracts
    // rather than a convention, and it is what decides whether a
    // compound proof can be scheduled at all.
    let target = reviewed_target();
    let deepest = target
        .definition()
        .opcodes()
        .values()
        .map(|spec| spec.stack().operands().len())
        .max()
        .expect("the reviewed contract states at least one primitive");

    assert_eq!(
        deepest, 3,
        "no reviewed primitive declares a fourth operand"
    );

    // And nothing moves an item to or from the alternate stack, so the
    // window cannot be widened by parking a value.
    for spec in target.definition().opcodes().values() {
        for case in spec.stack().success().cases() {
            assert!(
                case.effect().consumed_operands() <= 3,
                "a reviewed primitive consumes at most the top three items"
            );
        }
    }
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

// -- The constructor prototype's predecessor proof ----------------

/// The metadata leaf's constant preimage prefix.
///
/// Both tag digests, the leaf version, the compact-size length, and the
/// push opcode introducing the metadata are fixed by the schema, so the
/// program pushes them as one literal rather than assembling them.
const LEAF_PREFIX_BYTES: usize = 32 + 32 + 1 + 1 + 1;

/// The metadata leaf script's constant tail, a false and a verify.
const LEAF_TAIL_BYTES: usize = 2;

/// The metadata schema's object width.
///
/// Restated from the constructor oracle's schema rather than imported,
/// because this crate is beneath it and must not depend on it. The test
/// below is what keeps the two from drifting: it emits the leaf script
/// from this width and checks the framing the schedules assume.
const METADATA_BYTES: usize = 48;

/// The widest script a one-byte compact size can introduce.
///
/// A length below this is written as itself in one byte; at it and above
/// a compact size grows a marker and a payload, and the leaf preimage
/// would carry more framing than the schedules allow for.
const ONE_BYTE_COMPACT_SIZE_LIMIT: usize = 253;

/// Both tag digests of one tagged hash.
const TAG_PREFIX_BYTES: usize = 64;

/// Both tag digests of the tweak hash, and the internal key after them.
const TWEAK_PREFIX_BYTES: usize = TAG_PREFIX_BYTES + 32;

/// The metadata leaf's script, as the reviewed builder emits it.
///
/// The same three instructions the constructor oracle's leaf carries:
/// the metadata as one literal, a false, and the verification that ends
/// evaluation on it.
fn metadata_leaf_script(metadata_bytes: usize) -> Vec<u8> {
    let target = reviewed_target();
    TapscriptProgram::new(vec![
        TapscriptInstruction::Push(
            StackItem::new(&target, vec![0; metadata_bytes])
                .expect("the metadata is within the literal bound"),
        ),
        TapscriptInstruction::Push(StackItem::empty()),
        op(OpcodeId::Verify),
    ])
    .expect("three instructions are within the limit")
    .encode(&target)
}

#[test]
fn the_leaf_framing_constants_are_the_emitted_ones() {
    // The schedules above stream the metadata leaf's preimage as a
    // constant prefix, the metadata, and a constant tail. Those two
    // widths were asserted rather than derived when the schedules were
    // written, and a wrong one would make every schedule a statement
    // about a leaf no constructor builds.
    //
    // So they are computed here from the bytes the reviewed builder
    // actually emits. The prefix is both tag digests, the leaf version
    // byte, the script's compact-size length, and whatever the builder
    // puts in front of the metadata to push it; the tail is everything
    // the builder puts after it.
    let script = metadata_leaf_script(METADATA_BYTES);

    // The metadata appears once, contiguously, and the script is framing
    // around it.
    let metadata_at = script
        .windows(METADATA_BYTES)
        .position(|window| window == vec![0_u8; METADATA_BYTES])
        .expect("the emitted script carries the metadata");
    let push_framing = metadata_at;
    let tail = script.len() - metadata_at - METADATA_BYTES;

    // One byte of compact size, which holds only while the whole script
    // is shorter than the marker threshold.
    assert!(script.len() < ONE_BYTE_COMPACT_SIZE_LIMIT);
    let compact_size_bytes = 1;
    let leaf_version_bytes = 1;

    assert_eq!(
        LEAF_PREFIX_BYTES,
        TAG_PREFIX_BYTES + leaf_version_bytes + compact_size_bytes + push_framing,
        "the streamed prefix is the emitted framing"
    );
    assert_eq!(
        LEAF_TAIL_BYTES, tail,
        "the streamed tail is the emitted framing"
    );

    // And the whole preimage the schedules stream is the whole preimage
    // the leaf hash covers.
    assert_eq!(
        LEAF_PREFIX_BYTES + METADATA_BYTES + LEAF_TAIL_BYTES,
        TAG_PREFIX_BYTES + leaf_version_bytes + compact_size_bytes + script.len()
    );
}

#[test]
fn a_wider_metadata_object_would_move_the_leaf_framing() {
    // The negative control, and the reason the width above is not
    // incidental. A metadata object wide enough to need a longer push
    // encoding shifts the prefix, so the schedules are statements about
    // one schema width rather than about metadata in general.
    let narrow = metadata_leaf_script(METADATA_BYTES).len() - METADATA_BYTES;
    let wide = metadata_leaf_script(255).len() - 255;
    assert!(
        wide > narrow,
        "a wider object needs more framing than the schedules allow for"
    );
}

/// The predecessor half, ending with the curve step's operands placed.
///
/// Leaves, deepest first, the retained static root, the compressed
/// predecessor output key, and the derived tweak.
fn predecessor_proof() -> Vec<TapscriptInstruction> {
    vec![
        // -- Bind the witnessed output key to the consumed program.
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::InspectInputScriptPubKey),
        number(1),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::Swap),
        op(OpcodeId::Duplicate),
        number(1),
        number(32),
        op(OpcodeId::Substring),
        op(OpcodeId::Rotate),
        op(OpcodeId::EqualVerify),
        // -- The metadata leaf hash.
        op(OpcodeId::Swap),
        raw(vec![0; LEAF_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        raw(vec![0; LEAF_TAIL_BYTES]),
        op(OpcodeId::Sha256Finalize),
        // -- The branch hash, in the fixed child order.
        raw(vec![0; TAG_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        // The static root is hashed from a copy, so the authenticated
        // instance survives (Guide-10 `rule:guide10:static-root`).
        op(OpcodeId::Rotate),
        op(OpcodeId::Duplicate),
        op(OpcodeId::Rotate),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
        // -- The tweak.
        raw(vec![0; TWEAK_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
        // -- Place the curve step's operands, root left beneath them.
        op(OpcodeId::Rotate),
        op(OpcodeId::Swap),
    ]
}

/// The successor half, from the retained root to the successor tweak.
///
/// Consumes the retained static root rather than a second witnessed
/// one, which is what makes the composition a continuity proof
/// (Guide-10 `rule:guide10:static-root`).
fn successor_proof() -> Vec<TapscriptInstruction> {
    vec![
        op(OpcodeId::Swap),
        raw(vec![0; LEAF_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        raw(vec![0; LEAF_TAIL_BYTES]),
        op(OpcodeId::Sha256Finalize),
        raw(vec![0; TAG_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        // No copy is kept: the one authenticated root is consumed here,
        // by the successor's own branch hash.
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
        raw(vec![0; TWEAK_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
    ]
}

/// The successor output binding at one exact role.
fn output_binding() -> Vec<TapscriptInstruction> {
    vec![
        number(0),
        op(OpcodeId::InspectOutputScriptPubKey),
        number(1),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::Rotate),
        op(OpcodeId::Duplicate),
        number(1),
        number(32),
        op(OpcodeId::Substring),
        op(OpcodeId::Rotate),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::Swap),
    ]
}

/// The curve step: the internal key, then the tweak verification.
///
/// Until the tweak position was corrected against the target this was a
/// model of the instruction's declared stack effect rather than the
/// instruction — the derived tweak was typed as a digest and the
/// position was declared as one exact encoding, so the composition was
/// refused here. The position now says what the target says, and the
/// composed schedule carries the real primitive
/// (`a_derived_digest_is_admitted_as_a_tweak_operand`).
///
/// The internal key is a fixed thirty-two byte literal the program
/// pushes: the prototype's key is a published nothing-up-my-sleeve
/// point, so it is a constant of the program rather than a witness.
fn curve_step() -> Vec<TapscriptInstruction> {
    vec![raw(vec![0; 32]), op(OpcodeId::TweakVerify)]
}

/// Proves one metadata field identical in both objects.
///
/// Reads the same constant-width slice out of each and requires byte
/// equality, leaving both objects in place so the checks compose.
fn unchanged_field(begin: i64, length: i64) -> Vec<TapscriptInstruction> {
    vec![
        op(OpcodeId::DuplicateTwo),
        number(begin),
        number(length),
        op(OpcodeId::Substring),
        op(OpcodeId::Swap),
        number(begin),
        number(length),
        op(OpcodeId::Substring),
        op(OpcodeId::EqualVerify),
    ]
}

/// Proves the successor's reserved field is exactly zero.
fn reserved_is_zero() -> Vec<TapscriptInstruction> {
    vec![
        op(OpcodeId::Duplicate),
        number(40),
        number(8),
        op(OpcodeId::Substring),
        raw(vec![0; 8]),
        op(OpcodeId::EqualVerify),
    ]
}

#[test]
fn the_transition_moves_one_field_and_pins_the_rest() {
    // Stage C6
    // (Guide-10 `rule:guide10:constructor-transition-stage`).
    //
    // The schema's field order is what makes this expressible: domain,
    // schema, and object kind are contiguous, so one constant-width
    // slice pins all three, and every remaining field is its own slice
    // at a fixed offset. Nothing is read at a caller-chosen offset, so
    // no witness can move a field boundary.
    //
    // # The counter's flag is consumed where it is produced
    //
    // The increment is the only arithmetic here, and its success flag is
    // verified immediately rather than dropped. An overflowing counter
    // retains both operands and pushes a false, which the verification
    // then ends evaluation on — so a wrapped counter cannot reach the
    // equality and pass it (Guide-10 `rule:guide10:successor-metadata`).
    //
    // # Why the nonce is absent
    //
    // The representation nonce is deliberately unchecked. It is not
    // state: the host oracle's `same_state` ignores it and `successor`
    // resets it, and its value is ground by the creator to canonicalize
    // the branch order. The program that checked it equal would reject
    // every honest successor; the program that checked it reset would
    // duplicate a constraint the branch comparison already enforces.
    let mut instructions = Vec::new();
    // Domain, schema, and object kind, in one slice.
    instructions.extend(unchanged_field(0, 24));
    // The counter, incremented by exactly one.
    instructions.extend([
        op(OpcodeId::DuplicateTwo),
        number(24),
        number(8),
        op(OpcodeId::Substring),
        op(OpcodeId::Swap),
        number(24),
        number(8),
        op(OpcodeId::Substring),
        value(1),
        op(OpcodeId::Add64),
        op(OpcodeId::Verify),
        op(OpcodeId::EqualVerify),
    ]);
    // The flags.
    instructions.extend(unchanged_field(32, 4));
    // The reserved field of each object.
    instructions.extend(reserved_is_zero());
    instructions.push(op(OpcodeId::Swap));
    instructions.extend(reserved_is_zero());
    instructions.push(op(OpcodeId::Swap));

    let result = schedule(
        instructions,
        // Deepest first: the predecessor metadata, then the successor.
        &AbstractStackState::from_main(vec![literal(48), literal(48)]),
    );

    // Both objects survive, unconsumed: the transition proof is a
    // constraint on them, not a replacement for them, and each is still
    // needed to derive its own constructor.
    assert_eq!(result.success(), &states(&[vec![literal(48), literal(48)]]));
    assert!(result.nonaborting_failure().is_empty());

    // The counter's overflow path does not survive, and the two
    // inequality causes are the ones a mutated field reaches.
    let mut expected = domain_abort();
    expected.insert(FailureCause::FalseVerification);
    expected.insert(FailureCause::UnequalOperands);
    expected.insert(FailureCause::SliceOutOfRange);
    expected.insert(FailureCause::MalformedScriptNumber);
    assert_eq!(result.aborts(), &expected);
}

#[test]
fn a_counter_slice_of_the_wrong_width_is_refused() {
    // The negative control for the field reads. A counter slice that is
    // not eight bytes wide does not satisfy the arithmetic operand, so a
    // schema drift that moved the field would be a refusal here rather
    // than a silently different number.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        number(24),
        number(4),
        op(OpcodeId::Substring),
        value(1),
        op(OpcodeId::Add64),
    ])
    .expect("the schedule is within the limit");
    let initial = AbstractStackState::from_main(vec![literal(48)]);

    assert!(
        validate_program(
            &target,
            &program,
            &initial,
            AbstractLimits::for_target(&target),
        )
        .is_err(),
        "a four-byte slice does not satisfy a fixed-width operand"
    );
}

#[test]
fn the_curve_step_consumes_exactly_its_three_operands() {
    // The contract the model above stands on, read through the
    // validator rather than restated: given operands of the declared
    // types, the tweak verification leaves nothing behind.
    let result = schedule(
        vec![op(OpcodeId::TweakVerify)],
        &AbstractStackState::from_main(vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            StackValueType::Encoded(EncodingClass::TaprootTweak),
            StackValueType::Encoded(EncodingClass::XOnlyPublicKey),
        ]),
    );

    assert_eq!(result.success(), &states(&[Vec::new()]));
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn the_successor_proof_consumes_the_one_authenticated_root() {
    // Stage C4 (Guide-10 `rule:guide10:constructor-successor-stage`).
    //
    // The successor half never reads a static root of its own. It
    // begins holding the root the predecessor proof authenticated and
    // retained, and consumes it in its own branch hash, so the two
    // constructors are bound to the same value by construction rather
    // than by comparing two witnesses
    // (Guide-10 `rule:guide10:static-root`).
    //
    // # The nonce needs no check here
    //
    // The successor's representation nonce is ground by the creator
    // until the successor metadata leaf hash falls on the fixed side of
    // the same static root. Nothing in the program inspects it: the
    // check *is* the successor branch comparison succeeding, because a
    // nonce that failed to canonicalize the order yields a different
    // root, a different tweak, and a created program that does not
    // match the one the output actually carries.
    let result = schedule(
        successor_proof(),
        // Deepest first: the compressed successor output key, the
        // successor metadata, and the retained authenticated root.
        &AbstractStackState::from_main(vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            literal(48),
            literal(32),
        ]),
    );

    assert_eq!(
        result.success(),
        &states(&[vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            StackValueType::Encoded(EncodingClass::Sha256Digest),
        ]])
    );
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn the_successor_binding_names_one_exact_output_role() {
    // The created program is read at one stated role rather than
    // searched for among the outputs, so an instance that puts the
    // successor somewhere else does not satisfy this program
    // (Guide-10 `rule:guide10:successor-constructor`).
    let result = schedule(
        output_binding(),
        &AbstractStackState::from_main(vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            StackValueType::Encoded(EncodingClass::Sha256Digest),
        ]),
    );

    // The curve step's first two operands, in its declared order.
    assert_eq!(
        result.success(),
        &states(&[vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            StackValueType::Encoded(EncodingClass::Sha256Digest),
        ]])
    );
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn the_continuity_composition_carries_one_root_across_both_halves() {
    // Stage C5, the composition
    // (Guide-10 `rule:guide10:constructor-continuity-stage`).
    //
    // The open question was whether the reach bound permits it at all:
    // no reviewed primitive reads below the third item, so a program may
    // hold at most two computed values and still reach its next witness,
    // and the composed proof has to keep a static root alive across an
    // entire second constructor derivation.
    //
    // It does, and the arrangement is forced rather than chosen. The
    // root is the deepest of the three values the predecessor half
    // retains, every later witness sits beneath it in consumption order,
    // and the successor half consumes the root last. No second root is
    // witnessed, so the split-root instance has nothing to supply: there
    // is no second value to disagree with the first.
    let mut instructions = predecessor_proof();
    instructions.extend(curve_step());
    instructions.extend(successor_proof());
    instructions.extend(output_binding());
    instructions.extend(curve_step());
    // The final truth value a standalone spend needs.
    instructions.push(number(1));

    let result = schedule(
        instructions,
        // Deepest first, in reverse consumption order: the successor
        // key, the successor metadata, the one static root, the
        // predecessor metadata, the predecessor key.
        &AbstractStackState::from_main(vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            literal(48),
            literal(32),
            literal(48),
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
        ]),
    );

    // One item, canonically true, and nothing proof-local left behind.
    // The literal is typed by the width the program fixed for it, which
    // is what a final truth value has to be: a one-byte item the target
    // reads as true.
    assert_eq!(result.success(), &states(&[vec![literal(1)]]));
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn the_predecessor_proof_schedules_and_leaves_one_authenticated_root() {
    // Stage C3, end to end
    // (Guide-10 `rule:guide10:constructor-predecessor-stage`).
    //
    // # Why no orientation witness appears
    //
    // Canonical child ordering is enforced without being computed. The
    // program hashes the two children in one fixed order, and the
    // instance is spendable only when that order is the canonical one —
    // because the tweak it derives is compared against the consumed
    // input's actual program, and a tree whose canonical order differs
    // yields a different root and fails that comparison.
    //
    // Ordering is therefore a property the creator establishes, not one
    // the caller asserts: the schema's representation nonce is ground
    // until the metadata leaf hash falls on the fixed side of the
    // static root, the same public deterministic retry §9.12 already
    // admits for tweak totality. Nothing is hashed in caller order, no
    // order bit is trusted, and no verdict is combined
    // (Guide-10 `rule:guide10:tapbranch-order`).
    //
    // # Why the witness order is what it is
    //
    // No reviewed primitive reads below the third item, so a witness is
    // reachable only while fewer than three computed values sit above
    // it. The layout below is the consumption order: the compressed key
    // is bound first, the metadata is consumed next, and the static
    // root stays deepest because it must outlive both.
    let result = schedule(
        predecessor_proof(),
        // Deepest first: the static root, the predecessor metadata, and
        // the compressed predecessor output key.
        &AbstractStackState::from_main(vec![
            literal(32),
            literal(48),
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
        ]),
    );

    // The retained static root, beneath the two operands the curve
    // check consumes above it. The root's position is the point: it is
    // deeper than everything the predecessor half still needs, so it
    // survives that check rather than being consumed by it.
    assert_eq!(
        result.success(),
        &states(&[vec![
            literal(32),
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            StackValueType::Encoded(EncodingClass::Sha256Digest),
        ]])
    );

    // Both introspection alternatives were carried and both reached the
    // same shape: the program never assumed the consumed output was a
    // witness program, and the equality discriminating them is one the
    // target performs.
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn a_derived_digest_is_admitted_as_a_tweak_operand() {
    // The step the abstract validator used to refuse, corrected against
    // the target rather than worked around.
    //
    // Until this wave the tweak position was declared as one exact
    // encoding, so a derived digest did not satisfy it and the
    // predecessor proof could not schedule its own curve check. The
    // target has no such rule. Its entire operand guard is
    // `vchTweak.size() != 32`, and what the thirty-two bytes mean is
    // decided afterwards, inside `CheckPayToContract`
    // (`src/script/interpreter.cpp:2206-2220`). The contract now says
    // that, so the composition the target performs is the composition
    // this validator schedules.
    //
    // # This is a correction, not a relaxation
    //
    // Nothing here promises the derived tweak is a scalar. The residual
    // is unchanged and is still the target's to decide: the rare
    // instance whose tweak is at or above the group order fails inside
    // the curve arithmetic, which is exactly where the target fails it
    // (Guide-10 `rule:guide10:tweak-totality`). What changed is that a
    // refusal the target does not make is no longer made here.
    let result = schedule(
        vec![op(OpcodeId::TweakVerify)],
        &AbstractStackState::from_main(vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            StackValueType::Encoded(EncodingClass::Sha256Digest),
            StackValueType::Encoded(EncodingClass::XOnlyPublicKey),
        ]),
    );

    assert_eq!(result.success(), &states(&[Vec::new()]));
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn an_operand_of_the_wrong_width_is_still_refused_as_a_tweak() {
    // The negative control for the correction, and the reason it is not
    // a widening.
    //
    // The position admits thirty-two bytes and nothing else, so a
    // fixed-width integer, a script number, or a literal of any other
    // length is refused exactly where the target refuses it — at the
    // size guard, before any curve arithmetic happens.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![op(OpcodeId::TweakVerify)])
        .expect("one instruction is within the limit");

    for wrong in [signed64(), literal(31), literal(33), literal(64)] {
        let initial = AbstractStackState::from_main(vec![
            StackValueType::Encoded(EncodingClass::CompressedPublicKey),
            wrong.clone(),
            StackValueType::Encoded(EncodingClass::XOnlyPublicKey),
        ]);
        assert!(
            validate_program(
                &target,
                &program,
                &initial,
                AbstractLimits::for_target(&target),
            )
            .is_err(),
            "an item of the wrong width does not satisfy the tweak position: {wrong:?}"
        );
    }
}

#[test]
fn an_unsettled_width_does_not_satisfy_the_tweak_position() {
    // The honest floor of the correction. Where the abstract state has
    // not fixed the item's width, the item could be one of the widths
    // the target refuses, and a validator that admitted it would be
    // claiming a fact the state does not carry.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![op(OpcodeId::TweakVerify)])
        .expect("one instruction is within the limit");
    let initial = AbstractStackState::from_main(vec![
        StackValueType::Encoded(EncodingClass::CompressedPublicKey),
        StackValueType::Bytes {
            minimum: 0,
            maximum: 64,
        },
        StackValueType::Encoded(EncodingClass::XOnlyPublicKey),
    ]);

    assert!(
        validate_program(
            &target,
            &program,
            &initial,
            AbstractLimits::for_target(&target),
        )
        .is_err(),
        "an unsettled width settles no operand"
    );
}

// -- The transition composed into the continuity proof -------------

/// How much of the object the recipe pins to its own constants: the
/// domain and the schema, which are contiguous and come first.
const RECIPE_PREFIX_BYTES: i64 = 20;

/// How wide the contiguous domain, schema, and object-kind prefix is.
///
/// Restated from the constructor oracle's schema for the reason
/// [`METADATA_BYTES`] is: this crate is beneath the oracle and must not
/// depend on it. Drift is caught by the framing test above and by the
/// emitted program's own schedule.
const DOMAIN_THROUGH_KIND_BYTES: i64 = 24;

/// Where the counter begins.
const COUNTER_AT: i64 = 24;

/// The counter's width, which is the fixed-width arithmetic width.
const COUNTER_BYTES: i64 = 8;

/// Where the flags field begins.
const FLAGS_AT: i64 = 32;

/// The flags field's width.
const FLAGS_BYTES: i64 = 4;

/// The representation nonce's width.
const NONCE_BYTES: i64 = 4;

/// Where the reserved field begins.
const RESERVED_AT: i64 = 40;

/// The reserved field's width.
const RESERVED_BYTES: i64 = 8;

/// Derives the successor metadata from the predecessor's own bytes.
///
/// # Why the successor is derived rather than witnessed
///
/// A witnessed successor has to be compared against the predecessor
/// field by field, and that comparison needs both objects adjacent
/// while the continuity layout needs the one static root between them.
/// No reviewed primitive reads below the third item, so the two
/// requirements cannot both hold, and the composition was previously
/// recorded as a checked absence.
///
/// Deriving removes the comparison rather than rearranging it. Every
/// unchanged field is a slice of the predecessor object, the counter is
/// that object's own counter incremented under a verified success flag,
/// the reserved field is a literal zero, and the representation nonce
/// is the one witness item the successor needs. There is no second
/// object for a caller to choose, so the unchanged-field and
/// exact-transition properties hold by construction.
///
/// # The counter's domain
///
/// The increment is signed, so the success flag catches the overflow at
/// the signed maximum and not the unsigned one. The schedule pins the
/// predecessor counter to the nonnegative half first: the reachable
/// counters are then exactly zero through `2^63 - 2`, and the wrap from
/// the unsigned maximum back to zero, which the flag alone would admit,
/// is refused.
///
/// Consumes the representation nonce from the top and reads slices of
/// the predecessor object. Leaves the predecessor object where it was,
/// with the derived successor object above it.
fn derived_successor_metadata() -> Vec<TapscriptInstruction> {
    let reserved_zero = usize::try_from(RESERVED_BYTES).unwrap_or(8);
    vec![
        // The nonce is exactly its schema width, so the derived object
        // is exactly the schema's width.
        op(OpcodeId::Size),
        number(NONCE_BYTES),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::Swap),
        // The recipe's own domain and schema, pinned to constants: an
        // object of another schema must not be advanced by a transition
        // rule written for this one.
        op(OpcodeId::Duplicate),
        number(0),
        number(RECIPE_PREFIX_BYTES),
        op(OpcodeId::Substring),
        raw(vec![0; usize::try_from(RECIPE_PREFIX_BYTES).unwrap_or(20)]),
        op(OpcodeId::EqualVerify),
        // Domain, schema, and object kind, in one contiguous slice.
        op(OpcodeId::Duplicate),
        number(0),
        number(DOMAIN_THROUGH_KIND_BYTES),
        op(OpcodeId::Substring),
        op(OpcodeId::Swap),
        // The counter, pinned nonnegative and incremented by one.
        op(OpcodeId::Duplicate),
        number(COUNTER_AT),
        number(COUNTER_BYTES),
        op(OpcodeId::Substring),
        op(OpcodeId::Duplicate),
        value(0),
        op(OpcodeId::GreaterThanOrEqual64),
        op(OpcodeId::Verify),
        value(1),
        op(OpcodeId::Add64),
        op(OpcodeId::Verify),
        op(OpcodeId::Rotate),
        op(OpcodeId::Swap),
        op(OpcodeId::Concatenate),
        // The flags, unchanged.
        op(OpcodeId::Swap),
        op(OpcodeId::Duplicate),
        number(FLAGS_AT),
        number(FLAGS_BYTES),
        op(OpcodeId::Substring),
        op(OpcodeId::Rotate),
        op(OpcodeId::Swap),
        op(OpcodeId::Concatenate),
        // The representation nonce, the one witnessed field.
        op(OpcodeId::Rotate),
        op(OpcodeId::Concatenate),
        // The reserved field, zero by construction.
        raw(vec![0; reserved_zero]),
        op(OpcodeId::Concatenate),
        // The predecessor's own reserved field, zero by requirement.
        op(OpcodeId::Swap),
        op(OpcodeId::Duplicate),
        number(RESERVED_AT),
        number(RESERVED_BYTES),
        op(OpcodeId::Substring),
        raw(vec![0; reserved_zero]),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::Swap),
    ]
}

/// The successor constructor, derived from the derived object.
///
/// Incoming, deepest first: the one static root, the predecessor
/// metadata, and the derived successor metadata. Leaves the predecessor
/// metadata, the retained root, and the successor tweak.
fn successor_from_derived() -> Vec<TapscriptInstruction> {
    vec![
        raw(vec![0; LEAF_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        raw(vec![0; LEAF_TAIL_BYTES]),
        op(OpcodeId::Sha256Finalize),
        raw(vec![0; TAG_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        // The root is hashed from a copy, so the one instance survives
        // into the predecessor half
        // (Guide-10 `rule:guide10:static-root`).
        op(OpcodeId::Rotate),
        op(OpcodeId::Duplicate),
        op(OpcodeId::Rotate),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
        raw(vec![0; TWEAK_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
    ]
}

/// The predecessor constructor, consuming the one retained root.
///
/// Incoming, deepest first: the predecessor metadata, the retained
/// static root, and the successor tweak. Leaves the successor tweak and
/// the predecessor tweak.
fn predecessor_from_retained_root() -> Vec<TapscriptInstruction> {
    vec![
        op(OpcodeId::Rotate),
        raw(vec![0; LEAF_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        raw(vec![0; LEAF_TAIL_BYTES]),
        op(OpcodeId::Sha256Finalize),
        raw(vec![0; TAG_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Update),
        // No copy is kept: the one instance is consumed here.
        op(OpcodeId::Rotate),
        op(OpcodeId::Sha256Finalize),
        raw(vec![0; TWEAK_PREFIX_BYTES]),
        op(OpcodeId::Sha256Initialize),
        op(OpcodeId::Swap),
        op(OpcodeId::Sha256Finalize),
    ]
}

/// Binds a witnessed compressed key to an introspected program.
fn bind_key_to_introspected_program() -> Vec<TapscriptInstruction> {
    vec![
        number(1),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::Swap),
        op(OpcodeId::Duplicate),
        number(1),
        number(32),
        op(OpcodeId::Substring),
        op(OpcodeId::Rotate),
        op(OpcodeId::EqualVerify),
    ]
}

/// The consumed input's binding and its curve check.
///
/// Incoming, deepest first: the witnessed predecessor output key, the
/// successor tweak, and the predecessor tweak. Leaves the successor
/// tweak alone.
fn input_binding_and_curve() -> Vec<TapscriptInstruction> {
    let mut out = vec![
        op(OpcodeId::Rotate),
        op(OpcodeId::PushCurrentInputIndex),
        op(OpcodeId::InspectInputScriptPubKey),
    ];
    out.extend(bind_key_to_introspected_program());
    out.push(op(OpcodeId::Swap));
    out.extend(curve_step());
    out
}

/// The created output's binding at one exact role, and its curve check.
///
/// Incoming, deepest first: the witnessed successor output key and the
/// successor tweak. Leaves nothing.
fn output_binding_and_curve() -> Vec<TapscriptInstruction> {
    let mut out = vec![
        op(OpcodeId::Swap),
        number(0),
        op(OpcodeId::InspectOutputScriptPubKey),
    ];
    out.extend(bind_key_to_introspected_program());
    out.push(op(OpcodeId::Swap));
    out.extend(curve_step());
    out
}

/// The whole composed proof: continuity and transition in one program.
fn composed_transition_program() -> Vec<TapscriptInstruction> {
    let mut instructions = derived_successor_metadata();
    instructions.extend(successor_from_derived());
    instructions.extend(predecessor_from_retained_root());
    instructions.extend(input_binding_and_curve());
    instructions.extend(output_binding_and_curve());
    instructions.push(number(1));
    instructions
}

/// The witness the composed proof consumes, deepest first.
fn composed_transition_stack() -> AbstractStackState {
    let nonce = usize::try_from(NONCE_BYTES).unwrap_or(4);
    AbstractStackState::from_main(vec![
        StackValueType::Encoded(EncodingClass::CompressedPublicKey),
        StackValueType::Encoded(EncodingClass::CompressedPublicKey),
        literal(32),
        literal(METADATA_BYTES),
        literal(nonce),
    ])
}

/// The abstract type the reviewed concatenation reports.
///
/// It is the one result width the contract does not settle: the join of
/// two admissible operands may exceed the literal bound, so the width is
/// a property of the values rather than of the types.
fn joined_bytes() -> StackValueType {
    let target = reviewed_target();
    let spec = target
        .definition()
        .opcodes()
        .get(&OpcodeId::Concatenate)
        .expect("the reviewed contract states a contract for concatenation");
    spec.stack()
        .success()
        .cases()
        .first()
        .expect("concatenation has a successful form")
        .effect()
        .computed_types()
        .first()
        .cloned()
        .expect("concatenation pushes a result")
}

#[test]
fn the_derivation_builds_one_successor_object_from_the_predecessor() {
    // Stage C6, as a derivation rather than a comparison
    // (Guide-10 `rule:guide10:constructor-transition-stage`).
    //
    // The predecessor object survives unconsumed — its own constructor
    // still has to be derived from it — and the successor object is
    // built above it out of the predecessor's own bytes, one literal
    // zero field, and one witnessed nonce.
    let nonce = usize::try_from(NONCE_BYTES).unwrap_or(4);
    let result = schedule(
        derived_successor_metadata(),
        &AbstractStackState::from_main(vec![literal(METADATA_BYTES), literal(nonce)]),
    );

    // The derived object's width is fixed by construction and by the
    // nonce's checked width rather than by its abstract type, which
    // concatenation leaves unconstrained.
    assert_eq!(
        result.success(),
        &states(&[vec![literal(METADATA_BYTES), joined_bytes()]])
    );
    assert!(result.nonaborting_failure().is_empty());

    // The overflow path does not survive: the flag is verified where it
    // is produced (Guide-10 `rule:guide10:successor-metadata`).
    let mut expected = domain_abort();
    expected.insert(FailureCause::FalseVerification);
    expected.insert(FailureCause::UnequalOperands);
    expected.insert(FailureCause::SliceOutOfRange);
    expected.insert(FailureCause::MalformedScriptNumber);
    expected.insert(FailureCause::ResultSizeExceeded);
    assert_eq!(result.aborts(), &expected);
}

#[test]
fn the_composed_proof_carries_continuity_and_transition_together() {
    // Stages C5 and C6 in one program
    // (Guide-10 `rule:guide10:constructor-continuity-stage`,
    // `rule:guide10:constructor-transition-stage`).
    //
    // The order is forced by the reach bound rather than chosen. The
    // successor constructor is derived first, because its metadata is
    // the value the derivation just produced and nothing else may pile
    // on top of it; the root is retained across the predecessor half
    // and consumed there; and the two curve checks run last, when the
    // two witnessed output keys are the only witnesses left and the two
    // tweaks are the only computed values above them — exactly the two
    // the reach bound admits.
    let result = schedule(composed_transition_program(), &composed_transition_stack());

    assert_eq!(result.success(), &states(&[vec![literal(1)]]));
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn the_composed_proof_witnesses_no_successor_metadata() {
    // The property that makes the composition a transition proof: the
    // successor metadata is not a witness, so there is no second object
    // for a caller to choose. Every field of it is a slice of the
    // predecessor object, the checked increment of that object's
    // counter, a literal zero, or the one witnessed nonce.
    let metadata_items = composed_transition_stack()
        .main()
        .iter()
        .filter(|value| **value == literal(METADATA_BYTES))
        .count();
    assert_eq!(metadata_items, 1);
}

#[test]
fn a_derived_compressed_key_does_not_satisfy_the_curve_position() {
    // The refutation of a cheaper witness, machine-checked rather than
    // assumed.
    //
    // A compressed key is a parity byte and an x coordinate, and the
    // introspection already pushes the coordinate. Witnessing only the
    // parity byte and joining the two would save thirty-two witness
    // bytes per key and would bind the key to the program by
    // construction instead of by comparison.
    //
    // The reviewed contract refuses it. Concatenation's result is an
    // unconstrained byte string, and the curve check's key position is
    // declared as one exact encoding, so an item whose width the
    // abstract state has not fixed does not satisfy it. The composed
    // proof therefore witnesses whole compressed keys and compares
    // their coordinates.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        op(OpcodeId::Concatenate),
        raw(vec![0; 32]),
        op(OpcodeId::TweakVerify),
    ])
    .expect("the schedule is within the limit");
    let initial = AbstractStackState::from_main(vec![
        StackValueType::Encoded(EncodingClass::Sha256Digest),
        literal(1),
        literal(32),
    ]);

    assert!(
        validate_program(
            &target,
            &program,
            &initial,
            AbstractLimits::for_target(&target),
        )
        .is_err(),
        "a joined key does not satisfy an exactly encoded key position"
    );
}
