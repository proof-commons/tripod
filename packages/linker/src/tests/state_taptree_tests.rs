//! The maturity constructor's tree, over the announcement leaf itself.
//!
//! # The leaf under test is the production one
//!
//! The singleton fixtures below reach the static subtree the way a
//! consumer would: compose the announcement program through tapscript's
//! own public entry points, hand it to `production_static_subtree`, and
//! derive a constructor over the result. Nothing here hand-assembles the
//! production leaf, because a binding checked against a leaf no emitter
//! produced would be a binding to bytes nobody commits.
//!
//! The hand-built subtrees are a different thing and say so: they exist
//! to exercise shapes the admitted recipe does not carry — a second
//! static leaf, a repeated role, one instruction changed — and their
//! programs are distinguishable fixture material standing for no real
//! operation.
//!
//! # What is proof here and what is a second opinion
//!
//! The tree's optimality is established inside the shared construction
//! by the strict subset oracle, over the static leaf set and under the
//! declared objective. The closed-form and enumerated figures compared
//! below are a second opinion recorded beside that proof: they are exact
//! arithmetic rather than estimates, and they are not the route by which
//! any tree here was admitted.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;
use std::sync::LazyLock;

use realization::{Cycle, Maturity, ProtocolAmount, StateMetadata};
use tapscript::{
    CandidateStateConstructor, STATE_NUMS_KEY, StackItem, StateAnnouncementBindings,
    StateAnnouncementProgram, StateAnnouncementSymbol, StateConstructorRefusal,
    StateCurveCapability, StateInternalKeyPolicy, StateLeafRole, StateNonceBudget,
    StateOperatorBindings, StateOperatorSymbol, StatePatternBindings, StatePatternSymbol,
    StateStaticLeaf, StateStaticNode, StateStaticSubtree, StateTweakOutcome, TapscriptInstruction,
    TapscriptProgram, build_state_announcement_program, build_state_operator_pattern,
    production_static_subtree, state_announcement_patterns, state_announcement_program,
    state_operator_fragment, state_structural_patterns,
};
use target_elements::{EncodingClass, LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::error::LinkRefusal;
use crate::state_taptree::{
    STATE_LEAF_WEIGHT, StateLinkedTaptree, assemble_state_static, state_static_taptree_input,
};
use crate::taptree::{
    ExactOptimumRoute, enumerated_minimum_cost, equal_weight_minimum_cost, exact_minimum_cost,
};

use super::reviewed_target;

/// The deployment's cap on the complete tree in these fixtures.
const DEPLOYMENT_DEPTH: u32 = 8;

fn depth(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("the fixture depths are nonzero")
}

/// The static cap one deployment cap leaves: a level goes to the pair.
///
/// Floored at one because a cap is a positive number, which is also why
/// it is only a guard: the complete-tree check inside the binding is
/// what enforces the deployment's own figure.
fn static_cap(deployment: u32) -> NonZeroU32 {
    depth(deployment.saturating_sub(1).max(1))
}

/// A curve that answers without computing, as the tapscript fixtures do.
///
/// Public test material: the returned point stands for no key and is
/// paired with no secret. Only the merkle root the constructor computes
/// matters here, and that one is real.
struct ScriptedCurve;

impl StateCurveCapability for ScriptedCurve {
    fn internal_key_is_a_point(&self, x_only: &[u8; 32]) -> bool {
        assert_eq!(x_only, &STATE_NUMS_KEY);
        true
    }

    fn output_key(&self, internal_key: &[u8; 32], _: &[u8; 32]) -> StateTweakOutcome {
        assert_eq!(internal_key, &STATE_NUMS_KEY);
        StateTweakOutcome::OutputKey {
            key: [0x42; 32],
            parity: true,
        }
    }
}

/// The canonical metadata one constructor is derived over.
fn metadata() -> StateMetadata {
    StateMetadata {
        omega: ProtocolAmount::new(1).expect("the fixture amounts are in range"),
        y_l: ProtocolAmount::new(2).expect("the fixture amounts are in range"),
        y_t: ProtocolAmount::new(3).expect("the fixture amounts are in range"),
        q: ProtocolAmount::new(4).expect("the fixture amounts are in range"),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    }
}

/// The composed announcement program, built once and shared by
/// reference.
fn announcement_program() -> &'static StateAnnouncementProgram {
    static PROGRAM: LazyLock<StateAnnouncementProgram> = LazyLock::new(|| {
        let target = reviewed_target();
        let item = |bytes: Vec<u8>| StackItem::new(&target, bytes).expect("fixture widths");

        let bindings = StatePatternBindings::new(
            &target,
            BTreeMap::from([
                (StatePatternSymbol::StateAsset, item(vec![0x11; 32])),
                (
                    StatePatternSymbol::StateAmount,
                    StackItem::signed_le64(&target, 1),
                ),
            ]),
        )
        .expect("the structural fixtures bind");
        let structural =
            state_structural_patterns(&target, &bindings).expect("the structural patterns build");

        let bindings = StateAnnouncementBindings::new(
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
        .expect("the semantic fixtures bind");
        let semantic =
            state_announcement_patterns(&target, &bindings).expect("the semantic patterns build");

        let bindings = StateOperatorBindings::new(
            &target,
            &BTreeMap::from([(
                StateOperatorSymbol::CommittedOperatorKey,
                StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0x33; 32])
                    .expect("the operator fixture is the approved encoding"),
            )]),
        )
        .expect("the operator fixtures bind");
        let operator = build_state_operator_pattern(
            &target,
            &bindings,
            state_operator_fragment(&bindings).expect("the operator fragment builds"),
        )
        .expect("the operator pattern builds");

        let composed = state_announcement_program(
            &target,
            &structural,
            &semantic,
            &operator,
            tapscript::StateWitnessSchedule::WholeMetadata,
        )
        .expect("the announcement program composes");
        build_state_announcement_program(
            &target,
            &structural,
            &semantic,
            &operator,
            tapscript::StateWitnessSchedule::WholeMetadata,
            composed,
        )
        .expect("the composed program is admitted")
    });
    &PROGRAM
}

