//! Deterministic taptree assembly and its exhaustive oracle (§14.5).
//!
//! # What this module commits to, and what it does not
//!
//! It settles the *shape* of the committed tree: which leaf sits at
//! which depth, what each leaf's control path looks like, and therefore
//! the exact `ControlPathDepth` the relocatable bundle left to this
//! layer. It computes no merkle root, no tweak, and no output key.
//!
//! That is a deliberate boundary rather than an unfinished one. §14.5
//! states the tree input as a leaf set, a leaf version, roles, weights,
//! a maximum depth, and a tie-break, and states the obligation as
//! comparing the selected tree against exhaustive enumeration under the
//! declared objective — all of which is structure. The hashes and the
//! curve arithmetic that turn a tree into an output key belong to the
//! layer that builds a transaction against a real deployment, and
//! computing them here would mint an identity §1.10 admits only once a
//! consumer of one exists. The obligation is carried explicitly instead,
//! as [`crate::LinkObligation::TaprootOutputKeyUndischarged`].
//!
//! # Why the optimum is checked rather than trusted
//!
//! The construction is Huffman's, which is optimal for this objective
//! by a proof this module does not contain. §1.11 asks for
//! independently checked small-instance oracles, so the tree is checked
//! against one: an exact subset dynamic program that computes the
//! minimum achievable cost over *every* binary tree on the leaf set,
//! by a completely different route. The construction is accepted only
//! when the two agree exactly. The dynamic program is in turn checked
//! against literal enumeration of every tree at small sizes, so neither
//! oracle is trusted on its own authority.
//!
//! # The domain the objective is computed in, and why it is enough
//!
//! Every number in the objective — the Huffman pool's subtree weights,
//! the assembled tree's cost, and both oracles' answers — is a `u128`,
//! computed with checked arithmetic and never saturated. One domain
//! shared by the construction and by both checks is what makes the
//! comparison mean anything: saturating in `u64` let distinct costs
//! collapse onto `u64::MAX`, where they compared equal, and two answers
//! that agree because both lost the same information agree about
//! nothing. The two oracles stay algorithmically independent — a subset
//! recurrence and a literal enumeration — but they now answer in the
//! same exact arithmetic, which is the only way their agreement is
//! evidence.
//!
//! `u128` is sufficient because the leaf count is bounded before any
//! arithmetic runs. [`ORACLE_LEAF_BUDGET`] is sixteen, and [`assemble`]
//! and both oracles refuse a larger leaf set at their head rather than
//! partway through. So: each weight is below `2^64`, sixteen of them
//! sum below `2^68`, and no leaf of a binary tree on sixteen leaves sits
//! deeper than fifteen, so the total weighted depth stays below
//! `2^68 * 2^4 = 2^72`. Every intermediate is the weight of a subset or
//! the cost of a tree over one, so every intermediate obeys the same
//! bound, leaving `u128` a factor of `2^56` in hand.
//!
//! The arithmetic is checked anyway, and
//! [`LinkRefusal::TreeCostOverflow`] is what a failure would return.
//! That refusal is unreachable at the budget above, and deliberately so:
//! it is what keeps the bound load-bearing rather than assumed, so that
//! raising the budget past what the proof covers refuses instead of
//! quietly returning a number that is not any tree's cost.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU32, NonZeroU64};

use tapscript::{LeafRole, ProgramRole};
use target_elements::LeafVersion;

use crate::error::LinkRefusal;

/// The largest leaf set the exact oracle is run over.
///
/// The dynamic program below costs three-to-the-n set operations, so
/// the budget is a real one rather than a decoration: sixteen leaves is
/// about forty-three million steps, and beyond it the analysis returns
/// a typed complexity failure and no partial result, which is what
/// §1.11 requires of a search that exceeds its budget.
pub const ORACLE_LEAF_BUDGET: usize = 16;

/// The objective the deterministic tree is selected under (§14.5).
///
/// One variant. Weights are implementation configuration rather than
/// protocol semantics, and so is the objective they feed; a second
/// objective would move bundle identity, which is a decision no link
/// takes silently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeObjective {
    /// Minimize the sum over leaves of weight times depth.
    MinimumTotalWeightedDepth,
}

/// One leaf of the candidate tree input (§14.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TapLeafInput {
    leaf: LeafRole,
    weight: NonZeroU64,
}

