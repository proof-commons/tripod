//! The sponsored live-transfer ceremony, as a lane rather than a test.
//!
//! # Why this module exists
//!
//! The sponsor ceremony was written inside an integration test, and for
//! as long as the only sponsored control anybody built was the one that
//! test built, that was the right place for it. It stopped being the
//! right place the moment a second sponsored shape was wanted: a test
//! cannot be called from another test, so a lane that lives in one can
//! answer exactly one row.
//!
//! What moved here is the ceremony and nothing else. The judging stayed
//! where judging belongs — this planner records what happened and
//! decides none of it, on the same rule
//! [`crate::live_explicit_shapes`] follows.
//!
//! # The one axis
//!
//! [`SponsorShape`] varies exactly one thing: whether the sponsor asks
//! for change. Everything else — the issuance, the funding amounts per
//! receipt, the two destinations, the owners, the auxiliary randomness,
//! the order of the stages — is held equal, so that two runs differ in
//! the change role and in nothing a reader has to take on trust.
//!
//! [`SponsorShape::ChangeAbsent`] is the ceremony the sponsor wave ran,
//! unchanged in what it builds. It is kept rather than deleted because
//! it is this module's own byte-identity check: it has a recorded
//! identity, and a lift that changed what the ceremony builds would
//! produce a different one.
//!
//! # Why a sponsored control that takes change needs a funding change
//!
//! The construction path places a change output only where the sponsor's
//! offer states a change amount, and it writes that output FROM the
//! deployment's own sponsor-change symbol — refusing outright if a
//! sponsor capability offers any other destination. So the obstacle to a
//! with-change control was never the symbol. It was that the sponsor
//! funding step funded its coin to exactly the offer, leaving no residue
//! for a change output to carry: nothing in the reserve asset was left
//! over to put in one. [`SponsorShape::sponsor_funding`] is where that
//! is fixed, and it is the whole fix.
//!
//! Nothing in this workspace checks the reserve tally before a node
//! does, which is deliberate — §10.7 keeps individual sponsor amounts
//! out of every protocol claim — so the arithmetic is stated here, at
//! the one place that chooses both numbers, and the target is what
//! verifies it.
//!
//! # Every key here is published test material
//!
//! The signing scalars are the BIP-340 specification's own appendix
//! secret keys and the sponsor's key is the executor's fixed regtest
//! one: ADR-015 public disposable test material at each use, authorizing
//! nothing on any network anybody uses. The chain each run is taken
//! against is created and destroyed by that run.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::LeafVersion;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::owner_key_oracle::verify_owner_signature;
use target_elements_conformance::protocol::{
    MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId,
    OperationSubject, TargetFundingSubject, TargetSponsorFundingSubject,
    TargetSponsorSigningSubject, TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
};
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, Txid, ValueField};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{
    AnnexDisposition, IssuanceDisposition, LiveDeployment, OWNER_CODESEPARATOR_POSITION,
    OwnerSigningCensus, OwnerSigningInputRequest,
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
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::taproot::{Digest32, leaf_hash, witness_program_script};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::bundle::fee_program_digest;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed_order};
use crate::live_plan::{
    FIRST_SCALAR, SECOND_SCALAR, demonstration_live_abi, live_abi_for_asset, published_owner,
    reviewed_target, signing_material,
};

/// What each funded receipt is asked to hold.
///
/// The same figure every other live lane funds with, so that two runs'
/// weights stay comparable.
const RECEIPT_AMOUNT: u64 = 5_000;

/// The auxiliary randomness every owner signature here is taken with.
///
/// Published, fixed, and disposable: ADR-015 test material, like the two
/// signing scalars themselves.
const SIGNING_AUXILIARY: [u8; 32] = [0x37; 32];

/// The fee every sponsored offer in this lane declares.
///
/// Held equal across both shapes on purpose. It is the term the two
/// runs must NOT differ in, because a change role that appeared beside a
/// moved fee would leave a reader unable to say which of the two the
/// difference in the outputs came from.
const SPONSOR_FEE: u64 = 250;

/// What the sponsor asks back when it asks for change.
///
/// Chosen above the target's dust floor rather than at it: a change
/// output refused for dust would be a refusal about a number this lane
/// picked, and this lane is about the change ROLE. The figure is
/// otherwise arbitrary, and it is the one number that separates the two
/// shapes.
const SPONSOR_CHANGE: u64 = 1_000;

/// Which sponsored shape a run builds.
///
/// One axis, and the two members are the two sides of §15.1's
/// sponsor-change pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SponsorShape {
    /// The sponsor funds the fee exactly and asks for nothing back.
    ///
    /// The ceremony the sponsor wave ran, kept byte-for-byte in what it
    /// builds so that its recorded identity is reproducible.
    ChangeAbsent,
    /// The sponsor funds above the fee and takes the residue back.
    ChangePresent,
}

impl SponsorShape {
    /// Both shapes, in the order a reader meets them.
    pub const ALL: [Self; 2] = [Self::ChangeAbsent, Self::ChangePresent];

    /// The name a transcript files this run under.
    #[must_use]
    pub const fn case_name(self) -> &'static str {
        match self {
            Self::ChangeAbsent => "sponsored-change-absent",
            Self::ChangePresent => "sponsored-change-present",
        }
    }

    /// The §15.1 row this shape is an instance of.
    #[must_use]
    pub const fn row_name(self) -> &'static str {
        match self {
            Self::ChangeAbsent => "sponsor-change-absent",
            Self::ChangePresent => "sponsor-change-present",
        }
    }

    /// What the offer states as change, where it states one.
    #[must_use]
    pub const fn change(self) -> Option<u64> {
        match self {
            Self::ChangeAbsent => None,
            Self::ChangePresent => Some(SPONSOR_CHANGE),
        }
    }

    /// What the request asks the construction for.
    #[must_use]
    pub const fn change_request(self) -> SponsorChangeRequest {
        match self {
            Self::ChangeAbsent => SponsorChangeRequest::NotRequested,
            Self::ChangePresent => SponsorChangeRequest::Requested,
        }
    }

    /// What the sponsor coin is funded to.
    ///
    /// The fee plus whatever the offer takes back, and never a round
    /// number chosen separately. Elements balances per asset, so the
    /// reserve sub-equation of this transaction is exactly
    /// `sponsor_input == fee + change`; funding to anything else builds
    /// a transaction the target refuses at its tally, and the tally is
    /// the only place it is checked.
    #[must_use]
    pub const fn sponsor_funding(self) -> u64 {
        match self.change() {
            None => SPONSOR_FEE,
            Some(change) => SPONSOR_FEE + change,
        }
    }

    /// How many outputs the finalized control carries.
    ///
    /// Two destinations, the sponsored form's fee, and the change role
    /// where the shape has one.
    #[must_use]
    pub const fn output_count(self) -> usize {
        match self {
            Self::ChangeAbsent => 3,
            Self::ChangePresent => 4,
        }
    }
}

/// How the staged envelope answers a signing request.
enum Answers {
    /// Record what was asked and answer with a placeholder, so that the
    /// request the adapter must be handed can be collected at all.
    ///
    /// The transaction this arm completes is discarded. A
    /// placeholder-authorized control is not evidence and must never be
    /// able to become any.
    Recording(RefCell<Vec<(u16, Vec<u8>)>>),
    /// Answer with what the adapter returned, keyed by input position.
    Replaying(BTreeMap<u16, SponsorSignature>),
}

