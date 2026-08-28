//! One owner authorization, produced against the Wave-2 message and
//! observed on the explicit lane.
//!
//! # What this module is for, in one sentence
//!
//! The selected owner sighash profile has a source review and an
//! independent recomputation, and the accepted evidence ruling requires
//! a third thing: one target-accepted spend of a first-party candidate
//! whose signature verifies against the recomputed message. This module
//! is the ceremony that asks for that third thing.
//!
//! # Nothing here establishes anything about the proof-bearing lane
//!
//! Stated first because it is the boundary a reader is most likely to
//! cross by accident, and it is a property of the *message* rather than
//! a matter of caution.
//!
//! The message's output-witness term is taken over every entry the
//! output-witness vector happens to hold. On the explicit lane every
//! entry is default-constructed — an empty surjection proof and an empty
//! range proof, two zero bytes — so the whole term is a function of the
//! output count, and the output count is in the protected bytes. A
//! component told nothing but the preimage can therefore rebuild that
//! term exactly, and that recoverability is the only reason an explicit
//! candidate can be authorized from a preimage at all.
//!
//! The proof-bearing lane's entries carry real range proofs. Nothing
//! about those bytes is present in the preimage, in any encoded form, at
//! any length, and no arithmetic recovers them. So the recoverability
//! argument this observation rests on does not hold there, and an
//! acceptance observed here says nothing whatever about a candidate
//! whose outputs carry proofs. [`OwnerObservationRecord::non_claims`]
//! carries that sentence as data rather than leaving it in this comment.
//!
//! # Why acceptance and recomputation stay two origins
//!
//! An acceptance whose message nobody recomputed establishes that some
//! signature was valid, not which dimensions it committed to. So the
//! ceremony does two separate things and keeps them apart.
//!
//! The target forms its own message with its own `SignatureHashSchnorr`
//! and checks the signature with its own verifying primitive. That is
//! the first origin and this workspace does not participate in it.
//!
//! Then the accepted transaction is read back out of the node — the
//! target's own copy, at the identity the target computed — the
//! signature is taken *from those bytes* rather than from the value the
//! builder still holds, and it is verified against a message this
//! workspace recomputes from the source review's own term table using
//! its own curve arithmetic. That is the second origin.
//!
//! The two agreeing is a claim with content: a wrong model of the
//! message would still have been accepted by the node, because the node
//! uses its own, and the first-party verification is what would fail.
//! Checking the builder's signature against the builder's message would
//! have been a value compared with itself.
//!
//! # The negative controls, and what each one is for
//!
//! A positive control alone cannot say *which* message was committed
//! to: several wrong models would produce a signature some node
//! somewhere accepts. Each control below changes exactly one term and is
//! offered to the same target on the same chain, so a refusal is
//! attributable to the term that moved.
//!
//! Two of them are refused twice, at two different layers, and the
//! ceremony records both without merging them: a construction refusal is
//! this workspace declining to build something, and a target verdict is
//! a node declining to accept it. §1.7's discipline keeps the two apart,
//! and a run that reported one as the other would be reporting a
//! first-party opinion as target evidence.
//!
//! # Every key here is published test material
//!
//! The two signing scalars are the BIP-340 specification's own appendix
//! secret keys, admitted under ADR-015's test-material rule. They
//! authorize nothing on any network anyone uses, the chain this ceremony
//! runs against is created and destroyed by the run, and nothing here is
//! custody of anything.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::owner_key_oracle::{SignatureRejection, verify_owner_signature};
use target_elements_conformance::protocol::{
    FundedOutput, MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, Txid, ValueField};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{
    LiveDeployment, OWNER_SIGNATURE_BYTES, OwnerCensusRefusal, OwnerSigningCensus,
    check_signature_width, check_type_byte,
};
use transaction::live_construct::{
    LiveConstructionReport, complete_live_transfer, finalize_live_transfer,
};
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::taproot::Digest32;
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, RESERVE_ASSET, SECOND_SCALAR, demonstration_live_abi,
    live_abi_for_asset, published_owner, reviewed_target, signing_material,
};

/// What each funded receipt holds.
///
/// The same figure the existing live-transfer lane funds with, kept
/// equal deliberately: this ceremony submits the same candidate shape
/// with a different signature in it, and a different amount would make
/// the two runs incomparable for no gain.
const RECEIPT_AMOUNT: u64 = 5_000;

/// How many receipts the ceremony funds and then consumes.
const RECEIPT_COUNT: u8 = 2;

/// The auxiliary value every signature here is taken with.
///
/// A published constant rather than randomness. BIP-340 masks the scalar
/// with it before deriving the nonce, so fixing it is what makes every
/// signature this ceremony produces reproducible from inputs that are
/// all written down — which is what the test-material rule asks of a
/// fixture wherever the scheme permits it.
const OBSERVATION_AUXILIARY: [u8; FIELD_ELEMENT_BYTES] = [0x33; FIELD_ELEMENT_BYTES];

/// The type byte the profile does not select.
///
/// `0x01` is explicit-all: the same coverage as the default, stated on
/// the wire and one byte wider. The profile ruling closed it, and this
/// constant exists so the control that carries it is spelled once.
const NON_SELECTED_TYPE_BYTE: u8 = 0x01;

