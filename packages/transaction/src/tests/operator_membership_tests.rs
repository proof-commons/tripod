//! Membership evidence over the shared public-data signing double.
//!
//! These fixtures prove boundary decisions, not Schnorr or native acceptance.

use architecture::OperationId;
use realization::{OperatorMembershipDisposition, RealizationScope};
use target_elements::TargetContractVersion;

use super::live_support::{OperatorRightFixture, OperatorRightVerifier};
use super::{outpoint, reviewed_target};
use crate::operator_membership::{
    OperatorMappingRefusal, OperatorMembershipMapping, OperatorMembershipRefusal,
    OperatorMembershipRequest, produce_operator_membership,
};
use crate::operator_right::{
    BranchContext, NonEquivocationEvent, OperatorRightRegistry, RightRefusal, RightScope,
};
use crate::operator_signing::{OperatorAuthorizedCandidate, authorize_operator};

fn fixture() -> OperatorRightFixture {
    OperatorRightFixture::new(0x11, [0x21; 32], 0x33)
}
fn operation(operation: OperationId) -> RealizationScope {
    RealizationScope::from_operations([operation]).unwrap()
}
fn mapping(fixture: &OperatorRightFixture) -> OperatorMembershipMapping {
    OperatorMembershipMapping::new(
        fixture.binding.clone(),
        fixture.binding.profile().clone(),
        operation(OperationId::AnnounceMaturity),
    )
    .unwrap()
}
fn authorized(fixture: &OperatorRightFixture) -> OperatorAuthorizedCandidate<'_> {
    let request = fixture.freeze();
    let response = fixture.response(&request);
    authorize_operator(request, [response], &OperatorRightVerifier::default()).unwrap()
}
fn request<'binding>(
    fixture: &'binding OperatorRightFixture,
    registry: &mut OperatorRightRegistry,
) -> OperatorMembershipRequest<'binding> {
    let frozen = fixture.freeze();
    let branch = BranchContext::new([0x41; 32], 7).unwrap();
    let scope = RightScope::new(&frozen, branch).unwrap();
    let right = registry.issue(scope, frozen.frozen_bytes()).unwrap();
    OperatorMembershipRequest::new(frozen, right, operation(OperationId::AnnounceMaturity)).unwrap()
}

#[test]
fn verified_member_carries_operation_and_exact_producer_run() {
    let fixture = fixture();
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    let witness = produce_operator_membership(
        &reviewed_target(),
        &mapping(&fixture),
        request,
        authorized(&fixture),
        &mut registry,
        "fixture-7",
    )
    .unwrap();
    assert_eq!(witness.operation(), OperationId::AnnounceMaturity);
    assert_eq!(witness.disposition(), OperatorMembershipDisposition::Member);
    assert_eq!(
        witness.provenance(),
        "transaction::produce_operator_membership run fixture-7"
    );
    assert_eq!(
        registry.record().entries().last().unwrap().event,
        NonEquivocationEvent::Signed
    );
}

#[test]
fn a_different_model_key_is_non_member_only_after_verified_signing() {
    let fixture = fixture();
    let other = OperatorRightFixture::new(0x11, [0x21; 32], 0x44);
    let mapping = mapping(&other);
    assert_eq!(
        mapping.check(TargetContractVersion::V2, &fixture.binding),
        Err(OperatorMappingRefusal::Key)
    );
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    let witness = produce_operator_membership(
        &reviewed_target(),
        &mapping,
        request,
        authorized(&fixture),
        &mut registry,
        "key-refusal",
    )
    .unwrap();
    assert_eq!(
        witness.disposition(),
        OperatorMembershipDisposition::NonMember
    );
    assert_eq!(
        registry.record().entries().last().unwrap().event,
        NonEquivocationEvent::Signed
    );
}

