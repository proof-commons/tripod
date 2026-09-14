//! Operator authorization over one immutable script-path candidate.
//!
//! The deployment binding owns the public identity and established profile.
//! This boundary validates the leaf selection and candidate census, then checks
//! a separately returned response before assembling any witness.

use linker::live_backend::{OperatorKey, OperatorProfileDisposition};
use linker::{CandidateDeploymentIdentity, LinkRefusal, OperatorDeploymentBinding};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::bytes::{InputWitness, TargetTransaction};
use crate::live_taproot::LiveCurveCapability;
use crate::script_path_signing::{
    self, AnnexDisposition, CandidateMessagePair, IssuanceDisposition, LiveDeployment,
    ScriptPathCensusParts, ScriptPathCensusRefusal, ScriptPathInputRequest, ScriptPathProfile,
    ScriptPathSigningCensus, ScriptPathSigningInputCensus, SpentOutputCensusEntry,
};
use crate::taproot::{Digest32, leaf_hash};

/// The operator's default hash-type byte.
pub const OPERATOR_SIGHASH_TYPE_BYTE: u8 = 0x00;
/// The operator's script-path spend type without an annex.
pub const OPERATOR_SPEND_TYPE_BYTE: u8 = 0x02;
/// The operator's signature width under the default hash type.
pub const OPERATOR_SIGNATURE_BYTES: usize = 64;
/// The operator's tapscript key version.
pub const OPERATOR_KEY_VERSION_BYTE: u8 = 0x00;
/// The operator's position when no code separator executes.
pub const OPERATOR_CODESEPARATOR_POSITION: u32 = 0xffff_ffff;

/// The announcement selection has the same five values as the owner selection.
///
/// They remain role-specific constants: equality does not make an owner profile
/// evidence for operator authority.
const OPERATOR_PROFILE: ScriptPathProfile = ScriptPathProfile {
    type_byte: OPERATOR_SIGHASH_TYPE_BYTE,
    spend_type_byte: OPERATOR_SPEND_TYPE_BYTE,
    signature_bytes: OPERATOR_SIGNATURE_BYTES,
    key_version_byte: OPERATOR_KEY_VERSION_BYTE,
    codeseparator_position: OPERATOR_CODESEPARATOR_POSITION,
};

/// The leaf selection offered for one candidate input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorSigningInput {
    input_index: u32,
    tapleaf_hash: Digest32,
    leaf_version: LeafVersion,
    leaf_script: Vec<u8>,
    control_block: Vec<u8>,
}

impl OperatorSigningInput {
    /// States the selection whose hash and commitment freeze will check.
    #[must_use]
    pub const fn new(
        input_index: u32,
        tapleaf_hash: Digest32,
        leaf_version: LeafVersion,
        leaf_script: Vec<u8>,
        control_block: Vec<u8>,
    ) -> Self {
        Self {
            input_index,
            tapleaf_hash,
            leaf_version,
            leaf_script,
            control_block,
        }
    }

    /// The selected candidate input.
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.input_index
    }

    /// The claimed tapleaf hash.
    #[must_use]
    pub const fn tapleaf_hash(&self) -> &Digest32 {
        &self.tapleaf_hash
    }

    /// The selected leaf version.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The script whose hash must agree with the selection.
    #[must_use]
    pub fn leaf_script(&self) -> &[u8] {
        &self.leaf_script
    }

    /// The control block that must authenticate the selection.
    #[must_use]
    pub fn control_block(&self) -> &[u8] {
        &self.control_block
    }

    fn kernel(&self) -> ScriptPathInputRequest<'_> {
        ScriptPathInputRequest {
            input_index: self.input_index,
            tapleaf_hash: &self.tapleaf_hash,
            leaf_version: self.leaf_version,
            codeseparator_position: OPERATOR_CODESEPARATOR_POSITION,
            annex: AnnexDisposition::Absent,
            issuance: IssuanceDisposition::Absent,
            control_block: &self.control_block,
        }
    }
}

/// One frozen candidate and input, borrowing their deployment commitment.
#[derive(Debug, PartialEq, Eq)]
pub struct OperatorSigningRequest<'binding> {
    binding: &'binding OperatorDeploymentBinding,
    census: ScriptPathSigningCensus,
    input: OperatorSigningInput,
    signing_input: ScriptPathSigningInputCensus,
    curve_validity_discharged: bool,
}

