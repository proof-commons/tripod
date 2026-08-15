//! The development deployment binding, and its combination with the
//! static target contract.
//!
//! # A deployment instance is not the target contract
//!
//! The typed contract describes a compatibility surface. A deployment
//! binding names one network the project intends to exercise that
//! surface against. Keeping them apart means the contract does not
//! change when the network does, and it means a binding cannot quietly
//! edit what the target is said to do: nothing here mutates a
//! [`ValidatedTargetDefinition`].
//!
//! # There is no production constructor
//!
//! [`DeploymentEnvironment::Production`] exists so that the
//! distinction is nameable and so that validation can refuse it. There
//! is no function in this crate that returns a validated production
//! binding, and adding one would require a production evidence
//! boundary that does not exist. A development binding cannot be
//! upgraded into a production one by any public API here.
//!
//! # A declaration is not an observation
//!
//! An [`ActivationDeclaration`] states what the caller *intends* to
//! test against. It is typed input. It is not a report, it does not
//! record what a node did, and passing validation with one does not
//! mean any network satisfies it.
//!
//! # No credentials
//!
//! There is no endpoint, no username, no password, no cookie path, no
//! bearer token, no key, and no wallet path in any type in this
//! module, and none may be added. A future runner that needs to talk
//! to a node needs its own security design; smuggling a field in here
//! — even into a test fixture — would move that boundary without the
//! design.

use std::collections::BTreeSet;

use crate::capability::{CapabilityContract, ElementsCapability, StaticCapabilityStatus};
use crate::definition::{
    ReviewedElementsTapscriptDefinition, TargetContractVersion, TargetProjection,
    ValidatedTargetDefinition,
};
use crate::error::TargetError;
use crate::opcode::LeafVersion;
use crate::resource::{PolicyResourceLimits, ResourceDimension};

/// Which class of deployment a binding describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeploymentEnvironment {
    /// A network under the project's own control, used for testing.
    Development,
    /// A network carrying real value.
    ///
    /// Nameable, and refused. No constructor in this crate produces a
    /// validated binding in this class.
    Production,
}

/// What a caller intends the target environment to have active.
///
/// Typed input, not evidence. Nothing here records an observation, and
/// a validated binding carrying this declaration asserts only that the
/// declaration is internally coherent and agrees with the contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationDeclaration {
    tapscript_expected_active: bool,
    required_leaf_version: LeafVersion,
    required_capabilities: BTreeSet<ElementsCapability>,
}

impl ActivationDeclaration {
    /// States what the caller intends to test against.
    #[must_use]
    pub fn new(
        tapscript_expected_active: bool,
        required_leaf_version: LeafVersion,
        required_capabilities: impl IntoIterator<Item = ElementsCapability>,
    ) -> Self {
        Self {
            tapscript_expected_active,
            required_leaf_version,
            required_capabilities: required_capabilities.into_iter().collect(),
        }
    }

    /// Whether the caller expects the reviewed domain to be active.
    #[must_use]
    pub const fn tapscript_expected_active(&self) -> bool {
        self.tapscript_expected_active
    }

    /// The leaf version the caller expects to be usable.
    #[must_use]
    pub const fn required_leaf_version(&self) -> LeafVersion {
        self.required_leaf_version
    }

    /// The capabilities the caller intends to rely upon.
    #[must_use]
    pub const fn required_capabilities(&self) -> &BTreeSet<ElementsCapability> {
        &self.required_capabilities
    }
}

/// Resource bounds a development deployment enforces beyond the
/// target's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentResourceOverrides {
    policy: PolicyResourceLimits,
}

impl DevelopmentResourceOverrides {
    /// States the overriding policy bounds.
    #[must_use]
    pub const fn new(policy: PolicyResourceLimits) -> Self {
        Self { policy }
    }

    /// The overriding policy bounds.
    #[must_use]
    pub const fn policy(&self) -> &PolicyResourceLimits {
        &self.policy
    }
}

