//! Walked STATE structural fragments and their unresolved consumers.
//!
//! Two fragments carry the whole structural side: the executing input is
//! pinned to position zero, and the object consumed there is recognized by its
//! asset, its explicit amount and its script version. Recognizing that asset
//! and amount does not establish current-root freshness or semantic metadata,
//! and those obligations remain separate residuals. The metadata commitment
//! leaf can be adapted by a later composing recipe; it is not an executing
//! authentication fragment and is not embedded here.
//!
//! Nothing here constrains a position other than input zero. A transaction may
//! carry any number of further inputs and outputs of any other asset, and this
//! leaf neither observes nor refuses them. What keeps the singleton exclusive
//! to input zero and output zero is the position pin together with two facts
//! about the deployment and one about the substrate, named as external evidence
//! roles rather than checked here.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::RequiredSourceKind;
use target_elements::{
    ElementsCapability, EncodingClass, FailureCause, OpcodeId, ResourceDimension,
    ReviewedElementsTapscriptDefinition, TargetEvidenceRequirementId,
};
use thiserror::Error;

use crate::capability::census_enum;
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::pattern::{fragment_prerequisites, number, op, require_explicit};
use crate::program::TapscriptProgram;
use crate::stack::{
    AbstractLimits, AbstractStackState, SignatureSuccessForm, resource_projection, validate_program,
};

census_enum! {
    /// Identities admitted only through a successful structural walk.
    pub enum StatePatternId {
        /// The executing input is the singleton's own position zero.
        StateCoordinatorRoleV1,
        /// The predecessor matches the exact asset, amount and script version.
        StateInputRecognitionV1,
    }
}

census_enum! {
    /// Semantic responsibilities of the structural fragments.
    pub enum StatePatternOwner {
        /// Coordinator participation at input zero.
        CoordinatorRole,
        /// Exact predecessor asset, amount and script version.
        PredecessorRecognition,
    }
}

census_enum! {
    /// Typed consumers awaiting linker definitions and resolution.
    ///
    /// Neither is one of the constructor's seven reference declarations: they
    /// name an asset and a value, not constructor policy. Each therefore
    /// requires a corresponding future linker reference type.
    pub enum StatePatternSymbol {
        /// Exact PID singleton asset payload.
        StateAsset,
        /// Exact explicit singleton amount payload.
        StateAmount,
    }
}

census_enum! {
    /// What a structural fragment publishes.
    pub enum StateDisclosure {
        /// Asset identities checked at positions.
        AssetIdentities,
        /// Exact predecessor asset and amount.
        PredecessorCommitment,
    }
}

census_enum! {
    /// Obligations no structural walk discharges.
    pub enum StatePatternResidual {
        /// Checked fixture substitutions are not authenticated link resolution.
        UnresolvedConsumers,
        /// Reviewed abstract execution is not target-native evidence.
        TargetNativeEvidence,
        /// Report-layer chain context must establish the current STATE root.
        CurrentStateRootFreshness,
        /// An asset and amount check does not authenticate semantic metadata.
        SemanticMetadataAuthentication,
        /// The substrate, not this walk, conserves the singleton across the transaction.
        SubstrateConservation,
    }
}

census_enum! {
    /// Evidence roles outside the structural leaf's authority.
    pub enum StateExternalEvidenceRole {
        /// Report-layer chain context establishes freshness of the current STATE root.
        CurrentStateRootFreshness,
        /// Target consensus conserves every asset across a transaction.
        SubstrateConservation,
        /// The asset declaration fixes the singleton as non-reissuable.
        ///
        /// No input of any transaction can mint it, so the conservation
        /// equation for this asset carries no issuance term. This is a fact
        /// about the deployed declaration, not a check the leaf performs.
        SingletonNonReissuable,
        /// The singleton's issuance placed its whole amount under the constructor.
        ///
        /// This is the induction base for output closure: every unit has been
        /// under the covenant since issuance, and the covenant refuses to
        /// execute anywhere but position zero, so no second input can hold a
        /// unit to spend. It is a fact about a past transaction, established
        /// by deployment records rather than by these instructions.
        SingletonIssuedUnderConstructor,
    }
}

