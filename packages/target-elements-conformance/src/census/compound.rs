//! The compound-proof substrate: stack rearrangement, equality and
//! verification, and the byte-string operations.
//!
//! # These cases can say what the older ones could not
//!
//! Every group written before this one had to state its outcome as a
//! final *depth*, because the reviewed census had no way to reduce a
//! stack. There was no equality primitive to compare a result against an
//! expected value with, no verify to consume a flag, and no drop to
//! discard a witness. So an arithmetic case could establish that its
//! primitive succeeded and could not establish what it computed.
//!
//! The primitives reviewed here are exactly the missing reduction. A
//! case can now push an independently computed expectation, compare it
//! with what the target produced, and end in one true item — which makes
//! the accepting verdict itself the assertion about the bytes, rather
//! than a statement about how many items were left
//! `(´[PLAN-rule:guide10:primitive-admission]´)`.
//!
//! # Expectations are computed here, never observed
//!
//! Every expected byte string below is written out or computed by this
//! module from the reviewed contract. The concatenations, slices, and
//! bitwise results are stated independently of the target and of the
//! executor, which is the whole point of a case: an expectation the
//! target supplied would make the comparison a tautology
//! `(´[PLAN-rule:guide10:independent-oracles]´)`.
//!
//! # Why equality is byte equality
//!
//! The reviewed equality primitive compares operands byte for byte with
//! no numeric interpretation whatever. A case below states that
//! deliberately: two items that denote the same number in different
//! encodings compare unequal, and a census that only ever compared
//! canonically encoded items would leave a consumer free to assume a
//! numeric comparison the target does not perform.

use tapscript::{StackItem, TapscriptInstruction};
use target_elements::OpcodeId;

use crate::census::author::{Case, CensusAuthor, falsity, op, push, truth};
use crate::fixture::NativeCaseGroup;
use crate::protocol::ObservedFailureClass;

/// One case with no transaction context.
const fn bare<'a>(
    group: NativeCaseGroup,
    id: OpcodeId,
    script: &'a [TapscriptInstruction],
    stack: &'a [StackItem],
) -> Case<'a> {
    Case {
        group,
        opcode: Some(id),
        script,
        stack,
        context: None,
    }
}

/// Every compound-proof substrate case.
pub fn cases(author: &mut CensusAuthor<'_>) {
    copying(author);
    reordering_and_removal(author);
    rearrangement_underflow(author);
    verification(author);
    concatenation(author);
    width(author);
    slicing(author);
    bitwise(author);
}

/// The stack operations that add a copy, each reduced to one true item.
///
/// Each script leaves exactly one item whose value is only reachable if
/// the primitive copied what the contract says it copies. A duplicate
/// that pushed nothing ends at a different depth, and the verdict
/// differs.
fn copying(author: &mut CensusAuthor<'_>) {
    let one = author.item(vec![0x01]);
    let two = author.item(vec![0x02]);

    // A copy compares equal to what it was copied from.
    let script = [
        push(one.clone()),
        op(OpcodeId::Duplicate),
        op(OpcodeId::Equal),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::Duplicate,
            &script,
            &[],
        ),
        vec![truth()],
    );

    // Four items where there were two, the upper pair equal to the
    // lower.
    let script = [
        push(one.clone()),
        push(one.clone()),
        op(OpcodeId::DuplicateTwo),
        op(OpcodeId::EqualVerify),
        op(OpcodeId::Equal),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::DuplicateTwo,
            &script,
            &[],
        ),
        vec![truth()],
    );

    // The second item copied to the top, leaving three where there were
    // two.
    let script = [
        push(one.clone()),
        push(one.clone()),
        op(OpcodeId::CopyOver),
        op(OpcodeId::EqualVerify),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::CopyOver,
            &script,
            &[],
        ),
        vec![vec![0x01]],
    );

    // A copy of the top is inserted below the second, so two drops leave
    // it.
    let script = [
        push(one),
        push(two),
        op(OpcodeId::Tuck),
        op(OpcodeId::Drop),
        op(OpcodeId::Drop),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::Tuck,
            &script,
            &[],
        ),
        vec![vec![0x02]],
    );
}

