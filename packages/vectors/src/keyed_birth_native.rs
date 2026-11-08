//! Construction and capture for the A0–A3 birth experiment, not admission.
//!
//! Each placement gets fresh two-output sponsor funding. Output zero funds a
//! mined parent; output one is reserved for both children. The negative is
//! signed and submitted before the control. Amounts are fixture choices, not
//! funding floors. No issuance, peg-in or flag coverage is claimed.
//!
//! An equality error is attributed only after exact transcript replay and pair
//! comparison. The sponsor's P2WPKH equality is checked by hashing its retained
//! public key. Its signature opcode is CHECKSIG, which cannot report an
//! EQUALVERIFY failure. This excludes a signature *site*; it does not independently
//! verify ECDSA or claim the negative's signature was executed.

use std::collections::BTreeSet;

use serde_json::{Value, json};
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::constructor::{
    UNSPENDABLE_INTERNAL_KEY,
    tagged::sha256,
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
use transaction::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};

use crate::keyed_birth::{
    BirthBoundary, BirthCampaignManifest, BirthCollateral, BirthCoordinate, BirthOutpoint,
    BirthOutput, BirthPairContext, BirthSubject, BirthSubjectId, Wave1TargetPacket,
};
use crate::maturity_native::sponsored::{check_readback, detail, hash160};

/// Disposable fixture amount per sponsor output, in satoshis.
pub const SPONSOR_VALUE: u64 = 100_000;
/// Equal covenant output amount, in satoshis.
pub const BIRTH_VALUE: u64 = 10_000;
/// Explicit parent and child fee, in satoshis; policy is still observed.
pub const FIXTURE_FEE: u64 = 1_000;
/// Byte offset of the sole equality opcode in the birth leaf.
pub const BIRTH_EQUALVERIFY_OFFSET: usize = 8;

/// Construction or replay refused without promoting an observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BirthNativeRefusal {
    /// The reviewed leaf could not be constructed.
    Leaf,
    /// Funding has the wrong census, amount, asset, program or position.
    Funding,
    /// Transaction bytes or balanced construction could not be established.
    Transaction,
    /// A signing echo, profile, key hash or witness shape disagrees.
    Signing,
    /// The response does not answer the frozen request.
    Response,
    /// A mined acceptance does not bind the exact submitted bytes.
    Readback,
    /// Observed policy is absent or malformed.
    Policy,
    /// A transcript is incomplete, reordered or differs from construction.
    Replay,
    /// The four-subject manifest does not preserve the declared pairs.
    Pair,
}

/// Reviewed leaf and its fixed unspendable-key commitment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BirthLeaf {
    /// Exact script, including final truth.
    pub script: Vec<u8>,
    /// Exact output scriptPubKey.
    pub program: Vec<u8>,
    /// Exact single-leaf control block.
    pub control_block: Vec<u8>,
}

impl BirthLeaf {
    /// Construct the typed fragment plus final truth under the reviewed version.
    /// # Errors
    /// Returns `Leaf` if fragment or commitment construction refuses.
    pub fn new(target: &ReviewedElementsTapscriptDefinition) -> Result<Self, BirthNativeRefusal> {
        let _span = tracing::debug_span!("keyed_birth_leaf").entered();
        let mut script = tapscript::born_first_fragment(target)
            .map_err(|_| BirthNativeRefusal::Leaf)?
            .encode(target);
        script.push(0x51);
        let tree = FixtureTapTree::Leaf {
            version: target.definition().leaf_version().get(),
            script: script.clone(),
        };
        let output = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &tree)
            .map_err(|_| BirthNativeRefusal::Leaf)?;
        Ok(Self {
            script,
            program: output.output_program().to_vec(),
            control_block: output.control_block().to_vec(),
        })
    }
}

fn decode_hex(text: &str) -> Result<Vec<u8>, BirthNativeRefusal> {
    if !text.len().is_multiple_of(2) || !text.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(BirthNativeRefusal::Funding);
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|s| u8::from_str_radix(s, 16).ok())
                .ok_or(BirthNativeRefusal::Funding)
        })
        .collect()
}

pub(crate) fn coin_output(coin: &FundedOutput) -> Result<BirthOutput, BirthNativeRefusal> {
    let asset = Txid::from_target_display(&coin.asset).map_err(|_| BirthNativeRefusal::Funding)?;
    Ok(BirthOutput {
        program: decode_hex(&coin.script)?,
        asset: *asset.internal(),
        value: coin.amount_satoshis,
    })
}

