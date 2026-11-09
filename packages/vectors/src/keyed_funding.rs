//! Finite funding/standardness staging, not a measured floor or admission.
//!
//! Reuses constructor, sponsor signing, readback and wire replay machinery.
//! Each point has fresh sponsor coins; source parents come only from mined bytes.
use crate::keyed_birth_native::{
    BIRTH_VALUE, BirthNativeRefusal as Refusal, FIXTURE_FEE, SPONSOR_VALUE, apply_birth_signature,
    coin_outpoint, coin_output, identity, output, replay_operations, transaction,
};
use crate::keyed_constructor::{
    KeyedConstructorDescriptor, PublicKeyedCurve, construct_keyed, reconstruct_keyed,
};
use crate::keyed_continuation_native::{continuation_capture_json, decode_wire_operations};
use crate::maturity_native::sponsored::{Policy, check_readback, detail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tapscript::keyed_constructor::KeyedSemanticKey;
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::executor::{
    NativeOperationCapture, OperationStep, PlanRefused, TargetOperationPlanner,
};
use target_elements_conformance::protocol::{
    FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireSighashProfile,
};
use transaction::bytes::{InputWitness, Outpoint, TargetInput, TargetTransaction};

/// Frozen satoshi grid, including a zero consensus-positivity control.
pub const HANDLE_POINTS: [u64; 11] = [0, 1, 2, 10, 100, 299, 300, 301, 999, 1000, 10_000];
/// Frozen offsets from the corresponding control's reported relay/mempool floor.
/// Negative results clamp at zero; duplicate points are retained, never resampled.
pub const FEE_OFFSETS: [i64; 5] = [-2, -1, 0, 1, 2];
/// Complete schedule, including setup, signing and submission.
pub const EXCHANGE_COUNT: usize = 146;
/// Transaction role; each role has its own signed-weight floor anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FundingRole {
    Source,
    Merge,
}
/// One immutable point coordinate. Dependent change always balances the asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FundingPoint {
    Control,
    Handle(u64),
    FeeOffset(i64),
    OrdinaryOutput,
    Redistributed,
}
/// Finite subject identity, fixed before any executor call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FundingSubject {
    /// SOURCE construction or MERGE of a mined birth-zero handle.
    pub role: FundingRole,
    /// Changed coordinate relative to the funded control.
    pub point: FundingPoint,
}
/// Immutable order: SOURCE/MERGE controls, paired handle points, paired fee
/// offsets, then ordinary-output and redistribution MERGE controls.
#[must_use]
pub fn funding_subjects() -> Vec<FundingSubject> {
    let mut rows = Vec::new();
    for point in std::iter::once(FundingPoint::Control)
        .chain(HANDLE_POINTS.into_iter().map(FundingPoint::Handle))
        .chain(FEE_OFFSETS.into_iter().map(FundingPoint::FeeOffset))
    {
        for role in [FundingRole::Source, FundingRole::Merge] {
            rows.push(FundingSubject { role, point });
        }
    }
    for point in [FundingPoint::OrdinaryOutput, FundingPoint::Redistributed] {
        rows.push(FundingSubject {
            role: FundingRole::Merge,
            point,
        });
    }
    rows
}

/// Conservative separation of ordinary relay and separately attempted consensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FundingOutcome {
    /// Ordinary relay and template inclusion, exact mined bytes.
    RelayMined,
    /// Relay refused; a separate direct block accepted these same bytes.
    RelayRefusedBlockMined,
    /// Explicit block-validation refusal after relay refusal.
    ConsensusRefused,
    /// Ambiguous, missing, unsupported, script or infrastructure result.
    Open,
}
/// Exact complete exchanges plus numeric construction coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundingRecord {
    /// Fixed subject, not an observed interpretation.
    pub subject: FundingSubject,
    /// Output-zero amount in satoshis.
    pub handle: u64,
    /// Explicit fee in satoshis.
    pub fee: u64,
    /// Complete signing request and response.
    pub signing: (OperationStep, NativeOperationResponse),
    /// Complete submission request and response with policy and block evidence.
    pub submission: (OperationStep, NativeOperationResponse),
}
impl FundingRecord {
    /// Lossless report projection; replay uses the full campaign wire journal.
    #[must_use]
    pub fn capture_json(&self) -> Value {
        json!({"subject": self.subject, "handle": self.handle, "fee": self.fee,
            "signing_request": self.signing.0.subject(), "signing_response": self.signing.1,
            "submission_request": self.submission.0.subject(), "submission_response": self.submission.1})
    }
}

