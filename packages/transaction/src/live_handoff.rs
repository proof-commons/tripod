//! The owner-sighash handoff: four states, four refusals, and no digest.
//!
//! # The digest implementation is not here and may not become here
//!
//! **This module designs no digest, computes no digest, and consumes no
//! digest.** The message an owner forms over the protected bytes is the
//! separately reviewed owner-sighash work's, entirely: its profile, its
//! dimension census, its stream, and its acceptance are that work's
//! output and this module's input. Nothing that crosses this boundary in
//! either direction is a digest — protected bytes and observed target
//! data go out, opaque authorization bytes and a hash-type byte come
//! back — and a later edit that computed a message here would be
//! importing the design this guide's own ruling puts outside it
//! (rule:guide-ctf-exec:external-sighash, rule:guide-ctf-exec:handoff-ownership).
//!
//! What this module owns is the ORDER, the BINDING, and the REFUSALS:
//! that a candidate is proof-finalized before any owner is asked, that
//! every answer is bound to one candidate by exact bytes, that every
//! required owner answered, and that nothing protected moved once the
//! asking started.
//!
//! # The output-witness commitment is tested elsewhere, on purpose
//!
//! The rule that makes this handoff's order mandatory is that the
//! target's `SIGHASH_ALL` hashes the output-witness vector at whatever
//! length the vector happens to have, so the proofs must be final before
//! any owner signs. That rule is the parallel sighash work's to
//! establish, and it is established there rather than re-tested here.
//! The evidence, by name:
//!
//! - `packages/vectors/tests/sighash_verdict.rs` recomputes the selected
//!   profile's disposition to established over its six-member required
//!   set, and checks that the required set did not grow to do it;
//! - `packages/vectors/tests/guide13_live_native.rs`, test
//!   `one_owner_authorization_is_observed_on_the_proof_bearing_lane`,
//!   carries the observed proof-bearing acceptance and the re-verification
//!   whose emptied-vector control fails where the real-length message
//!   succeeds;
//! - `packages/transaction/src/tests/live_census_tests.rs`, tests
//!   `the_message_moves_when_one_range_proof_byte_moves` and
//!   `the_message_is_the_stream_the_review_describes`, hold the message
//!   side of the same rule;
//! - `scripts/diagnose-taproot-output-witness-digest.py` is the runnable
//!   diagnosis behind `G11-W11-06` that all of the above rest on.
//!
//! Duplicating any of them here would produce a second opinion about a
//! rule this module does not own, and the second opinion would be the
//! one that drifted.
//!
//! # The mandatory order, as types
//!
//! ```text
//! ProofFinalizedCandidate -> SigningStarted -> FullyAuthorizedCandidate
//!                                           -> SubmitReadyPrivateCandidate
//! ```
//!
//! Each arrow is a type transition and not a flag, so a candidate cannot
//! be in two states and no field can disagree with a state
//! (def:guide-ctf-exec:handoff-states). [`SigningStarted`] is reached
//! only from a materialized candidate, which is reached only from the
//! materializer, so there is no route that asks an owner about a
//! candidate whose proofs are not final.
//!
//! # Submission is not here either
//!
//! [`SubmitReadyPrivateCandidate`] is where this module stops. It has no
//! method that submits, encodes for a wire, or produces a record,
//! because submission and the evidence it produces belong to the
//! restart wave (task:guide-ctf-exec:wave5) and a value that could
//! submit itself would let this wave's exit be claimed by running it.

use std::collections::BTreeSet;

use target_elements::{ReviewedElementsTapscriptDefinition, SighashDimension};

use crate::bytes::TargetTransaction;
use crate::live_accepted::{
    AcceptedOwnerAuthorizations, AcceptedResultRefusal, OfferedOwnerAuthorization,
};
use crate::live_census::{
    LiveDeployment, OwnerCensusRefusal, OwnerSigningCensus, OwnerSigningInputRequest,
};
use crate::live_materialize::{MaterializedConfidentialCandidate, ProofFinalizedRegion};
use crate::live_taproot::LiveCurveCapability;

// --- The acceptance the handoff reads ----------------------------------

