//! Shared fixtures for the crate's own test suites.
//!
//! The network and genesis identifiers here are arbitrary nonzero
//! development values. They name nothing, authorize nothing, and are not
//! secret material of any kind.

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    ReviewedElementsTapscriptDefinition, TargetContractVersion, ValidatedDevelopmentBinding,
    reviewed_elements_tapscript, validate_development_binding,
};

/// A public development network identifier.
pub const TEST_NETWORK_ID: [u8; 32] = [0x11; 32];

/// A public development genesis identifier.
pub const TEST_GENESIS_ID: [u8; 32] = [0x22; 32];

/// The reviewed first-party target contract.
pub fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// A validated development binding over the reviewed contract.
pub fn development_binding(
    target: &ReviewedElementsTapscriptDefinition,
) -> ValidatedDevelopmentBinding {
    let binding = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        TEST_NETWORK_ID,
        TEST_GENESIS_ID,
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    validate_development_binding(target.validated(), binding).expect("the binding validates")
}