/// One case the ceremony submits.
///
/// The order is the order they run in, and it is not cosmetic: a refused
/// transaction spends nothing, so every control can be offered against
/// the same pair of coins, and the one that is meant to be accepted goes
/// last because acceptance is what finally consumes them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OwnerObservationCase {
    /// The signature taken over the message with the output-witness
    /// vector empty.
    ///
    /// The recorded hazard, reproduced deliberately. A signer working
    /// from the witnessless serialization hashes the empty string in the
    /// output-witness position while consensus hashes one entry per
    /// output, and the target's own wallet does exactly this. The
    /// control is what makes the accepted case a statement about the
    /// grown vector rather than about signatures in general.
    EmptyOutputWitnessVector,
    /// The signature taken over another deployment's message.
    ///
    /// The hasher is seeded with the genesis block hash twice, so two
    /// candidates identical to the last byte have different messages on
    /// two chains. Failure-matrix row 4.
    AnotherDeployment,
    /// The witness carrying the type byte the profile does not select.
    ///
    /// The signature is the accepted case's own, with the non-selected
    /// type byte appended. The target then reads a hash type of `0x01`,
    /// forms the message that type determines, and is offered a
    /// signature over the `0x00` one. Failure-matrix row 5.
    NonSelectedTypeByte,
    /// A valid signature by a key the input does not authenticate.
    ///
    /// The owner-key obligation's `ValidSignatureAgainstAnotherKey`,
    /// exercised on this lane: the signature is well formed and verifies
    /// for the other published owner over this very message.
    AnotherOwnersKey,
    /// A valid signature over another candidate's message.
    ///
    /// The owner-key obligation's
    /// `ValidSignatureOverAnotherTransaction`. The other candidate is
    /// this one with its two destinations exchanged, so the output list
    /// differs and the message differs while everything else is held
    /// fixed.
    AnotherCandidate,
    /// The signature taken over a message seeded with the deployment's
    /// genesis hash in the order the target *prints* it.
    ///
    /// A target prints a block identity in the reverse of the order it
    /// hashes it in, and the deployment seed is the one place in this
    /// message where that distinction has no other witness: every other
    /// byte-order question in the stream is settled by a term whose
    /// encoding this crate already evidences, and this one arrives from
    /// a run's own environment as a printed string.
    ///
    /// So it is a control rather than a comment. The two orders produce
    /// two distinct messages over the same candidate on the same chain,
    /// both are offered to the same node, and which one the node accepts
    /// is a measured fact about the target's seed rather than a reading
    /// of its source.
    DeploymentSeedInPrintedOrder,
    /// The candidate authorized under the selected profile.
    ///
    /// The only case the ceremony expects a target to accept, and the
    /// only one whose acceptance is evidence of anything.
    SelectedProfile,
}

impl OwnerObservationCase {
    /// Every case, in submission order.
    pub const ALL: &'static [Self] = &[
        Self::EmptyOutputWitnessVector,
        Self::AnotherDeployment,
        Self::NonSelectedTypeByte,
        Self::AnotherOwnersKey,
        Self::AnotherCandidate,
        Self::DeploymentSeedInPrintedOrder,
        Self::SelectedProfile,
    ];

    /// The ceremony's own name for the case.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EmptyOutputWitnessVector => "control-empty-output-witness-vector",
            Self::AnotherDeployment => "control-another-deployment",
            Self::NonSelectedTypeByte => "control-non-selected-type-byte",
            Self::AnotherOwnersKey => "control-another-owners-key",
            Self::AnotherCandidate => "control-another-candidate",
            Self::DeploymentSeedInPrintedOrder => "control-deployment-seed-in-printed-order",
            Self::SelectedProfile => "selected-profile-authorization",
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
}

/// The historical-v1 T5-026 run, in the vocabulary of this ceremony.
///
/// Every value below is recovered from the ceremony, the Wave-3 records,
/// and the closed T5-026 backlog row. The six controls were refused at the
/// script path with invalid Schnorr signatures; the selected profile was
/// accepted and reverified from the target's readback at the identity below.
///
/// # Rerun drift fails closed
///
/// These values are an immutable record, not defaults for the next run. A
/// future rerun that differs must fail the native binding and remain failed
/// pending owner review. It must not advance this record automatically or
/// rewrite the historical T5-026 result. New historical callers use
/// [`crate::live_history_v1::owner_observation`]; this compatibility path
/// remains for the separately owned native-guide cleanup.
pub mod run_of_record {
    use super::{ObservedOutcomeLayer, OwnerObservationCase};

