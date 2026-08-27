//! The restart order's first step: one sponsorless private one-to-one
//! control, against a real node.
//!
//! # What this ceremony is, and what it is the entry condition for
//!
//! Everything in the mandatory restart order depends on this
//! (task:guide-ctf-exec:restart-order). Step one is one accepted
//! sponsorless private control, and every later step — both parities,
//! CT conservation, the three proof-negatives, the remaining shapes,
//! the minimality pairs — has it as its entry condition. A negative case
//! run before it is not evidence about its own mutation, which is what
//! [`crate::live_evidence::LiveInfrastructureBlocker::NoAcceptingControlExists`]
//! already records.
//!
//! So this module builds exactly one candidate and submits exactly one
//! candidate. It runs no controls and no mutations, because the order
//! forbids them until this one accepts.
//!
//! # Why it is a different ceremony from the proof-bearing one
//!
//! [`crate::live_proof_bearing_observation`] already submits a
//! proof-bearing private candidate and carries an observed acceptance.
//! It is not this step, and the difference is one thing: its leaf is a
//! bare owner check, because when it was written the private branch of
//! the finalization API did not take the transaction-wide path and
//! there was no covenant-bearing private candidate to spend. It says so
//! in its own header.
//!
//! That gap closed. `transaction::live_construct::finalize_private_live_transfer`
//! is the private lane's own entry point, and it produces a candidate
//! whose inputs are LIVE RECEIPTS — each one carrying the leaf its
//! position executes and the control block that authenticates it —
//! rather than a candidate whose inputs happen to be spendable.
//!
//! The distinction is the whole of why the earlier acceptance did not
//! move a matrix row and why this one could. The §15.2 rows are about
//! private transfers of this deployment's receipts. An acceptance of
//! some other program's spend is an acceptance of some other program's
//! spend.
//!
//! # The one-to-one reading, stated because it is a reading
//!
//! "One-to-one" is taken here as ONE receipt consumed and ONE recipient
//! created, with the balancing output that absorbs the blinder residue
//! going back to the sender as change.
//!
//! It has to be read, because the strict reading — one confidential
//! input and one confidential output, full stop — is not constructible
//! under this guide's own rules, and that is checkable rather than a
//! matter of taste. The fixture registry refuses a manifest with fewer
//! than two outputs, on the ground that a balance needs at least two and
//! that exactly one of them must be balancing; and the materializer
//! refuses an intent whose destination count differs from its fixture's
//! output count. Together those two make a one-output private
//! transaction unsayable.
//!
//! The reading is recorded here rather than resolved silently, and the
//! conflict is reported as a divergence rather than repaired, because
//! this module does not own either rule.
//!
//! # The native carrier binds this record exactly
//!
//! The ignored private-restart tests write their fresh artifacts first and
//! then assert every stable field against [`run_of_record`]: exact accepted
//! layer and identity, fixture digests, receipt parity, byte and proof
//! counts, and unconditional readback reverification. A changed honest
//! result therefore remains recorded while the reproduction gate fails.
//! Superseding it requires an explicit decision recorded as a new
//! forward record; the historical constants are never overwritten to
//! make a rerun green.

use std::collections::BTreeMap;

use linker::OwnerParameter;
use linker::live_backend::{LiveTransferComposition, LiveTransferRepresentationPlan};
use target_elements::LeafVersion;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::owner_key_oracle::verify_owner_signature;
use target_elements_conformance::protocol::{
    ConfidentialFundedOutput, ConfidentialFundingBinding, ConfidentialFundingDestination,
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    OperationSubject, TargetConfidentialFundingSubject, TargetFundingSubject,
    TargetSubmissionSubject,
};
use transaction::bytes::{
    AssetField, AssetId, COMMITMENT_BYTES, InputWitness, Outpoint, TargetTransaction, ValueField,
};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{OwnerCensusRefusal, OwnerSigningCensus, OwnerSigningInputRequest};
use transaction::live_construct::{
    PrivateDestinationOpening, PrivateInputOpening, PrivateLiveFinalization, PrivateLiveOpenings,
    finalize_private_live_transfer, private_sponsor_witnesses,
};
use transaction::live_materialize::{
    ConfidentialInputRegion, ConfidentialOutputRole, FixtureOpeningReference,
    FrozenConfidentialFixtureView, IndependentCommitmentCheck as _, NonProtocolFundingRegion,
    SCALAR_BYTES,
};
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::sponsor::SponsorCapability;
use transaction::taproot::{Digest32, leaf_hash};
use transaction::view::{PublicConstructionView, PublicOutputView};
use transaction::{AnnexDisposition, IssuanceDisposition, LiveDeployment};

use crate::confidential_materializer::{
    FirstPartyCommitmentCheck, ReferenceConfidentialMaterializer,
};
use crate::confidential_predecessor::{FUND_STEP, ISSUE_STEP, PredecessorShape, selected_profiles};
use crate::error::VectorError;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed_order};
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, LiveShapeVocabulary, RESERVE_ASSET, SECOND_SCALAR,
    live_abi_composing, published_owner, reviewed_target, signing_material,
};
use crate::live_proof_bearing_observation::{materialization_profiles, register, register_multi};
use target_elements_conformance::confidential_fixture::ConfidentialFixtureOutput;

/// The caller's own name for the one submission this ceremony makes.
pub const CONTROL_STEP: &str = "submit-private-one-to-one-control";

/// The successor fixture's handle.
///
/// Its own, and not the proof-bearing ceremony's: two ceremonies sharing
/// a handle would make a digest drift between them undetectable.
const SUCCESSOR_HANDLE: &str = "ctf-v1/restart-wave-five-one-to-one-successor";

/// What the issuing step creates so a reserve exists at all.
const ISSUE_OUTPUTS: u8 = 1;

/// What the issuing step pays into each of those outputs.
const ISSUE_AMOUNT_PER_OUTPUT: u64 = 1;

/// The program the issuing step pays into.
const ISSUE_PROGRAM: [u8; 1] = [0x51];

/// Which of the predecessor's two outputs one control consumes.
///
/// # Why this is a parameter and not a constant
///
/// Step two of the restart order asks for both predecessor commitment
/// parities exercised in complete accepted successors, and the two
/// parities are carried by the two predecessor outputs. So the ceremony
/// runs once per output rather than once, and the parity a run
/// exercised is read off the coin the node reported rather than
/// inferred from which output was chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsumedReceipt {
    /// The predecessor's primary output.
    Primary,
    /// The predecessor's balancing output.
    Balancing,
}

impl ConsumedReceipt {
    /// Both, in the order the restart runs them.
    pub const ALL: [Self; 2] = [Self::Primary, Self::Balancing];

    /// Which predecessor output this is.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Primary => 0,
            Self::Balancing => 1,
        }
    }

    /// The ceremony's own name for the run.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Balancing => "balancing",
        }
    }

    /// The recipient's share and the sender's change, in that order.
    ///
    /// The two sum to the consumed receipt's own semantic amount, which
    /// is what conservation means on this lane. It is checked by the
    /// target rather than asserted here, and a split that did not add up
    /// would be refused by the target's commitment balance rather than
    /// by this module.
    #[must_use]
    pub const fn split(self) -> [u64; 2] {
        match self {
            Self::Primary => [500_000_000, 200_000_000],
            Self::Balancing => [200_000_000, 100_000_000],
        }
    }
}

/// The auxiliary value every signature here is taken with.
///
/// Published rather than random, on the two existing ceremonies' own
/// ground: fixing it is what makes every signature reproducible from
/// inputs that are all written down.
const RESTART_AUXILIARY: [u8; SCALAR_BYTES] = [0x44; SCALAR_BYTES];