/// One development network the project intends to exercise the target
/// contract against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentDeploymentBinding {
    target_version: TargetContractVersion,
    environment: DeploymentEnvironment,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    activation: ActivationDeclaration,
    resource_overrides: Option<DevelopmentResourceOverrides>,
}

impl DevelopmentDeploymentBinding {
    /// Assembles an unvalidated binding.
    ///
    /// The value this returns carries no guarantee. It is an input to
    /// [`validate_development_binding`], and nothing downstream
    /// accepts it.
    #[must_use]
    pub const fn new(
        target_version: TargetContractVersion,
        environment: DeploymentEnvironment,
        network_id: [u8; 32],
        genesis_id: [u8; 32],
        activation: ActivationDeclaration,
        resource_overrides: Option<DevelopmentResourceOverrides>,
    ) -> Self {
        Self {
            target_version,
            environment,
            network_id,
            genesis_id,
            activation,
            resource_overrides,
        }
    }

    /// The contract revision the binding is stated against.
    #[must_use]
    pub const fn target_version(&self) -> TargetContractVersion {
        self.target_version
    }

    /// The class of deployment.
    #[must_use]
    pub const fn environment(&self) -> DeploymentEnvironment {
        self.environment
    }

    /// The network's identifier.
    #[must_use]
    pub const fn network_id(&self) -> [u8; 32] {
        self.network_id
    }

    /// The network's genesis identifier.
    #[must_use]
    pub const fn genesis_id(&self) -> [u8; 32] {
        self.genesis_id
    }

    /// What the caller intends the environment to have active.
    #[must_use]
    pub const fn activation(&self) -> &ActivationDeclaration {
        &self.activation
    }

    /// Bounds this deployment enforces beyond the target's.
    #[must_use]
    pub const fn resource_overrides(&self) -> Option<&DevelopmentResourceOverrides> {
        self.resource_overrides.as_ref()
    }
}

/// A binding that has passed [`validate_development_binding`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedDevelopmentBinding {
    binding: DevelopmentDeploymentBinding,
}

impl ValidatedDevelopmentBinding {
    /// The validated binding.
    #[must_use]
    pub const fn binding(&self) -> &DevelopmentDeploymentBinding {
        &self.binding
    }

    /// The stable comparison form of this binding.
    ///
    /// It carries the environment class, the two identifiers, the
    /// contract revision, the activation declaration, and any policy
    /// override. It carries no claim that the network actually
    /// satisfies the declaration, because this crate has no way to
    /// find out and no field in which to record an answer.
    #[must_use]
    pub fn projection(&self) -> DeploymentProjection {
        DeploymentProjection {
            environment: self.binding.environment,
            network_id: self.binding.network_id,
            genesis_id: self.binding.genesis_id,
            target_version: self.binding.target_version,
            activation: self.binding.activation.clone(),
            resource_overrides: self.binding.resource_overrides.clone(),
        }
    }
}

/// The stable comparison form of a validated binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeploymentProjection {
    environment: DeploymentEnvironment,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    target_version: TargetContractVersion,
    activation: ActivationDeclaration,
    resource_overrides: Option<DevelopmentResourceOverrides>,
}

impl DeploymentProjection {
    /// The class of deployment.
    #[must_use]
    pub const fn environment(&self) -> DeploymentEnvironment {
        self.environment
    }

    /// The network's identifier.
    #[must_use]
    pub const fn network_id(&self) -> [u8; 32] {
        self.network_id
    }

    /// The network's genesis identifier.
    #[must_use]
    pub const fn genesis_id(&self) -> [u8; 32] {
        self.genesis_id
    }

    /// The contract revision the binding is stated against.
    #[must_use]
    pub const fn target_version(&self) -> TargetContractVersion {
        self.target_version
    }

    /// The declared activation the caller intends to test against.
    #[must_use]
    pub const fn activation(&self) -> &ActivationDeclaration {
        &self.activation
    }