impl<'binding> OperatorSigningRequest<'binding> {
    /// Freezes one candidate after checking its deployment and leaf selection.
    ///
    /// Revision, genesis, and curve membership are checked in that order,
    /// before any census clause. The historical capability method
    /// `owner_key_is_a_curve_point` checks the operator key here; its name
    /// remains unchanged for existing implementors. The binding's key retains
    /// its original validity marker; this request records the discharged check.
    /// Output witnesses and the exact wire serialization come from the candidate.
    ///
    /// # Errors
    /// Returns the first deployment, curve, census, or leaf-script refusal.
    pub fn freeze(
        target: &ReviewedElementsTapscriptDefinition,
        binding: &'binding OperatorDeploymentBinding,
        candidate: TargetTransaction,
        spent_outputs: Vec<SpentOutputCensusEntry>,
        deployment: LiveDeployment,
        input: OperatorSigningInput,
        curve: &dyn LiveCurveCapability,
    ) -> Result<Self, OperatorSigningRefusal> {
        check_freeze_binding(target, binding, deployment, curve)?;
        let frozen_bytes = candidate.encode();
        let census = ScriptPathSigningCensus::assemble(
            target,
            ScriptPathCensusParts {
                output_witnesses: candidate.output_witnesses().to_vec(),
                candidate,
                protected_bytes: frozen_bytes.clone(),
                spent_outputs,
                deployment,
                profile: OPERATOR_PROFILE,
            },
            &frozen_bytes,
            &[input.kernel()],
            curve,
            |_| check_leaf_script(&input),
        )?;
        let signing_input = census
            .signing_inputs()
            .first()
            .cloned()
            .ok_or(ScriptPathCensusRefusal::NoSigningInputRequested)?;
        Ok(Self {
            binding,
            census,
            input,
            signing_input,
            curve_validity_discharged: true,
        })
    }

    /// The immutable candidate.
    #[must_use]
    pub const fn candidate(&self) -> &TargetTransaction {
        self.census.candidate()
    }

    /// The validated census of this candidate and its one signing input.
    #[must_use]
    pub const fn census(&self) -> &ScriptPathSigningCensus {
        &self.census
    }

    /// Recomputes the recorded diagnostic pair for the frozen input.
    #[must_use]
    pub fn message(&self) -> CandidateMessagePair {
        script_path_signing::message_pair(&self.census, &self.signing_input)
    }

    /// The exact candidate serialization a signer must echo.
    #[must_use]
    pub fn frozen_bytes(&self) -> &[u8] {
        self.census.protected_bytes()
    }

    /// The single input this request authorizes.
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.signing_input.input_index()
    }

    /// The predecessor version spent by the selected frozen input.
    #[must_use]
    pub fn predecessor_outpoint(&self) -> Option<crate::bytes::Outpoint> {
        self.candidate()
            .inputs()
            .get(usize::try_from(self.input_index()).ok()?)
            .map(crate::bytes::TargetInput::outpoint)
    }

    /// The deployment commitment, without copying its key into the request.
    #[must_use]
    pub const fn binding(&self) -> &OperatorDeploymentBinding {
        self.binding
    }

    /// Whether the supplied curve capability accepted the committed key.
    #[must_use]
    pub const fn curve_validity_discharged(&self) -> bool {
        self.curve_validity_discharged
    }
}

fn check_freeze_binding(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &OperatorDeploymentBinding,
    deployment: LiveDeployment,
    curve: &dyn LiveCurveCapability,
) -> Result<(), OperatorSigningRefusal> {
    let current = target.definition().version();
    if binding.capability_revision() != current {
        return Err(OperatorSigningRefusal::BindingRevisionMismatch {
            bound: binding.capability_revision(),
            current,
        });
    }
    script_path_signing::check_deployment(*binding.deployment().genesis_id(), deployment).map_err(
        |_| OperatorSigningRefusal::GenesisMismatch {
            bound: *binding.deployment().genesis_id(),
            offered: *deployment.genesis_block_hash(),
        },
    )?;
    if !curve.owner_key_is_a_curve_point(binding.key().bytes()) {
        return Err(OperatorSigningRefusal::OperatorKeyIsNotACurvePoint {
            key: binding.key().bytes().to_vec(),
        });
    }
    Ok(())
}

