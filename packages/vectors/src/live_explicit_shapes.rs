//! The positive explicit shapes of §15.1, each as its own candidate
//! against a real node.
//!
//! # What this module is for
//!
//! §15.1 names sixteen positive explicit classes and the evidence plan
//! answered none of them. Not one was blocked on a missing component:
//! the owner message is computed, the explicit lane has had a
//! first-party spend accepted, and every one of the sixteen was waiting
//! on a run of its own shape that nobody had taken.
//!
//! This module takes those runs. It is the explicit-lane sibling of
//! [`crate::live_multi_shapes`], which did the same for the private
//! table, and it reuses the explicit spine
//! [`crate::live_owner_observation`] already established — issue a
//! disposable asset, relink the deployment against it, fund the
//! explicit destination constructor, finalize a candidate over the
//! funded coins, census the owners, sign, submit, and read the mined
//! bytes back.
//!
//! # A shape is what varies, and it is the only thing that varies
//!
//! The spine funds two coins and creates two outputs, fixed. Everything
//! a §15.1 row is *about* is one of those numbers or one of the owners
//! attached to them, so the ceremony takes the shape as a parameter:
//! how many coins each published owner is funded, how many destinations
//! are created and for whom, what each destination is worth, and in
//! which order the receipts are offered.
//!
//! Nothing else moves between shapes. Same deployment, same disposable
//! asset per run, same signing material, same message model, same
//! submission path — so two runs that differ are runs whose shapes
//! differ.
//!
//! # A row moves on an acceptance of its own shape and on nothing else
//!
//! The ceremony existing answers nothing. What answers a row is the
//! artifact a run produces: an identity the target computed for a
//! transaction of that row's own shape, the node's own copy of it read
//! back equal to the bytes it was handed, and every signature in that
//! copy verified against a message this workspace recomputed. Those
//! three are separate observations and the record keeps them separate.
//!
//! The recorded identities live in [`run_of_record`], and
//! [`crate::live_evidence`] cites them from there, so a reader
//! following a row's answer arrives at a value one execution against a
//! real node produced rather than at a claim in a source file.
//!
//! # What an explicit acceptance does not establish
//!
//! It says nothing about the proof-bearing lane, for the reason
//! [`crate::live_owner_observation`] states at length: an explicit
//! candidate's output-witness entries are default-constructed and so
//! recoverable from the preimage, and a candidate carrying real range
//! proofs is not.
//!
//! It also discharges no negative row. An accepted transfer is a
//! positive control; a refusal becomes attributable *because* of it,
//! but attributability is not an answer.
//!
//! # Every key here is published test material
//!
//! The two signing scalars are the BIP-340 specification's own appendix
//! secret keys, ADR-015 public disposable test material at each use.
//! They authorize nothing on any network anybody uses, and the chain
//! each run is taken against is created and destroyed by that run.

use std::fmt::Write as _;

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::LeafVersion;
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::owner_key_oracle::verify_owner_signature;
use target_elements_conformance::protocol::{
    FundedOutput, MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, ValueField};
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
use transaction::taproot::{Digest32, leaf_hash};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed_order};
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, RESERVE_ASSET, SECOND_SCALAR, demonstration_live_abi,
    live_abi_for_asset, published_owner, reviewed_target, signing_material,
};

/// What each funded receipt holds.
///
/// The same figure every other live lane funds with. Keeping it equal
/// is what makes two runs' weights comparable; a shape that needed a
/// different amount would be varying two things at once.
const RECEIPT_AMOUNT: u64 = 5_000;

/// The auxiliary value every signature here is taken with.
///
/// A published constant rather than randomness, on the spine's own
/// reasoning: BIP-340 masks the scalar with it before deriving the
/// nonce, so fixing it is what makes each signature reproducible from
/// values that are all written down. ADR-015 public disposable test
/// material.
const SHAPE_AUXILIARY: [u8; FIELD_ELEMENT_BYTES] = [0x37; FIELD_ELEMENT_BYTES];

/// Which published owner a coin or a destination belongs to.
///
/// Two members because the demonstration deployment publishes two
/// owners. The name travels into the transcript, so a reader can see
/// which owner a run put where without recomputing a key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeOwner {
    /// The first published owner.
    First,
    /// The second published owner.
    Second,
}

impl ShapeOwner {
    /// This owner's signing scalar. ADR-015 public disposable test
    /// material: the BIP-340 appendix keys, and nothing else.
    const fn scalar(self) -> &'static [u8; FIELD_ELEMENT_BYTES] {
        match self {
            Self::First => &FIRST_SCALAR,
            Self::Second => &SECOND_SCALAR,
        }
    }

    /// The name this owner is written down under.
    const fn name(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Second => "second",
        }
    }
}

/// How a shape divides the consumed total among its destinations.
///
/// Two rules and no third, because §15.1 asks for exactly two things:
/// most rows are about cardinality or ownership and want the division
/// to be unremarkable, and one row is about the values themselves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValueRule {
    /// Equal shares, the remainder going to the last destination.
    EvenShares,
    /// The smallest destination the request type admits, and the rest.
    ///
    /// The smallest is one: [`ProtocolValue`] refuses zero by name, so
    /// one is the boundary rather than a small number somebody chose.
    /// The last destination takes the remainder, which for a two-output
    /// division is the largest value this shape's inputs can express.
    BoundaryExtremes,
}

