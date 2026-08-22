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

/// Declare a closed enum and its census from one list of members.
///
/// A census constant written by hand is a second list that has to be
/// kept equal to the first, and nothing available at the projection
/// boundary can check that it is: [`crate::target::canonical_census`]
/// checks the order of the census and the membership of what an
/// analysis *presents*, neither of which ranges over the enum, so a
/// variant omitted from a hand-written census is invisible for as long
/// as nothing happens to emit it (Guide-13 §8.3, row `G13-R17`). This
/// macro removes the second list rather than checking it: `ALL` is
/// generated from the same members the enum is, so an omitted variant
/// is not a defect that has to be caught — it is unwriteable.
///
/// The derives are fixed rather than supplied by the caller, because a
/// census type owes the boundary a total order: `ALL` is emitted in
/// declaration order, and derived `Ord` is declaration order, so a
/// generated census is strictly increasing by construction too. What a
/// caller does supply is the documentation, any further attributes, and
/// the members. Everything else a census type wants — mappings,
/// dispositions, `Display` — stays outside the macro, where an
/// exhaustive `match` keeps its own guard over the same members.
macro_rules! census_enum {
    (
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl $name {
            #[doc = concat!(
                "The complete census of [`", stringify!($name), "`], in the \
                 type's own canonical order."
            )]
            ///
            /// Complete by construction: the enum above and this
            /// constant are generated from one declaration, so there is
            /// no second list to fall out of step with the first. The
            /// order is a stable census order; it ranks nothing.
            pub const ALL: &'static [Self] = &[ $(Self::$variant),+ ];
        }
    };
}

pub(crate) use census_enum;

census_enum! {
    /// One abstract requirement an approved proof places on a target.
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
