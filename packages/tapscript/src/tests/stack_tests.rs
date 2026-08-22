//! Abstract stack validation: alternatives, failure shapes, limits.

use std::collections::BTreeSet;
use std::num::{NonZeroU64, NonZeroUsize};

use target_elements::{
    ByteOrder, EncodingClass, FailureCause, OpcodeId, OperandContract, ResourceDimension,
    StackValueType,
};

use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::TapscriptProgram;
use crate::stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, SignatureSuccessForm,
    resource_projection, validate_program,
};

use super::reviewed_target;

/// A literal of `width` bytes, as an instruction.
fn push(width: usize) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(
        StackItem::new(&target, vec![0xab; width]).expect("the payload is within the bound"),
    )
}

/// The abstract type of a literal of `width` bytes.
fn literal(width: usize) -> StackValueType {
    StackValueType::Bytes {
        minimum: width,
        maximum: width,
    }
}

/// The target's signed fixed-width type.
fn signed64() -> StackValueType {
    StackValueType::SignedFixedWidth {
        bytes: NonZeroUsize::new(8).expect("eight is not zero"),
        byte_order: ByteOrder::LittleEndian,
    }
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

#[test]
fn a_push_grows_the_main_stack_and_leaves_the_alternate_one_alone() {
    let result = validate(vec![push(4), push(0)]);

    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![
            literal(4),
            StackValueType::Empty,
        ])],
    );
    assert!(result.nonaborting_failure().is_empty());
    assert!(result.aborts().is_empty());
    for state in result.success() {
        assert_eq!(state.alternate(), []);
        assert_eq!(state.depth(), 2);
    }
}

#[test]
fn a_relative_timelock_leaves_its_operand_exactly_where_it_found_it() {
    // The retained-operand shape. A consuming form with no results
    // would have described a net reduction of one and would
    // mis-schedule every program that uses a lock.
    let result = validate(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::CheckSequenceVerify),
    ]);

    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![literal(1)])],
    );
    assert!(result.nonaborting_failure().is_empty());
    assert!(result.aborts().contains(&FailureCause::UnsatisfiedTimelock));
    assert!(result.aborts().contains(&FailureCause::NegativeTimelock));
}

#[test]
fn arithmetic_overflow_keeps_both_operands_and_pushes_a_false_above_them() {
    // The most important reviewed asymmetry: the failing path leaves a
    // *deeper* stack than the successful one.
    let result = validate(vec![
        push(8),
        push(8),
        TapscriptInstruction::Opcode(OpcodeId::Add64),
    ]);

    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![
            signed64(),
            StackValueType::Bool,
        ])],
    );
    assert_eq!(
        result
            .nonaborting_failure()
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![
            literal(8),
            literal(8),
            StackValueType::Empty,
        ])],
    );

    let success_depth = result.success().iter().map(AbstractStackState::depth).max();
    let failure_depth = result
        .nonaborting_failure()
        .iter()
        .map(AbstractStackState::depth)
        .max();
    assert_eq!(success_depth, Some(2));
    assert_eq!(failure_depth, Some(3));
}

#[test]
fn a_division_states_both_of_its_retained_failure_causes() {
    let result = validate(vec![
        push(8),
        push(8),
        TapscriptInstruction::Opcode(OpcodeId::Div64),
    ]);

    // Two causes, one retained shape: the state set is the union, not a
    // duplicate for each cause.
    assert_eq!(result.nonaborting_failure().len(), 1);
    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![
            signed64(),
            signed64(),
            StackValueType::Bool,
        ])],
    );
}

#[test]
fn a_nonempty_signature_verifies_or_aborts_and_is_never_the_empty_path() {
    let result = validate(vec![
        push(64),
        push(32),
        TapscriptInstruction::Opcode(OpcodeId::CheckSig),
    ]);

    // A 64-byte literal cannot be the empty item, so the
    // empty-signature path is not something this program can reach.
    // The operand contract is what makes that sayable: while the
    // position was one exact 64-byte type, the empty path was carried
    // as reachable by every program that used a signature at all.
    assert!(result.nonaborting_failure().is_empty());
    assert!(!result.aborts().contains(&FailureCause::EmptySignature));

    assert!(result.aborts().contains(&FailureCause::InvalidSignature));
    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![StackValueType::Bool])],
    );
}

