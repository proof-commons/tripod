//! Staging of constructor paths and keyed continuation, not admission.
//!
//! Four reconstructed parents put every tested handle at birth position zero.
//! The mixed pair shares its first k coin and sponsor; changing the second
//! handle's key necessarily changes its outpoint and authenticated witness.
//! Separate k and l controls establish coexistence only on the captured history.
//! All complete responses use the existing wire journal; no debug text is read.

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

use crate::keyed_birth::BirthSubjectId;
use crate::keyed_birth_native::{
    BIRTH_VALUE, BirthLeaf, BirthNativeRefusal, SPONSOR_VALUE, apply_birth_signature,
    build_birth_parent, build_birth_spend, coin_outpoint, coin_output, identity, output,
    replay_operations, transaction,
};
use crate::keyed_constructor::{
    KeyedConstructorDescriptor, PublicKeyedCurve, authenticate_keyed_predecessor, construct_keyed,
    reconstruct_keyed,
};
use crate::keyed_continuation_native::{continuation_capture_json, decode_wire_operations};
use crate::maturity_native::sponsored::{check_readback, detail};

/// Birth, version and payload equalities after the key's canonical push/drop.
pub const KEYED_EQUALVERIFY_OFFSETS: [usize; 3] = [42, 49, 50];

/// Eight rows spanning the two measurements; acceptance is always capture-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedSubjectId {
    /// Canonical k script path; also the k half of independent coexistence.
    ControlK,
    /// Only bit zero of the control byte changes.
    WrongControlParity,
    /// Only offered leaf bytes change; the original control remains.
    UncommittedLeaf,
    /// Only the keyed input's witness path selection changes.
    KeyPathAttempt,
    /// Output zero uses the reconstructed l recipe instead of k.
    WrongKey,
    /// Second birth-zero handle is l, with output zero still k.
    MixedKeys,
    /// Same-key two-handle control for the mixed row.
    SameKeyControl,
    /// Separate accepted l transaction on the same history as `ControlK`.
    CoexistenceL,
}

impl KeyedSubjectId {
    /// Stable report order.
    pub const ALL: [Self; 8] = [
        Self::ControlK,
        Self::WrongControlParity,
        Self::UncommittedLeaf,
        Self::KeyPathAttempt,
        Self::WrongKey,
        Self::MixedKeys,
        Self::SameKeyControl,
        Self::CoexistenceL,
    ];
    /// Negatives precede their consuming controls.
    pub const SUBMISSION_ORDER: [Self; 8] = [
        Self::WrongControlParity,
        Self::UncommittedLeaf,
        Self::KeyPathAttempt,
        Self::WrongKey,
        Self::ControlK,
        Self::MixedKeys,
        Self::SameKeyControl,
        Self::CoexistenceL,
    ];
    /// The independently mined control needed before classifying a refusal.
    #[must_use]
    pub const fn control(self) -> Option<Self> {
        match self {
            Self::MixedKeys => Some(Self::SameKeyControl),
            Self::WrongControlParity
            | Self::UncommittedLeaf
            | Self::KeyPathAttempt
            | Self::WrongKey => Some(Self::ControlK),
            _ => None,
        }
    }
    /// One semantic mutation, or the accepted row's purpose.
    #[must_use]
    pub const fn coordinate(self) -> &'static str {
        match self {
            Self::ControlK => "canonical keyed script path under k",
            Self::WrongControlParity => "control parity bit only",
            Self::UncommittedLeaf => "offered leaf key literal byte only; original control",
            Self::KeyPathAttempt => "keyed input witness path selection only",
            Self::WrongKey => "output-zero semantic key k to l; same recipe",
            Self::MixedKeys => "second handle semantic key k to l; first k handle fixed",
            Self::SameKeyControl => "two birth-zero handles under k; output zero under k",
            Self::CoexistenceL => {
                "separate l spend on ControlK's history, no mixed acceptance claim"
            }
        }
    }
    /// Declared dependent changes; these are not additional semantic mutations.
    #[must_use]
    pub const fn collateral(self) -> &'static str {
        match self {
            Self::MixedKeys => {
                "second predecessor identity/program/control/leaf, sponsor signature, transaction identities, weight and observation; values/assets/fee/first coin/reserve/output program fixed"
            }
            Self::WrongKey => {
                "successor commitment, sponsor signature, transaction identities, weight and observation; values/assets/fee/inputs/leaf/control fixed"
            }
            Self::WrongControlParity | Self::UncommittedLeaf | Self::KeyPathAttempt => {
                "witness bytes, wtxid, weight and observation; non-witness transaction and sponsor signing digest fixed"
            }
            _ => "accepted control on its declared coins; no mutation",
        }
    }
    /// Expected boundary, never an observed result or native instruction trace.
    #[must_use]
    pub const fn boundary(self) -> &'static str {
        match self {
            Self::WrongControlParity | Self::UncommittedLeaf => {
                "taproot commitment refusal before clause execution"
            }
            Self::KeyPathAttempt => "key-path signature refusal; no absent-discrete-log proof",
            Self::WrongKey | Self::MixedKeys => {
                "program equality; shared equality error requires independent pair evidence"
            }
            _ => "exact mined script-path readback",
        }
    }
    const fn reserve(self) -> usize {
        match self {
            Self::MixedKeys | Self::SameKeyControl => 5,
            Self::CoexistenceL => 6,
            _ => 4,
        }
    }
}