/// One positive explicit class of §15.1, as a shape a run can take.
///
/// Each member names the row it is the shape of. Several members share
/// a cardinality and differ in what they are *about* — a two-input
/// transfer under one owner and a two-input transfer under two owners
/// have the same width and answer different rows — so the member is the
/// subject and never merely the counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplicitShape {
    /// One receipt consumed, one created.
    OneToOne,
    /// One receipt consumed, split into two.
    SplitIntoTwo,
    /// Two receipts consumed, merged into one.
    MergedIntoOne,
    /// Two receipts consumed, two created.
    SeveralToSeveral,
    /// Two receipts under the SAME owner, three created.
    ///
    /// The subject is the repetition rather than the width: one owner
    /// authorizes two separate inputs, each at its own position and each
    /// over its own message.
    RepeatedOwner,
    /// Two receipts under two DISTINCT published owners.
    SeveralDistinctOwners,
    /// Two destinations, both belonging to one owner.
    OneDestinationOwner,
    /// Two destinations belonging to two distinct owners.
    SeveralDestinationOwners,
    /// Destinations at the boundary values the request type admits.
    SemanticBoundaryValues,
    /// The receipts offered in the reverse of their canonical order.
    CanonicalInputNormalization,
    /// A transfer carrying no sponsor region at all.
    Sponsorless,
    /// As many receipt inputs as the candidate's bounds admit.
    MaximumInputs,
    /// As many receipt outputs as the candidate's bounds admit.
    MaximumOutputs,
}

impl ExplicitShape {
    /// Every shape this module runs.
    pub const ALL: &'static [Self] = &[
        Self::OneToOne,
        Self::SplitIntoTwo,
        Self::MergedIntoOne,
        Self::SeveralToSeveral,
        Self::RepeatedOwner,
        Self::SeveralDistinctOwners,
        Self::OneDestinationOwner,
        Self::SeveralDestinationOwners,
        Self::SemanticBoundaryValues,
        Self::CanonicalInputNormalization,
        Self::Sponsorless,
        Self::MaximumInputs,
        Self::MaximumOutputs,
    ];

    /// The §15.1 row this shape is the shape of.
    #[must_use]
    pub const fn row_name(self) -> &'static str {
        match self {
            Self::OneToOne => "one-input-to-one-output",
            Self::SplitIntoTwo => "one-input-split-into-two",
            Self::MergedIntoOne => "several-inputs-merged-into-one",
            Self::SeveralToSeveral => "several-inputs-to-several-outputs",
            Self::RepeatedOwner => "repeated-owner",
            Self::SeveralDistinctOwners => "several-distinct-owners",
            Self::OneDestinationOwner => "one-destination-owner",
            Self::SeveralDestinationOwners => "several-destination-owners",
            Self::SemanticBoundaryValues => "semantic-boundary-values",
            Self::CanonicalInputNormalization => "canonical-input-normalization",
            Self::Sponsorless => "sponsorless",
            Self::MaximumInputs => "candidate-maximum-inputs",
            Self::MaximumOutputs => "candidate-maximum-outputs",
        }
    }

    /// The name a run's transcript is written under.
    #[must_use]
    pub const fn case_name(self) -> &'static str {
        match self {
            Self::OneToOne => "explicit-one-to-one",
            Self::SplitIntoTwo => "explicit-split",
            Self::MergedIntoOne => "explicit-merge",
            Self::SeveralToSeveral => "explicit-several-to-several",
            Self::RepeatedOwner => "explicit-repeated-owner",
            Self::SeveralDistinctOwners => "explicit-several-owners",
            Self::OneDestinationOwner => "explicit-one-destination-owner",
            Self::SeveralDestinationOwners => "explicit-several-destination-owners",
            Self::SemanticBoundaryValues => "explicit-boundary-values",
            Self::CanonicalInputNormalization => "explicit-normalization",
            Self::Sponsorless => "explicit-sponsorless",
            Self::MaximumInputs => "explicit-maximum-inputs",
            Self::MaximumOutputs => "explicit-maximum-outputs",
        }
    }

    /// How many coins each published owner is funded, in order.
    ///
    /// A shape whose second entry is zero is funded in one step; the
    /// distinct-owners shape is the one that needs two, because two
    /// owners' explicit constructors are two different programs and a
    /// funding step names one program.
    #[must_use]
    pub const fn funded_coins(self) -> (u8, u8) {
        match self {
            Self::OneToOne
            | Self::SplitIntoTwo
            | Self::SemanticBoundaryValues
            | Self::Sponsorless
            | Self::SeveralDestinationOwners
            | Self::MaximumOutputs => (1, 0),
            Self::MergedIntoOne
            | Self::SeveralToSeveral
            | Self::RepeatedOwner
            | Self::OneDestinationOwner
            | Self::CanonicalInputNormalization => (2, 0),
            Self::SeveralDistinctOwners => (1, 1),
            Self::MaximumInputs => (3, 0),
        }
    }

    /// The owner each destination is created for, in output order.
    #[must_use]
    pub fn destination_owners(self) -> Vec<ShapeOwner> {
        match self {
            // The single-destination shapes, spelled in one arm: what
            // separates them is their INPUT side or the order their
            // receipts are offered in, and none of that is here.
            Self::OneToOne
            | Self::MergedIntoOne
            | Self::CanonicalInputNormalization
            | Self::Sponsorless
            | Self::MaximumInputs => {
                vec![ShapeOwner::Second]
            }
            Self::SplitIntoTwo
            | Self::SeveralToSeveral
            | Self::SeveralDistinctOwners
            | Self::SeveralDestinationOwners
            | Self::SemanticBoundaryValues => {
                vec![ShapeOwner::Second, ShapeOwner::First]
            }
            Self::OneDestinationOwner => vec![ShapeOwner::Second, ShapeOwner::Second],
            Self::RepeatedOwner | Self::MaximumOutputs => {
                vec![ShapeOwner::Second, ShapeOwner::First, ShapeOwner::Second]
            }
        }
    }

    /// How the consumed total is divided.
    const fn value_rule(self) -> ValueRule {
        match self {
            Self::SemanticBoundaryValues => ValueRule::BoundaryExtremes,
            _ => ValueRule::EvenShares,
        }
    }

    /// Whether the receipts are offered in the reverse of the order the
    /// node reported them.
    #[must_use]
    pub const fn offers_reversed_receipts(self) -> bool {
        matches!(self, Self::CanonicalInputNormalization)
    }

    /// How many receipts this shape consumes.
    #[must_use]
    pub const fn input_count(self) -> usize {
        let (first, second) = self.funded_coins();
        (first as usize) + (second as usize)
    }

    /// How many destinations this shape creates.
    #[must_use]
    pub fn output_count(self) -> usize {
        self.destination_owners().len()
    }
}

