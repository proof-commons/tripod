//! The restart order's third and fourth steps: target CT conservation
//! recorded against a balance-valid control, and the proof-negatives
//! run from that control.
//!
//! # What this ceremony is, and why it is one ceremony
//!
//! Step three records that the target's own commitment-balance rule
//! accepted a conserving private transaction, against a balance-valid
//! control. Step four mutates exactly one field of that same control, one
//! case at a time, and observes the target refuse each. The two steps
//! share a control — the control step three records the conservation of is
//! the control step four mutates — so a single ceremony builds it once and
//! both steps read the one run.
//!
//! # The one observed run that is both halves
//!
//! The orchestrator's interlock ruling is implemented here rather than
//! argued: step three's conservation claim needs a refused non-conserving
//! case beside the accepted conserving one, and that refused case is step
//! four's wrong-blinder mutant. The pinned target refuses the wrong-blinder
//! mutant at the BALANCE layer — the balance check
//! (`src/confidential_validation.cpp:364`) is queued before the range-proof
//! loop (`:368`), and the mempool path (`src/validation.cpp:1097`, pvChecks
//! null) runs each queued check inline in source order and returns at the
//! first failure — so ONE observed wrong-blinder run is the conservation
//! claim's non-conserving half AND step four's first case. The record
//! discloses this in words: step three's refused half is the same observed
//! run step four records as the wrong-blinder proof-negative, so a reader
//! of two accepted ledger entries cannot count them as two observations.
//!
//! # Why the mutants are submitted before the control
//!
//! All five candidates spend the same predecessor coin. A rejected mutant
//! never mines and never spends anything — `testmempoolaccept` is a dry run
//! and the consensus retry cannot include a consensus-invalid transaction —
//! so submitting the four mutants first leaves the coin unspent for the
//! control, which is accepted and mined last. Submitting the control first
//! would spend the coin and every mutant would then be refused for a
//! missing input rather than for its mutation.
//!
//! # Attribution is by mutated field, not by target layer
//!
//! The target emits one identical refusal for all four mutations —
//! `bad-txns-in-ne-out` / "value in != value out"
//! (`src/consensus/tx_verify.cpp:250-251`), because the internal
//! `SCRIPT_ERR_PEDERSEN_TALLY` and `SCRIPT_ERR_RANGEPROOF` codes are
//! discarded inside `VerifyAmounts` and never leave it, and
//! [`ObservedOutcomeLayer`] mirrors that with a single
//! `ConsensusRejectionBeforeScript` member. So each proof-negative is
//! attributed by the construction field it moved — the value-commitment
//! field for the wrong blinder, the range-proof bytes for the two
//! range-proof cases — through [`attribute_proof_negative`], which refuses
//! a mutant that changed anything outside its located field. The guide's
//! own "each attributed to its own layer" wording is undischargeable from
//! this target and is filed as an erratum; it is not repaired here.
//!
//! # Recording and binding stay separate
//!
//! The recorder captures a layer and the node's own words without grading
//! them. [`assert_conservation_matches_the_run_of_record`] is the separate
//! reproduction gate: after the transcript is preserved, it binds the fresh
//! ceremony to every stable semantic fact the committed record carries.

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetSubmissionSubject,
};
use transaction::bytes::{
    AssetId, COMMITMENT_BYTES, OutputWitness, TargetOutput, TargetTransaction, Txid, ValueField,
};
use transaction::live_materialize::IndependentCommitmentCheck as _;
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::taproot::Digest32;

use crate::confidential_materializer::FirstPartyCommitmentCheck;
use crate::confidential_predecessor::{FUND_STEP, PredecessorShape};
use crate::error::VectorError;
use crate::live_owner_observation::{asset_of, printed_order};
use crate::live_plan::{LiveShapeVocabulary, RESERVE_ASSET, reviewed_target};
use crate::live_private_restart::{
    BuiltControl, ConsumedReceipt, LinkedDeployment, build_control, confidential_funding_step,
    issue_step, link_and_register, observe_funded_coins, verify_readback_signature,
};
use crate::live_restart::{
    BalanceValidControl, ProofNegativeAttribution, ProofNegativeAttributionRefusal,
    ProofNegativeCase, ProofNegativeMutation, attribute_proof_negative,
};

/// The ceremony's own name for the balance-valid control submission.
pub const CONTROL_STEP: &str = "submit-balance-valid-control";

/// The output index the proof-negatives mutate.
///
/// The successor's first output is the recipient's confidential share. It
/// carries a real range proof and a real value commitment, which is what
/// the three mutations need. The balancing output would serve equally; the
/// point is that ONE output is mutated and the same one across the three
/// cases, so a reader compares like with like.
const MUTATED_OUTPUT: usize = 0;

/// The output the private-ct-imbalance mutant rewrites the value commitment
/// of.
///
/// The successor's SECOND output, the sender's confidential change, chosen
/// precisely because it is not [`MUTATED_OUTPUT`]: the imbalance mutant and
/// the wrong-blinder mutant both replace a 33-byte value commitment and
/// both draw the balance failure, so placing them at the same output would
/// give them the same located field range and leave the record unable to
/// tell `private-ct-imbalance` from `wrong-private-blinding-balance`. At
/// different outputs the two ranges differ, which is what separates the two
/// rows. The change output carries its own value commitment and range
/// proof, so the mutation is a real value-commitment surgery there.
const IMBALANCE_OUTPUT: usize = 1;

/// A value blinder the wrong-blinder mutant recomputes its commitment
/// under.
///
/// Any scalar that is not the successor output's own blinder produces a
/// valid Pedersen commitment to the same value whose blinding factor does
/// not close the transaction's balance — which is exactly a wrong blinder.
/// It is published rather than random so the mutant is reproducible, and it
/// is a valid scalar (nonzero and far below the curve order). If it ever
/// equalled the real blinder the recomputed commitment would equal the
/// control's and [`attribute_proof_negative`] would refuse the mutant as
/// identical, so a collision fails loudly rather than passing quietly.
const WRONG_VALUE_BLINDER: [u8; 32] = [0x77; 32];

