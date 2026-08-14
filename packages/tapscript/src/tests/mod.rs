//! Adapter tests and their shared target fixtures.
//!
//! # Mutated targets are built through the target package's own API
//!
//! Every fixture below reaches a modified target the way an external
//! consumer would: read the reviewed contract's parts back out, change
//! one of them, and hand the result to the target package's validator.
//! Nothing here reaches inside `target-elements`, and no test-only
//! constructor was added there to make a mutation convenient. A backdoor
//! into a validator is a backdoor into the validator's guarantee, and
//! the guarantee is the reason the fixture is worth building.
//!
//! Two mutations a reader might expect are therefore absent, because
//! the target validator makes them unconstructible: a capability
//! removed from the registry, and an evidence requirement removed from
//! the registry. Both censuses must be complete for a definition to
//! validate at all. The reachable neighbour of each — downgrading a
//! capability's reviewed status — is used instead, and the tests say so
//! where it matters.

mod census_tests;
mod mapping_tests;
mod non_weakening_tests;

use std::collections::BTreeMap;

use target_elements::{
    ActivationDeclaration, CapabilityContract, DeploymentEnvironment, DevelopmentDeploymentBinding,
    ElementsCapability, ElementsTarget, LeafVersion, StaticCapabilityStatus, TargetContractVersion,
    TargetDefinition, TargetDefinitionParts, ValidatedTargetDefinition, bind_development_target,
    reviewed_elements_tapscript, validate_development_binding, validate_target_definition,
};

/// The reviewed contract, unmodified.
fn reviewed_definition() -> ValidatedTargetDefinition {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .into_validated()
}

/// Bind a validated contract to a development instance.
///
/// The activation declares only the execution domain. A declaration
/// naming more would be refused when a fixture downgrades one of the
/// named capabilities, and the refusal would land on the binding rather
/// than on the assessment the test is about.
fn bind(definition: ValidatedTargetDefinition) -> ElementsTarget {
    let binding = validate_development_binding(
        &definition,
        DevelopmentDeploymentBinding::new(
            TargetContractVersion::V1,
            DeploymentEnvironment::Development,
            [0x11; 32],
            [0x22; 32],
            ActivationDeclaration::new(
                true,
                LeafVersion::TAPSCRIPT,
                [ElementsCapability::TapscriptExecution],
            ),
            None,
        ),
    )
    .expect("a well-formed development binding is accepted");

    bind_development_target(definition, binding).expect("the two agree on the contract revision")
}

/// The reviewed contract bound to a development instance.
fn reviewed_target() -> ElementsTarget {
    bind(reviewed_definition())
}

/// The reviewed contract with one capability's reviewed status changed.
///
/// The closest reachable analogue of "remove one target prerequisite":
/// the registry must stay complete, so the prerequisite stops being
/// established rather than stops existing.
fn target_with_status(
    capability: ElementsCapability,
    status: StaticCapabilityStatus,
) -> ElementsTarget {
    target_with_statuses(&[(capability, status)])
}

/// The reviewed contract with several capabilities' statuses changed.
fn target_with_statuses(
    changes: &[(ElementsCapability, StaticCapabilityStatus)],
) -> ElementsTarget {
    let reviewed = reviewed_definition();
    let contract = reviewed.definition();

    let mut capabilities: BTreeMap<_, _> = contract.capabilities().clone();

    for (capability, status) in changes {
        let previous = capabilities
            .get(capability)
            .expect("a validated contract's capability registry is complete");
        let restated = CapabilityContract::new(
            previous.capability(),
            previous.prerequisites().iter().copied(),
            previous.opcodes().iter().copied(),
            previous.encodings().iter().copied(),
            previous.evidence().iter().copied(),
            *status,
        );
        capabilities.insert(*capability, restated);
    }

    let definition = validate_target_definition(TargetDefinition::new(TargetDefinitionParts {
        version: contract.version(),
        execution_domain: contract.execution_domain(),
        leaf_version: contract.leaf_version(),
        opcodes: contract.opcodes().clone(),
        encodings: contract.encodings().clone(),
        authorization: contract.authorization().clone(),
        confidential_values: contract.confidential_values().clone(),
        issuance: contract.issuance().clone(),
        resources: contract.resources().clone(),
        capabilities,
        evidence_requirements: contract.evidence_requirements().clone(),
    }))
    .expect("restating a reviewed status leaves the contract structurally valid");

    bind(definition)
}
