//! The deterministic live-transfer taptree (§11.4).
//!
//! # What is new here, and what is deliberately not
//!
//! Nothing about the construction. The pool, the walk, the exact cost
//! arithmetic, and both oracles are [`crate::taptree`]'s, generic over
//! [`crate::taptree::TreeLeaf`], and this module supplies the live leaf
//! vocabulary and the two decisions that are Guide 13's rather than
//! Phase 4's: which weights the leaves carry, and which representation a
//! tree is allowed to hold.
//!
//! That is §11.1 taken literally. A second Huffman would have been a
//! second thing to establish optimality for, and §11.4's obligations —
//! duplicate rejection, role derivation, exact checked cost arithmetic,
//! one stable leaf census, exact weights, a stable tie-break, a declared
//! depth policy, and an exact independent oracle — are obligations about
//! *a* construction, not about each place one is used.
//!
//! # One tree holds one representation
//!
//! §11.3 keeps the explicit and private leaf sets disjoint and forbids
//! an attacker-selected in-script dispatch between semantic
//! representations. The leaf type carries its representation
//! ([`LiveTransferLeafRole::representation`]), so a mixed tree is
//! expressible — and [`live_taptree_input`] refuses it. That refusal is
//! the structural half of the foreclosure: a taproot output whose tree
//! held both representations' leaves would let a spender choose which
//! semantics to run, which is exactly the dispatch §11.3 names, and no
//! amount of care inside the programs would take that choice away.
//!
//! # The weights are equal because no frequency data exists
//!
//! §14.5 states the rule and §11.4 inherits it: equal weights where no
//! execution-frequency data exists. Guide 13 has none — the shape a
//! transfer takes is the requester's, and nothing in this candidate
//! observes how often each is asked for. Weighting the one-to-one shape
//! more heavily because it looks common would be inventing a
//! measurement, and the taptree that came out would be optimal for a
//! usage pattern nobody recorded.
//!
//! Equal weights are also what puts the live tree inside an exact route
//! at all: twenty-nine leaves is past the subset oracle's budget, and
//! [`crate::taptree::equal_weight_minimum_cost`] is exact arithmetic
//! that reaches it. The evidence that established one tree's optimality
//! travels on the tree, so a consumer never has to guess which route
//! answered.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU32, NonZeroU64};

use tapscript::LiveTransferLeafRole;
use tapscript::upstream::LiveTransferRepresentationPlan;
use target_elements::LeafVersion;

use crate::error::LinkRefusal;
use crate::taptree::{
    DeterministicTaptree, OptimumEvidencePolicy, TapLeafInput, TaptreeInput, TreeObjective,
    assemble_under,
};

/// The weight every live-transfer leaf carries.
///
/// One, because §14.5's rule for the absence of execution-frequency data
/// is equal weights and Guide 13 has no such data. Stated as a constant
/// rather than computed so that the day a candidate does have frequency
/// data, the change is visible as a change to this decision rather than
/// as a quietly different tree.
pub const LIVE_LEAF_WEIGHT: NonZeroU64 = NonZeroU64::MIN;

/// The live tree's optimum-evidence policy.
///
/// The permissive one, and it has to be: the candidate commits
/// twenty-nine leaves and the subset oracle stops at sixteen. Nothing is
/// given up by it — the closed form is exact, it is required to agree
/// with the oracle at every size the oracle reaches, and a leaf set with
/// unequal weights past the oracle's budget refuses under this policy
/// exactly as it does under the stricter one.
pub const LIVE_OPTIMUM_POLICY: OptimumEvidencePolicy =
    OptimumEvidencePolicy::SubsetOracleOrEqualWeightClosedForm;

/// The tree input for one representation's leaf set (§11.4).
///
/// The leaves arrive as a declaration sequence rather than as a set, and
/// that is the point of the signature: §11.4 requires duplicate leaf
/// declarations to be *rejected*, and a parameter that was already a set
/// would have absorbed the duplicate before this function saw it. The
/// program role is not a parameter either — it is derived from each
/// leaf's identity by [`crate::taptree::TapLeafInput::role`], so a leaf
/// declared the coordinator of a shape while carrying the member
/// program's role has no spelling.
///
/// # Errors
///
/// [`LinkRefusal::MixedRepresentationTree`] when the declarations do not
/// all belong to one representation plan,
/// [`LinkRefusal::DuplicateLiveTreeLeaf`] when one leaf is declared more
/// than once, and [`LinkRefusal::EmptyLeafSet`] when nothing is
/// declared — which is also §7.5's key-path escape arriving by omission,
/// since a taproot output with no committed leaf can be spent only
/// through its key path.
pub fn live_taptree_input(
    leaves: impl IntoIterator<Item = LiveTransferLeafRole>,
    leaf_version: LeafVersion,
    maximum_depth: NonZeroU32,
) -> Result<TaptreeInput<LiveTransferLeafRole>, LinkRefusal> {
    let declared: Vec<LiveTransferLeafRole> = leaves.into_iter().collect();

    // The representation check runs before the duplicate check, because
    // it is the harder statement: a tree holding both representations is
    // §11.3's dispatch, and reporting it as a bookkeeping complaint
    // about some later duplicate would bury what actually went wrong.
    let mut representations = BTreeSet::new();
    for leaf in &declared {
        representations.insert(leaf.representation());
    }
    if representations.len() > 1 {
        return Err(LinkRefusal::MixedRepresentationTree {
            representations: representations.into_iter().collect(),
        });
    }

    TaptreeInput::new(
        declared
            .into_iter()
            .map(|leaf| TapLeafInput::new(leaf, LIVE_LEAF_WEIGHT)),
        leaf_version,
        TreeObjective::MinimumTotalWeightedDepth,
        maximum_depth,
    )
}

/// Assemble the deterministic live-transfer tree (§11.4).
///
/// Every §11.4 obligation runs inside [`assemble_under`]: the tree is
/// compared with an exact optimum reached by an independent route, it is
/// rebuilt from a reversed declaration order and required to be
/// identical, the declared depth policy is enforced, and every step of
/// the cost arithmetic is checked rather than saturated.
///
/// # Errors
///
/// Every refusal [`assemble_under`] raises, with the live leaf
/// vocabulary in the ones that name a leaf.
pub fn assemble_live(
    input: &TaptreeInput<LiveTransferLeafRole>,
) -> Result<DeterministicTaptree<LiveTransferLeafRole>, LinkRefusal> {
    assemble_under(input, LIVE_OPTIMUM_POLICY)
}

/// The one representation a committed live tree holds.
///
/// `None` for a tree with no leaf, which [`live_taptree_input`] refuses
/// and so cannot reach a committed tree. Derived from the recipes rather
/// than carried beside them, so there is no second statement for the two
/// to disagree about.
#[must_use]
pub fn committed_representation(
    tree: &DeterministicTaptree<LiveTransferLeafRole>,
) -> Option<LiveTransferRepresentationPlan> {
    tree.recipes()
        .keys()
        .next()
        .map(|leaf| leaf.representation())
}

/// Every committed leaf's exact control-path depth, in canonical order.
///
/// The figure §11.6's linked bundle owes for `ControlPathDepth`, per
/// leaf rather than only as a maximum: a spender of one leaf carries
/// that leaf's control block and no other, so a per-leaf census is what
/// a witness-size statement is actually built from.
#[must_use]
pub fn control_path_depths(
    tree: &DeterministicTaptree<LiveTransferLeafRole>,
) -> BTreeMap<LiveTransferLeafRole, u32> {
    tree.recipes()
        .iter()
        .map(|(leaf, recipe)| (*leaf, recipe.depth()))
        .collect()
}