impl TapLeafInput {
    /// State one leaf's tree input.
    ///
    /// The weight is a positive integer by type, which is §14.5's
    /// "exact positive integer weight" made unrepresentable otherwise.
    /// The program role is not stated here because it is not a free
    /// fact: it is a function of the leaf's identity, and [`role`]
    /// evaluates that function rather than repeating an author's
    /// answer to it.
    ///
    /// [`role`]: Self::role
    #[must_use]
    pub const fn new(leaf: LeafRole, weight: NonZeroU64) -> Self {
        Self { leaf, weight }
    }

    /// The leaf's identity, which is also its stable tie-break key.
    #[must_use]
    pub const fn leaf(self) -> LeafRole {
        self.leaf
    }

    /// The program role the leaf carries, derived from its identity.
    ///
    /// [`LeafRole::program_role`] is the one definition of which
    /// program a leaf runs, and this is a call to it. A leaf therefore
    /// cannot be declared the coordinator of a shape while carrying the
    /// member program's role: the disagreement §14.5 would otherwise
    /// have to be checked for has no spelling.
    #[must_use]
    pub const fn role(self) -> ProgramRole {
        self.leaf.program_role()
    }

    /// The leaf's exact positive weight.
    #[must_use]
    pub const fn weight(self) -> NonZeroU64 {
        self.weight
    }
}

/// The complete candidate tree input (§14.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaptreeInput {
    leaves: BTreeMap<LeafRole, TapLeafInput>,
    leaf_version: LeafVersion,
    objective: TreeObjective,
    maximum_depth: NonZeroU32,
}

impl TaptreeInput {
    /// State the tree input.
    ///
    /// The leaf set is a map keyed by the leaf's own identity, which is
    /// the stable tie-break key §14.5 asks for. Declaring one leaf
    /// twice is refused rather than resolved: collecting into the map
    /// would let the last declaration win, which would make declaration
    /// order decide the weight while the reversed-order rebuild inside
    /// [`assemble`] compared two already-resolved maps and saw nothing.
    /// Equal duplicates are refused too, on the same footing as the
    /// exact censuses elsewhere in this crate — a leaf stated twice is
    /// an authored set that does not know its own size, whether or not
    /// the two statements agree.
    ///
    /// # Errors
    ///
    /// [`LinkRefusal::EmptyLeafSet`] when no leaf is declared, and
    /// [`LinkRefusal::DuplicateTreeLeaf`] when one leaf is declared
    /// more than once.
    pub fn new(
        leaves: impl IntoIterator<Item = TapLeafInput>,
        leaf_version: LeafVersion,
        objective: TreeObjective,
        maximum_depth: NonZeroU32,
    ) -> Result<Self, LinkRefusal> {
        let mut collected: BTreeMap<LeafRole, TapLeafInput> = BTreeMap::new();
        for input in leaves {
            if collected.insert(input.leaf, input).is_some() {
                return Err(LinkRefusal::DuplicateTreeLeaf(input.leaf));
            }
        }
        let leaves = collected;
        if leaves.is_empty() {
            return Err(LinkRefusal::EmptyLeafSet);
        }

        Ok(Self {
            leaves,
            leaf_version,
            objective,
            maximum_depth,
        })
    }

    /// Every leaf, in canonical order.
    #[must_use]
    pub const fn leaves(&self) -> &BTreeMap<LeafRole, TapLeafInput> {
        &self.leaves
    }

    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The declared objective.
    #[must_use]
    pub const fn objective(&self) -> TreeObjective {
        self.objective
    }

    /// The deepest control path the input admits.
    #[must_use]
    pub const fn maximum_depth(&self) -> NonZeroU32 {
        self.maximum_depth
    }
}

/// One leaf's control path, stated structurally.
///
/// The siblings are named by the leaf sets they cover, from the leaf
/// upward. That is a recipe a later layer turns into the control
/// block's hash sequence once it has a hash function and a reason to
/// use one; it is not a claim about any byte.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ControlPathRecipe {
    leaf: LeafRole,
    depth: u32,
    siblings: Vec<BTreeSet<LeafRole>>,
}

impl ControlPathRecipe {
    /// The leaf this path reaches.
    #[must_use]
    pub const fn leaf(&self) -> LeafRole {
        self.leaf
    }

    /// How many branches lie between the leaf and the root.
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }

    /// Every sibling subtree, from the leaf upward, named by its
    /// leaves.
    #[must_use]
    pub fn siblings(&self) -> &[BTreeSet<LeafRole>] {
        &self.siblings
    }
}

/// The committed tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterministicTaptree {
    leaf_version: LeafVersion,
    objective: TreeObjective,
    recipes: BTreeMap<LeafRole, ControlPathRecipe>,
    cost: u128,
    depth: u32,
}

impl DeterministicTaptree {
    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The objective the tree was selected under.
    #[must_use]
    pub const fn objective(&self) -> TreeObjective {
        self.objective
    }

