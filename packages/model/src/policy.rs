//! Branch policy.
//!
//! Implements `´def:verification:root-use´`,
//! `´def:verification:branch-policy´`,
//! `´rule:verification:branch-policy´`, and
//! `´rule:verification:value-flow-matrix´`.
//!
//! One may assume naively that these tables must be hand-maintained
//! and separately checked against the architecture; instead both the
//! branch policy and the value-flow matrix are derived directly from
//! the typed manifest in `tripod-architecture`, so no policy
//! copy exists to drift.
//!
//! For `Redeem`, the generic kernel permits either a normal RESV
//! successor or a terminal edge. The branch-specific postcondition
//! decides which form is legal.

use std::collections::BTreeSet;

use crate::history::BranchKind;
use crate::manifest::{operation_spec, root_use_of, value_flow_of};

// ´def:verification:root-use´

/// Discriminants are the architecture's stable root-use codes
/// (`architecture::RootUse`); the manifest welds the pairing
/// code-for-code at compile time.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootUse {
    Forbidden = 1,

    // The current root is consumed and exactly one
    // valid successor is created.
    Succession = 2,

    // The current root is consumed and either one
    // valid successor is created or the chain terminates.
    SuccessionOrTermination = 3,
}

// ´def:verification:branch-policy´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BranchPolicy {
    pub state: RootUse,
    pub resv: RootUse,
    pub pace: RootUse,
    pub entitlement_authority: RootUse,
    pub distribution_authority: RootUse,

    pub allows_burn_projection: bool,
    pub allows_clear_projection: bool,
    pub allows_distribution_residue_projection: bool,
}

// ´rule:verification:branch-policy´

/// Derived from the typed architecture manifest: root uses come from
/// the operation's root-use declarations, and projection permissions
/// from its projection rules.
pub fn branch_policy(branch: BranchKind) -> BranchPolicy {
    let spec = operation_spec(branch);

    let projection_allowed = |projection: architecture::ProjectionId| {
        spec.projection_rule(projection) != architecture::ProjectionRule::Forbidden
    };

    BranchPolicy {
        state: root_use_of(spec.root_use(architecture::RootId::State)),
        resv: root_use_of(spec.root_use(architecture::RootId::Resv)),
        pace: root_use_of(spec.root_use(architecture::RootId::Pace)),
        entitlement_authority: root_use_of(spec.root_use(architecture::RootId::EntAuth)),
        distribution_authority: root_use_of(spec.root_use(architecture::RootId::DistAuth)),

        allows_burn_projection: projection_allowed(architecture::ProjectionId::BurnEvent),
        allows_clear_projection: projection_allowed(architecture::ProjectionId::ClearEvent),
        allows_distribution_residue_projection: projection_allowed(
            architecture::ProjectionId::DistributionResidue,
        ),
    }
}

// ´rule:verification:value-flow-matrix´

/// Discriminants are the architecture's stable value-flow codes
/// (`architecture::ValueFlowClass`); the manifest welds the pairing
/// code-for-code at compile time.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueFlowClass {
    OwnerConsented = 1,
    ImmutableDestination = 2,
    FormulaBoundPayout = 3,
    PreauthorizedServiceBudget = 4,
    OwnerlessTerminalSink = 5,
    SponsorEnvelope = 6,
    OwnerlessBoundSink = 7,
    OwnerlessBoundMovement = 8,
}

/// Derived from the typed manifest's per-operation value-flow
/// declarations.
pub fn expected_value_flow_classes(branch: BranchKind) -> BTreeSet<ValueFlowClass> {
    operation_spec(branch)
        .value_flows
        .iter()
        .copied()
        .map(value_flow_of)
        .collect()
}