/// What this ceremony refuses, before any node is asked.
///
/// Every member is a construction or infrastructure fact. None is a
/// target verdict, and none may be read as one
/// (rule:guide-ctf-exec:failure-layers).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrivateRestartRefusal {
    /// The reviewed target or a published owner could not be built.
    SubstrateUnavailable,
    /// The issuance step named no asset to link against.
    IssuanceNamedNoAsset,
    /// The deployment could not be linked against the issued asset.
    RelinkRefused,
    /// The deployment could not be linked, and the link said why.
    ///
    /// Carries the substrate's own refusal rather than discarding it.
    /// A crossing deployment fails at more layers than a homogeneous
    /// one -- a constructor, an emission, a link and an ABI derivation
    /// -- and a ceremony that reported only that it failed would leave
    /// the reader to guess which.
    RelinkRefusedBy(String),
    /// The linked deployment's destination table holds no constructor
    /// at the plan a program was asked for.
    ///
    /// Separated from [`Self::RelinkRefused`] because the two are
    /// different failures a wave has to tell apart: one is a deployment
    /// that would not link at all, and this is a deployment that linked
    /// and then did not carry the key somebody asked it for. A crossing
    /// ceremony asks for two different keys, so a single refusal
    /// covering both would say which deployment failed and never which
    /// side.
    NoConstructorForPlan {
        /// Which plan the table was asked for.
        plan: LiveTransferRepresentationPlan,
    },
    /// A fixture the ceremony registers is not one the registry admits.
    FixtureNotRegistrable {
        /// Which handle.
        handle: String,
    },
    /// The predecessor's blinders do not close.
    PredecessorBlindersDoNotClose,
    /// The confidential funding step created no predecessor.
    FundingCreatedNoPredecessor,
    /// A confidential funded output could not be read.
    MalformedConfidentialOutput,
    /// The transfer request is not one this crate can state.
    ControlNotRequestable,
    /// The private finalization refused the control.
    ///
    /// Carries the transaction crate's own word, wrapped rather than
    /// re-spelled.
    FinalizationRefused(String),
    /// The owner census refused the finalized candidate.
    CensusRefused(OwnerCensusRefusal),
    /// An owner could not sign.
    SigningRefused,
    /// The sponsor's authorization was refused.
    ///
    /// Carries the transaction crate's own word, wrapped rather than
    /// re-spelled, on the pattern [`Self::FinalizationRefused`] already
    /// sets. The distinctions that matter -- no capability offered, a
    /// sponsor that declined, an answer bound to other bytes -- are
    /// drawn there, where the request was minted, and restating them
    /// here would be a second vocabulary that could disagree with the
    /// first.
    SponsorWitnessRefused(String),
    /// An input position nothing authorized.
    ///
    /// Neither an owner's witness nor the sponsor's landed at this
    /// position. It is refused rather than filled with an empty witness,
    /// because an unauthorized input is a candidate a node refuses at
    /// script with a verdict about the program rather than about the gap
    /// in this ceremony.
    InputPositionUnauthorized {
        /// Which input position.
        position: u16,
    },
    /// The candidate could not be assembled for the wire.
    CandidateNotSerializable,
}

/// One funded confidential coin, as the node reported it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestartConfidentialCoin {
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
    rangeproof_bytes: usize,
    matches_expectation: bool,
}

impl RestartConfidentialCoin {
    /// One coin the node created whose VALUE is EXPLICIT.
    ///
    /// The predecessor of an ENTRY crossing. It carries no commitment
    /// and no range proof, so the two facts a confidential coin is
    /// censused on are stated as what they are rather than as zeroes
    /// standing in for something: the proof byte count is genuinely
    /// zero, and `matches_expectation` is true because the expectation
    /// for an explicit coin IS the amount, which the caller compares
    /// before building this.
    #[must_use]
    pub const fn explicit(
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
            rangeproof_bytes: 0,
            matches_expectation,
        }
    }

    /// The outpoint the node created.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The asset field the node reported.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The value field the node reported.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The program the node reported.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }

    /// How many range-proof bytes the node reported for it.
    #[must_use]
    pub const fn rangeproof_bytes(&self) -> usize {
        self.rangeproof_bytes
    }

    /// Whether the node's report is what the ceremony asked for.
    #[must_use]
    pub const fn matches_expectation(&self) -> bool {
        self.matches_expectation
    }
}

/// What the second origin checked, where an acceptance was observed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestartReverification {
    accepted_txid: String,
    readback_matches_submission: bool,
    verified: bool,
}

impl RestartReverification {
    /// The identity the target computed.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether the accepted witness verifies against the independently
    /// recomputed message.
    #[must_use]
    pub const fn verified(&self) -> bool {
        self.verified
    }
}

/// The transcript one run of this ceremony produces.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrivateRestartRecord {
    issued_asset: Option<String>,
    predecessor_digest: Option<[u8; 32]>,
    successor_digest: Option<[u8; 32]>,
    coins: Vec<RestartConfidentialCoin>,
    consumed_receipt: Option<&'static str>,
    consumed_commitment_prefix: Option<u8>,
    receipt_leaves: usize,
    output_witness_proof_bytes: Vec<usize>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    reverification: Option<RestartReverification>,
    refusal: Option<PrivateRestartRefusal>,
}

impl PrivateRestartRecord {
    /// The disposable asset the run issued.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// The predecessor fixture's digest.
    #[must_use]
    pub const fn predecessor_digest(&self) -> Option<[u8; 32]> {
        self.predecessor_digest
    }

    /// The successor fixture's digest.
    #[must_use]
    pub const fn successor_digest(&self) -> Option<[u8; 32]> {
        self.successor_digest
    }

    /// The coins the node reported for the confidential funding step.
    #[must_use]
    pub fn coins(&self) -> &[RestartConfidentialCoin] {
        &self.coins
    }

    /// Which predecessor output this run consumed.
    #[must_use]
    pub const fn consumed_receipt(&self) -> Option<&'static str> {
        self.consumed_receipt
    }

    /// The commitment prefix the consumed coin carried, as the NODE
    /// reported it.
    ///
    /// The parity this run exercised. Read off the observed commitment
    /// rather than inferred from which output was chosen, so a run whose
    /// fixture and whose chain disagreed about parity would report the
    /// chain's answer.
    #[must_use]
    pub const fn consumed_commitment_prefix(&self) -> Option<u8> {
        self.consumed_commitment_prefix
    }

    /// How many receipt inputs the control consumed.
    ///
    /// One, for a one-to-one control, and recorded rather than assumed
    /// so a shape that drifted would be readable in the transcript.
    #[must_use]
    pub const fn receipt_leaves(&self) -> usize {
        self.receipt_leaves
    }

    /// The range-proof byte counts of the candidate's own outputs.
    #[must_use]
    pub fn output_witness_proof_bytes(&self) -> &[usize] {
        &self.output_witness_proof_bytes
    }

    /// How many bytes were handed to the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the node's verdict arrived at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The second origin's answer, where there was an acceptance to
    /// check.
    #[must_use]
    pub const fn reverification(&self) -> Option<&RestartReverification> {
        self.reverification.as_ref()
    }

    /// The construction refusal, where the ceremony stopped before the
    /// node.
    #[must_use]
    pub const fn refusal(&self) -> Option<&PrivateRestartRefusal> {
        self.refusal.as_ref()
    }

    /// Whether this run produced an accepted control.
    ///
    /// The one question the restart order asks of step one, answered
    /// from the observation rather than from the ceremony having
    /// completed.
    #[must_use]
    pub fn produced_an_accepted_control(&self) -> bool {
        self.observed_layer == Some(ObservedOutcomeLayer::Accepted)
            && self
                .reverification
                .as_ref()
                .is_some_and(|check| check.verified() && check.readback_matches_submission())
    }
}

/// What the ceremony is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset the deployment is linked against.
    Issue,
    /// Fund the confidential predecessor at the receipt constructors.
    Fund,
    /// Submit the one control.
    Submit,
    /// Nothing further.
    Done,
}