/// The stack operations that reorder or discard, reduced the same way.
///
/// A swap that left the order alone, or a rotate that took the wrong
/// item, ends with a different survivor than the one stated here.
fn reordering_and_removal(author: &mut CensusAuthor<'_>) {
    let one = author.item(vec![0x01]);
    let two = author.item(vec![0x02]);
    let three = author.item(vec![0x03]);

    // The exchange is observable because the item dropped afterwards is
    // the one that was pushed first.
    let script = [
        push(one.clone()),
        push(two.clone()),
        op(OpcodeId::Swap),
        op(OpcodeId::Drop),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::Swap,
            &script,
            &[],
        ),
        vec![vec![0x02]],
    );

    // The third item reaches the top, so the two dropped afterwards are
    // the upper two and the survivor is the second.
    let script = [
        push(one.clone()),
        push(two.clone()),
        push(three),
        op(OpcodeId::Rotate),
        op(OpcodeId::Drop),
        op(OpcodeId::Drop),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::Rotate,
            &script,
            &[],
        ),
        vec![vec![0x02]],
    );

    // The second item is removed and the top survives.
    let script = [
        push(two.clone()),
        push(one.clone()),
        op(OpcodeId::RemoveSecond),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::RemoveSecond,
            &script,
            &[],
        ),
        vec![vec![0x01]],
    );

    let script = [push(one.clone()), push(two.clone()), op(OpcodeId::Drop)];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::Drop,
            &script,
            &[],
        ),
        vec![vec![0x01]],
    );

    let script = [
        push(one),
        push(two.clone()),
        push(two),
        op(OpcodeId::DropTwo),
    ];
    author.accept(
        bare(
            NativeCaseGroup::StackRearrangement,
            OpcodeId::DropTwo,
            &script,
            &[],
        ),
        vec![vec![0x01]],
    );
}

/// Each stack operation offered one operand fewer than it consumes.
///
/// The negative half of the rearrangement group. Underflow is the only
/// way these primitives fail, so a case that did not exercise it would
/// leave their whole failure contract unobserved.
fn rearrangement_underflow(author: &mut CensusAuthor<'_>) {
    let one = author.item(vec![0x01]);

    // Each primitive paired with the greatest number of items that is
    // still one too few.
    let shortfalls: [(OpcodeId, usize); 9] = [
        (OpcodeId::Duplicate, 0),
        (OpcodeId::DuplicateTwo, 1),
        (OpcodeId::CopyOver, 1),
        (OpcodeId::Swap, 1),
        (OpcodeId::Rotate, 2),
        (OpcodeId::RemoveSecond, 1),
        (OpcodeId::Tuck, 1),
        (OpcodeId::Drop, 0),
        (OpcodeId::DropTwo, 1),
    ];

    for (id, supplied) in shortfalls {
        let mut script: Vec<TapscriptInstruction> = Vec::new();
        for _ in 0..supplied {
            script.push(push(one.clone()));
        }
        script.push(op(id));
        author.reject(
            bare(NativeCaseGroup::StackRearrangement, id, &script, &[]),
            &[ObservedFailureClass::StackUnderflow],
        );
    }
}