    /// Every leaf's control-path recipe, in canonical order.
    #[must_use]
    pub const fn recipes(&self) -> &BTreeMap<LeafRole, ControlPathRecipe> {
        &self.recipes
    }

    /// The tree's exact cost under the declared objective.
    ///
    /// Exact in the arithmetic sense and not only in the documentary
    /// one: the module's domain argument bounds this below `2^72`, and
    /// nothing that produced it saturated, so it is the cost of this
    /// tree rather than the nearest number some narrower type could
    /// hold.
    #[must_use]
    pub const fn cost(&self) -> u128 {
        self.cost
    }

    /// The deepest control path, which settles `ControlPathDepth`.
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }
}

/// One node of the tree under construction.
///
/// A leaf carries its own weight so that the cost walk never has to
/// look one up: weight and depth meet where both are known, and there
/// is no join to fall back on a default for.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Leaf { leaf: LeafRole, weight: u128 },
    Branch(Box<Self>, Box<Self>),
}

impl Node {
    /// Every leaf beneath this node.
    fn leaves(&self) -> BTreeSet<LeafRole> {
        match self {
            Self::Leaf { leaf, .. } => BTreeSet::from([*leaf]),
            Self::Branch(left, right) => {
                let mut set = left.leaves();
                set.extend(right.leaves());
                set
            }
        }
    }
}

/// Assemble the deterministic tree and check it against the oracle.
///
/// The three §14.5 obligations all run here rather than being left to a
/// test: the tree is compared with the exact optimum, it is rebuilt
/// from a reversed declaration order and required to be identical, and
/// its depth is required to be within the declared maximum.
///
/// # Errors
///
/// [`LinkRefusal::TreeOracleBudgetExceeded`] when the leaf set is
/// larger than the exact oracle's budget,
/// [`LinkRefusal::NonOptimalTree`] when the construction and the oracle
/// disagree, [`LinkRefusal::NonDeterministicTree`] when declaration
/// order changes the answer, [`LinkRefusal::TreeDepthExceeded`] when a
/// control path is deeper than the input admits, and
/// [`LinkRefusal::TreeCostOverflow`] which the module's domain argument
/// shows cannot be reached at the current budget.
pub fn assemble(input: &TaptreeInput) -> Result<DeterministicTaptree, LinkRefusal> {
    let ordered: Vec<TapLeafInput> = input.leaves.values().copied().collect();

    // The budget is checked here rather than only inside the oracle,
    // because it is what bounds the leaf count for the module's `u128`
    // sufficiency argument. A leaf set past it must reach no arithmetic
    // at all, not merely fail the comparison afterwards.
    if ordered.len() > ORACLE_LEAF_BUDGET {
        return Err(LinkRefusal::TreeOracleBudgetExceeded {
            leaves: ordered.len(),
            budget: ORACLE_LEAF_BUDGET,
        });
    }

    let root = huffman(&ordered)?;
    let reversed: Vec<TapLeafInput> = ordered.iter().copied().rev().collect();
    if huffman(&reversed)? != root {
        return Err(LinkRefusal::NonDeterministicTree);
    }

    let mut recipes = BTreeMap::new();
    let mut cost = 0u128;
    describe(&root, &mut Vec::new(), &mut recipes, &mut cost)?;

    let optimum = exact_minimum_cost(
        &ordered
            .iter()
            .map(|leaf| leaf.weight.get())
            .collect::<Vec<_>>(),
    )?;
    if cost != optimum {
        return Err(LinkRefusal::NonOptimalTree {
            constructed: cost,
            optimum,
        });
    }

    let depth = recipes
        .values()
        .map(ControlPathRecipe::depth)
        .max()
        .unwrap_or_default();
    if depth > input.maximum_depth.get() {
        let deepest = recipes
            .values()
            .max_by_key(|recipe| recipe.depth)
            .map_or(ordered[0].leaf, |recipe| recipe.leaf);
        return Err(LinkRefusal::TreeDepthExceeded {
            leaf: deepest,
            depth,
            maximum: input.maximum_depth.get(),
        });
    }

    Ok(DeterministicTaptree {
        leaf_version: input.leaf_version,
        objective: input.objective,
        recipes,
        cost,
        depth,
    })
}

