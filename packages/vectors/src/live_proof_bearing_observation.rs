//! One owner authorization observed on the PROOF-BEARING lane.
//!
//! # What this module is for, in one sentence
//!
//! The explicit-lane observation established the selected profile over a
//! candidate whose output-witness entries are two zero bytes each; this
//! is the same question asked where those entries carry real range
//! proofs, which is the one lane the explicit observation was forbidden
//! to speak about.
//!
//! # What belongs to the other guide, stated before anything else
//!
//! The funding, the materialization, and the blinding this candidate
//! rests on are the confidential-funding guide's work and are NOT
//! evidenced here. This module funds nothing confidential itself: it
//! asks that guide's ceremony machinery for a predecessor, hands that
//! guide's transaction-wide materializer an intent, and takes back a
//! frozen candidate it did not build. Whether those proofs are sound,
//! whether the balance closes, whether the openings are what they claim
//! — none of it is a claim this module makes or could support, and
//! [`ProofBearingObservationRecord::non_claims`] carries the sentence as
//! data rather than leaving it in this comment.
//!
//! What IS evidenced here is the message: which terms the target hashes
//! for a candidate whose outputs carry proofs, and whether an owner
//! authorization taken over that message is accepted.
//!
//! # Why the leaf is a bare owner check rather than a receipt covenant
//!
//! The explicit-lane observation spends the linked receipt constructor's
//! own coordinator and member leaves, because the explicit branch of
//! [`transaction::live_construct::finalize_live_transfer`] builds that
//! shape. The private branch does not yet take the transaction-wide
//! path, and supplying it an intent, a fixture view and two collaborators
//! is a signature change across every caller that the confidential-
//! funding guide's own restart owns. So there is no covenant-bearing
//! private candidate to spend, and this ceremony does not invent one.
//!
//! It spends a single-leaf tree whose leaf is the owner's signature
//! check and nothing else. That narrows what the run evidences and the
//! narrowing is stated rather than absorbed: an acceptance here is
//! evidence about the MESSAGE the target forms and the profile it is
//! taken under, and it is evidence about no covenant, no constructor
//! recognition, and no conservation arithmetic. The explicit lane's
//! observation already carries the covenant half.
//!
//! # Acceptance and recomputation stay two origins
//!
//! Exactly as on the explicit lane. The target forms its own message and
//! checks the signature with its own primitive; then the accepted
//! transaction is read back out of the node, the signature is taken from
//! THOSE bytes, and it is verified against a message this workspace
//! recomputes. Checking the builder's signature against the builder's
//! message would have been a value compared with itself.
//!
//! # The spent-output triple is the node's and never the ceremony's
//!
//! Three terms of the message are the spent outputs' asset, value and
//! script, and on this lane the value is a COMMITMENT rather than an
//! amount. A ceremony that signed over the amount it asked for would be
//! signing over a different message than the target forms, and it would
//! have no symptom until the chain refused it. So every spent-output
//! field carried into the census is the node's own report of the mined
//! predecessor, the ceremony's expectation is recorded beside it, and
//! two of the negative controls below are exactly the two ways of
//! getting that wrong.
//!
//! # Every key here is published test material
//!
//! The two signing scalars are the BIP-340 specification's own appendix
//! secret keys and the internal key is the published unspendable one,
//! admitted under ADR-015's test-material rule. The chain is created and
//! destroyed by the run.

use std::collections::BTreeMap;

use target_elements::{LeafVersion, ObservationIdentity};
use target_elements_conformance::confidential_fixture::{
    ConfidentialFixtureManifest, ConfidentialFixtureOutput, ConfidentialFixtureRegistry,
    FixtureDerivationProfile, FixtureOpenings, FixtureOutputRole,
    FrozenConfidentialFixtureRegistry, MAX_PARITY_COUNTER, PublicDisposableTestMaterial,
    RegistrationRefusal, ResolvedFixture, predecessor_handle,
};
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::owner_key_oracle::{SignatureRejection, verify_owner_signature};
use target_elements_conformance::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundedOutput,
    ConfidentialFundingBinding, ConfidentialFundingDestination, MinedFundingReadback,
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetConfidentialFundingSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::{
    AssetField, AssetId, COMMITMENT_BYTES, InputWitness, Outpoint, TargetTransaction, Txid,
    ValueField,
};
use transaction::live_census::{
    LiveDeployment, OwnerCensusRefusal, OwnerSigningCensus, ProofFinalizedReceiptInput,
    ProofFinalizedSigningCandidate,
};
use transaction::live_materialize::{
    ConfidentialConstructionIntent, ConfidentialCustodyProfile, ConfidentialDestinationIntent,
    ConfidentialFixtureOutputView, ConfidentialFixtureView, ConfidentialInputIntent,
    ConfidentialMaterializationProfiles, ConfidentialMaterializerProfile, ConfidentialNonceProfile,
    ConfidentialOrderProfile, ConfidentialOutputRole, ConfidentialProofProfile,
    ConfidentialRetryProfile, FixtureOpeningReference, FrozenConfidentialFixtureView,
    IndependentCommitmentCheck, MaterializationRefusal, NonProtocolFundingRegion, ParityOutcome,
    SCALAR_BYTES, materialize_confidential_candidate,
};
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_taproot::LiveCurveCapability;
use transaction::taproot::{
    CONTROL_BASE_BYTES, Digest32, TAPROOT_LEAF_MASK, TAPROOT_PROGRAM_BYTES,
    TAPROOT_WITNESS_VERSION, leaf_hash, witness_program_script,
};

use crate::confidential_materializer::{
    FirstPartyCommitmentCheck, ReferenceConfidentialMaterializer,
};
use crate::confidential_predecessor::{PREDECESSOR_AMOUNTS, selected_profiles};
use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed, printed_order};
use crate::live_plan::{
    FIRST_SCALAR, SECOND_SCALAR, published_owner, reviewed_target, signing_material,
};

/// The caller's own name for the step that issues the protocol asset.
const ISSUE_STEP: &str = "issue-proof-bearing-protocol-asset";

/// The caller's own name for the confidential funding step.
const FUND_STEP: &str = "fund-proof-bearing-predecessor";

/// What the issuing step creates, so the reserve the confidential step
/// draws on exists at all.
///
/// The same three figures the confidential-funding guide's own ceremony
/// issues with. They are re-spelled rather than imported because that
/// ceremony keeps them private, and a ceremony that quietly issued
/// differently would be drawing on a reserve nobody had described.
const ISSUE_OUTPUTS: u8 = 1;
const ISSUE_AMOUNT_PER_OUTPUT: u64 = 1;
const ISSUE_PROGRAM: [u8; 1] = [0x51];

/// The successor case's public handle.
///
/// First-party throughout: this fixture is never sent to any adapter,
/// because the candidate it describes is submitted as raw bytes rather
/// than materialized by a node. Only the predecessor's handle has to
/// resolve in a catalogue on the other side of the wire.
const SUCCESSOR_HANDLE: &str = "ctf-v1/sighash-wave-five-proof-bearing-successor";

/// The successor's two semantic amounts, summing to the consumed pair.
///
/// Deliberately not the predecessor's own split, so that a candidate
/// which had somehow re-materialized the predecessor rather than a
/// successor would be visible as such rather than agreeing by accident.
const SUCCESSOR_AMOUNTS: [u64; 2] = [600_000_000, 400_000_000];

/// The successor amounts with the two roles' figures exchanged.
///
/// The another-candidate control's second materialization, and the
/// exchange is what makes it a different candidate while every other
/// input to the materializer is held fixed.
const EXCHANGED_AMOUNTS: [u64; 2] = [400_000_000, 600_000_000];

/// The auxiliary value every signature here is taken with.
///
/// A published constant rather than randomness, on the explicit lane's
/// own ground: fixing it is what makes every signature this ceremony
/// produces reproducible from inputs that are all written down.
const OBSERVATION_AUXILIARY: [u8; FIELD_ELEMENT_BYTES] = [0x33; FIELD_ELEMENT_BYTES];

/// The opcode a tapscript owner check ends with.
///
/// `OP_CHECKSIG`. The leaf below is the two-item program BIP-342 fixes
/// for a single-key script path: a thirty-two byte push of the x-only
/// key, then this.
const OP_CHECKSIG: u8 = 0xac;

/// The push opcode for a thirty-two byte item.
const OP_PUSHBYTES_32: u8 = 0x20;

/// The transaction version every candidate this ceremony builds carries.
const CANDIDATE_VERSION: u32 = 2;

/// The lock time every candidate this ceremony builds carries.
const CANDIDATE_LOCK_TIME: u32 = 0;

/// The sequence every input this ceremony spends carries.
const CANDIDATE_SEQUENCE: u32 = 0xffff_ffff;

// --- The cases ---------------------------------------------------------

/// One case the ceremony submits to the target.
///
/// The order is the order they run in. A refused transaction spends
/// nothing, so every control is offered against the same pair of
/// confidential coins and the case meant to be accepted goes last,
/// because acceptance is what finally consumes them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProofBearingCase {
    /// The signature taken over the message with the output-witness
    /// vector emptied.
    ///
    /// **The one moved term: term 12, the output-witness hash.**
    ///
    /// The explicit lane has a control of the same shape and it is not
    /// the same claim, which is the whole reason this lane needed its
    /// own run. There, every entry is an empty surjection proof and an
    /// empty range proof, the term is a function of the output count,
    /// and the count is in the protected bytes — so a signer told
    /// nothing but the preimage can rebuild the term exactly and the
    /// control measures a recoverable quantity. Here the entries carry
    /// real range proofs whose bytes appear in the preimage in no
    /// encoded form at any length. The control therefore asks whether
    /// the target hashes the proofs themselves, which is a question the
    /// explicit lane cannot pose.
    ProofBearingVectorEmptied,
    /// The signature taken over the message a preimage-only signer
    /// forms.
    ///
    /// **The one moved term: both length-dependent terms at once, 10
    /// and 12.**
    ///
    /// Stated as one moved thing rather than two because it is one
    /// mistake: a signer that formed the message from a witnessless
    /// serialization alone resizes neither vector, and the pair is
    /// exactly what that signer hashes. It is separated from
    /// [`Self::ProofBearingVectorEmptied`] so that a refusal cannot be
    /// attributed to the output side when the input side moved too —
    /// the four-treatment vocabulary exists for this distinction.
    PreimageOnlySigner,
    /// The signature taken over another proof-bearing candidate's
    /// message.
    ///
    /// **The one moved term: the successor fixture — and on this lane
    /// that is one term rather than two.**
    ///
    /// The second candidate is this one with the two protocol outputs'
    /// semantic amounts exchanged. On the explicit lane moving an amount
    /// moves the outputs hash and nothing else. Here it cannot: a
    /// confidential value does not move without its commitment, and a
    /// commitment does not move without the range proof built over it,
    /// so terms 11 and 12 move together and no smaller change to a
    /// confidential amount exists. The control says that rather than
    /// pretending to isolate a term this lane does not have.
    AnotherProofBearingCandidate,
    /// The candidate authorized under the selected profile.
    ///
    /// The only case the ceremony expects a target to accept, and the
    /// only one whose acceptance is evidence of anything.
    SelectedProfile,
}

