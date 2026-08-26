//! The owner-signing negative ceremony: a script-path §15 negative signed
//! over its own mutated bytes, so its refusal is the leaf's own clause
//! rather than the signature gate.
//!
//! # What this ceremony is, and why the route it runs on exists
//!
//! Every §15 row the negative-half register carries under
//! [`crate::live_negative_half::NegativeHalfGap::OwnerSigningOverForeignBytesAbsent`]
//! declares a SCRIPT-PATH refusal. Reaching one needs the target to RUN
//! the leaf; the leaf checks an owner signature first; and byte surgery
//! after signing invalidates that signature — so a mutant carrying a stale
//! signature dies at the signature gate and the row's own class is never
//! what refused. This ceremony breaks that deadlock: it does the surgery,
//! then builds a census over the MUTATED bytes through
//! [`transaction::live_census::OwnerSigningCensus::over_foreign_bytes_for_negative_evidence`],
//! forms the owner message from it, and re-signs afresh. The re-signed
//! mutant passes `OP_CHECKSIGVERIFY` and reaches the leaf's own clause.
//!
//! # The one row this ceremony drives, and why only it
//!
//! `vault-control-entitlement-or-bare-u-output`. The coordinator leaf's
//! `InspectOutputScriptPubKey` clause constrains the WITNESS VERSION of
//! every destination output — and only the version, the 32-byte payload
//! being dropped. Mutating one destination's program to a different-version
//! ("bare-u") program, while keeping its asset and its explicit value,
//! leaves the per-asset consensus tally intact, so the mutant passes
//! `bad-txns-in-ne-out` and reaches the version clause, which refuses it.
//!
//! That is the CLEAN shape the governing per-row fact names: a re-signed
//! mutant authorizes its own outputs, so a row is drivable to its
//! script-path class only where a covenant clause constrains the mutated
//! field INDEPENDENT of the signature AND the consensus balance rule does
//! not fire first. An asset or a single-output value surgery breaks the
//! per-asset sum and is refused `bad-txns-in-ne-out` before the leaf runs;
//! that refusal is a consensus verdict and not the row's script class, so
//! those rows are not driven here.
//!
//! # Attribution is by mutated field, against a control on the same chain
//!
//! The mutant is offered FIRST and the unmutated control LAST, to one node
//! on one chain — a refused mutant spends nothing, so the control's coins
//! stay unspent for its acceptance. The two candidates differ in exactly
//! one field of the witnessless serialization: the mutated destination's
//! program. Every other output field is held fixed, so the `EQUALVERIFY`
//! the leaf fails is the one the destination's scriptPubKey-version clause
//! carries and no other. The signatures differ too, and by design: the
//! whole point of the route is that the mutant is re-signed over its own
//! mutated bytes. The declared field range is therefore measured over the
//! WITNESSLESS serialization, where the re-signing does not reach.
//!
//! # Every key here is published test material
//!
//! The signing scalars are the BIP-340 appendix secret keys, admitted
//! under ADR-015's test-material rule `(´[ADR015-rule:security:test-material]´)`.
//! They authorize nothing on any network anyone uses, the chain this
//! ceremony runs against is created and destroyed by the run, and nothing
//! here is custody of anything.

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::LeafVersion;
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    FundedOutput, MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::{
    AssetField, AssetId, InputWitness, Outpoint, TargetOutput, TargetTransaction, ValueField,
};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{
    AnnexDisposition, IssuanceDisposition, LiveDeployment, OWNER_CODESEPARATOR_POSITION,
    OwnerCensusRefusal, OwnerSigningCensus, OwnerSigningInputRequest,
};
use transaction::live_construct::finalize_live_transfer;
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::taproot::{Digest32, leaf_hash};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{
    ObservedFundedCoin, asset_of, decode_hex, outpoint_of, printed,
};
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, RESERVE_ASSET, SECOND_SCALAR, demonstration_live_abi,
    live_abi_for_asset, published_owner, reviewed_target, signing_material,
};

/// What each funded receipt holds.
///
/// Equal to the owner-observation ceremony's figure deliberately: the
/// control this ceremony accepts is the same candidate that ceremony
/// accepts, and a different amount would make the two runs incomparable.
const RECEIPT_AMOUNT: u64 = 5_000;

