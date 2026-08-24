//! The live-transfer target-native run (§13.2's requests and responses).
//!
//! A plan of funding and submission steps that drives the candidate
//! live-transfer pipeline against a real Elements node, and the typed
//! record of what that node observed.
//!
//! # What this run can establish, and what it cannot
//!
//! It can establish three things, and they are worth having. That the
//! linked live-receipt constructors are *fundable target outputs* — the
//! ceremony pays real coins to the exact programs the deployment's
//! constructors determine, which is §14.3's target materialization
//! performed rather than assumed. That the candidate's finalized bytes
//! are a transaction a node will decode and judge at all. And which
//! §1.11 layer the node's verdict arrives at, per submitted form.
//!
//! It cannot discharge a single §15 row, and the reason is stated in
//! [`crate::live_evidence`] rather than discovered here: no owner
//! signature can be produced over the digest §10.2's authorization
//! fragment checks against, so the submitted transfers carry a witness
//! whose signature position holds opaque bytes. Every rejection is
//! therefore attributable to that, and §19.2's condition — that a
//! negative case was refused *for its intended relation* — is met by
//! none of them.
//!
//! # So why run it
//!
//! Because *where* the node stops is itself a finding, and it is not one
//! anybody could have asserted in advance. The explicit transfer's whole
//! covenant runs: the coordinator authenticates the asset, the
//! constructors, the class, the counts and the conservation of a real
//! transaction spending real coins, and evaluation reaches the owner's
//! signature check before anything fails. A rejection at
//! [`ObservedOutcomeLayer::ScriptPathRejection`] naming the signature is
//! the sharpest available statement of what is left: one missing digest,
//! and nothing else in the pipeline.
//!
//! It also settles what the run *cannot* ask. The two questions Wave 9
//! left for a target — the value field's form and the missing range proof
//! — need a confidential receipt to spend, and the funding step has no
//! confidential form to create one with. That is recorded as
//! [`LiveFormNotSubmitted::NoConfidentialPredecessorCanBeFunded`] rather
//! than guessed at.
//!
//! # The observations are recorded, never graded
//!
//! [`LiveNativeObservation`] carries a layer and the node's own text. No
//! function here compares one against an expectation, because there is no
//! expectation this run is entitled to: §1.11 admits no target-negative
//! claim without a complete target transaction refused for its own
//! relation, and this run has none.

use std::collections::{BTreeMap, BTreeSet};

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetFundingSubject, TargetSubmissionSubject, WireOutpoint,
};
use transaction::bytes::{AssetField, AssetId, Outpoint, Txid, ValueField};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_construct::finalize_live_transfer;
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleFixtureValues;
use crate::live_evidence::UNAUTHORIZING_SIGNATURE;
use crate::live_plan::{
    FIRST_SCALAR, SECOND_SCALAR, demonstration_live_abi, live_abi_for_asset, published_owner,
    reviewed_target,
};

/// The published randomness the private construction consumes.
const PUBLISHED_RANDOMNESS: [u8; 32] = [0x7e; 32];

/// How many bytes the explicit transfer of record serialized to.
///
/// From the run [`observed_run_of_record`] describes: the exact length of
/// the byte string handed to the node, recorded so the weight beside it
/// can be read as a weight *of something* rather than as a bare figure.
const RECORDED_EXPLICIT_SERIALIZED_BYTES: u64 = 1_164;

/// The weight this workspace computed for those exact bytes.
///
/// §18.4's prediction half, taken by decoding the submitted serialization
/// and weighing the result.
const RECORDED_EXPLICIT_PREDICTED_WEIGHT: u64 = 1_911;

/// The weight the node computed for those exact bytes.
///
/// §18.4's observation half, and the figure this whole comparison rests
/// on. It exists because the executor reads a weight back from the node's
/// own `decoderawtransaction` even for a transaction the node refused —
/// which is the only reason a candidate that cannot be accepted (§1.7)
/// has any target resource figure at all.
///
/// It is a *separate constant* from the prediction above, and equal to it
/// only because the run made it so. Spelling one constant and using it
/// twice would have made the agreement true by construction, which is the
/// one thing §18.4's comparison must never be.
const RECORDED_EXPLICIT_OBSERVED_WEIGHT: u64 = 1_911;

/// What the plan is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset, at a program nobody will spend.
    ///
    /// The asset has to exist before the deployment can be linked against
    /// it, so this step's own outputs are incidental: what it is for is
    /// the identity the target chose.
    Issue,
    /// Pay the issued asset to the re-linked explicit constructor.
    FundExplicit,
    /// Pay the same asset to the re-linked private constructor.
    FundPrivate,
    /// Submit the explicit candidate transfer.
    SubmitExplicit,
    /// Submit the private candidate transfer.
    SubmitPrivate,
    /// Nothing further.
    Done,
}