impl ProofBearingCase {
    /// Every submitted case, in submission order.
    pub const ALL: &'static [Self] = &[
        Self::ProofBearingVectorEmptied,
        Self::PreimageOnlySigner,
        Self::AnotherProofBearingCandidate,
        Self::SelectedProfile,
    ];

    /// The ceremony's own name for the case.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ProofBearingVectorEmptied => "control-proof-bearing-vector-emptied",
            Self::PreimageOnlySigner => "control-preimage-only-signer",
            Self::AnotherProofBearingCandidate => "control-another-proof-bearing-candidate",
            Self::SelectedProfile => "selected-profile-proof-bearing-authorization",
        }
    }

    /// The one term this case moves, as the report renders it.
    #[must_use]
    pub const fn moved_term(self) -> &'static str {
        match self {
            Self::ProofBearingVectorEmptied => {
                "term 12, the output-witness hash, whose entries here are real range proofs"
            }
            Self::PreimageOnlySigner => {
                "terms 10 and 12 together, which is what a witnessless-serialization signer hashes"
            }
            Self::AnotherProofBearingCandidate => {
                "the successor fixture: a confidential amount cannot move without its commitment \
                 and its proof"
            }
            Self::SelectedProfile => "nothing; this is the case under the selected profile",
        }
    }

    /// Whether the case is a negative control.
    ///
    /// Recomputed from the case rather than stored beside it, so a case
    /// added later cannot be filed as a control by forgetting to say it
    /// is not one.
    #[must_use]
    pub const fn is_negative_control(self) -> bool {
        !matches!(self, Self::SelectedProfile)
    }

    /// How this case's message treats the two witness vectors.
    const fn treatment(self) -> WitnessVectorTreatment {
        match self {
            Self::ProofBearingVectorEmptied => WitnessVectorTreatment::OutputsEmptied,
            Self::PreimageOnlySigner => WitnessVectorTreatment::BothEmptied,
            Self::AnotherProofBearingCandidate | Self::SelectedProfile => {
                WitnessVectorTreatment::BothGrown
            }
        }
    }
}

/// One control the ceremony never submits, because construction refuses
/// it first.
///
/// Both are confidential-value controls with no explicit-lane twin: on
/// that lane the spent value IS the amount, so there is nothing for
/// either of them to move. Both are refused BEFORE any message is
/// formed, which is where the layer discipline bites — this workspace
/// declining to build something is not a node declining to accept it,
/// and a run that reported one as the other would be reporting a
/// first-party opinion as target evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProofBearingConstructionControl {
    /// The spent value declared as the amount the ceremony asked for
    /// rather than the commitment the node reported.
    ///
    /// **The one moved term: the spent output's value field.**
    ///
    /// The exact mistake the node-reported triple exists to prevent,
    /// made into a case. A ceremony that signed over its own expectation
    /// would produce a complete, well-formed signature over a message
    /// the target does not form, with no symptom until the chain refused
    /// it.
    SpentValueAsTheCeremonysExpectation,
    /// The input bound to the other predecessor output's opening.
    ///
    /// **The one moved term: which opening the observed commitment is
    /// checked against.**
    ///
    /// The commitment carried is still the node's own; what moves is the
    /// reference the preflight recomputes it from. Nothing about the
    /// candidate would look different, and on the explicit lane there is
    /// no opening for the mistake to be made with.
    OpeningOfTheOtherOutput,
}

impl ProofBearingConstructionControl {
    /// Every construction control, in the order they are exercised.
    pub const ALL: &'static [Self] = &[
        Self::SpentValueAsTheCeremonysExpectation,
        Self::OpeningOfTheOtherOutput,
    ];

    /// The ceremony's own name for the control.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SpentValueAsTheCeremonysExpectation => {
                "control-spent-value-as-the-ceremonys-expectation"
            }
            Self::OpeningOfTheOtherOutput => "control-opening-of-the-other-output",
        }
    }

    /// The one term this control moves.
    #[must_use]
    pub const fn moved_term(self) -> &'static str {
        match self {
            Self::SpentValueAsTheCeremonysExpectation => {
                "the spent output's value field, from the node's commitment to the ceremony's \
                 amount"
            }
            Self::OpeningOfTheOtherOutput => {
                "which registered opening the observed commitment is recomputed from"
            }
        }
    }
}

// --- Refusals ----------------------------------------------------------

/// Why the ceremony could not proceed.
///
/// Construction and infrastructure only. No member of this type is a
/// target verdict, and a run that stopped for one of these reasons
/// observed nothing about any node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofBearingRefusal {
    /// The reviewed target or a published owner was unavailable.
    SubstrateUnavailable,
    /// The issuance step named no asset to fund against.
    IssuanceNamedNoAsset,
    /// The fixture registry refused the manifest.
    ///
    /// The registry's own typed cause is carried rather than summarized:
    /// a ceremony that reported only that registration failed would
    /// leave a reader unable to tell a handle the grammar refuses from a
    /// derivation that did not settle.
    FixtureNotRegistrable {
        /// The handle offered.
        handle: String,
        /// The registry's own cause.
        refusal: RegistrationRefusal,
    },
    /// The registry registered a case and then did not resolve it under
    /// its own digest, which is a defect in the registry rather than in
    /// the run.
    FixtureDidNotResolve {
        /// The handle offered.
        handle: String,
    },
    /// The confidential funding step created no predecessor to spend.
    FundingCreatedNoPredecessor,
    /// A funded output's outpoint, asset, program, or commitment did not
    /// decode.
    MalformedConfidentialOutput,
    /// The registry's derived openings are not the byte-identity
    /// contract's derived form.
    OpeningsAreNotDerived,
    /// The two predecessor blinders do not sum to the value the
    /// successor is balanced against.
    PredecessorBlindersDoNotClose,
    /// The single-leaf owner tree did not build.
    OwnerLeafNotConstructible,
    /// The materializer refused the intent.
    MaterializationRefused(MaterializationRefusal),
    /// The census refused the materialized candidate.
    CensusRefused(OwnerCensusRefusal),
    /// The signing material refused to produce a signature.
    SigningRefused,
    /// A construction control was expected to be refused and was not.
    ///
    /// A control that built is a control that measured nothing, and the
    /// ceremony stops rather than writing a line saying it fired.
    ConstructionControlWasNotRefused,
    /// The node reported an acceptance with no readback to check.
    AcceptanceCarriedNoReadback,
    /// The bytes the node reported for the accepted identity did not
    /// decode here.
    ReadbackDidNotDecode,
    /// The read-back transaction carries no witness at the signing
    /// input's position.
    ReadbackCarriesNoWitness,
}

// --- What the run records ----------------------------------------------

/// One confidential predecessor output, as the node reported it and as
/// the ceremony expected it.
///
/// Both halves are kept because the spent-output triple is a term of the
/// message. What is signed over is the node's; the expectation is
/// recorded beside it so a divergence is a line in the report rather
/// than a signature silently taken over the wrong output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedConfidentialCoin {
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
    rangeproof_bytes: usize,
    matches_expectation: bool,
}

impl ObservedConfidentialCoin {
    /// The outpoint the node created.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The asset field the node reported, which this form keeps
    /// explicit.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The value field the node reported, which on this lane is a
    /// commitment.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The program the node reported.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }

    /// How many bytes of range proof the predecessor's own output
    /// carried.
    #[must_use]
    pub const fn rangeproof_bytes(&self) -> usize {
        self.rangeproof_bytes
    }

    /// Whether the node's report is what the ceremony asked for: the
    /// requested program, and the commitment the registered opening
    /// derives.
    #[must_use]
    pub const fn matches_expectation(&self) -> bool {
        self.matches_expectation
    }
}

/// What the target did with one submitted case.
///
/// The layer and the node's own text, recorded and never graded. No
/// function in this module compares an observed layer against an
/// expectation: the ceremony has an intention per case, and an intention
/// is not an entitlement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofBearingObservation {
    case: ProofBearingCase,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    accepted_txid: Option<String>,
    submitted_bytes: usize,
}

impl ProofBearingObservation {
    /// Which case this is about.
    #[must_use]
    pub const fn case(&self) -> ProofBearingCase {
        self.case
    }

    /// The outcome layer the target's answer reached.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// How many bytes were handed to the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }
}

/// One construction control and the refusal it drew.
///
/// The refusal is the materializer's own typed value, kept rather than
/// summarized: a control recorded as "refused" without saying which
/// clause refused would be a control nobody can check fired for its own
/// reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofBearingConstructionRefusal {
    control: ProofBearingConstructionControl,
    refusal: MaterializationRefusal,
}

impl ProofBearingConstructionRefusal {
    /// Which control this is about.
    #[must_use]
    pub const fn control(&self) -> ProofBearingConstructionControl {
        self.control
    }

    /// The typed refusal it drew.
    #[must_use]
    pub const fn refusal(&self) -> &MaterializationRefusal {
        &self.refusal
    }
}

/// The second origin's answer.
///
/// Every field is a fact about a comparison rather than a verdict about
/// a node. The signature came out of the target's own copy of the
/// transaction and the message came out of this workspace's own
/// construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofBearingReverification {
    accepted_txid: String,
    witness_txid: String,
    block_height: u32,
    readback_matches_submission: bool,
    recomputed_message: Digest32,
    signature_from_readback: Vec<u8>,
    verified: Result<(), SignatureRejection>,
    verifies_against_emptied_vector_message: bool,
}

impl ProofBearingReverification {
    /// The identity the target computed over the bytes it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// The witness identity the target reported for it.
    #[must_use]
    pub fn witness_txid(&self) -> &str {
        &self.witness_txid
    }

    /// The height the target confirmed it at.
    #[must_use]
    pub const fn block_height(&self) -> u32 {
        self.block_height
    }

    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// The message this workspace recomputed for the signing input.
    #[must_use]
    pub const fn recomputed_message(&self) -> &Digest32 {
        &self.recomputed_message
    }

    /// The signature as it stands in the target's own copy.
    #[must_use]
    pub fn signature_from_readback(&self) -> &[u8] {
        &self.signature_from_readback
    }

    /// Whether the read-back signature verifies against the recomputed
    /// message.
    ///
    /// # Errors
    ///
    /// The oracle's own [`SignatureRejection`], where it does not.
    #[must_use]
    pub const fn verified(&self) -> &Result<(), SignatureRejection> {
        &self.verified
    }

    /// Whether the read-back signature ALSO verifies against the
    /// emptied-vector message.
    ///
    /// Expected false, and checked rather than assumed. A signature that
    /// verified against both would make the whole comparison vacuous.
    #[must_use]
    pub const fn verifies_against_emptied_vector_message(&self) -> bool {
        self.verifies_against_emptied_vector_message
    }
}

/// Everything the ceremony recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProofBearingObservationRecord {
    issued_asset: Option<String>,
    predecessor_digest: Option<[u8; 32]>,
    coins: Vec<ObservedConfidentialCoin>,
    output_witness_vector_length: Option<usize>,
    output_witness_proof_bytes: Vec<usize>,
    spent_value_prefixes: Vec<u8>,
    observations: Vec<ProofBearingObservation>,
    construction_refusals: Vec<ProofBearingConstructionRefusal>,
    reverification: Option<ProofBearingReverification>,
    candidate_messages: BTreeMap<ProofBearingCase, Digest32>,
    refusal: Option<ProofBearingRefusal>,
}

