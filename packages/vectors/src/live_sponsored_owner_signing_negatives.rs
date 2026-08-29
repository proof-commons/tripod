//! The sponsored owner-signing negative ceremony: a sponsored successor
//! re-signed over its own mutated bytes, carrying the three mutants that
//! confuse the sponsor range with the protocol range.
//!
//! # The rows this ceremony drives
//!
//! Three, all of them faults of REGION rather than of value:
//! `receipt-sponsor-range-exchange` swaps a receipt coin into a sponsor
//! position; `sponsor-change-in-protocol-range` moves the sponsor-change
//! output into the destination prefix; and `sponsor-protocol-overlap`
//! claims one position for both regions at once. Each keeps the
//! per-asset sums intact, so each reaches the covenant's sponsor
//! isolation fragment, which inspects the asset at a sponsor position
//! and requires it not to be the protocol asset.
//!
//! # Why a ceremony of its own
//!
//! The sponsorless owner-signing ceremony builds a successor with no
//! sponsor input and no sponsor change, so none of the three regions
//! these rows confuse exists in it. A sponsored successor is a different
//! candidate, not a mutation of that one, and building it here leaves
//! every digest that ceremony recorded where it is.
//!
//! # The separator is a distinct byte range each
//!
//! All three refuse at the same clause and may draw the same words, so
//! the separating fact is the witnessless byte RANGE each mutation
//! confines itself to — the sponsor input region, the change-output
//! region, and the overlapped position are three disjoint areas of one
//! control. This is the `private-ct-imbalance` discipline applied across
//! the sponsor half of a candidate, and it takes the range rather than
//! the verdict because the verdict is the fragment's and not the row's.
//!
//! # Why every mutation is a REARRANGEMENT
//!
//! Each of the three keeps the multiset of assets and values exactly as
//! the control carried it, and moves only WHERE a coin or an output
//! sits, or which program a position names. That is not a stylistic
//! choice. The target's per-asset sum is checked before any script runs,
//! so a mutation that added, removed or revalued anything would be
//! refused at the tally and the observation would belong to conservation
//! rather than to a region row. Rearranging leaves the tally satisfied
//! and hands the candidate to the covenant, which is the only place a
//! region fault can be seen.
//!
//! # Why the receipt owners sign again and the sponsor does not
//!
//! The coordinator leaf checks the owner's signature BEFORE it inspects
//! any region, so a mutant carrying a stale owner signature would die at
//! that check and the row's own fault would never be reached. Each
//! mutant is therefore re-signed by the receipt owners over its own
//! bytes, through the negative-evidence census route built for exactly
//! this. The sponsor's authorization is NOT refreshed: it binds to the
//! bytes the control was finalized as, and the sponsor input sits in the
//! suffix, after the receipt inputs whose leaf carries the region
//! clauses. What that costs the ceremony is stated as a non-claim rather
//! than argued away.
//!
//! # Every key here is published test material
//!
//! The signing scalars are the BIP-340 appendix secret keys, admitted
//! under ADR-015's test-material rule `(´[ADR015-rule:security:test-material]´)`.
//! They authorize nothing on any network anyone uses, and the chain this
//! ceremony runs against is created and destroyed by the run.

use std::cell::RefCell;
use std::collections::BTreeMap;

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetFundingSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
};
use transaction::bytes::{
    AssetField, AssetId, Outpoint, TargetOutput, TargetTransaction, Txid, ValueField,
};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{LiveDeployment, OwnerSigningCensus, SpentOutputCensusEntry};
use transaction::live_construct::{
    LiveConstructionReport, complete_live_transfer, finalize_live_transfer,
};
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{
    AuthorizedLiveTransfer, LiveOwnerResponse, authorize_live_transfer,
};
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::taproot::Digest32;
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::bundle::fee_program_digest;
use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed_order};
use crate::live_owner_signing_negatives::{
    OwnerSigningNegativeRefusal, RECEIPT_AMOUNT, explicit_destination_program,
    explicit_spent_outputs, sign_explicit_candidate,
};
use crate::live_plan::{
    FIRST_SCALAR, SECOND_SCALAR, demonstration_live_abi, live_abi_for_asset, published_owner,
    reviewed_target, signing_material,
};
use crate::live_report::LiveMutationLocator;

/// How many receipts this ceremony funds and the control consumes.
const RECEIPT_COUNT: u8 = 2;

/// The fee the sponsored offer declares.
const SPONSOR_FEE: u64 = 250;

/// What the sponsor asks back as change.
///
/// A change output is REQUIRED here rather than optional: two of the
/// three rows are about where that output sits and what it names, and a
/// shape without one would leave them with nothing to move.
const SPONSOR_CHANGE: u64 = 1_000;

/// The auxiliary randomness every owner signature here is taken with.
///
/// Published, fixed and disposable, like the scalars themselves.
const SIGNING_AUXILIARY: [u8; 32] = [0x37; 32];

/// The shape the sponsored control must have for the three declarations
/// to mean what they say: three inputs, four outputs.
///
/// Two receipts and the sponsor suffix on the input side; two
/// destinations, the sponsor change and the fee on the output side.
const DECLARED_SHAPE: (usize, usize) = (3, 4);

/// The sponsor input's position: the suffix, after both receipts.
const SPONSOR_INPUT: usize = 2;

/// The receipt input that trades places with the sponsor input.
///
/// The SECOND receipt rather than the first. Input zero carries the
/// coordinator leaf, which is the one program that inspects the regions
/// at all; moving it would leave the candidate with no clause able to
/// see the fault its own mutation introduced.
const EXCHANGED_RECEIPT_INPUT: usize = 1;

/// The sponsor-change output's position: after both destinations.
const SPONSOR_CHANGE_OUTPUT: usize = 2;

/// The destination position the change is moved into, and whose program
/// the overlap mutant copies.
const PROTOCOL_RANGE_OUTPUT: usize = 1;

/// The ceremony's own name for the control submission.
pub const CONTROL_STEP: &str = "submit-sponsored-control";

/// The issuance step's name.
const ISSUE_STEP: &str = "issue-protocol-asset";

/// The sponsor-funding step's name.
const FUND_SPONSOR_STEP: &str = "fund-sponsor-region";