fn check_leaf_script(input: &OperatorSigningInput) -> Result<(), OperatorSigningRefusal> {
    let computed = leaf_hash(input.leaf_version, &input.leaf_script);
    if computed != input.tapleaf_hash {
        return Err(OperatorSigningRefusal::WrongLeaf {
            input_index: input.input_index,
            expected: input.tapleaf_hash,
            computed,
        });
    }
    Ok(())
}

/// An independently supplied signer response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorSigningResponse {
    input_index: u32,
    signature: Vec<u8>,
    type_byte: u8,
    echo: Vec<u8>,
    operator: OperatorKey,
    deployment: CandidateDeploymentIdentity,
    capability_revision: TargetContractVersion,
}

impl OperatorSigningResponse {
    /// Records the signer's answer without deriving fields from a request.
    #[must_use]
    pub const fn new(
        input_index: u32,
        signature: Vec<u8>,
        type_byte: u8,
        echo: Vec<u8>,
        operator: OperatorKey,
        deployment: CandidateDeploymentIdentity,
        capability_revision: TargetContractVersion,
    ) -> Self {
        Self {
            input_index,
            signature,
            type_byte,
            echo,
            operator,
            deployment,
            capability_revision,
        }
    }

    /// The input the signer answered.
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.input_index
    }
    /// The signature bytes returned.
    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
    /// The returned hash-type byte.
    #[must_use]
    pub const fn type_byte(&self) -> u8 {
        self.type_byte
    }
    /// The serialization the signer says it authorized.
    #[must_use]
    pub fn echo(&self) -> &[u8] {
        &self.echo
    }
    /// The offered public operator identity.
    #[must_use]
    pub const fn operator(&self) -> &OperatorKey {
        &self.operator
    }
    /// The offered deployment identity.
    #[must_use]
    pub const fn deployment(&self) -> &CandidateDeploymentIdentity {
        &self.deployment
    }
    /// The offered capability revision.
    #[must_use]
    pub const fn capability_revision(&self) -> TargetContractVersion {
        self.capability_revision
    }
}

/// An independent verifier's explanation for refusing a signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptPathVerifierRejection {
    reason: String,
}

impl ScriptPathVerifierRejection {
    /// Records the verifier's reason.
    #[must_use]
    pub const fn new(reason: String) -> Self {
        Self { reason }
    }
    /// The verifier's explanation.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// The seam for an independently implemented public-data signature verifier.
///
/// Production implementations live outside this crate. The conformance
/// implementation can be wired by the vectors crate without a reverse dependency.
/// Acceptance is in-process evidence only and is explicitly not native evidence.
pub trait ScriptPathSignatureVerifier {
    /// Checks the signature under the committed key and recomputed message.
    ///
    /// # Errors
    /// Returns the independent implementation's rejection when verification fails.
    fn verify(
        &self,
        key: &[u8],
        message: &Digest32,
        signature: &[u8],
    ) -> Result<(), ScriptPathVerifierRejection>;

    /// Identifies the implementation supplying the in-process evidence.
    fn description(&self) -> &str;
}

/// The standing available from this boundary's verifier.
///
/// Native acceptance belongs to later execution evidence, with no variant here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatorEvidenceStanding {
    /// The supplied implementation accepted the signature in this process.
    InProcessVerified {
        /// The verifier's description.
        verifier: String,
    },
}

/// One frozen candidate with a verified operator witness.
#[derive(Debug, PartialEq, Eq)]
pub struct OperatorAuthorizedCandidate<'binding> {
    request: OperatorSigningRequest<'binding>,
    witness: InputWitness,
    standing: OperatorEvidenceStanding,
}

