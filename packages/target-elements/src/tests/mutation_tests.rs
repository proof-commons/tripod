//! Mutation tests for the target-definition validator.
//!
//! Each test starts from the reviewed contract, damages exactly one
//! thing, and requires the validator to reject for the focused typed
//! reason. A validator that rejected everything for a generic reason
//! would pass a positive test and be useless to a reviewer.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use crate::definition::{
    TargetContractVersion, TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::encoding::ByteOrder;
use crate::error::TargetError;
use crate::opcode::{
    FailureCause, FailureContract, FailureEffect, FailureOutcome, LeafVersion, OpcodeId,
    OpcodeResourceCost, OpcodeSpec, StackContract, StackValueType,
};
use crate::success::{SuccessCase, SuccessCondition, SuccessContract, SuccessStackEffect};

/// The reviewed registry, as a mutable starting point.
fn registry() -> BTreeMap<OpcodeId, OpcodeSpec> {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .opcodes()
        .clone()
}

/// The reviewed contract's parts, as a mutable starting point.
///
/// Every mutation test damages exactly one field of this and leaves
/// the rest reviewed, so a rejection can only be attributed to the
/// damage.
fn parts() -> TargetDefinitionParts {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }
}

/// Assembles a contract from a possibly damaged registry.
fn definition(opcodes: BTreeMap<OpcodeId, OpcodeSpec>) -> TargetDefinition {
    TargetDefinition::new(TargetDefinitionParts { opcodes, ..parts() })
}

/// Runs the validator and requires it to reject.
fn reject(definition: TargetDefinition) -> Vec<TargetError> {
    validate_target_definition(definition).expect_err("the mutation must be rejected")
}

/// Rebuilds one specification with a replacement stack contract.
fn with_stack(spec: &OpcodeSpec, stack: StackContract) -> OpcodeSpec {
    OpcodeSpec::new(
        spec.id(),
        spec.code(),
        spec.domains().iter().copied(),
        stack,
        spec.resources(),
        spec.evidence().iter().copied(),
    )
}

#[test]
fn the_undamaged_contract_is_accepted() {
    // The control. Without it every mutation test below could be
    // passing for the wrong reason.
    assert!(validate_target_definition(definition(registry())).is_ok());
}

#[test]
fn a_duplicated_opcode_byte_is_rejected() {
    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Add64].clone();
    // Give subtraction addition's byte. One of the two would be
    // unreachable, and the contract does not say which.
    opcodes.insert(
        OpcodeId::Sub64,
        OpcodeSpec::new(
            OpcodeId::Sub64,
            victim.code(),
            victim.domains().iter().copied(),
            victim.stack().clone(),
            victim.resources(),
            victim.evidence().iter().copied(),
        ),
    );

    assert!(
        reject(definition(opcodes)).contains(&TargetError::DuplicateOpcodeCode(0xd7)),
        "the collision must name the contested byte"
    );
}

#[test]
fn an_entry_filed_under_the_wrong_identity_is_rejected() {
    let mut opcodes = registry();
    let addition = opcodes[&OpcodeId::Add64].clone();
    // File addition's contract under subtraction's key.
    opcodes.insert(
        OpcodeId::Sub64,
        OpcodeSpec::new(
            OpcodeId::Add64,
            0xff,
            addition.domains().iter().copied(),
            addition.stack().clone(),
            addition.resources(),
            addition.evidence().iter().copied(),
        ),
    );

    assert!(
        reject(definition(opcodes)).contains(&TargetError::OpcodeIdMismatch {
            key: OpcodeId::Sub64,
            declared: OpcodeId::Add64,
        })
    );
}

#[test]
fn a_missing_reviewed_primitive_is_rejected() {
    let mut opcodes = registry();
    opcodes.remove(&OpcodeId::TweakVerify);

    assert!(
        reject(definition(opcodes))
            .contains(&TargetError::MissingOpcodeContract(OpcodeId::TweakVerify))
    );
}

#[test]
fn an_absent_failure_contract_is_rejected() {
    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Add64].clone();
    let stack = StackContract::new(
        victim.stack().operands().to_vec(),
        victim.stack().success().clone(),
        FailureContract::new([]),
    );
    opcodes.insert(OpcodeId::Add64, with_stack(&victim, stack));

    assert!(
        reject(definition(opcodes))
            .contains(&TargetError::MissingOpcodeFailureContract(OpcodeId::Add64))
    );
}