/// The receipt-funding step's name.
const FUND_RECEIPTS_STEP: &str = "fund-explicit-constructor";

/// The sponsor-authorization step's name.
const AUTHORIZE_STEP: &str = "authorize-sponsor-input";

/// The ceremony's own name for the receipt-exchange submission.
pub const RECEIPT_EXCHANGE_STEP: &str = "sponsor-receipt-range-exchange";

/// The ceremony's own name for the change-in-protocol-range submission.
pub const CHANGE_IN_PROTOCOL_RANGE_STEP: &str = "sponsor-change-in-protocol-range";

/// The ceremony's own name for the region-overlap submission.
pub const PROTOCOL_OVERLAP_STEP: &str = "sponsor-protocol-overlap";

/// The three sponsor-range mutants, in the order they are offered.
///
/// Offered before the control, so the coins the control consumes are
/// still unspent when it is finally submitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SponsorRangeMutant {
    /// A receipt coin traded into the sponsor position.
    ReceiptExchange,
    /// The sponsor-change output moved into the destination prefix.
    ChangeInProtocolRange,
    /// One position claimed by the sponsor and protocol regions at once.
    ProtocolOverlap,
}

/// Every mutant this ceremony stages, in offer order.
pub const SPONSOR_RANGE_MUTANTS: [SponsorRangeMutant; 3] = [
    SponsorRangeMutant::ReceiptExchange,
    SponsorRangeMutant::ChangeInProtocolRange,
    SponsorRangeMutant::ProtocolOverlap,
];

impl SponsorRangeMutant {
    /// The §15 row this mutant drives.
    #[must_use]
    pub const fn row(self) -> &'static str {
        match self {
            Self::ReceiptExchange => "receipt-sponsor-range-exchange",
            Self::ChangeInProtocolRange => "sponsor-change-in-protocol-range",
            Self::ProtocolOverlap => "sponsor-protocol-overlap",
        }
    }

    /// The ceremony's own name for this mutant's submission.
    #[must_use]
    pub const fn step(self) -> &'static str {
        match self {
            Self::ReceiptExchange => RECEIPT_EXCHANGE_STEP,
            Self::ChangeInProtocolRange => CHANGE_IN_PROTOCOL_RANGE_STEP,
            Self::ProtocolOverlap => PROTOCOL_OVERLAP_STEP,
        }
    }

    /// The spent-output positions this mutant trades, where it trades any.
    ///
    /// Only the input-side mutant does. The message an owner signs
    /// commits to every spent output in input order, so a rearrangement
    /// of the inputs that left the census in the old order would sign
    /// over a candidate the target does not see, and the mutant would die
    /// at the signature check instead of at the region clause.
    #[must_use]
    pub const fn spent_output_exchange(self) -> Option<(usize, usize)> {
        match self {
            Self::ReceiptExchange => Some((EXCHANGED_RECEIPT_INPUT, SPONSOR_INPUT)),
            Self::ChangeInProtocolRange | Self::ProtocolOverlap => None,
        }
    }

    /// Rearrange the control into this mutant.
    ///
    /// # Errors
    ///
    /// [`SponsoredOwnerSigningRefusal::ShapeNotAsDeclared`] where the
    /// control is not the shape the declarations describe;
    /// [`SponsoredOwnerSigningRefusal::MutantNotDistinct`] where the
    /// rearrangement would reproduce the control;
    /// [`SponsoredOwnerSigningRefusal::MutantNotConstructible`] where the
    /// rearranged candidate does not reassemble.
    fn apply(
        self,
        control: &TargetTransaction,
    ) -> Result<TargetTransaction, SponsoredOwnerSigningRefusal> {
        let shape = (control.inputs().len(), control.outputs().len());
        if shape != DECLARED_SHAPE {
            return Err(SponsoredOwnerSigningRefusal::ShapeNotAsDeclared {
                inputs: shape.0,
                outputs: shape.1,
            });
        }

        let mut inputs = control.inputs().to_vec();
        let mut outputs = control.outputs().to_vec();
        match self {
            Self::ReceiptExchange => inputs.swap(EXCHANGED_RECEIPT_INPUT, SPONSOR_INPUT),
            Self::ChangeInProtocolRange => {
                outputs.swap(PROTOCOL_RANGE_OUTPUT, SPONSOR_CHANGE_OUTPUT)
            }
            Self::ProtocolOverlap => {
                let claimed = outputs
                    .get(PROTOCOL_RANGE_OUTPUT)
                    .ok_or(SponsoredOwnerSigningRefusal::MutantNotConstructible)?
                    .program()
                    .to_vec();
                let change = outputs
                    .get(SPONSOR_CHANGE_OUTPUT)
                    .ok_or(SponsoredOwnerSigningRefusal::MutantNotConstructible)?;
                // The asset and the value stay exactly the control's. Only
                // the program moves, which is what makes the position
                // claimed by both regions rather than simply revalued.
                let overlapped =
                    TargetOutput::new(change.asset(), change.value(), change.nonce(), claimed);
                *outputs
                    .get_mut(SPONSOR_CHANGE_OUTPUT)
                    .ok_or(SponsoredOwnerSigningRefusal::MutantNotConstructible)? = overlapped;
            }
        }

        let mutant = TargetTransaction::with_output_witnesses(
            control.version(),
            inputs,
            outputs,
            control.lock_time(),
            control.witnesses().to_vec(),
            control.output_witnesses().to_vec(),
        )
        .map_err(|_| SponsoredOwnerSigningRefusal::MutantNotConstructible)?;

        // A rearrangement that found the two positions already identical
        // would hand the node the control a second time under another
        // row's name, so a collision fails loudly rather than quietly.
        if mutant.encode_without_witness() == control.encode_without_witness() {
            return Err(SponsoredOwnerSigningRefusal::MutantNotDistinct { step: self.step() });
        }
        Ok(mutant)
    }
}

