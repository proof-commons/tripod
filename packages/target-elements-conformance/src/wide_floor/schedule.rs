//! The wide-floor target schedule, as reviewed primitives.
//!
//! # One packed proof item, and why
//!
//! No reviewed primitive reads below the third stack item, and none
//! copies from that depth at all — `ROT` moves the third item rather
//! than duplicating it. A schedule that held the five amounts, ten
//! limbs, eight product limbs, and four carries as separate stack items
//! would therefore have no way to reach most of them.
//!
//! So the schedule holds exactly one item: a byte string that the amounts
//! are concatenated into and that every derived value is appended to. A
//! value is read back with a constant-width slice, which costs two small
//! literals, and the deepest read anywhere below is the third item —
//! `SUBSTR` of a copy while two computed values are live
//! `(´[PLAN-rule:guide10:stack-schedule]´)`.
//!
//! The packing is not a compression trick. It is what makes the reach
//! bound satisfiable at all for a relation this wide, and it is the
//! reason the candidate comparison in `super::candidate` finds the
//! derived-limb candidate smaller rather than larger than the witnessed
//! one.
//!
//! # No limb is witnessed
//!
//! Every limb is derived by the target's own Euclidean division by the
//! base: the remainder is the low limb and the quotient is the high one,
//! both under a verified success flag. There is no limb, no carry, and no
//! product limb for a caller to supply, so the entire class of
//! underconstrained-witness attacks in the threat matrix has nothing to
//! attack — the rows that state them are refused because the value the
//! target computes disagrees, not because a bound check caught a witness
//! `(´[PLAN-candidate:guide10:derived-limbs]´)`.
//!
//! # Every flag is verified on the next instruction
//!
//! The reviewed arithmetic primitives push a result *and* a flag, and on
//! failure they retain their operands and push a false above them. Every
//! one of them is followed immediately by the verification that consumes
//! its flag, which is the required shape and is what leaves the abstract
//! validator with no surviving failure state
//! `(´[PLAN-rule:guide10:success-flags]´)`.

use tapscript::instruction::{StackItem, TapscriptInstruction};
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition, StackValueType};

use super::domain::{AMOUNT_DOMAIN, LIMB_BASE};
use super::oracle::AMOUNT_BYTES;

/// Where each witnessed amount sits in the packed proof item.
///
/// The order is the witness order, which is the order the schedule packs
/// from the top down. Offsets are stated once here because the schedule
/// reads them and the case matrix and the tests check against them.
pub const AMOUNT_AT: [usize; 5] = [0, 8, 16, 24, 32];

/// Where the authenticated quotient sits.
///
/// The value a composing operation consumes. Stated as its own constant
/// because it is the pattern's output contract rather than an
/// implementation detail `(´[PLAN-rule:guide10:wide-floor-output]´)`.
pub const QUOTIENT_AT: usize = AMOUNT_AT[2];

/// How wide the packed item is once the amounts are packed.
pub const PACKED_AMOUNTS_BYTES: usize = 5 * AMOUNT_BYTES;

/// Where each amount's two limbs sit, low limb first.
const LIMBS_AT: [usize; 5] = [40, 56, 72, 88, 104];

/// Where the product side's normalization results sit.
///
/// Each of the three steps appends its division's two outputs in the
/// order the division pushes them: the remainder, which is the limb, and
/// then the quotient, which is the carry into the next step. The last
/// step's carry is the top limb rather than a carry.
const PRODUCT_AT: [usize; 6] = [120, 128, 136, 144, 152, 160];

/// Where the quotient side's normalization results sit.
const QUOTIENT_SIDE_AT: [usize; 6] = [168, 176, 184, 192, 200, 208];

/// How wide the packed item is when the proof is complete.
pub const PACKED_PROOF_BYTES: usize = 216;

/// One reviewed primitive, as an instruction.
const fn op(id: OpcodeId) -> TapscriptInstruction {
    TapscriptInstruction::Opcode(id)
}

/// A signed fixed-width literal, as an instruction.
fn signed(target: &ReviewedElementsTapscriptDefinition, value: i64) -> TapscriptInstruction {
    TapscriptInstruction::Push(StackItem::signed_le64(target, value))
}

