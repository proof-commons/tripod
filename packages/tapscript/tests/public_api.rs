//! Public-API boundary test for the tapscript adapter package.
//!
//! This integration test compiles as an external consumer. At the
//! crate-boundary stage it proves that the package exists, that it is
//! consumable from outside, and that the two vocabularies it was
//! created to join are both reachable through it, and that the adapter
//! answers the whole compiler census in typed multi-state form without
//! claiming that anything has been completed.

use compiler::target::RequiredCapability;
use tapscript::{AssessmentDisposition, TapscriptError};
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
    let definition = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .into_validated();
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
fn an_external_consumer_can_assess_the_whole_census() {
    let assessed =
        tapscript::assess_complete_census(&reviewed_target()).expect("the census assesses");

    let projected: Vec<_> = assessed
        .projection()
        .into_iter()
        .map(|projection| projection.required())
        .collect();

    assert_eq!(projected, RequiredCapability::ALL);
}

#[test]
fn no_assessment_is_a_boolean_and_none_claims_completion() {
    // The typed multi-state result is the deliverable. An external
    // consumer cannot reduce an assessment to "supported": there is no
    // such accessor, and the one disposition that would mean it —
    // a complete backend pattern — has no constructible value, because
    // its identity type is uninhabited.
    let assessed = tapscript::assess_complete_census(&reviewed_target()).expect("census");

    for projection in assessed.projection() {
        assert_ne!(
            projection.disposition(),
            AssessmentDisposition::CompleteBackendPattern,
        );
    }

    // The unreviewed sighash is visible from outside rather than hidden
    // behind an optimistic pattern obligation.
    let owner = assessed
        .assessment(RequiredCapability::OwnerAuthorization)
        .expect("the census covers owner authorization");
    assert_eq!(
        owner.disposition(),
        AssessmentDisposition::MissingTargetPrimitives,
    );
    assert!(
        owner
            .projection()
            .primitives()
            .contains(&ElementsCapability::OutputCommittingSighash),
    );
}

#[test]
fn the_error_root_is_public_and_is_a_standard_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}

    let error =
        TapscriptError::DuplicateCapabilityAssessment(RequiredCapability::PublicConstructibility);

    assert!(error.to_string().contains("assessed twice"));
    assert_error(&error);
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