impl ProofBearingObservationRecord {
    /// The asset identity the target chose.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// The digest the predecessor fixture was registered under.
    #[must_use]
    pub const fn predecessor_digest(&self) -> Option<&[u8; 32]> {
        self.predecessor_digest.as_ref()
    }

    /// Every confidential coin the funding step created, as the node
    /// reported it.
    #[must_use]
    pub fn coins(&self) -> &[ObservedConfidentialCoin] {
        &self.coins
    }

    /// The output-witness vector's length on the candidate that was
    /// authorized.
    ///
    /// Recorded because it is the deliverable's own figure: the census
    /// is built at the vector's REAL proof-bearing length, and a run
    /// that had somehow authorized an emptied vector would say so here.
    #[must_use]
    pub const fn output_witness_vector_length(&self) -> Option<usize> {
        self.output_witness_vector_length
    }

    /// How many range-proof bytes each output-witness entry carried.
    ///
    /// The figure that separates this lane from the explicit one, where
    /// every entry is two zero bytes.
    #[must_use]
    pub fn output_witness_proof_bytes(&self) -> &[usize] {
        &self.output_witness_proof_bytes
    }

    /// The serialized prefix of every spent value commitment, in input
    /// order.
    ///
    /// Recorded and not asserted. The prefix is the first byte of the
    /// commitment the message's spent-value term hashes, so an
    /// acceptance is an agreement about these bytes as much as about any
    /// other — but WHICH prefixes a predecessor carries is the
    /// confidential-funding guide's own property, observed by that
    /// guide's own ceremony, and this ceremony writes them down rather
    /// than making a claim out of them.
    #[must_use]
    pub fn spent_value_prefixes(&self) -> &[u8] {
        &self.spent_value_prefixes
    }

    /// What the target did with each submitted case.
    #[must_use]
    pub fn observations(&self) -> &[ProofBearingObservation] {
        &self.observations
    }

    /// Every construction control and the refusal it drew.
    #[must_use]
    pub fn construction_refusals(&self) -> &[ProofBearingConstructionRefusal] {
        &self.construction_refusals
    }

    /// The second origin's answer, where an acceptance was observed.
    #[must_use]
    pub const fn reverification(&self) -> Option<&ProofBearingReverification> {
        self.reverification.as_ref()
    }

    /// The message each submitted case's signature was taken over.
    #[must_use]
    pub const fn candidate_messages(&self) -> &BTreeMap<ProofBearingCase, Digest32> {
        &self.candidate_messages
    }

    /// Why the ceremony stopped, where it did.
    #[must_use]
    pub const fn refusal(&self) -> Option<&ProofBearingRefusal> {
        self.refusal.as_ref()
    }

    /// What this ceremony does not claim, whatever it observed.
    ///
    /// Carried as data so that a report renders them rather than a
    /// reader remembering them, and so that a consumer which dropped one
    /// would be dropping a value rather than a paragraph.
    #[must_use]
    pub fn non_claims() -> Vec<&'static str> {
        vec![
            "evidences no confidential funding: the predecessor is funded by the \
             confidential-funding guide's own ceremony and adapter, and nothing about that \
             funding is reviewed here",
            "evidences no materialization: the candidate is frozen by that guide's \
             transaction-wide materializer, which this ceremony calls and does not check",
            "evidences no blinding: the openings, the balancing solve, and the range proofs are \
             that guide's, and their soundness is not a claim this run supports",
            "evidences no covenant: the leaf spent here is an owner signature check and not a \
             receipt constructor, because the private branch of the live finalization does not \
             yet build a covenant-bearing candidate",
            "establishes no dimension as reviewed: the profile's disposition was recomputed by \
             the review verdict and no run re-opens it",
            "discharges no safety-matrix row: a refused control is not a row's own relation \
             driven to its own refusal",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

// --- The owner's single-leaf tree --------------------------------------

/// One published owner's single-leaf taproot instance.
///
/// Derived here rather than taken from the candidate ABI, because the
/// ABI's instances carry the receipt constructor's coordinator and member
/// leaves and this ceremony spends neither. Everything below is
/// recomputed from published material by the workspace's own taproot
/// primitives; nothing is a constant a reader would have to check against
/// nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnerLeaf {
    scalar: [u8; FIELD_ELEMENT_BYTES],
    owner_bytes: Vec<u8>,
    leaf_script: Vec<u8>,
    tapleaf_hash: Digest32,
    control_block: Vec<u8>,
    program: Vec<u8>,
}

impl OwnerLeaf {
    /// The instance one published scalar's owner spends under.
    fn derive(scalar: &[u8; FIELD_ELEMENT_BYTES]) -> Result<Self, ProofBearingRefusal> {
        let target = reviewed_target().map_err(|_| ProofBearingRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| ProofBearingRefusal::SubstrateUnavailable)?,
        );
        let owner =
            published_owner(scalar).map_err(|_| ProofBearingRefusal::SubstrateUnavailable)?;
        let owner_bytes = owner.bytes().to_vec();

        // BIP-342's single-key script path: push the x-only key, check
        // the signature. Two items and no covenant, which is what the
        // module comment says this run evidences and does not.
        let mut leaf_script = Vec::with_capacity(owner_bytes.len() + 2);
        leaf_script.push(OP_PUSHBYTES_32);
        leaf_script.extend_from_slice(&owner_bytes);
        leaf_script.push(OP_CHECKSIG);

        let tapleaf_hash = leaf_hash(LeafVersion::TAPSCRIPT, &leaf_script);

        // A single leaf is its own merkle root, so the control block
        // carries no path at all.
        let output_key = curve
            .output_key(&UNSPENDABLE_INTERNAL_KEY, &tapleaf_hash)
            .ok_or(ProofBearingRefusal::OwnerLeafNotConstructible)?;
        let program = witness_program_script(&target, TAPROOT_WITNESS_VERSION, output_key.key())
            .map_err(|_| ProofBearingRefusal::OwnerLeafNotConstructible)?;

        let mut control_block = Vec::with_capacity(CONTROL_BASE_BYTES);
        control_block
            .push((LeafVersion::TAPSCRIPT.get() & TAPROOT_LEAF_MASK) | output_key.parity().bit());
        control_block.extend_from_slice(&UNSPENDABLE_INTERNAL_KEY);

        Ok(Self {
            scalar: *scalar,
            owner_bytes,
            leaf_script,
            tapleaf_hash,
            control_block,
            program,
        })
    }

    /// The finalized receipt selection for one input position.
    fn finalized_receipt(&self, index: u32) -> ProofFinalizedReceiptInput {
        ProofFinalizedReceiptInput::for_evidence(
            index,
            self.leaf_script.clone(),
            self.control_block.clone(),
        )
    }

    /// The witness stack a script-path spend of this leaf carries.
    fn witness(&self, signature: Vec<u8>) -> InputWitness {
        InputWitness::new(vec![
            signature,
            self.leaf_script.clone(),
            self.control_block.clone(),
        ])
    }
}

// --- The fixtures ------------------------------------------------------

/// The materialization profiles this ceremony runs under.
pub(crate) const fn materialization_profiles() -> ConfidentialMaterializationProfiles {
    ConfidentialMaterializationProfiles {
        reproducibility_contract: target_elements::ReproducibilityContract::ByteIdentity,
        custody_profile: ConfidentialCustodyProfile::CentralPublicFixtures,
        materializer_profile: ConfidentialMaterializerProfile::GuideCtfDeterministicV1,
        proof_profile: ConfidentialProofProfile::ExplicitAssetRangeproofV1,
        nonce_profile: ConfidentialNonceProfile::DeterministicDerivedV1,
        order_profile: ConfidentialOrderProfile::FixtureFixedOrder,
        retry_profile: ConfidentialRetryProfile::NoRetry,
    }
}

/// One manifest, built from its own parts.
fn manifest(
    handle: &str,
    explicit_asset: [u8; 32],
    input_blinder_sum: [u8; 32],
    amounts: [u64; 2],
    programs: [Vec<u8>; 2],
) -> ConfidentialFixtureManifest {
    let [first, second] = programs;
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new(handle.to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: selected_profiles(),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset,
        input_blinder_sum,
        outputs: vec![
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Primary,
                semantic_amount: amounts[0],
                output_program: first,
            },
            ConfidentialFixtureOutput {
                role: FixtureOutputRole::Balancing,
                semantic_amount: amounts[1],
                output_program: second,
            },
        ],
    }
}

/// The derived openings of one resolved fixture.
fn derived_openings(
    fixture: &ResolvedFixture,
) -> Result<
    (
        &[Option<target_elements_conformance::confidential_fixture::DerivedOpening>],
        u16,
    ),
    ProofBearingRefusal,
> {
    match fixture.openings() {
        FixtureOpenings::Derived {
            openings,
            parity_counter,
        } => Ok((openings, *parity_counter)),
        FixtureOpenings::RunProduced => Err(ProofBearingRefusal::OpeningsAreNotDerived),
    }
}