/// Why a shape run stopped before the node answered.
///
/// A construction refusal is this workspace declining to build
/// something, and it is never a target verdict. The two are kept apart
/// so a run that stopped here cannot be read as a chain having said
/// anything.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplicitShapeRefusal {
    /// The candidate substrate did not build.
    SubstrateUnavailable,
    /// The issuing step named no asset to relink against.
    IssuanceNamedNoAsset,
    /// The relink against the issued asset was refused.
    RelinkRefused,
    /// The funding step created no predecessor.
    FundingCreatedNoPredecessor,
    /// The node reported a funded output this ceremony could not read.
    MalformedFundedOutput,
    /// The node funded a different number of coins than the shape asked
    /// for.
    FundedWidthDisagrees,
    /// The candidate did not finalize over the funded coins.
    CandidateNotConstructible,
    /// An owner census over the finalized candidate was refused.
    CensusRefused,
    /// A signing request named an owner this deployment does not
    /// publish.
    SigningOwnerUnpublished,
    /// The signing material refused to sign.
    SigningRefused,
    /// The target accepted a transaction and reported no copy of it.
    AcceptanceCarriedNoReadback,
    /// The node's own copy did not decode.
    ReadbackDidNotDecode,
    /// The node's own copy carried no witness at an input position.
    ReadbackCarriesNoWitness,
}

/// One funded coin, as the node reported it and as the shape expected
/// it.
#[derive(Clone, Debug)]
pub struct ObservedShapeCoin {
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
    owner: ShapeOwner,
    matches_expectation: bool,
}

impl ObservedShapeCoin {
    /// Whether the node reported the coin the ceremony asked for.
    #[must_use]
    pub const fn matches_expectation(&self) -> bool {
        self.matches_expectation
    }

    /// The owner whose explicit constructor this coin was paid to.
    #[must_use]
    pub const fn owner(&self) -> ShapeOwner {
        self.owner
    }

    /// The outpoint the node reported.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }
}

/// One input's signature, taken out of the node's own copy and checked
/// against a message this workspace recomputed.
///
/// Per input rather than per transaction, and deliberately: a
/// multi-input candidate whose first signature verified would tell a
/// reader nothing about its second, and the shapes this module runs are
/// mostly multi-input.
#[derive(Clone, Debug)]
pub struct InputVerification {
    input_index: usize,
    owner: ShapeOwner,
    recomputed_message: [u8; 32],
    signature_bytes: usize,
    verified: bool,
}

impl InputVerification {
    /// Whether the accepted signature verifies against the recomputed
    /// message.
    #[must_use]
    pub const fn verified(&self) -> bool {
        self.verified
    }

    /// Which input this is.
    #[must_use]
    pub const fn input_index(&self) -> usize {
        self.input_index
    }
}

/// What reading the accepted transaction back established.
#[derive(Clone, Debug)]
pub struct ShapeReverification {
    accepted_txid: String,
    witness_txid: String,
    block_height: u32,
    readback_matches_submission: bool,
    inputs: Vec<InputVerification>,
}

impl ShapeReverification {
    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether every input's accepted signature verifies against its own
    /// recomputed message.
    #[must_use]
    pub fn every_input_verified(&self) -> bool {
        !self.inputs.is_empty() && self.inputs.iter().all(InputVerification::verified)
    }

    /// The identity the target computed.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// The per-input verifications.
    #[must_use]
    pub fn inputs(&self) -> &[InputVerification] {
        &self.inputs
    }
}

/// One shape run's record.
#[derive(Clone, Debug)]
pub struct ExplicitShapeRecord {
    shape: ExplicitShape,
    issued_asset: Option<String>,
    relinked: bool,
    coins: Vec<ObservedShapeCoin>,
    offered_outpoints: Vec<Outpoint>,
    canonical_outpoints: Vec<Outpoint>,
    destination_amounts: Vec<u64>,
    destination_owners: Vec<ShapeOwner>,
    input_owners: Vec<ShapeOwner>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    reverification: Option<ShapeReverification>,
    refusal: Option<ExplicitShapeRefusal>,
}

impl ExplicitShapeRecord {
    /// The shape this run took.
    #[must_use]
    pub const fn shape(&self) -> ExplicitShape {
        self.shape
    }

    /// The coins the node funded.
    #[must_use]
    pub fn coins(&self) -> &[ObservedShapeCoin] {
        &self.coins
    }

    /// How many bytes the ceremony handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the target answered at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// What reading the accepted transaction back established.
    #[must_use]
    pub const fn reverification(&self) -> Option<&ShapeReverification> {
        self.reverification.as_ref()
    }

    /// The construction refusal that stopped the run, if one did.
    #[must_use]
    pub const fn refusal(&self) -> Option<ExplicitShapeRefusal> {
        self.refusal
    }

    /// How many receipts were consumed.
    #[must_use]
    pub const fn input_count(&self) -> usize {
        self.input_owners.len()
    }

    /// How many destinations were created.
    #[must_use]
    pub const fn output_count(&self) -> usize {
        self.destination_amounts.len()
    }

