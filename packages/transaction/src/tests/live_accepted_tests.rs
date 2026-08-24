//! The accepted result: one census, bound to one complete set of
//! returned owner authorizations.
//!
//! Every case here is a construction case. Nothing in this file offers
//! anything to a node, and no assertion below is a target verdict — the
//! whole subject is whether a set of answers is *accepted as a result*,
//! which is a question about binding and cardinality rather than about
//! validity.
//!
//! The census under test is the pinned one the census tests already
//! build through the parts seam, reused rather than rebuilt so that the
//! two files cannot drift into censusing different candidates.

use super::live_census_tests::{GENESIS, pinned_census};
use super::reviewed_target;
use crate::live_accepted::{
    AcceptedOwnerAuthorizations, AcceptedResultRefusal, OfferedOwnerAuthorization,
};
use crate::live_census::{
    LiveDeployment, OWNER_SIGHASH_TYPE_BYTE, OWNER_SIGNATURE_BYTES, OwnerCensusRefusal,
    OwnerSigningCensus,
};

/// A well formed answer for one signing input.
///
/// The bytes are a repeated fill and stand for nothing. The accepted
/// result never parses an authorization, so what matters about this
/// value is its width and the input it names.
fn answer(input_index: u32) -> OfferedOwnerAuthorization {
    OfferedOwnerAuthorization::new(
        input_index,
        vec![0x5c; OWNER_SIGNATURE_BYTES],
        OWNER_SIGHASH_TYPE_BYTE,
    )
}

/// Every answer the pinned census needs.
fn every_answer(census: &OwnerSigningCensus) -> Vec<OfferedOwnerAuthorization> {
    census
        .signing_inputs()
        .iter()
        .map(|input| answer(input.input_index()))
        .collect()
}

/// Binds the pinned census, with the answers rewritten by `rewrite`.
fn bind_with(
    rewrite: impl FnOnce(Vec<OfferedOwnerAuthorization>) -> Vec<OfferedOwnerAuthorization>,
) -> Result<AcceptedOwnerAuthorizations, AcceptedResultRefusal> {
    let target = reviewed_target();
    let census = pinned_census(&target);
    let protected = census.protected_bytes().to_vec();
    let answers = rewrite(every_answer(&census));

    AcceptedOwnerAuthorizations::bind(census, &protected, LiveDeployment::new(GENESIS), answers)
}

#[test]
fn a_complete_set_of_answers_binds_to_the_census() {
    let bound = bind_with(|answers| answers).expect("a complete set binds");

    assert_eq!(bound.answered(), bound.census().signing_inputs().len());
    for input in bound.census().signing_inputs() {
        assert_eq!(
            bound.authorization(input.input_index()),
            Some(&[0x5c; OWNER_SIGNATURE_BYTES][..]),
        );
    }
}

#[test]
fn the_bound_protected_bytes_are_the_census_bytes() {
    // One value and not two. The accepted result's whole binding claim
    // is that the bytes every owner signed over are the bytes the
    // candidate carries, so the result reads them off the census rather
    // than keeping a copy that could disagree with it.
    let target = reviewed_target();
    let census = pinned_census(&target);
    let expected = census.protected_bytes().to_vec();

    let bound = bind_with(|answers| answers).expect("a complete set binds");

    assert_eq!(bound.protected_bytes(), expected.as_slice());
}

#[test]
fn a_candidate_whose_protected_bytes_differ_is_the_wrong_candidate() {
    // The consuming guide's `WrongCandidate`, recomputed here by exact
    // byte comparison rather than by trusting that the signer looked.
    // One byte is enough: this is the refusal that catches an
    // authorization taken over a candidate whose output-witness vector
    // changed after the census was built.
    let target = reviewed_target();
    let census = pinned_census(&target);
    let mut offered = census.protected_bytes().to_vec();
    offered[0] ^= 0xff;
    let answers = every_answer(&census);

    assert_eq!(
        AcceptedOwnerAuthorizations::bind(census, &offered, LiveDeployment::new(GENESIS), answers,),
        Err(AcceptedResultRefusal::Census(
            OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates
        )),
    );
}

