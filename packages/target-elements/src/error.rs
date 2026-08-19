//! The target package's single error root.
//!
//! Every validation failure in this crate is reported as a
//! [`TargetError`]. The enum is deliberately hand-written rather than
//! derived: the package carries no dependencies at all, so neither a
//! derive macro nor a `Display` helper crate is available, and none is
//! wanted for a vocabulary this small.
//!
//! # Admission rule
//!
//! A variant exists only when some validation branch in this crate
//! actually constructs it. A variant reserved for a failure no code
//! path can reach would be a claim about validation that has not been
//! implemented, so the vocabulary grows with the validators rather
//! than ahead of them.

use core::fmt;

use crate::authorization::SighashDimension;
use crate::capability::ElementsCapability;
use crate::confidential::ConfidentialValueCapability;
use crate::encoding::EncodingClass;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::{FailureCause, OpcodeId};
use crate::push::{PushDefect, PushForm};
use crate::resource::ResourceDimension;
use crate::success::SuccessCondition;

/// A typed target-contract failure.
///
/// Validation of a target definition reports *all* diagnostics rather
/// than the first, so this type is normally seen inside a
/// `Vec<TargetError>`. The deployment-binding and combination
/// validators reject on a single typed reason and return one value.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetError {
    /// A target-contract version was offered that this crate does not
    /// implement. The version is an explicit compatibility decision,
    /// not an identity digest: an adapter that cannot interpret a
    /// revision must refuse it rather than guess.
    UnsupportedTargetContractVersion {
        /// The unsupported version number that was offered.
        offered: u32,
    },

    /// A leaf version was offered that this package has not reviewed.
    /// A leaf version selects the semantics of everything executed
    /// beneath it, so an unreviewed byte describes semantics this
    /// contract cannot speak for.
    UnreviewedLeafVersion {
        /// The unreviewed leaf version byte that was offered.
        offered: u8,
    },

    /// An offered contract is internally coherent but is not the
    /// first-party reviewed Elements contract, so it cannot carry the
    /// reviewed trust state. This is not a claim that the offered
    /// contract is malformed: it is a claim about whose contract it
    /// is.
    ReviewedDefinitionMismatch,

    /// A reviewed primitive identity has no contract in the registry.
    MissingOpcodeContract(OpcodeId),

    /// A registry entry claims one identity while being filed under
    /// another, which makes the key the registry is indexed by a lie.
    OpcodeIdMismatch {
        /// The identity the entry is filed under.
        key: OpcodeId,
        /// The identity the entry claims.
        declared: OpcodeId,
    },

    /// Two registry entries claim the same target byte. One of them
    /// would be unreachable, and which one is not determinable from
    /// the contract.
    DuplicateOpcodeCode(u8),

    /// A primitive declares no execution domain, or declares none that
    /// includes the domain the contract describes.
    UnsupportedOpcodeExecutionDomain(OpcodeId),

    /// A primitive's operand or result widths are incoherent, so the
    /// contract describes no admissible stack shape.
    InvalidOpcodeStackContract(OpcodeId),

    /// A primitive offers alternative successful forms but names none,
    /// so the contract states no successful behavior at all.
    IncompleteSuccessContract(OpcodeId),

    /// A primitive names two successful forms under the same
    /// condition, so the contract does not say which one the target
    /// produces.
    DuplicateSuccessCase {
        /// The primitive carrying the duplicate.
        opcode: OpcodeId,
        /// The condition claimed twice.
        case: SuccessCondition,
    },

    /// A successful form consumes more operands than the primitive
    /// declares, so its resulting stack depth is not computable.
    ContradictorySuccessCase(OpcodeId),

    /// A primitive declares that it retains its operands while
    /// declaring no operands to retain.
    InvalidRetainedOperandContract(OpcodeId),

    /// A successful form carries through an operand position the
    /// primitive does not declare, so there is no item to carry.
    UndeclaredCarriedOperand(OpcodeId),

    /// A primitive declares no failure behavior at all. Every reviewed
    /// primitive can fail, so an empty failure contract is an
    /// incomplete transcription.
    MissingOpcodeFailureContract(OpcodeId),

    /// A primitive declares one failure cause with two different
    /// effects, so the contract does not say what the target does.
    ContradictoryFailureCause {
        /// The primitive carrying the contradiction.
        opcode: OpcodeId,
        /// The cause declared twice.
        cause: FailureCause,
    },

    /// A primitive declares no resource cost where the target requires
    /// a positive one.
    MissingOpcodeResourceCost(OpcodeId),

    /// A primitive names no evidence requirement, so its semantics
    /// rest on this crate's assertion alone and no deployment is ever
    /// asked to demonstrate them.
    MissingOpcodeEvidence(OpcodeId),

    /// An encoding key has no specification in the registry.
    MissingEncodingSpec(EncodingClass),

    /// An encoding entry claims one identity while filed under
    /// another.
    EncodingClassMismatch {
        /// The identity the entry is filed under.
        key: EncodingClass,
        /// The identity the entry claims.
        declared: EncodingClass,
    },

    /// Two encodings in one field group claim the same prefix byte, so
    /// a decoder cannot tell their forms apart.
    DuplicateEncodingPrefix {
        /// The encoding whose prefix collides.
        class: EncodingClass,
        /// The contested prefix byte.
        prefix: u8,
    },

    /// An encoding's admissible widths are incoherent, so it describes
    /// no representable value.
    InvalidEncodingWidth(EncodingClass),

    /// An encoding carries a number but states no byte order, so its
    /// bytes do not determine a value.
    MissingByteOrder(EncodingClass),

    /// An encoding carries opaque bytes but states a byte order,
    /// claiming a numeric interpretation the field does not have.
    SpuriousByteOrder(EncodingClass),

    /// A numeric encoding states an order the contract revision does
    /// not give that class, so its bytes decode to a different value.
    EncodingByteOrderMismatch(EncodingClass),

    /// An encoding is filed under a field group the contract revision
    /// does not put it in, so its prefixes would have to be
    /// unambiguous against the wrong set of forms.
    EncodingDomainMismatch(EncodingClass),

    /// An encoding states a width the contract revision does not give
    /// that class.
    EncodingWidthMismatch(EncodingClass),

    /// An encoding states a canonicality rule the contract revision
    /// does not give that class, so a decoder would accept or reject
    /// the wrong byte strings.
    EncodingCanonicalityMismatch(EncodingClass),

    /// An encoding names no evidence requirement.
    MissingEncodingEvidence(EncodingClass),

    /// A sighash dimension is recorded as both reviewed and
    /// unreviewed.
    ContradictorySighashDimension(SighashDimension),

    /// A sighash dimension is recorded neither way, so the contract is
    /// silent about whether the review reached it.
    UnclassifiedSighashDimension(SighashDimension),

    /// The sequence disable, mode, and value fields share bits, so a
    /// decoder cannot separate them.
    OverlappingSequenceFields,

    /// The relative-timelock contract declares no mode at all.
    MissingTimelockMode,

    /// A confidential-value claim is left unclassified.
    UnclassifiedConfidentialCapability(ConfidentialValueCapability),

    /// A required consensus resource dimension is absent, so the
    /// contract simply fails to say what the target enforces.
    MissingResourceDimension(ResourceDimension),

    /// A resource bound forbids a resource the target requires.
    InvalidResourceContract(ResourceDimension),

    /// A policy bound is looser than the consensus bound it sits
    /// under, describing a state that cannot exist.
    PolicyLooserThanConsensus(ResourceDimension),

    /// A capability key has no contract in the registry.
    MissingCapabilityContract(ElementsCapability),

    /// A capability entry claims one identity while filed under
    /// another.
    CapabilityIdMismatch {
        /// The identity the entry is filed under.
        key: ElementsCapability,
        /// The identity the entry claims.
        declared: ElementsCapability,
    },

    /// A capability requires a capability the contract does not
    /// declare.
    UnknownCapabilityPrerequisite(ElementsCapability),

    /// A capability is built from a primitive the contract does not
    /// declare.
    CapabilityNamesUnknownOpcode {
        /// The capability naming it.
        capability: ElementsCapability,
        /// The primitive that is absent.
        opcode: OpcodeId,
    },

    /// A capability depends on an encoding the contract does not
    /// declare.
    CapabilityNamesUnknownEncoding {
        /// The capability naming it.
        capability: ElementsCapability,
        /// The encoding that is absent.
        encoding: EncodingClass,
    },

    /// A capability names no evidence requirement.
    MissingCapabilityEvidence(ElementsCapability),

    /// A capability claims a stronger status than one of the
    /// capabilities it transitively requires. Status is closed over
    /// the prerequisite relation, because a capability cannot be more
    /// usable than the weakest thing it is built on.
    CapabilityStatusExceedsPrerequisite {
        /// The capability claiming too much.
        capability: ElementsCapability,
        /// The weakest prerequisite it exceeds.
        prerequisite: ElementsCapability,
    },

    /// Capabilities require each other in a cycle, so no order in
    /// which they could be established exists.
    ///
    /// The members are every capability on a cycle or depending on
    /// one, which is the complete set a reviewer must look at.
    CapabilityDependencyCycle {
        /// The capabilities that cannot be ordered.
        members: Vec<ElementsCapability>,
    },

    /// Something names an evidence requirement the registry does not
    /// declare.
    UnknownEvidenceRequirement(TargetEvidenceRequirementId),

    /// An evidence requirement claims one identity while filed under
    /// another.
    EvidenceRequirementIdMismatch {
        /// The identity the entry is filed under.
        key: TargetEvidenceRequirementId,
        /// The identity the entry claims.
        declared: TargetEvidenceRequirementId,
    },

    /// An evidence requirement states no condition under which
    /// evidence for it goes stale, so a report for it would never
    /// expire.
    MissingStaleCondition(TargetEvidenceRequirementId),

    /// A reviewed push form has no contract. The census is the
    /// authority: a missing form is an incomplete contract rather than
    /// a target that pushes literals in fewer ways.
    MissingPushForm(PushForm),

    /// A push form states an opcode span or a width range that admits
    /// nothing.
    InvalidPushForm(PushForm),

    /// Two push forms claim the same opcode byte, so a decoder could
    /// not say which form a script used.
    DuplicatePushOpcode(u8),

    /// A push form names no evidence, so its rule rests on this
    /// crate's assertion alone.
    MissingPushFormEvidence(PushForm),

    /// A push defect has no stated enforcement, so a consumer could not
    /// tell a program the target refuses from one nodes decline to
    /// relay.
    MissingPushEnforcement(PushDefect),

    /// The push forms, the minimal-form rule, the literal bound, the
    /// primitive bytes, and the evidence link do not describe the same
    /// rules.
    PushContractMismatch,

    /// The signature primitives, the signature primitive contract, the
    /// per-check budget, the operand encodings, and the evidence link
    /// do not describe the same behavior.
    SignatureContractMismatch,

    /// The timelock primitive, the relative-timelock contract, the
    /// version prerequisite, and the evidence link do not describe the
    /// same behavior.
    TimelockContractMismatch,

    /// The confidential-value claim states and the capability rows
    /// describing the same claims do not agree.
    ConfidentialContractMismatch,

    /// The issuance census, the introspection result, the null marker,
    /// the outpoint flag, and the issuance capabilities do not agree.
    IssuanceContractMismatch,

    /// The per-opcode resource costs, the per-check budget, and the
    /// consensus and policy dimensions do not agree.
    ResourceContractMismatch,

    /// A primitive's declared maximum stack growth is not the growth
    /// its own success and non-aborting failure effects imply, or it
    /// claims an alternate-stack growth no reviewed behavior produces.
    ///
    /// Separate from [`Self::ResourceContractMismatch`] because the
    /// budget dimensions and the depth dimension are welded against
    /// different views: the budget against the signature subcontract,
    /// the depth against each primitive's own stack algebra.
    StackGrowthContractMismatch,

    /// A subcontract names no evidence requirement, or names one the
    /// registry does not declare.
    EvidenceContractMismatch,

    /// A binding named the production environment. There is no
    /// production evidence boundary, so there is no production
    /// binding.
    ProductionBindingUnsupported,

    /// A binding carried an all-zero network identifier, which is the
    /// shape of an uninitialized buffer rather than of a network.
    ZeroNetworkId,

    /// A binding carried an all-zero genesis identifier.
    ZeroGenesisId,

    /// A binding was stated against a different contract revision than
    /// the definition it was checked against.
    TargetDeploymentVersionMismatch,

    /// A binding declared a leaf version the contract does not
    /// require, so the two describe different execution semantics.
    ActivationLeafVersionMismatch,

    /// A binding expects the execution domain to be inactive while
    /// declaring capabilities that exist only inside it.
    InconsistentActivationDeclaration,

    /// A binding declares that it intends to rely on a capability the
    /// contract marks unsupported. A deployment cannot make available
    /// what no reviewed mechanism provides, so this is a caller
    /// mistake rather than an environment to go and check.
    UnsupportedRequiredCapability(ElementsCapability),

    /// A resource override names a dimension the target does not
    /// bound, so there is nothing for it to narrow.
    UnknownOverrideDimension(ResourceDimension),

    /// A resource override is looser than the target's own bound. A
    /// deployment may narrow what the target permits; it cannot widen
    /// it.
    IncompatibleResourceOverride(ResourceDimension),
}