pub(crate) fn coin_outpoint(coin: &FundedOutput) -> Result<Outpoint, BirthNativeRefusal> {
    Outpoint::new(
        Txid::from_target_display(&coin.outpoint.txid).map_err(|_| BirthNativeRefusal::Funding)?,
        coin.outpoint.vout,
    )
    .map_err(|_| BirthNativeRefusal::Funding)
}

pub(crate) const fn output(fact: &BirthOutput, value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(fact.asset)),
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}

pub(crate) fn transaction(
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    witnesses: Vec<InputWitness>,
) -> Result<TargetTransaction, BirthNativeRefusal> {
    TargetTransaction::new(2, inputs, outputs, 0, witnesses)
        .map_err(|_| BirthNativeRefusal::Transaction)
}

/// Build a balanced parent with equal birth outputs zero and one, change and fee.
/// # Errors
/// Returns `Funding` for an unsuitable sponsor coin or insufficient fixture value.
pub fn build_birth_parent(
    leaf: &BirthLeaf,
    coin: &FundedOutput,
) -> Result<TargetTransaction, BirthNativeRefusal> {
    let _span = tracing::debug_span!("build_birth_parent").entered();
    let fact = coin_output(coin)?;
    let change = fact
        .value
        .checked_sub(2 * BIRTH_VALUE + FIXTURE_FEE)
        .filter(|value| *value > 0)
        .ok_or(BirthNativeRefusal::Funding)?;
    transaction(
        vec![TargetInput::new(coin_outpoint(coin)?, u32::MAX)],
        vec![
            output(&fact, BIRTH_VALUE, leaf.program.clone()),
            output(&fact, BIRTH_VALUE, leaf.program.clone()),
            output(&fact, change, fact.program.clone()),
            output(&fact, FIXTURE_FEE, vec![]),
        ],
        vec![InputWitness::new(vec![])],
    )
}

pub(crate) fn identity(tx: &TargetTransaction) -> Txid {
    Txid::from_internal(sha256(&sha256(&tx.encode_without_witness())))
}

/// Build a two-input child with the covenant in the subject's declared position.
/// # Errors
/// Returns a construction refusal for an unsuitable parent or sponsor coin.
pub fn build_birth_spend(
    leaf: &BirthLeaf,
    parent: &TargetTransaction,
    coin: &FundedOutput,
    id: BirthSubjectId,
) -> Result<TargetTransaction, BirthNativeRefusal> {
    let _span = tracing::debug_span!("build_birth_spend", ?id).entered();
    let fact = coin_output(coin)?;
    let expected = output(&fact, BIRTH_VALUE, leaf.program.clone());
    if parent.outputs().get(..2) != Some([expected.clone(), expected.clone()].as_slice()) {
        return Err(BirthNativeRefusal::Funding);
    }
    let change = fact
        .value
        .checked_sub(FIXTURE_FEE)
        .filter(|value| *value > 0)
        .ok_or(BirthNativeRefusal::Funding)?;
    let (birth, current) = id.positions();
    let birth_point =
        Outpoint::new(identity(parent), birth).map_err(|_| BirthNativeRefusal::Transaction)?;
    let mut inputs = vec![
        TargetInput::new(birth_point, u32::MAX),
        TargetInput::new(coin_outpoint(coin)?, u32::MAX),
    ];
    let mut witnesses = vec![
        InputWitness::new(vec![leaf.script.clone(), leaf.control_block.clone()]),
        InputWitness::new(vec![]),
    ];
    if current == 1 {
        inputs.swap(0, 1);
        witnesses.swap(0, 1);
    }
    transaction(
        inputs,
        vec![
            expected,
            output(&fact, change, fact.program.clone()),
            output(&fact, FIXTURE_FEE, vec![]),
        ],
        witnesses,
    )
}

