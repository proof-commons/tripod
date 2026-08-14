//! Mutation tests for the encoding, authorization, confidential,
//! resource, capability, and evidence validators.
//!
//! Same discipline as the primitive mutations: start from the reviewed
//! contract, damage exactly one thing, and require a focused typed
//! rejection.

use std::num::NonZeroUsize;

use crate::authorization::{
    AuthorizationContract, RelativeTimelockContract, SequenceFieldLayout, SighashCapability,
    SighashDimension,
};
use crate::capability::{CapabilityContract, ElementsCapability};
use crate::confidential::{
    ConfidentialCapabilityState, ConfidentialValueCapability, ConfidentialValueContract,
};
use crate::definition::{
    TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::encoding::{
    ByteOrder, CanonicalEncodingRule, EncodingClass, EncodingDomain, EncodingSpec,
    PayloadInterpretation, PayloadWidth,
};
use crate::error::TargetError;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::OpcodeId;
use crate::resource::{
    ConsensusResourceLimits, PolicyResourceLimits, ResourceBound, ResourceContract,
    ResourceDimension,
};

/// The reviewed contract's parts, as a mutable starting point.
fn parts() -> TargetDefinitionParts {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }
}

/// Runs the validator and requires it to reject.
fn reject(parts: TargetDefinitionParts) -> Vec<TargetError> {
    validate_target_definition(TargetDefinition::new(parts))
        .expect_err("the mutation must be rejected")
}

#[test]
fn a_prefix_collision_within_one_field_group_is_rejected() {
    let mut parts = parts();
    let victim = parts.encodings[&EncodingClass::ConfidentialValue].clone();
    // Give the blinded value form the explicit form's prefix. A
    // decoder reading that byte could no longer tell which form it
    // has, and the two forms have different payload widths.
    parts.encodings.insert(
        EncodingClass::ConfidentialValue,
        EncodingSpec::new(
            victim.class(),
            victim.domain(),
            [0x01],
            victim.payload(),
            victim.byte_order(),
            victim.canonicality(),
            victim.evidence().iter().copied(),
        ),
    );

    let errors = reject(parts);
    assert!(errors.iter().any(|error| matches!(
        error,
        TargetError::DuplicateEncodingPrefix { prefix: 0x01, .. }
    )));
}

#[test]
fn the_same_prefix_in_two_field_groups_is_accepted() {
    // The control for the test above. The explicit marker means the
    // same thing in three different field groups, and a contract that
    // rejected that would be rejecting the target.
    let parts = parts();
    let asset = parts.encodings[&EncodingClass::ExplicitAsset].clone();
    let value = parts.encodings[&EncodingClass::ExplicitValue].clone();
    assert_eq!(asset.prefixes(), value.prefixes());
    assert_ne!(asset.domain(), value.domain());
    assert!(validate_target_definition(TargetDefinition::new(parts)).is_ok());
}

/// The numeric encodings, stated independently of the registry.
const NUMERIC_CLASSES: &[EncodingClass] = &[
    EncodingClass::ExplicitValue,
    EncodingClass::OutPointIndex,
    EncodingClass::Sequence,
    EncodingClass::ScriptNumber,
    EncodingClass::SignedLittleEndian64,
    EncodingClass::UnsignedLittleEndian32,
    EncodingClass::UnsignedLittleEndian64,
];

/// Restates one encoding with a replacement byte order.
fn with_byte_order(
    parts: &mut TargetDefinitionParts,
    class: EncodingClass,
    byte_order: Option<ByteOrder>,
) {
    let victim = parts.encodings[&class].clone();
    parts.encodings.insert(
        class,
        EncodingSpec::new(
            victim.class(),
            victim.domain(),
            victim.prefixes().iter().copied(),
            victim.payload(),
            byte_order,
            victim.canonicality(),
            victim.evidence().iter().copied(),
        ),
    );
}

#[test]
fn numericity_comes_from_the_class_and_not_from_the_offered_order() {
    // The defect this replaces: numericity used to be *defined* as
    // "an order was supplied", so stripping the order silently turned
    // a signed sixty-four bit integer into an opaque blob and the
    // validator's two byte-order branches could never fire.
    let mut parts = parts();
    with_byte_order(&mut parts, EncodingClass::SignedLittleEndian64, None);

    let rebuilt = parts.encodings[&EncodingClass::SignedLittleEndian64].clone();
    assert!(
        rebuilt.is_numeric(),
        "a stripped order does not change the field"
    );
    assert_eq!(rebuilt.byte_order(), None);
    assert_eq!(
        rebuilt.interpretation(),
        PayloadInterpretation::SignedInteger
    );
}