census_enum! {
    /// The one absence this recipe still claims, and what discharges it.
    ///
    /// Three things establish it together, and only one of them is in these
    /// bytes. The leaf pins the executing input to position zero, so a second
    /// object of this family cannot be spent elsewhere: its own covenant
    /// refuses to run there. The deployed asset declaration makes the singleton
    /// non-reissuable, so no input can mint another unit. The substrate
    /// conserves every asset across a transaction, so with one unit consumed
    /// and output zero receiving its exact amount, every other output carries
    /// none of it. The external evidence roles name the two facts this walk
    /// does not check.
    ///
    /// The absences this census used to carry about other object families were
    /// true only because the emitter had claimed every position. They are not
    /// facts about this operation once a transaction may compose several, and
    /// the recipe no longer asserts them.
    pub enum StateStructuralEvidence {
        /// The singleton occurs only at input zero and output zero.
        SingletonExclusiveToPositionZero,
    }
}

/// Refusals at the STATE pattern boundary.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum StatePatternRefusal {
    /// A typed program or its abstract schedule is invalid.
    #[error(transparent)]
    Program(#[from] TapscriptError),
    /// Bindings must contain exactly the consumers of this recipe.
    #[error("missing or unused STATE consumer")]
    ConsumerCensus,
    /// A binding has the wrong encoding or semantic domain.
    #[error("invalid STATE consumer binding: {0:?}")]
    InvalidBinding(StatePatternSymbol),
    /// A changed program cannot inherit the canonical pattern's identity.
    #[error("fragment differs from its authenticated structural recipe")]
    FragmentMismatch,
    /// The walk must preserve its empty stack on every successful path.
    #[error("structural walk leaves residue, failure or an unverified signature")]
    InvalidContract,
    /// A recipe must contain both distinct compatible components in order.
    #[error("structural component recipe is incomplete or incompatible")]
    ComponentRecipe,
}

/// Checked substitutions for walking a pre-link candidate.
///
/// These bytes remain unresolved consumers, even after encoding checks. No
/// method turns them into deployment bindings or establishes link closure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatePatternBindings {
    values: BTreeMap<StatePatternSymbol, StackItem>,
}

impl StatePatternBindings {
    /// Check exact consumer coverage and each binding's encoding.
    ///
    /// No shape selects these bindings. The emitted fragments read position
    /// zero and nothing else, so there is no count, suffix or output role for a
    /// shape to choose, and one recipe serves every transaction the contract
    /// permits.
    ///
    /// # Errors
    /// Returns a typed refusal for missing, unused or malformed bindings.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        values: BTreeMap<StatePatternSymbol, StackItem>,
    ) -> Result<Self, StatePatternRefusal> {
        if values.keys().copied().collect::<Vec<_>>() != StatePatternSymbol::ALL {
            return Err(StatePatternRefusal::ConsumerCensus);
        }
        for (&symbol, value) in &values {
            validate_binding(target, symbol, value)?;
        }
        Ok(Self { values })
    }
}

fn validate_binding(
    target: &ReviewedElementsTapscriptDefinition,
    symbol: StatePatternSymbol,
    item: &StackItem,
) -> Result<(), StatePatternRefusal> {
    use StatePatternSymbol as S;
    let valid = match symbol {
        S::StateAsset => item.len() == 32,
        S::StateAmount => item
            .signed_le64_value(target)
            .is_some_and(|value| value > 0),
    };
    if valid {
        Ok(())
    } else {
        Err(StatePatternRefusal::InvalidBinding(symbol))
    }
}

/// Exact instruction sites for one unresolved consumer.
///
/// Unlike the compact relocation site, this names a structural component,
/// not a compact leaf. Each set is nonempty and computed while emitting pushes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateConsumerRequirement {
    /// The future linker reference type required by these consumers.
    pub symbol: StatePatternSymbol,
    /// Every push site, grouped by the component that contains it.
    pub sites: BTreeMap<StatePatternId, BTreeSet<usize>>,
}

