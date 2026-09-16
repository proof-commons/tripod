//! Scoped authority checks using the shared public-data operator double.

use std::cell::Cell;

use linker::CandidateDeploymentIdentity;
use tapscript::StateConstructorGeneration;

use super::live_support::{OperatorRightFixture, OperatorRightVerifier};
use super::outpoint;
use crate::bytes::{AssetId, OUTPOINT_INDEX_MASK, Outpoint, TargetInput, TargetTransaction};
use crate::operator_right::{
    BranchContext, ConstructionRight, NonEquivocationEvent, OperatorRightOutcome,
    OperatorRightRegistry, RightRefusal, RightScope, StateCheckpointPolicy, StateContinuation,
    StateContinuationStanding, StateContinuityEvidence, StateThreadAnchor,
    StateThreadContinuations, StateThreadOrigin, StateThreadProvenance, StateThreadRefusal,
};
use crate::operator_signing::{
    OperatorSigningRefusal, OperatorSigningRequest, authorize_operator,
    authorize_operator_under_right,
};
use crate::taproot::tagged_hash;

fn fixture() -> OperatorRightFixture {
    OperatorRightFixture::new(0x11, [0x21; 32], 0x33)
}
fn branch() -> BranchContext {
    BranchContext::new([0x41; 32], 7).expect("nonzero branch")
}
fn scope(request: &OperatorSigningRequest<'_>) -> RightScope {
    RightScope::new(request, branch()).expect("frozen predecessor")
}
fn issue(
    registry: &mut OperatorRightRegistry,
    request: &OperatorSigningRequest<'_>,
) -> ConstructionRight {
    registry
        .issue(scope(request), request.frozen_bytes())
        .expect("first issuance")
}
fn signed(registry: &mut OperatorRightRegistry, fixture: &OperatorRightFixture) -> RightScope {
    let request = fixture.freeze();
    let scoped = scope(&request);
    let right = issue(registry, &request);
    let answer = fixture.response(&request);
    authorize_operator_under_right(
        registry,
        right,
        request,
        [answer],
        &OperatorRightVerifier::default(),
    )
    .expect("one authorization");
    scoped
}
fn changed(fixture: &OperatorRightFixture, predecessor: bool) -> TargetTransaction {
    let candidate = fixture.finalized.protected();
    let inputs = if predecessor {
        vec![TargetInput::new(
            outpoint(0xa2, 0),
            candidate.inputs()[0].sequence(),
        )]
    } else {
        candidate.inputs().to_vec()
    };
    TargetTransaction::with_output_witnesses(
        candidate.version(),
        inputs,
        candidate.outputs().to_vec(),
        candidate.lock_time() + 1,
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
    .expect("preserved cardinalities")
}

#[test]
fn branch_constructor_and_readers() {
    assert_eq!(
        BranchContext::new([0; 32], 9),
        Err(RightRefusal::ZeroBranchIdentifier { checkpoint: 9 })
    );
    assert_eq!(branch().identifier(), &[0x41; 32]);
    assert_eq!(branch().checkpoint(), 7);
}

#[test]
fn issuance_succeeds_once_and_carries_its_identity() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    assert_eq!(right.scope(), &scope(&request));
    assert_eq!(right.ordinal(), 0);
    assert!(matches!(
        registry.issue(scope(&request), request.frozen_bytes()),
        Err(RightRefusal::Outstanding(_))
    ));
    assert_eq!(registry.record().entries().len(), 1);
}

#[test]
fn dropping_an_outstanding_token_does_not_reopen_issuance() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    drop(issue(&mut registry, &request));
    assert!(matches!(
        registry.issue(scope(&request), request.frozen_bytes()),
        Err(RightRefusal::Outstanding(_))
    ));
}

#[test]
fn a_consumed_scope_cannot_be_reacquired() {
    let fixture = fixture();
    let mut registry = OperatorRightRegistry::default();
    let scoped = signed(&mut registry, &fixture);
    assert!(matches!(
        registry.issue(scoped, fixture.freeze().frozen_bytes()),
        Err(RightRefusal::Consumed(_))
    ));
}

