//! The deterministic taptree and its exhaustive oracle (§14.5).
//!
//! # Three independent routes to one number
//!
//! The construction is Huffman's. The link-time check is a subset
//! dynamic program. The check on that check is literal enumeration of
//! every binary tree. All three compute the minimum total weighted
//! depth, by different means, and the tests below require them to
//! agree exactly — so an error would have to occur three times, in
//! three different algorithms, in the same direction.
//!
//! # And one route that is not an algorithm at all
//!
//! For equal weights the optimum has a closed form: with `n` leaves
//! and `k` the least integer with `2^k` at least `n`, the minimum total
//! depth is `n*k - (2^k - n)`. That is arithmetic rather than search,
//! so it checks the search at sizes where enumeration is hopeless —
//! including the twelve-leaf set this candidate actually commits.

use std::num::{NonZeroU32, NonZeroU64};

use tapscript::{LeafRole, ProgramRole};
use target_elements::LeafVersion;

use crate::taptree::{
    ControlPathRecipe, ORACLE_LEAF_BUDGET, TapLeafInput, TaptreeInput, TreeObjective, assemble,
    enumerated_minimum_cost, exact_minimum_cost,
};

/// A leaf identity for the synthetic trees below.
///
/// Member leaves, keyed by batch size, because that gives a distinct
/// stable key per index without inventing a shape.
fn leaf(index: u8) -> LeafRole {
    LeafRole::Member { ash_inputs: index }
}

/// A synthetic tree input over the given weights.
fn input(weights: &[u64]) -> TaptreeInput {
    TaptreeInput::new(
        weights.iter().enumerate().map(|(index, weight)| {
            TapLeafInput::new(
                leaf(u8::try_from(index).expect("the synthetic sizes fit a byte")),
                ProgramRole::Member,
                NonZeroU64::new(*weight).expect("the synthetic weights are positive"),
            )
        }),
        LeafVersion::TAPSCRIPT,
        TreeObjective::MinimumTotalWeightedDepth,
        NonZeroU32::new(32).expect("thirty-two is nonzero"),
    )
    .expect("a non-empty synthetic leaf set")
}

/// The closed-form optimum for `count` equal unit weights.
///
/// Arithmetic, not search: `k` is the least integer with `2^k` at least
/// `count`, `2^k - count` leaves sit one level shallower than `k`, and
/// the rest sit at `k`.
fn equal_weight_optimum(count: u64) -> u64 {
    if count <= 1 {
        return 0;
    }
    let mut k = 0;
    while (1u64 << k) < count {
        k += 1;
    }
    count * k - ((1u64 << k) - count)
}

#[test]
fn the_subset_oracle_agrees_with_literal_enumeration_at_every_small_size() {
    // The dynamic program's claim is that fixing the lowest set bit on
    // one side of each split visits every unordered binary tree exactly
    // once. That claim is checked here against a function that really
    // does build every split, over every weight vector drawn from a
    // small alphabet, at every size the enumeration can afford.
    //
    // Sizes one through six, weights from {1,2,3}: 3 + 9 + 27 + 81 +
    // 243 + 729 weight vectors, 1092 in all, each compared exactly.
    for count in 1..=6usize {
        let mut vector = vec![1u64; count];
        loop {
            assert_eq!(
                exact_minimum_cost(&vector).expect("six leaves is inside the budget"),
                enumerated_minimum_cost(&vector),
                "the two oracles disagree at {vector:?}",
            );

            // Odometer over {1,2,3}^count.
            let mut position = 0;
            while position < count {
                if vector[position] == 3 {
                    vector[position] = 1;
                    position += 1;
                } else {
                    vector[position] += 1;
                    break;
                }
            }
            if position == count {
                break;
            }
        }
    }
}

#[test]
fn the_subset_oracle_agrees_with_the_closed_form_on_equal_weights() {
    // Equal weights are the case §14.5 mandates when no
    // execution-frequency data exists, which is this candidate's case,
    // so it is the one the closed form is worth having for. Checked to
    // the oracle's whole budget, which reaches well past the twelve
    // leaves the compact-ASH constructor commits.
    for count in 1..=ORACLE_LEAF_BUDGET {
        let weights = vec![1u64; count];
        let counted = u64::try_from(count).expect("a small count fits");
        assert_eq!(
            exact_minimum_cost(&weights).expect("the budget covers its own bound"),
            equal_weight_optimum(counted),
            "the oracle and the closed form disagree at {count} equal leaves",
        );
    }
}

#[test]
fn the_constructed_tree_reaches_the_optimum_at_every_small_size() {
    // The construction itself, checked the same way: Huffman's answer
    // against the enumerated one, over the same 1092 weight vectors.
    // `assemble` already refuses a tree that misses the subset oracle,
    // so reaching a tree at all is half the assertion; the other half
    // is that the cost equals the independently enumerated minimum.
    for count in 1..=6usize {
        let mut vector = vec![1u64; count];
        loop {
            let tree = assemble(&input(&vector)).expect("a small tree assembles");
            assert_eq!(
                tree.cost(),
                enumerated_minimum_cost(&vector),
                "the constructed tree is not optimal at {vector:?}",
            );

            let mut position = 0;
            while position < count {
                if vector[position] == 3 {
                    vector[position] = 1;
                    position += 1;
                } else {
                    vector[position] += 1;
                    break;
                }
            }
            if position == count {
                break;
            }
        }
    }
}

