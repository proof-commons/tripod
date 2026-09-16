//! Walked operator authorization with an unresolved committed-key consumer.
//!
//! This record follows the structural STATE record's contract and metadata shape.
//! Its single signature schedule is separate from the structural empty schedule.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::RequiredSourceKind;
use target_elements::{
    ElementsCapability, EncodingClass, OpcodeId, ResourceDimension,
    ReviewedElementsTapscriptDefinition, StackValueType, TargetEvidenceRequirementId,
};
use thiserror::Error;

use crate::capability::census_enum;
use crate::operator_authorization::{OperatorSighashProfile, selected_operator_profile};
use crate::pattern::{fragment_prerequisites, op};
use crate::state_pattern::StatePatternConstructibility;
use crate::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, SignatureSuccessForm, StackItem,
    TapscriptError, TapscriptInstruction, TapscriptProgram, resource_projection, validate_program,
};

census_enum! {
    /// The identity minted by a successful operator walk.
    pub enum StateOperatorPatternId {
        /// One committed key verifies one signature without a Boolean result.
        OperatorAuthorizationV1,
    }
}
census_enum! {
    /// The semantic responsibility of this fragment.
    pub enum StateOperatorOwner {
        /// Authorize the frozen announcement message with the committed operator.
        OperatorAuthorization,
    }
}
census_enum! {
    /// Link-time inputs, requiring a future linker reference type.
    ///
    /// The constructor's seven policy references do not contain an operator key.
    pub enum StateOperatorSymbol {
        /// The deployment's committed recognized operator public key.
        CommittedOperatorKey,
    }
}
census_enum! {
    /// What execution discloses.
    pub enum StateOperatorDisclosure {
        /// The program contains its committed public key.
        OperatorPublicKey,
        /// The witness supplies one signature under the selected profile.
        OperatorSignature,
    }
}
census_enum! {
    /// Evidence that this pre-link walk does not establish.
    pub enum StateOperatorResidual {
        /// The consumer still needs authenticated deployment resolution.
        UnresolvedConsumer,
        /// Abstract execution does not establish native acceptance.
        TargetNativeEvidence,
        /// Key encoding does not establish curve membership.
        OperatorCurveValidity,
        /// The transaction boundary must map verified authority to the model.
        DeploymentMembership,
        /// The observation boundary must associate evidence with finalized bytes.
        FinalizedObservationBinding,
    }
}

/// The fragment's entire witness role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateOperatorWitness {
    /// Exactly one recognized, nonempty signature item, with no witness key.
    OneSignature,
}

/// A refused consumer, program or walked contract.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum StateOperatorRefusal {
    /// Every consumer must occur exactly once in the binding census.
    #[error("missing or unused operator consumer")]
    ConsumerCensus,
    /// Only the reviewed public-key encoding may be substituted.
    #[error("operator consumer is not a recognized public key")]
    KeyEncoding,
    /// Mutation cannot inherit this pattern's identity.
    #[error("operator fragment differs from the verifying recipe")]
    FragmentMismatch,
    /// A success must consume the entire witness and verify a recognized key.
    #[error("operator walk violates its signature contract")]
    InvalidContract,
    /// The typed program or abstract schedule is invalid.
    #[error(transparent)]
    Program(#[from] TapscriptError),
}

/// Checked fixture substitutions, never authenticated deployment resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateOperatorBindings {
    key: StackItem,
}

impl StateOperatorBindings {
    /// Check the exact consumer census and recognized key encoding.
    ///
    /// # Errors
    /// Refuses a missing consumer, an unused consumer or a non-key encoding.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        values: &BTreeMap<StateOperatorSymbol, StackItem>,
    ) -> Result<Self, StateOperatorRefusal> {
        if values.keys().copied().collect::<BTreeSet<_>>()
            != BTreeSet::from([StateOperatorSymbol::CommittedOperatorKey])
        {
            return Err(StateOperatorRefusal::ConsumerCensus);
        }
        let key = &values[&StateOperatorSymbol::CommittedOperatorKey];
        let encoding =
            crate::operator_key_encoding_closure(target.definition().authorization()).approved();
        if StackItem::encoded(target, encoding, key.bytes().to_vec()).is_err() {
            return Err(StateOperatorRefusal::KeyEncoding);
        }
        Ok(Self { key: key.clone() })
    }
}

/// The operator component's admitted contract and derived metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateOperatorPattern {
    id: StateOperatorPatternId,
    owner: StateOperatorOwner,
    witness: StateOperatorWitness,
    constructibility: StatePatternConstructibility,
    disclosure: &'static [StateOperatorDisclosure],
    residuals: &'static [StateOperatorResidual],
    fragment: TapscriptProgram,
    precondition: AbstractStackState,
    execution: AbstractExecutionResult,
    profile: OperatorSighashProfile,
    prerequisites: BTreeSet<ElementsCapability>,
    resources: BTreeMap<ResourceDimension, u64>,
    sources: BTreeSet<RequiredSourceKind>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
    consumers: BTreeMap<StateOperatorSymbol, BTreeSet<usize>>,
}