/// The production static subtree: one announcement leaf, identity zero.
fn production_subtree() -> StateStaticSubtree {
    production_static_subtree(&reviewed_target(), announcement_program())
        .expect("the production subtree builds")
}

/// One distinguishable fixture leaf, standing for no real operation.
fn fixture_leaf(
    target: &ReviewedElementsTapscriptDefinition,
    role: StateLeafRole,
    number: i64,
) -> StateStaticLeaf {
    let push = TapscriptInstruction::Push(
        StackItem::script_number(target, number).expect("the fixture numbers are in range"),
    );
    StateStaticLeaf {
        role,
        program: TapscriptProgram::new(vec![push]).expect("the fixture program is admitted"),
        version: LeafVersion::TAPSCRIPT.get(),
    }
}

fn node(
    target: &ReviewedElementsTapscriptDefinition,
    identity: u32,
    role: StateLeafRole,
    number: i64,
) -> StateStaticNode {
    StateStaticNode::Leaf {
        identity,
        leaf: fixture_leaf(target, role, number),
    }
}

/// A one-leaf subtree whose program is the given fixture number.
fn singleton_subtree(number: i64) -> StateStaticSubtree {
    let target = reviewed_target();
    StateStaticSubtree::new(
        &target,
        Some(node(&target, 0, StateLeafRole::Announcement, number)),
    )
    .expect("the fixture subtree validates")
}

/// A two-leaf subtree: the announcement beside one support leaf.
fn two_leaf_subtree() -> StateStaticSubtree {
    let target = reviewed_target();
    StateStaticSubtree::new(
        &target,
        Some(StateStaticNode::Branch(
            Box::new(node(&target, 0, StateLeafRole::Announcement, 1)),
            Box::new(node(&target, 1, StateLeafRole::Support(0), 2)),
        )),
    )
    .expect("the fixture subtree validates")
}

/// A subtree committing the announcement role at two identities.
fn repeated_role_subtree() -> StateStaticSubtree {
    let target = reviewed_target();
    StateStaticSubtree::new(
        &target,
        Some(StateStaticNode::Branch(
            Box::new(node(&target, 0, StateLeafRole::Announcement, 1)),
            Box::new(node(&target, 1, StateLeafRole::Announcement, 2)),
        )),
    )
    .expect("the fixture subtree validates")
}

