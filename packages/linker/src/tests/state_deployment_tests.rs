//! The maturity link's sources, bound from typed fixtures.
//!
//! # The sources are real ones
//!
//! The plan below is the compiler's own validated announcement plan,
//! reached through its public planning entry over a derived realization,
//! and the record is the composed announcement program built through
//! tapscript's public structural, semantic and operator entries. Nothing
//! here hand-assembles either one, because a hand-assembled source would
//! let the bridge bind something no compiler planned and no backend
//! composed, which is the one thing binding by type exists to prevent.
//!
//! # The fixture values say they are fixtures
//!
//! The asset, amount, operator key and identity bytes below are public,
//! meaningless test material `(´[ADR015-rule:security:test-material]´)`:
//! they carry no secret and stand for no deployed object. The lead
//! magnitudes 2 and 4 are the same kind of thing, and the binding says so
//! in the type rather than in a comment, which is what
//! [`StateLeadBoundOrigin::Fixture`] is for.
//!
//! # Raw bytes are refused by the compiler, not by a test
//!
//! That no source can arrive as bytes is a statement about types, so it
//! is checked where the type checker can fail it: the `compile_fail`
//! doctests on [`StateLinkDeploymentParameters::bind`] offer a byte
//! vector in place of the plan and a value map in place of the record. An
//! assertion here could only restate the claim in prose, and prose cannot
//! fail on the day a byte parameter is added.
//!
//! # Two arms of the reused check are not reachable from here
//!
//! [`crate::LinkRefusal::OperatorKeyMismatch`] cannot be raised through
//! the bridge, which offers the binding's own key, and
//! [`crate::LinkRefusal::InvalidOperatorProfile`] cannot be raised while
//! one reviewed definition exists, because the profile's pin and the
//! bridge's revision are both read from it. The second is exercised where
//! it is reachable: `check_refuses_stale_revision_v1` in the
//! operator-deployment tests offers the earlier revision to
//! [`crate::OperatorDeploymentBinding::check`] directly.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::maturity_announcement_plan::{
    ValidatedMaturityAnnouncementOperationPlan, plan_maturity_announcement_target_operation,
};
use compiler::operation_plan::PlacementSearchLimits;
use realization::{RealizationScope, derive};
use tapscript::upstream::{AnnouncementLeadBounds, Cycle};
use tapscript::{
    EstablishedOperatorProfile, OperatorKey, STATE_NUMS_KEY, StackItem, StateAnnouncementBindings,
    StateAnnouncementProgram, StateAnnouncementSymbol, StateExternalEvidenceRole,
    StateOperatorBindings, StateOperatorSymbol, StatePatternBindings, StatePatternSymbol,
    build_state_announcement_program, build_state_operator_pattern, operator_key_encoding_closure,
    selected_operator_profile, state_announcement_patterns, state_announcement_program,
    state_operator_fragment, state_structural_patterns,
};
use target_elements::{EncodingClass, TargetContractVersion};

use crate::tests::reviewed_target;
use crate::{
    CandidateDeploymentIdentity, LinkRefusal, OperatorDeploymentBinding, StateLeadBoundOrigin,
    StateLeadBounds, StateLinkDeploymentParameters,
};

/// The validated announcement plan, derived once and handed out by clone.
fn plan() -> ValidatedMaturityAnnouncementOperationPlan {
    static PLAN: LazyLock<ValidatedMaturityAnnouncementOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let operations = [
            OperationId::AnnounceMaturity,
            OperationId::CompactAsh,
            OperationId::TransferLive,
        ];
        let scope = RealizationScope::from_operations(operations).expect("a three-operation scope");
        let realization = derive(&ARCHITECTURE, scope).expect("the operations derive");
        let scope = CompilationScope::from_operations([OperationId::AnnounceMaturity])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_maturity_announcement_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// The composed announcement record, built once and handed out by clone.
fn record() -> StateAnnouncementProgram {
    static RECORD: LazyLock<StateAnnouncementProgram> = LazyLock::new(|| {
        let target = reviewed_target();
        let item = |bytes| StackItem::new(&target, bytes).expect("fixture bytes are a stack item");

        let structural = StatePatternBindings::new(
            &target,
            BTreeMap::from([
                (StatePatternSymbol::StateAsset, item(vec![0x11; 32])),
                (
                    StatePatternSymbol::StateAmount,
                    StackItem::signed_le64(&target, 1),
                ),
            ]),
        )
        .expect("the structural census is complete");
        let structural =
            state_structural_patterns(&target, &structural).expect("the structural recipe builds");

        let semantic = StateAnnouncementBindings::new(
            &target,
            BTreeMap::from([
                (
                    StateAnnouncementSymbol::InternalKey,
                    item(STATE_NUMS_KEY.to_vec()),
                ),
                (
                    StateAnnouncementSymbol::MaturityLeadMin,
                    StackItem::unsigned_le64(&target, 2),
                ),
                (
                    StateAnnouncementSymbol::MaturityLeadMax,
                    StackItem::unsigned_le64(&target, 4),
                ),
                (StateAnnouncementSymbol::StateAsset, item(vec![0x11; 32])),
                (
                    StateAnnouncementSymbol::StateAmount,
                    StackItem::signed_le64(&target, 1),
                ),
            ]),
        )
        .expect("the semantic census is complete");
        let semantic =
            state_announcement_patterns(&target, &semantic).expect("the semantic recipe builds");

        let operator = StateOperatorBindings::new(
            &target,
            &BTreeMap::from([(
                StateOperatorSymbol::CommittedOperatorKey,
                StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0x33; 32])
                    .expect("the fixture key has the reviewed width"),
            )]),
        )
        .expect("the operator census is complete");
        let fragment = state_operator_fragment(&operator).expect("the operator fragment builds");
        let operator = build_state_operator_pattern(&target, &operator, fragment)
            .expect("the operator pattern builds");

        let raw = state_announcement_program(&target, &structural, &semantic, &operator)
            .expect("the composed program assembles");
        build_state_announcement_program(&target, &structural, &semantic, &operator, raw)
            .expect("the composed record is admitted")
    });
    RECORD.clone()
}

