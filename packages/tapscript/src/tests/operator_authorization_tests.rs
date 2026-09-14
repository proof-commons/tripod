//! Literal operator closure and STATE coverage expectations. All offered bytes
//! are disposable public test patterns, with no asserted curve membership.

use std::any::TypeId;
use std::collections::BTreeSet;

use target_elements::{
    EncodingClass, EncodingDomain, ExercisingObservation, ObservationIdentity, ReviewedGround,
    SighashCapability, SighashDimension, SighashSourceCitation, TargetContractVersion,
    TargetEvidenceRequirementId, UnknownPublicKeyTypeRule, UnreviewedGround,
};

use super::reviewed_target;
use crate::{
    DimensionRefusal, DimensionRole, EstablishedOperatorProfile, OperatorKey,
    OperatorKeyCurveValidity, OperatorKeyEncodingClosure, OperatorKeyFaultClass,
    OperatorKeyNegative, OperatorKeyObligation, OperatorKeyRejection, OperatorProfileDisposition,
    OutsideMessageGround, OwnerKeyEncodingClosure, StateProtectedDatum,
    operator_key_encoding_closure, operator_profile_classifies_every_offered_dimension,
    operator_profile_coverage_lands_only_on_required_dimensions, owner_key_encoding_closure,
    selected_operator_profile, selected_owner_profile,
};

fn closure() -> OperatorKeyEncodingClosure {
    operator_key_encoding_closure(reviewed_target().definition().authorization())
}

const REQUIRED: [SighashDimension; 6] = [
    SighashDimension::AllOutputs,
    SighashDimension::AllInputs,
    SighashDimension::Version,
    SighashDimension::LockTime,
    SighashDimension::TapleafHash,
    SighashDimension::SpentOutputs,
];

// Like the owner fixture, use the target's public constructor and explicit
// fixture grounds. A modified capability cannot become a reviewed definition.
fn capability_without(missing: SighashDimension) -> SighashCapability {
    const CITATION: SighashSourceCitation =
        SighashSourceCitation::new("fixture", "fixture", "fixture");
    let reviewed = REQUIRED
        .into_iter()
        .filter(|dimension| *dimension != missing)
        .map(|dimension| {
            (
                dimension,
                ReviewedGround::new(
                    CITATION,
                    ExercisingObservation::new(
                        ObservationIdentity::new("fixture", "fixture", "fixture"),
                        &[],
                    ),
                ),
            )
        });
    let unreviewed = SighashDimension::ALL
        .iter()
        .copied()
        .filter(|dimension| *dimension == missing || !REQUIRED.contains(dimension))
        .map(|dimension| {
            (
                dimension,
                UnreviewedGround::NoCandidateThisArcBuildsCarriesTheSubject(CITATION),
            )
        });
    SighashCapability::new(
        reviewed,
        unreviewed,
        [TargetEvidenceRequirementId::SighashSemantics],
    )
}

