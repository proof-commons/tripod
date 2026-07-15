//! Transition kinds, root edges, canonical deltas, transition
//! certificates, derived event projections, and canonical chain history.
//!
//! Implements `´def:verification:branch-kind´`,
//! `´def:verification:root-edge´`, `´def:verification:canonical-delta´`,
//! `´def:verification:transition-certificate´`, `´def:state:burn-event´`,
//! `´def:state:clear-event´`, `´def:state:distribution-residue-event´`,
//! `´def:verification:genesis-history´`, and `´def:verification:history´`.
//!
//! All projections are derived by the transition kernel from actual
//! consumed/created objects. Operation code does not author them
//! directly.

use std::collections::BTreeSet;

use crate::asset::Asset;
use crate::object::Tag;
use crate::scalar::{AttestationAddress, CanonicalOrder, Cycle, OutPoint, Sat, TxId};

// ´def:verification:branch-kind´

/// Discriminants are the architecture's stable operation codes
/// (`architecture::OperationId`); the manifest welds the pairing
/// code-for-code at compile time.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BranchKind {
    CreateRequest = 1,
    CancelRequest = 2,
    AdmitDeposits = 3,
    Cycle = 4,
    SettleDistribution = 5,
    TransferLive = 6,
    TransferTimeLocked = 7,
    Redeem = 8,
    ReceiptRelabel = 9,
    Burn = 10,
    CompactAsh = 11,
    Clear = 12,
    AnnounceMaturity = 13,
}

// ´def:verification:root-edge´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootEdge {
    Succ { input: OutPoint, output: OutPoint },

    Term { input: OutPoint },
}

// ´def:verification:canonical-delta´

/// Discriminants are the architecture's stable delta-kind codes
/// (`architecture::DeltaKind`); the manifest welds the pairing
/// code-for-code at compile time.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeltaKind {
    Issuance = 1,
    Destruction = 2,
    Lateral = 3,
    OwnerlessLateral = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalDelta {
    pub asset: Asset,
    pub kind: DeltaKind,
    pub amount: Sat,

    pub authority_input: Option<OutPoint>,
    pub source_inputs: Vec<OutPoint>,
    pub destination_outputs: Vec<OutPoint>,

    pub destruction_tag: Option<Tag>,
}

// ´def:verification:open-flow-kind´

/// Open-value flow roles.
///
/// Discriminants are the architecture's stable open-flow codes
/// (`architecture::OpenFlowKind`); the manifest welds the pairing
/// code-for-code at compile time.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpenFlowKind {
    // Wallet inputs create one request and optional wallet change.
    RequestCreation = 1,

    // Request value returns in full to its refund key.
    RequestRefund = 2,

    // Active RESV plus request values become:
    // successor RESV + optional admission reward + chain fee.
    DepositAdmission = 3,

    // Active RESV is recreated unchanged by cycle.
    ReserveCarry = 4,

    // Active RESV becomes:
    // successor RESV + formula-bound redemption payout.
    Redemption = 5,

    // Ordinary wallet inputs become:
    // sponsor change + explicit chain fee.
    FeeSponsor = 6,
}

// ´def:verification:open-flow-projection´

/// Proof metadata derived from already validated transaction inputs,
/// outputs, and flow declarations. Not a chain output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenFlowProjection {
    pub kind: OpenFlowKind,

    pub source_inputs: Vec<OutPoint>,
    pub destination_outputs: Vec<OutPoint>,

    pub fee: Sat,
}

// ´def:verification:transition-certificate´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionCertificate {
    pub txid: TxId,
    pub order: CanonicalOrder,
    pub branch: BranchKind,

    pub consumed: BTreeSet<OutPoint>,
    pub created: BTreeSet<OutPoint>,

    pub state_edge: Option<RootEdge>,
    pub resv_edge: Option<RootEdge>,
    pub pace_edge: Option<RootEdge>,
    pub entitlement_authority_edge: Option<RootEdge>,
    pub distribution_authority_edge: Option<RootEdge>,

    pub canonical_deltas: Vec<CanonicalDelta>,

    pub open_flows: Vec<OpenFlowProjection>,

    pub chain_fee: Sat,

    pub burn: Option<BurnProjection>,
    pub clear: Option<ClearProjection>,
    pub distribution_residue: Option<DistributionResidueProjection>,
}

// ´def:state:burn-event´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnProjection {
    pub ash_output: OutPoint,
    pub ash_value: Sat,
    pub records: Vec<BurnRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurnRecord {
    pub record_index: u32,
    pub address: AttestationAddress,
    pub amount: Sat,
}

// ´def:state:clear-event´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClearProjection {
    pub omega: Sat,
    pub y: Sat,
}

// ´def:state:distribution-residue-event´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionResidueProjection {
    pub cycle: Cycle,
    pub control_input: OutPoint,
    pub vault_input: Option<OutPoint>,
    pub live_residue: Sat,
    pub time_locked_residue: Sat,
}

// ´def:verification:genesis-history´

/// Genesis trusted-setup projection.
///
/// Genesis is not represented as `BranchKind::CreateRequest`. It is a
/// separate trusted setup object carried by [`History`], and the
/// genesis clear is derived from it separately from ordinary branch
/// semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenesisProjection {
    pub txid: TxId,
    pub order: CanonicalOrder,

    pub state_out: OutPoint,
    pub resv_out: OutPoint,
    pub pace_out: OutPoint,
    pub entitlement_authority_out: OutPoint,
    pub distribution_authority_out: OutPoint,

    /// Genesis receipt outputs. Recorded so the history replay's
    /// creation census covers every genesis-created outpoint, not only
    /// the five roots: a later certificate claiming one of these as
    /// newly created must be rejected.
    pub live_receipt_out: OutPoint,
    pub time_locked_receipt_out: OutPoint,

    pub omega: Sat,
    pub y: Sat,
}

// ´def:verification:history´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    pub genesis: GenesisProjection,
    pub transitions: Vec<TransitionCertificate>,
}

impl History {
    pub fn last_order(&self) -> CanonicalOrder {
        self.transitions
            .last()
            .map(|transition| transition.order)
            .unwrap_or(self.genesis.order)
    }

    pub fn burn_projections(
        &self,
    ) -> impl Iterator<Item = (&TransitionCertificate, &BurnProjection)> {
        self.transitions
            .iter()
            .filter_map(|certificate| certificate.burn.as_ref().map(|burn| (certificate, burn)))
    }

    pub fn clear_projections(
        &self,
    ) -> impl Iterator<Item = (&TransitionCertificate, &ClearProjection)> {
        self.transitions
            .iter()
            .filter_map(|certificate| certificate.clear.as_ref().map(|clear| (certificate, clear)))
    }

    pub fn residue_projections(
        &self,
    ) -> impl Iterator<Item = (&TransitionCertificate, &DistributionResidueProjection)> {
        self.transitions.iter().filter_map(|certificate| {
            certificate
                .distribution_residue
                .as_ref()
                .map(|residue| (certificate, residue))
        })
    }
}
