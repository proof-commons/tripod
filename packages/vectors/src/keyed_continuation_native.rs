//! Key-free continuation staging and capture, never archive admission.
//!
//! One two-output sponsor funding creates a mined parent and reserves the second
//! coin for three negatives followed by their control. Birth is valid in every
//! row. Both leaf recipes are reconstructed under the fixed unspendable internal
//! key; there is no semantic-key separation claim. Fixture amounts are not floors.
//! Complete protocol responses belong in the capture's wire-JSON journal, not
//! in a report's debug rendering. Earlier-layer and ambiguous refusals stay open.

use std::collections::BTreeSet;

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
    FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, NativeOperationResponse,
    ObservedOutcomeLayer, OperationCaseId, OperationSubject, TargetSponsorFundingSubject,
    TargetSponsorSigningSubject, TargetSubmissionSubject, WireSighashProfile,
};
use transaction::bytes::{InputWitness, Outpoint, TargetInput, TargetTransaction};

use crate::keyed_birth::{BirthOutpoint, BirthOutput, Wave1TargetPacket};
use crate::keyed_birth_native::{
    BIRTH_VALUE, BirthLeaf, BirthNativeRefusal, FIXTURE_FEE, SPONSOR_VALUE, apply_birth_signature,
    build_birth_parent, coin_outpoint, coin_output, identity, output, replay_operations,
    transaction,
};
use crate::keyed_continuation::{
    ContinuationAttribution, ContinuationBoundary, ContinuationCampaignManifest,
    ContinuationCollateral, ContinuationPairContext, ContinuationSubject, ContinuationSubjectId,
};
use crate::maturity_native::sponsored::{check_readback, detail};

/// Birth, version and payload equality offsets in the primary key-free leaf.
pub const CONTINUATION_EQUALVERIFY_OFFSETS: [usize; 3] = [8, 15, 16];

/// Reconstructed primary and alternative single-leaf commitments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationLeaves {
    /// Born first, continue first, final true.
    pub primary: BirthLeaf,
    /// Same clauses, with a harmless empty push/drop before final true.
    pub alternative: BirthLeaf,
}

impl ContinuationLeaves {
    /// Construct both recipes, never accepting an externally supplied program.
    /// # Errors
    /// Returns `Leaf` for a typed schedule or commitment construction refusal.
    pub fn new(target: &ReviewedElementsTapscriptDefinition) -> Result<Self, BirthNativeRefusal> {
        let _span = tracing::debug_span!("keyed_continuation_leaves").entered();
        let mut prefix = tapscript::born_first_fragment(target)
            .map_err(|_| BirthNativeRefusal::Leaf)?
            .encode(target);
        prefix.extend(
            tapscript::keyed_pattern::continue_first_fragment(target)
                .map_err(|_| BirthNativeRefusal::Leaf)?
                .encode(target),
        );
        let commit = |alternative: bool| {
            let mut script = prefix.clone();
            if alternative {
                script.extend([0, 0x75]);
            }
            script.extend(
                tapscript::pattern::final_truth_fragment(target)
                    .map_err(|_| BirthNativeRefusal::Leaf)?
                    .encode(target),
            );
            let tree = FixtureTapTree::Leaf {
                version: target.definition().leaf_version().get(),
                script: script.clone(),
            };
            let built = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &tree)
                .map_err(|_| BirthNativeRefusal::Leaf)?;
            Ok(BirthLeaf {
                script,
                program: built.output_program().to_vec(),
                control_block: built.control_block().to_vec(),
            })
        };
        let leaves = Self {
            primary: commit(false)?,
            alternative: commit(true)?,
        };
        if leaves.primary.program == leaves.alternative.program {
            return Err(BirthNativeRefusal::Leaf);
        }
        Ok(leaves)
    }
}

