//! Static schedule checks, not observations of a concrete transaction.

use std::collections::BTreeSet;

use target_elements::{EncodingClass, FailureCause, StackValueType};

use crate::keyed_pattern::born_first_fragment;
use crate::tests::reviewed_target;
use crate::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, StackItem, TapscriptInstruction,
    TapscriptProgram, program_stack_profile, validate_program,
};

fn fragment() -> TapscriptProgram {
    born_first_fragment(&reviewed_target()).expect("fragment")
}

fn success(main: Vec<StackValueType>) -> BTreeSet<AbstractStackState> {
    BTreeSet::from([AbstractStackState::from_main(main)])
}

fn execute(program: &TapscriptProgram) -> AbstractExecutionResult {
    let target = reviewed_target();
    validate_program(
        &target,
        program,
        &AbstractStackState::from_main(vec![]),
        AbstractLimits::for_target(&target),
    )
    .expect("the schedule is well typed")
}

fn aborts() -> BTreeSet<FailureCause> {
    BTreeSet::from([
        FailureCause::UnsupportedExecutionDomain,
        FailureCause::IntrospectionContextUnavailable,
        FailureCause::MalformedScriptNumber,
        FailureCause::IntrospectionIndexOutOfRange,
        FailureCause::UnequalOperands,
    ])
}

const fn bytes(width: usize) -> StackValueType {
    StackValueType::Bytes {
        minimum: width,
        maximum: width,
    }
}

#[test]
fn birth_has_exact_empty_success_and_failure_sets() {
    let result = execute(&fragment());
    assert_eq!(
        result.success(),
        &BTreeSet::from([AbstractStackState::new(vec![], vec![])])
    );
    assert_eq!(result.nonaborting_failure(), &BTreeSet::new());
    assert_eq!(result.aborts(), &aborts());
    assert!(result.signature_forms().is_empty());
}

#[test]
fn every_birth_prefix_has_the_reviewed_stack_and_failure_sets() {
    use EncodingClass::{OutPointFlags, OutPointIndex, OutPointTxid};
    use StackValueType::{Encoded, ScriptNumber};

    let program = fragment();
    let shapes = [
        vec![],
        vec![ScriptNumber],
        vec![
            Encoded(OutPointTxid),
            Encoded(OutPointIndex),
            Encoded(OutPointFlags),
        ],
        vec![Encoded(OutPointTxid), Encoded(OutPointIndex)],
        vec![Encoded(OutPointTxid), Encoded(OutPointIndex), bytes(4)],
        vec![Encoded(OutPointTxid)],
        vec![],
    ];
    for (length, shape) in shapes.into_iter().enumerate() {
        let prefix =
            TapscriptProgram::new(program.instructions()[..length].to_vec()).expect("prefix");
        let result = execute(&prefix);
        let expected_aborts = match length {
            0 => BTreeSet::new(),
            1 => BTreeSet::from([
                FailureCause::UnsupportedExecutionDomain,
                FailureCause::IntrospectionContextUnavailable,
            ]),
            2..=4 => aborts()
                .difference(&BTreeSet::from([FailureCause::UnequalOperands]))
                .copied()
                .collect(),
            _ => aborts(),
        };
        assert_eq!(result.success(), &success(shape));
        assert_eq!(result.nonaborting_failure(), &BTreeSet::new());
        assert_eq!(result.aborts(), &expected_aborts);
    }
}

#[test]
fn birth_peaks_at_three_main_items_and_no_alternate_items() {
    let target = reviewed_target();
    let program = born_first_fragment(&target).expect("fragment");
    let profile = program_stack_profile(
        &target,
        &program,
        &AbstractStackState::from_main(vec![]),
        AbstractLimits::for_target(&target),
    );
    assert_eq!((profile.peak_main(), profile.peak_alternate()), (3, 0));
}

#[test]
fn birth_bytes_pin_the_four_byte_zero_and_round_trip() {
    let target = reviewed_target();
    let program = born_first_fragment(&target).expect("fragment");
    assert_eq!(
        program.encode(&target),
        [0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75]
    );
    assert_eq!(program.encoded_length(&target), 10);
    assert_eq!(
        TapscriptProgram::decode(&target, &program.encode(&target)),
        Ok(program.clone())
    );
    assert_eq!(
        program.instructions()[3],
        TapscriptInstruction::Push(
            StackItem::encoded(&target, EncodingClass::OutPointIndex, vec![0; 4])
                .expect("index literal")
        )
    );
    assert!(StackItem::encoded(&target, EncodingClass::OutPointIndex, vec![]).is_err());
}

#[test]
fn standalone_birth_appends_exactly_one_true_item() {
    let target = reviewed_target();
    let mut instructions = fragment().instructions().to_vec();
    instructions.extend(
        crate::pattern::final_truth_fragment(&target)
            .expect("truth")
            .instructions()
            .to_vec(),
    );
    let program = TapscriptProgram::new(instructions).expect("composed program");
    let result = execute(&program);
    assert_eq!(result.success(), &success(vec![bytes(1)]));
    assert_eq!(result.nonaborting_failure(), &BTreeSet::new());
    assert_eq!(result.aborts(), &aborts());
    assert_eq!(program.encode(&target).last(), Some(&0x51));
}

#[test]
fn missing_flags_drop_compares_incompatible_widths() {
    let mut instructions = fragment().instructions().to_vec();
    instructions.remove(2);
    let result = execute(&TapscriptProgram::new(instructions).expect("mutant"));
    assert!(result.always_aborts());
    assert_eq!(result.aborts(), &aborts());
}

#[test]
fn missing_txid_drop_leaves_residue() {
    let mut instructions = fragment().instructions().to_vec();
    instructions.pop();
    let result = execute(&TapscriptProgram::new(instructions).expect("mutant"));
    assert_eq!(
        result.success(),
        &success(vec![StackValueType::Encoded(EncodingClass::OutPointTxid)])
    );
}

#[test]
fn missing_equality_leaves_the_outpoint_and_loses_inequality_refusal() {
    let mut instructions = fragment().instructions().to_vec();
    instructions.remove(4);
    let result = execute(&TapscriptProgram::new(instructions).expect("mutant"));
    assert_eq!(
        result.success(),
        &success(vec![
            StackValueType::Encoded(EncodingClass::OutPointTxid),
            StackValueType::Encoded(EncodingClass::OutPointIndex),
        ])
    );
    assert!(!result.aborts().contains(&FailureCause::UnequalOperands));
}

#[test]
fn numeric_zero_is_not_the_outpoint_zero_literal() {
    let target = reviewed_target();
    let mut instructions = fragment().instructions().to_vec();
    instructions[3] =
        TapscriptInstruction::Push(StackItem::script_number(&target, 0).expect("number"));
    let result = execute(&TapscriptProgram::new(instructions).expect("mutant"));
    assert!(result.always_aborts());
    assert_eq!(result.aborts(), &aborts());
}

#[test]
fn same_width_wrong_literal_needs_native_subjects_to_decide_birth() {
    let target = reviewed_target();
    let program = born_first_fragment(&target).expect("fragment");
    let mut instructions = program.instructions().to_vec();
    instructions[3] = TapscriptInstruction::Push(
        StackItem::encoded(&target, EncodingClass::OutPointIndex, vec![1, 0, 0, 0])
            .expect("wrong index"),
    );
    let mutant = TapscriptProgram::new(instructions).expect("mutant");
    assert_ne!(mutant.encode(&target), program.encode(&target));
    // Unknown transaction bytes have the same abstract type at either index.
    assert_eq!(execute(&mutant), execute(&program));
}
