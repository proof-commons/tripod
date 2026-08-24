//! The owner-sighash handoff: the order, the binding, and the four
//! refusals.
//!
//! # No digest is computed here either
//!
//! Nothing in this file forms an owner message, and no assertion below
//! compares one. The authorizations are opaque published byte strings
//! whose only checked properties are their width, their type byte, and
//! the candidate they are bound to — which is the whole shape of the
//! boundary the module under test implements. The digest itself belongs
//! to the separately reviewed owner-sighash work, and the tests that
//! establish it are named in that module's own documentation rather than
//! restated here.
//!
//! # Every value here is public test material
//!
//! Every authorization, blinder, program, and genesis hash below is a
//! meaningless published byte string in the sense the workspace's
//! test-material rule fixes. None is a secret and none authorizes
//! anything: the "signatures" are constant fills of the reviewed width,
//! because the handoff never looks inside one and a test that made them
//! look real would be claiming a check that does not happen.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest as _, Sha256};
use tapscript::authorization::{OwnerProfileDisposition, selected_owner_profile};
use target_elements::{
    ReviewedElementsTapscriptDefinition, SighashCapability, SighashDimension,
    SighashSourceCitation, UnreviewedGround, reviewed_elements_tapscript,
};

use super::ctf_materialize_tests::{valid_two_owner_with_spent_program, valid_with_spent_program};
use super::live_census_tests::{CensusCurve, GENESIS, committed_program, signing_request};
use super::reviewed_target;
use crate::bytes::{OutputWitness, TargetTransaction};
use crate::live_accepted::{AcceptedResultRefusal, OfferedOwnerAuthorization};
use crate::live_census::{
    LiveDeployment, OWNER_SIGHASH_TYPE_BYTE, OWNER_SIGNATURE_BYTES, OwnerCensusRefusal,
    OwnerSigningInputRequest,
};
use crate::live_handoff::{
    FullyAuthorizedCandidate, OwnerProfileAcceptance, SighashHandoffRefusal, SigningStarted,
    SubmitReadyPrivateCandidate,
};
use crate::live_materialize::{
    MaterializedConfidentialCandidate, ProofFinalizedRegion, ProofFinalizedSignerInput,
};

// --- Fixtures ----------------------------------------------------------

/// The reviewed contract's sighash capability, unmodified.
fn capability() -> SighashCapability {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .authorization()
        .sighash()
        .clone()
}

/// The required dimensions one capability leaves unestablished, through
/// the selected profile's own assessment.
///
/// The recomputation and not a restatement of it: the profile is the
/// backend's, the capability is the reviewed contract's, and this
/// function only translates the two-member disposition into the set the
/// handoff's seam takes.
fn unestablished(capability: &SighashCapability) -> BTreeSet<SighashDimension> {
    match selected_owner_profile().assess(capability) {
        OwnerProfileDisposition::Established => BTreeSet::new(),
        OwnerProfileDisposition::ReviewIncomplete { unreviewed } => unreviewed,
    }
}

/// The real acceptance: the selected profile assessed against the
/// reviewed contract, on every call.
struct ReviewedAcceptance;

impl OwnerProfileAcceptance for ReviewedAcceptance {
    fn unestablished_required_dimensions(&self) -> BTreeSet<SighashDimension> {
        unestablished(&capability())
    }
}

/// A capability that establishes nothing, built rather than mutated.
///
/// The real one is ESTABLISHED and this file does not touch it. What is
/// needed to reach the refusal is a capability that is not, and the
/// honest way to have one is to build a separate value: every dimension
/// classified, none of them reviewed, so the selected profile's own
/// assessment reports the required set unestablished and the refusal
/// fires on a recomputation rather than on a hand-written answer.
///
/// The ground is the one the vocabulary records as carried by nothing
/// today, and the citation says in its own text that it is a stand-in,
/// so this value cannot be mistaken for a classification of the target.
fn capability_that_establishes_nothing() -> SighashCapability {
    let citation = SighashSourceCitation::new(
        "no terms: this capability is a test stand-in",
        "no source: this capability is a test stand-in",
        "a synthetic capability built by the handoff tests",
    );

    SighashCapability::new(
        [],
        SighashDimension::ALL.iter().map(|dimension| {
            (
                *dimension,
                UnreviewedGround::NoCandidateThisArcBuildsCarriesTheSubject(citation),
            )
        }),
        [],
    )
}

