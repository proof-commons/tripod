use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, VecDeque};
use std::fmt::Write;

use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, Maturity, ProtocolAmount, StateField,
    StateMetadata, StateRepresentationNonce, TransactionSide, announce_maturity,
    encode_state_metadata,
};
use sha2::{Digest, Sha256};
use target_elements::{LeafVersion, StackValueType};

use crate::state_constructor::*;
use crate::{
    AbstractLimits, AbstractStackState, InternalKeyPolicy, KeyPathPolicy, StackItem,
    TapscriptInstruction, TapscriptProgram, validate_program,
};

use super::reviewed_target;
use super::state_program_tests::{fixtures, independent_leaf_hash, production_tree, tagged_hash};

const GOLDEN_PROGRAM: &str = "4c56747269706f642f73746174652d6d6574616461746100000001000000000000000100000000000000020000000000000003000000000000000400000000000000050000000000000000000000000100000000000000000069";
const GOLDEN_LEAF: &str = "1dd35257bddfbf0c1c9e0eb4751e91e440cdb2e618c5b02863ffeb21987ae6a6";
const GOLDEN_STATIC: &str = "58a38d98026e1a3a592df40f11f32e503fdaa0a6c0c69effcbed5eb9d2780688";
const GOLDEN_ROOT: &str = "d1eaf25684834d51216c2ca8486662f38aa193bd4e43fb3c8495c28546938d98";
const GOLDEN_TWEAK: &str = "b40f1fbc52d420ebd6d589f1ee1900cb246ca84e584db0ade442bde886fcd365";
const GOLDEN_METADATA_CONTROL: &str = "c550929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac058a38d98026e1a3a592df40f11f32e503fdaa0a6c0c69effcbed5eb9d2780688";
const GOLDEN_STATIC_CONTROL: &str = "c550929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac01dd35257bddfbf0c1c9e0eb4751e91e440cdb2e618c5b02863ffeb21987ae6a6";

struct ScriptedCurve {
    valid: bool,
    outcomes: RefCell<VecDeque<StateTweakOutcome>>,
    fallback: StateTweakOutcome,
    calls: Cell<usize>,
}

impl ScriptedCurve {
    fn new(outcomes: impl IntoIterator<Item = StateTweakOutcome>) -> Self {
        Self {
            valid: true,
            outcomes: RefCell::new(outcomes.into_iter().collect()),
            fallback: StateTweakOutcome::OutputKey {
                key: [0x42; 32],
                parity: true,
            },
            calls: Cell::new(0),
        }
    }
}

impl StateCurveCapability for ScriptedCurve {
    fn internal_key_is_a_point(&self, x_only: &[u8; 32]) -> bool {
        assert_eq!(x_only, &STATE_NUMS_KEY);
        self.valid
    }
    fn output_key(&self, internal_key: &[u8; 32], _: &[u8; 32]) -> StateTweakOutcome {
        assert_eq!(internal_key, &STATE_NUMS_KEY);
        self.calls.set(self.calls.get() + 1);
        self.outcomes
            .borrow_mut()
            .pop_front()
            .unwrap_or(self.fallback)
    }
}

fn metadata() -> StateMetadata {
    StateMetadata {
        omega: ProtocolAmount::new(1).unwrap(),
        y_l: ProtocolAmount::new(2).unwrap(),
        y_t: ProtocolAmount::new(3).unwrap(),
        q: ProtocolAmount::new(4).unwrap(),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    }
}

fn program(number: i64) -> TapscriptProgram {
    TapscriptProgram::new(vec![TapscriptInstruction::Push(
        StackItem::script_number(&reviewed_target(), number).unwrap(),
    )])
    .unwrap()
}

fn node(identity: u32, role: StateLeafRole, number: i64) -> StateStaticNode {
    StateStaticNode::Leaf {
        identity,
        leaf: StateStaticLeaf {
            role,
            program: program(number),
            version: LeafVersion::TAPSCRIPT.get(),
        },
    }
}

fn branch(left: StateStaticNode, right: StateStaticNode) -> StateStaticNode {
    StateStaticNode::Branch(Box::new(left), Box::new(right))
}

fn subtree(number: i64) -> StateStaticSubtree {
    StateStaticSubtree::new(
        &reviewed_target(),
        Some(node(0, StateLeafRole::Announcement, number)),
    )
    .unwrap()
}

fn policy(curve: &ScriptedCurve) -> StateInternalKeyPolicy {
    StateInternalKeyPolicy::new(STATE_NUMS_KEY, curve).unwrap()
}

fn derive(
    number: i64,
    attempts: u32,
    curve: &ScriptedCurve,
) -> Result<CandidateStateConstructor, StateConstructorRefusal> {
    CandidateStateConstructor::derive(
        &reviewed_target(),
        &metadata(),
        &subtree(number),
        policy(curve),
        StateNonceBudget::new(attempts).unwrap(),
        curve,
    )
}

