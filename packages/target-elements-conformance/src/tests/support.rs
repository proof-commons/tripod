//! Shared fixtures for the crate's own test suites.
//!
//! The network and genesis identifiers here are arbitrary nonzero
//! development values. They name nothing, authorize nothing, and are not
//! secret material of any kind.

use std::collections::BTreeSet;

use crate::protocol::{
    ExecutorCapability, ExecutorEnvironmentObservation, ExecutorHandshake, NATIVE_PROTOCOL_SCHEMA,
    WireEnvironment, WireExecutionDomain,
};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
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
) -> ReviewedDevelopmentBinding {
    let binding = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        DeploymentEnvironment::Development,
        TEST_NETWORK_ID,
        TEST_GENESIS_ID,
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    validate_reviewed_development_binding(target, binding).expect("the binding validates")
}

/// The chain name the test environment observation states.
pub const TEST_CHAIN_NAME: &str = "elementsregtest";

/// An environment observation agreeing with the test binding.
pub fn observed_environment() -> ExecutorEnvironmentObservation {
    ExecutorEnvironmentObservation {
        schema: NATIVE_PROTOCOL_SCHEMA,
        environment: WireEnvironment::Development,
        chain_name: TEST_CHAIN_NAME.to_owned(),
        network_id: TEST_NETWORK_ID,
        genesis_id: TEST_GENESIS_ID,
        active_domains: BTreeSet::from([WireExecutionDomain::Tapscript]),
        active_leaf_versions: BTreeSet::from([LeafVersion::TAPSCRIPT.get()]),
    }
}

/// A handshake from a well-behaved nonmock executor.
pub fn nonmock_handshake() -> ExecutorHandshake {
    ExecutorHandshake {
        protocol_schema: NATIVE_PROTOCOL_SCHEMA,
        adapter_name: "test-adapter".to_owned(),
        adapter_version: "0.0.0".to_owned(),
        framework_revision: Some("framework-revision".to_owned()),
        node_name: "test-node".to_owned(),
        node_version: "test-node v0".to_owned(),
        binary_reported_revision: Some("binary-revision".to_owned()),
        intended_executed_tip: Some("intended-tip".to_owned()),
        upstream_base: Some("upstream-base".to_owned()),
        included_local_topics: BTreeSet::from(["fix/example".to_owned()]),
        supported_domains: BTreeSet::from([WireExecutionDomain::Tapscript]),
        supported_leaf_versions: BTreeSet::from([LeafVersion::TAPSCRIPT.get()]),
        capabilities: BTreeSet::from([
            ExecutorCapability::FailureClassReporting,
            ExecutorCapability::ResourceObservation,
            ExecutorCapability::TransactionContext,
        ]),
    }
}