/// One projected fixture, from a resolved one.
///
/// The projection the transaction-wide materializer consumes. It is the
/// registry's own values re-shaped and nothing else: no amount, program,
/// blinder or seed is chosen here.
fn project(fixture: &ResolvedFixture) -> Result<ConfidentialFixtureView, ProofBearingRefusal> {
    let (openings, parity_counter) = derived_openings(fixture)?;
    let mut outputs = Vec::with_capacity(openings.len());
    for (output, opening) in fixture.outputs().iter().zip(openings) {
        // The fee role is projected rather than refused, and it is
        // projected to the view's own fee constructor rather than through
        // the opening-bearing one. The registry gives a fee output no
        // opening; the view takes none; and the materializer's fee stage
        // emits an explicit value, an explicit asset, a null nonce and an
        // empty program. That chain is what the typed stop used to say
        // did not exist, and mapping the role onto `Balancing` instead
        // would have produced a blinded fee output — the one outcome
        // worse than the stop.
        if output.role == FixtureOutputRole::Fee {
            // The registry holds a fee output out of the solve and gives
            // it no opening. One that arrived with an opening would be a
            // registry that had changed its mind, and this projection is
            // not the place to decide which half to believe.
            if opening.is_some() {
                return Err(ProofBearingRefusal::OpeningsAreNotDerived);
            }
            outputs.push(ConfidentialFixtureOutputView::fee(output.semantic_amount));
            continue;
        }
        // The sponsor change is projected through the view's own
        // sponsor-change constructor for the reason the fee is projected
        // through the fee one: it is the single output whose ASSET is not
        // the fixture's, and the opening-bearing constructor has no
        // parameter to say so. Routing it through `new` would have
        // compiled and produced a remainder committed against the
        // protocol asset — a candidate the target reads as the wrong
        // asset entirely, which is the silent wrong transaction rather
        // than the honest stop.
        //
        // The asset travels in the ROLE, so it arrives here already bound
        // to the output that carries it and nothing downstream has to
        // pair an asset with an index.
        if let FixtureOutputRole::SponsorChange { asset } = output.role {
            // A committed remainder with no opening is a registry that
            // changed its mind between deriving and resolving, and this
            // projection does not decide which half to believe.
            let opening = opening
                .as_ref()
                .ok_or(ProofBearingRefusal::OpeningsAreNotDerived)?;
            outputs.push(ConfidentialFixtureOutputView::sponsor_change(
                output.semantic_amount,
                output.output_program.clone(),
                AssetId::from_internal(asset),
                opening.value_blinder,
                opening.nonce_input,
                opening.rangeproof_seed,
            ));
            continue;
        }
        // The explicit destination is projected through its own
        // constructor for the reason the fee is projected through the
        // fee one, and the reason is the same fact said from the other
        // side: it carries NO opening. Routing it through the
        // opening-bearing constructor below would demand an opening the
        // registry never derived and refuse a manifest that is correct,
        // and giving it a placeholder would be worse -- the materializer
        // would build a commitment for an output whose whole point is
        // that it publishes its amount.
        if output.role == FixtureOutputRole::ExplicitDestination {
            // An explicit destination that arrived WITH an opening is a
            // registry that changed its mind between deriving and
            // resolving, and this projection does not decide which half
            // to believe.
            if opening.is_some() {
                return Err(ProofBearingRefusal::OpeningsAreNotDerived);
            }
            outputs.push(ConfidentialFixtureOutputView::explicit_destination(
                output.semantic_amount,
                output.output_program.clone(),
            ));
            continue;
        }
        let role = match output.role {
            FixtureOutputRole::Primary => ConfidentialOutputRole::Primary,
            // Both solving roles project to the view's one solving role,
            // and that is not a role being flattened away. The view's
            // `Balancing` means "this output's blinder is solved from
            // the others", and the sole form is that statement with no
            // others — the same instruction to the materializer, whose
            // solve over an empty set of other blinders returns the
            // input blinder sum. What the registry's extra member
            // carries is a DECLARATION about the manifest's shape, and a
            // declaration has done its work by the time the manifest is
            // registered.
            FixtureOutputRole::Balancing | FixtureOutputRole::SoleBalancing => {
                ConfidentialOutputRole::Balancing
            }
            // The registry's role vocabulary is open and the view's is
            // not. A role added there with no place here is a refusal
            // rather than a silent substitution.
            _ => return Err(ProofBearingRefusal::OpeningsAreNotDerived),
        };
        // Every role that reaches here carries an opening, the one that
        // does not having refused above.
        let opening = opening
            .as_ref()
            .ok_or(ProofBearingRefusal::OpeningsAreNotDerived)?;
        outputs.push(ConfidentialFixtureOutputView::new(
            role,
            output.semantic_amount,
            output.output_program.clone(),
            opening.value_blinder,
            opening.nonce_input,
            opening.rangeproof_seed,
        ));
    }

    Ok(ConfidentialFixtureView::new(
        *fixture.digest().bytes(),
        AssetId::from_internal(*fixture.explicit_asset()),
        *fixture.input_blinder_sum(),
        ParityOutcome::Settled {
            counter: parity_counter,
        },
        outputs,
    ))
}

/// One already-frozen registry's case, resolved and projected.
///
/// The seam a ceremony needs when the manifest is somebody else's. The
/// sponsor reserve case is registered by its own module, because the
/// digest the EXECUTOR funds against comes from that manifest and a
/// second spelling of it here would resolve to a different digest and
/// refuse. So the case is registered once, there, and this projects the
/// frozen result rather than re-registering it.
///
/// # Errors
///
/// [`ProofBearingRefusal::FixtureDidNotResolve`] where the frozen
/// registry does not hold the handle at that digest, and the projection's
/// own refusals otherwise.
pub(crate) fn project_frozen(
    frozen: &FrozenConfidentialFixtureRegistry,
    handle: &ConfidentialFixtureHandle,
    digest: &ConfidentialFixtureDigest,
) -> Result<ConfidentialFixtureView, ProofBearingRefusal> {
    let resolved =
        frozen
            .resolve(handle, digest)
            .map_err(|_| ProofBearingRefusal::FixtureDidNotResolve {
                handle: handle.as_str().to_owned(),
            })?;
    project(resolved)
}

/// One registered and frozen fixture, resolved under its own digest.
pub(crate) fn register(
    handle: &str,
    explicit_asset: [u8; 32],
    input_blinder_sum: [u8; 32],
    amounts: [u64; 2],
    programs: [Vec<u8>; 2],
) -> Result<(ConfidentialFixtureDigest, ConfidentialFixtureView), ProofBearingRefusal> {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(manifest(
            handle,
            explicit_asset,
            input_blinder_sum,
            amounts,
            programs,
        ))
        .map_err(|refusal| ProofBearingRefusal::FixtureNotRegistrable {
            handle: handle.to_owned(),
            refusal,
        })?;
    let frozen = registry.freeze();
    let identity = ConfidentialFixtureHandle::new(handle.to_owned());
    let digest = *frozen.registered_digest(&identity).ok_or_else(|| {
        ProofBearingRefusal::FixtureDidNotResolve {
            handle: handle.to_owned(),
        }
    })?;
    let resolved = frozen.resolve(&identity, &digest).map_err(|_| {
        ProofBearingRefusal::FixtureDidNotResolve {
            handle: handle.to_owned(),
        }
    })?;

    Ok((digest, project(resolved)?))
}

/// One manifest of arbitrary output arity, built from its own parts.
///
/// The multi-output constructor the restart order's fifth step needs:
/// [`manifest`] is fixed at the two outputs a one-to-one control has,
/// while a split, a many-to-many, or a several-owner transfer has more.
///
/// # The roles are stated by the caller, not by a position
///
/// This builder used to cast the LAST output as the balancing one and
/// every earlier output as primary. That read as a convenient spelling of
/// the registry's exactly-one-balancing rule, and it was one for as long
/// as every output was a blinded protocol output.
///
/// It stopped being one twice over. A FEE output is the one output that
/// must never balance, and arriving last it arrived cast as the output
/// that does. And a SINGLE-output manifest must DECLARE the fully-solved
/// form rather than be assigned a role that asks to be solved from others
/// that are not there — a positional rule cannot express a declaration,
/// because a position is not something a caller says.
///
/// So the role travels with the output. The builder no longer decides
/// anything about the balance, which moves the decision to the call site
/// that actually knows the shape it is building.
fn multi_manifest(
    handle: &str,
    explicit_asset: [u8; 32],
    input_blinder_sum: [u8; 32],
    outputs: Vec<ConfidentialFixtureOutput>,
) -> ConfidentialFixtureManifest {
    ConfidentialFixtureManifest {
        handle: ConfidentialFixtureHandle::new(handle.to_owned()),
        material_class: PublicDisposableTestMaterial::EXPECTED,
        derivation_profile: FixtureDerivationProfile::GuideCtfV1,
        profiles: selected_profiles(),
        retry_limit: MAX_PARITY_COUNTER,
        explicit_asset,
        input_blinder_sum,
        outputs,
    }
}

/// One registered and frozen fixture of arbitrary output arity, resolved
/// under its own digest.
///
/// The arity-general sibling of [`register`]: it registers a manifest of
/// `amounts.len()` outputs rather than exactly two, and returns the same
/// digest and projected view. The registry's floor of two outputs, its
/// exactly-one-balancing rule, and its positive-amount and non-empty
/// program checks all still apply, so a caller that offers one output or
/// an unbalanced set is refused here rather than at the node.
///
/// # Errors
///
/// [`ProofBearingRefusal::FixtureNotRegistrable`] where the registry
/// refuses the manifest, and [`ProofBearingRefusal::FixtureDidNotResolve`]
/// where the frozen registry does not hold it.
pub(crate) fn register_multi(
    handle: &str,
    explicit_asset: [u8; 32],
    input_blinder_sum: [u8; 32],
    outputs: Vec<ConfidentialFixtureOutput>,
) -> Result<(ConfidentialFixtureDigest, ConfidentialFixtureView), ProofBearingRefusal> {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(multi_manifest(
            handle,
            explicit_asset,
            input_blinder_sum,
            outputs,
        ))
        .map_err(|refusal| ProofBearingRefusal::FixtureNotRegistrable {
            handle: handle.to_owned(),
            refusal,
        })?;
    let frozen = registry.freeze();
    let identity = ConfidentialFixtureHandle::new(handle.to_owned());
    let digest = *frozen.registered_digest(&identity).ok_or_else(|| {
        ProofBearingRefusal::FixtureDidNotResolve {
            handle: handle.to_owned(),
        }
    })?;
    let resolved = frozen.resolve(&identity, &digest).map_err(|_| {
        ProofBearingRefusal::FixtureDidNotResolve {
            handle: handle.to_owned(),
        }
    })?;

    Ok((digest, project(resolved)?))
}

/// Whether the registry refuses a manifest of a given output arity, and
/// with which refusal.
///
/// The one-output shapes of the fixture catalogue — private-merge, the
/// strict one-to-one, and the fee-only case — are unconstructible on this
/// lane, and this states the refusal the registry answers with rather than
/// leaving it to be discovered at a node that never sees them. It builds
/// no fixture and reaches no node. `None` is the answer where a manifest
/// the caller expected to be refused was instead admitted.
#[cfg(test)]
#[must_use]
pub(crate) fn registry_refusal_for(
    handle: &str,
    explicit_asset: [u8; 32],
    input_blinder_sum: [u8; 32],
    outputs: Vec<ConfidentialFixtureOutput>,
) -> Option<RegistrationRefusal> {
    let mut registry = ConfidentialFixtureRegistry::new();
    registry
        .register(multi_manifest(
            handle,
            explicit_asset,
            input_blinder_sum,
            outputs,
        ))
        .err()
}

// --- The ceremony ------------------------------------------------------

/// What the plan is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset the confidential step funds against.
    Issue,
    /// Fund one confidential predecessor paying the owners' programs.
    Fund,
    /// Submit the case at this position of the census.
    Submit(usize),
    /// Nothing further.
    Done,
}

/// One submitted case, held until its answer arrives.
struct PendingCase {
    case: ProofBearingCase,
    bytes: Vec<u8>,
}

/// Everything the ceremony learned once the predecessor was mined.
struct Predecessor {
    digest: ConfidentialFixtureDigest,
    view: ConfidentialFixtureView,
    successor_digest: ConfidentialFixtureDigest,
    successor_view: ConfidentialFixtureView,
    exchanged_digest: ConfidentialFixtureDigest,
    exchanged_view: ConfidentialFixtureView,
    asset: AssetId,
}

/// The Wave-5 proof-bearing observation ceremony.
pub struct ProofBearingObservationPlanner {
    stage: Stage,
    genesis_block_hash: Digest32,
    owners: [OwnerLeaf; 2],
    predecessor: Option<Predecessor>,
    pending: Option<PendingCase>,
    record: ProofBearingObservationRecord,
}

impl ProofBearingObservationPlanner {
    /// The ceremony bound to one deployment's printed genesis identity.
    ///
    /// The seed the target hashes with is the reverse of the identity it
    /// prints, and the explicit lane's ceremony observed which of the
    /// two orders the target actually uses. That question is settled and
    /// is not re-asked here: this ceremony applies the observed relation
    /// through the same function.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the reviewed
    /// target or a published owner is unavailable.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let first =
            OwnerLeaf::derive(&FIRST_SCALAR).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
        let second =
            OwnerLeaf::derive(&SECOND_SCALAR).map_err(|_| VectorError::LiveSubstrateUnavailable)?;

