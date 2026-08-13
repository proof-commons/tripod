//! The typed target-contract version.
//!
//! This module will grow into the whole target definition. At the
//! crate-boundary stage it owns only the contract version, because
//! that is the one value a downstream adapter must inspect before it
//! may interpret anything else the package says.

use crate::error::TargetError;

/// The revision of the typed target compatibility contract.
///
/// This is a stable typed key, not a digest. It changes when the
/// semantic shape or interpretation of the contract changes, and a
/// consumer accepts or rejects it as an explicit compatibility
/// decision. Editorial review provenance — which upstream revision was
/// consulted, when, by whom — never moves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetContractVersion(u32);

impl TargetContractVersion {
    /// The first typed contract revision.
    pub const V1: Self = Self(1);

    /// Every contract revision this crate implements.
    pub const SUPPORTED: &'static [Self] = &[Self::V1];

    /// Accepts a contract version number this crate implements.
    ///
    /// # Errors
    ///
    /// Returns [`TargetError::UnsupportedTargetContractVersion`] when
    /// the offered number names no revision in [`Self::SUPPORTED`].
    pub fn supported(value: u32) -> Result<Self, TargetError> {
        let candidate = Self(value);
        if Self::SUPPORTED.contains(&candidate) {
            Ok(candidate)
        } else {
            Err(TargetError::UnsupportedTargetContractVersion { offered: value })
        }
    }

    /// The contract revision number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}
