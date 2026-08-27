//! One key-path spend attempt against a funded explicit constructor.
//!
//! # What this probe is, in one sentence
//!
//! Every constructor this workspace builds is spent by its script path,
//! and its internal key is a published point nobody is supposed to hold
//! a scalar for. This module funds one such constructor, offers the
//! target a spend that takes the *key* path instead, and writes down
//! what the target says.
//!
//! # What a refusal discharges, and what it does not
//!
//! Stated first, because it is the whole discipline of the probe and the
//! sentence a reader is most likely to overshoot.
//!
//! A refusal discharges exactly one thing: that the attempt was made
//! against a real target and was refused. It establishes NOTHING about
//! who knows the discrete logarithm of the internal key. It could not:
//! the signature this probe offers is by a published test scalar that is
//! not the output key, so a target refusing it is refusing a signature
//! that does not verify, which is what any target would do for any key
//! anyone does not hold. The residual assumption named in
//! [`target_elements_conformance::constructor::internal_key`] — that no
//! scalar for the nothing-up-my-sleeve point exists or can be found —
//! stands unchanged, before this run and after it.
//!
//! [`KeyPathProbeRecord::non_claims`] carries those sentences as data
//! rather than leaving them in this comment, so the committed artifact
//! says them in its own bytes.
//!
//! # Why the probe is worth running anyway
//!
//! Because the alternative to an observed refusal is an unobserved
//! assumption about the wire. Nobody had offered this target a one-item
//! witness at one of these programs, so nothing was known about what
//! layer it lands at, what the node says, or whether the submission wire
//! carries such a candidate at all. Those are facts about a boundary,
//! and this run measures them.
//!
//! # The wire is used unchanged
//!
//! The submission subject carries transaction bytes and nothing else, so
//! a key-path candidate is expressible through it exactly as a
//! script-path one is. Nothing here modifies the generic wire, and had
//! the attempt turned out to be inexpressible that would have been a
//! finding to report rather than a licence to widen the wire.
//!
//! # The accepted control, and why phase B needed one
//!
//! Phase A offered the attempt and nothing else, so its refusal had no
//! pair. A census of rejections from a pipeline that has never had a
//! transaction accepted establishes that the target rejects things,
//! which every target that rejects everything also does — so the §15
//! register admits a refusal as evidence only against an UNMUTATED
//! control accepted ON THE SAME CHAIN.
//!
//! The control here is not a second candidate. It is the SAME candidate,
//! spent by the script path the constructor is built for, and the two
//! submissions' witnessless serializations are compared byte for byte
//! and recorded rather than asserted, so "differing in the witness
//! alone" is a measurement.
//!
//! The attempt is offered FIRST and the control second, and the order is
//! forced rather than chosen: both spend the one funded receipt, so a
//! control accepted first would have spent the coin and the attempt
//! after it would have drawn a missing input rather than the verdict the
//! probe is about.
//!
//! # Every key here is published test material
//!
//! The signing scalar is the BIP-340 specification's own first appendix
//! secret key, admitted under ADR-015's test-material rule and already
//! the first published owner of every other ceremony in this package.
//! Using the owner's own key rather than a fresh one is deliberate: it
//! is the key whose script-path authorization this deployment has
//! already seen a target accept, so the run records that the same key
//! taking the other path is refused. That is a sharper record and it is
//! still not a claim about the internal key.

use std::fmt::Write as _;

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::LeafVersion;
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    FundedOutput, NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::{
    AssetField, AssetId, InputWitness, Outpoint, TargetTransaction, ValueField,
};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{
    AnnexDisposition, IssuanceDisposition, LiveDeployment, OWNER_CODESEPARATOR_POSITION,
    OwnerCensusRefusal, OwnerSigningCensus, OwnerSigningInputRequest,
};
use transaction::live_construct::{
    LiveConstructionReport, complete_live_transfer, finalize_live_transfer,
};
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_message::{
    WitnessVectorTreatment, candidate_key_path_message, candidate_owner_message,
};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::taproot::{Digest32, leaf_hash};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{
    ObservedFundedCoin, asset_of, decode_hex, outpoint_of, printed, printed_order,
};
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, RESERVE_ASSET, SECOND_SCALAR, demonstration_live_abi,
    live_abi_for_asset, published_owner, reviewed_target, signing_material,
};

/// What the single funded receipt holds.
///
/// The figure the explicit observation ceremony funds with, kept equal
/// so the two runs' candidates are comparable at the amounts.
const RECEIPT_AMOUNT: u64 = 5_000;

/// How many receipts the probe funds.
///
/// ONE, and the departure from the observation ceremony's two is the
/// point rather than an economy. The subject of this run is a single
/// input carrying a single witness item, and a two-input candidate would
/// have made "the one-item witness" a phrase about two witnesses that a
/// reader has to reassemble.
const RECEIPT_COUNT: u8 = 1;

/// The auxiliary value the probe's signature is taken with.
///
/// A published constant rather than randomness, for the reason every
/// other ceremony here fixes one: BIP-340 masks the scalar with it, so a
/// fixed value is what makes the submitted bytes reproducible from
/// inputs that are all written down.
const PROBE_AUXILIARY: [u8; FIELD_ELEMENT_BYTES] = [0x66; FIELD_ELEMENT_BYTES];

/// The probe's name for its one submission.
pub const ATTEMPT_STEP: &str = "key-path-spend-attempt";

/// The probe's name for the accepted control that follows it.
pub const CONTROL_STEP: &str = "script-path-control";

/// The probe's name for its issuance step.
pub const ISSUE_STEP: &str = "issue-protocol-asset";

