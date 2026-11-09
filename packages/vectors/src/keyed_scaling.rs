//! Measurement (f) staging: distinct mined SOURCE handles, never sibling handles.
//!
//! Counts are provisional construction choices, not covenant laws. The fixed
//! explicit-asset envelope has one P2WPKH fee input, one keyed successor, ordinary
//! change and an explicit fee. Native stack and validation costs remain absent.
//! Each boundary row has fresh handles: the adapter may directly mine a transaction
//! that relay policy refuses. Preparation is recorded separately from MERGE.

use crate::keyed_birth_native::{
    BIRTH_VALUE, BirthNativeRefusal as Refusal, FIXTURE_FEE, SPONSOR_VALUE, apply_birth_signature,
    coin_outpoint, coin_output, identity, output, replay_operations, transaction,
};
use crate::keyed_constructor::{
    KeyedConstructorDescriptor, PublicKeyedCurve, construct_keyed, reconstruct_keyed,
};
use crate::keyed_continuation_native::{continuation_capture_json, decode_wire_operations};
use crate::keyed_funding::reported_floor;
use crate::maturity_native::sponsored::{Policy, check_readback, detail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tapscript::keyed_constructor::KeyedSemanticKey;
use target_elements::{ResourceDimension, ReviewedElementsTapscriptDefinition};
use target_elements_conformance::executor::{
    NativeOperationCapture, OperationStep, PlanRefused, TargetOperationPlanner,
};
use target_elements_conformance::protocol::{
    FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireSighashProfile,
};
use transaction::bytes::{InputWitness, Outpoint, TargetInput, TargetTransaction};

/// Reviewed standard-weight envelope, checked against the supplied target.
pub const STANDARD_WEIGHT: u64 = 400_000;
/// Marginal keyed input: 41 stripped bytes times four plus 91 witness bytes.
pub const INPUT_WEIGHT: u64 = 1586 - 1331;
/// Conservative P2WPKH signature including sighash, under low-S DER policy.
pub const SPONSOR_SIGNATURE_BYTES: usize = 72;
/// Counts beyond 251 covenant inputs need eight extra weight units for `CompactSize`.
///
/// 1331 - 255 = 1076 fixed units in the admitted one-input carrier; one extra
/// signature byte bounds the admitted 71-byte witness by 72 bytes: 1077 + 8.
pub const LARGE_FIXED_WEIGHT: u64 = 1085;
/// Largest input count below the standard envelope in this fixed family.
pub const PROPOSED_MAXIMUM: usize =
    ((STANDARD_WEIGHT - 1 - LARGE_FIXED_WEIGHT) / INPUT_WEIGHT) as usize;
/// Mid-range point, frozen before capture.
pub const INTERIOR: usize = PROPOSED_MAXIMUM / 2;
/// SOURCE, small MERGEs, interior, overweight neighbour, maximum control last.
pub const COUNTS: [usize; 6] = [0, 1, 2, INTERIOR, PROPOSED_MAXIMUM + 1, PROPOSED_MAXIMUM];
/// Fixed explicit MERGE fee; actual reported floors are checked per submission.
pub const MERGE_FEE: u64 = 20_000;
/// SOURCE preparation count, including the standalone SOURCE subject.
pub const SOURCE_COUNT: usize = 1 + 1 + 2 + INTERIOR + PROPOSED_MAXIMUM + 1 + PROPOSED_MAXIMUM;
/// Batch funding respects the adapter's 255-output limit, not an input limit.
pub const FUNDING_COUNT: usize = 21;
/// Two exchanges per SOURCE and MERGE plus the funding batches.
pub const EXCHANGE_COUNT: usize = SOURCE_COUNT * 2 + 5 * 2 + FUNDING_COUNT;

/// Capture-stage observation, never archive admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalingClass {
    /// Default relay, template inclusion and exact mined bytes.
    RelayMined,
    /// Exact tx-size relay refusal followed by exact direct-block readback.
    WeightPolicyRefusedBlockMined,
    /// Exact tx-size relay refusal without independently established consensus.
    WeightPolicyRefused,
    /// An explicit bad-txns-oversize consensus diagnostic; no mined result.
    ConsensusWeightRefused,
    /// Missing, mismatched or earlier-layer evidence.
    Open,
}

