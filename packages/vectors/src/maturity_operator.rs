//! One synthetic operator authorization over public live-transfer fixtures.
//!
//! The predecessor commits a fixed public operator key through a checked leaf.
//! Successor programs retain the live fixture's commitments. Complete STATE
//! construction, metadata and nonce commitments remain later obligations.
//! An in-process registry cannot establish global operator non-equivocation.

use linker::live_backend::LiveTransferRepresentationPlan;
use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding, OwnerParameter};
use tapscript::{
    EstablishedOperatorProfile, OperatorKey, StackItem, TapscriptInstruction, TapscriptProgram,
    operator_key_encoding_closure, selected_operator_profile,
};
use target_elements::{EncodingClass, LeafVersion, OpcodeId};
use target_elements_conformance::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use target_elements_conformance::owner_key_oracle::verify_owner_signature;
use target_elements_conformance::test_material::PublicTestSignerHandle;
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, ValueField};
use transaction::live_construct::finalize_live_transfer;
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::live_taproot::LiveCurveCapability;
use transaction::operator_signing::{
    OperatorSigningInput, OperatorSigningRequest, ScriptPathSignatureVerifier,
    ScriptPathVerifierRejection,
};
use transaction::script_path_signing::{LiveDeployment, SpentOutputCensusEntry};
use transaction::taproot::{Digest32, leaf_hash, witness_program_script};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_plan::{demonstration_live_abi, reviewed_target};

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetFundingSubject, TargetScriptPathSigningSubject, TargetSubmissionSubject,
    WireSighashProfile, WireSpentOutput, WireTapleaf,
};
use transaction::bytes::{InputWitness, TargetInput};
use transaction::operator_right::{
    BranchContext, ConstructionRight, OperatorRightOutcome, OperatorRightRegistry, RightRefusal,
    RightScope,
};
use transaction::operator_signing::{OperatorSigningResponse, authorize_operator_under_right};

use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed_order};

/// The exact native step order, with the consuming acceptance last.
pub const NATIVE_STEPS: [&str; 11] = [
    "issue-operator-asset",
    "fund-operator-predecessor",
    "sign-wrong-key",
    "sign-operator",
    "wrong-key",
    "wrong-candidate",
    "wrong-leaf",
    "signature-width",
    "signature-type",
    "protected-term",
    "operator-positive",
];

/// The spend, eight controls, verification and the unresolved membership premise.
pub const EVIDENCE_ROWS: [&str; 11] = [
    "operator-positive",
    "wrong-key",
    "wrong-candidate",
    "wrong-leaf",
    "unknown-key-admission",
    "signature-width",
    "signature-type",
    "protected-term",
    "duplicate-authorization",
    "signature-verification",
    "operator-membership",
];

/// One transcript-derived standing, retaining the actual refusal layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorEvidenceRow {
    /// The stable subject within this ceremony.
    pub subject: &'static str,
    /// The native layer required by this subject, if any.
    pub expected_layer: Option<ObservedOutcomeLayer>,
    /// The layer actually returned for this exact subject.
    pub observed_layer: Option<ObservedOutcomeLayer>,
    /// The existing evidence vocabulary keeps unanswered obligations visible.
    pub standing: crate::live_evidence::LiveRowStanding,
}

/// Private-evidence operands derived by replaying the exact operation transcript.
///
/// Mock exchanges exercise verification but cannot establish target acceptance.
/// The protocol returns a signature and public context, not a digest echo:
/// independent verification binds that signature to the recomputed message.
/// Realization's operator relation requires external evidence and exposes no
/// observed-operator membership decision. Complete STATE construction remains
/// open; no metadata or nonce claim is inferred from this receipt candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorEvidence {
    /// Realization leaves approved-operator membership to external evidence.
    pub membership_requirement: realization::ExternalEvidenceRequirement,
    /// STATE fields for which this receipt candidate provides no construction.
    pub unconstructed_state: Vec<realization::FactId>,
    /// One standing for every declared subject and remaining premise.
    pub rows: Vec<OperatorEvidenceRow>,
    /// The independently recomputed signing message, after completed replay.
    pub message: Option<Vec<u8>>,
    /// The returned key, profile, genesis and candidate echo matched the binding.
    pub public_context_verified: bool,
    /// The public-data Schnorr verifier accepted the returned signature.
    pub signature_verified: bool,
    /// The accepted candidate was read back without any byte changing.
    pub byte_identical_readback: bool,
}

impl OperatorEvidence {
    /// Replays admitted subjects through the planner's existing settlement.
    pub(crate) fn replay_recorded(
        identity: CandidateDeploymentIdentity,
        exchanges: &[(OperationSubject, NativeOperationResponse)],
    ) -> Result<Self, crate::live_corpus_native_operator::NativeOperatorImportRefusal> {
        use crate::live_corpus_native_operator::NativeOperatorImportRefusal as Refusal;
        use target_elements_conformance::executor::ExecutorTrust;
        if exchanges.len() != NATIVE_STEPS.len() {
            return Err(Refusal::ReplaySettlement { step: "census" });
        }
        let mut planner = OperatorPlanner::new(identity);
        let mut next = planner
            .next_step(None)
            .map_err(|_| Refusal::ReplaySettlement {
                step: NATIVE_STEPS[0],
            })?;
        for ((subject, response), name) in exchanges.iter().zip(NATIVE_STEPS) {
            let current = next.ok_or(Refusal::ReplaySubject { step: name })?;
            if current.subject() != subject || current.case() != &response.case {
                return Err(Refusal::ReplaySubject { step: name });
            }
            next = planner
                .next_step(Some((current.case(), response)))
                .map_err(|_| Refusal::ReplaySettlement { step: name })?;
        }
        if next.is_some() {
            return Err(Refusal::ReplaySettlement { step: "surplus" });
        }
        planner
            .completed_evidence(ExecutorTrust::ReviewedNonMock)
            .map_err(|_| Refusal::ReplaySettlement { step: "completion" })
    }