/// The probe's name for its funding step.
pub const FUND_STEP: &str = "fund-explicit-constructor";

/// Why the ceremony stopped before it reached the target.
///
/// A first-party vocabulary, and every member is this workspace
/// declining to build something. None of them is a target verdict, and
/// the record keeps them in a different field from the observation for
/// exactly that reason.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyPathProbeRefusal {
    /// The reviewed target, the candidate ABI or the signing material
    /// was unavailable.
    SubstrateUnavailable,
    /// The issuance step named no asset.
    IssuanceNamedNoAsset,
    /// The funding step created no predecessor.
    FundingCreatedNoPredecessor,
    /// A funded output did not decode.
    MalformedFundedOutput,
    /// The deployment could not be relinked against the issued asset.
    RelinkRefused,
    /// The candidate could not be constructed over the funded coin.
    CandidateNotConstructible,
    /// The signing census refused the candidate.
    CensusRefused(OwnerCensusRefusal),
    /// The signature could not be produced.
    SigningRefused,
    /// The key-path candidate's bytes could not be assembled.
    ///
    /// The typed home of the outcome this probe was told to report
    /// rather than work around: a one-item witness that the transaction
    /// vocabulary declines to carry is a finding about the wire, and
    /// this member is where it would land.
    AttemptNotExpressible,
    /// The script-path control could not be authorized or completed.
    ///
    /// Kept apart from [`Self::CandidateNotConstructible`] because the
    /// candidate the control authorizes is the one the attempt was
    /// already built over: reaching this member means the CONTROL half
    /// failed, and a record that folded the two would leave a reader
    /// unable to see which half the ceremony lost.
    ControlNotAuthorizable,
}

/// The constructor's program, and what it is a commitment to.
///
/// Four values rather than one, because the probe's subject is the
/// relation between them: the program is the witness-version-one script
/// for an output key that is the internal key tweaked by the merkle
/// root, and a record carrying only the program would leave a reader
/// unable to see that the internal key is in it at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramBinding {
    program: Vec<u8>,
    internal_key: Vec<u8>,
    output_key: [u8; 32],
    merkle_root: Digest32,
}

impl ProgramBinding {
    /// The witness program an output carrying this constructor pays to.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }

    /// The internal key the constructor inherited.
    #[must_use]
    pub fn internal_key(&self) -> &[u8] {
        &self.internal_key
    }

    /// The tweaked output key the program carries.
    #[must_use]
    pub const fn output_key(&self) -> &[u8; 32] {
        &self.output_key
    }

    /// The taptree root the output key is tweaked by.
    #[must_use]
    pub const fn merkle_root(&self) -> &Digest32 {
        &self.merkle_root
    }

    /// Whether the internal key is the published unspendable point.
    ///
    /// Recomputed against the constant rather than assumed, because the
    /// whole probe is about that point and a constructor that had
    /// inherited some other internal key would make the run a report
    /// about a different question.
    #[must_use]
    pub fn internal_key_is_the_published_point(&self) -> bool {
        self.internal_key == UNSPENDABLE_INTERNAL_KEY
    }
}

/// Where the run's target came from.
///
/// Every member is a value the run's own environment supplied or the
/// reviewed contract fixed. The declared source tip is the operator's
/// declaration about the binary they pointed the adapter at; this probe
/// does not verify it and the field's name says so.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProbeProvenance {
    /// The bound development network, as the run's environment spelled
    /// it.
    pub network_id: String,
    /// The chain the run is bound to, as the run's environment spelled
    /// it.
    pub genesis_id: String,
    /// The reviewed contract version the binding validates against.
    pub target_version: String,
    /// The operator's declaration of the target's source tip.
    pub declared_source_tip: Option<String>,
    /// The operator's trust declaration about the adapter.
    pub executor_trust: String,
}

/// Exactly what was offered to the target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyPathAttempt {
    submitted_bytes: Vec<u8>,
    witness_stack: Vec<Vec<u8>>,
    spent_outpoint: Outpoint,
    candidate_key_path_message: Digest32,
    signing_public_key: [u8; FIELD_ELEMENT_BYTES],
}

impl KeyPathAttempt {
    /// The exact bytes handed to the submission wire.
    #[must_use]
    pub fn submitted_bytes(&self) -> &[u8] {
        &self.submitted_bytes
    }

    /// The witness stack the single input carries, bottom item first.
    #[must_use]
    pub fn witness_stack(&self) -> &[Vec<u8>] {
        &self.witness_stack
    }

    /// Whether the witness is the one-item shape a key-path spend has.
    ///
    /// A measurement rather than a construction claim: the stack is read
    /// back out of what was built, so a builder that had grown a second
    /// item is visible here rather than only in the bytes.
    #[must_use]
    pub const fn is_the_one_item_shape(&self) -> bool {
        self.witness_stack.len() == 1
    }

    /// The outpoint the attempt spends.
    #[must_use]
    pub const fn spent_outpoint(&self) -> Outpoint {
        self.spent_outpoint
    }

    /// The candidate key-path message the signature was taken over.
    ///
    /// Candidate-scoped in the strict sense. No review covers the key
    /// path and no target has been asked whether it forms this message,
    /// so the value is a datum about what was signed rather than a claim
    /// about what was verified against.
    #[must_use]
    pub const fn candidate_key_path_message(&self) -> &Digest32 {
        &self.candidate_key_path_message
    }

    /// The x-only public key of the scalar that signed.
    ///
    /// Recorded so that a reader can see for themselves that the signing
    /// key is not the output key in the program, which is the reason the
    /// refusal says nothing about the internal key.
    #[must_use]
    pub const fn signing_public_key(&self) -> &[u8; FIELD_ELEMENT_BYTES] {
        &self.signing_public_key
    }
}

