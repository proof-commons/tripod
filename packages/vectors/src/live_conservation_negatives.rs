//! The restart order's third and fourth steps: target CT conservation
//! recorded against a balance-valid control, and the three proof-negatives
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
//! All four candidates spend the same predecessor coin. A rejected mutant
//! never mines and never spends anything — `testmempoolaccept` is a dry run
//! and the consensus retry cannot include a consensus-invalid transaction —
//! so submitting the three mutants first leaves the coin unspent for the
//! control, which is accepted and mined last. Submitting the control first
//! would spend the coin and every mutant would then be refused for a
//! missing input rather than for its mutation.
//!
//! # Attribution is by mutated field, not by target layer
//!
//! The target emits one identical refusal for all three mutations —
//! `bad-txns-in-ne-out` / "value in != value out"
//! (`src/consensus/tx_verify.cpp:250-251`), because the internal
//! `SCRIPT_ERR_PEDERSEN_TALLY` and `SCRIPT_ERR_RANGEPROOF` codes are
//! discarded inside `VerifyAmounts` and never leave it, and
//! [`ObservedOutcomeLayer`] mirrors that with a single
//! `ConsensusRejectionBeforeScript` member. So each proof-negative is
//! attributed by the construction field it moved — the value-commitment
//! field for the wrong blinder, the range-proof bytes for the two
//! range-proof cases — through [`attribute_proof_negative`], which refuses
//! a mutant that changed anything outside its declared field. The guide's
//! own "each attributed to its own layer" wording is undischargeable from
//! this target and is filed as an erratum; it is not repaired here.
//!
//! # Nothing here decides what the node should have found
//!
//! The observation is a layer and the node's own words. The control's
//! acceptance is asserted nowhere; where the run did observe an acceptance,
//! the one content check is the two-origin agreement, on the pattern the
//! one-to-one ceremony set.

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetSubmissionSubject,
};
use transaction::bytes::{
    AssetId, COMMITMENT_BYTES, OutputWitness, TargetOutput, TargetTransaction, ValueField,
};
use transaction::live_materialize::IndependentCommitmentCheck as _;
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::taproot::Digest32;

use crate::confidential_materializer::FirstPartyCommitmentCheck;
use crate::confidential_predecessor::FUND_STEP;
use crate::error::VectorError;
use crate::live_owner_observation::{asset_of, printed_order};
use crate::live_plan::reviewed_target;
use crate::live_private_restart::{
    BuiltControl, ConsumedReceipt, LinkedDeployment, build_control, confidential_funding_step,
    issue_step, link_and_register, observe_funded_coins, verify_readback_signature,
};
use crate::live_restart::{
    BalanceValidControl, ProofNegativeAttribution, ProofNegativeAttributionRefusal,
    ProofNegativeCase, attribute_proof_negative,
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
    /// A proof-negative mutant reached outside its declared field, or was
    /// identical to the control.
    MutantNotAttributable(ProofNegativeAttributionRefusal),
    /// The wrong-blinder commitment would not recompute.
    WrongBlinderNotRecomputable,
}

/// One proof-negative, as this ceremony submitted and observed it.
#[derive(Clone, Debug)]
pub struct MutantObservation {
    case: ProofNegativeCase,
    declared_field_range: (usize, usize),
    mutant_bytes: Vec<u8>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl MutantObservation {
    /// The proof-negative case.
    #[must_use]
    pub const fn case(&self) -> ProofNegativeCase {
        self.case
    }

    /// The half-open byte range, in the control's coordinates, that the
    /// case declared it was mutating.
    #[must_use]
    pub const fn declared_field_range(&self) -> (usize, usize) {
        self.declared_field_range
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
    /// was not accepted, the mutant reached outside its declared field, or
    /// it was identical to the control.
    pub fn attribute(
        &self,
        control: &BalanceValidControl,
    ) -> Result<ProofNegativeAttribution, ProofNegativeAttributionRefusal> {
        let layer = self
            .observed_layer
            .unwrap_or(ObservedOutcomeLayer::Accepted);
        attribute_proof_negative(
            control,
            self.case,
            self.declared_field_range,
            &self.mutant_bytes,
            layer,
            self.observed_detail.clone(),
        )
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

    /// The three proof-negatives, in the order they ran.
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

/// The stage machine: issue, fund, three mutants, then the control.
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
        let linked = link_and_register(self.consumed, printed).map_err(|refusal| {
            ConservationNegativeRefusal::LinkOrRegisterRefused(format!("{refusal:?}"))
        })?;
        self.record.issued_asset = Some(printed.to_owned());
        self.record.predecessor_digest = Some(linked.predecessor_digest());
        self.record.successor_digest = Some(linked.successor_digest());
        self.linked = Some(linked);
        Ok(())
    }

    /// Take the funded coins and build the control and its three mutants.
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
            mutant.case.name(),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: mutant.mutant_bytes.clone(),
            })),
        ))
    }
}

