//! Focused staging for missing cost dimensions, never a covenant admission.
//!
//! An isolated continuation control supplies its own transaction weight. A
//! separate drop/true leaf consumes one argument: one byte is the control and
//! 521 bytes is the oversized subject. Only argument length changes, with witness
//! serialization, wtxid, weight and response as collateral. It tests resource
//! admission before execution, not birth or continuation. Relay and consensus
//! outcomes remain separate; a refusal is never attributed to a covenant clause.
//! The oversized subject is submitted before the same-coin control. An altered
//! resource report is an offline validator subject, not a second node verdict.

use serde_json::{Value, json};
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::constructor::{
    UNSPENDABLE_INTERNAL_KEY,
    tree::{FixtureTapTree, construct},
};
use target_elements_conformance::executor::{
    NativeOperationCapture, OperationStep, PlanRefused, TargetOperationPlanner,
};
use target_elements_conformance::protocol::{
    FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireSighashProfile,
};
use transaction::bytes::{InputWitness, TargetTransaction};

use crate::keyed_birth::BirthSubjectId;
use crate::keyed_birth_native::{
    BirthLeaf, BirthNativeRefusal, SPONSOR_VALUE, apply_birth_signature, build_birth_parent,
    build_birth_spend, coin_outpoint, coin_output, replay_operations, transaction,
};
use crate::keyed_continuation_native::{continuation_capture_json, decode_wire_operations};
use crate::maturity_native::sponsored::check_readback;

/// The resource subject exceeds the consensus element maximum by one byte.
pub const OVERSIZED_ARGUMENT_BYTES: usize = 521;

/// Reconstruct an exact standalone leaf with the reviewed version and NUMS policy.
/// # Errors
/// Returns a leaf construction refusal.
pub fn cost_leaf(
    target: &ReviewedElementsTapscriptDefinition,
    resource_probe: bool,
) -> Result<BirthLeaf, BirthNativeRefusal> {
    let _span = tracing::debug_span!("cost_leaf", resource_probe).entered();
    let mut script = if resource_probe {
        vec![0x75]
    } else {
        tapscript::keyed_pattern::continue_first_fragment(target)
            .map_err(|_| BirthNativeRefusal::Leaf)?
            .encode(target)
    };
    script.push(0x51);
    let tree = FixtureTapTree::Leaf {
        version: target.definition().leaf_version().get(),
        script: script.clone(),
    };
    let output =
        construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &tree).map_err(|_| BirthNativeRefusal::Leaf)?;
    Ok(BirthLeaf {
        script,
        program: output.output_program().to_vec(),
        control_block: output.control_block().to_vec(),
    })
}

/// Three retained submissions in order; parents and funding are separate exchanges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostSubjectId {
    /// Exact standalone continuation plus truth, no key or birth fragment.
    IsolatedContinuation,
    /// Drop/true leaf with a 521-byte argument, targeting resource admission.
    OversizedWitness,
    /// Same resource leaf and same coin with one-byte argument.
    ResourceControl,
}

