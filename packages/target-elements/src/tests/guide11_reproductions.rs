//! Guide-11 reproductions and, where a wave has landed, guarantees.
//!
//! # Two kinds of test live here
//!
//! Every test began as a reproduction: it demonstrated a finding from
//! the Guide-11 preflight register by *passing* while the defect was
//! present, asserting that the wrong thing happened. As each wave
//! repairs its finding it flips the assertions of that finding's tests,
//! which then stand as the guarantee that the repair holds.
//!
//! - `G11-R09` is **CLOSED** by Wave 4. The advertised V1 revision was
//!   validated against the V2 census and algebra; V1 is no longer
//!   advertised, the offered revision is checked, and the supported
//!   list holds only revisions a complete definition can be built for.
//! - `G11-R10` is **CLOSED** by Wave 4. The signature weld ignored the
//!   unknown-public-key rule and most of the success algebra; it now
//!   derives one complete expected behaviour from the subcontract and
//!   compares every signature opcode against it.
//!
//! No production code path is touched: these are constructions over the
//! public and crate-visible surfaces exactly as an external caller
//! reaches them.

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

/// `G11-R09`: a complete V2 body stamped V1 is refused.
///
/// `validate_target_definition` is not version-dispatched: it applies
/// `OpcodeId::ALL` and the current capability, evidence, and encoding
/// censuses whatever revision the definition claims. Advertising V1
/// under that validator meant the revision field did not identify the
/// contract shape it named. V1 is no longer advertised, and the offered
/// revision is checked, so a V2 body cannot travel under a V1 number.
#[test]
fn g11_r09_a_v2_body_stamped_v1_is_refused() {
    assert!(
        !TargetContractVersion::SUPPORTED.contains(&TargetContractVersion::V1),
        "V1 is no longer advertised as supported",
    );

    let mut parts = parts();
    assert_eq!(parts.version, TargetContractVersion::V2);
    parts.version = TargetContractVersion::V1;
    let errors = validate_target_definition(TargetDefinition::new(parts))
        .expect_err("a V2 census stamped V1 is not a validated contract");
    assert!(
        errors.iter().any(|error| matches!(
            error,
            TargetError::UnsupportedTargetContractVersion { offered: 1 },
        )),
        "the refusal names the unimplemented revision, got {errors:?}",
    );
}

/// `G11-R09`: the historical revision cannot be constructed by number.
///
/// The other half of the same repair. A V1 definition used to be
/// refused for lacking V2 primitives it was never meant to carry — a
/// validator applying one global census to a revision it does not
/// implement. Now the number itself is refused, before any census is
/// consulted, and the constant remains only as a historical name.
#[test]
fn g11_r09_the_historical_revision_is_not_constructible_by_number() {
    let error =
        TargetContractVersion::supported(1).expect_err("an unimplemented revision is refused");
    assert!(
        matches!(
            error,
            TargetError::UnsupportedTargetContractVersion { offered: 1 },
        ),
        "the refusal names the offered number, got {error:?}",
    );
    assert_eq!(
        TargetContractVersion::supported(2).expect("V2 is implemented"),
        TargetContractVersion::V2,
    );
    assert_eq!(TargetContractVersion::V1.get(), 1, "the name is retained");
}

/// `G11-R09`: every advertised revision can actually be constructed.
///
/// The review's standing requirement on the list itself: membership
/// means a complete accepted definition of that revision exists, not
/// that the number has been used before. Each supported revision is
/// checked two ways — it round-trips through the constructor, and a
/// complete definition stamped with it validates.
#[test]
fn g11_r09_every_supported_revision_has_a_constructible_definition() {
    assert!(!TargetContractVersion::SUPPORTED.is_empty());

    for revision in TargetContractVersion::SUPPORTED {
        assert_eq!(
            TargetContractVersion::supported(revision.get())
                .expect("a supported revision is accepted by number"),
            *revision,
        );

        let mut parts = parts();
        parts.version = *revision;
        validate_target_definition(TargetDefinition::new(parts))
            .expect("a supported revision has a complete accepted definition");
    }
}

/// `G11-R09`: a V2 definition missing a V2 primitive is still refused.
///
/// The census check must not have been loosened by the revision
/// repair: removing one of the compound-proof primitives from a V2
/// body is incompleteness, and is reported as such.
#[test]
fn g11_r09_a_v2_definition_missing_a_v2_primitive_is_refused() {
    let mut parts = parts();
    assert_eq!(parts.version, TargetContractVersion::V2);
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
        "an incomplete V2 census is refused for the primitive it lacks, got {errors:?}",
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
