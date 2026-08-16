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

/// Both tag digests of one tagged hash.
const TAG_PREFIX_BYTES: usize = 64;

/// Both tag digests of the tweak hash, and the internal key after them.
const TWEAK_PREFIX_BYTES: usize = TAG_PREFIX_BYTES + 32;

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

/// The declared stack effect of the curve step, and only that.
///
/// The tweak verification consumes its three operands and pushes
/// nothing. The validator will not schedule the instruction itself
/// here, because the tweak it derives is typed as a digest rather than
/// a scalar — see `a_derived_digest_is_not_admitted_as_a_tweak_scalar`,
/// which records that boundary. So the composition schedules the
/// declared *stack* effect in its place: the internal key is pushed and
/// the three items are removed, exactly as the contract states.
///
/// This models the depth, never the check. Nothing here claims the
/// curve relation holds, and the composed schedule is evidence about
/// stack discipline alone.
fn curve_step_stack_effect() -> Vec<TapscriptInstruction> {
    vec![raw(vec![0; 32]), op(OpcodeId::DropTwo), op(OpcodeId::Drop)]
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
    instructions.extend(curve_step_stack_effect());
    instructions.extend(successor_proof());
    instructions.extend(output_binding());
    instructions.extend(curve_step_stack_effect());
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
fn a_derived_digest_is_not_admitted_as_a_tweak_scalar() {
    // The one step of the predecessor proof the abstract validator
    // cannot express, recorded rather than worked around.
    //
    // The streaming hash produces a digest, and the tweak verification
    // takes a scalar. Both are exactly thirty-two bytes, and the
    // reviewed contracts type them apart on purpose: a digest is an
    // opaque hash, a scalar must lie below the group order, and no
    // reviewed primitive converts one into the other. So the
    // composition is refused here even though the target performs it —
    // the interpreter accepts any thirty-two byte tweak and decides
    // scalar validity when it does the arithmetic.
    //
    // That gap is the abstract form of tweak totality
    // (Guide-10 `rule:guide10:tweak-totality`): the rare instance whose
    // derived tweak is not a scalar is exactly the instance this type
    // boundary declines to promise. The prototype does not close it by
    // asserting the conversion, because asserting it would claim a
    // totality the construction does not have.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![op(OpcodeId::TweakVerify)])
        .expect("one instruction is within the limit");
    let initial = AbstractStackState::from_main(vec![
        StackValueType::Encoded(EncodingClass::CompressedPublicKey),
        StackValueType::Encoded(EncodingClass::Sha256Digest),
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
        "a derived digest does not satisfy a scalar position"
    );
}