fn key_policy() -> StateInternalKeyPolicy {
    StateInternalKeyPolicy::new(STATE_NUMS_KEY, &ScriptedCurve).expect("the key is admitted")
}

fn constructor(subtree: &StateStaticSubtree) -> CandidateStateConstructor {
    CandidateStateConstructor::derive(
        &reviewed_target(),
        &metadata(),
        subtree,
        key_policy(),
        StateNonceBudget::default(),
        &ScriptedCurve,
    )
    .expect("the constructor derives")
}

#[test]
fn the_singleton_tree_carries_one_leaf_at_the_root() {
    let input = state_static_taptree_input(
        [StateLeafRole::Announcement],
        LeafVersion::TAPSCRIPT,
        static_cap(DEPLOYMENT_DEPTH),
    )
    .expect("the singleton declaration is admitted");
    assert_eq!(input.leaves().len(), 1);
    let declared = input.leaves()[&StateLeafRole::Announcement];
    assert_eq!(declared.weight(), STATE_LEAF_WEIGHT);
    assert_eq!(declared.role(), StateLeafRole::Announcement);

    let tree = assemble_state_static(&input).expect("the singleton tree assembles");
    assert_eq!(tree.optimum_route(), ExactOptimumRoute::SubsetOracle);
    assert_eq!(tree.depth(), 0);
    assert_eq!(tree.cost(), 0);
    let recipe = &tree.recipes()[&StateLeafRole::Announcement];
    assert_eq!(recipe.depth(), 0);
    let no_siblings: [BTreeSet<StateLeafRole>; 0] = [];
    assert_eq!(recipe.siblings(), no_siblings);
}

#[test]
fn the_singleton_tree_binds_to_the_production_constructor() {
    let target = reviewed_target();
    let subtree = production_subtree();
    let constructor = constructor(&subtree);
    let input = state_static_taptree_input(
        [StateLeafRole::Announcement],
        LeafVersion::TAPSCRIPT,
        static_cap(DEPLOYMENT_DEPTH),
    )
    .expect("the singleton declaration is admitted");
    let tree = assemble_state_static(&input).expect("the singleton tree assembles");

    let cap = depth(DEPLOYMENT_DEPTH);
    let linked = StateLinkedTaptree::bind(&target, tree, &constructor, cap)
        .expect("the singleton tree binds to its own constructor");

    assert_eq!(linked.cost().static_cost(), 0);
    assert_eq!(linked.cost().complete_cost(), 2);
    let expected = BTreeMap::from([
        (StateLeafRole::MetadataCommitment, 1),
        (StateLeafRole::Announcement, 1),
    ]);
    assert_eq!(linked.depths(), &expected);
    assert_eq!(linked.merkle_root(), constructor.merkle_root());
    assert_eq!(linked.static_root(), constructor.static_subtree().root());
    assert_eq!(linked.subtree(), constructor.static_subtree());

    let metadata_recipe = constructor
        .control_recipe(StateLeafRole::MetadataCommitment)
        .expect("the metadata recipe states");
    let announcement_recipe = constructor
        .control_recipe(StateLeafRole::Announcement)
        .expect("the announcement recipe states");
    assert_eq!(linked.metadata_hash(), &metadata_recipe.executing_leaf_hash);
    assert_eq!(metadata_recipe.siblings, vec![*linked.static_root()]);
    assert_eq!(
        announcement_recipe.siblings,
        vec![*linked.metadata_hash()],
        "the announcement's only sibling is the metadata leaf"
    );
}

