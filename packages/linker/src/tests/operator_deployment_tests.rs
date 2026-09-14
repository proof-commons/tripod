//! Deployment binding through checked public constructors and fixture bytes.

use tapscript::{
    EstablishedOperatorProfile, OperatorKey, OperatorKeyCurveValidity, OperatorProfileDisposition,
    StackItem, operator_key_encoding_closure, selected_operator_profile,
};
use target_elements::{EncodingClass, TargetContractVersion};

use crate::tests::reviewed_target;
use crate::{
    CandidateDeploymentIdentity, LinkRefusal, OperatorDeploymentBinding, OperatorDeploymentStatus,
};

// Public, meaningless fixture bytes: no party, real deployment, or secret.
fn operator_key(byte: u8) -> OperatorKey {
    let target = reviewed_target();
    let closure = operator_key_encoding_closure(target.definition().authorization());
    OperatorKey::new(&closure, closure.approved(), vec![byte; 32])
        .expect("fixture public bytes have the approved shape")
}

fn established_profile() -> EstablishedOperatorProfile {
    EstablishedOperatorProfile::establish(selected_operator_profile(), &reviewed_target())
        .expect("the reviewed target establishes the source selection")
}

fn identity(network: u8, genesis: u8) -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new([network; 32], [genesis; 32])
        .expect("fixture identifiers are nonzero")
}

fn binding() -> OperatorDeploymentBinding {
    let target = reviewed_target();
    let internal_key = StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0xb6; 32])
        .expect("fixture internal key has the reviewed width");
    OperatorDeploymentBinding::bind(
        &target,
        operator_key(0x33),
        established_profile(),
        identity(0x11, 0x22),
        &internal_key,
    )
    .expect("the candidate deployment binds")
}

#[test]
fn identity_refuses_zero_network_id() {
    assert_eq!(
        CandidateDeploymentIdentity::new([0; 32], [0x22; 32]),
        Err(LinkRefusal::ZeroNetworkId {
            network_id: [0; 32]
        })
    );
}

#[test]
fn identity_refuses_zero_genesis_id() {
    assert_eq!(
        CandidateDeploymentIdentity::new([0x11; 32], [0; 32]),
        Err(LinkRefusal::ZeroGenesisId {
            genesis_id: [0; 32]
        })
    );
}

#[test]
fn binding_commits_all_values_and_pins_v2() {
    let bound = binding();
    assert_eq!(bound.key(), &operator_key(0x33));
    assert_eq!(bound.profile(), &established_profile());
    assert_eq!(bound.deployment(), &identity(0x11, 0x22));
    assert_eq!(bound.deployment().network_id(), &[0x11; 32]);
    assert_eq!(bound.deployment().genesis_id(), &[0x22; 32]);
    assert_eq!(bound.capability_revision(), TargetContractVersion::V2);
    assert_eq!(
        bound.profile().capability_revision(),
        bound.capability_revision()
    );
    assert_eq!(
        bound.key().curve_validity(),
        OperatorKeyCurveValidity::Unverified
    );
}

#[test]
fn status_is_read_and_never_written() {
    // No field, constructor argument, or setter can claim promotion.
    let bound = binding();
    assert_eq!(bound.status(), OperatorDeploymentStatus::Candidate);
    assert_ne!(
        bound.status(),
        OperatorDeploymentStatus::CandidateOperationProven
    );
    assert_ne!(bound.status(), OperatorDeploymentStatus::ProductionApproved);
}

#[test]
fn bind_refuses_the_internal_key_as_operator() {
    let target = reviewed_target();
    let key = operator_key(0xb6);
    let internal_key =
        StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, key.bytes().to_vec())
            .expect("fixture internal key has the reviewed width");
    assert_eq!(
        OperatorDeploymentBinding::bind(
            &target,
            key.clone(),
            established_profile(),
            identity(0x11, 0x22),
            &internal_key,
        ),
        Err(LinkRefusal::OperatorKeyIsInternalKey { key })
    );
}

#[test]
fn check_accepts_the_bound_key_deployment_and_revision() {
    assert_eq!(
        binding().check(
            &operator_key(0x33),
            &identity(0x11, 0x22),
            TargetContractVersion::V2
        ),
        Ok(())
    );
}

#[test]
fn check_refuses_another_operator_key() {
    let other = operator_key(0x44);
    assert_eq!(
        binding().check(&other, &identity(0x11, 0x22), TargetContractVersion::V2),
        Err(LinkRefusal::OperatorKeyMismatch {
            bound: operator_key(0x33),
            offered: other
        })
    );
}

#[test]
fn check_refuses_another_genesis() {
    let other = identity(0x11, 0x44);
    assert_eq!(
        binding().check(&operator_key(0x33), &other, TargetContractVersion::V2),
        Err(LinkRefusal::OperatorDeploymentMismatch {
            bound: Box::new(identity(0x11, 0x22)),
            offered: Box::new(other),
        })
    );
}

#[test]
fn check_refuses_another_network() {
    let other = identity(0x44, 0x22);
    assert_eq!(
        binding().check(&operator_key(0x33), &other, TargetContractVersion::V2),
        Err(LinkRefusal::OperatorDeploymentMismatch {
            bound: Box::new(identity(0x11, 0x22)),
            offered: Box::new(other),
        })
    );
}

#[test]
fn check_refuses_stale_revision_v1() {
    assert_eq!(
        binding().check(
            &operator_key(0x33),
            &identity(0x11, 0x22),
            TargetContractVersion::V1
        ),
        Err(LinkRefusal::InvalidOperatorProfile(
            OperatorProfileDisposition::StaleRevision {
                pinned: TargetContractVersion::V2,
                offered: TargetContractVersion::V1,
            }
        ))
    );
}

#[test]
fn profile_matches_a_fresh_establishment_of_the_same_selection() {
    assert_eq!(binding().profile_matches(&established_profile()), Ok(()));
}

#[test]
fn check_refuses_key_before_deployment_and_deployment_before_revision() {
    let bound = binding();
    let other_key = operator_key(0x44);
    let other_deployment = identity(0x55, 0x66);
    assert_eq!(
        bound.check(&other_key, &other_deployment, TargetContractVersion::V1),
        Err(LinkRefusal::OperatorKeyMismatch {
            bound: operator_key(0x33),
            offered: other_key
        })
    );
    assert_eq!(
        bound.check(bound.key(), &other_deployment, TargetContractVersion::V1),
        Err(LinkRefusal::OperatorDeploymentMismatch {
            bound: Box::new(identity(0x11, 0x22)),
            offered: Box::new(other_deployment),
        })
    );
}