#[test]
fn another_model_deployment_is_non_member() {
    let fixture = fixture();
    let other = OperatorRightFixture::new(0x12, [0x21; 32], 0x33);
    let mapping = mapping(&other);
    assert_eq!(
        mapping.check(TargetContractVersion::V2, &fixture.binding),
        Err(OperatorMappingRefusal::Deployment)
    );
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    let witness = produce_operator_membership(
        &reviewed_target(),
        &mapping,
        request,
        authorized(&fixture),
        &mut registry,
        "deployment-refusal",
    )
    .unwrap();
    assert_eq!(
        witness.disposition(),
        OperatorMembershipDisposition::NonMember
    );
}

#[test]
fn mapping_rechecks_the_established_revision() {
    let fixture = fixture();
    assert_eq!(
        mapping(&fixture).check(TargetContractVersion::V1, &fixture.binding),
        Err(OperatorMappingRefusal::Revision)
    );
    assert_eq!(
        mapping(&fixture).check(TargetContractVersion::V2, &fixture.binding),
        Ok(())
    );
}

#[test]
fn mapping_requires_exactly_one_operation() {
    let fixture = fixture();
    assert!(matches!(
        OperatorMembershipMapping::new(
            fixture.binding.clone(),
            fixture.binding.profile().clone(),
            RealizationScope::phase1_pilots()
        ),
        Err(OperatorMembershipRefusal::OperationScope)
    ));
}

#[test]
fn request_requires_exactly_one_operation() {
    let fixture = fixture();
    let frozen = fixture.freeze();
    let branch = BranchContext::new([0x41; 32], 7).unwrap();
    let scope = RightScope::new(&frozen, branch).unwrap();
    let mut registry = OperatorRightRegistry::default();
    let right = registry.issue(scope, frozen.frozen_bytes()).unwrap();
    assert!(matches!(
        OperatorMembershipRequest::new(frozen, right, RealizationScope::phase1_pilots()),
        Err(OperatorMembershipRefusal::OperationScope)
    ));
}

#[test]
fn authorization_assigned_to_another_operation_is_refused() {
    let fixture = fixture();
    let mapping = OperatorMembershipMapping::new(
        fixture.binding.clone(),
        fixture.binding.profile().clone(),
        operation(OperationId::Cycle),
    )
    .unwrap();
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    assert!(matches!(
        produce_operator_membership(
            &reviewed_target(),
            &mapping,
            request,
            authorized(&fixture),
            &mut registry,
            "wrong-operation"
        ),
        Err(OperatorMembershipRefusal::OperationMismatch)
    ));
    assert_eq!(registry.record().entries().len(), 1);
}

#[test]
fn authorization_from_another_deployment_emits_no_decision() {
    let fixture = fixture();
    let other = OperatorRightFixture::new(0x12, [0x21; 32], 0x44);
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    assert!(matches!(
        produce_operator_membership(
            &reviewed_target(),
            &mapping(&fixture),
            request,
            authorized(&other),
            &mut registry,
            "wrong-authorization"
        ),
        Err(OperatorMembershipRefusal::AuthorizationMismatch)
    ));
    assert_eq!(registry.record().entries().len(), 1);
}