/// The dependency metadata carried by records and unioned by recipes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatePatternMetadata {
    sources: BTreeSet<RequiredSourceKind>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
    disclosure: BTreeSet<StateDisclosure>,
    residuals: BTreeSet<StatePatternResidual>,
    structural: BTreeSet<StateStructuralEvidence>,
    external: BTreeSet<StateExternalEvidenceRole>,
}

/// A complete structural record, constructed only by an abstract walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatePattern {
    id: StatePatternId,
    bindings: StatePatternBindings,
    witness: StatePatternWitness,
    constructibility: StatePatternConstructibility,
    fragment: TapscriptProgram,
    precondition: AbstractStackState,
    success: BTreeSet<AbstractStackState>,
    nonaborting: BTreeSet<AbstractStackState>,
    aborts: BTreeSet<FailureCause>,
    signature_forms: BTreeMap<usize, BTreeSet<SignatureSuccessForm>>,
    prerequisites: BTreeSet<ElementsCapability>,
    resources: BTreeMap<ResourceDimension, u64>,
    metadata: StatePatternMetadata,
    consumers: BTreeMap<StatePatternSymbol, StateConsumerRequirement>,
}

/// Structural fragments require no witness item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatePatternWitness {
    /// All operands come from introspection or unresolved consumer substitutions.
    NoWitnessItem,
}

/// What construction of these pre-link fragments requires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatePatternConstructibility {
    /// Checked substitutions permit a walk but do not resolve a consumer.
    CheckedUnresolvedConsumers,
}

impl StatePattern {
    /// The semantic responsibility of this identity.
    #[must_use]
    pub const fn owner(&self) -> StatePatternOwner {
        match self.id {
            StatePatternId::StateCoordinatorRoleV1 => StatePatternOwner::CoordinatorRole,
            StatePatternId::StateInputRecognitionV1 => StatePatternOwner::PredecessorRecognition,
        }
    }

    /// The witness role from which this record schedules.
    #[must_use]
    pub const fn witness(&self) -> StatePatternWitness {
        self.witness
    }

    /// Construction remains conditional on unresolved consumers.
    #[must_use]
    pub const fn constructibility(&self) -> StatePatternConstructibility {
        self.constructibility
    }

    /// Whether any successful signature form bypasses verification.
    #[must_use]
    pub fn reaches_unverified_success(&self) -> bool {
        self.signature_forms
            .values()
            .any(|forms| forms.contains(&SignatureSuccessForm::UnknownKeyTypeUnverified))
    }
}

/// The two-component structural recipe, without a composed program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatePatternRecipe {
    components: Vec<StatePattern>,
    metadata: StatePatternMetadata,
    consumers: BTreeMap<StatePatternSymbol, StateConsumerRequirement>,
}

impl StatePatternRecipe {
    /// Derive all aggregate metadata from exactly both compatible components.
    ///
    /// # Errors
    /// Returns `ComponentRecipe` for missing, duplicate, reordered or incompatible records.
    pub fn new(components: Vec<StatePattern>) -> Result<Self, StatePatternRefusal> {
        let ids: Vec<_> = components.iter().map(StatePattern::id).collect();
        if ids != StatePatternId::ALL
            || components
                .windows(2)
                .any(|pair| pair[0].bindings != pair[1].bindings)
        {
            return Err(StatePatternRefusal::ComponentRecipe);
        }
        let mut metadata = StatePatternMetadata::default();
        let mut consumers: BTreeMap<StatePatternSymbol, StateConsumerRequirement> = BTreeMap::new();
        for component in &components {
            metadata.sources.extend(&component.metadata.sources);
            metadata.evidence.extend(&component.metadata.evidence);
            metadata.disclosure.extend(&component.metadata.disclosure);
            metadata.residuals.extend(&component.metadata.residuals);
            metadata.structural.extend(&component.metadata.structural);
            metadata.external.extend(&component.metadata.external);
            for (&symbol, requirement) in &component.consumers {
                consumers
                    .entry(symbol)
                    .or_insert_with(|| StateConsumerRequirement {
                        symbol,
                        sites: BTreeMap::new(),
                    })
                    .sites
                    .extend(requirement.sites.clone());
            }
        }
        Ok(Self {
            components,
            metadata,
            consumers,
        })
    }
}