impl<'binding> OperatorAuthorizedCandidate<'binding> {
    /// The frozen request, still borrowing its deployment commitment.
    #[must_use]
    pub const fn request(&self) -> &OperatorSigningRequest<'binding> {
        &self.request
    }
    /// The signature, leaf script, and control block in ABI order.
    #[must_use]
    pub const fn witness(&self) -> &InputWitness {
        &self.witness
    }
    /// The in-process evidence and its named verifier.
    #[must_use]
    pub const fn standing(&self) -> &OperatorEvidenceStanding {
        &self.standing
    }
}

/// Why operator signing produced no accepted artifact.
///
/// Metadata, successor program, sponsor input, and fee-role changes are decided
/// by the exact echo and recomputed message bindings. They need no separate
/// mutation variants. Every refusal carries diagnostics and no witness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatorSigningRefusal {
    /// The binding pins another reviewed target revision.
    BindingRevisionMismatch {
        /// The binding's revision.
        bound: TargetContractVersion,
        /// The offered target's revision.
        current: TargetContractVersion,
    },
    /// The message genesis differs from the deployment commitment.
    GenesisMismatch {
        /// The committed genesis.
        bound: Digest32,
        /// The message genesis.
        offered: Digest32,
    },
    /// The supplied curve capability refused the committed key.
    OperatorKeyIsNotACurvePoint {
        /// The refused public bytes.
        key: Vec<u8>,
    },
    /// A structural or leaf commitment clause failed.
    Census(ScriptPathCensusRefusal),
    /// The leaf script does not hash to the selected leaf.
    WrongLeaf {
        /// The selected input.
        input_index: u32,
        /// The selected hash.
        expected: Digest32,
        /// The script's computed hash.
        computed: Digest32,
    },
    /// No response arrived.
    MissingResponse {
        /// The unanswered input.
        input_index: u32,
    },
    /// A second response names the frozen input.
    DuplicateResponse {
        /// The repeated input.
        input_index: u32,
    },
    /// A second response names another input.
    UnexpectedResponse {
        /// The surplus response's input.
        input_index: u32,
    },
    /// The sole response names another input.
    WrongInput {
        /// The frozen input.
        expected: u32,
        /// The response's input.
        offered: u32,
    },
    /// The offered operator differs from the binding.
    WrongOperator {
        /// The committed public key.
        bound: OperatorKey,
        /// The offered public key.
        offered: OperatorKey,
    },
    /// The offered deployment differs from the binding.
    WrongDeployment {
        /// The committed deployment.
        bound: Box<CandidateDeploymentIdentity>,
        /// The offered deployment.
        offered: Box<CandidateDeploymentIdentity>,
    },
    /// The response offers a profile revision the binding refuses.
    WrongProfile(OperatorProfileDisposition),
    /// A future binding check refused for another typed reason.
    BindingRefused(Box<LinkRefusal>),
    /// The response uses a non-default type.
    WrongTypeByte {
        /// The returned byte.
        offered: u8,
    },
    /// The response carries no signature bytes.
    EmptySignature {
        /// The answered input.
        input_index: u32,
    },
    /// The signature is not the selected width.
    MalformedSignature {
        /// The returned width.
        offered: usize,
    },
    /// The response echoes another candidate serialization.
    BoundToOtherBytes {
        /// The answered input.
        input_index: u32,
    },
    /// Verification under the frozen candidate and leaf message failed.
    SignatureDoesNotVerifyForFrozenMessage {
        /// The answered input.
        input_index: u32,
        /// The message actually checked.
        message: Digest32,
        /// The independent verifier's explanation.
        rejection: ScriptPathVerifierRejection,
    },
}

impl From<ScriptPathCensusRefusal> for OperatorSigningRefusal {
    fn from(refusal: ScriptPathCensusRefusal) -> Self {
        Self::Census(refusal)
    }
}

impl From<LinkRefusal> for OperatorSigningRefusal {
    fn from(refusal: LinkRefusal) -> Self {
        match refusal {
            LinkRefusal::OperatorKeyMismatch { bound, offered } => {
                Self::WrongOperator { bound, offered }
            }
            LinkRefusal::OperatorDeploymentMismatch { bound, offered } => {
                Self::WrongDeployment { bound, offered }
            }
            LinkRefusal::InvalidOperatorProfile(reason) => Self::WrongProfile(reason),
            other => Self::BindingRefused(Box::new(other)),
        }
    }
}

