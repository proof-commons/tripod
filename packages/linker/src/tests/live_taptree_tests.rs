//! The deterministic live-transfer taptree (§11.4).
//!
//! # Against the real leaf set, not a synthetic one
//!
//! The leaf sets below are the ones
//! [`tapscript::static_transfer_leaf_set`] derives from the
//! demonstration shape set, so the tree under test is the tree the
//! candidate actually commits: twenty-nine leaves, one representation,
//! twenty-seven coordinators and two shared members. A synthetic
//! twenty-nine would have checked the construction and said nothing
//! about the candidate.
//!
//! # The oracle comparison at this size
//!
//! Twenty-nine leaves is past the subset oracle's budget, so §11.4's
//! exact comparison runs through the closed form — which is checked
//! against the subset oracle at every size the oracle reaches
//! ([`super::taptree_tests`]) and against literal enumeration below it.
//! The tests here add the third thing that comparison needs: that the
//! tree the linker builds over the real leaf set is the one whose cost
//! the closed form states, and that the recipes are a real partition
//! rather than a list of the right length.

use std::collections::BTreeSet;
use std::num::{NonZeroU8, NonZeroU32};

use tapscript::upstream::LiveTransferRepresentationPlan;
use tapscript::{
    LiveProgramRole, LiveTransferLeafRole, LiveTransferShapeBounds, demonstration_live_shape_set,
    dense_live_shape_set, static_transfer_leaf_set,
};
use target_elements::LeafVersion;

use crate::live_taptree::{
    assemble_live, committed_representation, control_path_depths, live_taptree_input,
};
use crate::taptree::{
    ExactOptimumRoute, TapLeafInput, enumerated_minimum_cost, equal_weight_minimum_cost,
    exact_minimum_cost,
};
use crate::{LinkRefusal, TREE_LEAF_BUDGET};

/// The candidate's own leaf set for one representation.
fn leaves(representation: LiveTransferRepresentationPlan) -> BTreeSet<LiveTransferLeafRole> {
    static_transfer_leaf_set(representation, &demonstration_live_shape_set())
}

/// A generous depth policy, so the depth tests are about the tree.
fn deep() -> NonZeroU32 {
    NonZeroU32::new(32).expect("thirty-two is nonzero")
}

/// The committed tree for one representation.
fn tree(
    representation: LiveTransferRepresentationPlan,
) -> crate::DeterministicTaptree<LiveTransferLeafRole> {
    let input = live_taptree_input(leaves(representation), LeafVersion::TAPSCRIPT, deep())
        .expect("the candidate's own leaf set is one representation and free of duplicates");
    assemble_live(&input).expect("the candidate's tree assembles")
}

#[test]
fn the_candidate_commits_twenty_nine_leaves_in_each_representation() {
    // The size the rest of this file depends on, stated once. Twenty-
    // seven shapes each contribute a coordinator, and the two receipt-
    // input counts above one contribute a shared member leaf each.
    for representation in [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ] {
        let set = leaves(representation);
        assert_eq!(set.len(), 29, "{representation:?}");
        assert_eq!(
            set.iter()
                .filter(|leaf| leaf.program_role() == LiveProgramRole::Coordinator)
                .count(),
            27,
        );
        assert_eq!(
            set.iter()
                .filter(|leaf| leaf.program_role() == LiveProgramRole::Member)
                .count(),
            2,
        );
    }
}

#[test]
fn the_committed_tree_reaches_the_exact_optimum_by_the_closed_form() {
    // §11.4's exact comparison at the candidate's real size. The route
    // is recorded on the tree rather than inferred, and the cost is the
    // number the closed form states — which is the number the subset
    // oracle states everywhere the oracle can be run.
    for representation in [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ] {
        let committed = tree(representation);
        assert_eq!(
            committed.optimum_route(),
            ExactOptimumRoute::EqualWeightClosedForm,
        );
        assert_eq!(
            committed.cost(),
            equal_weight_minimum_cost(29, 1).expect("twenty-nine is inside the budget"),
        );
        // Thirty-two is the least power of two at least twenty-nine, so
        // three leaves sit at depth four and twenty-six at depth five.
        assert_eq!(committed.depth(), 5);
        let depths = control_path_depths(&committed);
        assert_eq!(depths.values().filter(|depth| **depth == 4).count(), 3);
        assert_eq!(depths.values().filter(|depth| **depth == 5).count(), 26);
    }
}

#[test]
fn every_leaf_of_the_candidate_reaches_the_tree_once_along_a_real_path() {
    // A cost that is right over recipes that are wrong would produce
    // unusable control blocks, so the structure is checked directly: one
    // recipe per committed leaf, each sibling list exactly as long as
    // its depth, and the siblings along a path a partition of everything
    // the leaf is not.
    let committed = tree(LiveTransferRepresentationPlan::Explicit);
    let every: BTreeSet<LiveTransferLeafRole> = committed.recipes().keys().copied().collect();
    assert_eq!(every, leaves(LiveTransferRepresentationPlan::Explicit));

    for (leaf, recipe) in committed.recipes() {
        assert_eq!(recipe.leaf(), *leaf);
        assert_eq!(
            recipe.siblings().len(),
            usize::try_from(recipe.depth()).expect("a small depth fits"),
        );

        let mut seen: BTreeSet<LiveTransferLeafRole> = BTreeSet::new();
        for sibling in recipe.siblings() {
            for member in sibling {
                assert!(
                    seen.insert(*member),
                    "sibling subtrees overlap at {member:?}"
                );
            }
        }
        let mut expected = every.clone();
        expected.remove(leaf);
        assert_eq!(seen, expected, "the path does not cover the whole tree");
    }
}

