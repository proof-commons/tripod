//! The maturity constructor's committed tree and its cost evidence.
//!
//! # What is decided here, and what is not
//!
//! Nothing about the construction. The pool, the walk, the exact
//! checked cost arithmetic and both oracles are [`crate::taptree`]'s,
//! generic over [`TreeLeaf`], and a second Huffman here would be a
//! second thing to establish optimality for. What this module supplies
//! is the STATE leaf vocabulary, the weights the leaves carry, the
//! evidence policy the static tree is admitted under, the arithmetic of
//! the fixed outer pair, the depth cap read over the complete tree, and
//! the binding of all of it to the exact leaf bytes one constructor
//! committed.
//!
//! # The weights are equal because no frequency data exists
//!
//! Equal weights are the rule where no execution-frequency data exists,
//! and this generation has none: nothing in the candidate observes how
//! often a leaf is spent, and the announcement is the only leaf the
//! admitted recipe carries anyway. Weighting one leaf above another
//! would be inventing a measurement, so the weight is a stated constant
//! ([`STATE_LEAF_WEIGHT`]) rather than a computed one, and the day a
//! deployment does carry frequency data the change is visible as a
//! change to this decision.
//!
//! # The policy is the strict one
//!
//! [`STATE_OPTIMUM_POLICY`] admits the subset oracle alone. The static
//! leaf set is one leaf, and every static set this vocabulary can
//! express sits far inside [`crate::taptree::ORACLE_LEAF_BUDGET`], so
//! the exhaustive route always answers and the equal-weight closed form
//! would buy nothing while establishing less: the oracle ranges over the
//! whole tree space for any weights at all, where the closed form is a
//! theorem about one case. A policy chosen for reach the candidate does
//! not need would weaken the statement the artifact makes about itself.
//!
//! # The outer pair is a fact, not an optimum
//!
//! The metadata leaf is the left child of the root and the static
//! subtree is the right, always, and the only topology decision in the
//! whole tree is inside the static subtree. So the outer pair's
//! contribution to the objective is arithmetic rather than a search
//! result, and [`StateTreeCost`] states the two numbers separately for
//! exactly that reason: `static_cost` is what a construction chose and
//! an oracle checked, `complete_cost` adds one unit of depth for every
//! static leaf and one for the metadata leaf, because the pair puts
//! every static leaf one level deeper and the metadata leaf at depth
//! one. For the singleton tree the static cost is zero — one leaf has no
//! topology to choose — and the complete cost is two.
//!
//! A reader checking those figures against the shared module's oracles
//! is reading a second opinion, not the proof. The tree's optimality is
//! established inside [`assemble_under`] by the strict oracle, against
//! the static leaf set and under the declared objective. That the
//! complete tree's cost also happens to equal what the equal-weight
//! closed form reports for one more unit leaf is a fact worth recording
//! beside it and nothing this module relies on, because the pair was
//! never selected by an objective in the first place.
//!
//! # The depth cap is read over the complete tree
//!
//! A spender walks the control path of the tree that exists, which is
//! the complete one, so a deployment's cap is a statement about complete
//! depth and [`StateLinkedTaptree::bind`] is where it is enforced. The
//! cap handed to [`state_static_taptree_input`] is the static one: the
//! deployment cap less one for the outer pair, floored at one because a
//! cap is a positive number. It is an early guard rather than a second
//! authority — it can refuse a static tree the deployment cap would also
//! have refused, and it cannot admit a complete tree the enforcing check
//! rejects, so the two cannot disagree in the direction that matters.
//!
//! # The evidence binds to bytes, never to a role list
//!
//! A committed leaf's hash was computed over an exact program, so the
//! identity of a tree is those bytes. Two trees whose declared roles
//! agree and whose programs differ by one instruction are different
//! trees, and a binding that compared role lists would call them the
//! same. [`StateLinkedTaptree`] therefore takes its hashes from the
//! constructor that derived them — the static root, the metadata leaf
//! hash and the merkle root — and checks the deterministic tree against
//! that committed subtree leaf by leaf rather than against a census of
//! names.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU32, NonZeroU64};