#[test]
fn an_empty_signature_consumes_the_operands_and_pushes_a_false() {
    // The case the old model could not state at all: an empty item in
    // the signature position was refused as a type mismatch before the
    // failure effect describing it could apply.
    let result = validate(vec![
        push(0),
        push(32),
        TapscriptInstruction::Opcode(OpcodeId::CheckSig),
    ]);

    assert_eq!(
        result
            .nonaborting_failure()
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![StackValueType::Empty])],
    );
    // Nothing was verified, so no verification can have failed, and
    // there is no successful form either.
    assert!(!result.aborts().contains(&FailureCause::InvalidSignature));
    assert!(result.success().is_empty());
}

#[test]
fn the_verifying_signature_form_leaves_no_branchable_result() {
    let result = validate(vec![
        push(64),
        push(32),
        TapscriptInstruction::Opcode(OpcodeId::CheckSigVerify),
    ]);

    assert!(result.nonaborting_failure().is_empty());
    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(Vec::new())],
    );
}

#[test]
fn an_empty_signature_aborts_the_verifying_form() {
    let result = validate(vec![
        push(0),
        push(32),
        TapscriptInstruction::Opcode(OpcodeId::CheckSigVerify),
    ]);

    assert!(result.aborts().contains(&FailureCause::EmptySignature));
    assert!(result.nonaborting_failure().is_empty());
    assert!(result.success().is_empty());
}

#[test]
fn an_unknown_nonempty_key_type_succeeds_without_verification() {
    // The target's forward-compatibility rule. A 33-byte key is not
    // the recognized x-only encoding, so nothing is verified — and a
    // model that reported this as a rejection would say a spend fails
    // that in fact stands.
    let result = validate(vec![
        push(64),
        push(33),
        TapscriptInstruction::Opcode(OpcodeId::CheckSig),
    ]);

    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(vec![StackValueType::Bool])],
    );
    assert!(
        !result.aborts().contains(&FailureCause::InvalidSignature),
        "a verification that does not happen cannot fail: {:?}",
        result.aborts(),
    );
}

#[test]
fn an_empty_public_key_is_rejected_rather_than_treated_as_unknown() {
    let result = validate(vec![
        push(64),
        push(0),
        TapscriptInstruction::Opcode(OpcodeId::CheckSig),
    ]);

    assert!(result.aborts().contains(&FailureCause::EmptyPublicKey));
    assert!(
        result.success().is_empty(),
        "the empty key is the one nonrecognized form that does not succeed: {:?}",
        result.success(),
    );
}

#[test]
fn every_alternative_of_an_undecidable_discriminant_is_retained() {
    let target = reviewed_target();

    // Issuance present or absent: six items or one, and no abstract
    // state can say which.
    let issuance = validate(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectInputIssuance),
    ]);
    let depths: BTreeSet<usize> = issuance
        .success()
        .iter()
        .map(AbstractStackState::depth)
        .collect();
    assert_eq!(depths, [1, 6].into_iter().collect());

    // Explicit or confidential: same depth, different payload widths.
    let value = validate(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectOutputValue),
    ]);
    assert_eq!(value.success().len(), 2);
    assert!(value.success().iter().any(|state| state.main()
        == [
            StackValueType::EncodedPayload(EncodingClass::ExplicitValue),
            StackValueType::EncodingPrefix(EncodingClass::ExplicitValue),
        ]));
    assert!(value.success().iter().any(|state| state.main()
        == [
            StackValueType::EncodedPayload(EncodingClass::ConfidentialValue),
            StackValueType::EncodingPrefix(EncodingClass::ConfidentialValue),
        ]));

    // Explicit, confidential, or absent.
    let nonce = validate(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectOutputNonce),
    ]);
    assert_eq!(nonce.success().len(), 3);
    assert!(
        nonce
            .success()
            .iter()
            .any(|state| state.main() == [StackValueType::Encoded(EncodingClass::NullNonce)])
    );

    // A witness program or a digest standing in for one.
    let program = validate(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectInputScriptPubKey),
    ]);
    assert_eq!(program.success().len(), 2);

    // Nothing above depends on the target being read twice, but the
    // fixture does depend on it being the reviewed one.
    assert_eq!(
        target.definition().opcodes().len(),
        OpcodeId::ALL.len(),
        "the fixtures ran against the whole reviewed census",
    );
}

#[test]
fn alternatives_multiply_through_a_sequence() {
    // Two independent undecidable discriminants leave four states, and
    // the validator keeps all of them rather than picking a path.
    let result = validate(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectOutputNonce),
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectInputScriptPubKey),
    ]);

    assert_eq!(result.success().len(), 6);
}