/// What the ceremony learned once the asset existed.
///
/// `pub(crate)` because the proof-negative ceremony
/// ([`crate::live_conservation_negatives`]) builds its balance-valid
/// control from the same linked deployment this one does, through the
/// shared [`link_and_register`] and [`build_control`] entry points. It
/// carries the two fixture views the successor is materialized against
/// and the two owner programs, so a second ceremony reconstructs the
/// identical control without re-deriving the registration.
#[derive(Clone, Debug)]
pub(crate) struct LinkedDeployment {
    pub(crate) abi: CandidateLiveTransferAbi,
    pub(crate) asset: AssetId,
    pub(crate) predecessor_digest: [u8; 32],
    pub(crate) predecessor_view: transaction::live_materialize::ConfidentialFixtureView,
    successor_digest: [u8; 32],
    successor_view: transaction::live_materialize::ConfidentialFixtureView,
    pub(crate) programs: Vec<Vec<u8>>,
    /// Which predecessor this deployment funded.
    ///
    /// Carried rather than assumed, because two ceremonies now fund two
    /// different predecessors through this one entry point and every
    /// later step -- the funding step's handle, the coin count it
    /// expects, the opening reference an input resolves against -- has to
    /// ask which.
    pub(crate) predecessor: PredecessorShape,
}

impl LinkedDeployment {
    /// The predecessor fixture's digest.
    pub(crate) const fn predecessor_digest(&self) -> [u8; 32] {
        self.predecessor_digest
    }

    /// The successor fixture's digest.
    pub(crate) const fn successor_digest(&self) -> [u8; 32] {
        self.successor_digest
    }
}

/// The Wave-5 restart ceremony, step one.
pub struct PrivateRestartPlanner {
    stage: Stage,
    consumed: ConsumedReceipt,
    genesis_block_hash: Digest32,
    linked: Option<LinkedDeployment>,
    submitted: Option<Vec<u8>>,
    census: Option<OwnerSigningCensus>,
    spent_owner_bytes: Option<Vec<u8>>,
    record: PrivateRestartRecord,
}

impl PrivateRestartPlanner {
    /// The ceremony bound to one deployment's printed genesis identity.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed
    /// target does not build.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        Self::spending(printed_genesis_identity, ConsumedReceipt::Primary)
    }

    /// The ceremony consuming one named predecessor output.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed
    /// target does not build.
    pub fn spending(
        printed_genesis_identity: Digest32,
        consumed: ConsumedReceipt,
    ) -> Result<Self, VectorError> {
        reviewed_target()?;
        Ok(Self {
            stage: Stage::Issue,
            consumed,
            genesis_block_hash: printed_order(printed_genesis_identity),
            linked: None,
            submitted: None,
            census: None,
            spent_owner_bytes: None,
            record: PrivateRestartRecord::default(),
        })
    }

    /// The transcript.
    #[must_use]
    pub const fn record(&self) -> &PrivateRestartRecord {
        &self.record
    }

    /// The case identity of the confidential funding step.
    #[must_use]
    pub fn funding_case() -> OperationCaseId {
        OperationCaseId {
            operation: OperationStepKind::FundConfidential,
            step: FUND_STEP.to_owned(),
        }
    }

    /// Record a refusal and stop.
    fn refuse(&mut self, refusal: PrivateRestartRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.stage = Stage::Done;
        PlanRefused
    }

    /// Link the deployment against the asset the target issued, and
    /// register both fixtures against it.
    ///
    /// The deployment is welded to its asset: the linked programs push
    /// the asset as a literal, so every destination program moves with
    /// it. Registering before linking would bind a fixture to programs
    /// of a different deployment.
    fn settle_asset(&mut self, printed: &str) -> Result<(), PrivateRestartRefusal> {
        let linked = link_and_register(
            PredecessorShape::DualParity,
            self.consumed,
            printed,
            LiveShapeVocabulary::Demonstration,
            RESERVE_ASSET,
        )?;
        self.record.issued_asset = Some(printed.to_owned());
        self.record.predecessor_digest = Some(linked.predecessor_digest);
        self.record.successor_digest = Some(linked.successor_digest);
        self.linked = Some(linked);
        Ok(())
    }

    /// The confidential funding step, against the registered
    /// predecessor.
    fn funding_step(&self) -> Result<OperationStep, PrivateRestartRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(PrivateRestartRefusal::IssuanceNamedNoAsset)?;
        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(PrivateRestartRefusal::IssuanceNamedNoAsset)?;
        Ok(confidential_funding_step(linked, printed))
    }

    /// Take the funded coins from the node's own report of them.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), PrivateRestartRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
        self.record.coins = observe_funded_coins(linked, response)?;
        self.record.consumed_receipt = Some(self.consumed.name());
        self.record.consumed_commitment_prefix = self
            .record
            .coins
            .get(self.consumed.index())
            .and_then(|coin| match coin.value() {
                ValueField::Commitment(commitment) => commitment.first().copied(),
                // A confidential coin the node reported as explicit has
                // no parity to read, and inventing one would be
                // reporting a fact about the chain nobody observed.
                _ => None,
            });
        Ok(())
    }

    /// The control, finalized through the private lane's own entry
    /// point.
    /// The candidate's wire bytes, with every owner's authorization in
    /// its own input's witness.
    ///
    /// A thin wrapper over the shared [`build_control`], which the
    /// proof-negative ceremony also uses to reproduce this exact control
    /// before mutating one field of it.
    fn control_bytes(&mut self) -> Result<Vec<u8>, PrivateRestartRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(PrivateRestartRefusal::FundingCreatedNoPredecessor)?;
        let coin = self
            .record
            .coins
            .get(self.consumed.index())
            .ok_or(PrivateRestartRefusal::FundingCreatedNoPredecessor)?;
        let built = build_control(linked, coin, self.consumed, self.genesis_block_hash)?;
        self.spent_owner_bytes = built.spent_owner_bytes;
        self.record.receipt_leaves = built.receipt_leaves;
        self.record.output_witness_proof_bytes = built.output_witness_proof_bytes;
        let bytes = built.transaction.encode();
        self.record.submitted_bytes = bytes.len();
        self.census = Some(built.census);
        Ok(bytes)
    }

    /// Record what the target did with the control.
    fn settle_control(&mut self, response: &NativeOperationResponse) {
        self.record.observed_layer = Some(response.observed_layer);
        self.record
            .observed_detail
            .clone_from(&response.observed_detail);
        self.record
            .accepted_txid
            .clone_from(&response.accepted_txid);

        // The second origin, and only where there is an acceptance to
        // check. A refusal has no witness to read back.
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return;
        }
        let (Some(readback), Some(submitted), Some(census)) = (
            response.mined_readback.as_ref(),
            self.submitted.as_ref(),
            self.census.as_ref(),
        ) else {
            return;
        };

        let readback_matches_submission = readback.raw_transaction == *submitted;
        let verified = census
            .signing_inputs()
            .first()
            .map(|input| candidate_owner_message(census, input, WitnessVectorTreatment::BothGrown))
            .zip(self.spent_owner_bytes.as_ref())
            .is_some_and(|(message, owner)| {
                verify_readback_signature(&readback.raw_transaction, &message, owner)
            });

        self.record.reverification = Some(RestartReverification {
            accepted_txid: readback.transaction_id.clone(),
            readback_matches_submission,
            verified,
        });
    }
}

/// Link the deployment against the issued asset and register both the
/// predecessor and the one-to-one successor fixtures.
///
/// # Why it is a free function
///
/// Two ceremonies register the identical predecessor and successor: this
/// module's one-to-one control and the proof-negative ceremony's
/// balance-valid control. They must register the SAME fixtures — a
/// balance-valid control the proof-negatives mutate is only balance-valid
/// because it is the control this module submits — so the registration
/// lives in one place rather than being copied and drifting.
///
/// The `consumed` receipt selects which predecessor output the successor
/// balances against, exactly as step one and step two select their own.
///
/// # Errors
///
/// [`PrivateRestartRefusal::IssuanceNamedNoAsset`] where the printed
/// asset does not decode, [`PrivateRestartRefusal::RelinkRefused`] where
/// the deployment does not relink, and
/// [`PrivateRestartRefusal::FixtureNotRegistrable`] where either fixture
/// is not one the registry admits.
pub(crate) fn link_and_register(
    predecessor: PredecessorShape,
    consumed: ConsumedReceipt,
    printed: &str,
    vocabulary: LiveShapeVocabulary,
    reserve: [u8; 32],
) -> Result<LinkedDeployment, PrivateRestartRefusal> {
    link_and_register_composing(
        predecessor,
        consumed,
        printed,
        vocabulary,
        LiveTransferComposition::HomogeneousPrivate,
        reserve,
    )
}