        Ok(Self {
            stage: Stage::Issue,
            genesis_block_hash: printed_order(printed_genesis_identity),
            owners: [first, second],
            predecessor: None,
            pending: None,
            record: ProofBearingObservationRecord::default(),
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &ProofBearingObservationRecord {
        &self.record
    }

    /// The identity the confidential funding step is asked under.
    #[must_use]
    pub fn funding_case() -> OperationCaseId {
        OperationCaseId {
            operation: target_elements_conformance::protocol::OperationStepKind::FundConfidential,
            step: FUND_STEP.to_owned(),
        }
    }

    /// Record one refusal and stop.
    fn refuse(&mut self, refusal: ProofBearingRefusal) -> PlanRefused {
        self.record.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The two owner programs the predecessor's outputs pay, in fixed
    /// order.
    ///
    /// Distinct, because a range proof binds the program it was built
    /// for and two outputs paying one program would leave that binding
    /// unable to fail.
    fn owner_programs(&self) -> [Vec<u8>; 2] {
        [
            self.owners[0].program.clone(),
            self.owners[1].program.clone(),
        ]
    }

    /// Register the three fixtures this run needs, against one issued
    /// asset.
    ///
    /// The predecessor is the case the adapter also holds; the successor
    /// and the exchanged successor are first-party and never leave this
    /// process.
    fn settle_asset(&mut self, printed: &str) -> Result<(), ProofBearingRefusal> {
        let asset = asset_of(printed).ok_or(ProofBearingRefusal::IssuanceNamedNoAsset)?;
        let commit_order = *asset.internal();
        let programs = self.owner_programs();

        let (digest, view) = register(
            predecessor_handle().as_str(),
            commit_order,
            // The funding input is explicit, so it contributes a zero
            // value blinder. This is the adapter's own catalogue entry
            // for the case, and a different figure here would be
            // refused on the other side as a digest that drifted.
            [0_u8; 32],
            PREDECESSOR_AMOUNTS,
            programs.clone(),
        )?;

        // The successor balances against the sum of the two consumed
        // blinders. The registry solved the predecessor's balancing
        // blinder as the additive inverse of its primary, so the sum is
        // zero — CHECKED here rather than assumed, because a ceremony
        // that assumed it and was wrong would produce a candidate whose
        // value balance does not close and would learn so from a chain.
        let blinders = [
            *view.outputs()[0]
                .value_blinder()
                .ok_or(ProofBearingRefusal::PredecessorBlindersDoNotClose)?,
            *view.outputs()[1]
                .value_blinder()
                .ok_or(ProofBearingRefusal::PredecessorBlindersDoNotClose)?,
        ];
        let checker = FirstPartyCommitmentCheck::new();
        let solved = checker
            .solve_balancing_blinder(&[0_u8; SCALAR_BYTES], &blinders[..1])
            .ok_or(ProofBearingRefusal::PredecessorBlindersDoNotClose)?;
        if solved != blinders[1] {
            return Err(ProofBearingRefusal::PredecessorBlindersDoNotClose);
        }

        let (successor_digest, successor_view) = register(
            SUCCESSOR_HANDLE,
            commit_order,
            [0_u8; 32],
            SUCCESSOR_AMOUNTS,
            programs.clone(),
        )?;
        let (exchanged_digest, exchanged_view) = register(
            SUCCESSOR_HANDLE,
            commit_order,
            [0_u8; 32],
            EXCHANGED_AMOUNTS,
            programs,
        )?;

        self.record.issued_asset = Some(printed.to_owned());
        self.record.predecessor_digest = Some(*digest.bytes());
        self.predecessor = Some(Predecessor {
            digest,
            view,
            successor_digest,
            successor_view,
            exchanged_digest,
            exchanged_view,
            asset,
        });
        Ok(())
    }

    /// The confidential funding step, against the registered
    /// predecessor.
    fn funding_step(&self) -> Result<OperationStep, ProofBearingRefusal> {
        let predecessor = self
            .predecessor
            .as_ref()
            .ok_or(ProofBearingRefusal::IssuanceNamedNoAsset)?;
        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(ProofBearingRefusal::IssuanceNamedNoAsset)?;

        Ok(OperationStep::new(
            FUND_STEP,
            OperationSubject::ConfidentialFunding(Box::new(TargetConfidentialFundingSubject {
                issue_asset: false,
                asset: Some(printed),
                destinations: self
                    .owner_programs()
                    .into_iter()
                    .map(|output_program| ConfidentialFundingDestination { output_program })
                    .collect(),
                binding: ConfidentialFundingBinding {
                    fixture_handle: predecessor_handle(),
                    fixture_digest: predecessor.digest,
                    profiles: selected_profiles(),
                },
            })),
        ))
    }

    /// Take the funded confidential coins from the node's own report.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), ProofBearingRefusal> {
        if response.confidential_funded_outputs.len() != PREDECESSOR_AMOUNTS.len() {
            return Err(ProofBearingRefusal::FundingCreatedNoPredecessor);
        }
        let mut coins = Vec::with_capacity(response.confidential_funded_outputs.len());
        for (index, funded) in response.confidential_funded_outputs.iter().enumerate() {
            coins.push(self.observed_coin(index, funded)?);
        }
        self.record.spent_value_prefixes = coins
            .iter()
            .filter_map(|coin| match coin.value() {
                ValueField::Commitment(commitment) => commitment.first().copied(),
                _ => None,
            })
            .collect();
        self.record.coins = coins;
        Ok(())
    }

    /// One confidential coin, as reported and as expected.
    fn observed_coin(
        &self,
        index: usize,
        funded: &ConfidentialFundedOutput,
    ) -> Result<ObservedConfidentialCoin, ProofBearingRefusal> {
        let predecessor = self
            .predecessor
            .as_ref()
            .ok_or(ProofBearingRefusal::MalformedConfidentialOutput)?;

        let outpoint = outpoint_of(&funded.outpoint)
            .ok_or(ProofBearingRefusal::MalformedConfidentialOutput)?;
        let asset = asset_of(&funded.explicit_asset)
            .ok_or(ProofBearingRefusal::MalformedConfidentialOutput)?;
        let program =
            decode_hex(&funded.script).ok_or(ProofBearingRefusal::MalformedConfidentialOutput)?;
        let commitment = <[u8; COMMITMENT_BYTES]>::try_from(funded.value_commitment.as_slice())
            .map_err(|_| ProofBearingRefusal::MalformedConfidentialOutput)?;

        // The expectation: the program the request named, the asset the
        // issuance reported, and the commitment the registered opening
        // derives. All three are compared and none is substituted — what
        // travels into the census below is the node's.
        let projected = predecessor
            .view
            .outputs()
            .get(index)
            .ok_or(ProofBearingRefusal::MalformedConfidentialOutput)?;
        let checker = FirstPartyCommitmentCheck::new();
        let derived = checker
            .recompute(
                predecessor.asset,
                projected.semantic_amount(),
                projected
                    .value_blinder()
                    .ok_or(ProofBearingRefusal::MalformedConfidentialOutput)?,
            )
            .ok_or(ProofBearingRefusal::MalformedConfidentialOutput)?;
        let matches_expectation = asset == predecessor.asset
            && program == *projected.output_program()
            && commitment == *derived.bytes();

        Ok(ObservedConfidentialCoin {
            outpoint,
            asset: AssetField::Explicit(asset),
            value: ValueField::Commitment(commitment),
            program,
            rangeproof_bytes: funded.rangeproof.len(),
            matches_expectation,
        })
    }

    /// The intent one materialization runs under.
    ///
    /// `exchanged` selects the second successor fixture, which is how
    /// the another-candidate control gets a candidate that differs in
    /// its protocol region and in nothing else.
    fn intent(
        &self,
        exchanged: bool,
    ) -> Result<ConfidentialConstructionIntent, ProofBearingRefusal> {
        let predecessor = self
            .predecessor
            .as_ref()
            .ok_or(ProofBearingRefusal::FundingCreatedNoPredecessor)?;
        let (digest, view, amounts) = if exchanged {
            (
                predecessor.exchanged_digest,
                &predecessor.exchanged_view,
                EXCHANGED_AMOUNTS,
            )
        } else {
            (
                predecessor.successor_digest,
                &predecessor.successor_view,
                SUCCESSOR_AMOUNTS,
            )
        };

        let mut inputs = Vec::with_capacity(self.record.coins.len());
        for (index, coin) in self.record.coins.iter().enumerate() {
            inputs.push(ConfidentialInputIntent::new(
                coin.outpoint,
                coin.asset,
                coin.value,
                coin.program.clone(),
                CANDIDATE_SEQUENCE,
                FixtureOpeningReference::new(
                    predecessor_handle().as_str().to_owned(),
                    *predecessor.digest.bytes(),
                    index,
                ),
                PREDECESSOR_AMOUNTS[index],
                [0_u8; SCALAR_BYTES],
            ));
        }

        let mut destinations = Vec::with_capacity(view.outputs().len());
        for (index, projected) in view.outputs().iter().enumerate() {
            destinations.push(ConfidentialDestinationIntent::new(
                amounts[index],
                predecessor.asset,
                projected.output_program().to_vec(),
                FixtureOpeningReference::new(SUCCESSOR_HANDLE.to_owned(), *digest.bytes(), index),
                projected.role(),
            ));
        }

        Ok(ConfidentialConstructionIntent::new(
            inputs,
            destinations,
            NonProtocolFundingRegion::default(),
            materialization_profiles(),
            CANDIDATE_VERSION,
            CANDIDATE_LOCK_TIME,
        ))
    }

    /// The frozen fixture view one materialization consumes.
    fn view(&self) -> Result<FrozenConfidentialFixtureView, ProofBearingRefusal> {
        let predecessor = self
            .predecessor
            .as_ref()
            .ok_or(ProofBearingRefusal::FundingCreatedNoPredecessor)?;
        let mut entries = BTreeMap::new();
        entries.insert(
            predecessor_handle().as_str().to_owned(),
            predecessor.view.clone(),
        );
        entries.insert(
            SUCCESSOR_HANDLE.to_owned(),
            predecessor.successor_view.clone(),
        );
        Ok(FrozenConfidentialFixtureView::new(entries))
    }

    /// The frozen fixture view the exchanged candidate consumes.
    fn exchanged_view(&self) -> Result<FrozenConfidentialFixtureView, ProofBearingRefusal> {
        let predecessor = self
            .predecessor
            .as_ref()
            .ok_or(ProofBearingRefusal::FundingCreatedNoPredecessor)?;
        let mut entries = BTreeMap::new();
        entries.insert(
            predecessor_handle().as_str().to_owned(),
            predecessor.view.clone(),
        );
        entries.insert(
            SUCCESSOR_HANDLE.to_owned(),
            predecessor.exchanged_view.clone(),
        );
        Ok(FrozenConfidentialFixtureView::new(entries))
    }

    /// One materialized proof-bearing candidate.
    fn materialize(
        &self,
        exchanged: bool,
    ) -> Result<ProofFinalizedSigningCandidate, ProofBearingRefusal> {
        let intent = self.intent(exchanged)?;
        let view = if exchanged {
            self.exchanged_view()?
        } else {
            self.view()?
        };
        let materialized = materialize_confidential_candidate(
            &intent,
            &view,
            &ReferenceConfidentialMaterializer::new(),
            &FirstPartyCommitmentCheck::new(),
        )
        .map_err(ProofBearingRefusal::MaterializationRefused)?;
        let receipts = self
            .owners
            .iter()
            .enumerate()
            .map(|(index, owner)| owner.finalized_receipt(u32::try_from(index).unwrap_or(u32::MAX)))
            .collect();
        Ok(ProofFinalizedSigningCandidate::for_receipt_evidence(
            materialized,
            receipts,
        ))
    }

    /// The census of one materialized candidate.
    fn census(
        &self,
        finalized: &ProofFinalizedSigningCandidate,
    ) -> Result<OwnerSigningCensus, ProofBearingRefusal> {
        let target = reviewed_target().map_err(|_| ProofBearingRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| ProofBearingRefusal::SubstrateUnavailable)?,
        );
        OwnerSigningCensus::from_proof_finalized(
            &target,
            finalized,
            LiveDeployment::new(self.genesis_block_hash),
            &curve,
        )
        .map_err(ProofBearingRefusal::CensusRefused)
    }

    /// The two construction controls, exercised before anything is
    /// submitted.
    ///
    /// Each replaces exactly one member of the intent and asks the
    /// materializer for a candidate. Both must be refused; a control
    /// that built is a control that measured nothing, and the ceremony
    /// stops rather than recording a refusal that did not happen.
    fn exercise_construction_controls(&mut self) -> Result<(), ProofBearingRefusal> {
        let mut refusals = Vec::with_capacity(ProofBearingConstructionControl::ALL.len());
        for control in ProofBearingConstructionControl::ALL {
            let mut intent_inputs = self.intent(false)?.inputs().to_vec();
            let predecessor = self
                .predecessor
                .as_ref()
                .ok_or(ProofBearingRefusal::FundingCreatedNoPredecessor)?;
            let first = intent_inputs
                .first()
                .cloned()
                .ok_or(ProofBearingRefusal::FundingCreatedNoPredecessor)?;

            let disturbed = match control {
                ProofBearingConstructionControl::SpentValueAsTheCeremonysExpectation => {
                    ConfidentialInputIntent::new(
                        first.outpoint(),
                        first.observed_asset(),
                        // The amount the ceremony asked for, in the
                        // place the node's commitment belongs.
                        ValueField::Explicit(PREDECESSOR_AMOUNTS[0]),
                        first.observed_program().to_vec(),
                        first.sequence(),
                        // A registered input, so its opening is
                        // present. Carried as a refusal rather than
                        // unwrapped: this control mutates ONE fact and
                        // an absent opening would mean it had mutated
                        // two.
                        first
                            .opening()
                            .ok_or(ProofBearingRefusal::OpeningsAreNotDerived)?
                            .clone(),
                        first.explicit_amount(),
                        *first.zero_asset_blinder(),
                    )
                }
                ProofBearingConstructionControl::OpeningOfTheOtherOutput => {
                    ConfidentialInputIntent::new(
                        first.outpoint(),
                        first.observed_asset(),
                        first.observed_value(),
                        first.observed_program().to_vec(),
                        first.sequence(),
                        // The other output's opening, under the same
                        // handle and the same digest.
                        FixtureOpeningReference::new(
                            predecessor_handle().as_str().to_owned(),
                            *predecessor.digest.bytes(),
                            1,
                        ),
                        first.explicit_amount(),
                        *first.zero_asset_blinder(),
                    )
                }
            };
            intent_inputs[0] = disturbed;

            let intent = ConfidentialConstructionIntent::new(
                intent_inputs,
                self.intent(false)?.destinations().to_vec(),
                NonProtocolFundingRegion::default(),
                materialization_profiles(),
                CANDIDATE_VERSION,
                CANDIDATE_LOCK_TIME,
            );
            let outcome = materialize_confidential_candidate(
                &intent,
                &self.view()?,
                &ReferenceConfidentialMaterializer::new(),
                &FirstPartyCommitmentCheck::new(),
            );
            let Err(refusal) = outcome else {
                return Err(ProofBearingRefusal::ConstructionControlWasNotRefused);
            };
            refusals.push(ProofBearingConstructionRefusal {
                control: *control,
                refusal,
            });
        }
        self.record.construction_refusals = refusals;
        Ok(())
    }

    /// One case's submitted bytes and the message its signatures were
    /// taken over.
    fn case_bytes(
        &self,
        case: ProofBearingCase,
    ) -> Result<(Vec<u8>, Digest32), ProofBearingRefusal> {
        // The candidate that is SUBMITTED is the same one in every case:
        // what varies is the message the signatures were taken over. A
        // control that also changed the bytes would leave a refusal
        // attributable to two things.
        let materialized = self.materialize(false)?;
        let census = self.census(&materialized)?;

        // The another-candidate control signs the exchanged
        // materialization's message, so its census is built from that
        // other candidate and never from this one.
        let other = match case {
            ProofBearingCase::AnotherProofBearingCandidate => Some(self.materialize(true)?),
            _ => None,
        };
        let signing_census = match other.as_ref() {
            Some(materialized) => Some(self.census(materialized)?),
            None => None,
        };
        let source = signing_census.as_ref().unwrap_or(&census);

        let frozen = materialized.materialized().proof_finalized().protected();
        let mut witnesses = Vec::with_capacity(self.owners.len());
        let mut first_message = None;
        for (index, owner) in self.owners.iter().enumerate() {
            let position = u32::try_from(index).unwrap_or(u32::MAX);
            let input = source
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == position)
                .ok_or(ProofBearingRefusal::CensusRefused(
                    OwnerCensusRefusal::NoSigningInputRequested,
                ))?;
            let message = candidate_owner_message(source, input, case.treatment());
            if first_message.is_none() {
                first_message = Some(message);
            }
            let material = signing_material(&owner.scalar)
                .map_err(|_| ProofBearingRefusal::SubstrateUnavailable)?;
            let signature = material
                .sign(&message, &OBSERVATION_AUXILIARY)
                .map_err(|_| ProofBearingRefusal::SigningRefused)?
                .to_vec();
            witnesses.push(owner.witness(signature));
        }
        let message = first_message.ok_or(ProofBearingRefusal::CensusRefused(
            OwnerCensusRefusal::NoSigningInputRequested,
        ))?;

        // The wire form: the frozen candidate's own protected members,
        // with the input witnesses the owners produced. Nothing the
        // freeze protects is touched — the outputs, the nonces, the
        // output-witness vector, the version and the lock time are the
        // materializer's, taken from the frozen value rather than
        // rebuilt.
        let submitted = TargetTransaction::with_output_witnesses(
            frozen.version(),
            frozen.inputs().to_vec(),
            frozen.outputs().to_vec(),
            frozen.lock_time(),
            witnesses,
            frozen.output_witnesses().to_vec(),
        )
        .map_err(|_| {
            ProofBearingRefusal::CensusRefused(
                OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates,
            )
        })?;

        Ok((submitted.encode(), message))
    }