/// The sponsor envelope this lane stages, in both of its passes.
///
/// One type rather than two, because the offer and the change
/// destination must be identical across the passes: a second spelling of
/// the offer could disagree with the first about what was finalized, and
/// then the bytes the adapter signed would not be the bytes replayed
/// into.
struct StagedEnvelope {
    offer: SponsorOffer,
    answers: Answers,
}

impl SponsorCapability for StagedEnvelope {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    /// No destination of its own.
    ///
    /// Not an omission. The construction writes the change output from
    /// the deployment's sponsor-change symbol and refuses any other
    /// destination outright, so a capability naming one would be naming
    /// the only value that is already going to be used, or being
    /// refused. Declining to name it is the honest spelling of a choice
    /// the deployment has already made.
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        match &self.answers {
            Answers::Recording(recorded) => {
                recorded
                    .borrow_mut()
                    .push((request.input(), request.transaction().to_vec()));
                Some(SponsorSignature::new(
                    request.transaction().to_vec(),
                    vec![Vec::new(), Vec::new()],
                ))
            }
            Answers::Replaying(answers) => answers.get(&request.input()).cloned(),
        }
    }
}

/// Everything one round trip through the adapter's signer produced.
#[derive(Clone, Debug, Default)]
pub struct SponsorRoundTrip {
    input: u16,
    sent: Vec<u8>,
    echoed: Vec<u8>,
    witness: Vec<Vec<u8>>,
    replayed: Vec<u8>,
    placeholder: Vec<u8>,
    mutated_refusal: Option<String>,
}

impl SponsorRoundTrip {
    /// The sponsor input's position in the finalized control.
    #[must_use]
    pub const fn input(&self) -> u16 {
        self.input
    }

    /// The exact bytes the sponsor request carried.
    #[must_use]
    pub fn sent(&self) -> &[u8] {
        &self.sent
    }

    /// Whether the adapter echoed back exactly the bytes it was sent.
    ///
    /// Compared byte for byte rather than by length, because two
    /// serializations of one shape have the same length and are not the
    /// same bytes.
    #[must_use]
    pub fn echo_matches(&self) -> bool {
        self.echoed == self.sent
    }

    /// The witness stack the adapter returned.
    #[must_use]
    pub fn witness(&self) -> &[Vec<u8>] {
        &self.witness
    }

    /// The control completed by replaying that witness.
    #[must_use]
    pub fn replayed(&self) -> &[u8] {
        &self.replayed
    }

    /// Whether the replay actually changed the completed control.
    ///
    /// A replay that changed nothing would mean no returned witness
    /// reached the bytes, and the run would be about the placeholder.
    #[must_use]
    pub fn replay_changed_the_control(&self) -> bool {
        !self.replayed.is_empty() && self.replayed != self.placeholder
    }

    /// Whether every returned witness item occurs in the replayed bytes.
    #[must_use]
    pub fn witness_reached_the_control(&self) -> bool {
        !self.witness.is_empty()
            && self
                .witness
                .iter()
                .all(|item| contains_run(&self.replayed, item))
    }

    /// What a signature bound to one mutated byte was refused with.
    #[must_use]
    pub fn mutated_refusal(&self) -> Option<&str> {
        self.mutated_refusal.as_deref()
    }
}

/// One input's signature, taken out of the node's own copy and checked
/// against a message this workspace recomputed.
#[derive(Clone, Debug)]
pub struct SponsorInputVerification {
    input_index: usize,
    recomputed_message: [u8; 32],
    signature_bytes: usize,
    verified: bool,
}

impl SponsorInputVerification {
    /// Whether the accepted signature verifies against the recomputed
    /// message.
    #[must_use]
    pub const fn verified(&self) -> bool {
        self.verified
    }
}

/// The change output, as the node's own copy of the mined transaction
/// carries it.
///
/// Read out of the readback rather than off the request, because a run
/// whose shape silently degraded to the without-change form would
/// otherwise report an acceptance that proves nothing about the change
/// role.
#[derive(Clone, Debug)]
pub struct ObservedSponsorChange {
    position: usize,
    amount: Option<u64>,
    program_matches_symbol: bool,
    asset_is_reserve: bool,
}

impl ObservedSponsorChange {
    /// Whether the node's copy carries a change output that is the one
    /// the deployment declares: the reserve asset, the offered amount,
    /// and the deployment's own sponsor-change program.
    #[must_use]
    pub fn is_the_declared_change(&self, offered: u64) -> bool {
        self.program_matches_symbol && self.asset_is_reserve && self.amount == Some(offered)
    }

    /// Which output position the change role occupies.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
}

/// The read-back copy of an accepted control, and what it verified.
#[derive(Clone, Debug)]
pub struct SponsorReverification {
    accepted_txid: String,
    block_height: u32,
    readback_matches_submission: bool,
    inputs: Vec<SponsorInputVerification>,
    sponsor_witness_in_readback: bool,
    change: Option<ObservedSponsorChange>,
}

impl SponsorReverification {
    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether every owner signature in the node's copy verifies against
    /// a message recomputed here.
    #[must_use]
    pub fn every_owner_verified(&self) -> bool {
        !self.inputs.is_empty() && self.inputs.iter().all(SponsorInputVerification::verified)
    }

    /// Whether every item the sponsor adapter returned occurs in the
    /// node's own copy of the mined transaction.
    ///
    /// The sponsor's witness is checked for PRESENCE here and for
    /// validity by the target: the sponsor program is the executor's own
    /// version-zero key hash, whose signature this workspace holds no
    /// key for and recomputes no message for. What it can say is that
    /// the adapter's exact items survived into what was mined, and that
    /// a target which evaluates every input accepted them.
    #[must_use]
    pub const fn sponsor_witness_in_readback(&self) -> bool {
        self.sponsor_witness_in_readback
    }

    /// The change output the node's copy carries, where it carries one.
    #[must_use]
    pub const fn change(&self) -> Option<&ObservedSponsorChange> {
        self.change.as_ref()
    }

    /// The identity the target computed.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// The height the target mined it at.
    #[must_use]
    pub const fn block_height(&self) -> u32 {
        self.block_height
    }
}

/// Why the lane stopped before its run completed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SponsorShapeRefusal {
    /// The reviewed substrate did not build.
    SubstrateUnavailable,
    /// The issuance step named no asset.
    IssuanceNamedNoAsset,
    /// The deployment could not be relinked to the chain's own assets.
    RelinkRefused,
    /// A funding step created no predecessor to spend.
    FundingCreatedNoPredecessor,
    /// A funded outpoint did not decode.
    MalformedFundedOutpoint,
    /// The control did not construct.
    ControlNotConstructible,
    /// The adapter refused, or answered at a layer below acceptance.
    AuthorizationDidNotHappen(ObservedOutcomeLayer),
    /// The adapter accepted and returned no witness.
    AuthorizationCarriedNoWitness,
    /// The adapter echoed bytes that are not the bytes it was sent.
    AuthorizationBoundToOtherBytes { sent: usize, echoed: usize },
    /// The completed control did not ask the envelope for exactly one
    /// sponsor authorization.
    UnexpectedSponsorRequestCount(usize),
    /// A signature bound to mutated bytes was accepted, which would mean
    /// the binding this lane rests on does not hold.
    MutatedBindingAccepted,
    /// The sponsor coins came back in more than one asset, so there is
    /// no single reserve identity to weld the deployment to.
    SponsorAssetsDisagree,
    /// The sponsor-funding step reported no reserve asset at all.
    SponsorFundingNamedNoReserve,
    /// The owner signing census refused to describe the control.
    OwnerCensusRefused(String),
    /// An owner's signature could not be produced over its message.
    OwnerSigningRefused,
    /// The node's own copy of the mined transaction did not decode.
    ReadbackDidNotDecode,
    /// The node's copy carries no witness for an input that has one.
    ReadbackCarriesNoWitness,
    /// The target's own copy of the mined transaction is not the bytes
    /// that were submitted.
    ReadbackDisagreesWithSubmittedBytes { submitted: usize, read_back: usize },
    /// A with-change run reached a node carrying no change output.
    ///
    /// A HARD stop rather than a finding. A run whose shape degraded to
    /// the without-change form would be an acceptance of a shape that
    /// already has one, reported under a row it says nothing about.
    ChangeRunCarriedNoChangeOutput,
}