/// How many receipts the ceremony funds and then consumes.
const RECEIPT_COUNT: u8 = 2;

/// The auxiliary value every signature here is taken with.
///
/// A published constant, not randomness: BIP-340 masks the scalar with it
/// before deriving the nonce, so fixing it makes every signature
/// reproducible from written-down inputs.
const SIGNING_AUXILIARY: [u8; FIELD_ELEMENT_BYTES] = [0x33; FIELD_ELEMENT_BYTES];

/// The output the mutant rewrites the program of.
///
/// The first destination, which the coordinator leaf at input zero
/// inspects along with every other destination. Any destination would
/// serve; the point is that ONE output's program is rewritten and the
/// asset and the value are left alone.
const MUTATED_OUTPUT: usize = 0;

/// A different-version program the mutant pays the first destination to.
///
/// A version-zero witness program (the P2WPKH shape) where the receipt
/// constructor's is version one. `InspectOutputScriptPubKey` pushes the
/// version, the coordinator leaf checks it against the destination version
/// one, and a version-zero program fails that `EQUALVERIFY` — which is the
/// "bare-u output" the row names. It is a standard output, so the node
/// reaches the leaf rather than refusing it as nonstandard, and it carries
/// the SAME asset and the SAME explicit value, so the per-asset tally is
/// undisturbed.
const BARE_U_PROGRAM: [u8; 22] = [
    0x00, 0x14, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
];

/// The ceremony's own name for the mutant submission.
pub const MUTANT_STEP: &str = "bare-u-output-mutant";

/// The ceremony's own name for the control submission.
pub const CONTROL_STEP: &str = "vault-control-entitlement-control";

/// What this ceremony refuses, before any node is asked.
///
/// Every member is a construction or infrastructure fact and none is a
/// target verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerSigningNegativeRefusal {
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
    /// The census refused the finalized candidate or its mutant.
    CensusRefused(OwnerCensusRefusal),
    /// The signing material refused to produce a signature.
    SigningRefused,
    /// The mutated program disturbed a field outside the declared one.
    MutationNotConfined {
        /// The half-open witnessless byte range the diff actually touched.
        touched: (usize, usize),
        /// The half-open range the mutation declared it would touch.
        declared: (usize, usize),
    },
}

/// The two-origin check on the accepted control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlReverification {
    readback_matches_submission: bool,
}

impl ControlReverification {
    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }
}