/// Emit a committed-key push followed by the single verifying primitive.
///
/// `CheckSigVerify` avoids a separately scheduled Boolean and `Verify` pair.
///
/// # Errors
/// Returns a typed program construction refusal.
pub fn state_operator_fragment(
    bindings: &StateOperatorBindings,
) -> Result<TapscriptProgram, StateOperatorRefusal> {
    Ok(TapscriptProgram::new(vec![
        TapscriptInstruction::Push(bindings.key.clone()),
        op(OpcodeId::CheckSigVerify),
    ])?)
}

/// Walk the exact one-signature recipe before minting its identity.
///
/// # Errors
/// Refuses changed instructions or a walk with residue or an unverified success.
pub fn build_state_operator_pattern(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StateOperatorBindings,
    fragment: TapscriptProgram,
) -> Result<StateOperatorPattern, StateOperatorRefusal> {
    if fragment != state_operator_fragment(bindings)? {
        return Err(StateOperatorRefusal::FragmentMismatch);
    }
    let precondition = AbstractStackState::from_main(vec![StackValueType::Encoded(
        EncodingClass::SchnorrSignature,
    )]);
    let execution = validate_program(
        target,
        &fragment,
        &precondition,
        AbstractLimits::for_target(target),
    )?;
    if execution.success() != &BTreeSet::from([AbstractStackState::from_main(Vec::new())])
        || !execution.nonaborting_failure().is_empty()
        || execution.signature_forms()
            != &BTreeMap::from([(
                1,
                BTreeSet::from([SignatureSuccessForm::RecognizedKeyVerified]),
            )])
    {
        return Err(StateOperatorRefusal::InvalidContract);
    }
    let mut evidence = BTreeSet::from([
        TargetEvidenceRequirementId::TapscriptExecutionDomain,
        TargetEvidenceRequirementId::LeafVersionActivation,
        TargetEvidenceRequirementId::PushEncodingSemantics,
        TargetEvidenceRequirementId::EncodingSemantics,
        TargetEvidenceRequirementId::ConsensusResourceLimits,
    ]);
    evidence.extend(target.definition().opcodes()[&OpcodeId::CheckSigVerify].evidence());
    Ok(StateOperatorPattern {
        id: StateOperatorPatternId::OperatorAuthorizationV1,
        owner: StateOperatorOwner::OperatorAuthorization,
        witness: StateOperatorWitness::OneSignature,
        constructibility: StatePatternConstructibility::CheckedUnresolvedConsumers,
        disclosure: StateOperatorDisclosure::ALL,
        residuals: StateOperatorResidual::ALL,
        precondition,
        execution,
        profile: selected_operator_profile(),
        prerequisites: fragment_prerequisites(&fragment),
        resources: resource_projection(target, &fragment),
        sources: BTreeSet::from([
            RequiredSourceKind::OperatorWitness,
            RequiredSourceKind::PublicConstructionData,
        ]),
        evidence,
        consumers: BTreeMap::from([(
            StateOperatorSymbol::CommittedOperatorKey,
            BTreeSet::from([0]),
        )]),
        fragment,
    })
}

impl StateOperatorPattern {
    /// The identity admitted by the walk.
    #[must_use]
    pub const fn id(&self) -> StateOperatorPatternId {
        self.id
    }
    /// The component's semantic responsibility.
    #[must_use]
    pub const fn owner(&self) -> StateOperatorOwner {
        self.owner
    }
    /// The entire witness schedule.
    #[must_use]
    pub const fn witness(&self) -> StateOperatorWitness {
        self.witness
    }
    /// Construction does not resolve the deployment consumer.
    #[must_use]
    pub const fn constructibility(&self) -> StatePatternConstructibility {
        self.constructibility
    }
    /// Exact admitted instructions.
    #[must_use]
    pub const fn fragment(&self) -> &TapscriptProgram {
        &self.fragment
    }
    /// One signature item on the main stack and an empty alternate stack.
    #[must_use]
    pub const fn precondition(&self) -> &AbstractStackState {
        &self.precondition
    }
    /// Walked success, failure, abort and signature-form census.
    #[must_use]
    pub const fn execution(&self) -> &AbstractExecutionResult {
        &self.execution
    }
    /// The source-selected operator coverage profile.
    #[must_use]
    pub const fn profile(&self) -> &OperatorSighashProfile {
        &self.profile
    }
    /// Capabilities derived from the emitted instructions.
    #[must_use]
    pub const fn prerequisites(&self) -> &BTreeSet<ElementsCapability> {
        &self.prerequisites
    }
    /// Resources projected from the emitted instructions.
    #[must_use]
    pub const fn resources(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.resources
    }
    /// Sources required by the signature and committed key.
    #[must_use]
    pub const fn sources(&self) -> &BTreeSet<RequiredSourceKind> {
        &self.sources
    }
    /// Evidence required by the primitive and execution domain.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }
    /// Unresolved consumer symbols and exact instruction indices.
    #[must_use]
    pub const fn consumers(&self) -> &BTreeMap<StateOperatorSymbol, BTreeSet<usize>> {
        &self.consumers
    }
    /// Public material carried by the operator fragment.
    #[must_use]
    pub const fn disclosure(&self) -> &'static [StateOperatorDisclosure] {
        self.disclosure
    }
    /// Outstanding obligations, never closure claims.
    #[must_use]
    pub const fn residuals(&self) -> &'static [StateOperatorResidual] {
        self.residuals
    }
}