/// One coin as the node reported it.
#[derive(Clone, Debug)]
struct ObservedCoin {
    outpoint: Outpoint,
    asset: AssetId,
    amount: u64,
    program: Vec<u8>,
}

/// Which step the lane is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Issue,
    FundSponsor,
    FundReceipts,
    SignSponsor,
    /// The §15.6 mutant, offered BEFORE its control.
    ///
    /// The order is not a preference. A sponsor-witness mutation changes
    /// the WITNESS, and a witness is not part of a transaction's
    /// identity, so the mutant and its control carry the SAME identity.
    /// With the control submitted and mined first, the mutant comes back
    /// refused `txn-already-known` at a layer before script evaluation
    /// — a true refusal about an identity already on the chain, and
    /// attributable to the submission order rather than to anything the
    /// witness offered. Counting that would be the exact error the
    /// attributability rule exists to prevent.
    SubmitMutant,
    Submit,
    Done,
}

/// What one §15.6 mutant offering observed.
#[derive(Clone, Debug)]
pub struct SponsorNegativeObservation {
    submitted_bytes: usize,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    sponsor_witness_item_bytes: Vec<usize>,
    differs_from_control_in_the_sponsor_witness: bool,
}

impl SponsorNegativeObservation {
    /// The target's own words.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// The layer the target typed its answer at.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// Whether the mutant differs from its control in the sponsor
    /// witness and in nothing else.
    ///
    /// Measured by comparing the two submissions rather than argued from
    /// the code that built them: both come from ONE finalization, and
    /// the only value that moved is what the sponsor capability
    /// returned. The two carry the same identity for the same reason,
    /// which is why the mutant is offered first.
    #[must_use]
    pub const fn differs_from_control_in_the_sponsor_witness(&self) -> bool {
        self.differs_from_control_in_the_sponsor_witness
    }

    /// How many bytes the mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }
}

/// The finalization a sponsor request was formed against.
type StagedFinalization = (
    transaction::live_signing::AuthorizedLiveTransfer,
    LiveConstructionReport,
);

/// A staged control: its finalization, every sponsor request the builder
/// issued against it, and the placeholder completion to compare against.
type StagedControl = (StagedFinalization, Vec<(u16, Vec<u8>)>, Vec<u8>);

/// What one run of this lane observed.
///
/// Observations, never verdicts. Every field is something the run saw or
/// computed, and nothing here says whether the run went well.
#[derive(Clone, Debug)]
pub struct SponsorShapeRecord {
    shape: SponsorShape,
    issued_asset: Option<String>,
    relinked: bool,
    receipt_coins: usize,
    sponsor_funded: Option<u64>,
    offered_fee: u64,
    offered_change: Option<u64>,
    round: Option<SponsorRoundTrip>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    target_weight: Option<u64>,
    reverification: Option<SponsorReverification>,
    negatives: Vec<SponsorNegativeObservation>,
    refusal: Option<SponsorShapeRefusal>,
}

impl SponsorShapeRecord {
    /// Which shape the run was asked for.
    #[must_use]
    pub const fn shape(&self) -> SponsorShape {
        self.shape
    }

    /// How many bytes were handed to the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the target typed its answer at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The weight the target itself computed for the bytes it took.
    #[must_use]
    pub const fn target_weight(&self) -> Option<u64> {
        self.target_weight
    }

    /// The sponsor round trip, where one completed.
    #[must_use]
    pub const fn round(&self) -> Option<&SponsorRoundTrip> {
        self.round.as_ref()
    }

    /// The read-back check, where an acceptance happened.
    #[must_use]
    pub const fn reverification(&self) -> Option<&SponsorReverification> {
        self.reverification.as_ref()
    }

    /// What the ceremony stopped on, where it stopped.
    #[must_use]
    pub const fn refusal(&self) -> Option<&SponsorShapeRefusal> {
        self.refusal.as_ref()
    }

    /// The §15.6 mutant offerings this run made, where it made any.
    #[must_use]
    pub fn negatives(&self) -> &[SponsorNegativeObservation] {
        &self.negatives
    }

    /// How much the sponsor coin was funded to.
    #[must_use]
    pub const fn sponsor_funded(&self) -> Option<u64> {
        self.sponsor_funded
    }

    /// What this lane does NOT establish, stated in the record itself.
    #[must_use]
    pub const fn non_claims() -> &'static [&'static str] {
        &[
            "production-multi-party-sponsor-signing",
            "confidential-sponsor-values",
            "any-negative-row",
        ]
    }
}

/// The lane: issue, fund a sponsor coin, fund receipts, sign, submit.
pub struct SponsorShapePlanner {
    stage: Stage,
    shape: SponsorShape,
    /// Whether this run offers the §15.6 mutant before its control.
    offers_the_mutant: bool,
    abi: CandidateLiveTransferAbi,
    genesis: Digest32,
    explicit_program: Vec<u8>,
    receipts: Vec<ObservedCoin>,
    sponsor: Option<ObservedCoin>,
    staged: Option<StagedFinalization>,
    record: SponsorShapeRecord,
}