/// The mutant, as this ceremony submitted and observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutantObservation {
    declared_field_range: (usize, usize),
    submitted_bytes: usize,
    message: Digest32,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl MutantObservation {
    /// The half-open witnessless byte range the mutation stayed within:
    /// the mutated destination's program.
    #[must_use]
    pub const fn declared_field_range(&self) -> (usize, usize) {
        self.declared_field_range
    }

    /// How many bytes the mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The message the mutant's signature was taken over.
    #[must_use]
    pub const fn message(&self) -> &Digest32 {
        &self.message
    }

    /// The layer the target refused the mutant at, where it was observed.
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlObservation {
    submitted_bytes: usize,
    message: Digest32,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    reverification: Option<ControlReverification>,
}

impl ControlObservation {
    /// How many bytes the control handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The message the control's signature was taken over.
    #[must_use]
    pub const fn message(&self) -> &Digest32 {
        &self.message
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

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The two-origin check, where an acceptance was observed.
    #[must_use]
    pub const fn reverification(&self) -> Option<&ControlReverification> {
        self.reverification.as_ref()
    }
}

/// Everything the ceremony recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnerSigningNegativeRecord {
    issued_asset: Option<String>,
    relinked: bool,
    coins: Vec<ObservedFundedCoin>,
    mutant: Option<MutantObservation>,
    control: Option<ControlObservation>,
    refusal: Option<OwnerSigningNegativeRefusal>,
}

impl OwnerSigningNegativeRecord {
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

    /// The mutant's observation, where it was built and submitted.
    #[must_use]
    pub const fn mutant(&self) -> Option<&MutantObservation> {
        self.mutant.as_ref()
    }

    /// The control's observation, where it was built and submitted.
    #[must_use]
    pub const fn control(&self) -> Option<&ControlObservation> {
        self.control.as_ref()
    }

    /// Why the ceremony stopped, where it did.
    #[must_use]
    pub const fn refusal(&self) -> Option<&OwnerSigningNegativeRefusal> {
        self.refusal.as_ref()
    }

    /// What this ceremony does not claim, whatever it observed.
    #[must_use]
    pub fn non_claims() -> Vec<&'static str> {
        vec![
            "establishes nothing about the proof-bearing lane: this mutant is an explicit \
             candidate and the covenant it reaches is the explicit destination's",
            "discharges no row but its own: the mutant drives \
             vault-control-entitlement-or-bare-u-output and no other, each other owner-signing \
             row needing its own surgery",
            "attributes no consensus refusal to a script clause: a mutant refused \
             bad-txns-in-ne-out reached no leaf and is not recorded as a script-path verdict",
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
    /// Submit the mutant, first, so the control's coins stay unspent.
    Mutant,
    /// Submit the unmutated control, last, which is what consumes them.
    Control,
    /// Nothing further.
    Done,
}

/// One submitted candidate, held until its answer arrives.
struct PendingSubmission {
    bytes: Vec<u8>,
}

/// The owner-signing negative ceremony.
pub struct OwnerSigningNegativePlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    explicit_program: Vec<u8>,
    genesis_block_hash: Digest32,
    pending: Option<PendingSubmission>,
    record: OwnerSigningNegativeRecord,
}

impl OwnerSigningNegativePlanner {
    /// The ceremony bound to one deployment's genesis block hash.
    ///
    /// The genesis hash is a constructor argument rather than something a
    /// candidate carries: the target seeds its message hasher with it
    /// twice, so two identical candidates on two chains have different
    /// messages, and it arrives from the run's own deployment binding in
    /// the same PRINTED order [`crate::live_owner_observation`] reverses to
    /// the seed.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI or
    /// the explicit destination constructor is unavailable.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        let explicit_program = explicit_destination_program(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            abi,
            explicit_program,
            genesis_block_hash: crate::live_owner_observation::printed_order(
                printed_genesis_identity,
            ),
            pending: None,
            record: OwnerSigningNegativeRecord::default(),
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &OwnerSigningNegativeRecord {
        &self.record
    }

    /// Record one refusal and stop.
    fn refuse(&mut self, refusal: OwnerSigningNegativeRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
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
    ) -> Result<(), OwnerSigningNegativeRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(OwnerSigningNegativeRefusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(OwnerSigningNegativeRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(*identity.internal(), RESERVE_ASSET, FEE_PROGRAM_DIGEST)
            .map_err(|_| OwnerSigningNegativeRefusal::RelinkRefused)?;
        self.explicit_program = explicit_destination_program(&abi)
            .map_err(|_| OwnerSigningNegativeRefusal::RelinkRefused)?;
        self.record.issued_asset = Some(asset);
        self.record.relinked = true;
        self.abi = abi;
        Ok(())
    }

    /// Take the funded coins from the node's own report of them.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), OwnerSigningNegativeRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(OwnerSigningNegativeRefusal::FundingCreatedNoPredecessor);
        }
        let expected_asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(OwnerSigningNegativeRefusal::IssuanceNamedNoAsset)?;

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
    ) -> Result<ObservedFundedCoin, OwnerSigningNegativeRefusal> {
        let outpoint = outpoint_of(&funded.outpoint)
            .ok_or(OwnerSigningNegativeRefusal::MalformedFundedOutput)?;
        let asset =
            asset_of(&funded.asset).ok_or(OwnerSigningNegativeRefusal::MalformedFundedOutput)?;
        let program =
            decode_hex(&funded.script).ok_or(OwnerSigningNegativeRefusal::MalformedFundedOutput)?;

        let matches_expectation = asset == expected_asset
            && funded.amount_satoshis == RECEIPT_AMOUNT
            && program == self.explicit_program;

        Ok(ObservedFundedCoin::observed(
            outpoint,
            AssetField::Explicit(asset),
            ValueField::Explicit(funded.amount_satoshis),
            program,
            matches_expectation,
        ))
    }

    /// The public view the ceremony constructs against.
    fn view(&self) -> Result<PublicConstructionView, OwnerSigningNegativeRefusal> {
        PublicConstructionView::new(self.record.coins.iter().map(|coin| {
            PublicOutputView::new(
                coin.outpoint(),
                coin.asset(),
                coin.value(),
                coin.program().to_vec(),
            )
        }))
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)
    }

    /// One finalized explicit candidate over the funded coins.
    fn finalize(&self) -> Result<FinalizedLiveTransfer, OwnerSigningNegativeRefusal> {
        let view = self.view()?;
        let points: Vec<Outpoint> = self
            .record
            .coins
            .iter()
            .map(ObservedFundedCoin::outpoint)
            .collect();
        let total = RECEIPT_AMOUNT
            .checked_mul(u64::try_from(points.len()).unwrap_or(0))
            .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

        let destination = |scalar: &[u8; FIELD_ELEMENT_BYTES], amount: u64| {
            let owner = published_owner(scalar)
                .map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
            let value = ProtocolValue::new(amount)
                .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
            Ok::<_, OwnerSigningNegativeRefusal>(LiveReceiptDestination::new(
                linker::OwnerParameter::new(owner),
                value,
            ))
        };
        let first = destination(&SECOND_SCALAR, total / 2)?;
        let second = destination(&FIRST_SCALAR, total - total / 2)?;

        let request = LiveTransferRequest::new(
            points,
            [first, second],
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        )
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

        let target =
            reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
        let finalization = finalize_live_transfer(&target, &self.abi, &request, &view, None, None)
            .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
        Ok(finalization.into_finalized())
    }

    /// The per-input signing requests for a finalized candidate.
    fn requests(finalized: &FinalizedLiveTransfer) -> Vec<OwnerSigningInputRequest> {
        finalized
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
            .collect()
    }

    /// The census of one candidate's parts, over the negative-evidence
    /// route.
    fn census(
        candidate: TargetTransaction,
        spent_outputs: Vec<transaction::live_census::SpentOutputCensusEntry>,
        genesis: Digest32,
        requests: &[OwnerSigningInputRequest],
    ) -> Result<OwnerSigningCensus, OwnerSigningNegativeRefusal> {
        let target =
            reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?,
        );
        let protected_bytes = candidate.encode_without_witness();
        let output_witnesses = candidate.output_witnesses().to_vec();
        OwnerSigningCensus::over_foreign_bytes_for_negative_evidence(
            &target,
            candidate,
            protected_bytes,
            output_witnesses,
            spent_outputs,
            LiveDeployment::new(genesis),
            requests,
            &curve,
        )
        .map_err(OwnerSigningNegativeRefusal::CensusRefused)
    }

    /// The spent-output census the finalized explicit form carries, read
    /// through the production route so its entries are exactly the ones
    /// that route would sign over.
    fn spent_outputs(
        &self,
        finalized: &FinalizedLiveTransfer,
    ) -> Result<Vec<transaction::live_census::SpentOutputCensusEntry>, OwnerSigningNegativeRefusal>
    {
        let target =
            reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?,
        );
        let requests = Self::requests(finalized);
        let census = OwnerSigningCensus::from_explicit_finalized(
            &target,
            finalized,
            LiveDeployment::new(self.genesis_block_hash),
            &requests,
            &curve,
        )
        .map_err(OwnerSigningNegativeRefusal::CensusRefused)?;
        Ok(census.spent_outputs().to_vec())
    }

    /// One candidate's submittable bytes and its first input's message.
    ///
    /// Both the control and the mutant are built here, differing only in
    /// whether the mutated destination's program is rewritten. Each input
    /// is re-signed over the candidate's own census and its witness stack
    /// assembled `[signature, leaf_script, control_block]`, which is the
    /// route's whole point: the mutant is signed over its own mutated
    /// bytes, so it passes the signature gate the leaf checks first.
    fn build(
        &self,
        finalized: &FinalizedLiveTransfer,
        spent_outputs: &[transaction::live_census::SpentOutputCensusEntry],
        mutate: bool,
    ) -> Result<(Vec<u8>, Digest32), OwnerSigningNegativeRefusal> {
        let candidate = finalized.protected().clone();
        let candidate = if mutate {
            rewrite_output_program(&candidate, MUTATED_OUTPUT, BARE_U_PROGRAM.to_vec())?
        } else {
            candidate
        };

        let requests = Self::requests(finalized);
        let census = Self::census(
            candidate.clone(),
            spent_outputs.to_vec(),
            self.genesis_block_hash,
            &requests,
        )?;

        let scalars = [SECOND_SCALAR, FIRST_SCALAR];
        let mut witnesses = candidate.witnesses().to_vec();
        let mut first_message = None;
        for record in finalized.receipts() {
            let position = usize::from(record.position());
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(record.position()))
                .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            if first_message.is_none() {
                first_message = Some(message);
            }
            let scalar = scalars
                .get(position)
                .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
            let material = signing_material(scalar)
                .map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
            let signature = material
                .sign(&message, &SIGNING_AUXILIARY)
                .map_err(|_| OwnerSigningNegativeRefusal::SigningRefused)?
                .to_vec();
            let witness = InputWitness::new(vec![
                signature,
                record.leaf_script().to_vec(),
                record.control_block().to_vec(),
            ]);
            *witnesses
                .get_mut(position)
                .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)? = witness;
        }

        let assembled = TargetTransaction::with_output_witnesses(
            candidate.version(),
            candidate.inputs().to_vec(),
            candidate.outputs().to_vec(),
            candidate.lock_time(),
            witnesses,
            candidate.output_witnesses().to_vec(),
        )
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

        let message =
            first_message.ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
        Ok((assembled.encode(), message))
    }

    /// Build the mutant and the control, and stage the mutant for
    /// submission. The declared field range is measured over the two
    /// candidates' WITNESSLESS serializations, where the re-signing does
    /// not reach, and checked to be the mutated output's program alone.
    fn stage_mutant(&mut self) -> Result<Vec<u8>, OwnerSigningNegativeRefusal> {
        let finalized = self.finalize()?;
        let spent_outputs = self.spent_outputs(&finalized)?;

        let (control_bytes, control_message) = self.build(&finalized, &spent_outputs, false)?;
        let (mutant_bytes, mutant_message) = self.build(&finalized, &spent_outputs, true)?;

        // The declared field: the mutated output's program, measured over
        // the witnessless serialization by substituting a sentinel program
        // of the mutant's own length and diffing. The witness bytes differ
        // between control and mutant because the mutant is re-signed, which
        // is the route's purpose and not a disturbance the declaration
        // covers.
        let control_candidate = finalized.protected().clone();
        let mutant_candidate =
            rewrite_output_program(&control_candidate, MUTATED_OUTPUT, BARE_U_PROGRAM.to_vec())?;
        let declared = changed_range(
            &control_candidate.encode_without_witness(),
            &mutant_candidate.encode_without_witness(),
        );

        // The whole witnessless diff between the two submitted candidates
        // must be that same range: nothing outside the mutated program
        // moved in the serialization the message is taken over.
        let touched = changed_range(
            &decode_witnessless(&control_bytes),
            &decode_witnessless(&mutant_bytes),
        );
        if touched != declared {
            return Err(OwnerSigningNegativeRefusal::MutationNotConfined { touched, declared });
        }

        self.record.mutant = Some(MutantObservation {
            declared_field_range: declared,
            submitted_bytes: mutant_bytes.len(),
            message: mutant_message,
            observed_layer: None,
            observed_detail: None,
        });
        self.record.control = Some(ControlObservation {
            submitted_bytes: control_bytes.len(),
            message: control_message,
            observed_layer: None,
            observed_detail: None,
            accepted_txid: None,
            reverification: None,
        });
        // The control's bytes are stashed on the control record's message
        // check; the bytes themselves are rebuilt for the control step so
        // the ceremony holds one pending submission at a time.
        Ok(mutant_bytes)
    }

    /// Record what the target did with the mutant.
    fn settle_mutant(&mut self, response: &NativeOperationResponse) {
        if let Some(mutant) = self.record.mutant.as_mut() {
            mutant.observed_layer = Some(response.observed_layer);
            mutant.observed_detail.clone_from(&response.observed_detail);
        }
    }

    /// Rebuild and stage the control for submission.
    fn stage_control(&mut self) -> Result<Vec<u8>, OwnerSigningNegativeRefusal> {
        let finalized = self.finalize()?;
        let spent_outputs = self.spent_outputs(&finalized)?;
        let (control_bytes, _message) = self.build(&finalized, &spent_outputs, false)?;
        Ok(control_bytes)
    }

    /// Record what the target did with the control, and the readback where
    /// it accepted.
    fn settle_control(&mut self, submitted: &[u8], response: &NativeOperationResponse) {
        if let Some(control) = self.record.control.as_mut() {
            control.observed_layer = Some(response.observed_layer);
            control
                .observed_detail
                .clone_from(&response.observed_detail);
            control.accepted_txid.clone_from(&response.accepted_txid);
            if response.observed_layer == ObservedOutcomeLayer::Accepted {
                control.reverification = reverify(response.mined_readback.as_ref(), submitted);
            }
        }
    }
}