/// Accepts exactly one answer over the frozen candidate and selected leaf.
///
/// The production route is [`authorize_operator_under_right`].
///
/// Count is checked first. A second answer for the frozen index is duplicate;
/// any other second answer is unexpected. Only a sole answer can be wrong-input.
/// Binding checks key, deployment, then revision; shape, echo, and verification
/// follow. A signature over another transaction or leaf fails verification because
/// either changes the recomputed message. No witness exists before all checks pass.
///
/// # Errors
/// Returns the first named response refusal, with no accepted artifact.
pub fn authorize_operator<'binding>(
    request: OperatorSigningRequest<'binding>,
    responses: impl IntoIterator<Item = OperatorSigningResponse>,
    verifier: &dyn ScriptPathSignatureVerifier,
) -> Result<OperatorAuthorizedCandidate<'binding>, OperatorSigningRefusal> {
    let response = single_response(request.input_index(), responses)?;
    request.binding.check(
        &response.operator,
        &response.deployment,
        response.capability_revision,
    )?;
    check_response_shape(&response)?;
    script_path_signing::check_protected_bytes(request.frozen_bytes(), &response.echo).map_err(
        |_| OperatorSigningRefusal::BoundToOtherBytes {
            input_index: response.input_index,
        },
    )?;
    let message = *request.message().with_vector_grown();
    verifier
        .verify(request.binding.key().bytes(), &message, &response.signature)
        .map_err(
            |rejection| OperatorSigningRefusal::SignatureDoesNotVerifyForFrozenMessage {
                input_index: response.input_index,
                message,
                rejection,
            },
        )?;
    let witness = script_path_signing::script_path_witness(
        response.signature,
        request.input.leaf_script(),
        request.input.control_block(),
    );
    Ok(OperatorAuthorizedCandidate {
        request,
        witness,
        standing: OperatorEvidenceStanding::InProcessVerified {
            verifier: verifier.description().to_owned(),
        },
    })
}

fn single_response(
    input_index: u32,
    responses: impl IntoIterator<Item = OperatorSigningResponse>,
) -> Result<OperatorSigningResponse, OperatorSigningRefusal> {
    let mut responses = responses.into_iter();
    let response = responses
        .next()
        .ok_or(OperatorSigningRefusal::MissingResponse { input_index })?;
    if let Some(extra) = responses.next() {
        return Err(if extra.input_index == input_index {
            OperatorSigningRefusal::DuplicateResponse { input_index }
        } else {
            OperatorSigningRefusal::UnexpectedResponse {
                input_index: extra.input_index,
            }
        });
    }
    if response.input_index != input_index {
        return Err(OperatorSigningRefusal::WrongInput {
            expected: input_index,
            offered: response.input_index,
        });
    }
    Ok(response)
}

fn check_response_shape(response: &OperatorSigningResponse) -> Result<(), OperatorSigningRefusal> {
    script_path_signing::check_type_byte(OPERATOR_PROFILE, response.type_byte).map_err(|_| {
        OperatorSigningRefusal::WrongTypeByte {
            offered: response.type_byte,
        }
    })?;
    if response.signature.is_empty() {
        return Err(OperatorSigningRefusal::EmptySignature {
            input_index: response.input_index,
        });
    }
    script_path_signing::check_signature_width(OPERATOR_PROFILE, response.signature.len()).map_err(
        |_| OperatorSigningRefusal::MalformedSignature {
            offered: response.signature.len(),
        },
    )
}

/// Authorizes the frozen request under its registry-issued affine right.
///
/// # Errors
/// Returns the original token with the registry or signing refusal.
pub fn authorize_operator_under_right<'binding>(
    registry: &mut crate::operator_right::OperatorRightRegistry,
    right: crate::operator_right::ConstructionRight,
    request: OperatorSigningRequest<'binding>,
    responses: impl IntoIterator<Item = OperatorSigningResponse>,
    verifier: &dyn ScriptPathSignatureVerifier,
) -> Result<
    crate::operator_right::OperatorRightOutcome<'binding>,
    Box<crate::operator_right::RightFailure>,
> {
    registry.consume(right, request, |request| {
        authorize_operator(request, responses, verifier)
    })
}