/// The linked deployment of one vocabulary and one seated composition.
///
/// The general form, of which [`link_and_register`] is the homogeneous
/// private case. Two things follow the composition rather than the
/// vocabulary, and both matter.
///
/// The ABI is linked through the composing entry point, so a crossing
/// deployment seats its crossing constructor at the key its CONSUMED
/// side is recognized under. A crossing deployment is therefore its own
/// deployment with its own taptree, exactly as the fee-bearing one is,
/// and no identity recorded against the demonstration can move to buy
/// it.
///
/// The predecessor's programs follow the CONSUMED side, because those
/// coins are what this transfer spends and they must resolve through
/// the constructor that recognizes them. A successor's programs follow
/// the created side, and that is the ceremony's business rather than
/// this function's.
pub(crate) fn link_and_register_composing(
    predecessor: PredecessorShape,
    consumed: ConsumedReceipt,
    printed: &str,
    vocabulary: LiveShapeVocabulary,
    composition: LiveTransferComposition,
    reserve: [u8; 32],
) -> Result<LinkedDeployment, PrivateRestartRefusal> {
    let asset = asset_of(printed).ok_or(PrivateRestartRefusal::IssuanceNamedNoAsset)?;
    let commit_order = *asset.internal();
    // WHICH FEE DIGEST A DEPLOYMENT IS WELDED TO FOLLOWS WHETHER IT
    // MEANS TO SPEND A CONTROL THAT HAS A FEE.
    //
    // FEE_PROGRAM_DIGEST is a fixture constant that hashes to no program
    // at all, and its own doc says so: the demonstration deployment
    // keeps it because moving it would move the demonstration's
    // committed taptree and with it every live run-of-record identity
    // the plans cite. Nothing the demonstration builds ever executes the
    // fee clause, so nothing there notices.
    //
    // A fee-bearing candidate executes exactly that clause. The covenant
    // compares the fee output's scriptPubKey digest against the symbol,
    // construction writes the target-structural EMPTY program, and the
    // empty program's digest is the specification's SHA-256 of the empty
    // string -- so a deployment linked against the fixture constant
    // demands a value no program hashes to and refuses its own candidate
    // at OP_EQUALVERIFY. That is the defect the sponsor arc met on the
    // other lane and diagnosed in these words; it is met here for the
    // same reason and answered the same way.
    //
    // Supplying the real digest costs nothing HERE precisely because the
    // fee-bearing vocabulary is a separate deployment: its taptree is
    // already its own, so the only digest that moves is one no node has
    // ever accepted.
    let fee_digest = match vocabulary {
        LiveShapeVocabulary::Demonstration => FEE_PROGRAM_DIGEST,
        LiveShapeVocabulary::FeeBearing => crate::bundle::fee_program_digest(),
    };
    let abi = live_abi_composing(vocabulary, composition, commit_order, reserve, fee_digest)
        .map_err(|refusal| PrivateRestartRefusal::RelinkRefusedBy(format!("{refusal:?}")))?;

    // The predecessor outputs pay to the published owners' PRIVATE receipt
    // constructors. That is the whole difference between this ceremony
    // and the proof-bearing one, and it is what makes the successor a
    // live-receipt transfer rather than a spend of some other program.
    let owners = [FIRST_SCALAR, SECOND_SCALAR];
    // Under the CONSUMED side, because these coins are what the
    // successor spends: a predecessor output paid to a program the
    // spending covenant does not recognize is a coin nobody can spend.
    let programs = predecessor
        .owner_indices()
        .iter()
        .map(|index| owner_program(&abi, &owners[*index], composition.consumed()))
        .collect::<Result<Vec<_>, _>>()?;

    let handle = predecessor.handle();
    let outputs: Vec<ConfidentialFixtureOutput> = predecessor
        .roles()
        .iter()
        .zip(predecessor.amounts())
        .zip(&programs)
        .map(|((role, amount), program)| ConfidentialFixtureOutput {
            role: *role,
            semantic_amount: *amount,
            output_program: program.clone(),
        })
        .collect();
    let (predecessor_digest, predecessor_view) = register_multi(
        handle.as_str(),
        commit_order,
        // The funding input is explicit and contributes a zero value
        // blinder. The adapter's own catalogue entry says the same, and a
        // different figure would be refused there as a digest that
        // drifted.
        //
        // It is the SAME zero for both predecessors, and that is the
        // point: what separates them is the output count and not how they
        // are funded.
        [0_u8; 32],
        outputs,
    )
    .map_err(|_| PrivateRestartRefusal::FixtureNotRegistrable {
        handle: handle.as_str().to_owned(),
    })?;

    // The successor balances against the ONE consumed blinder, not
    // against a sum of two. A one-to-one control spends one output, so
    // the input blinder sum is that output's blinder — stated here
    // because it is the term a two-input ceremony gets for free and this
    // one does not.
    let consumed_blinder = *predecessor_view
        .outputs()
        .get(consumed.index())
        .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?
        .value_blinder()
        // A consumed predecessor output has an opening. A fee output has
        // none, and is unspendable besides, so a receipt that resolved to
        // one names no coin.
        .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?;

    let split = consumed.split();
    let (successor_digest, successor_view) = register(
        SUCCESSOR_HANDLE,
        commit_order,
        consumed_blinder,
        split,
        // The recipient is the second owner and the change goes back to
        // the first, so the two programs are the two constructors in the
        // other order.
        [
            private_program(&abi, &SECOND_SCALAR)?,
            private_program(&abi, &FIRST_SCALAR)?,
        ],
    )
    .map_err(|_| PrivateRestartRefusal::FixtureNotRegistrable {
        handle: SUCCESSOR_HANDLE.to_owned(),
    })?;

    Ok(LinkedDeployment {
        abi,
        asset,
        predecessor_digest: *predecessor_digest.bytes(),
        predecessor_view,
        successor_digest: *successor_digest.bytes(),
        successor_view,
        programs,
        predecessor,
    })
}

/// The issuing step that creates the disposable reserve the deployment is
/// linked against.
///
/// Shared so the proof-negative ceremony issues the identical disposable
/// asset the one-to-one ceremony does.
pub(crate) fn issue_step() -> OperationStep {
    OperationStep::new(
        ISSUE_STEP,
        OperationSubject::Funding(Box::new(TargetFundingSubject {
            issue_asset: true,
            asset: None,
            output_program: ISSUE_PROGRAM.to_vec(),
            outputs: ISSUE_OUTPUTS,
            amount_per_output: ISSUE_AMOUNT_PER_OUTPUT,
        })),
    )
}

/// The confidential funding step against the registered predecessor.
///
/// Shared so the proof-negative ceremony funds the identical predecessor
/// at the identical two receipt constructors.
pub(crate) fn confidential_funding_step(
    linked: &LinkedDeployment,
    printed: String,
) -> OperationStep {
    OperationStep::new(
        FUND_STEP,
        OperationSubject::ConfidentialFunding(Box::new(TargetConfidentialFundingSubject {
            issue_asset: false,
            asset: Some(printed),
            destinations: linked
                .programs
                .iter()
                .map(|program| ConfidentialFundingDestination {
                    output_program: program.clone(),
                })
                .collect(),
            binding: ConfidentialFundingBinding {
                fixture_handle: linked.predecessor.handle(),
                fixture_digest:
                    target_elements_conformance::protocol::ConfidentialFixtureDigest::new(
                        linked.predecessor_digest,
                    ),
                profiles: selected_profiles(),
            },
        })),
    )
}