/// Which submitted form one observation is about.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveNativeStep {
    /// The issuance the deployment is then linked against.
    IssueProtocolAsset,
    /// The funding ceremony's explicit-constructor half.
    FundExplicitConstructor,
    /// The funding ceremony's private-constructor half.
    FundPrivateConstructor,
    /// The explicit candidate transfer.
    SubmitExplicitTransfer,
    /// The private candidate transfer.
    SubmitPrivateTransfer,
}

impl LiveNativeStep {
    /// Every step, in the order the plan asks for them.
    pub const ALL: &'static [Self] = &[
        Self::IssueProtocolAsset,
        Self::FundExplicitConstructor,
        Self::FundPrivateConstructor,
        Self::SubmitExplicitTransfer,
        Self::SubmitPrivateTransfer,
    ];

    /// The plan's own name for the step.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::IssueProtocolAsset => "issue-protocol-asset",
            Self::FundExplicitConstructor => "fund-explicit-constructor",
            Self::FundPrivateConstructor => "fund-private-constructor",
            Self::SubmitExplicitTransfer => "submit-explicit-transfer",
            Self::SubmitPrivateTransfer => "submit-private-transfer",
        }
    }
}

/// What this workspace predicts one submitted serialization costs
/// (§18.4).
///
/// Taken from the exact bytes the submission carries, and derived by
/// decoding them rather than by remembering what was built: §18.4
/// compares a prediction with an observation *over the same exact bytes*,
/// and a figure carried over from the builder would be a figure about the
/// value the builder held rather than about the serialization the node
/// was handed.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PredictedTransferResources {
    serialized_bytes: u64,
    weight: Option<u64>,
}

impl PredictedTransferResources {
    /// How many bytes were submitted.
    #[must_use]
    pub const fn serialized_bytes(self) -> u64 {
        self.serialized_bytes
    }

    /// The weight this workspace computes for those bytes.
    ///
    /// Absent when the submitted bytes do not decode here, which is
    /// itself a finding about the serializer and is carried as an absence
    /// rather than as a weight of zero.
    #[must_use]
    pub const fn weight(self) -> Option<u64> {
        self.weight
    }
}

/// One thing the node was observed to do.
///
/// The layer and the node's own words, and nothing derived from them. A
/// field holding "expected" or "passed" would be the discharge-by-intent
/// this whole package exists to prevent, and this run in particular is
/// entitled to no expectations at all.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveNativeObservation {
    step: LiveNativeStep,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    accepted_txid: Option<String>,
    funded: usize,
    observed_weight: Option<u64>,
}

impl LiveNativeObservation {
    /// Which step this is about.
    #[must_use]
    pub const fn step(&self) -> LiveNativeStep {
        self.step
    }

    /// The §1.11 layer the node's answer arrived at.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// The node's own text, verbatim and unmapped.
    ///
    /// Recorded because the failure *class* is the point of two of these
    /// submissions: a confidential output refused for a missing range
    /// proof and one refused for an unbalanced transaction are the same
    /// layer and different answers.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// The identity the target gave an accepted transaction.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// How many outputs a funding step created.
    #[must_use]
    pub const fn funded(&self) -> usize {
        self.funded
    }

    /// The weight the node computed for the bytes this step submitted.
    ///
    /// The observation half of §18.4, and the reason it exists at all is
    /// that a *refused* transaction still has a weight: the executor
    /// reads it back from the node's own `decoderawtransaction`, so a
    /// candidate nothing can witness still yields one real target figure
    /// over its real bytes. Absent for a step that reached no target
    /// verdict, and absent rather than zero — §18.4 forbids reading a
    /// missing observation as agreement.
    #[must_use]
    pub const fn observed_weight(&self) -> Option<u64> {
        self.observed_weight
    }
}

/// The plan's own record of one live-transfer run.
///
/// Kept beside the executor's transcript rather than inside it, on the
/// same reasoning [`crate::operation::OperationTranscript`] states: two
/// independently kept records of one run can be laid beside each other,
/// and one record cannot be checked against itself.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveNativeTranscript {
    issued_asset: Option<String>,
    relinked: bool,
    not_submitted: BTreeSet<(LiveTransferRepresentationPlan, LiveFormNotSubmitted)>,
    explicit_program: Vec<u8>,
    private_program: Vec<u8>,
    explicit_coins: Vec<WireOutpoint>,
    private_coins: Vec<WireOutpoint>,
    observations: Vec<LiveNativeObservation>,
    predicted: BTreeMap<LiveTransferRepresentationPlan, PredictedTransferResources>,
    refusal: Option<LiveNativeRefusal>,
}

impl LiveNativeTranscript {
    /// The disposable asset the ceremony issued.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// Whether the deployment was linked against that asset.
    ///
    /// The run's own precondition, recorded rather than assumed: every
    /// program funded and every transfer submitted after this is true
    /// belongs to a deployment welded to the chain the run is on. A run
    /// that funded before it linked would have paid coins to a deployment
    /// of a different asset.
    #[must_use]
    pub const fn relinked(&self) -> bool {
        self.relinked
    }