fn candidate() -> CandidateStateConstructor {
    let curve = ScriptedCurve::new([]);
    CandidateStateConstructor::derive(
        &reviewed_target(),
        &metadata(),
        &production_tree(),
        policy(&curve),
        StateNonceBudget::default(),
        &curve,
    )
    .unwrap()
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

struct IndependentGolden {
    nonce: StateRepresentationNonce,
    program: Vec<u8>,
    leaf: [u8; 32],
    static_root: [u8; 32],
    root: [u8; 32],
    tweak: [u8; 32],
    metadata_control: Vec<u8>,
    static_control: Vec<u8>,
}

fn independent_golden() -> IndependentGolden {
    let bytes = fixtures().program.program().encode(&reviewed_target());
    let static_root = independent_leaf_hash(&bytes);
    let curve = ScriptedCurve::new([]);
    for attempt in 0..4096 {
        let nonce = StateRepresentationNonce::new(attempt);
        let metadata = encode_state_metadata(&metadata(), nonce);
        let mut program = vec![0x4c, u8::try_from(metadata.len()).unwrap()];
        program.extend(metadata);
        program.extend([0x00, 0x69]);
        let leaf = independent_leaf_hash(&program);
        if leaf > static_root {
            continue;
        }
        let root = tagged_hash(b"TapBranch/elements", &[leaf, static_root].concat());
        let StateTweakOutcome::OutputKey { key, parity } = curve.output_key(&STATE_NUMS_KEY, &root)
        else {
            continue;
        };
        assert_eq!(key, [0x42; 32]);
        let tweak = tagged_hash(b"TapTweak/elements", &[STATE_NUMS_KEY, root].concat());
        let mut control = vec![0xc4 | u8::from(parity)];
        control.extend(STATE_NUMS_KEY);
        let mut metadata_control = control.clone();
        metadata_control.extend(static_root);
        control.extend(leaf);
        return IndependentGolden {
            nonce,
            program,
            leaf,
            static_root,
            root,
            tweak,
            metadata_control,
            static_control: control,
        };
    }
    panic!("independent production fixture search exhausted");
}

#[test]
fn golden_canonical_bytes_and_target_hashes() {
    let expected = independent_golden();
    let built = candidate();
    assert_eq!(built.nonce(), expected.nonce);
    assert_eq!(
        built.leaf_program().encode(&reviewed_target()),
        expected.program
    );
    assert_eq!(
        built
            .control_recipe(StateLeafRole::MetadataCommitment)
            .unwrap()
            .executing_leaf_hash,
        expected.leaf
    );
    assert_eq!(built.static_subtree().root(), &expected.static_root);
    assert_eq!(built.merkle_root(), &expected.root);
    assert_eq!(built.tweak_hash(), expected.tweak);
    assert_eq!(built.metadata_bytes().len(), 86);
    assert_eq!(
        &built.metadata_bytes()[74..78],
        &expected.nonce.get().to_be_bytes()
    );
    let actual = [
        hex(&expected.program),
        hex(&expected.leaf),
        hex(&expected.static_root),
        hex(&expected.root),
        hex(&expected.tweak),
        hex(&expected.metadata_control),
        hex(&expected.static_control),
    ];
    assert_eq!(
        actual,
        [
            GOLDEN_PROGRAM,
            GOLDEN_LEAF,
            GOLDEN_STATIC,
            GOLDEN_ROOT,
            GOLDEN_TWEAK,
            GOLDEN_METADATA_CONTROL,
            GOLDEN_STATIC_CONTROL
        ],
        "recomputed nonce={}\ncomponent map={:?}\nconsumers={:?}\nresources={:?}\naborts={:?}",
        expected.nonce.get(),
        fixtures().program.components(),
        fixtures().program.consumers(),
        fixtures().program.resources(),
        fixtures().program.execution().aborts()
    );
}

#[test]
fn golden_control_blocks_are_derived_for_both_direct_leaves() {
    let expected = independent_golden();
    let built = candidate();
    for (role, expected, golden) in [
        (
            StateLeafRole::MetadataCommitment,
            expected.metadata_control,
            GOLDEN_METADATA_CONTROL,
        ),
        (
            StateLeafRole::Announcement,
            expected.static_control,
            GOLDEN_STATIC_CONTROL,
        ),
    ] {
        let recipe = built.control_recipe(role).unwrap();
        let bytes = recipe.control_bytes().unwrap();
        assert_eq!(bytes.len(), 65);
        assert_eq!(bytes, expected);
        assert_eq!(hex(&bytes), golden);
        assert_eq!(recipe.role, role);
        assert_eq!(recipe.siblings.len(), 1);
    }
}

#[test]
fn predecessor_and_successor_recipes_preserve_the_production_subtree() {
    let curve = ScriptedCurve::new([]);
    let target = reviewed_target();
    let tree = production_tree();
    let input = metadata();
    let output = announce_maturity(
        &input,
        Cycle::new(7),
        AnnouncementLeadBounds::new(Cycle::new(2), Cycle::new(4)).unwrap(),
    )
    .unwrap();
    let before = predecessor_recipe(
        &target,
        &input,
        &tree,
        policy(&curve),
        StateNonceBudget::default(),
        &curve,
    )
    .unwrap();
    let after = successor_recipe(
        &target,
        &output,
        &tree,
        policy(&curve),
        StateNonceBudget::default(),
        &curve,
    )
    .unwrap();
    before.continuity(&after).unwrap();
    assert_eq!(before.static_subtree(), after.static_subtree());
    assert_ne!(before.metadata_bytes(), after.metadata_bytes());
}

#[test]
fn scripted_output_and_parity_do_not_claim_curve_conformance() {
    let built = candidate();
    assert_eq!(built.output_key(), &[0x42; 32]);
    assert!(built.parity());
    let mut expected = vec![0x51, 0x20];
    expected.extend([0x42; 32]);
    assert_eq!(built.output_program(), expected);
    let mut recipe = built.control_recipe(StateLeafRole::Announcement).unwrap();
    recipe.parity = false;
    assert_eq!(recipe.control_bytes().unwrap()[0], 0xc4);
}

#[test]
fn deterministic_repetition_preserves_all_evidence_and_recipes() {
    assert_eq!(candidate(), candidate());
    assert_eq!(
        candidate().generation(),
        StateConstructorGeneration::CanonicalMetadataV1
    );
}

#[test]
fn first_candidate_succeeds_when_both_relations_hold() {
    let curve = ScriptedCurve::new([]);
    let built = derive(4, 1, &curve).unwrap();
    assert_eq!(built.nonce(), StateRepresentationNonce::ZERO);
    assert_eq!(
        built.evidence().rejected,
        Vec::<(StateRepresentationNonce, StateConstructorRefusal)>::new()
    );
    assert_eq!(curve.calls.get(), 1);
}

#[test]
fn branch_failure_precedes_any_curve_call() {
    let curve = ScriptedCurve::new([]);
    let built = derive(1, 2, &curve).unwrap();
    assert_eq!(
        built.evidence().rejected,
        vec![(
            StateRepresentationNonce::ZERO,
            StateConstructorRefusal::CanonicalBranchSideNotSatisfied
        )]
    );
    assert_eq!(built.evidence().selected, built.nonce());
    assert_eq!(curve.calls.get(), 1);
}

#[test]
fn late_candidate_records_every_retry_in_nonce_order() {
    let curve = ScriptedCurve::new([
        StateTweakOutcome::TweakAboveGroupOrder,
        StateTweakOutcome::TweakedPointIsIdentity,
    ]);
    let built = derive(4, 4096, &curve).unwrap();
    assert_eq!(built.nonce().get(), 2);
    assert_eq!(
        built.evidence().rejected,
        vec![
            (
                StateRepresentationNonce::new(0),
                StateConstructorRefusal::TweakAboveGroupOrder
            ),
            (
                StateRepresentationNonce::new(1),
                StateConstructorRefusal::TweakedPointIsIdentity
            ),
        ]
    );
    assert_eq!(curve.calls.get(), 3);
}

#[test]
fn exhausted_branch_budget_reports_exhaustion() {
    let curve = ScriptedCurve::new([]);
    assert_eq!(
        derive(1, 1, &curve).unwrap_err(),
        StateConstructorRefusal::RepresentationSearchExhausted
    );
    assert_eq!(curve.calls.get(), 0);
}

#[test]
fn exhausted_curve_budget_does_not_retry_beyond_bound() {
    let curve = ScriptedCurve::new([StateTweakOutcome::TweakAboveGroupOrder]);
    assert_eq!(
        derive(4, 1, &curve).unwrap_err(),
        StateConstructorRefusal::RepresentationSearchExhausted
    );
    assert_eq!(curve.calls.get(), 1);
    assert!(StateNonceEvidence::RESIDUAL.contains("later admissible nonce"));
}

#[test]
fn budget_is_exactly_the_reviewed_range_and_zero_is_refused() {
    assert_eq!(StateNonceBudget::default().attempts(), 4096);
    assert_eq!(
        StateNonceBudget::new(0),
        Err(StateConstructorRefusal::ResourceBoundExceeded)
    );
    assert_eq!(
        StateNonceBudget::new(4097),
        Err(StateConstructorRefusal::ResourceBoundExceeded)
    );
    assert_eq!(StateNonceBudget::new(1).unwrap().attempts(), 1);
}

#[test]
fn unliftable_internal_key_is_not_retried() {
    let curve = ScriptedCurve::new([StateTweakOutcome::InternalKeyNotAPoint]);
    assert_eq!(
        derive(4, 4096, &curve).unwrap_err(),
        StateConstructorRefusal::InternalKeyNotAPoint
    );
    assert_eq!(curve.calls.get(), 1);
    let invalid = ScriptedCurve {
        valid: false,
        ..ScriptedCurve::new([])
    };
    assert_eq!(
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &invalid),
        Err(StateConstructorRefusal::InternalKeyNotAPoint)
    );
    assert_eq!(invalid.calls.get(), 0);
}

