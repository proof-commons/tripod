//! The accepted result of the owner sighash work, in the shape the
//! consuming guide's handoff takes.
//!
//! # What this type is
//!
//! The accepted result is option B: protected bytes, the signing-input
//! census, and opaque authorization bytes plus the returned hash-type
//! byte, exact-byte bound to one candidate identity
//! (`rule:guide-sighash:result-decision`). The confidential-funding
//! execution guide's own pending decision on the accepted-result type
//! now cites that ruling rather than its own three options
//! (`rule:guide-ctf-exec:pending-sighash-result`).
//!
//! The out half already existed: [`OwnerSigningCensus`] carries the
//! protected bytes, the output-witness vector at its consensus length,
//! the spent-output census per input, the deployment's genesis block
//! hash, and per signing input the index, leaf hash, leaf version,
//! codeseparator position and annex disposition. This module is the back
//! half, and the binding between them.
//!
//! # Why the returned bytes are opaque and stay opaque
//!
//! Nothing here parses an authorization, and there is no accessor that
//! interprets one. The whole direction of the accepted result is that no
//! digest crosses in either direction: the census records what the target
//! reads, not what anyone computed, and the answer is a byte string whose
//! only checked properties are its width and the candidate it is bound
//! to. A module that read a signature out of it would be re-importing the
//! digest design into the boundary the ruling exists to keep thin.
//!
//! # Why the hash-type byte is checked and not stored
//!
//! The ruling's third constraint is that the returned byte is *checked*
//! against the profile, never merely recorded: a returned `0x01` under a
//! default-only profile is a refusal and not a variant, because the two
//! bytes produce different messages and a report accepting both would be
//! naming a profile it did not hold to.
//!
//! So [`AcceptedOwnerAuthorizations`] has no field for it. A stored byte
//! invites a later reader to consult it, and the only value that can ever
//! reach storage is the constant the profile fixes — at which point the
//! field is either dead or a place for a second opinion to live. The byte
//! crosses the boundary, it is compared, and it is dropped.
//!
//! # What this module does not do
//!
//! It does not establish the profile and it cannot. Whether the selected
//! profile's disposition is established is recomputed by
//! `tapscript::OwnerSighashProfile::assess` from the reviewed contract's
//! sighash capability, and nothing in this module is an input to that
//! recomputation.
//!
//! What the recomputation reports has moved, and it is worth stating
//! here precisely because this module did not move it. It reported the
//! review incomplete, naming the issuance dimension: no candidate this
//! arc builds bears an issuance, so the two message terms that carry the
//! dimension are formed from the input count alone and the observed
//! acceptance exercised nothing about any issuance field. The owner then
//! re-typed that dimension from required to refused; the required set
//! became six; every member of it was already established by the review;
//! and the recomputation reports the profile established.
//!
//! The consuming guide's `ProfileNotAccepted` is the refusal that reads
//! that recomputation, and it belongs to that guide's handoff rather than
//! to this type (`def:guide-ctf-exec:handoff-states`). So what changed
//! for a value of this type is nothing about the value: it was well
//! formed and exact-byte bound before and it is now. Handing one across
//! is still not entered here — that is this work's Wave 5, whose entry
//! condition needs the other guide's proof finalization besides, and a
//! profile that has stopped being the blocking half does not make the
//! remaining half smaller.
//!
//! It also owns no state machine. The consuming guide's
//! `MutationAfterSigningStarted` is about transitions between its own
//! candidate states, and nothing here can express a mutation because the
//! census this type binds to is taken by value and never handed back out
//! in a form that could be edited.

use std::collections::BTreeMap;

use crate::live_census::{
    LiveDeployment, OwnerCensusRefusal, OwnerSigningCensus, check_signature_width, check_type_byte,
};

/// One owner's answer to one signing request.
///
/// The hash-type byte travels with the answer rather than being assumed,
/// which is what makes the profile check a check. An owner that formed
/// the message under another type byte returns that byte and is refused
/// on it, instead of returning a signature whose disagreement with the
/// profile is discoverable only on chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedOwnerAuthorization {
    input_index: u32,
    authorization: Vec<u8>,
    type_byte: u8,
}

impl OfferedOwnerAuthorization {
    /// States one returned answer.
    #[must_use]
    pub const fn new(input_index: u32, authorization: Vec<u8>, type_byte: u8) -> Self {
        Self {
            input_index,
            authorization,
            type_byte,
        }
    }

    /// Which signing input the answer is for.
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.input_index
    }

    /// The opaque authorization bytes.
    #[must_use]
    pub fn authorization(&self) -> &[u8] {
        &self.authorization
    }

    /// The hash-type byte the owner formed the message under.
    #[must_use]
    pub const fn type_byte(&self) -> u8 {
        self.type_byte
    }
}

