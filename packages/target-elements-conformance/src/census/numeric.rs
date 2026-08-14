//! Signed fixed-width arithmetic, signed comparison, and the numeric
//! conversions.
//!
//! # Expected values come from checked host arithmetic
//!
//! Every sum, difference, product, quotient, and remainder below is
//! computed by the host's own checked and Euclidean operations, and the
//! overflowing cases are exactly the ones where that arithmetic reports
//! an overflow. Nothing here asks the target, the abstract validator, or
//! the executor what the answer is (Guide-9 §13.6, §10.6).
//!
//! # Why the arithmetic cases end in a comparison
//!
//! The reviewed domain requires evaluation to finish with exactly one
//! item. An arithmetic primitive pushes two — a result and a success
//! flag — so a script that stops there is invalid whatever the
//! arithmetic did, and success and overflow would be indistinguishable
//! to an executor that reports a verdict rather than a stack.
//!
//! Converting the flag to the fixed width and comparing it with the
//! result collapses the successful path to a single item, while the
//! overflow path — which retains both operands and pushes a false —
//! collapses to two. The verdicts then differ, and the difference is
//! exactly the reviewed retained-operand behaviour:
//!
//! ```text
//! success:   result flag   -> result one   -> bool          one item
//! overflow:  a b false     -> a b zero     -> a bool        two items
//! ```
//!
//! What this cannot do is pin the result's exact bytes: the reviewed
//! primitive census has no equality primitive, so the strongest native
//! assertion available is an ordering against one. The exact bytes are
//! stated anyway and established statically (Guide-9 §15.4).
//!
//! # A false comparison is an answer, not a failure
//!
//! The comparison primitives always consume both operands and always
//! push exactly one item. When that item is a false, the primitive
//! succeeded and the script's value is false, which is why those cases
//! state a completed evaluation rather than a rejection class.

use tapscript::{StackItem, TapscriptInstruction};
use target_elements::OpcodeId;

use crate::census::author::{Case, CensusAuthor, falsity, op, push, truth};
use crate::fixture::NativeCaseGroup;
use crate::protocol::ObservedFailureClass;

/// The largest script number the target admits, and the boundary the
/// narrowing conversion is checked against.
const MAXIMUM_SCRIPT_NUMBER: i64 = 2_147_483_647;

/// The group every arithmetic case belongs to.
const ARITHMETIC: NativeCaseGroup = NativeCaseGroup::Arithmetic;

/// An operand of the wrong width for a fixed-width primitive.
fn wrong_width(author: &mut CensusAuthor<'_>) -> StackItem {
    author.item(vec![0x01; 7])
}

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

/// Every arithmetic, comparison, and conversion case.
pub fn cases(author: &mut CensusAuthor<'_>) {
    binary_arithmetic(author);
    division(author);
    negation(author);
    comparisons(author);
    widening(author);
    narrowing(author);
    unsigned_widening(author);
}

/// The reduction that turns a primitive's flag into a comparand.
const fn reduction() -> [TapscriptInstruction; 2] {
    [op(OpcodeId::ScriptNumToLe64), op(OpcodeId::GreaterThan64)]
}