    /// Whether the receipts were offered in a non-canonical order.
    ///
    /// The normalization row's own observable. It is read off the two
    /// recorded orders rather than declared by the shape, so a run in
    /// which the offered order happened to be canonical says so.
    #[must_use]
    pub fn offered_order_differs(&self) -> bool {
        self.offered_outpoints != self.canonical_outpoints
    }

    /// What this run establishes nothing about.
    #[must_use]
    pub const fn non_claims() -> [&'static str; 4] {
        [
            "evidences no negative case: an accepted transfer is a positive control, and a \
             refusal is attributable because of it rather than answered by it",
            "evidences nothing about the proof-bearing lane: an explicit candidate's \
             output-witness entries are default-constructed and a proof-bearing one's are not",
            "evidences no sponsor row: this ceremony builds no sponsor region at all",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

/// What the ceremony is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset the deployment is then linked against.
    Issue,
    /// Pay the issued asset to the first owner's explicit constructor.
    FundFirst,
    /// Pay the issued asset to the second owner's explicit constructor.
    FundSecond,
    /// Submit the shape's one candidate.
    Submit,
    /// Nothing further.
    Done,
}

/// The §15.1 shape ceremony.
pub struct ExplicitShapePlanner {
    stage: Stage,
    shape: ExplicitShape,
    abi: CandidateLiveTransferAbi,
    genesis_block_hash: Digest32,
    pending: Option<Vec<u8>>,
    record: ExplicitShapeRecord,
}

impl ExplicitShapePlanner {
    /// The ceremony for one shape, bound to one deployment's printed
    /// genesis identity.
    ///
    /// The identity arrives in the spelling the target prints and is
    /// reversed into the seed the target hashes with, the same relation
    /// the explicit spine applies at the same term.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI
    /// is unavailable.
    pub fn for_shape(
        shape: ExplicitShape,
        printed_genesis_identity: Digest32,
    ) -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        Ok(Self {
            stage: Stage::Issue,
            shape,
            abi,
            genesis_block_hash: printed_order(printed_genesis_identity),
            pending: None,
            record: ExplicitShapeRecord {
                shape,
                issued_asset: None,
                relinked: false,
                coins: Vec::new(),
                offered_outpoints: Vec::new(),
                canonical_outpoints: Vec::new(),
                destination_amounts: Vec::new(),
                destination_owners: Vec::new(),
                input_owners: Vec::new(),
                submitted_bytes: 0,
                observed_layer: None,
                observed_detail: None,
                accepted_txid: None,
                reverification: None,
                refusal: None,
            },
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &ExplicitShapeRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: ExplicitShapeRefusal) -> PlanRefused {
        self.record.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// One owner's explicit destination program under the current link.
    fn program_for(&self, owner: ShapeOwner) -> Result<Vec<u8>, ExplicitShapeRefusal> {
        explicit_destination_program(&self.abi, owner)
            .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)
    }

    /// A funding step paying `count` coins to one owner's constructor.
    fn funding_step(
        &self,
        name: &'static str,
        issue: bool,
        owner: ShapeOwner,
        count: u8,
    ) -> Result<OperationStep, ExplicitShapeRefusal> {
        Ok(OperationStep::new(
            name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.record.issued_asset.clone()
                },
                output_program: self.program_for(owner)?,
                outputs: count,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        ))
    }

    /// Link the deployment against the asset the target issued.
    fn relink(&mut self, response: &NativeOperationResponse) -> Result<(), ExplicitShapeRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(ExplicitShapeRefusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(ExplicitShapeRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(*identity.internal(), RESERVE_ASSET, FEE_PROGRAM_DIGEST)
            .map_err(|_| ExplicitShapeRefusal::RelinkRefused)?;
        self.record.issued_asset = Some(asset);
        self.record.relinked = true;
        self.abi = abi;
        Ok(())
    }

    /// Take one funding answer's coins, as the node reported them.
    ///
    /// Every field is the node's. The spent asset, value and program are
    /// terms of the owner message, so a signature over what the builder
    /// assumed rather than over what the chain holds would be a
    /// signature over a different message than the one the target forms.
    fn settle_funding(
        &mut self,
        owner: ShapeOwner,
        response: &NativeOperationResponse,
    ) -> Result<(), ExplicitShapeRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(ExplicitShapeRefusal::FundingCreatedNoPredecessor);
        }
        let expected_asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(ExplicitShapeRefusal::IssuanceNamedNoAsset)?;
        let program = self.program_for(owner)?;
        for funded in &response.funded_outputs {
            let coin = observed_coin(funded, expected_asset, &program, owner)?;
            self.record.coins.push(coin);
        }
        Ok(())
    }

    /// The public view the candidate is constructed against.
    fn view(&self) -> Result<PublicConstructionView, ExplicitShapeRefusal> {
        PublicConstructionView::new(self.record.coins.iter().map(|coin| {
            PublicOutputView::new(coin.outpoint, coin.asset, coin.value, coin.program.clone())
        }))
        .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)
    }

    /// The outpoints the shape offers, in the order it offers them.
    fn offered_outpoints(&self) -> Vec<Outpoint> {
        let mut points: Vec<Outpoint> = self
            .record
            .coins
            .iter()
            .map(ObservedShapeCoin::outpoint)
            .collect();
        if self.shape.offers_reversed_receipts() {
            points.reverse();
        }
        points
    }

    /// The destination values this shape asks for.
    ///
    /// Computed from the total the funded coins actually carry rather
    /// than from what the ceremony asked to be funded, because the
    /// division has to conserve against the chain's coins and not
    /// against an expectation.
    fn destination_amounts(&self, total: u64, outputs: usize) -> Option<Vec<u64>> {
        let width = u64::try_from(outputs).ok()?;
        if width == 0 || total < width {
            return None;
        }
        Some(match self.shape.value_rule() {
            ValueRule::EvenShares => {
                let share = total / width;
                let mut amounts = vec![share; outputs];
                let last = amounts.last_mut()?;
                *last = total - share * (width - 1);
                amounts
            }
            ValueRule::BoundaryExtremes => {
                let mut amounts = vec![1_u64; outputs];
                let last = amounts.last_mut()?;
                *last = total - (width - 1);
                amounts
            }
        })
    }