/// What this ceremony refuses, before any node is asked.
///
/// Every member is a construction or infrastructure fact and none is a
/// target verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SponsoredOwnerSigningRefusal {
    /// The shared explicit build route refused, carrying its own reason.
    ExplicitBuildRefused(OwnerSigningNegativeRefusal),
    /// The issuance answer named no asset.
    IssuanceNamedNoAsset,
    /// A funding answer created no coin.
    FundingCreatedNoPredecessor,
    /// A funded coin could not be read back from the node's report.
    MalformedFundedOutpoint,
    /// The sponsor funding named no reserve asset.
    SponsorFundingNamedNoReserve,
    /// Two sponsor coins claimed different reserve assets.
    SponsorAssetsDisagree,
    /// The deployment could not be welded to the two reported assets.
    RelinkRefused,
    /// The reviewed target or a published owner is unavailable.
    SubstrateUnavailable,
    /// The sponsored control did not finalize, authorize or complete.
    ControlNotConstructible,
    /// An owner signature was refused.
    OwnerSigningRefused,
    /// The builder asked for other than exactly one sponsor signature.
    UnexpectedSponsorRequestCount(usize),
    /// The adapter did not authorize the sponsor input.
    AuthorizationDidNotHappen(ObservedOutcomeLayer),
    /// The authorization carried no witness to replay.
    AuthorizationCarriedNoWitness,
    /// The adapter bound its signature to bytes other than the ones sent.
    AuthorizationBoundToOtherBytes {
        /// How many bytes were sent.
        sent: usize,
        /// How many came back.
        echoed: usize,
    },
    /// The completed control did not decode.
    ControlNotDecodable,
    /// The control is not the shape the three declarations describe.
    ShapeNotAsDeclared {
        /// How many inputs the control carried.
        inputs: usize,
        /// How many outputs it carried.
        outputs: usize,
    },
    /// A rearrangement would have reproduced the control.
    MutantNotDistinct {
        /// The step whose rearrangement changed nothing.
        step: &'static str,
    },
    /// A rearranged candidate did not reassemble.
    MutantNotConstructible,
    /// The submitted mutant's witnessless diff was not the range measured
    /// over the two candidates.
    MutationNotConfined {
        /// The step whose diff escaped its declaration.
        step: &'static str,
        /// What the submitted bytes actually moved.
        touched: Option<(usize, usize)>,
        /// What the candidates said would move.
        declared: Option<(usize, usize)>,
    },
    /// Two mutants declared the SAME witnessless range.
    ///
    /// The three rows are separated by range alone, so a collision would
    /// leave two of them indistinguishable in the record.
    RangesNotDistinct {
        /// The step that collided with an earlier one.
        step: &'static str,
        /// The range both declared.
        range: (usize, usize),
    },
}

impl From<OwnerSigningNegativeRefusal> for SponsoredOwnerSigningRefusal {
    fn from(refusal: OwnerSigningNegativeRefusal) -> Self {
        Self::ExplicitBuildRefused(refusal)
    }
}

/// One sponsor-range mutant, as this ceremony built, submitted and
/// observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SponsorRangeObservation {
    mutant: SponsorRangeMutant,
    declared_range: (usize, usize),
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl SponsorRangeObservation {
    /// Which of the three mutants this is.
    #[must_use]
    pub const fn mutant(&self) -> SponsorRangeMutant {
        self.mutant
    }

    /// The §15 row it drives.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.mutant.row()
    }

    /// The ceremony's own name for its submission.
    #[must_use]
    pub const fn step(&self) -> &'static str {
        self.mutant.step()
    }

    /// The separating fact: the half-open witnessless byte range this
    /// mutation confined itself to.
    ///
    /// All three refuse behind the same signature check on the same leaf
    /// and answer with the same generic equality failure, so the verdict
    /// is the fragment's rather than any row's. The range is what makes
    /// an observation this row's, which is why it is MEASURED over the
    /// two candidates and checked against the bytes actually submitted.
    #[must_use]
    pub const fn declared_range(&self) -> (usize, usize) {
        self.declared_range
    }

    /// How many bytes this mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the target refused this mutant at, where it was observed.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }
}

/// The control, as this ceremony submitted and observed it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SponsoredControlObservation {
    submitted_bytes: usize,
    sponsor_input: u16,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
}

impl SponsoredControlObservation {
    /// How many bytes the control handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The sponsor input's position in the finalized control.
    #[must_use]
    pub const fn sponsor_input(&self) -> u16 {
        self.sponsor_input
    }

    /// The layer the target answered the control at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }

    /// The transaction identity the node accepted the control under.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }
}

/// One run's transcript.
#[derive(Clone, Debug, Default)]
pub struct SponsoredOwnerSigningRecord {
    issued_asset: Option<String>,
    reserve_asset: Option<String>,
    relinked: bool,
    receipt_coins: usize,
    mutants: Vec<SponsorRangeObservation>,
    control: Option<SponsoredControlObservation>,
    refusal: Option<SponsoredOwnerSigningRefusal>,
}

impl SponsoredOwnerSigningRecord {
    /// The protocol asset the run issued, as the node printed it.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// The reserve asset the sponsor funding named.
    #[must_use]
    pub fn reserve_asset(&self) -> Option<&str> {
        self.reserve_asset.as_deref()
    }

    /// Whether the deployment relinked against both reported assets.
    #[must_use]
    pub const fn relinked(&self) -> bool {
        self.relinked
    }

    /// How many receipt coins the node funded.
    #[must_use]
    pub const fn receipt_coins(&self) -> usize {
        self.receipt_coins
    }

    /// The sponsor-range mutants, in the order they were offered.
    #[must_use]
    pub fn mutants(&self) -> &[SponsorRangeObservation] {
        &self.mutants
    }

    /// The control, where it was built.
    #[must_use]
    pub const fn control(&self) -> Option<&SponsoredControlObservation> {
        self.control.as_ref()
    }

    /// What the ceremony refused, where it refused.
    #[must_use]
    pub const fn refusal(&self) -> Option<&SponsoredOwnerSigningRefusal> {
        self.refusal.as_ref()
    }

    /// The capture locator for one step, where that step carries one.
    ///
    /// Only the three mutants do. The control is an acceptance, and a
    /// locator on it would file a mutation against a run that recorded
    /// none.
    #[must_use]
    pub fn capture_locator(&self, step: &str) -> Option<LiveMutationLocator> {
        let observed = self
            .mutants
            .iter()
            .find(|observation| observation.step() == step)?;
        let (start, end) = observed.declared_range();
        Some(LiveMutationLocator::WitnesslessRange { start, end })
    }

