//! Signing the maturity announcement: §12.7's last three states, the
//! response §13.3 admits, and the faults §13.4 names.
//!
//! # The chain, and where each arrow lives
//!
//! ```text
//! MaturityConstruction                     state_construct
//!     ↓ finalize_maturity_announcement     state_finalize
//! FinalizedMaturityAnnouncement
//!     ↓ OperatorSigningStarted::open       here
//! OperatorSigningStarted
//!     ↓ OperatorSigningStarted::authorize  here
//! OperatorAuthorizedMaturityAnnouncement
//!     ↓ bind_for_submission                here
//! SubmitReadyMaturityAnnouncement
//! ```
//!
//! Each arrow is a type transition and not a flag, so a candidate cannot
//! be in two states and no field can disagree with one. The chain is
//! STATE-specific and shaped on the live generation's rather than an
//! extension of it: the live types carry owner plurality and
//! confidential-proof operations that have no STATE subject here — one
//! signer, one input, no proofs, no owner census — and an extension
//! would have had to represent the absence of all of them.
//!
//! # Every state borrows one finalized candidate
//!
//! A signing request borrows the operator binding that lives inside the
//! bundle inside the finalized value, so a state holding a request
//! either borrows that value or freezes a second request over the same
//! candidate. Two requests would give the freeze two places to disagree,
//! so all three states carry the borrow. It also makes the last state's
//! standing structural: a submit-ready value cannot outlive the
//! candidate it is a state of, which is the difference between a state
//! and a promotion.
//!
//! # Mutation after signing started is a refusal, not an impossibility
//!
//! §12.7 protects twelve regions and asks for a typed refusal wherever
//! protected mutation is representable. [`MaturityProtectedRegion`] is
//! that census in the guide's own order, and [`OperatorSigningStarted`]
//! spells one operation per member so that a caller reaching for any of
//! them gets a refusal naming the region rather than a compile error
//! naming nothing. The distinction is the point: a mutation that is
//! unrepresentable cannot be *shown* to reject, and a census that can
//! only be read in prose stops being evidence the first time a member
//! arrives without an entry. This census can be walked by a test.
//!
//! It is neither of the two censuses already in the crate. The live
//! freeze's is eight members and carries confidential nonce fields,
//! which have no STATE subject; the finalized form's is eleven and
//! answers a different question — what a finalized value fixed, which
//! excludes the executing leaf data because these bytes do not carry it.
//!
//! # The four post-signing mutations are named
//!
//! §13.4 lists seventeen faults. Thirteen belong to the operator
//! boundary and keep its words, carried whole rather than flattened.
//! The remaining four — the successor metadata, the successor program, a
//! sponsor input and the fee role — are named by
//! [`OperatorAuthorizedMaturityAnnouncement::check_offered`] before the
//! finalized form's own comparison runs. They are named rather than left
//! to the exact echo because an echo that disagrees is evidence that
//! *something* moved, not that a named mutation rejects, and a reader
//! checking a refusal against §13.4's list would find nothing there to
//! check it against. The operator module's statement that those four
//! need no separate variants stands for its own synthetic scope, where
//! the echo is the whole question and no typed successor exists to
//! compare.
//!
//! # The witness, item by item
//!
//! [`OperatorAuthorizedMaturityAnnouncement::bind_for_submission`] walks
//! the ABI's own witness records rather than a second list of roles, so
//! the stack is in the record's deepest-first order by construction, and
//! appends the executing leaf's script and control block:
//!
//! ```text
//! 0  successor output-key prefix   1 byte    successor constructor's parity
//! 1  successor nonce               4 bytes   successor constructor's nonce
//! 2  requested cycle               8 bytes   the typed request
//! 3  static subtree root          32 bytes   the linked bundle's subtree
//! 4  predecessor metadata         86/53 bytes whole/variable schedule
//! 5  predecessor output-key prefix 1 byte    retained predecessor's parity
//! 6  operator signature           64 bytes   the authorized boundary witness
//! 7  leaf script                             the predecessor's committed leaf
//! 8  control block                           that leaf's authentication
//! ```
//! The variable schedule carries the metadata's changing region; its leaf
//! restores the canonical encoding before authenticating it.
//!
//! The two prefix bytes are compressed-key prefixes and neither is the
//! control block's first byte, which packs the same parity bit with a
//! leaf version and is a different encoding of it. The leaf says which
//! it wants: it concatenates the witness byte with the thirty-two-byte
//! coordinate it introspects and takes the thirty-three-byte result,
//! which is the reviewed compressed-key form, whose two prefixes select
//! the even and the odd point. Neither byte comes from the view: the
//! view states the predecessor program as the target does, an x-only key
//! with no prefix in it and no parity beside it, so the parity is read
//! from the constructor whose commitment produced the key. The successor
//! prefix is the successor constructor's and the predecessor prefix is
//! the retained predecessor constructor's — two different constructors,
//! and taking either from the other would state one point's parity about
//! the other.
//!
//! No width refusal exists here, and the omission is deliberate. Each of
//! the seven items is fixed-width by the encoding it is read from: a
//! parity is one byte, the two integers are four and eight big-endian
//! bytes, a static root is thirty-two, the metadata codec's canonical
//! width is exact, and the signature is sixty-four because the operator
//! boundary refused every other width before an authorized candidate
//! existed. The record's declared widths are those same seven figures
//! read off the composed program. A refusal for their disagreement could
//! therefore never be reached, and an unreachable variant is what the
//! crate's reachability convention exists to prevent; the agreement is
//! held by tests against each record's declared minimum and maximum
//! instead.
//!
//! # Authorization goes only through the construction right
//!
//! [`OperatorSigningStarted::authorize`] calls
//! [`authorize_operator_under_right`] and nothing else. The unmediated
//! entry point exists for the boundary's own tests and says so; the
//! route with a registry behind it is the one that records what was
//! issued and signed, and a second route into this chain would leave a
//! signature the registry never observed. A refused authorization
//! returns its token, because the registry's contract is that a refusal
//! grants no new authority and takes none away: the entry stays
//! outstanding under a recorded refusal event precisely so that a scope
//! refused once can be consumed again, and a signature that dropped the
//! token would let one malformed response lock a scope for good. The
//! started state is consumed as the registry consumes the request, so a
//! caller re-opens from the finalized value — deterministic, since the
//! same candidate freezes the same bytes under the same scope, and the
//! returned token still fits.
//!
//! # Submit-ready is a state
//!
//! [`SubmitReadyMaturityAnnouncement`] carries a single-variant status
//! that is read and never written, no digest and no field one could be
//! put in, and no method that submits, encodes for a wire or produces a
//! record. Its bytes are the finalized bytes with the one witness
//! inserted, which is the only thing §12.7 admits moving after
//! finalization, and its protected bytes are read off the finalized
//! value rather than recomputed. Nothing here is final: the candidate
//! stays lifecycle-incomplete while the later STATE operations are
//! absent, and a value that could submit itself would let the state be
//! claimed by running it rather than by reaching it.