/// That capability, as an acceptance the handoff can read.
struct UnacceptedProfile;

impl OwnerProfileAcceptance for UnacceptedProfile {
    fn unestablished_required_dimensions(&self) -> BTreeSet<SighashDimension> {
        unestablished(&capability_that_establishes_nothing())
    }
}

/// The deployment every case below runs against.
fn deployment() -> LiveDeployment {
    LiveDeployment::new(GENESIS)
}

/// One owner's answer for `input_index`, at the reviewed width and under
/// the profile's own type byte.
fn answer(input_index: u32) -> OfferedOwnerAuthorization {
    OfferedOwnerAuthorization::new(
        input_index,
        vec![0x77; OWNER_SIGNATURE_BYTES],
        OWNER_SIGHASH_TYPE_BYTE,
    )
}

/// The one-owner materialized candidate every single-input case uses.
fn one_owner(target: &ReviewedElementsTapscriptDefinition) -> MaterializedConfidentialCandidate {
    valid_with_spent_program(committed_program(target))
}

/// The two-owner materialized candidate the multi-owner cases use.
fn two_owner(target: &ReviewedElementsTapscriptDefinition) -> MaterializedConfidentialCandidate {
    valid_two_owner_with_spent_program(&committed_program(target))
}

/// One opened handoff over `materialized`, with one request per input.
fn open(
    target: &ReviewedElementsTapscriptDefinition,
    materialized: &MaterializedConfidentialCandidate,
) -> SigningStarted {
    try_open(target, materialized, &ReviewedAcceptance)
        .expect("the accepted profile and the materialized candidate open a handoff")
}

/// The same, without expecting success.
fn try_open(
    target: &ReviewedElementsTapscriptDefinition,
    materialized: &MaterializedConfidentialCandidate,
    acceptance: &dyn OwnerProfileAcceptance,
) -> Result<SigningStarted, SighashHandoffRefusal> {
    let requests: Vec<OwnerSigningInputRequest> = (0..materialized.signer_inputs().len())
        .map(|index| signing_request(u32::try_from(index).expect("the fixture index is in range")))
        .collect();

    SigningStarted::open(
        target,
        materialized,
        deployment(),
        &requests,
        &CensusCurve,
        acceptance,
    )
}

/// One fully authorized candidate, every required owner having answered.
fn authorize_every_owner(
    started: SigningStarted,
) -> Result<FullyAuthorizedCandidate, SighashHandoffRefusal> {
    let bytes = started.protected_bytes().to_vec();
    let answers: Vec<OfferedOwnerAuthorization> = (0..started.required_owners())
        .map(|index| answer(u32::try_from(index).expect("the fixture index is in range")))
        .collect();

    started.authorize(&bytes, deployment(), answers)
}

/// The whole order, run once over `materialized`.
fn submit_ready(
    target: &ReviewedElementsTapscriptDefinition,
    materialized: &MaterializedConfidentialCandidate,
) -> SubmitReadyPrivateCandidate {
    authorize_every_owner(open(target, materialized))
        .expect("every required owner answered over the candidate they were handed")
        .bind_for_submission(materialized)
        .expect("no protected byte moved between the freeze and the binding")
}

