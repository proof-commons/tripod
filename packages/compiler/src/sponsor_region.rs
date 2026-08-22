//! Which region of an operation ordinary L-BTC belongs to (Guide-8 F.4,
//! F.5).
//!
//! Sponsor erasure is a statement about a *flow role*, not about an
//! object family. The normative rule partitions the L-BTC references of
//! a transaction into a protocol region and a sponsor region and erases
//! individual amounts only in the sponsor region; the architecture uses
//! the ordinary-L-BTC family for both. Treating the family as
//! synonymous with "sponsor" makes every ordinary-L-BTC reference
//! opaque, which is wrong in both directions: a mandatory owner-funded
//! request-creation input becomes an optional sponsor region, and a
//! formula-bound redemption payout becomes an amount no relation may
//! read.
//!
//! # What can be decided here
//!
//! The compiler is given open flows per *operation*, through the typed
//! open-flow policy relation. It is not given the flow each individual
//! object reference belongs to. That is enough to decide the question
//! for an operation whose declared flows cannot claim ordinary L-BTC
//! for anything but fee sponsorship, and it is not enough for an
//! operation that declares a protocol flow which can.
//!
//! So the classification is three-valued and the undecidable case is
//! named rather than guessed. An operation that claims ordinary L-BTC
//! in a protocol flow needs per-reference flow membership the
//! declarations do not carry; the compiler reports that gap as a typed
//! refusal instead of picking one of the two wrong answers. Both
//! current pilots declare fee sponsorship and nothing else, so both
//! classify as a pure sponsor region and their analyses are unchanged.

use std::collections::BTreeSet;

use architecture::{ObjectId, OpenFlowKind, OperationId};
use realization::Relation;

use crate::{CompileError, relation::CompilerRelationAnalysis};

/// The object family the architecture uses for both regions.
///
/// Named once here rather than matched at each site: every remaining
/// mention of the family in a sponsor test is a place that would have
/// to be revisited if the architecture gained a second dual-use family,
/// and a single constant makes that census findable.
pub const ORDINARY_LBTC: ObjectId = ObjectId::PlainLbtc;

/// What ordinary L-BTC means inside one operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OrdinaryLbtcRole {
    /// No declared open flow claims ordinary L-BTC.
    ///
    /// The family has no region in this operation, so no relation over
    /// it is sponsor-conditional and no amount over it is erased.
    Absent,
    /// Ordinary L-BTC appears only as the fee-sponsor region.
    ///
    /// The family and the region coincide *for this operation*, which
    /// is the only condition under which the family-based test was ever
    /// right.
    SponsorRegion,
    /// A protocol flow of this operation claims ordinary L-BTC.
    ///
    /// The family no longer identifies the region, and the declarations
    /// do not say which references belong to which flow.
    ProtocolClaimed,
}

impl OrdinaryLbtcRole {
    /// Whether ordinary-L-BTC amounts are erased in this operation.
    ///
    /// True for [`Self::Absent`] as well as [`Self::SponsorRegion`].
    /// Absent means no declared flow claims the family for anything, so
    /// erasing it discards no protocol relation, and erasure is the safe
    /// reading of silence for a confidentiality property: a wrongly
    /// erased amount is a relation that cannot be expressed, while a
    /// wrongly readable one is a leak.
    ///
    /// That asymmetry is only safe because the third case is refused
    /// outright. [`Self::ProtocolClaimed`] is the reading under which
    /// erasure *would* discard a load-bearing relation, and the scope
    /// gate rejects those operations rather than resolving them either
    /// way.
    #[must_use]
    pub const fn is_sponsor_region(self) -> bool {
        !matches!(self, Self::ProtocolClaimed)
    }
}

/// The ordinary-L-BTC role of every operation an accepted analysis
/// contains.
///
/// [`validate_decidable_sponsor_regions`] refuses any scope holding an
/// operation whose ordinary L-BTC is protocol-claimed, and every
/// remaining architecture operation declares fee sponsorship. So inside
/// an analysis the compiler accepted, the role cannot vary, and a stage
/// with no relation analysis to hand may say so instead of threading a
/// value with one possible answer.
///
/// This is a property of the current gate rather than a permanent one.
/// Every use of it is a site that must receive the real per-operation
/// role when the gate is lifted — which is exactly why it is a named
/// constant and not a bare [`OrdinaryLbtcRole::SponsorRegion`] at each
/// call.
pub const GATED_ORDINARY_LBTC_ROLE: OrdinaryLbtcRole = OrdinaryLbtcRole::SponsorRegion;