use realization::{
    STATE_METADATA_BYTES, StateMetadata, StateRepresentationNonce, encode_state_metadata,
    state_metadata_variable_region,
};
use tapscript::{CandidateStateConstructor, StateProgramWitness, StateWitnessSchedule};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::bytes::{InputWitness, OutputWitness, TargetInput, TargetOutput, TargetTransaction};
use crate::error::TransactionRefusal;
use crate::live_taproot::LiveCurveCapability;
use crate::operator_right::{
    ConstructionRight, OperatorRightOutcome, OperatorRightRegistry, RightFailure, RightRefusal,
    RightScope,
};
use crate::operator_signing::{
    OperatorEvidenceStanding, OperatorSigningRequest, OperatorSigningResponse,
    ScriptPathSignatureVerifier, authorize_operator_under_right,
};
use crate::script_path_signing::SpentOutputCensusEntry;
use crate::state_abi::MaturityWitnessRole;
use crate::state_finalize::{FinalizedMaturityAnnouncement, MaturityExecutingLeaf};

/// The compressed-key prefix that states an even output key.
///
/// The reviewed target's compressed-key encoding selects its two forms
/// by these two prefixes, the lower naming the even point. Held as named
/// constants rather than computed, and held against the reviewed
/// registry's own prefix set by a test, so the pair is checked against
/// the target rather than asserted here.
pub(crate) const EVEN_OUTPUT_KEY_PREFIX: u8 = 0x02;

/// The compressed-key prefix that states an odd output key.
pub(crate) const ODD_OUTPUT_KEY_PREFIX: u8 = 0x03;