struct StructuralEmitter<'a> {
    target: &'a ReviewedElementsTapscriptDefinition,
    bindings: &'a StatePatternBindings,
    instructions: Vec<TapscriptInstruction>,
    sites: BTreeMap<StatePatternSymbol, BTreeSet<usize>>,
}

impl StructuralEmitter<'_> {
    fn symbol(&mut self, symbol: StatePatternSymbol) -> Result<(), StatePatternRefusal> {
        let item = self
            .bindings
            .values
            .get(&symbol)
            .ok_or(StatePatternRefusal::ConsumerCensus)?;
        self.sites
            .entry(symbol)
            .or_default()
            .insert(self.instructions.len());
        self.instructions
            .push(TapscriptInstruction::Push(item.clone()));
        Ok(())
    }

    fn asset(
        &mut self,
        inspect: OpcodeId,
        position: u16,
        symbol: StatePatternSymbol,
    ) -> Result<(), StatePatternRefusal> {
        self.instructions
            .extend([number(self.target, i64::from(position))?, op(inspect)]);
        self.instructions
            .extend(require_explicit(self.target, EncodingClass::ExplicitAsset)?);
        self.symbol(symbol)?;
        self.instructions.push(op(OpcodeId::EqualVerify));
        Ok(())
    }

    fn recognition(&mut self) -> Result<(), StatePatternRefusal> {
        self.asset(
            OpcodeId::InspectInputAsset,
            0,
            StatePatternSymbol::StateAsset,
        )?;
        self.instructions
            .extend([number(self.target, 0)?, op(OpcodeId::InspectInputValue)]);
        self.instructions
            .extend(require_explicit(self.target, EncodingClass::ExplicitValue)?);
        self.symbol(StatePatternSymbol::StateAmount)?;
        self.instructions.push(op(OpcodeId::EqualVerify));
        // Introspection returns the consumed program beneath its witness
        // version. The version is checked here and the program is dropped: a
        // literal equal to it cannot be linked, because its bytes would have to
        // occur inside the tree that commits to them, and comparing bytes to a
        // constant is in any case weaker than what the semantic authentication
        // component already does, which is to verify the tweak over the
        // internal key and the authenticated metadata for this same program.
        self.instructions.extend([
            number(self.target, 0)?,
            op(OpcodeId::InspectInputScriptPubKey),
            number(self.target, 1)?,
            op(OpcodeId::EqualVerify),
            op(OpcodeId::Drop),
        ]);
        Ok(())
    }
}

fn emit_structure<'a>(
    target: &'a ReviewedElementsTapscriptDefinition,
    bindings: &'a StatePatternBindings,
    id: StatePatternId,
) -> Result<StructuralEmitter<'a>, StatePatternRefusal> {
    let mut emitter = StructuralEmitter {
        target,
        bindings,
        instructions: Vec::new(),
        sites: BTreeMap::new(),
    };
    match id {
        // Three instructions are the whole coordinator: the executing index is
        // introspected and verified equal to zero. A second object of this
        // family therefore cannot be spent at any other position, because the
        // covenant it carries refuses to run there.
        StatePatternId::StateCoordinatorRoleV1 => emitter.instructions.extend([
            op(OpcodeId::PushCurrentInputIndex),
            number(target, 0)?,
            op(OpcodeId::EqualVerify),
        ]),
        StatePatternId::StateInputRecognitionV1 => emitter.recognition()?,
    }
    Ok(emitter)
}

/// Emit one structural fragment with checked, still-unresolved substitutions.
///
/// # Errors
/// Returns a typed refusal if emission exceeds the target instruction or encoding limits.
pub fn state_structural_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StatePatternBindings,
    id: StatePatternId,
) -> Result<TapscriptProgram, StatePatternRefusal> {
    Ok(TapscriptProgram::new(
        emit_structure(target, bindings, id)?.instructions,
    )?)
}

