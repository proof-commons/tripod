//! Linker tests and their shared fixtures.
//!
//! # The bundle under test is a real one
//!
//! Every fixture below reaches the relocatable bundle the way an
//! external consumer would: derive the realization, bind the compiler
//! input, plan the compact-ASH target operation, and emit the bundle
//! through the backend's own public entry point. Nothing here
//! hand-assembles a bundle, because a hand-assembled bundle would let a
//! link succeed against an artifact no backend produced.
//!
//! # The symbol values are fixtures and say so
//!
//! The resolutions below are distinguishable, meaningless byte strings.
//! They are test material in the sense
//! `(´[ADR015-rule:security:test-material]´)` fixes: public, carrying no
//! secret, and standing for no real object. A production deployment's
//! values are not this linker's to invent (§1.10), and the demonstration
//! link says it is a fixture link rather than implying otherwise.
//!
//! # The maturity sources are real ones too
//!
//! The announcement plan below is the compiler's own validated plan,
//! reached through its public planning entry over a derived realization;
//! the record is the composed announcement program built through
//! tapscript's public structural, semantic and operator entries; and the
//! candidate constructor is derived over the production static subtree
//! that record composes. Nothing here hand-assembles any of them,
//! because a hand-assembled source would let a link bind something no
//! compiler planned and no backend composed, which is the one thing
//! binding by type exists to prevent.
//!
//! The asset, amount, operator key and identity bytes are the same kind
//! of public, meaningless material as the rest. The lead magnitudes 2
//! and 4 are too, and the binding says so in the type rather than in a
//! comment, which is what [`crate::StateLeadBoundOrigin::Fixture`] is
//! for. The curve capability is scripted for the same reason: point
//! arithmetic is not what these tests are about, and a stub that asserts
//! which key it was asked about is what keeps it from quietly answering
//! for another one.

mod carrier_tests;
mod graph_tests;
mod guide13_reproductions;
mod link_tests;
mod live_link_tests;
mod live_taptree_tests;
mod operator_deployment_tests;
mod public_api_tests;
mod relocation_tests;
mod state_bundle_tests;
mod state_carrier_tests;
mod state_constructor_graph_tests;
mod state_deployment_tests;
mod state_graph_tests;
mod state_relocate_tests;
mod state_resource_tests;
mod state_symbol_tests;
mod state_taptree_tests;
mod symbol_tests;
mod taptree_tests;

use std::collections::BTreeMap;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, AssetId, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::live_transfer_plan::{
    LiveTransferRepresentationPlan, ValidatedLiveTransferOperationPlan,
    plan_live_transfer_target_operation,
};
use compiler::maturity_announcement_plan::{
    ValidatedMaturityAnnouncementOperationPlan, plan_maturity_announcement_target_operation,
};
use compiler::operation_plan::{
    PlacementSearchLimits, ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
};
use realization::{Cycle, Maturity, ProtocolAmount, RealizationScope, StateMetadata, derive};
use tapscript::upstream::{AnnouncementLeadBounds, StateSingletonDeclaration};
use tapscript::{
    CandidateRelocatableLiveTransferBundle, CandidateRelocatableTapscriptBundle,
    CandidateStateConstructor, CompactAshSymbols, EstablishedOperatorProfile, LiveTransferSymbols,
    OperatorKey, OwnerKey, STATE_NUMS_KEY, StackItem, StateAnnouncementBindings,
    StateAnnouncementProgram, StateAnnouncementSymbol, StateCurveCapability,
    StateInternalKeyPolicy, StateNonceBudget, StateOperatorBindings, StateOperatorSymbol,
    StatePatternBindings, StatePatternSymbol, StateStaticLeaf, StateStaticNode, StateStaticSubtree,
    StateTweakOutcome, build_state_announcement_program, build_state_operator_pattern,
    demonstration_live_shape_set, demonstration_policy, derive_live_receipt_constructor,
    emit_candidate_bundle, emit_candidate_live_bundle, operator_key_encoding_closure,
    owner_key_encoding_closure, production_static_subtree, selected_operator_profile,
    state_announcement_patterns, state_announcement_program, state_operator_fragment,
    state_structural_patterns, static_transfer_leaf_set,
};
use target_elements::{
    EncodingClass, LeafVersion, ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript,
};

use crate::deployment::{LinkDeploymentParameters, SelfCommitmentStrategy};
use crate::live_deployment::LiveLinkDeploymentParameters;
use crate::operator_deployment::{CandidateDeploymentIdentity, OperatorDeploymentBinding};
use crate::state_deployment::{
    StateLeadBoundOrigin, StateLeadBounds, StateLinkDeploymentParameters,
};
use crate::{
    CandidateLinkedMaturityBundle, LinkedStateLeafProgram, StateConsumerCensus, StateLinkSources,
    StateLinkedTaptree, StateResolvedCensus, StateSingletonAsset, assemble_state_static,
    collect_state_definitions, link_state_candidate, resolve_state_census,
    state_static_taptree_input, substitute_state,
};