/// The prefix byte one commitment's parity is stated with.
const fn output_key_prefix(odd: bool) -> u8 {
    if odd {
        ODD_OUTPUT_KEY_PREFIX
    } else {
        EVEN_OUTPUT_KEY_PREFIX
    }
}

/// One region §12.7 protects from the moment signing starts.
///
/// Twelve members in the guide's own order. The census is a value rather
/// than a paragraph so that a refusal can name its member and a test can
/// walk the list; a census whose membership lived in prose would stop
/// being evidence the first time a member arrived without an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturityProtectedRegion {
    /// The input census.
    Inputs,
    /// The spent-output census bound for the message.
    SpentOutputCensus,
    /// The output census, every field of it.
    Outputs,
    /// The successor's semantic metadata.
    SuccessorMetadata,
    /// The successor's canonical representation nonce.
    SuccessorRepresentationNonce,
    /// The successor's output program.
    SuccessorProgram,
    /// The output-witness vector.
    OutputWitnesses,
    /// The version field.
    Version,
    /// The lock-time field.
    LockTime,
    /// The sponsor region, present or absent.
    SponsorRegion,
    /// The fee region, present or absent.
    FeeRegion,
    /// The executing leaf, its version, its script and its control
    /// block.
    ExecutingLeafData,
}

impl MaturityProtectedRegion {
    /// The complete census, in §12.7's own order.
    pub const ALL: &'static [Self] = &[
        Self::Inputs,
        Self::SpentOutputCensus,
        Self::Outputs,
        Self::SuccessorMetadata,
        Self::SuccessorRepresentationNonce,
        Self::SuccessorProgram,
        Self::OutputWitnesses,
        Self::Version,
        Self::LockTime,
        Self::SponsorRegion,
        Self::FeeRegion,
        Self::ExecutingLeafData,
    ];
}

/// The status vocabulary a bound candidate is distinguished by.
///
/// One variant, read and never written. A second variant would make the
/// type a lifecycle field and the state a flag; a setter would make
/// reaching the state something a caller could assert rather than
/// something a caller could do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturitySubmissionStatus {
    /// The candidate is complete and is not submitted.
    SubmitReady,
}

/// A refused authorization, with the token it did not consume.
///
/// [`Debug`] alone: the token forbids duplication, so a failure carrying
/// one cannot be cloned or compared, which is the registry's own failure
/// type's shape and for the registry's own reason. A refusal grants no
/// new authority and takes none away, so the scope stays outstanding and
/// the caller keeps what it needs to try again.
#[derive(Debug)]
pub struct MaturityAuthorizationFailure {
    /// Why the authorization produced no accepted artifact.
    pub refusal: TransactionRefusal,
    /// The original token, still outstanding in its registry.
    pub right: ConstructionRight,
}

/// The signing request is frozen and the operator may be asked.
///
/// Holds the finalized candidate it was frozen from and the request
/// itself, and nothing beside them: every fact a caller could want is a
/// read off one of the two, and a third field would be a second place
/// for one of those facts to be stated.
#[derive(Debug, PartialEq, Eq)]
pub struct OperatorSigningStarted<'finalized> {
    finalized: &'finalized FinalizedMaturityAnnouncement,
    request: OperatorSigningRequest<'finalized>,
}

impl<'finalized> OperatorSigningStarted<'finalized> {
    /// Freezes the operator signing request over one finalized
    /// candidate.
    ///
    /// The request is frozen exactly once, here, through the
    /// finalization's own request builder, so all nine of §13.2's terms
    /// derive from this one value and the refusal the freeze raises
    /// arrives in the wrapper that layer already owns.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MaturityAnnouncementSigningRequestRefused`]
    /// carrying whatever the freeze refused, whole: the binding's
    /// revision, the deployment genesis, the curve's verdict on the
    /// committed operator key, the census clauses over the candidate and
    /// its spent output, and a leaf that does not hash to the selection
    /// are distinct findings and the boundary that performs the freeze
    /// has the words for all of them.
    ///
    /// # Panics
    ///
    /// Everything the finalized form's request builder panics on, for
    /// its reasons: a bundle retaining no constructor application, a
    /// control recipe the predecessor constructor refuses, and a
    /// committed leaf hash the subtree that produced it does not carry.
    /// A linked bundle can arrange none of the three.
    pub fn open(
        finalized: &'finalized FinalizedMaturityAnnouncement,
        target: &ReviewedElementsTapscriptDefinition,
        curve: &dyn LiveCurveCapability,
    ) -> Result<Self, TransactionRefusal> {
        let request = finalized.signing_request(target, curve)?;
        Ok(Self { finalized, request })
    }