    /// The finalized candidate this shape's request produces, with the
    /// construction report completion needs.
    fn finalize(
        &mut self,
    ) -> Result<(FinalizedLiveTransfer, LiveConstructionReport), ExplicitShapeRefusal> {
        let view = self.view()?;
        let points = self.offered_outpoints();
        let total = self
            .record
            .coins
            .iter()
            .try_fold(0_u64, |sum, coin| match coin.value {
                ValueField::Explicit(amount) => sum.checked_add(amount),
                // Any value field but an explicit one is a coin this
                // ceremony cannot total, and this is the explicit lane.
                // A guessed total would be a conservation claim about
                // bytes nothing here can read.
                _ => None,
            })
            .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;

        let owners = self.shape.destination_owners();
        let amounts = self
            .destination_amounts(total, owners.len())
            .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;

        let mut destinations = Vec::with_capacity(owners.len());
        for (owner, amount) in owners.iter().copied().zip(amounts.iter().copied()) {
            let published = published_owner(owner.scalar())
                .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
            let value = ProtocolValue::new(amount)
                .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;
            destinations.push(LiveReceiptDestination::new(
                linker::OwnerParameter::new(published),
                value,
            ));
        }

        let request = LiveTransferRequest::new(
            points.clone(),
            destinations,
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        )
        .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;

        let target = reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
        let finalization = finalize_live_transfer(&target, &self.abi, &request, &view, None, None)
            .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;

        self.record.offered_outpoints = points;
        self.record.canonical_outpoints = request.receipts().iter().copied().collect();
        self.record.destination_amounts = amounts;
        self.record.destination_owners = owners;
        let report = finalization.report().clone();
        Ok((finalization.into_finalized(), report))
    }

    /// The owner census of one finalized candidate, under one
    /// deployment.
    fn census(
        finalized: &FinalizedLiveTransfer,
        genesis: Digest32,
    ) -> Result<OwnerSigningCensus, ExplicitShapeRefusal> {
        let target = reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?,
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
            LiveDeployment::new(genesis),
            &requests,
            &curve,
        )
        .map_err(|_| ExplicitShapeRefusal::CensusRefused)
    }

    /// Which published owner one signing request names.
    fn owner_of(request_owner: &linker::OwnerParameter) -> Option<ShapeOwner> {
        for candidate in [ShapeOwner::First, ShapeOwner::Second] {
            let published = published_owner(candidate.scalar()).ok()?;
            if linker::OwnerParameter::new(published) == *request_owner {
                return Some(candidate);
            }
        }
        None
    }

    /// The shape's candidate bytes, signed by whichever owner holds each
    /// input.
    fn candidate_bytes(&mut self) -> Result<Vec<u8>, ExplicitShapeRefusal> {
        let (finalized, report) = self.finalize()?;
        let census = Self::census(&finalized, self.genesis_block_hash)?;

        let mut input_owners = Vec::new();
        let mut responses = Vec::new();
        for signing in finalized.signing_requests() {
            let owner = Self::owner_of(signing.owner())
                .ok_or(ExplicitShapeRefusal::SigningOwnerUnpublished)?;
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            let material = signing_material(owner.scalar())
                .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
            let signature = material
                .sign(&message, &SHAPE_AUXILIARY)
                .map_err(|_| ExplicitShapeRefusal::SigningRefused)?
                .to_vec();
            input_owners.push((usize::from(signing.input()), owner));
            responses.push((signing.input(), LiveOwnerResponse::to(&signing, signature)));
        }

        input_owners.sort_unstable_by_key(|(index, _)| *index);
        self.record.input_owners = input_owners.into_iter().map(|(_, owner)| owner).collect();

        let report_target =
            reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;
        let built = complete_live_transfer(&report_target, authorized, report, None)
            .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;
        Ok(built.bytes())
    }

    /// Record what the target did with the shape's candidate.
    fn settle_submission(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), ExplicitShapeRefusal> {
        let submitted = self
            .pending
            .take()
            .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;
        self.record.observed_layer = Some(response.observed_layer);
        self.record
            .observed_detail
            .clone_from(&response.observed_detail);
        self.record
            .accepted_txid
            .clone_from(&response.accepted_txid);

        if matches!(response.observed_layer, ObservedOutcomeLayer::Accepted) {
            let readback = response
                .mined_readback
                .as_ref()
                .ok_or(ExplicitShapeRefusal::AcceptanceCarriedNoReadback)?;
            let record = self.reverify(readback, &submitted)?;
            self.record.reverification = Some(record);
        }
        Ok(())
    }

    /// Read the accepted transaction back and verify every input's
    /// signature against a recomputed message.
    fn reverify(
        &mut self,
        readback: &MinedFundingReadback,
        submitted: &[u8],
    ) -> Result<ShapeReverification, ExplicitShapeRefusal> {
        let decoded = TargetTransaction::decode(&readback.raw_transaction)
            .map_err(|_| ExplicitShapeRefusal::ReadbackDidNotDecode)?;
        let (finalized, _report) = self.finalize()?;
        let census = Self::census(&finalized, self.genesis_block_hash)?;
        let target = reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;

        let mut inputs = Vec::new();
        for signing in finalized.signing_requests() {
            let index = usize::from(signing.input());
            let owner = Self::owner_of(signing.owner())
                .ok_or(ExplicitShapeRefusal::SigningOwnerUnpublished)?;
            let entry = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;
            let recomputed =
                candidate_owner_message(&census, entry, WitnessVectorTreatment::BothGrown);
            let signature = decoded
                .witnesses()
                .get(index)
                .and_then(|witness| witness.stack().first())
                .cloned()
                .ok_or(ExplicitShapeRefusal::ReadbackCarriesNoWitness)?;
            let published = published_owner(owner.scalar())
                .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
            let verified =
                verify_owner_signature(&target, published.bytes(), &recomputed, &signature).is_ok();
            inputs.push(InputVerification {
                input_index: index,
                owner,
                recomputed_message: recomputed,
                signature_bytes: signature.len(),
                verified,
            });
        }
        inputs.sort_unstable_by_key(|entry| entry.input_index);

        Ok(ShapeReverification {
            accepted_txid: readback.transaction_id.clone(),
            witness_txid: readback.witness_transaction_id.clone(),
            block_height: readback.block_height,
            readback_matches_submission: readback.raw_transaction == submitted,
            inputs,
        })
    }
}