#[test]
fn every_numeric_encoding_stripped_of_its_order_is_rejected() {
    for class in NUMERIC_CLASSES {
        let mut parts = parts();
        with_byte_order(&mut parts, *class, None);
        assert!(
            reject(parts).contains(&TargetError::MissingByteOrder(*class)),
            "{class:?} carries a number and needs an order"
        );
    }
}

#[test]
fn every_numeric_encoding_with_a_reversed_order_is_rejected() {
    for class in NUMERIC_CLASSES {
        let mut parts = parts();
        with_byte_order(&mut parts, *class, Some(ByteOrder::BigEndian));
        assert!(
            reject(parts).contains(&TargetError::EncodingByteOrderMismatch(*class)),
            "{class:?} is little-endian on the stack"
        );
    }
}

#[test]
fn an_opaque_encoding_with_a_spurious_order_is_rejected() {
    // The other direction, and the one the old design could not
    // express at all: assigning an order to a commitment used to make
    // it numeric by definition and therefore also consistent.
    for class in [
        EncodingClass::ConfidentialValue,
        EncodingClass::SchnorrSignature,
        EncodingClass::OutPointTxid,
    ] {
        let mut parts = parts();
        with_byte_order(&mut parts, class, Some(ByteOrder::LittleEndian));
        assert!(
            reject(parts).contains(&TargetError::SpuriousByteOrder(class)),
            "{class:?} is a byte string"
        );
    }
}

#[test]
fn a_v1_encoding_stating_the_wrong_shape_is_rejected() {
    // Under V1 the width, field group, and canonicality of each class
    // are fixed by the contract revision rather than offered by the
    // caller.
    let mut parts = parts();
    let victim = parts.encodings[&EncodingClass::OutPointIndex].clone();
    parts.encodings.insert(
        EncodingClass::OutPointIndex,
        EncodingSpec::new(
            victim.class(),
            EncodingDomain::Number,
            victim.prefixes().iter().copied(),
            PayloadWidth::Exact(NonZeroUsize::new(8).expect("8 is not zero")),
            victim.byte_order(),
            CanonicalEncodingRule::Minimal,
            victim.evidence().iter().copied(),
        ),
    );

    let errors = reject(parts);
    assert!(errors.contains(&TargetError::EncodingDomainMismatch(
        EncodingClass::OutPointIndex
    )));
    assert!(errors.contains(&TargetError::EncodingWidthMismatch(
        EncodingClass::OutPointIndex
    )));
    assert!(errors.contains(&TargetError::EncodingCanonicalityMismatch(
        EncodingClass::OutPointIndex
    )));
}

#[test]
fn the_reviewed_registry_matches_the_declared_v1_shapes() {
    // Two independent statements of the same facts have to agree. If
    // they ever stop agreeing, one of them is a transcription mistake
    // and the build says so rather than a consumer finding out.
    let parts = parts();
    for class in EncodingClass::ALL {
        let spec = &parts.encodings[class];
        let shape = class.v1_shape();
        assert_eq!(spec.domain(), shape.domain(), "{class:?}");
        assert_eq!(spec.payload(), shape.payload(), "{class:?}");
        assert_eq!(spec.canonicality(), shape.canonicality(), "{class:?}");
        assert_eq!(spec.byte_order(), shape.byte_order(), "{class:?}");
        assert_eq!(
            spec.byte_order().is_some(),
            class.interpretation().is_numeric(),
            "{class:?}"
        );
    }
}

#[test]
fn a_malformed_encoding_width_is_rejected() {
    let mut parts = parts();
    let victim = parts.encodings[&EncodingClass::Sha256Context].clone();
    parts.encodings.insert(
        EncodingClass::Sha256Context,
        EncodingSpec::new(
            victim.class(),
            victim.domain(),
            victim.prefixes().iter().copied(),
            PayloadWidth::Bounded {
                minimum: 200,
                maximum: NonZeroUsize::new(103).expect("103 is not zero"),
            },
            victim.byte_order(),
            victim.canonicality(),
            victim.evidence().iter().copied(),
        ),
    );

    assert!(reject(parts).contains(&TargetError::InvalidEncodingWidth(
        EncodingClass::Sha256Context
    )));
}