#[test]
fn a_cause_the_abstract_state_settles_is_not_recorded_as_an_abort() {
    // Both operands have one decided width, so an operand of the wrong
    // width is not something this program can do.
    let decided = validate(vec![
        push(8),
        push(8),
        TapscriptInstruction::Opcode(OpcodeId::Add64),
    ]);
    assert!(
        !decided
            .aborts()
            .contains(&FailureCause::InvalidOperandWidth)
    );
    assert!(!decided.aborts().contains(&FailureCause::StackUnderflow));

    // The domain claim survives: nothing in an abstract stack says
    // which domain the program will run in.
    assert!(
        decided
            .aborts()
            .contains(&FailureCause::UnsupportedExecutionDomain),
    );
}

#[test]
fn an_operand_whose_width_is_unsettled_never_reaches_the_width_question() {
    // A stack carrying "somewhere between nothing and eight bytes" does
    // not satisfy an operand the contract fixes at eight, so the
    // program is refused rather than validated with the width failure
    // left on the table.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Neg64)])
        .expect("one instruction is within the limit");
    let unsettled = StackValueType::Bytes {
        minimum: 0,
        maximum: 8,
    };
    let initial = AbstractStackState::from_main(vec![unsettled.clone()]);

    assert_eq!(
        validate_program(
            &target,
            &program,
            &initial,
            AbstractLimits::for_target(&target),
        ),
        Err(TapscriptError::StackTypeMismatch {
            instruction: 0,
            expected: OperandContract::Exact(signed64()),
            actual: unsettled,
        }),
    );
}

#[test]
fn an_instruction_with_too_few_operands_is_a_validation_failure() {
    let target = reviewed_target();
    let program =
        TapscriptProgram::new(vec![push(8), TapscriptInstruction::Opcode(OpcodeId::Add64)])
            .expect("two instructions are within the limit");

    assert_eq!(
        validate_program(
            &target,
            &program,
            &AbstractStackState::from_main(Vec::new()),
            AbstractLimits::for_target(&target),
        ),
        Err(TapscriptError::StackUnderflow { instruction: 1 }),
    );
}

#[test]
fn an_operand_of_the_wrong_shape_is_a_validation_failure() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        push(4),
        push(4),
        TapscriptInstruction::Opcode(OpcodeId::Add64),
    ])
    .expect("three instructions are within the limit");

    assert_eq!(
        validate_program(
            &target,
            &program,
            &AbstractStackState::from_main(Vec::new()),
            AbstractLimits::for_target(&target),
        ),
        Err(TapscriptError::StackTypeMismatch {
            instruction: 2,
            expected: OperandContract::Exact(signed64()),
            actual: literal(4),
        }),
    );
}

#[test]
fn exhausting_the_state_budget_returns_no_partial_result() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectOutputNonce),
    ])
    .expect("two instructions are within the limit");
    let limits = AbstractLimits::for_target(&target)
        .with_maximum_states(NonZeroU64::new(2).expect("two is not zero"));

    assert_eq!(
        validate_program(
            &target,
            &program,
            &AbstractStackState::from_main(Vec::new()),
            limits,
        ),
        Err(TapscriptError::AbstractStateLimitExceeded { maximum: 2 }),
    );
}

#[test]
fn exhausting_the_depth_budget_returns_no_partial_result() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![push(1), push(1)])
        .expect("two instructions are within the limit");
    let limits = AbstractLimits::for_target(&target).with_maximum_stack_depth(1);

    assert_eq!(
        validate_program(
            &target,
            &program,
            &AbstractStackState::from_main(Vec::new()),
            limits,
        ),
        Err(TapscriptError::StackLimitExceeded { maximum: 1 }),
    );
}

#[test]
fn exhausting_the_instruction_budget_returns_no_partial_result() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![push(1), push(1)])
        .expect("two instructions are within the limit");
    let limits = AbstractLimits::for_target(&target)
        .with_maximum_instructions(NonZeroU64::new(1).expect("one is not zero"));

    assert_eq!(
        validate_program(
            &target,
            &program,
            &AbstractStackState::from_main(Vec::new()),
            limits,
        ),
        Err(TapscriptError::InstructionLimitExceeded { maximum: 1 }),
    );
}

#[test]
fn exhausting_the_alternative_budget_returns_no_partial_result() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        push(1),
        TapscriptInstruction::Opcode(OpcodeId::InspectOutputNonce),
    ])
    .expect("two instructions are within the limit");
    let limits = AbstractLimits::for_target(&target)
        .with_maximum_result_alternatives(NonZeroU64::new(2).expect("two is not zero"));

    assert_eq!(
        validate_program(
            &target,
            &program,
            &AbstractStackState::from_main(Vec::new()),
            limits,
        ),
        Err(TapscriptError::ResultAlternativeLimitExceeded { maximum: 2 }),
    );
}