    /// The explicit constructor's own witness program.
    #[must_use]
    pub fn explicit_program(&self) -> &[u8] {
        &self.explicit_program
    }

    /// The private constructor's own witness program.
    #[must_use]
    pub fn private_program(&self) -> &[u8] {
        &self.private_program
    }

    /// How many predecessor receipts were materialized, per plan.
    #[must_use]
    pub fn materialized(&self) -> BTreeMap<LiveTransferRepresentationPlan, usize> {
        BTreeMap::from([
            (
                LiveTransferRepresentationPlan::Explicit,
                self.explicit_coins.len(),
            ),
            (
                LiveTransferRepresentationPlan::PrivateCommitted,
                self.private_coins.len(),
            ),
        ])
    }

    /// Every observation, in the order the steps were answered.
    #[must_use]
    pub fn observations(&self) -> &[LiveNativeObservation] {
        &self.observations
    }

    /// What this workspace predicted about each submitted serialization
    /// (§18.4).
    ///
    /// Keyed by the form submitted, and holding an entry only for a form
    /// this run actually offered a target. A form in
    /// [`Self::not_submitted`] has no prediction here, because there were
    /// no bytes to predict about.
    #[must_use]
    pub const fn predicted(
        &self,
    ) -> &BTreeMap<LiveTransferRepresentationPlan, PredictedTransferResources> {
        &self.predicted
    }

    /// Every form the run could not submit, and why.
    ///
    /// Read beside [`Self::observations`] rather than folded into it: a
    /// form with a gap here has no observation there, and the two
    /// together are the whole of what the run did.
    #[must_use]
    pub const fn not_submitted(
        &self,
    ) -> &BTreeSet<(LiveTransferRepresentationPlan, LiveFormNotSubmitted)> {
        &self.not_submitted
    }

    /// Why one form was never submitted, where it was not.
    #[must_use]
    pub fn gap_for(&self, plan: LiveTransferRepresentationPlan) -> Option<LiveFormNotSubmitted> {
        self.not_submitted
            .iter()
            .find_map(|(named, gap)| (*named == plan).then_some(*gap))
    }

    /// Why the plan stopped, where it stopped early.
    #[must_use]
    pub const fn refusal(&self) -> Option<LiveNativeRefusal> {
        self.refusal
    }

    /// One step's observation.
    #[must_use]
    pub fn observation(&self, step: LiveNativeStep) -> Option<&LiveNativeObservation> {
        self.observations
            .iter()
            .find(|observation| observation.step == step)
    }
}

/// Why one representation form was never submitted.
///
/// A gap, not a verdict. §1.11 keeps infrastructure failure and target
/// rejection distinct, and a form nothing could build a transaction for
/// has earned no verdict at all — recording one would be the failure
/// layer being inferred from the expected result.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveFormNotSubmitted {
    /// No confidential predecessor can be funded by THIS pipeline's own
    /// funding step.
    ///
    /// §6.3 admits a private transfer only over confidential receipt
    /// inputs, and the finalization refuses an input whose value field is
    /// explicit. The target-generic funding step names an
    /// `amount_per_output` and has no confidential form, so a run of this
    /// pipeline creates no confidential receipt for a private transfer to
    /// consume.
    ///
    /// That sentence is still true and is now scoped rather than
    /// absolute. The confidential funding arm exists, and a predecessor
    /// of the exact hybrid form has been submitted, accepted, mined, and
    /// read back raw through it. This pipeline reaches none of that: its
    /// funding step is the target-generic one, and its private
    /// materialization does not take the transaction-wide path that
    /// consumes an opening. The gap is therefore real and its name is
    /// wider than its ground.
    ///
    /// The consequence is exact and worth stating plainly: the two
    /// questions Wave 9 left for a target — whether the value field the
    /// private construction builds is a form the target reads back, and
    /// what the target does with a confidential output carrying no range
    /// proof — cannot be asked through this boundary at all. They are not
    /// answered here, and they are not recorded as though they were.
    NoConfidentialPredecessorCanBeFunded,
}

/// Why the plan could not state its next step.
///
/// The reason stays here rather than crossing into the executor package,
/// which holds no vocabulary for it (`PlanRefused` is a marker).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveNativeRefusal {
    /// The candidate substrate could not be built.
    SubstrateUnavailable,
    /// A funding step created no spendable output.
    ///
    /// Then there is no predecessor to spend and the transfer cannot be
    /// finalized: §14.3's materialization did not happen, and a
    /// submission built anyway would be a transaction about nothing.
    FundingCreatedNoPredecessor,
    /// A funding step reported an outpoint this crate cannot read.
    MalformedFundedOutpoint,
    /// The candidate transfer could not be finalized.
    TransferNotConstructible,
    /// The issuance step named no asset to link against.
    ///
    /// Then the deployment cannot be welded to the chain the run is on,
    /// and every later step would be about an asset the target does not
    /// have.
    IssuanceNamedNoAsset,
    /// The deployment could not be linked against the issued asset.
    RelinkRefused,
}

