//! Public-API boundary test for the typed Elements target contract.
//!
//! This integration test compiles as an external consumer. It proves
//! that the crate is consumable from outside, that a consumer can only
//! obtain a *validated* contract, and that no surface has leaked out
//! ahead of the deliverable that owns it.

use std::collections::BTreeMap;

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, ElementsCapability,
    EncodingClass, ExecutionDomain, LeafVersion, OpcodeId, TargetContractVersion, TargetDefinition,
    TargetDefinitionParts, TargetError, TargetEvidenceRequirementId, bind_development_target,
    reviewed_elements_tapscript, validate_development_binding, validate_target_definition,
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
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    let stripped = TargetDefinition::new(TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: BTreeMap::new(),
        encodings: source.encodings().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    });
    let errors = validate_target_definition(stripped).expect_err("an empty registry is incomplete");
    assert!(
        errors.len() > OpcodeId::ALL.len(),
        "every missing primitive is reported, and so is every capability that named one"
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

    // Each registry is complete against its own census.
    let definition = first.definition();
    assert_eq!(definition.encodings().len(), EncodingClass::ALL.len());
    assert_eq!(
        definition.capabilities().len(),
        ElementsCapability::ALL.len()
    );
    assert_eq!(
        definition.evidence_requirements().len(),
        TargetEvidenceRequirementId::ALL.len()
    );
}

#[test]
fn an_external_consumer_can_bind_a_development_instance_and_only_that() {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");

    let activation = ActivationDeclaration::new(
        true,
        LeafVersion::TAPSCRIPT,
        [ElementsCapability::TapscriptExecution],
    );
    let development = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        [0x11; 32],
        [0x22; 32],
        activation.clone(),
        None,
    );
    let bound = validate_development_binding(&definition, development)
        .expect("a well-formed development binding is accepted");

    // A production binding is nameable and is refused. There is no
    // public function in this crate that returns a validated one, and
    // no way to upgrade the development binding above into one.
    let production = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Production,
        [0x11; 32],
        [0x22; 32],
        activation,
        None,
    );
    assert_eq!(
        validate_development_binding(&definition, production),
        Err(TargetError::ProductionBindingUnsupported)
    );

    let combined =
        bind_development_target(definition, bound).expect("the contract and binding agree");

    // The combination proves typed self-consistency and nothing more.
    // Every evidence requirement the contract names is still a
    // requirement, and there is no field anywhere in this value that
    // could record otherwise.
    assert!(
        !combined
            .definition()
            .definition()
            .evidence_requirements()
            .is_empty()
    );
}