    /// The frozen request, handed out by reference and never by a
    /// mutable one, so what the operator is asked about cannot change
    /// while it is being asked.
    #[must_use]
    pub const fn request(&self) -> &OperatorSigningRequest<'finalized> {
        &self.request
    }

    /// The finalized candidate this was frozen from.
    #[must_use]
    pub const fn finalized(&self) -> &'finalized FinalizedMaturityAnnouncement {
        self.finalized
    }

    /// The exact bytes the operator is asked to bind to.
    ///
    /// The request's own frozen serialization, which is the finalized
    /// form's protected encoding unchanged. Read from the request rather
    /// than recomputed: a second computation of one rule is a second
    /// chance for the two to differ.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        self.request.frozen_bytes()
    }

    /// The scope a construction right over exactly this candidate is
    /// issued against.
    ///
    /// Every term is a fact this one value already carries: the
    /// deployment identity and the operator's public bytes are the
    /// binding's, the predecessor is the frozen input's outpoint, the
    /// capability revision is the one the binding was minted against,
    /// and the branch is the view's own current-root binding. A caller
    /// that had to assemble any of them beside the candidate could issue
    /// a right against something else.
    ///
    /// # Panics
    ///
    /// Panics only if the frozen request has no predecessor at the input
    /// it selected, which this chain cannot arrange: the announcement
    /// form has exactly one input, the request is frozen over input
    /// zero, and a request that froze at all carries that input.
    #[must_use]
    pub fn construction_right_scope(&self) -> RightScope {
        let branch = self
            .finalized
            .construction()
            .validated_view()
            .view()
            .current_root_binding();
        RightScope::new(&self.request, branch)
            .expect("a request frozen over the one STATE input carries that input's outpoint")
    }

    /// Extending the input census.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the input region. The operation is spelled so that a
    /// caller reaching for it gets a refusal naming what it reached for,
    /// and so the census can be walked rather than believed.
    pub const fn add_input(&self, offered: &TargetInput) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::Inputs,
        })
    }

    /// Restating the spent-output census.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the spent-output census, which the operator message is
    /// taken over and which a restatement would therefore move.
    pub const fn restate_spent_output(
        &self,
        offered: &SpentOutputCensusEntry,
    ) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::SpentOutputCensus,
        })
    }

    /// Extending the output census.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the output region.
    pub const fn add_output(&self, offered: &TargetOutput) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::Outputs,
        })
    }

    /// Restating the successor's semantic metadata.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the successor metadata, which the successor's own
    /// commitment is taken over.
    pub const fn set_successor_metadata(
        &self,
        offered: StateMetadata,
    ) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::SuccessorMetadata,
        })
    }

    /// Naming the successor's representation nonce.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the representation nonce. Unrepresentable one state
    /// earlier, where the nonce is the search's and a construction has
    /// no field for one; representable here, because a caller holding a
    /// started state holds a nonce it could offer.
    pub const fn set_successor_nonce(
        &self,
        offered: StateRepresentationNonce,
    ) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::SuccessorRepresentationNonce,
        })
    }

    /// Naming the successor's output program.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the successor program.
    pub const fn set_successor_program(&self, offered: &[u8]) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::SuccessorProgram,
        })
    }

    /// Writing an output witness.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the output-witness region. This is the operation the
    /// mandatory order exists to forbid in the confidential generation,
    /// and it is forbidden here for the same rule read over an empty
    /// vector: the message commits to the vector at whatever length it
    /// has, so a witness written after the asking started changes what
    /// was asked.
    pub const fn set_output_witness(
        &self,
        offered: &OutputWitness,
    ) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::OutputWitnesses,
        })
    }

    /// Restating the transaction version.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the version, which §12.7 censuses as a region of its own.
    pub const fn set_version(&self, offered: u32) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::Version,
        })
    }

    /// Restating the lock time.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the lock time, a region of its own for the version's
    /// reason.
    pub const fn set_lock_time(&self, offered: u32) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::LockTime,
        })
    }

    /// Opening a sponsor region.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the sponsor region. The finalized form fixed it as absent,
    /// and absent is a settled state rather than a vacancy: a region
    /// opened after the asking started is the sponsor-input fault §13.4
    /// names, seen one state earlier.
    pub const fn add_sponsor_input(
        &self,
        offered: &TargetInput,
    ) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::SponsorRegion,
        })
    }

    /// Placing a fee role.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the fee region, fixed as absent for the sponsor region's
    /// reason: the spent output and the successor carry the same asset
    /// and amount, so the balance has no difference for a fee to make
    /// up.
    pub const fn set_fee_role(&self, offered: &TargetOutput) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::FeeRegion,
        })
    }

    /// Substituting the executing leaf.
    ///
    /// # Errors
    ///
    /// Always [`TransactionRefusal::MaturityMutationAfterSigningStarted`]
    /// naming the executing leaf data. §13.2 forbids substituting
    /// another committed leaf of the same tree, and this is where that
    /// sentence is a refusal a caller can reach rather than a sentence:
    /// the leaf, its version, its script and its control block travel
    /// together into the frozen request, and any of them moving moves
    /// the message the operator was asked about.
    pub const fn substitute_executing_leaf(
        &self,
        offered: &MaturityExecutingLeaf,
    ) -> Result<Self, TransactionRefusal> {
        let _ = (self, offered);
        Err(TransactionRefusal::MaturityMutationAfterSigningStarted {
            region: MaturityProtectedRegion::ExecutingLeafData,
        })
    }

    /// Collects the operator's answer over this one candidate, under a
    /// construction right.
    ///
    /// Consumes the state rather than borrowing it, which is what makes
    /// the transition a transition: a caller holding the authorized
    /// state no longer holds the state that could have been asked again.
    /// The registry is the only route: it observes what was issued and
    /// signed, and an authorization taken beside it would be a signature
    /// the record does not carry.
    ///
    /// A refusal returns the token. The registry leaves a refused scope
    /// outstanding under a recorded refusal event, so re-opening from
    /// the finalized value and offering the same token is the retry the
    /// registry's own contract describes; the re-opened request freezes
    /// the same bytes under the same scope, so the token still fits.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MaturityOperatorAuthorizationRefused`]
    /// carrying the operator boundary's own finding — a missing,
    /// duplicate, unexpected or misdirected response, another operator,
    /// deployment or profile, a type byte outside the selection, an
    /// empty or misshapen signature, an echo of other bytes, or a
    /// signature that does not verify for the frozen message — and
    /// [`TransactionRefusal::MaturityConstructionRightRefused`] for
    /// everything the registry itself refuses about the token, the scope
    /// or the bytes it was issued over. Both arrive inside a
    /// [`MaturityAuthorizationFailure`] that also returns the token.
    pub fn authorize(
        self,
        registry: &mut OperatorRightRegistry,
        right: ConstructionRight,
        responses: impl IntoIterator<Item = OperatorSigningResponse>,
        verifier: &dyn ScriptPathSignatureVerifier,
    ) -> Result<OperatorAuthorizedMaturityAnnouncement<'finalized>, Box<MaturityAuthorizationFailure>>
    {
        let Self { finalized, request } = self;
        match authorize_operator_under_right(registry, right, request, responses, verifier) {
            // The fresh outcome is the first accepted signing action,
            // and it is the only outcome this route produces: the
            // registry's consumption always invokes the signer, and the
            // cached artifact is read by a retry this chain does not
            // call. The arm below is written to be right rather than to
            // be reached, and it is right because the cache carries
            // exactly the two values this state holds — the witness the
            // boundary returned and the standing its verifier gave it —
            // with everything else read off the finalized candidate.
            Ok(OperatorRightOutcome::Fresh(authorized)) => {
                Ok(OperatorAuthorizedMaturityAnnouncement {
                    finalized,
                    witness: authorized.witness().clone(),
                    standing: authorized.standing().clone(),
                })
            }
            Ok(OperatorRightOutcome::Cached(cached)) => {
                Ok(OperatorAuthorizedMaturityAnnouncement {
                    finalized,
                    witness: cached.witness,
                    standing: cached.standing,
                })
            }
            Err(failure) => {
                let RightFailure { refusal, right } = *failure;
                Err(Box::new(MaturityAuthorizationFailure {
                    refusal: translate(refusal),
                    right,
                }))
            }
        }
    }
}