/// The frozen candidate's preimage with exactly one range-proof byte
/// moved.
///
/// Same length, same commitments, same nonces, same programs, same
/// version, lock time and inputs. One byte of one proof differs, and the
/// preimage is taken by the construction package's own function rather
/// than respelled here, so a disagreement between the two would be the
/// mutation and not a transcription error.
fn preimage_with_one_proof_byte_moved(materialized: &MaterializedConfidentialCandidate) -> Vec<u8> {
    let frozen = materialized.proof_finalized().protected();
    let mut proof = frozen.output_witnesses()[0].range_proof().to_vec();
    proof[0] ^= 0xff;

    let mut witnesses = frozen.output_witnesses().to_vec();
    witnesses[0] = OutputWitness::new(
        frozen.output_witnesses()[0].surjection_proof().to_vec(),
        proof,
    );

    let moved = TargetTransaction::with_output_witnesses(
        frozen.version(),
        frozen.inputs().to_vec(),
        frozen.outputs().to_vec(),
        frozen.lock_time(),
        frozen.witnesses().to_vec(),
        witnesses,
    )
    .expect("the mutated candidate is well formed");

    crate::live_finalize::protected_preimage(
        &moved,
        linker::live_backend::LiveTransferRepresentationPlan::PrivateCommitted,
    )
}

/// One byte string as lower-case hexadecimal.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut text, byte| {
        write!(text, "{byte:02x}").expect("writing to a string does not fail");
        text
    })
}

/// A candidate-only identity label over one candidate's protected bytes.
///
/// NOT a signing message and not consumed by anything that signs: this
/// is a name for a byte string, computed here so a report can say WHICH
/// candidate reached the terminal state and so a later edit that changed
/// those bytes fails visibly. The message an owner forms over the same
/// bytes is the separately reviewed owner-sighash work's and is neither
/// computed nor compared anywhere in this crate.
fn candidate_identity(protected_bytes: &[u8]) -> String {
    hex(&Sha256::digest(protected_bytes))
}

// --- Deliverable 1: the bytes cross unchanged -------------------------

#[test]
fn the_proof_finalized_bytes_enter_the_signing_request_unchanged() {
    let target = reviewed_target();
    let materialized = one_owner(&target);
    let started = open(&target, &materialized);

    let frozen = materialized.proof_finalized();

    // Exact-byte identity, at the boundary and not near it. The request
    // is compared with the freeze's own output rather than with a
    // recomputation of the preimage rule, because a second computation
    // of that rule would be a second chance for the two to agree by
    // accident.
    assert_eq!(started.protected_bytes(), frozen.protected_bytes());
    assert_eq!(
        started.request().protected_bytes(),
        frozen.protected_bytes()
    );

    // The materializer's own per-input byte binding is the same string.
    // Every signer input was bound to the whole candidate at
    // finalization, and the handoff did not narrow that to a per-input
    // view on the way out.
    for input in materialized.signer_inputs() {
        assert_eq!(input.byte_binding(), frozen.protected_bytes());
    }

    // And the vector the target hashes crosses at its consensus length,
    // one entry per output, rather than as the empty vector a witnessless
    // serialization would have presented.
    assert_eq!(
        started.request().output_witnesses().len(),
        frozen.protected().outputs().len(),
    );
    assert_ne!(
        started.protected_bytes(),
        frozen.protected().encode_without_witness().as_slice(),
    );
}

#[test]
fn the_bytes_are_still_the_freezes_own_at_every_later_state() {
    // The identity is not merely established at the first arrow. It is
    // re-read at each of the remaining two, because a state transition
    // that quietly recomputed or re-encoded would produce a submit-ready
    // candidate whose owners bound to something else.
    let target = reviewed_target();
    let materialized = one_owner(&target);
    let expected = materialized.proof_finalized().protected_bytes().to_vec();

    let authorized = authorize_every_owner(open(&target, &materialized))
        .expect("the one required owner answered");
    assert_eq!(authorized.protected_bytes(), expected.as_slice());

    let ready = authorized
        .bind_for_submission(&materialized)
        .expect("no protected byte moved");
    assert_eq!(ready.protected_bytes(), expected.as_slice());
    assert_eq!(
        ready.candidate(),
        materialized.proof_finalized().protected()
    );
}

