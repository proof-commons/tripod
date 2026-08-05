//! Target-independent abstract proof capabilities (Guide-3 Tranche B).
//!
//! A capability names what an approved proof requires of a future
//! target — semantically, without importing any target package. No
//! opcode, tapleaf, stack index, or transaction slot appears here, and
//! a [`CapabilityView`] is analysis input, never target identity: it
//! is not hashed, and a real target-definition identity binds only
//! after a target package exists.

// The analysis stages have no non-test consumer until the P2-012
// analyzed program; unit tests exercise them until then. Remove with
// the first real consumer.
#![allow(dead_code)]

use std::collections::BTreeSet;

/// One abstract requirement an approved proof places on a target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequiredCapability {
    AuthenticatedObjectRecognition,
    AuthenticatedFamilyCardinality,
    AuthenticatedCanonicalPartition,
    AuthenticatedOpenFlowPartition,
    AuthenticatedRootEffects,
    AuthenticatedProjectionSet,

    ExactPublicAmountArithmetic,
    ConfidentialValueConservation,

    OwnerAuthorization,
    OperatorAuthorization,
    RefundAuthorization,
    PublicConstructibility,

    WholeTransactionValueConservation,
}

/// Optional planning filter over abstract capabilities.
///
/// Production Guide-3 analysis uses [`CapabilityView::Unconstrained`]
/// because no target package exists yet; tests use
/// [`CapabilityView::Available`] to prove fail-closed pruning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityView {
    /// Retain every realization-approved alternative and publish its
    /// abstract capability requirements.
    Unconstrained,

    /// Keep only alternatives whose requirements are contained here.
    Available(BTreeSet<RequiredCapability>),
}

impl CapabilityView {
    /// Whether a candidate with `required` capabilities survives this
    /// view. A missing capability rejects the candidate; no fallback
    /// proof is invented for it.
    #[must_use]
    pub fn supports(&self, required: &BTreeSet<RequiredCapability>) -> bool {
        match self {
            Self::Unconstrained => true,
            Self::Available(available) => required.is_subset(available),
        }
    }
}
