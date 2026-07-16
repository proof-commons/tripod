//! Atomic transition API.
//!
//! Implements `(´rule:verification:pure-transition´)`.
//!
//! [`execute`] is the normative public state-transition entry point:
//! it applies a declared operation and re-checks the global invariant
//! on the result. The [`Transition`] trait is sealed: only the
//! declared operation constructors may implement it, so an external
//! crate cannot add a new *transition kind* that bypasses branch
//! authorization while still passing the invariant wrapper.
//!
//! Sealing closes the operation vocabulary, not the state space. This
//! crate is a transparent reference model (see the crate-level trust
//! boundary): [`Transition::apply`] is public and unwrapped, and
//! `World` state is publicly mutable for audit and fault harnesses. A
//! valid-world precondition is part of `apply`'s contract; the
//! invariant re-check belongs to [`execute`].
//!
//! Because every transition receives `&World` and returns a fresh
//! `World`, an error leaves:
//!
//! - UTXOs;
//! - wallets;
//! - adversary budget;
//! - roots;
//! - history;
//! - cadence state;
//!
//! unchanged by construction.

use crate::guard::Guard;
use crate::invariant::check_invariant;
use crate::scalar::CanonicalOrder;
use crate::world::World;

pub(crate) mod sealed {
    use crate::ops::{
        AdmitDeposits, AnnounceMaturity, BurnReceipts, CancelRequest, ClearAsh, CompactAsh,
        CreateRequest, RedeemReceipt, RelabelReceipts, RunCycle, SettleDistribution,
        TransferReceipts,
    };

    /// Sealing marker: implemented exactly for the declared operation
    /// constructors. The module is crate-private, so external crates
    /// cannot name it and therefore cannot implement [`super::Transition`].
    pub trait Sealed {}

    impl Sealed for CreateRequest {}
    impl Sealed for CancelRequest {}
    impl Sealed for AdmitDeposits {}
    impl Sealed for RunCycle {}
    impl Sealed for SettleDistribution {}
    impl Sealed for TransferReceipts {}
    impl Sealed for RedeemReceipt {}
    impl Sealed for RelabelReceipts {}
    impl Sealed for BurnReceipts {}
    impl Sealed for CompactAsh {}
    impl Sealed for ClearAsh {}
    impl Sealed for AnnounceMaturity {}
}

// ´rule:verification:pure-transition´

pub trait Transition: sealed::Sealed {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard>;
}

pub fn execute<T>(world: &World, transition: &T, order: CanonicalOrder) -> Result<World, Guard>
where
    T: Transition,
{
    let next = transition.apply(world, order)?;

    check_invariant(&next).map_err(|_| Guard::InvariantFailure)?;

    Ok(next)
}