/// Canonical public recipes; programs can only enter through reconstruction.
#[derive(Clone, Debug)]
pub struct KeyedRecipes {
    descriptors: [KeyedConstructorDescriptor; 2],
}

impl KeyedRecipes {
    /// Construct distinct deployment keys; fixed test handles live only in tests.
    /// # Errors
    /// Refuses invalid reconstruction or equal keys/programs.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        keys: [[u8; 32]; 2],
    ) -> Result<Self, BirthNativeRefusal> {
        let _span = tracing::debug_span!("keyed_recipes").entered();
        let build = |key| {
            let descriptor = construct_keyed(target, KeyedSemanticKey::new(key), &PublicKeyedCurve)
                .map_err(|_| BirthNativeRefusal::Leaf)?;
            reconstruct_keyed(target, &descriptor, &PublicKeyedCurve)
                .map_err(|_| BirthNativeRefusal::Leaf)
        };
        let descriptors = [build(keys[0])?, build(keys[1])?];
        if keys[0] == keys[1] || descriptors[0].output_program == descriptors[1].output_program {
            return Err(BirthNativeRefusal::Leaf);
        }
        Ok(Self { descriptors })
    }
    /// Full public reconstruction data, in k,l order.
    #[must_use]
    pub const fn descriptors(&self) -> &[KeyedConstructorDescriptor; 2] {
        &self.descriptors
    }
    fn leaf(&self, key: usize) -> BirthLeaf {
        let d = &self.descriptors[key];
        BirthLeaf {
            script: d.leaf_bytes.clone(),
            program: d.output_program.clone(),
            control_block: d.control.bytes.clone(),
        }
    }
}