#[test]
fn a_missing_encoding_specification_is_rejected() {
    let mut parts = parts();
    parts.encodings.remove(&EncodingClass::ExplicitValue);

    assert!(reject(parts).contains(&TargetError::MissingEncodingSpec(
        EncodingClass::ExplicitValue
    )));
}

#[test]
fn an_unknown_capability_prerequisite_is_rejected() {
    let mut parts = parts();
    parts
        .capabilities
        .remove(&ElementsCapability::EcScalarVerification);

    let errors = reject(parts);
    assert!(errors.contains(&TargetError::MissingCapabilityContract(
        ElementsCapability::EcScalarVerification
    )));
    // The opening capability required it, so its prerequisite no
    // longer resolves either.
    assert!(errors.contains(&TargetError::UnknownCapabilityPrerequisite(
        ElementsCapability::EcScalarVerification
    )));
}

#[test]
fn a_capability_dependency_cycle_is_rejected() {
    let mut parts = parts();
    let leaf = parts.capabilities[&ElementsCapability::RequiredLeafVersion].clone();
    parts.capabilities.insert(
        ElementsCapability::RequiredLeafVersion,
        CapabilityContract::new(
            leaf.capability(),
            [ElementsCapability::TapscriptExecution],
            leaf.opcodes().iter().copied(),
            leaf.encodings().iter().copied(),
            leaf.evidence().iter().copied(),
            leaf.status(),
        ),
    );

    let errors = reject(parts);
    assert!(errors.iter().any(|error| matches!(
        error,
        TargetError::CapabilityDependencyCycle { members }
            if members.contains(&ElementsCapability::RequiredLeafVersion)
                && members.contains(&ElementsCapability::TapscriptExecution)
    )));
}

#[test]
fn a_capability_naming_no_evidence_is_rejected() {
    let mut parts = parts();
    let victim = parts.capabilities[&ElementsCapability::StreamingSha256].clone();
    parts.capabilities.insert(
        ElementsCapability::StreamingSha256,
        CapabilityContract::new(
            victim.capability(),
            victim.prerequisites().iter().copied(),
            victim.opcodes().iter().copied(),
            victim.encodings().iter().copied(),
            [],
            victim.status(),
        ),
    );

    assert!(
        reject(parts).contains(&TargetError::MissingCapabilityEvidence(
            ElementsCapability::StreamingSha256
        ))
    );
}

#[test]
fn a_capability_naming_an_undeclared_opcode_is_rejected() {
    let mut parts = parts();
    parts.opcodes.remove(&OpcodeId::Sha256Update);

    assert!(
        reject(parts).contains(&TargetError::CapabilityNamesUnknownOpcode {
            capability: ElementsCapability::StreamingSha256,
            opcode: OpcodeId::Sha256Update,
        })
    );
}

#[test]
fn a_zero_resource_bound_is_rejected() {
    let mut parts = parts();
    let consensus = parts.resources.consensus();
    let mut bounds = consensus.bounds().clone();
    bounds.insert(ResourceDimension::PeakStackItems, ResourceBound::Maximum(0));
    parts.resources = ResourceContract::new(
        ConsensusResourceLimits::new(
            bounds,
            consensus.witness_scale_factor(),
            consensus.validation_budget_offset(),
        ),
        parts.resources.policy().clone(),
    );

    assert!(
        reject(parts).contains(&TargetError::InvalidResourceContract(
            ResourceDimension::PeakStackItems
        ))
    );
}

#[test]
fn a_policy_bound_looser_than_consensus_is_rejected() {
    let mut parts = parts();
    parts.resources = ResourceContract::new(
        parts.resources.consensus().clone(),
        PolicyResourceLimits::new([(
            ResourceDimension::TransactionWeight,
            // Above the consensus ceiling. A transaction consensus
            // refuses is not made valid by a permissive local policy,
            // so this describes a state that cannot exist.
            ResourceBound::Maximum(8_000_000),
        )]),
    );

    assert!(
        reject(parts).contains(&TargetError::PolicyLooserThanConsensus(
            ResourceDimension::TransactionWeight
        ))
    );
}