impl TargetOperationPlanner for ExplicitShapePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let (first_coins, second_coins) = self.shape.funded_coins();
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.relink(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundFirst;
                }
                Stage::FundFirst => {
                    if let Err(refusal) = self.settle_funding(ShapeOwner::First, response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = if second_coins > 0 {
                        Stage::FundSecond
                    } else {
                        Stage::Submit
                    };
                }
                Stage::FundSecond => {
                    if let Err(refusal) = self.settle_funding(ShapeOwner::Second, response) {
                        return Err(self.refuse(refusal));
                    }
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

        match self.stage {
            Stage::Issue => match self.funding_step(
                "issue-protocol-asset",
                true,
                ShapeOwner::First,
                first_coins,
            ) {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::FundFirst => match self.funding_step(
                "fund-first-owner-constructor",
                false,
                ShapeOwner::First,
                first_coins,
            ) {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::FundSecond => match self.funding_step(
                "fund-second-owner-constructor",
                false,
                ShapeOwner::Second,
                second_coins,
            ) {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Submit => {
                if self.record.coins.len() != self.shape.input_count() {
                    return Err(self.refuse(ExplicitShapeRefusal::FundedWidthDisagrees));
                }
                match self.candidate_bytes() {
                    Ok(bytes) => {
                        self.record.submitted_bytes = bytes.len();
                        self.pending = Some(bytes.clone());
                        Ok(Some(OperationStep::new(
                            self.shape.case_name(),
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

/// One owner's explicit destination program.
fn explicit_destination_program(
    abi: &CandidateLiveTransferAbi,
    owner: ShapeOwner,
) -> Result<Vec<u8>, VectorError> {
    Ok(abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(published_owner(owner.scalar())?),
            LiveTransferRepresentationPlan::Explicit,
        )
        .ok_or(VectorError::LiveSubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

/// One funded coin, as reported and as expected.
fn observed_coin(
    funded: &FundedOutput,
    expected_asset: AssetId,
    expected_program: &[u8],
    owner: ShapeOwner,
) -> Result<ObservedShapeCoin, ExplicitShapeRefusal> {
    let outpoint =
        outpoint_of(&funded.outpoint).ok_or(ExplicitShapeRefusal::MalformedFundedOutput)?;
    let asset = asset_of(&funded.asset).ok_or(ExplicitShapeRefusal::MalformedFundedOutput)?;
    let program = decode_hex(&funded.script).ok_or(ExplicitShapeRefusal::MalformedFundedOutput)?;
    let matches_expectation = asset == expected_asset
        && funded.amount_satoshis == RECEIPT_AMOUNT
        && program == expected_program;
    Ok(ObservedShapeCoin {
        outpoint,
        asset: AssetField::Explicit(asset),
        value: ValueField::Explicit(funded.amount_satoshis),
        program,
        owner,
        matches_expectation,
    })
}

/// Hex, for the transcript.
fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// One shape run's transcript.
///
/// Lines rather than a structure, on the pattern every live lane here
/// sets: the artifact is read by people and diffed by machines. Every
/// line is a fact the run observed or a value it computed, and no line
/// is a verdict about whether the run went well.
#[must_use]
pub fn render_explicit_shape(record: &ExplicitShapeRecord) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "explicit_shape {}", record.shape.case_name());
    let _ = writeln!(out, "row_name {}", record.shape.row_name());
    let _ = writeln!(
        out,
        "issued_asset {}",
        record.issued_asset.as_deref().unwrap_or("none")
    );
    let _ = writeln!(out, "relinked {}", record.relinked);
    let _ = writeln!(out, "funded_coins {}", record.coins.len());
    for coin in &record.coins {
        let _ = writeln!(
            out,
            "coin owner {} matches_expectation {}",
            coin.owner.name(),
            coin.matches_expectation
        );
    }
    let _ = writeln!(out, "input_count {}", record.input_count());
    for owner in &record.input_owners {
        let _ = writeln!(out, "input_owner {}", owner.name());
    }
    let _ = writeln!(out, "output_count {}", record.output_count());
    for (owner, amount) in record
        .destination_owners
        .iter()
        .zip(record.destination_amounts.iter())
    {
        let _ = writeln!(out, "destination owner {} value {amount}", owner.name());
    }
    let _ = writeln!(
        out,
        "offered_order_differs {}",
        record.offered_order_differs()
    );
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
    if let Some(check) = &record.reverification {
        let _ = writeln!(out, "readback_txid {}", check.accepted_txid);
        let _ = writeln!(out, "readback_witness_txid {}", check.witness_txid);
        let _ = writeln!(out, "readback_block_height {}", check.block_height);
        let _ = writeln!(
            out,
            "readback_matches_submission {}",
            check.readback_matches_submission
        );
        for input in &check.inputs {
            let _ = writeln!(
                out,
                "input {} owner {} signature_bytes {} verified {} message {}",
                input.input_index,
                input.owner.name(),
                input.signature_bytes,
                input.verified,
                hex(&input.recomputed_message)
            );
        }
        let _ = writeln!(out, "every_input_verified {}", check.every_input_verified());
    }
    let _ = writeln!(
        out,
        "construction_refusal {}",
        record
            .refusal
            .map_or_else(|| "none".to_owned(), |refusal| format!("{refusal:?}"))
    );
    let _ = writeln!(out, "evidences_no_negative_case true");
    let _ = writeln!(out, "builds_no_sponsor_region true");
    for claim in ExplicitShapeRecord::non_claims() {
        let _ = writeln!(out, "does_not_establish {claim}");
    }
    out
}

/// What one execution of every shape against a real node produced.
///
/// Hand-recorded from the transcripts that run wrote, on the pattern the
/// private lane's own register sets and for the same reason: a matrix
/// row cites a value a chain produced, and a value a chain produced has
/// to be written down somewhere a reader can reach it.
///
/// The target was the pinned Elements node the live lane binds itself
/// to, on a disposable development chain each run created and
/// destroyed. Every run issued its own asset and every one issued the
/// same identity, which is what a deterministic disposable chain does
/// and not a sign that one run was reported thirteen times.
///
/// # Thirteen runs and TEN identities, which is a finding rather than a
/// defect
///
/// Three pairs of runs produced the same identity, and in each case for
/// the same reason: the two rows are two CLASSES of one transaction
/// rather than two transactions. §15.1 names sixteen classes and a
/// single transfer is an instance of several of them at once — a
/// one-input one-output sponsorless transfer is simultaneously the
/// `one-input-to-one-output` class and the `sponsorless` class, and
/// building a second, gratuitously different transfer so that each row
/// could cite its own hex string would be dressing one fact up as two.
///
/// So the identity is shared and the sharing is stated:
///
/// - `one-input-to-one-output` and `sponsorless` share
///   [`ONE_TO_ONE_ACCEPTED_TXID`]. The accepted bytes are both.
/// - `one-input-split-into-two` and `several-destination-owners` share
///   [`SPLIT_ACCEPTED_TXID`]. A split into two destinations belonging to
///   two distinct owners is both.
/// - `several-inputs-merged-into-one` and
///   `canonical-input-normalization` share [`MERGE_ACCEPTED_TXID`], and
///   this pair is the strongest of the three rather than the weakest.
///   The normalization run offered its two receipts in the REVERSE of
///   their canonical order and the merge run offered them in it; the two
///   built byte-identical transactions and the node computed one
///   identity for them. The collision IS the normalization, observed
///   rather than asserted, and a run that had produced a second identity
///   would have been evidence that the request does not normalize.
///
/// What the rule this register is held to actually forbids is citing an
/// acceptance of a DIFFERENT shape. None of these does: in each pair the
/// accepted bytes are an instance of both rows' classes.
pub mod run_of_record {
    /// The disposable asset every run issued.
    pub const ISSUED_ASSET: &str =
        "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

    /// One receipt consumed, one created, sponsorless.
    ///
    /// Cited by `one-input-to-one-output` and by `sponsorless`. The
    /// smallest submission this lane has made, 593 bytes, for the
    /// structural reason that an explicit transfer carries no range
    /// proof at all.
    pub const ONE_TO_ONE_ACCEPTED_TXID: &str =
        "872a2294da5ea650a7a74ffd8a5932210930ab70d6a08a991eb3ea471ee29abb";

    /// How many bytes the one-to-one handed the node.
    pub const ONE_TO_ONE_SUBMITTED_BYTES: usize = 593;

    /// One receipt consumed, TWO created for two distinct owners.
    ///
    /// Cited by `one-input-split-into-two` and by
    /// `several-destination-owners`.
    pub const SPLIT_ACCEPTED_TXID: &str =
        "0fcf267058e83e06e87a950bbeec920a15e3641ef7f76c55df0a8fff544e64c1";

    /// How many bytes the split handed the node.
    pub const SPLIT_SUBMITTED_BYTES: usize = 751;

    /// TWO receipts consumed, ONE created.
    ///
    /// Cited by `several-inputs-merged-into-one` and by
    /// `canonical-input-normalization`, the second because the
    /// normalization run offered the same two receipts in the reverse
    /// order and the node computed this same identity for what it built.
    pub const MERGE_ACCEPTED_TXID: &str =
        "7a0ac33f0268e48ebeb1316dbc262c8d40569ba5c96274d1b8262f394c6f7c39";

    /// How many bytes the merge handed the node.
    pub const MERGE_SUBMITTED_BYTES: usize = 1_006;

    /// TWO receipts consumed, TWO created.
    pub const SEVERAL_TO_SEVERAL_ACCEPTED_TXID: &str =
        "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029";

    /// TWO receipts under ONE owner, three outputs created.
    ///
    /// The repetition is the subject: one published owner authorized two
    /// separate inputs, each at its own position and each over its own
    /// recomputed message, and both signatures verify out of the node's
    /// own copy.
    pub const REPEATED_OWNER_ACCEPTED_TXID: &str =
        "3f833570061c28f1b6cae0cd2abda65ed2c573bf62f418e847836a7999382114";

    /// TWO receipts under two DISTINCT published owners.
    ///
    /// Its destinations are the several-to-several run's exactly, and
    /// the identities differ anyway — because the SPENT programs differ,
    /// one coin having been paid to each owner's explicit constructor.
    /// That the two runs diverge on their input side alone is what makes
    /// this run about its input owners.
    pub const SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID: &str =
        "f87e1ef327f69fe1f6de5f763cc73d14edbe9425372f7a451d79d8e30e42b660";

    /// TWO destinations, both created for ONE owner.
    pub const ONE_DESTINATION_OWNER_ACCEPTED_TXID: &str =
        "c9bd2bd7ea47f2e7df3d95751d008e2db2448d6b9611425114b06e09d7a2a0a8";

    /// Destinations at the boundary values the request type admits.
    ///
    /// One and the remainder. The smallest is ONE because
    /// [`super::ProtocolValue`] refuses zero by name, so the value is
    /// the boundary the type states rather than a small number somebody
    /// picked — and the node accepted it, which is the fact worth
    /// having: nothing on this chain turned a one-unit output away.
    pub const BOUNDARY_VALUES_ACCEPTED_TXID: &str =
        "a53626927129a23c37681973eede8d23996743a1fb04adc71853e2bfe0d608be";

    /// THREE receipts consumed, the candidate's stated input bound.
    ///
    /// The widest submission of this table at 1421 bytes.
    pub const MAXIMUM_INPUTS_ACCEPTED_TXID: &str =
        "fce6e069897f841297803e36da5d51f3e7b4e15422ff0083a1cac4735147c112";

    /// THREE destinations created, the candidate's stated output bound.
    pub const MAXIMUM_OUTPUTS_ACCEPTED_TXID: &str =
        "6a5617cc547f0fe22cefa261ed2fb885a82a9e7293aefad76be5c35f0b531613";
}

impl ExplicitShape {
    /// The identity the target computed for this shape's accepted
    /// transaction.
    ///
    /// Every member answers, because every member ran and every run was
    /// accepted. Three identities are each shared by two shapes, and
    /// [`run_of_record`] states which and why.
    #[must_use]
    pub const fn observed_identity(self) -> &'static str {
        match self {
            Self::OneToOne | Self::Sponsorless => run_of_record::ONE_TO_ONE_ACCEPTED_TXID,
            Self::SplitIntoTwo | Self::SeveralDestinationOwners => {
                run_of_record::SPLIT_ACCEPTED_TXID
            }
            Self::MergedIntoOne | Self::CanonicalInputNormalization => {
                run_of_record::MERGE_ACCEPTED_TXID
            }
            Self::SeveralToSeveral => run_of_record::SEVERAL_TO_SEVERAL_ACCEPTED_TXID,
            Self::RepeatedOwner => run_of_record::REPEATED_OWNER_ACCEPTED_TXID,
            Self::SeveralDistinctOwners => run_of_record::SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID,
            Self::OneDestinationOwner => run_of_record::ONE_DESTINATION_OWNER_ACCEPTED_TXID,
            Self::SemanticBoundaryValues => run_of_record::BOUNDARY_VALUES_ACCEPTED_TXID,
            Self::MaximumInputs => run_of_record::MAXIMUM_INPUTS_ACCEPTED_TXID,
            Self::MaximumOutputs => run_of_record::MAXIMUM_OUTPUTS_ACCEPTED_TXID,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{ExplicitShape, run_of_record};

    #[test]
    fn every_shape_names_a_distinct_row_of_the_explicit_table() {
        // The shape table and the matrix table are two authorities, and
        // a shape that answered a row twice would be one of them
        // disagreeing with itself.
        let rows: BTreeSet<&str> = ExplicitShape::ALL
            .iter()
            .map(|shape| shape.row_name())
            .collect();
        assert_eq!(rows.len(), ExplicitShape::ALL.len());
        assert_eq!(ExplicitShape::ALL.len(), 13);
    }

    #[test]
    fn every_recorded_identity_is_a_target_identity() {
        for shape in ExplicitShape::ALL {
            let identity = shape.observed_identity();
            assert_eq!(
                identity.len(),
                64,
                "{shape:?} cites something that is not a target identity",
            );
            assert!(identity.chars().all(|digit| digit.is_ascii_hexdigit()));
        }
        assert_eq!(run_of_record::ISSUED_ASSET.len(), 64);
    }

    #[test]
    fn exactly_three_identities_are_shared_and_the_sharing_is_the_documented_one() {
        // The collision census, spelled rather than derived. A pair that
        // started sharing an identity because a shape stopped differing
        // would fail here, which is the whole point: two rows may share
        // an acceptance only where the accepted bytes are an instance of
        // both classes, and that is a judgement rather than an accident.
        let mut by_identity: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for shape in ExplicitShape::ALL {
            by_identity
                .entry(shape.observed_identity())
                .or_default()
                .insert(shape.row_name());
        }
        assert_eq!(by_identity.len(), 10, "thirteen runs, ten identities");

        let shared: BTreeSet<BTreeSet<&str>> = by_identity
            .values()
            .filter(|rows| rows.len() > 1)
            .cloned()
            .collect();
        assert_eq!(
            shared,
            BTreeSet::from([
                BTreeSet::from(["one-input-to-one-output", "sponsorless"]),
                BTreeSet::from(["one-input-split-into-two", "several-destination-owners"]),
                BTreeSet::from([
                    "canonical-input-normalization",
                    "several-inputs-merged-into-one"
                ]),
            ]),
        );
    }

    #[test]
    fn the_shape_cardinalities_are_the_ones_the_runs_reported() {
        // Read back off the shape rather than off the transcript, so a
        // shape edited after its run fails here instead of quietly
        // citing an identity for something else.
        assert_eq!(ExplicitShape::OneToOne.input_count(), 1);
        assert_eq!(ExplicitShape::OneToOne.output_count(), 1);
        assert_eq!(ExplicitShape::MergedIntoOne.input_count(), 2);
        assert_eq!(ExplicitShape::MergedIntoOne.output_count(), 1);
        assert_eq!(ExplicitShape::MaximumInputs.input_count(), 3);
        assert_eq!(ExplicitShape::MaximumOutputs.output_count(), 3);
        assert_eq!(ExplicitShape::SeveralDistinctOwners.funded_coins(), (1, 1));
        assert!(ExplicitShape::CanonicalInputNormalization.offers_reversed_receipts());
    }
}
