//! The typed, reviewed Elements tapscript target compatibility
//! contract.
//!
//! This crate states target facts. It states nothing about the attestation contract
//! .
//!
//! # Boundary
//!
//! The package owns the tapscript execution domain and leaf version,
//! reviewed opcode identities and their stack and failure contracts,
//! field-specific operand and result encodings, sighash and
//! relative-timelock dimensions, confidential-value and issuance
//! capability descriptions, consensus and policy resource interfaces,
//! and the registry of target evidence that a future deployment must
//! produce.
//!
//! It owns no attestation-contract operation, object, relation, proof
//! plan, authorization policy, batch bound, or transaction layout.
//! Nothing here maps a target fact onto a protocol meaning; that
//! mapping belongs to a downstream adapter, which may depend on this
//! crate while this crate depends on nothing.
//!
//! # Dependencies
//!
//! None. Not first-party, not third-party. Every declaration is a
//! standard-library type. The crate serializes nothing, hashes
//! nothing, parses nothing, and opens no file.
//!
//! # Review provenance is not target identity
//!
//! The typed facts here were transcribed from a reviewed reading of
//! upstream Elements interpreter source. The upstream repository, the
//! revision consulted, the source paths, the node version, and the
//! review date are *review provenance*: they are recorded in the human
//! reference under `plans/reference/elements-tapscript.md` and they
//! never enter this crate's types or its stable projections. The
//! contract identifies a typed compatibility surface, not one
//! implementation revision.
//!
//! # Identity
//!
//! The crate mints no digest. There is no target-definition hash, no
//! deployment-instance hash, and no field reserved for one. Direct
//! typed comparison of validated values is the whole comparison
//! mechanism, and a stable contract version carries the one
//! compatibility decision a consumer actually makes.
//!
//! # State
//!
//! Implemented: the crate boundary, the typed error root, the
//! target-contract version, the tapscript execution domain and leaf
//! version, the reviewed primitive registry with complete stack,
//! failure, and resource contracts, the target validator, the stable
//! semantic projection,
//! the field-specific encoding registry, the literal-push forms with
//! their minimal-form rule and its enforcement, signature and sighash and
//! relative-timelock dimensions, confidential-value and issuance
//! capability descriptions, separate consensus and policy resource
//! interfaces, the capability registry with an acyclic prerequisite
//! relation, the evidence-requirement registry, and the development
//! deployment binding with its validated combination.
//!
//! # Requirements, not evidence completion
//!
//! Not claimed: any target-native evidence whatever. This crate owns
//! the requirements and carries no mutable evidence-completion status
//! for them, which is what keeps a static contract from depending on
//! when a run last happened.
//!
//! Evidence about the requirements named here is produced and recorded
//! separately, by `tripod-target-elements-conformance`, and
//! development native evidence exists there. Nothing about that is
//! readable from this crate, deliberately. Production target evidence
//! remains absent, and there is no production deployment binding.
//!
//! # Public modules
//!
//! The contract and its trust states:
//!
//! - [`definition`] — [`TargetDefinition`] and its parts, the
//!   [`TargetContractVersion`] revision key, the two trust states
//!   ([`ValidatedTargetDefinition`] and
//!   [`ReviewedElementsTapscriptDefinition`]), the stable
//!   [`TargetProjection`], and the validator.
//! - [`deployment`] — the development-only binding: what a caller
//!   declares, its validation, and the welded and paired results.
//! - [`error`] — [`TargetError`], the crate's single error root.
//!
//! The reviewed facts the contract carries:
//!
//! - [`opcode`] — the execution domain, the leaf version, the primitive
//!   identities, and each primitive's stack, failure, and resource
//!   contract.
//! - [`operand`] — what one operand position constrains, including the
//!   V2 position that constrains nothing.
//! - [`success`] — the success algebra: what a primitive leaves on the
//!   stack, including forms whose results depend on their operands.
//! - [`encoding`] — the field-specific encoding registry.
//! - [`push`] — the literal-push forms, the minimal-form rule, and
//!   which rule is consensus and which is relay policy.
//! - [`authorization`] — signature behavior, the sighash dimensions,
//!   and the relative-timelock contract.
//! - [`confidential`] — confidential-value and issuance descriptions.
//! - [`resource`] — the separate consensus and policy interfaces.
//! - [`capability`] — the capability vocabulary, each capability's
//!   typed contract, and the graph predicates over the prerequisite
//!   relation.
//! - [`evidence`] and [`evidence_registry`] — the identities of the
//!   evidence a future deployment must produce, and what each one is
//!   about.
//!
//! # Quickstart
//!
//! Obtain the reviewed contract, validate a binding against it, then
//! inspect a capability.
//!
//! ```
//! use target_elements::{
//!     ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding,
//!     ElementsCapability, LeafVersion, StaticCapabilityStatus, TargetContractVersion, TargetError,
//!     reviewed_elements_tapscript, validate_reviewed_development_binding,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // The reviewed contract has no public constructor. This is the one
//! // way to obtain it, and it is a V2 contract.
//! let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
//! assert_eq!(target.definition().version(), TargetContractVersion::V2);
//!
//! // A binding names one network a caller intends to test against.
//! let binding = DevelopmentDeploymentBinding::new(
//!     TargetContractVersion::V2,
//!     DeploymentEnvironment::Development,
//!     [0x11; 32],
//!     [0x22; 32],
//!     ActivationDeclaration::new(
//!         true,
//!         LeafVersion::TAPSCRIPT,
//!         [ElementsCapability::TapscriptExecution],
//!     ),
//!     None,
//! );
//! let bound = validate_reviewed_development_binding(&target, binding)?;
//!
//! // The binding is welded to the whole projection of the contract
//! // that validated it, not to its revision number.
//! assert!(bound.welded_to(&target));
//!
//! // Production is nameable precisely so that validation can refuse
//! // it. No function here returns a validated production binding.
//! let production = DevelopmentDeploymentBinding::new(
//!     TargetContractVersion::V2,
//!     DeploymentEnvironment::Production,
//!     [0x11; 32],
//!     [0x22; 32],
//!     ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
//!     None,
//! );
//! assert!(matches!(
//!     validate_reviewed_development_binding(&target, production),
//!     Err(TargetError::ProductionBindingUnsupported),
//! ));
//!
//! // Support is not a boolean. A validated contract states a contract
//! // for every capability in the census, so the lookup is total.
//! let capabilities = target.definition().capabilities();
//! let status = |capability| {
//!     capabilities
//!         .get(&capability)
//!         .expect("a validated contract states every capability")
//!         .status()
//! };
//!
//! // Reviewed means the typed contract was checked against upstream
//! // source. It does not mean a node was ever asked.
//! assert_eq!(
//!     status(ElementsCapability::TapscriptExecution),
//!     StaticCapabilityStatus::Reviewed,
//! );
//! // The review reached the signature primitives but not the sighash
//! // construction, and that is recorded rather than guessed.
//! assert_eq!(
//!     status(ElementsCapability::OutputCommittingSighash),
//!     StaticCapabilityStatus::Incomplete,
//! );
//! // No reviewed primitive orders two byte strings at all.
//! assert_eq!(
//!     status(ElementsCapability::CanonicalByteOrdering),
//!     StaticCapabilityStatus::Unsupported,
//! );
//! # Ok(())
//! # }
//! ```
//!
//! # Errors
//!
//! Every fallible operation returns [`TargetError`], which implements
//! `core::error::Error`. The one exception in shape rather than type is
//! [`validate_target_definition`], which returns `Vec<TargetError>`
//! because it reports every defect in a definition rather than the
//! first — as does [`reviewed_elements_tapscript`], which wraps it.
//!
//! Two comparisons are worth keeping straight.
//! [`StaticCapabilityStatus`] derives `Ord` in declaration order, which
//! is the inverse of semantic strength; compare with
//! [`StaticCapabilityStatus::strength`] or
//! [`StaticCapabilityStatus::at_most`], never with `<`.
//!
//! The full public-API tour, the primitive census, and the worked
//! discussion of each boundary are in the package README.

