//! Receipt-accounting audit projection and its differential
//! comparison.
//!
//! Implements `´def:verification:residue-audit-event´`,
//! `´def:verification:receipt-accounting-audit-projection´`,
//! `´rule:verification:receipt-accounting-audit´`, and
//! `´rule:verification:compare-receipt-accounting-audit´`.
//!
//! Historical distribution residue is audit-only by the manifest's
//! reader matrix: it is readable by the invariant checker and the
//! external auditor and forbidden to the attestation indexer, the
//! consumer formula, and every covenant operation. The
//! differential-conformance correction therefore lives here, in a
//! separate harness under the external-auditor role — never inside
//! [`crate::ledger::ReferenceIndexer`], which must remain
//! residue-blind.
//!
//! An extra and a missing residue of equal value preserve the
//! aggregate totals; the exact event-sequence comparison detects the
//! substitution. The totals are derived from the event list, not
//! accepted independently.

use num_bigint::BigUint;
use num_traits::Zero;

use crate::guard::InvariantError;
use crate::invariant::fold_accounting;
use crate::ledger::DifferentialError;
use crate::scalar::{CanonicalOrder, Cycle, OutPoint, Sat, TxId};
use crate::world::World;

// ´def:verification:residue-audit-event´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResidueAuditEvent {
    pub txid: TxId,
    pub order: CanonicalOrder,
    pub cycle: Cycle,
    pub control_input: OutPoint,
    pub vault_input: Option<OutPoint>,
    pub live_residue: Sat,
    pub time_locked_residue: Sat,
}

// ´def:verification:receipt-accounting-audit-projection´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptAccountingAuditProjection {
    /// The exact canonical residue event sequence. The historical
    /// totals below are derived from this list, never accepted
    /// independently.
    pub residue_events: Vec<ResidueAuditEvent>,

    pub historical_live_residue: BigUint,
    pub historical_time_locked_residue: BigUint,

    pub current_y_live: Sat,
    pub current_y_time_locked: Sat,

    pub circulating_live: BigUint,
    pub circulating_time_locked: BigUint,
    pub ash: BigUint,
    pub distribution_live: BigUint,
    pub distribution_time_locked: BigUint,
}

// ´rule:verification:receipt-accounting-audit´
//
// Derived from state and history under the external-auditor /
// invariant-checker role.

pub fn receipt_accounting_audit(
    world: &World,
) -> Result<ReceiptAccountingAuditProjection, InvariantError> {
    let state = world
        .state()
        .map_err(|_| InvariantError::IdentityAuthority)?
        .1;

    let fold = fold_accounting(world)?;

    let mut residue_events = Vec::new();

    let mut historical_live_residue = BigUint::zero();
    let mut historical_time_locked_residue = BigUint::zero();

    for (certificate, residue) in world.history.residue_projections() {
        residue_events.push(ResidueAuditEvent {
            txid: certificate.txid,
            order: certificate.order,
            cycle: residue.cycle,
            control_input: residue.control_input,
            vault_input: residue.vault_input,
            live_residue: residue.live_residue,
            time_locked_residue: residue.time_locked_residue,
        });

        historical_live_residue += BigUint::from(residue.live_residue.get());

        historical_time_locked_residue += BigUint::from(residue.time_locked_residue.get());
    }

    Ok(ReceiptAccountingAuditProjection {
        residue_events,

        historical_live_residue,
        historical_time_locked_residue,

        current_y_live: state.y_l,
        current_y_time_locked: state.y_t,

        circulating_live: fold.circulating_live,
        circulating_time_locked: fold.circulating_time_locked,
        ash: fold.ash,
        distribution_live: fold.distribution_live,
        distribution_time_locked: fold.distribution_time_locked,
    })
}

// ´rule:verification:compare-receipt-accounting-audit´
//
// Both the exact residue event sequence and the aggregate accounting
// terms are compared: offsetting residue events with equal totals are
// a projection mismatch even though every aggregate agrees.

pub fn compare_receipt_accounting_audit(
    expected: &ReceiptAccountingAuditProjection,
    candidate: &ReceiptAccountingAuditProjection,
) -> Result<(), DifferentialError> {
    if expected != candidate {
        return Err(DifferentialError::ReceiptAccountingProjectionMismatch);
    }

    Ok(())
}