impl SponsorShapePlanner {
    /// The ceremony for one shape, bound to one chain.
    ///
    /// Takes the genesis identity in the spelling a target PRINTS it,
    /// which is the one an environment variable carries, and reverses it
    /// here. The explicit shape lane takes it the same way, and taking
    /// it differently is how two lanes bound to one chain end up
    /// committing to two different deployments.
    ///
    /// # Errors
    ///
    /// [`SponsorShapeRefusal::SubstrateUnavailable`] when the reviewed
    /// candidate substrate does not build.
    pub fn for_shape(
        shape: SponsorShape,
        printed_genesis_identity: Digest32,
    ) -> Result<Self, SponsorShapeRefusal> {
        let abi =
            demonstration_live_abi().map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;
        let explicit_program = destination_program(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            shape,
            offers_the_mutant: false,
            abi,
            genesis: printed_order(printed_genesis_identity),
            explicit_program,
            receipts: Vec::new(),
            sponsor: None,
            staged: None,
            record: SponsorShapeRecord {
                shape,
                issued_asset: None,
                relinked: false,
                receipt_coins: 0,
                sponsor_funded: None,
                offered_fee: SPONSOR_FEE,
                offered_change: shape.change(),
                round: None,
                submitted_bytes: 0,
                observed_layer: None,
                observed_detail: None,
                accepted_txid: None,
                target_weight: None,
                reverification: None,
                negatives: Vec::new(),
                refusal: None,
            },
        })
    }

    /// The same ceremony, offering the §15.6 mutant before its control.
    ///
    /// The mutant is the completion the RECORDING pass already produced:
    /// one finalization, every owner really signing, and the sponsor
    /// capability answering with an empty stack instead of the adapter's
    /// witness. So it is a sponsored control whose sponsor input carries
    /// no authorization, and it differs from the control that follows it
    /// in the sponsor witness and in nothing else.
    ///
    /// It is not a second ceremony and it must not become one. A mutant
    /// built by a second finalization would differ from its control in
    /// whatever else the second build chose, and the refusal would then
    /// be attributable to nothing in particular.
    ///
    /// # Errors
    ///
    /// [`SponsorShapeRefusal::SubstrateUnavailable`] when the reviewed
    /// candidate substrate does not build.
    pub fn for_missing_authorization_negative(
        shape: SponsorShape,
        printed_genesis_identity: Digest32,
    ) -> Result<Self, SponsorShapeRefusal> {
        let mut planner = Self::for_shape(shape, printed_genesis_identity)?;
        planner.offers_the_mutant = true;
        Ok(planner)
    }

    /// What the run observed.
    #[must_use]
    pub const fn record(&self) -> &SponsorShapeRecord {
        &self.record
    }

    fn refuse(&mut self, refusal: SponsorShapeRefusal) -> PlanRefused {
        self.record.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// Remember the asset the chain just issued.
    ///
    /// The deployment is NOT relinked here, because half of what it is
    /// welded to is still unknown: the reserve arrives from the
    /// sponsor-funding answer, and a link taken now would be a link
    /// against the fixture reserve.
    fn remember_issued_asset(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsorShapeRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(SponsorShapeRefusal::IssuanceNamedNoAsset)?;
        asset_of(&asset).ok_or(SponsorShapeRefusal::IssuanceNamedNoAsset)?;
        self.record.issued_asset = Some(asset);
        Ok(())
    }

    /// Weld the deployment to BOTH assets the chain reported, and to the
    /// digest of the fee program this lane actually constructs.
    fn relink(&mut self, reserve: AssetId) -> Result<(), SponsorShapeRefusal> {
        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(SponsorShapeRefusal::IssuanceNamedNoAsset)?;
        let protocol = asset_of(&printed).ok_or(SponsorShapeRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(
            *protocol.internal(),
            *reserve.internal(),
            fee_program_digest(),
        )
        .map_err(|_| SponsorShapeRefusal::RelinkRefused)?;
        self.explicit_program = destination_program(&abi)?;
        self.abi = abi;
        self.record.relinked = true;
        Ok(())
    }

    /// Read the sponsor coins and settle the ONE reserve identity.
    fn settle_sponsor_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsorShapeRefusal> {
        let coins = Self::settle_funding(response)?;
        let mut reserve: Option<AssetId> = None;
        for coin in &coins {
            match reserve {
                Some(known) if known != coin.asset => {
                    return Err(SponsorShapeRefusal::SponsorAssetsDisagree);
                }
                Some(_) => {}
                None => reserve = Some(coin.asset),
            }
        }
        let reserve = reserve.ok_or(SponsorShapeRefusal::SponsorFundingNamedNoReserve)?;
        let coin = coins
            .into_iter()
            .next()
            .ok_or(SponsorShapeRefusal::FundingCreatedNoPredecessor)?;
        self.record.sponsor_funded = Some(coin.amount);
        self.sponsor = Some(coin);
        self.relink(reserve)
    }

    /// One funding step's coins, taken from the node's report of them.
    fn settle_funding(
        response: &NativeOperationResponse,
    ) -> Result<Vec<ObservedCoin>, SponsorShapeRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(SponsorShapeRefusal::FundingCreatedNoPredecessor);
        }
        let mut coins = Vec::with_capacity(response.funded_outputs.len());
        for funded in &response.funded_outputs {
            coins.push(ObservedCoin {
                outpoint: outpoint_of(&funded.outpoint)
                    .ok_or(SponsorShapeRefusal::MalformedFundedOutpoint)?,
                asset: asset_of(&funded.asset)
                    .ok_or(SponsorShapeRefusal::MalformedFundedOutpoint)?,
                amount: funded.amount_satoshis,
                program: decode_hex(&funded.script)
                    .ok_or(SponsorShapeRefusal::MalformedFundedOutpoint)?,
            });
        }
        Ok(coins)
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
                outputs: 2,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        )
    }

    /// The offer this run's sponsor makes.
    ///
    /// Built in one place and never spelled twice, because the staging
    /// pass and the replay pass must make the SAME offer: a second
    /// spelling that disagreed would finalize different bytes than the
    /// ones the adapter signed.
    fn offer(&self) -> Result<SponsorOffer, SponsorShapeRefusal> {
        let sponsor_coin = self
            .sponsor
            .as_ref()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        SponsorOffer::new(
            [sponsor_coin.outpoint],
            SPONSOR_FEE,
            self.shape.change().map(ValueField::Explicit),
        )
        .map_err(|_| SponsorShapeRefusal::ControlNotConstructible)
    }

    /// The sponsored control, finalized and owner-authorized, with every
    /// sponsor request the builder issues collected.
    fn stage_control(&self) -> Result<StagedControl, SponsorShapeRefusal> {
        let sponsor_coin = self
            .sponsor
            .as_ref()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        let mut points = Vec::with_capacity(self.receipts.len());
        let mut views = Vec::with_capacity(self.receipts.len() + 1);
        let mut total = 0_u64;
        for coin in &self.receipts {
            points.push(coin.outpoint);
            views.push(PublicOutputView::new(
                coin.outpoint,
                AssetField::Explicit(coin.asset),
                ValueField::Explicit(coin.amount),
                coin.program.clone(),
            ));
            total = total
                .checked_add(coin.amount)
                .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        }
        // The sponsor coin is SHOWN and not merely named. Construction
        // refuses a sponsor input it cannot see, and refuses one whose
        // asset is not the deployment's reserve.
        views.push(PublicOutputView::new(
            sponsor_coin.outpoint,
            AssetField::Explicit(sponsor_coin.asset),
            ValueField::Explicit(sponsor_coin.amount),
            sponsor_coin.program.clone(),
        ));
        let view = PublicConstructionView::new(views)
            .map_err(|_| SponsorShapeRefusal::ControlNotConstructible)?;

        let request = LiveTransferRequest::new(
            points,
            [
                destination(&SECOND_SCALAR, total / 2)?,
                destination(&FIRST_SCALAR, total - total / 2)?,
            ],
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsored,
            self.shape.change_request(),
            None,
        )
        .map_err(|_| SponsorShapeRefusal::ControlNotConstructible)?;

        let envelope = StagedEnvelope {
            offer: self.offer()?,
            answers: Answers::Recording(RefCell::new(Vec::new())),
        };

        let target = reviewed_target().map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;
        let finalization =
            finalize_live_transfer(&target, &self.abi, &request, &view, Some(&envelope), None)
                .map_err(|_| SponsorShapeRefusal::ControlNotConstructible)?;
        let report = finalization.report().clone();
        let finalized = finalization.into_finalized();

        // The owners really sign: a target evaluates the owner's leaf,
        // and placeholder bytes are refused there with a verdict that
        // says nothing about the sponsor envelope.
        let responses = self.owner_responses(&finalized)?;
        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| SponsorShapeRefusal::ControlNotConstructible)?;

        let placeholder =
            complete_live_transfer(&target, authorized.clone(), report.clone(), Some(&envelope))
                .map_err(|_| SponsorShapeRefusal::ControlNotConstructible)?
                .bytes();

        let Answers::Recording(recorded) = &envelope.answers else {
            // Unreachable: the value was just built with this arm.
            return Err(SponsorShapeRefusal::ControlNotConstructible);
        };
        let recorded = recorded.borrow().clone();
        Ok(((authorized, report), recorded, placeholder))
    }

    /// The owner signing census for one finalization.
    fn census(
        &self,
        finalized: &FinalizedLiveTransfer,
    ) -> Result<OwnerSigningCensus, SponsorShapeRefusal> {
        let target = reviewed_target().map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?,
        );
        let requests: Vec<OwnerSigningInputRequest> = finalized
            .receipts()
            .iter()
            .map(|record| {
                OwnerSigningInputRequest::new(
                    u32::from(record.position()),
                    leaf_hash(LeafVersion::TAPSCRIPT, record.leaf_script()),
                    LeafVersion::TAPSCRIPT,
                    OWNER_CODESEPARATOR_POSITION,
                    AnnexDisposition::Absent,
                    IssuanceDisposition::Absent,
                    record.control_block().to_vec(),
                )
            })
            .collect();
        OwnerSigningCensus::from_explicit_finalized(
            &target,
            finalized,
            LiveDeployment::new(self.genesis),
            &requests,
            &curve,
        )
        .map_err(|cause| SponsorShapeRefusal::OwnerCensusRefused(format!("{cause:?}")))
    }

    /// Every owner signature the finalized control asks for.
    ///
    /// Both receipts were funded to one destination program — the first
    /// owner's explicit constructor — so one scalar answers for both.
    fn owner_responses(
        &self,
        finalized: &FinalizedLiveTransfer,
    ) -> Result<Vec<(u16, LiveOwnerResponse)>, SponsorShapeRefusal> {
        let census = self.census(finalized)?;
        let material = signing_material(&FIRST_SCALAR)
            .map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;
        let mut responses = Vec::new();
        for signing in finalized.signing_requests() {
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            let signature = material
                .sign(&message, &SIGNING_AUXILIARY)
                .map_err(|_| SponsorShapeRefusal::OwnerSigningRefused)?
                .to_vec();
            responses.push((signing.input(), LiveOwnerResponse::to(&signing, signature)));
        }
        Ok(responses)
    }

    /// The sponsor signing step, carrying the exact finalized bytes.
    fn sign_step(&mut self) -> Result<OperationStep, SponsorShapeRefusal> {
        let (staged, recorded, placeholder) = self.stage_control()?;
        if recorded.len() != 1 {
            return Err(SponsorShapeRefusal::UnexpectedSponsorRequestCount(
                recorded.len(),
            ));
        }
        let (input, sent) = recorded[0].clone();
        let sponsor_coin = self
            .sponsor
            .clone()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;

        self.record.round = Some(SponsorRoundTrip {
            input,
            sent: sent.clone(),
            placeholder,
            ..SponsorRoundTrip::default()
        });
        self.staged = Some(staged);

        Ok(OperationStep::new(
            "authorize-sponsor-input",
            OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                finalized_transaction: sent,
                sponsor_input_index: input,
                sponsor_outpoint: WireOutpoint {
                    txid: printed_txid(&sponsor_coin.outpoint.txid()),
                    vout: sponsor_coin.outpoint.index(),
                },
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            })),
        ))
    }

    /// What the adapter answered: checked for binding, then replayed.
    fn settle_signature(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsorShapeRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(SponsorShapeRefusal::AuthorizationDidNotHappen(
                response.observed_layer,
            ));
        }
        let echoed = response
            .signature_bound_to
            .clone()
            .ok_or(SponsorShapeRefusal::AuthorizationCarriedNoWitness)?;
        if response.sponsor_witness.is_empty() {
            return Err(SponsorShapeRefusal::AuthorizationCarriedNoWitness);
        }
        let mut round = self
            .record
            .round
            .clone()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        if echoed != round.sent {
            return Err(SponsorShapeRefusal::AuthorizationBoundToOtherBytes {
                sent: round.sent.len(),
                echoed: echoed.len(),
            });
        }
        round.echoed.clone_from(&echoed);
        round.witness.clone_from(&response.sponsor_witness);

        let (authorized, report) = self
            .staged
            .clone()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        let target = reviewed_target().map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;
        let offer = self.offer()?;

        // The replay: the adapter's own witness, bound to the adapter's
        // own echo, back through the same capability the construction
        // asks — and into the same finalization the request was formed
        // against, never a rebuilt one.
        let replaying = StagedEnvelope {
            offer: offer.clone(),
            answers: Answers::Replaying(BTreeMap::from([(
                round.input,
                SponsorSignature::new(echoed.clone(), response.sponsor_witness.clone()),
            )])),
        };
        round.replayed = complete_live_transfer(
            &target,
            authorized.clone(),
            report.clone(),
            Some(&replaying),
        )
        .map_err(|_| SponsorShapeRefusal::ControlNotConstructible)?
        .bytes();

        // The control for the binding claim: one byte of the echo moved
        // and the same witness. Completion must refuse, or "bound to
        // these exact bytes" is a sentence nothing enforces.
        let mut mutated = echoed;
        if let Some(first) = mutated.first_mut() {
            *first ^= 0x01;
        }
        let refusing = StagedEnvelope {
            offer,
            answers: Answers::Replaying(BTreeMap::from([(
                round.input,
                SponsorSignature::new(mutated, response.sponsor_witness.clone()),
            )])),
        };
        match complete_live_transfer(&target, authorized, report, Some(&refusing)) {
            Ok(_) => return Err(SponsorShapeRefusal::MutatedBindingAccepted),
            Err(refusal) => round.mutated_refusal = Some(format!("{refusal:?}")),
        }

        self.record.round = Some(round);
        Ok(())
    }

    /// Hand the target the §15.6 mutant: the same control, the sponsor
    /// input carrying no authorization.
    fn mutant_step(&self) -> Result<OperationStep, SponsorShapeRefusal> {
        let round = self
            .record
            .round
            .clone()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        if round.placeholder.is_empty() {
            return Err(SponsorShapeRefusal::ControlNotConstructible);
        }
        Ok(OperationStep::new(
            "submit-unauthorized-sponsor-control",
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: round.placeholder,
            })),
        ))
    }

    /// What the target said to the mutant, recorded before it is judged.
    ///
    /// A refusal is the expected outcome and is not treated as one: the
    /// layer and the target's own words are kept, and the run carries
    /// on to its control. An ACCEPTANCE here would be the finding, and
    /// the test that reads this record is where that is judged.
    fn settle_mutant(&mut self, response: &NativeOperationResponse) {
        let Some(round) = self.record.round.clone() else {
            return;
        };
        // Attributability, MEASURED rather than argued: the mutant and
        // its control come from one finalization and differ only in what
        // the sponsor capability returned, so the two byte strings agree
        // everywhere the sponsor witness is not.
        let differs = round.placeholder.len() + round.witness.iter().map(Vec::len).sum::<usize>()
            == round.replayed.len()
            && round.placeholder != round.replayed;
        self.record.negatives.push(SponsorNegativeObservation {
            submitted_bytes: round.placeholder.len(),
            layer: response.observed_layer,
            detail: response.observed_detail.clone(),
            sponsor_witness_item_bytes: round.witness.iter().map(Vec::len).collect(),
            differs_from_control_in_the_sponsor_witness: differs,
        });
    }

    /// Hand the target the sponsor-signed control, exactly as replayed.
    fn submit_step(&self) -> Result<OperationStep, SponsorShapeRefusal> {
        let round = self
            .record
            .round
            .clone()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        if round.replayed.is_empty() {
            return Err(SponsorShapeRefusal::ControlNotConstructible);
        }
        Ok(OperationStep::new(
            "submit-sponsor-signed-control",
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: round.replayed,
            })),
        ))
    }

    /// What the target did with it, recorded before it is judged.
    fn settle_submission(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsorShapeRefusal> {
        let round = self
            .record
            .round
            .clone()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        self.record.submitted_bytes = round.replayed.len();
        self.record.observed_layer = Some(response.observed_layer);
        self.record
            .observed_detail
            .clone_from(&response.observed_detail);
        self.record
            .accepted_txid
            .clone_from(&response.accepted_txid);
        self.record.target_weight = response.resources.transaction_weight;

        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Ok(());
        }
        let readback = response
            .mined_readback
            .as_ref()
            .ok_or(SponsorShapeRefusal::ReadbackDidNotDecode)?;
        if readback.raw_transaction != round.replayed {
            return Err(SponsorShapeRefusal::ReadbackDisagreesWithSubmittedBytes {
                submitted: round.replayed.len(),
                read_back: readback.raw_transaction.len(),
            });
        }
        let check = self.reverify(readback, &round)?;
        // The hard stop the shape's own subject demands: a with-change
        // run that reached a node carrying no change output proves
        // nothing about the change role, and reporting it under the
        // with-change row would be reporting an acceptance of the shape
        // that already has one.
        if self.shape.change().is_some() && check.change.is_none() {
            return Err(SponsorShapeRefusal::ChangeRunCarriedNoChangeOutput);
        }
        self.record.reverification = Some(check);
        Ok(())
    }

    /// Read the accepted transaction back and check both origins.
    fn reverify(
        &self,
        readback: &MinedFundingReadback,
        round: &SponsorRoundTrip,
    ) -> Result<SponsorReverification, SponsorShapeRefusal> {
        let decoded = TargetTransaction::decode(&readback.raw_transaction)
            .map_err(|_| SponsorShapeRefusal::ReadbackDidNotDecode)?;
        // The SAME finalization the sponsor request was formed against,
        // kept rather than rebuilt: a rebuilt one is a different one,
        // and the messages recomputed against it would be messages about
        // other bytes.
        let (authorized, _report) = self
            .staged
            .as_ref()
            .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
        let finalized = authorized.finalized();
        let census = self.census(finalized)?;
        let target = reviewed_target().map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;
        let published = published_owner(&FIRST_SCALAR)
            .map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;

        let mut inputs = Vec::new();
        for signing in finalized.signing_requests() {
            let index = usize::from(signing.input());
            let entry = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(SponsorShapeRefusal::ControlNotConstructible)?;
            let recomputed =
                candidate_owner_message(&census, entry, WitnessVectorTreatment::BothGrown);
            let signature = decoded
                .witnesses()
                .get(index)
                .and_then(|witness| witness.stack().first())
                .cloned()
                .ok_or(SponsorShapeRefusal::ReadbackCarriesNoWitness)?;
            let verified =
                verify_owner_signature(&target, published.bytes(), &recomputed, &signature).is_ok();
            inputs.push(SponsorInputVerification {
                input_index: index,
                recomputed_message: recomputed,
                signature_bytes: signature.len(),
                verified,
            });
        }
        inputs.sort_unstable_by_key(|entry| entry.input_index);

        let sponsor_witness_in_readback = !round.witness.is_empty()
            && round
                .witness
                .iter()
                .all(|item| contains_run(&readback.raw_transaction, item));

        Ok(SponsorReverification {
            accepted_txid: readback.transaction_id.clone(),
            block_height: readback.block_height,
            readback_matches_submission: true,
            inputs,
            sponsor_witness_in_readback,
            change: self.observed_change(&decoded),
        })
    }

    /// The change output as the node's own copy carries it.
    ///
    /// Located by the deployment's own sponsor-change program rather
    /// than by position, because a position is what a degraded shape
    /// would still have and the program is what a degraded shape would
    /// not.
    fn observed_change(&self, decoded: &TargetTransaction) -> Option<ObservedSponsorChange> {
        let target = reviewed_target().ok()?;
        let expected = witness_program_script(
            &target,
            self.abi.symbols().sponsor_change_version(),
            self.abi.symbols().sponsor_change_program(),
        )
        .ok()?;
        decoded
            .outputs()
            .iter()
            .enumerate()
            .find(|(_, output)| output.program() == expected.as_slice())
            .map(|(position, output)| ObservedSponsorChange {
                position,
                // A committed change value has no amount to read, and
                // reporting one would be inventing it. The absence is
                // the observation, and Phase C's own shape is where a
                // commitment here becomes the expected form.
                amount: match output.value() {
                    ValueField::Explicit(amount) => Some(amount),
                    _ => None,
                },
                program_matches_symbol: true,
                asset_is_reserve: output.asset()
                    == AssetField::Explicit(self.abi.symbols().reserve_asset()),
            })
    }
}