/// The EXPLICIT funding step of an entry crossing.
///
/// It funds ordinary explicit coins at a receipt constructor's program,
/// which is what makes an entry crossing's consumed side explicit. The
/// coin is a RECEIPT and not a funding coin: what the covenant governs
/// is decided by the program it sits at, and this step puts it at one.
pub(crate) fn explicit_funding_step(
    program: Vec<u8>,
    printed: String,
    outputs: u8,
    amount_per_output: u64,
) -> OperationStep {
    OperationStep::new(
        FUND_STEP,
        OperationSubject::Funding(Box::new(TargetFundingSubject {
            issue_asset: false,
            asset: Some(printed),
            output_program: program,
            outputs,
            amount_per_output,
        })),
    )
}

/// Take the EXPLICIT funded coins the node reported.
///
/// Each is compared against what was asked for rather than replaced by
/// it: the amount the node reports must be the amount requested and the
/// program must be the one funded, which are the only two facts an
/// explicit coin has to agree about.
///
/// # Errors
///
/// [`PrivateRestartRefusal::FundingCreatedNoPredecessor`] where the node
/// reported a different number of outputs than were asked for.
pub(crate) fn observe_explicit_coins(
    response: &NativeOperationResponse,
    asset: AssetId,
    program: &[u8],
    expected: usize,
    amount: u64,
) -> Result<Vec<RestartConfidentialCoin>, PrivateRestartRefusal> {
    if response.funded_outputs.len() != expected {
        return Err(PrivateRestartRefusal::FundingCreatedNoPredecessor);
    }
    response
        .funded_outputs
        .iter()
        .map(|funded| {
            let outpoint = crate::live_owner_observation::outpoint_of(&funded.outpoint)
                .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
            let observed_asset = crate::live_owner_observation::asset_of(&funded.asset)
                .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
            let observed_program = crate::live_owner_observation::decode_hex(&funded.script)
                .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
            let agrees = observed_asset == asset
                && funded.amount_satoshis == amount
                && observed_program == program;
            Ok(RestartConfidentialCoin::explicit(
                outpoint,
                AssetField::Explicit(observed_asset),
                ValueField::Explicit(funded.amount_satoshis),
                observed_program,
                agrees,
            ))
        })
        .collect()
}

/// Take the funded coins from the node's own report of them, each decoded
/// and compared against the expectation rather than replaced by it.
///
/// # Errors
///
/// [`PrivateRestartRefusal::FundingCreatedNoPredecessor`] where the node
/// reported a different number of coins than the predecessor has outputs,
/// and [`PrivateRestartRefusal::MalformedConfidentialOutput`] where a
/// reported coin does not decode.
pub(crate) fn observe_funded_coins(
    linked: &LinkedDeployment,
    response: &NativeOperationResponse,
) -> Result<Vec<RestartConfidentialCoin>, PrivateRestartRefusal> {
    if response.confidential_funded_outputs.len() != linked.predecessor.outputs() {
        return Err(PrivateRestartRefusal::FundingCreatedNoPredecessor);
    }
    let mut coins = Vec::with_capacity(response.confidential_funded_outputs.len());
    for (index, funded) in response.confidential_funded_outputs.iter().enumerate() {
        coins.push(observe_one_coin(linked, index, funded)?);
    }
    Ok(coins)
}

/// One confidential coin, decoded from the node's report and compared
/// against the expectation rather than replaced by it.
///
/// # Errors
///
/// [`PrivateRestartRefusal::MalformedConfidentialOutput`] where the
/// reported coin does not decode or its projected output is absent.
fn observe_one_coin(
    linked: &LinkedDeployment,
    index: usize,
    funded: &ConfidentialFundedOutput,
) -> Result<RestartConfidentialCoin, PrivateRestartRefusal> {
    let outpoint =
        outpoint_of(&funded.outpoint).ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
    let asset = asset_of(&funded.explicit_asset)
        .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
    let program =
        decode_hex(&funded.script).ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
    let commitment = <[u8; COMMITMENT_BYTES]>::try_from(funded.value_commitment.as_slice())
        .map_err(|_| PrivateRestartRefusal::MalformedConfidentialOutput)?;

    let projected = linked
        .predecessor_view
        .outputs()
        .get(index)
        .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
    let checker = FirstPartyCommitmentCheck::new();
    let derived = checker
        .recompute(
            linked.asset,
            projected.semantic_amount(),
            projected
                .value_blinder()
                .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?,
        )
        .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
    let matches_expectation = asset == linked.asset
        && program == *projected.output_program()
        && commitment == *derived.bytes();

    Ok(RestartConfidentialCoin {
        outpoint,
        asset: AssetField::Explicit(asset),
        value: ValueField::Commitment(commitment),
        program,
        rangeproof_bytes: funded.rangeproof.len(),
        matches_expectation,
    })
}

/// One built control: the structured candidate and everything a caller
/// needs to submit it, verify its readback, or derive a proof-negative
/// mutant from it.
///
/// The transaction is structured rather than encoded, so a proof-negative
/// ceremony can mutate exactly one output field and re-encode rather than
/// hunt for a byte offset in a serialized blob.
pub(crate) struct BuiltControl {
    /// The complete candidate, every owner authorization in place.
    pub(crate) transaction: TargetTransaction,
    /// The signing census the authorizations were taken over.
    pub(crate) census: OwnerSigningCensus,
    /// The spent receipt owner's key bytes, for readback verification.
    pub(crate) spent_owner_bytes: Option<Vec<u8>>,
    /// How many receipt leaves the control consumed.
    pub(crate) receipt_leaves: usize,
    /// The range-proof byte length of each output-witness entry.
    pub(crate) output_witness_proof_bytes: Vec<usize>,
}

/// Finalize the one-to-one control through the private lane's own entry
/// point, against a coin the node reported.
///
/// # Errors
///
/// [`PrivateRestartRefusal::ControlNotRequestable`] where the request or
/// its view will not build, [`PrivateRestartRefusal::SubstrateUnavailable`]
/// where a published owner does not build, and
/// [`PrivateRestartRefusal::FinalizationRefused`] where the private
/// finalization refuses the control.
fn finalize_control(
    linked: &LinkedDeployment,
    coin: &RestartConfidentialCoin,
    consumed: ConsumedReceipt,
) -> Result<PrivateLiveFinalization, PrivateRestartRefusal> {
    // The view is the node's report of the coin, not the ceremony's
    // expectation of it.
    let view = PublicConstructionView::new([PublicOutputView::new(
        coin.outpoint(),
        coin.asset(),
        coin.value(),
        coin.program().to_vec(),
    )])
    .map_err(|_| PrivateRestartRefusal::ControlNotRequestable)?;

    let recipient =
        published_owner(&SECOND_SCALAR).map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?;
    let sender =
        published_owner(&FIRST_SCALAR).map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?;
    let destination = |owner, amount| {
        ProtocolValue::new(amount)
            .map(|value| LiveReceiptDestination::new(OwnerParameter::new(owner), value))
            .map_err(|_| PrivateRestartRefusal::ControlNotRequestable)
    };
    let request = LiveTransferRequest::new(
        [coin.outpoint()],
        [
            destination(recipient, consumed.split()[0])?,
            destination(sender, consumed.split()[1])?,
        ],
        LiveTransferRepresentationPlan::PrivateCommitted,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        // The private form carries published randomness by the request's
        // own rule. The materializer takes its blinders from fixtures
        // rather than from this, so it is present because the request
        // vocabulary requires it and is not a source of any opening.
        Some(PublicTestRandomness::from_published_bytes([0x7e; 32])),
    )
    .map_err(|_| PrivateRestartRefusal::ControlNotRequestable)?;

    let openings = PrivateLiveOpenings::new(
        vec![PrivateInputOpening {
            region: ConfidentialInputRegion::Receipt,
            opening: Some(FixtureOpeningReference::new(
                linked.predecessor.handle().as_str().to_owned(),
                linked.predecessor_digest,
                consumed.index(),
            )),
            explicit_amount: linked.predecessor.amounts()[consumed.index()],
            zero_asset_blinder: [0_u8; SCALAR_BYTES],
        }],
        vec![
            PrivateDestinationOpening {
                fixture: FixtureOpeningReference::new(
                    SUCCESSOR_HANDLE.to_owned(),
                    linked.successor_digest,
                    0,
                ),
                role: ConfidentialOutputRole::Primary,
            },
            PrivateDestinationOpening {
                fixture: FixtureOpeningReference::new(
                    SUCCESSOR_HANDLE.to_owned(),
                    linked.successor_digest,
                    1,
                ),
                role: ConfidentialOutputRole::Balancing,
            },
        ],
        NonProtocolFundingRegion::default(),
        materialization_profiles(),
    );

    let fixtures = FrozenConfidentialFixtureView::new(BTreeMap::from([
        (
            linked.predecessor.handle().as_str().to_owned(),
            linked.predecessor_view.clone(),
        ),
        (SUCCESSOR_HANDLE.to_owned(), linked.successor_view.clone()),
    ]));

    finalize_private_live_transfer(
        &reviewed_target().map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?,
        &linked.abi,
        &request,
        &view,
        None,
        &openings,
        &fixtures,
        &ReferenceConfidentialMaterializer::new(),
        &FirstPartyCommitmentCheck::new(),
    )
    .map_err(|refusal| PrivateRestartRefusal::FinalizationRefused(format!("{refusal:?}")))
}