#[test]
fn a_missing_required_consensus_dimension_is_rejected() {
    let mut parts = parts();
    let consensus = parts.resources.consensus();
    let mut bounds = consensus.bounds().clone();
    bounds.remove(&ResourceDimension::ControlPathDepth);
    parts.resources = ResourceContract::new(
        ConsensusResourceLimits::new(
            bounds,
            consensus.witness_scale_factor(),
            consensus.validation_budget_offset(),
        ),
        parts.resources.policy().clone(),
    );

    assert!(
        reject(parts).contains(&TargetError::MissingResourceDimension(
            ResourceDimension::ControlPathDepth
        ))
    );
}

#[test]
fn a_missing_evidence_requirement_is_rejected() {
    let mut parts = parts();
    parts
        .evidence_requirements
        .remove(&TargetEvidenceRequirementId::ArithmeticSemantics);

    assert!(
        reject(parts).contains(&TargetError::UnknownEvidenceRequirement(
            TargetEvidenceRequirementId::ArithmeticSemantics
        ))
    );
}

#[test]
fn an_unclassified_sighash_dimension_is_rejected() {
    let mut parts = parts();
    // Drop one dimension from the unreviewed set without adding it to
    // the reviewed one. The contract would then be silent about
    // whether the review reached it, which is worse than either
    // answer, because silence reads as absence.
    let remaining: Vec<SighashDimension> = SighashDimension::ALL
        .iter()
        .copied()
        .filter(|dimension| *dimension != SighashDimension::Issuance)
        .collect();
    parts.authorization = AuthorizationContract::new(
        parts.authorization.signature().clone(),
        SighashCapability::new(
            [],
            remaining,
            parts.authorization.sighash().evidence().iter().copied(),
        ),
        parts.authorization.relative_timelock().clone(),
    );

    assert!(
        reject(parts).contains(&TargetError::UnclassifiedSighashDimension(
            SighashDimension::Issuance
        ))
    );
}

#[test]
fn a_sighash_dimension_classified_both_ways_is_rejected() {
    let mut parts = parts();
    parts.authorization = AuthorizationContract::new(
        parts.authorization.signature().clone(),
        SighashCapability::new(
            [SighashDimension::AllOutputs],
            SighashDimension::ALL.iter().copied(),
            parts.authorization.sighash().evidence().iter().copied(),
        ),
        parts.authorization.relative_timelock().clone(),
    );

    assert!(
        reject(parts).contains(&TargetError::ContradictorySighashDimension(
            SighashDimension::AllOutputs
        ))
    );
}

#[test]
fn overlapping_sequence_fields_are_rejected() {
    let mut parts = parts();
    let timelock = parts.authorization.relative_timelock();
    parts.authorization = AuthorizationContract::new(
        parts.authorization.signature().clone(),
        parts.authorization.sighash().clone(),
        RelativeTimelockContract::new(
            timelock.modes().iter().copied(),
            SequenceFieldLayout {
                // Widen the value mask over the mode flag. A decoder
                // could no longer separate the lock's magnitude from
                // its mode.
                value_mask: 0x007f_ffff,
                ..timelock.layout()
            },
            timelock.minimum_transaction_version(),
            timelock.unsatisfied(),
            timelock.evidence().iter().copied(),
        ),
    );

    assert!(reject(parts).contains(&TargetError::OverlappingSequenceFields));
}

#[test]
fn an_unclassified_confidential_claim_is_rejected() {
    let mut parts = parts();
    let contract = &parts.confidential_values;
    let reduced: Vec<(ConfidentialValueCapability, ConfidentialCapabilityState)> = contract
        .states()
        .iter()
        .filter(|(claim, _)| **claim != ConfidentialValueCapability::AuthenticatedOpening)
        .map(|(claim, state)| (*claim, *state))
        .collect();
    parts.confidential_values = ConfidentialValueContract::new(
        reduced,
        contract.participating_encodings().iter().copied(),
        contract.evidence().iter().copied(),
    );

    assert!(
        reject(parts).contains(&TargetError::UnclassifiedConfidentialCapability(
            ConfidentialValueCapability::AuthenticatedOpening
        ))
    );
}