impl TargetOperationPlanner for SponsorShapePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.remember_issued_asset(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundSponsor;
                }
                // The sponsor region is funded BEFORE the receipts, and
                // the order is load-bearing. The reserve arrives in this
                // answer; the deployment is welded to it; and only then
                // are the receipts funded — to destination programs that
                // moved when the reserve did.
                Stage::FundSponsor => {
                    if let Err(refusal) = self.settle_sponsor_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundReceipts;
                }
                Stage::FundReceipts => match Self::settle_funding(response) {
                    Ok(coins) => {
                        self.record.receipt_coins = coins.len();
                        self.receipts = coins;
                        self.stage = Stage::SignSponsor;
                    }
                    Err(refusal) => return Err(self.refuse(refusal)),
                },
                Stage::SignSponsor => {
                    if let Err(refusal) = self.settle_signature(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = if self.offers_the_mutant {
                        Stage::SubmitMutant
                    } else {
                        Stage::Submit
                    };
                }
                Stage::SubmitMutant => {
                    self.settle_mutant(response);
                    self.stage = Stage::Submit;
                }
                Stage::Submit => {
                    if let Err(refusal) = self.settle_submission(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        let step = match self.stage {
            Stage::Issue => self.funding_step("issue-protocol-asset", true),
            Stage::FundSponsor => OperationStep::new(
                "fund-sponsor-region",
                OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                    sponsor_outputs: 1,
                    amount_per_sponsor_output: self.shape.sponsor_funding(),
                })),
            ),
            Stage::FundReceipts => self.funding_step("fund-explicit-constructor", false),
            Stage::SignSponsor => match self.sign_step() {
                Ok(step) => step,
                Err(refusal) => return Err(self.refuse(refusal)),
            },
            Stage::SubmitMutant => match self.mutant_step() {
                Ok(step) => step,
                Err(refusal) => return Err(self.refuse(refusal)),
            },
            Stage::Submit => match self.submit_step() {
                Ok(step) => step,
                Err(refusal) => return Err(self.refuse(refusal)),
            },
            Stage::Done => return Ok(None),
        };
        Ok(Some(step))
    }
}