    /// An unanswered record for an absent or interrupted exchange.
    #[must_use]
    pub fn incomplete() -> Self {
        Self {
            membership_requirement:
                realization::ExternalEvidenceRequirement::OperatorAuthorization {
                    operation: architecture::OperationId::AnnounceMaturity,
                },
            unconstructed_state: [
                realization::TransactionSide::Input,
                realization::TransactionSide::Output,
            ]
            .into_iter()
            .flat_map(|side| {
                realization::StateField::ALL
                    .iter()
                    .copied()
                    .map(move |field| realization::FactId::StateField {
                        operation: architecture::OperationId::AnnounceMaturity,
                        side,
                        field,
                    })
            })
            .collect(),
            rows: EVIDENCE_ROWS
                .iter()
                .map(|subject| OperatorEvidenceRow {
                    subject,
                    expected_layer: expected_layer(subject),
                    observed_layer: None,
                    standing: crate::live_evidence::LiveRowStanding::NativeRunRequired(None),
                })
                .collect(),
            message: None,
            public_context_verified: false,
            signature_verified: false,
            byte_identical_readback: false,
        }
    }

    /// Replays every request and response under the supplied deployment.
    ///
    /// # Errors
    /// Refuses a different environment, altered subject, missing response,
    /// surplus operation, invalid signature or changed positive readback.
    pub fn from_transcript(
        identity: CandidateDeploymentIdentity,
        transcript: &target_elements_conformance::executor::ExecutionTranscript,
    ) -> Result<Self, VectorError> {
        if transcript.environment().network_id != *identity.network_id()
            || transcript.environment().genesis_id != *identity.genesis_id()
            || transcript.operation_requests().len() != NATIVE_STEPS.len()
            || transcript.operation_responses().len() != NATIVE_STEPS.len()
        {
            return Err(VectorError::LiveSubstrateUnavailable);
        }
        let mut planner = OperatorPlanner::new(identity);
        let mut step = substrate(planner.next_step(None))?;
        while let Some(current) = step {
            if transcript.operation_requests().get(current.case()) != Some(current.subject()) {
                return Err(VectorError::LiveSubstrateUnavailable);
            }
            let response = transcript
                .operation_responses()
                .get(current.case())
                .ok_or(VectorError::LiveSubstrateUnavailable)?;
            step = substrate(planner.next_step(Some((current.case(), response))))?;
        }
        planner.completed_evidence(transcript.trust())
    }
}

/// The layer each submitted subject declares before execution.
#[must_use]
pub fn expected_layer(subject: &str) -> Option<ObservedOutcomeLayer> {
    match subject {
        "operator-positive" => Some(ObservedOutcomeLayer::Accepted),
        "wrong-key" | "wrong-candidate" | "wrong-leaf" | "signature-width" | "signature-type"
        | "protected-term" => Some(ObservedOutcomeLayer::ScriptPathRejection),
        _ => None,
    }
}

impl OperatorPlanner {
    fn completed_evidence(
        &self,
        trust: target_elements_conformance::executor::ExecutorTrust,
    ) -> Result<OperatorEvidence, VectorError> {
        use crate::live_evidence::{LiveRowStanding, NativeObservedAcceptance};
        use target_elements_conformance::executor::ExecutorTrust;
        if self.position != NATIVE_STEPS.len() || self.exchanges.len() != NATIVE_STEPS.len() {
            return Ok(OperatorEvidence::incomplete());
        }
        let fixture = self
            .fixture
            .as_ref()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        let positive = &self.exchanges[10].1;
        self.settle_positive(positive)?;
        let identity = positive
            .accepted_txid
            .as_deref()
            .and_then(|value| transaction::Txid::from_target_display(value).ok())
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        let mut evidence = OperatorEvidence::incomplete();
        evidence.message = Some(fixture.freeze()?.message().with_vector_grown().to_vec());
        evidence.public_context_verified = true;
        evidence.signature_verified = true;
        evidence.byte_identical_readback = true;
        for row in &mut evidence.rows {
            if let Some((_, response)) = self
                .exchanges
                .iter()
                .find(|(step, _)| step.case().step == row.subject)
            {
                row.observed_layer = Some(response.observed_layer);
                if trust == ExecutorTrust::ReviewedNonMock {
                    if row.subject == "operator-positive" {
                        row.standing = LiveRowStanding::NativeRunObserved {
                            acceptance: NativeObservedAcceptance::Single(identity),
                        };
                    } else if row.expected_layer == row.observed_layer {
                        row.standing = LiveRowStanding::NativeRefusalObserved {
                            declared_boundary: crate::matrix::EvidenceBoundary::ScriptPathRejection,
                            observed_layer: response.observed_layer,
                            control_identity: identity,
                            refusal_detail: response.observed_detail.clone().unwrap_or_default(),
                        };
                    }
                }
            }
            let fact = match row.subject {
                "unknown-key-admission" if self.unknown_key_refused => {
                    Some("deployment binding refused an unbound operator key")
                }
                "duplicate-authorization" if self.duplicate_refused => {
                    Some("affine registry refused a second outstanding right before signing")
                }
                "signature-verification" => Some(
                    "public-data Schnorr verification bound the signature to the recomputed message",
                ),
                _ => None,
            };
            if let Some(fact) = fact {
                row.standing = LiveRowStanding::FirstPartyFactObserved {
                    fact,
                    observed_by: "OperatorEvidence::from_transcript",
                };
            }
        }
        Ok(evidence)
    }
}