/// Build a balanced parent: primary output zero, alternative output one,
/// sponsor change and explicit fee. The birth builder supplies the balance.
/// # Errors
/// Returns a funding or transaction refusal for an unsuitable sponsor coin.
pub fn build_continuation_parent(
    leaves: &ContinuationLeaves,
    coin: &FundedOutput,
) -> Result<TargetTransaction, BirthNativeRefusal> {
    let _span = tracing::debug_span!("build_continuation_parent").entered();
    let parent = build_birth_parent(&leaves.primary, coin)?;
    let mut outputs = parent.outputs().to_vec();
    outputs[1] = output(
        &coin_output(coin)?,
        BIRTH_VALUE,
        leaves.alternative.program.clone(),
    );
    transaction(
        parent.inputs().to_vec(),
        outputs,
        parent.witnesses().to_vec(),
    )
}

/// Build one exact child below safe covenant-form admission, so the version-zero
/// mutant can reach the native target. Every child spends parent output zero.
/// # Errors
/// Returns a funding/transaction refusal if the parent or reserve is unsuitable.
pub fn build_continuation_spend(
    leaves: &ContinuationLeaves,
    parent: &TargetTransaction,
    coin: &FundedOutput,
    id: ContinuationSubjectId,
) -> Result<TargetTransaction, BirthNativeRefusal> {
    let _span = tracing::debug_span!("build_continuation_spend", ?id).entered();
    let fact = coin_output(coin)?;
    let primary = output(&fact, BIRTH_VALUE, leaves.primary.program.clone());
    let alternative = output(&fact, BIRTH_VALUE, leaves.alternative.program.clone());
    if parent.outputs().get(..2) != Some([primary.clone(), alternative].as_slice()) {
        return Err(BirthNativeRefusal::Funding);
    }
    let change = fact
        .value
        .checked_sub(FIXTURE_FEE)
        .filter(|n| *n > 0)
        .ok_or(BirthNativeRefusal::Funding)?;
    let mut outputs = vec![
        primary,
        output(&fact, change, leaves.alternative.program.clone()),
    ];
    match id {
        ContinuationSubjectId::Control => {}
        ContinuationSubjectId::WrongProgram => {
            outputs[0] = output(&fact, BIRTH_VALUE, leaves.alternative.program.clone());
        }
        ContinuationSubjectId::WrongVersion => {
            let mut program = leaves.primary.program.clone();
            program[0] = 0;
            outputs[0] = output(&fact, BIRTH_VALUE, program);
        }
        ContinuationSubjectId::DisplacedSuccessor => outputs.swap(0, 1),
    }
    outputs.push(output(&fact, FIXTURE_FEE, vec![]));
    transaction(
        vec![
            TargetInput::new(
                Outpoint::new(identity(parent), 0).map_err(|_| BirthNativeRefusal::Transaction)?,
                u32::MAX,
            ),
            TargetInput::new(coin_outpoint(coin)?, u32::MAX),
        ],
        outputs,
        vec![
            InputWitness::new(vec![
                leaves.primary.script.clone(),
                leaves.primary.control_block.clone(),
            ]),
            InputWitness::new(vec![]),
        ],
    )
}

/// Capture-stage standing, not admission or universal clause attribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinuationObservationClass {
    /// The control has an exact mined readback.
    MinedControl,
    /// Exact equality refusal, independently isolated to the version coordinate.
    VersionEqualityRefusal,
    /// Exact equality refusal, independently isolated to the payload coordinate.
    ProgramEqualityRefusal,
    /// Missing, ambiguous or earlier-layer evidence; obligation remains open.
    Open,
}

/// Exact signed and submitted subject with its fixed pair projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationSubjectRecord {
    /// Canonical subject coordinates and declared attribution limits.
    pub subject: ContinuationSubject,
    /// Complete signing request and response.
    pub signing: (OperationStep, NativeOperationResponse),
    /// Complete submission request and response, including any mined readback.
    pub submission: (OperationStep, NativeOperationResponse),
}

