//! Independent cross-checks of the reviewed primitive registry.
//!
//! The expected values in this file are written out by hand from the
//! same review that produced the registry. Nothing here derives an
//! expectation by reading the registry it is checking: an oracle that
//! asks the subject for the answer proves only that the subject is
//! self-consistent.

use std::collections::{BTreeMap, BTreeSet};

use crate::definition::reviewed_elements_tapscript;
use crate::encoding::{ByteOrder, EncodingClass};
use crate::opcode::{
    ExecutionDomain, FailureCause, FailureOutcome, LeafVersion, OpcodeId, OpcodeSpec,
    StackValueType, VALIDATION_BUDGET_PER_CHECK,
};

/// The expected identity-to-byte census, stated independently.
///
/// These bytes come from the reviewed upstream opcode declaration, not
/// from a lookup through the registry under test.
const EXPECTED_CODES: &[(OpcodeId, u8)] = &[
    (OpcodeId::CheckSig, 0xac),
    (OpcodeId::CheckSigVerify, 0xad),
    (OpcodeId::CheckSequenceVerify, 0xb2),
    (OpcodeId::CheckSigFromStack, 0xc1),
    (OpcodeId::CheckSigFromStackVerify, 0xc2),
    (OpcodeId::Sha256Initialize, 0xc4),
    (OpcodeId::Sha256Update, 0xc5),
    (OpcodeId::Sha256Finalize, 0xc6),
    (OpcodeId::InspectInputOutpoint, 0xc7),
    (OpcodeId::InspectInputAsset, 0xc8),
    (OpcodeId::InspectInputValue, 0xc9),
    (OpcodeId::InspectInputScriptPubKey, 0xca),
    (OpcodeId::InspectInputSequence, 0xcb),
    (OpcodeId::InspectInputIssuance, 0xcc),
    (OpcodeId::PushCurrentInputIndex, 0xcd),
    (OpcodeId::InspectOutputAsset, 0xce),
    (OpcodeId::InspectOutputValue, 0xcf),
    (OpcodeId::InspectOutputNonce, 0xd0),
    (OpcodeId::InspectOutputScriptPubKey, 0xd1),
    (OpcodeId::InspectVersion, 0xd2),
    (OpcodeId::InspectLockTime, 0xd3),
    (OpcodeId::InspectNumInputs, 0xd4),
    (OpcodeId::InspectNumOutputs, 0xd5),
    (OpcodeId::TxWeight, 0xd6),
    (OpcodeId::Add64, 0xd7),
    (OpcodeId::Sub64, 0xd8),
    (OpcodeId::Mul64, 0xd9),
    (OpcodeId::Div64, 0xda),
    (OpcodeId::Neg64, 0xdb),
    (OpcodeId::LessThan64, 0xdc),
    (OpcodeId::LessThanOrEqual64, 0xdd),
    (OpcodeId::GreaterThan64, 0xde),
    (OpcodeId::GreaterThanOrEqual64, 0xdf),
    (OpcodeId::ScriptNumToLe64, 0xe0),
    (OpcodeId::Le64ToScriptNum, 0xe1),
    (OpcodeId::Le32ToLe64, 0xe2),
    (OpcodeId::EcMulScalarVerify, 0xe3),
    (OpcodeId::TweakVerify, 0xe4),
];

/// Fetches one specification from the built-in reviewed contract.
fn spec(id: OpcodeId) -> OpcodeSpec {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    definition
        .definition()
        .opcodes()
        .get(&id)
        .expect("every reviewed identity has a contract")
        .clone()
}

#[test]
fn the_reviewed_contract_validates() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(
        definition.definition().execution_domain(),
        ExecutionDomain::Tapscript
    );
    assert_eq!(
        definition.definition().leaf_version(),
        LeafVersion::TAPSCRIPT
    );
}

#[test]
fn the_leaf_version_is_the_target_specific_byte() {
    // The reviewed target does not share upstream Bitcoin's tapscript
    // leaf version. Transcribing the upstream constant would have
    // produced a contract that validates cleanly and describes the
    // wrong leaf, so the byte is asserted literally here.
    assert_eq!(LeafVersion::TAPSCRIPT.get(), 0xc4);
    assert_ne!(LeafVersion::TAPSCRIPT.get(), 0xc0);

    assert!(LeafVersion::new(0xc4).is_ok());
    for unreviewed in [0x00_u8, 0xc0, 0xc5, 0xfe, 0xff] {
        assert!(
            LeafVersion::new(unreviewed).is_err(),
            "leaf version {unreviewed:#04x} was not reviewed"
        );
    }
}