#[test]
fn one_failure_cause_with_two_effects_is_rejected() {
    // A contract that says overflow both aborts and pushes a false
    // does not say what the target does.
    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Add64].clone();
    let stack = StackContract::new(
        victim.stack().operands().to_vec(),
        victim.stack().success().clone(),
        FailureContract::new([
            FailureEffect::new(
                FailureCause::ArithmeticOverflow,
                FailureOutcome::AbortEvaluation,
            ),
            FailureEffect::new(
                FailureCause::ArithmeticOverflow,
                FailureOutcome::RetainOperandsPushFalse,
            ),
        ]),
    );
    opcodes.insert(OpcodeId::Add64, with_stack(&victim, stack));

    assert!(
        reject(definition(opcodes)).contains(&TargetError::ContradictoryFailureCause {
            opcode: OpcodeId::Add64,
            cause: FailureCause::ArithmeticOverflow,
        })
    );
}

#[test]
fn a_malformed_operand_width_is_rejected() {
    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Sha256Initialize].clone();
    let stack = StackContract::new(
        vec![StackValueType::Bytes {
            minimum: 64,
            maximum: 32,
        }],
        victim.stack().success().clone(),
        victim.stack().failure().clone(),
    );
    opcodes.insert(OpcodeId::Sha256Initialize, with_stack(&victim, stack));

    assert!(
        reject(definition(opcodes)).contains(&TargetError::InvalidOpcodeStackContract(
            OpcodeId::Sha256Initialize
        ))
    );
}

#[test]
fn a_primitive_declaring_no_execution_domain_is_rejected() {
    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Add64].clone();
    opcodes.insert(
        OpcodeId::Add64,
        OpcodeSpec::new(
            victim.id(),
            victim.code(),
            [],
            victim.stack().clone(),
            victim.resources(),
            victim.evidence().iter().copied(),
        ),
    );

    assert!(
        reject(definition(opcodes)).contains(&TargetError::UnsupportedOpcodeExecutionDomain(
            OpcodeId::Add64
        ))
    );
}

#[test]
fn a_primitive_with_no_script_byte_cost_is_rejected() {
    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Add64].clone();
    opcodes.insert(
        OpcodeId::Add64,
        OpcodeSpec::new(
            victim.id(),
            victim.code(),
            victim.domains().iter().copied(),
            victim.stack().clone(),
            OpcodeResourceCost::new(0, 0, 0, 1, 0),
            victim.evidence().iter().copied(),
        ),
    );

    assert!(
        reject(definition(opcodes))
            .contains(&TargetError::MissingOpcodeResourceCost(OpcodeId::Add64))
    );
}

#[test]
fn a_primitive_naming_no_evidence_is_rejected() {
    // A capability whose claim nothing is ever asked to demonstrate
    // rests on this crate's assertion alone.
    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Add64].clone();
    opcodes.insert(
        OpcodeId::Add64,
        OpcodeSpec::new(
            victim.id(),
            victim.code(),
            victim.domains().iter().copied(),
            victim.stack().clone(),
            victim.resources(),
            [],
        ),
    );

    assert!(
        reject(definition(opcodes)).contains(&TargetError::MissingOpcodeEvidence(OpcodeId::Add64))
    );
}

#[test]
fn the_version_and_leaf_mutations_are_refused_before_assembly() {
    // These two mutations cannot reach the validator, because neither
    // type has a public unchecked constructor. The refusal happens
    // earlier, which is strictly stronger than a validation
    // diagnostic: a contract carrying an unsupported revision or an
    // unreviewed leaf simply cannot be built.
    assert_eq!(
        TargetContractVersion::supported(2),
        Err(TargetError::UnsupportedTargetContractVersion { offered: 2 })
    );
    assert_eq!(
        LeafVersion::new(0xc0),
        Err(TargetError::UnreviewedLeafVersion { offered: 0xc0 })
    );
}

#[test]
fn the_validator_reports_every_defect_rather_than_the_first() {
    let mut opcodes = registry();
    opcodes.remove(&OpcodeId::TweakVerify);
    opcodes.remove(&OpcodeId::EcMulScalarVerify);

    let victim = opcodes[&OpcodeId::Neg64].clone();
    opcodes.insert(
        OpcodeId::Neg64,
        OpcodeSpec::new(
            victim.id(),
            victim.code(),
            victim.domains().iter().copied(),
            victim.stack().clone(),
            victim.resources(),
            [],
        ),
    );

    let errors = reject(definition(opcodes));
    assert!(errors.contains(&TargetError::MissingOpcodeContract(OpcodeId::TweakVerify)));
    assert!(errors.contains(&TargetError::MissingOpcodeContract(
        OpcodeId::EcMulScalarVerify
    )));
    assert!(errors.contains(&TargetError::MissingOpcodeEvidence(OpcodeId::Neg64)));
    assert!(
        errors.len() >= 3,
        "fixing three defects one build at a time wastes a reviewer's attention"
    );
}