/// The registry's refusal in this crate's own vocabulary.
///
/// The signing arm keeps the operator boundary's words, whole, because
/// which response clause failed is a finding only that layer has the
/// vocabulary for. Everything else is the registry's own statement about
/// the token, the scope or the bytes, and it travels whole for the same
/// reason.
fn translate(refusal: RightRefusal) -> TransactionRefusal {
    match refusal {
        RightRefusal::Signing(refusal) => {
            TransactionRefusal::MaturityOperatorAuthorizationRefused { refusal }
        }
        other => TransactionRefusal::MaturityConstructionRightRefused {
            refusal: Box::new(other),
        },
    }
}

/// The operator has answered over one finalized candidate.
///
/// No mutation surface and none spelled: this state carries no candidate
/// in a form a caller could edit, so a mutation here is unrepresentable
/// rather than refused, and a refusing method would be a claim that
/// there was something to refuse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorAuthorizedMaturityAnnouncement<'finalized> {
    finalized: &'finalized FinalizedMaturityAnnouncement,
    witness: InputWitness,
    standing: OperatorEvidenceStanding,
}

impl<'finalized> OperatorAuthorizedMaturityAnnouncement<'finalized> {
    /// The signature, leaf script and control block the boundary
    /// returned.
    #[must_use]
    pub const fn witness(&self) -> &InputWitness {
        &self.witness
    }