/// The unmutated control the attempt is attributable against.
///
/// The same candidate, spent by its script path. What is carried is the
/// bytes' length, the measured relation to the attempt, and nothing the
/// attempt already carries: two copies of one transaction's terms would
/// be two places for them to drift.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyPathControl {
    submitted_bytes: Vec<u8>,
    witness_items: usize,
    shares_the_attempts_witnessless_bytes: bool,
}

impl KeyPathControl {
    /// The exact bytes handed to the submission wire.
    #[must_use]
    pub fn submitted_bytes(&self) -> &[u8] {
        &self.submitted_bytes
    }

    /// How many items the control's first input carries.
    ///
    /// Three for a script-path spend — signature, leaf script, control
    /// block — against the attempt's one, which is the whole visible
    /// difference between the pair.
    #[must_use]
    pub const fn witness_items(&self) -> usize {
        self.witness_items
    }

    /// Whether the pair differs in the witness ALONE.
    ///
    /// Measured by comparing the two witnessless serializations rather
    /// than asserted from the construction: a control that had been
    /// finalized over some other coin would be a second candidate wearing
    /// the control's name, and this is the fact that would say so.
    #[must_use]
    pub const fn shares_the_attempts_witnessless_bytes(&self) -> bool {
        self.shares_the_attempts_witnessless_bytes
    }
}

/// What the target did with the attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyPathObservation {
    layer: ObservedOutcomeLayer,
    accepted_txid: Option<String>,
    detail: Option<String>,
}

impl KeyPathObservation {
    /// The layer the adapter reported.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// The identity the target gave the transaction, where it took one.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// What the target or the adapter said, verbatim and unmapped.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

/// One run's record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyPathProbeRecord {
    provenance: ProbeProvenance,
    issued_asset: Option<String>,
    relinked: bool,
    coins: Vec<ObservedFundedCoin>,
    binding: Option<ProgramBinding>,
    attempt: Option<KeyPathAttempt>,
    observation: Option<KeyPathObservation>,
    control: Option<KeyPathControl>,
    control_observation: Option<KeyPathObservation>,
    refusal: Option<KeyPathProbeRefusal>,
}

impl KeyPathProbeRecord {
    /// Where the run's target came from.
    #[must_use]
    pub const fn provenance(&self) -> &ProbeProvenance {
        &self.provenance
    }

    /// The asset the target issued.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// Whether the deployment was relinked before anything was funded.
    #[must_use]
    pub const fn relinked(&self) -> bool {
        self.relinked
    }

    /// The funded coins, as the node reported them.
    #[must_use]
    pub fn coins(&self) -> &[ObservedFundedCoin] {
        &self.coins
    }

    /// The program the funded coin pays to, and what it commits to.
    #[must_use]
    pub const fn binding(&self) -> Option<&ProgramBinding> {
        self.binding.as_ref()
    }

    /// What was offered to the target.
    #[must_use]
    pub const fn attempt(&self) -> Option<&KeyPathAttempt> {
        self.attempt.as_ref()
    }

    /// What the target said.
    #[must_use]
    pub const fn observation(&self) -> Option<&KeyPathObservation> {
        self.observation.as_ref()
    }

    /// The unmutated control offered after the attempt.
    #[must_use]
    pub const fn control(&self) -> Option<&KeyPathControl> {
        self.control.as_ref()
    }

    /// What the target said about the control.
    #[must_use]
    pub const fn control_observation(&self) -> Option<&KeyPathObservation> {
        self.control_observation.as_ref()
    }

    /// Why the ceremony stopped, where it stopped.
    #[must_use]
    pub const fn refusal(&self) -> Option<&KeyPathProbeRefusal> {
        self.refusal.as_ref()
    }

    /// What this run does not establish, as data.
    ///
    /// Carried rather than commented so the committed artifact states it
    /// in its own bytes, on the pattern the observation ceremonies set.
    #[must_use]
    pub const fn non_claims() -> &'static [&'static str] {
        &[
            "a refusal discharges only that the attempt was observed and refused",
            "nothing here establishes who knows the internal key's discrete logarithm",
            "the residual discrete-log assumption on the internal key stands regardless",
            "the offered signature is by a published test key that is not the output key",
            "the key-path message construction is candidate-scoped and reviewed by nobody",
            "no blocker and no residual moves on this run",
            "the one §15 row this run answers records an observed refusal, never a discharged assumption",
        ]
    }
}

/// Which step the ceremony is at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Issue,
    Fund,
    Attempt,
    Control,
    Done,
}

/// The ceremony that funds one constructor and probes its key path.
#[derive(Debug)]
pub struct KeyPathProbePlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    genesis_block_hash: Digest32,
    record: KeyPathProbeRecord,
}

impl KeyPathProbePlanner {
    /// The ceremony, seeded with the deployment it runs against.
    ///
    /// The genesis identity arrives in the order a target prints it and
    /// is reversed into the order the message hasher is seeded with, the
    /// same relation the observation ceremonies apply. This probe offers
    /// no control for that order: the observation ceremony already
    /// measured it, and repeating a settled measurement here would be a
    /// second opinion rather than a second run.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI
    /// or the explicit destination constructor is unavailable.
    pub fn new(
        printed_genesis_identity: Digest32,
        provenance: ProbeProvenance,
    ) -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        let binding = explicit_binding(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            abi,
            genesis_block_hash: printed_order(printed_genesis_identity),
            record: KeyPathProbeRecord {
                provenance,
                binding: Some(binding),
                ..KeyPathProbeRecord::default()
            },
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &KeyPathProbeRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: KeyPathProbeRefusal) -> PlanRefused {
        self.record.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The program the funding step pays to.
    fn funded_program(&self) -> Vec<u8> {
        self.record
            .binding
            .as_ref()
            .map(|binding| binding.program.clone())
            .unwrap_or_default()
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
                output_program: self.funded_program(),
                outputs: RECEIPT_COUNT,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        )
    }