#[test]
fn the_cost_figures_are_recomputed_by_independent_routes() {
    let weight = STATE_LEAF_WEIGHT.get();

    // The static side: what the construction chose, reached again by the
    // exhaustive subset recurrence.
    assert_eq!(exact_minimum_cost(&[weight]), Ok(0));
    assert_eq!(exact_minimum_cost(&[weight, weight]), Ok(2));

    // The complete side, which no objective selected: the pair's cost is
    // arithmetic, and these figures are the second opinion that it is
    // also what an equal-weight tree of one more leaf would cost.
    assert_eq!(equal_weight_minimum_cost(2, weight), Ok(2));
    assert_eq!(equal_weight_minimum_cost(3, weight), Ok(5));
    assert_eq!(enumerated_minimum_cost(&[weight, weight]), Ok(2));
    assert_eq!(enumerated_minimum_cost(&[weight, weight, weight]), Ok(5));
}

#[test]
fn the_metadata_leaf_cannot_be_declared_static() {
    assert_eq!(
        state_static_taptree_input(
            [
                StateLeafRole::MetadataCommitment,
                StateLeafRole::Announcement
            ],
            LeafVersion::TAPSCRIPT,
            static_cap(DEPLOYMENT_DEPTH),
        ),
        Err(LinkRefusal::MetadataLeafDeclaredStatic)
    );
}

#[test]
fn a_static_tree_without_an_announcement_is_refused() {
    assert_eq!(
        state_static_taptree_input(
            [StateLeafRole::Support(0)],
            LeafVersion::TAPSCRIPT,
            static_cap(DEPLOYMENT_DEPTH),
        ),
        Err(LinkRefusal::StaticTreeWithoutAnnouncement)
    );
    let none: [StateLeafRole; 0] = [];
    assert_eq!(
        state_static_taptree_input(none, LeafVersion::TAPSCRIPT, static_cap(DEPLOYMENT_DEPTH)),
        Err(LinkRefusal::StaticTreeWithoutAnnouncement),
        "an empty sequence is refused for the announcement it lacks"
    );
}

#[test]
fn one_leaf_declared_twice_is_refused() {
    assert_eq!(
        state_static_taptree_input(
            [StateLeafRole::Announcement, StateLeafRole::Announcement],
            LeafVersion::TAPSCRIPT,
            static_cap(DEPLOYMENT_DEPTH),
        ),
        Err(LinkRefusal::DuplicateStateTreeLeaf(
            StateLeafRole::Announcement
        ))
    );
}

#[test]
fn a_declaration_the_constructor_did_not_commit_is_refused() {
    let target = reviewed_target();
    let subtree = production_subtree();
    let constructor = constructor(&subtree);
    let input = state_static_taptree_input(
        [StateLeafRole::Announcement, StateLeafRole::Support(0)],
        LeafVersion::TAPSCRIPT,
        static_cap(DEPLOYMENT_DEPTH),
    )
    .expect("the two-leaf declaration is admitted");
    let tree = assemble_state_static(&input).expect("the two-leaf tree assembles");

    assert_eq!(
        StateLinkedTaptree::bind(&target, tree, &constructor, depth(DEPLOYMENT_DEPTH)),
        Err(LinkRefusal::StateStaticLeafSetMismatch {
            declared: BTreeSet::from([StateLeafRole::Announcement, StateLeafRole::Support(0)]),
            committed: BTreeSet::from([StateLeafRole::Announcement]),
        })
    );
}

#[test]
fn a_role_committed_twice_never_reaches_the_link() {
    // The constructor refuses a subtree committing one role twice, so
    // the binding's own census check is its independent statement of the
    // same rule rather than a path a derived constructor can reach.
    let derived = CandidateStateConstructor::derive(
        &reviewed_target(),
        &metadata(),
        &repeated_role_subtree(),
        key_policy(),
        StateNonceBudget::default(),
        &ScriptedCurve,
    );
    assert_eq!(derived, Err(StateConstructorRefusal::ExecutingLeafRepeated));
}