#[test]
fn valid_output_equal_to_internal_key_is_admitted() {
    // Models the valid sum P + 0G = P; this seam makes no arithmetic claim.
    let curve = ScriptedCurve::new([StateTweakOutcome::OutputKey {
        key: STATE_NUMS_KEY,
        parity: false,
    }]);
    let built = derive(4, 1, &curve).unwrap();
    assert_eq!(built.output_key(), &STATE_NUMS_KEY);
    assert!(!built.parity());
}

#[test]
fn nums_is_recomputed_from_public_uncompressed_generator() {
    let mut generator = vec![0x04];
    generator.extend(STATE_GENERATOR_X);
    generator.extend(STATE_GENERATOR_Y);
    let derived: [u8; 32] = Sha256::digest(&generator).into();
    assert_eq!(
        hex(&derived),
        "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"
    );
    let curve = ScriptedCurve::new([]);
    assert_eq!(policy(&curve).key(), &derived);
    assert_eq!(
        StateInternalKeyPolicy::new([0; 32], &curve),
        Err(StateConstructorRefusal::WrongInternalKey)
    );
}

#[test]
fn key_path_policy_retains_the_residual_without_claiming_impossibility() {
    assert_eq!(
        StateInternalKeyPolicy::INTERNAL,
        InternalKeyPolicy::UnspendableWithResidualDiscreteLogAssumption
    );
    assert_eq!(
        StateInternalKeyPolicy::KEY_PATH,
        KeyPathPolicy::NoAcceptedEscape
    );
    assert!(StateInternalKeyPolicy::RESIDUAL.contains("discrete-log"));
    assert!(StateInternalKeyPolicy::RESIDUAL.contains("preimage"));
    assert!(StateInternalKeyPolicy::RESIDUAL.contains("no proof"));
}