#[test]
fn seven_negatives_have_literal_order_and_decision_boundaries() {
    use OperatorKeyFaultClass as Fault;
    use OperatorKeyNegative as Negative;
    let expected = [
        (Negative::EmptyKey, Fault::KeyShapeGate),
        (Negative::WrongWidth, Fault::KeyShapeGate),
        (
            Negative::MalformedApprovedKey,
            Fault::CurveValidityAtTransactionBoundaryAndNativeEvidence,
        ),
        (Negative::UnknownNonemptyKeyType, Fault::KeyShapeGate),
        (
            Negative::ApprovedKeyOfAnotherOperator,
            Fault::CommittedDeploymentKey,
        ),
        (
            Negative::ValidSignatureAgainstAnotherKey,
            Fault::SigningBoundaryAndNativeEvidence,
        ),
        (
            Negative::ValidSignatureOverAnotherTransaction,
            Fault::SigningBoundaryAndNativeEvidence,
        ),
    ];
    assert_eq!(OperatorKeyNegative::ALL.len(), 7);
    assert_eq!(
        closure()
            .negatives()
            .map(|negative| (negative, negative.decided_by()))
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn closure_derives_the_reviewed_encoding_and_fail_closed_obligation() {
    let closure = closure();
    assert_eq!(closure.approved(), EncodingClass::XOnlyPublicKey);
    assert_eq!(
        closure.unknown_key_rule(),
        UnknownPublicKeyTypeRule::SucceedsWithoutVerification
    );
    assert_eq!(
        closure.obligation(),
        OperatorKeyObligation::AuthenticateEncodingIndependently
    );
}

#[test]
fn owner_and_operator_closures_are_distinct_with_the_same_target_encoding() {
    let owner = owner_key_encoding_closure(reviewed_target().definition().authorization());
    assert_eq!(owner.approved(), closure().approved());
    assert_ne!(
        TypeId::of::<OwnerKeyEncodingClosure>(),
        TypeId::of::<OperatorKeyEncodingClosure>()
    );
}

#[test]
fn approved_32_byte_shape_retains_unverified_curve_membership() {
    let key = OperatorKey::new(&closure(), EncodingClass::XOnlyPublicKey, vec![0x42; 32])
        .expect("approved shape");
    assert_eq!(key.encoding(), EncodingClass::XOnlyPublicKey);
    assert_eq!(key.bytes(), &[0x42; 32]);
    assert_eq!(key.curve_validity(), OperatorKeyCurveValidity::Unverified);
}

#[test]
fn empty_offering_is_omitted_before_domain_or_width() {
    for encoding in [EncodingClass::XOnlyPublicKey, EncodingClass::EcScalar] {
        assert_eq!(
            OperatorKey::new(&closure(), encoding, vec![]),
            Err(OperatorKeyRejection::OperatorOmitted)
        );
    }
}

#[test]
fn a_31_byte_key_has_the_named_width_refusal() {
    assert_eq!(
        OperatorKey::new(&closure(), EncodingClass::XOnlyPublicKey, vec![0x42; 31]),
        Err(OperatorKeyRejection::WrongWidth {
            offered: 31,
            required: 32
        })
    );
}

#[test]
fn a_33_byte_key_has_the_named_width_refusal() {
    assert_eq!(
        OperatorKey::new(&closure(), EncodingClass::XOnlyPublicKey, vec![0x42; 33]),
        Err(OperatorKeyRejection::WrongWidth {
            offered: 33,
            required: 32
        })
    );
}

#[test]
fn scalar_class_is_not_a_public_key_even_at_the_approved_width() {
    assert_eq!(
        OperatorKey::new(&closure(), EncodingClass::EcScalar, vec![0x42; 32]),
        Err(OperatorKeyRejection::NotAKeyEncoding {
            offered: EncodingClass::EcScalar,
            domain: EncodingDomain::Scalar,
        })
    );
}

#[test]
fn compressed_class_is_an_alternate_before_its_canonicality_or_width() {
    assert_eq!(
        OperatorKey::new(
            &closure(),
            EncodingClass::CompressedPublicKey,
            vec![0x42; 33]
        ),
        Err(OperatorKeyRejection::AlternateEncodingOfApprovedKey {
            offered: EncodingClass::CompressedPublicKey,
            approved: EncodingClass::XOnlyPublicKey,
        })
    );
}

#[test]
fn profile_requires_the_six_reviewed_dimensions_and_classifies_the_whole_census() {
    let profile = selected_operator_profile();
    assert_eq!(profile.required().collect::<Vec<_>>(), REQUIRED);
    assert!(operator_profile_classifies_every_offered_dimension(
        &profile
    ));
    for dimension in SighashDimension::ALL {
        assert!(profile.role(*dimension).is_some());
        assert_eq!(
            profile.role(*dimension),
            selected_owner_profile().role(*dimension)
        );
    }
}

#[test]
fn narrowing_and_issuance_have_named_refusals() {
    assert_eq!(
        selected_operator_profile().refused().collect::<Vec<_>>(),
        [
            (
                SighashDimension::SingleOutput,
                DimensionRefusal::LeavesOtherOutputsFree
            ),
            (
                SighashDimension::InputExtensionPermitted,
                DimensionRefusal::LeavesInputSetOpen
            ),
            (
                SighashDimension::Issuance,
                DimensionRefusal::SubjectRefusedByTheCensusAndTheDecoder
            ),
        ]
    );
}

#[test]
fn internal_key_composes_outside_the_message_through_spent_outputs() {
    let profile = selected_operator_profile();
    assert_eq!(
        profile.not_carried_by_the_message().collect::<Vec<_>>(),
        [(
            SighashDimension::InternalKey,
            SighashDimension::SpentOutputs,
            OutsideMessageGround::ComposedThroughTheControlBlockCheck,
        )]
    );
    assert_eq!(
        profile.role(SighashDimension::SpentOutputs),
        Some(DimensionRole::Required)
    );
}

#[test]
fn state_census_and_each_carrier_match_the_protocol_items() {
    use SighashDimension as Dimension;
    use StateProtectedDatum as Datum;
    let expected = [
        (Datum::InputFields, Dimension::AllInputs),
        (Datum::SpentOutputFields, Dimension::SpentOutputs),
        (Datum::EveryOutput, Dimension::AllOutputs),
        (Datum::SuccessorMetadataPublication, Dimension::AllOutputs),
        (Datum::SuccessorOutputProgram, Dimension::AllOutputs),
        (Datum::TransactionVersion, Dimension::Version),
        (Datum::LockTime, Dimension::LockTime),
        (Datum::RelevantOutputWitnesses, Dimension::AllOutputs),
        (Datum::IssuanceAbsence, Dimension::AllInputs),
        (
            Datum::ExecutingLeafAndScriptPathTerms,
            Dimension::TapleafHash,
        ),
    ];
    let profile = selected_operator_profile();
    assert_eq!(StateProtectedDatum::ALL.len(), 10);
    assert_eq!(StateProtectedDatum::ALL, expected.map(|(datum, _)| datum));
    assert_eq!(profile.coverage().collect::<Vec<_>>(), expected);
    for (datum, dimension) in expected {
        assert_eq!(profile.carrier(datum), &BTreeSet::from([dimension]));
    }
    assert!(operator_profile_coverage_lands_only_on_required_dimensions(
        &profile
    ));
}

#[test]
fn reviewed_target_establishes_the_profile_and_pins_its_revision() {
    let target = reviewed_target();
    let profile = selected_operator_profile();
    assert_eq!(
        profile.assess(target.definition().authorization().sighash()),
        OperatorProfileDisposition::Established
    );
    let witness =
        EstablishedOperatorProfile::establish(profile.clone(), &target).expect("reviewed profile");
    assert_eq!(witness.profile(), &profile);
    assert_eq!(witness.established_dimensions(), &BTreeSet::from(REQUIRED));
    assert_eq!(witness.capability_revision(), target.definition().version());
}

#[test]
fn losing_each_required_dimension_refuses_establishment_by_name() {
    // The same non-public assessment core is used by public establishment.
    // The reviewed wrapper cannot be fabricated from this degraded capability.
    for missing in REQUIRED {
        let capability = capability_without(missing);
        let expected = OperatorProfileDisposition::ReviewIncomplete {
            unreviewed: BTreeSet::from([missing]),
        };
        assert_eq!(selected_operator_profile().assess(&capability), expected);
        assert_eq!(
            EstablishedOperatorProfile::establish_against(
                selected_operator_profile(),
                &capability,
                TargetContractVersion::V2,
            ),
            Err(expected)
        );
    }
}

#[test]
fn revision_check_accepts_the_pin() {
    let witness =
        EstablishedOperatorProfile::establish(selected_operator_profile(), &reviewed_target())
            .expect("reviewed profile");
    assert_eq!(witness.check_revision(TargetContractVersion::V2), Ok(()));
}

#[test]
fn revision_check_refuses_a_different_revision_with_both_values() {
    let witness =
        EstablishedOperatorProfile::establish(selected_operator_profile(), &reviewed_target())
            .expect("reviewed profile");
    assert_eq!(
        witness.check_revision(TargetContractVersion::V1),
        Err(OperatorProfileDisposition::StaleRevision {
            pinned: TargetContractVersion::V2,
            offered: TargetContractVersion::V1,
        })
    );
}