/// Bind a sponsor response to its exact unsigned request and P2WPKH coin.
/// This checks the echo and equality preimage, not ECDSA validity.
/// # Errors
/// Returns `Signing` for any mismatch; returns `Transaction` for undecodable bytes.
pub fn apply_birth_signature(
    request: &TargetSponsorSigningSubject,
    response: &NativeOperationResponse,
    coin: &FundedOutput,
) -> Result<TargetTransaction, BirthNativeRefusal> {
    let _span = tracing::debug_span!("apply_birth_signature").entered();
    let mut program = vec![0, 20];
    let witness = &response.sponsor_witness;
    if response.observed_layer != ObservedOutcomeLayer::Accepted
        || response.signature_bound_to.as_deref() != Some(&request.finalized_transaction)
        || request.sighash_profile != WireSighashProfile::AllInputsAllOutputs
        || request.sponsor_outpoint != coin.outpoint
        || witness.len() != 2
        || witness[0].last() != Some(&1)
        || witness[0].len() < 9
        || witness[1].len() != 33
        || !matches!(witness[1][0], 2 | 3)
    {
        return Err(BirthNativeRefusal::Signing);
    }
    program.extend(hash160(&witness[1]));
    if program != decode_hex(&coin.script)? {
        return Err(BirthNativeRefusal::Signing);
    }
    let tx = TargetTransaction::decode(&request.finalized_transaction)
        .map_err(|_| BirthNativeRefusal::Transaction)?;
    let index = usize::from(request.sponsor_input_index);
    if tx.inputs().get(index).map(TargetInput::outpoint) != Some(coin_outpoint(coin)?)
        || tx
            .witnesses()
            .get(index)
            .is_none_or(|w| !w.stack().is_empty())
    {
        return Err(BirthNativeRefusal::Signing);
    }
    let mut witnesses = tx.witnesses().to_vec();
    witnesses[index] = InputWitness::new(witness.clone());
    transaction(tx.inputs().to_vec(), tx.outputs().to_vec(), witnesses)
}

/// Capture-stage standing. No variant constitutes archive admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BirthObservationClass {
    /// Exact accepted bytes were read back from a block.
    MinedControl,
    /// The paired subject refused at the uniquely identified equality site.
    BornFirstEqualVerify,
    /// Evidence does not uniquely establish the declared boundary.
    Open,
}

/// A frozen subject and exact linked signing/submission exchanges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BirthSubjectRecord {
    /// Declared coordinate, collateral and funding context.
    pub subject: BirthSubject,
    /// Signing request and response, including the exact byte echo.
    pub signing: (OperationStep, NativeOperationResponse),
    /// Submission request and response, including raw detail and mined readback.
    pub submission: (OperationStep, NativeOperationResponse),
}

impl BirthSubjectRecord {
    /// Retain relay, consensus route, policy, refusal text and complete raw responses.
    /// Unknown or absent observations remain null, not inferred consensus failures.
    #[must_use]
    pub fn capture_json(&self) -> Value {
        let response = &self.submission.1;
        let context = &self.subject.context;
        let output = |o: &BirthOutput| json!({"program": o.program, "asset_internal": o.asset, "value": o.value});
        let detail = response
            .observed_detail
            .as_deref()
            .and_then(|text| serde_json::from_str::<Value>(text).ok())
            .unwrap_or(Value::Null);
        json!({
            "subject": format!("{:?}", self.subject.id),
            "declaration": format!("{:?}", self.subject),
            "current_input_index": self.subject.current_input_index,
            "spent_output_index": self.subject.spent_outpoint.output_index,
            "parent": {"txid_internal": context.parent_id, "mined_bytes": context.parent_bytes,
                "outputs": context.parent_outputs.iter().map(output).collect::<Vec<_>>()},
            "funding": {"txid_internal": context.funding_outpoint.transaction_id,
                "output_index": context.funding_outpoint.output_index,
                "output": output(&context.funding_output)},
            "spending_outputs": context.spending_outputs.iter().map(output).collect::<Vec<_>>(),
            "explicit_fee": context.fee,
            "signing_case": self.signing.0.case(),
            "signing_request": self.signing.0.subject(),
            "signing_response": self.signing.1,
            "submission_case": self.submission.0.case(),
            "submission_request": self.submission.0.subject(),
            "submission_response": response,
            "relay_allowed": detail["relay_allowed"],
            "relay_reason": detail["relay_detail"],
            "policy_and_startup": detail["policy"],
            "consensus_route": detail["route"],
            "consensus_evidence": detail["detail"],
            "refusal_text": response.observed_detail,
            "birth_equality_byte_offset": BIRTH_EQUALVERIFY_OFFSET,
        })
    }
}

/// Deterministic fourteen-operation construction, with retained responses for replay.
#[derive(Clone, Debug)]
pub struct BirthCampaignPlanner {
    leaf: BirthLeaf,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    coins: Vec<FundedOutput>,
    parent: Option<TargetTransaction>,
    signed: Option<TargetTransaction>,
    records: Vec<BirthSubjectRecord>,
    refusal: Option<BirthNativeRefusal>,
}