#[test]
fn identical_retry_is_owned_and_invokes_no_second_signing_action() {
    let fixture = fixture();
    let request = fixture.freeze();
    let bytes = request.frozen_bytes().to_vec();
    let scoped = scope(&request);
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    let answer = fixture.response(&request);
    let verifier = OperatorRightVerifier::default();
    let attempts = Cell::new(0);
    let fresh = registry
        .consume(right, request, |request| {
            attempts.set(attempts.get() + 1);
            authorize_operator(request, [answer], &verifier)
        })
        .expect("fresh authorization");
    let OperatorRightOutcome::Fresh(authorized) = fresh else {
        panic!("fresh required")
    };
    let expected = authorized.witness().clone();
    let standing = authorized.standing().clone();
    drop(authorized);
    drop(fixture);
    for _ in 0..3 {
        let OperatorRightOutcome::Cached(cached) = registry.retry(&scoped, &bytes).expect("cached")
        else {
            panic!("cached required")
        };
        assert_eq!(cached.witness, expected);
        assert_eq!(cached.standing, standing);
        assert_eq!(cached.right.scope.as_ref(), &scoped);
        assert_eq!(cached.right.ordinal, 0);
        assert_eq!(
            cached.digest,
            tagged_hash("operator-right/candidate", &bytes)
        );
    }
    assert_eq!(attempts.get(), 1);
    assert_eq!(verifier.calls.get(), 1);
}

#[test]
fn competing_retry_has_its_own_refusal() {
    let fixture = fixture();
    let mut registry = OperatorRightRegistry::default();
    let scoped = signed(&mut registry, &fixture);
    let mut bytes = fixture.freeze().frozen_bytes().to_vec();
    bytes[0] ^= 1;
    assert!(matches!(
        registry.retry(&scoped, &bytes),
        Err(RightRefusal::CompetingCandidate { .. })
    ));
    assert_eq!(registry.record().entries().len(), 2);
}

#[test]
fn replacement_changes_bound_bytes_before_signing() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    let replacement = fixture.freeze_candidate(changed(&fixture, false));
    let right = registry
        .replace(right, replacement.frozen_bytes())
        .expect("unsigned replacement");
    let failure = registry
        .consume(right, request, |_| panic!("old bytes must not sign"))
        .expect_err("old bytes");
    assert!(matches!(
        failure.refusal,
        RightRefusal::MismatchedBytes { .. }
    ));
    let answer = fixture.response(&replacement);
    authorize_operator_under_right(
        &mut registry,
        failure.right,
        replacement,
        [answer],
        &OperatorRightVerifier::default(),
    )
    .expect("new bytes authorize");
    assert_eq!(registry.record().entries().len(), 3);
    assert_eq!(
        registry.record().entries()[1].event,
        NonEquivocationEvent::Replaced
    );
}

#[test]
fn unchanged_replacement_is_a_noop() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    let right = registry
        .replace(right, request.frozen_bytes())
        .expect("no change");
    assert_eq!(right.ordinal(), 0);
    assert_eq!(registry.record().entries().len(), 1);
}

#[test]
fn terminal_state_refusals_precede_a_foreign_token_check() {
    // A successful consume leaves no token. Another registry's token cannot
    // bypass the terminal scope check, and does not reopen that consumed right.
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    signed(&mut registry, &fixture);
    let mut foreign = OperatorRightRegistry::default();
    let right = issue(&mut foreign, &request);
    let failure = registry
        .replace(right, request.frozen_bytes())
        .expect_err("signed replacement");
    assert!(matches!(
        failure.refusal,
        RightRefusal::ReplacementAfterSigning(_)
    ));
    let failure = registry
        .consume(failure.right, request, |_| panic!("consumed"))
        .expect_err("consumed");
    assert!(matches!(failure.refusal, RightRefusal::Consumed(_)));
}