/// What the separately reviewed owner-sighash work reports about its own
/// profile.
///
/// A trait defined here and implemented outside, on the pattern this
/// crate's other cryptographic collaborators already follow
/// (rule:guide-ctf-exec:dependency-directions): the profile, its
/// dimension roles, and its assessment live in the backend package this
/// crate deliberately does not depend on, and the answer reaches the
/// handoff as a value rather than as an edge.
///
/// # Why the answer is a set and not a boolean
///
/// Because the refusal has to be able to say WHICH required dimension is
/// unestablished. A boolean would make an incomplete review
/// indistinguishable from a review that reached everything except the
/// one dimension a candidate actually needs, and the handoff would be
/// refusing without being able to name what would repair it.
///
/// The empty set is the accepted state, and it is the RECOMPUTED empty
/// set: an implementor is expected to assess the selected profile
/// against the reviewed contract on every call, not to return a stored
/// verdict. Nothing here can enforce that, exactly as nothing can
/// enforce that a curve capability really does curve arithmetic, and the
/// obligation is stated where the implementor will read it.
pub trait OwnerProfileAcceptance {
    /// The required sighash dimensions the reviewed contract does not
    /// establish, in census order.
    ///
    /// Empty exactly when the selected profile's disposition recomputes
    /// to established.
    fn unestablished_required_dimensions(&self) -> BTreeSet<SighashDimension>;
}

// --- The refusals ------------------------------------------------------

/// Why a handoff did not reach its next state.
///
/// Construction refusals, every one, and none is a target verdict: a
/// handoff refused for one of these reasons never reached a node and
/// observed nothing about any deployment
/// (rule:guide-ctf-exec:failure-layers).
///
/// The first four members are the handoff states' own refusals, named by
/// the guide (def:guide-ctf-exec:handoff-states). The last two WRAP
/// the vocabularies this handoff calls into rather than extending them,
/// on the pattern [`AcceptedResultRefusal::Census`] already sets: the
/// census answers whether a candidate can be censused at all, and the
/// accepted result answers whether a set of answers covers that census
/// exactly once, and flattening either into the four would have made a
/// duplicate-answer complaint indistinguishable from a missing owner.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SighashHandoffRefusal {
    /// The selected owner-sighash profile's disposition does not
    /// recompute to established.
    ///
    /// The gate of task:guide-ctf-exec:wave4's entry condition, read
    /// at every handoff rather than once at a wave boundary. A profile
    /// that stopped being established between two runs is a profile no
    /// candidate may be handed to, and a stored answer could not have
    /// noticed.
    ProfileNotAccepted {
        /// The required dimensions the review does not establish, in
        /// census order.
        unestablished: BTreeSet<SighashDimension>,
    },
    /// An authorization is not about this candidate.
    ///
    /// Reached two ways, and both mean the same thing: the returning
    /// party censused something else. Either the protected bytes it
    /// offers back are not the ones it was handed — the exact-byte
    /// comparison `FinalizedLiveTransfer::check_offered` sets the
    /// pattern for, which is what catches a candidate whose proofs moved
    /// after finalization — or it answered for a signing input this
    /// census does not have.
    WrongCandidate,
    /// A signing input the census carries was not answered.
    ///
    /// Every required owner signs the same protected candidate, so a
    /// result short by one input is a partial authorization presented as
    /// a complete one.
    MissingOwner {
        /// The signing input left unanswered.
        input_index: u32,
    },
    /// A protected region would have moved after the asking started.
    ///
    /// From [`SigningStarted`] onward, proof repair, reblinding,
    /// regeneration, and any protected mutation are refusals rather than
    /// operations. The refusal names the region so a caller learns what
    /// it reached for, and so the census of regions can be walked by a
    /// test.
    MutationAfterSigningStarted {
        /// The region the operation would have touched.
        region: ProofFinalizedRegion,
    },
    /// The candidate could not be censused.
    ///
    /// The census's own vocabulary, carried whole. Nothing here is a
    /// handoff state: a candidate that cannot be censused never reached
    /// the first arrow.
    TheCandidateCouldNotBeCensused(OwnerCensusRefusal),
    /// The answers were not an accepted result, for a reason that is not
    /// a handoff state.
    ///
    /// A duplicate answer, a hash-type byte outside the profile, a
    /// signature of the wrong width, or a run against another
    /// deployment. Each is the accepted result's own complaint and keeps
    /// its own words.
    TheAnswersWereNotAnAcceptedResult(AcceptedResultRefusal),
}