/// Construct one subject from four mined parents and its fixed sponsor reserve.
/// Parent order is single k, mixed first k, mixed second k, second l.
/// # Errors
/// Refuses a mismatched parent, asset/value or insufficient reserve.
pub fn build_keyed_spend(
    recipes: &KeyedRecipes,
    parents: &[TargetTransaction; 4],
    coin: &FundedOutput,
    id: KeyedSubjectId,
) -> Result<TargetTransaction, BirthNativeRefusal> {
    let _span = tracing::debug_span!("build_keyed_spend", ?id).entered();
    let (parent, key) = match id {
        KeyedSubjectId::MixedKeys | KeyedSubjectId::SameKeyControl => (1, 0),
        KeyedSubjectId::CoexistenceL => (3, 1),
        _ => (0, 0),
    };
    let base = build_birth_spend(
        &recipes.leaf(key),
        &parents[parent],
        coin,
        BirthSubjectId::A0,
    )?;
    let mut inputs = base.inputs().to_vec();
    let mut outputs = base.outputs().to_vec();
    let mut witnesses = base.witnesses().to_vec();
    match id {
        KeyedSubjectId::WrongControlParity => {
            let mut stack = witnesses[0].stack().to_vec();
            stack[1][0] ^= 1;
            witnesses[0] = InputWitness::new(stack);
        }
        KeyedSubjectId::UncommittedLeaf => {
            let mut stack = witnesses[0].stack().to_vec();
            stack[0][1] ^= 1;
            witnesses[0] = InputWitness::new(stack);
        }
        KeyedSubjectId::KeyPathAttempt => {
            witnesses[0] = InputWitness::new(vec![vec![0; 64]]);
        }
        KeyedSubjectId::WrongKey => {
            outputs[0] = output(&coin_output(coin)?, BIRTH_VALUE, recipes.leaf(1).program);
        }
        KeyedSubjectId::MixedKeys | KeyedSubjectId::SameKeyControl => {
            let (second, key) = if id == KeyedSubjectId::MixedKeys {
                (3, 1)
            } else {
                (2, 0)
            };
            let leaf = recipes.leaf(key);
            let expected = output(&coin_output(coin)?, BIRTH_VALUE, leaf.program);
            if parents[second].outputs().first() != Some(&expected)
                || identity(&parents[second]) == identity(&parents[parent])
            {
                return Err(BirthNativeRefusal::Pair);
            }
            inputs.insert(
                1,
                TargetInput::new(
                    Outpoint::new(identity(&parents[second]), 0)
                        .map_err(|_| BirthNativeRefusal::Transaction)?,
                    u32::MAX,
                ),
            );
            witnesses.insert(1, InputWitness::new(vec![leaf.script, leaf.control_block]));
            outputs[0] = output(
                &coin_output(coin)?,
                2 * BIRTH_VALUE,
                recipes.leaf(0).program,
            );
        }
        _ => {}
    }
    transaction(inputs, outputs, witnesses)
}

/// Frozen linked signing and submission responses; all outcomes remain data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedSubjectRecord {
    /// Stable subject identity.
    pub id: KeyedSubjectId,
    /// Exact sponsor request and complete response.
    pub signing: (OperationStep, NativeOperationResponse),
    /// Frozen bytes and full relay/consensus/readback evidence.
    pub submission: (OperationStep, NativeOperationResponse),
}
impl KeyedSubjectRecord {
    /// Report projection; admission replays the wire journal instead.
    #[must_use]
    pub fn capture_json(&self) -> Value {
        json!({"subject": format!("{:?}", self.id), "coordinate": self.id.coordinate(), "collateral": self.id.collateral(), "boundary": self.id.boundary(), "control": self.id.control().map(|c| format!("{c:?}")), "signing_request": self.signing.0.subject(), "signing_response": self.signing.1, "submission_request": self.submission.0.subject(), "submission_response": self.submission.1})
    }
}