impl BirthCampaignPlanner {
    /// Start an unobserved campaign with the reviewed birth commitment.
    /// # Errors
    /// Returns `Leaf` if construction fails.
    pub fn new(target: &ReviewedElementsTapscriptDefinition) -> Result<Self, BirthNativeRefusal> {
        Ok(Self {
            leaf: BirthLeaf::new(target)?,
            pending: None,
            exchanges: vec![],
            coins: vec![],
            parent: None,
            signed: None,
            records: vec![],
            refusal: None,
        })
    }

    /// Exact completed exchanges, including parent funding and readback.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }

    /// Frozen subject records in submission order A1, A0, A3, A2.
    #[must_use]
    pub fn records(&self) -> &[BirthSubjectRecord] {
        &self.records
    }

    /// Retained reason when the planner stops early.
    #[must_use]
    pub const fn refusal(&self) -> Option<BirthNativeRefusal> {
        self.refusal
    }

    /// Bind the canonical census to a separately reviewed target packet.
    /// # Errors
    /// Returns `Replay` for an incomplete campaign or `Pair` for an invalid census.
    pub fn manifest(
        &self,
        packet: Wave1TargetPacket,
    ) -> Result<BirthCampaignManifest, BirthNativeRefusal> {
        self.complete()?;
        let subjects = BirthSubjectId::ALL
            .iter()
            .map(|id| {
                self.records
                    .iter()
                    .find(|r| r.subject.id == *id)
                    .map(|r| r.subject.clone())
                    .ok_or(BirthNativeRefusal::Pair)
            })
            .collect::<Result<Vec<_>, _>>()?;
        BirthCampaignManifest::new(packet, subjects).map_err(|_| BirthNativeRefusal::Pair)
    }