    /// Whether the recorded case was refused or accepted.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ExpectedAcceptance {
        /// The target refused the submitted case and returned no accepted
        /// transaction identity.
        Refused,
        /// The target accepted the case at one identity and the subsequent
        /// reverification named the target's readback by identity.
        Accepted {
            /// The identity returned by the submission.
            identity: &'static str,
            /// The identity carried by the reverification record.
            reverification_identity: &'static str,
        },
    }

    /// One case's typed expected outcome.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ExpectedCaseOutcome {
        /// The in-tree evidence records the complete expected result.
        Recorded {
            /// The typed ceremony case.
            case: OwnerObservationCase,
            /// The case name written into the transcript.
            name: &'static str,
            /// The target boundary the recorded answer reached.
            layer: ObservedOutcomeLayer,
            /// Whether the answer carried an accepted identity.
            acceptance: ExpectedAcceptance,
        },
        /// The in-tree evidence does not recover an outcome for this case.
        ///
        /// A future batched rerun may fill such a member forward after owner
        /// review; callers must never infer a value for it.
        NotRecorded {
            /// The typed ceremony case whose result is absent.
            case: OwnerObservationCase,
            /// The case name whose result is absent.
            name: &'static str,
        },
    }

    /// The identity the target computed for the selected-profile acceptance.
    pub const SELECTED_PROFILE_ACCEPTED_TXID: &str =
        "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        selected_profile_accepted,
        SELECTED_PROFILE_ACCEPTED_TXID
    );

    /// The accepted identity carried by the target readback that was
    /// reverified independently.
    pub const SELECTED_PROFILE_REVERIFICATION_IDENTITY: &str = SELECTED_PROFILE_ACCEPTED_TXID;

    /// The empty-output-witness-vector control's recorded refusal.
    pub const EMPTY_OUTPUT_WITNESS_VECTOR: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
        case: OwnerObservationCase::EmptyOutputWitnessVector,
        name: "control-empty-output-witness-vector",
        layer: ObservedOutcomeLayer::ScriptPathRejection,
        acceptance: ExpectedAcceptance::Refused,
    };

    /// The another-deployment control's recorded refusal.
    pub const ANOTHER_DEPLOYMENT: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
        case: OwnerObservationCase::AnotherDeployment,
        name: "control-another-deployment",
        layer: ObservedOutcomeLayer::ScriptPathRejection,
        acceptance: ExpectedAcceptance::Refused,
    };

    /// The non-selected-type-byte control's recorded refusal.
    pub const NON_SELECTED_TYPE_BYTE: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
        case: OwnerObservationCase::NonSelectedTypeByte,
        name: "control-non-selected-type-byte",
        layer: ObservedOutcomeLayer::ScriptPathRejection,
        acceptance: ExpectedAcceptance::Refused,
    };

    /// The another-owner's-key control's recorded refusal.
    pub const ANOTHER_OWNERS_KEY: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
        case: OwnerObservationCase::AnotherOwnersKey,
        name: "control-another-owners-key",
        layer: ObservedOutcomeLayer::ScriptPathRejection,
        acceptance: ExpectedAcceptance::Refused,
    };

    /// The another-candidate control's recorded refusal.
    pub const ANOTHER_CANDIDATE: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
        case: OwnerObservationCase::AnotherCandidate,
        name: "control-another-candidate",
        layer: ObservedOutcomeLayer::ScriptPathRejection,
        acceptance: ExpectedAcceptance::Refused,
    };

    /// The printed-order deployment-seed control's recorded refusal.
    pub const DEPLOYMENT_SEED_IN_PRINTED_ORDER: ExpectedCaseOutcome =
        ExpectedCaseOutcome::Recorded {
            case: OwnerObservationCase::DeploymentSeedInPrintedOrder,
            name: "control-deployment-seed-in-printed-order",
            layer: ObservedOutcomeLayer::ScriptPathRejection,
            acceptance: ExpectedAcceptance::Refused,
        };

    /// The selected profile's recorded acceptance and reverification.
    pub const SELECTED_PROFILE: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
        case: OwnerObservationCase::SelectedProfile,
        name: "selected-profile-authorization",
        layer: ObservedOutcomeLayer::Accepted,
        acceptance: ExpectedAcceptance::Accepted {
            identity: SELECTED_PROFILE_ACCEPTED_TXID,
            reverification_identity: SELECTED_PROFILE_REVERIFICATION_IDENTITY,
        },
    };

    /// Every T5-026 case outcome, in the ceremony's submission order.
    pub const EXPECTED_CASE_OUTCOMES: [ExpectedCaseOutcome; 7] = [
        EMPTY_OUTPUT_WITNESS_VECTOR,
        ANOTHER_DEPLOYMENT,
        NON_SELECTED_TYPE_BYTE,
        ANOTHER_OWNERS_KEY,
        ANOTHER_CANDIDATE,
        DEPLOYMENT_SEED_IN_PRINTED_ORDER,
        SELECTED_PROFILE,
    ];
}

/// Why the ceremony could not proceed.
///
/// Construction and infrastructure only. No member of this type is a
/// target verdict, and a run that stopped for one of these reasons
/// observed nothing about any node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerObservationRefusal {
    /// The reviewed target, the demonstration link, or a published owner
    /// was unavailable.
    SubstrateUnavailable,
    /// The issuance step named no asset to link against.
    IssuanceNamedNoAsset,
    /// The funding step created no coin to spend.
    FundingCreatedNoPredecessor,
    /// A funded coin's outpoint, asset, or program did not decode.
    MalformedFundedOutput,
    /// Re-linking the deployment against the issued asset was refused.
    RelinkRefused,
    /// The candidate did not construct or finalize.
    CandidateNotConstructible,
    /// The census refused the finalized candidate.
    CensusRefused(OwnerCensusRefusal),
    /// The signing material refused to produce a signature.
    SigningRefused,
    /// The node reported an acceptance with no readback to check.
    AcceptanceCarriedNoReadback,
    /// The bytes the node reported for the accepted identity did not
    /// decode here.
    ReadbackDidNotDecode,
    /// The read-back transaction carries no witness at the signing
    /// input's position.
    ReadbackCarriesNoWitness,
}

/// What one funded coin is, as the node reported it and as this
/// ceremony expected it.
///
/// Both halves are kept because the spent-output triple is a term of the
/// message. The census is *observed target data*, so the value that is
/// signed over is the node's; the expectation is recorded beside it so
/// that a divergence is a line in the report rather than a signature
/// silently taken over the wrong output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedFundedCoin {
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
    matches_expectation: bool,
}

impl ObservedFundedCoin {
    /// One coin, from what the node reported and what was expected.
    ///
    /// Crate-visible rather than private because the live-transfer
    /// planner takes its funded coins the same way, and the concept is
    /// one concept. A second type of the same shape in the other module
    /// would be a second place for the rule "the node's fields are the
    /// ones that get signed over" to be stated, and a second place for
    /// it to drift.
    ///
    /// The middle three arguments are in the order the spent-output
    /// census carries them, so a call site that transposed two of them
    /// would be transposing them in the message as well.
    pub(crate) const fn observed(
        outpoint: Outpoint,
        asset: AssetField,
        value: ValueField,
        program: Vec<u8>,
        matches_expectation: bool,
    ) -> Self {
        Self {
            outpoint,
            asset,
            value,
            program,
            matches_expectation,
        }
    }

