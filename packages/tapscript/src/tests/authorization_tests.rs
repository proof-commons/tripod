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
//! contract it must now report the profile established, because the
//! review reached every dimension the profile requires. Against
//! fixtures whose sighash capability reviews less than the required set
//! it must report the review incomplete and name what is missing, which
//! is what shows the disposition is derived from the contract rather
//! than hard-coded to today's answer — and that direction is the one
//! worth keeping sharp now that today's answer is the affirmative one.

use std::collections::BTreeSet;

use target_elements::{
    AuthorizationContract, EncodingClass, ExercisingObservation, ObservationIdentity,
    ReviewedGround, SighashCapability, SighashDimension, SighashSourceCitation,
    TargetEvidenceRequirementId, UnknownPublicKeyTypeRule, UnreviewedGround,
};

use super::reviewed_target;
use crate::authorization::{
    DimensionRefusal, DimensionRole, OutsideMessageGround, OwnerKeyNegative, OwnerKeyObligation,
    OwnerProfileDisposition, ProtectedDatum, owner_key_encoding_closure,
    profile_classifies_every_offered_dimension, profile_coverage_lands_only_on_required_dimensions,
    selected_owner_profile,
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
///
/// The grounds are fixture material and say so in their own text. What
/// this fixture exists to vary is *which dimensions* a capability
/// establishes, so that the assessment can be shown to follow the
/// contract rather than today's answer; the grounds a real verdict
/// carries are the target package's to state, and its own tests are
/// where they are checked.
fn capability_reviewing(reviewed: &[SighashDimension]) -> SighashCapability {
    const FIXTURE_CITATION: SighashSourceCitation =
        SighashSourceCitation::new("fixture", "fixture", "fixture");

    let reviewed = reviewed.iter().copied().collect::<BTreeSet<_>>();
    let unreviewed = SighashDimension::ALL
        .iter()
        .copied()
        .filter(|dimension| !reviewed.contains(dimension))
        .map(|dimension| {
            (
                dimension,
                UnreviewedGround::NoCandidateThisArcBuildsCarriesTheSubject(FIXTURE_CITATION),
            )
        })
        .collect::<Vec<_>>();
    let reviewed = reviewed
        .into_iter()
        .map(|dimension| {
            (
                dimension,
                ReviewedGround::new(
                    FIXTURE_CITATION,
                    ExercisingObservation::new(
                        ObservationIdentity::new("fixture", "fixture", "fixture"),
                        &[],
                    ),
                ),
            )
        })
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
    // names, with the two narrowing dimensions absent; with the internal
    // key absent too, because the source review found that the message
    // carries no term for it and the owner re-typed it onto the spent
    // outputs; and with the issuance dimension absent, because the owner
    // re-typed that one to refused after the review verdict, on the
    // ground that the census and the decoder both refuse its subject.
    // Six dimensions, not eight, and each of the two that left did so by
    // a ruling recorded at the point of use.
    let expected = [
        SighashDimension::AllOutputs,
        SighashDimension::AllInputs,
        SighashDimension::Version,
        SighashDimension::LockTime,
        SighashDimension::TapleafHash,
        SighashDimension::SpentOutputs,
    ];

    assert_eq!(
        selected_owner_profile().required().collect::<Vec<_>>(),
        expected,
    );
}

// --- The internal key is re-typed, not deleted ---

#[test]
fn the_internal_key_is_not_a_required_dimension() {
    // The re-typing's first half, stated as the absence it is. A
    // dimension no message term carries cannot be established by reading
    // the message or by recomputing it, so requiring it would hold the
    // disposition at review-incomplete forever.
    let profile = selected_owner_profile();

    assert!(
        !profile
            .required()
            .any(|dimension| dimension == SighashDimension::InternalKey),
    );
    assert!(
        !profile
            .refused()
            .any(|(dimension, _)| dimension == SighashDimension::InternalKey),
        "the profile does not decline a protection it in fact has",
    );
}

#[test]
fn the_internal_keys_protection_is_recorded_as_carried_by_the_spent_outputs() {
    // The re-typing's second half. The role is not merely "absent": it
    // names the dimension whose term fixes the value the internal key
    // composes with, so a reader who notices the internal key is not
    // required finds where its protection went.
    let profile = selected_owner_profile();

    assert_eq!(
        profile.role(SighashDimension::InternalKey),
        Some(DimensionRole::NotCarriedByTheMessage {
            carried_by: SighashDimension::SpentOutputs,
            ground: OutsideMessageGround::ComposedThroughTheControlBlockCheck,
        }),
    );

    // And the dimension it was re-typed onto is one the profile
    // requires, so the protection is recorded as carried by something
    // the signature actually commits to rather than parked on a
    // dimension nobody checks.
    assert_eq!(
        profile.role(SighashDimension::SpentOutputs),
        Some(DimensionRole::Required),
    );
}

#[test]
fn the_internal_key_is_the_only_dimension_no_message_term_carries() {
    // The review found exactly one such dimension, and this is the
    // measurement rather than the claim. A second member appearing here
    // is a second dimension somebody re-typed without arguing for it.
    let profile = selected_owner_profile();

    assert_eq!(
        profile.not_carried_by_the_message().collect::<Vec<_>>(),
        [(
            SighashDimension::InternalKey,
            SighashDimension::SpentOutputs,
            OutsideMessageGround::ComposedThroughTheControlBlockCheck,
        )],
    );
}

#[test]
fn re_typing_the_internal_key_moves_no_protected_datum() {
    // The reason the re-typing is free of consequence, checked rather
    // than asserted in prose: the coverage map assigned the internal key
    // no protected datum, so nothing had to be moved off it. A coverage
    // map that had named it would have made this a much larger change.
    let profile = selected_owner_profile();

    assert!(
        !profile
            .coverage()
            .any(|(_, dimension)| dimension == SighashDimension::InternalKey),
    );
}

#[test]
fn the_profile_refuses_three_dimensions_each_on_a_stated_ground() {
    // Two narrowings and one re-typing, and the third is deliberately in
    // the same assertion as the other two: the issuance dimension's
    // re-typing made it a refusal *with content*, on the pattern the two
    // narrowings already set, rather than a dimension that stopped being
    // required and acquired no reason for it.
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
            (
                SighashDimension::Issuance,
                DimensionRefusal::SubjectRefusedByTheCensusAndTheDecoder,
            ),
        ],
    );
}

