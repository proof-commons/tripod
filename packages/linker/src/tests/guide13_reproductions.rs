//! Guide-13 preflight reproductions owned by this crate.
//!
//! Each test here belongs to a row of the Guide-13 preflight register
//! and reaches the branch that row reports. A row whose test is
//! `#[ignore]`d is one Wave 0 CONFIRMED: the assertion states the
//! property that *should* hold, so the test fails while the defect
//! stands, and the wave that repairs the row removes the attribute
//! rather than writing a new test.
//!
//! - `G13-R08` — CONFIRMED: the construction and both oracles total in
//!   saturating `u64`, so a legal weight set is certified optimal by two
//!   numbers that agree only because both lost the same information.
//! - `G13-R15` — CONFIRMED: a duplicate leaf declaration is resolved
//!   last-value-wins, so declaration order selects the weight; and a
//!   leaf's program role is accepted independently of its identity.

use std::collections::BTreeMap;
use std::num::{NonZeroU8, NonZeroU32, NonZeroU64};

use tapscript::{
    CompactAshShape, CompactAshShapeBounds, LeafRole, ProgramRole, SponsorChangePresence,
};
use target_elements::LeafVersion;

use crate::taptree::{
    ControlPathRecipe, TapLeafInput, TaptreeInput, TreeObjective, assemble, exact_minimum_cost,
};

/// A leaf identity for the synthetic trees below.
fn leaf(index: u8) -> LeafRole {
    LeafRole::Member { ash_inputs: index }
}

/// A positive weight for the fixtures.
fn weight(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("the fixture weights are positive")
}

/// A synthetic tree input over the given weights, in declaration order.
fn input(weights: &[u64]) -> TaptreeInput {
    TaptreeInput::new(
        weights.iter().enumerate().map(|(index, w)| {
            TapLeafInput::new(
                leaf(u8::try_from(index).expect("the synthetic sizes fit a byte")),
                weight(*w),
            )
        }),
        LeafVersion::TAPSCRIPT,
        TreeObjective::MinimumTotalWeightedDepth,
        NonZeroU32::new(32).expect("thirty-two is nonzero"),
    )
    .expect("a non-empty synthetic leaf set")
}

/// The exact minimum total weighted depth, in a domain wide enough.
///
/// The same subset recurrence [`exact_minimum_cost`] uses, in `u128`
/// and with checked arithmetic, so it is an independent *number* rather
/// than an independent algorithm: the point of the row is the domain,
/// not the recurrence. Sixteen leaves of `u64::MAX` total below
/// `2^68`, and the deepest tree multiplies that by at most sixteen, so
/// `u128` holds every value these fixtures can reach.
fn exact_minimum_cost_u128(weights: &[u64]) -> u128 {
    let count = weights.len();
    if count <= 1 {
        return 0;
    }
    let full = 1usize << count;

    let mut total = vec![0u128; full];
    for (mask, slot) in total.iter_mut().enumerate() {
        *slot = (0..count)
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| u128::from(weights[bit]))
            .sum();
    }

    let mut best = vec![u128::MAX; full];
    for mask in 1..full {
        if mask.is_power_of_two() {
            best[mask] = 0;
            continue;
        }
        let lowest = 1usize << mask.trailing_zeros();
        let rest = mask & !lowest;
        let mut part = rest;
        loop {
            let left = part | lowest;
            let right = mask & !left;
            if right != 0 {
                best[mask] = best[mask].min(best[left] + best[right]);
            }
            if part == 0 {
                break;
            }
            part = (part - 1) & rest;
        }
        best[mask] += total[mask];
    }

    best[full - 1]
}

/// The exact cost of one assembled tree, in a domain wide enough.
fn exact_cost_u128(recipes: &BTreeMap<LeafRole, ControlPathRecipe>, weights: &[u64]) -> u128 {
    recipes
        .values()
        .map(|recipe| {
            let LeafRole::Member { ash_inputs } = recipe.leaf() else {
                unreachable!("the synthetic leaves are member leaves")
            };
            u128::from(weights[usize::from(ash_inputs)]) * u128::from(recipe.depth())
        })
        .sum()
}

/// The weight set the row turns on.
///
/// Three maximal weights and one unit weight, every one of them a legal
/// `NonZeroU64` the public constructor admits. Exact arithmetic pairs
/// the unit leaf with one maximal leaf and the other two together, and
/// every leaf lands at depth two. Saturating arithmetic reports the
/// first combined pair as `u64::MAX` rather than `u64::MAX + 1`, which
/// makes it tie with the two untouched leaves, and the greedy step then
/// builds a spine instead — one leaf at depth one, one at two, two at
/// three.
const SATURATING_WEIGHTS: [u64; 4] = [u64::MAX, u64::MAX, u64::MAX, 1];