#[test]
fn a_limit_can_only_be_narrowed() {
    let target = reviewed_target();
    let default = AbstractLimits::for_target(&target);
    let widened = default
        .with_maximum_stack_depth(u64::MAX)
        .with_maximum_states(NonZeroU64::MAX)
        .with_maximum_instructions(NonZeroU64::MAX)
        .with_maximum_result_alternatives(NonZeroU64::MAX);

    assert_eq!(widened, default);
    assert_eq!(default.maximum_stack_depth(), 1_000);
}

#[test]
fn a_resource_projection_states_each_unit_separately() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![
        push(64),
        push(32),
        TapscriptInstruction::Opcode(OpcodeId::CheckSig),
        TapscriptInstruction::Opcode(OpcodeId::TxWeight),
    ])
    .expect("four instructions are within the limit");

    let projection = resource_projection(&target, &program);

    // Two primitives, one of which can charge the per-check budget, and
    // two literals that charge neither. Script bytes are the program's
    // own encoded length and so count all four instructions: two
    // payloads of sixty-four and thirty-two bytes, their two push
    // opcodes, and the two primitive bytes.
    assert_eq!(projection.get(&ResourceDimension::ScriptBytes), Some(&100));
    assert_eq!(
        projection.get(&ResourceDimension::ValidationBudget),
        Some(&50),
    );
    assert_eq!(projection.get(&ResourceDimension::OperationCost), Some(&0));
}

// --- Computed truth (`G13-R11`) ---------------------------------------

/// A literal of exactly these bytes, as an instruction.
fn push_bytes(payload: &[u8]) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(
        StackItem::new(&target, payload.to_vec()).expect("the payload is within the bound"),
    )
}

/// The target's signed fixed-width encoding of `value`, as a push.
fn push_le64(value: i64) -> TapscriptInstruction {
    let target = reviewed_target();
    TapscriptInstruction::Push(StackItem::signed_le64(&target, value))
}

#[test]
fn a_comparison_of_unequal_literals_settles_its_result_false() {
    // Read through the primitive that consumes a truth value, because
    // that is where the knowledge is spent. A settled false removes the
    // successful form of the verification, and the abort it removes
    // nothing from stays: both halves are asserted, since a repair that
    // simply dropped every state would satisfy the first alone.
    let result = validate(vec![
        push_bytes(&[0x01]),
        push_bytes(&[0x02]),
        TapscriptInstruction::Opcode(OpcodeId::Equal),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ]);

    assert!(result.success().is_empty());
    assert!(result.always_aborts());
    assert!(result.aborts().contains(&FailureCause::FalseVerification));
}

#[test]
fn a_comparison_of_equal_literals_settles_its_result_true() {
    // The other direction, and the sharper assertion of the two:
    // removing an abort is a claim, and only exact knowledge licenses
    // it. A walk that merely kept the successful form would still list
    // the false verification as reachable.
    let result = validate(vec![
        push_bytes(&[0x01]),
        push_bytes(&[0x01]),
        TapscriptInstruction::Opcode(OpcodeId::Equal),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ]);

    assert!(!result.success().is_empty());
    assert!(!result.aborts().contains(&FailureCause::FalseVerification));
}

#[test]
fn equality_is_settled_on_the_bytes_and_not_on_how_they_read() {
    // Both operands are items the target reads as false, and they are
    // not the same item. A comparison settled on the reading rather
    // than on the bytes would call them equal and would keep a state
    // this program cannot reach.
    let result = validate(vec![
        push_bytes(&[0x00]),
        push_bytes(&[]),
        TapscriptInstruction::Opcode(OpcodeId::Equal),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ]);

    assert!(result.always_aborts());
}

#[test]
fn a_fixed_width_ordering_of_known_operands_settles_its_result() {
    // Operands are declared deepest first, so this is one against two
    // and not two against one. An ordering that read the pair in the
    // other direction would settle the opposite value, and these two
    // results would cross over rather than merely blur.
    let less = validate(vec![
        push_le64(1),
        push_le64(2),
        TapscriptInstruction::Opcode(OpcodeId::LessThan64),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ]);
    assert!(!less.success().is_empty());
    assert!(!less.aborts().contains(&FailureCause::FalseVerification));

    let greater = validate(vec![
        push_le64(1),
        push_le64(2),
        TapscriptInstruction::Opcode(OpcodeId::GreaterThan64),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ]);
    assert!(greater.always_aborts());
}