#[test]
fn every_leaf_reaches_the_tree_exactly_once_and_its_path_has_its_depth() {
    // A tree whose cost is right but whose recipes are wrong would pass
    // the optimality checks and produce unusable control blocks. So the
    // structure is checked directly: one recipe per leaf, and each
    // recipe's sibling list exactly as long as its stated depth.
    for count in 1..=8u64 {
        let weights = vec![1u64; usize::try_from(count).expect("a small count fits")];
        let tree = assemble(&input(&weights)).expect("an equal-weight tree assembles");

        assert_eq!(tree.recipes().len(), weights.len());
        for (leaf, recipe) in tree.recipes() {
            assert_eq!(recipe.leaf(), *leaf);
            assert_eq!(
                recipe.siblings().len(),
                usize::try_from(recipe.depth()).expect("a small depth fits"),
            );
        }
    }
}

#[test]
fn the_sibling_sets_along_a_path_partition_everything_the_leaf_is_not() {
    // The property that makes a recipe a control path: walking from a
    // leaf to the root, the sibling subtrees are disjoint and together
    // cover every other leaf. A recipe that dropped or repeated a
    // sibling would still have the right length and would be wrong.
    let weights = [3u64, 1, 4, 1, 5, 9, 2, 6];
    let tree = assemble(&input(&weights)).expect("the tree assembles");
    let every: std::collections::BTreeSet<LeafRole> = tree.recipes().keys().copied().collect();

    for (leaf, recipe) in tree.recipes() {
        let mut seen: std::collections::BTreeSet<LeafRole> = std::collections::BTreeSet::new();
        for sibling in recipe.siblings() {
            for member in sibling {
                assert!(
                    seen.insert(*member),
                    "sibling subtrees overlap at {member:?}"
                );
            }
        }
        assert!(!seen.contains(leaf), "a leaf is its own sibling");

        let mut expected = every.clone();
        expected.remove(leaf);
        assert_eq!(seen, expected, "the path does not cover the whole tree");
    }
}

#[test]
fn declaration_order_does_not_reach_the_answer() {
    // §14.5 requires construction to be independent of declaration
    // order. `assemble` checks the reversed order itself and refuses
    // otherwise, so this test does the harder version: every rotation
    // of an asymmetric weight vector must produce the same cost and the
    // same multiset of depths.
    let weights = [7u64, 1, 1, 2, 3, 5, 8];
    let base = assemble(&input(&weights)).expect("the tree assembles");
    let mut base_depths: Vec<u32> = base
        .recipes()
        .values()
        .map(ControlPathRecipe::depth)
        .collect();
    base_depths.sort_unstable();

    for rotation in 1..weights.len() {
        let mut rotated = weights.to_vec();
        rotated.rotate_left(rotation);
        // The rotation moves weights between leaf keys, so the tree is
        // a different assignment; what must agree is the cost and the
        // shape, which is what order-independence is about.
        let tree = assemble(&input(&rotated)).expect("the rotated tree assembles");
        let mut depths: Vec<u32> = tree
            .recipes()
            .values()
            .map(ControlPathRecipe::depth)
            .collect();
        depths.sort_unstable();

        assert_eq!(
            tree.cost(),
            base.cost(),
            "rotation {rotation} changed the cost"
        );
        assert_eq!(depths, base_depths, "rotation {rotation} changed the shape");
    }
}

#[test]
fn a_tree_past_the_declared_maximum_depth_is_a_refusal() {
    // Huffman minimizes cost, not depth, so a skewed weight vector
    // produces a deep tree. The depth cap is a separate obligation and
    // it refuses rather than silently rebalancing — length-limited
    // trees are the package contract's open question, not a decision
    // this wave takes quietly.
    let weights: Vec<u64> = (0..8).map(|index| 1u64 << (index * 2)).collect();
    let deep = TaptreeInput::new(
        weights.iter().enumerate().map(|(index, weight)| {
            TapLeafInput::new(
                leaf(u8::try_from(index).expect("a small index fits")),
                ProgramRole::Member,
                NonZeroU64::new(*weight).expect("the weights are positive"),
            )
        }),
        LeafVersion::TAPSCRIPT,
        TreeObjective::MinimumTotalWeightedDepth,
        NonZeroU32::new(3).expect("three is nonzero"),
    )
    .expect("a non-empty leaf set");

    assert!(matches!(
        assemble(&deep),
        Err(crate::LinkRefusal::TreeDepthExceeded { .. })
    ));
}

#[test]
fn the_oracle_budget_is_a_typed_complexity_failure_and_not_a_guess() {
    // §1.11: a search past its budget returns a typed complexity
    // failure and no partial result. One past the bound is the only
    // interesting case, and it must refuse rather than return a number
    // computed some cheaper way.
    let weights = vec![1u64; ORACLE_LEAF_BUDGET + 1];
    assert!(matches!(
        exact_minimum_cost(&weights),
        Err(crate::LinkRefusal::TreeOracleBudgetExceeded { .. })
    ));
    assert!(exact_minimum_cost(&[1u64; ORACLE_LEAF_BUDGET]).is_ok());
}

#[test]
fn an_empty_leaf_set_has_no_tree() {
    assert!(matches!(
        TaptreeInput::new(
            std::iter::empty(),
            LeafVersion::TAPSCRIPT,
            TreeObjective::MinimumTotalWeightedDepth,
            NonZeroU32::new(8).expect("eight is nonzero"),
        ),
        Err(crate::LinkRefusal::EmptyLeafSet)
    ));
}