/// Why a set of returned answers is not an accepted result.
///
/// Construction refusals, every one. No member of this type is a target
/// verdict: a refusal here means the answers never reached a node, and a
/// run that stopped for one of these reasons observed nothing about any
/// deployment.
///
/// The census's own vocabulary is wrapped rather than extended, because
/// the two ask different questions. [`OwnerCensusRefusal`] is about
/// whether a candidate can be censused and whether an answer agrees with
/// the profile; the three members below are about whether a *set* of
/// answers covers the census exactly once. Merging them would have made
/// a cardinality complaint indistinguishable from a profile complaint.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AcceptedResultRefusal {
    /// The census, the deployment, or one answer's own shape was
    /// refused.
    Census(OwnerCensusRefusal),
    /// An answer names an input the census has no signing request for.
    ///
    /// Not a missing owner but a surplus one, and the distinction is
    /// worth a member: a surplus answer means the returning party
    /// censused a different candidate, which is a stronger fault than
    /// having failed to answer.
    AuthorizationForAnInputTheCensusDoesNotHave {
        /// The index the answer named.
        input_index: u32,
    },
    /// Two answers name the same signing input.
    ///
    /// Refused rather than resolved. A later answer silently replacing
    /// an earlier one would make which authorization was bound depend on
    /// iteration order, and the census is bound exactly once or not at
    /// all.
    TwoAuthorizationsForOneInput {
        /// The index answered twice.
        input_index: u32,
    },
    /// A signing input the census carries has no answer.
    ///
    /// The consuming guide's `MissingOwner`, recomputed on this side
    /// from the census rather than trusted: every required owner signs
    /// the same protected candidate, and a result short by one input is
    /// a partial authorization presented as a complete one.
    SigningInputWithNoAuthorization {
        /// The index left unanswered.
        input_index: u32,
    },
}

impl From<OwnerCensusRefusal> for AcceptedResultRefusal {
    fn from(refusal: OwnerCensusRefusal) -> Self {
        Self::Census(refusal)
    }
}

/// The accepted result: one census, bound to one complete set of
/// authorizations.
///
/// No public constructor but [`Self::bind`], for the reason the census
/// itself has none: a value assembled field by field would be a claim
/// that the checks were run, and nothing downstream could tell that
/// claim apart from a value that had run them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedOwnerAuthorizations {
    census: OwnerSigningCensus,
    authorizations: BTreeMap<u32, Vec<u8>>,
}

impl AcceptedOwnerAuthorizations {
    /// Binds a complete set of answers to one candidate.
    ///
    /// The offered protected bytes are the returning party's own copy of
    /// what it signed over, compared byte for byte against the census's,
    /// in the pattern `FinalizedLiveTransfer::check_offered` sets — the
    /// consuming guide's `WrongCandidate`, recomputed here rather than
    /// trusted. An authorization for a candidate whose output-witness
    /// vector changed is refused before anything else is looked at.
    ///
    /// # Errors
    ///
    /// [`AcceptedResultRefusal::Census`] when the offered bytes are not
    /// the census's, when the run is another deployment's, when a
    /// returned byte is outside the profile, or when a returned
    /// authorization is the wrong width.
    ///
    /// [`AcceptedResultRefusal::AuthorizationForAnInputTheCensusDoesNotHave`],
    /// [`AcceptedResultRefusal::TwoAuthorizationsForOneInput`], and
    /// [`AcceptedResultRefusal::SigningInputWithNoAuthorization`] when
    /// the answers do not cover the census's signing inputs exactly
    /// once.
    pub fn bind(
        census: OwnerSigningCensus,
        offered_protected_bytes: &[u8],
        run: LiveDeployment,
        offered: impl IntoIterator<Item = OfferedOwnerAuthorization>,
    ) -> Result<Self, AcceptedResultRefusal> {
        census.check_offered(offered_protected_bytes)?;
        census.check_deployment(run)?;

        let mut authorizations: BTreeMap<u32, Vec<u8>> = BTreeMap::new();

        for answer in offered {
            let index = answer.input_index();

            if !census
                .signing_inputs()
                .iter()
                .any(|input| input.input_index() == index)
            {
                return Err(
                    AcceptedResultRefusal::AuthorizationForAnInputTheCensusDoesNotHave {
                        input_index: index,
                    },
                );
            }

            // Both profile checks, on every answer. The width is checked
            // as well as the byte because the two faults are different
            // and the target reports them differently: a sixty-five byte
            // witness carries a type byte the target reads, and a
            // sixty-four byte one carries none.
            check_type_byte(answer.type_byte())?;
            check_signature_width(answer.authorization().len())?;

            if authorizations
                .insert(index, answer.authorization().to_vec())
                .is_some()
            {
                return Err(AcceptedResultRefusal::TwoAuthorizationsForOneInput {
                    input_index: index,
                });
            }
        }

        for input in census.signing_inputs() {
            if !authorizations.contains_key(&input.input_index()) {
                return Err(AcceptedResultRefusal::SigningInputWithNoAuthorization {
                    input_index: input.input_index(),
                });
            }
        }

        Ok(Self {
            census,
            authorizations,
        })
    }

    /// The census the authorizations are bound to.
    #[must_use]
    pub const fn census(&self) -> &OwnerSigningCensus {
        &self.census
    }

    /// The exact bytes every authorization was taken over.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        self.census.protected_bytes()
    }

    /// One signing input's opaque authorization.
    ///
    /// `None` only for an index the census does not carry: a bound
    /// result answers every signing input it has, which is what
    /// [`Self::bind`] refuses to produce otherwise.
    #[must_use]
    pub fn authorization(&self, input_index: u32) -> Option<&[u8]> {
        self.authorizations
            .get(&input_index)
            .map(std::vec::Vec::as_slice)
    }

    /// How many signing inputs this result answers.
    #[must_use]
    pub fn answered(&self) -> usize {
        self.authorizations.len()
    }
}