/// Equality and Boolean verification.
fn verification(author: &mut CensusAuthor<'_>) {
    let one = author.item(vec![0x01]);
    let two = author.item(vec![0x02]);
    let empty = author.item(Vec::new());
    // The same number, encoded in two widths. Numerically equal, and the
    // reviewed primitive compares bytes.
    let padded_one = author.item(vec![0x01, 0x00]);

    let script = [push(one.clone()), push(one.clone()), op(OpcodeId::Equal)];
    author.accept(
        bare(NativeCaseGroup::Verification, OpcodeId::Equal, &script, &[]),
        vec![truth()],
    );

    let script = [push(one.clone()), push(two.clone()), op(OpcodeId::Equal)];
    author.evaluated_false(
        bare(NativeCaseGroup::Verification, OpcodeId::Equal, &script, &[]),
        vec![falsity()],
    );

    // The byte-equality statement. A numeric comparison would accept
    // this; the reviewed one does not.
    let script = [push(one.clone()), push(padded_one), op(OpcodeId::Equal)];
    author.evaluated_false(
        bare(NativeCaseGroup::Verification, OpcodeId::Equal, &script, &[]),
        vec![falsity()],
    );

    let script = [op(OpcodeId::Equal)];
    author.reject(
        bare(NativeCaseGroup::Verification, OpcodeId::Equal, &script, &[]),
        &[ObservedFailureClass::StackUnderflow],
    );

    // The verifying form consumes the pair it compared and leaves the
    // item beneath.
    let script = [
        push(one.clone()),
        push(one.clone()),
        push(one.clone()),
        op(OpcodeId::EqualVerify),
    ];
    author.accept(
        bare(
            NativeCaseGroup::Verification,
            OpcodeId::EqualVerify,
            &script,
            &[],
        ),
        vec![vec![0x01]],
    );

    let script = [push(one.clone()), push(two), op(OpcodeId::EqualVerify)];
    author.reject(
        bare(
            NativeCaseGroup::Verification,
            OpcodeId::EqualVerify,
            &script,
            &[],
        ),
        &[ObservedFailureClass::UnequalOperands],
    );

    let script = [push(one.clone()), op(OpcodeId::EqualVerify)];
    author.reject(
        bare(
            NativeCaseGroup::Verification,
            OpcodeId::EqualVerify,
            &script,
            &[],
        ),
        &[ObservedFailureClass::StackUnderflow],
    );

    let script = [push(one.clone()), push(one.clone()), op(OpcodeId::Verify)];
    author.accept(
        bare(
            NativeCaseGroup::Verification,
            OpcodeId::Verify,
            &script,
            &[],
        ),
        vec![vec![0x01]],
    );

    // The empty item is the target's false, and verifying it ends
    // evaluation rather than leaving a false behind.
    let script = [push(one), push(empty), op(OpcodeId::Verify)];
    author.reject(
        bare(
            NativeCaseGroup::Verification,
            OpcodeId::Verify,
            &script,
            &[],
        ),
        &[ObservedFailureClass::FalseVerification],
    );

    let script = [op(OpcodeId::Verify)];
    author.reject(
        bare(
            NativeCaseGroup::Verification,
            OpcodeId::Verify,
            &script,
            &[],
        ),
        &[ObservedFailureClass::StackUnderflow],
    );
}

/// Concatenation, including the width bound it enforces on its result.
fn concatenation(author: &mut CensusAuthor<'_>) {
    let left = vec![0x01, 0x02];
    let right = vec![0x03];
    // Computed here, not read back from the target.
    let joined: Vec<u8> = left.iter().chain(right.iter()).copied().collect();

    let left_item = author.item(left);
    let right_item = author.item(right);
    let joined_item = author.item(joined);

    let script = [
        push(left_item),
        push(right_item),
        op(OpcodeId::Concatenate),
        push(joined_item),
        op(OpcodeId::Equal),
    ];
    author.accept(
        bare(
            NativeCaseGroup::ByteString,
            OpcodeId::Concatenate,
            &script,
            &[],
        ),
        vec![truth()],
    );

    // The join of the widest admissible literal with one further byte is
    // one byte too wide. Both operands are admissible on their own,
    // which is what makes this the primitive's failure rather than the
    // pushes'.
    let widest_admissible = author.maximum_literal_bytes();
    let widest = author.item(vec![0x00; widest_admissible]);
    let overflowing = author.item(vec![0x01]);
    let script = [push(widest), push(overflowing), op(OpcodeId::Concatenate)];
    author.reject(
        bare(
            NativeCaseGroup::ByteString,
            OpcodeId::Concatenate,
            &script,
            &[],
        ),
        &[ObservedFailureClass::ResultSizeExceeded],
    );

    let lone = author.item(vec![0x01]);
    let script = [push(lone), op(OpcodeId::Concatenate)];
    author.reject(
        bare(
            NativeCaseGroup::ByteString,
            OpcodeId::Concatenate,
            &script,
            &[],
        ),
        &[ObservedFailureClass::StackUnderflow],
    );
}

/// The width primitive, which leaves the item it measured.
fn width(author: &mut CensusAuthor<'_>) {
    let measured = vec![0x01, 0x02, 0x03];
    let expected_width = i64::try_from(measured.len()).unwrap_or(i64::MAX);
    let item = author.item(measured.clone());
    let width_item = author.number(expected_width);

    // The width is pushed above the item, so verifying it against the
    // independently computed length leaves the item itself — which is
    // nonempty, and therefore the target's true.
    let script = [
        push(item),
        op(OpcodeId::Size),
        push(width_item),
        op(OpcodeId::EqualVerify),
    ];
    author.accept(
        bare(NativeCaseGroup::ByteString, OpcodeId::Size, &script, &[]),
        vec![measured],
    );

    let script = [op(OpcodeId::Size)];
    author.reject(
        bare(NativeCaseGroup::ByteString, OpcodeId::Size, &script, &[]),
        &[ObservedFailureClass::StackUnderflow],
    );
}