/// One funding, four signed/mined parents and eight signed/submitted subjects.
#[derive(Clone, Debug)]
pub struct KeyedCampaignPlanner {
    target: ReviewedElementsTapscriptDefinition,
    recipes: KeyedRecipes,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    coins: Vec<FundedOutput>,
    parents: Vec<TargetTransaction>,
    signed: Option<TargetTransaction>,
    records: Vec<KeyedSubjectRecord>,
    refusal: Option<BirthNativeRefusal>,
}
impl KeyedCampaignPlanner {
    /// Start without a node or any observed outcomes.
    /// # Errors
    /// Refuses invalid or non-distinct reconstructed recipes.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        keys: [[u8; 32]; 2],
    ) -> Result<Self, BirthNativeRefusal> {
        Ok(Self {
            target: target.clone(),
            recipes: KeyedRecipes::new(target, keys)?,
            pending: None,
            exchanges: vec![],
            coins: vec![],
            parents: vec![],
            signed: None,
            records: vec![],
            refusal: None,
        })
    }
    /// Public descriptors included in the capture.
    #[must_use]
    pub const fn recipes(&self) -> &KeyedRecipes {
        &self.recipes
    }
    /// All linked exchanges, including infrastructure and parent evidence.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }
    /// Subject records in submission order.
    #[must_use]
    pub fn records(&self) -> &[KeyedSubjectRecord] {
        &self.records
    }
    /// Construction/replay stop reason.
    #[must_use]
    pub const fn refusal(&self) -> Option<BirthNativeRefusal> {
        self.refusal
    }
    /// Complete request census; this does not admit outcomes.
    /// # Errors
    /// Refuses incomplete or stopped runs.
    pub const fn complete(&self) -> Result<(), BirthNativeRefusal> {
        if let Some(reason) = self.refusal {
            return Err(reason);
        }
        if self.exchanges.len() != 25
            || self.records.len() != 8
            || self.parents.len() != 4
            || self.pending.is_some()
        {
            return Err(BirthNativeRefusal::Replay);
        }
        Ok(())
    }
    fn sign(tx: &TargetTransaction, coin: &FundedOutput, index: u16) -> OperationSubject {
        OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
            finalized_transaction: tx.encode(),
            sponsor_input_index: index,
            sponsor_outpoint: coin.outpoint.clone(),
            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
        }))
    }
    fn next(&self) -> Result<Option<OperationStep>, BirthNativeRefusal> {
        let n = self.exchanges.len();
        if n == 25 {
            return Ok(None);
        }
        let subject = if n == 0 {
            OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 7,
                amount_per_sponsor_output: SPONSOR_VALUE,
            }))
        } else if n % 2 == 0 {
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self
                    .signed
                    .as_ref()
                    .ok_or(BirthNativeRefusal::Replay)?
                    .encode(),
            }))
        } else if n < 9 {
            let p = (n - 1) / 2;
            Self::sign(
                &build_birth_parent(&self.recipes.leaf(usize::from(p == 3)), &self.coins[p])?,
                &self.coins[p],
                0,
            )
        } else {
            let id = KeyedSubjectId::SUBMISSION_ORDER[(n - 9) / 2];
            let parents: &[TargetTransaction; 4] = self
                .parents
                .as_slice()
                .try_into()
                .map_err(|_| BirthNativeRefusal::Replay)?;
            Self::sign(
                &build_keyed_spend(&self.recipes, parents, &self.coins[id.reserve()], id)?,
                &self.coins[id.reserve()],
                if matches!(
                    id,
                    KeyedSubjectId::MixedKeys | KeyedSubjectId::SameKeyControl
                ) {
                    2
                } else {
                    1
                },
            )
        };
        Ok(Some(OperationStep::new(&format!("keyed-{n}"), subject)))
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
                if response.observed_layer != ObservedOutcomeLayer::Accepted
                    || response.funded_outputs.len() != 7
                {
                    return Err(BirthNativeRefusal::Funding);
                }
                let first = &response.funded_outputs[0];
                let fact = coin_output(first)?;
                for (i, coin) in response.funded_outputs.iter().enumerate() {
                    if coin.outpoint.vout as usize != i
                        || coin.outpoint.txid != first.outpoint.txid
                        || coin_output(coin)? != fact
                        || fact.value != SPONSOR_VALUE
                        || fact.program.len() != 22
                        || fact.program[..2] != [0, 20]
                    {
                        return Err(BirthNativeRefusal::Funding);
                    }
                    coin_outpoint(coin)?;
                }
                self.coins.clone_from(&response.funded_outputs);
            }
            OperationSubject::SponsorSigning(request) => {
                let reserve = if n < 9 {
                    (n - 1) / 2
                } else {
                    KeyedSubjectId::SUBMISSION_ORDER[(n - 9) / 2].reserve()
                };
                self.signed = Some(apply_birth_signature(
                    request,
                    response,
                    &self.coins[reserve],
                )?);
            }
            OperationSubject::Submission(request) if n < 9 => {
                if response.observed_layer != ObservedOutcomeLayer::Accepted {
                    return Err(BirthNativeRefusal::Readback);
                }
                check_readback(&request.transaction_bytes, response)
                    .map_err(|_| BirthNativeRefusal::Readback)?;
                let parent = TargetTransaction::decode(
                    &response
                        .mined_readback
                        .as_ref()
                        .ok_or(BirthNativeRefusal::Readback)?
                        .raw_transaction,
                )
                .map_err(|_| BirthNativeRefusal::Readback)?;
                let key = usize::from(self.parents.len() == 3);
                authenticate_keyed_predecessor(
                    &self.target,
                    &self.recipes.descriptors[key],
                    parent.outputs()[0].program(),
                    &PublicKeyedCurve,
                )
                .map_err(|_| BirthNativeRefusal::Pair)?;
                if self
                    .parents
                    .iter()
                    .any(|p| identity(p) == identity(&parent))
                {
                    return Err(BirthNativeRefusal::Pair);
                }
                self.parents.push(parent);
            }
            OperationSubject::Submission(_) => {
                self.records.push(KeyedSubjectRecord {
                    id: KeyedSubjectId::SUBMISSION_ORDER[self.records.len()],
                    signing: self
                        .exchanges
                        .last()
                        .ok_or(BirthNativeRefusal::Replay)?
                        .clone(),
                    submission: (step.clone(), response.clone()),
                });
            }
            _ => return Err(BirthNativeRefusal::Response),
        }
        Ok(())
    }
}
impl TargetOperationPlanner for KeyedCampaignPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let _span =
            tracing::debug_span!("keyed_campaign_step", completed = self.exchanges.len()).entered();
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
        result.map_err(|reason| {
            tracing::debug!(?reason, "keyed campaign stopped");
            self.refusal = Some(reason);
            PlanRefused
        })
    }
}

