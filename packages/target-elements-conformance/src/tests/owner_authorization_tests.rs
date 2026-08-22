//! The owner-authorization case census (Guide-13 §1.8, §9.2, §19.3).
//!
//! The three source lists are written out here as literals, separately,
//! and each is required of the census on its own. A merged expectation
//! would pass whenever one list's members happened to sit inside
//! another's, which is the way a merged census loses a list.
//!
//! The profile coupling is checked by narrowing the committed set
//! rather than by reading the census back: a mutation case is only
//! evidence because the signature covers what it changed, so the
//! discriminating test is that a commitment which stopped covering it
//! makes the census fail — once per mutation case, so the guard covers
//! the census rather than one convenient row of it.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::{OwnerKeyNegative, ProtectedDatum, selected_owner_profile};

use crate::owner_authorization::{
    CaseCensusDefect, CaseResidual, OwnerAuthorizationCase, OwnerAuthorizationCaseId,
    OwnerAuthorizationExpectation, case_for_key_negative, committed_protected_data,
    covers_every_required_list, owner_authorization_cases, validate_case_census,
    validated_owner_authorization_cases,
};

fn cases() -> BTreeMap<OwnerAuthorizationCaseId, OwnerAuthorizationCase> {
    validated_owner_authorization_cases().expect("the census validates against the profile")
}

// --- §9.2 and §19.3: every required case is present ---

#[test]
fn the_census_answers_the_eight_target_native_tests() {
    // §9.2's list, written out in the guide's own order.
    let required = [
        OwnerAuthorizationCaseId::ValidOwnerSignature,
        OwnerAuthorizationCaseId::MissingSignature,
        OwnerAuthorizationCaseId::WrongOwner,
        OwnerAuthorizationCaseId::OutputChangedAfterSigning,
        OwnerAuthorizationCaseId::InputAddedAfterSigning,
        OwnerAuthorizationCaseId::OutputRemovedAfterSigning,
        OwnerAuthorizationCaseId::WrongSighashByte,
        OwnerAuthorizationCaseId::SignatureBoundToAnotherTransaction,
    ];
    let census = cases();

    for case in required {
        assert!(census.contains_key(&case), "{case:?}");
    }
}

#[test]
fn the_census_answers_the_eleven_authorization_coverage_rows() {
    // §19.3's list, written out.
    let required = [
        OwnerAuthorizationCaseId::ValidOwnerSignature,
        OwnerAuthorizationCaseId::MissingSignature,
        OwnerAuthorizationCaseId::WrongOwner,
        OwnerAuthorizationCaseId::InvalidSignature,
        OwnerAuthorizationCaseId::EmptySignature,
        OwnerAuthorizationCaseId::UnknownKeyForm,
        OwnerAuthorizationCaseId::OutputChangedAfterSigning,
        OwnerAuthorizationCaseId::InputAddedAfterSigning,
        OwnerAuthorizationCaseId::IncompleteOwnerSet,
        OwnerAuthorizationCaseId::RepeatedOwnerWithOneWitnessOmitted,
        OwnerAuthorizationCaseId::SponsorOwnerOmission,
    ];
    let census = cases();

    for case in required {
        assert!(census.contains_key(&case), "{case:?}");
    }
}

#[test]
fn every_key_negative_has_a_distinct_case() {
    let census = cases();
    let mut answered = BTreeSet::new();

    for negative in OwnerKeyNegative::ALL {
        let case = case_for_key_negative(*negative);

        assert!(census.contains_key(&case), "{negative:?}");
        assert!(
            answered.insert(case),
            "{negative:?} shares {case:?} with another negative",
        );
    }

    // Six negatives, six cases. A mapping that folded two onto one
    // would still satisfy a containment check while leaving one of
    // §1.8's negatives with nothing that exercises it.
    assert_eq!(answered.len(), OwnerKeyNegative::ALL.len());
    assert!(covers_every_required_list(&census));
}

// --- the shape of the census itself ---

#[test]
fn exactly_one_case_is_expected_to_be_accepted() {
    let accepted = cases()
        .into_values()
        .filter(|case| case.expectation() == OwnerAuthorizationExpectation::Accepted)
        .map(|case| case.id())
        .collect::<Vec<_>>();

    // One, not "at least one". A census with a second accepting case
    // would have to say which authorization it establishes, and none of
    // the remaining cases is an authorization.
    assert_eq!(accepted, [OwnerAuthorizationCaseId::ValidOwnerSignature],);
}

#[test]
fn no_case_claims_it_could_run_today() {
    // §1.11: a target-negative claim needs a complete target
    // transaction and an observed verdict. Every case carries at least
    // the unreviewed profile, because the census is stated against a
    // profile the review has not established.
    for case in cases().values() {
        let residuals = case.residuals().collect::<BTreeSet<_>>();

        assert!(
            residuals.contains(&CaseResidual::ProfileUnreviewed),
            "{:?}",
            case.id(),
        );
    }
}