/// The deterministic Huffman construction.
///
/// Order-independent by construction rather than by convention: the two
/// subtrees combined at each step are the least under the total order
/// (weight, least leaf), the leaf sets are disjoint so that order is
/// total, and a branch's children are stored in that same order. The
/// declaration order of the input never enters.
///
/// Pool weights are subset sums of the leaf weights and so stay below
/// `2^68` under the module's budget; the addition is checked rather
/// than saturated because a saturated pool weight is what reordered the
/// pool and produced a suboptimal tree.
///
/// # Errors
///
/// [`LinkRefusal::EmptyLeafSet`] when there is no leaf to build from,
/// and [`LinkRefusal::TreeCostOverflow`] on an unreachable overflow.
fn huffman(leaves: &[TapLeafInput]) -> Result<Node, LinkRefusal> {
    let mut pool: Vec<(u128, LeafRole, Node)> = leaves
        .iter()
        .map(|input| {
            let weight = u128::from(input.weight.get());
            (
                weight,
                input.leaf,
                Node::Leaf {
                    leaf: input.leaf,
                    weight,
                },
            )
        })
        .collect();

    while pool.len() > 1 {
        let first = take_least(&mut pool);
        let second = take_least(&mut pool);
        let (low, high) = if (first.0, first.1) <= (second.0, second.1) {
            (first, second)
        } else {
            (second, first)
        };
        pool.push((
            low.0
                .checked_add(high.0)
                .ok_or(LinkRefusal::TreeCostOverflow)?,
            low.1.min(high.1),
            Node::Branch(Box::new(low.2), Box::new(high.2)),
        ));
    }

    pool.pop()
        .map(|entry| entry.2)
        .ok_or(LinkRefusal::EmptyLeafSet)
}

/// Remove and return the least entry under (weight, least leaf).
fn take_least(pool: &mut Vec<(u128, LeafRole, Node)>) -> (u128, LeafRole, Node) {
    let mut best = 0;
    for (index, entry) in pool.iter().enumerate() {
        if (entry.0, entry.1) < (pool[best].0, pool[best].1) {
            best = index;
        }
    }
    pool.swap_remove(best)
}

/// Walk the tree, recording each leaf's control path and its cost.
///
/// The cost is totalled here rather than from the finished recipes
/// because this is where a leaf's weight and its depth are both in
/// hand. Reading the weight back out of a map afterwards needed a
/// default for the leaf that is not there, and a default is a silent
/// zero in the objective.
///
/// # Errors
///
/// [`LinkRefusal::TreeCostOverflow`] if a depth did not fit its type or
/// the running total did not fit the exact domain, neither of which the
/// module's budget admits.
fn describe(
    node: &Node,
    siblings: &mut Vec<BTreeSet<LeafRole>>,
    recipes: &mut BTreeMap<LeafRole, ControlPathRecipe>,
    cost: &mut u128,
) -> Result<(), LinkRefusal> {
    match node {
        Node::Leaf { leaf, weight } => {
            let depth = u32::try_from(siblings.len()).map_err(|_| LinkRefusal::TreeCostOverflow)?;
            let term = weight
                .checked_mul(u128::from(depth))
                .ok_or(LinkRefusal::TreeCostOverflow)?;
            *cost = cost
                .checked_add(term)
                .ok_or(LinkRefusal::TreeCostOverflow)?;

            let mut path = siblings.clone();
            path.reverse();
            recipes.insert(
                *leaf,
                ControlPathRecipe {
                    leaf: *leaf,
                    depth,
                    siblings: path,
                },
            );
        }
        Node::Branch(left, right) => {
            siblings.push(right.leaves());
            describe(left, siblings, recipes, cost)?;
            siblings.pop();

            siblings.push(left.leaves());
            describe(right, siblings, recipes, cost)?;
            siblings.pop();
        }
    }
    Ok(())
}