/// Reconstruct and replay every request and signing echo against full responses.
/// # Errors
/// Refuses omitted, changed or surplus exchanges and invalid construction.
pub fn replay_keyed_campaign(
    target: &ReviewedElementsTapscriptDefinition,
    keys: [[u8; 32]; 2],
    exchanges: &[(OperationStep, NativeOperationResponse)],
) -> Result<KeyedCampaignPlanner, BirthNativeRefusal> {
    let _span =
        tracing::debug_span!("replay_keyed_campaign", exchanges = exchanges.len()).entered();
    let planner = replay_operations(
        KeyedCampaignPlanner::new(target, keys)?,
        exchanges,
        KeyedCampaignPlanner::refusal,
    )?;
    planner.complete()?;
    Ok(planner)
}

/// Shared complete wire carrier, with this campaign's descriptors and schema.
#[must_use]
pub fn keyed_capture_json(capture: &NativeOperationCapture, recipes: &KeyedRecipes) -> Value {
    let mut journal = continuation_capture_json(capture);
    journal
        .as_object_mut()
        .map(|o| o.remove("continuation_wire_schema"));
    journal["keyed_wire_schema"] = json!(1);
    journal["descriptors"] = json!(recipes.descriptors());
    journal
}

/// Read only wire JSON, reconstruct both descriptors, then replay the journal.
/// # Errors
/// Refuses incomplete responses, malformed links or changed recipe/requests.
pub fn replay_keyed_capture(
    target: &ReviewedElementsTapscriptDefinition,
    journal: &Value,
) -> Result<KeyedCampaignPlanner, BirthNativeRefusal> {
    let _span = tracing::debug_span!("replay_keyed_capture").entered();
    if journal["keyed_wire_schema"] != 1 || journal["terminal"] != "complete" {
        return Err(BirthNativeRefusal::Replay);
    }
    let descriptors: [KeyedConstructorDescriptor; 2] =
        serde_json::from_value(journal["descriptors"].clone())
            .map_err(|_| BirthNativeRefusal::Leaf)?;
    for d in &descriptors {
        reconstruct_keyed(target, d, &PublicKeyedCurve).map_err(|_| BirthNativeRefusal::Leaf)?;
    }
    replay_keyed_campaign(
        target,
        [descriptors[0].semantic_key, descriptors[1].semantic_key],
        &decode_wire_operations(journal)?,
    )
}

/// Conservative capture classification, not archive admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedObservationClass {
    /// Exact mined bytes under the canonical script path.
    MinedControl,
    /// Exact commitment error beside the same-coin script control.
    CommitmentRefusal,
    /// Exact Schnorr error with a reconstructed key-path witness.
    KeyPathSignatureRefusal,
    /// Equality error isolated by the fixed single-key successor coordinate.
    ProgramEqualityRefusal,
    /// Equality error isolated to the l handle beside the same-key control.
    MixedKeyEqualityRefusal,
    /// Ambiguous, earlier-layer or incomplete evidence.
    Open,
}