    /// The outpoint the node created.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The asset field the node reported for it.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The value field the node reported for it.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The program the node reported for it.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }

    /// Whether the node's three fields are the ones the ceremony asked
    /// for.
    #[must_use]
    pub const fn matches_expectation(&self) -> bool {
        self.matches_expectation
    }
}

/// A refusal this workspace produced before any node saw the case.
///
/// Carried per case and never merged with the target's answer. Two
/// controls are refusable on this side as well as on the target's, and
/// the whole point of naming the layer is that "we declined to build it"
/// and "the node declined to accept it" are different facts about
/// different things.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructionRefusal {
    clause: &'static str,
    refusal: OwnerCensusRefusal,
}

impl ConstructionRefusal {
    /// Which clause of the profile refused.
    #[must_use]
    pub const fn clause(&self) -> &'static str {
        self.clause
    }

    /// The typed refusal it produced.
    #[must_use]
    pub const fn refusal(&self) -> &OwnerCensusRefusal {
        &self.refusal
    }
}

/// What the target did with one submitted case.
///
/// The layer and the node's own text, recorded and never graded. No
/// function in this module compares an observed layer against an
/// expectation: the ceremony has an intention for each case, and an
/// intention is not an entitlement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseObservation {
    case: OwnerObservationCase,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    accepted_txid: Option<String>,
    submitted_bytes: usize,
    construction_refusals: Vec<ConstructionRefusal>,
}

impl CaseObservation {
    /// Which case this is about.
    #[must_use]
    pub const fn case(&self) -> OwnerObservationCase {
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

    /// Every refusal this workspace produced for the case before
    /// submitting it.
    #[must_use]
    pub fn construction_refusals(&self) -> &[ConstructionRefusal] {
        &self.construction_refusals
    }
}

/// The second origin's answer.
///
/// Every field is a fact about a comparison rather than a verdict about
/// a node. The signature came out of the target's own copy of the
/// transaction and the message came out of this workspace's own
/// construction, so agreement is two computations meeting rather than
/// one repeated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReverificationRecord {
    accepted_txid: String,
    witness_txid: String,
    block_height: u32,
    readback_matches_submission: bool,
    recomputed_message: Digest32,
    signature_from_readback: Vec<u8>,
    verified: Result<(), SignatureRejection>,
    verifies_against_empty_vector_message: bool,
}

impl ReverificationRecord {
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
    ///
    /// Recorded rather than assumed. A node is entitled to report a
    /// transaction in its own serialization, and a difference here would
    /// mean the accepted witness is not the submitted one — which is a
    /// finding about the boundary and not about the signature.
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

    /// Whether the read-back signature also verifies against the
    /// empty-vector message.
    ///
    /// Expected false, and checked rather than assumed. If a signature
    /// verified against both candidates the whole comparison would be
    /// vacuous — the recorded diagnosis checks the two candidates are
    /// distinct before reporting which one a signature matches, and this
    /// is the same check made on the accepted witness.
    #[must_use]
    pub const fn verifies_against_empty_vector_message(&self) -> bool {
        self.verifies_against_empty_vector_message
    }
}

/// Everything the ceremony recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnerObservationRecord {
    issued_asset: Option<String>,
    relinked: bool,
    coins: Vec<ObservedFundedCoin>,
    observations: Vec<CaseObservation>,
    reverification: Option<ReverificationRecord>,
    candidate_messages: BTreeMap<OwnerObservationCase, Digest32>,
    refusal: Option<OwnerObservationRefusal>,
}

impl OwnerObservationRecord {
    /// The asset identity the target chose.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// Whether the deployment was linked against that asset before
    /// anything was funded.
    #[must_use]
    pub const fn relinked(&self) -> bool {
        self.relinked
    }

    /// Every coin the funding step created, as the node reported it.
    #[must_use]
    pub fn coins(&self) -> &[ObservedFundedCoin] {
        &self.coins
    }

    /// What the target did with each submitted case.
    #[must_use]
    pub fn observations(&self) -> &[CaseObservation] {
        &self.observations
    }

    /// The second origin's answer, where an acceptance was observed.
    #[must_use]
    pub const fn reverification(&self) -> Option<&ReverificationRecord> {
        self.reverification.as_ref()
    }

    /// The message each case's signature was taken over.
    #[must_use]
    pub const fn candidate_messages(&self) -> &BTreeMap<OwnerObservationCase, Digest32> {
        &self.candidate_messages
    }

    /// Why the ceremony stopped, where it did.
    #[must_use]
    pub const fn refusal(&self) -> Option<&OwnerObservationRefusal> {
        self.refusal.as_ref()
    }

