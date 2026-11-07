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
use crate::invariant::{check_invariant_core, report_invariant_failure};
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
    pub trait Sealed {
        fn operation_code(&self) -> u16;
    }

    macro_rules! operation_code {
        ($request:ty, $operation:ident) => {
            impl Sealed for $request {
                fn operation_code(&self) -> u16 {
                    architecture::OperationId::$operation.code()
                }
            }
        };
    }

    operation_code!(CreateRequest, CreateRequest);
    operation_code!(CancelRequest, CancelRequest);
    operation_code!(AdmitDeposits, AdmitDeposits);
    operation_code!(RunCycle, Cycle);
    operation_code!(SettleDistribution, SettleDistribution);
    operation_code!(RedeemReceipt, Redeem);
    operation_code!(RelabelReceipts, ReceiptRelabel);
    operation_code!(BurnReceipts, Burn);
    operation_code!(CompactAsh, CompactAsh);
    operation_code!(ClearAsh, Clear);
    operation_code!(AnnounceMaturity, AnnounceMaturity);

    impl Sealed for TransferReceipts {
        fn operation_code(&self) -> u16 {
            match self.class {
                crate::asset::ReceiptClass::Live => architecture::OperationId::TransferLive.code(),
                crate::asset::ReceiptClass::TimeLocked => {
                    architecture::OperationId::TransferTimeLocked.code()
                }
            }
        }
    }
}

// ´rule:verification:pure-transition´

pub trait Transition: sealed::Sealed {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard>;
}

pub fn execute<T>(world: &World, transition: &T, order: CanonicalOrder) -> Result<World, Guard>
where
    T: Transition,
{
    let span = tracing::debug_span!(
        target: "model::transition",
        "execute",
        operation_code = sealed::Sealed::operation_code(transition),
        history_count = u64::try_from(world.history.transitions.len()).unwrap_or(u64::MAX),
    );
    let _entered = span.enter();
    let result = execute_core(world, transition, order);
    match &result {
        Ok(next) => tracing::debug!(
            target: "model::transition",
            outcome = "completed",
            operation_code = sealed::Sealed::operation_code(transition),
            history_count = u64::try_from(next.history.transitions.len()).unwrap_or(u64::MAX),
            "model transition completed",
        ),
        Err(Guard::InvariantFailure) => tracing::debug!(
            target: "model::transition",
            outcome = "failed",
            operation_code = sealed::Sealed::operation_code(transition),
            error_kind = "invariant_failure",
            "model transition completed",
        ),
        Err(error) => tracing::debug!(
            target: "model::transition",
            outcome = "refused",
            operation_code = sealed::Sealed::operation_code(transition),
            refusal_kind = error.name(),
            "model transition completed",
        ),
    }
    result
}

fn execute_core<T>(world: &World, transition: &T, order: CanonicalOrder) -> Result<World, Guard>
where
    T: Transition,
{
    let next = transition.apply(world, order)?;

    if let Err(error) = check_invariant_core(&next) {
        report_invariant_failure(error);
        return Err(Guard::InvariantFailure);
    }

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

impl ExecutionBindingError {
    pub const fn name(self) -> &'static str {
        match self {
            Self::InvalidSuccessorExtension => "invalid_successor_extension",
            Self::RequestBindingMismatch => "request_binding_mismatch",
        }
    }
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
    let span = tracing::debug_span!(
        target: "model::transition",
        "bind_execution",
        operation_code = sealed::Sealed::operation_code(&request),
        history_count = u64::try_from(before.history.transitions.len()).unwrap_or(u64::MAX),
    );
    let _entered = span.enter();
    let result = bind_execution_core(before, request, after);
    match &result {
        Ok(_) => tracing::debug!(
            target: "model::transition",
            outcome = "completed",
            "model binding completed",
        ),
        Err(error) => tracing::debug!(
            target: "model::transition",
            outcome = "refused",
            refusal_kind = error.name(),
            "model binding completed",
        ),
    }
    result
}

fn bind_execution_core<T>(
    before: &World,
    request: T,
    after: &World,
) -> Result<ExecutedTransition<T>, ExecutionBindingError>
where
    T: Transition,
{
    let appended = one_appended_certificate(before, after)
        .ok_or(ExecutionBindingError::InvalidSuccessorExtension)?;
    let replayed = execute_core(before, &request, appended.order)
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