#[test]
fn dispatch_timeout_never_reopens_any_route() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let scoped = signed(&mut registry, &fixture);
    registry.dispatch(&scoped).expect("signed dispatch");
    registry.timeout(&scoped).expect("timeout");
    assert!(matches!(
        registry.issue(scoped.clone(), request.frozen_bytes()),
        Err(RightRefusal::Indeterminate(_))
    ));
    assert!(matches!(
        registry.retry(&scoped, request.frozen_bytes()),
        Err(RightRefusal::Indeterminate(_))
    ));
    assert!(matches!(
        registry.dispatch(&scoped),
        Err(RightRefusal::Indeterminate(_))
    ));
    assert!(matches!(
        registry.timeout(&scoped),
        Err(RightRefusal::Indeterminate(_))
    ));
    let mut foreign = OperatorRightRegistry::default();
    let right = issue(&mut foreign, &request);
    let failure = registry
        .replace(right, request.frozen_bytes())
        .expect_err("indeterminate");
    assert!(matches!(failure.refusal, RightRefusal::Indeterminate(_)));
    let failure = registry
        .consume(failure.right, request, |_| panic!("indeterminate"))
        .expect_err("indeterminate");
    assert!(matches!(failure.refusal, RightRefusal::Indeterminate(_)));
    assert_eq!(registry.record().entries().len(), 4);
}

#[test]
fn unsigned_dispatch_retry_and_timeout_are_typed_refusals() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    assert!(matches!(
        registry.dispatch(right.scope()),
        Err(RightRefusal::NotSigned(_))
    ));
    assert!(matches!(
        registry.retry(right.scope(), request.frozen_bytes()),
        Err(RightRefusal::NotSigned(_))
    ));
    assert!(matches!(
        registry.timeout(right.scope()),
        Err(RightRefusal::NotDispatched(_))
    ));
    assert_eq!(registry.record().entries().len(), 1);
}

#[test]
fn signed_timeout_requires_dispatch_and_repeated_dispatch_is_a_noop() {
    let fixture = fixture();
    let mut registry = OperatorRightRegistry::default();
    let scoped = signed(&mut registry, &fixture);
    assert!(matches!(
        registry.timeout(&scoped),
        Err(RightRefusal::NotDispatched(_))
    ));
    registry.dispatch(&scoped).expect("dispatch");
    registry.dispatch(&scoped).expect("already dispatched");
    assert!(matches!(
        registry.retry(&scoped, fixture.freeze().frozen_bytes()),
        Ok(OperatorRightOutcome::Cached(_))
    ));
    assert_eq!(registry.record().entries().len(), 4);
}

#[test]
fn reorganization_invalidates_outstanding_right_and_requires_fresh_context() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    registry.reorganize(&branch());
    assert_eq!(
        registry
            .issue(scope(&request), request.frozen_bytes())
            .expect_err("stale"),
        RightRefusal::StaleBranch(branch())
    );
    let failure = registry
        .replace(right, request.frozen_bytes())
        .expect_err("stale replacement");
    assert_eq!(failure.refusal, RightRefusal::StaleBranch(branch()));
    let failure = registry
        .consume(failure.right, request, |_| panic!("stale"))
        .expect_err("stale consume");
    assert_eq!(failure.refusal, RightRefusal::StaleBranch(branch()));
    let fresh = BranchContext::new([0x42; 32], 8).expect("new evidence context");
    let request = fixture.freeze();
    let right = registry
        .issue(
            RightScope::new(&request, fresh).expect("scope"),
            request.frozen_bytes(),
        )
        .expect("fresh context");
    assert_eq!(right.ordinal(), 1);
    assert_eq!(registry.record().entries().len(), 3);
}