/// Build the three proof-negative mutants from the structured control.
///
/// Each mutates exactly one field. The declared field range is computed by
/// substituting a sentinel into the control and diffing, so the range is
/// the field's real serialized extent rather than a hand-derived offset.
fn build_mutants(
    control: &TargetTransaction,
    asset: AssetId,
    consumed: ConsumedReceipt,
) -> Result<Vec<MutantObservation>, ConservationNegativeRefusal> {
    let control_bytes = control.encode();

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
    let wrong_blinder_mutant = replace_output_value(control, MUTATED_OUTPUT, wrong_bytes)?.encode();

    // The declared value-commitment field: substitute a sentinel that
    // differs in every one of the 33 bytes, so the diff is the whole field.
    let sentinel_commitment = commitment_sentinel(control, MUTATED_OUTPUT)?;
    let sentinel_bytes =
        replace_output_value(control, MUTATED_OUTPUT, sentinel_commitment)?.encode();
    let value_field_range = changed_range(&control_bytes, &sentinel_bytes);

    // The two range-proof mutants: empty the bytes, and corrupt them in
    // place. The commitments are untouched, so the balance check passes and
    // the range-proof check is the one that fails.
    let missing_mutant = replace_output_range_proof(control, MUTATED_OUTPUT, Vec::new())?.encode();
    let malformed_bytes = corrupt_range_proof(control, MUTATED_OUTPUT)?;
    let malformed_mutant =
        replace_output_range_proof(control, MUTATED_OUTPUT, malformed_bytes)?.encode();

    // The declared range-proof field: substitute an empty range proof and
    // diff, so the declared range covers the length prefix the emptying
    // moves as well as the bytes the corruption moves.
    let range_field_range = changed_range(&control_bytes, &missing_mutant);

    Ok(vec![
        MutantObservation {
            case: ProofNegativeCase::WrongBlinder,
            declared_field_range: value_field_range,
            submitted_bytes: wrong_blinder_mutant.len(),
            mutant_bytes: wrong_blinder_mutant,
            observed_layer: None,
            observed_detail: None,
        },
        MutantObservation {
            case: ProofNegativeCase::MissingRangeproof,
            declared_field_range: range_field_range,
            submitted_bytes: missing_mutant.len(),
            mutant_bytes: missing_mutant,
            observed_layer: None,
            observed_detail: None,
        },
        MutantObservation {
            case: ProofNegativeCase::MalformedRangeproof,
            declared_field_range: range_field_range,
            submitted_bytes: malformed_mutant.len(),
            mutant_bytes: malformed_mutant,
            observed_layer: None,
            observed_detail: None,
        },
    ])
}

/// A commitment that differs from the mutated output's own in every byte,
/// for locating the value-commitment field by diffing.
fn commitment_sentinel(
    control: &TargetTransaction,
    output: usize,
) -> Result<[u8; COMMITMENT_BYTES], ConservationNegativeRefusal> {
    let value = control
        .outputs()
        .get(output)
        .map(TargetOutput::value)
        .ok_or_else(|| {
            ConservationNegativeRefusal::ControlNotConstructible(
                "the mutated output is absent".to_owned(),
            )
        })?;
    let ValueField::Commitment(original) = value else {
        return Err(ConservationNegativeRefusal::ControlNotConstructible(
            "the mutated output is not confidential".to_owned(),
        ));
    };
    let mut sentinel = [0_u8; COMMITMENT_BYTES];
    for (index, byte) in original.iter().enumerate() {
        sentinel[index] = !byte;
    }
    Ok(sentinel)
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

/// The half-open range, in the control's coordinates, over which two byte
/// strings differ.
///
/// The common prefix and suffix bound the change from both ends, so an
/// insertion or a deletion is a change inside a range rather than a change
/// to everything downstream of it — the same computation
/// [`attribute_proof_negative`] uses, exposed here so the declared field
/// range is measured rather than asserted.
fn changed_range(control: &[u8], mutant: &[u8]) -> (usize, usize) {
    let prefix = control
        .iter()
        .zip(mutant)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = control
        .iter()
        .rev()
        .zip(mutant.iter().rev())
        .take_while(|(left, right)| left == right)
        .count()
        .min(control.len() - prefix)
        .min(mutant.len().saturating_sub(prefix));
    (prefix, control.len() - suffix)
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
        let _ = writeln!(
            out,
            "mutant {} field {} declared_range {}..{} submitted_bytes {} observed_layer {} detail {}",
            mutant.case().name(),
            mutant.case().mutated_field().name(),
            mutant.declared_field_range().0,
            mutant.declared_field_range().1,
            mutant.submitted_bytes,
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        );
        let attribution = control.as_ref().map(|control| mutant.attribute(control));
        let _ = writeln!(
            out,
            "mutant {} attribution {}",
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