/// Addition, subtraction, and multiplication.
fn binary_arithmetic(author: &mut CensusAuthor<'_>) {
    let operands: [(OpcodeId, (i64, i64)); 15] = [
        (OpcodeId::Add64, (2, 3)),
        (OpcodeId::Add64, (0, 0)),
        (OpcodeId::Add64, (1, -1)),
        (OpcodeId::Add64, (i64::MAX - 1, 1)),
        (OpcodeId::Add64, (i64::MIN, 0)),
        (OpcodeId::Sub64, (5, 3)),
        (OpcodeId::Sub64, (3, 5)),
        (OpcodeId::Sub64, (0, 0)),
        (OpcodeId::Sub64, (i64::MAX, i64::MAX)),
        (OpcodeId::Sub64, (-1, i64::MAX)),
        (OpcodeId::Mul64, (2, 3)),
        (OpcodeId::Mul64, (-2, 3)),
        (OpcodeId::Mul64, (-2, -3)),
        (OpcodeId::Mul64, (1, i64::MAX)),
        (OpcodeId::Mul64, (0, i64::MAX)),
    ];

    for (id, (left, right)) in operands {
        let Some(result) = checked(id, left, right) else {
            continue;
        };
        // The bare form, whose two items are what the contract says the
        // primitive pushes.
        let script = [push(author.le64(left)), push(author.le64(right)), op(id)];
        let bare_stack = vec![author.le64_bytes(result), truth()];
        author.left_multiple(bare(ARITHMETIC, id, &script, &[]), bare_stack);

        // The reduced form, which the target can accept.
        let script = [
            push(author.le64(left)),
            push(author.le64(right)),
            op(id),
            reduction()[0].clone(),
            reduction()[1].clone(),
        ];
        let case = bare(ARITHMETIC, id, &script, &[]);
        if result > 1 {
            author.accept(case, vec![truth()]);
        } else {
            author.evaluated_false(case, vec![falsity()]);
        }
    }

    for (id, (left, right)) in [
        (OpcodeId::Add64, (i64::MAX, 1_i64)),
        (OpcodeId::Add64, (i64::MIN, -1)),
        (OpcodeId::Sub64, (i64::MIN, 1)),
        (OpcodeId::Sub64, (i64::MAX, -1)),
        (OpcodeId::Mul64, (i64::MAX, 2)),
        (OpcodeId::Mul64, (i64::MIN, -1)),
    ] {
        overflow(author, id, left, right);
    }

    for id in [OpcodeId::Add64, OpcodeId::Sub64, OpcodeId::Mul64] {
        binary_failures(author, id, ARITHMETIC);
    }
}

/// The host's answer, and whether there is one at all.
const fn checked(id: OpcodeId, left: i64, right: i64) -> Option<i64> {
    match id {
        OpcodeId::Add64 => left.checked_add(right),
        OpcodeId::Sub64 => left.checked_sub(right),
        _ => left.checked_mul(right),
    }
}

/// One overflowing case, in both its bare and its reduced form.
fn overflow(author: &mut CensusAuthor<'_>, id: OpcodeId, left: i64, right: i64) {
    let script = [push(author.le64(left)), push(author.le64(right)), op(id)];
    let retained = vec![author.le64_bytes(left), author.le64_bytes(right), falsity()];
    author.left_multiple(bare(ARITHMETIC, id, &script, &[]), retained);

    // Reduced, the retained operands are still two items where a
    // successful case leaves one: the depth is the observation.
    let script = [
        push(author.le64(left)),
        push(author.le64(right)),
        op(id),
        reduction()[0].clone(),
        reduction()[1].clone(),
    ];
    let expected = vec![
        author.le64_bytes(left),
        if right > 0 { truth() } else { falsity() },
    ];
    author.left_multiple(bare(ARITHMETIC, id, &script, &[]), expected);
}

/// The failure cases every binary fixed-width primitive shares.
fn binary_failures(author: &mut CensusAuthor<'_>, id: OpcodeId, group: NativeCaseGroup) {
    let script = [op(id)];
    let one = author.le64(1);
    let narrow = wrong_width(author);

    for stack in [Vec::new(), vec![one.clone()]] {
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::StackUnderflow],
        );
    }
    for stack in [vec![one.clone(), narrow.clone()], vec![narrow, one]] {
        author.reject(
            bare(group, id, &script, &stack),
            &[ObservedFailureClass::InvalidOperandWidth],
        );
    }
}

/// Division, whose successful form pushes a remainder, a quotient, and
/// a flag, in that order.
///
/// Two reductions rather than one, because the successful form pushes
/// three items where the others push two.
fn division(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::Div64;

    for (dividend, divisor) in [(17_i64, 5_i64), (-17, 5), (17, -5), (-17, -5), (6, 3)] {
        let (Some(quotient), Some(remainder)) = (
            dividend.checked_div_euclid(divisor),
            dividend.checked_rem_euclid(divisor),
        ) else {
            continue;
        };
        let script = [
            push(author.le64(dividend)),
            push(author.le64(divisor)),
            op(id),
        ];
        let bare_stack = vec![
            author.le64_bytes(remainder),
            author.le64_bytes(quotient),
            truth(),
        ];
        author.left_multiple(bare(ARITHMETIC, id, &script, &[]), bare_stack);

        // Euclidean: the remainder is normalized non-negative, so the
        // quotient is not the truncating one on the negative cases.
        let script = [
            push(author.le64(dividend)),
            push(author.le64(divisor)),
            op(id),
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::GreaterThan64),
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::GreaterThan64),
        ];
        let case = bare(ARITHMETIC, id, &script, &[]);
        let first = i64::from(quotient > 1);
        if remainder > first {
            author.accept(case, vec![truth()]);
        } else {
            author.evaluated_false(case, vec![falsity()]);
        }
    }

    // A zero divisor and the one overflowing division retain their
    // operands and push a false, which the same two reductions leave as
    // a single false rather than a single true.
    for (dividend, divisor) in [(-17_i64, 0_i64), (i64::MIN, -1)] {
        let script = [
            push(author.le64(dividend)),
            push(author.le64(divisor)),
            op(id),
        ];
        let retained = vec![
            author.le64_bytes(dividend),
            author.le64_bytes(divisor),
            falsity(),
        ];
        author.left_multiple(bare(ARITHMETIC, id, &script, &[]), retained);

        let script = [
            push(author.le64(dividend)),
            push(author.le64(divisor)),
            op(id),
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::GreaterThan64),
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::GreaterThan64),
        ];
        author.evaluated_false(bare(ARITHMETIC, id, &script, &[]), vec![falsity()]);
    }

    binary_failures(author, id, ARITHMETIC);
}