#[test]
fn static_empty_and_incomplete_sets_are_refused() {
    let target = reviewed_target();
    assert_eq!(
        StateStaticSubtree::new(&target, None).unwrap_err(),
        StateConstructorRefusal::StaticSubtreeEmpty
    );
    for role in [StateLeafRole::Support(0), StateLeafRole::MetadataCommitment] {
        assert_eq!(
            StateStaticSubtree::new(&target, Some(node(0, role, 1))).unwrap_err(),
            StateConstructorRefusal::StaticSubtreeIncomplete
        );
    }
}

#[test]
fn identity_duplicates_and_all_three_conflicts_are_refused() {
    let target = reviewed_target();
    let original = node(0, StateLeafRole::Announcement, 1);
    assert_eq!(
        StateStaticSubtree::new(&target, Some(branch(original.clone(), original.clone())))
            .unwrap_err(),
        StateConstructorRefusal::DuplicateLeaf
    );
    let mut wrong_version = original.clone();
    if let StateStaticNode::Leaf { leaf, .. } = &mut wrong_version {
        leaf.version = 0xc0;
    }
    for conflicting in [
        node(0, StateLeafRole::Support(0), 1),
        node(0, StateLeafRole::Announcement, 2),
        wrong_version,
    ] {
        assert_eq!(
            StateStaticSubtree::new(&target, Some(branch(original.clone(), conflicting)))
                .unwrap_err(),
            StateConstructorRefusal::ConflictingLeaf
        );
    }
}

#[test]
fn bitcoin_leaf_version_cannot_enter_the_elements_constructor() {
    let mut offered = node(0, StateLeafRole::Announcement, 1);
    if let StateStaticNode::Leaf { leaf, .. } = &mut offered {
        leaf.version = 0xc0;
    }
    assert_eq!(
        StateStaticSubtree::new(&reviewed_target(), Some(offered)).unwrap_err(),
        StateConstructorRefusal::WrongLeafVersion
    );
}

#[test]
fn static_branch_order_is_hash_order_and_paths_are_reconstructed() {
    let left = node(0, StateLeafRole::Announcement, 1);
    let right = node(1, StateLeafRole::Support(1), 2);
    let target = reviewed_target();
    let forward =
        StateStaticSubtree::new(&target, Some(branch(left.clone(), right.clone()))).unwrap();
    let reverse = StateStaticSubtree::new(&target, Some(branch(right, left))).unwrap();
    assert_eq!(forward.root(), reverse.root());
    for entry in forward.leaves() {
        assert_eq!(forward.path_to(&entry.hash), reverse.path_to(&entry.hash));
        assert_eq!(entry.siblings.len(), 1);
    }
    assert!(matches!(forward.tree(), StateStaticNode::Branch(_, _)));
}

#[test]
fn absent_and_repeated_executing_hashes_are_refused() {
    let single = subtree(1);
    assert_eq!(
        single.path_to(&[0; 32]),
        Err(StateConstructorRefusal::ExecutingLeafAbsent)
    );
    let repeated = StateStaticSubtree::new(
        &reviewed_target(),
        Some(branch(
            node(0, StateLeafRole::Announcement, 1),
            node(1, StateLeafRole::Support(1), 1),
        )),
    )
    .unwrap();
    assert_eq!(
        repeated.path_to(single.root()),
        Err(StateConstructorRefusal::ExecutingLeafRepeated)
    );
    let curve = ScriptedCurve::new([]);
    assert_eq!(
        CandidateStateConstructor::derive(
            &reviewed_target(),
            &metadata(),
            &repeated,
            policy(&curve),
            StateNonceBudget::default(),
            &curve
        )
        .unwrap_err(),
        StateConstructorRefusal::ExecutingLeafRepeated
    );
}

