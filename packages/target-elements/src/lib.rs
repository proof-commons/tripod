#![doc = include_str!("../README.md")]
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
    AssetGeneratorDerivation, CommitmentRelation, CommitmentTermRoles, ConfidentialCapabilityState,
    ConfidentialFieldEncoding, ConfidentialReviewFacts, ConfidentialValueCapability,
    ConfidentialValueContract, ConservationContract, ConservationForm, IssuanceContract,
    IssuanceField, OpeningBlocker, OpeningFeasibility, PointParityConvention, ProofRequirements,
    ScalarByteOrder, reviewed_confidential_review_facts,
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