use tapscript::{CandidateStateConstructor, StateBranchSide, StateLeafRole, StateStaticSubtree};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::error::LinkRefusal;
use crate::taptree::{
    DeterministicTaptree, OptimumEvidencePolicy, TapLeafInput, TaptreeInput, TreeLeaf,
    TreeObjective, assemble_under,
};

/// The weight every STATE leaf carries.
///
/// One, because equal weights are what the absence of
/// execution-frequency data leaves, and this generation observes no
/// spend frequencies at all. Stated rather than computed, so a future
/// weighting is a visible change to this decision instead of a quietly
/// different tree.
pub const STATE_LEAF_WEIGHT: NonZeroU64 = NonZeroU64::MIN;

/// The static tree's optimum-evidence policy.
///
/// The strict one. Every static leaf set this vocabulary expresses is
/// far inside the subset oracle's budget, so the exhaustive route
/// answers and admitting a second route would only let a future leaf set
/// past the budget establish its optimum by the weaker statement instead
/// of being refused.
pub const STATE_OPTIMUM_POLICY: OptimumEvidencePolicy = OptimumEvidencePolicy::SubsetOracleOnly;

impl TreeLeaf for StateLeafRole {
    /// The STATE vocabulary carries no separate program-role enum.
    ///
    /// A leaf role names the program it runs, so the role function is
    /// the identity rather than a lookup into a second vocabulary that
    /// could disagree with this one.
    type Role = Self;

    fn leaf_program_role(self) -> Self {
        self
    }

    fn duplicate_declaration(self) -> LinkRefusal {
        LinkRefusal::DuplicateStateTreeLeaf(self)
    }

    fn depth_exceeded(self, depth: u32, maximum: u32) -> LinkRefusal {
        LinkRefusal::StateTreeDepthExceeded {
            leaf: self,
            depth,
            maximum,
        }
    }
}

/// The static tree input from a declaration sequence.
///
/// The leaves arrive as a sequence rather than as a set, which is the
/// point of the signature: a duplicate declaration is refused, and a
/// parameter that was already a set would have absorbed it before this
/// function saw it. The metadata leaf is not a static leaf and cannot be
/// declared as one — it is the outer pair's left child, committed by the
/// constructor rather than by this input — and a static tree with no
/// announcement leaf is refused because the announcement is the leaf the
/// operation is spent through.
///
/// `maximum_depth` is the *static* cap: the deployment's cap on the
/// complete tree, less one for the outer pair, floored at one. It is an
/// early guard, and [`StateLinkedTaptree::bind`] is what enforces the
/// deployment's own figure over the complete tree.
///
/// # Errors
///
/// [`LinkRefusal::MetadataLeafDeclaredStatic`] when the metadata leaf is
/// declared static and [`LinkRefusal::StaticTreeWithoutAnnouncement`]
/// when no announcement leaf is declared — that one also answers an
/// empty declaration sequence, because a static tree carrying no
/// announcement is what is wrong with it whether or not it carries
/// anything else. Then [`LinkRefusal::DuplicateStateTreeLeaf`] for a
/// leaf declared more than once.
pub fn state_static_taptree_input(
    leaves: impl IntoIterator<Item = StateLeafRole>,
    leaf_version: LeafVersion,
    maximum_depth: NonZeroU32,
) -> Result<TaptreeInput<StateLeafRole>, LinkRefusal> {
    let declared: Vec<StateLeafRole> = leaves.into_iter().collect();

    // The vocabulary check runs before the duplicate check because it is
    // the harder statement: a metadata leaf inside the static subtree
    // would move the leaf the outer pair commits, and reporting that as
    // a complaint about some later duplicate would bury what went wrong.
    if declared.contains(&StateLeafRole::MetadataCommitment) {
        return Err(LinkRefusal::MetadataLeafDeclaredStatic);
    }
    if !declared.contains(&StateLeafRole::Announcement) {
        return Err(LinkRefusal::StaticTreeWithoutAnnouncement);
    }

    TaptreeInput::new(
        declared
            .into_iter()
            .map(|leaf| TapLeafInput::new(leaf, STATE_LEAF_WEIGHT)),
        leaf_version,
        TreeObjective::MinimumTotalWeightedDepth,
        maximum_depth,
    )
}