    /// What this ceremony does not claim, whatever it observed.
    ///
    /// Carried as data so that a report renders them rather than a
    /// reader remembering them, and so that a consumer that dropped one
    /// would be dropping a value rather than a paragraph.
    #[must_use]
    pub fn non_claims() -> Vec<&'static str> {
        vec![
            "establishes nothing about the proof-bearing lane: the output-witness term's \
             recoverability argument does not hold for a rangeproof",
            "establishes no dimension as reviewed: populating the sighash capability and \
             recomputing the profile's disposition is a later wave's work",
            "clears no profile residual: SighashProfileUnreviewed is cleared by a review \
             verdict and never by a run",
            "discharges no safety-matrix row: an accepted control is not a row's own relation \
             driven to its own refusal",
            "evidences no funding, materialization, or blinding: those belong to the \
             confidential-funding guide and are not reviewed here",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

/// What the plan is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset the deployment is then linked against.
    Issue,
    /// Pay the issued asset to the re-linked explicit constructor.
    Fund,
    /// Submit the case at this position of the census.
    Submit(usize),
    /// Nothing further.
    Done,
}

/// The Wave-3 observation ceremony.
pub struct OwnerObservationPlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    explicit_program: Vec<u8>,
    genesis_block_hash: Digest32,
    /// The case that has been handed to the executor and not yet
    /// answered, with the exact bytes it was handed.
    ///
    /// Held because the answer arrives on the next call and the
    /// comparison the acceptance owes is against *those* bytes. A
    /// ceremony that rebuilt the candidate to compare against would be
    /// comparing the node's copy with a second construction rather than
    /// with what it was given.
    pending: Option<PendingCase>,
    record: OwnerObservationRecord,
}

/// One submitted case, held until its answer arrives.
struct PendingCase {
    case: OwnerObservationCase,
    bytes: Vec<u8>,
    refusals: Vec<ConstructionRefusal>,
}