/// Admit a canonical fragment only after walking its contract.
///
/// No stack contract or metadata claim is accepted from the caller. Comparing
/// against the emitting recipe prevents a mutation from inheriting its identity.
///
/// # Errors
/// Returns a typed refusal for changed bytes or an invalid walk.
pub fn build_state_pattern(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StatePatternBindings,
    id: StatePatternId,
    fragment: TapscriptProgram,
) -> Result<StatePattern, StatePatternRefusal> {
    let emitted = emit_structure(target, bindings, id)?;
    if fragment.instructions() != emitted.instructions {
        return Err(StatePatternRefusal::FragmentMismatch);
    }
    let precondition = AbstractStackState::from_main(Vec::new());
    let execution = validate_program(
        target,
        &fragment,
        &precondition,
        AbstractLimits::for_target(target),
    )?;
    if execution.success() != &BTreeSet::from([precondition.clone()])
        || !execution.nonaborting_failure().is_empty()
        || execution
            .signature_forms()
            .values()
            .any(|forms| forms.contains(&SignatureSuccessForm::UnknownKeyTypeUnverified))
    {
        return Err(StatePatternRefusal::InvalidContract);
    }
    let consumers = emitted
        .sites
        .into_iter()
        .map(|(symbol, indices)| {
            (
                symbol,
                StateConsumerRequirement {
                    symbol,
                    sites: BTreeMap::from([(id, indices)]),
                },
            )
        })
        .collect();
    Ok(StatePattern {
        id,
        bindings: bindings.clone(),
        witness: StatePatternWitness::NoWitnessItem,
        constructibility: StatePatternConstructibility::CheckedUnresolvedConsumers,
        precondition,
        success: execution.success().clone(),
        nonaborting: execution.nonaborting_failure().clone(),
        aborts: execution.aborts().clone(),
        signature_forms: execution.signature_forms().clone(),
        prerequisites: fragment_prerequisites(&fragment),
        resources: resource_projection(target, &fragment),
        metadata: structural_metadata(target, id, &fragment),
        consumers,
        fragment,
    })
}

/// Walk both components and derive their shared recipe metadata.
///
/// # Errors
/// Propagates the first emission, walk or recipe refusal.
pub fn state_structural_patterns(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StatePatternBindings,
) -> Result<StatePatternRecipe, StatePatternRefusal> {
    let components = StatePatternId::ALL
        .iter()
        .map(|&id| {
            let fragment = state_structural_fragment(target, bindings, id)?;
            build_state_pattern(target, bindings, id, fragment)
        })
        .collect::<Result<Vec<_>, StatePatternRefusal>>()?;
    StatePatternRecipe::new(components)
}

fn structural_metadata(
    target: &ReviewedElementsTapscriptDefinition,
    id: StatePatternId,
    fragment: &TapscriptProgram,
) -> StatePatternMetadata {
    use RequiredSourceKind as Source;
    use StatePatternResidual as Residual;
    use TargetEvidenceRequirementId as Evidence;
    let mut metadata = StatePatternMetadata::default();
    metadata.evidence.extend([
        Evidence::TapscriptExecutionDomain,
        Evidence::LeafVersionActivation,
        Evidence::PushEncodingSemantics,
        Evidence::EncodingSemantics,
        Evidence::ConsensusResourceLimits,
    ]);
    for instruction in fragment.instructions() {
        if let TapscriptInstruction::Opcode(opcode) = instruction {
            metadata
                .evidence
                .extend(target.definition().opcodes()[opcode].evidence());
        }
    }
    metadata.residuals.extend([
        Residual::UnresolvedConsumers,
        Residual::TargetNativeEvidence,
    ]);
    metadata.sources.extend([
        Source::AuthenticatedInputObject,
        Source::PublicConstructionData,
    ]);
    metadata.sources.insert(Source::ExternalEvidence);
    match id {
        StatePatternId::StateInputRecognitionV1 => {
            metadata
                .external
                .insert(StateExternalEvidenceRole::CurrentStateRootFreshness);
            metadata.disclosure.extend([
                StateDisclosure::AssetIdentities,
                StateDisclosure::PredecessorCommitment,
            ]);
            metadata.residuals.extend([
                Residual::CurrentStateRootFreshness,
                Residual::SemanticMetadataAuthentication,
            ]);
        }
        // The pin is the leaf's whole contribution to singleton exclusivity.
        // The other two facts it rests on are about the deployment and the
        // substrate, so they are published as external roles and as the
        // conservation residual, never as something these bytes established.
        StatePatternId::StateCoordinatorRoleV1 => {
            metadata
                .structural
                .insert(StateStructuralEvidence::SingletonExclusiveToPositionZero);
            metadata.external.extend([
                StateExternalEvidenceRole::SubstrateConservation,
                StateExternalEvidenceRole::SingletonNonReissuable,
                StateExternalEvidenceRole::SingletonIssuedUnderConstructor,
            ]);
            metadata.residuals.insert(Residual::SubstrateConservation);
        }
    }
    metadata
}