#![forbid(unsafe_code)]

pub mod authorization;
pub mod capability;
pub mod confidential;
pub mod definition;
pub mod deployment;
pub mod encoding;
pub mod error;
pub mod evidence;
pub mod evidence_registry;
pub mod opcode;
pub mod operand;
pub mod push;
pub mod resource;
pub mod success;
mod weld;

pub use authorization::{
    AuthorizationContract, RelativeTimelockContract, SequenceFieldLayout, SighashCapability,
    SighashDimension, SignaturePrimitiveContract, TimelockMode, UnknownPublicKeyTypeRule,
};
pub use capability::{
    CapabilityContract, ElementsCapability, StaticCapabilityStatus, prerequisite_cycle_residual,
    status_closure_violations, transitive_prerequisites,
};
pub use confidential::{
    ConfidentialCapabilityState, ConfidentialValueCapability, ConfidentialValueContract,
    IssuanceContract, IssuanceField,
};
pub use definition::{
    ReviewedElementsTapscriptDefinition, TargetContractVersion, TargetDefinition,
    TargetDefinitionParts, TargetProjection, ValidatedTargetDefinition,
    reviewed_elements_tapscript, validate_as_reviewed_elements, validate_target_definition,
};
pub use deployment::{
    ActivationDeclaration, DeploymentEnvironment, DeploymentProjection,
    DevelopmentDeploymentBinding, DevelopmentResourceOverrides, ElementsTarget,
    ReviewedDevelopmentBinding, ValidatedDevelopmentBinding, bind_development_target,
    overridable_dimensions, validate_development_binding, validate_reviewed_development_binding,
};
pub use encoding::{
    ByteOrder, CanonicalEncodingRule, EncodingClass, EncodingDomain, EncodingSpec,
    PayloadInterpretation, PayloadWidth, UnknownPrefixRule, V1EncodingShape,
};
pub use error::TargetError;
pub use evidence::TargetEvidenceRequirementId;
pub use evidence_registry::{
    EvidenceClaimClass, EvidenceStaleCondition, RequiredEvidenceEnvironment,
    TargetEvidenceRequirement, TargetEvidenceSubject,
};
pub use opcode::{
    ExecutionDomain, FailureCause, FailureContract, FailureEffect, FailureOutcome, LeafVersion,
    OpcodeId, OpcodeResourceCost, OpcodeSpec, StackContract, StackValueType,
};
pub use operand::{OperandContract, PublicKeyOperandFacts, SignatureOperandFacts};
pub use push::{
    PushContract, PushContractDefect, PushDefect, PushEnforcement, PushForm, PushFormParts,
    PushFormSpec, PushMinimalityStep, PushOpcodeMapping, PushPayloadPredicate,
};
pub use resource::{
    ConsensusResourceLimits, PolicyResourceLimits, ResourceBound, ResourceContract,
    ResourceDimension,
};
pub use success::{
    ResultValue, SuccessCase, SuccessCondition, SuccessContract, SuccessContractDefect,
    SuccessStackEffect,
};

#[cfg(test)]
mod tests;