/// Construction/replay with two fresh two-output fundings and twelve exchanges.
#[derive(Clone, Debug)]
pub struct CostCampaignPlanner {
    leaves: [BirthLeaf; 2],
    coins: Vec<FundedOutput>,
    parent: Option<TargetTransaction>,
    signed: Option<TargetTransaction>,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    refusal: Option<BirthNativeRefusal>,
}
impl CostCampaignPlanner {
    /// Construct scripts locally; no observations or caller-supplied programs.
    /// # Errors
    /// Refuses unavailable leaf construction.
    pub fn new(target: &ReviewedElementsTapscriptDefinition) -> Result<Self, BirthNativeRefusal> {
        Ok(Self {
            leaves: [cost_leaf(target, false)?, cost_leaf(target, true)?],
            coins: vec![],
            parent: None,
            signed: None,
            pending: None,
            exchanges: vec![],
            refusal: None,
        })
    }
    /// Frozen requests and complete responses in execution order.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }
    /// Retained construction or replay refusal.
    #[must_use]
    pub const fn refusal(&self) -> Option<BirthNativeRefusal> {
        self.refusal
    }
    /// Complete staging, without interpreting a negative as an expected verdict.
    /// # Errors
    /// Refuses stopped or incomplete execution.
    pub fn complete(&self) -> Result<(), BirthNativeRefusal> {
        if self.exchanges.len() == 12 && self.refusal.is_none() {
            Ok(())
        } else {
            Err(self.refusal.unwrap_or(BirthNativeRefusal::Replay))
        }
    }
    /// Exact subject submission, absent until its response has been retained.
    #[must_use]
    pub fn subject(&self, id: CostSubjectId) -> Option<&(OperationStep, NativeOperationResponse)> {
        self.exchanges.get(match id {
            CostSubjectId::IsolatedContinuation => 4,
            CostSubjectId::OversizedWitness => 9,
            CostSubjectId::ResourceControl => 11,
        })
    }
    fn signing(tx: &TargetTransaction, index: u16, coin: &FundedOutput) -> OperationSubject {
        OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
            finalized_transaction: tx.encode(),
            sponsor_input_index: index,
            sponsor_outpoint: coin.outpoint.clone(),
            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
        }))
    }
    fn next(&self) -> Result<Option<OperationStep>, BirthNativeRefusal> {
        let n = self.exchanges.len();
        let leaf = &self.leaves[usize::from(n >= 5)];
        let subject = match n {
            12 => return Ok(None),
            0 | 5 => OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 2,
                amount_per_sponsor_output: SPONSOR_VALUE,
            })),
            1 | 6 => Self::signing(
                &build_birth_parent(leaf, &self.coins[0])?,
                0,
                &self.coins[0],
            ),
            2 | 4 | 7 | 9 | 11 => OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self
                    .signed
                    .as_ref()
                    .ok_or(BirthNativeRefusal::Replay)?
                    .encode(),
            })),
            3 | 8 | 10 => {
                let mut tx = build_birth_spend(
                    leaf,
                    self.parent.as_ref().ok_or(BirthNativeRefusal::Replay)?,
                    &self.coins[1],
                    BirthSubjectId::A0,
                )?;
                if n != 3 {
                    let mut witnesses = tx.witnesses().to_vec();
                    let mut stack =
                        vec![vec![0; if n == 8 { OVERSIZED_ARGUMENT_BYTES } else { 1 }]];
                    stack.extend_from_slice(witnesses[0].stack());
                    witnesses[0] = InputWitness::new(stack);
                    tx = transaction(tx.inputs().to_vec(), tx.outputs().to_vec(), witnesses)?;
                }
                Self::signing(&tx, 1, &self.coins[1])
            }
            _ => return Err(BirthNativeRefusal::Replay),
        };
        Ok(Some(OperationStep::new(
            &format!("keyed-cost-{n}"),
            subject,
        )))
    }
    fn settle(
        &mut self,
        step: &OperationStep,
        response: &NativeOperationResponse,
    ) -> Result<(), BirthNativeRefusal> {
        if response.schema != NATIVE_PROTOCOL_SCHEMA
            || &response.case != step.case()
            || response.validate_shape().is_err()
        {
            return Err(BirthNativeRefusal::Response);
        }
        let n = self.exchanges.len();
        match step.subject() {
            OperationSubject::SponsorFunding(_) => {
                let coins = &response.funded_outputs;
                if response.observed_layer != ObservedOutcomeLayer::Accepted || coins.len() != 2 {
                    return Err(BirthNativeRefusal::Funding);
                }
                let fact = coin_output(&coins[0])?;
                if coins[0].outpoint.vout != 0
                    || coins[1].outpoint.vout != 1
                    || coins[0].outpoint.txid != coins[1].outpoint.txid
                    || fact != coin_output(&coins[1])?
                    || fact.value != SPONSOR_VALUE
                    || fact.program.len() != 22
                    || fact.program[..2] != [0, 20]
                {
                    return Err(BirthNativeRefusal::Funding);
                }
                coin_outpoint(&coins[0])?;
                if n == 5 && self.coins[0].outpoint.txid == coins[0].outpoint.txid {
                    return Err(BirthNativeRefusal::Funding);
                }
                self.coins.clone_from(coins);
                self.parent = None;
            }
            OperationSubject::SponsorSigning(request) => {
                self.signed = Some(apply_birth_signature(
                    request,
                    response,
                    &self.coins[usize::from(n != 1 && n != 6)],
                )?);
            }
            OperationSubject::Submission(request) if n == 2 || n == 7 => {
                if response.observed_layer != ObservedOutcomeLayer::Accepted {
                    return Err(BirthNativeRefusal::Readback);
                }
                check_readback(&request.transaction_bytes, response)
                    .map_err(|_| BirthNativeRefusal::Readback)?;
                self.parent = Some(
                    TargetTransaction::decode(&request.transaction_bytes)
                        .map_err(|_| BirthNativeRefusal::Readback)?,
                );
            }
            OperationSubject::Submission(_) => {}
            _ => return Err(BirthNativeRefusal::Response),
        }
        Ok(())
    }
}
impl TargetOperationPlanner for CostCampaignPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let _span =
            tracing::debug_span!("keyed_cost_step", completed = self.exchanges.len()).entered();
        let result = (|| {
            match (self.pending.take(), previous) {
                (None, None) if self.exchanges.is_empty() => {}
                (Some(step), Some((case, response))) if step.case() == case => {
                    self.settle(&step, response)?;
                    self.exchanges.push((step, response.clone()));
                }
                _ => return Err(BirthNativeRefusal::Replay),
            }
            let next = self.next()?;
            self.pending.clone_from(&next);
            Ok(next)
        })();
        result.map_err(|refusal| {
            tracing::debug!(?refusal, "cost campaign stopped");
            self.refusal = Some(refusal);
            PlanRefused
        })
    }
}