/// Build the complete one-to-one control against a coin the node
/// reported, every owner's authorization in its own input's witness.
///
/// Shared by the one-to-one control ceremony and the proof-negative
/// ceremony: both submit the same control, and a control the
/// proof-negatives mutate is only balance-valid because it is the control
/// this builds.
///
/// # Errors
///
/// Every construction member of [`PrivateRestartRefusal`]: the
/// finalization's, [`PrivateRestartRefusal::CensusRefused`],
/// [`PrivateRestartRefusal::SigningRefused`], and
/// [`PrivateRestartRefusal::CandidateNotSerializable`].
pub(crate) fn build_control(
    linked: &LinkedDeployment,
    coin: &RestartConfidentialCoin,
    consumed: ConsumedReceipt,
    genesis_block_hash: Digest32,
) -> Result<BuiltControl, PrivateRestartRefusal> {
    let finalization = finalize_control(linked, coin, consumed)?;
    assemble_control(&finalization, genesis_block_hash, None)
}

/// Census, sign, and assemble one finalized private candidate into its
/// wire form, every owner's authorization in its own input's witness.
///
/// The shape-independent tail of [`build_control`]: it takes whatever the
/// private finalization produced — one input or several, two outputs or
/// more, sponsored or not — and signs each receipt over the leaf that
/// input executes. The one-to-one control and the multi-output and
/// multi-input shapes of the restart order's fifth step all share it, so
/// a change to how a candidate is signed and serialized is a change in
/// one place.
///
/// # The witness vector is indexed by position and not appended to
///
/// Because the input order has two regions and only one of them is
/// signed here. This used to push one witness per RECEIPT record onto a
/// vector sized by the receipt count, which is correct exactly while
/// every input is a receipt: the vector is handed to the transaction
/// type positionally, so a sponsored candidate would have given a
/// three-input transaction two witnesses, and a sponsor coin anywhere
/// but last would have shifted every receipt's witness onto another
/// input. Each witness is now placed at the position it authorizes, and
/// the positions nothing placed are the sponsor's.
///
/// # Why the sponsor is signed here and the owners are signed the same
///
/// §1.9 keeps the sponsor's own authorization outside protocol data, so
/// it is collected through the capability rather than through the owner
/// responses — the explicit lane's arrangement, mirrored rather than
/// reinvented. What it signs over is the PROOF-FINALIZED bytes, which is
/// the whole reason this lane needs its own sponsor stage: the explicit
/// lane's protected bytes are the witnessless serialization, and this
/// candidate's are the frozen ones the materializer produced.
///
/// # Errors
///
/// The census, signing, sponsor, and serialization members of
/// [`PrivateRestartRefusal`].
pub(crate) fn assemble_control(
    finalization: &PrivateLiveFinalization,
    genesis_block_hash: Digest32,
    sponsor: Option<&dyn SponsorCapability>,
) -> Result<BuiltControl, PrivateRestartRefusal> {
    let materialized = finalization.materialized();

    // Each receipt's signing request is built from the leaf THAT INPUT
    // executes, which the finalization supplies. A ceremony that chose
    // its own leaf would be authorizing a different program than the one
    // the coin pays to.
    let requests: Vec<OwnerSigningInputRequest> = finalization
        .receipts()
        .iter()
        .map(|record| {
            OwnerSigningInputRequest::new(
                u32::from(record.position()),
                leaf_hash(LeafVersion::TAPSCRIPT, record.leaf_script()),
                LeafVersion::TAPSCRIPT,
                transaction::OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                record.control_block().to_vec(),
            )
        })
        .collect();

    let target = reviewed_target().map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?;
    let curve = crate::live_capability::OracleLiveCurve::new(
        reviewed_target().map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?,
    );
    let census = OwnerSigningCensus::from_proof_finalized(
        &target,
        materialized,
        LiveDeployment::new(genesis_block_hash),
        &requests,
        &curve,
    )
    .map_err(PrivateRestartRefusal::CensusRefused)?;

    let mut placed: BTreeMap<u16, InputWitness> = BTreeMap::new();
    for record in finalization.receipts() {
        let position = u32::from(record.position());
        let input = census
            .signing_inputs()
            .iter()
            .find(|entry| entry.input_index() == position)
            .ok_or(PrivateRestartRefusal::CensusRefused(
                OwnerCensusRefusal::NoSigningInputRequested,
            ))?;
        // BothGrown is what consensus hashes. This ceremony runs no
        // witness-vector control, so there is one treatment and it is the
        // real one.
        let message = candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
        let scalar = scalar_of(record.owner())?;
        let material =
            signing_material(&scalar).map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?;
        let signature = material
            .sign(&message, &RESTART_AUXILIARY)
            .map_err(|_| PrivateRestartRefusal::SigningRefused)?
            .to_vec();
        placed.insert(
            record.position(),
            InputWitness::new(vec![
                signature,
                record.leaf_script().to_vec(),
                record.control_block().to_vec(),
            ]),
        );
    }

    let spent_owner_bytes = finalization
        .receipts()
        .first()
        .map(|record| record.owner().key().bytes().to_vec());

    let frozen = materialized.proof_finalized().protected();

    // The sponsor's half, minted where the candidate is. This ceremony
    // does not build the signing requests: a request names the exact
    // bytes an authorization is produced against, and the transaction
    // crate keeps that constructor to itself so no caller can ask a
    // sponsor to authorize bytes that are not the candidate's.
    let sponsored = private_sponsor_witnesses(finalization, sponsor)
        .map_err(|refusal| PrivateRestartRefusal::SponsorWitnessRefused(format!("{refusal:?}")))?;

    // Every input position, receipts from one map and the sponsor suffix
    // from the other. The walk is over the CANDIDATE's inputs rather
    // than over either region's count, so a position neither side claims
    // is a missing witness here instead of a cardinality a node
    // discovers.
    let mut witnesses = Vec::with_capacity(frozen.inputs().len());
    for index in 0..frozen.inputs().len() {
        let position = u16::try_from(index).unwrap_or(u16::MAX);
        let witness = placed
            .get(&position)
            .or_else(|| sponsored.get(&position))
            .ok_or(PrivateRestartRefusal::InputPositionUnauthorized { position })?;
        witnesses.push(witness.clone());
    }
    let receipt_leaves = finalization.receipts().len();
    let output_witness_proof_bytes = frozen
        .output_witnesses()
        .iter()
        .map(|witness| witness.range_proof().len())
        .collect();

    let transaction = TargetTransaction::with_output_witnesses(
        frozen.version(),
        frozen.inputs().to_vec(),
        frozen.outputs().to_vec(),
        frozen.lock_time(),
        witnesses,
        frozen.output_witnesses().to_vec(),
    )
    .map_err(|_| PrivateRestartRefusal::CandidateNotSerializable)?;

    Ok(BuiltControl {
        transaction,
        census,
        spent_owner_bytes,
        receipt_leaves,
        output_witness_proof_bytes,
    })
}