    /// The in-process evidence and its named verifier.
    ///
    /// In-process and nothing more: native acceptance is later
    /// execution evidence and has no variant here.
    #[must_use]
    pub const fn standing(&self) -> &OperatorEvidenceStanding {
        &self.standing
    }

    /// The finalized candidate the signature was taken over.
    #[must_use]
    pub const fn finalized(&self) -> &'finalized FinalizedMaturityAnnouncement {
        self.finalized
    }

    /// The exact bytes the signature commits to, unchanged since the
    /// freeze.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        self.finalized.protected_bytes()
    }

    /// Whether an offered candidate is the one that was signed (§13.4).
    ///
    /// The offering is a transaction together with the successor
    /// metadata it claims to commit to, because the bytes alone cannot
    /// tell the two faults apart: semantic metadata reaches the
    /// transaction only through the constructor that commits it into the
    /// successor's program, so a moved metadata and a swapped program
    /// present as one differing program. A builder that re-entered
    /// construction holds both values, which is what makes the pair the
    /// honest argument rather than an extra one.
    ///
    /// The four checks run most specific first. A moved metadata also
    /// moves the program, so naming it a program change would name the
    /// symptom; a sponsor input and a fee role are each caught before
    /// the finalized form's own comparison, which would otherwise report
    /// them as an extended input census and a mutated output.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MaturitySuccessorMetadataChangedAfterSigning`],
    /// [`TransactionRefusal::MaturitySuccessorProgramChangedAfterSigning`],
    /// [`TransactionRefusal::MaturitySponsorInputAddedAfterSigning`] and
    /// [`TransactionRefusal::MaturityFeeRoleChangedAfterSigning`] for
    /// §13.4's four post-signing mutations, then whatever the finalized
    /// form's own comparison refuses about the regions it names and the
    /// exact protected encoding it closes over.
    pub fn check_offered(
        &self,
        offered: &TargetTransaction,
        successor_metadata: StateMetadata,
    ) -> Result<(), TransactionRefusal> {
        let census = self.finalized.outputs();
        if successor_metadata != self.finalized.construction().successor_metadata() {
            return Err(TransactionRefusal::MaturitySuccessorMetadataChangedAfterSigning);
        }

        let position = census.successor_position();
        let at = usize::from(position);
        if offered.outputs().get(at).map(TargetOutput::program)
            != census.outputs().get(at).map(TargetOutput::program)
        {
            return Err(
                TransactionRefusal::MaturitySuccessorProgramChangedAfterSigning { position },
            );
        }

        let fixed = self.finalized.protected().inputs().len();
        if offered.inputs().len() > fixed {
            return Err(TransactionRefusal::MaturitySponsorInputAddedAfterSigning {
                finalized: fixed,
                offered: offered.inputs().len(),
            });
        }

        if offered.outputs().len() > census.outputs().len() {
            return Err(TransactionRefusal::MaturityFeeRoleChangedAfterSigning {
                position: position_of(census.outputs().len()),
            });
        }

        self.finalized.check_offered(offered)
    }

    /// Populates the STATE input's witness and binds the candidate for
    /// submission.
    ///
    /// The stack is built by walking the ABI's own witness records, so
    /// its order is the record's deepest-first schedule rather than a
    /// second list that could come to disagree with it, and the
    /// executing leaf's script and control block follow the seven. The
    /// result is the finalized transaction with that one witness at the
    /// input the finalized form spends, which is the only change §12.7
    /// admits after finalization.
    ///
    /// # Errors
    ///
    /// Whatever [`TargetTransaction::new`] refuses about the assembled
    /// roles. For this form that is unreachable — the announcement has
    /// one input, one output and therefore one witness — and the arm is
    /// kept because the law it states is about the census rather than
    /// about its present size.
    ///
    /// # Panics
    ///
    /// Panics only if the boundary witness carries no first item, which
    /// the operator boundary cannot arrange: it assembles the signature,
    /// the leaf script and the control block in that order, and returns
    /// no witness at all until every response check has passed.
    pub fn bind_for_submission(
        self,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<SubmitReadyMaturityAnnouncement<'finalized>, TransactionRefusal> {
        let finalized = self.finalized;
        let leaf = finalized.executing_leaf(target);
        let authorization = self
            .witness
            .stack()
            .first()
            .expect("the operator boundary assembles its signature as the witness's first item")
            .clone();

        let mut stack: Vec<Vec<u8>> = finalized
            .construction()
            .abi()
            .witness_roles()
            .iter()
            .map(|record| {
                let item = witness_item(finalized, record.role(), &authorization)?;
                checked_witness_item_width(record, item)
            })
            .collect::<Result<_, _>>()?;
        stack.push(leaf.leaf_script().to_vec());
        stack.push(leaf.control_block().to_vec());
        let witness = InputWitness::new(stack);

        let protected = finalized.protected();
        let candidate = TargetTransaction::new(
            protected.version(),
            protected.inputs().to_vec(),
            protected.outputs().to_vec(),
            protected.lock_time(),
            vec![witness.clone()],
        )?;
        let bytes = candidate.encode();

        Ok(SubmitReadyMaturityAnnouncement {
            finalized,
            candidate,
            bytes,
            witness,
            authorization,
            standing: self.standing,
        })
    }
}

