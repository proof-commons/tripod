//! An independent reference transfer, compared state set for state set.
//!
//! The expected contracts below are written out by hand: operands,
//! every successful alternative with what it consumes and pushes, the
//! non-aborting failure shapes, and the aborting causes. The reference
//! transfer walks a short instruction sequence with them and enumerates
//! the complete outcome sets.
//!
//! Nothing here calls the production step helper, and nothing reads the
//! contracts out of the target registry. That is the point: a reference
//! that consulted the production contract would agree with the
//! production validator whatever either of them said.

use std::collections::BTreeSet;
use std::num::NonZeroUsize;

use target_elements::{ByteOrder, EncodingClass, FailureCause, OpcodeId, StackValueType};

use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::TapscriptProgram;
use crate::stack::{AbstractLimits, AbstractStackState, validate_program};

use super::reviewed_target;

/// One successful form: how many operands it consumes, and what it
/// pushes in push order.
struct OracleCase {
    consumed: usize,
    results: Vec<StackValueType>,
}

/// The complete expected contract of one primitive.
struct OracleContract {
    operands: usize,
    cases: Vec<OracleCase>,
    /// Failures that consume the declared operands and push a false.
    consume_and_push_false: bool,
    /// Failures that leave the operands and push a false above them.
    retain_and_push_false: bool,
    /// Causes on which evaluation ends, less the ones an abstract stack
    /// settles by itself.
    aborts: &'static [FailureCause],
}

/// A signed little-endian sixty-four bit item.
fn signed64() -> StackValueType {
    StackValueType::SignedFixedWidth {
        bytes: NonZeroUsize::new(8).expect("eight is not zero"),
        byte_order: ByteOrder::LittleEndian,
    }
}

/// The forms an issuance amount arrives in.
fn issuance_amount() -> BTreeSet<EncodingClass> {
    [
        EncodingClass::ExplicitValue,
        EncodingClass::ConfidentialValue,
    ]
    .into_iter()
    .collect()
}

/// One successful form.
fn case(consumed: usize, results: Vec<StackValueType>) -> OracleCase {
    OracleCase { consumed, results }
}

/// The causes every introspection primitive can abort on, less the
/// operand-shape ones an abstract stack settles.
const INTROSPECTION_ABORTS: &[FailureCause] = &[
    FailureCause::UnsupportedExecutionDomain,
    FailureCause::MalformedScriptNumber,
    FailureCause::IntrospectionContextUnavailable,
    FailureCause::IntrospectionIndexOutOfRange,
];

/// The one cause a fixed-width arithmetic or comparison primitive
/// aborts on that an abstract stack does not settle.
const ARITHMETIC_ABORTS: &[FailureCause] = &[FailureCause::UnsupportedExecutionDomain];

/// The independently expected contract of one introspection primitive.
fn introspection_oracle(id: OpcodeId) -> OracleContract {
    use EncodingClass as E;
    use StackValueType as S;

    match id {
        // Six items when the input carries an issuance, one when it
        // does not. Either amount is independently explicit or blinded,
        // which is a property of the amount rather than a condition the
        // target branches on.
        OpcodeId::InspectInputIssuance => OracleContract {
            operands: 1,
            cases: vec![
                case(
                    1,
                    vec![
                        S::EncodedPayloadAlternatives(issuance_amount()),
                        S::EncodingPrefixAlternatives(issuance_amount()),
                        S::EncodedPayloadAlternatives(issuance_amount()),
                        S::EncodingPrefixAlternatives(issuance_amount()),
                        S::Encoded(E::IssuanceEntropy),
                        S::Encoded(E::IssuanceBlindingNonce),
                    ],
                ),
                case(1, vec![S::Encoded(E::NullValue)]),
            ],
            consume_and_push_false: false,
            retain_and_push_false: false,
            aborts: INTROSPECTION_ABORTS,
        },
        // A payload and then its prefix, in one of two forms.
        OpcodeId::InspectOutputValue => OracleContract {
            operands: 1,
            cases: vec![
                case(
                    1,
                    vec![
                        S::EncodedPayload(E::ExplicitValue),
                        S::EncodingPrefix(E::ExplicitValue),
                    ],
                ),
                case(
                    1,
                    vec![
                        S::EncodedPayload(E::ConfidentialValue),
                        S::EncodingPrefix(E::ConfidentialValue),
                    ],
                ),
            ],
            consume_and_push_false: false,
            retain_and_push_false: false,
            aborts: INTROSPECTION_ABORTS,
        },
        // One item with its prefix still attached, in one of three
        // forms; the absent one is a form in its own right.
        OpcodeId::InspectOutputNonce => OracleContract {
            operands: 1,
            cases: vec![
                case(1, vec![S::Encoded(E::ExplicitNonce)]),
                case(1, vec![S::Encoded(E::ConfidentialNonce)]),
                case(1, vec![S::Encoded(E::NullNonce)]),
            ],
            consume_and_push_false: false,
            retain_and_push_false: false,
            aborts: INTROSPECTION_ABORTS,
        },
        // A witness program with its version, or a digest with a
        // negative marker above it.
        OpcodeId::InspectInputScriptPubKey => OracleContract {
            operands: 1,
            cases: vec![
                case(1, vec![S::Encoded(E::WitnessProgram), S::ScriptNumber]),
                case(1, vec![S::Encoded(E::ScriptPubKeySha256), S::ScriptNumber]),
            ],
            consume_and_push_false: false,
            retain_and_push_false: false,
            aborts: INTROSPECTION_ABORTS,
        },
        other => panic!("the oracle states no introspection contract for {other:?}"),
    }
}