impl OwnerObservationPlanner {
    /// The ceremony bound to one deployment's genesis block hash.
    ///
    /// The genesis hash is a constructor argument rather than something
    /// read from a candidate, because nothing about a candidate
    /// determines it: the target seeds its message hasher with it twice,
    /// and two identical candidates on two chains have different
    /// messages. It arrives from the run's own deployment binding.
    ///
    /// # The argument is the printed identity, and the seed is not
    ///
    /// A deployment binding carries the identity in the spelling the
    /// target prints, because that is the spelling a run's environment
    /// reports and the spelling the binding is compared against. A
    /// target prints a block identity in the reverse of the order it
    /// hashes it in, so what the message is seeded with is the reverse
    /// of what arrives here — the same relation
    /// [`crate::live_native`]'s outpoint and asset readers already
    /// apply, applied at the one term of this message that carries a
    /// value a run printed.
    ///
    /// The reversal is not asserted. `DeploymentSeedInPrintedOrder`
    /// offers the other order to the same node on the same chain, so
    /// which order the target seeds with is something this ceremony
    /// observes.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI
    /// or the explicit destination constructor is unavailable.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        let explicit_program = explicit_destination_program(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            abi,
            explicit_program,
            genesis_block_hash: printed_order(printed_genesis_identity),
            pending: None,
            record: OwnerObservationRecord::default(),
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &OwnerObservationRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: OwnerObservationRefusal) -> PlanRefused {
        self.record.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The funding step for the explicit constructor's program.
    fn funding_step(&self, name: &str, issue: bool) -> OperationStep {
        OperationStep::new(
            name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.record.issued_asset.clone()
                },
                output_program: self.explicit_program.clone(),
                outputs: RECEIPT_COUNT,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        )
    }

    /// Link the deployment against the asset the target issued.
    fn relink(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), OwnerObservationRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(OwnerObservationRefusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(OwnerObservationRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(*identity.internal(), RESERVE_ASSET, FEE_PROGRAM_DIGEST)
            .map_err(|_| OwnerObservationRefusal::RelinkRefused)?;
        self.explicit_program = explicit_destination_program(&abi)
            .map_err(|_| OwnerObservationRefusal::RelinkRefused)?;
        self.record.issued_asset = Some(asset);
        self.record.relinked = true;
        self.abi = abi;
        Ok(())
    }

    /// Take the funded coins from the node's own report of them.
    ///
    /// Every field is the node's. The ceremony's expectation is compared
    /// against it and recorded, and it is the node's answer that is
    /// carried forward: the spent asset, value and program are terms of
    /// the message, so a signature taken over what the builder assumed
    /// rather than over what the chain holds would be a signature over a
    /// different message than the one the target forms.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), OwnerObservationRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(OwnerObservationRefusal::FundingCreatedNoPredecessor);
        }
        let expected_asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(OwnerObservationRefusal::IssuanceNamedNoAsset)?;

        let mut coins = Vec::with_capacity(response.funded_outputs.len());
        for funded in &response.funded_outputs {
            coins.push(self.observed_coin(funded, expected_asset)?);
        }
        self.record.coins = coins;
        Ok(())
    }

    /// One funded coin, as reported and as expected.
    fn observed_coin(
        &self,
        funded: &FundedOutput,
        expected_asset: AssetId,
    ) -> Result<ObservedFundedCoin, OwnerObservationRefusal> {
        let outpoint =
            outpoint_of(&funded.outpoint).ok_or(OwnerObservationRefusal::MalformedFundedOutput)?;
        let asset =
            asset_of(&funded.asset).ok_or(OwnerObservationRefusal::MalformedFundedOutput)?;
        let program =
            decode_hex(&funded.script).ok_or(OwnerObservationRefusal::MalformedFundedOutput)?;

        let matches_expectation = asset == expected_asset
            && funded.amount_satoshis == RECEIPT_AMOUNT
            && program == self.explicit_program;

        Ok(ObservedFundedCoin {
            outpoint,
            asset: AssetField::Explicit(asset),
            value: ValueField::Explicit(funded.amount_satoshis),
            program,
            matches_expectation,
        })
    }

    /// The public view the ceremony constructs against.
    fn view(&self) -> Result<PublicConstructionView, OwnerObservationRefusal> {
        PublicConstructionView::new(self.record.coins.iter().map(|coin| {
            PublicOutputView::new(coin.outpoint, coin.asset, coin.value, coin.program.clone())
        }))
        .map_err(|_| OwnerObservationRefusal::CandidateNotConstructible)
    }

    /// One finalized explicit candidate over the funded coins.
    ///
    /// `exchanged` swaps the two destinations, which is how the
    /// another-candidate control gets a second candidate that differs in
    /// its output list and in nothing else.
    fn finalize(
        &self,
        exchanged: bool,
    ) -> Result<(FinalizedLiveTransfer, LiveConstructionReport), OwnerObservationRefusal> {
        let view = self.view()?;
        let points: Vec<Outpoint> = self.record.coins.iter().map(|coin| coin.outpoint).collect();
        let total = RECEIPT_AMOUNT
            .checked_mul(u64::try_from(points.len()).unwrap_or(0))
            .ok_or(OwnerObservationRefusal::CandidateNotConstructible)?;

        let destination = |scalar: &[u8; FIELD_ELEMENT_BYTES], amount: u64| {
            let owner = published_owner(scalar)
                .map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?;
            let value = ProtocolValue::new(amount)
                .map_err(|_| OwnerObservationRefusal::CandidateNotConstructible)?;
            Ok::<_, OwnerObservationRefusal>(LiveReceiptDestination::new(
                linker::OwnerParameter::new(owner),
                value,
            ))
        };
        let first = destination(&SECOND_SCALAR, total / 2)?;
        let second = destination(&FIRST_SCALAR, total - total / 2)?;
        let destinations = if exchanged {
            [second, first]
        } else {
            [first, second]
        };

        let request = LiveTransferRequest::new(
            points,
            destinations,
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        )
        .map_err(|_| OwnerObservationRefusal::CandidateNotConstructible)?;

        let target =
            reviewed_target().map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?;
        let finalization = finalize_live_transfer(&target, &self.abi, &request, &view, None, None)
            .map_err(|_| OwnerObservationRefusal::CandidateNotConstructible)?;
        let report = finalization.report().clone();
        Ok((finalization.into_finalized(), report))
    }

    /// The census of one finalized explicit candidate, under one
    /// deployment.
    fn census(
        finalized: &FinalizedLiveTransfer,
        genesis: Digest32,
    ) -> Result<OwnerSigningCensus, OwnerObservationRefusal> {
        let target =
            reviewed_target().map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?,
        );
        OwnerSigningCensus::from_explicit_finalized(
            &target,
            finalized,
            LiveDeployment::new(genesis),
            &curve,
        )
        .map_err(OwnerObservationRefusal::CensusRefused)
    }

    /// One case's submitted bytes, and the construction refusals it
    /// produced on the way.
    fn case_bytes(
        &self,
        case: OwnerObservationCase,
    ) -> Result<(Vec<u8>, Digest32, Vec<ConstructionRefusal>), OwnerObservationRefusal> {
        let (finalized, report) = self.finalize(false)?;
        let mut refusals = Vec::new();

        // The census under the deployment the case is taken against. The
        // another-deployment control is the one case whose census is a
        // different chain's, and the run's own deployment record refuses
        // it here — a construction refusal, recorded as one, and not the
        // reason the node will give.
        let genesis = match case {
            OwnerObservationCase::AnotherDeployment => another_deployment(self.genesis_block_hash),
            OwnerObservationCase::DeploymentSeedInPrintedOrder => {
                printed_order(self.genesis_block_hash)
            }
            _ => self.genesis_block_hash,
        };
        let census = Self::census(&finalized, genesis)?;
        if case == OwnerObservationCase::AnotherDeployment
            && let Err(refusal) =
                census.check_deployment(LiveDeployment::new(self.genesis_block_hash))
        {
            refusals.push(ConstructionRefusal {
                clause: "check_deployment",
                refusal,
            });
        }

        // The profile's two answer checks, run against what this case
        // offers rather than against what it should have offered. Both
        // are construction refusals and neither is a verdict.
        if case == OwnerObservationCase::NonSelectedTypeByte {
            if let Err(refusal) = check_type_byte(NON_SELECTED_TYPE_BYTE) {
                refusals.push(ConstructionRefusal {
                    clause: "check_type_byte",
                    refusal,
                });
            }
            if let Err(refusal) = check_signature_width(OWNER_SIGNATURE_BYTES + 1) {
                refusals.push(ConstructionRefusal {
                    clause: "check_signature_width",
                    refusal,
                });
            }
        }

        // The message every signature in this case is taken over. The
        // empty-vector control is the one case whose treatment differs;
        // every other case takes the message consensus forms.
        let treatment = match case {
            OwnerObservationCase::EmptyOutputWitnessVector => {
                WitnessVectorTreatment::OutputsEmptied
            }
            _ => WitnessVectorTreatment::BothGrown,
        };

        // The another-candidate control signs the exchanged candidate's
        // message and offers it here, so its census is built from that
        // other candidate and never from this one.
        let signing_census = match case {
            OwnerObservationCase::AnotherCandidate => {
                let (other, _) = self.finalize(true)?;
                Some(Self::census(&other, self.genesis_block_hash)?)
            }
            _ => None,
        };
        let source = signing_census.as_ref().unwrap_or(&census);

        let scalar = match case {
            OwnerObservationCase::AnotherOwnersKey => &SECOND_SCALAR,
            _ => &FIRST_SCALAR,
        };
        let material =
            signing_material(scalar).map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?;

        let mut first_message = None;
        let mut responses = Vec::new();
        for signing in finalized.signing_requests() {
            let input = source
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(OwnerObservationRefusal::CandidateNotConstructible)?;
            let message = candidate_owner_message(source, input, treatment);
            if first_message.is_none() {
                first_message = Some(message);
            }
            let mut signature = material
                .sign(&message, &OBSERVATION_AUXILIARY)
                .map_err(|_| OwnerObservationRefusal::SigningRefused)?
                .to_vec();
            if case == OwnerObservationCase::NonSelectedTypeByte {
                signature.push(NON_SELECTED_TYPE_BYTE);
            }
            responses.push((signing.input(), LiveOwnerResponse::to(&signing, signature)));
        }
        let message = first_message.ok_or(OwnerObservationRefusal::CandidateNotConstructible)?;

        let report_target =
            reviewed_target().map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?;
        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| OwnerObservationRefusal::CandidateNotConstructible)?;
        let built = complete_live_transfer(&report_target, authorized, report, None)
            .map_err(|_| OwnerObservationRefusal::CandidateNotConstructible)?;

        Ok((built.bytes(), message, refusals))
    }

    /// Record what the target did with one case.
    ///
    /// The observation is written down whatever the layer was. Nothing
    /// here compares it against what the case intended: the ceremony
    /// has an intention per case and an intention is not an
    /// entitlement, so a control the node accepted would be a *finding*
    /// rendered in the report rather than a panic that hid it.
    fn settle_case(
        &mut self,
        position: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), OwnerObservationRefusal> {
        let pending = self
            .pending
            .take()
            .ok_or(OwnerObservationRefusal::CandidateNotConstructible)?;
        let case = OwnerObservationCase::ALL
            .get(position)
            .copied()
            .ok_or(OwnerObservationRefusal::CandidateNotConstructible)?;
        if case != pending.case {
            return Err(OwnerObservationRefusal::CandidateNotConstructible);
        }

        self.record.observations.push(CaseObservation {
            case,
            layer: response.observed_layer,
            detail: response.observed_detail.clone(),
            accepted_txid: response.accepted_txid.clone(),
            submitted_bytes: pending.bytes.len(),
            construction_refusals: pending.refusals,
        });

        // The second origin runs for an acceptance and for nothing
        // else. A refusal has no accepted witness to read back, and a
        // re-verification against bytes nobody accepted would be a
        // computation about a transaction that is not on any chain.
        if matches!(response.observed_layer, ObservedOutcomeLayer::Accepted) {
            let readback = response
                .mined_readback
                .as_ref()
                .ok_or(OwnerObservationRefusal::AcceptanceCarriedNoReadback)?;
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
    ) -> Result<ReverificationRecord, OwnerObservationRefusal> {
        let decoded = TargetTransaction::decode(&readback.raw_transaction)
            .map_err(|_| OwnerObservationRefusal::ReadbackDidNotDecode)?;
        let signature = decoded
            .witnesses()
            .first()
            .and_then(|witness| witness.stack().first())
            .cloned()
            .ok_or(OwnerObservationRefusal::ReadbackCarriesNoWitness)?;

        let (finalized, _) = self.finalize(false)?;
        let census = Self::census(&finalized, self.genesis_block_hash)?;
        let input = census
            .signing_inputs()
            .first()
            .ok_or(OwnerObservationRefusal::CandidateNotConstructible)?;
        let recomputed = candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
        let emptied =
            candidate_owner_message(&census, input, WitnessVectorTreatment::OutputsEmptied);

        let target =
            reviewed_target().map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?;
        let owner = published_owner(&FIRST_SCALAR)
            .map_err(|_| OwnerObservationRefusal::SubstrateUnavailable)?;

        let verified = verify_owner_signature(&target, owner.bytes(), &recomputed, &signature);
        let verifies_against_empty_vector_message =
            verify_owner_signature(&target, owner.bytes(), &emptied, &signature).is_ok();

        Ok(ReverificationRecord {
            accepted_txid: readback.transaction_id.clone(),
            witness_txid: readback.witness_transaction_id.clone(),
            block_height: readback.block_height,
            readback_matches_submission: readback.raw_transaction == submitted,
            recomputed_message: recomputed,
            signature_from_readback: signature,
            verified,
            verifies_against_empty_vector_message,
        })
    }
}