/// The explicit constructor's destination program for the first
/// published owner.
fn destination_program(abi: &CandidateLiveTransferAbi) -> Result<Vec<u8>, SponsorShapeRefusal> {
    Ok(abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(
                published_owner(&FIRST_SCALAR)
                    .map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?,
            ),
            LiveTransferRepresentationPlan::Explicit,
        )
        .ok_or(SponsorShapeRefusal::SubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

fn destination(
    scalar: &[u8; 32],
    amount: u64,
) -> Result<LiveReceiptDestination, SponsorShapeRefusal> {
    let owner = published_owner(scalar).map_err(|_| SponsorShapeRefusal::SubstrateUnavailable)?;
    let value =
        ProtocolValue::new(amount).map_err(|_| SponsorShapeRefusal::ControlNotConstructible)?;
    Ok(LiveReceiptDestination::new(
        linker::OwnerParameter::new(owner),
        value,
    ))
}

/// One transaction identity, in the spelling a target prints.
fn printed_txid(txid: &Txid) -> String {
    let mut bytes = *txid.internal();
    bytes.reverse();
    hex(&bytes)
}

/// Whether `needle` occurs in `haystack` as a contiguous run.
fn contains_run(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// Hex, for the transcript.
fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// One sponsored run's transcript.
///
/// Lines rather than a structure, on the pattern every live lane here
/// sets. Every line is a fact the run observed or a value it computed,
/// and no line is a verdict about whether the run went well.
#[must_use]
pub fn render_sponsor_shape(record: &SponsorShapeRecord) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "sponsor_shape {}", record.shape.case_name());
    let _ = writeln!(out, "row_name {}", record.shape.row_name());
    let _ = writeln!(
        out,
        "issued_asset {}",
        record.issued_asset.as_deref().unwrap_or("none")
    );
    let _ = writeln!(out, "relinked {}", record.relinked);
    let _ = writeln!(out, "receipt_coins {}", record.receipt_coins);
    let _ = writeln!(
        out,
        "sponsor_funded {}",
        record
            .sponsor_funded
            .map_or_else(|| "none".to_owned(), |amount| amount.to_string())
    );
    let _ = writeln!(out, "offered_fee {}", record.offered_fee);
    let _ = writeln!(
        out,
        "offered_change {}",
        record
            .offered_change
            .map_or_else(|| "none".to_owned(), |amount| amount.to_string())
    );
    let _ = writeln!(out, "expected_output_count {}", record.shape.output_count());
    render_sponsor_round_trip(&mut out, record);
    let _ = writeln!(out, "submitted_bytes {}", record.submitted_bytes);
    let _ = writeln!(
        out,
        "observed_layer {}",
        record
            .observed_layer
            .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}"))
    );
    let _ = writeln!(
        out,
        "observed_detail {}",
        record.observed_detail.as_deref().unwrap_or("none")
    );
    let _ = writeln!(
        out,
        "accepted_txid {}",
        record.accepted_txid.as_deref().unwrap_or("none")
    );
    let _ = writeln!(
        out,
        "target_weight {}",
        record
            .target_weight
            .map_or_else(|| "none".to_owned(), |weight| weight.to_string())
    );
    render_sponsor_reverification(&mut out, record);
    let _ = writeln!(
        out,
        "construction_refusal {}",
        record
            .refusal
            .as_ref()
            .map_or_else(|| "none".to_owned(), |refusal| format!("{refusal:?}"))
    );
    let _ = writeln!(
        out,
        "evidences_no_negative_case {}",
        record.negatives.is_empty()
    );
    for negative in &record.negatives {
        let _ = writeln!(
            out,
            "negative missing-sponsor-authorization submitted_bytes {} layer {:?} \
             sponsor_witness_item_bytes {:?} \
             differs_from_control_in_the_sponsor_witness {} detail {}",
            negative.submitted_bytes,
            negative.layer,
            negative.sponsor_witness_item_bytes,
            negative.differs_from_control_in_the_sponsor_witness,
            negative.detail.as_deref().unwrap_or("none"),
        );
    }
    for claim in SponsorShapeRecord::non_claims() {
        let _ = writeln!(out, "does_not_establish {claim}");
    }
    out
}