/// What this ceremony refuses, before any node is asked.
///
/// Every member is a construction or infrastructure fact and none is a
/// target verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConservationNegativeRefusal {
    /// The reviewed target did not build.
    SubstrateUnavailable,
    /// The issuance step named no asset to link against.
    IssuanceNamedNoAsset,
    /// The linked deployment or a fixture would not build.
    LinkOrRegisterRefused(String),
    /// The confidential funding step created no predecessor.
    FundingCreatedNoPredecessor,
    /// The balance-valid control would not construct.
    ControlNotConstructible(String),
    /// A proof-negative mutant did not differ only at its located field.
    MutantNotAttributable(ProofNegativeAttributionRefusal),
    /// The wrong-blinder commitment would not recompute.
    WrongBlinderNotRecomputable,
}

/// One proof-negative, as this ceremony submitted and observed it.
#[derive(Clone, Debug)]
pub struct MutantObservation {
    mutation: ProofNegativeMutation,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl MutantObservation {
    /// The proof-negative case.
    #[must_use]
    pub const fn case(&self) -> ProofNegativeCase {
        self.mutation.case()
    }

    /// The planner-selected serialized field this mutant changes.
    ///
    /// This is capture metadata only. The submitted request bytes remain
    /// owned by the executor journal.
    #[must_use]
    pub const fn capture_locator(&self) -> transaction::bytes::SerializedFieldLocator {
        self.mutation.locator()
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

    /// The attribution of this mutant against the balance-valid control.
    ///
    /// # Errors
    ///
    /// Every member of [`ProofNegativeAttributionRefusal`]: the control
    /// was not accepted, the selected field could not be located or did not
    /// change, or bytes outside that field also changed.
    pub fn attribute(
        &self,
        control: &BalanceValidControl,
    ) -> Result<ProofNegativeAttribution, ProofNegativeAttributionRefusal> {
        let layer = self
            .observed_layer
            .unwrap_or(ObservedOutcomeLayer::Accepted);
        attribute_proof_negative(control, &self.mutation, layer, self.observed_detail.clone())
    }
}

/// The transcript of one conservation-and-proof-negatives run.
#[derive(Clone, Debug, Default)]
pub struct ConservationNegativeRecord {
    issued_asset: Option<String>,
    predecessor_digest: Option<[u8; 32]>,
    successor_digest: Option<[u8; 32]>,
    consumed_commitment_prefix: Option<u8>,
    control_bytes: Option<Vec<u8>>,
    control_submitted_bytes: usize,
    control_observed_layer: Option<ObservedOutcomeLayer>,
    control_accepted_txid: Option<String>,
    reverification: Option<ControlReverification>,
    mutants: Vec<MutantObservation>,
    refusal: Option<ConservationNegativeRefusal>,
}

/// The two-origin check on the accepted control.
#[derive(Clone, Debug)]
pub struct ControlReverification {
    readback_matches_submission: bool,
    verified: bool,
}

impl ControlReverification {
    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether the accepted witness verifies against the recomputed
    /// message.
    #[must_use]
    pub const fn verified(&self) -> bool {
        self.verified
    }
}

impl ConservationNegativeRecord {
    /// The asset the run issued.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// The predecessor fixture digest selected by the completed planner.
    #[must_use]
    pub const fn predecessor_digest(&self) -> Option<[u8; 32]> {
        self.predecessor_digest
    }

    /// The successor fixture digest selected by the completed planner.
    #[must_use]
    pub const fn successor_digest(&self) -> Option<[u8; 32]> {
        self.successor_digest
    }

    /// The identity the target computed for the accepted control.
    #[must_use]
    pub fn control_accepted_txid(&self) -> Option<&str> {
        self.control_accepted_txid.as_deref()
    }

    /// The layer the control's submission observed.
    #[must_use]
    pub const fn control_observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.control_observed_layer
    }

    /// The two-origin check, where an acceptance was observed.
    #[must_use]
    pub const fn reverification(&self) -> Option<&ControlReverification> {
        self.reverification.as_ref()
    }

    /// The four proof-negatives, in the order they ran.
    #[must_use]
    pub fn mutants(&self) -> &[MutantObservation] {
        &self.mutants
    }

    /// The construction or infrastructure refusal, where the ceremony
    /// stopped before the node.
    #[must_use]
    pub const fn refusal(&self) -> Option<&ConservationNegativeRefusal> {
        self.refusal.as_ref()
    }

    /// The balance-valid control, where the control was accepted.
    ///
    /// The only path to a [`BalanceValidControl`] is an observed
    /// acceptance, so this returns `None` for any other outcome: a control
    /// the target did not accept never had its commitment balance checked,
    /// and a proof-negative derived from it would attribute nothing.
    #[must_use]
    pub fn balance_valid_control(&self) -> Option<BalanceValidControl> {
        let layer = self.control_observed_layer?;
        let identity = self.control_accepted_txid.clone()?;
        let bytes = self.control_bytes.clone()?;
        BalanceValidControl::from_observed(layer, identity, bytes).ok()
    }
}

/// The stage machine: issue, fund, four mutants, then the control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Issue the disposable asset the deployment is linked against.
    Issue,
    /// Fund the confidential predecessor at the receipt constructors.
    Fund,
    /// Submit the proof-negative at this index, mutant first so the coin
    /// stays unspent for the control.
    Mutant(usize),
    /// Submit the balance-valid control.
    Control,
    /// Nothing further.
    Done,
}

/// The conservation-and-proof-negatives ceremony.
pub struct ConservationNegativePlanner {
    stage: Stage,
    consumed: ConsumedReceipt,
    genesis_block_hash: Digest32,
    linked: Option<LinkedDeployment>,
    built: Option<BuiltControl>,
    record: ConservationNegativeRecord,
}

