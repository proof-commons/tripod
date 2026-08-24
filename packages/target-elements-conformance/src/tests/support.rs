//! Shared fixtures for the crate's own test suites.
//!
//! The network and genesis identifiers here are arbitrary nonzero
//! development values. They name nothing, authorize nothing, and are not
//! secret material of any kind.

use std::collections::{BTreeMap, BTreeSet};

use crate::fixture::{NativeCaseId, PrimitiveExecutionSubject, PrimitiveFixture};
use crate::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundingAdvertisement,
    ConfidentialFundingBinding, ConfidentialFundingDestination, ConfidentialFundingProfiles,
    ExecutorCapability, ExecutorEnvironmentObservation, ExecutorHandshake, FundingCustodyProfile,
    FundingMaterializerProfile, FundingRepresentationProfile, NATIVE_PROTOCOL_SCHEMA,
    TargetConfidentialFundingSubject, WireEnvironment, WireExecutionDomain,
};
use crate::prototype::{CompoundPrototypeFixture, PrototypeCaseId, PrototypeExecutionSubject};
use crate::provenance::ExpectedExecutorProvenance;
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

/// The requests an honest run over these fixtures would have sent.
///
/// Taken as an iterator of fixtures rather than as one census type, so
/// that the canonical wrapper and a bare census can both be described.
pub fn subjects_of<'a>(
    fixtures: impl IntoIterator<Item = &'a PrimitiveFixture>,
) -> BTreeMap<NativeCaseId, PrimitiveExecutionSubject> {
    fixtures
        .into_iter()
        .map(|fixture| (fixture.case(), fixture.subject()))
        .collect()
}

/// The requests an honest run over these compound rows would have sent.
pub fn prototype_subjects_of(
    rows: &[CompoundPrototypeFixture],
) -> BTreeMap<PrototypeCaseId, PrototypeExecutionSubject> {
    rows.iter()
        .map(|row| (row.case.clone(), row.subject()))
        .collect()
}

/// The integration tip the test runs declare they meant to execute.
///
/// Forty lowercase hexadecimal digits, naming nothing: no such object
/// exists, and none needs to. What the tests exercise is the comparison
/// between what an executor reports and what the operator expected.
pub const TEST_INTENDED_TIP: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678";

/// The upstream base the test runs declare that tip derives from.
pub const TEST_UPSTREAM_BASE: &str = "0123456789abcdef0123456789abcdef01234567";

/// What the test binary reports about itself.
///
/// The narrowest admitted abbreviation of [`TEST_INTENDED_TIP`], because
/// that is the shape a real node binary embeds in its version line: the
/// default test handshake therefore exercises the prefix rule rather
/// than the equality case.
pub const TEST_BINARY_REVISION: &str = "a1b2c3d";

/// The local topic the test runs declare was folded into that tip.
pub const TEST_LOCAL_TOPIC: &str = "fix/example";

/// The provenance expectation the default nonmock handshake satisfies.
pub fn expected_provenance() -> ExpectedExecutorProvenance {
    ExpectedExecutorProvenance::new(TEST_INTENDED_TIP, TEST_UPSTREAM_BASE, [TEST_LOCAL_TOPIC])
        .expect("the test expectation is in the admitted syntax")
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
        binary_reported_revision: Some(TEST_BINARY_REVISION.to_owned()),
        intended_executed_tip: Some(TEST_INTENDED_TIP.to_owned()),
        upstream_base: Some(TEST_UPSTREAM_BASE.to_owned()),
        included_local_topics: BTreeSet::from([TEST_LOCAL_TOPIC.to_owned()]),
        supported_domains: BTreeSet::from([WireExecutionDomain::Tapscript]),
        supported_leaf_versions: BTreeSet::from([LeafVersion::TAPSCRIPT.get()]),
        capabilities: BTreeSet::from([
            ExecutorCapability::FailureClassReporting,
            ExecutorCapability::ResourceObservation,
            ExecutorCapability::TransactionContext,
        ]),
        // Absent, so that a test wanting confidential funding states
        // both halves itself: the capability and the advertisement are
        // held consistent, and a helper that quietly supplied one of
        // them would make the disagreement unreachable.
        confidential_funding: None,
    }
}

/// The public handle the first confidential fixture case is registered
/// under.
///
/// A public deterministic test identity. It encodes no amount, no
/// opening, and no transaction identity, and it authorizes nothing.
pub const TEST_FIXTURE_HANDLE: &str = "ctf-v1/predecessor-dual-parity";

/// The digest that case is registered under, in these tests.
///
/// An arbitrary public development value. The registry computes the real
/// one over a framed transcript; what this stands in for is the drift
/// check, which is a comparison rather than a derivation.
pub const TEST_FIXTURE_DIGEST: [u8; 32] = [0x5a; 32];

/// The two witness programs the first slice pays.
///
/// Arbitrary public development bytes. No key is derived from either and
/// none exists.
pub const TEST_DESTINATION_PROGRAMS: [[u8; 4]; 2] =
    [[0x51, 0x20, 0xaa, 0xbb], [0x51, 0x20, 0xcc, 0xdd]];

/// The profiles the test ceremony selects.
///
/// The selection is the ceremony plan's, and every member of it must
/// appear in what the executor advertised before a request is written.
pub fn confidential_profiles() -> ConfidentialFundingProfiles {
    ConfidentialFundingProfiles {
        representation: FundingRepresentationProfile::ExplicitAssetConfidentialValue,
        custody: FundingCustodyProfile::CentralPublicFixtures,
        materializer: FundingMaterializerProfile::GuideCtfDeterministicV1,
        reproducibility_contract: target_elements::ReproducibilityContract::ByteIdentity,
    }
}

/// What a test request binds itself to.
pub fn confidential_binding() -> ConfidentialFundingBinding {
    ConfidentialFundingBinding {
        fixture_handle: ConfidentialFixtureHandle::new(TEST_FIXTURE_HANDLE.to_owned()),
        fixture_digest: ConfidentialFixtureDigest::new(TEST_FIXTURE_DIGEST),
        profiles: confidential_profiles(),
    }
}

/// One confidential funding subject, paying both test destinations.
pub fn confidential_subject() -> TargetConfidentialFundingSubject {
    TargetConfidentialFundingSubject {
        issue_asset: false,
        asset: Some("aa".repeat(32)),
        destinations: TEST_DESTINATION_PROGRAMS
            .iter()
            .map(|program| ConfidentialFundingDestination {
                output_program: program.to_vec(),
            })
            .collect(),
        binding: confidential_binding(),
    }
}

/// An advertisement admitting exactly what the test ceremony selects.
pub fn confidential_advertisement() -> ConfidentialFundingAdvertisement {
    ConfidentialFundingAdvertisement {
        representation_profiles: BTreeSet::from([
            FundingRepresentationProfile::ExplicitAssetConfidentialValue,
        ]),
        custody_profiles: BTreeSet::from([FundingCustodyProfile::CentralPublicFixtures]),
        materializer_profiles: BTreeSet::from([
            FundingMaterializerProfile::GuideCtfDeterministicV1,
        ]),
        reproducibility_contracts: BTreeSet::from([
            target_elements::ReproducibilityContract::ByteIdentity,
        ]),
    }
}

/// A handshake from an executor that offers confidential funding, with
/// the capability and the advertisement agreeing.
pub fn confidential_handshake() -> ExecutorHandshake {
    let mut handshake = nonmock_handshake();
    handshake
        .capabilities
        .insert(ExecutorCapability::ConfidentialValueTestFunding);
    handshake.confidential_funding = Some(confidential_advertisement());
    handshake
}