#[test]
fn reorganization_discards_all_caches_and_tombstones_unissued_scopes() {
    let fixture = fixture();
    let other = OperatorRightFixture::new(0x12, [0x21; 32], 0x33);
    let mut registry = OperatorRightRegistry::default();
    let scoped = signed(&mut registry, &fixture);
    let other_scope = signed(&mut registry, &other);
    registry.dispatch(&other_scope).expect("dispatch");
    registry.timeout(&other_scope).expect("timeout");
    registry.reorganize(&branch());
    let count = registry.record().entries().len();
    registry.reorganize(&branch());
    assert_eq!(registry.record().entries().len(), count);
    assert_eq!(count, 8);
    for scoped in [scoped, other_scope] {
        assert_eq!(
            registry.retry(&scoped, &[]),
            Err(RightRefusal::StaleBranch(branch()))
        );
        assert_eq!(
            registry.dispatch(&scoped),
            Err(RightRefusal::StaleBranch(branch()))
        );
        assert_eq!(
            registry.timeout(&scoped),
            Err(RightRefusal::StaleBranch(branch()))
        );
    }
    let unissued = OperatorRightFixture::new(0x13, [0x21; 32], 0x33);
    let request = unissued.freeze();
    assert_eq!(
        registry
            .issue(scope(&request), request.frozen_bytes())
            .expect_err("tombstone"),
        RightRefusal::StaleBranch(branch())
    );
}

#[test]
fn unrelated_branch_survives_reorganization() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    registry.reorganize(&BranchContext::new([0x42; 32], 7).expect("other branch"));
    let answer = fixture.response(&request);
    authorize_operator_under_right(
        &mut registry,
        right,
        request,
        [answer],
        &OperatorRightVerifier::default(),
    )
    .expect("unaffected");
}

#[test]
fn signing_refusal_returns_the_outstanding_token_for_another_attempt() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    let failure = authorize_operator_under_right(
        &mut registry,
        right,
        request,
        [],
        &OperatorRightVerifier::default(),
    )
    .expect_err("no response");
    assert_eq!(
        failure.refusal,
        RightRefusal::Signing(Box::new(OperatorSigningRefusal::MissingResponse {
            input_index: 0
        }))
    );
    assert_eq!(
        registry.record().entries()[1].event,
        NonEquivocationEvent::SigningRefused
    );
    let request = fixture.freeze();
    assert!(matches!(
        registry.issue(scope(&request), request.frozen_bytes()),
        Err(RightRefusal::Outstanding(_))
    ));
    let answer = fixture.response(&request);
    authorize_operator_under_right(
        &mut registry,
        failure.right,
        request,
        [answer],
        &OperatorRightVerifier::default(),
    )
    .expect("retry succeeds");
    assert_eq!(registry.record().entries().len(), 3);
}

#[test]
fn foreign_tokens_cannot_consume_or_replace_an_outstanding_right() {
    let fixture = fixture();
    let request = fixture.freeze();
    let mut registry = OperatorRightRegistry::default();
    let mut foreign = OperatorRightRegistry::default();
    let local = issue(&mut registry, &request);
    let right = issue(&mut foreign, &request);
    let failure = registry
        .replace(right, request.frozen_bytes())
        .expect_err("foreign");
    assert!(matches!(failure.refusal, RightRefusal::ForeignRight(_)));
    let failure = registry
        .consume(failure.right, request, |_| panic!("foreign"))
        .expect_err("foreign");
    assert!(matches!(failure.refusal, RightRefusal::ForeignRight(_)));
    assert_eq!(local.ordinal(), 0);
    assert_eq!(registry.record().entries().len(), 1);
}

#[test]
fn unknown_scopes_have_no_right() {
    let fixture = fixture();
    let request = fixture.freeze();
    let scoped = scope(&request);
    let mut registry = OperatorRightRegistry::default();
    let refusal = RightRefusal::NoRight(Box::new(scoped.clone()));
    assert_eq!(
        registry.retry(&scoped, request.frozen_bytes()),
        Err(refusal.clone())
    );
    assert_eq!(registry.dispatch(&scoped), Err(refusal.clone()));
    assert_eq!(registry.timeout(&scoped), Err(refusal.clone()));
    let mut foreign = OperatorRightRegistry::default();
    let right = issue(&mut foreign, &request);
    let failure = registry
        .replace(right, request.frozen_bytes())
        .expect_err("no right");
    assert_eq!(failure.refusal, refusal);
    let failure = registry
        .consume(failure.right, request, |_| panic!("no right"))
        .expect_err("no right");
    assert_eq!(failure.refusal, refusal);
}

