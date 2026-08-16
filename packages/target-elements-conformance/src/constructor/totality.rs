//! What the constructor does when the target's tweak rules reject a
//! derived value.
//!
//! # The question
//!
//! An output key exists only if the tweak is a valid scalar and the sum
//! is not the identity. Both hold for very nearly every input and
//! neither holds for all of them, so the constructor is not total, and
//! a constructor that did not say what happens in the remaining case
//! would have chosen a policy silently
//! (Guide-10 `rule:guide10:tweak-totality`).
//!
//! # No policy is chosen here
//!
//! This module makes the candidates representable and computable so
//! that they can be compared. It does not select one: selection is a
//! reviewed decision, and the type is a value a caller supplies rather
//! than a default this module picks.
//!
//! # What can and cannot be measured
//!
//! A corpus measurement can show that no instance in the corpus needed
//! a retry. It cannot show that none ever will. The measurement in this
//! package's tests reports what it saw and claims nothing about the
//! tail, because a corpus of any size a test can run is far too small
//! to observe an event of this rarity even once.

use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::constructor::metadata::PrototypeMetadata;
use crate::constructor::tree::{ConstructedOutput, ConstructionDefect, FixtureTapTree, construct};

/// What a constructor does with an instance that has no output key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TweakTotalityPolicy {
    /// Refuse the instance.
    ///
    /// The simplest policy, and the one that leaves the constructor
    /// visibly partial: such an object cannot be constructed at all,
    /// and whatever holds it is stuck. Nothing about the encoding
    /// changes, which is its whole appeal.
    RejectInstance,

    /// Retry with the next canonical representation nonce.
    ///
    /// Deterministic and publicly computable: anybody with the object's
    /// state derives the same nonce by the same search, so the nonce is
    /// not a secret, a choice, or a thing to be communicated. The
    /// search runs from zero upward and stops at `maximum_attempts`.
    ///
    /// The bound is honest rather than decorative. An unbounded search
    /// would be a total constructor only under an argument nobody has,
    /// so the bounded search names its own failure case instead.
    CanonicalNonceRetry {
        /// How many nonces the search may try before giving up.
        maximum_attempts: u32,
    },

    /// Accept that the constructor is partial and name the residual.
    ///
    /// Distinct from [`Self::RejectInstance`] in what it claims rather
    /// than in what it computes: rejection is a behaviour, and this is
    /// a recorded decision that the behaviour is acceptable because the
    /// case is negligible. Kept separate so that a report cannot show a
    /// residual nobody accepted.
    NamedNegligibleResidual,
}

/// What a construction under a policy produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TotalityOutcome {
    metadata: PrototypeMetadata,
    output: ConstructedOutput,
    attempts: u32,
}

impl TotalityOutcome {
    /// The metadata as it was finally written down, nonce included.
    #[must_use]
    pub const fn metadata(&self) -> &PrototypeMetadata {
        &self.metadata
    }

    /// The values the instance determines.
    #[must_use]
    pub const fn output(&self) -> &ConstructedOutput {
        &self.output
    }

    /// How many nonces were tried, counting the one that worked.
    ///
    /// One means the first attempt succeeded, which is what every
    /// measurement so far reports.
    #[must_use]
    pub const fn attempts(&self) -> u32 {
        self.attempts
    }
}

/// Why a construction under a policy produced nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TotalityDefect {
    /// The instance has no output key and the policy does not retry.
    InstanceRefused(ConstructionDefect),
    /// The retry search reached its bound without finding a nonce.
    RetryExhausted {
        /// How many nonces were tried.
        attempts: u32,
    },
    /// The defect is not one a retry could repair.
    ///
    /// A retry changes the metadata leaf, and so the root and the
    /// tweak. It cannot repair a tree that does not contain the
    /// executing leaf, or an internal key that is not a curve point:
    /// retrying those would loop over a fixed failure, so the policy
    /// declines rather than spinning.
    NotRepairableByRetry(ConstructionDefect),
}

/// Builds one constructor instance under one totality policy.
///
/// `tree_of` maps a metadata encoding to the complete tree, so the
/// caller keeps ownership of how the metadata leaf sits beside the
/// static subtree, and this module keeps ownership of the retry.
///
/// # Errors
///
/// [`TotalityDefect`] when the policy produces no instance.
pub fn construct_under_policy<Tree>(
    internal_key: &[u8; FIELD_ELEMENT_BYTES],
    metadata: PrototypeMetadata,
    executing_leaf: &FixtureTapTree,
    policy: TweakTotalityPolicy,
    tree_of: Tree,
) -> Result<TotalityOutcome, TotalityDefect>
where
    Tree: Fn(&PrototypeMetadata) -> FixtureTapTree,
{
    let attempts = match policy {
        TweakTotalityPolicy::CanonicalNonceRetry { maximum_attempts } => maximum_attempts,
        TweakTotalityPolicy::RejectInstance | TweakTotalityPolicy::NamedNegligibleResidual => 1,
    };

    let mut last = None;
    for attempt in 0..attempts.max(1) {
        let written = metadata.with_nonce(attempt);
        match construct(internal_key, &tree_of(&written), executing_leaf) {
            Ok(output) => {
                return Ok(TotalityOutcome {
                    metadata: written,
                    output,
                    attempts: attempt.saturating_add(1),
                });
            }
            // A defect the metadata cannot move is not retried: the
            // next nonce would meet it again unchanged.
            Err(defect @ ConstructionDefect::Tree(_)) => {
                return Err(TotalityDefect::NotRepairableByRetry(defect));
            }
            Err(defect) => last = Some(defect),
        }
    }

    Err(match policy {
        TweakTotalityPolicy::CanonicalNonceRetry { .. } => {
            TotalityDefect::RetryExhausted { attempts }
        }
        TweakTotalityPolicy::RejectInstance | TweakTotalityPolicy::NamedNegligibleResidual => {
            TotalityDefect::InstanceRefused(last.unwrap_or(ConstructionDefect::Tree(
                crate::constructor::tree::TreeDefect::ExecutingLeafAbsent,
            )))
        }
    })
}