impl ConservationNegativePlanner {
    /// The ceremony bound to one deployment's printed genesis identity.
    ///
    /// It consumes the predecessor's primary output, because the control
    /// step three records the conservation of is a one-to-one control of
    /// exactly the shape step one submits.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed target
    /// does not build.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        reviewed_target()?;
        Ok(Self {
            stage: Stage::Issue,
            consumed: ConsumedReceipt::Primary,
            genesis_block_hash: printed_order(printed_genesis_identity),
            linked: None,
            built: None,
            record: ConservationNegativeRecord::default(),
        })
    }

    /// The transcript.
    #[must_use]
    pub const fn record(&self) -> &ConservationNegativeRecord {
        &self.record
    }

    /// Record a refusal and stop.
    fn refuse(&mut self, refusal: ConservationNegativeRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.stage = Stage::Done;
        PlanRefused
    }

    /// Link and register from the issued asset.
    fn settle_asset(&mut self, printed: &str) -> Result<(), ConservationNegativeRefusal> {
        let linked = link_and_register(
            PredecessorShape::DualParity,
            self.consumed,
            printed,
            LiveShapeVocabulary::Demonstration,
            RESERVE_ASSET,
        )
        .map_err(|refusal| {
            ConservationNegativeRefusal::LinkOrRegisterRefused(format!("{refusal:?}"))
        })?;
        self.record.issued_asset = Some(printed.to_owned());
        self.record.predecessor_digest = Some(linked.predecessor_digest());
        self.record.successor_digest = Some(linked.successor_digest());
        self.linked = Some(linked);
        Ok(())
    }

    /// Take the funded coins and build the control and its four mutants.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), ConservationNegativeRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(ConservationNegativeRefusal::FundingCreatedNoPredecessor)?;
        let coins = observe_funded_coins(linked, response)
            .map_err(|_| ConservationNegativeRefusal::FundingCreatedNoPredecessor)?;
        let coin = coins
            .get(self.consumed.index())
            .ok_or(ConservationNegativeRefusal::FundingCreatedNoPredecessor)?;
        self.record.consumed_commitment_prefix = match coin.value() {
            ValueField::Commitment(commitment) => commitment.first().copied(),
            _ => None,
        };

        let built = build_control(linked, coin, self.consumed, self.genesis_block_hash).map_err(
            |refusal| ConservationNegativeRefusal::ControlNotConstructible(format!("{refusal:?}")),
        )?;
        let control_bytes = built.transaction.encode();
        self.record.control_bytes = Some(control_bytes.clone());
        self.record.control_submitted_bytes = control_bytes.len();

        let asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(ConservationNegativeRefusal::IssuanceNamedNoAsset)?;
        self.record.mutants = build_mutants(&built.transaction, asset, self.consumed)?;

        // The built control is kept for the control's two-origin check,
        // which only runs on an acceptance and reads its census and
        // spent-owner bytes.
        self.built = Some(built);
        Ok(())
    }

    /// Record one mutant's observation.
    fn settle_mutant(&mut self, index: usize, response: &NativeOperationResponse) {
        if let Some(mutant) = self.record.mutants.get_mut(index) {
            mutant.observed_layer = Some(response.observed_layer);
            mutant.observed_detail.clone_from(&response.observed_detail);
        }
    }

    /// Record what the target did with the control, and the two origins
    /// where it accepted.
    fn settle_control(&mut self, response: &NativeOperationResponse) {
        self.record.control_observed_layer = Some(response.observed_layer);
        self.record
            .control_accepted_txid
            .clone_from(&response.accepted_txid);

        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return;
        }
        let (Some(readback), Some(control_bytes), Some(material)) = (
            response.mined_readback.as_ref(),
            self.record.control_bytes.as_ref(),
            self.built.as_ref(),
        ) else {
            return;
        };
        let readback_matches_submission = readback.raw_transaction == *control_bytes;
        let verified = material
            .census
            .signing_inputs()
            .first()
            .map(|input| {
                candidate_owner_message(&material.census, input, WitnessVectorTreatment::BothGrown)
            })
            .zip(material.spent_owner_bytes.as_ref())
            .is_some_and(|(message, owner)| {
                verify_readback_signature(&readback.raw_transaction, &message, owner)
            });
        self.record.reverification = Some(ControlReverification {
            readback_matches_submission,
            verified,
        });
    }

    /// The submission step for one mutant.
    fn mutant_step(&self, index: usize) -> Option<OperationStep> {
        let mutant = self.record.mutants.get(index)?;
        Some(OperationStep::new(
            mutant.case().name(),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: mutant.mutation.mutant().encode(),
            })),
        ))
    }
}

