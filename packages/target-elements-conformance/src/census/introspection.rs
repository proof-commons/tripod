//! Input, output, transaction, and issuance introspection.
//!
//! # Bracketing, because there is no equality primitive
//!
//! A primitive that pushes one item can be compared against a pushed
//! constant, and two cases that bracket the value from both sides pin it
//! exactly: `value > n` accepting and `value > n + 1` completing false
//! together say `value == n + 1`. The reviewed census has no equality
//! primitive, so this is how a stated transaction field is established
//! natively rather than merely observed.
//!
//! # Depth, where bracketing is out of reach
//!
//! The asset, value, program, and outpoint primitives push two or three
//! items, and the reviewed domain requires exactly one at the end. Those
//! cases are therefore invalid spends whatever the field contained, and
//! what they establish is the *count* the primitive pushed — which is
//! precisely the contract's success alternative — plus, through the one
//! reduction that fits, that an explicit amount arrives under the
//! explicit prefix.
//!
//! # Bytes nobody here can state
//!
//! Outpoints, spent programs, and assets belong to the network the
//! executor funds on. Those cases state a verdict and no stack, because
//! a stack naming them would be a claim about a deployment rather than
//! about the target.

use tapscript::{StackItem, TapscriptInstruction};
use target_elements::OpcodeId;

use crate::census::author::{Case, CensusAuthor, falsity, op, push, truth};
use crate::census::context::{
    CURRENT_INPUT_INDEX, FIRST_INPUT_SEQUENCE, FIRST_OUTPUT_AMOUNT, INPUT_COUNT, LAST_INPUT_INDEX,
    LAST_OUTPUT_AMOUNT, LAST_OUTPUT_INDEX, OUTPUT_COUNT, SECOND_INPUT_SEQUENCE,
    TRANSACTION_LOCKTIME, TRANSACTION_VERSION, census_transaction, census_transaction_at_input,
};
use crate::fixture::NativeCaseGroup;
use crate::protocol::ObservedFailureClass;

/// The explicit prefix, as the script number a conversion reads it as.
///
/// The prefix byte of a field carried in the clear is `0x01`, which is
/// also the minimal script-number encoding of one — which is why the
/// reduction below both consumes the prefix and asserts what it was.
const EXPLICIT_PREFIX_AS_NUMBER: i64 = 1;

/// Every introspection case.
pub fn cases(author: &mut CensusAuthor<'_>) {
    input_fields(author);
    input_amount(author);
    input_sequence(author);
    current_index(author);
    issuance(author);
    output_fields(author);
    output_amount(author);
    output_nonce(author);
    transaction_fields(author);
    transaction_weight(author);
    index_failures(author);
}

/// One case reading the census transaction.
fn reading<'a>(
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
        context: Some(census_transaction()),
    }
}

/// The multi-item fields of an input: what the primitive pushed is the
/// observation, because what it contained is the network's.
fn input_fields(author: &mut CensusAuthor<'_>) {
    for id in [
        OpcodeId::InspectInputOutpoint,
        OpcodeId::InspectInputAsset,
        OpcodeId::InspectInputScriptPubKey,
    ] {
        for index in [0, LAST_INPUT_INDEX] {
            let script = [push(author.number(index)), op(id)];
            let case = reading(NativeCaseGroup::InputIntrospection, id, &script, &[]);
            author.left_multiple_unstated(case);
        }
    }

    let id = OpcodeId::InspectInputValue;
    for index in [0, LAST_INPUT_INDEX] {
        let script = [push(author.number(index)), op(id)];
        let case = reading(NativeCaseGroup::InputIntrospection, id, &script, &[]);
        author.left_multiple_unstated(case);
    }
}