/// One complete submitted carrier, with preparation kept distinct from MERGE.
#[derive(Clone, Debug)]
pub struct ScalingRecord {
    /// Range row, an index into COUNTS.
    pub row: usize,
    /// SOURCE ordinal within this row, or None for its MERGE.
    pub preparation: Option<usize>,
    /// Exact signing and submission links.
    pub signing: (OperationStep, NativeOperationResponse),
    /// Full wire submission response, including policy and node-read weight.
    pub submission: (OperationStep, NativeOperationResponse),
}

/// Finite campaign state. Only mined output-zero SOURCE bytes become handles.
#[derive(Clone, Debug)]
pub struct ScalingPlanner {
    descriptor: KeyedConstructorDescriptor,
    row: usize,
    coins: Vec<FundedOutput>,
    parents: Vec<TargetTransaction>,
    signed: Option<TargetTransaction>,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    records: Vec<ScalingRecord>,
    policy: Option<Policy>,
    asset_program: Option<(String, String)>,
    refusal: Option<Refusal>,
}
impl ScalingPlanner {
    /// Construct a frozen schedule and canonical public keyed recipe.
    /// # Errors
    /// Refuses another target weight envelope or a noncanonical recipe.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        key: [u8; 32],
    ) -> Result<Self, Refusal> {
        let _span = tracing::debug_span!("keyed_scaling_new").entered();
        if target
            .definition()
            .resources()
            .policy()
            .bounds()
            .get(&ResourceDimension::TransactionWeight)
            .and_then(|b| b.maximum())
            != Some(STANDARD_WEIGHT)
        {
            return Err(Refusal::Policy);
        }
        Ok(Self {
            descriptor: construct_keyed(target, KeyedSemanticKey::new(key), &PublicKeyedCurve)
                .map_err(|_| Refusal::Leaf)?,
            row: 0,
            coins: vec![],
            parents: vec![],
            signed: None,
            pending: None,
            exchanges: vec![],
            records: vec![],
            policy: None,
            asset_program: None,
            refusal: None,
        })
    }
    /// Exact public recipe bound into the wire journal.
    #[must_use]
    pub const fn descriptor(&self) -> &KeyedConstructorDescriptor {
        &self.descriptor
    }
    /// All setup and subject exchanges in acquisition order.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }
    /// SOURCE preparation and MERGE records, separately tagged.
    #[must_use]
    pub fn records(&self) -> &[ScalingRecord] {
        &self.records
    }
    /// Construction or replay refusal, not a node verdict.
    #[must_use]
    pub const fn refusal(&self) -> Option<Refusal> {
        self.refusal
    }
    /// Verify the exact terminal census without claiming any acceptance.
    /// # Errors
    /// Refuses stopped, partial or extra exchanges.
    pub const fn complete(&self) -> Result<(), Refusal> {
        if self.refusal.is_some()
            || self.row != COUNTS.len()
            || self.pending.is_some()
            || self.exchanges.len() != EXCHANGE_COUNT
            || self.records.len() != SOURCE_COUNT + 5
        {
            return Err(Refusal::Replay);
        }
        Ok(())
    }
    fn source(&self) -> bool {
        self.parents.len() < COUNTS[self.row].max(1)
    }
    fn sponsor(&self) -> Result<&FundedOutput, Refusal> {
        self.coins
            .get(if self.source() {
                self.parents.len()
            } else {
                COUNTS[self.row]
            })
            .ok_or(Refusal::Funding)
    }
    fn build(&self) -> Result<TargetTransaction, Refusal> {
        let source = self.source();
        let n = if source { 0 } else { self.parents.len() };
        let _span = tracing::debug_span!("keyed_scaling_build", source, inputs = n).entered();
        let coin = self.sponsor()?;
        let fact = coin_output(coin)?;
        let fee = if source { FIXTURE_FEE } else { MERGE_FEE };
        let total = fact.value + BIRTH_VALUE * n as u64;
        let outputs = vec![
            output(&fact, BIRTH_VALUE, self.descriptor.output_program.clone()),
            output(&fact, total - BIRTH_VALUE - fee, fact.program.clone()),
            output(&fact, fee, vec![]),
        ];
        let mut inputs = Vec::new();
        let mut witnesses = Vec::new();
        if !source {
            for parent in &self.parents {
                if parent.outputs().first()
                    != Some(&output(
                        &fact,
                        BIRTH_VALUE,
                        self.descriptor.output_program.clone(),
                    ))
                {
                    return Err(Refusal::Pair);
                }
                inputs.push(TargetInput::new(
                    Outpoint::new(identity(parent), 0).map_err(|_| Refusal::Transaction)?,
                    u32::MAX,
                ));
                witnesses.push(InputWitness::new(vec![
                    self.descriptor.leaf_bytes.clone(),
                    self.descriptor.control.bytes.clone(),
                ]));
            }
        }
        inputs.push(TargetInput::new(coin_outpoint(coin)?, u32::MAX));
        witnesses.push(InputWitness::new(vec![]));
        transaction(inputs, outputs, witnesses)
    }
    fn next(&self) -> Result<Option<OperationStep>, Refusal> {
        let Some(&count) = COUNTS.get(self.row) else {
            return Ok(None);
        };
        let required = if count == 0 { 1 } else { count + 1 };
        let subject = if self.coins.len() < required {
            OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: u8::try_from((required - self.coins.len()).min(255))
                    .map_err(|_| Refusal::Funding)?,
                amount_per_sponsor_output: SPONSOR_VALUE,
            }))
        } else if let Some(tx) = &self.signed {
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: tx.encode(),
            }))
        } else {
            let tx = self.build()?;
            OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                finalized_transaction: tx.encode(),
                sponsor_input_index: u16::try_from(tx.inputs().len() - 1)
                    .map_err(|_| Refusal::Transaction)?,
                sponsor_outpoint: self.sponsor()?.outpoint.clone(),
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            }))
        };
        Ok(Some(OperationStep::new(
            &format!("keyed-scaling-{}", self.exchanges.len()),
            subject,
        )))
    }
    fn settle(
        &mut self,
        step: &OperationStep,
        response: &NativeOperationResponse,
    ) -> Result<(), Refusal> {
        if response.schema != NATIVE_PROTOCOL_SCHEMA
            || &response.case != step.case()
            || response.validate_shape().is_err()
        {
            return Err(Refusal::Response);
        }
        match step.subject() {
            OperationSubject::SponsorFunding(request) => {
                if response.observed_layer != ObservedOutcomeLayer::Accepted
                    || response.funded_outputs.len() != usize::from(request.sponsor_outputs)
                {
                    return Err(Refusal::Funding);
                }
                let first = response.funded_outputs.first().ok_or(Refusal::Funding)?;
                let fact = coin_output(first)?;
                if fact.value != SPONSOR_VALUE
                    || fact.program.len() != 22
                    || fact.program[..2] != [0, 20]
                {
                    return Err(Refusal::Funding);
                }
                let binding = (first.asset.clone(), first.script.clone());
                if self.asset_program.as_ref().is_some_and(|p| p != &binding) {
                    return Err(Refusal::Funding);
                }
                for (i, coin) in response.funded_outputs.iter().enumerate() {
                    if coin.outpoint.txid != first.outpoint.txid
                        || coin.outpoint.vout as usize != i
                        || coin_output(coin)? != fact
                        || self
                            .exchanges
                            .iter()
                            .flat_map(|(_, r)| &r.funded_outputs)
                            .any(|prior| prior.outpoint == coin.outpoint)
                    {
                        return Err(Refusal::Funding);
                    }
                    coin_outpoint(coin)?;
                }
                self.asset_program = Some(binding);
                self.coins.extend_from_slice(&response.funded_outputs);
            }
            OperationSubject::SponsorSigning(request) => {
                if response
                    .sponsor_witness
                    .first()
                    .is_none_or(|s| s.len() > SPONSOR_SIGNATURE_BYTES)
                {
                    return Err(Refusal::Signing);
                }
                self.signed = Some(apply_birth_signature(request, response, self.sponsor()?)?);
            }
            OperationSubject::Submission(request) => {
                let source = self.source();
                let (policy, _) = detail(response).map_err(|_| Refusal::Policy)?;
                if self.policy.as_ref().is_some_and(|p| p != &policy) {
                    return Err(Refusal::Policy);
                }
                self.policy = Some(policy);
                let signing = self.exchanges.last().cloned().ok_or(Refusal::Replay)?;
                let record = ScalingRecord {
                    row: self.row,
                    preparation: source.then_some(self.parents.len()),
                    signing,
                    submission: (step.clone(), response.clone()),
                };
                if source
                    && classify(&record, self.policy.as_ref().ok_or(Refusal::Policy)?)
                        != ScalingClass::RelayMined
                {
                    return Err(Refusal::Readback);
                }
                self.records.push(record);
                if source {
                    self.parents.push(
                        TargetTransaction::decode(
                            &response
                                .mined_readback
                                .as_ref()
                                .ok_or(Refusal::Readback)?
                                .raw_transaction,
                        )
                        .map_err(|_| Refusal::Transaction)?,
                    );
                }
                // The bytes used for classification and the mined handle are exactly
                // those submitted, independently checked by check_readback.
                if request.transaction_bytes
                    != self.signed.as_ref().ok_or(Refusal::Replay)?.encode()
                {
                    return Err(Refusal::Replay);
                }
                self.signed = None;
                if !source || COUNTS[self.row] == 0 {
                    self.row += 1;
                    self.coins.clear();
                    self.parents.clear();
                }
            }
            _ => return Err(Refusal::Replay),
        }
        Ok(())
    }
    /// Conservative result for a retained record, keeping policy and consensus apart.
    #[must_use]
    pub fn classification(&self, record: &ScalingRecord) -> ScalingClass {
        self.policy
            .as_ref()
            .map_or(ScalingClass::Open, |p| classify(record, p))
    }
    /// Eligibility requires default-relay controls and a classified weight neighbour.
    #[must_use]
    pub fn eligible(&self) -> bool {
        self.complete().is_ok()
            && self.records.iter().all(|r| {
                let c = self.classification(r);
                if r.preparation.is_none() && COUNTS[r.row] == PROPOSED_MAXIMUM + 1 {
                    matches!(
                        c,
                        ScalingClass::WeightPolicyRefusedBlockMined
                            | ScalingClass::WeightPolicyRefused
                            | ScalingClass::ConsensusWeightRefused
                    )
                } else {
                    c == ScalingClass::RelayMined
                }
            })
    }
    /// Compact report; full responses live only in the wire journal, not debug text.
    #[must_use]
    pub fn report_json(&self) -> Value {
        let costs: Vec<_> = COUNTS.iter().enumerate().map(|(row, count)| {
            let sources: Vec<_> = self.records.iter().filter(|r| r.row == row && r.preparation.is_some()).collect();
            let preparation_weight = sources.iter().map(|r| r.submission.1.resources.transaction_weight).collect::<Option<Vec<_>>>().map(|weights| weights.into_iter().sum::<u64>());
            let merge = self.records.iter().find(|r| r.row == row && r.preparation.is_none());
            json!({"covenant_inputs": count, "source_count": sources.len(), "source_total_node_weight": preparation_weight, "merge_node_weight": merge.and_then(|r| r.submission.1.resources.transaction_weight), "merge_worst_serialized_prediction": (*count > 0).then_some(1077 + INPUT_WEIGHT * *count as u64 + if *count >= 252 { 8 } else { 0 })})
        }).collect();
        json!({"counts": COUNTS, "source_preparations": SOURCE_COUNT, "exchange_count": self.exchanges.len(), "costs": costs, "absent": ["native stack peaks", "native largest element", "validation budget", "opcode timing"], "constructor_n_plus_one": "unsupported: keyed descriptor has no input-count bound; consensus check_weight is not this policy bound", "records": self.records.iter().map(|r| json!({"row":r.row,"preparation":r.preparation,"signing_case":r.signing.0.case(),"submission_case":r.submission.0.case(),"class":self.classification(r),"node_weight":r.submission.1.resources.transaction_weight,"sponsor_signature_bytes":r.signing.1.sponsor_witness.first().map(Vec::len)})).collect::<Vec<_>>(), "admitted":false})
    }
}