impl ContinuationSubjectRecord {
    /// Human-readable capture projection; replay uses the complete wire journal.
    #[must_use]
    pub fn capture_json(&self) -> Value {
        json!({"subject": format!("{:?}", self.subject.id),
            "changed_coordinate": format!("{:?}", self.subject.changed_coordinate),
            "collateral": format!("{:?}", self.subject.collateral),
            "attribution_limit": "shared equality sites; coordinate isolation beside the exact mined control, not error-text-only attribution",
            "equality_offsets": CONTINUATION_EQUALVERIFY_OFFSETS,
            "parent_bytes": self.subject.context.parent_bytes,
            "birth_position": self.subject.context.spent_outpoint.output_index,
            "signing_request": self.signing.0.subject(), "signing_response": self.signing.1,
            "submission_request": self.submission.0.subject(), "submission_response": self.submission.1})
    }
}

/// Eleven-operation sponsor-funded campaign with negatives preceding control.
#[derive(Clone, Debug)]
pub struct ContinuationCampaignPlanner {
    leaves: ContinuationLeaves,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    coins: Vec<FundedOutput>,
    parent: Option<TargetTransaction>,
    signed: Option<TargetTransaction>,
    records: Vec<ContinuationSubjectRecord>,
    refusal: Option<BirthNativeRefusal>,
}