#[test]
fn another_frozen_message_cannot_reuse_the_operation_assignment() {
    let fixture = fixture();
    let candidate = fixture.finalized.protected();
    let changed = crate::bytes::TargetTransaction::with_output_witnesses(
        candidate.version(),
        candidate.inputs().to_vec(),
        candidate.outputs().to_vec(),
        candidate.lock_time() + 1,
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
    .unwrap();
    let frozen = fixture.freeze_candidate(changed);
    let response = fixture.response(&frozen);
    let authorized =
        authorize_operator(frozen, [response], &OperatorRightVerifier::default()).unwrap();
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    assert!(matches!(
        produce_operator_membership(
            &reviewed_target(),
            &mapping(&fixture),
            request,
            authorized,
            &mut registry,
            "other-message"
        ),
        Err(OperatorMembershipRefusal::AuthorizationMismatch)
    ));
}

#[test]
fn rights_for_another_deployment_or_predecessor_are_refused() {
    let fixture = fixture();
    let other = OperatorRightFixture::new(0x12, [0x21; 32], 0x33);
    let candidate = fixture.finalized.protected();
    let changed = crate::bytes::TargetTransaction::with_output_witnesses(
        candidate.version(),
        vec![crate::bytes::TargetInput::new(
            outpoint(0xa2, 0),
            candidate.inputs()[0].sequence(),
        )],
        candidate.outputs().to_vec(),
        candidate.lock_time(),
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
    .unwrap();
    for frozen in [other.freeze(), fixture.freeze_candidate(changed)] {
        let mut registry = OperatorRightRegistry::default();
        let scope = RightScope::new(&frozen, BranchContext::new([0x41; 32], 7).unwrap()).unwrap();
        let right = registry.issue(scope, frozen.frozen_bytes()).unwrap();
        assert!(matches!(
            OperatorMembershipRequest::new(
                fixture.freeze(),
                right,
                operation(OperationId::AnnounceMaturity)
            ),
            Err(OperatorMembershipRefusal::RightScope)
        ));
    }
}

#[test]
fn empty_run_emits_no_decision_and_does_not_sign_the_registry() {
    let fixture = fixture();
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    assert!(matches!(
        produce_operator_membership(
            &reviewed_target(),
            &mapping(&fixture),
            request,
            authorized(&fixture),
            &mut registry,
            " \n\t"
        ),
        Err(OperatorMembershipRefusal::EmptyRun)
    ));
    assert_eq!(registry.record().entries().len(), 1);
}

#[test]
fn a_reorganized_right_emits_no_decision() {
    let fixture = fixture();
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    registry.reorganize(&BranchContext::new([0x41; 32], 7).unwrap());
    let refusal = produce_operator_membership(
        &reviewed_target(),
        &mapping(&fixture),
        request,
        authorized(&fixture),
        &mut registry,
        "stale-right",
    )
    .unwrap_err();
    assert!(matches!(refusal, OperatorMembershipRefusal::Right(failure)
        if matches!(failure.refusal, RightRefusal::StaleBranch(_))));
}

#[test]
fn registry_checks_candidate_bytes_even_when_scope_matches() {
    let fixture = fixture();
    let frozen = fixture.freeze();
    let scope = RightScope::new(&frozen, BranchContext::new([0x41; 32], 7).unwrap()).unwrap();
    let mut registry = OperatorRightRegistry::default();
    let right = registry.issue(scope, b"different candidate bytes").unwrap();
    let request =
        OperatorMembershipRequest::new(frozen, right, operation(OperationId::AnnounceMaturity))
            .unwrap();
    let refusal = produce_operator_membership(
        &reviewed_target(),
        &mapping(&fixture),
        request,
        authorized(&fixture),
        &mut registry,
        "wrong-right-bytes",
    )
    .unwrap_err();
    assert!(matches!(refusal, OperatorMembershipRefusal::Right(failure)
        if matches!(failure.refusal, RightRefusal::MismatchedBytes { .. })));
}

#[test]
fn membership_consumption_keeps_the_existing_retry_cache() {
    let fixture = fixture();
    let frozen = fixture.freeze();
    let scope = RightScope::new(&frozen, BranchContext::new([0x41; 32], 7).unwrap()).unwrap();
    let mut registry = OperatorRightRegistry::default();
    let request = request(&fixture, &mut registry);
    produce_operator_membership(
        &reviewed_target(),
        &mapping(&fixture),
        request,
        authorized(&fixture),
        &mut registry,
        "cached-run",
    )
    .unwrap();
    assert!(matches!(
        registry.retry(&scope, frozen.frozen_bytes()).unwrap(),
        crate::OperatorRightOutcome::Cached(_)
    ));
    assert!(matches!(
        registry.issue(scope, frozen.frozen_bytes()),
        Err(RightRefusal::Consumed(_))
    ));
}