    /// Link the deployment against the asset the target issued.
    fn relink(&mut self, response: &NativeOperationResponse) -> Result<(), KeyPathProbeRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(KeyPathProbeRefusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(KeyPathProbeRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(*identity.internal(), RESERVE_ASSET, FEE_PROGRAM_DIGEST)
            .map_err(|_| KeyPathProbeRefusal::RelinkRefused)?;
        self.record.binding =
            Some(explicit_binding(&abi).map_err(|_| KeyPathProbeRefusal::RelinkRefused)?);
        self.record.issued_asset = Some(asset);
        self.record.relinked = true;
        self.abi = abi;
        Ok(())
    }

    /// Take the funded coin from the node's own report of it.
    ///
    /// The node's asset, value and program are the ones carried forward,
    /// because they are terms of the message the target forms. The
    /// ceremony's expectation is compared against them and recorded as a
    /// boolean beside them, never substituted for them.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), KeyPathProbeRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(KeyPathProbeRefusal::FundingCreatedNoPredecessor);
        }
        let expected_asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(KeyPathProbeRefusal::IssuanceNamedNoAsset)?;

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
    ) -> Result<ObservedFundedCoin, KeyPathProbeRefusal> {
        let outpoint =
            outpoint_of(&funded.outpoint).ok_or(KeyPathProbeRefusal::MalformedFundedOutput)?;
        let asset = asset_of(&funded.asset).ok_or(KeyPathProbeRefusal::MalformedFundedOutput)?;
        let program =
            decode_hex(&funded.script).ok_or(KeyPathProbeRefusal::MalformedFundedOutput)?;

        let matches_expectation = asset == expected_asset
            && funded.amount_satoshis == RECEIPT_AMOUNT
            && program == self.funded_program();

        Ok(ObservedFundedCoin::observed(
            outpoint,
            AssetField::Explicit(asset),
            ValueField::Explicit(funded.amount_satoshis),
            program,
            matches_expectation,
        ))
    }

    /// One finalized explicit candidate over the funded coin.
    ///
    /// The construction report travels with it because the control's
    /// completion needs it; the attempt does not, and dropping it there
    /// is what made phase A's finalization return one value.
    fn finalize(
        &self,
    ) -> Result<(FinalizedLiveTransfer, LiveConstructionReport), KeyPathProbeRefusal> {
        let view = PublicConstructionView::new(self.record.coins.iter().map(|coin| {
            PublicOutputView::new(
                coin.outpoint(),
                coin.asset(),
                coin.value(),
                coin.program().to_vec(),
            )
        }))
        .map_err(|_| KeyPathProbeRefusal::CandidateNotConstructible)?;

        let points: Vec<Outpoint> = self
            .record
            .coins
            .iter()
            .map(ObservedFundedCoin::outpoint)
            .collect();
        let total = RECEIPT_AMOUNT
            .checked_mul(u64::try_from(points.len()).unwrap_or(0))
            .ok_or(KeyPathProbeRefusal::CandidateNotConstructible)?;

        let destination = |scalar: &[u8; FIELD_ELEMENT_BYTES], amount: u64| {
            let owner =
                published_owner(scalar).map_err(|_| KeyPathProbeRefusal::SubstrateUnavailable)?;
            let value = ProtocolValue::new(amount)
                .map_err(|_| KeyPathProbeRefusal::CandidateNotConstructible)?;
            Ok::<_, KeyPathProbeRefusal>(LiveReceiptDestination::new(
                linker::OwnerParameter::new(owner),
                value,
            ))
        };
        let destinations = [
            destination(&SECOND_SCALAR, total / 2)?,
            destination(&FIRST_SCALAR, total - total / 2)?,
        ];

        let request = LiveTransferRequest::new(
            points,
            destinations,
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        )
        .map_err(|_| KeyPathProbeRefusal::CandidateNotConstructible)?;

        let target = reviewed_target().map_err(|_| KeyPathProbeRefusal::SubstrateUnavailable)?;
        let finalization = finalize_live_transfer(&target, &self.abi, &request, &view, None, None)
            .map_err(|_| KeyPathProbeRefusal::CandidateNotConstructible)?;
        let report = finalization.report().clone();
        Ok((finalization.into_finalized(), report))
    }

    /// The census of one finalized explicit candidate.
    ///
    /// Built from the script-path signing requests the finalization
    /// produced, which is not a contradiction and is worth one sentence:
    /// the census's job here is the candidate, the spent-output triple
    /// and the deployment seed, all of which a key-path message needs
    /// and none of which depends on the path. The leaf data it also
    /// carries is exactly what the key-path construction declines to
    /// write.
    fn census(
        finalized: &FinalizedLiveTransfer,
        genesis: Digest32,
    ) -> Result<OwnerSigningCensus, KeyPathProbeRefusal> {
        let target = reviewed_target().map_err(|_| KeyPathProbeRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| KeyPathProbeRefusal::SubstrateUnavailable)?,
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
        .map_err(KeyPathProbeRefusal::CensusRefused)
    }

    /// The pair: one candidate, offered twice, differing in the witness.
    ///
    /// ONE finalization answers both halves, which is the point rather
    /// than an economy. The attempt is the census's own candidate wearing
    /// a one-item witness; the control is the same candidate authorized
    /// the way this workspace authorizes every transfer. A second
    /// finalization for the control would have been a second candidate a
    /// reader had to take on trust as the first one's twin.
    ///
    /// The attempt's bytes are built exactly as phase A built them, and
    /// the control is added beside them.
    fn build_pair(&self) -> Result<(KeyPathAttempt, KeyPathControl), KeyPathProbeRefusal> {
        let (finalized, report) = self.finalize()?;
        let attempt = self.build_attempt(&finalized)?;
        let control = self.build_control(finalized, report, &attempt)?;
        Ok((attempt, control))
    }

    /// The attempt: one candidate, one input, one witness item.
    ///
    /// The transaction is rebuilt from the census's own candidate rather
    /// than from anything the ceremony still holds, so the bytes
    /// submitted are the bytes the message was formed over, differing in
    /// the witness alone.
    fn build_attempt(
        &self,
        finalized: &FinalizedLiveTransfer,
    ) -> Result<KeyPathAttempt, KeyPathProbeRefusal> {
        let census = Self::census(finalized, self.genesis_block_hash)?;
        let input = census
            .signing_inputs()
            .first()
            .ok_or(KeyPathProbeRefusal::CandidateNotConstructible)?;

        let message = candidate_key_path_message(&census, input, WitnessVectorTreatment::BothGrown);
        let material = signing_material(&FIRST_SCALAR)
            .map_err(|_| KeyPathProbeRefusal::SubstrateUnavailable)?;
        let signature = material
            .sign(&message, &PROBE_AUXILIARY)
            .map_err(|_| KeyPathProbeRefusal::SigningRefused)?
            .to_vec();

        // The one-item witness. A key-path spend supplies the signature
        // and nothing else: no leaf script and no control block, which
        // is the whole visible difference between this candidate and
        // every other one this workspace submits.
        let stack = vec![signature];
        let candidate = census.candidate();
        let witnesses = vec![InputWitness::new(stack.clone()); candidate.inputs().len()];
        let attempted = TargetTransaction::with_output_witnesses(
            candidate.version(),
            candidate.inputs().to_vec(),
            candidate.outputs().to_vec(),
            candidate.lock_time(),
            witnesses,
            candidate.output_witnesses().to_vec(),
        )
        .map_err(|_| KeyPathProbeRefusal::AttemptNotExpressible)?;

        let spent_outpoint = candidate
            .inputs()
            .first()
            .ok_or(KeyPathProbeRefusal::CandidateNotConstructible)?
            .outpoint();

        Ok(KeyPathAttempt {
            submitted_bytes: attempted.encode(),
            witness_stack: stack,
            spent_outpoint,
            candidate_key_path_message: message,
            signing_public_key: material.x_only_public_key(),
        })
    }

    /// The control: the same candidate, spent by its script path.
    ///
    /// Authorized through the ordinary owner route — one signature per
    /// receipt over the message the census forms — and completed the way
    /// every accepted transfer in this package is completed. Nothing
    /// here is probe-specific except the auxiliary value, which is the
    /// probe's own so that both halves of the pair are reproducible from
    /// constants that are written down.
    fn build_control(
        &self,
        finalized: FinalizedLiveTransfer,
        report: LiveConstructionReport,
        attempt: &KeyPathAttempt,
    ) -> Result<KeyPathControl, KeyPathProbeRefusal> {
        let census = Self::census(&finalized, self.genesis_block_hash)?;
        let material = signing_material(&FIRST_SCALAR)
            .map_err(|_| KeyPathProbeRefusal::SubstrateUnavailable)?;

        let mut responses = Vec::new();
        for signing in finalized.signing_requests() {
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(KeyPathProbeRefusal::ControlNotAuthorizable)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            let signature = material
                .sign(&message, &PROBE_AUXILIARY)
                .map_err(|_| KeyPathProbeRefusal::SigningRefused)?
                .to_vec();
            responses.push((signing.input(), LiveOwnerResponse::to(&signing, signature)));
        }

        let target = reviewed_target().map_err(|_| KeyPathProbeRefusal::SubstrateUnavailable)?;
        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| KeyPathProbeRefusal::ControlNotAuthorizable)?;
        let built = complete_live_transfer(&target, authorized, report, None)
            .map_err(|_| KeyPathProbeRefusal::ControlNotAuthorizable)?;
        let bytes = built.bytes();

        // The pair's relation, measured out of the two encodings rather
        // than taken from the fact that one finalization produced both.
        let decoded = TargetTransaction::decode(&bytes)
            .map_err(|_| KeyPathProbeRefusal::ControlNotAuthorizable)?;
        let attempted = TargetTransaction::decode(attempt.submitted_bytes())
            .map_err(|_| KeyPathProbeRefusal::ControlNotAuthorizable)?;
        let witness_items = decoded
            .witnesses()
            .first()
            .map_or(0, |witness| witness.stack().len());

        Ok(KeyPathControl {
            shares_the_attempts_witnessless_bytes: decoded.encode_without_witness()
                == attempted.encode_without_witness(),
            witness_items,
            submitted_bytes: bytes,
        })
    }

    /// Record what the target did with the attempt.
    ///
    /// Whatever it was. Nothing here compares the layer against what the
    /// probe expected: the probe has an expectation and an expectation is
    /// not an entitlement, so an acceptance would be a finding rendered
    /// into the artifact rather than a panic that hid it.
    fn settle_attempt(&mut self, response: &NativeOperationResponse) {
        self.record.observation = Some(KeyPathObservation {
            layer: response.observed_layer,
            accepted_txid: response.accepted_txid.clone(),
            detail: response.observed_detail.clone(),
        });
    }

    /// Record what the target did with the control.
    ///
    /// On the same rule the attempt is recorded under: the control has an
    /// intention and an intention is not an entitlement, so a control the
    /// node refused is written into the artifact rather than panicked on.
    /// A refused control does not make the attempt's refusal evidence,
    /// and the register is what would say so.
    fn settle_control(&mut self, response: &NativeOperationResponse) {
        self.record.control_observation = Some(KeyPathObservation {
            layer: response.observed_layer,
            accepted_txid: response.accepted_txid.clone(),
            detail: response.observed_detail.clone(),
        });
    }
}