/// The independently expected contract of one fixture primitive.
fn oracle(id: OpcodeId) -> OracleContract {
    use FailureCause as C;
    use StackValueType as S;

    match id {
        // Inspects its operand and leaves it exactly where it was.
        OpcodeId::CheckSequenceVerify => OracleContract {
            operands: 1,
            cases: vec![case(0, Vec::new())],
            consume_and_push_false: false,
            retain_and_push_false: false,
            aborts: &[
                C::MalformedScriptNumber,
                C::NegativeTimelock,
                C::UnsatisfiedTimelock,
            ],
        },
        // Consumes two, pushes a result and a flag; on overflow leaves
        // both operands and pushes a false above them.
        OpcodeId::Add64 => OracleContract {
            operands: 2,
            cases: vec![case(2, vec![signed64(), S::Bool])],
            consume_and_push_false: false,
            retain_and_push_false: true,
            aborts: ARITHMETIC_ABORTS,
        },
        // Always consumes both operands and pushes one answer; the
        // false it can push is the comparison's result, not a failure.
        OpcodeId::LessThan64 => OracleContract {
            operands: 2,
            cases: vec![case(2, vec![S::Bool])],
            consume_and_push_false: false,
            retain_and_push_false: false,
            aborts: ARITHMETIC_ABORTS,
        },
        // An empty signature consumes the operands and pushes a false;
        // a signature that does not verify aborts.
        OpcodeId::CheckSig => OracleContract {
            operands: 2,
            cases: vec![case(2, vec![S::Bool])],
            consume_and_push_false: true,
            retain_and_push_false: false,
            aborts: &[
                C::InvalidPublicKeyEncoding,
                C::ValidationBudgetExhausted,
                C::InvalidSignature,
            ],
        },
        other => introspection_oracle(other),
    }
}

/// The outcome sets the reference transfer produces.
#[derive(Debug, Default, PartialEq, Eq)]
struct Expected {
    success: BTreeSet<Vec<StackValueType>>,
    nonaborting_failure: BTreeSet<Vec<StackValueType>>,
    aborts: BTreeSet<FailureCause>,
}