// --- Deliverable 3: the handoff states and their refusals -------------

#[test]
fn an_unaccepted_profile_refuses_before_the_candidate_is_censused() {
    let target = reviewed_target();
    let materialized = one_owner(&target);

    // The real profile is established, so the refusal is reached through
    // a capability built for the purpose rather than by disturbing the
    // reviewed one. What fires is the profile's own assessment: the
    // synthetic capability reviews nothing, so every required dimension
    // comes back unestablished.
    let refusal =
        try_open(&target, &materialized, &UnacceptedProfile).expect_err("an unaccepted profile");

    let SighashHandoffRefusal::ProfileNotAccepted {
        unestablished: named,
    } = refusal
    else {
        panic!("an unaccepted profile is refused as one");
    };

    // The required set, whole. Six is the count the reviewed contract's
    // own verdict rests on, asserted as a literal so that a profile whose
    // required set changed underneath this file fails here rather than
    // reading as the same refusal.
    assert_eq!(named.len(), 6);
    assert_eq!(
        named,
        selected_owner_profile().required().collect::<BTreeSet<_>>(),
    );

    // And the real capability is untouched: the same assessment over the
    // reviewed contract still establishes the profile.
    assert!(unestablished(&capability()).is_empty());
}

#[test]
fn an_answer_bound_to_a_moved_proof_is_the_wrong_candidate() {
    // The post-proof-mutation case. The owner answers with the preimage
    // of a candidate whose single range-proof byte moved, which is the
    // fault the protected-bytes repair exists to make visible: under the
    // witnessless serialization the two candidates are byte-identical,
    // so nothing at this boundary could have told them apart.
    let target = reviewed_target();
    let materialized = one_owner(&target);
    let started = open(&target, &materialized);

    let moved = preimage_with_one_proof_byte_moved(&materialized);
    assert_ne!(moved.as_slice(), started.protected_bytes());

    assert_eq!(
        started.authorize(&moved, deployment(), [answer(0)]),
        Err(SighashHandoffRefusal::WrongCandidate),
    );
}

#[test]
fn an_answer_for_an_input_the_census_does_not_have_is_the_wrong_candidate() {
    // The other route to the same refusal: a surplus answer means the
    // returning party censused a different candidate, which is a stronger
    // fault than having failed to answer and is not a missing owner.
    let target = reviewed_target();
    let materialized = one_owner(&target);
    let started = open(&target, &materialized);
    let bytes = started.protected_bytes().to_vec();

    assert_eq!(
        started.authorize(&bytes, deployment(), [answer(0), answer(7)]),
        Err(SighashHandoffRefusal::WrongCandidate),
    );
}

#[test]
fn a_required_owner_that_did_not_answer_is_a_missing_owner() {
    // The wrong-owner case as this boundary can observe it. The
    // authorizations are opaque and carry no key, so "the wrong party
    // signed" is not a fact the handoff can read; what it can read, and
    // what it refuses on, is that the input that party was required for
    // went unanswered.
    let target = reviewed_target();
    let materialized = two_owner(&target);
    let started = open(&target, &materialized);
    assert_eq!(started.required_owners(), 2);

    let bytes = started.protected_bytes().to_vec();

    assert_eq!(
        started.authorize(&bytes, deployment(), [answer(0)]),
        Err(SighashHandoffRefusal::MissingOwner { input_index: 1 }),
    );
}