#[test]
fn one_leaf_declared_twice_is_a_refusal_at_the_candidate_size() {
    // §11.4's duplicate rejection over the real leaf set. Collecting the
    // declarations into a map would absorb this silently, which is what
    // makes declaration order decide a leaf's weight.
    let mut declared: Vec<LiveTransferLeafRole> = leaves(LiveTransferRepresentationPlan::Explicit)
        .into_iter()
        .collect();
    let repeated = *declared.first().expect("the candidate has leaves");
    declared.push(repeated);

    assert_eq!(
        live_taptree_input(declared, LeafVersion::TAPSCRIPT, deep()),
        Err(LinkRefusal::DuplicateLiveTreeLeaf(repeated)),
    );
}

#[test]
fn a_tree_holding_both_representations_is_refused() {
    // §11.3's dispatch, arriving as a tree. A taproot output committing
    // the explicit and the private leaves at once lets the spender pick
    // which semantics to run, and that choice cannot be taken back
    // inside the programs — so the tree is what has to refuse.
    let mut mixed: Vec<LiveTransferLeafRole> = leaves(LiveTransferRepresentationPlan::Explicit)
        .into_iter()
        .collect();
    mixed.extend(leaves(LiveTransferRepresentationPlan::PrivateCommitted));

    assert_eq!(
        live_taptree_input(mixed, LeafVersion::TAPSCRIPT, deep()),
        Err(LinkRefusal::MixedRepresentationTree {
            representations: BTreeSet::from([
                LiveTransferRepresentationPlan::Explicit,
                LiveTransferRepresentationPlan::PrivateCommitted,
            ]),
        }),
    );
}

#[test]
fn the_two_representations_commit_disjoint_leaf_sets_and_identical_shapes() {
    // §11.3 keeps the leaf sets disjoint, which the leaf type enforces.
    // What is worth checking here is the consequence for the tree: two
    // disjoint sets of the same size under equal weights produce trees
    // of the same cost and the same depth profile, so the two
    // representations cost the same to spend and neither is the cheap
    // one a requester would be nudged towards.
    let explicit = tree(LiveTransferRepresentationPlan::Explicit);
    let private = tree(LiveTransferRepresentationPlan::PrivateCommitted);

    let explicit_leaves: BTreeSet<_> = explicit.recipes().keys().copied().collect();
    let private_leaves: BTreeSet<_> = private.recipes().keys().copied().collect();
    assert!(explicit_leaves.is_disjoint(&private_leaves));

    assert_eq!(explicit.cost(), private.cost());
    assert_eq!(explicit.depth(), private.depth());
    assert_eq!(
        committed_representation(&explicit),
        Some(LiveTransferRepresentationPlan::Explicit),
    );
    assert_eq!(
        committed_representation(&private),
        Some(LiveTransferRepresentationPlan::PrivateCommitted),
    );
}

#[test]
fn an_empty_live_leaf_set_has_no_tree() {
    // §7.5's key-path escape arriving by omission: an output with no
    // committed leaf can be spent only through its key path, so the
    // empty declaration is refused rather than committed.
    assert_eq!(
        live_taptree_input(std::iter::empty(), LeafVersion::TAPSCRIPT, deep()),
        Err(LinkRefusal::EmptyLeafSet),
    );
}

#[test]
fn a_depth_policy_the_candidate_exceeds_is_a_refusal_naming_a_live_leaf() {
    // The declared depth policy is enforced rather than reported, and
    // the refusal names the live leaf that reached too deep — which is
    // the leaf vocabulary of the tree that failed, not compact ASH's.
    let input = live_taptree_input(
        leaves(LiveTransferRepresentationPlan::Explicit),
        LeafVersion::TAPSCRIPT,
        NonZeroU32::new(4).expect("four is nonzero"),
    )
    .expect("the input is well formed");

    assert!(matches!(
        assemble_live(&input),
        Err(LinkRefusal::LiveTreeDepthExceeded {
            depth: 5,
            maximum: 4,
            ..
        }),
    ));
}

