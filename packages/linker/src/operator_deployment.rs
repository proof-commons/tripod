//! Candidate deployment commitment for operator authorization.

use tapscript::{
    EstablishedOperatorProfile, OperatorKey, StackItem, operator_key_encoding_closure,
};
use target_elements::{ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::LinkRefusal;

/// The network and genesis identity of one candidate deployment.
///
/// The linker owns this value because candidate binding precedes release
/// evidence. Its field names and widths mirror the release deployment profile
/// so a later consumer can compare them without importing release policy here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateDeploymentIdentity {
    network_id: [u8; 32],
    genesis_id: [u8; 32],
}

impl CandidateDeploymentIdentity {
    /// Checks the supplied deployment identifiers.
    ///
    /// # Errors
    /// Returns [`LinkRefusal::ZeroNetworkId`] or [`LinkRefusal::ZeroGenesisId`]
    /// for an all-zero identifier, checking the network first.
    pub fn new(network_id: [u8; 32], genesis_id: [u8; 32]) -> Result<Self, LinkRefusal> {
        if network_id == [0; 32] {
            return Err(LinkRefusal::ZeroNetworkId { network_id });
        }
        if genesis_id == [0; 32] {
            return Err(LinkRefusal::ZeroGenesisId { genesis_id });
        }
        Ok(Self {
            network_id,
            genesis_id,
        })
    }

    /// The supplied network identifier.
    #[must_use]
    pub const fn network_id(&self) -> &[u8; 32] {
        &self.network_id
    }

    /// The supplied genesis identifier.
    #[must_use]
    pub const fn genesis_id(&self) -> &[u8; 32] {
        &self.genesis_id
    }
}

/// The status vocabulary for an operator deployment binding.
///
/// Only `Candidate` is reachable from a binding. The other variants reserve
/// vocabulary for later evidence and promotion; no constructor or setter can
/// assert them. Binding establishes neither curve validity nor a signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OperatorDeploymentStatus {
    /// Bound candidate data, with transaction evidence still owed.
    Candidate,
    /// Operation evidence has passed for the bound deployment.
    CandidateOperationProven,
    /// Approved for production deployment.
    ProductionApproved,
}

/// One operator key and profile committed to one candidate deployment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorDeploymentBinding {
    key: OperatorKey,
    profile: EstablishedOperatorProfile,
    deployment: CandidateDeploymentIdentity,
    capability_revision: TargetContractVersion,
}

impl OperatorDeploymentBinding {
    /// Binds deployment-supplied operator data against the reviewed target.
    ///
    /// The internal key comes from the deployment parameters. Rejecting equal
    /// bytes keeps operator authority distinct from the key path; it does not
    /// establish that either point is curve-valid or the internal key unspendable.
    /// No request supplies or replaces the committed values.
    ///
    /// Owner keys cannot enter this constructor:
    /// ```compile_fail,E0308
    /// use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding};
    /// use tapscript::{EstablishedOperatorProfile, OwnerKey, StackItem};
    /// use target_elements::ReviewedElementsTapscriptDefinition;
    /// fn wrong_role(target: &ReviewedElementsTapscriptDefinition, owner: OwnerKey,
    ///     profile: EstablishedOperatorProfile, deployment: CandidateDeploymentIdentity,
    ///     internal_key: &StackItem) {
    ///     OperatorDeploymentBinding::bind(target, owner, profile, deployment, internal_key);
    /// }
    /// ```
    /// Owner profiles cannot stand in for established operator profiles:
    /// ```compile_fail,E0308
    /// use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding};
    /// use tapscript::{OperatorKey, OwnerSighashProfile, StackItem};
    /// use target_elements::ReviewedElementsTapscriptDefinition;
    /// fn wrong_role(target: &ReviewedElementsTapscriptDefinition, key: OperatorKey,
    ///     profile: OwnerSighashProfile, deployment: CandidateDeploymentIdentity,
    ///     internal_key: &StackItem) {
    ///     OperatorDeploymentBinding::bind(target, key, profile, deployment, internal_key);
    /// }
    /// ```
    /// Carrying an owner disposition does not change its role:
    /// ```compile_fail,E0308
    /// use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding, SelectedSighashProfile};
    /// use tapscript::{OperatorKey, StackItem};
    /// use target_elements::ReviewedElementsTapscriptDefinition;
    /// fn wrong_role(target: &ReviewedElementsTapscriptDefinition, key: OperatorKey,
    ///     profile: SelectedSighashProfile, deployment: CandidateDeploymentIdentity,
    ///     internal_key: &StackItem) {
    ///     OperatorDeploymentBinding::bind(target, key, profile, deployment, internal_key);
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses a stale profile with [`LinkRefusal::InvalidOperatorProfile`],
    /// then an unapproved encoding with [`LinkRefusal::OperatorKeyEncodingMismatch`],
    /// then internal-key reuse with [`LinkRefusal::OperatorKeyIsInternalKey`].
    pub fn bind(
        target: &ReviewedElementsTapscriptDefinition,
        key: OperatorKey,
        profile: EstablishedOperatorProfile,
        deployment: CandidateDeploymentIdentity,
        internal_key: &StackItem,
    ) -> Result<Self, LinkRefusal> {
        let capability_revision = target.definition().version();
        profile
            .check_revision(capability_revision)
            .map_err(LinkRefusal::InvalidOperatorProfile)?;
        let approved =
            operator_key_encoding_closure(target.definition().authorization()).approved();
        if key.encoding() != approved {
            return Err(LinkRefusal::OperatorKeyEncodingMismatch {
                approved,
                offered: key.encoding(),
            });
        }
        if key.bytes() == internal_key.bytes() {
            return Err(LinkRefusal::OperatorKeyIsInternalKey { key });
        }
        Ok(Self {
            key,
            profile,
            deployment,
            capability_revision,
        })
    }

