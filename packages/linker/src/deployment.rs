//! The typed public deployment parameters a link is given.
//!
//! # Supplied, never minted
//!
//! Every value here arrives from the caller. The linker mints no asset
//! identifier, no key, and no program, because §1.10 refuses a
//! speculative identity and a linker that invented one would be
//! inventing exactly the thing a deployment is supposed to fix.
//!
//! The resolved symbol values are carried as [`CompactAshSymbols`] —
//! the backend's own type — rather than as a second bag of byte
//! vectors. That is §1.12 applied to a value the backend already
//! authored a type for: the widths are checked once, by the code that
//! knows what each field is compared with, and a resolution of the
//! wrong shape is refused where it is supplied rather than several
//! layers later.
//!
//! # Private keys never enter
//!
//! The internal key is an x-only public point, and it is the only key
//! this type carries. Nothing here holds, derives, or accepts a secret.

use std::num::NonZeroU32;

use tapscript::{CompactAshSymbols, StackItem};
use target_elements::{EncodingClass, ReviewedElementsTapscriptDefinition};

use crate::error::LinkRefusal;

/// How the caller resolves the ASH constructor's self-commitment.
///
/// The ASH constructor's witness program commits to the taptree over
/// the very leaves whose programs compare against that program, so the
/// symbol's value is a function of itself. §14.4 requires an explicit
/// authenticated resolution strategy for such an edge and refuses an
/// unclassified one, so the strategy is a parameter of the link rather
/// than a decision the linker takes on the caller's behalf.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SelfCommitmentStrategy {
    /// No strategy is stated.
    ///
    /// The honest default, and a refusal rather than an omission: a
    /// link under this strategy reaches the cycle, classifies it, and
    /// returns [`LinkRefusal::ImpossibleStaticFixedPoint`]. §14.4
    /// prohibits searching for the fixed point by repeated hashing, and
    /// this variant is what makes not searching a typed outcome.
    NotStated,
    /// The referring programs obtain the value from the target at spend
    /// time rather than from a link-time literal.
    ///
    /// The sound resolution: a program that introspects the input it is
    /// spending has no need of a literal committing to itself, so the
    /// edge disappears instead of being resolved. The linker checks the
    /// claim rather than believing it — a leaf that still pushes a
    /// literal for the symbol contradicts the strategy and is refused.
    IdentityIntrospection,
    /// The value is fixed by a separately authenticated ceremony and
    /// supplied here, with the self-commitment equality left as an
    /// explicit outstanding obligation.
    ///
    /// Cuts the edge by authority rather than by computation: the
    /// linker resolves the symbol to the supplied value and records
    /// [`crate::LinkObligation::SelfCommitmentEqualityUndischarged`],
    /// which no part of this wave discharges. The obligation is carried
    /// structurally, so a bundle linked this way cannot be read as one
    /// whose commitment was checked.
    ExternallyAuthenticatedCommitment,
}

/// The public deployment parameters one link is given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkDeploymentParameters {
    resolved: CompactAshSymbols,
    internal_key: StackItem,
    self_commitment: SelfCommitmentStrategy,
    maximum_control_path_depth: NonZeroU32,
}

impl LinkDeploymentParameters {
    /// State the parameters, checking the internal key's width.
    ///
    /// The symbol resolutions were checked when [`CompactAshSymbols`]
    /// was built, which is why they are taken as that type; the
    /// internal key is checked here against the reviewed x-only public
    /// key class, because no other type owns it.
    ///
    /// # Errors
    ///
    /// [`LinkRefusal::InvalidDeploymentParameters`] when the internal
    /// key is not the width the reviewed contract fixes for an x-only
    /// public key.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        resolved: CompactAshSymbols,
        internal_key: Vec<u8>,
        self_commitment: SelfCommitmentStrategy,
        maximum_control_path_depth: NonZeroU32,
    ) -> Result<Self, LinkRefusal> {
        let internal_key = StackItem::encoded(target, EncodingClass::XOnlyPublicKey, internal_key)
            .map_err(LinkRefusal::InvalidDeploymentParameters)?;

        Ok(Self {
            resolved,
            internal_key,
            self_commitment,
            maximum_control_path_depth,
        })
    }

    /// The resolved link-time symbol values.
    #[must_use]
    pub const fn resolved(&self) -> &CompactAshSymbols {
        &self.resolved
    }

    /// The unspendable taproot internal key.
    #[must_use]
    pub const fn internal_key(&self) -> &StackItem {
        &self.internal_key
    }

    /// The declared self-commitment strategy.
    #[must_use]
    pub const fn self_commitment(&self) -> SelfCommitmentStrategy {
        self.self_commitment
    }

    /// The deepest control path the deployment admits.
    #[must_use]
    pub const fn maximum_control_path_depth(&self) -> NonZeroU32 {
        self.maximum_control_path_depth
    }
}