impl TargetOperationPlanner for KeyPathProbePlanner {
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
                    self.stage = Stage::Attempt;
                }
                Stage::Attempt => {
                    self.settle_attempt(response);
                    self.stage = Stage::Control;
                }
                Stage::Control => {
                    self.settle_control(response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(self.funding_step(ISSUE_STEP, true))),
            Stage::Fund => Ok(Some(self.funding_step(FUND_STEP, false))),
            // Both halves are built here, and the control is held until
            // its own step. Building the pair at the attempt's step is
            // what makes the two one candidate: a control built later
            // would be finalized over a coin the attempt had already been
            // offered against, and nothing in the record would show the
            // two came from one construction.
            Stage::Attempt => match self.build_pair() {
                Ok((attempt, control)) => {
                    let bytes = attempt.submitted_bytes.clone();
                    self.record.attempt = Some(attempt);
                    self.record.control = Some(control);
                    Ok(Some(OperationStep::new(
                        ATTEMPT_STEP,
                        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                            transaction_bytes: bytes,
                        })),
                    )))
                }
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Control => {
                let held = self
                    .record
                    .control
                    .as_ref()
                    .map(|control| control.submitted_bytes.clone());
                let Some(bytes) = held else {
                    return Err(self.refuse(KeyPathProbeRefusal::ControlNotAuthorizable));
                };
                Ok(Some(OperationStep::new(
                    CONTROL_STEP,
                    OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                        transaction_bytes: bytes,
                    })),
                )))
            }
            Stage::Done => Ok(None),
        }
    }
}

