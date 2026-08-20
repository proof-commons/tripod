//! The public construction view (§15.6).
//!
//! # What a permissionless constructor is allowed to know
//!
//! §15.6 fixes the list, and the omission at the end of it is the
//! point: no protocol owner or operator secret. So this type carries
//! outpoints, the target asset, value, and program fields those
//! outpoints already have on the target, and nothing else. There is no
//! blinding factor here, no opening, no key, and no field one could be
//! put in.
//!
//! # Sponsor-local data is not in here
//!
//! A sponsor's private wallet amounts and openings are real and a
//! sponsor adapter genuinely needs them to build a balanced
//! transaction. They arrive through the sponsor capability instead, are
//! used to construct, and are erased by the protocol projection. Mixing
//! them into the public view would be the exact confusion §1.6 exists
//! to stop: it would make the sponsor's amount look like something the
//! protocol relation may read.

use std::collections::BTreeMap;

use crate::bytes::{AssetField, Outpoint, ValueField};

/// The target's public facts about one unspent output.
///
/// Every field is one a target node hands out to anybody. Nothing here
/// is private to its owner, which is what makes a construction built
/// from it permissionless.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicOutputView {
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
}

impl PublicOutputView {
    /// The view of `outpoint` carrying these fields.
    #[must_use]
    pub const fn new(
        outpoint: Outpoint,
        asset: AssetField,
        value: ValueField,
        program: Vec<u8>,
    ) -> Self {
        Self {
            outpoint,
            asset,
            value,
            program,
        }
    }

    /// The outpoint this describes.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The asset field the output carries.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The value field the output carries.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The output's program.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }
}

/// The public chain view one construction is given.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PublicConstructionView {
    outputs: BTreeMap<Outpoint, PublicOutputView>,
}

impl PublicConstructionView {
    /// The view holding exactly these outputs.
    ///
    /// A later entry for one outpoint replaces an earlier one, which
    /// cannot produce two views of one outpoint: the map is keyed by
    /// outpoint, so a contradictory pair is unrepresentable rather than
    /// resolved.
    #[must_use]
    pub fn new(outputs: impl IntoIterator<Item = PublicOutputView>) -> Self {
        Self {
            outputs: outputs
                .into_iter()
                .map(|view| (view.outpoint(), view))
                .collect(),
        }
    }

    /// One outpoint's view, if the caller supplied it.
    #[must_use]
    pub fn get(&self, outpoint: Outpoint) -> Option<&PublicOutputView> {
        self.outputs.get(&outpoint)
    }

    /// Every view, in canonical outpoint order.
    #[must_use]
    pub const fn outputs(&self) -> &BTreeMap<Outpoint, PublicOutputView> {
        &self.outputs
    }
}