#[test]
fn the_multi_owner_cases_wait_on_the_review_alone_and_the_sponsor_case_on_more() {
    // What §12 moved. Both multi-owner cases named a
    // `MultiOwnerTransaction` residual while no candidate ABI built one,
    // and the ABI now builds both shapes the two cases need: a transfer
    // consuming two owners' receipts, and a transfer consuming two
    // receipts of a single owner whose census reports one semantic owner
    // and two concrete signatures. The residual is gone rather than kept
    // as a discharged marker.
    //
    // The sponsor case is not flipped with them, and the difference is
    // what this test holds: §1.9 keeps the sponsor's own authorization
    // outside protocol data, so the sponsored form being constructible
    // is not the same as its owner being modelled.
    let census = cases();
    let residuals = |id| census[&id].residuals().collect::<BTreeSet<_>>();

    for id in [
        OwnerAuthorizationCaseId::IncompleteOwnerSet,
        OwnerAuthorizationCaseId::RepeatedOwnerWithOneWitnessOmitted,
    ] {
        assert_eq!(
            residuals(id),
            BTreeSet::from([CaseResidual::ProfileUnreviewed]),
            "{id:?}"
        );
    }

    assert!(
        residuals(OwnerAuthorizationCaseId::SponsorOwnerOmission)
            .contains(&CaseResidual::SponsorEnvelope),
    );

    // The valid case needs nothing beyond the review, so a residual
    // census that had become uniform would fail here rather than pass
    // by saying the same thing everywhere.
    assert_eq!(
        residuals(OwnerAuthorizationCaseId::ValidOwnerSignature),
        BTreeSet::from([CaseResidual::ProfileUnreviewed]),
    );
    // And the sponsor case is the only one left carrying more than the
    // review, which is the count a later wave will be closing.
    let beyond_the_review: Vec<_> = census
        .values()
        .filter(|case| case.residuals().count() > 1)
        .map(super::super::owner_authorization::OwnerAuthorizationCase::id)
        .collect();
    assert_eq!(
        beyond_the_review,
        vec![OwnerAuthorizationCaseId::SponsorOwnerOmission],
    );
}

// --- the coupling that makes a mutation case evidence ---

#[test]
fn every_mutation_rests_on_a_datum_the_profile_commits_to() {
    let profile = selected_owner_profile();
    let census = cases();
    let mut mutations = 0_usize;

    let committed = committed_protected_data(&profile);

    for case in census.values() {
        let Some(datum) = case.disturbed() else {
            continue;
        };

        mutations += 1;

        assert!(
            committed.contains(&datum),
            "{:?} disturbs {datum:?}",
            case.id()
        );
    }

    // The output, input, and foreign-transaction mutations. A census
    // whose mutation cases had all lost their datum would satisfy the
    // loop above vacuously.
    assert_eq!(mutations, 4);
}

#[test]
fn the_selected_profile_commits_to_every_protected_datum() {
    // The premise the census rests on, stated separately from it. Under
    // the selected profile every protected datum lands on a required
    // dimension, which is why no mutation case is stranded today — and
    // is also why the guard below has to be shown firing against a
    // narrower set rather than against this one.
    let committed = committed_protected_data(&selected_owner_profile());

    assert_eq!(committed.len(), ProtectedDatum::ALL.len());
}

#[test]
fn a_narrower_commitment_refuses_the_mutation_case_that_rests_on_it() {
    // The discriminating test, and the whole reason the coupling
    // exists. §1.7 permits a narrower profile only where it is
    // separately proved to preserve every required commitment; under
    // one that dropped the destination values, the output-change case
    // is a transaction the target *accepts*, and a census that still
    // called it a refusal would report evidence it did not have.
    //
    // The narrowing is applied to the committed set rather than to the
    // profile because `OwnerSighashProfile` has no public constructor —
    // that is the property being relied on, not one being worked
    // around, and the set is the thing the guard is actually about.
    let census = owner_authorization_cases();
    let mut committed = committed_protected_data(&selected_owner_profile());

    assert!(committed.remove(&ProtectedDatum::DestinationSemanticValues));

    assert_eq!(
        validate_case_census(&census, &committed),
        Err(CaseCensusDefect::UncommittedMutation {
            case: OwnerAuthorizationCaseId::OutputChangedAfterSigning,
            datum: ProtectedDatum::DestinationSemanticValues,
        }),
    );

    // Every mutation case, one at a time: dropping the datum any one of
    // them rests on must be refused, so the guard covers the census
    // rather than one convenient row of it.
    for case in census.values() {
        let Some(datum) = case.disturbed() else {
            continue;
        };

        let mut narrowed = committed_protected_data(&selected_owner_profile());
        assert!(narrowed.remove(&datum));

        assert!(
            matches!(
                validate_case_census(&census, &narrowed),
                Err(CaseCensusDefect::UncommittedMutation { datum: refused, .. })
                    if refused == datum,
            ),
            "{:?}",
            case.id(),
        );
    }
}

#[test]
fn an_incomplete_case_census_is_refused() {
    let committed = committed_protected_data(&selected_owner_profile());
    let mut census = owner_authorization_cases();

    census.remove(&OwnerAuthorizationCaseId::ValidOwnerSignature);

    assert!(matches!(
        validate_case_census(&census, &committed),
        Err(CaseCensusDefect::CensusMismatch { .. }),
    ));
}

#[test]
fn a_census_with_no_accepting_case_is_refused() {
    // A census of refusals establishes that the pattern refuses things,
    // which a pattern that refuses everything also does. The mismatch
    // check runs first, so the accepting case is replaced rather than
    // removed.
    let committed = committed_protected_data(&selected_owner_profile());
    let mut census = owner_authorization_cases();
    let refusing =
        OwnerAuthorizationCase::clone(&census[&OwnerAuthorizationCaseId::MissingSignature]);

    census.insert(OwnerAuthorizationCaseId::ValidOwnerSignature, refusing);

    // The replaced entry keeps the key it was inserted under, so the
    // census is complete and its only defect is having no acceptance.
    assert!(matches!(
        validate_case_census(&census, &committed),
        Err(CaseCensusDefect::NoAcceptingCase),
    ));
}