/// `G13-R08`: the assembled tree is the optimum, not a saturated tie.
///
/// `assemble` compares its construction against `exact_minimum_cost`
/// and refuses a disagreement, which is the whole assurance the module
/// claims. Both sides total in saturating `u64`, so on these weights
/// both report `u64::MAX` and agree — while the tree actually built
/// costs one more than the true optimum.
///
/// The oracle is recomputed here in `u128` so the comparison is between
/// the tree and the mathematics, never between two saturated numbers.
#[test]
#[ignore = "G13-R08: confirmed, repair pending"]
fn an_assembled_tree_over_large_legal_weights_is_actually_optimal() {
    let tree = assemble(&input(&SATURATING_WEIGHTS)).expect("the fixture assembles");

    let actual = exact_cost_u128(tree.recipes(), &SATURATING_WEIGHTS);
    let optimum = exact_minimum_cost_u128(&SATURATING_WEIGHTS);

    assert_eq!(
        actual, optimum,
        "assemble certified a tree costing {actual} as optimal, but the optimum is {optimum}",
    );
}

/// `G13-R08`: the reported cost is the tree's mathematical cost.
///
/// `DeterministicTaptree::cost()` is documented as "the tree's exact
/// cost under the declared objective". On these weights it reports
/// `u64::MAX`, which is not the cost of any tree over them.
#[test]
#[ignore = "G13-R08: confirmed, repair pending"]
fn an_assembled_tree_reports_its_mathematical_cost() {
    let tree = assemble(&input(&SATURATING_WEIGHTS)).expect("the fixture assembles");

    let actual = exact_cost_u128(tree.recipes(), &SATURATING_WEIGHTS);

    assert_eq!(
        u128::from(tree.cost()),
        actual,
        "the reported cost is a saturated number, not the tree's cost",
    );
}

/// `G13-R08`: the link-time oracle answers in an exact domain.
///
/// The narrowest statement of the row, with no tree involved at all:
/// the oracle the construction is checked against returns a number that
/// is not the minimum it is documented to compute. A caller cannot tell
/// the saturated answer from a real one, because nothing typed says the
/// budget was the domain rather than the leaf count.
#[test]
#[ignore = "G13-R08: confirmed, repair pending"]
fn the_exact_oracle_returns_the_exact_minimum_or_a_typed_refusal() {
    let saturated = exact_minimum_cost(&SATURATING_WEIGHTS).expect("four leaves are within budget");
    let optimum = exact_minimum_cost_u128(&SATURATING_WEIGHTS);

    assert_eq!(
        u128::from(saturated),
        optimum,
        "the oracle reported {saturated}, and the exact minimum is {optimum}",
    );
}

/// `G13-R15`: declaration order does not select a duplicate's value.
///
/// Both declarations name the same two leaves with the same two
/// candidate weights; only the order differs. The constructor refuses
/// the duplicate instead of collecting it away, so both orders reach
/// the same typed refusal and neither weight is ever selected.
///
/// The state this forbids is the one the row named: a silent
/// last-value-wins collection that made declaration order semantic
/// where nothing could observe it — least of all the reversed-order
/// rebuild inside `assemble`, which compares the map's *values* and so
/// only ever saw the winner.
#[test]
fn a_duplicate_leaf_declaration_does_not_make_declaration_order_semantic() {
    let light = TapLeafInput::new(leaf(0), weight(1));
    let heavy = TapLeafInput::new(leaf(0), weight(10));
    let other = TapLeafInput::new(leaf(1), weight(2));

    let build = |leaves: [TapLeafInput; 3]| {
        TaptreeInput::new(
            leaves,
            LeafVersion::TAPSCRIPT,
            TreeObjective::MinimumTotalWeightedDepth,
            NonZeroU32::new(32).expect("thirty-two is nonzero"),
        )
    };

    let light_first = build([light, heavy, other]);
    let heavy_first = build([heavy, light, other]);

    assert_eq!(
        light_first, heavy_first,
        "two declarations of one leaf set produced two different inputs",
    );
}

/// `G13-R15`: a leaf's program role follows its identity.
///
/// `LeafRole::Coordinator` names the coordinator program of one exact
/// shape, and `ProgramRole::Coordinator` names the same thing about the
/// program. The constructor takes the identity alone and derives the
/// role from it, so the two are one fact with one spelling.
///
/// The state this forbids is the one the row named: a role accepted as
/// an independent argument, leaving a leaf free to be declared the
/// coordinator of a shape while carrying the member program's role.
/// That declaration no longer type-checks, which is why this test can
/// only state the derivation and not attempt the contradiction.
#[test]
fn a_leaf_carries_the_program_role_its_identity_names() {
    let count = |value: u8| NonZeroU8::new(value).expect("the fixture counts are nonzero");
    let bounds = CompactAshShapeBounds::new(count(4), 1).expect("four is above the minimum");
    let shape = CompactAshShape::new(bounds, count(2), 0, SponsorChangePresence::Absent)
        .expect("the shape is within its bounds");

    let coordinator = TapLeafInput::new(LeafRole::Coordinator { shape }, weight(1));

    let expected = match coordinator.leaf() {
        LeafRole::Coordinator { .. } => ProgramRole::Coordinator,
        LeafRole::Member { .. } => ProgramRole::Member,
    };
    assert_eq!(
        coordinator.role(),
        expected,
        "a coordinator leaf did not carry the coordinator program role",
    );
}