#[test]
fn scope_compares_every_reachable_component() {
    let fixture = fixture();
    let request = fixture.freeze();
    let scoped = scope(&request);
    assert_eq!(scoped, scope(&fixture.freeze()));
    assert_eq!(scoped.deployment(), fixture.binding.deployment());
    assert_eq!(
        scoped.predecessor(),
        request.predecessor_outpoint().expect("input")
    );
    assert_eq!(scoped.operator(), fixture.binding.key().bytes());
    assert_eq!(
        scoped.capability_revision(),
        fixture.binding.capability_revision()
    );
    assert_eq!(scoped.branch(), &branch());
    for context in [
        BranchContext::new([0x42; 32], 7),
        BranchContext::new([0x41; 32], 8),
    ] {
        assert_ne!(
            scoped,
            RightScope::new(&request, context.expect("context")).expect("scope")
        );
    }
    for other in [
        OperatorRightFixture::new(0x12, [0x21; 32], 0x33),
        OperatorRightFixture::new(0x11, [0x22; 32], 0x33),
        OperatorRightFixture::new(0x11, [0x21; 32], 0x34),
    ] {
        assert_ne!(scoped, scope(&other.freeze()));
    }
    assert_ne!(
        scoped,
        scope(&fixture.freeze_candidate(changed(&fixture, true)))
    );
    assert_eq!(
        scoped,
        scope(&fixture.freeze_candidate(changed(&fixture, false)))
    );
}

#[test]
fn scope_mismatch_is_refused_before_the_signing_callback() {
    let fixture = fixture();
    let request = fixture.freeze();
    let other = fixture.freeze_candidate(changed(&fixture, true));
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    let right = registry
        .replace(right, other.frozen_bytes())
        .expect("bytes replacement");
    let failure = registry
        .consume(right, other, |_| panic!("wrong predecessor"))
        .expect_err("scope mismatch");
    assert!(matches!(
        failure.refusal,
        RightRefusal::ScopeMismatch { .. }
    ));
}

#[test]
fn a_callback_substituting_an_authorized_candidate_cannot_reopen_the_right() {
    let fixture = fixture();
    let request = fixture.freeze();
    let other = fixture.freeze_candidate(changed(&fixture, false));
    let answer = fixture.response(&other);
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &request);
    let failure = registry
        .consume(right, request, |_| {
            authorize_operator(other, [answer], &OperatorRightVerifier::default())
        })
        .expect_err("callback substitution");
    assert!(matches!(
        failure.refusal,
        RightRefusal::UnexpectedAuthorization(_)
    ));
    assert!(matches!(
        registry.issue(failure.right.scope().clone(), &[]),
        Err(RightRefusal::Indeterminate(_))
    ));
    assert_eq!(
        registry.record().entries()[1].event,
        NonEquivocationEvent::UnexpectedAuthorization
    );
}

#[test]
fn record_contains_one_signature_free_entry_per_observed_action() {
    let fixture = fixture();
    let request = fixture.freeze();
    let bytes = request.frozen_bytes().to_vec();
    let mut registry = OperatorRightRegistry::default();
    let scoped = signed(&mut registry, &fixture);
    registry.retry(&scoped, &bytes).expect("cache read");
    registry.dispatch(&scoped).expect("dispatch");
    registry.timeout(&scoped).expect("timeout");
    registry.reorganize(&branch());
    let expected = [
        NonEquivocationEvent::Issued,
        NonEquivocationEvent::Signed,
        NonEquivocationEvent::Cached,
        NonEquivocationEvent::Dispatched,
        NonEquivocationEvent::TimedOut,
        NonEquivocationEvent::Invalidated,
    ];
    assert_eq!(registry.record().entries().len(), expected.len());
    for (entry, event) in registry.record().entries().iter().zip(expected) {
        // Exhaustive destructuring pins the record's signature-free field census.
        let crate::NonEquivocationEntry {
            right,
            event: observed,
            bytes_len,
            digest,
        } = entry;
        assert_eq!(observed, &event);
        assert_eq!(right.scope.as_ref(), &scoped);
        assert_eq!(right.ordinal, 0);
        assert_eq!(*bytes_len, bytes.len());
        assert_eq!(*digest, tagged_hash("operator-right/candidate", &bytes));
    }
}