/// A script-number literal, as an instruction.
fn number(
    target: &ReviewedElementsTapscriptDefinition,
    value: usize,
) -> Option<TapscriptInstruction> {
    let value = i64::try_from(value).ok()?;
    StackItem::script_number(target, value)
        .map(TapscriptInstruction::Push)
        .ok()
}

/// The limb base, as the literal every division uses.
fn base(target: &ReviewedElementsTapscriptDefinition) -> Option<TapscriptInstruction> {
    Some(signed(target, i64::try_from(LIMB_BASE).ok()?))
}

/// Requires the item on top to be a nonnegative in-domain amount, and
/// leaves it where it was.
///
/// Both comparisons take a signed fixed-width operand, so an item of any
/// other width is refused by the primitive rather than by a width check
/// this schedule would otherwise have to write
/// `(´[PLAN-rule:guide10:limb-encoding]´)`.
fn require_in_domain(
    target: &ReviewedElementsTapscriptDefinition,
) -> Option<Vec<TapscriptInstruction>> {
    let domain = i64::try_from(AMOUNT_DOMAIN).ok()?;
    Some(vec![
        op(OpcodeId::Duplicate),
        signed(target, 0),
        op(OpcodeId::GreaterThanOrEqual64),
        op(OpcodeId::Verify),
        op(OpcodeId::Duplicate),
        signed(target, domain),
        op(OpcodeId::LessThan64),
        op(OpcodeId::Verify),
    ])
}

/// Copies the packed item and slices one constant-width field out of it.
///
/// `lift` is how the packed item is brought within reach, and differs
/// with how many computed values are already live: with none it is
/// immediately below, and with one it is one deeper.
fn read_field(
    target: &ReviewedElementsTapscriptDefinition,
    lift: OpcodeId,
    at: usize,
) -> Option<Vec<TapscriptInstruction>> {
    Some(vec![
        op(lift),
        number(target, at)?,
        number(target, AMOUNT_BYTES)?,
        op(OpcodeId::Substring),
    ])
}

/// Multiplies two packed fields, with nothing else live.
fn multiply_fields(
    target: &ReviewedElementsTapscriptDefinition,
    left: usize,
    right: usize,
) -> Option<Vec<TapscriptInstruction>> {
    let mut out = read_field(target, OpcodeId::Duplicate, left)?;
    out.extend(read_field(target, OpcodeId::CopyOver, right)?);
    out.extend([op(OpcodeId::Mul64), op(OpcodeId::Verify)]);
    Some(out)
}

/// Multiplies two packed fields while one computed value is live.
///
/// The live value is moved out of the way, the product is built exactly
/// as it is with nothing live, and the two are then restored in the
/// order the caller expects: the older value below, the product above.
/// The deepest read is the rotation, which is the third item.
fn multiply_fields_beside(
    target: &ReviewedElementsTapscriptDefinition,
    left: usize,
    right: usize,
) -> Option<Vec<TapscriptInstruction>> {
    let mut out = vec![op(OpcodeId::Swap)];
    out.extend(multiply_fields(target, left, right)?);
    out.extend([op(OpcodeId::Rotate), op(OpcodeId::Swap)]);
    Some(out)
}

/// Adds one packed field to the live computed value.
fn add_field(
    target: &ReviewedElementsTapscriptDefinition,
    at: usize,
) -> Option<Vec<TapscriptInstruction>> {
    let mut out = read_field(target, OpcodeId::CopyOver, at)?;
    out.extend([op(OpcodeId::Add64), op(OpcodeId::Verify)]);
    Some(out)
}

/// Divides the live value by the base and appends both results.
///
/// The division pushes the remainder first and the quotient above it, so
/// the two concatenations append them in exactly that order and the
/// offsets stated above are the offsets the division determines rather
/// than an order this schedule chose.
fn normalize_and_append(
    target: &ReviewedElementsTapscriptDefinition,
) -> Option<Vec<TapscriptInstruction>> {
    Some(vec![
        base(target)?,
        op(OpcodeId::Div64),
        op(OpcodeId::Verify),
        op(OpcodeId::Concatenate),
        op(OpcodeId::Concatenate),
    ])
}