#[test]
fn answers_carried_to_another_deployment_are_refused() {
    // Failure-matrix row 4, checked on the result rather than only on
    // the census: two candidates identical to the last byte have
    // different messages on two chains, and nothing about a returned
    // authorization reveals which chain it was formed for.
    let target = reviewed_target();
    let census = pinned_census(&target);
    let protected = census.protected_bytes().to_vec();
    let answers = every_answer(&census);
    let mut other = GENESIS;
    other[0] ^= 0xff;

    let refused =
        AcceptedOwnerAuthorizations::bind(census, &protected, LiveDeployment::new(other), answers);

    assert!(matches!(
        refused,
        Err(AcceptedResultRefusal::Census(
            OwnerCensusRefusal::DeploymentMismatch { .. }
        )),
    ));
}

#[test]
fn a_returned_type_byte_outside_the_profile_is_a_refusal_and_not_a_variant() {
    // The accepted result's third constraint. A returned `0x01` under a
    // default-only profile forms a different message, so accepting it
    // would be naming a profile this result does not hold to.
    let refused = bind_with(|answers| {
        answers
            .into_iter()
            .map(|answer| {
                OfferedOwnerAuthorization::new(
                    answer.input_index(),
                    answer.authorization().to_vec(),
                    0x01,
                )
            })
            .collect()
    });

    assert_eq!(
        refused,
        Err(AcceptedResultRefusal::Census(
            OwnerCensusRefusal::TypeByteOutsideProfile { offered: 0x01 }
        )),
    );
}

#[test]
fn a_returned_authorization_of_the_other_width_is_refused() {
    // Sixty-five is the width a non-default type byte would carry, so
    // this is the same disagreement arriving in the other of its two
    // shapes. The two are separate refusals because the target reports
    // them separately.
    let refused = bind_with(|answers| {
        answers
            .into_iter()
            .map(|answer| {
                let mut wider = answer.authorization().to_vec();
                wider.push(OWNER_SIGHASH_TYPE_BYTE);
                OfferedOwnerAuthorization::new(answer.input_index(), wider, answer.type_byte())
            })
            .collect()
    });

    assert_eq!(
        refused,
        Err(AcceptedResultRefusal::Census(
            OwnerCensusRefusal::SignatureWidthOutsideProfile {
                offered: OWNER_SIGNATURE_BYTES + 1
            }
        )),
    );
}

#[test]
fn a_signing_input_left_unanswered_is_a_missing_owner() {
    // The consuming guide's `MissingOwner`, recomputed from the census.
    // A result short by one input is a partial authorization presented
    // as a complete one, and it is refused rather than reported.
    let refused = bind_with(|mut answers| {
        answers.pop();
        answers
    });

    assert!(matches!(
        refused,
        Err(AcceptedResultRefusal::SigningInputWithNoAuthorization { .. }),
    ));
}

#[test]
fn an_answer_for_an_input_the_census_does_not_have_is_refused() {
    // A surplus answer rather than a missing one, and the two are
    // different faults: this one means the returning party censused a
    // different candidate.
    let refused = bind_with(|mut answers| {
        answers.push(answer(u32::MAX));
        answers
    });

    assert_eq!(
        refused,
        Err(
            AcceptedResultRefusal::AuthorizationForAnInputTheCensusDoesNotHave {
                input_index: u32::MAX
            }
        ),
    );
}

#[test]
fn two_answers_for_one_input_are_refused_rather_than_resolved() {
    // A later answer silently replacing an earlier one would make which
    // authorization was bound depend on iteration order.
    let refused = bind_with(|mut answers| {
        let first = answers[0].input_index();
        answers.push(answer(first));
        answers
    });

    assert!(matches!(
        refused,
        Err(AcceptedResultRefusal::TwoAuthorizationsForOneInput { .. }),
    ));
}
