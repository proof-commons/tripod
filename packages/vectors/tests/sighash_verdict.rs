//! The owner sighash review verdict, checked where every half is
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
//! What is checked is that the disposition is *recomputed*, and that the
//! two independently maintained facts it is recomputed from still say
//! what the verdict and the ruling left them saying.
//!
//! # What this file used to check, and why it now checks something else
//!
//! It checked that the stop was *narrow*: exactly one required
//! dimension, named, and the same one everywhere it was named. That stop
//! was real and this file does not deny it. What repaired it is not a
//! run — nothing has been observed since — but the post-verdict
//! re-typing of the issuance dimension from required to refused, on
//! the ground that the census refuses an issuance-bearing signing
//! request and the decoder refuses issuance-bearing bytes, so the
//! required typing was the one layer promising evidence the other two
//! refuse to admit.
//!
//! So the assertions moved with the fact and not with the wish. Where
//! the stop was asserted, its repair is asserted *and its shape checked*
//! — the dimension is refused, not reviewed, and the reviewed set did
//! not grow to swallow it.

use tapscript::authorization::{
    DimensionRefusal, DimensionRole, OwnerProfileDisposition, selected_owner_profile,
};
use target_elements::{SighashDimension, UnreviewedGround, reviewed_elements_tapscript};
use vectors::live_corpus_native_v2_r7;

// The Wave-3 selected-profile identity projected from the validated current corpus.
fn current_owner_observation_accepted_txid() -> &'static str {
    let corpus = live_corpus_native_v2_r7::run_of_record()
        .expect("the reviewed native-v2/revision-7 corpus validates");
    let acceptances = corpus
        .acceptance_projections("owner-observation")
        .expect("the corpus projects the current owner-observation ceremony");
    let [acceptance] = acceptances else {
        panic!("the current owner-observation ceremony has one acceptance");
    };
    acceptance.identity_display()
}

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
fn the_disposition_is_recomputed_to_established_over_a_six_member_required_set() {
    // Both sides, and the count first. An established disposition is
    // reachable two ways — every required dimension reviewed, or the
    // required set emptied until none is left to be unreviewed — and only
    // the first is the verdict. So the size of the required set is
    // asserted as a literal beside the disposition, and a later edit that
    // reached the same answer by dropping a seventh dimension fails here
    // rather than reading as the same verdict.
    let profile = selected_owner_profile();

    assert_eq!(profile.required().count(), 6);
    assert_eq!(
        profile.assess(&capability()),
        OwnerProfileDisposition::Established,
    );
}

#[test]
fn every_required_dimension_rests_on_the_run_that_was_observed() {
    // The discriminating half. A verdict that had quietly populated the
    // reviewed set with ungrounded members would also produce an
    // established disposition, so each required dimension is walked and
    // both halves of the accepted evidence ruling are read off it: the
    // source citation, and the observation that exercised it.
    //
    // The transaction identity is imported from the Wave-3 ceremony's
    // provenance-correct run record, so a reviewed set grown against some
    // other run fails here.
    let profile = selected_owner_profile();
    let capability = capability();
    let accepted_txid = current_owner_observation_accepted_txid();

    let mut walked = 0_usize;
    for dimension in profile.required() {
        let ground = capability
            .reviewed()
            .get(&dimension)
            .unwrap_or_else(|| panic!("{dimension:?} is established"));

        assert!(!ground.citation().terms().is_empty(), "{dimension:?}");
        assert_eq!(
            ground.exercised_by().observation().accepted_transaction(),
            accepted_txid,
            "{dimension:?} rests on the run that was observed",
        );
        walked += 1;
    }

    assert_eq!(walked, 6, "every required dimension was walked");
}

#[test]
fn the_re_typed_dimension_is_refused_by_the_profile_and_unreviewed_by_the_contract() {
    // The shape of the repair, and the reason this file asserts it in two
    // places rather than one. The ruling refused the issuance dimension;
    // it did not establish it. A tree that had cleared the stop by
    // marking the dimension reviewed would satisfy the disposition test
    // above and would be recording evidence no run produced — so the
    // profile side is checked for a refusal *with its ground*, and the
    // contract side is checked for the dimension still being unreviewed.
    let profile = selected_owner_profile();
    let capability = capability();

    assert_eq!(
        profile.role(SighashDimension::Issuance),
        Some(DimensionRole::Refused(
            DimensionRefusal::SubjectRefusedByTheCensusAndTheDecoder,
        )),
    );
    assert!(!capability.is_reviewed(SighashDimension::Issuance));

    let ground = capability
        .unreviewed()
        .get(&SighashDimension::Issuance)
        .expect("the re-typed dimension is classified");

    let UnreviewedGround::TheSelectedProfileRefusesIt(citation) = ground else {
        panic!("the re-typing is recorded as the wrong kind of unreviewed");
    };

    // The citation stays the review's own, so a reader can still check
    // the claim that the target does carry the dimension even though this
    // workspace refuses its subject. The ruling changed what the profile
    // asks of the dimension, not what the reviewer read.
    assert_eq!(citation.terms(), "terms 9 and 10");
    assert_eq!(
        citation.written_at(),
        "src/script/interpreter.cpp:2738-2739",
    );
}

#[test]
fn the_profile_is_established_and_the_reviewed_set_did_not_grow_to_do_it() {
    // The gate the consuming guide's handoff reads, stated here as the
    // single fact it is, with the guard that keeps it honest beside it.
    // An accepted result can be built and bound — that is `transaction`'s
    // own tests — and whether handing one across is entered is the
    // consuming guide's decision, which this now permits rather than
    // decides.
    let capability = capability();

    assert_eq!(
        selected_owner_profile().assess(&capability),
        OwnerProfileDisposition::Established,
    );

    // Six reviewed and four unreviewed, over a ten-dimension census. The
    // partition is asserted rather than the reviewed count alone, because
    // a dimension that fell out of both sets would leave the required
    // ones untouched and the contract's own totality quietly broken.
    assert_eq!(capability.reviewed().len(), 6);
    assert_eq!(capability.unreviewed().len(), 4);
    assert_eq!(capability.unclassified(), None);
    assert_eq!(capability.contradictory(), None);
}
