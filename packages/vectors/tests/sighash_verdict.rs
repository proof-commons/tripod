//! The owner sighash review verdict, checked where both halves are
//! visible at once.
//!
//! The selected profile and its assessment live in `tapscript`; the
//! reviewed capability the assessment reads lives in `target-elements`;
//! the accepted result the verdict gates lives in `transaction`. This
//! crate is the only one that depends on all three, so it is the only
//! place the whole claim can be stated as one thing rather than as three
//! statements that happen to agree.
//!
//! Nothing here runs a node, and no assertion below is a target verdict.
//! What is checked is that the disposition is *recomputed* and that the
//! stop is *narrow*: exactly one required dimension, named, and the same
//! one everywhere it is named.

use tapscript::authorization::{OwnerProfileDisposition, selected_owner_profile};
use target_elements::{SighashDimension, UnreviewedGround, reviewed_elements_tapscript};

/// The reviewed contract's sighash capability.
fn capability() -> target_elements::SighashCapability {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .authorization()
        .sighash()
        .clone()
}

#[test]
fn the_disposition_is_recomputed_and_stops_on_one_named_dimension() {
    // The verdict of record. Six of the seven required dimensions were
    // exercised by the observed acceptance; the seventh was not, and the
    // disposition names it rather than reporting a bare incompleteness.
    let disposition = selected_owner_profile().assess(&capability());

    assert_eq!(
        disposition,
        OwnerProfileDisposition::ReviewIncomplete {
            unreviewed: [SighashDimension::Issuance].into_iter().collect(),
        },
    );
}

#[test]
fn the_stop_is_not_wider_than_one_dimension() {
    // The discriminating half. A verdict that had quietly failed to
    // populate the capability would also produce a review-incomplete
    // disposition, so the count is asserted from the other side: every
    // required dimension but the stopped one is established, with both
    // halves of the evidence ruling behind it.
    let profile = selected_owner_profile();
    let capability = capability();

    let established = profile
        .required()
        .filter(|dimension| capability.is_reviewed(*dimension))
        .count();

    assert_eq!(established, profile.required().count() - 1);

    for dimension in profile.required() {
        if dimension == SighashDimension::Issuance {
            continue;
        }

        let ground = capability
            .reviewed()
            .get(&dimension)
            .unwrap_or_else(|| panic!("{dimension:?} is established"));

        assert!(!ground.citation().terms().is_empty(), "{dimension:?}");
        assert_eq!(
            ground.exercised_by().observation().accepted_transaction(),
            "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029",
            "{dimension:?} rests on the run that was observed",
        );
    }
}

#[test]
fn the_stopped_dimension_records_why_no_candidate_exercises_it() {
    // A stopped verdict is valid and an unexplained one is not. The
    // ground says which of the three kinds of unreviewed this is, and
    // the kind is what a later wave reads to know that the repair is a
    // construction rather than a rerun.
    let capability = capability();
    let ground = capability
        .unreviewed()
        .get(&SighashDimension::Issuance)
        .expect("the stopped dimension is classified");

    let UnreviewedGround::NoCandidateThisArcBuildsCarriesTheSubject(citation) = ground else {
        panic!("the stop is recorded as the wrong kind of unreviewed");
    };

    // The citation is the review's own, so a reader can check the claim
    // that the target does carry the dimension even though this arc
    // cannot exercise it.
    assert_eq!(citation.terms(), "terms 9 and 10");
    assert_eq!(
        citation.written_at(),
        "src/script/interpreter.cpp:2738-2739",
    );
}

#[test]
fn the_profile_is_not_established_while_the_stop_stands() {
    // The gate the consuming guide's handoff reads, stated here as the
    // single fact it is. An accepted result can be built and bound today
    // — that is `transaction`'s own tests — and handing one across is
    // not entered while this holds.
    assert_ne!(
        selected_owner_profile().assess(&capability()),
        OwnerProfileDisposition::Established,
    );
}
