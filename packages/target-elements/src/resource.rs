//! Consensus and policy resource interfaces.
//!
//! # The units stay apart
//!
//! Transaction weight, stack depth, element bytes, and the script-path
//! validation budget are measured in different things and bound
//! different limits. There is no combined score here and there must
//! not be one: a weighted total would be expressed in a unit that no
//! target limit is stated in, so no comparison against it would mean
//! anything.
//!
//! # Consensus and policy stay apart
//!
//! A deployment may enforce a stricter bound than consensus requires.
//! Collapsing the two would make a policy-only rejection look like an
//! invalid transaction, and would make a policy relaxation look like a
//! consensus change.
//!
//! # No calibration
//!
//! Nothing here derives a protocol batch bound, input count, or
//! sponsor bound from a target limit. Those remain architecture
//! decisions until complete transactions exist to measure.

use std::collections::BTreeMap;

use crate::capability::census_enum;

census_enum! {
    /// One resource the target accounts for, in its own unit.
    #[non_exhaustive]
    pub enum ResourceDimension {
        /// Weight units of a whole transaction.
        TransactionWeight,
        /// Serialized bytes of the witness.
        WitnessBytes,
        /// Bytes of one script.
        ScriptBytes,
        /// Items on the witness stack before execution begins.
        InitialStackItems,
        /// The greatest combined main and alternate stack depth reached.
        PeakStackItems,
        /// Bytes of one stack element.
        StackElementBytes,
        /// The script-path validation budget.
        ValidationBudget,
        /// The per-script operation budget.
        OperationCost,
        /// Nodes on a control path from the root to the executing leaf.
        ControlPathDepth,
        /// Transactions or weight admitted in one package.
        PackageLimit,
    }
}

impl ResourceDimension {
    /// The dimensions a consensus contract must state.
    ///
    /// A missing entry among these is an incomplete contract, not a
    /// target that happens to be unbounded there: each of them is
    /// enforced by the reviewed execution domain.
    pub const REQUIRED_BY_CONSENSUS: &'static [Self] = &[
        Self::TransactionWeight,
        Self::PeakStackItems,
        Self::StackElementBytes,
        Self::ValidationBudget,
        Self::ControlPathDepth,
    ];
}

/// A bound on one dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResourceBound {
    /// The reviewed domain enforces no bound on this dimension.
    ///
    /// Distinct from a bound of zero, which would forbid the resource
    /// entirely, and distinct from an absent entry, which would mean
    /// the contract simply failed to say.
    Unbounded,
    /// The greatest admissible value.
    Maximum(u64),
}

impl ResourceBound {
    /// The bound's numeric ceiling, if it has one.
    #[must_use]
    pub const fn maximum(self) -> Option<u64> {
        match self {
            Self::Unbounded => None,
            Self::Maximum(value) => Some(value),
        }
    }
}

/// The bounds the target's own rules enforce.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusResourceLimits {
    bounds: BTreeMap<ResourceDimension, ResourceBound>,
    witness_scale_factor: u64,
    validation_budget_offset: u64,
}

impl ConsensusResourceLimits {
    /// States the consensus limits.
    #[must_use]
    pub fn new(
        bounds: impl IntoIterator<Item = (ResourceDimension, ResourceBound)>,
        witness_scale_factor: u64,
        validation_budget_offset: u64,
    ) -> Self {
        Self {
            bounds: bounds.into_iter().collect(),
            witness_scale_factor,
            validation_budget_offset,
        }
    }

    /// The bound on each stated dimension.
    #[must_use]
    pub const fn bounds(&self) -> &BTreeMap<ResourceDimension, ResourceBound> {
        &self.bounds
    }

    /// The factor relating non-witness bytes to weight units.
    #[must_use]
    pub const fn witness_scale_factor(&self) -> u64 {
        self.witness_scale_factor
    }

    /// The constant added to the witness size to form the starting
    /// validation budget.
    #[must_use]
    pub const fn validation_budget_offset(&self) -> u64 {
        self.validation_budget_offset
    }
}