/// The explicit destination constructor's program and its commitments.
fn explicit_binding(abi: &CandidateLiveTransferAbi) -> Result<ProgramBinding, VectorError> {
    let instance = abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(published_owner(&FIRST_SCALAR)?),
            LiveTransferRepresentationPlan::Explicit,
        )
        .ok_or(VectorError::LiveSubstrateUnavailable)?
        .instance();

    Ok(ProgramBinding {
        program: instance.program().to_vec(),
        internal_key: instance.internal_key().to_vec(),
        output_key: *instance.output_key().key(),
        merkle_root: instance.tree().merkle_root(),
    })
}

/// One run's transcript, as the report artifact carries it.
///
/// Lines rather than a structure, on the pattern the other native lanes
/// set. Every line is a fact the run observed or a value it computed,
/// and no line is a verdict about whether the run went well.
#[must_use]
pub fn render_keypath_probe(record: &KeyPathProbeRecord) -> String {
    let mut lines = vec!["role internal-key-unspendability-probe-phase-b".to_owned()];

    lines.extend(provenance_lines(record.provenance()));
    lines.push(format!(
        "issued_asset {}",
        record.issued_asset().unwrap_or("none")
    ));
    lines.push(format!("relinked {}", record.relinked()));
    lines.extend(coin_lines(record));
    if let Some(binding) = record.binding() {
        lines.extend(binding_lines(binding));
    }
    if let Some(attempt) = record.attempt() {
        lines.extend(attempt_lines(record, attempt));
    }
    if let Some(observation) = record.observation() {
        lines.extend(observation_lines("observed", observation));
    }
    if let Some(control) = record.control() {
        lines.extend(control_lines(control));
    }
    if let Some(observation) = record.control_observation() {
        lines.extend(observation_lines("control_observed", observation));
    }
    if let Some(refusal) = record.refusal() {
        lines.push(format!("ceremony_refusal {refusal:?}"));
    }

    for claim in KeyPathProbeRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }
    lines.push("residual_internal_key_unspendability_stands true".to_owned());
    lines.push("discharges_no_residual true".to_owned());
    lines.push("answers_matrix_row key-path-escape".to_owned());

    let mut rendered = lines.join("\n");
    let _ = writeln!(rendered);
    rendered
}

/// Where the run's target came from.
fn provenance_lines(provenance: &ProbeProvenance) -> Vec<String> {
    let mut lines = Vec::with_capacity(5);
    lines.push(format!("provenance network_id {}", provenance.network_id));
    lines.push(format!("provenance genesis_id {}", provenance.genesis_id));
    lines.push(format!(
        "provenance target_version {}",
        provenance.target_version
    ));
    lines.push(format!(
        "provenance declared_source_tip {}",
        provenance.declared_source_tip.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "provenance executor_trust {}",
        provenance.executor_trust
    ));
    lines
}

/// Each funded coin, as the node reported it.
fn coin_lines(record: &KeyPathProbeRecord) -> Vec<String> {
    let mut lines = Vec::with_capacity(record.coins().len());
    for (index, coin) in record.coins().iter().enumerate() {
        lines.push(format!(
            "coin {index} value {} program_bytes {} node_fields_match_expectation {}",
            if let ValueField::Explicit(amount) = coin.value() {
                amount.to_string()
            } else {
                "not-explicit".to_owned()
            },
            coin.program().len(),
            coin.matches_expectation(),
        ));
    }
    lines
}