impl From<OwnerCensusRefusal> for SighashHandoffRefusal {
    fn from(refusal: OwnerCensusRefusal) -> Self {
        Self::TheCandidateCouldNotBeCensused(refusal)
    }
}

// --- The states --------------------------------------------------------

/// The handoff request has been formed and owners may be asked.
///
/// Holds the census and nothing beside it, because the census IS the
/// request the accepted result names: protected bytes, the output-witness
/// vector at its consensus length, the spent-output census, the
/// deployment's genesis block hash, and one signing-input entry per owner
/// (rule:guide-ctf-exec:pending-sighash-result). A second type carrying
/// the same fields would be a second spelling of the boundary, and the
/// two spellings would be free to disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigningStarted {
    request: OwnerSigningCensus,
}

impl SigningStarted {
    /// Hands one proof-finalized candidate to the reviewed owner-sighash
    /// work.
    ///
    /// The order of rule:guide-ctf-exec:handoff-order is enforced by
    /// what this takes rather than by a check: the argument is a
    /// materialized candidate, which only the materializer produces and
    /// only after its own freeze, so proof finalization precedes the
    /// asking by construction.
    ///
    /// The acceptance is read FIRST, before the census is assembled. A
    /// candidate censused under an unestablished profile would have
    /// produced a well-formed request nobody may answer, and the work of
    /// building it would have made the refusal look like a late
    /// discovery rather than an entry condition.
    ///
    /// # Errors
    ///
    /// [`SighashHandoffRefusal::ProfileNotAccepted`] when the selected
    /// profile's disposition does not recompute to established, and
    /// [`SighashHandoffRefusal::TheCandidateCouldNotBeCensused`] at the
    /// first census clause the request fails.
    pub fn open(
        target: &ReviewedElementsTapscriptDefinition,
        materialized: &MaterializedConfidentialCandidate,
        deployment: LiveDeployment,
        requests: &[OwnerSigningInputRequest],
        curve: &dyn LiveCurveCapability,
        acceptance: &dyn OwnerProfileAcceptance,
    ) -> Result<Self, SighashHandoffRefusal> {
        let unestablished = acceptance.unestablished_required_dimensions();
        if !unestablished.is_empty() {
            return Err(SighashHandoffRefusal::ProfileNotAccepted { unestablished });
        }

        let request = OwnerSigningCensus::from_proof_finalized(
            target,
            materialized,
            deployment,
            requests,
            curve,
        )?;

        Ok(Self { request })
    }

    /// The request handed out, whole.
    ///
    /// The value the separately reviewed component reads. It is handed
    /// out by reference and never by a mutable one, so what an owner is
    /// asked about cannot change while it is being asked.
    #[must_use]
    pub const fn request(&self) -> &OwnerSigningCensus {
        &self.request
    }