/// The fixture lead window: test material standing for no deployment.
fn fixture_lead_bounds() -> StateLeadBounds {
    let bounds = AnnouncementLeadBounds::new(Cycle::new(2), Cycle::new(4))
        .expect("the fixture window is nonzero and ordered");
    StateLeadBounds::new(bounds, StateLeadBoundOrigin::Fixture)
}

// Public, meaningless fixture bytes: no party, real deployment, or secret.
fn operator_key(byte: u8) -> OperatorKey {
    let target = reviewed_target();
    let closure = operator_key_encoding_closure(target.definition().authorization());
    OperatorKey::new(&closure, closure.approved(), vec![byte; 32])
        .expect("fixture public bytes have the approved shape")
}

fn identity(network: u8, genesis: u8) -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new([network; 32], [genesis; 32])
        .expect("fixture identifiers are nonzero")
}

fn binding() -> OperatorDeploymentBinding {
    let target = reviewed_target();
    let internal_key = StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0xb6; 32])
        .expect("fixture internal key has the reviewed width");
    let profile = EstablishedOperatorProfile::establish(selected_operator_profile(), &target)
        .expect("the reviewed target establishes the source selection");
    OperatorDeploymentBinding::bind(
        &target,
        operator_key(0x33),
        profile,
        identity(0x11, 0x22),
        &internal_key,
    )
    .expect("the candidate deployment binds")
}

fn depth() -> NonZeroU32 {
    NonZeroU32::new(8).expect("the fixture depth is nonzero")
}

fn bridge() -> StateLinkDeploymentParameters {
    StateLinkDeploymentParameters::bind(
        &reviewed_target(),
        plan(),
        fixture_lead_bounds(),
        identity(0x11, 0x22),
        binding(),
        depth(),
        &record(),
    )
    .expect("the demonstration sources bind")
}

#[test]
fn binding_returns_every_supplied_source_and_the_reviewed_revision() {
    let bound = bridge();
    assert_eq!(bound.plan(), &plan());
    assert_eq!(bound.lead_bounds(), fixture_lead_bounds());
    assert_eq!(bound.lead_bounds().origin(), StateLeadBoundOrigin::Fixture);
    assert_eq!(bound.lead_bounds().bounds().minimum(), Cycle::new(2));
    assert_eq!(bound.lead_bounds().bounds().maximum(), Cycle::new(4));
    assert_eq!(bound.identity(), &identity(0x11, 0x22));
    assert_eq!(bound.operator(), &binding());
    assert_eq!(bound.maximum_control_path_depth(), depth());
    assert_eq!(bound.revision(), reviewed_target().definition().version());
    assert_eq!(bound.revision(), TargetContractVersion::V2);
    assert_eq!(bound.operator().capability_revision(), bound.revision());
}

#[test]
fn the_deployment_facts_are_exactly_the_three_deployment_side_roles() {
    assert_eq!(
        bridge().deployment_facts(),
        &BTreeSet::from([
            StateExternalEvidenceRole::SubstrateConservation,
            StateExternalEvidenceRole::SingletonNonReissuable,
            StateExternalEvidenceRole::SingletonIssuedUnderConstructor,
        ])
    );
}

// The two facts the reduced leaf rests on and does not check. Naming them
// beside the binding is what sends a reader to deployment records for
// them; the filter is a filter, so the report-layer role the record also
// carries stays out of the binding.
#[test]
fn both_singleton_facts_appear_in_the_binding_and_freshness_does_not() {
    let bound = bridge();
    let facts = bound.deployment_facts();
    assert!(facts.contains(&StateExternalEvidenceRole::SingletonNonReissuable));
    assert!(facts.contains(&StateExternalEvidenceRole::SingletonIssuedUnderConstructor));

    let composed = record();
    let external = &composed.metadata().external;
    assert!(external.contains(&StateExternalEvidenceRole::CurrentStateRootFreshness));
    assert!(!facts.contains(&StateExternalEvidenceRole::CurrentStateRootFreshness));
}

#[test]
fn bind_refuses_an_identity_other_than_the_operators() {
    let other = identity(0x11, 0x44);
    assert_eq!(
        StateLinkDeploymentParameters::bind(
            &reviewed_target(),
            plan(),
            fixture_lead_bounds(),
            other.clone(),
            binding(),
            depth(),
            &record(),
        ),
        Err(LinkRefusal::OperatorDeploymentMismatch {
            bound: Box::new(identity(0x11, 0x22)),
            offered: Box::new(other),
        })
    );
}

#[test]
fn the_bound_plan_is_the_announcement_plan() {
    assert_eq!(bridge().plan().operation(), OperationId::AnnounceMaturity);
}