fn classify(record: &ScalingRecord, policy: &Policy) -> ScalingClass {
    let read = || -> Option<ScalingClass> {
        let OperationSubject::Submission(request) = record.submission.0.subject() else {
            return None;
        };
        let r = &record.submission.1;
        if &detail(r).ok()?.0 != policy {
            return None;
        }
        let tx = TargetTransaction::decode(&request.transaction_bytes).ok()?;
        let weight = r.resources.transaction_weight?;
        if weight != tx.weight() {
            return None;
        }
        let fee = if record.preparation.is_some() {
            FIXTURE_FEE
        } else {
            MERGE_FEE
        };
        if fee < reported_floor(policy, weight).ok()? {
            return None;
        }
        let d: Value = serde_json::from_str(r.observed_detail.as_deref()?).ok()?;
        if d["detail"]["txid"] != identity(&tx).to_target_display() {
            return None;
        }
        let neighbour =
            record.preparation.is_none() && *COUNTS.get(record.row)? == PROPOSED_MAXIMUM + 1;
        if d["route"] == "relay-and-template" && d["relay_allowed"] == true {
            check_readback(&request.transaction_bytes, r).ok()?;
            return Some(ScalingClass::RelayMined);
        }
        if !neighbour
            || weight < STANDARD_WEIGHT
            || d["relay_allowed"] != false
            || d["relay_detail"] != "tx-size"
        {
            return None;
        }
        if d["route"] == "direct-block" {
            check_readback(&request.transaction_bytes, r).ok()?;
            return Some(ScalingClass::WeightPolicyRefusedBlockMined);
        }
        if d["route"] != "refused" || r.mined_readback.is_some() || r.accepted_txid.is_some() {
            return None;
        }
        if r.observed_layer == ObservedOutcomeLayer::ConsensusRejectionBeforeScript
            && d["detail"]["verdict_reason"] == "bad-txns-oversize"
        {
            return Some(ScalingClass::ConsensusWeightRefused);
        }
        // The exact relay reason attributes only policy. Any block diagnostic is
        // retained raw; this variant makes no claim about whether a block ran.
        if r.observed_layer == ObservedOutcomeLayer::RelayPolicyRejection {
            return Some(ScalingClass::WeightPolicyRefused);
        }
        None
    };
    read().unwrap_or(ScalingClass::Open)
}