/// The private receipt constructor's program for one published owner.
pub(crate) fn private_program(
    abi: &CandidateLiveTransferAbi,
    scalar: &[u8; SCALAR_BYTES],
) -> Result<Vec<u8>, PrivateRestartRefusal> {
    owner_program(
        abi,
        scalar,
        LiveTransferRepresentationPlan::PrivateCommitted,
    )
}

/// One published owner's receipt program under a STATED plan.
///
/// The general form, of which [`private_program`] is the private case.
/// A crossing ceremony needs it because the plan a destination's program
/// is resolved under is the CREATED side's, which for an exit crossing
/// is not the side its own receipts were recognized under. Resolving a
/// crossing successor's programs under the consumed plan would register
/// a fixture whose outputs the construction does not pay to, and the
/// candidate would then diverge from its own manifest.
pub(crate) fn owner_program(
    abi: &CandidateLiveTransferAbi,
    scalar: &[u8; SCALAR_BYTES],
    plan: LiveTransferRepresentationPlan,
) -> Result<Vec<u8>, PrivateRestartRefusal> {
    let owner = published_owner(scalar).map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?;
    Ok(abi
        .destinations()
        .get(&OwnerParameter::new(owner), plan)
        .ok_or(PrivateRestartRefusal::NoConstructorForPlan { plan })?
        .instance()
        .program()
        .to_vec())
}

/// Which published scalar an owner parameter belongs to.
///
/// The ceremony has two published owners and the finalization reports
/// which of them owns each receipt. Matching on the parameter rather
/// than assuming a position is what keeps the signature and the leaf
/// about the same owner.
fn scalar_of(owner: &OwnerParameter) -> Result<[u8; SCALAR_BYTES], PrivateRestartRefusal> {
    for scalar in [FIRST_SCALAR, SECOND_SCALAR] {
        let candidate =
            published_owner(&scalar).map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?;
        if OwnerParameter::new(candidate) == *owner {
            return Ok(scalar);
        }
    }
    Err(PrivateRestartRefusal::SigningRefused)
}

/// Whether the signature the node reported verifies against `message`.
///
/// The signature is taken from the target's own copy of the transaction
/// rather than from the value the builder still holds, which is what
/// makes this a second origin at all: checking the builder's signature
/// against the builder's message would be comparing a value with
/// itself. The owner it is checked for is the one the finalization said
/// owns the spent receipt, so a run whose leaf and whose key were about
/// two different owners fails here rather than passing quietly.
pub(crate) fn verify_readback_signature(
    raw: &[u8],
    message: &Digest32,
    owner_bytes: &[u8],
) -> bool {
    let Ok(decoded) = TargetTransaction::decode(raw) else {
        return false;
    };
    let Some(signature) = decoded
        .witnesses()
        .first()
        .and_then(|witness| witness.stack().first())
    else {
        return false;
    };
    let Ok(target) = reviewed_target() else {
        return false;
    };
    verify_owner_signature(&target, owner_bytes, message, signature).is_ok()
}

impl TargetOperationPlanner for PrivateRestartPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    let Some(printed) = response.issued_asset.clone() else {
                        return Err(self.refuse(PrivateRestartRefusal::IssuanceNamedNoAsset));
                    };
                    if let Err(refusal) = self.settle_asset(&printed) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund;
                }
                Stage::Fund => {
                    if let Err(refusal) = self.settle_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Submit;
                }
                Stage::Submit => {
                    self.settle_control(response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(issue_step())),
            Stage::Fund => match self.funding_step() {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Submit => match self.control_bytes() {
                Ok(bytes) => {
                    self.submitted = Some(bytes.clone());
                    Ok(Some(OperationStep::new(
                        CONTROL_STEP,
                        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                            transaction_bytes: bytes,
                        })),
                    )))
                }
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Done => Ok(None),
        }
    }
}

/// The transcript, one fact per line.
#[must_use]
pub fn render_private_restart(record: &PrivateRestartRecord) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    out.push_str("run private-one-to-one-control\n");
    let _ = writeln!(
        out,
        "issued_asset {}",
        record.issued_asset().unwrap_or("absent"),
    );
    for (name, digest) in [
        ("predecessor_digest", record.predecessor_digest()),
        ("successor_digest", record.successor_digest()),
    ] {
        let _ = writeln!(
            out,
            "{name} {}",
            digest.map_or_else(|| "absent".to_owned(), hex),
        );
    }
    for (index, coin) in record.coins().iter().enumerate() {
        let _ = writeln!(
            out,
            "coin {index} rangeproof_bytes {} matches_expectation {}",
            coin.rangeproof_bytes(),
            coin.matches_expectation(),
        );
    }
    let _ = writeln!(
        out,
        "consumed_receipt {}",
        record.consumed_receipt().unwrap_or("none"),
    );
    let _ = writeln!(
        out,
        "consumed_commitment_prefix {}",
        record
            .consumed_commitment_prefix()
            .map_or_else(|| "none".to_owned(), |prefix| format!("{prefix:#04x}")),
    );
    let _ = writeln!(out, "receipt_leaves {}", record.receipt_leaves());
    let _ = writeln!(
        out,
        "output_witness_proof_bytes {:?}",
        record.output_witness_proof_bytes(),
    );
    let _ = writeln!(out, "submitted_bytes {}", record.submitted_bytes());
    let _ = writeln!(
        out,
        "observed_layer {}",
        record
            .observed_layer()
            .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
    );
    let _ = writeln!(
        out,
        "observed_detail {}",
        record.observed_detail().unwrap_or("none"),
    );
    let _ = writeln!(
        out,
        "accepted_txid {}",
        record.accepted_txid().unwrap_or("none"),
    );
    if let Some(check) = record.reverification() {
        let _ = writeln!(
            out,
            "reverification readback_matches_submission {} verified {}",
            check.readback_matches_submission(),
            check.verified(),
        );
    }
    let _ = writeln!(
        out,
        "construction_refusal {}",
        record
            .refusal()
            .map_or_else(|| "none".to_owned(), |refusal| format!("{refusal:?}")),
    );
    let _ = writeln!(
        out,
        "produced_an_accepted_control {}",
        record.produced_an_accepted_control(),
    );

    // What this run does NOT establish, in its own bytes.
    out.push_str("evidences_no_negative_case true\n");
    out.push_str("evidences_no_conservation_claim true\n");
    out.push_str("evidences_no_minimality_relation true\n");
    out.push_str("moves_the_sponsor_row false\n");
    out
}

/// The run of record: what one execution of step one observed.
///
/// # Why the observation is a constant and not a stored file
///
/// The evidence a run produces is the observation, and an observation
/// nobody can name is not evidence. These constants are the identities
/// and figures ONE run against a real node produced, written down so
/// that a later reader can ask the chain the same question, and so that
/// a claim made anywhere in this workspace about step one can be traced
/// to a transaction identity rather than to a test having been written.
///
/// It re-runs nothing and proves nothing by existing. What it does is
/// make the run's own answer quotable.
///
/// The target: Elements Core v28.99.0-b7fc5d080a7e, at the pinned tip
/// the lane binds itself to, on a disposable development chain the run
/// created and destroyed.
pub mod run_of_record {
    /// The disposable asset the run issued.
    pub const ISSUED_ASSET: &str =
        "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

    /// The predecessor fixture's digest.
    pub const PREDECESSOR_DIGEST: &str =
        "1d5dc685de3ab2cd6d9baee7169993c0f7ab5df20a48a4bfb553ce9ec51f39fc";

    /// The successor fixture's digest.
    pub const SUCCESSOR_DIGEST: &str =
        "f28590a2c927b62e356dd7f79bc450d51917135ecc99c6c4bba2053b06d48fcc";

    /// The identity the target computed for the accepted control.
    ///
    /// The whole of step one's evidence, in one string. Every row this
    /// wave moves is moved on THIS acceptance and cites it.
    pub const ACCEPTED_TXID: &str =
        "4571a077826d45f64402a5c83ac9c0454fe42cf53b75f7aac2c8d07b574ad152";