    /// Require all fourteen exchanges; this does not admit their outcomes.
    /// # Errors
    /// Returns the retained refusal or `Replay` for an incomplete capture.
    pub const fn complete(&self) -> Result<(), BirthNativeRefusal> {
        if let Some(refusal) = self.refusal {
            return Err(refusal);
        }
        if self.exchanges.len() != 14 || self.records.len() != 4 || self.pending.is_some() {
            return Err(BirthNativeRefusal::Replay);
        }
        Ok(())
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
        if n == 14 {
            return Ok(None);
        }
        let pair = n / 7;
        let phase = n % 7;
        let subject = match phase {
            0 => OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 2,
                amount_per_sponsor_output: SPONSOR_VALUE,
            })),
            1 => Self::signing(
                &build_birth_parent(&self.leaf, &self.coins[0])?,
                0,
                &self.coins[0],
            ),
            2 | 4 | 6 => OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self
                    .signed
                    .as_ref()
                    .ok_or(BirthNativeRefusal::Replay)?
                    .encode(),
            })),
            3 | 5 => {
                let id = BirthSubjectId::SUBMISSION_ORDER[pair * 2 + usize::from(phase == 5)];
                let tx = build_birth_spend(
                    &self.leaf,
                    self.parent.as_ref().ok_or(BirthNativeRefusal::Replay)?,
                    &self.coins[1],
                    id,
                )?;
                Self::signing(&tx, 1 - id.positions().1, &self.coins[1])
            }
            _ => return Err(BirthNativeRefusal::Replay),
        };
        Ok(Some(OperationStep::new(
            &format!("birth-{pair}-{phase}"),
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
        let phase = self.exchanges.len() % 7;
        match step.subject() {
            OperationSubject::SponsorFunding(_) => {
                let coins = &response.funded_outputs;
                if response.observed_layer != ObservedOutcomeLayer::Accepted || coins.len() != 2 {
                    return Err(BirthNativeRefusal::Funding);
                }
                let funding_id = *coin_outpoint(&coins[1])?.txid().internal();
                let program = decode_hex(&coins[0].script)?;
                if coins[0].outpoint.vout != 0
                    || coins[1].outpoint.vout != 1
                    || coins[0].outpoint.txid != coins[1].outpoint.txid
                    || coin_output(&coins[0])? != coin_output(&coins[1])?
                    || coins[0].amount_satoshis != SPONSOR_VALUE
                    || program.get(..2) != Some(&[0, 20])
                    || program.len() != 22
                    || self
                        .records
                        .iter()
                        .any(|r| r.subject.context.funding_outpoint.transaction_id == funding_id)
                {
                    return Err(BirthNativeRefusal::Funding);
                }
                self.coins.clone_from(coins);
                self.parent = None;
            }
            OperationSubject::SponsorSigning(request) => {
                self.signed = Some(apply_birth_signature(
                    request,
                    response,
                    &self.coins[usize::from(phase != 1)],
                )?);
            }
            OperationSubject::Submission(request) => {
                if phase == 2 {
                    if response.observed_layer != ObservedOutcomeLayer::Accepted {
                        return Err(BirthNativeRefusal::Readback);
                    }
                    check_readback(&request.transaction_bytes, response)
                        .map_err(|_| BirthNativeRefusal::Readback)?;
                    self.parent.clone_from(&self.signed);
                } else {
                    self.retain_subject(step, response)?;
                }
            }
            _ => return Err(BirthNativeRefusal::Response),
        }
        Ok(())
    }

    fn retain_subject(
        &mut self,
        step: &OperationStep,
        response: &NativeOperationResponse,
    ) -> Result<(), BirthNativeRefusal> {
        let id = BirthSubjectId::SUBMISSION_ORDER[self.records.len()];
        let parent = self.parent.as_ref().ok_or(BirthNativeRefusal::Replay)?;
        let funding = coin_output(&self.coins[1])?;
        let point = coin_outpoint(&self.coins[1])?;
        let birth = BirthOutput {
            program: self.leaf.program.clone(),
            asset: funding.asset,
            value: BIRTH_VALUE,
        };
        let mut change = funding.clone();
        change.value -= FIXTURE_FEE;
        let context = BirthPairContext {
            parent_bytes: parent.encode(),
            parent_id: *identity(parent).internal(),
            parent_outputs: [birth.clone(), birth.clone()],
            funding_outpoint: BirthOutpoint {
                transaction_id: *point.txid().internal(),
                output_index: point.index(),
            },
            funding_output: funding,
            spending_outputs: vec![birth, change],
            fee: FIXTURE_FEE,
        };
        let negative = id.control().is_some();
        self.records.push(BirthSubjectRecord {
            subject: BirthSubject {
                id,
                control: id.control(),
                current_input_index: id.positions().1,
                spent_outpoint: BirthOutpoint {
                    transaction_id: context.parent_id,
                    output_index: id.positions().0,
                },
                context,
                changed_coordinate: negative.then_some(BirthCoordinate::SpentOutputIndex),
                collateral: if negative {
                    BirthCollateral::ALL.into_iter().collect()
                } else {
                    BTreeSet::default()
                },
                boundary: if negative {
                    BirthBoundary::BornFirstEqualVerify
                } else {
                    BirthBoundary::ScriptAcceptance
                },
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

impl TargetOperationPlanner for BirthCampaignPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let _span =
            tracing::debug_span!("keyed_birth_step", completed = self.exchanges.len()).entered();
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
            tracing::debug!(?refusal, "birth campaign stopped");
            self.refusal = Some(refusal);
            PlanRefused
        })
    }
}

/// Replay every exact request before exposing any observation classification.
/// # Errors
/// Returns the planner's refusal or `Replay` for missing, changed or surplus steps.
pub fn replay_birth_campaign(
    target: &ReviewedElementsTapscriptDefinition,
    exchanges: &[(OperationStep, NativeOperationResponse)],
) -> Result<BirthCampaignPlanner, BirthNativeRefusal> {
    let _span =
        tracing::debug_span!("replay_birth_campaign", exchanges = exchanges.len()).entered();
    let planner = replay_operations(
        BirthCampaignPlanner::new(target)?,
        exchanges,
        BirthCampaignPlanner::refusal,
    )?;
    planner.complete()?;
    Ok(planner)
}

/// Replay exact requests through the same planner used for live staging.
pub(crate) fn replay_operations<P: TargetOperationPlanner>(
    mut planner: P,
    exchanges: &[(OperationStep, NativeOperationResponse)],
    refusal: impl Fn(&P) -> Option<BirthNativeRefusal>,
) -> Result<P, BirthNativeRefusal> {
    let mut expected = planner
        .next_step(None)
        .map_err(|_| BirthNativeRefusal::Replay)?;
    for (step, response) in exchanges {
        if expected.as_ref() != Some(step) {
            return Err(BirthNativeRefusal::Replay);
        }
        expected = planner
            .next_step(Some((step.case(), response)))
            .map_err(|_| refusal(&planner).unwrap_or(BirthNativeRefusal::Replay))?;
    }
    if expected.is_some() {
        return Err(BirthNativeRefusal::Replay);
    }
    Ok(planner)
}

/// Replay the capture's request/response links and then the frozen construction.
/// # Errors
/// Returns `Replay` for broken journal links or the construction's exact refusal.
pub fn replay_birth_capture(
    target: &ReviewedElementsTapscriptDefinition,
    capture: &(impl BirthCaptureJournal + ?Sized),
) -> Result<BirthCampaignPlanner, BirthNativeRefusal> {
    replay_birth_campaign(target, &capture.birth_exchanges()?)
}

/// A linked capture journal, including an archive decoded without a live executor.
pub trait BirthCaptureJournal {
    /// Return the retained exchanges after checking request/response linkage.
    /// # Errors
    /// Returns `Replay` when linkage or a response is missing.
    fn birth_exchanges(
        &self,
    ) -> Result<Vec<(OperationStep, NativeOperationResponse)>, BirthNativeRefusal>;
}

impl BirthCaptureJournal for NativeOperationCapture {
    fn birth_exchanges(
        &self,
    ) -> Result<Vec<(OperationStep, NativeOperationResponse)>, BirthNativeRefusal> {
        self.operations()
            .iter()
            .enumerate()
            .map(|(ordinal, operation)| {
                if operation.operation_id() != format!("operation-{ordinal}")
                    || operation.request_id() != format!("request-{ordinal}")
                    || operation.response_id() != Some(format!("response-{ordinal}").as_str())
                    || operation.response_request_id() != Some(operation.request_id())
                    || operation.response_operation_id() != Some(operation.operation_id())
                    || operation.request().schema != NATIVE_PROTOCOL_SCHEMA
                {
                    return Err(BirthNativeRefusal::Replay);
                }
                Ok((
                    OperationStep::new(
                        &operation.request().case.step,
                        operation.request().subject.clone(),
                    ),
                    operation
                        .response()
                        .ok_or(BirthNativeRefusal::Replay)?
                        .clone(),
                ))
            })
            .collect()
    }
}

/// Classify a replayed pair conservatively, leaving ambiguous sites open.
/// This is capture analysis, never admission of a run of record.
#[must_use]
pub fn classify_birth_subject(
    planner: &BirthCampaignPlanner,
    id: BirthSubjectId,
) -> BirthObservationClass {
    let classify = || -> Option<BirthObservationClass> {
        planner.complete().ok()?;
        let row = planner.records.iter().find(|r| r.subject.id == id)?;
        let OperationSubject::Submission(request) = row.submission.0.subject() else {
            return None;
        };
        let response = &row.submission.1;
        let observation: Value = serde_json::from_str(response.observed_detail.as_deref()?).ok()?;
        detail(response).ok()?;
        if id.control().is_none() {
            if response.observed_layer != ObservedOutcomeLayer::Accepted
                || !matches!(
                    observation["route"].as_str(),
                    Some("relay-and-template" | "direct-block")
                )
            {
                return None;
            }
            check_readback(&request.transaction_bytes, response).ok()?;
            return Some(BirthObservationClass::MinedControl);
        }
        let control = planner
            .records
            .iter()
            .find(|r| Some(r.subject.id) == id.control())?;
        if row.subject.context != control.subject.context
            || classify_birth_subject(planner, control.subject.id)
                != BirthObservationClass::MinedControl
            || response.observed_layer != ObservedOutcomeLayer::ScriptPathRejection
            || observation["route"] != "refused"
            || observation["relay_allowed"] != false
            || planner.leaf.script != [0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0x51]
        {
            return None;
        }
        let reason = observation["relay_detail"].as_str()?;
        if observation["detail"]["verdict_reason"] != reason
            || !matches!(
                reason,
                "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)"
                    | "non-mandatory-script-verify-flag (Script failed an OP_EQUALVERIFY operation)"
            )
        {
            return None;
        }
        // Replay checked the sponsor key's P2WPKH hash equality and byte echo.
        // CHECKSIG is the only signature site and cannot emit this exact error.
        Some(BirthObservationClass::BornFirstEqualVerify)
    };
    classify().unwrap_or(BirthObservationClass::Open)
}