impl OperatorFixture {
    /// The generic signer's exact public context.
    ///
    /// # Errors
    /// Returns a substrate refusal if the public fixture cannot be frozen.
    pub fn signing_subject(&self) -> Result<TargetScriptPathSigningSubject, VectorError> {
        let request = self.freeze()?;
        let spent_outputs = self
            .spent
            .iter()
            .map(|spent| {
                let mut asset_field = Vec::new();
                match spent.asset() {
                    AssetField::Explicit(asset) => {
                        asset_field.push(1);
                        asset_field.extend_from_slice(asset.internal());
                    }
                    AssetField::Commitment(bytes) => asset_field.extend_from_slice(&bytes),
                    _ => return Err(VectorError::LiveSubstrateUnavailable),
                }
                let mut value_field = Vec::new();
                match spent.value() {
                    ValueField::Explicit(amount) => {
                        value_field.push(1);
                        value_field.extend_from_slice(&amount.to_be_bytes());
                    }
                    ValueField::Commitment(bytes) => value_field.extend_from_slice(&bytes),
                    _ => return Err(VectorError::LiveSubstrateUnavailable),
                }
                Ok(WireSpentOutput {
                    asset_field,
                    value_field,
                    program: spent.program().to_vec(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TargetScriptPathSigningSubject {
            finalized_transaction: request.frozen_bytes().to_vec(),
            input_index: request.input_index(),
            spent_outputs,
            executing_leaf: WireTapleaf {
                leaf_version: self.input.leaf_version().get(),
                script: self.input.leaf_script().to_vec(),
                control_block: self.input.control_block().to_vec(),
            },
            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            signer: OPERATOR_HANDLE,
        })
    }
}

/// A single-run planner retaining the registry across the signing exchange.
pub struct OperatorPlanner {
    identity: CandidateDeploymentIdentity,
    position: usize,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    asset: Option<String>,
    fixture: Option<OperatorFixture>,
    registry: OperatorRightRegistry,
    right: Option<ConstructionRight>,
    wrong_key_signature: Vec<u8>,
    authorized: Option<TargetTransaction>,
    duplicate_refused: bool,
    unknown_key_refused: bool,
}

impl OperatorPlanner {
    /// Starts a run under an explicit deployment identity.
    #[must_use]
    pub fn new(identity: CandidateDeploymentIdentity) -> Self {
        Self {
            identity,
            position: 0,
            pending: None,
            exchanges: Vec::new(),
            asset: None,
            fixture: None,
            registry: OperatorRightRegistry::default(),
            right: None,
            wrong_key_signature: Vec::new(),
            authorized: None,
            duplicate_refused: false,
            unknown_key_refused: false,
        }
    }

    /// The funded, frozen fixture once its predecessor exists.
    #[must_use]
    pub const fn fixture(&self) -> Option<&OperatorFixture> {
        self.fixture.as_ref()
    }

    fn make_step(&self) -> Result<Option<OperationStep>, VectorError> {
        let Some(name) = NATIVE_STEPS.get(self.position) else {
            return Ok(None);
        };
        let subject = match self.position {
            0 | 1 => {
                let target = reviewed_target()?;
                let closure = operator_key_encoding_closure(target.definition().authorization());
                let key = substrate(OperatorKey::new(
                    &closure,
                    closure.approved(),
                    substrate(OPERATOR_HANDLE.x_only_public_key())?.to_vec(),
                ))?;
                OperationSubject::Funding(Box::new(TargetFundingSubject {
                    issue_asset: self.position == 0,
                    asset: self.asset.clone(),
                    output_program: operator_leaf(&key)?.0,
                    outputs: 1,
                    amount_per_output: PREDECESSOR_AMOUNT,
                }))
            }
            2 | 3 => {
                let mut subject = self
                    .fixture
                    .as_ref()
                    .ok_or(VectorError::LiveSubstrateUnavailable)?
                    .signing_subject()?;
                if self.position == 2 {
                    subject.signer = PublicTestSignerHandle::First;
                }
                OperationSubject::ScriptPathSigning(Box::new(subject))
            }
            _ => OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self.submission(self.position)?.encode(),
            })),
        };
        Ok(Some(OperationStep::new(name, subject)))
    }

    fn settle(&mut self, response: &NativeOperationResponse) -> Result<(), VectorError> {
        if self.position < 4 && response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(VectorError::LiveSubstrateUnavailable);
        }
        match self.position {
            0 => {
                let asset = response
                    .issued_asset
                    .as_ref()
                    .filter(|value| asset_of(value).is_some())
                    .ok_or(VectorError::LiveSubstrateUnavailable)?;
                self.asset = Some(asset.clone());
            }
            1 => self.settle_funding(response)?,
            2 => {
                self.check_signing(response, PublicTestSignerHandle::First)?;
                self.wrong_key_signature
                    .clone_from(&response.script_path_witness[0]);
            }
            3 => self.settle_signing(response)?,
            10 => self.settle_positive(response)?,
            _ => {}
        }
        Ok(())
    }