    crate::recorded_acceptance::mint_recorded_acceptance!(accepted, ACCEPTED_TXID);

    /// How many bytes were handed to the node.
    pub const SUBMITTED_BYTES: usize = 9_136;

    /// The range-proof bytes each of the candidate's outputs carried.
    pub const OUTPUT_WITNESS_PROOF_BYTES: [usize; 2] = [4_174, 4_174];

    /// How many receipt inputs the control consumed.
    ///
    /// One. It is a one-to-one control, and the figure is here so that a
    /// later reader does not have to take the word "one-to-one" for it.
    pub const RECEIPT_LEAVES: usize = 1;

    /// The commitment prefix the consumed coin carried.
    ///
    /// The first of the target's two admitted parities, as the NODE
    /// reported the commitment.
    pub const CONSUMED_COMMITMENT_PREFIX: u8 = 0x08;

    /// The run's wall time, in seconds.
    pub const WALL_SECONDS: f64 = 11.5;

    /// The successor fixture's digest for the second parity's run.
    ///
    /// Different from [`SUCCESSOR_DIGEST`] because the two runs consume
    /// different receipts and therefore balance against different
    /// blinders and split different amounts. A pair of runs whose
    /// successor digests agreed would be one run reported twice.
    pub const PARITY_SUCCESSOR_DIGEST: &str =
        "6b80d64e5692745ce4dbfa0fb6fcf36e5ef27c3c452afad49d11de0a2ac54748";

    /// The identity the target computed for the second parity's
    /// accepted successor.
    ///
    /// The run that COMPLETED the pair. The first parity's acceptance is
    /// [`ACCEPTED_TXID`]; both are complete accepted successors, and it
    /// takes both to say that both parities were exercised.
    pub const PARITY_ACCEPTED_TXID: &str =
        "45f1c5669cdc868f5612f6b18b2e285b791d7f093e45b2d28147eac63427dd95";

    crate::recorded_acceptance::mint_recorded_acceptance!(parity_accepted, PARITY_ACCEPTED_TXID);

    /// The commitment prefix the second run's consumed coin carried.
    pub const PARITY_CONSUMED_COMMITMENT_PREFIX: u8 = 0x09;
}

/// One digest as its printed spelling.
fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

#[cfg(test)]
mod tests {
    use super::{PrivateRestartPlanner, PrivateRestartRecord, render_private_restart};
    use crate::confidential_predecessor::PREDECESSOR_AMOUNTS;
    use transaction::taproot::Digest32;

    #[test]
    fn the_control_conserves_the_one_receipt_it_consumes() {
        // Not a claim that the target agrees. A statement that the
        // amounts this ceremony asks for add up, so that a target
        // refusal cannot be attributed to arithmetic nobody did.
        for consumed in super::ConsumedReceipt::ALL {
            let [recipient, change] = consumed.split();
            assert_eq!(
                recipient + change,
                PREDECESSOR_AMOUNTS[consumed.index()],
                "{} does not conserve",
                consumed.name(),
            );
        }
    }

    #[test]
    fn a_run_that_observed_nothing_produced_no_accepted_control() {
        // The question step one asks, answered from the observation and
        // never from the ceremony having been built.
        let record = PrivateRestartRecord::default();
        assert!(!record.produced_an_accepted_control());
        let rendered = render_private_restart(&record);
        assert!(rendered.contains("produced_an_accepted_control false"));
        assert!(rendered.contains("observed_layer none"));
        assert!(rendered.contains("moves_the_sponsor_row false"));
    }

    #[test]
    fn the_run_of_record_names_one_acceptance_and_one_receipt() {
        // The figures are the run's, and this checks their SHAPE rather
        // than re-deriving them: an identity of the right width, one
        // receipt consumed, and two outputs each carrying a real proof.
        // A run of record whose numbers disagreed with its own claim
        // would be the one thing it exists to prevent.
        use super::run_of_record as run;

        assert_eq!(run::ACCEPTED_TXID.len(), 64);
        assert_eq!(run::PREDECESSOR_DIGEST.len(), 64);
        assert_ne!(run::PREDECESSOR_DIGEST, run::SUCCESSOR_DIGEST);
        assert_eq!(run::RECEIPT_LEAVES, 1);
        assert_eq!(run::OUTPUT_WITNESS_PROOF_BYTES.len(), 2);
        assert!(
            run::OUTPUT_WITNESS_PROOF_BYTES
                .iter()
                .all(|bytes| *bytes > 2)
        );
        assert!(run::SUBMITTED_BYTES > run::OUTPUT_WITNESS_PROOF_BYTES.iter().sum::<usize>());

        // The two runs are two runs. Different successors, different
        // identities, and the two admitted parities between them — a
        // pair whose members agreed anywhere here would be one run
        // reported twice.
        assert_ne!(run::ACCEPTED_TXID, run::PARITY_ACCEPTED_TXID);
        assert_ne!(run::SUCCESSOR_DIGEST, run::PARITY_SUCCESSOR_DIGEST);
        assert_eq!(run::PARITY_ACCEPTED_TXID.len(), 64);
        assert_eq!(
            [
                run::CONSUMED_COMMITMENT_PREFIX,
                run::PARITY_CONSUMED_COMMITMENT_PREFIX
            ],
            [0x08, 0x09],
            "the two runs did not exercise the two admitted parities",
        );
    }

    #[test]
    fn the_planner_builds_and_starts_at_the_issuing_step() {
        let planner =
            PrivateRestartPlanner::new(Digest32::from([0x11; 32])).expect("the ceremony builds");
        assert!(planner.record().issued_asset().is_none());
        assert_eq!(planner.record().receipt_leaves(), 0);
    }
}

#[cfg(test)]
mod byte_identity_tests {
    use super::{
        ConsumedReceipt, LiveShapeVocabulary, RESERVE_ASSET, hex, link_and_register,
        run_of_record as run,
    };
    use crate::confidential_predecessor::PredecessorShape;

    /// The two fixtures of the run of record register under exactly the
    /// digests that run recorded.
    ///
    /// # Why this test is worth its weight
    ///
    /// The digests below were written down by a ceremony that ran against
    /// a pinned node BEFORE the single-output form and the fee role were
    /// added to the registry's vocabulary. They are therefore an
    /// expectation this workspace cannot quietly move: recomputing them
    /// from the manifests re-derives every blinder, every nonce input,
    /// every range-proof seed, and every commitment prefix of a fixture
    /// whose successor a target ACCEPTED at
    /// `run_of_record::ACCEPTED_TXID`.
    ///
    /// So this is the byte-identity clause of both removals, stated as a
    /// running check rather than as a claim in a commit message. A
    /// vocabulary change that perturbed the derivation of an existing
    /// case — a transcript member added unconditionally, a role code
    /// reassigned, a search whose counter moved — would land here, and it
    /// would land here before it landed on a chain.
    #[test]
    fn the_run_of_record_fixtures_register_under_the_digests_it_recorded() {
        for consumed in ConsumedReceipt::ALL {
            let linked = link_and_register(
                PredecessorShape::DualParity,
                consumed,
                run::ISSUED_ASSET,
                LiveShapeVocabulary::Demonstration,
                RESERVE_ASSET,
            )
            .expect("the run of record's own fixtures register");

            // The predecessor is the same manifest for both runs, so both
            // must land on the one recorded digest.
            assert_eq!(
                hex(linked.predecessor_digest()),
                run::PREDECESSOR_DIGEST,
                "the predecessor fixture drifted from the run of record",
            );

            // The successors differ, and each run recorded its own. The
            // primary receipt is the run whose consumed commitment
            // carried the first admitted prefix.
            let expected = match consumed {
                ConsumedReceipt::Primary => run::SUCCESSOR_DIGEST,
                ConsumedReceipt::Balancing => run::PARITY_SUCCESSOR_DIGEST,
            };
            assert_eq!(
                hex(linked.successor_digest()),
                expected,
                "the successor fixture for {} drifted from the run of record",
                consumed.name(),
            );
        }
    }
}