impl ContinuationCampaignPlanner {
    /// Start an unobserved campaign from reconstructed leaf recipes.
    /// # Errors
    /// Returns `Leaf` if reconstruction fails.
    pub fn new(target: &ReviewedElementsTapscriptDefinition) -> Result<Self, BirthNativeRefusal> {
        Ok(Self {
            leaves: ContinuationLeaves::new(target)?,
            pending: None,
            exchanges: vec![],
            coins: vec![],
            parent: None,
            signed: None,
            records: vec![],
            refusal: None,
        })
    }
    /// Complete linked exchanges, including parent creation and readback.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }
    /// Records in negative-first submission order.
    #[must_use]
    pub fn records(&self) -> &[ContinuationSubjectRecord] {
        &self.records
    }
    /// Retained construction/replay stop reason.
    #[must_use]
    pub const fn refusal(&self) -> Option<BirthNativeRefusal> {
        self.refusal
    }
    /// Require all eleven exchanges; this does not admit observed outcomes.
    /// # Errors
    /// Returns the retained refusal or `Replay` for an incomplete campaign.
    pub fn complete(&self) -> Result<(), BirthNativeRefusal> {
        if let Some(reason) = self.refusal {
            return Err(reason);
        }
        if self.exchanges.len() != 11 || self.records.len() != 4 || self.pending.is_some() {
            return Err(BirthNativeRefusal::Replay);
        }
        self.validate_pairs()
    }
    /// Bind the replayed census to a separately reviewed target packet.
    /// # Errors
    /// Returns a replay or pair refusal for an incomplete/inconsistent campaign.
    pub fn manifest(
        &self,
        packet: Wave1TargetPacket,
    ) -> Result<ContinuationCampaignManifest, BirthNativeRefusal> {
        self.complete()?;
        ContinuationCampaignManifest::new(packet, self.canonical_subjects()?)
            .map_err(|_| BirthNativeRefusal::Pair)
    }
    fn canonical_subjects(&self) -> Result<Vec<ContinuationSubject>, BirthNativeRefusal> {
        ContinuationSubjectId::ALL
            .into_iter()
            .map(|id| {
                self.records
                    .iter()
                    .find(|r| r.subject.id == id)
                    .map(|r| r.subject.clone())
                    .ok_or(BirthNativeRefusal::Pair)
            })
            .collect()
    }
    fn validate_pairs(&self) -> Result<(), BirthNativeRefusal> {
        // The same manifest validator is used before eligibility without inventing
        // a packet from unverified live policy or deployment facts.
        crate::keyed_continuation::validate_subjects(&self.canonical_subjects()?)
            .map_err(|_| BirthNativeRefusal::Pair)
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
        if n == 11 {
            return Ok(None);
        }
        let subject = match n {
            0 => OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 2,
                amount_per_sponsor_output: SPONSOR_VALUE,
            })),
            1 => Self::signing(
                &build_continuation_parent(&self.leaves, &self.coins[0])?,
                0,
                &self.coins[0],
            ),
            2 | 4 | 6 | 8 | 10 => OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self
                    .signed
                    .as_ref()
                    .ok_or(BirthNativeRefusal::Replay)?
                    .encode(),
            })),
            3 | 5 | 7 | 9 => Self::signing(
                &build_continuation_spend(
                    &self.leaves,
                    self.parent.as_ref().ok_or(BirthNativeRefusal::Replay)?,
                    &self.coins[1],
                    ContinuationSubjectId::SUBMISSION_ORDER[(n - 3) / 2],
                )?,
                1,
                &self.coins[1],
            ),
            _ => return Err(BirthNativeRefusal::Replay),
        };
        Ok(Some(OperationStep::new(
            &format!("continuation-{n}"),
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
                self.coins.clone_from(coins);
            }
            OperationSubject::SponsorSigning(request) => {
                self.signed = Some(apply_birth_signature(
                    request,
                    response,
                    &self.coins[usize::from(n != 1)],
                )?);
            }
            OperationSubject::Submission(request) if n == 2 => {
                if response.observed_layer != ObservedOutcomeLayer::Accepted {
                    return Err(BirthNativeRefusal::Readback);
                }
                check_readback(&request.transaction_bytes, response)
                    .map_err(|_| BirthNativeRefusal::Readback)?;
                self.parent = Some(
                    TargetTransaction::decode(
                        &response
                            .mined_readback
                            .as_ref()
                            .ok_or(BirthNativeRefusal::Readback)?
                            .raw_transaction,
                    )
                    .map_err(|_| BirthNativeRefusal::Readback)?,
                );
            }
            OperationSubject::Submission(_) => self.retain_subject(step, response)?,
            _ => return Err(BirthNativeRefusal::Response),
        }
        Ok(())
    }
    fn retain_subject(
        &mut self,
        step: &OperationStep,
        response: &NativeOperationResponse,
    ) -> Result<(), BirthNativeRefusal> {
        let id = ContinuationSubjectId::SUBMISSION_ORDER[self.records.len()];
        let parent = self.parent.as_ref().ok_or(BirthNativeRefusal::Replay)?;
        let funding = coin_output(&self.coins[1])?;
        let point = coin_outpoint(&self.coins[1])?;
        let predecessor = BirthOutput {
            program: self.leaves.primary.program.clone(),
            asset: funding.asset,
            value: BIRTH_VALUE,
        };
        let change = BirthOutput {
            program: self.leaves.alternative.program.clone(),
            asset: funding.asset,
            value: funding.value - FIXTURE_FEE,
        };
        let context = ContinuationPairContext {
            parent_bytes: parent.encode(),
            spent_outpoint: BirthOutpoint {
                transaction_id: *identity(parent).internal(),
                output_index: 0,
            },
            current_input_index: 0,
            predecessor: predecessor.clone(),
            // No semantic key literal exists in this key-free experiment.
            public_key: [0; 32],
            leaf: self.leaves.primary.script.clone(),
            alternative_leaf: self.leaves.alternative.script.clone(),
            alternative_program: self.leaves.alternative.program.clone(),
            funding_outpoint: BirthOutpoint {
                transaction_id: *point.txid().internal(),
                output_index: point.index(),
            },
            funding_output: funding,
            control_outputs: [predecessor, change],
            fee: FIXTURE_FEE,
        };
        let mut outputs = context.control_outputs.clone();
        let boundary = match id {
            ContinuationSubjectId::Control => ContinuationBoundary::ScriptAcceptance,
            ContinuationSubjectId::WrongProgram => {
                outputs[0]
                    .program
                    .clone_from(&self.leaves.alternative.program);
                ContinuationBoundary::ProgramEqualVerify
            }
            ContinuationSubjectId::WrongVersion => {
                outputs[0].program[0] = 0;
                ContinuationBoundary::VersionEqualVerify
            }
            ContinuationSubjectId::DisplacedSuccessor => {
                outputs.swap(0, 1);
                ContinuationBoundary::ProgramEqualVerify
            }
        };
        let negative = id != ContinuationSubjectId::Control;
        self.records.push(ContinuationSubjectRecord {
            subject: ContinuationSubject {
                id,
                control: negative.then_some(ContinuationSubjectId::Control),
                context,
                outputs,
                changed_coordinate: id.coordinate(),
                collateral: if negative {
                    ContinuationCollateral::ALL.into_iter().collect()
                } else {
                    BTreeSet::default()
                },
                boundary,
                attribution: ContinuationAttribution::SharedEqualityRequiresIndependentEvidence,
            },
            signing: self
                .exchanges
                .last()
                .ok_or(BirthNativeRefusal::Replay)?
                .clone(),
            submission: (step.clone(), response.clone()),
        });
        Ok(())
    }
}

