//! Operator contracts and independent mutation walks over public fixture bytes.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::RequiredSourceKind;
use target_elements::{
    EncodingClass, FailureCause, OpcodeId, ResourceDimension, StackValueType,
    TargetEvidenceRequirementId,
};

use super::reviewed_target;
use crate::pattern::op;
use crate::state_operator::{
    StateOperatorBindings, StateOperatorDisclosure, StateOperatorOwner, StateOperatorPattern,
    StateOperatorPatternId, StateOperatorRefusal, StateOperatorResidual, StateOperatorSymbol,
    StateOperatorWitness, build_state_operator_pattern, state_operator_fragment,
};
use crate::{
    AbstractLimits, AbstractStackState, SignatureSuccessForm, StackItem,
    StatePatternConstructibility, TapscriptInstruction, TapscriptProgram, resource_projection,
    selected_operator_profile, validate_program,
};

fn values() -> BTreeMap<StateOperatorSymbol, StackItem> {
    BTreeMap::from([(
        StateOperatorSymbol::CommittedOperatorKey,
        StackItem::encoded(
            &reviewed_target(),
            EncodingClass::XOnlyPublicKey,
            vec![0x33; 32],
        )
        .unwrap(),
    )])
}
fn bindings() -> StateOperatorBindings {
    StateOperatorBindings::new(&reviewed_target(), &values()).unwrap()
}
fn pattern() -> StateOperatorPattern {
    let bindings = bindings();
    build_state_operator_pattern(
        &reviewed_target(),
        &bindings,
        state_operator_fragment(&bindings).unwrap(),
    )
    .unwrap()
}
const fn signature() -> StackValueType {
    StackValueType::Encoded(EncodingClass::SchnorrSignature)
}

#[test]
fn walked_contract_consumes_exactly_one_signature_and_no_boolean_survives() {
    let pattern = pattern();
    assert_eq!(
        pattern.id(),
        StateOperatorPatternId::OperatorAuthorizationV1
    );
    assert_eq!(pattern.owner(), StateOperatorOwner::OperatorAuthorization);
    assert_eq!(pattern.witness(), StateOperatorWitness::OneSignature);
    assert_eq!(
        pattern.precondition(),
        &AbstractStackState::from_main(vec![signature()])
    );
    assert_eq!(
        pattern.execution().success(),
        &BTreeSet::from([AbstractStackState::from_main(vec![])])
    );
    assert_eq!(pattern.execution().nonaborting_failure(), &BTreeSet::new());
    assert!(
        pattern
            .execution()
            .aborts()
            .contains(&FailureCause::InvalidSignature)
    );
    assert_eq!(pattern.profile(), &selected_operator_profile());
    assert_eq!(
        pattern.fragment().instructions(),
        &[
            TapscriptInstruction::Push(
                values()[&StateOperatorSymbol::CommittedOperatorKey].clone()
            ),
            op(OpcodeId::CheckSigVerify)
        ]
    );
}

#[test]
fn signature_form_census_contains_only_recognized_verification() {
    let pattern = pattern();
    assert_eq!(
        pattern.execution().signature_forms(),
        &BTreeMap::from([(
            1,
            BTreeSet::from([SignatureSuccessForm::RecognizedKeyVerified])
        )])
    );
    assert!(!pattern.execution().reaches_unverified_signature_success());
}

#[test]
fn consumer_census_names_the_committed_program_push_only() {
    let pattern = pattern();
    assert_eq!(
        pattern.consumers(),
        &BTreeMap::from([(
            StateOperatorSymbol::CommittedOperatorKey,
            BTreeSet::from([0])
        )])
    );
    assert_eq!(
        pattern.constructibility(),
        StatePatternConstructibility::CheckedUnresolvedConsumers
    );
    assert_eq!(
        StateOperatorBindings::new(&reviewed_target(), &BTreeMap::new()),
        Err(StateOperatorRefusal::ConsumerCensus)
    );
    // The closed one-symbol map cannot represent an additional unused key.
    assert_eq!(
        StateOperatorSymbol::ALL,
        &[StateOperatorSymbol::CommittedOperatorKey]
    );
}

