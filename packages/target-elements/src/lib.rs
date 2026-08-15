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
//! relation, and the evidence-requirement registry.
//!
//! and the development deployment binding with its validated
//! combination.
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
    SuccessCase, SuccessCondition, SuccessContract, SuccessContractDefect, SuccessStackEffect,
};

#[cfg(test)]
mod tests;