#[test]
fn repeated_role_is_refused_even_when_hashes_differ() {
    let tree = StateStaticSubtree::new(
        &reviewed_target(),
        Some(branch(
            node(0, StateLeafRole::Announcement, 1),
            node(1, StateLeafRole::Announcement, 2),
        )),
    )
    .unwrap();
    let curve = ScriptedCurve::new([]);
    assert_eq!(
        CandidateStateConstructor::derive(
            &reviewed_target(),
            &metadata(),
            &tree,
            policy(&curve),
            StateNonceBudget::default(),
            &curve
        )
        .unwrap_err(),
        StateConstructorRefusal::ExecutingLeafRepeated
    );
    assert_eq!(curve.calls.get(), 0);
}

#[test]
fn control_recipe_refuses_absent_role_and_appends_outer_sibling_last() {
    let tree = StateStaticSubtree::new(
        &reviewed_target(),
        Some(branch(
            node(0, StateLeafRole::Announcement, 1),
            node(1, StateLeafRole::Support(1), 2),
        )),
    )
    .unwrap();
    let curve = ScriptedCurve::new([]);
    let built = CandidateStateConstructor::derive(
        &reviewed_target(),
        &metadata(),
        &tree,
        policy(&curve),
        StateNonceBudget::default(),
        &curve,
    )
    .unwrap();
    let metadata_hash = built
        .control_recipe(StateLeafRole::MetadataCommitment)
        .unwrap()
        .executing_leaf_hash;
    for entry in tree.leaves() {
        let recipe = built.control_recipe(entry.leaf.role).unwrap();
        assert_eq!(recipe.siblings[..1], entry.siblings);
        assert_eq!(recipe.siblings.last(), Some(&metadata_hash));
        assert_eq!(recipe.control_bytes().unwrap().len(), 97);
    }
    assert_eq!(
        built
            .control_recipe(StateLeafRole::Support(99))
            .unwrap_err(),
        StateConstructorRefusal::ExecutingLeafAbsent
    );
}

fn deep_node(depth: u32) -> StateStaticNode {
    let mut tree = node(0, StateLeafRole::Announcement, 1);
    for identity in 1..=depth {
        tree = branch(
            tree,
            node(
                identity,
                StateLeafRole::Support(identity),
                i64::from(identity) + 1,
            ),
        );
    }
    tree
}

#[test]
fn inner_path_of_128_nodes_is_refused_when_outer_sibling_is_added() {
    let target = reviewed_target();
    let tree = StateStaticSubtree::new(&target, Some(deep_node(128))).unwrap();
    assert_eq!(tree.leaves()[0].siblings.len(), 128);
    let curve = ScriptedCurve::new([]);
    assert_eq!(
        CandidateStateConstructor::derive(
            &target,
            &metadata(),
            &tree,
            policy(&curve),
            StateNonceBudget::default(),
            &curve
        )
        .unwrap_err(),
        StateConstructorRefusal::ControlPathTooDeep
    );
    assert_eq!(curve.calls.get(), 0);
    assert_eq!(
        StateStaticSubtree::new(&target, Some(deep_node(129))).unwrap_err(),
        StateConstructorRefusal::ControlPathTooDeep
    );
}

#[test]
fn control_encoding_enforces_128_node_limit() {
    let mut recipe = candidate()
        .control_recipe(StateLeafRole::Announcement)
        .unwrap();
    recipe.siblings = vec![[0; 32]; 128];
    assert_eq!(recipe.control_bytes().unwrap().len(), 33 + 128 * 32);
    recipe.siblings.push([1; 32]);
    assert_eq!(
        recipe.control_bytes(),
        Err(StateConstructorRefusal::ControlPathTooDeep)
    );
}

#[test]
fn metadata_pattern_protects_exact_bytes_and_only_aborts() {
    let built = candidate();
    let pattern = built.pattern();
    assert_eq!(pattern.protected_source, built.metadata_bytes());
    assert!(pattern.execution.success().is_empty());
    assert!(pattern.execution.nonaborting_failure().is_empty());
    assert!(!pattern.execution.aborts().is_empty());
    assert!(pattern.execution.always_aborts());
    assert_eq!(pattern.program.len(), 3);
    assert_eq!(pattern.stack.peak_main(), 2);
    assert_eq!(pattern.stack.peak_alternate(), 0);
    assert_eq!(
        pattern.resources[&target_elements::ResourceDimension::ScriptBytes],
        90
    );
    assert!(StateMetadataPattern::NATIVE_EVIDENCE.starts_with("outstanding"));
    assert!(StateMetadataPattern::WITNESS_ROLE.starts_with("non-executing"));
    assert!(StateMetadataPattern::RESIDUAL.contains("native"));
    assert!(StateMetadataPattern::PREREQUISITES.contains("Verify"));
}