/// The reviewed contract, unmodified.
fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// The validated compact-ASH plan, derived once and handed out by
/// clone.
fn compact_ash_plan() -> ValidatedTargetOperationPlan {
    static PLAN: LazyLock<ValidatedTargetOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let realization =
            derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
        let scope = CompilationScope::from_operations([OperationId::CompactAsh])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_compact_ash_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// The placeholder symbols the bundle is laid out against.
///
/// The backend's own fixture values, repeated here so the link is
/// resolving away exactly the placeholders the bundle recorded.
fn placeholder_symbols(target: &ReviewedElementsTapscriptDefinition) -> CompactAshSymbols {
    CompactAshSymbols::new(
        target,
        vec![0x11; 32],
        vec![0x22; 32],
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .expect("the placeholder symbols are the reviewed widths")
}

/// The resolved symbol values of the demonstration link.
///
/// Different from the placeholders in every field, and different in
/// *width* for the sponsor-change witness program, so a link that
/// substituted nothing and a link that patched bytes in place both fail
/// rather than pass by coincidence.
fn resolved_symbols(target: &ReviewedElementsTapscriptDefinition) -> CompactAshSymbols {
    CompactAshSymbols::new(
        target,
        vec![0xa1; 32],
        vec![0xa2; 32],
        vec![0xa4; 32],
        0,
        vec![0xa5; 32],
    )
    .expect("the resolved symbols are the reviewed widths")
}

/// The demonstration relocatable bundle.
fn relocatable_bundle() -> CandidateRelocatableTapscriptBundle {
    let target = reviewed_target();
    emit_candidate_bundle(
        &target,
        &compact_ash_plan(),
        demonstration_policy(),
        placeholder_symbols(&target),
    )
    .expect("the demonstration bundle emits")
}

/// The demonstration deployment parameters under one strategy.
fn deployment(
    target: &ReviewedElementsTapscriptDefinition,
    strategy: SelfCommitmentStrategy,
) -> LinkDeploymentParameters {
    LinkDeploymentParameters::new(
        target,
        resolved_symbols(target),
        vec![0xa6; 32],
        strategy,
        NonZeroU32::new(8).expect("eight is nonzero"),
    )
    .expect("the demonstration deployment parameters are the reviewed widths")
}

// --- Guide-13 live-transfer fixtures ----------------------------------

/// The validated live-transfer plan, derived once and handed out by
/// clone.
///
/// The derivation is the whole scoped analysis, its validator, the
/// Phase-5 representation filter, and an independent re-derivation of the
/// assembled plan. A clone of the finished value is the same value by
/// construction, so nothing about what is tested changes — only how many
/// times the compiler is asked for it.
fn live_transfer_plan() -> ValidatedLiveTransferOperationPlan {
    static PLAN: LazyLock<ValidatedLiveTransferOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let realization =
            derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
        let scope = CompilationScope::from_operations([OperationId::TransferLive])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_live_transfer_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// The placeholder live symbols the bundles are laid out against.
///
/// The backend's own fixture values, repeated here so a link is resolving
/// away exactly the placeholders the bundles recorded.
fn placeholder_live_symbols(target: &ReviewedElementsTapscriptDefinition) -> LiveTransferSymbols {
    LiveTransferSymbols::new(
        target,
        vec![0x5a; 32],
        vec![0x22; 32],
        1,
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .expect("the placeholder symbols are the reviewed widths")
}

/// The resolved live symbol values of the demonstration link.
///
/// Different from the placeholders in every field, and different in
/// *width* for the sponsor-change witness program, so a link that
/// substituted nothing and a link that patched bytes in place both fail
/// rather than pass by coincidence. The destination program version is
/// left at one because it is the version a taproot output is read at
/// under the reviewed contract, and a different value would be a claim
/// about a different target rather than a different deployment.
fn resolved_live_symbols(target: &ReviewedElementsTapscriptDefinition) -> LiveTransferSymbols {
    LiveTransferSymbols::new(
        target,
        vec![0xb1; 32],
        vec![0xb2; 32],
        1,
        vec![0xb4; 32],
        0,
        vec![0xb5; 32],
    )
    .expect("the resolved symbols are the reviewed widths")
}

/// The demonstration owner's canonical metadata.
///
/// Public test material `(´[ADR015-rule:security:test-material]´)`: a
/// distinguishable byte string at the approved encoding's exact width,
/// standing for no party and paired with no secret. §1.10 gives
/// [`OwnerKey`] no way to hold a private key, and nothing here would have
/// one to offer.
fn live_owner() -> OwnerKey {
    owner_key(&[0x11; 32])
}

/// One owner's canonical metadata from fixture bytes.
fn owner_key(bytes: &[u8]) -> OwnerKey {
    let target = reviewed_target();
    let closure = owner_key_encoding_closure(target.definition().authorization());
    OwnerKey::new(&closure, closure.approved(), bytes.to_vec())
        .expect("the fixture is the approved encoding at its exact width")
}

/// One representation's relocatable bundle for one owner.
fn single_live_bundle(
    representation: LiveTransferRepresentationPlan,
    owner: &[u8],
) -> CandidateRelocatableLiveTransferBundle {
    let target = reviewed_target();
    let shapes = demonstration_live_shape_set();
    let leaves = static_transfer_leaf_set(representation, &shapes);
    let constructor = derive_live_receipt_constructor(
        &target,
        &live_transfer_plan(),
        representation,
        owner_key(owner),
        shapes,
        leaves,
    )
    .expect("the demonstration constructor derives");

    emit_candidate_live_bundle(
        &target,
        &live_transfer_plan(),
        &constructor,
        placeholder_live_symbols(&target),
    )
    .expect("the demonstration bundle emits")
}

/// Both representations' relocatable bundles for the demonstration owner.
///
/// Two bundles rather than one, because §11.3 keeps the representations'
/// leaf sets disjoint and each emission produces a bundle for one
/// constructor. Linking both is what makes §11.5's per-plan comparison a
/// comparison at all.
fn live_bundles() -> Vec<CandidateRelocatableLiveTransferBundle> {
    vec![
        single_live_bundle(LiveTransferRepresentationPlan::Explicit, &[0x11; 32]),
        single_live_bundle(
            LiveTransferRepresentationPlan::PrivateCommitted,
            &[0x11; 32],
        ),
    ]
}

/// The demonstration live deployment parameters.
fn live_deployment(target: &ReviewedElementsTapscriptDefinition) -> LiveLinkDeploymentParameters {
    live_deployment_at_depth(target, 8)
}

/// The demonstration live deployment parameters at one depth policy.
fn live_deployment_at_depth(
    target: &ReviewedElementsTapscriptDefinition,
    depth: u32,
) -> LiveLinkDeploymentParameters {
    LiveLinkDeploymentParameters::new(
        target,
        resolved_live_symbols(target),
        vec![0xb6; 32],
        NonZeroU32::new(depth).expect("the fixture depths are nonzero"),
    )
    .expect("the demonstration deployment parameters are the reviewed widths")
}

// --- Guide-14 maturity fixtures ---------------------------------------

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

        let raw = state_announcement_program(
            &target,
            &structural,
            &semantic,
            &operator,
            tapscript::StateWitnessSchedule::WholeMetadata,
        )
        .expect("the composed program assembles");
        build_state_announcement_program(
            &target,
            &structural,
            &semantic,
            &operator,
            tapscript::StateWitnessSchedule::WholeMetadata,
            raw,
        )
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

/// The maturity link's bound sources over the demonstration fixtures.
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

/// A scripted curve capability.
///
/// The point arithmetic is not under test in the linker, and the
/// assertion inside is what keeps the stub from answering for a key it
/// was not asked about.
struct ScriptedCurve;

impl StateCurveCapability for ScriptedCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        assert_eq!(key, &STATE_NUMS_KEY);
        true
    }

    fn output_key(&self, key: &[u8; 32], _: &[u8; 32]) -> StateTweakOutcome {
        assert_eq!(key, &STATE_NUMS_KEY);
        StateTweakOutcome::OutputKey {
            key: [0x42; 32],
            parity: true,
        }
    }
}

/// The semantic metadata every constructor fixture is derived from.
///
/// The four quantities are pairwise distinct, so a copy-through that
/// swapped two of them would fail rather than pass by coincidence.
fn state_metadata() -> StateMetadata {
    StateMetadata {
        omega: ProtocolAmount::new(1).expect("the fixture quantities are in domain"),
        y_l: ProtocolAmount::new(2).expect("the fixture quantities are in domain"),
        y_t: ProtocolAmount::new(3).expect("the fixture quantities are in domain"),
        q: ProtocolAmount::new(4).expect("the fixture quantities are in domain"),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    }
}

/// The issued identifier: public, meaningless material standing for no
/// issued asset, and the same bytes the record's fixture asset carries
/// so that a site comparison is a comparison.
fn singleton() -> StateSingletonAsset {
    StateSingletonAsset::new([0x11; 32])
}

/// The architecture's own declaration of the identity singleton.
fn declaration() -> StateSingletonDeclaration {
    let spec = ARCHITECTURE
        .asset(AssetId::Pid)
        .expect("the identity asset is declared");
    StateSingletonDeclaration::from_architecture_asset(spec)
        .expect("the declaration is a singleton")
}

/// The resolved symbol census over the demonstration sources.
///
/// One artifact rather than one per test file: the census the graph is
/// built over and the census the symbol tests assert about are the same
/// census, and two copies could drift by a fixture byte with neither
/// test able to notice.
fn resolved_census() -> StateResolvedCensus {
    let target = reviewed_target();
    let definitions = collect_state_definitions(
        &target,
        &bridge(),
        &state_constructor(),
        &singleton(),
        &declaration(),
    )
    .expect("the demonstration sources define every key");

    resolve_state_census(
        &definitions,
        &StateConsumerCensus::from_sources(&record(), &state_constructor()),
    )
    .expect("the demonstration census resolves")
}

/// The demonstration deployment's linked announcement leaf.
///
/// One artifact rather than one per test file, for the reason the
/// resolved census is one: the leaf the tree is bound over, the leaf the
/// resources are measured on and the leaf the carriers are located in
/// have to be the same leaf, and three copies could drift by a fixture
/// byte with no test able to notice.
fn linked_leaf() -> LinkedStateLeafProgram {
    substitute_state(&reviewed_target(), &record(), &resolved_census())
        .expect("the demonstration deployment links")
}

/// The committed tree of a constructor derived over one linked leaf.
///
/// Derived over the leaf's own subtree rather than over the composed
/// record's, because what a deployment publishes commits the bytes a
/// spend runs: a tree bound over the pre-link program would commit a
/// program nobody spends.
fn linked_taptree(leaf: &LinkedStateLeafProgram) -> StateLinkedTaptree {
    let target = reviewed_target();
    let subtree = StateStaticSubtree::new(
        &target,
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role: leaf.leaf(),
                program: leaf.program().clone(),
                version: LeafVersion::TAPSCRIPT.get(),
            },
        }),
    )
    .expect("one linked leaf is a complete static subtree");

    let constructor = CandidateStateConstructor::derive(
        &target,
        &state_metadata(),
        &subtree,
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &ScriptedCurve)
            .expect("the reviewed internal key is the derived one"),
        StateNonceBudget::default(),
        &ScriptedCurve,
    )
    .expect("the constructor derives over the linked subtree");

    let input = state_static_taptree_input(
        [leaf.leaf()],
        LeafVersion::TAPSCRIPT,
        NonZeroU32::new(7).expect("the fixture static cap is nonzero"),
    )
    .expect("the singleton declaration is admitted");
    let tree = assemble_state_static(&input).expect("the singleton tree assembles");

    StateLinkedTaptree::bind(&target, tree, &constructor, depth())
        .expect("the singleton tree binds to the linked leaf's constructor")
}