/// One wire outpoint, as this crate's own type.
fn outpoint_of(wire: &WireOutpoint) -> Option<Outpoint> {
    let raw = decode_hex(&wire.txid)?;
    let mut internal = <[u8; 32]>::try_from(raw.as_slice()).ok()?;
    // A target prints a transaction identity in the reverse of the order
    // it hashes it in, and this crate's `Txid` holds the hashed order.
    internal.reverse();
    Outpoint::new(Txid::from_internal(internal), wire.vout).ok()
}

/// One asset identity, from the spelling the target printed.
fn asset_of(text: &str) -> Option<AssetId> {
    let mut raw = decode_hex(text)?;
    raw.reverse();
    <[u8; 32]>::try_from(raw.as_slice())
        .ok()
        .map(AssetId::from_internal)
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
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

/// What each funded receipt holds.
const RECEIPT_AMOUNT: u64 = 5_000;

pub struct LiveTransferOperationPlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    transcript: LiveNativeTranscript,
}

impl LiveTransferOperationPlanner {
    /// The planner over the demonstration candidate.
    ///
    /// The ABI it starts with is the constant-asset one, and it is used
    /// for exactly one thing: a program to pay the issuance's own outputs
    /// to. The deployment the run actually submits against is linked once
    /// the target has chosen an asset.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI
    /// or either destination constructor is unavailable.
    pub fn new() -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        let transcript = LiveNativeTranscript {
            explicit_program: destination_program(&abi, LiveTransferRepresentationPlan::Explicit)?,
            private_program: destination_program(
                &abi,
                LiveTransferRepresentationPlan::PrivateCommitted,
            )?,
            ..LiveNativeTranscript::default()
        };
        Ok(Self {
            stage: Stage::Issue,
            abi,
            transcript,
        })
    }

    /// The plan's own record of the run.
    #[must_use]
    pub const fn transcript(&self) -> &LiveNativeTranscript {
        &self.transcript
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: LiveNativeRefusal) -> PlanRefused {
        self.transcript.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// Record what the node did with one step.
    fn observe(&mut self, step: LiveNativeStep, response: &NativeOperationResponse) {
        self.transcript.observations.push(LiveNativeObservation {
            step,
            layer: response.observed_layer,
            detail: response.observed_detail.clone(),
            accepted_txid: response.accepted_txid.clone(),
            funded: response.funded_outputs.len(),
            observed_weight: response.resources.transaction_weight,
        });
    }

    /// Link the deployment against the asset the target issued.
    ///
    /// The step that makes the rest of the run about a real chain. The
    /// constructors push the asset as a literal, so linking again moves
    /// every committed tree and therefore every destination program — the
    /// programs the ceremony funds next are the ones *this* deployment
    /// determines, not the constant-asset one's.
    fn relink(&mut self, response: &NativeOperationResponse) -> Result<(), LiveNativeRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(LiveNativeRefusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(LiveNativeRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(*identity.internal())
            .map_err(|_| LiveNativeRefusal::RelinkRefused)?;
        self.transcript.explicit_program =
            destination_program(&abi, LiveTransferRepresentationPlan::Explicit)
                .map_err(|_| LiveNativeRefusal::RelinkRefused)?;
        self.transcript.private_program =
            destination_program(&abi, LiveTransferRepresentationPlan::PrivateCommitted)
                .map_err(|_| LiveNativeRefusal::RelinkRefused)?;
        self.transcript.issued_asset = Some(asset);
        self.transcript.relinked = true;
        self.abi = abi;
        Ok(())
    }

    /// One funding step's created coins.
    fn settle_funding(
        &mut self,
        step: LiveNativeStep,
        response: &NativeOperationResponse,
    ) -> Result<Vec<WireOutpoint>, LiveNativeRefusal> {
        self.observe(step, response);
        if response.funded_outputs.is_empty() {
            return Err(LiveNativeRefusal::FundingCreatedNoPredecessor);
        }
        for funded in &response.funded_outputs {
            if outpoint_of(&funded.outpoint).is_none() {
                return Err(LiveNativeRefusal::MalformedFundedOutpoint);
            }
        }
        Ok(response
            .funded_outputs
            .iter()
            .map(|funded| funded.outpoint.clone())
            .collect())
    }

    /// The funding step for one plan's constructor program.
    fn funding_step(&self, step: LiveNativeStep, issue: bool) -> OperationStep {
        let program = match step {
            LiveNativeStep::FundPrivateConstructor => self.transcript.private_program.clone(),
            _ => self.transcript.explicit_program.clone(),
        };
        OperationStep::new(
            step.name(),
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.transcript.issued_asset.clone()
                },
                output_program: program,
                outputs: 2,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        )
    }

    /// The candidate transfer of one form, as target bytes.
    fn transfer_bytes(
        &self,
        plan: LiveTransferRepresentationPlan,
    ) -> Result<Vec<u8>, LiveNativeRefusal> {
        let coins = match plan {
            LiveTransferRepresentationPlan::Explicit => &self.transcript.explicit_coins,
            LiveTransferRepresentationPlan::PrivateCommitted => &self.transcript.private_coins,
        };
        let asset = self
            .transcript
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(LiveNativeRefusal::IssuanceNamedNoAsset)?;
        let program = match plan {
            LiveTransferRepresentationPlan::Explicit => &self.transcript.explicit_program,
            LiveTransferRepresentationPlan::PrivateCommitted => &self.transcript.private_program,
        };

        let mut points = Vec::with_capacity(coins.len());
        let mut views = Vec::with_capacity(coins.len());
        for wire in coins {
            let point = outpoint_of(wire).ok_or(LiveNativeRefusal::MalformedFundedOutpoint)?;
            points.push(point);
            views.push(PublicOutputView::new(
                point,
                AssetField::Explicit(asset),
                ValueField::Explicit(RECEIPT_AMOUNT),
                program.clone(),
            ));
        }
        let view = PublicConstructionView::new(views)
            .map_err(|_| LiveNativeRefusal::TransferNotConstructible)?;

        let total = RECEIPT_AMOUNT
            .checked_mul(u64::try_from(coins.len()).unwrap_or(0))
            .ok_or(LiveNativeRefusal::TransferNotConstructible)?;
        let destination = |scalar: &[u8; 32], amount: u64| {
            let owner =
                published_owner(scalar).map_err(|_| LiveNativeRefusal::SubstrateUnavailable)?;
            let value = ProtocolValue::new(amount)
                .map_err(|_| LiveNativeRefusal::TransferNotConstructible)?;
            Ok::<_, LiveNativeRefusal>(LiveReceiptDestination::new(
                linker::OwnerParameter::new(owner),
                value,
            ))
        };
        let randomness = match plan {
            LiveTransferRepresentationPlan::Explicit => None,
            LiveTransferRepresentationPlan::PrivateCommitted => Some(
                PublicTestRandomness::from_published_bytes(PUBLISHED_RANDOMNESS),
            ),
        };
        let request = LiveTransferRequest::new(
            points,
            [
                destination(&SECOND_SCALAR, total / 2)?,
                destination(&FIRST_SCALAR, total - total / 2)?,
            ],
            plan,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            randomness,
        )
        .map_err(|_| LiveNativeRefusal::TransferNotConstructible)?;

        let target = reviewed_target().map_err(|_| LiveNativeRefusal::SubstrateUnavailable)?;
        let private: Option<&dyn transaction::live_private::PrivateValueCapability> = match plan {
            LiveTransferRepresentationPlan::Explicit => None,
            LiveTransferRepresentationPlan::PrivateCommitted => Some(&OracleFixtureValues),
        };
        let finalization =
            finalize_live_transfer(&target, &self.abi, &request, &view, None, private)
                .map_err(|_| LiveNativeRefusal::TransferNotConstructible)?;
        let report = finalization.report().clone();
        let finalized = finalization.into_finalized();

        // Every owner "signs" with bytes that authorize nothing. The
        // builder checks binding and nothing else, which is exactly what
        // makes this expressible — and exactly why no verdict it earns
        // discharges anything.
        let responses: Vec<_> = finalized
            .signing_requests()
            .iter()
            .map(|signing| {
                (
                    signing.input(),
                    LiveOwnerResponse::to(signing, UNAUTHORIZING_SIGNATURE.to_vec()),
                )
            })
            .collect();
        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| LiveNativeRefusal::TransferNotConstructible)?;
        let built =
            transaction::live_construct::complete_live_transfer(&target, authorized, report, None)
                .map_err(|_| LiveNativeRefusal::TransferNotConstructible)?;
        Ok(built.bytes())
    }

    /// One submission step.
    fn submit_step(
        &mut self,
        step: LiveNativeStep,
        plan: LiveTransferRepresentationPlan,
    ) -> Result<OperationStep, LiveNativeRefusal> {
        let bytes = self.transfer_bytes(plan)?;

        // §18.4's first-party half, recorded here rather than recomputed
        // later, because "the same exact bytes" is only checkable if the
        // prediction is taken from the serialization that is about to be
        // submitted. A figure computed afterwards from a rebuilt
        // transaction would be a prediction about some other bytes that
        // happened to have the same shape.
        self.transcript.predicted.insert(
            plan,
            PredictedTransferResources {
                serialized_bytes: bytes.len() as u64,
                weight: weight_of(&bytes),
            },
        );

        Ok(OperationStep::new(
            step.name(),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: bytes,
            })),
        ))
    }
}