impl TargetOperationPlanner for OwnerObservationPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.relink(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund;
                }
                Stage::Fund => {
                    if let Err(refusal) = self.settle_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Submit(0);
                }
                Stage::Submit(position) => {
                    if let Err(refusal) = self.settle_case(position, response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = if position + 1 < OwnerObservationCase::ALL.len() {
                        Stage::Submit(position + 1)
                    } else {
                        Stage::Done
                    };
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(self.funding_step("issue-protocol-asset", true))),
            Stage::Fund => Ok(Some(self.funding_step("fund-explicit-constructor", false))),
            Stage::Submit(position) => {
                let case = OwnerObservationCase::ALL[position];
                match self.case_bytes(case) {
                    Ok((bytes, message, refusals)) => {
                        self.record.candidate_messages.insert(case, message);
                        self.pending = Some(PendingCase {
                            case,
                            bytes: bytes.clone(),
                            refusals,
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

/// The explicit destination program of the first published owner.
fn explicit_destination_program(abi: &CandidateLiveTransferAbi) -> Result<Vec<u8>, VectorError> {
    Ok(abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(published_owner(&FIRST_SCALAR)?),
            LiveTransferRepresentationPlan::Explicit,
        )
        .ok_or(VectorError::LiveSubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

/// A genesis block hash that is not this run's.
///
/// One byte moved, and moved by a fixed rule rather than by a second
/// constant: what the control needs is a different chain, and a hash
/// spelled out here would be a value a reader has to check against
/// nothing.
/// One block identity in the other of its two byte orders.
///
/// Its own inverse, which is why one function serves both directions:
/// the constructor uses it to turn the printed identity into the seed,
/// and the printed-order control uses it to turn the seed back.
pub(crate) fn printed_order(identity: Digest32) -> Digest32 {
    let mut other = identity;
    other.reverse();
    other
}

const fn another_deployment(genesis: Digest32) -> Digest32 {
    let mut other = genesis;
    other[0] ^= 0xff;
    other
}

/// One outpoint, from the spelling the target printed.
pub(crate) fn outpoint_of(
    wire: &target_elements_conformance::protocol::WireOutpoint,
) -> Option<Outpoint> {
    let raw = decode_hex(&wire.txid)?;
    let mut internal = <[u8; 32]>::try_from(raw.as_slice()).ok()?;
    internal.reverse();
    Outpoint::new(Txid::from_internal(internal), wire.vout).ok()
}

/// One asset identity, from the spelling the target printed.
pub(crate) fn asset_of(text: &str) -> Option<AssetId> {
    let mut raw = decode_hex(text)?;
    raw.reverse();
    <[u8; 32]>::try_from(raw.as_slice())
        .ok()
        .map(AssetId::from_internal)
}

pub(crate) fn decode_hex(text: &str) -> Option<Vec<u8>> {
    let raw = text.as_bytes();
    let (pairs, remainder) = raw.as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let digits = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(digits, 16).ok()?);
    }
    Some(bytes)
}

/// One run's transcript, as the report artifact carries it.
///
/// Lines rather than a structure, on the pattern the existing native
/// lane sets: the artifact is read by people and diffed by machines, and
/// a rendering that grew a schema would need a reader before either
/// could use it. Every line is a fact the run observed or a value it
/// computed, and no line is a verdict about whether the run went well.
#[must_use]
pub fn render_owner_observation(record: &OwnerObservationRecord) -> String {
    let mut lines = vec!["role owner-sighash-observation-run".to_owned()];
    lines.push(format!(
        "issued_asset {}",
        record.issued_asset().unwrap_or("none")
    ));
    lines.push(format!("relinked {}", record.relinked()));

    for (index, coin) in record.coins().iter().enumerate() {
        lines.push(format!(
            "coin {index} value {} program_bytes {} node_fields_match_expectation {}",
            if let ValueField::Explicit(amount) = coin.value() {
                amount.to_string()
            } else {
                // Unreachable for a coin this ceremony funds, and
                // rendered rather than refused: the artifact's job is to
                // say what was there, and a value form the explicit lane
                // does not produce is a line worth reading rather than a
                // panic in a renderer.
                "not-explicit".to_owned()
            },
            coin.program().len(),
            coin.matches_expectation(),
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
            "observed {} negative_control {} layer {:?} txid {} submitted_bytes {} detail {}",
            observation.case().name(),
            observation.case().is_negative_control(),
            observation.layer(),
            observation.accepted_txid().unwrap_or("none"),
            observation.submitted_bytes(),
            observation.detail().unwrap_or("none"),
        ));
        for refusal in observation.construction_refusals() {
            lines.push(format!(
                "construction_refusal {} {} {:?}",
                observation.case().name(),
                refusal.clause(),
                refusal.refusal(),
            ));
        }
    }

    if let Some(check) = record.reverification() {
        lines.push(format!(
            "reverification accepted_txid {}",
            check.accepted_txid()
        ));
        lines.push(format!(
            "reverification witness_txid {}",
            check.witness_txid()
        ));
        lines.push(format!(
            "reverification block_height {}",
            check.block_height()
        ));
        lines.push(format!(
            "reverification readback_matches_submission {}",
            check.readback_matches_submission()
        ));
        lines.push(format!(
            "reverification recomputed_message {}",
            printed(check.recomputed_message().as_slice())
        ));
        lines.push(format!(
            "reverification signature_from_readback {}",
            printed(check.signature_from_readback())
        ));
        lines.push(format!(
            "reverification verifies_against_recomputed_message {}",
            check.verified().is_ok()
        ));
        if let Err(rejection) = check.verified() {
            lines.push(format!("reverification rejection {rejection:?}"));
        }
        lines.push(format!(
            "reverification verifies_against_empty_vector_message {}",
            check.verifies_against_empty_vector_message()
        ));
    } else {
        lines.push("reverification none".to_owned());
    }

    // Whether any case was accepted at all, stated as its own line. The
    // observations above carry it, and a reader looking for the one fact
    // this ceremony exists to produce should not have to scan them.
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

    for claim in OwnerObservationRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }

    // The two standings this run does not touch, written into the
    // artifact rather than left to a reader's memory of which wave owns
    // which residual.
    lines.push("clears_sighash_profile_unreviewed false".to_owned());
    lines.push("establishes_the_proof_bearing_lane false".to_owned());
    lines.push("discharges_no_matrix_row true".to_owned());

    lines.join("\n") + "\n"
}

/// One byte string in the order this workspace prints digests.
pub(crate) fn printed(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

#[cfg(test)]
mod tests {
    use crate::live_history_v1::owner_observation as history;

    #[test]
    fn selected_profile_identity_matches_target_elements_authorization() {
        // packages/target-elements/src/authorization.rs pins this literal
        // in OWNER_OBSERVATION. The integration verdict reads this vectors
        // constant, so either side drifting independently fails.
        assert_eq!(
            history::SELECTED_PROFILE_ACCEPTED_TXID,
            "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029",
        );
    }
}