#[test]
fn the_opcode_byte_census_matches_the_independent_table() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let registry = definition.definition().opcodes();

    let expected: BTreeMap<OpcodeId, u8> = EXPECTED_CODES.iter().copied().collect();
    assert_eq!(
        expected.len(),
        EXPECTED_CODES.len(),
        "the independent table itself must be duplicate-free"
    );

    let actual: BTreeMap<OpcodeId, u8> = registry
        .iter()
        .map(|(id, spec)| (*id, spec.code()))
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn the_identity_census_is_exact_in_both_directions() {
    let declared: BTreeSet<OpcodeId> = OpcodeId::ALL.iter().copied().collect();
    assert_eq!(
        declared.len(),
        OpcodeId::ALL.len(),
        "the identity census is duplicate-free"
    );

    let tabled: BTreeSet<OpcodeId> = EXPECTED_CODES.iter().map(|(id, _)| *id).collect();
    assert_eq!(declared, tabled, "census and independent table agree");

    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let registered: BTreeSet<OpcodeId> =
        definition.definition().opcodes().keys().copied().collect();
    assert_eq!(registered, declared, "registry and census agree");
}

#[test]
fn declaration_order_is_not_target_byte_order() {
    // A consumer must read the byte from the specification. This test
    // exists so that the day someone reorders the census for tidiness,
    // nothing silently starts depending on the new order.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let registry = definition.definition().opcodes();

    let declaration_ordered: Vec<u8> = OpcodeId::ALL
        .iter()
        .map(|id| registry.get(id).expect("registered").code())
        .collect();
    let mut ascending = declaration_ordered.clone();
    ascending.sort_unstable();

    assert_ne!(
        declaration_ordered, ascending,
        "the grouping order deliberately differs from byte order"
    );
}

#[test]
fn arithmetic_failure_retains_its_operands() {
    // The single most important reviewed fact in the registry: on
    // overflow the operands are left in place and a false is pushed
    // above them, so the failing path leaves a *deeper* stack than the
    // succeeding one.
    for id in [
        OpcodeId::Add64,
        OpcodeId::Sub64,
        OpcodeId::Mul64,
        OpcodeId::Neg64,
    ] {
        let spec = spec(id);
        let retaining: Vec<FailureCause> = spec
            .stack()
            .failure()
            .effects()
            .iter()
            .filter(|effect| effect.outcome() == FailureOutcome::RetainOperandsPushFalse)
            .map(|effect| effect.cause())
            .collect();
        assert_eq!(
            retaining,
            vec![FailureCause::ArithmeticOverflow],
            "{id:?} retains its operands only on overflow"
        );
    }

    let division = spec(OpcodeId::Div64);
    let retaining: BTreeSet<FailureCause> = division
        .stack()
        .failure()
        .effects()
        .iter()
        .filter(|effect| effect.outcome() == FailureOutcome::RetainOperandsPushFalse)
        .map(|effect| effect.cause())
        .collect();
    assert_eq!(
        retaining,
        BTreeSet::from([
            FailureCause::ArithmeticOverflow,
            FailureCause::DivisionByZero,
        ])
    );
}

#[test]
fn comparison_failure_never_retains_operands() {
    // Comparisons always consume both operands and push exactly one
    // item. The false they can push is the answer, not a failure flag,
    // so it must not appear as a failure effect.
    for id in [
        OpcodeId::LessThan64,
        OpcodeId::LessThanOrEqual64,
        OpcodeId::GreaterThan64,
        OpcodeId::GreaterThanOrEqual64,
    ] {
        let spec = spec(id);
        assert_eq!(spec.stack().success_results(), &[StackValueType::Bool]);
        assert!(
            spec.stack()
                .failure()
                .effects()
                .iter()
                .all(|effect| effect.outcome() == FailureOutcome::AbortEvaluation),
            "{id:?} can only abort"
        );
        assert_eq!(spec.resources().maximum_stack_growth(), -1);
    }
}

#[test]
fn arithmetic_operands_and_results_are_exactly_eight_bytes_little_endian() {
    let expected = StackValueType::SignedFixedWidth {
        bytes: std::num::NonZeroUsize::new(8).expect("8 is not zero"),
        byte_order: ByteOrder::LittleEndian,
    };

    let addition = spec(OpcodeId::Add64);
    assert_eq!(
        addition.stack().operands(),
        &[expected.clone(), expected.clone()]
    );
    assert_eq!(
        addition.stack().success_results(),
        &[expected.clone(), StackValueType::Bool]
    );

    // Division alone pushes three items: remainder, quotient, flag.
    let division = spec(OpcodeId::Div64);
    assert_eq!(
        division.stack().success_results(),
        &[expected.clone(), expected, StackValueType::Bool]
    );
}

#[test]
fn asset_and_value_introspection_split_payload_from_prefix() {
    // The interpreter pushes the payload first and the prefix second,
    // as two separate items. A backend that expected one combined item
    // would misjudge every stack depth after the inspection.
    for (id, class) in [
        (OpcodeId::InspectInputAsset, EncodingClass::ExplicitAsset),
        (OpcodeId::InspectOutputAsset, EncodingClass::ExplicitAsset),
        (OpcodeId::InspectInputValue, EncodingClass::ExplicitValue),
        (OpcodeId::InspectOutputValue, EncodingClass::ExplicitValue),
    ] {
        let spec = spec(id);
        assert_eq!(
            spec.stack().success_results(),
            &[
                StackValueType::EncodedPayload(class),
                StackValueType::EncodingPrefix(class),
            ],
            "{id:?} splits payload from prefix, prefix on top"
        );
    }
}