    /// Record what the target did with one case.
    fn settle_case(
        &mut self,
        position: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), ProofBearingRefusal> {
        let pending = self
            .pending
            .take()
            .ok_or(ProofBearingRefusal::FundingCreatedNoPredecessor)?;
        let case = ProofBearingCase::ALL
            .get(position)
            .copied()
            .ok_or(ProofBearingRefusal::FundingCreatedNoPredecessor)?;
        if case != pending.case {
            return Err(ProofBearingRefusal::FundingCreatedNoPredecessor);
        }

        self.record.observations.push(ProofBearingObservation {
            case,
            layer: response.observed_layer,
            detail: response.observed_detail.clone(),
            accepted_txid: response.accepted_txid.clone(),
            submitted_bytes: pending.bytes.len(),
        });

        // The second origin runs for an acceptance and for nothing else.
        if matches!(response.observed_layer, ObservedOutcomeLayer::Accepted) {
            let readback = response
                .mined_readback
                .as_ref()
                .ok_or(ProofBearingRefusal::AcceptanceCarriedNoReadback)?;
            let record = self.reverify(readback, &pending.bytes)?;
            self.record.reverification = Some(record);
        }
        Ok(())
    }

    /// Read the accepted witness back and verify it against a
    /// recomputed message.
    fn reverify(
        &self,
        readback: &MinedFundingReadback,
        submitted: &[u8],
    ) -> Result<ProofBearingReverification, ProofBearingRefusal> {
        let decoded = TargetTransaction::decode(&readback.raw_transaction)
            .map_err(|_| ProofBearingRefusal::ReadbackDidNotDecode)?;
        let signature = decoded
            .witnesses()
            .first()
            .and_then(|witness| witness.stack().first())
            .cloned()
            .ok_or(ProofBearingRefusal::ReadbackCarriesNoWitness)?;

        // The message recomputed from this workspace's own construction,
        // over a census rebuilt from the same observed inputs. The
        // signature is the target's copy and the message is ours, so the
        // two agreeing is two computations meeting.
        let materialized = self.materialize(false)?;
        let census = self.census(&materialized)?;
        let input = census
            .signing_inputs()
            .first()
            .ok_or(ProofBearingRefusal::CensusRefused(
                OwnerCensusRefusal::NoSigningInputRequested,
            ))?;
        let recomputed = candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
        let emptied =
            candidate_owner_message(&census, input, WitnessVectorTreatment::OutputsEmptied);

        let target = reviewed_target().map_err(|_| ProofBearingRefusal::SubstrateUnavailable)?;
        let owner = &self.owners[0].owner_bytes;
        let verified = verify_owner_signature(&target, owner, &recomputed, &signature);
        let verifies_against_emptied_vector_message =
            verify_owner_signature(&target, owner, &emptied, &signature).is_ok();

        Ok(ProofBearingReverification {
            accepted_txid: readback.transaction_id.clone(),
            witness_txid: readback.witness_transaction_id.clone(),
            block_height: readback.block_height,
            readback_matches_submission: readback.raw_transaction == submitted,
            recomputed_message: recomputed,
            signature_from_readback: signature,
            verified,
            verifies_against_emptied_vector_message,
        })
    }

    /// Record the authorized candidate's proof-bearing shape.
    ///
    /// The deliverable's own figures: the output-witness vector's real
    /// length, and how many proof bytes each entry carries. Written once
    /// the candidate exists and before anything is submitted.
    fn settle_candidate_shape(&mut self) -> Result<(), ProofBearingRefusal> {
        let materialized = self.materialize(false)?;
        let frozen = materialized.materialized().proof_finalized().protected();
        self.record.output_witness_vector_length = Some(frozen.output_witnesses().len());
        self.record.output_witness_proof_bytes = frozen
            .output_witnesses()
            .iter()
            .map(|witness| witness.range_proof().len())
            .collect();
        Ok(())
    }
}

