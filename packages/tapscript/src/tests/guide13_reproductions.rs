//! Guide-13 preflight reproductions owned by this crate.
//!
//! Each test here belongs to a row of the Guide-13 preflight register
//! and reaches the branch that row reports. A row whose test is
//! `#[ignore]`d is one Wave 0 CONFIRMED: the assertion states the
//! property that *should* hold, so the test fails while the defect
//! stands, and the wave that repairs the row removes the attribute
//! rather than writing a new test. A row whose test runs is one Wave 0
//! REFUTED, and the test then stands as the standing guarantee.
//!
//! Nothing here reaches a crate-private constructor an external caller
//! could not use: every fixture below is built through the same public
//! API a consumer has.
//!
//! - `G13-R04` — CONFIRMED: `CandidateShapeSet::new` is infallible and
//!   checks no member against the bounds it advertises.
//! - `G13-R05` — CONFIRMED: the public constructors admit a total input
//!   count whose `u8` sponsor-range arithmetic does not fit.
//! - `G13-R11` — CONFIRMED: a computed Boolean loses the literal
//!   knowledge that settles it, so `EQUAL; VERIFY` over known unequal
//!   operands keeps a successful state the target cannot reach.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

use target_elements::OpcodeId;

use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::TapscriptProgram;
use crate::shape::{
    CandidateShapeSet, CompactAshShape, CompactAshShapeBounds, ShapeRejection,
    SponsorChangePresence, demonstration_shape_set,
};
use crate::stack::{AbstractExecutionResult, AbstractLimits, AbstractStackState, validate_program};

use super::reviewed_target;

/// A nonzero count for the fixtures.
fn count(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("the fixture counts are nonzero")
}

/// Validates one instruction sequence from an empty stack.
fn validate(instructions: Vec<TapscriptInstruction>) -> AbstractExecutionResult {
    let target = reviewed_target();
    let program = TapscriptProgram::new(instructions).expect("the fixture is within the limit");
    validate_program(
        &target,
        &program,
        &AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(&target),
    )
    .expect("the fixture validates")
}

/// One literal, as an instruction.
fn push(payload: &[u8]) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(
        StackItem::new(&target, payload.to_vec()).expect("the payload is within the bound"),
    )
}

/// `G13-R04`: a candidate set admits only shapes its bounds admit.
///
/// The shape below is valid — it was built through the checked
/// constructor, under bounds that admit it. What is not valid is the
/// *set*: it declares four ASH inputs and one sponsor input as the
/// window it emits programs for, and then holds a member outside both.
///
/// The two statements are not equally visible downstream. The linker
/// resolves `CandidateAshBound` from `shapes().bounds().ash_inputs()`
/// while `emit_candidate_bundle` iterates the members, so a candidate
/// built this way advertises a bound of four and emits a specialization
/// of eight, and nothing between the two ever compares them.
///
/// The repair establishes the property by refusing the set, so the
/// constructor is now fallible and this test states the property in the
/// two halves that fallibility splits it into: the disagreeing set has
/// no value at all, and a set that does exist holds only what its
/// bounds admit. The original single-expression form could not be kept
/// unchanged — `new` no longer returns a `CandidateShapeSet` — and the
/// assertions below are the same two comparisons it made.
#[test]
fn a_candidate_shape_set_holds_only_shapes_its_own_bounds_admit() {
    let wide = CompactAshShapeBounds::new(count(8), 4).expect("eight is above the minimum");
    let outside = CompactAshShape::new(wide, count(8), 4, SponsorChangePresence::Present)
        .expect("the shape is valid under the wide bounds");

    let narrow = CompactAshShapeBounds::new(count(4), 1).expect("four is above the minimum");

    // The ASH count is reported first because it is the one the linker
    // resolves the advertised bound from, so it is the disagreement a
    // consumer would have acted on.
    assert_eq!(
        CandidateShapeSet::new(narrow, BTreeSet::from([outside]), false),
        Err(ShapeRejection::AshInputsAboveBound {
            offered: 8,
            bound: 4,
        }),
        "a set may not hold a specialization its own declared window excludes",
    );

    // The other half: what a set that exists carries. Recomputed here
    // rather than asked of the set, so one that agreed with itself
    // about the wrong window would still fail.
    let set = demonstration_shape_set();
    assert_eq!(set.bounds().ash_inputs(), 4);
    assert_eq!(set.bounds().sponsor_inputs(), 1);

    for shape in set.shapes() {
        assert!(
            shape.ash_inputs() <= set.bounds().ash_inputs(),
            "an emitted specialization of {} ASH inputs is outside the declared bound of {}",
            shape.ash_inputs(),
            set.bounds().ash_inputs(),
        );
        assert!(
            shape.sponsor_inputs() <= set.bounds().sponsor_inputs(),
            "an emitted specialization of {} sponsor inputs is outside the declared bound of {}",
            shape.sponsor_inputs(),
            set.bounds().sponsor_inputs(),
        );
    }
}