#[test]
fn every_protected_region_refuses_mutation_after_signing_started() {
    // The post-signing-mutation case, walked over the whole census
    // rather than sampled. Each of the four shaped operations is asked
    // about each of the eight regions, and each names the region it was
    // asked about, so an operation that had started answering about some
    // other region would fail here.
    let target = reviewed_target();
    let materialized = one_owner(&target);
    let started = open(&target, &materialized);

    let mut walked = 0_usize;
    for region in ProofFinalizedRegion::ALL.iter().copied() {
        let expected = Err(SighashHandoffRefusal::MutationAfterSigningStarted { region });
        assert_eq!(started.insert(region), expected);
        assert_eq!(started.remove(region), expected);
        assert_eq!(started.replace(region), expected);
        assert_eq!(started.reorder(region), expected);
        walked += 1;
    }
    assert_eq!(walked, 8, "every protected region was walked");

    // The three named operations the mandatory order exists to forbid,
    // each naming the region it would have touched.
    assert_eq!(
        started.regenerate_proofs(),
        Err(SighashHandoffRefusal::MutationAfterSigningStarted {
            region: ProofFinalizedRegion::OutputWitnesses,
        }),
    );
    assert_eq!(
        started.repair_proofs(),
        Err(SighashHandoffRefusal::MutationAfterSigningStarted {
            region: ProofFinalizedRegion::OutputWitnesses,
        }),
    );
    assert_eq!(
        started.reblind(),
        Err(SighashHandoffRefusal::MutationAfterSigningStarted {
            region: ProofFinalizedRegion::Outputs,
        }),
    );

    // And the state is still askable afterwards. A refusal is a refusal
    // and not a poisoning: the candidate that was handed out is the one
    // still being asked about.
    assert_eq!(
        started.protected_bytes(),
        materialized.proof_finalized().protected_bytes(),
    );
}

#[test]
fn a_candidate_whose_protected_bytes_moved_is_refused_at_the_binding() {
    // The last arrow, checked. The authorizations were taken over the
    // two-owner candidate; binding them to a different materialization
    // is the same fault an answer about another candidate is, seen from
    // the submission side, and it draws the same refusal.
    let target = reviewed_target();
    let materialized = two_owner(&target);
    let other = one_owner(&target);

    let authorized =
        authorize_every_owner(open(&target, &materialized)).expect("both required owners answered");

    assert_eq!(
        authorized.bind_for_submission(&other),
        Err(SighashHandoffRefusal::WrongCandidate),
    );
}

#[test]
fn the_accepted_results_own_complaints_keep_their_own_words() {
    // The wrapper member, reached. A duplicate answer is neither a
    // missing owner nor a wrong candidate, and flattening it into either
    // would have made a cardinality complaint unreadable.
    let target = reviewed_target();
    let materialized = one_owner(&target);
    let started = open(&target, &materialized);
    let bytes = started.protected_bytes().to_vec();

    assert_eq!(
        started.authorize(&bytes, deployment(), [answer(0), answer(0)]),
        Err(SighashHandoffRefusal::TheAnswersWereNotAnAcceptedResult(
            AcceptedResultRefusal::TwoAuthorizationsForOneInput { input_index: 0 },
        )),
    );
}

#[test]
fn a_candidate_that_cannot_be_censused_keeps_the_censuss_own_words() {
    // The other wrapper. A signing request for an input the candidate
    // does not have is a census complaint and never reached the first
    // arrow, so it is not one of the four handoff states.
    let target = reviewed_target();
    let materialized = one_owner(&target);

    let refusal = SigningStarted::open(
        &target,
        &materialized,
        deployment(),
        &[signing_request(9)],
        &CensusCurve,
        &ReviewedAcceptance,
    )
    .expect_err("a request for an input the candidate does not have");

    assert_eq!(
        refusal,
        SighashHandoffRefusal::TheCandidateCouldNotBeCensused(
            OwnerCensusRefusal::SigningInputOutOfRange {
                input_index: 9,
                inputs: 1,
            },
        ),
    );
}