/// The amount of a spent output, reduced so the target can accept it.
///
/// The reduction consumes the prefix by reading it as a script number,
/// so the case asserts both that the amount arrived in the clear and
/// that it is above one.
fn input_amount(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::InspectInputValue;
    for index in [0, LAST_INPUT_INDEX] {
        let script = [
            push(author.number(index)),
            op(id),
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::GreaterThan64),
        ];
        let case = reading(NativeCaseGroup::InputIntrospection, id, &script, &[]);
        author.accept(case, vec![truth()]);
    }
}

/// An input's sequence, which the fixture states exactly.
fn input_sequence(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::InspectInputSequence;
    let group = NativeCaseGroup::InputIntrospection;

    // Pushed as a four-byte field, so its own truth is the script's.
    let script = [push(author.number(0)), op(id)];
    let expected = vec![author.le32_bytes(FIRST_INPUT_SEQUENCE)];
    author.accept(reading(group, id, &script, &[]), expected);

    let script = [push(author.number(LAST_INPUT_INDEX)), op(id)];
    let expected = vec![author.le32_bytes(SECOND_INPUT_SEQUENCE)];
    author.evaluated_false(reading(group, id, &script, &[]), expected);

    // A four-byte field is not a script number, and the target says so
    // rather than reading the low bytes of one.
    let script = [
        push(author.number(0)),
        op(id),
        op(OpcodeId::ScriptNumToLe64),
    ];
    author.reject(
        reading(group, id, &script, &[]),
        &[ObservedFailureClass::MalformedScriptNumber],
    );

    // Bracketed through the widening conversion, which pins the field.
    let prefix = [push(author.number(0)), op(id)];
    bracket_unsigned(author, group, id, &prefix, u64::from(FIRST_INPUT_SEQUENCE));
}

/// The index of the input being validated.
fn current_index(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::PushCurrentInputIndex;
    let group = NativeCaseGroup::InputIntrospection;
    let script = [op(id)];

    // Validating the first input pushes the empty script number, which
    // is the target's false.
    let case = Case {
        group,
        opcode: Some(id),
        script: &script,
        stack: &[],
        context: Some(census_transaction_at_input(CURRENT_INPUT_INDEX)),
    };
    let expected = vec![author.number_bytes(0)];
    author.evaluated_false(case, expected);

    let case = Case {
        group,
        opcode: Some(id),
        script: &script,
        stack: &[],
        context: Some(census_transaction_at_input(1)),
    };
    let expected = vec![author.number_bytes(1)];
    author.accept(case, expected);
}

/// Issuance introspection, over inputs that carry none.
///
/// The absent form is one empty item, which is the whole result and the
/// target's false. The present form is six items and is not stated here:
/// the reviewed executor cannot yet materialize an issuing transaction,
/// and that gap is recorded as a residual rather than as a case that
/// cannot run.
fn issuance(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::InspectInputIssuance;
    let group = NativeCaseGroup::Issuance;

    for index in [0, LAST_INPUT_INDEX] {
        let script = [push(author.number(index)), op(id)];
        author.evaluated_false(reading(group, id, &script, &[]), vec![falsity()]);
    }
}

/// The multi-item fields of an output.
fn output_fields(author: &mut CensusAuthor<'_>) {
    for id in [
        OpcodeId::InspectOutputAsset,
        OpcodeId::InspectOutputScriptPubKey,
        OpcodeId::InspectOutputValue,
    ] {
        for index in [0, LAST_OUTPUT_INDEX] {
            let script = [push(author.number(index)), op(id)];
            let case = reading(NativeCaseGroup::OutputIntrospection, id, &script, &[]);
            author.left_multiple_unstated(case);
        }
    }
}

