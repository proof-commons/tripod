//! Mutation tests for capability status closure.
//!
//! The prerequisite relation already had to resolve and had to be
//! acyclic. Neither of those says anything about *strength*, and a
//! reviewed capability standing on an unsupported one is not a strong
//! claim resting on a weak one — it is a claim that cannot be
//! realized. These tests damage exactly one status at a time and
//! require the validator to name both ends of the violation.

use std::collections::BTreeMap;

use crate::capability::{
    CapabilityContract, ElementsCapability, StaticCapabilityStatus, status_closure_violations,
    transitive_prerequisites,
};
use crate::definition::{
    TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::error::TargetError;

/// The reviewed capability registry, as a mutable starting point.
fn registry() -> BTreeMap<ElementsCapability, CapabilityContract> {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .capabilities()
        .clone()
}

/// Rewrites one capability's status, leaving everything else reviewed.
fn with_status(
    registry: &mut BTreeMap<ElementsCapability, CapabilityContract>,
    capability: ElementsCapability,
    status: StaticCapabilityStatus,
) {
    let source = registry
        .get(&capability)
        .expect("the reviewed registry declares it")
        .clone();
    registry.insert(
        capability,
        CapabilityContract::new(
            source.capability(),
            source.prerequisites().iter().copied(),
            source.opcodes().iter().copied(),
            source.encodings().iter().copied(),
            source.evidence().iter().copied(),
            status,
        ),
    );
}

/// Validates a contract carrying a damaged capability registry.
fn errors_for(capabilities: BTreeMap<ElementsCapability, CapabilityContract>) -> Vec<TargetError> {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    let definition = TargetDefinition::new(TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities,
        evidence_requirements: source.evidence_requirements().clone(),
    });
    validate_target_definition(definition).expect_err("the mutation must be rejected")
}

#[test]
fn the_status_order_runs_from_unsupported_to_reviewed() {
    use StaticCapabilityStatus::{Incomplete, Reviewed, Unsupported};

    // Written out rather than derived: the declaration order runs the
    // other way, so a derived comparison would invert every check
    // below without changing a single test name.
    assert!(Unsupported.strength() < Incomplete.strength());
    assert!(Incomplete.strength() < Reviewed.strength());

    assert!(Unsupported.at_most(Reviewed));
    assert!(Incomplete.at_most(Incomplete));
    assert!(!Reviewed.at_most(Incomplete));
    assert!(!Incomplete.at_most(Unsupported));
}

#[test]
fn the_reviewed_registry_is_status_closed() {
    // The control, and the honest question this whole check asks of
    // the project's own contract: every reviewed row must stand on
    // reviewed ground.
    assert_eq!(status_closure_violations(&registry()), Vec::new());
}

#[test]
fn a_reviewed_capability_over_an_incomplete_prerequisite_is_rejected() {
    let mut capabilities = registry();
    with_status(
        &mut capabilities,
        ElementsCapability::TapscriptExecution,
        StaticCapabilityStatus::Incomplete,
    );

    // Input asset inspection reaches the execution domain directly.
    assert!(
        errors_for(capabilities).contains(&TargetError::CapabilityStatusExceedsPrerequisite {
            capability: ElementsCapability::InputAssetInspection,
            prerequisite: ElementsCapability::TapscriptExecution,
        })
    );
}

#[test]
fn a_reviewed_capability_over_an_unsupported_prerequisite_is_rejected() {
    let mut capabilities = registry();
    with_status(
        &mut capabilities,
        ElementsCapability::TapscriptExecution,
        StaticCapabilityStatus::Unsupported,
    );

    let errors = errors_for(capabilities);
    assert!(
        errors.contains(&TargetError::CapabilityStatusExceedsPrerequisite {
            capability: ElementsCapability::InputProgramInspection,
            prerequisite: ElementsCapability::TapscriptExecution,
        })
    );
}

