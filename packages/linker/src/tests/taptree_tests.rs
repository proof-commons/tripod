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
//!
//! The closed form is production code now
//! ([`crate::taptree::equal_weight_minimum_cost`]) rather than a
//! fixture, because Guide 13's live tree is past the oracle's budget and
//! needs an exact route that reaches it. That makes the comparison below
//! stronger rather than weaker: the function the linker actually uses is
//! the one required to agree with an exhaustive search over the whole
//! tree space, at every size that search can reach.

use std::num::{NonZeroU32, NonZeroU64};

use tapscript::LeafRole;
use target_elements::LeafVersion;

use crate::LinkRefusal;
use crate::taptree::{
    ControlPathRecipe, ExactOptimumRoute, ORACLE_LEAF_BUDGET, OptimumEvidencePolicy,
    TREE_LEAF_BUDGET, TapLeafInput, TaptreeInput, TreeObjective, assemble, assemble_under,
    enumerated_minimum_cost, equal_weight_minimum_cost, exact_minimum_cost,
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
                NonZeroU64::new(*weight).expect("the synthetic weights are positive"),
            )
        }),
        LeafVersion::TAPSCRIPT,
        TreeObjective::MinimumTotalWeightedDepth,
        NonZeroU32::new(32).expect("thirty-two is nonzero"),
    )
    .expect("a non-empty synthetic leaf set")
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
                enumerated_minimum_cost(&vector).expect("six leaves is inside the budget"),
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
    //
    // The closed form under test is the linker's own, and it is what
    // establishes the live tree's optimality past the oracle's budget.
    // So this is not a fixture agreeing with a fixture: it is the
    // production route required to agree with an exhaustive search over
    // the whole tree space, everywhere that search can be run.
    for count in 1..=ORACLE_LEAF_BUDGET {
        let weights = vec![1u64; count];
        assert_eq!(
            exact_minimum_cost(&weights).expect("the budget covers its own bound"),
            equal_weight_minimum_cost(count, 1).expect("a non-empty set inside the budget"),
            "the oracle and the closed form disagree at {count} equal leaves",
        );
    }
}

#[test]
fn the_closed_form_agrees_with_the_subset_oracle_at_every_shared_weight() {
    // The closed form multiplies by the shared weight rather than
    // assuming unit leaves, and the multiplication is part of the claim.
    // Checked against the oracle over several weights at every size the
    // oracle reaches, so a formula right about unit weights and wrong
    // about the scaling would fail here rather than in a linked bundle.
    for weight in [1u64, 2, 7, 1_000_003] {
        for count in 1..=ORACLE_LEAF_BUDGET {
            let weights = vec![weight; count];
            assert_eq!(
                exact_minimum_cost(&weights).expect("the budget covers its own bound"),
                equal_weight_minimum_cost(count, weight).expect("a non-empty set in the budget"),
                "the closed form disagrees at {count} leaves of weight {weight}",
            );
        }
    }
}

#[test]
fn the_closed_form_agrees_with_literal_enumeration_where_enumeration_can_run() {
    // The check on the check, over the third algorithm. Enumeration is
    // combinatorially expensive, so it runs only at the small sizes —
    // but at those sizes it is the strongest statement available, and
    // the closed form has to survive it too.
    for count in 1..=6usize {
        let weights = vec![3u64; count];
        assert_eq!(
            enumerated_minimum_cost(&weights).expect("six leaves is inside the budget"),
            equal_weight_minimum_cost(count, 3).expect("a non-empty set inside the budget"),
            "the closed form and the enumeration disagree at {count} leaves",
        );
    }
}

#[test]
fn a_leaf_set_past_the_oracle_budget_assembles_only_where_a_route_reaches_it() {
    // Guide 13's live candidate commits twenty-nine leaves, which is
    // past the subset oracle. §11.4 still requires the tree to be
    // compared with an exact optimum, so the closed form is what
    // establishes it — and only under a policy that admits the closed
    // form, and only for the equal weights the formula is a theorem
    // about. Every other combination refuses.
    let live_sized = 29usize;
    let equal: Vec<TapLeafInput> = (0..live_sized)
        .map(|index| {
            TapLeafInput::new(
                leaf(u8::try_from(index).expect("twenty-nine fits a byte")),
                positive(1),
            )
        })
        .collect();
    let input = declared(equal).expect("the leaves are distinct and non-empty");

    // The strict policy refuses: the oracle is the only route it admits
    // and the oracle does not reach.
    assert_eq!(
        assemble_under(&input, OptimumEvidencePolicy::SubsetOracleOnly),
        Err(LinkRefusal::TreeOracleBudgetExceeded {
            leaves: live_sized,
            budget: ORACLE_LEAF_BUDGET,
        }),
    );
    assert_eq!(
        assemble(&input),
        Err(LinkRefusal::TreeOracleBudgetExceeded {
            leaves: live_sized,
            budget: ORACLE_LEAF_BUDGET,
        }),
        "compact ASH's entry point must keep the stricter policy",
    );

    // The permissive policy assembles, records which route answered, and
    // reaches the cost the closed form states.
    let tree = assemble_under(
        &input,
        OptimumEvidencePolicy::SubsetOracleOrEqualWeightClosedForm,
    )
    .expect("twenty-nine equal leaves have an exact closed-form optimum");
    assert_eq!(
        tree.optimum_route(),
        ExactOptimumRoute::EqualWeightClosedForm
    );
    assert_eq!(
        tree.cost(),
        equal_weight_minimum_cost(live_sized, 1).expect("a non-empty set inside the budget"),
    );
    assert_eq!(tree.recipes().len(), live_sized);
    // Five levels: `2^5` is the least power of two at least twenty-nine.
    assert_eq!(tree.depth(), 5);
}