fn anchor_identity(
    context: ([u8; 32], u64),
    deployment: (u8, u8),
    starting: Outpoint,
    provenance: StateThreadProvenance,
) -> StateThreadAnchor {
    StateThreadAnchor::new(
        CandidateDeploymentIdentity::new([deployment.0; 32], [deployment.1; 32])
            .expect("nonzero deployment"),
        context,
        starting,
        provenance,
        StateConstructorGeneration::CanonicalMetadataV1,
        StateCheckpointPolicy::ExactBranch,
        StateThreadOrigin::Synthetic,
    )
    .expect("nonzero branch")
}
fn anchor() -> StateThreadAnchor {
    anchor_identity(
        ([0x41; 32], 7),
        (0x11, 0x22),
        outpoint(0x31, 0),
        StateThreadProvenance::ExistingAsset(AssetId::from_internal([0x51; 32])),
    )
}
fn anchor_policy(policy: StateCheckpointPolicy, origin: StateThreadOrigin) -> StateThreadAnchor {
    let base = anchor();
    StateThreadAnchor::new(
        base.deployment().clone(),
        (*base.branch().identifier(), base.branch().checkpoint()),
        base.starting_outpoint(),
        base.provenance(),
        base.generation(),
        policy,
        origin,
    )
    .expect("checked anchor shape")
}
fn continuation(identifier: u8, checkpoint: u64, output: u8) -> StateContinuation {
    StateContinuation {
        branch: BranchContext::new([identifier; 32], checkpoint).expect("nonzero branch"),
        successor: outpoint(output, 0),
        standing: StateContinuationStanding::Accepted,
    }
}

#[test]
fn anchor_refuses_zero_branch_through_the_existing_context_check() {
    let base = anchor();
    assert_eq!(
        StateThreadAnchor::new(
            base.deployment().clone(),
            ([0; 32], 7),
            base.starting_outpoint(),
            base.provenance(),
            base.generation(),
            base.checkpoint_policy(),
            base.origin(),
        ),
        Err(StateThreadRefusal::ZeroBranchIdentifier { checkpoint: 7 })
    );
}

#[test]
fn anchor_outpoint_type_refuses_maximum_integer_but_has_no_separate_null_sentinel() {
    let base = anchor();
    assert_eq!(
        Outpoint::new(base.starting_outpoint().txid(), u32::MAX),
        Err(crate::error::TransactionRefusal::OutpointIndexOutOfRange { offered: u32::MAX })
    );
    let boundary = outpoint(0x31, OUTPOINT_INDEX_MASK);
    let checked = anchor_identity(([0x41; 32], 7), (0x11, 0x22), boundary, base.provenance());
    assert_eq!(checked.starting_outpoint(), boundary);
}

#[test]
fn anchor_branch_identifier_and_checkpoint_are_distinct_and_visible() {
    let base = anchor();
    for context in [([0x42; 32], 7), ([0x41; 32], 8)] {
        let other = anchor_identity(
            context,
            (0x11, 0x22),
            base.starting_outpoint(),
            base.provenance(),
        );
        assert_ne!(base, other);
        assert_eq!(other.branch().identifier(), &context.0);
        assert_eq!(other.branch().checkpoint(), context.1);
    }
}

