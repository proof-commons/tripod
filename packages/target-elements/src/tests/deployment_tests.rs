//! Tests for the development deployment binding.

use crate::capability::ElementsCapability;
use crate::definition::{
    TargetContractVersion, TargetDefinition, TargetDefinitionParts, ValidatedTargetDefinition,
    reviewed_elements_tapscript, validate_target_definition,
};
use crate::deployment::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding,
    DevelopmentResourceOverrides, bind_development_target, overridable_dimensions,
    validate_development_binding, validate_reviewed_development_binding,
};
use crate::error::TargetError;
use crate::opcode::LeafVersion;
use crate::resource::{PolicyResourceLimits, ResourceBound, ResourceContract, ResourceDimension};

/// A synthetic development network identifier.
///
/// Nonzero in every byte position that matters, because an all-zero
/// identifier is exactly what the validator refuses.
const NETWORK_ID: [u8; 32] = [0x11; 32];

/// A synthetic development genesis identifier.
const GENESIS_ID: [u8; 32] = [0x22; 32];

/// The reviewed contract.
fn target() -> ValidatedTargetDefinition {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .into_validated()
}

/// A well-formed activation declaration.
fn activation() -> ActivationDeclaration {
    ActivationDeclaration::new(
        true,
        LeafVersion::TAPSCRIPT,
        [
            ElementsCapability::TapscriptExecution,
            ElementsCapability::SignedFixedWidthArithmetic,
            ElementsCapability::InputValueInspection,
        ],
    )
}

/// A well-formed development binding.
fn binding() -> DevelopmentDeploymentBinding {
    DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        GENESIS_ID,
        activation(),
        None,
    )
}

#[test]
fn a_well_formed_development_binding_is_accepted() {
    let validated =
        validate_development_binding(&target(), binding()).expect("the binding is coherent");
    assert_eq!(
        validated.binding().environment(),
        DeploymentEnvironment::Development
    );
    assert_eq!(validated.binding().network_id(), NETWORK_ID);
}

#[test]
fn a_production_binding_is_refused() {
    // Refused first and unconditionally. There is no production
    // evidence boundary, so there is no valid production binding and
    // no public function in this crate returns one.
    let production = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Production,
        NETWORK_ID,
        GENESIS_ID,
        activation(),
        None,
    );
    assert_eq!(
        validate_development_binding(&target(), production),
        Err(TargetError::ProductionBindingUnsupported)
    );
}

#[test]
fn a_production_binding_is_refused_even_when_everything_else_is_wrong() {
    // The environment check must not be reachable only after the
    // other checks pass, or a caller could learn that a production
    // binding is "otherwise fine".
    let production = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Production,
        [0_u8; 32],
        [0_u8; 32],
        activation(),
        None,
    );
    assert_eq!(
        validate_development_binding(&target(), production),
        Err(TargetError::ProductionBindingUnsupported)
    );
}

#[test]
fn a_zero_network_identifier_is_refused() {
    // An all-zero identifier is the shape an uninitialized buffer
    // takes. Accepting one would let a binding that names no network
    // look exactly like a binding that names a network.
    let zeroed = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        [0_u8; 32],
        GENESIS_ID,
        activation(),
        None,
    );
    assert_eq!(
        validate_development_binding(&target(), zeroed),
        Err(TargetError::ZeroNetworkId)
    );
}

#[test]
fn a_zero_genesis_identifier_is_refused() {
    let zeroed = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        [0_u8; 32],
        activation(),
        None,
    );
    assert_eq!(
        validate_development_binding(&target(), zeroed),
        Err(TargetError::ZeroGenesisId)
    );
}

#[test]
fn an_almost_zero_identifier_is_accepted() {
    // The control. The rule is "all zero", not "mostly zero": a real
    // identifier may legitimately be mostly zero bytes, and rejecting
    // one would be a different and wrong rule.
    let mut nearly = [0_u8; 32];
    nearly[31] = 1;
    let sparse = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        nearly,
        nearly,
        activation(),
        None,
    );
    assert!(validate_development_binding(&target(), sparse).is_ok());
}

#[test]
fn a_declaration_naming_another_leaf_version_is_refused() {
    // There is no way to build a `LeafVersion` the contract does not
    // review, so this mismatch cannot be constructed at all: the
    // reviewed set has one member and the contract requires it.
    assert_eq!(LeafVersion::REVIEWED.len(), 1);
    assert_eq!(target().definition().leaf_version(), LeafVersion::TAPSCRIPT);
    assert!(LeafVersion::new(0xc0).is_err());
}