    fn settle_funding(&mut self, response: &NativeOperationResponse) -> Result<(), VectorError> {
        let [coin] = response.funded_outputs.as_slice() else {
            return Err(VectorError::LiveSubstrateUnavailable);
        };
        if Some(&coin.asset) != self.asset.as_ref() || coin.amount_satoshis != PREDECESSOR_AMOUNT {
            return Err(VectorError::LiveSubstrateUnavailable);
        }
        let fixture = OperatorFixture::new(
            self.identity.clone(),
            outpoint_of(&coin.outpoint).ok_or(VectorError::LiveSubstrateUnavailable)?,
            asset_of(&coin.asset).ok_or(VectorError::LiveSubstrateUnavailable)?,
        )?;
        if decode_hex(&coin.script).as_deref() != Some(fixture.spent[0].program()) {
            return Err(VectorError::LiveSubstrateUnavailable);
        }
        let request = fixture.freeze()?;
        let branch = substrate(BranchContext::new(*self.identity.genesis_id(), 0))?;
        let scope = substrate(RightScope::new(&request, branch))?;
        let right = substrate(self.registry.issue(scope.clone(), request.frozen_bytes()))?;
        self.duplicate_refused = matches!(
            self.registry.issue(scope, request.frozen_bytes()),
            Err(RightRefusal::Outstanding(_))
        );
        let target = reviewed_target()?;
        let closure = operator_key_encoding_closure(target.definition().authorization());
        let foreign = substrate(OperatorKey::new(
            &closure,
            closure.approved(),
            substrate(PublicTestSignerHandle::First.x_only_public_key())?.to_vec(),
        ))?;
        self.unknown_key_refused = matches!(
            fixture
                .binding
                .check(&foreign, &self.identity, target.definition().version()),
            Err(linker::LinkRefusal::OperatorKeyMismatch { .. })
        );
        self.right = Some(right);
        self.fixture = Some(fixture);
        Ok(())
    }

    fn check_signing(
        &self,
        response: &NativeOperationResponse,
        signer: PublicTestSignerHandle,
    ) -> Result<(), VectorError> {
        substrate(response.validate_shape())?;
        let fixture = self
            .fixture
            .as_ref()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        if response.signer_public_key != Some(substrate(signer.x_only_public_key())?)
            || response.signing_genesis != Some(*self.identity.genesis_id())
            || response.signed_profile != Some(WireSighashProfile::AllInputsAllOutputs)
            || response.signature_bound_to.as_deref() != Some(fixture.candidate.encode().as_slice())
        {
            return Err(VectorError::LiveSubstrateUnavailable);
        }
        substrate(OperatorVerifier.verify(
            &substrate(signer.x_only_public_key())?,
            fixture.freeze()?.message().with_vector_grown(),
            &response.script_path_witness[0],
        ))
    }

    fn settle_signing(&mut self, response: &NativeOperationResponse) -> Result<(), VectorError> {
        self.check_signing(response, OPERATOR_HANDLE)?;
        let fixture = self
            .fixture
            .as_ref()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        let request = fixture.freeze()?;
        let answer = OperatorSigningResponse::new(
            request.input_index(),
            response.script_path_witness[0].clone(),
            0,
            response
                .signature_bound_to
                .clone()
                .ok_or(VectorError::LiveSubstrateUnavailable)?,
            fixture.binding.key().clone(),
            self.identity.clone(),
            fixture.binding.capability_revision(),
        );
        let right = self
            .right
            .take()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        let authorized = substrate(authorize_operator_under_right(
            &mut self.registry,
            right,
            request,
            [answer],
            &OperatorVerifier,
        ))?;
        let OperatorRightOutcome::Fresh(authorized) = authorized else {
            return Err(VectorError::LiveSubstrateUnavailable);
        };
        self.authorized = Some(rebuild(
            &fixture.candidate,
            fixture.candidate.inputs().to_vec(),
            fixture.candidate.outputs().to_vec(),
            vec![authorized.witness().clone()],
        )?);
        Ok(())
    }

    fn settle_positive(&self, response: &NativeOperationResponse) -> Result<(), VectorError> {
        let candidate = self
            .authorized
            .as_ref()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        let readback = response
            .mined_readback
            .as_ref()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        if response.observed_layer != ObservedOutcomeLayer::Accepted
            || response.accepted_txid.as_ref() != Some(&readback.transaction_id)
            || readback.raw_transaction != candidate.encode()
        {
            return Err(VectorError::LiveSubstrateUnavailable);
        }
        let decoded = substrate(TargetTransaction::decode(&readback.raw_transaction))?;
        let signature = decoded
            .witnesses()
            .first()
            .and_then(|witness| witness.stack().first())
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        let fixture = self
            .fixture
            .as_ref()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        substrate(OperatorVerifier.verify(
            fixture.binding.key().bytes(),
            fixture.freeze()?.message().with_vector_grown(),
            signature,
        ))
    }

    fn submission(&self, position: usize) -> Result<TargetTransaction, VectorError> {
        let candidate = self
            .authorized
            .as_ref()
            .ok_or(VectorError::LiveSubstrateUnavailable)?;
        let mut inputs = candidate.inputs().to_vec();
        let mut outputs = candidate.outputs().to_vec();
        let mut stack = candidate.witnesses()[0].stack().to_vec();
        match position {
            4 => stack[0].clone_from(&self.wrong_key_signature),
            5 => inputs[0] = TargetInput::new(inputs[0].outpoint(), inputs[0].sequence() ^ 1),
            6 => {
                let target = reviewed_target()?;
                let closure = operator_key_encoding_closure(target.definition().authorization());
                let key = substrate(OperatorKey::new(
                    &closure,
                    closure.approved(),
                    substrate(PublicTestSignerHandle::First.x_only_public_key())?.to_vec(),
                ))?;
                let (_, input) = operator_leaf(&key)?;
                stack[1] = input.leaf_script().to_vec();
                stack[2] = input.control_block().to_vec();
            }
            7 => {
                stack[0].pop();
            }
            8 => stack[0].push(1),
            9 => outputs.swap(0, 1),
            _ => {}
        }
        rebuild(candidate, inputs, outputs, vec![InputWitness::new(stack)])
    }
}