impl StatePattern {
    /// Admitted identity.
    #[must_use]
    pub const fn id(&self) -> StatePatternId {
        self.id
    }

    /// Exact typed instructions.
    #[must_use]
    pub const fn fragment(&self) -> &TapscriptProgram {
        &self.fragment
    }

    /// Declared empty-stack schedule.
    #[must_use]
    pub const fn precondition(&self) -> &AbstractStackState {
        &self.precondition
    }

    /// Every successful stack state.
    #[must_use]
    pub const fn success(&self) -> &BTreeSet<AbstractStackState> {
        &self.success
    }

    /// Every non-aborting failure state.
    #[must_use]
    pub const fn nonaborting_failure(&self) -> &BTreeSet<AbstractStackState> {
        &self.nonaborting
    }

    /// Every named abort cause.
    #[must_use]
    pub const fn aborts(&self) -> &BTreeSet<FailureCause> {
        &self.aborts
    }

    /// Every reached signature form.
    #[must_use]
    pub const fn signature_forms(&self) -> &BTreeMap<usize, BTreeSet<SignatureSuccessForm>> {
        &self.signature_forms
    }

    /// Capabilities derived from emitted primitives.
    #[must_use]
    pub const fn prerequisites(&self) -> &BTreeSet<ElementsCapability> {
        &self.prerequisites
    }

    /// Exact projected resource dimensions.
    #[must_use]
    pub const fn resources(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.resources
    }

    /// Dependencies derived for this component.
    #[must_use]
    pub const fn metadata(&self) -> &StatePatternMetadata {
        &self.metadata
    }

    /// Every unresolved consumer and its actual push sites.
    #[must_use]
    pub const fn consumers(&self) -> &BTreeMap<StatePatternSymbol, StateConsumerRequirement> {
        &self.consumers
    }
}

impl StatePatternMetadata {
    /// Required authenticated sources.
    #[must_use]
    pub const fn sources(&self) -> &BTreeSet<RequiredSourceKind> {
        &self.sources
    }

    /// Target evidence required by the emitted instructions.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Disclosures made by these components.
    #[must_use]
    pub const fn disclosure(&self) -> &BTreeSet<StateDisclosure> {
        &self.disclosure
    }

    /// Outstanding obligations, never closure claims.
    #[must_use]
    pub const fn residuals(&self) -> &BTreeSet<StatePatternResidual> {
        &self.residuals
    }

    /// Absences derived from the authenticated position census.
    #[must_use]
    pub const fn structural(&self) -> &BTreeSet<StateStructuralEvidence> {
        &self.structural
    }
}

impl StatePatternRecipe {
    /// The exact ordered component records.
    #[must_use]
    pub const fn components(&self) -> &Vec<StatePattern> {
        &self.components
    }

    /// Unions over this recipe only.
    #[must_use]
    pub const fn metadata(&self) -> &StatePatternMetadata {
        &self.metadata
    }

    /// One requirement per symbol, with every consumer site.
    #[must_use]
    pub const fn consumers(&self) -> &BTreeMap<StatePatternSymbol, StateConsumerRequirement> {
        &self.consumers
    }
}

impl StatePatternMetadata {
    /// External roles kept distinct from emitted and structural evidence.
    #[must_use]
    pub const fn external(&self) -> &BTreeSet<StateExternalEvidenceRole> {
        &self.external
    }
}