#[test]
fn the_two_leaf_tree_costs_two_statically_and_five_complete() {
    let target = reviewed_target();
    let subtree = two_leaf_subtree();
    let constructor = constructor(&subtree);
    let input = state_static_taptree_input(
        [StateLeafRole::Announcement, StateLeafRole::Support(0)],
        LeafVersion::TAPSCRIPT,
        static_cap(DEPLOYMENT_DEPTH),
    )
    .expect("the two-leaf declaration is admitted");
    let tree = assemble_state_static(&input).expect("the two-leaf tree assembles");
    assert_eq!(tree.depth(), 1);

    let cap = depth(DEPLOYMENT_DEPTH);
    let linked = StateLinkedTaptree::bind(&target, tree, &constructor, cap)
        .expect("the two-leaf tree binds to its own constructor");
    assert_eq!(linked.cost().static_cost(), 2);
    assert_eq!(linked.cost().complete_cost(), 5);
    let expected = BTreeMap::from([
        (StateLeafRole::MetadataCommitment, 1),
        (StateLeafRole::Announcement, 2),
        (StateLeafRole::Support(0), 2),
    ]);
    assert_eq!(linked.depths(), &expected);
}

#[test]
fn the_depth_cap_is_read_over_the_complete_tree() {
    let target = reviewed_target();
    let subtree = two_leaf_subtree();
    let constructor = constructor(&subtree);

    // The static cap a deployment cap of one leaves admits this tree:
    // its static depth is one. The complete tree is a level deeper, and
    // that is the figure the deployment's cap is about.
    let input = state_static_taptree_input(
        [StateLeafRole::Announcement, StateLeafRole::Support(0)],
        LeafVersion::TAPSCRIPT,
        static_cap(1),
    )
    .expect("the two-leaf declaration is admitted at the static cap");
    let tree = assemble_state_static(&input).expect("the two-leaf tree assembles");

    assert_eq!(
        StateLinkedTaptree::bind(&target, tree.clone(), &constructor, depth(1)),
        Err(LinkRefusal::StateTreeDepthExceeded {
            leaf: StateLeafRole::Support(0),
            depth: 2,
            maximum: 1,
        })
    );
    assert!(StateLinkedTaptree::bind(&target, tree, &constructor, depth(2)).is_ok());
}

#[test]
fn reversing_the_declarations_assembles_the_same_tree() {
    let forward = state_static_taptree_input(
        [StateLeafRole::Announcement, StateLeafRole::Support(0)],
        LeafVersion::TAPSCRIPT,
        static_cap(DEPLOYMENT_DEPTH),
    )
    .expect("the two-leaf declaration is admitted");
    let reversed = state_static_taptree_input(
        [StateLeafRole::Support(0), StateLeafRole::Announcement],
        LeafVersion::TAPSCRIPT,
        static_cap(DEPLOYMENT_DEPTH),
    )
    .expect("the reversed declaration is admitted");
    assert_eq!(forward, reversed);

    let forward = assemble_state_static(&forward).expect("the two-leaf tree assembles");
    let reversed = assemble_state_static(&reversed).expect("the reversed tree assembles");
    assert_eq!(forward, reversed);
    assert_eq!(forward.recipes(), reversed.recipes());
}

#[test]
fn the_binding_is_to_bytes_rather_than_to_a_role_list() {
    let target = reviewed_target();
    let one = constructor(&singleton_subtree(1));
    let other = constructor(&singleton_subtree(2));
    assert_ne!(one.static_subtree().root(), other.static_subtree().root());
    assert_ne!(one.merkle_root(), other.merkle_root());

    let input = state_static_taptree_input(
        [StateLeafRole::Announcement],
        LeafVersion::TAPSCRIPT,
        static_cap(DEPLOYMENT_DEPTH),
    )
    .expect("the singleton declaration is admitted");
    let tree = assemble_state_static(&input).expect("the singleton tree assembles");

    let cap = depth(DEPLOYMENT_DEPTH);
    let first = StateLinkedTaptree::bind(&target, tree.clone(), &one, cap)
        .expect("the tree binds to the first constructor");
    let second = StateLinkedTaptree::bind(&target, tree, &other, cap)
        .expect("the tree binds to the second constructor");

    // One declared role set, two trees: the evidence is the constructor's
    // bytes and not the names the declaration carried.
    assert_eq!(first.tree(), second.tree());
    assert_eq!(first.depths(), second.depths());
    assert_eq!(first.cost(), second.cost());
    assert_ne!(first.merkle_root(), second.merkle_root());
    assert_eq!(first.merkle_root(), one.merkle_root());
    assert_eq!(second.merkle_root(), other.merkle_root());
}