/// One announcement that is ready to be submitted, and is not submitted.
///
/// A caller cannot submit it:
///
/// ```compile_fail,E0599
/// use transaction::SubmitReadyMaturityAnnouncement;
/// fn submit(ready: &SubmitReadyMaturityAnnouncement<'_>) {
///     ready.submit();
/// }
/// ```
///
/// cannot finalize it:
///
/// ```compile_fail,E0599
/// use transaction::SubmitReadyMaturityAnnouncement;
/// fn finalize(ready: &SubmitReadyMaturityAnnouncement<'_>) {
///     ready.finalize();
/// }
/// ```
///
/// and cannot take a digest of it:
///
/// ```compile_fail,E0599
/// use transaction::SubmitReadyMaturityAnnouncement;
/// fn digest(ready: &SubmitReadyMaturityAnnouncement<'_>) {
///     ready.digest();
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmitReadyMaturityAnnouncement<'finalized> {
    finalized: &'finalized FinalizedMaturityAnnouncement,
    candidate: TargetTransaction,
    bytes: Vec<u8>,
    witness: InputWitness,
    authorization: Vec<u8>,
    standing: OperatorEvidenceStanding,
}

impl<'finalized> SubmitReadyMaturityAnnouncement<'finalized> {
    /// The candidate transaction, witness included.
    #[must_use]
    pub const fn candidate(&self) -> &TargetTransaction {
        &self.candidate
    }

    /// The exact candidate bytes: the finalized bytes with the one
    /// witness inserted.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The exact protected bytes, unchanged since the freeze.
    ///
    /// Read off the finalized candidate rather than recomputed from
    /// these bytes, which now carry a witness section the protected
    /// encoding does not.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        self.finalized.protected_bytes()
    }

    /// The STATE input's witness: the record's seven items, then the
    /// leaf script and the control block.
    #[must_use]
    pub const fn witness(&self) -> &InputWitness {
        &self.witness
    }

    /// The accepted authorization, as opaque here as at the boundary it
    /// came from.
    #[must_use]
    pub fn authorization(&self) -> &[u8] {
        &self.authorization
    }

    /// The in-process evidence and its named verifier.
    #[must_use]
    pub const fn standing(&self) -> &OperatorEvidenceStanding {
        &self.standing
    }

    /// The candidate's status: submit-ready, and read only.
    #[must_use]
    pub const fn status(&self) -> MaturitySubmissionStatus {
        let _ = self;
        MaturitySubmissionStatus::SubmitReady
    }

    /// The finalized candidate this is a bound state of.
    #[must_use]
    pub const fn finalized(&self) -> &'finalized FinalizedMaturityAnnouncement {
        self.finalized
    }
}