/// `G13-R04`, the two refusals that are not about the bounds.
///
/// A candidate is the shapes it emits programs for, so a set holding
/// none of them is refused rather than treated as a very narrow one.
/// And the sparsity declaration reports a limitation, so declaring one
/// over a dense set is refused too — §9.3's declaration exists to turn
/// a gap into a reported limitation, and there is no gap to report.
#[test]
fn a_candidate_set_refuses_emptiness_and_a_limitation_it_does_not_carry() {
    let bounds = CompactAshShapeBounds::new(count(4), 1).expect("four is above the minimum");

    assert_eq!(
        CandidateShapeSet::new(bounds, BTreeSet::new(), false),
        Err(ShapeRejection::EmptyShapeSet),
    );

    let dense = demonstration_shape_set();
    assert_eq!(
        CandidateShapeSet::new(dense.bounds(), dense.shapes().collect(), true),
        Err(ShapeRejection::DenseSetDeclaredSparse),
        "the dense unrolling carries no gap, so it may not declare one",
    );
}

/// `G13-R05`: a shape's sponsor suffix is the region it declares.
///
/// The counts below are admitted by every check the public
/// constructors make: the ASH bound is above the minimum, the shape is
/// within its bounds, and a sponsor input exists for the change role to
/// sit in. The total is 256, which `inputs()` reports correctly because
/// it is `u16` — and which `sponsor_range()` cannot report at all,
/// because it adds the two counts in `u8`.
///
/// The first two assertions are exact and hold in every profile: they
/// say the counts are a `u8` domain and their total is not one. The
/// third reaches the branch that used to derive the suffix in that
/// domain, where the dev profile panicked on the overflow and a profile
/// with overflow checks off wrapped the suffix to `(255, 0)` — the
/// empty range `sponsor_isolation_fragment` then looped over.
///
/// The repair moved the *indices* to `u16` while leaving the *counts* a
/// `u8`, so all four assertions now hold as written and hold in either
/// profile: the counts still overflow their own domain, and the suffix
/// is still derived exactly, because it is no longer derived there.
#[test]
fn a_sponsored_shape_reports_a_nonempty_sponsor_suffix() {
    let bounds = CompactAshShapeBounds::new(count(255), 1).expect("255 is above the minimum");
    let shape = CompactAshShape::new(bounds, count(255), 1, SponsorChangePresence::Absent)
        .expect("the shape is within its own bounds");

    // The defect, stated without reaching it: the accessor's operands
    // are exactly these, and their sum is not a `u8`.
    assert!(
        shape
            .ash_inputs()
            .checked_add(shape.sponsor_inputs())
            .is_none(),
        "the sponsor-range arithmetic is performed in a domain that cannot hold its own result",
    );
    // The wider accessor on the same shape gets the total right, which
    // is why this is an inconsistent index domain and not an extreme.
    assert_eq!(shape.inputs(), 256);

    let (start, end) = shape.sponsor_range();
    assert!(
        end > start,
        "a shape declaring a sponsor input has a nonempty sponsor suffix, not {start}..{end}",
    );
    assert_eq!(
        usize::from(end - start),
        usize::from(shape.sponsor_inputs()),
        "the suffix length is the declared sponsor count",
    );
}

/// `G13-R11`: `EQUAL; VERIFY` over unequal literals reaches nothing.
///
/// On the target the two literals differ, `EQUAL` pushes its false
/// value, and `VERIFY` ends evaluation. No execution reaches the end of
/// this program, so the abstract result has no successful state.
///
/// The walk holds both literals exactly — they were pushed by the
/// program — but `apply_case` transfers a computed result as its type
/// alone, so the Boolean `EQUAL` produced arrives at `VERIFY` as an
/// abstract `Bool`. `Bool` admits both truth values, `LiteralFacts` has
/// no literal to read at that position, and the successful branch
/// survives a program that always aborts.
///
/// The contrast is the next test, which passes today: the verifying
/// primitive keeps the knowledge because it never has to transfer it.
#[test]
#[ignore = "G13-R11: confirmed, repair pending"]
fn unequal_literals_compared_then_verified_reach_no_successful_state() {
    let result = validate(vec![
        push(&[0x01]),
        push(&[0x02]),
        TapscriptInstruction::Opcode(OpcodeId::Equal),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ]);

    assert!(
        result.success().is_empty(),
        "the target cannot reach the end of this program, so no successful state exists: {:?}",
        result.success(),
    );
    assert!(
        result.always_aborts(),
        "every path through this program ends evaluation",
    );
}

/// `G13-R11`, the direction that already holds.
///
/// `EQUALVERIFY` names the comparison and the verification in one
/// primitive, so the unequal case is a failure cause the contract
/// declares and `LiteralFacts` suppresses the successful form from the
/// literals directly. Nothing has to survive a result transfer, which
/// is precisely why the composed spelling above loses it.
///
/// Kept as a running test rather than an ignored one: it is the control
/// that says the machinery works where the knowledge never leaves, so a
/// repair that fixed the composed form by weakening this one would fail
/// here.
#[test]
fn unequal_literals_verified_in_one_primitive_reach_no_successful_state() {
    let result = validate(vec![
        push(&[0x01]),
        push(&[0x02]),
        TapscriptInstruction::Opcode(OpcodeId::EqualVerify),
    ]);

    assert!(result.success().is_empty());
    assert!(result.always_aborts());
}

/// `G13-R11`, the equal case, which must keep its successful state.
///
/// The row is a soundness claim in one direction only. A repair that
/// suppressed the successful state for *equal* operands too would be
/// unsound the other way, so the equal spelling is pinned here.
#[test]
fn equal_literals_compared_then_verified_reach_a_successful_state() {
    let result = validate(vec![
        push(&[0x01]),
        push(&[0x01]),
        TapscriptInstruction::Opcode(OpcodeId::Equal),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ]);

    assert!(
        !result.success().is_empty(),
        "the target reaches the end of this program",
    );
}