    /// What this ceremony does NOT establish, in its own words.
    #[must_use]
    pub fn non_claims() -> Vec<&'static str> {
        vec![
            "separates its three rows by RANGE and not by verdict: all three are refused behind \
             one leaf's signature check and answer with the same generic equality failure, so the \
             witnessless byte range each mutation confined itself to is what makes an observation \
             one row's rather than another's",
            "does not name WHICH clause of the coordinator leaf refuses a mutant: the destination \
             closure, the sponsor isolation and the issuance absence all sit behind that leaf's \
             signature check, and which of them speaks first for a given rearrangement is the \
             target's selection and not this ceremony's prediction",
            "does not claim the sponsor's own authorization is valid on a mutant: it binds to the \
             bytes the control was finalized as, and only the receipt owners re-sign. The sponsor \
             input sits in the suffix, after the receipt inputs whose leaf carries the region \
             clauses, so a refusal reached on an earlier input is reported as the layer it was \
             reached at and nothing is asserted about the suffix",
            "establishes nothing about an unbalanced region fault: every mutation here rearranges \
             what the control already carried, so the per-asset sums are the control's and the \
             refusal observed is the covenant's rather than the tally's",
            "predicts no words beyond the layer, the refusal being the target's own to phrase",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

/// How the staged envelope answers a signing request.
enum Answers {
    /// Record what was asked and answer with a placeholder, so the request
    /// the adapter must be handed can be collected at all.
    Recording(RefCell<Vec<(u16, Vec<u8>)>>),
    /// Answer with what the adapter returned, keyed by input position.
    Replaying(BTreeMap<u16, SponsorSignature>),
}

/// The sponsor envelope this ceremony stages, in both of its passes.
///
/// One type rather than two, because the offer must be identical across
/// the passes: a second spelling could disagree with the first about what
/// was finalized, and the bytes the adapter signed would then not be the
/// bytes replayed into.
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
    /// The construction writes the change output from the deployment's
    /// sponsor-change symbol and refuses any other destination, so a
    /// capability naming one would be naming the value already in use.
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

/// One coin the node reported funding.
#[derive(Clone, Debug)]
struct ObservedCoin {
    outpoint: Outpoint,
    asset: AssetId,
    value: ValueField,
    program: Vec<u8>,
}

impl ObservedCoin {
    /// The explicit amount this coin holds, where it holds one explicitly.
    ///
    /// Written as a test of the one form that HAS an amount rather than as
    /// a match over the forms. The value vocabulary is open, so a form
    /// this ceremony has never met is not an explicit amount either: the
    /// answer for it is the same answer, reached by the same reading,
    /// instead of by a wildcard arm that would have to be revisited to say
    /// so and would meanwhile answer quietly for a form somebody added on
    /// purpose.
    const fn explicit_amount(&self) -> Option<u64> {
        if let ValueField::Explicit(amount) = self.value {
            Some(amount)
        } else {
            None
        }
    }
}

/// Where the ceremony is in its order of operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Issue the protocol asset.
    Issue,
    /// Fund the sponsor region, which is what names the reserve.
    FundSponsor,
    /// Fund the two receipts, against a deployment now welded to both
    /// assets.
    FundReceipts,
    /// Ask the adapter to authorize the sponsor input.
    Authorize,
    /// Offer one of the three mutants, before the control so the coins it
    /// consumes stay unspent.
    Mutant(usize),
    /// Submit the sponsored control, last, which is what consumes them.
    Control,
    /// Nothing further.
    Done,
}

/// The sponsored owner-signing negative ceremony.
pub struct SponsoredOwnerSigningNegativePlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    explicit_program: Vec<u8>,
    genesis_block_hash: Digest32,
    sponsor: Option<ObservedCoin>,
    receipts: Vec<ObservedCoin>,
    staged: Option<(AuthorizedLiveTransfer, LiveConstructionReport)>,
    finalized: Option<FinalizedLiveTransfer>,
    spent_outputs: Vec<SpentOutputCensusEntry>,
    sponsor_request: Option<(u16, Vec<u8>)>,
    control_bytes: Option<Vec<u8>>,
    mutant_bytes: Vec<Vec<u8>>,
    record: SponsoredOwnerSigningRecord,
}

impl SponsoredOwnerSigningNegativePlanner {
    /// The ceremony bound to one deployment's printed genesis identity.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the candidate ABI or
    /// the explicit destination constructor is unavailable.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        let explicit_program = explicit_destination_program(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            abi,
            explicit_program,
            genesis_block_hash: printed_order(printed_genesis_identity),
            sponsor: None,
            receipts: Vec::new(),
            staged: None,
            finalized: None,
            spent_outputs: Vec::new(),
            sponsor_request: None,
            control_bytes: None,
            mutant_bytes: Vec::new(),
            record: SponsoredOwnerSigningRecord::default(),
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &SponsoredOwnerSigningRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: SponsoredOwnerSigningRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The issuance or receipt-funding step for the constructor's program.
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