/// The round-trip lines, where a round trip completed.
///
/// Split out of the renderer for the reason the read-back lines are:
/// the renderer had grown past what one function is allowed to be, and
/// this is a seam — everything here is about what the ADAPTER did,
/// before any of it reached a node.
fn render_sponsor_round_trip(out: &mut String, record: &SponsorShapeRecord) {
    let Some(round) = &record.round else {
        return;
    };
    let _ = writeln!(out, "sponsor_input_position {}", round.input);
    let _ = writeln!(out, "finalized_bytes {}", round.sent.len());
    let _ = writeln!(out, "adapter_echo_matches {}", round.echo_matches());
    let _ = writeln!(out, "sponsor_witness_items {}", round.witness.len());
    for item in &round.witness {
        let _ = writeln!(out, "sponsor_witness_item_bytes {}", item.len());
    }
    let _ = writeln!(
        out,
        "replay_changed_the_control {}",
        round.replay_changed_the_control()
    );
    let _ = writeln!(
        out,
        "witness_reached_the_control {}",
        round.witness_reached_the_control()
    );
    let _ = writeln!(
        out,
        "mutated_binding_refusal {}",
        round.mutated_refusal.as_deref().unwrap_or("none")
    );
}

/// The read-back lines, where an acceptance produced any.
fn render_sponsor_reverification(out: &mut String, record: &SponsorShapeRecord) {
    let Some(check) = &record.reverification else {
        return;
    };
    let _ = writeln!(out, "readback_txid {}", check.accepted_txid);
    let _ = writeln!(out, "readback_block_height {}", check.block_height);
    let _ = writeln!(
        out,
        "readback_matches_submission {}",
        check.readback_matches_submission
    );
    for input in &check.inputs {
        let _ = writeln!(
            out,
            "input {} signature_bytes {} verified {} message {}",
            input.input_index,
            input.signature_bytes,
            input.verified,
            hex(&input.recomputed_message)
        );
    }
    let _ = writeln!(out, "every_owner_verified {}", check.every_owner_verified());
    let _ = writeln!(
        out,
        "sponsor_witness_in_readback {}",
        check.sponsor_witness_in_readback
    );
    match &check.change {
        None => {
            let _ = writeln!(out, "sponsor_change_output none");
        }
        Some(change) => {
            let _ = writeln!(
                out,
                "sponsor_change_output position {} value {} program_matches_symbol {} \
                 asset_is_reserve {}",
                change.position,
                change
                    .amount
                    .map_or_else(|| "committed".to_owned(), |amount| amount.to_string()),
                change.program_matches_symbol,
                change.asset_is_reserve,
            );
        }
    }
}

/// What this lane observed, for the three §15.1 rows whose subject is
/// the sponsor region.
///
/// The register moved here with the ceremony that produces it. It used
/// to sit beside the explicit shape lane's own, which built no sponsor
/// region at all and so could only host it; now the lane and its record
/// are in one place, and a change to what the ceremony builds is next to
/// the identities it has to keep.
///
/// # Both members were observed in ONE session, on one chain
///
/// The two runs differ in the change role and in nothing else: the same
/// issuance, the same two receipts at the same amount, the same two
/// destinations, the same owners, the same fee. So the difference
/// between their outputs is attributable to the change role, which is
/// the whole reason the axis is one axis.
///
/// # The without-change run is the lift's own control
///
/// It reproduced [`SPONSORED_ACCEPTED_TXID`] — the identity the sponsor
/// wave first observed and the explicit-runs wave observed again —
/// after the ceremony had been moved out of the test it lived in. A lift
/// that changed what the ceremony builds would have produced a different
/// identity, and this one did not move.
///
/// # ADR-015 public disposable test material throughout
///
/// One fixed regtest key answers the sponsor's request. A single key
/// answering once is not production multi-party sponsor signing, and
/// neither run should be read as establishing any.
pub mod sponsored_run_of_record {
    /// The identity the target computed for the accepted sponsor-signed
    /// explicit control that takes NO change.
    ///
    /// 1480 bytes submitted and 1480 read back from the node's own copy,
    /// equal to the submitted bytes; mined at height 6; a two-item
    /// sponsor witness of 72 and 33 bytes replayed from the adapter's
    /// answer; both owner signatures verified out of that copy against
    /// messages recomputed here, per input rather than per transaction.
    pub const SPONSORED_ACCEPTED_TXID: &str =
        "36cd6616ee518320b45f196d6ad0e6db894a5441683f57230e66e5608e6bc08c";