/// The program the funded coin pays to, and what it commits to.
fn binding_lines(binding: &ProgramBinding) -> Vec<String> {
    let mut lines = Vec::with_capacity(5);
    lines.push(format!("binding program {}", printed(binding.program())));
    lines.push(format!(
        "binding internal_key {}",
        printed(binding.internal_key())
    ));
    lines.push(format!(
        "binding internal_key_is_the_published_point {}",
        binding.internal_key_is_the_published_point()
    ));
    lines.push(format!(
        "binding output_key {}",
        printed(binding.output_key())
    ));
    lines.push(format!(
        "binding merkle_root {}",
        printed(binding.merkle_root())
    ));
    lines
}

/// Exactly what was offered to the target.
fn attempt_lines(record: &KeyPathProbeRecord, attempt: &KeyPathAttempt) -> Vec<String> {
    let mut lines = Vec::with_capacity(8 + attempt.witness_stack().len());
    lines.push(format!(
        "attempt submitted_bytes {}",
        attempt.submitted_bytes().len()
    ));
    lines.push(format!(
        "attempt submitted {}",
        printed(attempt.submitted_bytes())
    ));
    lines.push(format!(
        "attempt witness_items {}",
        attempt.witness_stack().len()
    ));
    for (index, item) in attempt.witness_stack().iter().enumerate() {
        lines.push(format!("attempt witness_item {index} {}", printed(item)));
    }
    lines.push(format!(
        "attempt spent_outpoint_index {}",
        attempt.spent_outpoint().index()
    ));
    lines.push(format!(
        "attempt candidate_key_path_message {}",
        printed(attempt.candidate_key_path_message())
    ));
    lines.push(format!(
        "attempt signing_public_key {}",
        printed(attempt.signing_public_key())
    ));
    lines.push(format!(
        "attempt signing_key_is_the_output_key {}",
        record
            .binding()
            .is_some_and(|binding| binding.output_key() == attempt.signing_public_key()),
    ));
    lines
}

/// The unmutated control offered after the attempt.
fn control_lines(control: &KeyPathControl) -> Vec<String> {
    let mut lines = Vec::with_capacity(4);
    lines.push(format!(
        "control submitted_bytes {}",
        control.submitted_bytes().len()
    ));
    lines.push(format!(
        "control submitted {}",
        printed(control.submitted_bytes())
    ));
    lines.push(format!("control witness_items {}", control.witness_items()));
    lines.push(format!(
        "control shares_the_attempts_witnessless_bytes {}",
        control.shares_the_attempts_witnessless_bytes()
    ));
    lines
}

/// What the target did with one submission.
///
/// The prefix names WHICH submission, so the attempt's verdict and the
/// control's cannot be read for each other in a transcript that carries
/// both.
fn observation_lines(prefix: &str, observation: &KeyPathObservation) -> Vec<String> {
    let mut lines = Vec::with_capacity(4);
    lines.push(format!("{prefix} layer {:?}", observation.layer()));
    lines.push(format!(
        "{prefix} is_target_verdict {}",
        observation.layer().is_target_verdict()
    ));
    lines.push(format!(
        "{prefix} accepted_txid {}",
        observation.accepted_txid().unwrap_or("none")
    ));
    lines.push(format!(
        "{prefix} detail {}",
        observation.detail().unwrap_or("none")
    ));
    lines
}

/// The run of record: what one execution of this probe observed.
///
/// # Why the observation is a constant and not a stored file
///
/// The evidence a run produces is the observation, and an observation
/// nobody can name is not evidence. These are the figures ONE run
/// against a real node produced, written down so that a claim made
/// anywhere in this workspace about the key path can be traced to bytes
/// and to a verdict rather than to a test having been written.
///
/// It re-runs nothing and proves nothing by existing.
///
/// # There is no accepted identity here, and there should not be
///
/// The other ceremonies' runs of record are anchored on the identity
/// the target gave an accepted transaction. This one has none, because
/// the target accepted nothing, and minting an observation identity for
/// a refusal would file a non-acceptance in the vocabulary acceptances
/// are cited from. What anchors this run instead is the candidate's own
/// bytes, which are reproducible from inputs that are all written down,
/// and the verdict recorded verbatim beside them.
///
/// The target: Elements Core v28.99.0-b7fc5d080a7e, at the pinned tip
/// the lane binds itself to, on a disposable development chain the run
/// created and destroyed. The outpoint is omitted, as the other runs of
/// record omit theirs: it names a chain that no longer exists.
pub mod run_of_record {
    /// The disposable asset the run issued.
    pub const ISSUED_ASSET: &str =
        "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

    /// The witness program the funded coin paid to.
    pub const FUNDED_PROGRAM: &str =
        "51208d696527d4d1517c67ba5e72b9c5b9c730cb463f20d59b84d7e813d9a188ef69";

    /// The tweaked output key that program carries.
    pub const OUTPUT_KEY: &str = "8d696527d4d1517c67ba5e72b9c5b9c730cb463f20d59b84d7e813d9a188ef69";

    /// The taptree root the output key is tweaked by.
    pub const MERKLE_ROOT: &str =
        "70c084e34b9e1f63b6806d999d0a2df507493b67ef1a3d8711f062d678cf6e45";

    /// The x-only public key of the scalar that signed the attempt.
    ///
    /// The first published owner's. Not [`OUTPUT_KEY`], which is the
    /// whole reason the refusal below establishes nothing about the
    /// internal key.
    pub const SIGNING_PUBLIC_KEY: &str =
        "dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659";