/// Complete responses in wire JSON, reusing the common journal carrier.
#[must_use]
pub fn cost_capture_json(capture: &NativeOperationCapture) -> Value {
    let mut journal = continuation_capture_json(capture);
    journal["keyed_cost_wire_schema"] = json!(1);
    journal
}

/// Replay only wire JSON and deterministic construction, never debug text.
/// # Errors
/// Refuses changed links, missing fields, changed requests or incomplete execution.
pub fn replay_cost_capture(
    target: &ReviewedElementsTapscriptDefinition,
    journal: &Value,
) -> Result<CostCampaignPlanner, BirthNativeRefusal> {
    let _span = tracing::info_span!("replay_cost_capture").entered();
    if journal["keyed_cost_wire_schema"] != 1 || journal["terminal"] != "complete" {
        return Err(BirthNativeRefusal::Replay);
    }
    let planner = replay_operations(
        CostCampaignPlanner::new(target)?,
        &decode_wire_operations(journal)?,
        CostCampaignPlanner::refusal,
    )?;
    planner.complete()?;
    Ok(planner)
}

/// Exact mined control, independently of whether its report includes a weight.
#[must_use]
pub fn cost_control_mined(planner: &CostCampaignPlanner, id: CostSubjectId) -> bool {
    if id == CostSubjectId::OversizedWitness {
        return false;
    }
    let Some((step, response)) = planner.subject(id) else {
        return false;
    };
    let OperationSubject::Submission(request) = step.subject() else {
        return false;
    };
    response.observed_layer == ObservedOutcomeLayer::Accepted
        && check_readback(&request.transaction_bytes, response).is_ok()
}

/// Require the resource size refusal beside the exact mined one-byte control.
///
/// Relay policy and consensus evidence remain available in the raw response;
/// this names only the explicit size error, never a covenant equality site.
#[must_use]
pub fn oversized_resource_refused(planner: &CostCampaignPlanner) -> bool {
    if !cost_control_mined(planner, CostSubjectId::ResourceControl) {
        return false;
    }
    let Some((_, response)) = planner.subject(CostSubjectId::OversizedWitness) else {
        return false;
    };
    let Some(detail) = response
        .observed_detail
        .as_deref()
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
    else {
        return false;
    };
    response.observed_layer == ObservedOutcomeLayer::ScriptPathRejection
        && response.mined_readback.is_none()
        && response.accepted_txid.is_none()
        && detail["route"] == "refused"
        && detail["relay_allowed"] == false
        && detail["relay_detail"]
            == "mandatory-script-verify-flag-failed (Push value size limit exceeded)"
        && detail["detail"]["verdict_reason"] == detail["relay_detail"]
}

/// Observe an explicit witness-policy refusal beside the exact mined control.
///
/// This is a relay observation only. The raw response retains the subsequent
/// consensus route without converting its layer or reason into a size verdict.
#[must_use]
pub fn oversized_policy_refused(planner: &CostCampaignPlanner) -> bool {
    if !cost_control_mined(planner, CostSubjectId::ResourceControl) {
        return false;
    }
    let Some((_, response)) = planner.subject(CostSubjectId::OversizedWitness) else {
        return false;
    };
    let Some(detail) = response
        .observed_detail
        .as_deref()
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
    else {
        return false;
    };
    matches!(
        response.observed_layer,
        ObservedOutcomeLayer::ScriptPathRejection
            | ObservedOutcomeLayer::ConsensusRejectionBeforeScript
            | ObservedOutcomeLayer::RelayPolicyRejection
    ) && response.mined_readback.is_none()
        && response.accepted_txid.is_none()
        && detail["route"] == "refused"
        && detail["relay_allowed"] == false
        && detail["relay_detail"] == "bad-witness-nonstandard"
}