/// Build the four proof-negative mutants from the structured control.
///
/// Each mutation carries only its case, output index, and structured mutant.
/// Attribution asks the canonical encoder to locate the derived field.
fn build_mutants(
    control: &TargetTransaction,
    asset: AssetId,
    consumed: ConsumedReceipt,
) -> Result<Vec<MutantObservation>, ConservationNegativeRefusal> {
    // The wrong-blinder mutant: recompute the mutated output's value
    // commitment under a blinder that is not its own, a valid commitment
    // to the same value whose blinding factor does not close the balance.
    // The range proof is NOT regenerated: regenerating would move the
    // witness bytes too, and the mutation must stay inside the
    // value-commitment field.
    let amount = consumed.split()[0];
    let checker = FirstPartyCommitmentCheck::new();
    let wrong = checker
        .recompute(asset, amount, &WRONG_VALUE_BLINDER)
        .ok_or(ConservationNegativeRefusal::WrongBlinderNotRecomputable)?;
    let wrong_bytes = *wrong.bytes();
    let wrong_blinder_mutant = replace_output_value(control, MUTATED_OUTPUT, wrong_bytes)?;

    // The two range-proof mutants: empty the bytes, and corrupt them in
    // place. The commitments are untouched, so the balance check passes and
    // the range-proof check is the one that fails.
    let missing_mutant = replace_output_range_proof(control, MUTATED_OUTPUT, Vec::new())?;
    let malformed_bytes = corrupt_range_proof(control, MUTATED_OUTPUT)?;
    let malformed_mutant = replace_output_range_proof(control, MUTATED_OUTPUT, malformed_bytes)?;

    // The private-ct-imbalance mutant: at the SENDER's change output, a
    // value commitment to a value one unit above the change the control
    // balances, so the Pedersen tally no longer closes. The range proof is
    // left in place, because the balance check is queued before it
    // (`src/confidential_validation.cpp:364`), so the tally is what fails
    // rather than the proof. Placed at IMBALANCE_OUTPUT rather than
    // MUTATED_OUTPUT so its located field range separates it from the
    // wrong-blinder mutant they otherwise share a verdict with.
    let imbalance_amount = consumed.split()[1]
        .checked_add(1)
        .ok_or(ConservationNegativeRefusal::WrongBlinderNotRecomputable)?;
    let imbalance = checker
        .recompute(asset, imbalance_amount, &WRONG_VALUE_BLINDER)
        .ok_or(ConservationNegativeRefusal::WrongBlinderNotRecomputable)?;
    let imbalance_mutant = replace_output_value(control, IMBALANCE_OUTPUT, *imbalance.bytes())?;

    Ok(vec![
        mutant_observation(
            ProofNegativeCase::WrongBlinder,
            MUTATED_OUTPUT,
            wrong_blinder_mutant,
        ),
        mutant_observation(
            ProofNegativeCase::MissingRangeproof,
            MUTATED_OUTPUT,
            missing_mutant,
        ),
        mutant_observation(
            ProofNegativeCase::PrivateCtImbalance,
            IMBALANCE_OUTPUT,
            imbalance_mutant,
        ),
        mutant_observation(
            ProofNegativeCase::MalformedRangeproof,
            MUTATED_OUTPUT,
            malformed_mutant,
        ),
    ])
}

fn mutant_observation(
    case: ProofNegativeCase,
    output_index: usize,
    mutant: TargetTransaction,
) -> MutantObservation {
    let submitted_bytes = mutant.encode().len();
    MutantObservation {
        mutation: ProofNegativeMutation::at_output(case, output_index, mutant),
        submitted_bytes,
        observed_layer: None,
        observed_detail: None,
    }
}

/// The control with one output's value commitment replaced.
fn replace_output_value(
    control: &TargetTransaction,
    output: usize,
    commitment: [u8; COMMITMENT_BYTES],
) -> Result<TargetTransaction, ConservationNegativeRefusal> {
    let mut outputs = control.outputs().to_vec();
    let target = outputs.get_mut(output).ok_or_else(|| {
        ConservationNegativeRefusal::ControlNotConstructible(
            "the mutated output is absent".to_owned(),
        )
    })?;
    *target = TargetOutput::new(
        target.asset(),
        ValueField::Commitment(commitment),
        target.nonce(),
        target.program().to_vec(),
    );
    rebuild(control, outputs, control.output_witnesses().to_vec())
}

/// The control with one output-witness entry's range proof replaced.
fn replace_output_range_proof(
    control: &TargetTransaction,
    output: usize,
    range_proof: Vec<u8>,
) -> Result<TargetTransaction, ConservationNegativeRefusal> {
    let mut witnesses = control.output_witnesses().to_vec();
    let target = witnesses.get_mut(output).ok_or_else(|| {
        ConservationNegativeRefusal::ControlNotConstructible(
            "the mutated output witness is absent".to_owned(),
        )
    })?;
    *target = OutputWitness::new(target.surjection_proof().to_vec(), range_proof);
    rebuild(control, control.outputs().to_vec(), witnesses)
}

/// The mutated output-witness range proof corrupted in place, same length,
/// at least one byte moved.
fn corrupt_range_proof(
    control: &TargetTransaction,
    output: usize,
) -> Result<Vec<u8>, ConservationNegativeRefusal> {
    let mut bytes = control
        .output_witnesses()
        .get(output)
        .map(|witness| witness.range_proof().to_vec())
        .ok_or_else(|| {
            ConservationNegativeRefusal::ControlNotConstructible(
                "the mutated output witness is absent".to_owned(),
            )
        })?;
    let first = bytes.first_mut().ok_or_else(|| {
        ConservationNegativeRefusal::ControlNotConstructible(
            "the mutated output carries no range proof".to_owned(),
        )
    })?;
    *first = !*first;
    Ok(bytes)
}

/// Rebuild a transaction from the control's version, inputs, lock time, and
/// witnesses, with the given outputs and output witnesses.
fn rebuild(
    control: &TargetTransaction,
    outputs: Vec<TargetOutput>,
    output_witnesses: Vec<OutputWitness>,
) -> Result<TargetTransaction, ConservationNegativeRefusal> {
    TargetTransaction::with_output_witnesses(
        control.version(),
        control.inputs().to_vec(),
        outputs,
        control.lock_time(),
        control.witnesses().to_vec(),
        output_witnesses,
    )
    .map_err(|_| {
        ConservationNegativeRefusal::ControlNotConstructible(
            "the mutant will not serialize".to_owned(),
        )
    })
}