    /// The candidate key-path message the signature was taken over.
    ///
    /// Candidate-scoped. Reviewed by nobody, and observed to be the
    /// target's message by nothing.
    pub const CANDIDATE_KEY_PATH_MESSAGE: &str =
        "d165412ea39cc0ee067af1313c04bacdaff2f9add0db6ae895da207ab0943c05";

    /// How many bytes were handed to the submission wire.
    pub const SUBMITTED_BYTES: usize = 281;

    /// How many items the single input's witness carried.
    pub const WITNESS_ITEMS: usize = 1;

    /// How wide that one item was.
    pub const WITNESS_ITEM_BYTES: usize = 64;

    /// What the target said, verbatim and unmapped.
    pub const OBSERVED_DETAIL: &str =
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";

    /// The layer the adapter filed the verdict under.
    ///
    /// Recorded as the string the run produced rather than as the enum,
    /// because the name is itself the finding: the observed-layer
    /// vocabulary has no key-path member, so a key-path refusal is filed
    /// under a script-path name. The adapter classifies on the refusal
    /// text's prefix and cannot do otherwise with the vocabulary it has.
    /// Repairing that is the typed carrier the follow-up phase owns; the
    /// probe reports it and changes nothing.
    ///
    /// The repair LANDED, and this constant does not move for it. What
    /// phase A observed is what phase A observed, and rewriting it to the
    /// name a later vocabulary would have given it would replace a record
    /// of a run with a reconstruction of one. The corrected observation
    /// is phase B's own, beside this module in
    /// [`super::run_of_record_phase_b`].
    pub const OBSERVED_LAYER: &str = "ScriptPathRejection";

    /// The run's wall time.
    pub const WALL_SECONDS: f64 = 4.1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_probe_funds_exactly_one_receipt() {
        // The one-item witness is a claim about one input, and one input
        // is what one receipt buys. A second receipt would make the
        // record's central phrase a description of two witnesses.
        assert_eq!(RECEIPT_COUNT, 1);
    }

    #[test]
    fn the_non_claims_carry_the_residual_and_the_key() {
        // The two sentences the probe exists to keep attached to its own
        // result, asserted as data rather than trusted to a comment: a
        // record that had dropped either would be a record a reader
        // could take for a discharge.
        let claims = KeyPathProbeRecord::non_claims();
        assert!(
            claims
                .iter()
                .any(|claim| claim.contains("discrete-log assumption") && claim.contains("stands")),
        );
        assert!(
            claims
                .iter()
                .any(|claim| claim.contains("not the output key")),
        );
    }

    #[test]
    fn the_rendering_states_the_residual_and_discharges_nothing() {
        // Over an empty record, deliberately: the non-claims are
        // properties of the run's kind rather than of what it observed,
        // so a run that reached nothing must still say them.
        let rendered = render_keypath_probe(&KeyPathProbeRecord::default());

        assert!(rendered.contains("residual_internal_key_unspendability_stands true"));
        // The line phase A carried said no matrix row moved, which was
        // true of phase A. Phase B answers one row and still discharges
        // no residual, and the transcript says both rather than letting
        // the second be read out of the first.
        assert!(rendered.contains("discharges_no_residual true"));
        assert!(rendered.contains("answers_matrix_row key-path-escape"));
        assert!(rendered.ends_with('\n'));
    }

    #[test]
    fn the_run_of_record_names_one_refused_attempt_at_one_witness_item() {
        // The figures are the run's, and this checks their SHAPE rather
        // than re-deriving them. The two that carry the claim are the
        // witness census and the key disagreement: a run of record whose
        // witness had grown a second item would not be a key-path
        // attempt, and one whose signing key equalled the output key
        // would be a different experiment reported under this name.
        use super::run_of_record as run;

        assert_eq!(run::WITNESS_ITEMS, 1);
        assert_eq!(run::WITNESS_ITEM_BYTES, 64);
        assert_ne!(run::SIGNING_PUBLIC_KEY, run::OUTPUT_KEY);

        // The program is the witness-version-one script for the output
        // key: two prefix bytes and the key. Checked by construction so
        // that a transcription slip in either constant is a failure
        // rather than a pair of numbers nobody compared.
        assert_eq!(run::FUNDED_PROGRAM, format!("5120{}", run::OUTPUT_KEY));

        assert_eq!(run::OUTPUT_KEY.len(), 64);
        assert_eq!(run::MERKLE_ROOT.len(), 64);
        assert_eq!(run::ISSUED_ASSET.len(), 64);
        assert_eq!(run::CANDIDATE_KEY_PATH_MESSAGE.len(), 64);

        // The submitted bytes carry a transaction and not only the
        // witness item.
        const { assert!(run::SUBMITTED_BYTES > run::WITNESS_ITEM_BYTES) };

        // The verdict was a refusal, and the artifact says so in the
        // target's own words rather than in a mapped name.
        assert!(run::OBSERVED_DETAIL.contains("Invalid Schnorr signature"));
    }

    #[test]
    fn the_published_internal_key_is_the_one_the_probe_is_about() {
        // The constructor's own binding, recomputed rather than assumed.
        // A deployment whose constructor had inherited some other
        // internal key would make this run a report about a different
        // question, and the record's own predicate is what would say so.
        let abi = demonstration_live_abi().expect("the demonstration ABI links");
        let binding = explicit_binding(&abi).expect("the explicit constructor is present");

        assert!(binding.internal_key_is_the_published_point());
        assert_ne!(binding.output_key().as_slice(), binding.internal_key());
    }
}