/// Classify rebuilt rows; raw adapter layers and exact error text are retained.
///
/// The adapter labels a sponsored key-path attempt as script-path rejection
/// because not every input uses a taproot key path. Exact Schnorr text plus the
/// frozen keyed witness separates it from the sponsor's ECDSA path. This is not
/// an instruction trace, ECDSA verification or proof of absent discrete log.
#[must_use]
pub fn classify_keyed_subject(
    planner: &KeyedCampaignPlanner,
    id: KeyedSubjectId,
) -> KeyedObservationClass {
    let classify = || -> Option<KeyedObservationClass> {
        planner.complete().ok()?;
        let row = planner.records.iter().find(|r| r.id == id)?;
        let OperationSubject::Submission(request) = row.submission.0.subject() else {
            return None;
        };
        let response = &row.submission.1;
        let observation: Value = serde_json::from_str(response.observed_detail.as_deref()?).ok()?;
        detail(response).ok()?;
        if let Some(control) = id.control() {
            if classify_keyed_subject(planner, control) != KeyedObservationClass::MinedControl {
                return None;
            }
        } else {
            if response.observed_layer != ObservedOutcomeLayer::Accepted
                || !matches!(
                    observation["route"].as_str(),
                    Some("relay-and-template" | "direct-block")
                )
            {
                return None;
            }
            check_readback(&request.transaction_bytes, response).ok()?;
            if id == KeyedSubjectId::CoexistenceL {
                if classify_keyed_subject(planner, KeyedSubjectId::ControlK)
                    != KeyedObservationClass::MinedControl
                {
                    return None;
                }
                let k = planner
                    .records
                    .iter()
                    .find(|r| r.id == KeyedSubjectId::ControlK)?;
                if response.mined_readback.as_ref()?.block_height
                    <= k.submission.1.mined_readback.as_ref()?.block_height
                {
                    return None;
                }
            }
            return Some(KeyedObservationClass::MinedControl);
        }
        if !matches!(
            response.observed_layer,
            ObservedOutcomeLayer::ScriptPathRejection | ObservedOutcomeLayer::KeyPathRejection
        ) || observation["route"] != "refused"
            || observation["relay_allowed"] != false
            || response.accepted_txid.is_some()
            || response.mined_readback.is_some()
        {
            return None;
        }
        let tx = TargetTransaction::decode(&request.transaction_bytes).ok()?;
        if observation["detail"]["txid"] != identity(&tx).to_target_display() {
            return None;
        }
        let reason = observation["relay_detail"].as_str()?;
        if observation["detail"]["verdict_reason"] != reason {
            return None;
        }
        let error = reason
            .strip_prefix("mandatory-script-verify-flag-failed (")
            .or_else(|| reason.strip_prefix("non-mandatory-script-verify-flag ("))?
            .strip_suffix(')')?;
        match (id, error) {
            (KeyedSubjectId::KeyPathAttempt, "Invalid Schnorr signature") => {
                Some(KeyedObservationClass::KeyPathSignatureRefusal)
            }
            (
                KeyedSubjectId::WrongControlParity | KeyedSubjectId::UncommittedLeaf,
                "Witness program hash mismatch",
            ) if response.observed_layer == ObservedOutcomeLayer::ScriptPathRejection => {
                Some(KeyedObservationClass::CommitmentRefusal)
            }
            (KeyedSubjectId::WrongKey, "Script failed an OP_EQUALVERIFY operation")
                if response.observed_layer == ObservedOutcomeLayer::ScriptPathRejection =>
            {
                Some(KeyedObservationClass::ProgramEqualityRefusal)
            }
            (KeyedSubjectId::MixedKeys, "Script failed an OP_EQUALVERIFY operation")
                if response.observed_layer == ObservedOutcomeLayer::ScriptPathRejection =>
            {
                Some(KeyedObservationClass::MixedKeyEqualityRefusal)
            }
            _ => None,
        }
    };
    classify().unwrap_or(KeyedObservationClass::Open)
}
