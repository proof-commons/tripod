//! Public-API boundary test for the typed Elements target contract.
//!
//! This integration test compiles as an external consumer. It proves
//! that the crate is consumable from outside, that a consumer can only
//! obtain a *validated* contract, and that no surface has leaked out
//! ahead of the deliverable that owns it.

use std::collections::BTreeMap;

use target_elements::{
    ExecutionDomain, LeafVersion, OpcodeId, TargetContractVersion, TargetDefinition, TargetError,
    reviewed_elements_tapscript, validate_target_definition,
};

#[test]
fn the_contract_version_is_publicly_decidable() {
    let version = TargetContractVersion::supported(TargetContractVersion::V1.get())
        .expect("the declared revision is implemented");
    assert_eq!(version, TargetContractVersion::V1);

    assert_eq!(
        TargetContractVersion::supported(2),
        Err(TargetError::UnsupportedTargetContractVersion { offered: 2 })
    );
}

#[test]
fn the_error_root_is_public_and_displays() {
    let error = TargetError::UnsupportedTargetContractVersion { offered: 99 };
    assert_eq!(error.to_string(), "unsupported target contract version 99");

    // The error root implements the standard error trait without a
    // derive crate, so an external consumer can box it.
    let boxed: Box<dyn core::error::Error> = Box::new(error);
    assert!(boxed.to_string().contains("99"));
}

#[test]
fn an_external_consumer_receives_a_validated_contract() {
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(
        target.definition().execution_domain(),
        ExecutionDomain::Tapscript
    );
    assert_eq!(target.definition().leaf_version(), LeafVersion::TAPSCRIPT);
    assert_eq!(
        target.definition().opcodes().len(),
        OpcodeId::ALL.len(),
        "every reviewed identity has a contract"
    );
}

#[test]
fn an_external_consumer_cannot_skip_validation() {
    // The wrapper has no public constructor, so an empty registry
    // cannot be smuggled past the validator by building the wrapper
    // directly. The only route is the validator, which refuses.
    let empty = TargetDefinition::new(
        TargetContractVersion::V1,
        ExecutionDomain::Tapscript,
        LeafVersion::TAPSCRIPT,
        BTreeMap::new(),
    );
    let errors = validate_target_definition(empty).expect_err("an empty registry is incomplete");
    assert_eq!(
        errors.len(),
        OpcodeId::ALL.len(),
        "every missing primitive is reported, not just the first"
    );
}

#[test]
fn an_unreviewed_leaf_version_is_refused_externally() {
    // Upstream Bitcoin's tapscript leaf version is not this target's.
    assert_eq!(
        LeafVersion::new(0xc0),
        Err(TargetError::UnreviewedLeafVersion { offered: 0xc0 })
    );
    assert!(LeafVersion::new(0xc4).is_ok());
}

#[test]
fn the_public_surface_carries_no_identity_and_no_protocol_vocabulary() {
    // Compile-time boundary. There is no `TargetDefinitionId`,
    // `TargetDefinitionHash`, or `DeploymentInstanceHash` to name, and
    // the projection is the comparison mechanism instead.
    let first = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let second = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(first.projection(), second.projection());

    // The crate names target facts only. No attestation-contract
    // operation, object, relation, or sponsor role appears in this
    // import list, and none can: the crate depends on no first-party
    // package and so has no way to name one.
    let _: fn() -> Result<_, Vec<TargetError>> = reviewed_elements_tapscript;
}