#[test]
fn an_incomplete_capability_over_an_unsupported_prerequisite_is_rejected() {
    let mut capabilities = registry();
    // Sighash commitment is already incomplete; make the signature
    // primitive it stands on unsupported and nothing else.
    with_status(
        &mut capabilities,
        ElementsCapability::SignatureVerification,
        StaticCapabilityStatus::Unsupported,
    );

    assert!(
        errors_for(capabilities).contains(&TargetError::CapabilityStatusExceedsPrerequisite {
            capability: ElementsCapability::OutputCommittingSighash,
            prerequisite: ElementsCapability::SignatureVerification,
        })
    );
}

#[test]
fn a_violation_is_found_through_a_transitive_chain() {
    // Reissuance reaches the execution domain only through issuance
    // introspection and input issuance inspection. A check that looked
    // at direct prerequisites alone would miss it entirely.
    let mut capabilities = registry();
    with_status(
        &mut capabilities,
        ElementsCapability::TapscriptExecution,
        StaticCapabilityStatus::Unsupported,
    );

    let closure =
        transitive_prerequisites(&capabilities, ElementsCapability::ReissuanceIntrospection);
    assert!(closure.contains(&ElementsCapability::TapscriptExecution));
    assert!(
        !registry()[&ElementsCapability::ReissuanceIntrospection]
            .prerequisites()
            .contains(&ElementsCapability::TapscriptExecution),
        "the violation is reachable only transitively"
    );

    assert!(
        errors_for(capabilities).contains(&TargetError::CapabilityStatusExceedsPrerequisite {
            capability: ElementsCapability::ReissuanceIntrospection,
            prerequisite: ElementsCapability::TapscriptExecution,
        })
    );
}

#[test]
fn a_weaker_capability_over_a_stronger_prerequisite_is_accepted() {
    // Closure bounds a capability from above and says nothing from
    // below. Authenticated opening is unsupported while everything it
    // requires is reviewed, and that is a complete chain rather than a
    // defect.
    let mut capabilities = registry();
    with_status(
        &mut capabilities,
        ElementsCapability::ExplicitValueInspection,
        StaticCapabilityStatus::Unsupported,
    );
    assert_eq!(status_closure_violations(&capabilities), Vec::new());
}

#[test]
fn the_closure_does_not_depend_on_registry_insertion_order() {
    // The registry is keyed, so its iteration order is fixed; what is
    // being checked is that the closure walk itself reaches the same
    // verdict no matter which capability is damaged first.
    let mut forward = registry();
    with_status(
        &mut forward,
        ElementsCapability::RequiredLeafVersion,
        StaticCapabilityStatus::Incomplete,
    );
    with_status(
        &mut forward,
        ElementsCapability::TapscriptExecution,
        StaticCapabilityStatus::Incomplete,
    );

    let mut reverse = registry();
    with_status(
        &mut reverse,
        ElementsCapability::TapscriptExecution,
        StaticCapabilityStatus::Incomplete,
    );
    with_status(
        &mut reverse,
        ElementsCapability::RequiredLeafVersion,
        StaticCapabilityStatus::Incomplete,
    );

    assert_eq!(
        status_closure_violations(&forward),
        status_closure_violations(&reverse)
    );
    assert!(!status_closure_violations(&forward).is_empty());
}

#[test]
fn a_prerequisite_cycle_does_not_hang_the_closure_walk() {
    // The cycle is reported by its own check. This one only has to
    // terminate, and it must not be tempted to expand a capability it
    // has already seen.
    let mut capabilities = registry();
    let leaf = capabilities[&ElementsCapability::RequiredLeafVersion].clone();
    capabilities.insert(
        ElementsCapability::RequiredLeafVersion,
        CapabilityContract::new(
            leaf.capability(),
            [ElementsCapability::TapscriptExecution],
            leaf.opcodes().iter().copied(),
            leaf.encodings().iter().copied(),
            leaf.evidence().iter().copied(),
            leaf.status(),
        ),
    );

    let closure = transitive_prerequisites(&capabilities, ElementsCapability::RequiredLeafVersion);
    assert!(closure.contains(&ElementsCapability::TapscriptExecution));
    assert!(!closure.contains(&ElementsCapability::RequiredLeafVersion));
}