impl TargetOperationPlanner for ProofBearingObservationPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    let Some(printed) = response.issued_asset.clone() else {
                        return Err(self.refuse(ProofBearingRefusal::IssuanceNamedNoAsset));
                    };
                    if let Err(refusal) = self.settle_asset(&printed) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund;
                }
                Stage::Fund => {
                    if let Err(refusal) = self
                        .settle_funding(response)
                        .and_then(|()| self.settle_candidate_shape())
                        .and_then(|()| self.exercise_construction_controls())
                    {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Submit(0);
                }
                Stage::Submit(position) => {
                    if let Err(refusal) = self.settle_case(position, response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = if position + 1 < ProofBearingCase::ALL.len() {
                        Stage::Submit(position + 1)
                    } else {
                        Stage::Done
                    };
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(OperationStep::new(
                ISSUE_STEP,
                OperationSubject::Funding(Box::new(TargetFundingSubject {
                    issue_asset: true,
                    asset: None,
                    output_program: ISSUE_PROGRAM.to_vec(),
                    outputs: ISSUE_OUTPUTS,
                    amount_per_output: ISSUE_AMOUNT_PER_OUTPUT,
                })),
            ))),
            Stage::Fund => match self.funding_step() {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Submit(position) => {
                let case = ProofBearingCase::ALL[position];
                match self.case_bytes(case) {
                    Ok((bytes, message)) => {
                        self.record.candidate_messages.insert(case, message);
                        self.pending = Some(PendingCase {
                            case,
                            bytes: bytes.clone(),
                        });
                        Ok(Some(OperationStep::new(
                            case.name(),
                            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                                transaction_bytes: bytes,
                            })),
                        )))
                    }
                    Err(refusal) => Err(self.refuse(refusal)),
                }
            }
            Stage::Done => Ok(None),
        }
    }
}

// --- The artifact ------------------------------------------------------

/// The second origin's lines, or the one line saying there was none.
///
/// Split out because the renderer is a census of a run and a census
/// that outgrew its own reading is one nobody checks. Nothing here
/// decides anything: every line is a value the comparison already
/// carried.
fn reverification_lines(record: &ProofBearingObservationRecord) -> Vec<String> {
    let Some(check) = record.reverification() else {
        return vec!["reverification none".to_owned()];
    };

    let mut lines = vec![
        format!("reverification accepted_txid {}", check.accepted_txid()),
        format!("reverification witness_txid {}", check.witness_txid()),
        format!("reverification block_height {}", check.block_height()),
        format!(
            "reverification readback_matches_submission {}",
            check.readback_matches_submission()
        ),
        format!(
            "reverification recomputed_message {}",
            printed(check.recomputed_message().as_slice())
        ),
        format!(
            "reverification signature_from_readback {}",
            printed(check.signature_from_readback())
        ),
        format!(
            "reverification verifies_against_recomputed_message {}",
            check.verified().is_ok()
        ),
    ];
    if let Err(rejection) = check.verified() {
        lines.push(format!("reverification rejection {rejection:?}"));
    }
    lines.push(format!(
        "reverification verifies_against_emptied_vector_message {}",
        check.verifies_against_emptied_vector_message()
    ));
    lines
}

/// One run's transcript, as the report artifact carries it.
///
/// Lines rather than a structure, on the pattern the existing native
/// lanes set. Every line is a fact the run observed or a value it
/// computed, and no line is a verdict about whether the run went well.
#[must_use]
pub fn render_proof_bearing_observation(record: &ProofBearingObservationRecord) -> String {
    let mut lines = vec!["role owner-sighash-proof-bearing-observation-run".to_owned()];
    lines.push(format!(
        "issued_asset {}",
        record.issued_asset().unwrap_or("none")
    ));
    lines.push(format!(
        "predecessor_fixture_digest {}",
        record
            .predecessor_digest()
            .map_or_else(|| "none".to_owned(), |digest| printed(digest)),
    ));

    for (index, coin) in record.coins().iter().enumerate() {
        lines.push(format!(
            "coin {index} value {} program_bytes {} predecessor_rangeproof_bytes {} \
             node_fields_match_expectation {}",
            match coin.value() {
                ValueField::Commitment(_) => "commitment",
                ValueField::Explicit(_) => "explicit",
                // The field vocabulary is open and this rendering is
                // not. A form this ceremony does not produce is a line
                // worth reading rather than a panic in a renderer.
                _ => "unrecognized",
            },
            coin.program().len(),
            coin.rangeproof_bytes(),
            coin.matches_expectation(),
        ));
    }

    lines.push(format!(
        "output_witness_vector_length {}",
        record
            .output_witness_vector_length()
            .map_or_else(|| "none".to_owned(), |length| length.to_string()),
    ));
    for (index, bytes) in record.output_witness_proof_bytes().iter().enumerate() {
        lines.push(format!("output_witness {index} rangeproof_bytes {bytes}"));
    }
    for (index, prefix) in record.spent_value_prefixes().iter().enumerate() {
        lines.push(format!("spent_value_prefix {index} {prefix:#04x}"));
    }

    for refusal in record.construction_refusals() {
        lines.push(format!(
            "construction_control {} moved_term {} refusal {:?}",
            refusal.control().name(),
            refusal.control().moved_term(),
            refusal.refusal(),
        ));
    }

    for (case, message) in record.candidate_messages() {
        lines.push(format!(
            "message {} {}",
            case.name(),
            printed(message.as_slice())
        ));
    }

    for observation in record.observations() {
        lines.push(format!(
            "observed {} negative_control {} moved_term {} layer {:?} txid {} \
             submitted_bytes {} detail {}",
            observation.case().name(),
            observation.case().is_negative_control(),
            observation.case().moved_term(),
            observation.layer(),
            observation.accepted_txid().unwrap_or("none"),
            observation.submitted_bytes(),
            observation.detail().unwrap_or("none"),
        ));
    }

    lines.extend(reverification_lines(record));

    lines.push(format!(
        "observed_acceptance {}",
        record
            .observations()
            .iter()
            .any(|observation| matches!(observation.layer(), ObservedOutcomeLayer::Accepted))
    ));

    if let Some(refusal) = record.refusal() {
        lines.push(format!("ceremony_refused {refusal:?}"));
    }

    for claim in ProofBearingObservationRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }

    // The standings this run does not touch, written into the artifact
    // rather than left to a reader's memory of which wave owns which.
    lines.push("evidences_the_other_guides_funding false".to_owned());
    lines.push("evidences_the_other_guides_materialization false".to_owned());
    lines.push("evidences_the_other_guides_blinding false".to_owned());
    lines.push("evidences_the_receipt_covenant false".to_owned());
    lines.push("discharges_no_matrix_row true".to_owned());

    lines.join("\n") + "\n"
}

// --- The run of record -------------------------------------------------

/// The issued asset the run of record was funded against.
const RECORDED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// The digest the predecessor fixture was registered under, on the run
/// of record.
const RECORDED_PREDECESSOR_DIGEST: &str =
    "00da5ef7aaef159237ef5479b419abeee6307e913cc4e244f3226c64c5489262";

/// How many range-proof bytes each output-witness entry carried.
///
/// The single figure that separates this lane from the explicit one,
/// where every entry is an empty surjection proof and an empty range
/// proof — two bytes in total. Four thousand one hundred and
/// seventy-four bytes of it are in the message the owner signed, and
/// none of them is recoverable from the witnessless serialization at any
/// length.
const RECORDED_RANGEPROOF_BYTES: usize = 4174;

/// The serialized prefixes the two spent value commitments carried.
///
/// The two the target admits, one square and one non-square, which is
/// what the other guide's bounded parity search settles on. They are
/// written down here because the accepted message hashed them; what they
/// establish about the target's reading of a confidential value field is
/// that guide's question and not this one's.
const RECORDED_SPENT_VALUE_PREFIXES: [u8; 2] = [0x08, 0x09];

/// The identity the target computed over the bytes it accepted.
const RECORDED_ACCEPTED_TXID: &str =
    "a176394a67fa839058b4efbba53f59af47899dab718a85251a1106f496671d86";

/// The witness identity the target reported for it.
const RECORDED_WITNESS_TXID: &str =
    "2ddc8694242459a9e65d60616f9c4133f8eacf10332ef008319004bf85241c5e";

/// The height the target confirmed it at.
const RECORDED_BLOCK_HEIGHT: u32 = 6;

/// The message the accepted authorization was taken over.
const RECORDED_ACCEPTED_MESSAGE: &str =
    "c7931addeeefa3e4ac4b67c9ee5cb5ab65f6de9409007e2415e8c749c61bd27f";

/// The signature as it stood in the target's own copy.
const RECORDED_SIGNATURE: &str = "582cc46a31e0111a7894a702d976741fb16711500bdcd719dcc16e211e89e39c\
ebcdbb8675319b48b2e04fae3ca4fbcaa4030aa618f8d2b6084a6f45ba9b7408";

/// The node's own words for every refused control.
///
/// All three drew the same sentence, which is the result rather than a
/// simplification: each control moved a different term of the message
/// and the target's answer to a message it did not form is one answer.
const RECORDED_REFUSAL_DETAIL: &str =
    "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";

/// The messages each submitted case's signatures were taken over.
///
/// Four distinct digests, in case order. They are committed because
/// their DISTINCTNESS is what gives the three refusals content: a run in
/// which two of them had coincided would have offered one control twice
/// and reported it as two.
const RECORDED_MESSAGES: [(ProofBearingCase, &str); 4] = [
    (
        ProofBearingCase::ProofBearingVectorEmptied,
        "16a454854658413c97c281b1f20c84cead7ee08cdf06ed0e0a8b19bb0de47c4d",
    ),
    (
        ProofBearingCase::PreimageOnlySigner,
        "94c5ed3bc5734102a6fa6949d54355dcced4763af838892ccf38868cbe8ed042",
    ),
    (
        ProofBearingCase::AnotherProofBearingCandidate,
        "7f56f606a9678dab6158e913778cbe8c6612d9aaab8b4fe5052f46ecd075f7a4",
    ),
    (ProofBearingCase::SelectedProfile, RECORDED_ACCEPTED_MESSAGE),
];

/// The identity of the run of record, in the shape the reviewed
/// contract already names an observation by.
///
/// The explicit lane's run is named this way where the six established
/// dimensions cite it, and the Wave-4 audit found the gap on this side:
/// a ceremony whose evidence was a file at a path an operator chose
/// named nothing a later reader could cite. So this run carries the same
/// three members — the ceremony's own name for the case, the identity
/// the TARGET computed, and where the run is recorded.
///
/// It is deliberately NOT added to the reviewed contract's dimension
/// table. Those six dimensions are established, they were established on
/// the explicit run, and a second identity beside them would read as a
/// second establishment of things this run did not re-establish.
pub const PROOF_BEARING_OBSERVATION: ObservationIdentity = ObservationIdentity::new(
    "selected-profile-proof-bearing-authorization",
    RECORDED_ACCEPTED_TXID,
    "plans/backlog.md T5-031",
);