/// The candidate linked bundle over the demonstration sources.
///
/// One artifact rather than one per test file, for the reason the
/// resolved census and the linked leaf are one: a bundle the aggregate's
/// own tests assert about and a bundle a later bite reads have to be the
/// same bundle, and two copies could drift by a fixture byte with
/// neither able to notice.
fn linked_bundle() -> CandidateLinkedMaturityBundle {
    link_state_candidate(
        &reviewed_target(),
        &StateLinkSources::new(
            &record(),
            &bridge(),
            &state_constructor(),
            &singleton(),
            &declaration(),
            &state_metadata(),
            &ScriptedCurve,
        ),
    )
    .expect("the demonstration sources link")
}

/// The candidate constructor over the production static subtree.
///
/// Derived over the subtree the composed record itself produces, so the
/// constructor and the record are two views of one artifact rather than
/// two unrelated fixtures that happen to be linked together.
fn state_constructor() -> CandidateStateConstructor {
    static CONSTRUCTOR: LazyLock<CandidateStateConstructor> = LazyLock::new(|| {
        let target = reviewed_target();
        let subtree = production_static_subtree(&target, &record())
            .expect("the composed record yields the production subtree");

        CandidateStateConstructor::derive(
            &target,
            &state_metadata(),
            &subtree,
            StateInternalKeyPolicy::new(STATE_NUMS_KEY, &ScriptedCurve)
                .expect("the reviewed internal key is the derived one"),
            StateNonceBudget::default(),
            &ScriptedCurve,
        )
        .expect("the demonstration constructor derives")
    });
    CONSTRUCTOR.clone()
}