/// The bytes one witness role is populated with, from the source the ABI
/// names for it.
///
/// # Panics
///
/// Everything [`retained_predecessor`] panics on, for its reason.
fn witness_item(
    finalized: &FinalizedMaturityAnnouncement,
    role: StateProgramWitness,
    authorization: &[u8],
) -> Result<Vec<u8>, TransactionRefusal> {
    let construction = finalized.construction();
    let view = construction.validated_view().view();
    Ok(match role {
        StateProgramWitness::SuccessorOutputKeyPrefix => {
            vec![output_key_prefix(
                construction.successor_constructor().parity(),
            )]
        }
        StateProgramWitness::SuccessorNonce => construction
            .successor_constructor()
            .nonce()
            .get()
            .to_be_bytes()
            .to_vec(),
        StateProgramWitness::RequestedCycle => construction
            .request()
            .announced_cycle()
            .get()
            .to_be_bytes()
            .to_vec(),
        StateProgramWitness::StaticSubtreeRoot => view
            .accepted_linked_bundle()
            .static_subtree()
            .root()
            .to_vec(),
        StateProgramWitness::PredecessorMetadata => {
            let canonical = encode_state_metadata(
                &view.predecessor_metadata(),
                view.predecessor_representation_nonce(),
            );
            match construction.abi().schedule() {
                StateWitnessSchedule::WholeMetadata => canonical,
                StateWitnessSchedule::VariableMetadata => {
                    let canonical: [u8; STATE_METADATA_BYTES] =
                        canonical.try_into().map_err(|bytes: Vec<u8>| {
                            TransactionRefusal::WitnessItemWidthMismatch {
                                role,
                                declared: STATE_METADATA_BYTES,
                                populated: bytes.len(),
                            }
                        })?;
                    state_metadata_variable_region(&canonical).to_vec()
                }
            }
        }
        StateProgramWitness::PredecessorOutputKeyPrefix => {
            vec![output_key_prefix(retained_predecessor(finalized).parity())]
        }
        StateProgramWitness::OperatorSignature => authorization.to_vec(),
    })
}

/// Compare one populated item with the ABI's exact declared width.
///
/// # Errors
/// Refuses an item whose width differs from its role's declaration.
pub(crate) fn checked_witness_item_width(
    record: &MaturityWitnessRole,
    item: Vec<u8>,
) -> Result<Vec<u8>, TransactionRefusal> {
    let declared = record.maximum_width().unwrap_or(0);
    if record.minimum_width() != Some(declared) || item.len() != declared {
        return Err(TransactionRefusal::WitnessItemWidthMismatch {
            role: record.role(),
            declared,
            populated: item.len(),
        });
    }
    Ok(item)
}

/// The predecessor constructor the link itself retained.
///
/// The successor's parity is the successor constructor's and this one is
/// the predecessor's; the two commitments are over different metadata
/// and their points need not share a parity, so reading either from the
/// other would state one point's bit about the other.
///
/// # Panics
///
/// Panics only if the accepted linked bundle retains no constructor
/// application, which a link cannot arrange: it retains the application
/// it ran over its own sources.
fn retained_predecessor(finalized: &FinalizedMaturityAnnouncement) -> &CandidateStateConstructor {
    finalized
        .construction()
        .validated_view()
        .view()
        .accepted_linked_bundle()
        .instances()
        .first()
        .expect("a linked bundle retains the constructor application its own link ran")
        .constructor()
}

/// One output index as a position, saturating rather than wrapping.
///
/// A transaction with more than `u16::MAX` outputs is not one this
/// crate's construction can build, and reporting the last representable
/// position is a true statement about where the report stopped being
/// exact.
fn position_of(index: usize) -> u16 {
    u16::try_from(index).unwrap_or(u16::MAX)
}