/// Negation, the one unary arithmetic primitive.
fn negation(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::Neg64;

    for value in [-5_i64, -1, 1, 7, i64::MAX] {
        let Some(result) = value.checked_neg() else {
            continue;
        };
        let script = [push(author.le64(value)), op(id)];
        let bare_stack = vec![author.le64_bytes(result), truth()];
        author.left_multiple(bare(ARITHMETIC, id, &script, &[]), bare_stack);

        let script = [
            push(author.le64(value)),
            op(id),
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::GreaterThan64),
        ];
        let case = bare(ARITHMETIC, id, &script, &[]);
        if result > 1 {
            author.accept(case, vec![truth()]);
        } else {
            author.evaluated_false(case, vec![falsity()]);
        }
    }

    // The one value with no negation: the operand stays and a false is
    // pushed above it, so the reduction compares the operand itself.
    let script = [push(author.le64(i64::MIN)), op(id)];
    let retained = vec![author.le64_bytes(i64::MIN), falsity()];
    author.left_multiple(bare(ARITHMETIC, id, &script, &[]), retained);

    let script = [
        push(author.le64(i64::MIN)),
        op(id),
        op(OpcodeId::ScriptNumToLe64),
        op(OpcodeId::GreaterThan64),
    ];
    author.evaluated_false(bare(ARITHMETIC, id, &script, &[]), vec![falsity()]);

    let script = [op(id)];
    author.reject(
        bare(ARITHMETIC, id, &script, &[]),
        &[ObservedFailureClass::StackUnderflow],
    );
    let narrow = wrong_width(author);
    author.reject(
        bare(ARITHMETIC, id, &script, &[narrow]),
        &[ObservedFailureClass::InvalidOperandWidth],
    );
}

/// Whether one comparison holds of two operands.
///
/// Written out rather than read from the target: this is the census's
/// own statement of what the four orderings mean.
const fn comparison_holds(id: OpcodeId, left: i64, right: i64) -> bool {
    match id {
        OpcodeId::LessThan64 => left < right,
        OpcodeId::LessThanOrEqual64 => left <= right,
        OpcodeId::GreaterThan64 => left > right,
        _ => left >= right,
    }
}

/// The four signed comparisons.
fn comparisons(author: &mut CensusAuthor<'_>) {
    for id in [
        OpcodeId::LessThan64,
        OpcodeId::LessThanOrEqual64,
        OpcodeId::GreaterThan64,
        OpcodeId::GreaterThanOrEqual64,
    ] {
        for (left, right) in [
            (1_i64, 2_i64),
            (2, 2),
            (3, 2),
            (-2, -1),
            (-1, -1),
            (-1, -2),
            (i64::MIN, i64::MAX),
            (i64::MAX, i64::MIN),
            (0, 0),
        ] {
            let script = [push(author.le64(left)), push(author.le64(right)), op(id)];
            let case = bare(NativeCaseGroup::Comparison, id, &script, &[]);
            if comparison_holds(id, left, right) {
                author.accept(case, vec![truth()]);
            } else {
                author.evaluated_false(case, vec![falsity()]);
            }
        }
        binary_failures(author, id, NativeCaseGroup::Comparison);
    }
}