/// Slicing, including the bounds it refuses rather than clamps.
fn slicing(author: &mut CensusAuthor<'_>) {
    let whole = vec![0x01, 0x02, 0x03, 0x04];
    let start = 1_usize;
    let length = 2_usize;
    // The expected slice, taken here by the host rather than by the
    // target.
    let expected: Vec<u8> = whole[start..start + length].to_vec();

    let whole_item = author.item(whole.clone());
    let start_number = author.number(i64::try_from(start).unwrap_or(i64::MAX));
    let length_number = author.number(i64::try_from(length).unwrap_or(i64::MAX));
    let expected_item = author.item(expected);

    let script = [
        push(whole_item.clone()),
        push(start_number),
        push(length_number),
        op(OpcodeId::Substring),
        push(expected_item),
        op(OpcodeId::Equal),
    ];
    author.accept(
        bare(
            NativeCaseGroup::ByteString,
            OpcodeId::Substring,
            &script,
            &[],
        ),
        vec![truth()],
    );

    // A start at or past the end is refused outright. The reviewed
    // primitive does not clamp; the separate clamping byte is not
    // reviewed here.
    let past_end = author.number(i64::try_from(whole.len()).unwrap_or(i64::MAX));
    let nothing = author.number(0);
    let script = [
        push(whole_item.clone()),
        push(past_end),
        push(nothing),
        op(OpcodeId::Substring),
    ];
    author.reject(
        bare(
            NativeCaseGroup::ByteString,
            OpcodeId::Substring,
            &script,
            &[],
        ),
        &[ObservedFailureClass::SliceOutOfRange],
    );

    // A length running past the end is refused for the same reason.
    let zero = author.number(0);
    let too_long = author.number(i64::try_from(whole.len() + 1).unwrap_or(i64::MAX));
    let script = [
        push(whole_item.clone()),
        push(zero),
        push(too_long),
        op(OpcodeId::Substring),
    ];
    author.reject(
        bare(
            NativeCaseGroup::ByteString,
            OpcodeId::Substring,
            &script,
            &[],
        ),
        &[ObservedFailureClass::SliceOutOfRange],
    );

    let start_only = author.number(0);
    let script = [push(whole_item), push(start_only), op(OpcodeId::Substring)];
    author.reject(
        bare(
            NativeCaseGroup::ByteString,
            OpcodeId::Substring,
            &script,
            &[],
        ),
        &[ObservedFailureClass::StackUnderflow],
    );
}

/// The bitwise primitives, including the width agreement they require.
fn bitwise(author: &mut CensusAuthor<'_>) {
    let left = 0x0f_u8;
    let right = 0x35_u8;

    for (id, expected) in [
        (OpcodeId::BitwiseAnd, left & right),
        (OpcodeId::BitwiseXor, left ^ right),
    ] {
        let left_item = author.item(vec![left]);
        let right_item = author.item(vec![right]);
        let expected_item = author.item(vec![expected]);
        let script = [
            push(left_item),
            push(right_item),
            op(id),
            push(expected_item),
            op(OpcodeId::Equal),
        ];
        author.accept(
            bare(NativeCaseGroup::ByteString, id, &script, &[]),
            vec![truth()],
        );

        // The operands are paired byte for byte, so a width
        // disagreement ends evaluation instead of padding either side.
        let narrow = author.item(vec![0x01]);
        let wide = author.item(vec![0x01, 0x02]);
        let script = [push(narrow.clone()), push(wide), op(id)];
        author.reject(
            bare(NativeCaseGroup::ByteString, id, &script, &[]),
            &[ObservedFailureClass::MismatchedOperandWidths],
        );

        let script = [push(narrow), op(id)];
        author.reject(
            bare(NativeCaseGroup::ByteString, id, &script, &[]),
            &[ObservedFailureClass::StackUnderflow],
        );
    }
}