/// One representation plan's destination program for the first published
/// owner.
fn destination_program(
    abi: &CandidateLiveTransferAbi,
    plan: LiveTransferRepresentationPlan,
) -> Result<Vec<u8>, VectorError> {
    Ok(abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(published_owner(&FIRST_SCALAR)?),
            plan,
        )
        .ok_or(VectorError::LiveSubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

impl TargetOperationPlanner for LiveTransferOperationPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    self.observe(LiveNativeStep::IssueProtocolAsset, response);
                    if let Err(refusal) = self.relink(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundExplicit;
                }
                Stage::FundExplicit => {
                    match self.settle_funding(LiveNativeStep::FundExplicitConstructor, response) {
                        Ok(coins) => self.transcript.explicit_coins = coins,
                        Err(refusal) => return Err(self.refuse(refusal)),
                    }
                    self.stage = Stage::FundPrivate;
                }
                Stage::FundPrivate => {
                    match self.settle_funding(LiveNativeStep::FundPrivateConstructor, response) {
                        Ok(coins) => self.transcript.private_coins = coins,
                        Err(refusal) => return Err(self.refuse(refusal)),
                    }
                    self.stage = Stage::SubmitExplicit;
                }
                Stage::SubmitExplicit => {
                    self.observe(LiveNativeStep::SubmitExplicitTransfer, response);
                    self.stage = Stage::SubmitPrivate;
                }
                Stage::SubmitPrivate => {
                    self.observe(LiveNativeStep::SubmitPrivateTransfer, response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        let step = match self.stage {
            Stage::Issue => self.funding_step(LiveNativeStep::IssueProtocolAsset, true),
            Stage::FundExplicit => {
                self.funding_step(LiveNativeStep::FundExplicitConstructor, false)
            }
            Stage::FundPrivate => self.funding_step(LiveNativeStep::FundPrivateConstructor, false),
            Stage::SubmitExplicit => {
                match self.submit_step(
                    LiveNativeStep::SubmitExplicitTransfer,
                    LiveTransferRepresentationPlan::Explicit,
                ) {
                    Ok(step) => step,
                    Err(refusal) => return Err(self.refuse(refusal)),
                }
            }
            Stage::SubmitPrivate => {
                // Declared rather than attempted. The planner knows what
                // it funded — explicit outputs, because THIS pipeline's
                // funding step has no other form — so it knows before
                // trying that no confidential receipt exists in this run
                // to consume, and says so. Trying anyway and catching the
                // builder's refusal would record a construction failure
                // where the honest record is a materialization gap.
                //
                // A confidential predecessor is fundable elsewhere in
                // this package and one has been mined. Nothing here
                // consumes it, so nothing here changes.
                self.transcript.not_submitted.insert((
                    LiveTransferRepresentationPlan::PrivateCommitted,
                    LiveFormNotSubmitted::NoConfidentialPredecessorCanBeFunded,
                ));
                self.stage = Stage::Done;
                return Ok(None);
            }
            Stage::Done => return Ok(None),
        };
        Ok(Some(step))
    }
}

/// The run of record: what one real node did, on one host, once.
///
/// # This is a transcription, and it is not evidence
///
/// Nothing here discharges a §15 row, and nothing here is recomputed from
/// anything. It is a *record*, committed so that a reader without a node
/// can see what the target actually said — and so that the blocker
/// [`crate::live_evidence::LiveInfrastructureBlocker::OwnerSighashNotComputable`]
/// is readable as a thing that was observed rather than a thing that was
/// argued for.
///
/// The observed verdict is the sharpest single fact this wave produced.
/// The explicit transfer was refused at
/// [`ObservedOutcomeLayer::ScriptPathRejection`] with the node's own
/// words: `mandatory-script-verify-flag-failed (Invalid Schnorr
/// signature)`. That means the covenant *ran*. Over a real transaction
/// spending real coins at the linked constructors' own programs, the
/// coordinator's asset check, constructor recognition, class closure,
/// count and conservation arithmetic all passed, and evaluation reached
/// the owner's signature check before anything failed. What stands
/// between this candidate and an accepted transfer is one digest.
///
/// # What a reader may not do with it
///
/// Read it as a verdict about a *relation*. §19.2 requires a negative
/// case to have been refused for its own intended relation, and this
/// refusal is about the signature offered — which is opaque bytes by
/// construction, because no first-party component computes the message
/// they would have to be over.
#[must_use]
pub fn observed_run_of_record() -> LiveNativeTranscript {
    LiveNativeTranscript {
        issued_asset: Some(
            "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb".to_owned(),
        ),
        relinked: true,
        // The outpoints the run funded are deliberately absent. They name
        // a chain that was destroyed when the run ended, and a record
        // carrying them would look like something a later run could
        // resume from.
        explicit_program: Vec::new(),
        private_program: Vec::new(),
        explicit_coins: Vec::new(),
        private_coins: Vec::new(),
        not_submitted: BTreeSet::from([(
            LiveTransferRepresentationPlan::PrivateCommitted,
            LiveFormNotSubmitted::NoConfidentialPredecessorCanBeFunded,
        )]),
        // §18.4's first-party half, from the run that produced the
        // observation below. Both figures are the same run's: a
        // prediction transcribed from some other run would be a
        // prediction about other bytes, which is the substitution §18.4's
        // "same exact bytes" is there to refuse.
        predicted: BTreeMap::from([(
            LiveTransferRepresentationPlan::Explicit,
            PredictedTransferResources {
                serialized_bytes: RECORDED_EXPLICIT_SERIALIZED_BYTES,
                weight: Some(RECORDED_EXPLICIT_PREDICTED_WEIGHT),
            },
        )]),
        observations: vec![
            recorded(
                LiveNativeStep::IssueProtocolAsset,
                ObservedOutcomeLayer::Accepted,
                None,
                2,
                None,
            ),
            recorded(
                LiveNativeStep::FundExplicitConstructor,
                ObservedOutcomeLayer::Accepted,
                None,
                2,
                None,
            ),
            recorded(
                LiveNativeStep::FundPrivateConstructor,
                ObservedOutcomeLayer::Accepted,
                None,
                2,
                None,
            ),
            recorded(
                LiveNativeStep::SubmitExplicitTransfer,
                ObservedOutcomeLayer::ScriptPathRejection,
                Some("mandatory-script-verify-flag-failed (Invalid Schnorr signature)"),
                0,
                Some(RECORDED_EXPLICIT_OBSERVED_WEIGHT),
            ),
        ],
        refusal: None,
    }
}

/// One observation with a different weight, for staging a disagreement.
///
/// Test-only, and it exists so that §18.4's comparison can be shown to
/// have teeth. An agreement between a prediction and an observation is
/// evidence only if the two could have differed; the comparison's own
/// tests therefore build a run whose observed figure has moved and check
/// that a typed mismatch comes back. Nothing outside a test can reach
/// this, so no report can be assembled over a figure a caller chose.
#[cfg(test)]
pub(crate) fn rewitnessed(
    observation: &LiveNativeObservation,
    observed_weight: Option<u64>,
) -> LiveNativeObservation {
    LiveNativeObservation {
        observed_weight,
        ..observation.clone()
    }
}

/// One transcript carrying a different observation list.
///
/// Test-only, for the reason [`rewitnessed`] states.
#[cfg(test)]
pub(crate) fn with_observations(
    transcript: LiveNativeTranscript,
    observations: Vec<LiveNativeObservation>,
) -> LiveNativeTranscript {
    LiveNativeTranscript {
        observations,
        ..transcript
    }
}

/// The weight this workspace computes for one submitted serialization.
///
/// By decoding the bytes and weighing the result, so the figure is a
/// function of the exact serialization the node was handed rather than of
/// the value that produced it. A serialization this workspace cannot
/// decode yields no weight — an absence, and a real finding about the
/// encoder, rather than a weight of zero.
fn weight_of(bytes: &[u8]) -> Option<u64> {
    transaction::bytes::TargetTransaction::decode(bytes)
        .ok()
        .map(|transaction| transaction.weight())
}

/// One recorded observation, spelled once.
fn recorded(
    step: LiveNativeStep,
    layer: ObservedOutcomeLayer,
    detail: Option<&str>,
    funded: usize,
    observed_weight: Option<u64>,
) -> LiveNativeObservation {
    LiveNativeObservation {
        step,
        layer,
        detail: detail.map(ToOwned::to_owned),
        accepted_txid: None,
        funded,
        observed_weight,
    }
}

/// Render one run's record, canonically.
///
/// The layers and the node's own words. No expectation appears, because
/// this run is entitled to none.
///
/// # Panics
///
/// Never: the sink is a `String`, whose writes cannot fail.
#[must_use]
pub fn render_live_native_run(transcript: &LiveNativeTranscript) -> String {
    use std::fmt::Write as _;

    let mut text = String::new();
    let _ = writeln!(text, "role live-transfer-native-run");
    let _ = writeln!(text, "discharges_no_matrix_row true");
    let _ = writeln!(
        text,
        "issued_asset {}",
        transcript.issued_asset().unwrap_or("none")
    );
    let _ = writeln!(text, "relinked {}", transcript.relinked());
    for (plan, count) in transcript.materialized() {
        let _ = writeln!(text, "materialized {plan:?} {count}");
    }
    for (plan, gap) in transcript.not_submitted() {
        let _ = writeln!(text, "not_submitted {plan:?} {gap:?}");
    }
    // §18.4's two halves, side by side and neither derived from the
    // other. The prediction is this workspace's own weight for the bytes
    // it submitted; the observation is the node's, read back from its own
    // decoder. A step that reached no verdict prints `none` rather than a
    // zero, because a zero here would read as a figure the node gave.
    for (plan, predicted) in transcript.predicted() {
        let _ = writeln!(
            text,
            "predicted {plan:?} serialized_bytes={} weight={}",
            predicted.serialized_bytes(),
            predicted
                .weight()
                .map_or_else(|| "none".to_owned(), |w| w.to_string()),
        );
    }
    for observation in transcript.observations() {
        let _ = writeln!(
            text,
            "observed {} {:?} txid={} weight={} detail={}",
            observation.step().name(),
            observation.layer(),
            observation.accepted_txid().unwrap_or("none"),
            observation
                .observed_weight()
                .map_or_else(|| "none".to_owned(), |weight| weight.to_string()),
            observation.detail().unwrap_or("none"),
        );
    }
    if let Some(refusal) = transcript.refusal() {
        let _ = writeln!(text, "refused {refusal:?}");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::{LiveNativeStep, LiveTransferOperationPlanner, render_live_native_run};
    use linker::live_backend::LiveTransferRepresentationPlan;

    #[test]
    fn the_planner_names_the_two_constructor_programs_it_will_fund() {
        // Buildable without a node: the ceremony's whole point is that it
        // pays to the programs the deployment's constructors determine,
        // and those are known before anything runs.
        let planner = LiveTransferOperationPlanner::new().expect("the planner builds");
        let transcript = planner.transcript();
        assert_ne!(transcript.explicit_program().len(), 0);
        assert_ne!(transcript.private_program().len(), 0);
        assert_ne!(
            transcript.explicit_program(),
            transcript.private_program(),
            "the two representation plans share a destination program",
        );
        assert_eq!(
            transcript.materialized()[&LiveTransferRepresentationPlan::Explicit],
            0,
            "the planner claims coins before any step ran",
        );
    }

    #[test]
    fn the_run_of_record_says_the_covenant_reached_the_signature_check() {
        // The wave's sharpest observation, committed so a reader without
        // a node can see it, and checked so that editing the record has
        // to be deliberate. It establishes nothing — the rendering it
        // produces says so in its own first lines — and what it is for is
        // that the blocker the evidence plan carries was *observed*.
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let record = super::observed_run_of_record();
        assert!(record.relinked());
        assert!(record.issued_asset().is_some());
        assert!(record.refusal().is_none());

        let explicit = record
            .observation(LiveNativeStep::SubmitExplicitTransfer)
            .expect("the explicit transfer was submitted");
        assert_eq!(explicit.layer(), ObservedOutcomeLayer::ScriptPathRejection);
        assert_eq!(
            explicit.detail(),
            Some("mandatory-script-verify-flag-failed (Invalid Schnorr signature)"),
        );

        // Every funding step was accepted, which is §14.3's
        // materialization: the linked constructors' own programs really
        // are outputs a target will create and hold coins at.
        for step in [
            LiveNativeStep::IssueProtocolAsset,
            LiveNativeStep::FundExplicitConstructor,
            LiveNativeStep::FundPrivateConstructor,
        ] {
            let observation = record.observation(step).expect("the step ran");
            assert_eq!(observation.layer(), ObservedOutcomeLayer::Accepted);
            assert_ne!(observation.funded(), 0);
        }

        // And the private half was never submitted, for a stated reason
        // rather than a silence.
        assert!(
            record
                .observation(LiveNativeStep::SubmitPrivateTransfer)
                .is_none()
        );
        assert_eq!(
            record.gap_for(LiveTransferRepresentationPlan::PrivateCommitted),
            Some(super::LiveFormNotSubmitted::NoConfidentialPredecessorCanBeFunded),
        );

        // The rendering still refuses to read as coverage.
        let rendered = render_live_native_run(&record);
        assert!(rendered.contains("discharges_no_matrix_row true"));
        assert!(rendered.contains("ScriptPathRejection"));
    }

    #[test]
    fn a_run_that_never_happened_renders_as_one() {
        // The rendering of an empty transcript says the run discharged
        // nothing and names no observation, rather than reading as a run
        // whose every step passed.
        let planner = LiveTransferOperationPlanner::new().expect("the planner builds");
        let rendered = render_live_native_run(planner.transcript());
        assert!(rendered.contains("discharges_no_matrix_row true"));
        assert!(rendered.contains("issued_asset none"));
        assert!(!rendered.contains("observed "));
        for step in LiveNativeStep::ALL {
            assert!(planner.transcript().observation(*step).is_none());
        }
    }
}