/// Widening a script number to the fixed width.
fn widening(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::ScriptNumToLe64;
    let group = NativeCaseGroup::Conversion;

    for value in [
        1_i64,
        -1,
        2,
        -2,
        MAXIMUM_SCRIPT_NUMBER,
        -MAXIMUM_SCRIPT_NUMBER,
        0,
    ] {
        let script = [push(author.number(value)), op(id)];
        let expected = vec![author.le64_bytes(value)];
        let case = bare(group, id, &script, &[]);
        // A widened value is the script's whole final stack, so the
        // verdict is the value's own truth: zero widens to eight zero
        // bytes, which the target reads as false.
        if value == 0 {
            author.evaluated_false(case, expected);
        } else {
            author.accept(case, expected);
        }
    }

    // A five-byte operand is wider than the target reads, which is its
    // own rule: the spend is invalid.
    let script = [op(id)];
    let oversized = author.item(vec![0x01, 0x00, 0x00, 0x00, 0x00]);
    author.reject(
        bare(group, id, &script, &[oversized]),
        &[ObservedFailureClass::MalformedScriptNumber],
    );

    // A trailing zero byte is never the minimal encoding of a script
    // number — and minimality is a relay rule, not the target's own. At
    // consensus the operand is simply read; the refusal is real, and it
    // is real one layer up.
    let nonminimal = author.item(vec![0x01, 0x00]);
    author.reject_at_relay(
        bare(group, id, &script, &[nonminimal]),
        &[ObservedFailureClass::MalformedScriptNumber],
    );
    author.reject(
        bare(group, id, &script, &[]),
        &[ObservedFailureClass::StackUnderflow],
    );
}

/// Narrowing the fixed width to a script number.
fn narrowing(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::Le64ToScriptNum;
    let group = NativeCaseGroup::Conversion;

    for value in [
        1_i64,
        -1,
        127,
        128,
        -128,
        MAXIMUM_SCRIPT_NUMBER,
        -MAXIMUM_SCRIPT_NUMBER,
        0,
    ] {
        let script = [push(author.le64(value)), op(id)];
        let expected = vec![author.number_bytes(value)];
        let case = bare(group, id, &script, &[]);
        if value == 0 {
            author.evaluated_false(case, expected);
        } else {
            author.accept(case, expected);
        }
    }

    // One above and one below the admissible range: neither has a
    // script-number representation to push, so both abort rather than
    // pushing a false.
    let script = [op(id)];
    //
    // The target answers this and a conversion operand of the wrong width
    // with one code, so the undistinguished observation is admitted
    // alongside the cause the contract names. Requiring the finer of the
    // two would fail an honest executor over a line the target does not
    // draw.
    for value in [MAXIMUM_SCRIPT_NUMBER + 1, -MAXIMUM_SCRIPT_NUMBER - 1] {
        let operand = author.le64(value);
        author.reject(
            bare(group, id, &script, &[operand]),
            &[
                ObservedFailureClass::ScriptNumberRangeExceeded,
                ObservedFailureClass::FixedWidthConversionRefused,
            ],
        );
    }

    let narrow = wrong_width(author);
    author.reject(
        bare(group, id, &script, &[narrow]),
        &[ObservedFailureClass::InvalidOperandWidth],
    );
    author.reject(
        bare(group, id, &script, &[]),
        &[ObservedFailureClass::StackUnderflow],
    );
}

/// Widening an unsigned 32-bit value to the signed fixed width.
fn unsigned_widening(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::Le32ToLe64;
    let group = NativeCaseGroup::Conversion;

    for value in [1_u32, 2, 0x8000_0000, 0xffff_ffff, 0x7fff_ffff, 0] {
        let script = [push(author.le32(value)), op(id)];
        // Read unsigned and zero-extended, so the high bit widens to a
        // large positive value rather than to a negative one.
        let expected = vec![author.le64_bytes(i64::from(value))];
        let case = bare(group, id, &script, &[]);
        if value == 0 {
            author.evaluated_false(case, expected);
        } else {
            author.accept(case, expected);
        }
    }

    let script = [op(id)];
    let narrow = author.item(vec![0x01, 0x02, 0x03]);
    let wide = author.item(vec![0x01; 8]);
    for operand in [narrow, wide] {
        // The same undistinguished conversion refusal as the narrowing
        // primitive's range failure: one code, two reviewed causes.
        author.reject(
            bare(group, id, &script, &[operand]),
            &[
                ObservedFailureClass::InvalidOperandWidth,
                ObservedFailureClass::FixedWidthConversionRefused,
            ],
        );
    }
    author.reject(
        bare(group, id, &script, &[]),
        &[ObservedFailureClass::StackUnderflow],
    );
}