#[test]
fn unequal_weights_past_the_oracle_budget_refuse_under_every_policy() {
    // The closed form is a theorem about equal weights and nothing else,
    // so a weight vector past the oracle's budget that is not all-equal
    // has no exact route at all. §1.11 requires that to be a typed
    // failure rather than a tree nothing checked.
    let skewed: Vec<TapLeafInput> = (0..=ORACLE_LEAF_BUDGET)
        .map(|index| {
            TapLeafInput::new(
                leaf(u8::try_from(index).expect("a small index fits")),
                positive(u64::try_from(index).expect("a small index fits") + 1),
            )
        })
        .collect();
    let input = declared(skewed).expect("the leaves are distinct and non-empty");

    for policy in [
        OptimumEvidencePolicy::SubsetOracleOnly,
        OptimumEvidencePolicy::SubsetOracleOrEqualWeightClosedForm,
    ] {
        assert_eq!(
            assemble_under(&input, policy),
            Err(LinkRefusal::TreeOracleBudgetExceeded {
                leaves: ORACLE_LEAF_BUDGET + 1,
                budget: ORACLE_LEAF_BUDGET,
            }),
            "{policy:?} established an optimum it has no route to",
        );
    }
}

#[test]
fn the_closed_form_refuses_outside_the_domain_its_argument_covers() {
    // An empty leaf set has no tree rather than a zero-cost one, and a
    // set past the construction's own budget is outside the exact-domain
    // argument the arithmetic rests on. Both refuse rather than return a
    // number nothing establishes.
    assert_eq!(
        equal_weight_minimum_cost(0, 1),
        Err(LinkRefusal::EmptyLeafSet)
    );
    assert_eq!(
        equal_weight_minimum_cost(TREE_LEAF_BUDGET + 1, 1),
        Err(LinkRefusal::TreeLeafBudgetExceeded {
            leaves: TREE_LEAF_BUDGET + 1,
            budget: TREE_LEAF_BUDGET,
        }),
    );
    assert!(equal_weight_minimum_cost(TREE_LEAF_BUDGET, u64::MAX).is_ok());
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
                enumerated_minimum_cost(&vector).expect("six leaves is inside the budget"),
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
fn both_oracles_answer_above_the_old_domain_and_still_agree() {
    // The row's weights, reached through both oracles rather than
    // through the tree. Three maximal weights and a unit one: the exact
    // optimum pairs the unit leaf with one maximal leaf and the other
    // two together, so every leaf sits at depth two and the cost is
    // twice the total — a number above `u64::MAX`, which is the whole
    // reason the domain had to move. Saturating `u64` reported
    // `u64::MAX` here, from both oracles, agreeing about nothing.
    let weights = [u64::MAX, u64::MAX, u64::MAX, 1];
    let doubled_total = 2 * (3 * u128::from(u64::MAX) + 1);

    let exact = exact_minimum_cost(&weights).expect("four leaves are inside the budget");
    let enumerated = enumerated_minimum_cost(&weights).expect("four leaves are inside the budget");

    assert_eq!(exact, doubled_total, "the subset oracle lost the optimum");
    assert_eq!(
        enumerated, doubled_total,
        "the enumerating oracle lost the optimum"
    );
    assert!(
        doubled_total > u128::from(u64::MAX),
        "the fixture no longer exceeds the domain it was chosen to exceed",
    );
}

#[test]
fn a_leaf_set_past_the_budget_refuses_before_any_arithmetic() {
    // The budget bounds the leaf count, and the leaf count is what the
    // module's `u128` sufficiency argument rests on. So `assemble` has
    // to refuse an oversized set at its head rather than build a tree,
    // total its cost, and only then discover the oracle will not run.
    let oversized: Vec<TapLeafInput> = (0..=ORACLE_LEAF_BUDGET)
        .map(|index| {
            TapLeafInput::new(
                leaf(u8::try_from(index).expect("a small index fits")),
                positive(1),
            )
        })
        .collect();
    let input = declared(oversized).expect("the leaves are distinct and non-empty");

    assert_eq!(
        assemble(&input),
        Err(LinkRefusal::TreeOracleBudgetExceeded {
            leaves: ORACLE_LEAF_BUDGET + 1,
            budget: ORACLE_LEAF_BUDGET,
        }),
    );
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

    // The enumerating oracle shares the budget so that one sufficiency
    // argument covers both domains, and refuses the same way.
    assert!(matches!(
        enumerated_minimum_cost(&weights),
        Err(crate::LinkRefusal::TreeOracleBudgetExceeded { .. })
    ));
}

/// A positive weight for the fixtures below.
fn positive(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("the fixture weights are positive")
}

/// A tree input over exactly these declarations, in exactly this order.
fn declared(leaves: impl IntoIterator<Item = TapLeafInput>) -> Result<TaptreeInput, LinkRefusal> {
    TaptreeInput::new(
        leaves,
        LeafVersion::TAPSCRIPT,
        TreeObjective::MinimumTotalWeightedDepth,
        NonZeroU32::new(32).expect("thirty-two is nonzero"),
    )
}

/// Every ordering of `count` positions, as index permutations.
fn permutations(count: usize) -> Vec<Vec<usize>> {
    if count == 0 {
        return vec![Vec::new()];
    }
    let mut orders = Vec::new();
    for shorter in permutations(count - 1) {
        for position in 0..=shorter.len() {
            let mut extended = shorter.clone();
            extended.insert(position, count - 1);
            orders.push(extended);
        }
    }
    orders
}

#[test]
fn one_leaf_declared_twice_is_a_refusal_whether_or_not_the_declarations_agree() {
    // Collecting declarations into a map would resolve a duplicate
    // last-value-wins, which makes declaration order pick the weight,
    // and would absorb an equal duplicate in silence — either way a
    // leaf set that does not know its own size. Both are refused, on
    // the same footing as the exact censuses elsewhere in this crate.
    //
    // The third way two declarations of one leaf could disagree — over
    // the program role — has no spelling to test: the role is derived
    // from the identity, so two declarations of one identity name one
    // role by construction.
    let conflicting = declared([
        TapLeafInput::new(leaf(0), positive(5)),
        TapLeafInput::new(leaf(1), positive(2)),
        TapLeafInput::new(leaf(0), positive(9)),
    ]);
    assert_eq!(
        conflicting,
        Err(LinkRefusal::DuplicateTreeLeaf(leaf(0))),
        "a leaf declared twice with two weights was resolved rather than refused",
    );

    let agreeing = declared([
        TapLeafInput::new(leaf(0), positive(5)),
        TapLeafInput::new(leaf(1), positive(2)),
        TapLeafInput::new(leaf(0), positive(5)),
    ]);
    assert_eq!(
        agreeing,
        Err(LinkRefusal::DuplicateTreeLeaf(leaf(0))),
        "a leaf declared twice with one weight was absorbed rather than refused",
    );
}

#[test]
fn every_declaration_order_of_one_leaf_set_produces_one_identical_tree() {
    // `assemble` checks the reversed order and nothing else, which was
    // enough only because the reversal happened after a lossy
    // collection had already picked winners. With duplicates refused
    // the leaf set is exactly what the declaration says, so the whole
    // statement is available: every one of the twenty-four orderings of
    // a four-leaf set, compared as whole trees rather than as costs.
    let leaves = [
        TapLeafInput::new(leaf(0), positive(3)),
        TapLeafInput::new(leaf(1), positive(1)),
        TapLeafInput::new(leaf(2), positive(4)),
        TapLeafInput::new(leaf(3), positive(1)),
    ];

    let build = |order: &[usize]| {
        let ordered: Vec<TapLeafInput> = order.iter().map(|index| leaves[*index]).collect();
        declared(ordered)
            .and_then(|input| assemble(&input))
            .expect("every ordering of one valid leaf set assembles")
    };

    let orders = permutations(leaves.len());
    assert_eq!(
        orders.len(),
        24,
        "four positions have twenty-four orderings"
    );

    let first = build(&orders[0]);
    for order in &orders {
        assert_eq!(
            build(order),
            first,
            "declaration order {order:?} produced a different tree",
        );
    }
}

#[test]
fn an_empty_leaf_set_has_no_tree() {
    assert!(matches!(
        TaptreeInput::new(
            std::iter::empty::<TapLeafInput<LeafRole>>(),
            LeafVersion::TAPSCRIPT,
            TreeObjective::MinimumTotalWeightedDepth,
            NonZeroU32::new(8).expect("eight is nonzero"),
        ),
        Err(crate::LinkRefusal::EmptyLeafSet)
    ));
}
