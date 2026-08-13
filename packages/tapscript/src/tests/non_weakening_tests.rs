//! Non-weakening and sponsor-opacity tests (Guide-8 §16.4, §20.6,
//! §20.7, Appendix G.3).

use std::collections::BTreeSet;

use compiler::target::RequiredCapability;
use target_elements::{ElementsCapability, StaticCapabilityStatus};

use super::{reviewed_target, target_with_status, target_with_statuses};
use crate::capability::{
    AssessmentDisposition, BackendFoundationRequirement, CapabilityAssessment, UnsupportedReason,
    assess_capability,
};

/// The dispositions in which no obligation has been discharged.
///
/// Every one of the six is such a disposition except the last, which is
/// unreachable; the list exists so a test can say "still blocked or
/// still owed" without enumerating the negation.
const fn is_blocked_or_owed(disposition: AssessmentDisposition) -> bool {
    !matches!(disposition, AssessmentDisposition::CompleteBackendPattern)
}

#[test]
fn removing_a_prerequisite_blocks_and_keeps_the_requirement() {
    // Guide-8 §20.6, one row at a time. The registry census must stay
    // complete for the contract to validate, so the reachable mutation
    // is a prerequisite that stops being established rather than one
    // that stops existing; the assessment must degrade either way.
    let downgrades = [
        (
            RequiredCapability::AuthenticatedObjectRecognition,
            ElementsCapability::OutputProgramInspection,
        ),
        (
            RequiredCapability::AuthenticatedFamilyCardinality,
            ElementsCapability::InputCountInspection,
        ),
        (
            RequiredCapability::AuthenticatedCanonicalPartition,
            ElementsCapability::InputValueInspection,
        ),
        (
            RequiredCapability::AuthenticatedOpenFlowPartition,
            ElementsCapability::OutputCountInspection,
        ),
        (
            RequiredCapability::AuthenticatedRootEffects,
            ElementsCapability::TweakVerification,
        ),
        (
            RequiredCapability::AuthenticatedProjectionSet,
            ElementsCapability::OutputProgramInspection,
        ),
        (
            RequiredCapability::ExactPublicAmountArithmetic,
            ElementsCapability::SignedFixedWidthArithmetic,
        ),
        (
            RequiredCapability::OwnerAuthorization,
            ElementsCapability::SignatureVerification,
        ),
    ];

    for (capability, prerequisite) in downgrades {
        let target = target_with_status(prerequisite, StaticCapabilityStatus::Incomplete);
        let assessment = assess_capability(&target, capability);

        assert_eq!(
            assessment.required(),
            capability,
            "the compiler requirement survives its own blocking",
        );

        let CapabilityAssessment::MissingTargetPrimitives { missing, .. } = &assessment else {
            panic!("{capability:?} must block on {prerequisite:?}, got {assessment:?}");
        };
        assert!(missing.contains(&prerequisite));
    }
}

#[test]
fn a_reviewed_negative_fact_is_not_reported_as_an_incomplete_review() {
    // Unsupported outranks Incomplete. A capability the contract says
    // does not exist must not be reported as one that further review
    // could still establish.
    let target = target_with_status(
        ElementsCapability::OutputAssetInspection,
        StaticCapabilityStatus::Unsupported,
    );
    let assessment = assess_capability(&target, RequiredCapability::AuthenticatedObjectRecognition);

    let CapabilityAssessment::Unsupported {
        required,
        reason: UnsupportedReason::ReviewedTargetPrimitivesUnsupported { primitives },
    } = &assessment
    else {
        panic!("expected a reviewed negative fact, got {assessment:?}");
    };

    assert_eq!(
        *required,
        RequiredCapability::AuthenticatedObjectRecognition
    );
    assert!(primitives.contains(&ElementsCapability::OutputAssetInspection));
}

#[test]
fn no_downgrade_ever_discharges_a_capability() {
    // The general form of §20.6: whatever is taken away from the
    // target, no capability's disposition improves into a completion.
    for primitive in ElementsCapability::ALL {
        // The activation declaration names the execution domain, and a
        // binding that declares reliance on an unsupported capability is
        // refused by the target package before an assessment happens.
        if *primitive == ElementsCapability::TapscriptExecution {
            continue;
        }

        let target = target_with_status(*primitive, StaticCapabilityStatus::Incomplete);

        for capability in RequiredCapability::ALL {
            let assessment = assess_capability(&target, *capability);

            assert_eq!(assessment.required(), *capability);
            assert!(is_blocked_or_owed(assessment.disposition()));
        }
    }
}

#[test]
fn whole_transaction_conservation_stays_external_evidence() {
    // It must not become sponsor positivity, a protocol-local subtotal,
    // or a primitive-availability question — and it must not turn into
    // an object-family claim either.
    let target = reviewed_target();

    for capability in [
        RequiredCapability::WholeTransactionValueConservation,
        RequiredCapability::ConfidentialValueConservation,
    ] {
        let assessment = assess_capability(&target, capability);
        let CapabilityAssessment::ExternalEvidenceRequired { evidence, .. } = &assessment else {
            panic!("{capability:?} is external evidence, got {assessment:?}");
        };
        assert!(!evidence.is_empty());

        let projection = assessment.projection();
        assert!(
            projection.primitives().is_empty(),
            "an external claim is not a question of primitive availability",
        );
        assert!(
            projection.structural().is_empty(),
            "an external claim is not a structural obligation either",
        );
    }
}