#[test]
fn metadata_leaf_ignores_arbitrary_admitted_initial_stack_contents() {
    let target = reviewed_target();
    let built = candidate();
    for stack in [
        Vec::new(),
        vec![StackValueType::Bool],
        vec![
            StackValueType::Bytes {
                minimum: 0,
                maximum: 520
            };
            17
        ],
    ] {
        let result = validate_program(
            &target,
            built.leaf_program(),
            &AbstractStackState::from_main(stack),
            AbstractLimits::for_target(&target),
        )
        .unwrap();
        assert!(result.always_aborts());
    }
}

#[test]
fn empty_or_changed_metadata_program_is_not_admitted_as_the_pattern() {
    let target = reviewed_target();
    let built = candidate();
    for offered in [TapscriptProgram::new(Vec::new()).unwrap(), program(1)] {
        assert_eq!(
            StateMetadataPattern::validate(&target, built.encoded_metadata(), &offered)
                .unwrap_err(),
            StateConstructorRefusal::MetadataLeafNotUnspendable
        );
    }
    let mut changed = *built.encoded_metadata();
    changed.representation = StateRepresentationNonce::new(7);
    assert_eq!(
        StateMetadataPattern::validate(&target, &changed, built.leaf_program()).unwrap_err(),
        StateConstructorRefusal::MetadataLeafNotUnspendable
    );
}

#[test]
fn prototype_schema_and_noncanonical_state_bytes_are_refused() {
    let target = reviewed_target();
    for bytes in [vec![0; 48], vec![0; 86], vec![0; 87]] {
        assert_eq!(
            StateMetadataPattern::from_bytes(&target, &bytes).unwrap_err(),
            StateConstructorRefusal::MetadataEncodingRefused
        );
    }
    let built = candidate();
    assert_eq!(
        &StateMetadataPattern::from_bytes(&target, built.metadata_bytes()).unwrap(),
        built.pattern()
    );
    let mut reserved = built.metadata_bytes().to_vec();
    reserved[85] = 1;
    assert_eq!(
        StateMetadataPattern::from_bytes(&target, &reserved).unwrap_err(),
        StateConstructorRefusal::MetadataEncodingRefused
    );
}

#[test]
fn field_commitments_follow_realization_maturity_transition_on_both_sides() {
    let target = reviewed_target();
    let curve = ScriptedCurve::new([]);
    let input = metadata();
    let output = announce_maturity(
        &input,
        Cycle::new(7),
        AnnouncementLeadBounds::new(Cycle::new(1), Cycle::new(3)).unwrap(),
    )
    .unwrap();
    let tree = subtree(1);
    let predecessor = predecessor_recipe(
        &target,
        &input,
        &tree,
        policy(&curve),
        StateNonceBudget::default(),
        &curve,
    )
    .unwrap();
    let successor = successor_recipe(
        &target,
        &output,
        &tree,
        policy(&curve),
        StateNonceBudget::default(),
        &curve,
    )
    .unwrap();
    predecessor.continuity(&successor).unwrap();
    assert_eq!(
        predecessor.static_subtree().root(),
        successor.static_subtree().root()
    );
    let before = predecessor.field_commitments(TransactionSide::Input);
    let after = successor.field_commitments(TransactionSide::Output);
    assert_eq!(before.len() + after.len(), 12);
    for (old, new) in before.iter().zip(&after) {
        assert_eq!(old.side, TransactionSide::Input);
        assert_eq!(new.side, TransactionSide::Output);
        assert_eq!(old.field, new.field);
        assert_eq!(old.range, new.range);
        assert_eq!(old.bytes, predecessor.metadata_bytes()[old.range.clone()]);
        assert_eq!(new.bytes, successor.metadata_bytes()[new.range.clone()]);
        assert_eq!(old.bytes != new.bytes, old.field == StateField::Maturity);
    }
    assert_eq!(
        before.iter().map(|entry| entry.field).collect::<Vec<_>>(),
        StateField::ALL
    );
    assert_eq!(
        after.last().unwrap().bytes,
        [vec![1], 7_u64.to_be_bytes().to_vec()].concat()
    );
}

#[test]
fn continuity_refuses_static_subtree_migration() {
    let before = candidate();
    let after = derive(4, 4096, &ScriptedCurve::new([])).unwrap();
    assert_eq!(
        before.continuity(&after),
        Err(StateConstructorRefusal::ConflictingLeaf)
    );
}

#[test]
fn reference_declarations_are_deterministic_and_acyclic() {
    let declarations = candidate().reference_declarations();
    assert_eq!(declarations, candidate().reference_declarations());
    assert_eq!(declarations.len(), 7);
    let census = StateReferenceCensus::new(&declarations).unwrap();
    assert_eq!(census.components.len(), 7);
    census.require_acyclic().unwrap();
    let mut reversed = declarations.clone();
    reversed.reverse();
    assert_eq!(census, StateReferenceCensus::new(&reversed).unwrap());
    assert!(
        declarations
            .iter()
            .all(|declaration| declaration.dependencies.is_empty())
    );
}

