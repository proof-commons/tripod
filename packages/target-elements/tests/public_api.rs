//! Public-API boundary test for the typed Elements target contract.
//!
//! This integration test compiles as an external consumer. It proves
//! that the crate is consumable from outside, that a consumer can only
//! obtain a *validated* contract, and that no surface has leaked out
//! ahead of the deliverable that owns it.

use std::collections::BTreeMap;

use target_elements::{
    ActivationDeclaration, AuthorizationContract, CapabilityContract, DeploymentEnvironment,
    DevelopmentDeploymentBinding, ElementsCapability, EncodingClass, ExecutionDomain, FailureCause,
    FailureContract, FailureEffect, FailureOutcome, LeafVersion, OpcodeId, OpcodeSpec,
    PolicyResourceLimits, ResourceBound, ResourceContract, ResourceDimension,
    ReviewedElementsTapscriptDefinition, SignaturePrimitiveContract, StackContract,
    StaticCapabilityStatus, TargetContractVersion, TargetDefinition, TargetDefinitionParts,
    TargetError, TargetEvidenceRequirementId, ValidatedTargetDefinition, bind_development_target,
    reviewed_elements_tapscript, validate_as_reviewed_elements, validate_development_binding,
    validate_target_definition,
};

/// The reviewed contract's parts, ready for one deliberate mutation.
fn reviewed_parts() -> TargetDefinitionParts {
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

/// Validates a mutated contract and asserts that it stays generic.
///
/// The two assertions are the whole point of the reviewed trust state:
/// a caller's contract may be perfectly well-formed, and being
/// well-formed still does not make it this project's contract.
fn stays_generic(parts: TargetDefinitionParts) {
    let validated = validate_target_definition(TargetDefinition::new(parts))
        .expect("the mutation leaves an internally coherent contract");
    assert_eq!(
        validate_as_reviewed_elements(validated),
        Err(TargetError::ReviewedDefinitionMismatch)
    );
}

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
    let definition = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .into_validated();

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

#[test]
fn only_the_first_party_contract_carries_the_reviewed_trust_state() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");

    // The exact reviewed value promotes.
    let promoted = validate_as_reviewed_elements(reviewed.validated().clone())
        .expect("the reviewed contract is the reviewed contract");
    assert_eq!(promoted, reviewed);

    // Promotion is the only route into the reviewed state from
    // outside, and it consumes a *validated* contract: there is no
    // public constructor taking an unvalidated one and no way to
    // assert the state directly.
    let _: fn(
        ValidatedTargetDefinition,
    ) -> Result<ReviewedElementsTapscriptDefinition, TargetError> = validate_as_reviewed_elements;
}

#[test]
fn a_permuted_opcode_byte_never_becomes_reviewed() {
    let mut parts = reviewed_parts();
    let source = parts
        .opcodes
        .get(&OpcodeId::Sha256Initialize)
        .expect("the reviewed registry declares it")
        .clone();
    // 0xc3 is unclaimed by the reviewed census, so the mutated
    // registry keeps unique bytes and stays internally coherent.
    parts.opcodes.insert(
        OpcodeId::Sha256Initialize,
        OpcodeSpec::new(
            source.id(),
            0xc3,
            source.domains().iter().copied(),
            source.stack().clone(),
            source.resources(),
            source.evidence().iter().copied(),
        ),
    );
    stays_generic(parts);
}

#[test]
fn changed_signature_failure_behavior_never_becomes_reviewed() {
    let mut parts = reviewed_parts();

    // Both views of the same fact move together, so the mutated
    // contract survives the cross-contract welds as well as the local
    // checks. It is coherent; it is simply not this project's.
    for opcode in [OpcodeId::CheckSig, OpcodeId::CheckSigFromStack] {
        let source = parts
            .opcodes
            .get(&opcode)
            .expect("the reviewed registry declares it")
            .clone();
        let stack = source.stack();
        let mut effects: Vec<FailureEffect> = stack
            .failure()
            .effects()
            .iter()
            .filter(|effect| effect.cause() != FailureCause::EmptySignature)
            .copied()
            .collect();
        effects.push(FailureEffect::new(
            FailureCause::EmptySignature,
            FailureOutcome::AbortEvaluation,
        ));
        parts.opcodes.insert(
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

    let signature = parts.authorization.signature();
    parts.authorization = AuthorizationContract::new(
        SignaturePrimitiveContract::new(
            signature.public_key_encoding(),
            signature.signature_encoding(),
            FailureOutcome::AbortEvaluation,
            signature.invalid_signature(),
            signature.unknown_public_key_type(),
            signature.budget_per_check(),
            signature.evidence().iter().copied(),
        ),
        parts.authorization.sighash().clone(),
        parts.authorization.relative_timelock().clone(),
    );

    stays_generic(parts);
}

#[test]
fn a_changed_capability_status_never_becomes_reviewed() {
    let mut parts = reviewed_parts();
    // A leaf of the prerequisite graph, so downgrading it cannot make
    // any other capability exceed its weakest prerequisite.
    let source = parts
        .capabilities
        .get(&ElementsCapability::PolicyResourceLimits)
        .expect("the reviewed registry declares it")
        .clone();
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
    stays_generic(parts);
}

#[test]
fn a_changed_resource_limit_never_becomes_reviewed() {
    let mut parts = reviewed_parts();
    // Stricter than consensus, so still a coherent policy bound.
    parts.resources = ResourceContract::new(
        parts.resources.consensus().clone(),
        PolicyResourceLimits::new([(
            ResourceDimension::TransactionWeight,
            ResourceBound::Maximum(300_000),
        )]),
    );
    stays_generic(parts);
}
