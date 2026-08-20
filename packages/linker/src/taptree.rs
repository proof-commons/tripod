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
    role: ProgramRole,
    weight: NonZeroU64,
}

impl TapLeafInput {
    /// State one leaf's tree input.
    ///
    /// The weight is a positive integer by type, which is §14.5's
    /// "exact positive integer weight" made unrepresentable otherwise.
    #[must_use]
    pub const fn new(leaf: LeafRole, role: ProgramRole, weight: NonZeroU64) -> Self {
        Self { leaf, role, weight }
    }

    /// The leaf's identity, which is also its stable tie-break key.
    #[must_use]
    pub const fn leaf(self) -> LeafRole {
        self.leaf
    }

    /// The program role the leaf carries.
    #[must_use]
    pub const fn role(self) -> ProgramRole {
        self.role
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
    /// The leaf set is a map keyed by the leaf's own identity, so a
    /// leaf declared twice is one entry rather than a duplicate, and
    /// the stable tie-break key §14.5 asks for is the key itself.
    ///
    /// # Errors
    ///
    /// [`LinkRefusal::EmptyLeafSet`] when no leaf is declared.
    pub fn new(
        leaves: impl IntoIterator<Item = TapLeafInput>,
        leaf_version: LeafVersion,
        objective: TreeObjective,
        maximum_depth: NonZeroU32,
    ) -> Result<Self, LinkRefusal> {
        let leaves: BTreeMap<LeafRole, TapLeafInput> = leaves
            .into_iter()
            .map(|input| (input.leaf, input))
            .collect();
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
    cost: u64,
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
    #[must_use]
    pub const fn cost(&self) -> u64 {
        self.cost
    }

    /// The deepest control path, which settles `ControlPathDepth`.
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }
}

/// One node of the tree under construction.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Leaf(LeafRole),
    Branch(Box<Self>, Box<Self>),
}

impl Node {
    /// Every leaf beneath this node.
    fn leaves(&self) -> BTreeSet<LeafRole> {
        match self {
            Self::Leaf(leaf) => BTreeSet::from([*leaf]),
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
/// order changes the answer, and [`LinkRefusal::TreeDepthExceeded`]
/// when a control path is deeper than the input admits.
pub fn assemble(input: &TaptreeInput) -> Result<DeterministicTaptree, LinkRefusal> {
    let ordered: Vec<TapLeafInput> = input.leaves.values().copied().collect();
    let weights: BTreeMap<LeafRole, u64> = ordered
        .iter()
        .map(|leaf| (leaf.leaf, leaf.weight.get()))
        .collect();

    let root = huffman(&ordered);
    let reversed: Vec<TapLeafInput> = ordered.iter().copied().rev().collect();
    if huffman(&reversed) != root {
        return Err(LinkRefusal::NonDeterministicTree);
    }

    let mut recipes = BTreeMap::new();
    describe(&root, &mut Vec::new(), &mut recipes);

    let cost = recipes
        .values()
        .map(|recipe| {
            weights
                .get(&recipe.leaf)
                .copied()
                .unwrap_or_default()
                .saturating_mul(u64::from(recipe.depth))
        })
        .fold(0, u64::saturating_add);

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
fn huffman(leaves: &[TapLeafInput]) -> Node {
    let mut pool: Vec<(u64, LeafRole, Node)> = leaves
        .iter()
        .map(|input| (input.weight.get(), input.leaf, Node::Leaf(input.leaf)))
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
            low.0.saturating_add(high.0),
            low.1.min(high.1),
            Node::Branch(Box::new(low.2), Box::new(high.2)),
        ));
    }

    pool.pop()
        .map_or(Node::Leaf(leaves[0].leaf), |entry| entry.2)
}

/// Remove and return the least entry under (weight, least leaf).
fn take_least(pool: &mut Vec<(u64, LeafRole, Node)>) -> (u64, LeafRole, Node) {
    let mut best = 0;
    for (index, entry) in pool.iter().enumerate() {
        if (entry.0, entry.1) < (pool[best].0, pool[best].1) {
            best = index;
        }
    }
    pool.swap_remove(best)
}

/// Walk the tree, recording each leaf's control path.
fn describe(
    node: &Node,
    siblings: &mut Vec<BTreeSet<LeafRole>>,
    recipes: &mut BTreeMap<LeafRole, ControlPathRecipe>,
) {
    match node {
        Node::Leaf(leaf) => {
            let depth = u32::try_from(siblings.len()).unwrap_or(u32::MAX);
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
            describe(left, siblings, recipes);
            siblings.pop();

            siblings.push(left.leaves());
            describe(right, siblings, recipes);
            siblings.pop();
        }
    }
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
/// past [`ORACLE_LEAF_BUDGET`]. No partial answer is returned.
pub fn exact_minimum_cost(weights: &[u64]) -> Result<u64, LinkRefusal> {
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
    let mut total = vec![0u64; full];
    for (mask, slot) in total.iter_mut().enumerate() {
        *slot = (0..count)
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| weights[bit])
            .fold(0, u64::saturating_add);
    }

    let mut best = vec![u64::MAX; full];
    for mask in 1..full {
        if mask.is_power_of_two() {
            best[mask] = 0;
            continue;
        }
        // Every split is considered once by fixing the lowest set bit
        // on one side, which is what makes the two halves unordered.
        let lowest = 1usize << mask.trailing_zeros();
        let rest = mask & !lowest;
        let mut part = rest;
        loop {
            let left = part | lowest;
            let right = mask & !left;
            if right != 0 {
                let candidate = best[left].saturating_add(best[right]);
                if candidate < best[mask] {
                    best[mask] = candidate;
                }
            }
            if part == 0 {
                break;
            }
            part = (part - 1) & rest;
        }
        best[mask] = best[mask].saturating_add(total[mask]);
    }

    Ok(best[full - 1])
}

/// The minimum cost found by literally enumerating every tree.
///
/// The check on the check. Exponential and unapologetic: it exists to
/// establish that [`exact_minimum_cost`]'s recurrence really does range
/// over the whole tree space, at sizes where every tree can be built
/// and measured. Nothing in the link path calls it.
#[must_use]
pub fn enumerated_minimum_cost(weights: &[u64]) -> u64 {
    fn walk(items: &[u64]) -> u64 {
        if items.len() <= 1 {
            return 0;
        }
        let total: u64 = items.iter().copied().fold(0, u64::saturating_add);
        let count = items.len();
        let full = 1usize << count;
        let mut best = u64::MAX;
        // Fix the lowest element on the left so each unordered split is
        // visited exactly once.
        for mask in 0..full {
            if mask & 1 == 0 {
                continue;
            }
            let left: Vec<u64> = (0..count)
                .filter(|bit| mask & (1 << bit) != 0)
                .map(|bit| items[bit])
                .collect();
            let right: Vec<u64> = (0..count)
                .filter(|bit| mask & (1 << bit) == 0)
                .map(|bit| items[bit])
                .collect();
            if right.is_empty() {
                continue;
            }
            best = best.min(walk(&left).saturating_add(walk(&right)));
        }
        best.saturating_add(total)
    }

    walk(weights)
}
