//! The typed public deployment parameters one live link is given.
//!
//! # Supplied, never minted
//!
//! [`crate::deployment`]'s rule, unchanged: every resolvable value
//! arrives from the caller, because §1.10 refuses a speculative identity
//! and a linker that invented an asset or a key would be inventing
//! exactly the thing a deployment is supposed to fix.
//!
//! # The sighash profile is derived, not supplied
//!
//! The one exception, and it is an exception for a reason worth stating.
//! The selected profile is not a deployment's preference — §1.7 fixes it
//! as the all-inputs, all-outputs profile unless a narrower one is
//! separately proved, and [`selected_owner_profile`] is that argument
//! written down. A parameter for it would be a way to weaken §1.7 by
//! passing a different value, so there is none: the link reads the
//! profile and assesses it against the reviewed contract's own sighash
//! capability.
//!
//! What the assessment says today is
//! [`tapscript::OwnerProfileDisposition::ReviewIncomplete`], and that is
//! carried rather than smoothed over. §11.2 puts the profile at the link,
//! which is what this module does; it does not put the *review* at the
//! link, and no part of Guide 13 completes one.
//!
//! # Private keys never enter
//!
//! The internal key is an x-only public point and the owner keys are the
//! constructors' own committed public metadata. Nothing here holds,
//! derives, or accepts a secret, and §1.10 defers any first-party
//! production interface for owner private keys to a separate design.

use std::num::NonZeroU32;

use tapscript::{LiveTransferSymbols, OwnerKey, StackItem, selected_owner_profile};
use target_elements::{EncodingClass, ReviewedElementsTapscriptDefinition};

use crate::error::LinkRefusal;
use crate::live_symbol::SelectedSighashProfile;

/// The public deployment parameters one live link is given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveLinkDeploymentParameters {
    resolved: LiveTransferSymbols,
    internal_key: StackItem,
    maximum_control_path_depth: NonZeroU32,
    sighash_profile: SelectedSighashProfile,
}

impl LiveLinkDeploymentParameters {
    /// State the parameters, checking the internal key's width and
    /// assessing the selected sighash profile.
    ///
    /// The symbol resolutions were checked when [`LiveTransferSymbols`]
    /// was built, which is why they are taken as that type; the internal
    /// key is checked here against the reviewed x-only public key class,
    /// because no other type owns it.
    ///
    /// # Errors
    ///
    /// [`LinkRefusal::InvalidDeploymentParameters`] when the internal key
    /// is not the width the reviewed contract fixes for an x-only public
    /// key.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        resolved: LiveTransferSymbols,
        internal_key: Vec<u8>,
        maximum_control_path_depth: NonZeroU32,
    ) -> Result<Self, LinkRefusal> {
        let internal_key = StackItem::encoded(target, EncodingClass::XOnlyPublicKey, internal_key)
            .map_err(LinkRefusal::InvalidDeploymentParameters)?;

        let profile = selected_owner_profile();
        let disposition = profile.assess(target.definition().authorization().sighash());

        Ok(Self {
            resolved,
            internal_key,
            maximum_control_path_depth,
            sighash_profile: SelectedSighashProfile::new(profile, disposition),
        })
    }

    /// The resolved link-time symbol values.
    #[must_use]
    pub const fn resolved(&self) -> &LiveTransferSymbols {
        &self.resolved
    }

    /// The unspendable taproot internal key.
    #[must_use]
    pub const fn internal_key(&self) -> &StackItem {
        &self.internal_key
    }

    /// The deepest control path the deployment admits.
    #[must_use]
    pub const fn maximum_control_path_depth(&self) -> NonZeroU32 {
        self.maximum_control_path_depth
    }

    /// The selected sighash profile and what the review establishes.
    #[must_use]
    pub const fn sighash_profile(&self) -> &SelectedSighashProfile {
        &self.sighash_profile
    }
}

/// One committed owner's key as the stack item a program pushes for it.
///
/// The owner is a constructor's, never a parameter of a link, and that is
/// §7.6 holding: a request has no way to supply owner bytes to a link, so
/// the only owner a link can place is one some constructor already
/// committed through [`OwnerKey::new`]'s gate.
///
/// # Errors
///
/// [`LinkRefusal::InvalidDeploymentParameters`] if the committed owner is
/// not the reviewed contract's approved encoding at its exact width.
/// Unreachable for a constructed [`OwnerKey`], which passed that gate;
/// refusing rather than unwrapping keeps the reasoning out of the
/// caller's soundness.
pub fn owner_stack_item(
    target: &ReviewedElementsTapscriptDefinition,
    owner: &OwnerKey,
) -> Result<StackItem, LinkRefusal> {
    StackItem::encoded(target, owner.encoding(), owner.bytes().to_vec())
        .map_err(LinkRefusal::InvalidDeploymentParameters)
}