    /// Bounds this deployment enforces beyond the target's.
    #[must_use]
    pub const fn resource_overrides(&self) -> Option<&DevelopmentResourceOverrides> {
        self.resource_overrides.as_ref()
    }
}

/// Validates a deployment binding against a validated contract.
///
/// # Errors
///
/// Rejects a production environment, an all-zero network or genesis
/// identifier, a contract revision the definition does not carry, a
/// declared leaf version the contract does not require, a declared
/// capability the contract marks unsupported, and a resource override
/// looser than the target's own bound.
pub fn validate_development_binding(
    definition: &ValidatedTargetDefinition,
    binding: DevelopmentDeploymentBinding,
) -> Result<ValidatedDevelopmentBinding, TargetError> {
    // Refused first and unconditionally. There is no path by which a
    // production binding becomes valid, and no later check can undo
    // this one.
    if binding.environment != DeploymentEnvironment::Development {
        return Err(TargetError::ProductionBindingUnsupported);
    }

    // An all-zero identifier is the shape an uninitialized buffer
    // takes, so accepting one would let a binding that names no
    // network at all look like a binding that names a network.
    if binding.network_id == [0_u8; 32] {
        return Err(TargetError::ZeroNetworkId);
    }
    if binding.genesis_id == [0_u8; 32] {
        return Err(TargetError::ZeroGenesisId);
    }

    let contract = definition.definition();

    if binding.target_version != contract.version() {
        return Err(TargetError::TargetDeploymentVersionMismatch);
    }

    let activation = &binding.activation;

    if activation.required_leaf_version() != contract.leaf_version() {
        return Err(TargetError::ActivationLeafVersionMismatch);
    }

    // A binding that expects the domain to be inactive while declaring
    // capabilities that only exist inside it describes an environment
    // that cannot satisfy it.
    if !activation.tapscript_expected_active() && !activation.required_capabilities().is_empty() {
        return Err(TargetError::InconsistentActivationDeclaration);
    }

    // A validated contract's capability registry is already complete
    // against its census, so "the contract does not declare this
    // capability" cannot arise here and no branch pretends to check
    // it. What can arise, and what matters more, is a caller
    // declaring that it intends to rely on a capability the contract
    // says no reviewed mechanism provides.
    for capability in activation.required_capabilities() {
        let status = contract
            .capabilities()
            .get(capability)
            .map(CapabilityContract::status);
        if status == Some(StaticCapabilityStatus::Unsupported) {
            return Err(TargetError::UnsupportedRequiredCapability(*capability));
        }
    }

    if let Some(overrides) = binding.resource_overrides.as_ref() {
        let consensus = contract.resources().consensus();
        for (dimension, bound) in overrides.policy().bounds() {
            let Some(ceiling) = consensus.bounds().get(dimension) else {
                return Err(TargetError::UnknownOverrideDimension(*dimension));
            };
            let looser = match (bound.maximum(), ceiling.maximum()) {
                (Some(override_max), Some(consensus_max)) => override_max > consensus_max,
                (None, Some(_)) => true,
                _ => false,
            };
            if looser {
                return Err(TargetError::IncompatibleResourceOverride(*dimension));
            }
        }
    }

    Ok(ValidatedDevelopmentBinding { binding })
}

/// A development binding welded to the exact reviewed contract that
/// validated it.
///
/// # Why a version number is not the weld
///
/// [`ValidatedDevelopmentBinding`] records the contract *revision* it was
/// validated against and nothing more, so a binding validated against one
/// contract combines with any other contract carrying the same revision
/// number. Several internally coherent contracts share a revision while
/// differing in capability status, primitive behavior, failure behavior,
/// encodings, resource bounds, and evidence requirements — and a consumer
/// treating a binding as evidence that *this* contract was the validated
/// one would be reading a number as an identity.
///
/// This state retains the complete validated target projection instead.
/// Equality of the retained projection with a candidate contract's
/// projection is exact typed equality over the whole contract, which is
/// the statement the consumer needs and the version number is not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewedDevelopmentBinding {
    target: TargetProjection,
    deployment: ValidatedDevelopmentBinding,
}