    /// The committed operator public key.
    #[must_use]
    pub const fn key(&self) -> &OperatorKey {
        &self.key
    }

    /// The committed established operator profile.
    #[must_use]
    pub const fn profile(&self) -> &EstablishedOperatorProfile {
        &self.profile
    }

    /// The committed candidate deployment identity.
    #[must_use]
    pub const fn deployment(&self) -> &CandidateDeploymentIdentity {
        &self.deployment
    }

    /// The reviewed target revision pinned at binding.
    #[must_use]
    pub const fn capability_revision(&self) -> TargetContractVersion {
        self.capability_revision
    }

    /// The read-only candidate status; promotion has no entry point.
    #[must_use]
    pub const fn status(&self) -> OperatorDeploymentStatus {
        OperatorDeploymentStatus::Candidate
    }

    /// Checks the transaction boundary's offered key, deployment, and revision.
    ///
    /// # Errors
    /// Returns [`LinkRefusal::OperatorKeyMismatch`] for different encoding or
    /// bytes, then [`LinkRefusal::OperatorDeploymentMismatch`] for either changed
    /// identifier, then [`LinkRefusal::InvalidOperatorProfile`] for a stale pin.
    pub fn check(
        &self,
        key: &OperatorKey,
        deployment: &CandidateDeploymentIdentity,
        offered: TargetContractVersion,
    ) -> Result<(), LinkRefusal> {
        if key.encoding() != self.key.encoding() || key.bytes() != self.key.bytes() {
            return Err(LinkRefusal::OperatorKeyMismatch {
                bound: self.key.clone(),
                offered: key.clone(),
            });
        }
        if deployment != &self.deployment {
            return Err(LinkRefusal::OperatorDeploymentMismatch {
                bound: Box::new(self.deployment.clone()),
                offered: Box::new(deployment.clone()),
            });
        }
        self.profile
            .check_revision(offered)
            .map_err(LinkRefusal::InvalidOperatorProfile)
    }

    /// Checks another established profile against the committed value.
    ///
    /// The only public way to obtain an established profile is establishment
    /// from the source selection. Two established profiles can therefore differ
    /// only across revisions or a changed source selection. Compare by value,
    /// including the established dimensions and revision, rather than by name.
    ///
    /// # Errors
    /// Returns [`LinkRefusal::OperatorProfileMismatch`] for any value difference.
    pub fn profile_matches(&self, other: &EstablishedOperatorProfile) -> Result<(), LinkRefusal> {
        if other != &self.profile {
            return Err(LinkRefusal::OperatorProfileMismatch {
                bound: Box::new(self.profile.clone()),
                offered: Box::new(other.clone()),
            });
        }
        Ok(())
    }
}