impl TargetOperationPlanner for ConservationNegativePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    let Some(printed) = response.issued_asset.clone() else {
                        return Err(self.refuse(ConservationNegativeRefusal::IssuanceNamedNoAsset));
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
                    self.stage = Stage::Mutant(0);
                }
                Stage::Mutant(index) => {
                    self.settle_mutant(index, response);
                    self.stage = if index + 1 < self.record.mutants.len() {
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
        }

        match self.stage {
            Stage::Issue => Ok(Some(issue_step())),
            Stage::Fund => {
                let Some(linked) = self.linked.as_ref() else {
                    return Err(
                        self.refuse(ConservationNegativeRefusal::FundingCreatedNoPredecessor)
                    );
                };
                let Some(printed) = self.record.issued_asset.clone() else {
                    return Err(self.refuse(ConservationNegativeRefusal::IssuanceNamedNoAsset));
                };
                Ok(Some(confidential_funding_step(linked, printed)))
            }
            Stage::Mutant(index) => self.mutant_step(index).map_or_else(
                || {
                    Err(
                        self.refuse(ConservationNegativeRefusal::ControlNotConstructible(
                            "a proof-negative mutant is missing".to_owned(),
                        )),
                    )
                },
                |step| Ok(Some(step)),
            ),
            Stage::Control => {
                let Some(bytes) = self.record.control_bytes.clone() else {
                    return Err(
                        self.refuse(ConservationNegativeRefusal::ControlNotConstructible(
                            "the control was not built".to_owned(),
                        )),
                    );
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

/// The funding case identity, so the native test can name the step it
/// funds under.
#[must_use]
pub fn funding_case() -> OperationCaseId {
    use target_elements_conformance::protocol::OperationStepKind;
    OperationCaseId {
        operation: OperationStepKind::FundConfidential,
        step: FUND_STEP.to_owned(),
    }
}

/// One digest as its printed spelling.
fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// The transcript, one fact per line.
#[must_use]
pub fn render_conservation_negatives(record: &ConservationNegativeRecord) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    out.push_str("run conservation-and-proof-negatives\n");
    let _ = writeln!(
        out,
        "issued_asset {}",
        record.issued_asset().unwrap_or("absent")
    );
    for (name, digest) in [
        ("predecessor_digest", record.predecessor_digest),
        ("successor_digest", record.successor_digest),
    ] {
        let _ = writeln!(
            out,
            "{name} {}",
            digest.map_or_else(|| "absent".to_owned(), hex)
        );
    }
    let _ = writeln!(
        out,
        "consumed_commitment_prefix {}",
        record
            .consumed_commitment_prefix
            .map_or_else(|| "none".to_owned(), |prefix| format!("{prefix:#04x}")),
    );
    let _ = writeln!(
        out,
        "control_submitted_bytes {}",
        record.control_submitted_bytes
    );
    let _ = writeln!(
        out,
        "control_observed_layer {}",
        record
            .control_observed_layer()
            .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
    );
    let _ = writeln!(
        out,
        "control_accepted_txid {}",
        record.control_accepted_txid().unwrap_or("none"),
    );
    if let Some(check) = record.reverification() {
        let _ = writeln!(
            out,
            "control_reverification readback_matches_submission {} verified {}",
            check.readback_matches_submission(),
            check.verified(),
        );
    }
    let control = record.balance_valid_control();
    for mutant in record.mutants() {
        let attribution = control.as_ref().map(|control| mutant.attribute(control));
        let located_range = attribution
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .map_or_else(
                || "none".to_owned(),
                |attribution| {
                    let range = attribution.located_field().control_range();
                    format!("{}..{}", range.start, range.end)
                },
            );
        let locator = mutant.mutation.locator();
        let _ = writeln!(
            out,
            "mutant case {} serialized_field {} output {} located_range {} submitted_bytes {} observed_layer {} detail {}",
            mutant.case().name(),
            locator.field().name(),
            locator.output_index(),
            located_range,
            mutant.submitted_bytes,
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        );
        let _ = writeln!(
            out,
            "mutant case {} attribution {}",
            mutant.case().name(),
            match attribution {
                Some(Ok(_)) => "attributed".to_owned(),
                Some(Err(refusal)) => format!("refused {refusal:?}"),
                None => "no-balance-valid-control".to_owned(),
            },
        );
    }
    let _ = writeln!(
        out,
        "construction_refusal {}",
        record
            .refusal()
            .map_or_else(|| "none".to_owned(), |refusal| format!("{refusal:?}")),
    );
    // What this run does NOT establish, in its own bytes.
    out.push_str("moves_the_sponsor_row false\n");
    out
}

/// The run of record: what one execution of steps three and four observed.
///
/// # Why the observation is a constant and not a stored file
///
/// The evidence a run produces is the observation, and an observation
/// nobody can name is not evidence. These constants are the identities and
/// figures ONE run against a real node produced, written down so a later
/// reader can ask the chain the same question. The target: Elements Core
/// v28.99.0-b7fc5d080a7e, at the pinned tip, on a disposable development
/// chain the run created and destroyed. It re-runs nothing and proves
/// nothing by existing; it makes the run's own answer quotable.
pub mod run_of_record {
    use target_elements_conformance::protocol::ObservedOutcomeLayer;

    /// The disposable asset the run issued.
    pub const ISSUED_ASSET: &str =
        "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

    /// The predecessor fixture's digest.
    pub const PREDECESSOR_DIGEST: &str =
        "ca43b210d6e74b76f7b3d3f79a6123556f150a7af9fa78e2571e2812b9d51fc2";

    /// The successor fixture's digest.
    pub const SUCCESSOR_DIGEST: &str =
        "31501b776502ee48d48b115d8bc80f55ba01bfc8cb6e163e3f2882848d025aa0";

    /// The identity the target computed for the accepted balance-valid
    /// control.
    ///
    /// The conserving half of step three's conservation record and the
    /// control step four's four mutants are derived from. Byte-identical
    /// to the one-to-one control's own run of record, because the ceremony
    /// is deterministic: two independent runs on two fresh chains produced
    /// this same identity.
    pub const CONTROL_ACCEPTED_TXID: &str =
        "4571a077826d45f64402a5c83ac9c0454fe42cf53b75f7aac2c8d07b574ad152";

    crate::recorded_acceptance::mint_recorded_acceptance!(control_accepted, CONTROL_ACCEPTED_TXID);

    /// How many bytes the accepted control submitted.
    pub const CONTROL_SUBMITTED_BYTES: usize = 9_136;

    /// The commitment prefix the consumed coin carried, the first of the
    /// two admitted parities.
    pub const CONSUMED_COMMITMENT_PREFIX: u8 = 0x08;

    /// The half-open byte range the wrong-blinder mutant declared and
    /// stayed within: the mutated output's 33-byte value-commitment field.
    pub const WRONG_BLINDER_FIELD_RANGE: (usize, usize) = (81, 114);

    /// The half-open byte range the private-ct-imbalance mutant declared
    /// and stayed within: the SECOND output's 33-byte value-commitment field.
    ///
    /// Distinct from [`WRONG_BLINDER_FIELD_RANGE`] because the two mutants
    /// sit at different outputs, which is what separates `private-ct-imbalance`
    /// from `wrong-private-blinding-balance` when the target draws the same
    /// `bad-txns-in-ne-out` for both.
    pub const PRIVATE_CT_IMBALANCE_FIELD_RANGE: (usize, usize) = (215, 248);

    /// The half-open byte range the two range-proof mutants declared: the
    /// mutated output-witness entry's range-proof region, length prefix
    /// included so the emptying and the corruption both fall inside it.
    pub const RANGEPROOF_FIELD_RANGE: (usize, usize) = (781, 4_958);

    /// The one identical refusal the target gave every mutant, at the
    /// [`super::ObservedOutcomeLayer::ConsensusRejectionBeforeScript`] layer.
    ///
    /// The wrong-blinder, range-proof and private-ct-imbalance mutants all
    /// draw it; the FIELD each declared is what tells their rows apart, the
    /// words being the same.
    pub const MUTANT_REJECT_DETAIL: &str = "bad-txns-in-ne-out";

    /// The layer the target refused every mutant at, TYPED.
    ///
    /// The doc above already named it and only a reader could act on
    /// that. This is the same fact in the vocabulary, so the classifier
    /// can compare the layer a run REACHED against the boundary a row
    /// DECLARED instead of assuming the two agree.
    pub const MUTANT_OBSERVED_LAYER: ObservedOutcomeLayer =
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript;

    /// The run's wall time, in seconds.
    pub const WALL_SECONDS: f64 = 12.7;
}

/// Bind one completed conservation-negatives ceremony to every stable
/// semantic fact in its committed run of record.
///
/// Wall time is intentionally absent: it is machine telemetry rather than
/// a reproducible result.
///
/// Under Q19, the recorded v1 fixture digests remain immutable historical
/// run data. A fresh conservation ceremony builds the same `DualParity`,
/// `Primary` predecessor and successor as private restart, so its fixture
/// assertions import that shared fixture's sole-live forward-v2 identities
/// from
/// [`crate::live_private_restart::run_of_record::forward_fixture_digest_v2`].
///
/// # Panics
///
/// Panics if a recorded fact is absent or differs, or if a mutant cannot be
/// attributed solely to its case-derived serialized field.
pub fn assert_conservation_matches_the_run_of_record(record: &ConservationNegativeRecord) {
    use run_of_record as run;

    let control = assert_recorded_control(record);
    assert_eq!(
        record.mutants().len(),
        4,
        "the ceremony did not record exactly four proof-negative mutants",
    );
    for (case, field_range) in [
        (
            ProofNegativeCase::WrongBlinder,
            run::WRONG_BLINDER_FIELD_RANGE,
        ),
        (
            ProofNegativeCase::MissingRangeproof,
            run::RANGEPROOF_FIELD_RANGE,
        ),
        (
            ProofNegativeCase::PrivateCtImbalance,
            run::PRIVATE_CT_IMBALANCE_FIELD_RANGE,
        ),
        (
            ProofNegativeCase::MalformedRangeproof,
            run::RANGEPROOF_FIELD_RANGE,
        ),
    ] {
        assert_recorded_mutant(record, &control, case, field_range);
    }
    assert_recorded_fixture(record);
}

fn assert_recorded_control(record: &ConservationNegativeRecord) -> BalanceValidControl {
    use run_of_record as run;

    assert_eq!(
        record.control_observed_layer(),
        Some(ObservedOutcomeLayer::Accepted),
        "the control was not ACCEPTED, so no mutant refusal is attributable",
    );
    let recorded_acceptance =
        run::control_accepted().expect("the run-of-record control identity parses");
    let observed_acceptance = record
        .control_accepted_txid()
        .map(Txid::from_target_display)
        .transpose()
        .expect("the observed control identity is valid hexadecimal");
    assert_eq!(
        observed_acceptance,
        Some(recorded_acceptance.accepted_identity()),
        "the control was accepted at an identity the run of record does not carry",
    );
    assert_eq!(
        record.control_submitted_bytes,
        run::CONTROL_SUBMITTED_BYTES,
        "the control handed the node a different number of bytes",
    );
    assert_eq!(
        record.consumed_commitment_prefix,
        Some(run::CONSUMED_COMMITMENT_PREFIX),
        "the consumed coin carried a different commitment prefix",
    );

    let reverification = record
        .reverification()
        .expect("the accepted control carries unconditional reverification");
    assert!(
        reverification.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
    assert!(
        reverification.verified(),
        "the accepted witness does not verify against the recomputed message",
    );

    let control = record
        .balance_valid_control()
        .expect("the accepted run carries a balance-valid control");
    assert_eq!(
        control.accepted_identity(),
        run::CONTROL_ACCEPTED_TXID,
        "the balance-valid control does not carry the recorded identity",
    );
    control
}

fn assert_recorded_mutant(
    record: &ConservationNegativeRecord,
    control: &BalanceValidControl,
    case: ProofNegativeCase,
    field_range: (usize, usize),
) {
    use run_of_record as run;

    let mut matching = record
        .mutants()
        .iter()
        .filter(|mutant| mutant.case() == case);
    let mutant = matching
        .next()
        .unwrap_or_else(|| panic!("the {} mutant is absent", case.name()));
    assert!(
        matching.next().is_none(),
        "the {} mutant was recorded more than once",
        case.name(),
    );
    assert_eq!(
        mutant.observed_layer(),
        Some(run::MUTANT_OBSERVED_LAYER),
        "the {} mutant was not refused at the recorded layer",
        case.name(),
    );
    assert_eq!(
        mutant.observed_detail(),
        Some(run::MUTANT_REJECT_DETAIL),
        "the {} mutant drew words the run of record does not carry",
        case.name(),
    );

    let attribution = mutant.attribute(control).unwrap_or_else(|refusal| {
        panic!(
            "the {} mutant is not confined to its declared field: {refusal:?}",
            case.name(),
        )
    });
    assert_eq!(
        attribution.case(),
        case,
        "the attributed mutant changed case",
    );
    assert_eq!(
        attribution.serialized_field(),
        case.serialized_field(),
        "the attributed mutant changed serialized field",
    );
    assert_eq!(
        attribution.control_identity(),
        run::CONTROL_ACCEPTED_TXID,
        "the mutant attributed against a different control",
    );
    assert_eq!(
        attribution.observed_layer(),
        run::MUTANT_OBSERVED_LAYER,
        "the attributed mutant changed observed layer",
    );
    assert_eq!(
        attribution.detail(),
        Some(run::MUTANT_REJECT_DETAIL),
        "the attributed mutant changed target detail",
    );
    let expected_range = field_range.0..field_range.1;
    assert_eq!(
        attribution.located_field().control_range(),
        &expected_range,
        "the {} mutant's field moved outside the run-of-record range",
        case.name(),
    );
}

fn assert_recorded_fixture(record: &ConservationNegativeRecord) {
    use crate::live_private_restart::run_of_record::forward_fixture_digest_v2 as forward;
    use run_of_record as run;

    assert_eq!(
        record.issued_asset(),
        Some(run::ISSUED_ASSET),
        "the ceremony issued a different asset",
    );
    let predecessor_digest = record.predecessor_digest.map(hex);
    assert_eq!(
        predecessor_digest.as_deref(),
        Some(forward::PREDECESSOR_DIGEST),
        "the predecessor digest differs from the shared forward-v2 fixture",
    );
    let successor_digest = record.successor_digest.map(hex);
    assert_eq!(
        successor_digest.as_deref(),
        Some(forward::SUCCESSOR_DIGEST),
        "the successor digest differs from the shared forward-v2 fixture",
    );
}

#[cfg(test)]
mod tests {
    use super::{
        ConservationNegativeRecord, ControlReverification, MutantObservation, ProofNegativeCase,
        ProofNegativeMutation, assert_conservation_matches_the_run_of_record,
        assert_recorded_fixture, render_conservation_negatives, run_of_record as run,
    };
    use crate::live_private_restart::run_of_record::forward_fixture_digest_v2 as forward;
    use target_elements_conformance::protocol::ObservedOutcomeLayer;
    use transaction::bytes::{
        AssetField, AssetId, InputWitness, NonceField, Outpoint, OutputWitness, TargetInput,
        TargetOutput, TargetTransaction, Txid, ValueField,
    };

    fn renderer_transaction(commitment: [u8; 33]) -> TargetTransaction {
        renderer_transaction_with_proof(commitment, vec![1_u8, 2, 3])
    }

    fn renderer_transaction_with_proof(
        commitment: [u8; 33],
        range_proof: Vec<u8>,
    ) -> TargetTransaction {
        TargetTransaction::with_output_witnesses(
            2,
            vec![TargetInput::new(
                Outpoint::new(Txid::from_internal([0x22; 32]), 0)
                    .expect("the fixture outpoint is in range"),
                u32::MAX,
            )],
            vec![TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal([0x11; 32])),
                ValueField::Commitment(commitment),
                NonceField::Null,
                vec![0x51],
            )],
            0,
            vec![InputWitness::new(Vec::new())],
            vec![OutputWitness::range_proof_only(range_proof)],
        )
        .expect("the renderer fixture is structurally complete")
    }

    fn recorded_digest(text: &str) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        let (pairs, _) = text.as_bytes().as_chunks::<2>();
        for (slot, pair) in bytes.iter_mut().zip(pairs) {
            let digits = std::str::from_utf8(pair).expect("the recorded digest is ASCII hex");
            *slot = u8::from_str_radix(digits, 16).expect("the recorded digest is hexadecimal");
        }
        bytes
    }

    fn answered_mutant(case: ProofNegativeCase) -> MutantObservation {
        let mutant = match case {
            ProofNegativeCase::WrongBlinder | ProofNegativeCase::PrivateCtImbalance => {
                renderer_transaction([0x09; 33])
            }
            ProofNegativeCase::MissingRangeproof => {
                renderer_transaction_with_proof([0x08; 33], vec![0xff_u8, 2, 3])
            }
            ProofNegativeCase::MalformedRangeproof => {
                renderer_transaction_with_proof([0x08; 33], vec![1_u8, 0xff, 3])
            }
        };
        MutantObservation {
            submitted_bytes: mutant.encode().len(),
            mutation: ProofNegativeMutation::at_output(case, 0, mutant),
            observed_layer: Some(run::MUTANT_OBSERVED_LAYER),
            observed_detail: Some(run::MUTANT_REJECT_DETAIL.to_owned()),
        }
    }

    fn synthetic_record() -> ConservationNegativeRecord {
        let control = renderer_transaction([0x08; 33]);
        ConservationNegativeRecord {
            issued_asset: Some(run::ISSUED_ASSET.to_owned()),
            predecessor_digest: Some(recorded_digest(run::PREDECESSOR_DIGEST)),
            successor_digest: Some(recorded_digest(run::SUCCESSOR_DIGEST)),
            consumed_commitment_prefix: Some(run::CONSUMED_COMMITMENT_PREFIX),
            control_bytes: Some(control.encode()),
            control_submitted_bytes: run::CONTROL_SUBMITTED_BYTES,
            control_observed_layer: Some(ObservedOutcomeLayer::Accepted),
            control_accepted_txid: Some(run::CONTROL_ACCEPTED_TXID.to_owned()),
            reverification: Some(ControlReverification {
                readback_matches_submission: true,
                verified: true,
            }),
            mutants: vec![
                answered_mutant(ProofNegativeCase::WrongBlinder),
                answered_mutant(ProofNegativeCase::MissingRangeproof),
                answered_mutant(ProofNegativeCase::PrivateCtImbalance),
                answered_mutant(ProofNegativeCase::MalformedRangeproof),
            ],
            refusal: None,
        }
    }

    #[test]
    #[should_panic(expected = "predecessor digest differs from the shared forward-v2 fixture")]
    fn recorded_v1_fixture_digests_refuse_the_forward_binding() {
        let record = synthetic_record();

        assert_recorded_fixture(&record);
    }

    #[test]
    fn forward_v2_fixture_digests_pass_the_forward_binding() {
        let mut record = synthetic_record();
        record.predecessor_digest = Some(recorded_digest(forward::PREDECESSOR_DIGEST));
        record.successor_digest = Some(recorded_digest(forward::SUCCESSOR_DIGEST));

        assert_recorded_fixture(&record);
    }

    #[test]
    #[should_panic(expected = "accepted at an identity the run of record does not carry")]
    fn a_flipped_control_identity_refuses_the_record_binding() {
        let mut record = synthetic_record();
        record
            .control_accepted_txid
            .as_mut()
            .expect("the synthetic control has an accepted identity")
            .replace_range(..1, "0");

        assert_conservation_matches_the_run_of_record(&record);
    }

    #[test]
    #[should_panic(expected = "drew words the run of record does not carry")]
    fn a_flipped_mutant_detail_refuses_the_record_binding() {
        let mut record = synthetic_record();
        record.mutants[0].observed_detail = Some("bad-txns-in-ne-ou0".to_owned());

        assert_conservation_matches_the_run_of_record(&record);
    }

    #[test]
    #[should_panic(expected = "accepted control carries unconditional reverification")]
    fn missing_reverification_refuses_the_record_binding() {
        let mut record = synthetic_record();
        record.reverification = None;

        assert_conservation_matches_the_run_of_record(&record);
    }

    #[test]
    #[should_panic(expected = "field moved outside the run-of-record range")]
    fn a_wrong_located_field_range_refuses_the_record_binding() {
        let record = synthetic_record();

        assert_conservation_matches_the_run_of_record(&record);
    }

    #[test]
    fn the_run_of_record_names_one_control_and_four_field_ranges() {
        // The figures are the run's, and this checks their SHAPE rather
        // than re-deriving them: a 64-hex control identity, two
        // value-commitment fields exactly 33 bytes wide at DIFFERENT
        // offsets, and a range-proof field wide enough to hold a real
        // proof.
        assert_eq!(run::CONTROL_ACCEPTED_TXID.len(), 64);
        assert_eq!(run::PREDECESSOR_DIGEST.len(), 64);
        assert_ne!(run::PREDECESSOR_DIGEST, run::SUCCESSOR_DIGEST);
        assert_eq!(run::CONSUMED_COMMITMENT_PREFIX, 0x08);
        assert_eq!(run::WRONG_BLINDER_FIELD_RANGE, (81, 114));
        assert_eq!(run::PRIVATE_CT_IMBALANCE_FIELD_RANGE, (215, 248));
        assert_eq!(run::RANGEPROOF_FIELD_RANGE, (781, 4_958));

        let (wb_start, wb_end) = run::WRONG_BLINDER_FIELD_RANGE;
        assert_eq!(
            wb_end - wb_start,
            33,
            "the value-commitment field is 33 bytes"
        );

        // The imbalance mutant's field is also 33 bytes and sits at a
        // DIFFERENT offset: the two do not overlap, which is what separates
        // the two rows that share the target's words.
        let (im_start, im_end) = run::PRIVATE_CT_IMBALANCE_FIELD_RANGE;
        assert_eq!(
            im_end - im_start,
            33,
            "the second value-commitment field is 33 bytes"
        );
        assert!(
            im_start >= wb_end,
            "the two value-commitment fields do not overlap"
        );

        let (rp_start, rp_end) = run::RANGEPROOF_FIELD_RANGE;
        assert!(
            rp_end - rp_start > 1000,
            "the range-proof field holds a real proof"
        );
        assert!(
            rp_start > im_end,
            "the range proof follows both value commitments"
        );
    }

    #[test]
    fn the_two_range_proof_cases_share_a_field_and_the_wrong_blinder_does_not() {
        assert_eq!(
            ProofNegativeCase::MissingRangeproof.serialized_field(),
            ProofNegativeCase::MalformedRangeproof.serialized_field(),
        );
        assert_ne!(
            ProofNegativeCase::WrongBlinder.serialized_field(),
            ProofNegativeCase::MissingRangeproof.serialized_field(),
        );
    }

    #[test]
    fn forward_renderer_names_case_and_serialized_field_truthfully() {
        let control = renderer_transaction([0x08; 33]);
        let mutant = renderer_transaction([0x09; 33]);
        let submitted_bytes = mutant.encode().len();
        let mutation = ProofNegativeMutation::at_output(ProofNegativeCase::WrongBlinder, 0, mutant);
        let location = control
            .locate_serialized_field(mutation.locator())
            .expect("the renderer fixture's commitment is serialized");
        let record = ConservationNegativeRecord {
            control_bytes: Some(control.encode()),
            control_observed_layer: Some(ObservedOutcomeLayer::Accepted),
            control_accepted_txid: Some("accepted-control".to_owned()),
            mutants: vec![MutantObservation {
                mutation,
                submitted_bytes,
                observed_layer: None,
                observed_detail: None,
            }],
            ..ConservationNegativeRecord::default()
        };

        let rendered = render_conservation_negatives(&record);
        let expected = format!(
            "mutant case wrong-blinder serialized_field value-commitment output 0 located_range {}..{}",
            location.range().start,
            location.range().end,
        );
        assert!(rendered.contains(&expected));
        assert!(!rendered.contains("field value-blinder"));
    }
}