impl ReviewedDevelopmentBinding {
    /// The complete contract projection the binding was validated
    /// against.
    #[must_use]
    pub const fn target(&self) -> &TargetProjection {
        &self.target
    }

    /// The generic validated binding.
    #[must_use]
    pub const fn deployment(&self) -> &ValidatedDevelopmentBinding {
        &self.deployment
    }

    /// The binding itself.
    #[must_use]
    pub const fn binding(&self) -> &DevelopmentDeploymentBinding {
        self.deployment.binding()
    }

    /// The stable comparison form of the binding.
    #[must_use]
    pub fn projection(&self) -> DeploymentProjection {
        self.deployment.projection()
    }

    /// Whether this binding was validated against exactly this contract.
    ///
    /// Exact typed equality over the complete projection, never version
    /// equality.
    #[must_use]
    pub fn welded_to(&self, target: &ReviewedElementsTapscriptDefinition) -> bool {
        self.target == target.projection()
    }
}

/// Validates a deployment binding against one exact reviewed contract.
///
/// The returned state retains that contract's complete projection, so a
/// binding validated here cannot later be combined with a different
/// contract carrying the same revision number.
///
/// # Errors
///
/// Everything [`validate_development_binding`] rejects.
pub fn validate_reviewed_development_binding(
    target: &ReviewedElementsTapscriptDefinition,
    binding: DevelopmentDeploymentBinding,
) -> Result<ReviewedDevelopmentBinding, TargetError> {
    let deployment = validate_development_binding(target.validated(), binding)?;
    Ok(ReviewedDevelopmentBinding {
        target: target.projection(),
        deployment,
    })
}

/// A validated contract paired with a validated development binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementsTarget {
    definition: ValidatedTargetDefinition,
    deployment: ValidatedDevelopmentBinding,
}

impl ElementsTarget {
    /// The validated contract.
    #[must_use]
    pub const fn definition(&self) -> &ValidatedTargetDefinition {
        &self.definition
    }

    /// The validated binding.
    #[must_use]
    pub const fn deployment(&self) -> &ValidatedDevelopmentBinding {
        &self.deployment
    }
}

/// Pairs a validated contract with a validated development binding.
///
/// # What passing this means
///
/// Only that the static contract is internally valid, that the
/// deployment declaration is internally valid, and that the two agree.
///
/// # What passing this does not mean
///
/// That the reviewed domain is active anywhere. That any node behaves
/// as the contract describes. That the network exists. That the target
/// implementation is correct. That production is equivalent. That
/// anything is ready to deploy. Every evidence requirement the
/// contract names remains unresolved after this function returns
/// successfully, and it will remain unresolved until a target-native
/// test produces evidence for it.
///
/// # Errors
///
/// Returns [`TargetError::TargetDeploymentVersionMismatch`] when the
/// two values were validated against different contract revisions.
pub fn bind_development_target(
    definition: ValidatedTargetDefinition,
    deployment: ValidatedDevelopmentBinding,
) -> Result<ElementsTarget, TargetError> {
    if deployment.binding().target_version() != definition.definition().version() {
        return Err(TargetError::TargetDeploymentVersionMismatch);
    }

    Ok(ElementsTarget {
        definition,
        deployment,
    })
}

/// The resource dimensions a development override may narrow.
///
/// Stated so a caller can see the admissible set rather than
/// discovering it by rejection.
#[must_use]
pub fn overridable_dimensions(
    definition: &ValidatedTargetDefinition,
) -> BTreeSet<ResourceDimension> {
    definition
        .definition()
        .resources()
        .consensus()
        .bounds()
        .keys()
        .copied()
        .collect()
}