/// Requires two packed fields to be byte-identical.
fn require_equal_fields(
    target: &ReviewedElementsTapscriptDefinition,
    left: usize,
    right: usize,
) -> Option<Vec<TapscriptInstruction>> {
    let mut out = read_field(target, OpcodeId::Duplicate, left)?;
    out.extend(read_field(target, OpcodeId::CopyOver, right)?);
    out.push(op(OpcodeId::EqualVerify));
    Some(out)
}

/// Checks every witnessed amount and packs the five into one item.
///
/// The witness arrives with the remainder on top and the divisor
/// immediately below it, which is what makes `r < d` decidable without
/// reaching past the second item. Each amount is checked while it is on
/// top and joined to the growing item as soon as it has been, so no
/// amount is ever read from a depth the reviewed primitives cannot
/// reach.
fn check_and_pack(
    target: &ReviewedElementsTapscriptDefinition,
) -> Option<Vec<TapscriptInstruction>> {
    // The remainder: nonnegative and in domain. Its upper bound against
    // the divisor is the next check, and the divisor is itself in
    // domain, so `r < d < 2^51` needs no separate statement.
    let mut out = require_in_domain(target)?;

    // `r < d`, with both preserved: the pair is copied, the copies are
    // put in comparison order, and the comparison consumes only them.
    out.extend([
        op(OpcodeId::DuplicateTwo),
        op(OpcodeId::Swap),
        op(OpcodeId::LessThan64),
        op(OpcodeId::Verify),
    ]);

    // The divisor: in domain and strictly positive.
    out.push(op(OpcodeId::Swap));
    out.extend(require_in_domain(target)?);
    out.extend([
        op(OpcodeId::Duplicate),
        signed(target, 0),
        op(OpcodeId::GreaterThan64),
        op(OpcodeId::Verify),
        op(OpcodeId::Swap),
        op(OpcodeId::Concatenate),
    ]);

    // The quotient, the second factor, and the first factor, each
    // checked on top and joined beneath what is already packed.
    for _ in 0..3 {
        out.push(op(OpcodeId::Swap));
        out.extend(require_in_domain(target)?);
        out.extend([op(OpcodeId::Swap), op(OpcodeId::Concatenate)]);
    }
    Some(out)
}

/// Derives both limbs of every packed amount and appends them.
fn derive_limbs(target: &ReviewedElementsTapscriptDefinition) -> Option<Vec<TapscriptInstruction>> {
    let mut out = Vec::new();
    for at in AMOUNT_AT {
        out.extend(read_field(target, OpcodeId::Duplicate, at)?);
        out.extend(normalize_and_append(target)?);
    }
    Some(out)
}

/// Normalizes `x·y`, optionally with an addend's limbs folded in.
///
/// The addend is the remainder on the quotient side and absent on the
/// product side. Folding it into this cascade rather than running a
/// second one is what makes the quotient side three divisions instead of
/// six; both compute the canonical base-`B` representation of the same
/// integer, and the host oracle checks the two forms against each other.
fn normalize_product(
    target: &ReviewedElementsTapscriptDefinition,
    left: usize,
    right: usize,
    addend: Option<usize>,
    carries: [usize; 2],
) -> Option<Vec<TapscriptInstruction>> {
    let low = LIMBS_AT[left];
    let high = LIMBS_AT[left] + AMOUNT_BYTES;
    let other_low = LIMBS_AT[right];
    let other_high = LIMBS_AT[right] + AMOUNT_BYTES;

    // The low coefficient, plus the addend's low limb.
    let mut out = multiply_fields(target, low, other_low)?;
    if let Some(addend) = addend {
        out.extend(add_field(target, LIMBS_AT[addend])?);
    }
    out.extend(normalize_and_append(target)?);

    // The cross coefficient, the addend's high limb, and the first
    // carry.
    out.extend(multiply_fields(target, low, other_high)?);
    out.extend(multiply_fields_beside(target, high, other_low)?);
    out.extend([op(OpcodeId::Add64), op(OpcodeId::Verify)]);
    if let Some(addend) = addend {
        out.extend(add_field(target, LIMBS_AT[addend] + AMOUNT_BYTES)?);
    }
    out.extend(add_field(target, carries[0])?);
    out.extend(normalize_and_append(target)?);

    // The high coefficient and the second carry. This division's
    // quotient is the top limb, which is why the domain needs no fifth.
    out.extend(multiply_fields(target, high, other_high)?);
    out.extend(add_field(target, carries[1])?);
    out.extend(normalize_and_append(target)?);
    Some(out)
}