#[test]
fn a_settled_truth_never_answers_a_question_about_bytes() {
    // The reviewed contract states a computed Boolean as a type and
    // never says which bytes carry it. So the comparison below is
    // settled by nothing, and `EQUALVERIFY` keeps both its successful
    // form and its inequality abort. Recording a plausible canonical
    // byte for the truth would decide this, and would decide it with no
    // ground in the contract — which is why the walk records the truth
    // and not the bytes.
    let result = validate(vec![
        push_bytes(&[0x01]),
        push_bytes(&[0x01]),
        TapscriptInstruction::Opcode(OpcodeId::Equal),
        push_bytes(&[0x01]),
        TapscriptInstruction::Opcode(OpcodeId::EqualVerify),
    ]);

    assert!(!result.success().is_empty());
    assert!(result.aborts().contains(&FailureCause::UnequalOperands));
}

// --- Which signature form a program reaches (Guide-13 §1.8) ---------

/// One reviewed primitive, as an instruction.
const fn op(id: OpcodeId) -> TapscriptInstruction {
    TapscriptInstruction::Opcode(id)
}

/// The forms one signature primitive can reach at `index`.
///
/// # Panics
///
/// If the program schedules no signature primitive at `index`, which
/// would make the fixture rather than the walk the thing under test.
fn forms(result: &AbstractExecutionResult, index: usize) -> BTreeSet<SignatureSuccessForm> {
    result
        .signature_forms()
        .get(&index)
        .cloned()
        .expect("the fixture schedules a signature primitive there")
}

#[test]
fn a_recognized_key_leaves_only_the_verified_form() {
    let result = validate(vec![push(64), push(32), op(OpcodeId::CheckSigVerify)]);

    assert_eq!(
        forms(&result, 2),
        BTreeSet::from([SignatureSuccessForm::RecognizedKeyVerified]),
    );
    assert!(!result.reaches_unverified_signature_success());
}

#[test]
fn an_unknown_key_type_leaves_only_the_unverified_form() {
    // The §1.8 danger, as the walk sees it: the program still has a
    // successful path, and nothing on that path verified anything.
    let result = validate(vec![push(64), push(33), op(OpcodeId::CheckSigVerify)]);

    assert_eq!(
        forms(&result, 2),
        BTreeSet::from([SignatureSuccessForm::UnknownKeyTypeUnverified]),
    );
    assert!(result.reaches_unverified_signature_success());
}

#[test]
fn an_empty_key_leaves_a_signature_primitive_with_no_successful_form() {
    let result = validate(vec![push(64), push(0), op(OpcodeId::CheckSigVerify)]);

    // Present and empty, which is the finding: the instruction is a
    // signature check, and no form of it survives. An absent entry
    // would have said the program verifies no signature at all.
    assert_eq!(forms(&result, 2), BTreeSet::new());
    assert!(result.success().is_empty());
}

#[test]
fn an_unsettled_key_width_leaves_both_forms_open() {
    // Nothing here pushed the key, so its width is whatever the witness
    // supplies. Both forms stay, which is the honest answer and the
    // reason a program that means to authorize pushes the key itself.
    let target = reviewed_target();
    let program =
        TapscriptProgram::new(vec![op(OpcodeId::CheckSigVerify)]).expect("the fixture is short");
    let initial = AbstractStackState::from_main(vec![
        StackValueType::Encoded(EncodingClass::SchnorrSignature),
        StackValueType::Bytes {
            minimum: 1,
            maximum: 40,
        },
    ]);
    let result = validate_program(
        &target,
        &program,
        &initial,
        AbstractLimits::for_target(&target),
    )
    .expect("the fixture validates");

    assert_eq!(
        forms(&result, 0),
        BTreeSet::from([
            SignatureSuccessForm::RecognizedKeyVerified,
            SignatureSuccessForm::UnknownKeyTypeUnverified,
        ]),
    );
}

#[test]
fn a_program_verifying_no_signature_reports_no_forms() {
    let result = validate(vec![push(4), push(4), op(OpcodeId::EqualVerify)]);

    assert!(result.signature_forms().is_empty());
}

#[test]
fn the_two_verifying_forms_are_indistinguishable_by_stack_shape() {
    // The whole reason the forms are reported beside the states. Both
    // programs reach exactly one successful state, and it is the same
    // state; only one of them verified anything.
    let recognized = validate(vec![push(64), push(32), op(OpcodeId::CheckSigVerify)]);
    let unknown = validate(vec![push(64), push(33), op(OpcodeId::CheckSigVerify)]);

    assert_eq!(recognized.success(), unknown.success());
    assert_ne!(recognized.signature_forms(), unknown.signature_forms());
}
