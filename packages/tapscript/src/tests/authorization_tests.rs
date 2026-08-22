//! The owner sighash profile and the owner-key encoding closure
//! (Guide-13 §1.7, §1.8, §9.2).
//!
//! Every expectation is written from the guide rather than folded out
//! of the module under test. The profile's required set is spelled out
//! as a literal, the refusals are named with the ground each is refused
//! on, and the coverage map is checked for the property §1.7 actually
//! asks for — that no protected datum lands on a dimension the profile
//! refuses — rather than for agreement with itself.
//!
//! The assessment is checked in both directions. Against the reviewed
//! contract it must report the review incomplete, because the review
//! did not reach the sighash construction. Against a fixture whose
//! sighash capability has been rewritten to review the required
//! dimensions it must report the profile established, which is what
//! shows the disposition is derived from the contract rather than
//! hard-coded to today's answer.

use std::collections::BTreeSet;

use target_elements::{
    AuthorizationContract, EncodingClass, SighashCapability, SighashDimension,
    TargetEvidenceRequirementId, UnknownPublicKeyTypeRule,
};

use super::reviewed_target;
use crate::authorization::{
    DimensionRefusal, DimensionRole, OwnerKeyNegative, OwnerKeyObligation, OwnerProfileDisposition,
    owner_key_encoding_closure, profile_classifies_every_offered_dimension, selected_owner_profile,
};

/// The reviewed authorization contract, unmodified.
fn contract() -> AuthorizationContract {
    reviewed_target()
        .into_validated()
        .definition()
        .authorization()
        .clone()
}

/// A sighash capability that reviews exactly one stated set.
///
/// Built through the target package's own public constructor, the way
/// an external consumer would, so nothing here reaches inside
/// `target-elements` to make a fixture convenient.
fn capability_reviewing(reviewed: &[SighashDimension]) -> SighashCapability {
    let reviewed = reviewed.iter().copied().collect::<BTreeSet<_>>();
    let unreviewed = SighashDimension::ALL
        .iter()
        .copied()
        .filter(|dimension| !reviewed.contains(dimension))
        .collect::<Vec<_>>();

    SighashCapability::new(
        reviewed,
        unreviewed,
        [TargetEvidenceRequirementId::SighashSemantics],
    )
}

// --- §1.7: the profile is all inputs and all outputs ---

#[test]
fn the_selected_profile_requires_exactly_the_stated_dimensions() {
    // An independently written expectation. §9.2's list of what the
    // profile must establish, mapped onto the dimensions this target
    // names, with the two narrowing dimensions absent.
    let expected = [
        SighashDimension::AllOutputs,
        SighashDimension::AllInputs,
        SighashDimension::Issuance,
        SighashDimension::Version,
        SighashDimension::LockTime,
        SighashDimension::TapleafHash,
        SighashDimension::InternalKey,
        SighashDimension::SpentOutputs,
    ];

    assert_eq!(
        selected_owner_profile().required().collect::<Vec<_>>(),
        expected,
    );
}

#[test]
fn the_profile_refuses_each_narrowing_dimension_on_a_stated_ground() {
    let profile = selected_owner_profile();

    assert_eq!(
        profile.refused().collect::<Vec<_>>(),
        [
            (
                SighashDimension::SingleOutput,
                DimensionRefusal::LeavesOtherOutputsFree,
            ),
            (
                SighashDimension::InputExtensionPermitted,
                DimensionRefusal::LeavesInputSetOpen,
            ),
        ],
    );
}

#[test]
fn the_profile_classifies_every_dimension_the_target_offers() {
    let profile = selected_owner_profile();

    // The compile-time signal a same-crate census would give is not
    // available here: the target's dimension census is non-exhaustive,
    // so a dimension added to it reaches the profile's default arm.
    // This is the loud signal instead, and it must be checked as well
    // as offered, or the fail-closed default would quietly become the
    // profile's real answer for the new dimension.
    assert!(profile_classifies_every_offered_dimension(&profile));

    for dimension in SighashDimension::ALL {
        assert!(
            profile.role(*dimension).is_some(),
            "{dimension:?} has no role",
        );
    }

    assert_eq!(
        profile.required().count() + profile.refused().count(),
        SighashDimension::ALL.len(),
        "every offered dimension is classified exactly once",
    );
}

// --- §1.7: every protected datum lands on a required dimension ---

#[test]
fn no_protected_datum_rests_on_a_dimension_the_profile_refuses() {
    let profile = selected_owner_profile();
    let mut seen = 0_usize;

    for (datum, dimension) in profile.coverage() {
        seen += 1;

        assert_eq!(
            profile.role(dimension),
            Some(DimensionRole::Required),
            "{datum:?} rests on {dimension:?}",
        );
        assert_eq!(profile.carrier(datum), dimension);
    }

    // The map is total over the protocol's own list, so a datum the
    // profile forgot fails here rather than passing vacuously.
    assert_eq!(seen, crate::authorization::ProtectedDatum::ALL.len());
}

