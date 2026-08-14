//! Trust-state and boundary tests for the public assessment API
//! (Guide-9 §1.2, §1.3, §7.1–§7.3).
//!
//! Two properties are structural rather than assertable at run time:
//! the public entry points take
//! `ReviewedElementsTapscriptDefinition` by type, and no function in
//! this crate takes an `ElementsTarget` at all. A test cannot observe
//! the absence of a signature, so what is checked here is the fact that
//! makes the type boundary bite — that a contract this crate can
//! actually build, and that the generic validator accepts, cannot
//! become reviewed — together with the consequences a reader would
//! otherwise have to take on trust.

use std::collections::BTreeSet;

use compiler::target::{ExternalEvidenceRole, RequiredCapability};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, ElementsCapability,
    LeafVersion, StaticCapabilityStatus, TargetContractVersion, TargetError,
    bind_development_target, validate_as_reviewed_elements, validate_development_binding,
};

use super::{reviewed_definition, reviewed_target, target_with_status};

#[test]
fn a_degraded_contract_cannot_become_the_reviewed_one() {
    // The fixture the non-weakening tests assess is a validated
    // contract and can be nothing more. If this ever stopped holding,
    // the crate-private assessment path would have a public twin and
    // the trust state would be decorative.
    let degraded = target_with_status(
        ElementsCapability::OutputProgramInspection,
        StaticCapabilityStatus::Incomplete,
    );

    assert_ne!(degraded, reviewed_definition());
    assert_eq!(
        validate_as_reviewed_elements(degraded).unwrap_err(),
        TargetError::ReviewedDefinitionMismatch,
    );
}

#[test]
fn the_unmodified_reviewed_contract_still_promotes() {
    // The mirror of the test above: the rejection is a property of the
    // difference, not a validator that refuses everything.
    let promoted = validate_as_reviewed_elements(reviewed_definition())
        .expect("the reviewed contract is the reviewed contract");

    assert_eq!(promoted, reviewed_target());
}

#[test]
fn a_deployment_binding_reaches_no_assessment() {
    // Guide-9 §1.3 and R2-N02. A binding is constructible, and two
    // bindings can differ in network, genesis, and activation while
    // naming one static contract. None of that is an input here: the
    // adapter's entry points take the reviewed static contract, so
    // there is no route by which a binding could change an assessment,
    // and equally no route by which one could be read and quietly
    // ignored.
    let bindings = [
        ([0x11; 32], [0x22; 32], true),
        ([0xaa; 32], [0xbb; 32], false),
    ];

    for (network, genesis, active) in bindings {
        let definition = reviewed_definition();
        let binding = validate_development_binding(
            &definition,
            DevelopmentDeploymentBinding::new(
                TargetContractVersion::V1,
                DeploymentEnvironment::Development,
                network,
                genesis,
                ActivationDeclaration::new(
                    active,
                    LeafVersion::TAPSCRIPT,
                    if active {
                        vec![ElementsCapability::TapscriptExecution]
                    } else {
                        vec![]
                    },
                ),
                None,
            ),
        )
        .expect("a well-formed development binding is accepted");
        let target = bind_development_target(definition, binding)
            .expect("the two agree on the contract revision");

        // The one value an `ElementsTarget` can offer this crate is its
        // static definition, and even that has to earn the reviewed
        // state before any public entry point takes it.
        let reviewed = validate_as_reviewed_elements(target.definition().clone())
            .expect("this deployment binds the reviewed contract");

        assert_eq!(
            crate::assess_complete_census(&reviewed).expect("census"),
            crate::assess_complete_census(&reviewed_target()).expect("census"),
            "the static assessment is a function of the static contract alone",
        );
    }
}

#[test]
fn both_published_censuses_are_answered_exhaustively() {
    // The compiler publishes two censuses; a set that answered one of
    // them would be the R2-C05 defect restored.
    let assessed = crate::assess_complete_census(&reviewed_target()).expect("census");

    assert_eq!(
        assessed
            .capability_assessments()
            .map(|(capability, _)| capability)
            .collect::<BTreeSet<_>>(),
        RequiredCapability::ALL.iter().copied().collect(),
    );
    assert_eq!(
        assessed
            .evidence_assessments()
            .map(|(role, _)| role)
            .collect::<BTreeSet<_>>(),
        ExternalEvidenceRole::ALL.iter().copied().collect(),
    );
}