impl TargetOperationPlanner for OwnerSigningNegativePlanner {
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
                    self.stage = Stage::Mutant;
                }
                Stage::Mutant => {
                    self.settle_mutant(response);
                    self.stage = Stage::Control;
                }
                Stage::Control => {
                    let submitted = self
                        .pending
                        .take()
                        .map(|pending| pending.bytes)
                        .unwrap_or_default();
                    self.settle_control(&submitted, response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(self.funding_step("issue-protocol-asset", true))),
            Stage::Fund => Ok(Some(self.funding_step("fund-explicit-constructor", false))),
            Stage::Mutant => match self.stage_mutant() {
                Ok(bytes) => {
                    self.pending = Some(PendingSubmission {
                        bytes: bytes.clone(),
                    });
                    Ok(Some(OperationStep::new(
                        MUTANT_STEP,
                        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                            transaction_bytes: bytes,
                        })),
                    )))
                }
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Control => match self.stage_control() {
                Ok(bytes) => {
                    self.pending = Some(PendingSubmission {
                        bytes: bytes.clone(),
                    });
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

/// One candidate with a single output's program rewritten, asset, value
/// and nonce held fixed.
fn rewrite_output_program(
    candidate: &TargetTransaction,
    output: usize,
    program: Vec<u8>,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let mut outputs = candidate.outputs().to_vec();
    let target = outputs
        .get_mut(output)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    *target = TargetOutput::new(target.asset(), target.value(), target.nonce(), program);
    TargetTransaction::with_output_witnesses(
        candidate.version(),
        candidate.inputs().to_vec(),
        outputs,
        candidate.lock_time(),
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
    .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)
}

/// The witnessless serialization of a candidate that decoded from wire
/// bytes, for confining the mutation to the serialization the message is
/// taken over.
fn decode_witnessless(bytes: &[u8]) -> Vec<u8> {
    TargetTransaction::decode(bytes)
        .map_or_else(|_| bytes.to_vec(), |tx| tx.encode_without_witness())
}

/// The half-open range, in the left string's coordinates, over which two
/// byte strings differ, bounded from both ends so an insertion or deletion
/// is a change inside a range rather than a change to everything after it.
fn changed_range(left: &[u8], right: &[u8]) -> (usize, usize) {
    let prefix = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(left.len() - prefix)
        .min(right.len().saturating_sub(prefix));
    (prefix, left.len() - suffix)
}

/// The two-origin readback check for an accepted control.
fn reverify(
    readback: Option<&MinedFundingReadback>,
    submitted: &[u8],
) -> Option<ControlReverification> {
    let readback = readback?;
    Some(ControlReverification {
        readback_matches_submission: readback.raw_transaction == submitted,
    })
}

/// One run's transcript, one fact per line.
#[must_use]
pub fn render_owner_signing_negatives(record: &OwnerSigningNegativeRecord) -> String {
    let mut lines = vec!["role owner-signing-negative-run".to_owned()];
    lines.push(format!(
        "issued_asset {}",
        record.issued_asset().unwrap_or("none")
    ));
    lines.push(format!("relinked {}", record.relinked()));

    for (index, coin) in record.coins().iter().enumerate() {
        lines.push(format!(
            "coin {index} program_bytes {} node_fields_match_expectation {}",
            coin.program().len(),
            coin.matches_expectation(),
        ));
    }

    if let Some(mutant) = record.mutant() {
        lines.push(format!(
            "mutant row vault-control-entitlement-or-bare-u-output declared_range {}..{} submitted_bytes {} message {} layer {} detail {}",
            mutant.declared_field_range().0,
            mutant.declared_field_range().1,
            mutant.submitted_bytes(),
            printed(mutant.message().as_slice()),
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        ));
    } else {
        lines.push("mutant none".to_owned());
    }

    if let Some(control) = record.control() {
        lines.push(format!(
            "control submitted_bytes {} message {} layer {} txid {} detail {}",
            control.submitted_bytes(),
            printed(control.message().as_slice()),
            control
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            control.accepted_txid().unwrap_or("none"),
            control.observed_detail().unwrap_or("none"),
        ));
        if let Some(check) = control.reverification() {
            lines.push(format!(
                "control_reverification readback_matches_submission {}",
                check.readback_matches_submission(),
            ));
        }
    } else {
        lines.push("control none".to_owned());
    }

    // Whether the two candidates' messages differ, stated as its own line:
    // a mutant whose message coincided with the control's would be signed
    // over the same bytes and the whole comparison would be vacuous.
    let distinct_messages = matches!(
        (record.mutant(), record.control()),
        (Some(mutant), Some(control)) if mutant.message() != control.message()
    );
    lines.push(format!("messages_differ {distinct_messages}"));

    if let Some(refusal) = record.refusal() {
        lines.push(format!("ceremony_refused {refusal:?}"));
    }

    for claim in OwnerSigningNegativeRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }
    lines.push("discharges_only_its_own_row true".to_owned());

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// The run of record: what one execution against a real node observed.
///
/// # Why the observation is a constant and not a stored file
///
/// The evidence a run produces is the observation, and an observation
/// nobody can name is not evidence. These constants are the identities and
/// figures ONE run against a real node produced, written down so a later
/// reader can ask the chain the same question. They re-run nothing and
/// prove nothing by existing; they make the run's own answer quotable.
pub mod run_of_record {
    /// The identity the target computed for the accepted control.
    ///
    /// The unmutated vault-control-entitlement candidate, accepted after
    /// the bare-u mutant had been refused — which is what makes the
    /// mutant's refusal attributable rather than merely recorded.
    pub const CONTROL_ACCEPTED_TXID: &str = "";

    /// What the target said to the bare-u-output mutant, verbatim.
    ///
    /// The coordinator leaf's `InspectOutputScriptPubKey` version clause,
    /// reached because the mutant is re-signed over its own bytes and its
    /// asset and value are unchanged, so the per-asset tally passes and the
    /// leaf runs.
    pub const MUTANT_REJECT_DETAIL: &str = "";

    /// The half-open witnessless byte range the mutant declared and stayed
    /// within: the mutated destination's program.
    pub const DECLARED_FIELD_RANGE: (usize, usize) = (0, 0);

    /// How many bytes the mutant handed the node.
    pub const MUTANT_SUBMITTED_BYTES: usize = 0;

    /// The run's wall time, in seconds.
    pub const WALL_SECONDS: f64 = 0.0;
}

#[cfg(test)]
mod tests {
    use super::{BARE_U_PROGRAM, changed_range};

    #[test]
    fn changed_range_bounds_a_mutation_from_both_ends() {
        assert_eq!(changed_range(&[0, 1, 2, 3, 4], &[0, 1, 9, 3, 4]), (2, 3));
        assert_eq!(changed_range(&[0, 1, 2, 3, 4], &[0, 1, 3, 4]), (2, 3));
        assert_eq!(changed_range(&[7, 7], &[7, 7]), (2, 2));
    }

    #[test]
    fn the_bare_u_program_is_a_version_zero_witness_program() {
        // Version zero where the receipt constructor's is version one, and
        // a whole 20-byte program body, which is what makes the coordinator
        // leaf's version clause the one that refuses it.
        assert_eq!(BARE_U_PROGRAM[0], 0x00, "the version byte is not zero");
        assert_eq!(BARE_U_PROGRAM[1], 0x14, "the push is not twenty bytes");
        assert_eq!(BARE_U_PROGRAM.len(), 22);
    }
}