#[test]
fn dropping_either_total_commitment_would_strand_most_of_the_protected_data() {
    // The selection argument, made checkable. §1.7 permits a narrower
    // profile only where it is separately proved to preserve every
    // required commitment; this is the measurement of how much such a
    // proof would owe, and it is most of the list rather than an edge
    // case.
    let profile = selected_owner_profile();
    let stranded = |dimension| {
        profile
            .coverage()
            .filter(move |(_, carrier)| *carrier == dimension)
            .count()
    };

    assert!(stranded(SighashDimension::AllOutputs) >= 7);
    assert!(stranded(SighashDimension::AllInputs) >= 2);
    assert!(
        stranded(SighashDimension::AllOutputs) + stranded(SighashDimension::AllInputs)
            > crate::authorization::ProtectedDatum::ALL.len() / 2,
    );
}

// --- §9.2: the review has not reached the profile ---

#[test]
fn the_reviewed_contract_leaves_every_required_dimension_unreviewed() {
    let profile = selected_owner_profile();
    let disposition = profile.assess(contract().sighash());

    let OwnerProfileDisposition::ReviewIncomplete { unreviewed } = disposition else {
        panic!("the review reached the sighash construction");
    };

    assert_eq!(
        unreviewed,
        profile.required().collect::<BTreeSet<_>>(),
        "a selection is not an availability claim",
    );
}

#[test]
fn a_contract_that_reviewed_the_required_dimensions_establishes_the_profile() {
    let profile = selected_owner_profile();
    let required = profile.required().collect::<Vec<_>>();

    assert_eq!(
        profile.assess(&capability_reviewing(&required)),
        OwnerProfileDisposition::Established,
    );

    // Reviewing everything *except* one required dimension is the
    // discriminating case: a disposition folded out of the profile
    // rather than the contract would still say established here.
    for dimension in &required {
        let narrower = required
            .iter()
            .copied()
            .filter(|member| member != dimension)
            .collect::<Vec<_>>();

        assert_eq!(
            profile.assess(&capability_reviewing(&narrower)),
            OwnerProfileDisposition::ReviewIncomplete {
                unreviewed: BTreeSet::from([*dimension]),
            },
            "{dimension:?}",
        );
    }
}

#[test]
fn reviewing_a_refused_dimension_does_not_establish_the_profile() {
    // A target that reviewed only the narrowing dimensions has
    // established nothing this profile relies on, and the assessment
    // must not be satisfied by an unrelated review.
    let profile = selected_owner_profile();
    let refused = profile
        .refused()
        .map(|(dimension, _)| dimension)
        .collect::<Vec<_>>();

    assert_eq!(
        profile.assess(&capability_reviewing(&refused)),
        OwnerProfileDisposition::ReviewIncomplete {
            unreviewed: profile.required().collect(),
        },
    );
}

// --- §1.8: the owner-key encoding closure ---

#[test]
fn the_reviewed_target_obliges_an_independent_encoding_check() {
    let closure = owner_key_encoding_closure(&contract());

    // The sharp edge itself: the reviewed primitive reports success
    // without verifying for an unrecognized nonempty key, so the
    // signature result alone is not an authorization.
    assert_eq!(
        closure.unknown_key_rule(),
        UnknownPublicKeyTypeRule::SucceedsWithoutVerification,
    );
    assert_eq!(
        closure.obligation(),
        OwnerKeyObligation::AuthenticateEncodingIndependently,
    );
    assert_eq!(closure.approved(), EncodingClass::XOnlyPublicKey);
}

#[test]
fn the_obligation_follows_the_target_rather_than_this_module() {
    // The non-weakening property in miniature, run the other way: a
    // target that refused unrecognized keys would carry the encoding in
    // its own answer, and the obligation must relax with it rather than
    // stay fixed at today's target's sharpest edge.
    let reviewed = contract();
    let signature = reviewed.signature();
    let refusing = AuthorizationContract::new(
        target_elements::SignaturePrimitiveContract::new(
            signature.public_key_encoding(),
            signature.signature_encoding(),
            signature.empty_signature(),
            signature.invalid_signature(),
            UnknownPublicKeyTypeRule::Rejected,
            signature.budget_per_check(),
            signature.evidence().iter().copied(),
        ),
        reviewed.sighash().clone(),
        reviewed.relative_timelock().clone(),
    );

    assert_eq!(
        owner_key_encoding_closure(&refusing).obligation(),
        OwnerKeyObligation::SignatureResultSuffices,
    );
}

#[test]
fn the_closure_requires_the_same_six_negatives_under_either_obligation() {
    // §1.8's list, written out. What the target's rule changes is which
    // component establishes the first three, never whether a run has to
    // show them failing, so the census is compared against the literal
    // rather than against the obligation.
    let expected = [
        OwnerKeyNegative::EmptyKey,
        OwnerKeyNegative::UnknownNonemptyKeyType,
        OwnerKeyNegative::MalformedApprovedKey,
        OwnerKeyNegative::ApprovedKeyOfAnotherOwner,
        OwnerKeyNegative::ValidSignatureAgainstAnotherKey,
        OwnerKeyNegative::ValidSignatureOverAnotherTransaction,
    ];

    assert_eq!(OwnerKeyNegative::ALL, expected);
    assert_eq!(
        owner_key_encoding_closure(&contract())
            .negatives()
            .collect::<Vec<_>>(),
        expected,
    );
}