    /// One funding step's coins, taken from the node's report of them.
    fn funded_coins(
        response: &NativeOperationResponse,
    ) -> Result<Vec<ObservedCoin>, SponsoredOwnerSigningRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(SponsoredOwnerSigningRefusal::FundingCreatedNoPredecessor);
        }
        let mut coins = Vec::with_capacity(response.funded_outputs.len());
        for funded in &response.funded_outputs {
            coins.push(ObservedCoin {
                outpoint: outpoint_of(&funded.outpoint)
                    .ok_or(SponsoredOwnerSigningRefusal::MalformedFundedOutpoint)?,
                asset: asset_of(&funded.asset)
                    .ok_or(SponsoredOwnerSigningRefusal::MalformedFundedOutpoint)?,
                value: ValueField::Explicit(funded.amount_satoshis),
                program: decode_hex(&funded.script)
                    .ok_or(SponsoredOwnerSigningRefusal::MalformedFundedOutpoint)?,
            });
        }
        Ok(coins)
    }

    /// Remember the protocol asset the issuance named.
    ///
    /// The deployment is NOT relinked here: half of what it is welded to
    /// is still unknown, the reserve arriving with the sponsor funding.
    fn remember_issued_asset(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsoredOwnerSigningRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::IssuanceNamedNoAsset)?;
        asset_of(&asset).ok_or(SponsoredOwnerSigningRefusal::IssuanceNamedNoAsset)?;
        self.record.issued_asset = Some(asset);
        Ok(())
    }

    /// Weld the deployment to both reported assets and the fee program.
    fn relink(&mut self, reserve: AssetId) -> Result<(), SponsoredOwnerSigningRefusal> {
        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::IssuanceNamedNoAsset)?;
        let protocol =
            asset_of(&printed).ok_or(SponsoredOwnerSigningRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(
            *protocol.internal(),
            *reserve.internal(),
            fee_program_digest(),
        )
        .map_err(|_| SponsoredOwnerSigningRefusal::RelinkRefused)?;
        self.explicit_program = explicit_destination_program(&abi)
            .map_err(|_| SponsoredOwnerSigningRefusal::RelinkRefused)?;
        self.abi = abi;
        self.record.relinked = true;
        Ok(())
    }

    /// Read the sponsor coin and settle the one reserve identity.
    fn settle_sponsor_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsoredOwnerSigningRefusal> {
        let coins = Self::funded_coins(response)?;
        let mut reserve: Option<AssetId> = None;
        for coin in &coins {
            match reserve {
                Some(known) if known != coin.asset => {
                    return Err(SponsoredOwnerSigningRefusal::SponsorAssetsDisagree);
                }
                Some(_) => {}
                None => reserve = Some(coin.asset),
            }
        }
        let reserve = reserve.ok_or(SponsoredOwnerSigningRefusal::SponsorFundingNamedNoReserve)?;
        let coin = coins
            .into_iter()
            .next()
            .ok_or(SponsoredOwnerSigningRefusal::FundingCreatedNoPredecessor)?;
        self.record.reserve_asset = Some(printed_asset(&reserve));
        self.sponsor = Some(coin);
        self.relink(reserve)
    }

    /// The offer this run's sponsor makes.
    ///
    /// Built in one place and never spelled twice, because the staging and
    /// the replay pass must make the SAME offer.
    fn offer(&self) -> Result<SponsorOffer, SponsoredOwnerSigningRefusal> {
        let sponsor = self
            .sponsor
            .as_ref()
            .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        SponsorOffer::new(
            [sponsor.outpoint],
            SPONSOR_FEE,
            Some(ValueField::Explicit(SPONSOR_CHANGE)),
        )
        .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)
    }

    /// The construction view: every receipt shown, and the sponsor coin
    /// shown beside them.
    fn construction_view(&self) -> Result<PublicConstructionView, SponsoredOwnerSigningRefusal> {
        let sponsor = self
            .sponsor
            .as_ref()
            .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        let mut views = Vec::with_capacity(self.receipts.len() + 1);
        for coin in &self.receipts {
            views.push(PublicOutputView::new(
                coin.outpoint,
                AssetField::Explicit(coin.asset),
                coin.value,
                coin.program.clone(),
            ));
        }
        // The sponsor coin is SHOWN and not merely named: construction
        // refuses a sponsor input it cannot see, and refuses one whose
        // asset is not the deployment's reserve.
        views.push(PublicOutputView::new(
            sponsor.outpoint,
            AssetField::Explicit(sponsor.asset),
            sponsor.value,
            sponsor.program.clone(),
        ));
        PublicConstructionView::new(views)
            .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)
    }

    /// The sponsored transfer request over the funded receipts.
    fn transfer_request(&self) -> Result<LiveTransferRequest, SponsoredOwnerSigningRefusal> {
        let mut points = Vec::with_capacity(self.receipts.len());
        let mut total = 0_u64;
        for coin in &self.receipts {
            points.push(coin.outpoint);
            total = total
                .checked_add(
                    coin.explicit_amount()
                        .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?,
                )
                .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        }
        let receipts = vec![
            destination(&SECOND_SCALAR, total / 2)?,
            destination(&FIRST_SCALAR, total - total / 2)?,
        ];
        LiveTransferRequest::new(
            points,
            receipts,
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsored,
            SponsorChangeRequest::Requested,
            None,
        )
        .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)
    }

    /// Every owner signature the finalized control asks for.
    ///
    /// Both receipts were funded to one destination program, so one scalar
    /// answers for both.
    fn owner_responses(
        &self,
        finalized: &FinalizedLiveTransfer,
    ) -> Result<Vec<(u16, LiveOwnerResponse)>, SponsoredOwnerSigningRefusal> {
        let target =
            reviewed_target().map_err(|_| SponsoredOwnerSigningRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| SponsoredOwnerSigningRefusal::SubstrateUnavailable)?,
        );
        let census = OwnerSigningCensus::from_explicit_finalized(
            &target,
            finalized,
            LiveDeployment::new(self.genesis_block_hash),
            &curve,
        )
        .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        let material = signing_material(&FIRST_SCALAR)
            .map_err(|_| SponsoredOwnerSigningRefusal::SubstrateUnavailable)?;
        let mut responses = Vec::new();
        for signing in finalized.signing_requests() {
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            let signature = material
                .sign(&message, &SIGNING_AUXILIARY)
                .map_err(|_| SponsoredOwnerSigningRefusal::OwnerSigningRefused)?
                .to_vec();
            responses.push((signing.input(), LiveOwnerResponse::to(&signing, signature)));
        }
        Ok(responses)
    }

    /// Finalize and owner-authorize the sponsored control, collecting the
    /// sponsor request the builder issues.
    fn stage_control(&mut self) -> Result<(), SponsoredOwnerSigningRefusal> {
        let view = self.construction_view()?;
        let request = self.transfer_request()?;
        let envelope = StagedEnvelope {
            offer: self.offer()?,
            answers: Answers::Recording(RefCell::new(Vec::new())),
        };

        let target =
            reviewed_target().map_err(|_| SponsoredOwnerSigningRefusal::SubstrateUnavailable)?;
        let finalization =
            finalize_live_transfer(&target, &self.abi, &request, &view, Some(&envelope), None)
                .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        let report = finalization.report().clone();
        let finalized = finalization.into_finalized();

        // The spent-output census is taken BEFORE the finalized form is
        // consumed by authorization, and retained: every mutant is signed
        // against it, and the input-side mutant against a rearrangement
        // of it.
        self.spent_outputs = explicit_spent_outputs(&finalized, self.genesis_block_hash)?;
        let responses = self.owner_responses(&finalized)?;
        self.finalized = Some(finalized.clone());

        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        complete_live_transfer(&target, authorized.clone(), report.clone(), Some(&envelope))
            .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)?;

        let Answers::Recording(recorded) = &envelope.answers else {
            // Unreachable: the value was just built with this arm.
            return Err(SponsoredOwnerSigningRefusal::ControlNotConstructible);
        };
        let recorded = recorded.borrow().clone();
        if recorded.len() != 1 {
            return Err(SponsoredOwnerSigningRefusal::UnexpectedSponsorRequestCount(
                recorded.len(),
            ));
        }
        self.sponsor_request = recorded.into_iter().next();
        self.staged = Some((authorized, report));
        Ok(())
    }

    /// The sponsor signing step, carrying the exact finalized bytes.
    fn authorize_step(&mut self) -> Result<OperationStep, SponsoredOwnerSigningRefusal> {
        self.stage_control()?;
        let (input, sent) = self
            .sponsor_request
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        let sponsor = self
            .sponsor
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        self.record.control = Some(SponsoredControlObservation {
            sponsor_input: input,
            ..SponsoredControlObservation::default()
        });
        Ok(OperationStep::new(
            AUTHORIZE_STEP,
            OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                finalized_transaction: sent,
                sponsor_input_index: input,
                sponsor_outpoint: WireOutpoint {
                    txid: printed_txid(&sponsor.outpoint.txid()),
                    vout: sponsor.outpoint.index(),
                },
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            })),
        ))
    }

    /// Replay the adapter's answer into the finalization the request was
    /// formed against, then cut all three mutants from the result.
    fn settle_authorization(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsoredOwnerSigningRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(SponsoredOwnerSigningRefusal::AuthorizationDidNotHappen(
                response.observed_layer,
            ));
        }
        let echoed = response
            .signature_bound_to
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::AuthorizationCarriedNoWitness)?;
        if response.sponsor_witness.is_empty() {
            return Err(SponsoredOwnerSigningRefusal::AuthorizationCarriedNoWitness);
        }
        let (input, sent) = self
            .sponsor_request
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        if echoed != sent {
            return Err(
                SponsoredOwnerSigningRefusal::AuthorizationBoundToOtherBytes {
                    sent: sent.len(),
                    echoed: echoed.len(),
                },
            );
        }

        let (authorized, report) = self
            .staged
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        let target =
            reviewed_target().map_err(|_| SponsoredOwnerSigningRefusal::SubstrateUnavailable)?;
        let replaying = StagedEnvelope {
            offer: self.offer()?,
            answers: Answers::Replaying(BTreeMap::from([(
                input,
                SponsorSignature::new(echoed, response.sponsor_witness.clone()),
            )])),
        };
        let control_bytes = complete_live_transfer(&target, authorized, report, Some(&replaying))
            .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)?
            .bytes();

        self.build_mutants(&control_bytes)?;
        if let Some(control) = self.record.control.as_mut() {
            control.submitted_bytes = control_bytes.len();
        }
        self.control_bytes = Some(control_bytes);
        Ok(())
    }

    /// Cut all three mutants from the sponsor-signed control's own bytes.
    fn build_mutants(&mut self, control_bytes: &[u8]) -> Result<(), SponsoredOwnerSigningRefusal> {
        let control = TargetTransaction::decode(control_bytes)
            .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotDecodable)?;
        let finalized = self
            .finalized
            .clone()
            .ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        let control_witnessless = control.encode_without_witness();

        for mutant in SPONSOR_RANGE_MUTANTS {
            let candidate = mutant.apply(&control)?;
            let declared = witnessless_changed_range(
                &control_witnessless,
                &candidate.encode_without_witness(),
            );
            let spent_outputs = exchanged_spent_outputs(&self.spent_outputs, mutant);
            let (bytes, _message) = sign_explicit_candidate(
                &candidate,
                &finalized,
                &spent_outputs,
                self.genesis_block_hash,
            )?;

            // The whole witnessless diff between the two SUBMITTED byte
            // strings must be the range measured over the two candidates:
            // the re-signing moves witness bytes, which the declaration
            // does not cover, and nothing else may have moved.
            let touched = witnessless_changed_range(
                &control_witnessless,
                &decode_witnessless(&bytes, control_bytes),
            );
            if touched != declared {
                return Err(SponsoredOwnerSigningRefusal::MutationNotConfined {
                    step: mutant.step(),
                    touched,
                    declared,
                });
            }
            let range = declared.ok_or(SponsoredOwnerSigningRefusal::MutantNotDistinct {
                step: mutant.step(),
            })?;
            // The three rows are separated by range alone, so a range this
            // run has already declared cannot be declared again.
            if self
                .record
                .mutants
                .iter()
                .any(|observed| observed.declared_range() == range)
            {
                return Err(SponsoredOwnerSigningRefusal::RangesNotDistinct {
                    step: mutant.step(),
                    range,
                });
            }

            self.record.mutants.push(SponsorRangeObservation {
                mutant,
                declared_range: range,
                submitted_bytes: bytes.len(),
                observed_layer: None,
                observed_detail: None,
            });
            self.mutant_bytes.push(bytes);
        }
        Ok(())
    }

    /// Record what the target did with the mutant at one offer position.
    fn settle_mutant(&mut self, index: usize, response: &NativeOperationResponse) {
        if let Some(observed) = self.record.mutants.get_mut(index) {
            observed.observed_layer = Some(response.observed_layer);
            observed
                .observed_detail
                .clone_from(&response.observed_detail);
        }
    }

    /// Record what the target did with the control.
    fn settle_control(&mut self, response: &NativeOperationResponse) {
        if let Some(control) = self.record.control.as_mut() {
            control.observed_layer = Some(response.observed_layer);
            control
                .observed_detail
                .clone_from(&response.observed_detail);
            control.accepted_txid.clone_from(&response.accepted_txid);
        }
    }

    /// One staged submission, by its position in the offer order.
    fn submission_step(
        name: &'static str,
        bytes: Option<&[u8]>,
    ) -> Result<OperationStep, SponsoredOwnerSigningRefusal> {
        let bytes = bytes.ok_or(SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
        Ok(OperationStep::new(
            name,
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: bytes.to_vec(),
            })),
        ))
    }

    /// Take the answer to the step just offered and advance the stage.
    fn settle(&mut self, response: &NativeOperationResponse) -> Result<(), PlanRefused> {
        match self.stage {
            Stage::Issue => {
                if let Err(refusal) = self.remember_issued_asset(response) {
                    return Err(self.refuse(refusal));
                }
                self.stage = Stage::FundSponsor;
            }
            // The sponsor region is funded BEFORE the receipts, and the
            // order is load-bearing: the reserve arrives in that answer,
            // the deployment is welded to it, and only then are the
            // receipts funded to programs that moved when it was.
            Stage::FundSponsor => {
                if let Err(refusal) = self.settle_sponsor_funding(response) {
                    return Err(self.refuse(refusal));
                }
                self.stage = Stage::FundReceipts;
            }
            Stage::FundReceipts => match Self::funded_coins(response) {
                Ok(coins) => {
                    self.record.receipt_coins = coins.len();
                    self.receipts = coins;
                    self.stage = Stage::Authorize;
                }
                Err(refusal) => return Err(self.refuse(refusal)),
            },
            Stage::Authorize => {
                if let Err(refusal) = self.settle_authorization(response) {
                    return Err(self.refuse(refusal));
                }
                self.stage = Stage::Mutant(0);
            }
            Stage::Mutant(index) => {
                self.settle_mutant(index, response);
                self.stage = if index + 1 < SPONSOR_RANGE_MUTANTS.len() {
                    Stage::Mutant(index + 1)
                } else {
                    Stage::Control
                };
            }
            Stage::Control => {
                self.settle_control(response);
                self.stage = Stage::Done;
            }
            Stage::Done => {}
        }
        Ok(())
    }

    /// The step this ceremony offers at its current stage.
    fn step(&mut self) -> Result<Option<OperationStep>, PlanRefused> {
        let staged = match self.stage {
            Stage::Issue => Ok(self.funding_step(ISSUE_STEP, true)),
            Stage::FundSponsor => Ok(OperationStep::new(
                FUND_SPONSOR_STEP,
                OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                    sponsor_outputs: 1,
                    amount_per_sponsor_output: SPONSOR_FEE.saturating_add(SPONSOR_CHANGE),
                })),
            )),
            Stage::FundReceipts => Ok(self.funding_step(FUND_RECEIPTS_STEP, false)),
            Stage::Authorize => self.authorize_step(),
            Stage::Mutant(index) => match SPONSOR_RANGE_MUTANTS.get(index) {
                Some(mutant) => Self::submission_step(
                    mutant.step(),
                    self.mutant_bytes.get(index).map(Vec::as_slice),
                ),
                None => Err(SponsoredOwnerSigningRefusal::ControlNotConstructible),
            },
            Stage::Control => Self::submission_step(CONTROL_STEP, self.control_bytes.as_deref()),
            Stage::Done => return Ok(None),
        };
        match staged {
            Ok(step) => Ok(Some(step)),
            Err(refusal) => Err(self.refuse(refusal)),
        }
    }
}