/// An output's amount, which the fixture states exactly.
fn output_amount(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::InspectOutputValue;
    let group = NativeCaseGroup::OutputIntrospection;

    for (index, amount) in [
        (0_i64, FIRST_OUTPUT_AMOUNT),
        (LAST_OUTPUT_INDEX, LAST_OUTPUT_AMOUNT),
    ] {
        // The bare form's two items are exactly stated: the amount
        // reaches the stack least significant byte first, under the
        // prefix that says it was carried in the clear.
        let script = [push(author.number(index)), op(id)];
        let amount_bytes = author.le64_bytes(i64::try_from(amount).unwrap_or(i64::MAX));
        let expected = vec![amount_bytes, author.number_bytes(EXPLICIT_PREFIX_AS_NUMBER)];
        author.left_multiple(reading(group, id, &script, &[]), expected);

        let script = [
            push(author.number(index)),
            op(id),
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::GreaterThan64),
        ];
        author.accept(reading(group, id, &script, &[]), vec![truth()]);
    }
}

/// An output's nonce, which every census output leaves absent.
fn output_nonce(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::InspectOutputNonce;
    let group = NativeCaseGroup::OutputIntrospection;

    for index in [0, LAST_OUTPUT_INDEX] {
        let script = [push(author.number(index)), op(id)];
        author.evaluated_false(reading(group, id, &script, &[]), vec![falsity()]);
    }
}

/// The whole-transaction fields, each pinned from both sides.
fn transaction_fields(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::TransactionIntrospection;

    for (id, value) in [
        (OpcodeId::InspectVersion, u64::from(TRANSACTION_VERSION)),
        (OpcodeId::InspectLockTime, u64::from(TRANSACTION_LOCKTIME)),
    ] {
        let script = [op(id)];
        let expected = vec![author.le32_bytes(u32::try_from(value).unwrap_or(u32::MAX))];
        author.accept(reading(group, id, &script, &[]), expected);

        // A four-byte field is not a script number.
        let script = [op(id), op(OpcodeId::ScriptNumToLe64)];
        author.reject(
            reading(group, id, &script, &[]),
            &[ObservedFailureClass::MalformedScriptNumber],
        );

        bracket_unsigned(author, group, id, &[op(id)], value);
    }

    for (id, count) in [
        (OpcodeId::InspectNumInputs, INPUT_COUNT),
        (OpcodeId::InspectNumOutputs, OUTPUT_COUNT),
    ] {
        let script = [op(id)];
        let expected = vec![author.number_bytes(count)];
        author.accept(reading(group, id, &script, &[]), expected);

        bracket_script_number(author, group, id, &[op(id)], count);
    }
}

/// The transaction's weight, which no fixture can state.
///
/// The weight depends on the very script the fixture carries and on the
/// witness the executor builds, so it is bounded rather than stated: a
/// positive value below the fixed width's maximum.
fn transaction_weight(author: &mut CensusAuthor<'_>) {
    let id = OpcodeId::TxWeight;
    let group = NativeCaseGroup::TransactionIntrospection;

    let script = [op(id), push(author.le64(0)), op(OpcodeId::GreaterThan64)];
    author.accept(reading(group, id, &script, &[]), vec![truth()]);

    let script = [
        op(id),
        push(author.le64(i64::MAX)),
        op(OpcodeId::LessThan64),
    ];
    author.accept(reading(group, id, &script, &[]), vec![truth()]);

    let script = [op(id), push(author.le64(0)), op(OpcodeId::LessThan64)];
    author.evaluated_false(reading(group, id, &script, &[]), vec![falsity()]);

    // The weight is a fixed-width field, not a script number.
    let script = [op(id), op(OpcodeId::Le64ToScriptNum)];
    author.reject(
        reading(group, id, &script, &[]),
        &[ObservedFailureClass::ScriptNumberRangeExceeded],
    );
}