#[test]
fn a_declaration_relying_on_an_unsupported_capability_is_refused() {
    // A deployment cannot make available what no reviewed mechanism
    // provides. Declaring an intent to rely on an unsupported
    // capability is a caller mistake, not an environment to go and
    // check, so it is refused here rather than deferred to a test that
    // would fail much later and less clearly.
    for unsupported in [
        ElementsCapability::AuthenticatedValueOpening,
        ElementsCapability::CommitmentEquality,
    ] {
        let optimistic = DevelopmentDeploymentBinding::new(
            TargetContractVersion::V1,
            DeploymentEnvironment::Development,
            NETWORK_ID,
            GENESIS_ID,
            ActivationDeclaration::new(
                true,
                LeafVersion::TAPSCRIPT,
                [ElementsCapability::TapscriptExecution, unsupported],
            ),
            None,
        );
        assert_eq!(
            validate_development_binding(&target(), optimistic),
            Err(TargetError::UnsupportedRequiredCapability(unsupported))
        );
    }
}

#[test]
fn a_declaration_relying_on_an_incomplete_capability_is_accepted() {
    // Incomplete is not unsupported. A caller may legitimately intend
    // to test against a capability whose review is unfinished — that
    // is what a development network is for — so only the capabilities
    // with no reviewed mechanism at all are refused.
    let exploratory = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        GENESIS_ID,
        ActivationDeclaration::new(
            true,
            LeafVersion::TAPSCRIPT,
            [
                ElementsCapability::TapscriptExecution,
                ElementsCapability::OutputCommittingSighash,
                ElementsCapability::ConfidentialValueConservation,
            ],
        ),
        None,
    );
    assert!(validate_development_binding(&target(), exploratory).is_ok());
}

#[test]
fn an_inconsistent_activation_declaration_is_refused() {
    // Expecting the domain to be inactive while requiring capabilities
    // that exist only inside it describes an environment that cannot
    // satisfy the declaration.
    let inconsistent = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        GENESIS_ID,
        ActivationDeclaration::new(
            false,
            LeafVersion::TAPSCRIPT,
            [ElementsCapability::TapscriptExecution],
        ),
        None,
    );
    assert_eq!(
        validate_development_binding(&target(), inconsistent),
        Err(TargetError::InconsistentActivationDeclaration)
    );
}

#[test]
fn a_narrowing_resource_override_is_accepted() {
    // A deployment may enforce a stricter bound than the target does.
    let narrowed = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        GENESIS_ID,
        activation(),
        Some(DevelopmentResourceOverrides::new(
            PolicyResourceLimits::new([(
                ResourceDimension::TransactionWeight,
                ResourceBound::Maximum(100_000),
            )]),
        )),
    );
    assert!(validate_development_binding(&target(), narrowed).is_ok());
}

#[test]
fn a_widening_resource_override_is_refused() {
    // It cannot widen what the target permits: a transaction the
    // target refuses is not made valid by a permissive local setting.
    let widened = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        GENESIS_ID,
        activation(),
        Some(DevelopmentResourceOverrides::new(
            PolicyResourceLimits::new([(
                ResourceDimension::TransactionWeight,
                ResourceBound::Maximum(9_000_000),
            )]),
        )),
    );
    assert_eq!(
        validate_development_binding(&target(), widened),
        Err(TargetError::IncompatibleResourceOverride(
            ResourceDimension::TransactionWeight
        ))
    );
}

#[test]
fn an_unbounded_override_over_a_bounded_dimension_is_refused() {
    // Declaring no bound where the target has one is the widest
    // possible widening.
    let unbounded = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        GENESIS_ID,
        activation(),
        Some(DevelopmentResourceOverrides::new(
            PolicyResourceLimits::new([(
                ResourceDimension::PeakStackItems,
                ResourceBound::Unbounded,
            )]),
        )),
    );
    assert_eq!(
        validate_development_binding(&target(), unbounded),
        Err(TargetError::IncompatibleResourceOverride(
            ResourceDimension::PeakStackItems
        ))
    );
}

#[test]
fn an_override_of_an_unbounded_dimension_is_refused() {
    // There is nothing to narrow where the target states no bound, so
    // an override there is a claim the contract cannot check.
    let stray = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        NETWORK_ID,
        GENESIS_ID,
        activation(),
        Some(DevelopmentResourceOverrides::new(
            PolicyResourceLimits::new([(
                ResourceDimension::PackageLimit,
                ResourceBound::Maximum(25),
            )]),
        )),
    );
    assert_eq!(
        validate_development_binding(&target(), stray),
        Err(TargetError::UnknownOverrideDimension(
            ResourceDimension::PackageLimit
        ))
    );
}

#[test]
fn the_overridable_dimensions_are_the_ones_the_target_bounds() {
    let dimensions = overridable_dimensions(&target());
    assert!(dimensions.contains(&ResourceDimension::TransactionWeight));
    assert!(dimensions.contains(&ResourceDimension::PeakStackItems));
    assert!(!dimensions.contains(&ResourceDimension::PackageLimit));
}