#[test]
fn re_typing_the_issuance_dimension_moves_one_protected_datum_and_names_where() {
    // The internal key's re-typing was free of consequence because the
    // coverage map named it nowhere. This one is not, and the difference
    // is measured rather than argued: the map did name the issuance
    // dimension, for the issuance fields, so that datum's carrier had to
    // move or the coverage argument would have been landing on a refused
    // dimension.
    //
    // Where it moved to follows from what the terms are — the two
    // issuance terms are one zero byte per input for every candidate
    // this arc builds, so what they commit to is the input count, and
    // the all-inputs dimension already carries that.
    let profile = selected_owner_profile();

    assert!(
        !profile
            .coverage()
            .any(|(_, dimension)| dimension == SighashDimension::Issuance),
        "no protected datum is carried by a refused dimension",
    );

    assert_eq!(
        profile.carrier(ProtectedDatum::IssuanceFields),
        &BTreeSet::from([SighashDimension::AllInputs]),
    );

    // And the invariant that would have caught the omission is checked
    // here too, because a datum parked on a refused dimension is exactly
    // what it exists to refuse.
    assert!(profile_coverage_lands_only_on_required_dimensions(&profile));
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
        profile.required().count()
            + profile.refused().count()
            + profile.not_carried_by_the_message().count(),
        SighashDimension::ALL.len(),
        "every offered dimension is classified exactly once",
    );
}

// --- §1.7: every protected datum lands on a required dimension ---