/// The reported floor for an exact node-read weight, rounded upward in satoshis.
/// This is an arithmetic comparison point, not an observed admission threshold.
/// # Errors
/// Refuses malformed policy, absent/zero weight and arithmetic overflow.
pub fn reported_floor(policy: &Policy, weight: u64) -> Result<u64, Refusal> {
    Policy::parse(policy.0.clone()).map_err(|_| Refusal::Policy)?;
    let rate = ["relay", "mempool"]
        .into_iter()
        .map(|key| {
            let text = policy.0[key].as_str().ok_or(Refusal::Policy)?;
            let (whole, fraction) = text.split_once('.').ok_or(Refusal::Policy)?;
            whole
                .parse::<u64>()
                .ok()
                .and_then(|n| n.checked_mul(100_000_000))
                .and_then(|n| fraction.parse::<u64>().ok().and_then(|f| n.checked_add(f)))
                .ok_or(Refusal::Policy)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .ok_or(Refusal::Policy)?;
    if weight == 0 {
        return Err(Refusal::Policy);
    }
    weight
        .checked_add(3)
        .map(|w| w / 4)
        .and_then(|v| v.checked_mul(rate))
        .and_then(|n| n.checked_add(999))
        .map(|n| n / 1000)
        .ok_or(Refusal::Policy)
}

fn observation(
    bytes: &[u8],
    response: &NativeOperationResponse,
    policy: &Policy,
) -> FundingOutcome {
    let classify = || -> Option<FundingOutcome> {
        let (actual, _) = detail(response).ok()?;
        if &actual != policy || response.resources.transaction_weight? == 0 {
            return None;
        }
        let value: Value = serde_json::from_str(response.observed_detail.as_deref()?).ok()?;
        let tx = TargetTransaction::decode(bytes).ok()?;
        if value["detail"]["txid"] != identity(&tx).to_target_display() {
            return None;
        }
        match value["route"].as_str()? {
            "relay-and-template" | "direct-block" => {
                check_readback(bytes, response).ok()?;
                Some(if value["route"] == "relay-and-template" {
                    FundingOutcome::RelayMined
                } else {
                    FundingOutcome::RelayRefusedBlockMined
                })
            }
            "refused"
                if response.observed_layer
                    == ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                    && response.accepted_txid.is_none()
                    && response.mined_readback.is_none()
                    && value["detail"]["verdict_reason"]
                        .as_str()
                        .is_some_and(|s| !s.is_empty())
                    && value["detail"]["block_script_error"].is_string() =>
            {
                Some(FundingOutcome::ConsensusRefused)
            }
            _ => None,
        }
    };
    classify().unwrap_or(FundingOutcome::Open)
}

/// Stateful schedule using the existing executor protocol, without an adapter fork.
#[derive(Clone, Debug)]
pub struct FundingPlanner {
    descriptor: KeyedConstructorDescriptor,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    records: Vec<FundingRecord>,
    coins: Vec<FundedOutput>,
    parent: Option<TargetTransaction>,
    signed: Option<TargetTransaction>,
    phase: usize,
    policy: Option<Policy>,
    asset_program: Option<(String, String)>,
    anchors: [Option<u64>; 2],
    refusal: Option<Refusal>,
}
impl FundingPlanner {
    /// Start the fixed schedule with a canonical constructor and no observations.
    /// # Errors
    /// Refuses constructor failures; no native execution takes place here.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        key: [u8; 32],
    ) -> Result<Self, Refusal> {
        let _span = tracing::debug_span!("keyed_funding_new").entered();
        let descriptor = construct_keyed(target, KeyedSemanticKey::new(key), &PublicKeyedCurve)
            .map_err(|_| Refusal::Leaf)?;
        Ok(Self {
            descriptor,
            pending: None,
            exchanges: vec![],
            records: vec![],
            coins: vec![],
            parent: None,
            signed: None,
            phase: 0,
            policy: None,
            asset_program: None,
            anchors: [None; 2],
            refusal: None,
        })
    }
    /// Constructor bound into the capture.
    #[must_use]
    pub const fn descriptor(&self) -> &KeyedConstructorDescriptor {
        &self.descriptor
    }
    /// Complete exchange journal, including all parent setups.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }
    /// Frozen point records in schedule order.
    #[must_use]
    pub fn records(&self) -> &[FundingRecord] {
        &self.records
    }
    /// Terminal construction or replay refusal, never a measured floor.
    #[must_use]
    pub const fn refusal(&self) -> Option<Refusal> {
        self.refusal
    }
    /// Check the finite census without asserting any native result.
    /// # Errors
    /// Refuses a partial or stopped schedule.
    pub fn complete(&self) -> Result<(), Refusal> {
        if let Some(r) = self.refusal {
            return Err(r);
        }
        if self.records.len() != funding_subjects().len()
            || self.exchanges.len() != EXCHANGE_COUNT
            || self.pending.is_some()
        {
            return Err(Refusal::Replay);
        }
        Ok(())
    }
    fn coordinates(&self, row: FundingSubject) -> Result<(u64, u64, u64), Refusal> {
        let handle = match row.point {
            FundingPoint::Handle(n) => n,
            FundingPoint::Redistributed => 1000,
            _ => BIRTH_VALUE,
        };
        let fee = match row.point {
            FundingPoint::FeeOffset(offset) => self.anchors
                [usize::from(row.role == FundingRole::Merge)]
            .ok_or(Refusal::Policy)?
            .saturating_add_signed(offset),
            _ => FIXTURE_FEE,
        };
        let ordinary = match row.point {
            FundingPoint::OrdinaryOutput => 1000,
            FundingPoint::Redistributed => 10_000,
            _ => 0,
        };
        Ok((handle, fee, ordinary))
    }
    fn build(
        &self,
        source: bool,
        handle: u64,
        fee: u64,
        ordinary: u64,
    ) -> Result<TargetTransaction, Refusal> {
        let _span =
            tracing::debug_span!("keyed_funding_build", source, handle, fee, ordinary).entered();
        let coin = self
            .coins
            .get(usize::from(!source))
            .ok_or(Refusal::Funding)?;
        let fact = coin_output(coin)?;
        let total = fact
            .value
            .checked_add(if source { 0 } else { BIRTH_VALUE })
            .ok_or(Refusal::Funding)?;
        let change = total
            .checked_sub(handle)
            .and_then(|n| n.checked_sub(fee))
            .and_then(|n| n.checked_sub(ordinary))
            .filter(|n| *n > 0)
            .ok_or(Refusal::Funding)?;
        let mut outputs = vec![output(
            &fact,
            handle,
            self.descriptor.output_program.clone(),
        )];
        if ordinary > 0 {
            let mut program = fact.program.clone();
            let last = program.last_mut().ok_or(Refusal::Funding)?;
            *last ^= 1;
            outputs.push(output(&fact, ordinary, program));
        }
        outputs.push(output(&fact, change, fact.program.clone()));
        outputs.push(output(&fact, fee, vec![]));
        let mut inputs = Vec::new();
        let mut witnesses = Vec::new();
        if !source {
            let parent = self.parent.as_ref().ok_or(Refusal::Readback)?;
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
        inputs.push(TargetInput::new(coin_outpoint(coin)?, u32::MAX));
        witnesses.push(InputWitness::new(vec![]));
        transaction(inputs, outputs, witnesses)
    }
    fn next(&self) -> Result<Option<OperationStep>, Refusal> {
        let Some(row) = funding_subjects().get(self.records.len()).copied() else {
            return Ok(None);
        };
        let subject = match self.phase {
            0 => OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: if row.role == FundingRole::Source {
                    1
                } else {
                    2
                },
                amount_per_sponsor_output: SPONSOR_VALUE,
            })),
            1 | 3 => {
                let source = self.phase == 1;
                let (handle, fee, ordinary) = if source && row.role == FundingRole::Merge {
                    (BIRTH_VALUE, FIXTURE_FEE, 0)
                } else {
                    self.coordinates(row)?
                };
                let tx = self.build(source, handle, fee, ordinary)?;
                let coin = &self.coins[usize::from(!source)];
                OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                    finalized_transaction: tx.encode(),
                    sponsor_input_index: u16::from(!source),
                    sponsor_outpoint: coin.outpoint.clone(),
                    sighash_profile: WireSighashProfile::AllInputsAllOutputs,
                }))
            }
            2 | 4 => OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self.signed.as_ref().ok_or(Refusal::Replay)?.encode(),
            })),
            _ => return Err(Refusal::Replay),
        };
        Ok(Some(OperationStep::new(
            &format!("keyed-funding-{}", self.exchanges.len()),
            subject,
        )))
    }
    fn accept_funding(
        &mut self,
        count: u8,
        response: &NativeOperationResponse,
    ) -> Result<(), Refusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted
            || response.funded_outputs.len() != usize::from(count)
        {
            return Err(Refusal::Funding);
        }
        let first = response.funded_outputs.first().ok_or(Refusal::Funding)?;
        let expected = coin_output(first)?;
        if expected.value != SPONSOR_VALUE
            || expected.program.len() != 22
            || expected.program[..2] != [0, 20]
        {
            return Err(Refusal::Funding);
        }
        for (index, coin) in response.funded_outputs.iter().enumerate() {
            if coin.outpoint.vout as usize != index
                || coin.outpoint.txid != first.outpoint.txid
                || coin_output(coin)? != expected
            {
                return Err(Refusal::Funding);
            }
            coin_outpoint(coin)?;
            if self
                .exchanges
                .iter()
                .flat_map(|(_, r)| &r.funded_outputs)
                .any(|prior| prior.outpoint == coin.outpoint)
            {
                return Err(Refusal::Funding);
            }
        }
        let binding = (first.asset.clone(), first.script.clone());
        if self
            .asset_program
            .as_ref()
            .is_some_and(|prior| prior != &binding)
        {
            return Err(Refusal::Funding);
        }
        self.asset_program = Some(binding);
        self.coins.clone_from(&response.funded_outputs);
        self.parent = None;
        self.phase = 1;
        Ok(())
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
        let row = funding_subjects()[self.records.len()];
        match step.subject() {
            OperationSubject::SponsorFunding(request) => {
                self.accept_funding(request.sponsor_outputs, response)?;
            }
            OperationSubject::SponsorSigning(request) => {
                self.signed = Some(apply_birth_signature(
                    request,
                    response,
                    &self.coins[usize::from(self.phase == 3)],
                )?);
                self.phase += 1;
            }
            OperationSubject::Submission(request) => {
                let setup = row.role == FundingRole::Merge && self.phase == 2;
                if setup || row.point == FundingPoint::Control {
                    let (policy, _) = detail(response).map_err(|_| Refusal::Policy)?;
                    if self.policy.as_ref().is_some_and(|p| p != &policy) {
                        return Err(Refusal::Policy);
                    }
                    if observation(&request.transaction_bytes, response, &policy)
                        != FundingOutcome::RelayMined
                    {
                        return Err(Refusal::Readback);
                    }
                    self.policy = Some(policy.clone());
                    if !setup {
                        let floor = reported_floor(
                            &policy,
                            response
                                .resources
                                .transaction_weight
                                .ok_or(Refusal::Policy)?,
                        )?;
                        if FIXTURE_FEE <= floor {
                            return Err(Refusal::Policy);
                        }
                        self.anchors[usize::from(row.role == FundingRole::Merge)] = Some(floor);
                    }
                }
                if setup {
                    self.parent = Some(
                        TargetTransaction::decode(
                            &response
                                .mined_readback
                                .as_ref()
                                .ok_or(Refusal::Readback)?
                                .raw_transaction,
                        )
                        .map_err(|_| Refusal::Readback)?,
                    );
                    self.phase = 3;
                } else {
                    let (handle, fee, _) = self.coordinates(row)?;
                    self.records.push(FundingRecord {
                        subject: row,
                        handle,
                        fee,
                        signing: self.exchanges.last().ok_or(Refusal::Replay)?.clone(),
                        submission: (step.clone(), response.clone()),
                    });
                    self.phase = 0;
                }
            }
            _ => return Err(Refusal::Response),
        }
        Ok(())
    }
    /// Classify a point only under the original frozen policy and complete census.
    #[must_use]
    pub fn outcome(&self, index: usize) -> FundingOutcome {
        if self.complete().is_err() {
            return FundingOutcome::Open;
        }
        let Some(row) = self.records.get(index) else {
            return FundingOutcome::Open;
        };
        let Some(policy) = &self.policy else {
            return FundingOutcome::Open;
        };
        let OperationSubject::Submission(request) = row.submission.0.subject() else {
            return FundingOutcome::Open;
        };
        observation(&request.transaction_bytes, &row.submission.1, policy)
    }
    /// Sampling result, not a universal dust/fee law. Only monotone, complete
    /// observations yield a bracket; gaps are reported, never interpolated.
    #[must_use]
    pub fn boundary(&self, role: FundingRole, fees: bool) -> Value {
        let rows: Vec<_> = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.subject.role == role
                    && matches!(
                        (fees, r.subject.point),
                        (true, FundingPoint::FeeOffset(_)) | (false, FundingPoint::Handle(1..))
                    )
            })
            .map(|(i, r)| {
                let detail: Value = r
                    .submission
                    .1
                    .observed_detail
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or(Value::Null);
                let value = if fees { r.fee } else { r.handle };
                let outcome = self.outcome(i);
                let reason = detail["relay_detail"].as_str().unwrap_or("");
                let attributable = if fees {
                    reason == "min relay fee not met" || reason == "mempool min fee not met"
                } else {
                    reason == "dust"
                };
                (value, outcome, attributable)
            })
            .collect();
        let mut last_refused = None;
        let mut first_accepted = None;
        let mut open = self.complete().is_err();
        for (value, outcome, attributable) in &rows {
            match outcome {
                FundingOutcome::RelayMined => {
                    first_accepted.get_or_insert(*value);
                }
                FundingOutcome::RelayRefusedBlockMined
                    if *attributable && first_accepted.is_none() =>
                {
                    last_refused = Some(*value);
                }
                _ => open = true,
            }
        }
        open |= last_refused
            .zip(first_accepted)
            .is_some_and(|(a, b)| a >= b);
        let reading = if open {
            "open: incomplete, nonmonotone or unattributed outcomes"
        } else if first_accepted.is_none() {
            "no admitted point in tested range"
        } else if last_refused.is_none() {
            "no discovered threshold in tested range"
        } else {
            "sampled refusal/admission bracket only"
        };
        json!({"role": role, "fee_points": fees, "points": rows, "reading": reading, "last_refused": last_refused, "first_accepted": first_accepted, "adjacent_satoshis": last_refused.zip(first_accepted).is_some_and(|(a,b)| a.checked_add(1) == Some(b)), "no_interpolation": true})
    }
    /// Capture projection retaining anchors and per-point exact-weight floors.
    #[must_use]
    pub fn report_json(&self) -> Value {
        json!({"anchors_source_merge": self.anchors, "asset_program": self.asset_program, "policy": self.policy.as_ref().map(|p| &p.0), "rows": self.records.iter().enumerate().map(|(i,r)| json!({"record": r.capture_json(), "outcome": self.outcome(i), "exact_weight_floor": self.policy.as_ref().and_then(|p| r.submission.1.resources.transaction_weight.and_then(|w| reported_floor(p,w).ok()))})).collect::<Vec<_>>(), "boundaries": [self.boundary(FundingRole::Source,false),self.boundary(FundingRole::Merge,false),self.boundary(FundingRole::Source,true),self.boundary(FundingRole::Merge,true)], "admitted": false})
    }
}
impl TargetOperationPlanner for FundingPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let _span = tracing::debug_span!(
            "keyed_funding_step",
            exchange_count = self.exchanges.len(),
            subject_count = self.records.len()
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
            tracing::debug!(complete = next.is_none(), "funding step settled");
            Ok(next)
        })();
        result.map_err(|reason| {
            self.refusal = Some(reason);
            tracing::debug!(?reason, "funding campaign stopped");
            PlanRefused
        })
    }
}
/// Complete shared wire carrier, augmented with the immutable funding recipe/grid.
#[must_use]
pub fn funding_capture_json(
    capture: &NativeOperationCapture,
    descriptor: &KeyedConstructorDescriptor,
) -> Value {
    let mut journal = continuation_capture_json(capture);
    journal["funding_wire_schema"] = json!(1);
    journal["descriptor"] = json!(descriptor);
    journal["subjects"] = json!(funding_subjects());
    journal["handle_points"] = json!(HANDLE_POINTS);
    journal["fee_offsets"] = json!(FEE_OFFSETS);
    journal
}
/// Reconstruct the recipe and every request from complete wire JSON alone.
/// # Errors
/// Refuses altered grids, descriptors, missing/surplus/reordered exchanges and echoes.
pub fn replay_funding_capture(
    target: &ReviewedElementsTapscriptDefinition,
    journal: &Value,
) -> Result<FundingPlanner, Refusal> {
    let _span = tracing::debug_span!("replay_keyed_funding").entered();
    if journal["funding_wire_schema"] != 1
        || journal["terminal"] != "complete"
        || journal["subjects"] != json!(funding_subjects())
        || journal["handle_points"] != json!(HANDLE_POINTS)
        || journal["fee_offsets"] != json!(FEE_OFFSETS)
    {
        return Err(Refusal::Replay);
    }
    let descriptor: KeyedConstructorDescriptor =
        serde_json::from_value(journal["descriptor"].clone()).map_err(|_| Refusal::Leaf)?;
    reconstruct_keyed(target, &descriptor, &PublicKeyedCurve).map_err(|_| Refusal::Leaf)?;
    let planner = replay_operations(
        FundingPlanner::new(target, descriptor.semantic_key)?,
        &decode_wire_operations(journal)?,
        FundingPlanner::refusal,
    )?;
    planner.complete()?;
    Ok(planner)
}