impl TargetOperationPlanner for ContinuationCampaignPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let _span =
            tracing::debug_span!("keyed_continuation_step", completed = self.exchanges.len())
                .entered();
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
            tracing::debug!(?refusal, "continuation campaign stopped");
            self.refusal = Some(refusal);
            PlanRefused
        })
    }
}

/// Replay exact requests with the birth campaign's shared deterministic machinery.
/// # Errors
/// Returns the construction refusal or `Replay` for missing/changed/surplus steps.
pub fn replay_continuation_campaign(
    target: &ReviewedElementsTapscriptDefinition,
    exchanges: &[(OperationStep, NativeOperationResponse)],
) -> Result<ContinuationCampaignPlanner, BirthNativeRefusal> {
    let _span =
        tracing::debug_span!("replay_continuation_campaign", exchanges = exchanges.len()).entered();
    let planner = replay_operations(
        ContinuationCampaignPlanner::new(target)?,
        exchanges,
        ContinuationCampaignPlanner::refusal,
    )?;
    planner.complete()?;
    Ok(planner)
}

/// Retain every complete response in protocol wire JSON, including exchanges of
/// an incomplete run. The enhanced outer carrier retains target and deployment.
#[must_use]
pub fn continuation_capture_json(capture: &NativeOperationCapture) -> Value {
    let _span = tracing::debug_span!(
        "continuation_capture_json",
        exchanges = capture.operations().len()
    )
    .entered();
    let operations: Vec<_> = capture.operations().iter().map(|op| json!({
        "operation_id": op.operation_id(), "request_id": op.request_id(), "request": op.request(),
        "response_id": op.response_id(), "response_request_id": op.response_request_id(),
        "response_operation_id": op.response_operation_id(), "response": op.response()
    })).collect();
    json!({"continuation_wire_schema": 1, "terminal": capture.terminal_state().map(target_elements_conformance::executor::CaptureTerminalState::as_str),
        "handshake": capture.handshake(), "environment": capture.environment(), "operations": operations})
}