/// Assemble the deterministic static tree under the strict policy.
///
/// Every obligation runs inside [`assemble_under`]: the tree is compared
/// with an exact optimum reached by an independent route, it is rebuilt
/// from a reversed declaration order and required to be identical, the
/// declared depth policy is enforced, and every step of the cost
/// arithmetic is checked rather than saturated.
///
/// # Errors
///
/// Every refusal [`assemble_under`] raises, with the STATE leaf
/// vocabulary in the ones that name a leaf.
pub fn assemble_state_static(
    input: &TaptreeInput<StateLeafRole>,
) -> Result<DeterministicTaptree<StateLeafRole>, LinkRefusal> {
    assemble_under(input, STATE_OPTIMUM_POLICY)
}

/// The exact cost of the static subtree and of the complete tree.
///
/// Two numbers rather than one, because they are established
/// differently. The static figure is what the construction chose and an
/// exhaustive oracle checked. The complete figure is arithmetic over a
/// pair nobody selected: the outer branch is fixed, so its contribution
/// is one unit of depth for every static leaf, plus one for the metadata
/// leaf at depth one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateTreeCost {
    static_cost: u128,
    complete_cost: u128,
}

impl StateTreeCost {
    /// The static subtree's exact total weighted depth.
    ///
    /// Zero for the singleton tree, which has no topology to choose.
    #[must_use]
    pub const fn static_cost(self) -> u128 {
        self.static_cost
    }

    /// The complete tree's exact total weighted depth.
    ///
    /// Two for the singleton tree: the announcement leaf and the
    /// metadata leaf each sit at depth one under the fixed pair.
    #[must_use]
    pub const fn complete_cost(self) -> u128 {
        self.complete_cost
    }

    /// The cost of one static tree and of the pair it sits under.
    ///
    /// # Errors
    ///
    /// [`LinkRefusal::TreeCostOverflow`] if a figure did not fit the
    /// exact domain the objective is computed in, which the shared
    /// module's leaf budget rules out. It refuses rather than reporting
    /// a number that is not any tree's cost.
    fn over(tree: &DeterministicTaptree<StateLeafRole>) -> Result<Self, LinkRefusal> {
        let static_cost = tree.cost();
        let count = tree.recipes().len();
        let leaves = u128::try_from(count).map_err(|_| LinkRefusal::TreeCostOverflow)?;
        let weight = u128::from(STATE_LEAF_WEIGHT.get());
        let complete_cost = leaves
            .checked_mul(weight)
            .and_then(|deepened| deepened.checked_add(weight))
            .and_then(|outer| static_cost.checked_add(outer))
            .ok_or(LinkRefusal::TreeCostOverflow)?;

        Ok(Self {
            static_cost,
            complete_cost,
        })
    }
}

/// A deterministic static tree bound to one constructor's exact bytes.
///
/// The hashes here are the constructor's own: it computed them over the
/// exact programs its subtree carries, and retaining them is what makes
/// this a statement about bytes rather than about a list of role names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateLinkedTaptree {
    tree: DeterministicTaptree<StateLeafRole>,
    subtree: StateStaticSubtree,
    metadata_hash: [u8; 32],
    static_root: [u8; 32],
    merkle_root: [u8; 32],
    cost: StateTreeCost,
    depths: BTreeMap<StateLeafRole, u32>,
}