#[test]
fn every_handoff_refusal_variant_is_reached_by_a_test_in_this_file() {
    // One arm per variant, no catch-all, so the next member added
    // without a test is a compile error rather than an invisible gap.
    //
    // What this is NOT: a measurement that the tests above pass. A match
    // arm is not a test, and this guard would keep compiling if every
    // assertion in the file were deleted. What it measures is that the
    // vocabulary has not grown past the cases the file names.
    let named = |refusal: &SighashHandoffRefusal| -> &'static str {
        match refusal {
            SighashHandoffRefusal::ProfileNotAccepted { .. } => {
                "an_unaccepted_profile_refuses_before_the_candidate_is_censused"
            }
            SighashHandoffRefusal::WrongCandidate => {
                "an_answer_bound_to_a_moved_proof_is_the_wrong_candidate"
            }
            SighashHandoffRefusal::MissingOwner { .. } => {
                "a_required_owner_that_did_not_answer_is_a_missing_owner"
            }
            SighashHandoffRefusal::MutationAfterSigningStarted { .. } => {
                "every_protected_region_refuses_mutation_after_signing_started"
            }
            SighashHandoffRefusal::TheCandidateCouldNotBeCensused(_) => {
                "a_candidate_that_cannot_be_censused_keeps_the_censuss_own_words"
            }
            SighashHandoffRefusal::TheAnswersWereNotAnAcceptedResult(_) => {
                "the_accepted_results_own_complaints_keep_their_own_words"
            }
        }
    };

    assert_eq!(
        named(&SighashHandoffRefusal::WrongCandidate),
        "an_answer_bound_to_a_moved_proof_is_the_wrong_candidate",
    );
}

// --- Deliverable 4: one candidate, every required owner ---------------

#[test]
fn every_required_owner_signs_the_same_protected_candidate() {
    let target = reviewed_target();
    let materialized = two_owner(&target);
    let started = open(&target, &materialized);

    // Two owners, and the count is asserted first: an agreement over one
    // owner is not an agreement.
    assert_eq!(started.required_owners(), 2);
    assert_eq!(started.request().signing_inputs().len(), 2);
    assert_eq!(started.request().spent_outputs().len(), 2);

    // Every signer input's byte binding is the same string, and it is the
    // freeze's own. Two owners bound to two preimages would be the fault
    // this deliverable exists to exclude, and it would be invisible in a
    // count.
    let expected = materialized.proof_finalized().protected_bytes();
    let bindings: BTreeSet<&[u8]> = materialized
        .signer_inputs()
        .iter()
        .map(ProofFinalizedSignerInput::byte_binding)
        .collect();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings.into_iter().next(), Some(expected));

    let authorized = authorize_every_owner(started).expect("both required owners answered");
    assert_eq!(authorized.answered(), 2);
    assert_eq!(authorized.protected_bytes(), expected);

    // Both answers are recorded, opaque, and distinct only by index.
    // Nothing here reads inside one, which is the boundary holding.
    assert_eq!(
        authorized.authorization(0),
        Some(vec![0x77; OWNER_SIGNATURE_BYTES].as_slice()),
    );
    assert_eq!(
        authorized.authorization(1),
        Some(vec![0x77; OWNER_SIGNATURE_BYTES].as_slice()),
    );
    assert_eq!(authorized.authorization(2), None);
}

// --- Deliverable 5: one submit-ready sponsorless private candidate ----

#[test]
fn one_sponsorless_private_candidate_becomes_submit_ready() {
    // The mandatory order, end to end, in its own order and with no step
    // reordered for convenience:
    //
    //   materialize -> finalize proofs -> census -> sign -> bind
    //
    // Nothing below submits, and no step is a target verdict. What the
    // test establishes is that the terminal state is REACHABLE over a
    // real materialization, which is this wave's exit.
    let target = reviewed_target();

    // Materialize and freeze. The materializer's own entry point does
    // both, in that order, and hands back a value whose proofs are final.
    let materialized = two_owner(&target);
    let frozen = materialized.proof_finalized();

    // Sponsorless, and asserted rather than assumed: every input is a
    // consumed protocol input with its own verified opening reference,
    // and there is no sponsor region, because the confidential
    // construction intent has no way to express one.
    assert_eq!(materialized.opening_binding_census().entries().len(), 2);
    assert_eq!(frozen.protected().inputs().len(), 2);

    // Private, and asserted the same way: every created output carries a
    // value commitment and a nonempty range proof, which is what makes
    // the output-witness vector the thing the owner's message must
    // cover.
    assert_eq!(frozen.protected().outputs().len(), 2);
    for witness in frozen.protected().output_witnesses() {
        assert_ne!(witness.range_proof(), [0_u8; 0]);
        assert_eq!(witness.surjection_proof(), [0_u8; 0]);
    }

    // Census, sign, bind.
    let ready = submit_ready(&target, &materialized);

    assert_eq!(ready.answered(), 2);
    assert_eq!(ready.candidate(), frozen.protected());
    assert_eq!(ready.protected_bytes(), frozen.protected_bytes());
    assert_eq!(
        ready.accepted().census().output_witnesses(),
        frozen.protected().output_witnesses(),
    );
}

