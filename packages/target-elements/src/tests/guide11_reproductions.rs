//! Guide-11 Wave-0 reproductions of the sixth static review.
//!
//! # These tests assert the defect, not the repair
//!
//! Each test demonstrates a finding from the Guide-11 preflight register
//! by *passing* while the defect is present: it asserts that the wrong
//! thing happens. Waves 1 to 4 flip each assertion as they repair the
//! finding, so a test here failing after a repair is the repair working
//! rather than a regression.
//!
//! Two findings are reproduced here: `G11-R09`, that the advertised V1
//! contract revision is validated against the V2 census and algebra, and
//! `G11-R10`, that the signature weld ignores the unknown-public-key
//! rule. No production code path is touched.

use std::collections::BTreeMap;

use crate::authorization::{AuthorizationContract, SignaturePrimitiveContract};
use crate::definition::{
    TargetContractVersion, TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::error::TargetError;
use crate::opcode::{OpcodeId, OpcodeSpec};
use crate::success::SuccessCondition;

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
        pushes: source.pushes().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }
}

// -- G11-R09 -----------------------------------------------------------

/// `G11-R09`: a complete V2 body stamped V1 is generically validated.
///
/// `validate_target_definition` is not version-dispatched: it applies
/// `OpcodeId::ALL` and the current capability, evidence, and encoding
/// censuses whatever revision the definition claims. The revision field
/// therefore does not identify the contract shape it names.
#[test]
fn g11_r09_a_v2_body_stamped_v1_validates() {
    assert!(
        TargetContractVersion::SUPPORTED.contains(&TargetContractVersion::V1),
        "V1 is advertised as supported",
    );

    let mut parts = parts();
    assert_eq!(parts.version, TargetContractVersion::V2);
    parts.version = TargetContractVersion::V1;
    let validated = validate_target_definition(TargetDefinition::new(parts))
        .expect("the defect: a V2 census stamped V1 is accepted as a validated contract");
    assert_eq!(
        validated.definition().version(),
        TargetContractVersion::V1,
        "the defect: the validated contract says V1 while its body is V2",
    );
    assert_eq!(
        validated.definition().opcodes().len(),
        OpcodeId::ALL.len(),
        "the body carries the complete V2 primitive census",
    );
}

/// `G11-R09`: a V1 definition lacking a V2-only primitive is refused for
/// the V2 census it was never meant to carry.
///
/// The other half of the same defect: because the validator applies one
/// global census, a genuine historical V1 contract cannot pass it.
#[test]
fn g11_r09_a_v1_definition_is_refused_for_missing_v2_primitives() {
    let mut parts = parts();
    parts.version = TargetContractVersion::V1;
    // One of the compound-proof primitives the second revision added.
    let removed = OpcodeId::Concatenate;
    let mut opcodes: BTreeMap<OpcodeId, OpcodeSpec> = parts.opcodes.clone();
    opcodes.remove(&removed);
    parts.opcodes = opcodes;

    let errors = validate_target_definition(TargetDefinition::new(parts))
        .expect_err("a definition missing a primitive is refused");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error, TargetError::MissingOpcodeContract(id) if *id == removed)),
        "the defect: a V1 contract is refused for lacking a V2 primitive, got {errors:?}",
    );
}

// -- G11-R10 -----------------------------------------------------------

/// `G11-R10`: the signature weld ignores the unknown-public-key rule.
///
/// The subcontract is changed to say an unknown key type is rejected,
/// while every signature opcode keeps its unknown-key success case and
/// its unknown-nonempty-key operand admission. `weld_signature` reads
/// neither, so the contradictory definition still validates.
#[test]
fn g11_r10_a_contradictory_unknown_key_rule_still_validates() {
    let mut parts = parts();
    let authorization = parts.authorization.clone();
    let signature = authorization.signature();
    assert_eq!(
        signature.unknown_public_key_type(),
        crate::authorization::UnknownPublicKeyTypeRule::SucceedsWithoutVerification,
        "the reviewed rule is the succeeding one",
    );

    // Every signature opcode still advertises the unknown-key success
    // that the changed rule denies.
    let unknown_key_successes = [
        OpcodeId::CheckSig,
        OpcodeId::CheckSigVerify,
        OpcodeId::CheckSigFromStack,
        OpcodeId::CheckSigFromStackVerify,
    ]
    .into_iter()
    .filter(|opcode| {
        parts.opcodes.get(opcode).is_some_and(|spec| {
            spec.stack()
                .success()
                .cases()
                .iter()
                .any(|case| case.condition() == SuccessCondition::UnknownKeyTypeUnverified)
        })
    })
    .count();
    assert!(
        unknown_key_successes > 0,
        "at least one signature opcode advertises unknown-key success",
    );

    parts.authorization = AuthorizationContract::new(
        SignaturePrimitiveContract::new(
            signature.public_key_encoding(),
            signature.signature_encoding(),
            signature.empty_signature(),
            signature.invalid_signature(),
            crate::authorization::UnknownPublicKeyTypeRule::Rejected,
            signature.budget_per_check(),
            signature.evidence().iter().copied(),
        ),
        authorization.sighash().clone(),
        authorization.relative_timelock().clone(),
    );

    let validated = validate_target_definition(TargetDefinition::new(parts))
        .expect("the defect: the contradictory signature contract validates");
    assert_eq!(
        validated
            .definition()
            .authorization()
            .signature()
            .unknown_public_key_type(),
        crate::authorization::UnknownPublicKeyTypeRule::Rejected,
        "the defect: the validated contract rejects unknown keys in one view and \
         succeeds on them in another",
    );
}
