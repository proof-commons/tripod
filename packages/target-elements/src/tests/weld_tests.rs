//! Mutation tests for the cross-subcontract welds.
//!
//! Each mutation below leaves every individual view well-formed and
//! makes two views disagree. That is the exact class of defect the
//! local shape checks cannot see, and the reason the contract is
//! allowed to state some facts twice.

use std::collections::BTreeMap;

use crate::authorization::{
    AuthorizationContract, RelativeTimelockContract, SignaturePrimitiveContract,
};
use crate::capability::{CapabilityContract, ElementsCapability, StaticCapabilityStatus};
use crate::confidential::{
    ConfidentialCapabilityState, ConfidentialValueCapability, ConfidentialValueContract,
    IssuanceContract, IssuanceField,
};
use crate::definition::{
    TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::encoding::EncodingClass;
use crate::error::TargetError;
use crate::opcode::{
    FailureCause, FailureContract, FailureEffect, FailureOutcome, OpcodeId, OpcodeResourceCost,
    OpcodeSpec, StackContract,
};

/// The reviewed contract's parts, as a mutable starting point.
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

/// Runs the validator and requires it to reject.
fn reject(parts: TargetDefinitionParts) -> Vec<TargetError> {
    validate_target_definition(TargetDefinition::new(parts))
        .expect_err("the mutation must be rejected")
}

/// Restates one primitive's failure effect for one cause.
fn set_failure(
    opcodes: &mut BTreeMap<OpcodeId, OpcodeSpec>,
    opcode: OpcodeId,
    cause: FailureCause,
    outcome: FailureOutcome,
) {
    let source = opcodes[&opcode].clone();
    let stack = source.stack();
    let mut effects: Vec<FailureEffect> = stack
        .failure()
        .effects()
        .iter()
        .filter(|effect| effect.cause() != cause)
        .copied()
        .collect();
    effects.push(FailureEffect::new(cause, outcome));
    opcodes.insert(
        opcode,
        OpcodeSpec::new(
            source.id(),
            source.code(),
            source.domains().iter().copied(),
            StackContract::new(
                stack.operands().to_vec(),
                stack.success().clone(),
                FailureContract::new(effects),
            ),
            source.resources(),
            source.evidence().iter().copied(),
        ),
    );
}

/// Restates the signature primitive contract, leaving the rest of the
/// authorization contract alone.
fn with_signature(parts: &mut TargetDefinitionParts, signature: SignaturePrimitiveContract) {
    parts.authorization = AuthorizationContract::new(
        signature,
        parts.authorization.sighash().clone(),
        parts.authorization.relative_timelock().clone(),
    );
}

#[test]
fn the_reviewed_contract_is_welded() {
    // The control. Every weld below can only be read as "these two
    // views disagree" if the reviewed contract's views agree.
    assert!(validate_target_definition(TargetDefinition::new(parts())).is_ok());
}

#[test]
fn a_signature_opcode_disagreeing_about_an_empty_signature_is_rejected() {
    // `CheckSig` says an empty signature aborts while the signature
    // primitive contract says it consumes the operands and pushes a
    // false. Each view is well-formed; together they describe two
    // different primitives.
    let mut parts = parts();
    set_failure(
        &mut parts.opcodes,
        OpcodeId::CheckSig,
        FailureCause::EmptySignature,
        FailureOutcome::AbortEvaluation,
    );

    assert!(reject(parts).contains(&TargetError::SignatureContractMismatch));
}

#[test]
fn a_verifying_signature_opcode_that_pushes_a_false_is_rejected() {
    // The other direction: the verifying form must not acquire a
    // branchable result, whatever the contract's field says.
    let mut parts = parts();
    set_failure(
        &mut parts.opcodes,
        OpcodeId::CheckSigVerify,
        FailureCause::EmptySignature,
        FailureOutcome::ConsumeOperandsPushFalse,
    );

    assert!(reject(parts).contains(&TargetError::SignatureContractMismatch));
}

#[test]
fn a_per_check_budget_disagreeing_with_the_opcode_cost_is_rejected() {
    // Fifty in the resource row, sixty in the contract. A planner
    // reading one and a validator reading the other would disagree
    // about how many checks fit in a script.
    let mut parts = parts();
    let signature = parts.authorization.signature();
    let restated = SignaturePrimitiveContract::new(
        signature.public_key_encoding(),
        signature.signature_encoding(),
        signature.empty_signature(),
        signature.invalid_signature(),
        signature.unknown_public_key_type(),
        60,
        signature.evidence().iter().copied(),
    );
    with_signature(&mut parts, restated);

    let errors = reject(parts);
    assert!(errors.contains(&TargetError::SignatureContractMismatch));
    assert!(errors.contains(&TargetError::ResourceContractMismatch));
}

#[test]
fn a_signature_contract_naming_an_operand_encoding_no_opcode_takes_is_rejected() {
    let mut parts = parts();
    let signature = parts.authorization.signature();
    let restated = SignaturePrimitiveContract::new(
        // The signature primitives take an x-only key, not a
        // compressed one; the curve primitives take the compressed
        // form. Swapping them keeps both encodings declared and makes
        // the contract describe operands no signature opcode has.
        EncodingClass::CompressedPublicKey,
        signature.signature_encoding(),
        signature.empty_signature(),
        signature.invalid_signature(),
        signature.unknown_public_key_type(),
        signature.budget_per_check(),
        signature.evidence().iter().copied(),
    );
    with_signature(&mut parts, restated);

    assert!(reject(parts).contains(&TargetError::SignatureContractMismatch));
}

#[test]
fn a_timelock_contract_disagreeing_about_an_unsatisfied_lock_is_rejected() {
    // The opcode aborts; the contract claims it retains its operands
    // and pushes a false. A program branching on the second would
    // never run.
    let mut parts = parts();
    let timelock = parts.authorization.relative_timelock();
    let restated = RelativeTimelockContract::new(
        timelock.modes().iter().copied(),
        timelock.layout(),
        timelock.minimum_transaction_version(),
        FailureOutcome::RetainOperandsPushFalse,
        timelock.evidence().iter().copied(),
    );
    parts.authorization = AuthorizationContract::new(
        parts.authorization.signature().clone(),
        parts.authorization.sighash().clone(),
        restated,
    );

    assert!(reject(parts).contains(&TargetError::TimelockContractMismatch));
}

#[test]
fn a_timelock_without_a_version_prerequisite_is_rejected() {
    // Below the stated version the lock is not enforced at all, so a
    // capability that cannot constrain the version cannot rely on it.
    let mut parts = parts();
    let source = parts.capabilities[&ElementsCapability::RelativeTimelock].clone();
    parts.capabilities.insert(
        ElementsCapability::RelativeTimelock,
        CapabilityContract::new(
            source.capability(),
            [ElementsCapability::TapscriptExecution],
            source.opcodes().iter().copied(),
            source.encodings().iter().copied(),
            source.evidence().iter().copied(),
            source.status(),
        ),
    );

    assert!(reject(parts).contains(&TargetError::TimelockContractMismatch));
}

#[test]
fn an_issuance_contract_omitting_a_field_is_rejected() {
    // A partial census describes a different issuance than the one the
    // introspection primitive pushes.
    let mut parts = parts();
    parts.issuance = IssuanceContract::new(
        IssuanceField::ALL
            .iter()
            .copied()
            .filter(|field| *field != IssuanceField::BlindingNonce),
        parts.issuance.introspection(),
        parts.issuance.absent_marker(),
        parts.issuance.outpoint_flag_reports_issuance(),
        parts.issuance.evidence().iter().copied(),
    );

    assert!(reject(parts).contains(&TargetError::IssuanceContractMismatch));
}

#[test]
fn an_issuance_marker_the_absent_form_does_not_push_is_rejected() {
    // The contract says an absent issuance is marked by the null nonce
    // while the primitive pushes the null value. Both encodings exist;
    // a decoder told the wrong one reads the wrong field.
    let mut parts = parts();
    parts.issuance = IssuanceContract::new(
        parts.issuance.fields().iter().copied(),
        parts.issuance.introspection(),
        EncodingClass::NullNonce,
        parts.issuance.outpoint_flag_reports_issuance(),
        parts.issuance.evidence().iter().copied(),
    );

    assert!(reject(parts).contains(&TargetError::IssuanceContractMismatch));
}

#[test]
fn a_confidential_claim_disagreeing_with_its_capability_row_is_rejected() {
    // The confidential-value contract says a reviewed primitive
    // establishes commitment equality; the capability row says nothing
    // does. One of them is wrong and the contract does not say which.
    let mut parts = parts();
    let source = &parts.confidential_values;
    let mut states: BTreeMap<_, _> = source.states().clone();
    states.insert(
        ConfidentialValueCapability::CommitmentEquality,
        ConfidentialCapabilityState::PrimitiveReviewed,
    );
    parts.confidential_values = ConfidentialValueContract::new(
        states,
        source.participating_encodings().iter().copied(),
        source.evidence().iter().copied(),
    );

    assert!(reject(parts).contains(&TargetError::ConfidentialContractMismatch));
}

#[test]
fn a_conservation_claim_omitting_a_value_encoding_is_rejected() {
    // The value-inspection primitives can push a blinded payload. A
    // conservation claim that does not cover that class is silent
    // about exactly the transactions it exists for.
    let mut parts = parts();
    let source = &parts.confidential_values;
    parts.confidential_values = ConfidentialValueContract::new(
        source.states().clone(),
        source
            .participating_encodings()
            .iter()
            .copied()
            .filter(|class| *class != EncodingClass::ConfidentialValue),
        source.evidence().iter().copied(),
    );

    assert!(reject(parts).contains(&TargetError::ConfidentialContractMismatch));
}

#[test]
fn an_opcode_charging_an_unrecognized_budget_is_rejected() {
    // A primitive charges nothing or it charges the one per-check
    // figure. A third number is a budget no bound is expressed in.
    let mut parts = parts();
    let source = parts.opcodes[&OpcodeId::TweakVerify].clone();
    let cost = source.resources();
    parts.opcodes.insert(
        OpcodeId::TweakVerify,
        OpcodeSpec::new(
            source.id(),
            source.code(),
            source.domains().iter().copied(),
            source.stack().clone(),
            OpcodeResourceCost::new(
                cost.script_bytes(),
                cost.operation_cost(),
                75,
                cost.maximum_stack_growth(),
                cost.maximum_altstack_growth(),
            ),
            source.evidence().iter().copied(),
        ),
    );

    assert!(reject(parts).contains(&TargetError::ResourceContractMismatch));
}

#[test]
fn a_subcontract_naming_no_evidence_is_rejected() {
    // Every subcontract's claim is something a deployment must be
    // asked to demonstrate. One naming nothing rests on this crate's
    // assertion alone.
    let mut parts = parts();
    parts.issuance = IssuanceContract::new(
        parts.issuance.fields().iter().copied(),
        parts.issuance.introspection(),
        parts.issuance.absent_marker(),
        parts.issuance.outpoint_flag_reports_issuance(),
        [],
    );

    let errors = reject(parts);
    assert!(errors.contains(&TargetError::EvidenceContractMismatch));
    assert!(errors.contains(&TargetError::IssuanceContractMismatch));
}

#[test]
fn a_capability_status_change_alone_does_not_trip_an_unrelated_weld() {
    // Focus check: a weld must fire for its own subject and stay quiet
    // otherwise, or a reviewer cannot tell what to repair.
    let mut parts = parts();
    let source = parts.capabilities[&ElementsCapability::PolicyResourceLimits].clone();
    parts.capabilities.insert(
        ElementsCapability::PolicyResourceLimits,
        CapabilityContract::new(
            source.capability(),
            source.prerequisites().iter().copied(),
            source.opcodes().iter().copied(),
            source.encodings().iter().copied(),
            source.evidence().iter().copied(),
            StaticCapabilityStatus::Incomplete,
        ),
    );

    assert!(validate_target_definition(TargetDefinition::new(parts)).is_ok());
}