#[test]
fn nonce_introspection_keeps_its_prefix_inline() {
    // The one asymmetric case among the reviewed introspection
    // primitives.
    let spec = spec(OpcodeId::InspectOutputNonce);
    assert_eq!(
        spec.stack().success_results(),
        &[StackValueType::Encoded(EncodingClass::ExplicitNonce)]
    );
    assert_eq!(spec.resources().maximum_stack_growth(), 0);
}

#[test]
fn signature_verification_distinguishes_empty_from_invalid() {
    // An empty signature consumes the operands and pushes a false; a
    // non-empty signature that does not verify aborts. A backend
    // cannot treat "verification failed" as a branchable condition.
    for id in [OpcodeId::CheckSig, OpcodeId::CheckSigFromStack] {
        let spec = spec(id);
        let effects: BTreeMap<FailureCause, FailureOutcome> = spec
            .stack()
            .failure()
            .effects()
            .iter()
            .map(|effect| (effect.cause(), effect.outcome()))
            .collect();
        assert_eq!(
            effects.get(&FailureCause::EmptySignature),
            Some(&FailureOutcome::ConsumeOperandsPushFalse),
            "{id:?} branches on an empty signature"
        );
        assert_eq!(
            effects.get(&FailureCause::InvalidSignature),
            Some(&FailureOutcome::AbortEvaluation),
            "{id:?} aborts on an invalid signature"
        );
    }

    // The verifying forms leave no branchable result at all.
    for id in [OpcodeId::CheckSigVerify, OpcodeId::CheckSigFromStackVerify] {
        let spec = spec(id);
        assert_eq!(spec.stack().success_results(), []);
        assert!(
            spec.stack()
                .failure()
                .effects()
                .iter()
                .all(|effect| effect.outcome() == FailureOutcome::AbortEvaluation),
            "{id:?} can only abort"
        );
    }
}

#[test]
fn only_the_curve_and_signature_primitives_charge_validation_budget() {
    let charging: BTreeSet<OpcodeId> = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .opcodes()
        .iter()
        .filter(|(_, spec)| spec.resources().validation_budget() > 0)
        .map(|(id, _)| *id)
        .collect();

    assert_eq!(
        charging,
        BTreeSet::from([
            OpcodeId::CheckSig,
            OpcodeId::CheckSigVerify,
            OpcodeId::CheckSigFromStack,
            OpcodeId::CheckSigFromStackVerify,
            OpcodeId::EcMulScalarVerify,
            OpcodeId::TweakVerify,
        ])
    );

    for id in &charging {
        assert_eq!(
            spec(*id).resources().validation_budget(),
            VALIDATION_BUDGET_PER_CHECK
        );
    }
    assert_eq!(VALIDATION_BUDGET_PER_CHECK, 50);
}

#[test]
fn no_reviewed_primitive_touches_the_alternate_stack_or_an_operation_budget() {
    // Both are reviewed target facts rather than unfilled fields: the
    // reviewed domain enforces no per-script operation budget, and no
    // reviewed primitive reaches the alternate stack.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    for (id, spec) in definition.definition().opcodes() {
        assert_eq!(spec.resources().maximum_altstack_growth(), 0, "{id:?}");
        assert_eq!(spec.resources().operation_cost(), 0, "{id:?}");
        assert_eq!(spec.resources().script_bytes(), 1, "{id:?}");
    }
}

#[test]
fn the_verify_style_curve_primitives_push_nothing() {
    for id in [OpcodeId::EcMulScalarVerify, OpcodeId::TweakVerify] {
        let spec = spec(id);
        assert_eq!(spec.stack().operands().len(), 3, "{id:?}");
        assert!(spec.stack().success_results().is_empty(), "{id:?}");
        assert_eq!(spec.resources().maximum_stack_growth(), -3, "{id:?}");
    }
}

#[test]
fn the_timelock_primitive_neither_pushes_nor_pops() {
    // The operand is inspected in place. Modelling it as consumed
    // would make every program that uses it one item short.
    let spec = spec(OpcodeId::CheckSequenceVerify);
    assert_eq!(spec.stack().operands(), &[StackValueType::ScriptNumber]);
    assert_eq!(spec.stack().success_results(), []);
    assert_eq!(spec.resources().maximum_stack_growth(), 0);
}

#[test]
fn every_primitive_is_gated_to_the_reviewed_domain_and_names_evidence() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    for (id, spec) in definition.definition().opcodes() {
        assert_eq!(
            spec.domains(),
            &BTreeSet::from([ExecutionDomain::Tapscript]),
            "{id:?}"
        );
        assert!(!spec.stack().failure().is_empty(), "{id:?} can fail");
        assert!(!spec.evidence().is_empty(), "{id:?} names evidence");
        assert!(
            spec.stack().failure().contradictory_cause().is_none(),
            "{id:?}"
        );
    }
}
