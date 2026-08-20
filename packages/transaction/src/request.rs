//! The typed compact-ASH operation request (§15.5).
//!
//! # A request is a list of choices the caller is entitled to make
//!
//! §15.5 gives that list and, more importantly, gives the list of
//! choices a request may *not* make: the successor's amount, asset, and
//! program; the coordinator; the family range; the target program role;
//! the witness order; the projection; the target fee role. Every one of
//! those is absent here, and absent structurally — there is no field to
//! set and no builder that would take one. They derive from the ABI,
//! which is where a reader should look for them.
//!
//! # The sponsor is named, not described
//!
//! A request selects "optional sponsor input capabilities", and a
//! capability is an adapter rather than a bag of values. So a request
//! carries no sponsor outpoints and no fee: those come from the
//! capability at construction time, and a request that carried them
//! would let a caller build a sponsored transaction against a sponsor
//! that never agreed to it.

use std::collections::BTreeSet;

use crate::bytes::Outpoint;
use crate::error::TransactionRefusal;

/// One typed compact-ASH request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactAshRequest {
    ash: BTreeSet<Outpoint>,
    sponsored: bool,
}

impl CompactAshRequest {
    /// The request consolidating exactly these ASH outpoints.
    ///
    /// Duplicates are rejected before sorting rather than collapsed by
    /// it. A set built by insertion would silently accept a caller who
    /// named one outpoint twice and consolidate a smaller family than
    /// the caller asked for, and §10.1 says duplicates are rejected.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::EmptyAshSelection`] for an empty
    /// selection, and [`TransactionRefusal::DuplicateOutpoint`] when
    /// one outpoint is named more than once.
    pub fn new(
        ash: impl IntoIterator<Item = Outpoint>,
        sponsored: bool,
    ) -> Result<Self, TransactionRefusal> {
        let mut selected = BTreeSet::new();
        for outpoint in ash {
            if !selected.insert(outpoint) {
                return Err(TransactionRefusal::DuplicateOutpoint(outpoint));
            }
        }
        if selected.is_empty() {
            return Err(TransactionRefusal::EmptyAshSelection);
        }
        Ok(Self {
            ash: selected,
            sponsored,
        })
    }

    /// The selected ASH outpoints, in canonical order.
    ///
    /// Canonical order is the set's own order, which is ascending by
    /// transaction identifier and then by index — the ABI's declared
    /// ordering, reached by construction rather than by a later sort
    /// somebody could forget.
    #[must_use]
    pub const fn ash(&self) -> &BTreeSet<Outpoint> {
        &self.ash
    }

    /// Whether the request asks for a sponsor suffix.
    #[must_use]
    pub const fn sponsored(&self) -> bool {
        self.sponsored
    }
}