/// The complete emitted schedule.
///
/// # What an accepting execution establishes
///
/// That the five witnessed amounts are in domain, that the divisor is
/// positive, that the remainder is below it, and that the canonical
/// base-`B` limbs of `a·b` and of `q·d + r` are equal in all four
/// positions. Those together are the complete relation, and the relation
/// implies `q = floor(a·b / d)`
/// `(´[PLAN-rule:guide10:wide-floor-relation]´)`.
///
/// # Where the pattern ends and the framing begins
///
/// After [`PROOF_INSTRUCTIONS`] instructions the stack holds exactly the
/// authenticated quotient, in one canonical eight-byte form, with no
/// unchecked flag and no proof-local item beside it. That is the
/// pattern's output contract. The remaining instructions are the
/// standalone framing a spend of this leaf alone needs: the quotient
/// nothing composes with is dropped and the domain's required true item
/// is pushed `(´[PLAN-rule:guide10:wide-floor-output]´)`.
pub(crate) fn instructions(
    target: &ReviewedElementsTapscriptDefinition,
) -> Option<Vec<TapscriptInstruction>> {
    let mut out = check_and_pack(target)?;
    out.extend(derive_limbs(target)?);

    // The product side: `a·b`.
    out.extend(normalize_product(
        target,
        0,
        1,
        None,
        [PRODUCT_AT[1], PRODUCT_AT[3]],
    )?);

    // The quotient side: `q·d + r`.
    out.extend(normalize_product(
        target,
        2,
        3,
        Some(4),
        [QUOTIENT_SIDE_AT[1], QUOTIENT_SIDE_AT[3]],
    )?);

    // All four limbs, compared exactly.
    for position in 0..4 {
        let (left, right) = limb_positions(position)?;
        out.extend(require_equal_fields(target, left, right)?);
    }

    // The pattern's output: the authenticated quotient alone.
    out.extend(read_field(target, OpcodeId::Duplicate, QUOTIENT_AT)?);
    out.push(op(OpcodeId::RemoveSecond));

    // The standalone framing.
    out.push(op(OpcodeId::Drop));
    out.push(TapscriptInstruction::Push(
        StackItem::new(target, vec![1]).ok()?,
    ));
    Some(out)
}

/// Where one normalized limb sits on each side.
///
/// The first two steps append a limb and a carry, so a limb's offset is
/// twice its position; the third appends the last two limbs adjacently.
const fn limb_positions(position: usize) -> Option<(usize, usize)> {
    let index = match position {
        0 => 0,
        1 => 2,
        2 => 4,
        3 => 5,
        _ => return None,
    };
    Some((PRODUCT_AT[index], QUOTIENT_SIDE_AT[index]))
}

/// How many instructions the pattern proper occupies.
///
/// The two after it are the standalone framing. Stated as a function of
/// the emitted sequence rather than as a hand-counted constant, so it
/// cannot drift from the schedule it describes.
#[must_use]
pub fn proof_instructions(target: &ReviewedElementsTapscriptDefinition) -> Option<usize> {
    instructions(target).map(|emitted| emitted.len() - 2)
}

/// The witness the schedule consumes, deepest item first.
///
/// Five canonical eight-byte amounts. The order is forced rather than
/// chosen: the schedule checks and packs from the top down, and the
/// divisor has to be immediately below the remainder for `r < d` to be
/// decidable within the reach bound.
#[must_use]
pub fn initial_stack_types() -> Vec<StackValueType> {
    (0..5)
        .map(|_| StackValueType::Bytes {
            minimum: AMOUNT_BYTES,
            maximum: AMOUNT_BYTES,
        })
        .collect()
}