#[test]
fn anchor_network_and_genesis_identity_are_separately_distinct_and_visible() {
    let base = anchor();
    for deployment in [(0x12, 0x22), (0x11, 0x23)] {
        let other = anchor_identity(
            ([0x41; 32], 7),
            deployment,
            base.starting_outpoint(),
            base.provenance(),
        );
        assert_ne!(base, other);
        assert_eq!(other.deployment().network_id(), &[deployment.0; 32]);
        assert_eq!(other.deployment().genesis_id(), &[deployment.1; 32]);
    }
}

#[test]
fn anchor_starting_transaction_and_index_are_separately_distinct_and_visible() {
    let base = anchor();
    for starting in [outpoint(0x32, 0), outpoint(0x31, 1)] {
        let other = anchor_identity(([0x41; 32], 7), (0x11, 0x22), starting, base.provenance());
        assert_ne!(base, other);
        assert_eq!(other.starting_outpoint(), starting);
    }
}

#[test]
fn anchor_asset_and_issuance_provenances_are_pairwise_distinct_and_visible() {
    let base = anchor();
    let provenances = [
        base.provenance(),
        StateThreadProvenance::ExistingAsset(AssetId::from_internal([0x52; 32])),
        StateThreadProvenance::Issuance(outpoint(0x61, 0)),
        StateThreadProvenance::Issuance(outpoint(0x61, 1)),
    ];
    for provenance in provenances {
        let checked = anchor_identity(
            ([0x41; 32], 7),
            (0x11, 0x22),
            base.starting_outpoint(),
            provenance,
        );
        assert_eq!(checked.provenance(), provenance);
        for other in provenances.into_iter().filter(|other| *other != provenance) {
            assert_ne!(
                checked,
                anchor_identity(
                    ([0x41; 32], 7),
                    (0x11, 0x22),
                    base.starting_outpoint(),
                    other
                )
            );
        }
    }
}

#[test]
fn anchor_generation_has_one_constructible_variant_so_only_accessor_is_testable() {
    assert_eq!(
        anchor().generation(),
        StateConstructorGeneration::CanonicalMetadataV1
    );
}

#[test]
fn observed_origin_is_admitted_with_outstanding_continuity_evidence() {
    let observed = anchor_policy(
        StateCheckpointPolicy::ExactBranch,
        StateThreadOrigin::Observed,
    );
    assert_ne!(observed, anchor());
    assert_eq!(observed.origin(), StateThreadOrigin::Observed);
    assert_eq!(
        observed.continuity_evidence(),
        StateContinuityEvidence::Outstanding
    );
    assert_eq!(
        observed.check_continuations(&[]).expect("empty").origin(),
        StateThreadOrigin::Observed
    );
}

#[test]
fn checkpoint_policy_is_distinct_and_visible() {
    let changed = anchor_policy(
        StateCheckpointPolicy::AtOrBeyond(7),
        StateThreadOrigin::Synthetic,
    );
    assert_ne!(changed, anchor());
    assert_eq!(
        changed.checkpoint_policy(),
        StateCheckpointPolicy::AtOrBeyond(7)
    );
    assert_eq!(
        anchor().checkpoint_policy(),
        StateCheckpointPolicy::ExactBranch
    );
}

#[test]
fn one_accepted_continuation_is_returned_under_the_anchor_hypothesis() {
    let offered = continuation(0x41, 7, 0x71);
    assert_eq!(
        anchor()
            .check_continuations(&[offered])
            .expect("unique")
            .accepted(),
        &[offered]
    );
}

#[test]
fn two_accepted_continuations_on_one_identifier_refuse_even_at_different_checkpoints() {
    let first = continuation(0x41, 7, 0x71);
    for second in [
        continuation(0x41, 7, 0x72),
        continuation(0x41, 8, 0x72),
        first,
    ] {
        assert_eq!(
            anchor().check_continuations(&[first, second]),
            Err(StateThreadRefusal::Equivocation([0x41; 32]))
        );
    }
}

