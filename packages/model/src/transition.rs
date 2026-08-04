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
use crate::history::TransitionCertificate;
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

/// Failure while binding a request to an already-produced successor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionBindingError {
    /// The successor does not extend the predecessor history by exactly
    /// one certificate.
    InvalidSuccessorExtension,
    /// Replaying the supplied request against the predecessor does not
    /// reproduce the supplied successor.
    RequestBindingMismatch,
}

/// One executed transition: predecessor, the exact request that ran, and
/// the successor it produced.
///
/// The fields are private and there is no unchecked constructor, so a
/// value of this type can only come from [`execute_bound`] or
/// [`bind_execution`]. Holding one is evidence that `request` applied to
/// `before` under the invariant wrapper yields `after` — the binding the
/// conformance projection requires so an observation cannot mix the
/// certificate of one execution with the authorization data of another.
///
/// External assembly from independent parts fails to compile:
///
/// ```compile_fail
/// let forged = model::ExecutedTransition {
///     before: todo!(),
///     request: todo!(),
///     after: todo!(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutedTransition<T> {
    before: World,
    request: T,
    after: World,
}

impl<T> ExecutedTransition<T> {
    pub fn before(&self) -> &World {
        &self.before
    }

    pub fn request(&self) -> &T {
        &self.request
    }

    pub fn after(&self) -> &World {
        &self.after
    }

    /// The certificate appended by this execution.
    pub fn certificate(&self) -> &TransitionCertificate {
        self.after
            .history
            .transitions
            .last()
            .expect("bound execution appends exactly one certificate")
    }

    /// Consume the binding, keeping only the successor world.
    pub fn into_world(self) -> World {
        self.after
    }
}

/// Execute `request` and retain the predecessor/request/successor binding.
///
/// Runs the invariant-wrapped [`execute`]; the successor must extend the
/// predecessor by exactly one certificate carrying `order`.
pub fn execute_bound<T>(
    world: &World,
    request: T,
    order: CanonicalOrder,
) -> Result<ExecutedTransition<T>, Guard>
where
    T: Transition,
{
    let after = execute(world, &request, order)?;
    let appended = one_appended_certificate(world, &after).ok_or(Guard::InvariantFailure)?;

    if appended.order != order {
        return Err(Guard::InvariantFailure);
    }

    Ok(ExecutedTransition {
        before: world.clone(),
        request,
        after,
    })
}

/// Bind a request to a successor produced earlier, by deterministic replay.
///
/// The successor must extend `before` by exactly one certificate; the
/// request is re-executed at that certificate's order and the replayed
/// world must equal `after` completely. Replay uses only model execution —
/// realization evaluation is never consulted, so the binding check cannot
/// become circular with conformance evaluation.
pub fn bind_execution<T>(
    before: &World,
    request: T,
    after: &World,
) -> Result<ExecutedTransition<T>, ExecutionBindingError>
where
    T: Transition,
{
    let appended = one_appended_certificate(before, after)
        .ok_or(ExecutionBindingError::InvalidSuccessorExtension)?;
    let replayed = execute(before, &request, appended.order)
        .map_err(|_| ExecutionBindingError::RequestBindingMismatch)?;

    if replayed != *after {
        return Err(ExecutionBindingError::RequestBindingMismatch);
    }

    Ok(ExecutedTransition {
        before: before.clone(),
        request,
        after: after.clone(),
    })
}

fn one_appended_certificate<'a>(
    before: &World,
    after: &'a World,
) -> Option<&'a TransitionCertificate> {
    let expected_length = before.history.transitions.len().checked_add(1)?;

    if after.history.transitions.len() != expected_length
        || !after
            .history
            .transitions
            .starts_with(&before.history.transitions)
    {
        return None;
    }

    after.history.transitions.last()
}