/// The proof-bearing observation this wave produced, committed.
///
/// # Why a committed record and not only a report file
///
/// The Wave-4 audit of the explicit lane found the gap: that ceremony
/// wrote its transcript to a path an operator names and committed
/// nothing, so the evidence a later reader is asked to rely on lived
/// outside the tree and could not be diffed, cited, or checked by any
/// test. The evidence lane's own doctrine already admits the repair —
/// the live-transfer lane commits [`crate::live_native::observed_run_of_record`]
/// as a function returning one run's transcript — so this follows that
/// pattern rather than inventing one.
///
/// # What it deliberately omits
///
/// The outpoints. They name a chain that was created and destroyed by
/// the run, and a record carrying them would look like something a later
/// run could resume from. The transaction identity is kept, because it
/// is the identity the TARGET computed over the bytes it accepted and it
/// is what a report cites; it resumes nothing.
///
/// # What it is not
///
/// It is not evidence by being here. A committed transcript is a
/// transcription of one run against one destroyed chain, and re-running
/// the ceremony is what produces another. Nothing in this function
/// establishes anything, and the test below asserts its shape rather
/// than its truth.
#[must_use]
pub fn observed_proof_bearing_run_of_record() -> ProofBearingObservationRecord {
    let commitment = || ObservedConfidentialCoin {
        // The outpoints are absent by the paragraph above, and the
        // record carries a placeholder rather than a real one so that a
        // reader cannot mistake it for a coin.
        outpoint: Outpoint::new(Txid::from_internal([0_u8; 32]), 0)
            .unwrap_or_else(|_| unreachable!("index zero is in range")),
        asset: AssetField::Explicit(AssetId::from_internal([0_u8; 32])),
        value: ValueField::Commitment([0_u8; COMMITMENT_BYTES]),
        program: vec![0_u8; TAPROOT_PROGRAM_BYTES + 2],
        rangeproof_bytes: RECORDED_RANGEPROOF_BYTES,
        matches_expectation: true,
    };

    let observation = |case: ProofBearingCase| ProofBearingObservation {
        case,
        layer: if matches!(case, ProofBearingCase::SelectedProfile) {
            ObservedOutcomeLayer::Accepted
        } else {
            ObservedOutcomeLayer::ScriptPathRejection
        },
        detail: if matches!(case, ProofBearingCase::SelectedProfile) {
            None
        } else {
            Some(RECORDED_REFUSAL_DETAIL.to_owned())
        },
        accepted_txid: if matches!(case, ProofBearingCase::SelectedProfile) {
            Some(RECORDED_ACCEPTED_TXID.to_owned())
        } else {
            None
        },
        submitted_bytes: RECORDED_SUBMITTED_BYTES,
    };

    ProofBearingObservationRecord {
        issued_asset: Some(RECORDED_ASSET.to_owned()),
        predecessor_digest: recorded_digest(RECORDED_PREDECESSOR_DIGEST),
        coins: vec![commitment(), commitment()],
        output_witness_vector_length: Some(2),
        output_witness_proof_bytes: vec![RECORDED_RANGEPROOF_BYTES, RECORDED_RANGEPROOF_BYTES],
        spent_value_prefixes: RECORDED_SPENT_VALUE_PREFIXES.to_vec(),
        observations: ProofBearingCase::ALL
            .iter()
            .copied()
            .map(observation)
            .collect(),
        construction_refusals: Vec::new(),
        reverification: Some(ProofBearingReverification {
            accepted_txid: RECORDED_ACCEPTED_TXID.to_owned(),
            witness_txid: RECORDED_WITNESS_TXID.to_owned(),
            block_height: RECORDED_BLOCK_HEIGHT,
            readback_matches_submission: true,
            recomputed_message: recorded_digest(RECORDED_ACCEPTED_MESSAGE).unwrap_or([0_u8; 32]),
            signature_from_readback: recorded_bytes(RECORDED_SIGNATURE),
            verified: Ok(()),
            verifies_against_emptied_vector_message: false,
        }),
        candidate_messages: RECORDED_MESSAGES
            .iter()
            .filter_map(|(case, digest)| recorded_digest(digest).map(|digest| (*case, digest)))
            .collect(),
        refusal: None,
    }
}

/// How many bytes each case handed the node, on the run of record.
///
/// The same figure for every case, which is the point: the SUBMITTED
/// candidate is one candidate and only the message its signatures were
/// taken over varies, so a refusal is attributable to the term that
/// moved rather than to a different transaction.
const RECORDED_SUBMITTED_BYTES: usize = 8993;

/// One recorded digest, from the order this workspace prints them in.
fn recorded_digest(text: &str) -> Option<Digest32> {
    <[u8; 32]>::try_from(recorded_bytes(text).as_slice()).ok()
}

/// One recorded byte string.
fn recorded_bytes(text: &str) -> Vec<u8> {
    decode_hex(text).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_owner_leaves_are_two_distinct_single_leaf_trees() {
        // The instrument, checked before any run uses it. Two owners,
        // two programs, two leaves, and one control block shape with no
        // path — a single leaf is its own merkle root.
        let first = OwnerLeaf::derive(&FIRST_SCALAR).expect("the first owner's leaf builds");
        let second = OwnerLeaf::derive(&SECOND_SCALAR).expect("the second owner's leaf builds");

        assert_ne!(first.program, second.program);
        assert_ne!(first.tapleaf_hash, second.tapleaf_hash);
        for owner in [&first, &second] {
            assert_eq!(owner.control_block.len(), CONTROL_BASE_BYTES);
            assert_eq!(owner.leaf_script.len(), 34);
            assert_eq!(owner.leaf_script[0], OP_PUSHBYTES_32);
            assert_eq!(owner.leaf_script[33], OP_CHECKSIG);
            assert_eq!(owner.control_block[1..], UNSPENDABLE_INTERNAL_KEY);
        }
    }

    #[test]
    fn every_case_and_control_names_the_term_it_moves() {
        // The report renders the moved term per case, so a case added
        // without one would be a control nobody can attribute a refusal
        // to. Checked rather than left to review.
        for case in ProofBearingCase::ALL {
            assert_ne!(case.name(), "");
            assert_ne!(case.moved_term(), "");
        }
        for control in ProofBearingConstructionControl::ALL {
            assert_ne!(control.name(), "");
            assert_ne!(control.moved_term(), "");
        }

        // Exactly one case is not a control, and it is the accepted one.
        let positives: Vec<_> = ProofBearingCase::ALL
            .iter()
            .filter(|case| !case.is_negative_control())
            .collect();
        assert_eq!(positives, vec![&ProofBearingCase::SelectedProfile]);
    }

    #[test]
    fn the_four_cases_ask_for_four_different_witness_treatments_where_they_differ() {
        // The two vector controls must not coincide: one moves the
        // output side alone and the other moves both, and a ceremony
        // that had given them the same treatment would be submitting one
        // control twice.
        assert_ne!(
            ProofBearingCase::ProofBearingVectorEmptied.treatment(),
            ProofBearingCase::PreimageOnlySigner.treatment(),
        );
        assert_eq!(
            ProofBearingCase::SelectedProfile.treatment(),
            WitnessVectorTreatment::BothGrown,
        );
    }

    #[test]
    fn the_record_says_what_belongs_to_the_other_guide() {
        // Deliverable six, carried as data. The funding, the
        // materialization and the blinding are named as the other
        // guide's in the record itself, so a report that dropped the
        // sentence would be dropping a value.
        let claims = ProofBearingObservationRecord::non_claims();
        for owed in ["funding", "materialization", "blinding"] {
            assert!(
                claims.iter().any(|claim| claim.contains(owed)),
                "the record does not disclaim {owed}",
            );
        }

        let rendered = render_proof_bearing_observation(&ProofBearingObservationRecord::default());
        assert!(rendered.contains("evidences_the_other_guides_funding false"));
        assert!(rendered.contains("evidences_the_other_guides_materialization false"));
        assert!(rendered.contains("evidences_the_other_guides_blinding false"));
        assert!(rendered.contains("evidences_the_receipt_covenant false"));
    }

    #[test]
    fn the_run_of_record_carries_a_proof_bearing_acceptance_and_three_refusals() {
        // The committed transcript's shape, asserted rather than its
        // truth: what a test can check here is that the record says what
        // a run of this ceremony says, in the places a later reader will
        // cite. Re-running the ceremony is what produces another run.
        let record = observed_proof_bearing_run_of_record();

        assert_eq!(record.observations().len(), ProofBearingCase::ALL.len());
        let accepted: Vec<_> = record
            .observations()
            .iter()
            .filter(|observation| matches!(observation.layer(), ObservedOutcomeLayer::Accepted))
            .map(ProofBearingObservation::case)
            .collect();
        assert_eq!(accepted, vec![ProofBearingCase::SelectedProfile]);

        // The vector was at its real proof-bearing length and every
        // entry carried a proof. On the explicit lane the same figure is
        // two bytes for the whole entry.
        assert_eq!(record.output_witness_vector_length(), Some(2));
        assert_eq!(
            record.output_witness_proof_bytes(),
            [RECORDED_RANGEPROOF_BYTES, RECORDED_RANGEPROOF_BYTES],
        );

        // The four messages are four, which is what gives the three
        // refusals content.
        let messages = record.candidate_messages();
        assert_eq!(messages.len(), ProofBearingCase::ALL.len());
        let distinct: std::collections::BTreeSet<_> = messages.values().collect();
        assert_eq!(distinct.len(), ProofBearingCase::ALL.len());

        // The two origins met, and the accepted witness does not verify
        // against the emptied-vector candidate.
        let check = record.reverification().expect("the run accepted a case");
        assert!(check.verified().is_ok());
        assert!(check.readback_matches_submission());
        assert!(!check.verifies_against_emptied_vector_message());
        assert_eq!(
            check.recomputed_message().as_slice(),
            recorded_bytes(RECORDED_ACCEPTED_MESSAGE).as_slice(),
        );
        assert_eq!(check.signature_from_readback().len(), 64);
    }

    #[test]
    fn the_run_of_record_names_where_it_is_recorded() {
        // The Wave-4 audit's gap, closed in the shape the reviewed
        // contract already uses for the explicit lane's run: the case's
        // own name, the identity the target computed, and the row a
        // reader finds the run under.
        let record = observed_proof_bearing_run_of_record();
        let check = record.reverification().expect("the run accepted a case");

        assert_eq!(
            PROOF_BEARING_OBSERVATION.accepted_transaction(),
            check.accepted_txid(),
            "the identity names a transaction the record does not carry",
        );
        assert_eq!(
            PROOF_BEARING_OBSERVATION.ceremony_case(),
            ProofBearingCase::SelectedProfile.name(),
            "the identity names a case the ceremony does not submit",
        );
        assert_eq!(
            PROOF_BEARING_OBSERVATION.recorded_at(),
            "plans/backlog.md T5-031"
        );

        // Both admitted value-commitment prefixes were inside the
        // message the target accepted a signature over. Recorded, not
        // claimed: what it establishes about the confidential field form
        // belongs to the guide that owns that obligation.
        assert_eq!(record.spent_value_prefixes(), [0x08, 0x09]);
    }

    #[test]
    fn the_run_of_record_carries_no_outpoint_a_later_run_could_resume_from() {
        // The omission stated as a check. The chain was destroyed when
        // the run ended, and a record whose coins named live outpoints
        // would read as something resumable.
        let record = observed_proof_bearing_run_of_record();

        for coin in record.coins() {
            assert_eq!(coin.outpoint().txid().internal(), &[0_u8; 32]);
        }
    }

    #[test]
    fn an_empty_record_renders_without_claiming_a_run_happened() {
        let rendered = render_proof_bearing_observation(&ProofBearingObservationRecord::default());

        assert!(rendered.contains("observed_acceptance false"));
        assert!(rendered.contains("reverification none"));
        assert!(rendered.contains("output_witness_vector_length none"));
    }
}