impl fmt::Display for TargetError {
    // One exhaustive match over the whole vocabulary. Its length
    // measures how many distinct failures the validators report, not
    // how complicated the function is: every arm is a single `write!`
    // and there is no branching beyond the match itself. Splitting it
    // into groups would need either a catch-all arm, which silently
    // swallows a new variant's message, or three passes over the same
    // value, which is worse to read than the list.
    #[expect(clippy::too_many_lines, reason = "one arm per reported failure")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTargetContractVersion { offered } => {
                write!(f, "unsupported target contract version {offered}")
            }
            Self::UnreviewedLeafVersion { offered } => {
                write!(f, "unreviewed leaf version {offered:#04x}")
            }
            Self::ReviewedDefinitionMismatch => {
                write!(
                    f,
                    "the offered contract is not the reviewed Elements contract"
                )
            }
            Self::MissingOpcodeContract(id) => {
                write!(f, "reviewed opcode {id:?} has no contract")
            }
            Self::OpcodeIdMismatch { key, declared } => {
                write!(f, "opcode entry {key:?} declares identity {declared:?}")
            }
            Self::DuplicateOpcodeCode(code) => {
                write!(f, "two opcodes claim target byte {code:#04x}")
            }
            Self::UnsupportedOpcodeExecutionDomain(id) => {
                write!(f, "opcode {id:?} declares no usable execution domain")
            }
            Self::InvalidOpcodeStackContract(id) => {
                write!(f, "opcode {id:?} has an incoherent stack contract")
            }
            Self::IncompleteSuccessContract(id) => {
                write!(f, "opcode {id:?} offers no successful form")
            }
            Self::DuplicateSuccessCase { opcode, case } => {
                write!(f, "opcode {opcode:?} states condition {case:?} twice")
            }
            Self::ContradictorySuccessCase(id) => {
                write!(f, "opcode {id:?} consumes operands it does not declare")
            }
            Self::InvalidRetainedOperandContract(id) => {
                write!(f, "opcode {id:?} retains operands it does not declare")
            }
            Self::UndeclaredCarriedOperand(id) => {
                write!(
                    f,
                    "opcode {id:?} carries through an operand it does not declare"
                )
            }
            Self::MissingOpcodeFailureContract(id) => {
                write!(f, "opcode {id:?} declares no failure behavior")
            }
            Self::ContradictoryFailureCause { opcode, cause } => {
                write!(
                    f,
                    "opcode {opcode:?} gives failure cause {cause:?} two different effects"
                )
            }
            Self::MissingOpcodeResourceCost(id) => {
                write!(f, "opcode {id:?} declares no resource cost")
            }
            Self::MissingOpcodeEvidence(id) => {
                write!(f, "opcode {id:?} names no evidence requirement")
            }
            Self::MissingEncodingSpec(class) => {
                write!(f, "encoding {class:?} has no specification")
            }
            Self::EncodingClassMismatch { key, declared } => {
                write!(f, "encoding entry {key:?} declares identity {declared:?}")
            }
            Self::DuplicateEncodingPrefix { class, prefix } => {
                write!(
                    f,
                    "encoding {class:?} reclaims prefix {prefix:#04x} within its field group"
                )
            }
            Self::InvalidEncodingWidth(class) => {
                write!(f, "encoding {class:?} has an incoherent width")
            }
            Self::MissingByteOrder(class) => {
                write!(f, "numeric encoding {class:?} states no byte order")
            }
            Self::SpuriousByteOrder(class) => {
                write!(f, "opaque encoding {class:?} states a byte order")
            }
            Self::EncodingByteOrderMismatch(class) => {
                write!(f, "encoding {class:?} states the wrong byte order")
            }
            Self::EncodingDomainMismatch(class) => {
                write!(f, "encoding {class:?} states the wrong field group")
            }
            Self::EncodingWidthMismatch(class) => {
                write!(f, "encoding {class:?} states the wrong payload width")
            }
            Self::EncodingCanonicalityMismatch(class) => {
                write!(f, "encoding {class:?} states the wrong canonicality rule")
            }
            Self::MissingEncodingEvidence(class) => {
                write!(f, "encoding {class:?} names no evidence requirement")
            }
            Self::ContradictorySighashDimension(dimension) => {
                write!(f, "sighash dimension {dimension:?} is classified twice")
            }
            Self::UnclassifiedSighashDimension(dimension) => {
                write!(f, "sighash dimension {dimension:?} is not classified")
            }
            Self::OverlappingSequenceFields => {
                write!(f, "the sequence flag and value fields share bits")
            }
            Self::MissingTimelockMode => {
                write!(f, "the relative-timelock contract declares no mode")
            }
            Self::UnclassifiedConfidentialCapability(claim) => {
                write!(f, "confidential-value claim {claim:?} is not classified")
            }
            Self::MissingResourceDimension(dimension) => {
                write!(f, "consensus dimension {dimension:?} is absent")
            }
            Self::InvalidResourceContract(dimension) => {
                write!(f, "dimension {dimension:?} carries an impossible bound")
            }
            Self::PolicyLooserThanConsensus(dimension) => {
                write!(f, "policy is looser than consensus on {dimension:?}")
            }
            Self::MissingCapabilityContract(capability) => {
                write!(f, "capability {capability:?} has no contract")
            }
            Self::CapabilityIdMismatch { key, declared } => {
                write!(f, "capability entry {key:?} declares identity {declared:?}")
            }
            Self::UnknownCapabilityPrerequisite(capability) => {
                write!(f, "prerequisite {capability:?} is not declared")
            }
            Self::CapabilityNamesUnknownOpcode { capability, opcode } => {
                write!(
                    f,
                    "capability {capability:?} names unknown opcode {opcode:?}"
                )
            }
            Self::CapabilityNamesUnknownEncoding {
                capability,
                encoding,
            } => {
                write!(
                    f,
                    "capability {capability:?} names unknown encoding {encoding:?}"
                )
            }
            Self::MissingCapabilityEvidence(capability) => {
                write!(f, "capability {capability:?} names no evidence requirement")
            }
            Self::CapabilityStatusExceedsPrerequisite {
                capability,
                prerequisite,
            } => {
                write!(
                    f,
                    "capability {capability:?} claims more than prerequisite {prerequisite:?}"
                )
            }
            Self::CapabilityDependencyCycle { members } => {
                write!(f, "capabilities require each other in a cycle: {members:?}")
            }
            Self::UnknownEvidenceRequirement(id) => {
                write!(f, "evidence requirement {id:?} is not declared")
            }
            Self::EvidenceRequirementIdMismatch { key, declared } => {
                write!(f, "evidence entry {key:?} declares identity {declared:?}")
            }
            Self::MissingStaleCondition(id) => {
                write!(f, "evidence requirement {id:?} never goes stale")
            }
            Self::MissingPushForm(form) => {
                write!(f, "reviewed push form {form:?} has no contract")
            }
            Self::InvalidPushForm(form) => {
                write!(f, "push form {form:?} admits no encoding")
            }
            Self::DuplicatePushOpcode(opcode) => {
                write!(f, "two push forms claim target byte {opcode:#04x}")
            }
            Self::MissingPushFormEvidence(form) => {
                write!(f, "push form {form:?} names no evidence")
            }
            Self::MissingPushEnforcement(defect) => {
                write!(f, "push defect {defect:?} states no enforcement")
            }
            Self::PushContractMismatch => {
                write!(f, "the push rules do not agree with the target contract")
            }
            Self::SignatureContractMismatch => {
                write!(f, "the signature views do not describe one behavior")
            }
            Self::TimelockContractMismatch => {
                write!(f, "the timelock views do not describe one behavior")
            }
            Self::ConfidentialContractMismatch => {
                write!(f, "the confidential-value views do not agree")
            }
            Self::IssuanceContractMismatch => {
                write!(f, "the issuance views do not agree")
            }
            Self::ResourceContractMismatch => {
                write!(f, "the resource views do not agree")
            }
            Self::StackGrowthContractMismatch => {
                write!(
                    f,
                    "a declared stack growth does not match the primitive's stack contract"
                )
            }
            Self::EvidenceContractMismatch => {
                write!(f, "a subcontract names no declared evidence")
            }
            Self::ProductionBindingUnsupported => {
                write!(f, "no production deployment binding exists")
            }
            Self::ZeroNetworkId => write!(f, "the network identifier is all zero"),
            Self::ZeroGenesisId => write!(f, "the genesis identifier is all zero"),
            Self::TargetDeploymentVersionMismatch => {
                write!(f, "the contract and the binding name different revisions")
            }
            Self::ActivationLeafVersionMismatch => {
                write!(f, "the declared leaf version is not the contract's")
            }
            Self::InconsistentActivationDeclaration => {
                write!(
                    f,
                    "capabilities are required while the execution domain is expected inactive"
                )
            }
            Self::UnsupportedRequiredCapability(capability) => {
                write!(f, "required capability {capability:?} is unsupported")
            }
            Self::UnknownOverrideDimension(dimension) => {
                write!(f, "override names unbounded dimension {dimension:?}")
            }
            Self::IncompatibleResourceOverride(dimension) => {
                write!(f, "override widens dimension {dimension:?}")
            }
        }
    }
}

impl core::error::Error for TargetError {}
