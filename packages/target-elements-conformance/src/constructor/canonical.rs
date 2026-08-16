//! Putting the metadata leaf on the side of the branch the program
//! hashes it on.
//!
//! # Why a creator has to grind anything
//!
//! The target hashes a branch's two children in byte-lexicographic
//! order, so a branch has one hash whichever way round it was built. A
//! program cannot reproduce that: no reviewed primitive orders two byte
//! strings, and the constructions that build an ordering out of chunk
//! comparisons all die on the same wall — a comparison pushes a truth
//! value, and no reviewed primitive takes one as an operand
//! (Guide-10 `rule:guide10:tapbranch-order`).
//!
//! So the prototype's program does not order anything. It hashes the
//! metadata leaf and then the static subtree root, in that one fixed
//! order, and the instance is spendable only when that order happens to
//! be the canonical one — because the tweak the program derives is
//! compared against the consumed input's actual program, and a tree
//! whose canonical order differs yields a different root, a different
//! tweak, and a comparison that fails.
//!
//! # The grind is the creator's, and it is public
//!
//! Ordering is therefore a property the creator establishes rather than
//! one the caller asserts. The schema carries a representation nonce
//! that is not state — the transition ignores it and a successor resets
//! it — and the creator advances it until the metadata leaf hash falls
//! on the fixed side. The search is deterministic and over public data,
//! so anybody holding the object recomputes the same nonce; nothing here
//! is secret, chosen, or communicated.
//!
//! # Two conditions, one search
//!
//! The same nonce must also produce a tweak the curve accepts, which is
//! the residual tweak totality already names. Both conditions move with
//! the nonce and neither is more likely to fail than the other is to
//! succeed, so they are one search rather than two: a nonce is admitted
//! when it satisfies both, and the bound is stated rather than implied.

use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::constructor::metadata::PrototypeMetadata;
use crate::constructor::metadata_leaf::metadata_leaf_script;
use crate::constructor::tagged::Digest32;
use crate::constructor::tree::{ConstructedOutput, ConstructionDefect, FixtureTapTree};
use target_elements::ReviewedElementsTapscriptDefinition;

/// Which side of the branch the program hashes the metadata leaf on.
///
/// The program streams the metadata leaf hash into the branch hash
/// first, so a canonically ordered branch is one whose metadata leaf
/// hash does not exceed the static subtree root.
///
/// Stated as a type rather than as a bare comparison because it is a
/// property of the *program*, not of the tree: a program written the
/// other way round would need the other side, and a reader comparing the
/// two needs to see which one this is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CanonicalSide {
    /// The metadata leaf hash is hashed first, so it must not exceed the
    /// static subtree root.
    MetadataFirst,
}

impl CanonicalSide {
    /// Whether one pair of children is already in this order.
    #[must_use]
    pub fn holds(self, metadata_leaf: &Digest32, static_root: &Digest32) -> bool {
        match self {
            Self::MetadataFirst => metadata_leaf <= static_root,
        }
    }
}

/// One canonically ordered constructor instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalConstruction {
    metadata: PrototypeMetadata,
    tree: FixtureTapTree,
    output: ConstructedOutput,
    attempts: u32,
}

impl CanonicalConstruction {
    /// The metadata as it was finally written down, ground nonce
    /// included.
    #[must_use]
    pub const fn metadata(&self) -> &PrototypeMetadata {
        &self.metadata
    }

    /// The complete tree the instance commits to.
    #[must_use]
    pub const fn tree(&self) -> &FixtureTapTree {
        &self.tree
    }

    /// Every exact value the instance determines.
    #[must_use]
    pub const fn output(&self) -> &ConstructedOutput {
        &self.output
    }

    /// How many nonces were tried, counting the one that worked.
    ///
    /// Two conditions have to hold at once and each holds for about half
    /// or almost all of the nonces respectively, so a small number here
    /// is the expected result and a large one is worth a reader's
    /// attention.
    #[must_use]
    pub const fn attempts(&self) -> u32 {
        self.attempts
    }
}

/// Why no canonically ordered instance was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalOrderDefect {
    /// The metadata is not expressible as a leaf script.
    MetadataNotExpressible,
    /// The search reached its bound without finding a nonce.
    ///
    /// Honest rather than decorative. An unbounded search would be a
    /// total construction only under an argument nobody has, so the
    /// bounded one names its own failure case.
    SearchExhausted {
        /// How many nonces were tried.
        attempts: u32,
    },
    /// The defect is not one a different nonce could repair.
    ///
    /// A nonce moves the metadata leaf, and so the root and the tweak.
    /// It cannot repair a tree that does not contain the executing leaf
    /// or an internal key that is not a curve point, so those are
    /// reported rather than retried.
    NotRepairableByRetry(ConstructionDefect),
}

/// Grinds the representation nonce until the branch is canonically
/// ordered and the instance has an output key.
///
/// The static subtree is stated whole rather than as its root, because
/// the instance commits to the tree and a caller that passed only a root
/// could not produce a control path.
///
/// # Errors
///
/// [`CanonicalOrderDefect`] when the metadata is not expressible, when a
/// defect no nonce can repair is reached, or when the search runs out.
pub fn construct_canonically_ordered(
    target: &ReviewedElementsTapscriptDefinition,
    internal_key: &[u8; FIELD_ELEMENT_BYTES],
    metadata: &PrototypeMetadata,
    static_subtree: &FixtureTapTree,
    executing_leaf: &FixtureTapTree,
    maximum_attempts: u32,
) -> Result<CanonicalConstruction, CanonicalOrderDefect> {
    let static_root = static_subtree.node_hash();

    for attempt in 0..maximum_attempts.max(1) {
        let written = metadata.with_nonce(attempt);
        let script = metadata_leaf_script(target, &written.encode())
            .map_err(|_| CanonicalOrderDefect::MetadataNotExpressible)?;
        let leaf = FixtureTapTree::leaf(script);

        // The order the program cannot compute, established here.
        if !CanonicalSide::MetadataFirst.holds(&leaf.node_hash(), &static_root) {
            continue;
        }

        let tree = FixtureTapTree::branch(leaf, static_subtree.clone());
        match crate::constructor::tree::construct(internal_key, &tree, executing_leaf) {
            Ok(output) => {
                return Ok(CanonicalConstruction {
                    metadata: written,
                    tree,
                    output,
                    attempts: attempt.saturating_add(1),
                });
            }
            // A tree defect meets the next nonce unchanged: the metadata
            // leaf moves, but whether the tree contains the executing
            // leaf, contains it once, and sits within the control depth
            // are all properties of the static subtree the nonce does
            // not touch. Retrying would loop over a fixed failure.
            Err(defect @ ConstructionDefect::Tree(_)) => {
                return Err(CanonicalOrderDefect::NotRepairableByRetry(defect));
            }
            Err(_) => {}
        }
    }

    Err(CanonicalOrderDefect::SearchExhausted {
        attempts: maximum_attempts.max(1),
    })
}