/// Two cases bracketing one unsigned four-byte field.
///
/// The widening conversion is what makes the field comparable, and the
/// pair pins it: above `value - 1` and not above `value`.
fn bracket_unsigned(
    author: &mut CensusAuthor<'_>,
    group: NativeCaseGroup,
    id: OpcodeId,
    prefix: &[TapscriptInstruction],
    value: u64,
) {
    let below = i64::try_from(value).unwrap_or(i64::MAX).saturating_sub(1);
    let at = i64::try_from(value).unwrap_or(i64::MAX);

    let mut script = prefix.to_vec();
    script.extend([
        op(OpcodeId::Le32ToLe64),
        push(author.le64(below)),
        op(OpcodeId::GreaterThan64),
    ]);
    author.accept(reading(group, id, &script, &[]), vec![truth()]);

    let mut script = prefix.to_vec();
    script.extend([
        op(OpcodeId::Le32ToLe64),
        push(author.le64(at)),
        op(OpcodeId::GreaterThan64),
    ]);
    author.evaluated_false(reading(group, id, &script, &[]), vec![falsity()]);
}

/// Two cases bracketing one script-number field.
fn bracket_script_number(
    author: &mut CensusAuthor<'_>,
    group: NativeCaseGroup,
    id: OpcodeId,
    prefix: &[TapscriptInstruction],
    value: i64,
) {
    let mut script = prefix.to_vec();
    script.extend([
        op(OpcodeId::ScriptNumToLe64),
        push(author.le64(value - 1)),
        op(OpcodeId::GreaterThan64),
    ]);
    author.accept(reading(group, id, &script, &[]), vec![truth()]);

    let mut script = prefix.to_vec();
    script.extend([
        op(OpcodeId::ScriptNumToLe64),
        push(author.le64(value)),
        op(OpcodeId::GreaterThan64),
    ]);
    author.evaluated_false(reading(group, id, &script, &[]), vec![falsity()]);
}

/// The index failures every index-taking primitive shares.
fn index_failures(author: &mut CensusAuthor<'_>) {
    let inputs = [
        OpcodeId::InspectInputOutpoint,
        OpcodeId::InspectInputAsset,
        OpcodeId::InspectInputValue,
        OpcodeId::InspectInputScriptPubKey,
        OpcodeId::InspectInputSequence,
    ];
    let outputs = [
        OpcodeId::InspectOutputAsset,
        OpcodeId::InspectOutputValue,
        OpcodeId::InspectOutputNonce,
        OpcodeId::InspectOutputScriptPubKey,
    ];

    for (id, count) in inputs
        .into_iter()
        .map(|id| (id, INPUT_COUNT))
        .chain(outputs.into_iter().map(|id| (id, OUTPUT_COUNT)))
        .chain(std::iter::once((
            OpcodeId::InspectInputIssuance,
            INPUT_COUNT,
        )))
    {
        let group = group_of(id);
        let script = [op(id)];

        for index in [-1, count] {
            let operand = author.number(index);
            author.reject(
                reading(group, id, &script, &[operand]),
                &[ObservedFailureClass::IntrospectionIndexOutOfRange],
            );
        }

        // A trailing zero byte is not a minimal script number, and five
        // bytes is wider than one.
        let nonminimal = author.item(vec![0x00, 0x00]);
        let oversized = author.item(vec![0x01, 0x00, 0x00, 0x00, 0x00]);
        for operand in [nonminimal, oversized] {
            author.reject(
                reading(group, id, &script, &[operand]),
                &[ObservedFailureClass::MalformedScriptNumber],
            );
        }

        author.reject(
            reading(group, id, &script, &[]),
            &[ObservedFailureClass::StackUnderflow],
        );
    }
}

/// Which dimension one index-taking primitive belongs to.
const fn group_of(id: OpcodeId) -> NativeCaseGroup {
    match id {
        OpcodeId::InspectInputIssuance => NativeCaseGroup::Issuance,
        OpcodeId::InspectOutputAsset
        | OpcodeId::InspectOutputValue
        | OpcodeId::InspectOutputNonce
        | OpcodeId::InspectOutputScriptPubKey => NativeCaseGroup::OutputIntrospection,
        _ => NativeCaseGroup::InputIntrospection,
    }
}