#[test]
fn the_combination_proves_only_typed_self_consistency() {
    let definition = target();
    let deployment =
        validate_development_binding(&definition, binding()).expect("the binding is coherent");
    let combined = bind_development_target(definition, deployment).expect("the two agree");

    // What it does prove.
    assert_eq!(
        combined.deployment().binding().target_version(),
        combined.definition().definition().version()
    );

    // What it does not prove: every evidence requirement the contract
    // names is still unresolved, and there is no field anywhere in the
    // combined value that could record otherwise.
    assert!(
        !combined
            .definition()
            .definition()
            .evidence_requirements()
            .is_empty(),
        "requirements exist and remain requirements"
    );
}

#[test]
fn repeated_binding_construction_is_equal() {
    let definition = target();
    let first =
        validate_development_binding(&definition, binding()).expect("the binding is coherent");
    let second =
        validate_development_binding(&definition, binding()).expect("the binding is coherent");
    assert_eq!(first, second);
    assert_eq!(first.projection(), second.projection());
}

#[test]
fn the_deployment_projection_tracks_the_network_identity() {
    let definition = target();
    let first =
        validate_development_binding(&definition, binding()).expect("the binding is coherent");

    let other = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        [0x33; 32],
        GENESIS_ID,
        activation(),
        None,
    );
    let second = validate_development_binding(&definition, other).expect("the binding is coherent");

    // A different network is a different deployment, and the
    // projection has to say so.
    assert_ne!(first.projection(), second.projection());

    // The target contract is untouched by either. A binding names a
    // network; it does not edit what the target is said to do.
    assert_eq!(definition, target());
}

#[test]
fn the_binding_carries_no_credential_field() {
    // A readable statement of a compile-time boundary. The accessor
    // surface below is the whole of it: there is no endpoint, no
    // username, no password, no cookie path, no bearer token, no key,
    // and no wallet path to read, and a future runner that needs one
    // needs its own security design rather than a field here.
    let definition = target();
    let validated =
        validate_development_binding(&definition, binding()).expect("the binding is coherent");
    let binding = validated.binding();

    let _ = binding.target_version();
    let _ = binding.environment();
    let _ = binding.network_id();
    let _ = binding.genesis_id();
    let _ = binding.activation();
    let _ = binding.resource_overrides();
}

#[test]
fn an_activation_declaration_is_input_rather_than_evidence() {
    // The declaration says what the caller intends to test against. It
    // records nothing about what any node did, and validating it
    // changes that not at all.
    let declaration = activation();
    assert!(declaration.tapscript_expected_active());
    assert_eq!(declaration.required_leaf_version(), LeafVersion::TAPSCRIPT);
    assert!(
        declaration
            .required_capabilities()
            .contains(&ElementsCapability::TapscriptExecution)
    );

    let definition = target();
    let validated =
        validate_development_binding(&definition, binding()).expect("the binding is coherent");
    assert_eq!(validated.binding().activation(), &declaration);
}

#[test]
fn a_reviewed_binding_retains_the_exact_contract_it_validated_against() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let welded =
        validate_reviewed_development_binding(&reviewed, binding()).expect("the binding is valid");

    // The retained value is the complete projection, not the revision
    // number: everything the contract says travels with the binding.
    assert_eq!(welded.target(), &reviewed.projection());
    assert!(welded.welded_to(&reviewed));
    assert_eq!(welded.binding().network_id(), NETWORK_ID);
    assert_eq!(welded.projection().genesis_id(), GENESIS_ID);
    assert_eq!(
        welded.deployment(),
        &validate_development_binding(&target(), binding()).expect("the binding is valid")
    );
}

#[test]
fn contract_revision_equality_does_not_establish_contract_equality() {
    // Two internally coherent contracts can carry one revision number
    // and still say different things. This one narrows the policy
    // resource interface to nothing, which the generic validator
    // accepts — a policy bound cannot be looser than consensus, and an
    // absent one is not looser. A consumer comparing revisions would
    // treat the two as interchangeable; the projection does not.
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    let neighbour = validate_target_definition(TargetDefinition::new(TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        pushes: source.pushes().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: ResourceContract::new(
            source.resources().consensus().clone(),
            PolicyResourceLimits::new([]),
        ),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }))
    .expect("narrowing the policy interface leaves a coherent contract");

    let welded =
        validate_reviewed_development_binding(&reviewed, binding()).expect("the binding is valid");

    // Same revision, different contract — and the weld sees it.
    assert_eq!(neighbour.definition().version(), source.version());
    assert_eq!(welded.target().version(), neighbour.projection().version());
    assert_ne!(welded.target(), &neighbour.projection());
}