#[test]
fn no_protected_datum_rests_on_a_dimension_the_profile_refuses() {
    let profile = selected_owner_profile();
    let mut seen = BTreeSet::new();

    for (datum, dimension) in profile.coverage() {
        seen.insert(datum);

        assert_eq!(
            profile.role(dimension),
            Some(DimensionRole::Required),
            "{datum:?} rests on {dimension:?}",
        );
        assert!(
            profile.carrier(datum).contains(&dimension),
            "{datum:?} is covered by {dimension:?} but does not carry it",
        );
    }

    // The map is total over the protocol's own list, so a datum the
    // profile forgot fails here rather than passing vacuously. Counted
    // over distinct data rather than over pairs, because the map is
    // set-valued and the proof fields contribute two pairs.
    assert_eq!(seen.len(), crate::authorization::ProtectedDatum::ALL.len());

    // The same argument as a recomputed value rather than as a loop, so
    // a consumer can ask it without running this test. It adds the one
    // failure the loop above cannot see: a datum whose carrier set went
    // empty contributes no pair at all.
    assert!(profile_coverage_lands_only_on_required_dimensions(&profile));
}

#[test]
fn the_proof_fields_are_the_only_datum_with_more_than_one_carrier() {
    // The set-valued widening, measured. Fourteen data have one carrier
    // and the proof fields have two — the spent outputs' anchoring the
    // map always named, and the created outputs' dimension the
    // confidential-funding guide's repair adds. A third multi-carrier
    // datum appearing here is a coverage change nobody argued for.
    let profile = selected_owner_profile();

    for datum in crate::authorization::ProtectedDatum::ALL {
        let carriers = profile.carrier(*datum);
        let expected = if *datum == crate::authorization::ProtectedDatum::ProofFields {
            2
        } else {
            1
        };

        assert_eq!(
            carriers.len(),
            expected,
            "{datum:?} is carried by {carriers:?}"
        );
    }
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

// --- §9.2: the review verdict, recomputed ---

#[test]
fn the_reviewed_contract_establishes_the_profile_over_a_six_member_required_set() {
    // The successor to the Wave-4 stop test, and it asserts a different
    // fact rather than the same fact flipped. The stop was real and this
    // does not deny it: six of the seven required dimensions were
    // exercised by the observed acceptance and the seventh was not,
    // because no candidate this arc builds bears an issuance and the
    // terms carrying the dimension are formed from the input count
    // alone. What repaired the stop is not a run — nothing has been
    // observed since — but the re-typing of that dimension from required
    // to refused, recorded at the profile-decision section of the owner
    // sighash concept, on the ground that the census and the decoder
    // both refuse the subject the dimension would commit to.
    //
    // So both halves are asserted, and the count is asserted first,
    // because it is the half that would catch a silent flip. An
    // established disposition reached by dropping dimensions out of the
    // required set until none was left unreviewed would satisfy the
    // second assertion and fail this one.
    let profile = selected_owner_profile();

    assert_eq!(
        profile.required().count(),
        6,
        "the required set lost the issuance dimension and nothing else",
    );
    assert_eq!(
        profile.required().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            SighashDimension::AllOutputs,
            SighashDimension::AllInputs,
            SighashDimension::Version,
            SighashDimension::LockTime,
            SighashDimension::TapleafHash,
            SighashDimension::SpentOutputs,
        ]),
    );

    assert_eq!(
        profile.assess(contract().sighash()),
        OwnerProfileDisposition::Established,
    );
}

#[test]
fn every_required_dimension_is_established_and_the_refused_one_is_not() {
    // The other half of the same verdict, asserted from the profile's
    // own required set rather than from a list copied beside it. The
    // second assertion is what keeps the first from being satisfied by a
    // capability that reviewed everything in sight: the re-typed
    // dimension must still be *unreviewed*, because the ruling refused
    // it and did not establish it, and a reviewed set that had grown to
    // include it would be recording evidence no run produced.
    let profile = selected_owner_profile();
    let capability = contract().sighash().clone();

    for dimension in profile.required() {
        assert!(capability.is_reviewed(dimension), "{dimension:?}");
    }

    assert!(!capability.is_reviewed(SighashDimension::Issuance));
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