    /// How many bytes the without-change control handed the node.
    pub const SPONSORED_SUBMITTED_BYTES: usize = 1_480;

    /// The weight the target itself computed for it.
    pub const SPONSORED_TARGET_WEIGHT: u64 = 2_482;

    /// The fee the target weighed, in the reserve asset.
    ///
    /// The same figure BOTH runs declare. Holding it equal is what makes
    /// the two submissions comparable at all.
    pub const SPONSORED_FEE_WEIGHED: u64 = 250;

    /// Whether the without-change control crossed the relay boundary
    /// before the mine.
    ///
    /// Read off the submission path rather than assumed: the adapter
    /// offers a submission to `testmempoolaccept` first and reports an
    /// acceptance only where that answered allowed, then confirms with
    /// `generateblock`.
    pub const SPONSORED_CROSSED_RELAY_AND_BLOCK: bool = true;

    /// The identity the target computed for the accepted sponsor-signed
    /// explicit control that TAKES CHANGE.
    ///
    /// The first sponsored control in this workspace to carry a change
    /// role. 1636 bytes submitted and read back equal, mined at height
    /// 6, and the change output observed in the node's own copy at
    /// position 2 rather than inferred from the request: the reserve
    /// asset, the offered amount, and the deployment's own
    /// sponsor-change program.
    pub const SPONSORED_CHANGE_ACCEPTED_TXID: &str =
        "e3a4e4319e024bb021501d2ef0b7118b871953e05cb13b6c00e4d8a592982bb8";

    /// How many bytes the with-change control handed the node.
    ///
    /// One hundred and fifty-six more than the without-change control,
    /// which is the change output and the sponsor input's larger amount.
    pub const SPONSORED_CHANGE_SUBMITTED_BYTES: usize = 1_635;

    /// The weight the target itself computed for it.
    pub const SPONSORED_CHANGE_TARGET_WEIGHT: u64 = 2_871;

    /// What the sponsor coin was funded to for the with-change run.
    ///
    /// Above the offer, which is the whole of what a sponsored control
    /// taking change was missing: the construction places a change
    /// output only where the offer states a change amount, and an offer
    /// can only state one where the coin holds more than the fee.
    pub const SPONSORED_CHANGE_SPONSOR_FUNDED: u64 = 1_250;

    /// What the sponsor took back.
    pub const SPONSORED_CHANGE_TAKEN: u64 = 1_000;

    /// Which output position the change role occupied.
    ///
    /// Read out of the node's copy of the mined transaction, located by
    /// the deployment's own sponsor-change program rather than by
    /// counting: a position is what a shape degraded to the
    /// without-change form would still have, and the program is what it
    /// would not.
    pub const SPONSORED_CHANGE_OUTPUT_POSITION: usize = 2;

    /// Whether any ceremony in this workspace builds a sponsored control
    /// that TAKES CHANGE.
    ///
    /// `true`, and the running is what changed it.
    ///
    /// # The obstacle was not the one the spike predicted, and running
    /// decided it
    ///
    /// Two readings stood against each other. One said the demonstration
    /// deployment's sponsor-change program symbol is a fixture pattern
    /// no program hashes to, so a control taking change would die at its
    /// own change-role check the way the first sponsored controls died
    /// at the fee-role check. The other said the two symbols only look
    /// alike: the FEE role's program is target-structural, so
    /// construction wrote the empty program while the symbol was
    /// arbitrary and the two disagreed, whereas the CHANGE role's
    /// program is a deployment's own choice and construction writes the
    /// change output FROM the symbol — refusing outright if a sponsor
    /// capability offers any other destination.
    ///
    /// The second reading is the one that survived. The control was
    /// accepted at the first attempt with the symbol untouched: nothing
    /// was threaded, nothing was repointed, the committed taptree did
    /// not move, and [`SPONSORED_ACCEPTED_TXID`] reproduced beside it.
    /// What was actually missing was an OFFER that carries change, and
    /// what supplies one is a sponsor funding step that funds above the
    /// fee.
    pub const A_SPONSORED_CONTROL_TAKING_CHANGE_EXISTS: bool = true;
}

#[cfg(test)]
mod tests {
    use super::{SPONSOR_CHANGE, SPONSOR_FEE, SponsorShape, sponsored_run_of_record};

    /// The reserve sub-equation, stated where a change to either number
    /// has to notice.
    ///
    /// Elements balances per asset, and the reserve asset enters this
    /// transaction through exactly one input and leaves through the fee
    /// and the change. Nothing in this workspace checks the tally before
    /// a node does, so this is where the arithmetic is held.
    #[test]
    fn the_sponsor_coin_is_funded_to_exactly_the_fee_and_the_change() {
        for shape in SponsorShape::ALL {
            let expected = SPONSOR_FEE + shape.change().unwrap_or(0);
            assert_eq!(
                shape.sponsor_funding(),
                expected,
                "{} funds a coin the reserve equation does not close on",
                shape.case_name(),
            );
        }
    }

    /// The without-change shape funds exactly the offer, which is what
    /// makes its recorded identity reproducible.
    #[test]
    fn the_without_change_shape_still_funds_exactly_the_offer() {
        assert_eq!(SponsorShape::ChangeAbsent.sponsor_funding(), SPONSOR_FEE);
        assert!(SponsorShape::ChangeAbsent.change().is_none());
    }

    /// The register cites identities for the shapes it names.
    ///
    /// Bound to the shape constants rather than left as free numbers, so
    /// that a ceremony edited after its run fails here instead of
    /// quietly citing an identity for something else. Every figure is
    /// one the node produced.
    #[test]
    fn the_register_is_bound_to_the_shapes_that_produced_it() {
        use sponsored_run_of_record as record;

        assert_eq!(
            SponsorShape::ChangeAbsent.sponsor_funding(),
            record::SPONSORED_FEE_WEIGHED,
        );
        assert_eq!(
            SponsorShape::ChangePresent.sponsor_funding(),
            record::SPONSORED_CHANGE_SPONSOR_FUNDED,
        );
        assert_eq!(
            SponsorShape::ChangePresent.change(),
            Some(record::SPONSORED_CHANGE_TAKEN),
        );
        assert_eq!(
            record::SPONSORED_CHANGE_SPONSOR_FUNDED,
            record::SPONSORED_FEE_WEIGHED + record::SPONSORED_CHANGE_TAKEN,
        );

        // The two identities are DISTINCT, which is what says the change
        // role reached the bytes: a with-change run that had silently
        // degraded would have reproduced the other one.
        assert_ne!(
            record::SPONSORED_ACCEPTED_TXID,
            record::SPONSORED_CHANGE_ACCEPTED_TXID,
        );
        // The measured width of the change role, stated rather than
        // bounded: the extra output, and the wider sponsor value beside
        // it, cost exactly this many bytes at the node.
        assert_eq!(
            record::SPONSORED_CHANGE_SUBMITTED_BYTES - record::SPONSORED_SUBMITTED_BYTES,
            155,
        );
        assert_eq!(
            record::A_SPONSORED_CONTROL_TAKING_CHANGE_EXISTS,
            SponsorShape::ChangePresent.change().is_some(),
        );
    }

    /// The two shapes differ in the change role and in nothing else a
    /// reader has to take on trust.
    #[test]
    fn the_change_role_is_the_only_axis() {
        assert_eq!(
            SponsorShape::ChangePresent.sponsor_funding()
                - SponsorShape::ChangeAbsent.sponsor_funding(),
            SPONSOR_CHANGE,
        );
        assert_eq!(
            SponsorShape::ChangePresent.output_count(),
            SponsorShape::ChangeAbsent.output_count() + 1,
        );
    }
}
