//! UTXO metadata, objects, and domain-separated data outputs.
//!
//! Implements `(´def:objects:metadata´)`, `(´def:verification:utxo´)`,
//! `(´def:objects:destruction-tags´)`, and `(´def:verification:data-output´)`.

use crate::asset::{Asset, ReceiptClass};
use crate::pool::PoolState;
use crate::scalar::{AttestationAddress, Cycle, OwnerKey, Sat};

// ´def:objects:metadata´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meta {
    // Roots.
    State(PoolState),
    Resv,
    Pace,
    EntitlementAuthority,
    DistributionAuthority,

    // User-owned receipt objects.
    Receipt {
        owner: OwnerKey,
        class: ReceiptClass,
    },

    // Open external deposit offer.
    DepositRequest {
        pool_id: u32,
        refund_key: OwnerKey,
        receipt_owner: OwnerKey,
        deposit_principal: Sat,
    },

    // Closed entitlement object.
    DepositEntitlement {
        owner: OwnerKey,
        target_cycle: Cycle,
    },

    // Closed per-cycle distribution control.
    DistributionControl {
        cycle: Cycle,
        principal: Sat,
        live_allocation: Sat,
        time_locked_allocation: Sat,
        remaining_principal: Sat,
        remaining_live_value: Sat,
        remaining_time_locked_value: Sat,
    },

    // Closed receipt-asset vault.
    DistributionVault {
        cycle: Cycle,
    },

    // Ownerless receipt-asset sink.
    Ash,

    // Ordinary open L-BTC output.
    PlainLbtc {
        owner: OwnerKey,
    },

    // Zero-value package-fee hook.
    CpfpAnchor,

    // Adversarial or unrelated shape.
    ForeignShape(u32),
}

// ´def:verification:utxo´

/// A modeled transaction output.
///
/// Consensus-value note (the consensus-value clause): in this model,
/// `value` is the one value an object has — a committed-vs-consensus
/// divergence is unrepresentable, so no runtime check compares them
/// and none is missing. The clause's on-chain discharge is the
/// `ExplicitValueIntrospection` substrate dependency plus issuance introspection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utxo {
    pub asset: Asset,
    pub value: Sat,
    pub meta: Meta,
}

// ´def:objects:destruction-tags´

/// Discriminants are the architecture's stable tag codes
/// (`architecture::TagId`); the manifest welds the pairing
/// code-for-code at compile time.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag {
    Burn = 1,
    Recon = 2,
    Redeem = 3,
    Entitlement = 4,
    DistributionControlClose = 5,
    DistributionResidue = 6,
}

// ´def:verification:data-output´

/// A `DataOutput` is a proof-model representation of a provably
/// unspendable transaction output. It is not inserted into the spendable
/// UTXO map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataOutput {
    BurnRecord {
        /// Canonical ordinal among this transaction's burn-record
        /// outputs. Valid burn records are indexed contiguously from
        /// zero in transaction-output order.
        record_index: u32,

        address: AttestationAddress,
        amount: Sat,
    },

    Destruction {
        tag: Tag,
        asset: Asset,
        amount: Sat,
    },
}