#[test]
fn accepted_continuations_on_two_selected_branches_are_returned_in_branch_order() {
    let checked = anchor_policy(
        StateCheckpointPolicy::AtOrBeyond(7),
        StateThreadOrigin::Synthetic,
    );
    let first = continuation(0x41, 7, 0x71);
    let second = continuation(0x42, 8, 0x72);
    for candidates in [[first, second], [second, first]] {
        assert_eq!(
            checked
                .check_continuations(&candidates)
                .expect("one per branch")
                .accepted(),
            &[first, second]
        );
    }
    assert_eq!(
        checked.check_continuations(&[first, continuation(0x41, 8, 0x72)]),
        Err(StateThreadRefusal::Equivocation([0x41; 32]))
    );
}

#[test]
fn nonaccepted_competitor_does_not_equivocate_or_appear_in_results() {
    let accepted = continuation(0x41, 7, 0x71);
    let rejected = StateContinuation {
        standing: StateContinuationStanding::NotAccepted,
        ..continuation(0x41, 7, 0x72)
    };
    for candidates in [[accepted, rejected], [rejected, accepted]] {
        assert_eq!(
            anchor()
                .check_continuations(&candidates)
                .expect("one accepted")
                .accepted(),
            &[accepted]
        );
    }
    assert_eq!(
        anchor()
            .check_continuations(&[rejected])
            .expect("none accepted")
            .accepted(),
        []
    );
    assert_eq!(
        anchor().check_continuations(&[]).expect("empty").accepted(),
        []
    );
}

#[test]
fn exact_branch_policy_refuses_foreign_branches_regardless_of_standing() {
    for standing in [
        StateContinuationStanding::Accepted,
        StateContinuationStanding::NotAccepted,
    ] {
        let offered = StateContinuation {
            standing,
            ..continuation(0x42, 99, 0x71)
        };
        assert_eq!(
            anchor().check_continuations(&[offered]),
            Err(StateThreadRefusal::OutsideCheckpointPolicy(offered.branch))
        );
    }
    let earlier = continuation(0x41, 0, 0x71);
    assert_eq!(
        anchor()
            .check_continuations(&[earlier])
            .expect("same identifier")
            .accepted(),
        &[earlier]
    );
}

#[test]
fn checkpoint_floor_selects_reorganized_branches_by_inclusive_ordinal() {
    let checked = anchor_policy(
        StateCheckpointPolicy::AtOrBeyond(7),
        StateThreadOrigin::Synthetic,
    );
    for ordinal in [7, 8, u64::MAX] {
        let offered = continuation(0x42, ordinal, 0x71);
        assert_eq!(
            checked
                .check_continuations(&[offered])
                .expect("at or beyond")
                .accepted(),
            &[offered]
        );
    }
    for standing in [
        StateContinuationStanding::Accepted,
        StateContinuationStanding::NotAccepted,
    ] {
        let offered = StateContinuation {
            standing,
            ..continuation(0x42, 6, 0x71)
        };
        assert_eq!(
            checked.check_continuations(&[offered]),
            Err(StateThreadRefusal::OutsideCheckpointPolicy(offered.branch))
        );
    }
}

#[test]
fn synthetic_anchor_and_positive_result_make_no_genesis_claim() {
    let synthetic = anchor();
    assert_eq!(synthetic.origin(), StateThreadOrigin::Synthetic);
    assert_eq!(
        synthetic.continuity_evidence(),
        StateContinuityEvidence::Outstanding
    );
    let result = synthetic
        .check_continuations(&[continuation(0x41, 7, 0x71)])
        .expect("conditional uniqueness");
    assert_eq!(result.origin(), StateThreadOrigin::Synthetic);
    assert_eq!(
        StateThreadContinuations::RESIDUAL,
        "conditional on a uniquely anchored predecessor; no genesis claim or global origin uniqueness; synthetic origin is not protocol genesis, trusted setup, earlier root history, or production STATE and authorizes nothing of value"
    );
}