#[test]
fn checked_substitutions_change_only_the_consumer_site() {
    let target = reviewed_target();
    let first = state_operator_fragment(&bindings()).unwrap();
    let mut other = values();
    other.insert(
        StateOperatorSymbol::CommittedOperatorKey,
        StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0x44; 32]).unwrap(),
    );
    let bindings = StateOperatorBindings::new(&target, &other).unwrap();
    let second = state_operator_fragment(&bindings).unwrap();
    assert_ne!(first.instructions()[0], second.instructions()[0]);
    assert_eq!(first.instructions()[1..], second.instructions()[1..]);
    assert_eq!(
        build_state_operator_pattern(&target, &bindings, first),
        Err(StateOperatorRefusal::FragmentMismatch)
    );
}

#[test]
fn witness_selected_key_is_walkable_but_cannot_inherit_identity() {
    let target = reviewed_target();
    let fragment = TapscriptProgram::new(vec![op(OpcodeId::CheckSigVerify)]).unwrap();
    let witness = AbstractStackState::from_main(vec![
        signature(),
        StackValueType::Encoded(EncodingClass::XOnlyPublicKey),
    ]);
    let result = validate_program(
        &target,
        &fragment,
        &witness,
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert_eq!(
        result.success(),
        &BTreeSet::from([AbstractStackState::from_main(vec![])])
    );
    assert_eq!(
        build_state_operator_pattern(&target, &bindings(), fragment),
        Err(StateOperatorRefusal::FragmentMismatch)
    );
}

#[test]
fn second_signature_item_leaves_residue_in_the_independent_walk() {
    let target = reviewed_target();
    let pattern = pattern();
    let result = validate_program(
        &target,
        pattern.fragment(),
        &AbstractStackState::from_main(vec![signature(), signature()]),
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert_eq!(
        result.success(),
        &BTreeSet::from([AbstractStackState::from_main(vec![signature()])])
    );
    assert_ne!(result.success(), pattern.execution().success());
}

#[test]
fn pushing_signature_form_leaves_a_boolean_and_is_refused() {
    let target = reviewed_target();
    let pattern = pattern();
    let fragment = TapscriptProgram::new(vec![
        pattern.fragment().instructions()[0].clone(),
        op(OpcodeId::CheckSig),
    ])
    .unwrap();
    let result = validate_program(
        &target,
        &fragment,
        pattern.precondition(),
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert!(result.success().iter().all(|state| state.depth() == 1));
    assert_ne!(result.success(), pattern.execution().success());
    assert_eq!(
        build_state_operator_pattern(&target, &bindings(), fragment),
        Err(StateOperatorRefusal::FragmentMismatch)
    );
}

#[test]
fn unknown_key_type_has_unverified_success_but_never_an_admitted_consumer() {
    let target = reviewed_target();
    let unknown = StackItem::new(&target, vec![0x55; 33]).unwrap();
    let fragment = TapscriptProgram::new(vec![
        TapscriptInstruction::Push(unknown.clone()),
        op(OpcodeId::CheckSigVerify),
    ])
    .unwrap();
    let result = validate_program(
        &target,
        &fragment,
        pattern().precondition(),
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert!(result.reaches_unverified_signature_success());
    assert_eq!(
        StateOperatorBindings::new(
            &target,
            &BTreeMap::from([(StateOperatorSymbol::CommittedOperatorKey, unknown)])
        ),
        Err(StateOperatorRefusal::KeyEncoding)
    );
    assert_eq!(
        build_state_operator_pattern(&target, &bindings(), fragment),
        Err(StateOperatorRefusal::FragmentMismatch)
    );
}

#[test]
fn empty_and_malformed_keys_are_refused_before_emission() {
    let target = reviewed_target();
    for width in [0, 1, 31, 33, 64, 65] {
        let item = StackItem::new(&target, vec![0x33; width]).unwrap();
        assert_eq!(
            StateOperatorBindings::new(
                &target,
                &BTreeMap::from([(StateOperatorSymbol::CommittedOperatorKey, item)])
            ),
            Err(StateOperatorRefusal::KeyEncoding)
        );
    }
}

#[test]
fn a_missing_signature_cannot_schedule_the_fragment() {
    let target = reviewed_target();
    assert!(
        validate_program(
            &target,
            pattern().fragment(),
            &AbstractStackState::from_main(vec![]),
            AbstractLimits::for_target(&target)
        )
        .is_err()
    );
}

#[test]
fn resource_projection_prices_one_key_push_and_one_verifying_primitive() {
    let target = reviewed_target();
    let pattern = pattern();
    let cost = target.definition().opcodes()[&OpcodeId::CheckSigVerify].resources();
    assert_eq!(
        pattern.resources(),
        &BTreeMap::from([
            (ResourceDimension::ScriptBytes, 34),
            (ResourceDimension::OperationCost, cost.operation_cost()),
            (
                ResourceDimension::ValidationBudget,
                cost.validation_budget()
            )
        ])
    );
    assert_eq!(
        pattern.resources(),
        &resource_projection(&target, pattern.fragment())
    );
}

#[test]
fn metadata_keeps_deployment_native_and_observation_evidence_outstanding() {
    let target = reviewed_target();
    let pattern = pattern();
    assert_eq!(
        pattern.sources(),
        &BTreeSet::from([
            RequiredSourceKind::OperatorWitness,
            RequiredSourceKind::PublicConstructionData
        ])
    );
    assert_eq!(
        pattern.prerequisites(),
        &crate::pattern::fragment_prerequisites(pattern.fragment())
    );
    let mut expected = BTreeSet::from([
        TargetEvidenceRequirementId::TapscriptExecutionDomain,
        TargetEvidenceRequirementId::LeafVersionActivation,
        TargetEvidenceRequirementId::PushEncodingSemantics,
        TargetEvidenceRequirementId::EncodingSemantics,
        TargetEvidenceRequirementId::ConsensusResourceLimits,
    ]);
    expected.extend(target.definition().opcodes()[&OpcodeId::CheckSigVerify].evidence());
    assert_eq!(pattern.evidence(), &expected);
    assert_eq!(
        pattern.disclosure(),
        &[
            StateOperatorDisclosure::OperatorPublicKey,
            StateOperatorDisclosure::OperatorSignature
        ]
    );
    assert_eq!(
        pattern.residuals(),
        &[
            StateOperatorResidual::UnresolvedConsumer,
            StateOperatorResidual::TargetNativeEvidence,
            StateOperatorResidual::OperatorCurveValidity,
            StateOperatorResidual::DeploymentMembership,
            StateOperatorResidual::FinalizedObservationBinding
        ]
    );
}

#[test]
fn signature_widths_outside_the_target_encoding_cannot_succeed() {
    let record = pattern();
    let target = reviewed_target();
    let target_elements::PayloadWidth::Exact(width) =
        target.definition().encodings()[&EncodingClass::SchnorrSignature].payload()
    else {
        panic!("fixed signature encoding")
    };
    for width in [width.get() - 1, width.get() + 1] {
        super::state_program_tests::assert_witness_changes_success(
            record.fragment(),
            vec![StackValueType::Bytes {
                minimum: width,
                maximum: width,
            }],
            record.execution().success(),
        );
    }
}

#[test]
fn operator_refusal_declaration_has_exact_exercised_and_unreachable_census() {
    super::state_program_tests::assert_refusal_census(
        include_str!("../state_operator.rs"),
        "StateOperatorRefusal",
        &[
            (
                "ConsumerCensus",
                consumer_census_names_the_committed_program_push_only,
            ),
            (
                "KeyEncoding",
                empty_and_malformed_keys_are_refused_before_emission,
            ),
            (
                "FragmentMismatch",
                witness_selected_key_is_walkable_but_cannot_inherit_identity,
            ),
        ],
        &[
            (
                "InvalidContract",
                "Checked key and exact recipe make the malformed contract inaccessible; needs a contract checker seam.",
            ),
            (
                "Program",
                "The two fixed instructions and checked key do not supply a failing walk; needs a walk failure seam.",
            ),
        ],
    );
}
