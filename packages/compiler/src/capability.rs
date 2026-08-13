//! Target-independent abstract proof capabilities (Guide-3 Tranche B).
//!
//! A capability names what an approved proof requires of a future
//! target — semantically, without importing any target package. No
//! opcode, tapleaf, stack index, or transaction slot appears here, and
//! a [`CapabilityView`] is analysis input, never target identity: it
//! is not hashed, and a real target-definition identity binds only
//! after a target package exists.

// One item-level allowance remains: `CapabilityView::Available` is the
// pruning view, and the canonical Phase-2 analysis is deliberately
// unconstrained (§6.4). The variant exists so fail-closed pruning can
// be exercised before a target exists to constrain anything.

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

impl RequiredCapability {
    /// The complete census of abstract capabilities, in the type's own
    /// canonical order (Guide-8 §15.2).
    ///
    /// A census constant rather than a derived iteration, because the
    /// property a downstream adapter needs is that *this list* and the
    /// enum agree: the projection boundary re-checks the constant
    /// against the type's ordering on every use, so a member added to
    /// the enum and forgotten here fails at the boundary, and a member
    /// listed twice fails there too. The order is a stable census
    /// order; it ranks nothing.
    pub const ALL: &'static [Self] = &[
        Self::AuthenticatedObjectRecognition,
        Self::AuthenticatedFamilyCardinality,
        Self::AuthenticatedCanonicalPartition,
        Self::AuthenticatedOpenFlowPartition,
        Self::AuthenticatedRootEffects,
        Self::AuthenticatedProjectionSet,
        Self::ExactPublicAmountArithmetic,
        Self::ConfidentialValueConservation,
        Self::OwnerAuthorization,
        Self::OperatorAuthorization,
        Self::RefundAuthorization,
        Self::PublicConstructibility,
        Self::WholeTransactionValueConservation,
    ];
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
    #[allow(dead_code)]
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