impl TargetOperationPlanner for SponsoredOwnerSigningNegativePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            self.settle(response)?;
        }
        self.step()
    }
}

/// One receipt destination, paid to a published owner.
fn destination(
    scalar: &[u8; 32],
    amount: u64,
) -> Result<LiveReceiptDestination, SponsoredOwnerSigningRefusal> {
    let owner =
        published_owner(scalar).map_err(|_| SponsoredOwnerSigningRefusal::SubstrateUnavailable)?;
    let value = ProtocolValue::new(amount)
        .map_err(|_| SponsoredOwnerSigningRefusal::ControlNotConstructible)?;
    Ok(LiveReceiptDestination::new(
        linker::OwnerParameter::new(owner),
        value,
    ))
}

/// The spent-output census a mutant is signed against.
///
/// Rearranged exactly as its inputs were, where it rearranged any. The
/// owner's message commits to every spent output in input order, so a
/// census left in the control's order would form a message for a
/// candidate the target does not see.
fn exchanged_spent_outputs(
    census: &[SpentOutputCensusEntry],
    mutant: SponsorRangeMutant,
) -> Vec<SpentOutputCensusEntry> {
    let mut entries = census.to_vec();
    match mutant.spent_output_exchange() {
        Some((left, right)) if left < entries.len() && right < entries.len() => {
            entries.swap(left, right);
        }
        Some(_) | None => {}
    }
    entries
}