impl StateLinkedTaptree {
    /// Bind a deterministic static tree to a constructor's committed tree.
    ///
    /// The deterministic tree says which leaf sits at which depth; the
    /// constructor says which bytes are committed where. Binding them is
    /// what turns two independent statements into one artifact, and
    /// every check below is a way the two could have disagreed: the leaf
    /// sets, the leaf versions, the depth of each committed path, the
    /// fixed outer pair, and the deployment's cap read over the complete
    /// tree.
    ///
    /// # Errors
    ///
    /// [`LinkRefusal::DuplicateStateTreeLeaf`] when the committed
    /// subtree carries one role twice, so its census does not know its
    /// own size; [`LinkRefusal::StateStaticLeafSetMismatch`] when the
    /// declared and committed leaf sets differ;
    /// [`LinkRefusal::StateLeafVersionMismatch`] when a committed leaf's
    /// version or the declared tree's version is not the reviewed one;
    /// [`LinkRefusal::StateStaticTopologyMismatch`] when a committed path
    /// is not the depth the construction chose;
    /// [`LinkRefusal::StateOuterPairMismatch`] when the announcement's
    /// committed path does not end at the metadata leaf;
    /// [`LinkRefusal::StateConstructor`] wrapping a refusal the
    /// constructor itself raises while stating a recipe or the pair's
    /// side; [`LinkRefusal::StateTreeDepthExceeded`], naming the deepest
    /// static leaf, when the complete tree is deeper than the deployment
    /// admits; and [`LinkRefusal::TreeCostOverflow`] on an arithmetic
    /// step the shared module's budget rules out.
    pub fn bind(
        target: &ReviewedElementsTapscriptDefinition,
        tree: DeterministicTaptree<StateLeafRole>,
        constructor: &CandidateStateConstructor,
        maximum_depth: NonZeroU32,
    ) -> Result<Self, LinkRefusal> {
        let subtree = constructor.static_subtree();

        // The committed census, with its depths, in one pass. A role
        // committed twice is refused before anything is compared against
        // it: the committed set would still equal the declared one while
        // the two censuses counted different numbers of leaves, and every
        // figure below — a depth, a cost, a path — would then be computed
        // over a tree nobody stated.
        let mut committed_depths: BTreeMap<StateLeafRole, u32> = BTreeMap::new();
        for entry in subtree.leaves() {
            let siblings = entry.siblings.len();
            let depth = u32::try_from(siblings).map_err(|_| LinkRefusal::TreeCostOverflow)?;
            if committed_depths.insert(entry.leaf.role, depth).is_some() {
                return Err(LinkRefusal::DuplicateStateTreeLeaf(entry.leaf.role));
            }
        }

        let constructed_depths: BTreeMap<StateLeafRole, u32> = tree
            .recipes()
            .iter()
            .map(|(leaf, recipe)| (*leaf, recipe.depth()))
            .collect();
        if !constructed_depths.keys().eq(committed_depths.keys()) {
            let declared: BTreeSet<StateLeafRole> = constructed_depths.keys().copied().collect();
            let committed: BTreeSet<StateLeafRole> = committed_depths.keys().copied().collect();
            return Err(LinkRefusal::StateStaticLeafSetMismatch {
                declared,
                committed,
            });
        }

        // Two statements of the leaf version reach this point — the one
        // each committed leaf carries and the one the declared tree was
        // built under — and both are compared against the reviewed
        // contract's, because a tree declared at a version the
        // constructor never committed is exactly the silent bind the
        // evidence exists to prevent.
        let reviewed = target.definition().leaf_version();
        for entry in subtree.leaves() {
            for offered in [entry.leaf.version, tree.leaf_version().get()] {
                if offered != reviewed.get() {
                    return Err(LinkRefusal::StateLeafVersionMismatch {
                        leaf: entry.leaf.role,
                        offered,
                        reviewed,
                    });
                }
            }
        }

        // Two maps over one key set iterate in the same order, so the
        // pairs below are the same leaf's two depths and no lookup can
        // miss and fall back on a default.
        let compared = constructed_depths.iter().zip(committed_depths.values());
        for ((leaf, constructed), committed) in compared {
            if constructed != committed {
                return Err(LinkRefusal::StateStaticTopologyMismatch {
                    leaf: *leaf,
                    constructed: *constructed,
                    committed: *committed,
                });
            }
        }

        let metadata_hash = constructor
            .control_recipe(StateLeafRole::MetadataCommitment)
            .map_err(LinkRefusal::StateConstructor)?
            .executing_leaf_hash;
        let static_root = *subtree.root();
        StateBranchSide::MetadataLeftStaticRight
            .check(&metadata_hash, &static_root)
            .map_err(LinkRefusal::StateConstructor)?;
        let announcement = constructor
            .control_recipe(StateLeafRole::Announcement)
            .map_err(LinkRefusal::StateConstructor)?;
        if announcement.siblings.last() != Some(&metadata_hash) {
            return Err(LinkRefusal::StateOuterPairMismatch {
                metadata_hash,
                static_root,
            });
        }

        let depth = tree
            .depth()
            .checked_add(1)
            .ok_or(LinkRefusal::TreeCostOverflow)?;
        if depth > maximum_depth.get() {
            let deepest = tree
                .recipes()
                .values()
                .max_by_key(|recipe| recipe.depth())
                .ok_or(LinkRefusal::EmptyLeafSet)?
                .leaf();
            return Err(LinkRefusal::StateTreeDepthExceeded {
                leaf: deepest,
                depth,
                maximum: maximum_depth.get(),
            });
        }

        // The metadata leaf sits at depth one by the fixed pair, and
        // every static leaf one level below the depth it holds inside
        // the subtree.
        let mut depths = BTreeMap::from([(StateLeafRole::MetadataCommitment, 1)]);
        for (leaf, constructed) in &constructed_depths {
            let complete = constructed
                .checked_add(1)
                .ok_or(LinkRefusal::TreeCostOverflow)?;
            depths.insert(*leaf, complete);
        }
        let cost = StateTreeCost::over(&tree)?;

        Ok(Self {
            tree,
            subtree: subtree.clone(),
            metadata_hash,
            static_root,
            merkle_root: *constructor.merkle_root(),
            cost,
            depths,
        })
    }