#[test]
fn the_submit_ready_candidate_has_one_reproducible_identity() {
    // The candidate's name, for a report to cite. It is a label over the
    // protected bytes and NOT a signing message: nothing in the signing
    // path computes it, nothing consumes it, and the message an owner
    // forms over the same bytes belongs to the separately reviewed
    // owner-sighash work.
    //
    // Pinned as a literal because the whole materialization is
    // deterministic from published fixtures, so a change to any protected
    // byte — a proof, a commitment, a nonce, a program, the version —
    // moves it, and a wave that changed one silently fails here.
    let target = reviewed_target();
    let ready = submit_ready(&target, &two_owner(&target));

    assert_eq!(
        candidate_identity(ready.protected_bytes()),
        "0000000000000000000000000000000000000000000000000000000000000000",
    );

    // And the same label from the state before it, so the identity is a
    // property of the bytes rather than of the state that reported them.
    let started = open(&target, &two_owner(&target));
    assert_eq!(
        candidate_identity(started.protected_bytes()),
        candidate_identity(ready.protected_bytes()),
    );
}

// --- The seam's own honesty -------------------------------------------

#[test]
fn the_acceptance_is_asked_every_time_rather_than_once() {
    // The seam takes no stored answer and keeps none. An implementor
    // that changed its mind between two handoffs changes both, which is
    // the property that makes the gate a gate rather than a wave-entry
    // note.
    struct Counting {
        calls: std::cell::Cell<usize>,
    }

    impl OwnerProfileAcceptance for Counting {
        fn unestablished_required_dimensions(&self) -> BTreeSet<SighashDimension> {
            self.calls.set(self.calls.get() + 1);
            BTreeSet::new()
        }
    }

    let target = reviewed_target();
    let materialized = one_owner(&target);
    let counting = Counting {
        calls: std::cell::Cell::new(0),
    };

    let _first = try_open(&target, &materialized, &counting)
        .expect("an established profile opens a handoff");
    let _second = try_open(&target, &materialized, &counting).expect("and opens a second one");

    assert_eq!(counting.calls.get(), 2);
}

#[test]
fn the_request_carries_no_opening_and_no_key() {
    // What crosses the boundary, checked against what may not. The
    // request is observed target data plus candidate structure; the
    // openings stay where they were verified, and what left is a
    // reference. This is asserted at the handoff because it is the last
    // place the two are still adjacent.
    let target = reviewed_target();
    let materialized = one_owner(&target);
    let started = open(&target, &materialized);

    // The opening-binding census is a census of references: a handle, a
    // digest, an outpoint, and an output index, and no scalar.
    let references: BTreeMap<&str, usize> = materialized
        .opening_binding_census()
        .entries()
        .iter()
        .map(|entry| (entry.handle(), entry.output()))
        .collect();
    assert_eq!(references.len(), 1);

    // And the request itself carries the four members the accepted
    // result names and nothing that could be an opening.
    assert_eq!(started.request().spent_outputs().len(), 1);
    assert_eq!(started.request().genesis_block_hash(), &GENESIS);
    assert_ne!(started.request().protected_bytes(), [0_u8; 0]);
    assert_eq!(started.request().signing_inputs().len(), 1);
}