/// The exact minimum cost over every binary tree on these weights.
///
/// An independent route to the answer, and deliberately not Huffman's:
/// the recurrence is over *subsets*, and it considers every way of
/// splitting a subset into two non-empty parts, so it ranges over the
/// whole tree space rather than over one greedy path through it. The
/// identity it rests on is that the sum over leaves of weight times
/// depth equals the sum over internal nodes of the total weight beneath
/// them, which is why the cost of a split is the subset's own weight
/// plus the two parts' costs.
///
/// # Errors
///
/// [`LinkRefusal::TreeOracleBudgetExceeded`] when the leaf count is
/// past [`ORACLE_LEAF_BUDGET`]. No partial answer is returned. Also
/// [`LinkRefusal::TreeCostOverflow`], which that budget rules out.
pub fn exact_minimum_cost(weights: &[u64]) -> Result<u128, LinkRefusal> {
    let count = weights.len();
    if count > ORACLE_LEAF_BUDGET {
        return Err(LinkRefusal::TreeOracleBudgetExceeded {
            leaves: count,
            budget: ORACLE_LEAF_BUDGET,
        });
    }
    if count <= 1 {
        return Ok(0);
    }

    let full = 1usize << count;
    let mut total = vec![0u128; full];
    for (mask, slot) in total.iter_mut().enumerate() {
        for (bit, weight) in weights.iter().enumerate().take(count) {
            if mask & (1 << bit) != 0 {
                *slot = slot
                    .checked_add(u128::from(*weight))
                    .ok_or(LinkRefusal::TreeCostOverflow)?;
            }
        }
    }

    // Every entry of `best` is a real cost rather than a sentinel: a
    // single-leaf subset costs nothing, and every larger subset is
    // seeded below from a split that always exists. Nothing here has to
    // add to a stand-in for "no answer yet", which is what made the
    // saturating version's `u64::MAX` both an answer and a marker.
    let mut best = vec![0u128; full];
    for mask in 1..full {
        if mask.is_power_of_two() {
            continue;
        }
        // Every split is considered once by fixing the lowest set bit
        // on one side, which is what makes the two halves unordered.
        let lowest = 1usize << mask.trailing_zeros();
        let rest = mask & !lowest;
        // `mask` has at least two bits, so `rest` is a non-empty proper
        // submask and the split that puts the lowest bit alone is a
        // real candidate. Seeding with it means the minimum below runs
        // over a non-empty set.
        let mut least = best[lowest]
            .checked_add(best[rest])
            .ok_or(LinkRefusal::TreeCostOverflow)?;
        let mut part = rest;
        loop {
            let left = part | lowest;
            let right = mask & !left;
            if right != 0 {
                let candidate = best[left]
                    .checked_add(best[right])
                    .ok_or(LinkRefusal::TreeCostOverflow)?;
                least = least.min(candidate);
            }
            if part == 0 {
                break;
            }
            part = (part - 1) & rest;
        }
        best[mask] = least
            .checked_add(total[mask])
            .ok_or(LinkRefusal::TreeCostOverflow)?;
    }

    Ok(best[full - 1])
}

/// The minimum cost found by literally enumerating every tree.
///
/// The check on the check. Exponential and unapologetic: it exists to
/// establish that [`exact_minimum_cost`]'s recurrence really does range
/// over the whole tree space, at sizes where every tree can be built
/// and measured. Nothing in the link path calls it.
///
/// It answers in the same exact domain and under the same leaf budget
/// as the oracle it checks, so one sufficiency argument covers both.
/// Its own combinatorial cost puts its practical ceiling far below that
/// budget; the budget is here to bound the arithmetic, not to promise
/// the run finishes.
///
/// # Errors
///
/// [`LinkRefusal::TreeOracleBudgetExceeded`] past
/// [`ORACLE_LEAF_BUDGET`], and [`LinkRefusal::TreeCostOverflow`], which
/// that budget rules out.
pub fn enumerated_minimum_cost(weights: &[u64]) -> Result<u128, LinkRefusal> {
    fn walk(items: &[u128]) -> Result<u128, LinkRefusal> {
        if items.len() <= 1 {
            return Ok(0);
        }
        let mut total = 0u128;
        for item in items {
            total = total
                .checked_add(*item)
                .ok_or(LinkRefusal::TreeCostOverflow)?;
        }
        let count = items.len();
        let full = 1usize << count;
        // Fix the lowest element on the left so each unordered split is
        // visited exactly once. The split that puts it alone always
        // exists here, so it seeds the minimum and no sentinel is
        // needed.
        let mut least = walk(&items[1..])?;
        for mask in 0..full {
            if mask & 1 == 0 {
                continue;
            }
            let left: Vec<u128> = (0..count)
                .filter(|bit| mask & (1 << bit) != 0)
                .map(|bit| items[bit])
                .collect();
            let right: Vec<u128> = (0..count)
                .filter(|bit| mask & (1 << bit) == 0)
                .map(|bit| items[bit])
                .collect();
            if right.is_empty() {
                continue;
            }
            let candidate = walk(&left)?
                .checked_add(walk(&right)?)
                .ok_or(LinkRefusal::TreeCostOverflow)?;
            least = least.min(candidate);
        }
        least
            .checked_add(total)
            .ok_or(LinkRefusal::TreeCostOverflow)
    }

    if weights.len() > ORACLE_LEAF_BUDGET {
        return Err(LinkRefusal::TreeOracleBudgetExceeded {
            leaves: weights.len(),
            budget: ORACLE_LEAF_BUDGET,
        });
    }
    let items: Vec<u128> = weights.iter().copied().map(u128::from).collect();
    walk(&items)
}