#[test]
fn confidential_conservation_does_not_imply_object_recognition() {
    let target = reviewed_target();
    let conservation =
        assess_capability(&target, RequiredCapability::ConfidentialValueConservation).projection();
    let recognition =
        assess_capability(&target, RequiredCapability::AuthenticatedObjectRecognition).projection();

    assert_ne!(conservation.disposition(), recognition.disposition());
    assert!(
        !recognition
            .evidence()
            .contains(&target_elements::TargetEvidenceRequirementId::ConfidentialValueConservation),
        "recognising an object is not conserving value, and neither obligation covers the other",
    );
}

#[test]
fn public_constructibility_stays_structural_and_needs_no_signature() {
    let target = reviewed_target();
    let assessment = assess_capability(&target, RequiredCapability::PublicConstructibility);

    let CapabilityAssessment::BackendStructural {
        primitives,
        requirements,
        ..
    } = &assessment
    else {
        panic!("public constructibility is structural, got {assessment:?}");
    };

    assert!(
        primitives.is_empty(),
        "it is not reducible to an opcode, and specifically not to a signature check",
    );
    assert!(requirements.contains(&BackendFoundationRequirement::SecretFreePermissionlessPath));
    assert!(requirements.contains(&BackendFoundationRequirement::PublicConstructionData));
}

#[test]
fn authorization_needs_both_a_signature_and_a_sighash_obligation() {
    // Against the reviewed target every authorization row blocks on the
    // unreviewed sighash. A hypothetical contract in which the sighash
    // construction had been reviewed is built here so the row's full
    // obligation is visible: it is never the signature check alone.
    let target = target_with_statuses(&[
        (
            ElementsCapability::OutputCommittingSighash,
            StaticCapabilityStatus::Reviewed,
        ),
        (
            ElementsCapability::InputCommitmentControl,
            StaticCapabilityStatus::Reviewed,
        ),
    ]);

    for capability in [
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::OperatorAuthorization,
        RequiredCapability::RefundAuthorization,
    ] {
        let assessment = assess_capability(&target, capability);
        let CapabilityAssessment::BackendPatternRequired {
            primitives,
            evidence,
            ..
        } = &assessment
        else {
            panic!("{capability:?} still owes a pattern, got {assessment:?}");
        };

        assert!(primitives.contains(&ElementsCapability::SignatureVerification));
        assert!(primitives.contains(&ElementsCapability::OutputCommittingSighash));
        assert!(
            evidence.contains(&target_elements::TargetEvidenceRequirementId::SighashSemantics),
            "a signature that does not commit to the right dimensions authorizes the wrong thing",
        );
    }

    // And against the target as actually reviewed, the gap is visible
    // rather than papered over.
    let reviewed = assess_capability(&reviewed_target(), RequiredCapability::OwnerAuthorization);
    assert_eq!(
        reviewed.disposition(),
        AssessmentDisposition::MissingTargetPrimitives,
    );
}

#[test]
fn no_assessment_requires_a_sponsor_amount() {
    // Guide-8 §20.7. The sponsor-adjacent capability is the open-flow
    // partition, and it must reach its answer without reading any
    // amount at all: no sponsor positivity, no exact sponsor input or
    // output amount, no public sponsor subtotal, no sponsor opening.
    // There is no sponsor vocabulary in the target package to test
    // against, which is the point — the only way to reintroduce a
    // sponsor amount would be to require a value primitive here.
    let target = reviewed_target();
    let projection =
        assess_capability(&target, RequiredCapability::AuthenticatedOpenFlowPartition).projection();

    let value_primitives = BTreeSet::from([
        ElementsCapability::InputValueInspection,
        ElementsCapability::OutputValueInspection,
        ElementsCapability::ExplicitValueInspection,
        ElementsCapability::AuthenticatedValueOpening,
        ElementsCapability::CommitmentEquality,
    ]);

    assert!(
        projection
            .primitives()
            .iter()
            .all(|primitive| !value_primitives.contains(primitive)),
        "the open flow is separated from the sponsor region by role, never by amount",
    );
    assert!(
        projection
            .structural()
            .contains(&BackendFoundationRequirement::ProtocolSponsorRegionSeparation),
        "the separation is stated as a structural obligation, so it cannot be forgotten",
    );

    // Protocol-role amounts remain readable where a capability genuinely
    // needs them: erasure is role-based, not a blanket ban on values.
    let partition = assess_capability(&target, RequiredCapability::AuthenticatedCanonicalPartition)
        .projection();
    assert!(
        partition
            .primitives()
            .contains(&ElementsCapability::InputValueInspection),
    );
}