/// The bounds a deployment enforces beyond consensus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyResourceLimits {
    bounds: BTreeMap<ResourceDimension, ResourceBound>,
}

impl PolicyResourceLimits {
    /// States the policy limits.
    #[must_use]
    pub fn new(bounds: impl IntoIterator<Item = (ResourceDimension, ResourceBound)>) -> Self {
        Self {
            bounds: bounds.into_iter().collect(),
        }
    }

    /// The bound on each stated dimension.
    #[must_use]
    pub const fn bounds(&self) -> &BTreeMap<ResourceDimension, ResourceBound> {
        &self.bounds
    }
}

/// The target's resource interfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceContract {
    consensus: ConsensusResourceLimits,
    policy: PolicyResourceLimits,
}

impl ResourceContract {
    /// States the resource contract.
    #[must_use]
    pub const fn new(consensus: ConsensusResourceLimits, policy: PolicyResourceLimits) -> Self {
        Self { consensus, policy }
    }

    /// The bounds the target's own rules enforce.
    #[must_use]
    pub const fn consensus(&self) -> &ConsensusResourceLimits {
        &self.consensus
    }

    /// The bounds a deployment enforces beyond them.
    #[must_use]
    pub const fn policy(&self) -> &PolicyResourceLimits {
        &self.policy
    }

    /// The first required consensus dimension the contract omits.
    #[must_use]
    pub fn missing_consensus_dimension(&self) -> Option<ResourceDimension> {
        ResourceDimension::REQUIRED_BY_CONSENSUS
            .iter()
            .find(|dimension| !self.consensus.bounds.contains_key(dimension))
            .copied()
    }

    /// The first dimension where a bound of zero forbids a resource
    /// the target requires.
    #[must_use]
    pub fn zero_bound(&self) -> Option<ResourceDimension> {
        self.consensus
            .bounds
            .iter()
            .chain(self.policy.bounds.iter())
            .find(|(_, bound)| bound.maximum() == Some(0))
            .map(|(dimension, _)| *dimension)
    }

    /// The first dimension where policy is looser than consensus.
    ///
    /// A deployment may be stricter than consensus. It cannot be
    /// looser: a transaction consensus refuses is not made valid by a
    /// permissive local policy, so a policy bound above the consensus
    /// one describes a state that cannot exist.
    #[must_use]
    pub fn policy_looser_than_consensus(&self) -> Option<ResourceDimension> {
        self.policy.bounds.iter().find_map(|(dimension, policy)| {
            let consensus = self.consensus.bounds.get(dimension)?;
            match (policy.maximum(), consensus.maximum()) {
                (Some(policy_max), Some(consensus_max)) if policy_max > consensus_max => {
                    Some(*dimension)
                }
                // An unbounded policy where consensus bounds the
                // dimension is looser too.
                (None, Some(_)) => Some(*dimension),
                _ => None,
            }
        })
    }
}

/// Builds the reviewed resource contract.
pub(crate) fn reviewed_resources() -> ResourceContract {
    use ResourceBound::{Maximum, Unbounded};
    use ResourceDimension as D;

    ResourceContract::new(
        ConsensusResourceLimits::new(
            [
                (D::TransactionWeight, Maximum(4_000_000)),
                (D::PeakStackItems, Maximum(1_000)),
                (D::StackElementBytes, Maximum(520)),
                (D::ControlPathDepth, Maximum(128)),
                // The starting budget depends on the witness size, so
                // no constant ceiling applies; the offset and the
                // per-check charge are what a planner needs.
                (D::ValidationBudget, Unbounded),
                // The reviewed execution domain enforces neither of
                // these, unlike the pre-tapscript domains. Recording
                // them as unbounded is a reviewed fact; omitting them
                // would have read as an oversight.
                (D::ScriptBytes, Unbounded),
                (D::OperationCost, Unbounded),
            ],
            4,
            50,
        ),
        PolicyResourceLimits::new([(D::TransactionWeight, Maximum(400_000))]),
    )
}