/// The witnessless serialization of a candidate that decoded from wire
/// bytes, falling back to the control's own when it does not decode.
///
/// The fallback is deliberately the CONTROL's bytes rather than the
/// mutant's: an undecodable submission then measures as no change at all
/// and is refused as an unconfined mutation, instead of measuring as a
/// change spanning the whole serialization and passing for a declaration.
fn decode_witnessless(bytes: &[u8], control_bytes: &[u8]) -> Vec<u8> {
    TargetTransaction::decode(bytes).map_or_else(
        |_| {
            TargetTransaction::decode(control_bytes)
                .map_or_else(|_| control_bytes.to_vec(), |tx| tx.encode_without_witness())
        },
        |tx| tx.encode_without_witness(),
    )
}

/// The half-open range two byte strings differ over, or `None` where they
/// do not differ.
///
/// The rule the recorder applies when it checks a `WitnesslessRange`
/// declaration, restated here so a ceremony declares what the recorder
/// will measure rather than something close to it: the start is the
/// common prefix, and the end is taken from the LEFT string's length less
/// the common suffix.
fn witnessless_changed_range(left: &[u8], right: &[u8]) -> Option<(usize, usize)> {
    if left == right {
        return None;
    }
    let start = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(left.len().saturating_sub(start))
        .min(right.len().saturating_sub(start));
    Some((start, left.len().saturating_sub(suffix)))
}

/// One asset identity, in the spelling a target prints.
fn printed_asset(asset: &AssetId) -> String {
    let mut bytes = *asset.internal();
    bytes.reverse();
    hex(&bytes)
}

/// One transaction identity, in the spelling a target prints.
fn printed_txid(txid: &Txid) -> String {
    let mut bytes = *txid.internal();
    bytes.reverse();
    hex(&bytes)
}