/// Decode a self-contained wire journal without reading any debug transcript.
/// # Errors
/// Returns `Replay` for malformed/missing responses or broken links, otherwise
/// the same construction refusal as deterministic replay.
pub fn replay_continuation_capture(
    target: &ReviewedElementsTapscriptDefinition,
    journal: &Value,
) -> Result<ContinuationCampaignPlanner, BirthNativeRefusal> {
    let _span = tracing::debug_span!("replay_continuation_capture").entered();
    if journal["continuation_wire_schema"] != 1 || journal["terminal"] != "complete" {
        return Err(BirthNativeRefusal::Replay);
    }
    let operations = journal["operations"]
        .as_array()
        .ok_or(BirthNativeRefusal::Replay)?;
    let exchanges = operations
        .iter()
        .enumerate()
        .map(|(n, op)| {
            if op["operation_id"] != format!("operation-{n}")
                || op["request_id"] != format!("request-{n}")
                || op["response_id"] != format!("response-{n}")
                || op["response_request_id"] != op["request_id"]
                || op["response_operation_id"] != op["operation_id"]
            {
                return Err(BirthNativeRefusal::Replay);
            }
            let request: NativeOperationRequest = serde_json::from_value(op["request"].clone())
                .map_err(|_| BirthNativeRefusal::Replay)?;
            let response: NativeOperationResponse = serde_json::from_value(op["response"].clone())
                .map_err(|_| BirthNativeRefusal::Replay)?;
            if serde_json::to_value(&response).map_err(|_| BirthNativeRefusal::Replay)?
                != op["response"]
            {
                return Err(BirthNativeRefusal::Replay);
            }
            let step = OperationStep::new(&request.case.step, request.subject);
            if request.schema != NATIVE_PROTOCOL_SCHEMA || &request.case != step.case() {
                return Err(BirthNativeRefusal::Replay);
            }
            Ok((step, response))
        })
        .collect::<Result<Vec<_>, _>>()?;
    replay_continuation_campaign(target, &exchanges)
}

/// Classify only a replayed, exact one-coordinate pair beside its mined control.
///
/// The error does not name a unique opcode site. Birth and sponsor equality are
/// excluded independently; version/payload attribution uses the fixed program
/// and version coordinates, never the error text alone. No ECDSA check is claimed.
#[must_use]
pub fn classify_continuation_subject(
    planner: &ContinuationCampaignPlanner,
    id: ContinuationSubjectId,
) -> ContinuationObservationClass {
    let classify = || -> Option<ContinuationObservationClass> {
        planner.complete().ok()?;
        let row = planner.records.iter().find(|r| r.subject.id == id)?;
        let OperationSubject::Submission(request) = row.submission.0.subject() else {
            return None;
        };
        let response = &row.submission.1;
        let observation: Value = serde_json::from_str(response.observed_detail.as_deref()?).ok()?;
        detail(response).ok()?;
        if id == ContinuationSubjectId::Control {
            if response.observed_layer != ObservedOutcomeLayer::Accepted
                || !matches!(
                    observation["route"].as_str(),
                    Some("relay-and-template" | "direct-block")
                )
            {
                return None;
            }
            check_readback(&request.transaction_bytes, response).ok()?;
            return Some(ContinuationObservationClass::MinedControl);
        }
        if classify_continuation_subject(planner, ContinuationSubjectId::Control)
            != ContinuationObservationClass::MinedControl
            || response.observed_layer != ObservedOutcomeLayer::ScriptPathRejection
            || observation["route"] != "refused"
            || observation["relay_allowed"] != false
        {
            return None;
        }
        let reason = observation["relay_detail"].as_str()?;
        let transaction = TargetTransaction::decode(&request.transaction_bytes).ok()?;
        if response.accepted_txid.is_some()
            || response.mined_readback.is_some()
            || observation["detail"]["txid"] != identity(&transaction).to_target_display()
        {
            return None;
        }
        if observation["detail"]["verdict_reason"] != reason
            || !matches!(
                reason,
                "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)"
                    | "non-mandatory-script-verify-flag (Script failed an OP_EQUALVERIFY operation)"
            )
        {
            return None;
        }
        // Requests are rebuilt from the recipes. The manifest holds birth zero,
        // predecessor, funding, version/payload and output permutation fixed as
        // declared. Signing replay excludes the sponsor hash-equality site.
        Some(if id == ContinuationSubjectId::WrongVersion {
            ContinuationObservationClass::VersionEqualityRefusal
        } else {
            ContinuationObservationClass::ProgramEqualityRefusal
        })
    };
    classify().unwrap_or(ContinuationObservationClass::Open)
}