fn rebuild(
    candidate: &TargetTransaction,
    inputs: Vec<TargetInput>,
    outputs: Vec<transaction::bytes::TargetOutput>,
    witnesses: Vec<InputWitness>,
) -> Result<TargetTransaction, VectorError> {
    substrate(TargetTransaction::with_output_witnesses(
        candidate.version(),
        inputs,
        outputs,
        candidate.lock_time(),
        witnesses,
        candidate.output_witnesses().to_vec(),
    ))
}

impl TargetOperationPlanner for OperatorPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        match (self.pending.take(), previous) {
            (Some(step), Some((case, response)))
                if step.case() == case && &response.case == case =>
            {
                substrate(response.validate_shape()).map_err(|_| PlanRefused)?;
                self.exchanges.push((step, response.clone()));
                self.settle(response).map_err(|_| PlanRefused)?;
                self.position += 1;
            }
            (None, None) if self.position == 0 => {}
            _ => return Err(PlanRefused),
        }
        let step = self.make_step().map_err(|_| PlanRefused)?;
        self.pending.clone_from(&step);
        Ok(step)
    }
}

/// The published signer absent from the demonstration owner's deployment.
pub const OPERATOR_HANDLE: PublicTestSignerHandle = PublicTestSignerHandle::Third;
/// The explicit amount funded for the selected predecessor.
pub const PREDECESSOR_AMOUNT: u64 = 5_000;

fn substrate<T, E>(value: Result<T, E>) -> Result<T, VectorError> {
    value.map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// Public-data Schnorr verification from the independent conformance oracle.
pub struct OperatorVerifier;

impl ScriptPathSignatureVerifier for OperatorVerifier {
    fn verify(
        &self,
        key: &[u8],
        message: &Digest32,
        signature: &[u8],
    ) -> Result<(), ScriptPathVerifierRejection> {
        let target = reviewed_target()
            .map_err(|error| ScriptPathVerifierRejection::new(format!("{error:?}")))?;
        verify_owner_signature(&target, key, message, signature)
            .map_err(|error| ScriptPathVerifierRejection::new(format!("{error:?}")))
    }

    fn description(&self) -> &'static str {
        "conformance owner_key_oracle public-data Schnorr verification"
    }
}

/// Public candidate material with one selected operator input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorFixture {
    binding: OperatorDeploymentBinding,
    candidate: TargetTransaction,
    spent: Vec<SpentOutputCensusEntry>,
    input: OperatorSigningInput,
}

impl OperatorFixture {
    /// Builds a synthetic spend from the existing finalized live fixture shape.
    ///
    /// Only the predecessor's authorization changes to the committed operator
    /// leaf. The existing ABI still constructs the live successor programs.
    /// No scalar is read by this construction.
    ///
    /// # Errors
    /// Returns a substrate refusal when fixture construction or binding fails.
    pub fn new(
        identity: CandidateDeploymentIdentity,
        predecessor: Outpoint,
        asset: AssetId,
    ) -> Result<Self, VectorError> {
        let target = reviewed_target()?;
        let closure = operator_key_encoding_closure(target.definition().authorization());
        let key = substrate(OperatorKey::new(
            &closure,
            closure.approved(),
            substrate(OPERATOR_HANDLE.x_only_public_key())?.to_vec(),
        ))?;
        let internal = substrate(StackItem::encoded(
            &target,
            EncodingClass::XOnlyPublicKey,
            UNSPENDABLE_INTERNAL_KEY.to_vec(),
        ))?;
        let profile = substrate(EstablishedOperatorProfile::establish(
            selected_operator_profile(),
            &target,
        ))?;
        let binding = substrate(OperatorDeploymentBinding::bind(
            &target, key, profile, identity, &internal,
        ))?;
        let (program, input) = operator_leaf(binding.key())?;
        let candidate = live_candidate(predecessor, asset)?;
        let fixture = Self {
            binding,
            candidate,
            spent: vec![SpentOutputCensusEntry::new(
                AssetField::Explicit(asset),
                ValueField::Explicit(PREDECESSOR_AMOUNT),
                program,
            )],
            input,
        };
        fixture.freeze()?;
        Ok(fixture)
    }

    /// Validates the exact candidate, deployment, census and selected leaf.
    ///
    /// # Errors
    /// Returns a substrate refusal when the operator signing boundary refuses.
    pub fn freeze(&self) -> Result<OperatorSigningRequest<'_>, VectorError> {
        let target = reviewed_target()?;
        substrate(OperatorSigningRequest::freeze(
            &target,
            &self.binding,
            self.candidate.clone(),
            self.spent.clone(),
            LiveDeployment::new(printed_order(*self.binding.deployment().genesis_id())),
            self.input.clone(),
            &OracleLiveCurve::new(target.clone()),
        ))
    }

    /// The exact candidate deployment commitment.
    #[must_use]
    pub const fn binding(&self) -> &OperatorDeploymentBinding {
        &self.binding
    }

    /// The candidate before operator witness insertion.
    #[must_use]
    pub const fn candidate(&self) -> &TargetTransaction {
        &self.candidate
    }
}