/// Lower-case hexadecimal, one pair per byte.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// One run's transcript, one fact per line.
#[must_use]
pub fn render_sponsored_owner_signing_negatives(record: &SponsoredOwnerSigningRecord) -> String {
    let mut lines = vec!["role sponsored-owner-signing-negative-run".to_owned()];
    lines.push(format!(
        "issued_asset {}",
        record.issued_asset().unwrap_or("none")
    ));
    lines.push(format!(
        "reserve_asset {}",
        record.reserve_asset().unwrap_or("none")
    ));
    lines.push(format!("relinked {}", record.relinked()));
    lines.push(format!("receipt_coins {}", record.receipt_coins()));

    if record.mutants().is_empty() {
        lines.push("sponsor_range_mutants none".to_owned());
    }
    for observed in record.mutants() {
        let (start, end) = observed.declared_range();
        lines.push(format!(
            "sponsor_range_mutant row {} step {} witnessless_range {start}-{end} submitted_bytes {} layer {} detail {}",
            observed.row(),
            observed.step(),
            observed.submitted_bytes(),
            observed
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            observed.observed_detail().unwrap_or("none"),
        ));
    }

    if let Some(control) = record.control() {
        lines.push(format!(
            "control sponsor_input {} submitted_bytes {} layer {} txid {} detail {}",
            control.sponsor_input(),
            control.submitted_bytes(),
            control
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            control.accepted_txid().unwrap_or("none"),
            control.observed_detail().unwrap_or("none"),
        ));
    } else {
        lines.push("control none".to_owned());
    }

    if let Some(refusal) = record.refusal() {
        lines.push(format!("ceremony_refused {refusal:?}"));
    }

    for claim in SponsoredOwnerSigningRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }
    lines.push("each_row_by_its_own_mutant true".to_owned());

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::{
        DECLARED_SHAPE, EXCHANGED_RECEIPT_INPUT, PROTOCOL_RANGE_OUTPUT, RECEIPT_COUNT,
        SPONSOR_CHANGE_OUTPUT, SPONSOR_INPUT, SPONSOR_RANGE_MUTANTS, SponsorRangeMutant,
        witnessless_changed_range,
    };

    #[test]
    fn the_declared_shape_is_the_sponsored_one_the_three_rows_need() {
        // Two receipts and the sponsor suffix; two destinations, the
        // sponsor change and the fee. Every position the three mutants
        // name has to exist in that shape, or a declaration would name a
        // field the candidate does not have.
        assert_eq!(
            DECLARED_SHAPE.0,
            usize::from(RECEIPT_COUNT) + 1,
            "the input side does not carry the sponsor suffix beside the receipts",
        );
        assert!(
            SPONSOR_INPUT < DECLARED_SHAPE.0 && EXCHANGED_RECEIPT_INPUT < DECLARED_SHAPE.0,
            "an input the exchange names is outside the declared shape",
        );
        assert!(
            SPONSOR_CHANGE_OUTPUT < DECLARED_SHAPE.1 && PROTOCOL_RANGE_OUTPUT < DECLARED_SHAPE.1,
            "an output a mutant names is outside the declared shape",
        );
    }

    #[test]
    fn the_exchange_leaves_the_coordinator_input_where_it_was() {
        // Input zero carries the only leaf that inspects the regions at
        // all. An exchange that moved it would leave the candidate with
        // no clause able to see the fault the exchange introduced, and
        // the observation would belong to whatever refused instead.
        assert_ne!(
            EXCHANGED_RECEIPT_INPUT, 0,
            "the exchange moves the coordinator input out of position zero",
        );
        assert_ne!(
            EXCHANGED_RECEIPT_INPUT, SPONSOR_INPUT,
            "the exchange would trade the sponsor input with itself",
        );
    }

    #[test]
    fn the_two_output_mutants_name_positions_in_different_regions() {
        // One moves the change INTO the protocol range and the other
        // gives a sponsor position a protocol program. Both rest on the
        // two indices being in different regions to begin with.
        assert_ne!(
            PROTOCOL_RANGE_OUTPUT, SPONSOR_CHANGE_OUTPUT,
            "the change output and the destination it is confused with are one position",
        );
        assert!(
            PROTOCOL_RANGE_OUTPUT < SPONSOR_CHANGE_OUTPUT,
            "the destination prefix does not precede the sponsor change",
        );
    }

    #[test]
    fn every_mutant_is_offered_once_and_names_its_own_row_and_step() {
        // The roster binds each step to one row, so two mutants sharing a
        // step or a row would file two observations under one name.
        for (index, mutant) in SPONSOR_RANGE_MUTANTS.iter().enumerate() {
            for other in SPONSOR_RANGE_MUTANTS.iter().skip(index + 1) {
                assert_ne!(mutant.row(), other.row(), "two mutants claim one row");
                assert_ne!(mutant.step(), other.step(), "two mutants claim one step");
            }
        }
    }

    #[test]
    fn only_the_input_side_mutant_rearranges_the_spent_outputs() {
        // The owner's message commits to the spent outputs in input
        // order, so the census follows an input rearrangement and must
        // not follow an output-only one.
        assert_eq!(
            SponsorRangeMutant::ReceiptExchange.spent_output_exchange(),
            Some((EXCHANGED_RECEIPT_INPUT, SPONSOR_INPUT)),
            "the input exchange does not carry the census with it",
        );
        assert!(
            SponsorRangeMutant::ChangeInProtocolRange
                .spent_output_exchange()
                .is_none(),
            "an output-only mutant disturbs the spent-output census",
        );
        assert!(
            SponsorRangeMutant::ProtocolOverlap
                .spent_output_exchange()
                .is_none(),
            "an output-only mutant disturbs the spent-output census",
        );
    }

    #[test]
    fn the_declared_range_is_the_one_the_recorder_measures() {
        // Worked by hand rather than by restating the implementation: the
        // two strings share three leading bytes and two trailing ones, so
        // the range runs from three to the left length less two.
        assert_eq!(
            witnessless_changed_range(&[1, 2, 3, 9, 9, 8, 7], &[1, 2, 3, 4, 5, 8, 7]),
            Some((3, 5)),
        );
        assert_eq!(witnessless_changed_range(&[1, 2, 3], &[1, 2, 3]), None);
    }
}