#[test]
fn reference_census_computes_components_and_refuses_cycles_without_a_cut() {
    let mut declarations = candidate().reference_declarations();
    let first = declarations[0].reference;
    let second = declarations[1].reference;
    declarations[0].dependencies.push(second);
    declarations[1].dependencies.push(first);
    let census = StateReferenceCensus::new(&declarations).unwrap();
    assert_eq!(census.components.len(), 6);
    assert_eq!(census.components[0].len(), 2);
    assert_eq!(
        census.require_acyclic(),
        Err(StateConstructorRefusal::ConstructorReferenceCycleUnresolved)
    );
    declarations.reverse();
    assert_eq!(census, StateReferenceCensus::new(&declarations).unwrap());
    let reference = declarations[0].reference;
    let self_cycle = [StateReferenceDeclaration {
        reference,
        dependencies: vec![reference],
    }];
    assert_eq!(
        StateReferenceCensus::new(&self_cycle)
            .unwrap()
            .require_acyclic(),
        Err(StateConstructorRefusal::ConstructorReferenceCycleUnresolved)
    );
}

#[test]
fn reference_census_accepts_dag_edges_and_bounds_graph_work() {
    let mut declarations = candidate().reference_declarations();
    let dependency = declarations[1].reference;
    declarations[0].dependencies.push(dependency);
    StateReferenceCensus::new(&declarations)
        .unwrap()
        .require_acyclic()
        .unwrap();
    let oversized: Vec<_> = (0..65)
        .map(|value| StateReferenceDeclaration {
            reference: StateConstructorReference::MetadataSchema(value),
            dependencies: Vec::new(),
        })
        .collect();
    assert_eq!(
        StateReferenceCensus::new(&oversized).unwrap_err(),
        StateConstructorRefusal::ResourceBoundExceeded
    );
}

#[test]
fn refusal_sum_has_eighteen_stable_names_and_exact_retry_split() {
    let expected = [
        "metadata-encoding-refused",
        "metadata-leaf-not-unspendable",
        "static-subtree-empty",
        "static-subtree-incomplete",
        "duplicate-leaf",
        "conflicting-leaf",
        "wrong-leaf-version",
        "wrong-internal-key",
        "internal-key-not-a-point",
        "canonical-branch-side-not-satisfied",
        "representation-search-exhausted",
        "tweak-above-group-order",
        "tweaked-point-is-identity",
        "control-path-too-deep",
        "executing-leaf-absent",
        "executing-leaf-repeated",
        "constructor-reference-cycle-unresolved",
        "resource-bound-exceeded",
    ];
    assert_eq!(StateConstructorRefusal::ALL.len(), 18);
    assert_eq!(
        StateConstructorRefusal::ALL
            .iter()
            .map(|refusal| refusal.name())
            .collect::<Vec<_>>(),
        expected
    );
    let retryable: BTreeSet<_> = StateConstructorRefusal::ALL
        .iter()
        .copied()
        .filter(|refusal| refusal.retryable())
        .collect();
    assert_eq!(
        retryable,
        BTreeSet::from([
            StateConstructorRefusal::TweakAboveGroupOrder,
            StateConstructorRefusal::TweakedPointIsIdentity,
            StateConstructorRefusal::CanonicalBranchSideNotSatisfied
        ])
    );
    for refusal in StateConstructorRefusal::ALL {
        let error: &dyn std::error::Error = refusal;
        assert_eq!(error.to_string(), refusal.name());
    }
}

#[test]
fn branch_side_admits_equality_and_refuses_reverse_order() {
    let side = StateBranchSide::MetadataLeftStaticRight;
    side.check(&[1; 32], &[1; 32]).unwrap();
    side.check(&[0; 32], &[1; 32]).unwrap();
    assert_eq!(
        side.check(&[2; 32], &[1; 32]),
        Err(StateConstructorRefusal::CanonicalBranchSideNotSatisfied)
    );
}

#[test]
fn canonical_leaf_builder_commits_every_maturity_variant() {
    let target = reviewed_target();
    for maturity in [
        Maturity::Unannounced,
        Maturity::Announced {
            cycle: Cycle::new(u64::MAX),
        },
        Maturity::Complete,
    ] {
        let semantic = StateMetadata {
            maturity,
            ..metadata()
        };
        let encoded = EncodedStateMetadata {
            semantic,
            representation: StateRepresentationNonce::new(u32::MAX),
        };
        let leaf = state_metadata_leaf_program(&target, &encoded).unwrap();
        let TapscriptInstruction::Push(item) = &leaf.instructions()[0] else {
            panic!("missing metadata push");
        };
        assert_eq!(
            item.bytes(),
            encode_state_metadata(&semantic, encoded.representation)
        );
        assert_eq!(&item.bytes()[74..78], &u32::MAX.to_be_bytes());
    }
}