fn operator_leaf(key: &OperatorKey) -> Result<(Vec<u8>, OperatorSigningInput), VectorError> {
    let target = reviewed_target()?;
    let literal = substrate(StackItem::encoded(
        &target,
        key.encoding(),
        key.bytes().to_vec(),
    ))?;
    let script = substrate(TapscriptProgram::new(vec![
        TapscriptInstruction::Push(literal),
        TapscriptInstruction::Opcode(OpcodeId::CheckSig),
    ]))?
    .encode(&target);
    let hash = leaf_hash(LeafVersion::TAPSCRIPT, &script);
    let output = OracleLiveCurve::new(target.clone())
        .output_key(&UNSPENDABLE_INTERNAL_KEY, &hash)
        .ok_or(VectorError::LiveSubstrateUnavailable)?;
    let program = substrate(witness_program_script(&target, 1, output.key()))?;
    let mut control = vec![LeafVersion::TAPSCRIPT.get() | output.parity().bit()];
    control.extend_from_slice(&UNSPENDABLE_INTERNAL_KEY);
    Ok((
        program,
        OperatorSigningInput::new(0, hash, LeafVersion::TAPSCRIPT, script, control),
    ))
}

fn live_candidate(predecessor: Outpoint, asset: AssetId) -> Result<TargetTransaction, VectorError> {
    let target = reviewed_target()?;
    let abi = if asset.internal() == &crate::live_plan::PROTOCOL_ASSET {
        demonstration_live_abi()?
    } else {
        crate::live_plan::live_abi_for_asset(
            *asset.internal(),
            crate::live_plan::RESERVE_ASSET,
            crate::live_plan::FEE_PROGRAM_DIGEST,
        )?
    };
    let owner = OwnerParameter::new(crate::live_plan::owner_key(&substrate(
        PublicTestSignerHandle::First.x_only_public_key(),
    )?)?);
    let destination = abi
        .destinations()
        .get(&owner, LiveTransferRepresentationPlan::Explicit)
        .ok_or(VectorError::LiveSubstrateUnavailable)?;
    let view = substrate(PublicConstructionView::new([PublicOutputView::new(
        predecessor,
        AssetField::Explicit(asset),
        ValueField::Explicit(PREDECESSOR_AMOUNT),
        destination.instance().program().to_vec(),
    )]))?;
    let receipts = [3_750, 1_250]
        .into_iter()
        .map(|amount| {
            substrate(ProtocolValue::new(amount))
                .map(|value| LiveReceiptDestination::new(owner.clone(), value))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let request = substrate(LiveTransferRequest::new(
        [predecessor],
        receipts,
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    ))?;
    let finalized = substrate(finalize_live_transfer(
        &target, &abi, &request, &view, None, None,
    ))?
    .into_finalized();
    Ok(finalized.protected().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use transaction::Txid;
    use transaction::operator_signing::{OperatorSigningRefusal, authorize_operator};

    fn fixture() -> OperatorFixture {
        static FIXTURE: OnceLock<OperatorFixture> = OnceLock::new();
        FIXTURE
            .get_or_init(|| {
                OperatorFixture::new(
                    CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("identity"),
                    Outpoint::new(Txid::from_internal([0x33; 32]), 0).expect("outpoint"),
                    AssetId::from_internal(crate::live_plan::PROTOCOL_ASSET),
                )
                .expect("public fixture")
            })
            .clone()
    }

    fn signature(fixture: &OperatorFixture, handle: PublicTestSignerHandle) -> Vec<u8> {
        handle
            .material()
            .expect("published material")
            .sign(
                fixture
                    .freeze()
                    .expect("freeze")
                    .message()
                    .with_vector_grown(),
                &[0; 32],
            )
            .expect("public signature")
            .to_vec()
    }

    fn response(fixture: &OperatorFixture, width: usize, type_byte: u8) -> OperatorSigningResponse {
        let mut signature = signature(fixture, OPERATOR_HANDLE);
        signature.truncate(width);
        OperatorSigningResponse::new(
            0,
            signature,
            type_byte,
            fixture.candidate.encode(),
            fixture.binding.key().clone(),
            fixture.binding.deployment().clone(),
            fixture.binding.capability_revision(),
        )
    }

    #[test]
    fn fixture_freezes_one_input_two_outputs_and_one_spent_output() {
        let fixture = fixture();
        let request = fixture.freeze().expect("freeze");
        assert_eq!(request.candidate().inputs().len(), 1);
        assert_eq!(request.candidate().outputs().len(), 2);
        assert_eq!(request.census().spent_outputs().len(), 1);
        assert_eq!(request.census().signing_inputs().len(), 1);
        assert_eq!(request.census().output_witnesses().len(), 2);
        assert_eq!(request.frozen_bytes(), request.candidate().encode());
    }

    #[test]
    fn third_handle_is_distinct_from_both_demonstration_owners() {
        let fixture = fixture();
        for scalar in [
            &crate::live_plan::FIRST_SCALAR,
            &crate::live_plan::SECOND_SCALAR,
        ] {
            let owner = crate::live_plan::published_owner(scalar).expect("public owner");
            assert_ne!(fixture.binding.key().bytes(), owner.bytes());
        }
        assert_eq!(
            fixture.binding.key().bytes(),
            OPERATOR_HANDLE.x_only_public_key().expect("key")
        );
    }

    #[test]
    fn wire_subject_pins_every_public_field() {
        let fixture = fixture();
        let subject = fixture.signing_subject().expect("subject");
        assert_eq!(subject.signer, PublicTestSignerHandle::Third);
        assert_eq!(subject.input_index, 0);
        assert_eq!(subject.finalized_transaction, fixture.candidate.encode());
        assert_eq!(subject.spent_outputs.len(), 1);
        assert_eq!(subject.spent_outputs[0].asset_field.len(), 33);
        assert_eq!(subject.spent_outputs[0].value_field.len(), 9);
        assert_eq!(subject.spent_outputs[0].program.len(), 34);
        assert_eq!(subject.executing_leaf.script.len(), 34);
        assert_eq!(subject.executing_leaf.control_block.len(), 33);
        assert_eq!(subject.executing_leaf.script, fixture.input.leaf_script());
        assert_eq!(
            subject.executing_leaf.control_block,
            fixture.input.control_block()
        );
        assert_eq!(
            subject.sighash_profile,
            WireSighashProfile::AllInputsAllOutputs
        );
    }

    #[test]
    fn independent_verifier_accepts_the_third_handle_signature() {
        let fixture = fixture();
        let signature = signature(&fixture, OPERATOR_HANDLE);
        assert!(
            OperatorVerifier
                .verify(
                    fixture.binding.key().bytes(),
                    fixture
                        .freeze()
                        .expect("freeze")
                        .message()
                        .with_vector_grown(),
                    &signature
                )
                .is_ok()
        );
    }

    #[test]
    fn independent_verifier_rejects_a_flipped_signature() {
        let fixture = fixture();
        let mut signature = signature(&fixture, OPERATOR_HANDLE);
        signature[0] ^= 1;
        assert!(
            OperatorVerifier
                .verify(
                    fixture.binding.key().bytes(),
                    fixture
                        .freeze()
                        .expect("freeze")
                        .message()
                        .with_vector_grown(),
                    &signature
                )
                .is_err()
        );
    }

    #[test]
    fn independent_verifier_rejects_the_other_published_key() {
        let fixture = fixture();
        let signature = signature(&fixture, PublicTestSignerHandle::First);
        assert!(
            OperatorVerifier
                .verify(
                    fixture.binding.key().bytes(),
                    fixture
                        .freeze()
                        .expect("freeze")
                        .message()
                        .with_vector_grown(),
                    &signature
                )
                .is_err()
        );
    }

    #[test]
    fn independent_verifier_rejects_another_message() {
        let fixture = fixture();
        let signature = signature(&fixture, OPERATOR_HANDLE);
        let mut message = *fixture
            .freeze()
            .expect("freeze")
            .message()
            .with_vector_grown();
        message[0] ^= 1;
        assert!(
            OperatorVerifier
                .verify(fixture.binding.key().bytes(), &message, &signature)
                .is_err()
        );
    }

    #[test]
    fn registry_refuses_a_second_authorization_before_planning() {
        let fixture = fixture();
        let request = fixture.freeze().expect("freeze");
        let branch = BranchContext::new([0x44; 32], 0).expect("branch");
        let scope = RightScope::new(&request, branch).expect("scope");
        let mut registry = OperatorRightRegistry::default();
        let right = registry
            .issue(scope.clone(), request.frozen_bytes())
            .expect("first right");
        assert!(matches!(
            registry.issue(scope, request.frozen_bytes()),
            Err(RightRefusal::Outstanding(_))
        ));
        assert_eq!(registry.record().entries().len(), 1);
        assert_eq!(right.ordinal(), 0);
    }

    #[test]
    fn deployment_refuses_a_key_outside_its_binding() {
        let fixture = fixture();
        let target = reviewed_target().expect("target");
        let closure = operator_key_encoding_closure(target.definition().authorization());
        let other = OperatorKey::new(
            &closure,
            closure.approved(),
            PublicTestSignerHandle::First
                .x_only_public_key()
                .expect("key")
                .to_vec(),
        )
        .expect("key shape");
        assert!(matches!(
            fixture.binding.check(
                &other,
                fixture.binding.deployment(),
                fixture.binding.capability_revision()
            ),
            Err(linker::LinkRefusal::OperatorKeyMismatch { .. })
        ));
    }

    #[test]
    fn changing_the_genesis_changes_the_recomputed_message() {
        let fixture = fixture();
        let other = OperatorFixture::new(
            CandidateDeploymentIdentity::new([0x11; 32], [0x23; 32]).expect("identity"),
            fixture.candidate.inputs()[0].outpoint(),
            AssetId::from_internal(crate::live_plan::PROTOCOL_ASSET),
        )
        .expect("other deployment");
        assert_eq!(fixture.candidate, other.candidate);
        assert_ne!(
            fixture.freeze().expect("freeze").message(),
            other.freeze().expect("freeze").message()
        );
    }

    #[test]
    fn frozen_deployment_reverses_the_printed_identity_before_hashing() {
        let base = fixture();
        let mut printed_genesis = [0x22; 32];
        printed_genesis[0] = 0x01;
        printed_genesis[31] = 0xfe;
        let mut internal_genesis = printed_genesis;
        internal_genesis.reverse();
        let fixtures = [printed_genesis, internal_genesis].map(|genesis| {
            OperatorFixture::new(
                CandidateDeploymentIdentity::new([0x11; 32], genesis).expect("identity"),
                base.candidate.inputs()[0].outpoint(),
                AssetId::from_internal(crate::live_plan::PROTOCOL_ASSET),
            )
            .expect("deployment")
        });
        let [correct, other] = &fixtures;
        let request = correct.freeze().expect("internal-order request");
        let unreversed = other.freeze().expect("opposite deployment request");
        assert_eq!(
            request.binding().deployment().genesis_id(),
            &printed_genesis
        );
        assert_eq!(request.census().genesis_block_hash(), &internal_genesis);
        assert_eq!(unreversed.census().genesis_block_hash(), &printed_genesis);
        assert_eq!(request.candidate(), unreversed.candidate());
        assert_eq!(
            request.census().spent_outputs(),
            unreversed.census().spent_outputs()
        );
        assert_eq!(correct.input, other.input);
        assert_ne!(request.message(), unreversed.message());
    }

    #[test]
    fn wrong_leaf_is_refused_by_freeze() {
        let mut fixture = fixture();
        let mut script = fixture.input.leaf_script().to_vec();
        script[1] ^= 1;
        fixture.input = OperatorSigningInput::new(
            0,
            leaf_hash(LeafVersion::TAPSCRIPT, &script),
            LeafVersion::TAPSCRIPT,
            script,
            fixture.input.control_block().to_vec(),
        );
        assert!(fixture.freeze().is_err());
    }

    #[test]
    fn moving_a_protected_output_changes_the_message() {
        let fixture = fixture();
        let mut other = fixture.clone();
        let mut outputs = other.candidate.outputs().to_vec();
        outputs.swap(0, 1);
        other.candidate = rebuild(
            &other.candidate,
            other.candidate.inputs().to_vec(),
            outputs,
            other.candidate.witnesses().to_vec(),
        )
        .expect("changed candidate");
        assert_ne!(
            fixture.freeze().expect("freeze").message(),
            other.freeze().expect("freeze").message()
        );
    }

    #[test]
    fn handoff_refuses_wrong_signature_width() {
        let fixture = fixture();
        let answer = response(&fixture, 63, 0);
        assert!(matches!(
            authorize_operator(
                fixture.freeze().expect("freeze"),
                [answer],
                &OperatorVerifier
            ),
            Err(OperatorSigningRefusal::MalformedSignature { offered: 63 })
        ));
    }

    #[test]
    fn handoff_refuses_the_nonselected_type_byte() {
        let fixture = fixture();
        let answer = response(&fixture, 64, 1);
        assert!(matches!(
            authorize_operator(
                fixture.freeze().expect("freeze"),
                [answer],
                &OperatorVerifier
            ),
            Err(OperatorSigningRefusal::WrongTypeByte { offered: 1 })
        ));
    }

    fn accepted_readback() -> (OperatorPlanner, NativeOperationResponse) {
        use target_elements_conformance::protocol::{
            MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA, NativeResourceObservation,
            OperationStepKind,
        };

        let fixture = fixture();
        let answer = response(&fixture, 64, 0);
        let authorized = authorize_operator(
            fixture.freeze().expect("freeze"),
            [answer],
            &OperatorVerifier,
        )
        .expect("independent authorization");
        let candidate = rebuild(
            &fixture.candidate,
            fixture.candidate.inputs().to_vec(),
            fixture.candidate.outputs().to_vec(),
            vec![authorized.witness().clone()],
        )
        .expect("witness insertion");
        let identity = "33".repeat(32);
        let readback = MinedFundingReadback {
            transaction_id: identity.clone(),
            witness_transaction_id: identity.clone(),
            block_hash: "44".repeat(32),
            block_height: 1,
            raw_transaction: candidate.encode(),
        };
        let response = NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: OperationCaseId {
                operation: OperationStepKind::Submit,
                step: "operator-positive".to_owned(),
            },
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: None,
            funded_outputs: Vec::new(),
            confidential_funded_outputs: Vec::new(),
            mined_readback: Some(readback),
            accepted_txid: Some(identity),
            sponsor_witness: Vec::new(),
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        };
        let mut planner = OperatorPlanner::new(fixture.binding.deployment().clone());
        planner.authorized = Some(candidate);
        planner.fixture = Some(fixture);
        (planner, response)
    }

    #[test]
    fn the_readback_signature_verifies_against_the_frozen_message() {
        let (planner, response) = accepted_readback();
        response.validate_shape().expect("accepted response shape");
        assert!(planner.settle_positive(&response).is_ok());
    }

    #[test]
    fn a_different_readback_cannot_complete_the_positive() {
        let (planner, mut response) = accepted_readback();
        let readback = response.mined_readback.as_mut().expect("readback");
        readback.raw_transaction[0] ^= 1;
        assert!(planner.settle_positive(&response).is_err());
    }

    #[test]
    fn a_funded_asset_relinks_the_successor_programs_before_finalization() {
        let fixture = fixture();
        let asset = AssetId::from_internal([0x55; 32]);
        let other = OperatorFixture::new(
            fixture.binding.deployment().clone(),
            fixture.candidate.inputs()[0].outpoint(),
            asset,
        )
        .expect("asset-specific fixture");
        assert!(
            other
                .candidate
                .outputs()
                .iter()
                .all(|output| output.asset() == AssetField::Explicit(asset))
        );
        assert_eq!(other.spent[0].asset(), AssetField::Explicit(asset));
        assert_ne!(
            other.candidate.outputs()[0].program(),
            fixture.candidate.outputs()[0].program()
        );
        assert_eq!(other.input, fixture.input);
        assert_ne!(
            other.freeze().expect("freeze").message(),
            fixture.freeze().expect("freeze").message()
        );
    }
}