    /// The exact bytes every owner is asked to bind to.
    ///
    /// The proof-finalized candidate's own preimage, unchanged. The
    /// handoff copies it and never recomputes it: a second computation
    /// of the same rule is a second chance for the two to differ.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        self.request.protected_bytes()
    }

    /// How many owners must answer.
    #[must_use]
    pub fn required_owners(&self) -> usize {
        self.request.signing_inputs().len()
    }

    /// Inserting into a protected region.
    ///
    /// # Errors
    ///
    /// Always [`SighashHandoffRefusal::MutationAfterSigningStarted`].
    /// The operation is spelled so that a caller reaching for it gets a
    /// refusal naming the region rather than a compile error naming
    /// nothing, and so that a test can walk the census — the pattern
    /// `ProofFinalizedCandidate` already sets one state earlier.
    pub const fn insert(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, SighashHandoffRefusal> {
        let _ = self;
        Err(SighashHandoffRefusal::MutationAfterSigningStarted { region })
    }

    /// Removing from a protected region.
    ///
    /// # Errors
    ///
    /// Always [`SighashHandoffRefusal::MutationAfterSigningStarted`].
    pub const fn remove(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, SighashHandoffRefusal> {
        let _ = self;
        Err(SighashHandoffRefusal::MutationAfterSigningStarted { region })
    }

    /// Replacing part of a protected region.
    ///
    /// # Errors
    ///
    /// Always [`SighashHandoffRefusal::MutationAfterSigningStarted`].
    pub const fn replace(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, SighashHandoffRefusal> {
        let _ = self;
        Err(SighashHandoffRefusal::MutationAfterSigningStarted { region })
    }

    /// Reordering a protected region.
    ///
    /// # Errors
    ///
    /// Always [`SighashHandoffRefusal::MutationAfterSigningStarted`].
    pub const fn reorder(
        &self,
        region: ProofFinalizedRegion,
    ) -> Result<Self, SighashHandoffRefusal> {
        let _ = self;
        Err(SighashHandoffRefusal::MutationAfterSigningStarted { region })
    }

    /// Regenerating the proofs.
    ///
    /// # Errors
    ///
    /// Always [`SighashHandoffRefusal::MutationAfterSigningStarted`]
    /// naming the output-witness region. This is the operation the
    /// mandatory order exists to forbid: a regeneration between the
    /// asking and the answering changes the vector the target hashes,
    /// and every answer already given becomes complete and invalid.
    pub const fn regenerate_proofs(&self) -> Result<Self, SighashHandoffRefusal> {
        let _ = self;
        Err(SighashHandoffRefusal::MutationAfterSigningStarted {
            region: ProofFinalizedRegion::OutputWitnesses,
        })
    }

    /// Repairing the proofs.
    ///
    /// # Errors
    ///
    /// Always [`SighashHandoffRefusal::MutationAfterSigningStarted`]
    /// naming the output-witness region.
    pub const fn repair_proofs(&self) -> Result<Self, SighashHandoffRefusal> {
        let _ = self;
        Err(SighashHandoffRefusal::MutationAfterSigningStarted {
            region: ProofFinalizedRegion::OutputWitnesses,
        })
    }

    /// Reblinding the outputs.
    ///
    /// # Errors
    ///
    /// Always [`SighashHandoffRefusal::MutationAfterSigningStarted`]
    /// naming the output region.
    pub const fn reblind(&self) -> Result<Self, SighashHandoffRefusal> {
        let _ = self;
        Err(SighashHandoffRefusal::MutationAfterSigningStarted {
            region: ProofFinalizedRegion::Outputs,
        })
    }

    /// Collects every required owner's answer over this one candidate.
    ///
    /// Consumes the state rather than borrowing it, which is what makes
    /// the transition a transition: a caller holding a
    /// [`FullyAuthorizedCandidate`] no longer holds the state that could
    /// have been asked again.
    ///
    /// The binding itself is [`AcceptedOwnerAuthorizations::bind`], not a
    /// re-implementation of it. What this adds is the translation into
    /// the handoff's own vocabulary, and the translation is total: the
    /// two ways of being about another candidate become
    /// [`SighashHandoffRefusal::WrongCandidate`], an unanswered signing
    /// input becomes [`SighashHandoffRefusal::MissingOwner`], and every
    /// other complaint keeps its own words.
    ///
    /// # Errors
    ///
    /// [`SighashHandoffRefusal::WrongCandidate`] when the offered bytes
    /// are not this candidate's or an answer names an input this census
    /// does not have; [`SighashHandoffRefusal::MissingOwner`] when a
    /// signing input was not answered; and
    /// [`SighashHandoffRefusal::TheAnswersWereNotAnAcceptedResult`] for
    /// a duplicate answer, a hash-type byte outside the profile, a
    /// signature of the wrong width, or another deployment's run.
    pub fn authorize(
        self,
        offered_protected_bytes: &[u8],
        run: LiveDeployment,
        offered: impl IntoIterator<Item = OfferedOwnerAuthorization>,
    ) -> Result<FullyAuthorizedCandidate, SighashHandoffRefusal> {
        match AcceptedOwnerAuthorizations::bind(self.request, offered_protected_bytes, run, offered)
        {
            Ok(accepted) => Ok(FullyAuthorizedCandidate { accepted }),
            Err(
                AcceptedResultRefusal::Census(
                    OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates,
                )
                | AcceptedResultRefusal::AuthorizationForAnInputTheCensusDoesNotHave { .. },
            ) => Err(SighashHandoffRefusal::WrongCandidate),
            Err(AcceptedResultRefusal::SigningInputWithNoAuthorization { input_index }) => {
                Err(SighashHandoffRefusal::MissingOwner { input_index })
            }
            Err(other) => Err(SighashHandoffRefusal::TheAnswersWereNotAnAcceptedResult(
                other,
            )),
        }
    }
}

/// Every required owner has answered over one candidate.
///
/// No mutation surface, and none is spelled: unlike [`SigningStarted`]
/// this state carries no candidate a caller could reach in a form that
/// could be edited — the accepted result takes its census by value and
/// hands out only immutable views — so a mutation here is unrepresentable
/// rather than refused. A refusing method would be a claim that there was
/// something to refuse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullyAuthorizedCandidate {
    accepted: AcceptedOwnerAuthorizations,
}

impl FullyAuthorizedCandidate {
    /// The accepted result, whole.
    #[must_use]
    pub const fn accepted(&self) -> &AcceptedOwnerAuthorizations {
        &self.accepted
    }

    /// The exact bytes every authorization was taken over.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        self.accepted.protected_bytes()
    }

    /// How many signing inputs are answered.
    #[must_use]
    pub fn answered(&self) -> usize {
        self.accepted.answered()
    }

    /// One signing input's opaque authorization.
    ///
    /// Opaque here as it is at the boundary it came from: nothing in
    /// this module parses one, and there is no accessor that interprets
    /// one.
    #[must_use]
    pub fn authorization(&self, input_index: u32) -> Option<&[u8]> {
        self.accepted.authorization(input_index)
    }

    /// Binds the authorized candidate to the materialization it came
    /// from, one last time, by exact bytes.
    ///
    /// The last arrow of rule:guide-ctf-exec:handoff-order reads
    /// "submit without changing any protected byte", and this is where
    /// that sentence is checked rather than trusted. The materialized
    /// candidate is re-read and its preimage compared with the one every
    /// owner bound to, so a protected byte that moved anywhere between
    /// finalization and submission is refused here — with the same
    /// refusal an answer about another candidate draws, because it is
    /// the same fault seen from the other side.
    ///
    /// # Errors
    ///
    /// [`SighashHandoffRefusal::WrongCandidate`] when the materialized
    /// candidate's protected bytes are not the ones the authorizations
    /// were taken over.
    pub fn bind_for_submission(
        self,
        materialized: &MaterializedConfidentialCandidate,
    ) -> Result<SubmitReadyPrivateCandidate, SighashHandoffRefusal> {
        let frozen = materialized.proof_finalized();
        if frozen.protected_bytes() != self.accepted.protected_bytes() {
            return Err(SighashHandoffRefusal::WrongCandidate);
        }

        Ok(SubmitReadyPrivateCandidate {
            candidate: frozen.protected().clone(),
            accepted: self.accepted,
        })
    }
}