    /// The deterministic static tree this binding was made over.
    #[must_use]
    pub const fn tree(&self) -> &DeterministicTaptree<StateLeafRole> {
        &self.tree
    }

    /// The constructor's committed static subtree, with its exact
    /// programs.
    #[must_use]
    pub const fn subtree(&self) -> &StateStaticSubtree {
        &self.subtree
    }

    /// The metadata leaf's committed hash, the outer pair's left child.
    #[must_use]
    pub const fn metadata_hash(&self) -> &[u8; 32] {
        &self.metadata_hash
    }

    /// The static subtree's committed root, the outer pair's right
    /// child.
    #[must_use]
    pub const fn static_root(&self) -> &[u8; 32] {
        &self.static_root
    }

    /// The complete tree's committed root.
    ///
    /// The binding to the exact leaf bytes: the constructor computed it
    /// over the programs it committed, so two trees with the same role
    /// list and different programs are two different roots here.
    #[must_use]
    pub const fn merkle_root(&self) -> &[u8; 32] {
        &self.merkle_root
    }

    /// The static and complete costs.
    #[must_use]
    pub const fn cost(&self) -> StateTreeCost {
        self.cost
    }

    /// Every leaf's exact control-path depth in the complete tree.
    ///
    /// Per leaf rather than only as a maximum, and including the
    /// metadata leaf at depth one: a spender carries the control block
    /// of the one leaf it runs, so a per-leaf census is what a
    /// witness-size statement is built from.
    #[must_use]
    pub const fn depths(&self) -> &BTreeMap<StateLeafRole, u32> {
        &self.depths
    }
}