#[test]
fn last_reviewed_candidate_can_succeed_with_complete_leastness_evidence() {
    // Independently counted branch-admissible candidates for this fixed fixture.
    let curve = ScriptedCurve::new(vec![StateTweakOutcome::TweakAboveGroupOrder; 3515]);
    let built = derive(4, 4096, &curve).unwrap();
    assert_eq!(built.nonce().get(), 4095);
    assert_eq!(built.evidence().rejected.len(), 4095);
    assert_eq!(curve.calls.get(), 3516);
    for (index, (nonce, refusal)) in built.evidence().rejected.iter().enumerate() {
        assert_eq!(usize::try_from(nonce.get()).unwrap(), index);
        assert!(refusal.retryable());
    }
    let exhausted = ScriptedCurve {
        fallback: StateTweakOutcome::TweakedPointIsIdentity,
        ..ScriptedCurve::new([])
    };
    assert_eq!(
        derive(4, 4096, &exhausted).unwrap_err(),
        StateConstructorRefusal::RepresentationSearchExhausted
    );
    assert_eq!(exhausted.calls.get(), 3516);
}

fn closure_metadata_leaf(nonce: StateRepresentationNonce) -> [u8; 32] {
    let bytes = encode_state_metadata(&metadata(), nonce);
    let mut preimage = vec![0xc4, 90, 0x4c, 86];
    preimage.extend(bytes);
    preimage.extend([0, 0x69]);
    super::state_announcement_tests::tagged(b"TapLeaf/elements", &preimage)
}

struct PublicArithmeticCurve;

impl StateCurveCapability for PublicArithmeticCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        assert_eq!(key, &STATE_NUMS_KEY);
        // A zero tweak reconstructs the even lift and checks its curve equation.
        let lifted = super::state_announcement_tests::output_key([0; 32]);
        lifted[0] == 2 && lifted[1..] == key[..]
    }

    fn output_key(&self, key: &[u8; 32], root: &[u8; 32]) -> StateTweakOutcome {
        assert_eq!(key, &STATE_NUMS_KEY);
        let tweak = super::state_announcement_tests::tagged(
            b"TapTweak/elements",
            &[key.as_slice(), root.as_slice()].concat(),
        );
        let compressed = super::state_announcement_tests::output_key(tweak);
        StateTweakOutcome::OutputKey {
            key: compressed[1..].try_into().unwrap(),
            parity: compressed[0] == 3,
        }
    }
}

#[test]
fn closure_recomputes_leaf_branch_and_real_tweak_output_with_public_arithmetic() {
    let expected = independent_golden();
    let leaf = closure_metadata_leaf(expected.nonce);
    assert_eq!(hex(&leaf), GOLDEN_LEAF);
    assert_eq!(leaf, expected.leaf);
    let root = super::state_announcement_tests::tagged(
        b"TapBranch/elements",
        &[leaf, expected.static_root].concat(),
    );
    assert_eq!(root, expected.root);
    assert_eq!(hex(&root), GOLDEN_ROOT);
    let tweak = super::state_announcement_tests::tagged(
        b"TapTweak/elements",
        &[STATE_NUMS_KEY, root].concat(),
    );
    assert_eq!(hex(&tweak), GOLDEN_TWEAK);
    let compressed = super::state_announcement_tests::output_key(tweak);
    let curve = PublicArithmeticCurve;
    let built = CandidateStateConstructor::derive(
        &reviewed_target(),
        &metadata(),
        &production_tree(),
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &curve).unwrap(),
        StateNonceBudget::default(),
        &curve,
    )
    .unwrap();
    assert_eq!(built.nonce(), expected.nonce);
    assert_eq!(built.merkle_root(), &root);
    assert_eq!(built.tweak_hash(), tweak);
    assert_eq!(built.output_key().as_slice(), &compressed[1..]);
    assert_eq!(built.parity(), compressed[0] == 3);
    assert_ne!(built.output_key(), &[0x42; 32]);
    let mut expected_program = vec![0x51, 0x20];
    expected_program.extend(&compressed[1..]);
    assert_eq!(built.output_program(), expected_program);
}

#[test]
fn every_nonce_below_the_production_golden_fails_the_independent_branch_order() {
    let expected = independent_golden();
    let built = candidate();
    assert!(expected.nonce.get() > 0);
    assert_eq!(built.nonce(), expected.nonce);
    assert_eq!(
        built.evidence().rejected.len(),
        usize::try_from(expected.nonce.get()).unwrap()
    );
    for attempt in 0..expected.nonce.get() {
        let nonce = StateRepresentationNonce::new(attempt);
        assert!(
            closure_metadata_leaf(nonce) > expected.static_root,
            "nonce {attempt}"
        );
        assert_eq!(
            built.evidence().rejected[usize::try_from(attempt).unwrap()],
            (
                nonce,
                StateConstructorRefusal::CanonicalBranchSideNotSatisfied
            )
        );
    }
    assert!(closure_metadata_leaf(expected.nonce) <= expected.static_root);
}