#[test]
fn every_declaration_order_of_the_candidate_leaf_set_produces_one_tree() {
    // §11.4's determinism at the real size. `assemble_live` checks the
    // reversed order itself; this checks orders it never sees, by
    // rotating the declaration sequence through every offset. Twenty-
    // nine rotations rather than twenty-nine factorial permutations,
    // because the construction's order-independence is a property of the
    // pool rather than of any particular shuffle, and a rotation moves
    // every leaf to every position.
    let base: Vec<LiveTransferLeafRole> = leaves(LiveTransferRepresentationPlan::Explicit)
        .into_iter()
        .collect();
    let first = assemble_live(
        &live_taptree_input(base.clone(), LeafVersion::TAPSCRIPT, deep()).expect("well formed"),
    )
    .expect("the tree assembles");

    for offset in 1..base.len() {
        let mut rotated = base.clone();
        rotated.rotate_left(offset);
        let committed = assemble_live(
            &live_taptree_input(rotated, LeafVersion::TAPSCRIPT, deep()).expect("well formed"),
        )
        .expect("the rotated tree assembles");
        assert_eq!(
            committed, first,
            "rotation {offset} produced a different tree"
        );
    }
}

#[test]
fn a_small_live_instance_is_compared_with_the_exhaustive_subset_oracle() {
    // §11.4's last clause, over the live leaf vocabulary rather than a
    // synthetic one. The candidate's own twenty-nine leaves are past the
    // subset oracle's budget and reach their optimum by the closed form,
    // so without this the oracle would never once have run over a real
    // live leaf set — and "compares small instances with an exact
    // independent oracle" would be a property of some other leaf type.
    //
    // A narrower bound assignment is a smaller candidate rather than a
    // different one: the shapes are built by the same unrolling, the
    // leaves by the same leaf-set derivation, and the tree by the same
    // construction. Two receipts in, two out, one optional sponsor gives
    // twelve shapes and thirteen leaves, which is inside the budget.
    let shapes = dense_live_shape_set(LiveTransferShapeBounds::new(
        NonZeroU8::new(2).expect("two is nonzero"),
        NonZeroU8::new(2).expect("two is nonzero"),
        1,
    ));
    let small = static_transfer_leaf_set(LiveTransferRepresentationPlan::Explicit, &shapes);
    assert_eq!(shapes.shapes().count(), 12);
    assert_eq!(small.len(), 13);

    let input = live_taptree_input(small.clone(), LeafVersion::TAPSCRIPT, deep())
        .expect("the narrower leaf set is one representation and free of duplicates");
    let committed = assemble_live(&input).expect("the narrower tree assembles");

    // Inside the budget the subset oracle is what establishes the
    // optimum, and `assemble_live` refuses a tree that misses it — so
    // reaching a tree at all is half the statement.
    assert_eq!(committed.optimum_route(), ExactOptimumRoute::SubsetOracle);
    assert_eq!(committed.recipes().len(), 13);

    // The other half, stated here rather than inferred: the cost the
    // construction reached is the one an exhaustive search over the
    // whole tree space names, and the one the closed form names. Three
    // routes, three algorithms, one number.
    let weights = vec![1u64; small.len()];
    assert_eq!(
        committed.cost(),
        exact_minimum_cost(&weights).expect("thirteen leaves are inside the budget"),
    );
    assert_eq!(
        committed.cost(),
        equal_weight_minimum_cost(small.len(), 1).expect("a non-empty set inside the budget"),
    );
}

#[test]
fn a_tiny_live_instance_agrees_with_literal_enumeration_of_every_tree() {
    // The check on the check, over live leaves. Enumeration is
    // combinatorially expensive, so it runs at the smallest live
    // candidate there is — one receipt in, one out, no sponsor, which is
    // a single coordinator — and at the two-in two-out sponsorless set,
    // which is five leaves and still enumerable.
    for (inputs, outputs, expected) in [(1u8, 1u8, 1usize), (2, 2, 5)] {
        let shapes = dense_live_shape_set(LiveTransferShapeBounds::new(
            NonZeroU8::new(inputs).expect("the fixture counts are nonzero"),
            NonZeroU8::new(outputs).expect("the fixture counts are nonzero"),
            0,
        ));
        let leaves = static_transfer_leaf_set(LiveTransferRepresentationPlan::Explicit, &shapes);
        assert_eq!(leaves.len(), expected);

        let input = live_taptree_input(leaves.clone(), LeafVersion::TAPSCRIPT, deep())
            .expect("the tiny leaf set is well formed");
        let committed = assemble_live(&input).expect("the tiny tree assembles");

        let weights = vec![1u64; leaves.len()];
        assert_eq!(
            committed.cost(),
            enumerated_minimum_cost(&weights).expect("a tiny set is inside the budget"),
            "the constructed live tree is not optimal at {} leaves",
            leaves.len(),
        );
    }
}

#[test]
fn the_live_weight_is_equal_across_every_leaf() {
    // §14.5's rule, as a property of the input rather than a comment:
    // every leaf carries the same exact positive weight, which is what
    // makes the closed form the exact route and what keeps one shape
    // from being cheaper to spend than another.
    let input = live_taptree_input(
        leaves(LiveTransferRepresentationPlan::Explicit),
        LeafVersion::TAPSCRIPT,
        deep(),
    )
    .expect("well formed");

    let weights: BTreeSet<_> = input
        .leaves()
        .values()
        .map(|leaf| TapLeafInput::weight(*leaf))
        .collect();
    assert_eq!(weights.len(), 1, "the leaves do not share one weight");
    assert_eq!(input.leaves().len(), 29);
    assert!(input.leaves().len() <= TREE_LEAF_BUDGET);
}