/// One private candidate that is ready to be submitted, and is not
/// submitted.
///
/// The terminal state of this guide's handoff. It carries the frozen
/// transaction and the complete set of authorizations bound to it, and
/// it carries no method that submits, encodes for a wire, or produces a
/// record: submission and the evidence it produces are the restart
/// wave's (task:guide-ctf-exec:wave5), and a value that could submit
/// itself would let this wave's exit be claimed by running it rather
/// than by reaching this state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmitReadyPrivateCandidate {
    candidate: TargetTransaction,
    accepted: AcceptedOwnerAuthorizations,
}

impl SubmitReadyPrivateCandidate {
    /// The frozen transaction.
    #[must_use]
    pub const fn candidate(&self) -> &TargetTransaction {
        &self.candidate
    }

    /// The accepted result, whole.
    #[must_use]
    pub const fn accepted(&self) -> &AcceptedOwnerAuthorizations {
        &self.accepted
    }

    /// The exact protected bytes, unchanged since the freeze.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        self.accepted.protected_bytes()
    }

    /// How many signing inputs are answered.
    #[must_use]
    pub fn answered(&self) -> usize {
        self.accepted.answered()
    }

    /// One signing input's opaque authorization.
    #[must_use]
    pub fn authorization(&self, input_index: u32) -> Option<&[u8]> {
        self.accepted.authorization(input_index)
    }
}