impl TargetOperationPlanner for ScalingPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let _span = tracing::debug_span!(
            "keyed_scaling_step",
            row = self.row,
            exchange = self.exchanges.len(),
            prepared = self.parents.len()
        )
        .entered();
        let result = (|| {
            match (self.pending.take(), previous) {
                (None, None) if self.exchanges.is_empty() => {}
                (Some(step), Some((case, response))) if step.case() == case => {
                    self.settle(&step, response)?;
                    self.exchanges.push((step, response.clone()));
                }
                _ => return Err(Refusal::Replay),
            }
            let next = self.next()?;
            self.pending.clone_from(&next);
            tracing::debug!(complete = next.is_none(), "scaling step settled");
            Ok(next)
        })();
        result.map_err(|reason| {
            self.refusal = Some(reason);
            tracing::debug!(?reason, "scaling stopped");
            PlanRefused
        })
    }
}

/// Complete wire-JSON capture bound to the fixed range and descriptor.
#[must_use]
pub fn scaling_capture_json(
    capture: &NativeOperationCapture,
    descriptor: &KeyedConstructorDescriptor,
) -> Value {
    let mut wire = continuation_capture_json(capture);
    wire["scaling_wire_schema"] = json!(1);
    wire["descriptor"] = json!(descriptor);
    wire["counts"] = json!(COUNTS);
    wire
}
/// Reconstruct all requests from public data and complete responses, without debug text.
/// # Errors
/// Refuses changed counts, descriptor, responses, links, order or incomplete replay.
pub fn replay_scaling_capture(
    target: &ReviewedElementsTapscriptDefinition,
    journal: &Value,
) -> Result<ScalingPlanner, Refusal> {
    let _span = tracing::info_span!("replay_keyed_scaling").entered();
    if journal["scaling_wire_schema"] != 1
        || journal["terminal"] != "complete"
        || journal["counts"] != json!(COUNTS)
    {
        return Err(Refusal::Replay);
    }
    let descriptor: KeyedConstructorDescriptor =
        serde_json::from_value(journal["descriptor"].clone()).map_err(|_| Refusal::Leaf)?;
    reconstruct_keyed(target, &descriptor, &PublicKeyedCurve).map_err(|_| Refusal::Leaf)?;
    let planner = replay_operations(
        ScalingPlanner::new(target, descriptor.semantic_key)?,
        &decode_wire_operations(journal)?,
        ScalingPlanner::refusal,
    )?;
    planner.complete()?;
    tracing::info!(
        exchanges = planner.exchanges.len(),
        eligible = planner.eligible(),
        "scaling replay completed"
    );
    Ok(planner)
}