#[test]
fn a_changed_opcode_byte_changes_the_projection() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");

    let mut opcodes = registry();
    let victim = opcodes[&OpcodeId::Add64].clone();
    opcodes.insert(
        OpcodeId::Add64,
        OpcodeSpec::new(
            victim.id(),
            0xf0,
            victim.domains().iter().copied(),
            victim.stack().clone(),
            victim.resources(),
            victim.evidence().iter().copied(),
        ),
    );

    let mutated =
        validate_target_definition(definition(opcodes)).expect("a free byte still validates");
    assert_ne!(reviewed.projection(), mutated.projection());
}

#[test]
fn an_unsigned_operand_is_not_a_signed_one() {
    // The widening conversion reads its operand unsigned. Treating the
    // two fixed-width shapes as interchangeable would silently admit
    // negative inputs the target rejects.
    let bytes = NonZeroUsize::new(4).expect("4 is not zero");
    let unsigned = StackValueType::UnsignedFixedWidth {
        bytes,
        byte_order: ByteOrder::LittleEndian,
    };
    let signed = StackValueType::SignedFixedWidth {
        bytes,
        byte_order: ByteOrder::LittleEndian,
    };
    assert_ne!(unsigned, signed);

    let widening = registry()[&OpcodeId::Le32ToLe64].clone();
    assert_eq!(widening.stack().operands(), &[unsigned]);
}

/// Replaces one primitive's successful behavior and rejects.
fn reject_success(id: OpcodeId, success: SuccessContract) -> Vec<TargetError> {
    let mut opcodes = registry();
    let victim = opcodes[&id].clone();
    let stack = StackContract::new(
        victim.stack().operands().to_vec(),
        success,
        victim.stack().failure().clone(),
    );
    opcodes.insert(id, with_stack(&victim, stack));
    reject(definition(opcodes))
}

#[test]
fn a_success_contract_offering_no_form_is_rejected() {
    // Alternatives with no cases states that the primitive succeeds
    // and refuses to say what that does to the stack.
    assert!(
        reject_success(
            OpcodeId::InspectOutputNonce,
            SuccessContract::Alternatives { cases: Vec::new() }
        )
        .contains(&TargetError::IncompleteSuccessContract(
            OpcodeId::InspectOutputNonce
        ))
    );
}

#[test]
fn two_forms_under_one_condition_are_rejected() {
    // Two answers to the same question is not a contract.
    let duplicated = SuccessContract::Alternatives {
        cases: vec![
            SuccessCase::new(
                SuccessCondition::ExplicitEncoding,
                SuccessStackEffect::new(1, vec![StackValueType::ScriptNumber]),
            ),
            SuccessCase::new(
                SuccessCondition::ExplicitEncoding,
                SuccessStackEffect::new(1, vec![StackValueType::Bool]),
            ),
        ],
    };
    assert!(
        reject_success(OpcodeId::InspectOutputNonce, duplicated).contains(
            &TargetError::DuplicateSuccessCase {
                opcode: OpcodeId::InspectOutputNonce,
                case: SuccessCondition::ExplicitEncoding,
            }
        )
    );
}

#[test]
fn a_form_consuming_undeclared_operands_is_rejected() {
    // The nonce primitive declares one operand. A form claiming to
    // consume three describes a depth no program could reach.
    assert!(
        reject_success(
            OpcodeId::InspectOutputNonce,
            SuccessContract::Fixed {
                consumed_operands: 3,
                results: vec![StackValueType::Bool],
            }
        )
        .contains(&TargetError::ContradictorySuccessCase(
            OpcodeId::InspectOutputNonce
        ))
    );
}

#[test]
fn retaining_operands_that_do_not_exist_is_rejected() {
    // The current-index primitive takes no operands, so there is
    // nothing for it to retain and the claim is empty.
    assert!(
        reject_success(
            OpcodeId::PushCurrentInputIndex,
            SuccessContract::RetainsOperands {
                results: vec![StackValueType::ScriptNumber],
            }
        )
        .contains(&TargetError::InvalidRetainedOperandContract(
            OpcodeId::PushCurrentInputIndex
        ))
    );
}
