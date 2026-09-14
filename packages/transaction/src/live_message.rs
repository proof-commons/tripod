//! An independently written construction of the target's owner message.
//!
//! # Where this construction's expectation comes from
//!
//! Every term below was written from the Wave-1 source review's own term
//! table (tab:sighash-review:terms in
//! `plans/reference/owner-sighash-review.md`) at the pinned Elements tip
//! `b7fc5d080a`, and each is cited at the line the review cites. Nothing
//! here was derived from the code that produces the census it consumes,
//! and nothing here consults a target.
//!
//! That separation is the whole value of the module, and the arc's rule
//! says why: one computation is never both observation and expectation.
//! A message construction whose expectation came from the same code path
//! as the value it checks would agree with itself and report the
//! agreement as evidence.
//!
//! # What is reused, and why that is not the same code path
//!
//! Two things are deliberately taken from this crate rather than
//! respelled here: the tagged-hash primitive, and the *serialization* of
//! an output, an output witness, and an asset or value field.
//!
//! The thing under check is the message — which terms are written, in
//! what order, over what. The serialization of a target output is a
//! different claim, evidenced elsewhere, and this crate exists to
//! produce exactly those bytes. Respelling it here would create a second
//! opinion about output encoding that is free to drift from the bytes
//! the crate actually emits, which would make a disagreement between
//! them look like a message finding when it was an encoding typo. So the
//! encodings are shared and the *stream* is independent, and this
//! paragraph is the boundary stated rather than left to be inferred.
//!
//! # What this module does not claim
//!
//! No digest here is authoritative. The wave that wrote this module
//! produced a candidate recomputation and nothing else:
//! `SighashProfileUnreviewed` stood, `OwnerSighashNotComputable` stood,
//! no dimension moved to reviewed, and no target had been asked what it
//! thought of any value this module returns. A recomputation matching a
//! target's digest establishes that the model is right about the
//! message, not that a node accepts a spend built from it.
//!
//! Both of those residuals have since been cleared, and neither by
//! anything in this module — the digest blocker by a later wave's
//! observed acceptance, and the profile residual by the review verdict
//! and the re-typing. What is unchanged is the non-claim
//! itself: this module still asserts no digest, and a value it returns
//! is still a first-party recomputation rather than a target's answer.

use crate::live_census::{OwnerSigningCensus, OwnerSigningInputCensus};
use crate::script_path_signing;
use crate::taproot::Digest32;

pub use crate::script_path_signing::{
    CandidateMessagePair, KEY_PATH_SPEND_TYPE_BYTE, TAP_SIGHASH_TAG, WitnessVectorTreatment,
};

/// The message the target forms for one signing input of one census.
///
/// Total rather than fallible: a signing input obtained from a census is
/// one the census already validated, so there is no clause left for this
/// function to fail. The refusals live where the census is assembled,
/// which is the only place a caller can supply something wrong.
#[must_use]
pub fn candidate_owner_message(
    census: &OwnerSigningCensus,
    input: &OwnerSigningInputCensus,
    treatment: WitnessVectorTreatment,
) -> Digest32 {
    script_path_signing::script_path_message(census.kernel(), input, treatment)
}

/// A candidate message for a KEY-PATH spend of the same candidate.
///
/// # This is not a reviewed construction, and the distinction matters
///
/// [`candidate_owner_message`] is written from the Wave-1 source
/// review's term table, term by term, at the lines the review cites. No
/// review in this repository covers the key path, because no constructor
/// here is meant to be spent by one. So this function is the script-path
/// stream with its tapscript tail replaced by what the source's own
/// composition rule says a key-path spend writes, and it is
/// candidate-scoped in the strict sense: nothing has observed that a
/// target forms this message, and nothing here claims it does.
///
/// It shares its whole-transaction prefix with the reviewed
/// construction — one private function writes terms 0 to 12 for both —
/// rather than respelling twelve terms, so the two messages differ in
/// their tail and in nothing else. A caller comparing
/// them is comparing the spend type and the tapscript additions, which
/// is the only comparison this function supports.
///
/// # What it is for
///
/// One probe, which offers a key-path witness to a target and records
/// what the target says. The probe's finding is the target's answer; the
/// message this function returns is a datum about what was submitted,
/// not a claim about what was verified against.
#[must_use]
pub fn candidate_key_path_message(
    census: &OwnerSigningCensus,
    input_index: u32,
    treatment: WitnessVectorTreatment,
) -> Digest32 {
    script_path_signing::key_path_message(census.kernel(), input_index, treatment)
}

/// The recorded diagnosis's two candidate messages for one signing
/// input.
///
/// The output side varies and the input side is held at its consensus
/// length in both, which is the recorded diagnosis's own shape: it
/// deep-copies the transaction, clears `vtxoutwit`, and hashes again,
/// leaving `vtxinwit` alone.
#[must_use]
pub fn candidate_message_pair(
    census: &OwnerSigningCensus,
    input: &OwnerSigningInputCensus,
) -> CandidateMessagePair {
    script_path_signing::message_pair(census.kernel(), input)
}
