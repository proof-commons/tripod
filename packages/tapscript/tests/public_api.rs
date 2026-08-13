//! Public-API boundary test for the tapscript adapter package.
//!
//! This integration test compiles as an external consumer. At the
//! crate-boundary stage it proves that the package exists, that it is
//! consumable from outside, and that the two vocabularies it was
//! created to join are both reachable through it — which is the whole
//! content of the boundary before an adapter exists.

use compiler::target::RequiredCapability;
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, ElementsCapability,
    ElementsTarget, LeafVersion, TargetContractVersion, bind_development_target,
    reviewed_elements_tapscript, validate_development_binding,
};

/// The reviewed contract bound to a development instance.
///
/// Built here rather than imported: `target-elements` publishes no
/// ready-made target, deliberately, so that every consumer states the
/// deployment it means instead of inheriting an unexamined one.
fn reviewed_target() -> ElementsTarget {
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
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

#[test]
fn both_joined_vocabularies_are_reachable_through_this_package() {
    // The dependency edge is the deliverable of this commit: an
    // external consumer of `tapscript` can name a compiler-owned
    // abstract capability and a target-owned primitive in one place.
    // Nothing here relates them — relating them is the adapter's job,
    // and inventing a relation in a test would be exactly the
    // unreviewed mapping the adapter exists to make explicit.
    assert_ne!(RequiredCapability::ALL, []);
    assert_ne!(ElementsCapability::ALL, []);
}

#[test]
fn the_reviewed_target_binds_and_resolves_no_evidence() {
    let target = reviewed_target();
    let contract = target.definition().definition();

    // Every requirement the contract names is still open. An adapter
    // built on this value cannot read anything here as evidence that
    // one has been discharged, because there is no such value: a
    // requirement identity carries no result and has no field for one.
    assert!(!contract.evidence_requirements().is_empty());
    assert!(!contract.capabilities().is_empty());
}