/// Whether one open flow can claim ordinary L-BTC in a protocol role.
///
/// Exhaustive over the flow vocabulary on purpose: a new open-flow kind
/// makes this match non-exhaustive at compile time, so the question
/// "can this flow hold a protocol L-BTC amount?" has to be answered
/// when the flow is introduced rather than defaulted.
const fn claims_protocol_lbtc(flow: OpenFlowKind) -> bool {
    match flow {
        // Owner-funded inputs and owner change.
        OpenFlowKind::RequestCreation
        // A formula- and destination-bound refund.
        | OpenFlowKind::RequestRefund
        // An optional admission reward.
        | OpenFlowKind::DepositAdmission
        // A formula-bound owner payout.
        | OpenFlowKind::Redemption => true,

        // The reserve carry moves the reserve asset; its ordinary-L-BTC
        // change is generic sponsor change in the current architecture.
        OpenFlowKind::ReserveCarry
        // The sponsor region itself.
        | OpenFlowKind::FeeSponsor => false,
    }
}

/// The open flows one operation declares, from its typed policy
/// relation.
///
/// Read from the realization's own open-flow policy rather than from
/// the architecture operation record, because that relation is the
/// channel the compiler already treats as the authority on which flows
/// an operation admits.
fn declared_open_flows(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
) -> BTreeSet<OpenFlowKind> {
    relations
        .graph
        .node_weights()
        .filter(|node| node.source.id.operation() == operation)
        .filter_map(|node| match &node.source.relation {
            Relation::OpenFlowPolicy { allowed } => Some(allowed.iter().copied()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// Classify ordinary L-BTC from one operation's declared open flows.
///
/// The classification itself, separated from where the flow set was
/// read. A boundary holding the realization projection rather than the
/// relation analysis reaches the same three-valued answer through this
/// function instead of restating [`claims_protocol_lbtc`], which is the
/// exhaustive match that must stay the one place the question is
/// answered.
#[must_use]
pub fn ordinary_lbtc_role_of(flows: &BTreeSet<OpenFlowKind>) -> OrdinaryLbtcRole {
    if flows.iter().copied().any(claims_protocol_lbtc) {
        return OrdinaryLbtcRole::ProtocolClaimed;
    }

    if flows.contains(&OpenFlowKind::FeeSponsor) {
        return OrdinaryLbtcRole::SponsorRegion;
    }

    OrdinaryLbtcRole::Absent
}

/// Classify ordinary L-BTC inside one operation.
#[must_use]
pub fn ordinary_lbtc_role(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
) -> OrdinaryLbtcRole {
    ordinary_lbtc_role_of(&declared_open_flows(relations, operation))
}

/// Refuse an operation whose ordinary-L-BTC region cannot be decided.
///
/// The compiler analyzes a scope, and every stage below this one asks
/// whether a given ordinary-L-BTC reference is erased. For an operation
/// whose protocol flows claim the family, that question has no answer
/// in the data the compiler is given, and both available answers are
/// wrong: treating a formula-bound payout as opaque discards a
/// load-bearing protocol relation, and treating the sponsor region as
/// readable breaks erasure. So the scope is refused.
///
/// # Errors
///
/// [`CompileError::UndecidableSponsorRegion`] for the first scope
/// operation whose ordinary-L-BTC region the declarations do not fix.
pub fn validate_decidable_sponsor_regions(
    relations: &CompilerRelationAnalysis,
    operations: &[OperationId],
) -> Result<(), CompileError> {
    for operation in operations {
        if ordinary_lbtc_role(relations, *operation) == OrdinaryLbtcRole::ProtocolClaimed {
            return Err(CompileError::UndecidableSponsorRegion {
                operation: *operation,
            });
        }
    }

    Ok(())
}