/// Walks one instruction sequence with the oracle contracts.
///
/// Written independently of the production validator: it carries live
/// stacks as plain vectors, applies every alternative, and records
/// whether a path passed through a non-aborting failure.
fn reference(instructions: &[TapscriptInstruction]) -> Expected {
    let mut live: Vec<(Vec<StackValueType>, bool)> = vec![(Vec::new(), false)];
    let mut aborts: BTreeSet<FailureCause> = BTreeSet::new();

    for instruction in instructions {
        let mut next: Vec<(Vec<StackValueType>, bool)> = Vec::new();
        for (stack, failed) in &live {
            match instruction {
                TapscriptInstruction::Push(item) => {
                    let mut grown = stack.clone();
                    grown.push(if item.is_empty() {
                        StackValueType::Empty
                    } else {
                        StackValueType::Bytes {
                            minimum: item.len(),
                            maximum: item.len(),
                        }
                    });
                    next.push((grown, *failed));
                }
                TapscriptInstruction::Opcode(id) => {
                    let contract = oracle(*id);
                    assert!(stack.len() >= contract.operands, "the fixture underflows");
                    aborts.extend(contract.aborts.iter().copied());

                    for form in &contract.cases {
                        let mut reached = stack.clone();
                        reached.truncate(reached.len() - form.consumed);
                        reached.extend(form.results.iter().cloned());
                        next.push((reached, *failed));
                    }
                    if contract.consume_and_push_false {
                        let mut reached = stack.clone();
                        reached.truncate(reached.len() - contract.operands);
                        reached.push(StackValueType::Empty);
                        next.push((reached, true));
                    }
                    if contract.retain_and_push_false {
                        let mut reached = stack.clone();
                        reached.push(StackValueType::Empty);
                        next.push((reached, true));
                    }
                }
            }
        }
        live = next;
    }

    let mut expected = Expected {
        aborts,
        ..Expected::default()
    };
    for (stack, failed) in live {
        if failed {
            expected.nonaborting_failure.insert(stack);
        } else {
            expected.success.insert(stack);
        }
    }
    expected
}

/// A literal of `width` bytes, as an instruction.
fn push(width: usize) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(
        StackItem::new(&target, vec![0xab; width]).expect("the payload is within the bound"),
    )
}

/// One reviewed primitive, as an instruction.
fn op(id: OpcodeId) -> TapscriptInstruction {
    TapscriptInstruction::Opcode(id)
}

/// Compares the production validator against the reference, exactly.
fn agree(instructions: Vec<TapscriptInstruction>) {
    let target = reviewed_target();
    let expected = reference(&instructions);
    let program = TapscriptProgram::new(instructions).expect("the fixture is within the limit");
    let produced = validate_program(
        &target,
        &program,
        &AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(&target),
    )
    .expect("the fixture validates");

    let produced_success: BTreeSet<Vec<StackValueType>> = produced
        .success()
        .iter()
        .map(|state| state.main().to_vec())
        .collect();
    let produced_failure: BTreeSet<Vec<StackValueType>> = produced
        .nonaborting_failure()
        .iter()
        .map(|state| state.main().to_vec())
        .collect();

    assert_eq!(produced_success, expected.success, "success states");
    assert_eq!(
        produced_failure, expected.nonaborting_failure,
        "non-aborting failure states",
    );
    assert_eq!(produced.aborts(), &expected.aborts, "aborting causes");
    assert!(
        produced
            .success()
            .iter()
            .all(|state| state.alternate().is_empty()),
        "no reviewed primitive touches the alternate stack",
    );
}

#[test]
fn the_retained_operand_sequence_agrees() {
    agree(vec![push(1), op(OpcodeId::CheckSequenceVerify)]);
    agree(vec![
        push(1),
        op(OpcodeId::CheckSequenceVerify),
        op(OpcodeId::CheckSequenceVerify),
    ]);
}

#[test]
fn the_retained_operand_failure_shape_agrees() {
    agree(vec![push(8), push(8), op(OpcodeId::Add64)]);
    agree(vec![push(8), push(8), op(OpcodeId::LessThan64)]);
}

#[test]
fn the_consumed_operand_failure_shape_agrees() {
    agree(vec![push(64), push(32), op(OpcodeId::CheckSig)]);
}

#[test]
fn the_issuance_alternatives_agree() {
    agree(vec![push(1), op(OpcodeId::InspectInputIssuance)]);
}

#[test]
fn the_value_and_nonce_alternatives_agree() {
    agree(vec![push(1), op(OpcodeId::InspectOutputValue)]);
    agree(vec![push(1), op(OpcodeId::InspectOutputNonce)]);
}

#[test]
fn the_program_alternatives_agree() {
    agree(vec![push(1), op(OpcodeId::InspectInputScriptPubKey)]);
}

#[test]
fn alternatives_and_failures_agree_when_they_compound() {
    // Two undecidable discriminants and a retained-operand failure in
    // one sequence, which is where an implementation that collapsed
    // alternatives or normalized failures would first disagree.
    agree(vec![
        push(1),
        op(OpcodeId::InspectOutputNonce),
        push(1),
        op(OpcodeId::InspectInputScriptPubKey),
    ]);
    agree(vec![
        push(8),
        push(8),
        op(OpcodeId::Add64),
        push(1),
        op(OpcodeId::InspectOutputValue),
    ]);
}
