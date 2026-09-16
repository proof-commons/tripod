//! Walked STATE structural fragments and their unresolved consumers.
//!
//! A fragment authenticates positions before deriving structural absence.
//! Constructor-program equality does not establish current-root freshness or
//! semantic metadata. Those obligations remain separate residuals. The metadata
//! commitment leaf can be adapted by a later composing recipe; it is not an
//! executing authentication fragment and is not embedded here.
//!
//! The partition consumes an authenticated family census. Reserve-asset equality
//! alone cannot distinguish plain sponsors from RESV, which also carries L-BTC.
//! Structural evidence rules out additional positions under that source
//! requirement; it is not a byte-level classification of arbitrary L-BTC inputs.
//! Successor authentication and family-role authentication remain obligations.

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
use crate::live_plan::reads_a_value_field;
use crate::live_shape::FeePresence;
use crate::pattern::{fragment_prerequisites, number, op, require_explicit};
use crate::program::TapscriptProgram;
use crate::shape::SponsorChangePresence;
use crate::stack::{
    AbstractLimits, AbstractStackState, SignatureSuccessForm, resource_projection, validate_program,
};

census_enum! {
    /// Identities admitted only through a successful structural walk.
    pub enum StatePatternId {
        /// Input zero coordinates the authenticated partition.
        StateCoordinatorRoleV1,
        /// Exactly one STATE position occurs on each side.
        StateCardinalityV1,
        /// The predecessor matches the exact asset, amount and constructor.
        StateInputRecognitionV1,
        /// The sponsor suffix never supplies a value operand.
        StateSponsorIsolationV1,
        /// Every input is issuance-free and every position is classified.
        StateIssuanceAbsenceV1,
    }
}

census_enum! {
    /// Semantic responsibilities of the structural fragments.
    pub enum StatePatternOwner {
        /// Coordinator participation at input zero.
        CoordinatorRole,
        /// STATE and sponsor family counts.
        FamilyCardinality,
        /// Exact predecessor constructor recognition.
        PredecessorRecognition,
        /// Reserve-asset isolation and sponsor roles.
        SponsorIsolation,
        /// Issuance and structurally excluded families.
        AbsenceRelations,
    }
}

census_enum! {
    /// Typed consumers awaiting linker definitions and resolution.
    ///
    /// None is one of the constructor's seven reference declarations: these
    /// name assets, values, programs or a fee bound, not constructor policy.
    /// Each therefore requires a corresponding future linker reference type.
    pub enum StatePatternSymbol {
        /// Exact PID singleton asset payload.
        StateAsset,
        /// Exact explicit singleton amount payload.
        StateAmount,
        /// Exact predecessor taproot output-program payload.
        PredecessorProgram,
        /// Inclusive fee-sponsor input bound, encoded as a script number.
        FeeSponsorInputMax,
        /// Reserve asset distinct from the STATE asset.
        ReserveAsset,
        /// Exact sponsor-change witness program.
        SponsorChangeProgram,
        /// Witness version of the sponsor-change program.
        SponsorChangeVersion,
        /// Digest identifying the target's empty fee program.
        FeeProgramDigest,
    }
}

census_enum! {
    /// What a structural fragment publishes.
    pub enum StateDisclosure {
        /// Exact input and output counts and their assigned roles.
        PositionCensus,
        /// Asset identities checked at positions.
        AssetIdentities,
        /// Exact predecessor amount and program.
        PredecessorCommitment,
        /// Sponsor-change and target fee programs.
        SponsorPrograms,
        /// Absence of issuance on every input.
        IssuanceAbsence,
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
        /// Constructor equality does not authenticate semantic metadata.
        SemanticMetadataAuthentication,
        /// The constructor program must be supplied without assuming a fixed point.
        PredecessorConstructorBinding,
        /// The required source census must authenticate family roles, not just assets.
        AuthenticatedFamilyRoles,
        /// Output zero still needs semantic successor reconstruction.
        SuccessorConstructorAuthentication,
        /// The substrate, not sponsor-value reads, establishes reserve conservation.
        SubstrateConservation,
    }
}

census_enum! {
    /// Evidence roles outside the structural leaf's authority.
    pub enum StateExternalEvidenceRole {
        /// Report-layer chain context establishes freshness of the current STATE root.
        CurrentStateRootFreshness,
        /// Target consensus establishes reserve conservation without sponsor-value reads.
        SubstrateConservation,
    }
}

census_enum! {
    /// Conditional absence evidence from a complete authenticated family census.
    ///
    /// The inspected counts and asset partition leave no additional position.
    /// These facts require authenticated family roles and successor recognition;
    /// the corresponding residuals prevent treating asset equality as either.
    pub enum StateStructuralEvidence {
        /// Only input zero and output zero carry the singleton asset.
        NoSecondState,
        /// No position remains for RESV, PACE, authority, receipt or ASH objects.
        NoOtherObjectFamily,
        /// No output remains for entitlement, vault or control objects.
        NoOtherOutputFamily,
        /// No position remains for destruction or specialized event roles.
        NoDestructionOrSpecializedEvent,
    }
}

/// An exact candidate partition, with fee outputs separate from sponsor change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateAnnouncementShape {
    sponsor_inputs: u8,
    change: SponsorChangePresence,
    fee: FeePresence,
}

impl StateAnnouncementShape {
    /// Select the bounded suffix and the two optional output roles.
    #[must_use]
    pub const fn new(sponsor_inputs: u8, change: SponsorChangePresence, fee: FeePresence) -> Self {
        Self {
            sponsor_inputs,
            change,
            fee,
        }
    }

    /// Number of sponsor inputs following input zero.
    #[must_use]
    pub const fn sponsor_inputs(self) -> u8 {
        self.sponsor_inputs
    }

    /// Exact input count, including the singleton.
    #[must_use]
    pub const fn inputs(self) -> u16 {
        1 + self.sponsor_inputs as u16
    }

    /// Exact output count, including the singleton and optional fee.
    #[must_use]
    pub const fn outputs(self) -> u16 {
        1 + self.has_change() as u16 + self.has_fee() as u16
    }

    /// Whether output one is sponsor change.
    #[must_use]
    pub const fn has_change(self) -> bool {
        matches!(self.change, SponsorChangePresence::Present)
    }

    /// Whether the last output is the fee role.
    #[must_use]
    pub const fn has_fee(self) -> bool {
        matches!(self.fee, FeePresence::Present)
    }

    fn symbols(self) -> BTreeSet<StatePatternSymbol> {
        use StatePatternSymbol as S;
        let mut symbols = BTreeSet::from([
            S::StateAsset,
            S::StateAmount,
            S::PredecessorProgram,
            S::FeeSponsorInputMax,
        ]);
        if self.sponsor_inputs > 0 || self.has_change() || self.has_fee() {
            symbols.insert(S::ReserveAsset);
        }
        if self.has_change() {
            symbols.extend([S::SponsorChangeProgram, S::SponsorChangeVersion]);
        }
        if self.has_fee() {
            symbols.insert(S::FeeProgramDigest);
        }
        symbols
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
    /// The two asset families must be disjoint.
    #[error("STATE and reserve assets coincide")]
    AssetFamiliesOverlap,
    /// A value-field primitive violates sponsor isolation.
    #[error("sponsor isolation reads a value field")]
    SponsorValueRead,
    /// A changed program cannot inherit the canonical pattern's identity.
    #[error("fragment differs from its authenticated structural recipe")]
    FragmentMismatch,
    /// The walk must preserve its empty stack on every successful path.
    #[error("structural walk leaves residue, failure or an unverified signature")]
    InvalidContract,
    /// A recipe must contain all five distinct compatible components in order.
    #[error("structural component recipe is incomplete or incompatible")]
    ComponentRecipe,
}

/// Checked substitutions for walking a pre-link candidate.
///
/// These bytes remain unresolved consumers, even after encoding checks. No
/// method turns them into deployment bindings or establishes link closure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatePatternBindings {
    shape: StateAnnouncementShape,
    values: BTreeMap<StatePatternSymbol, StackItem>,
}

impl StatePatternBindings {
    /// Check exact consumer coverage, encodings, the bound and disjoint assets.
    ///
    /// # Errors
    /// Returns a typed refusal for missing, unused, malformed or overlapping bindings.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        shape: StateAnnouncementShape,
        values: BTreeMap<StatePatternSymbol, StackItem>,
    ) -> Result<Self, StatePatternRefusal> {
        use StatePatternSymbol as S;
        if values.keys().copied().collect::<BTreeSet<_>>() != shape.symbols() {
            return Err(StatePatternRefusal::ConsumerCensus);
        }
        for (&symbol, value) in &values {
            validate_binding(target, symbol, value)?;
        }
        if values.get(&S::ReserveAsset) == values.get(&S::StateAsset) {
            return Err(StatePatternRefusal::AssetFamiliesOverlap);
        }
        let maximum = values[&S::FeeSponsorInputMax].script_number_value(target);
        if maximum.is_none_or(|bound| bound < i64::from(shape.sponsor_inputs)) {
            return Err(StatePatternRefusal::InvalidBinding(S::FeeSponsorInputMax));
        }
        Ok(Self { shape, values })
    }

    /// The shape these exact substitutions were checked against.
    #[must_use]
    pub const fn shape(&self) -> StateAnnouncementShape {
        self.shape
    }
}

fn validate_binding(
    target: &ReviewedElementsTapscriptDefinition,
    symbol: StatePatternSymbol,
    item: &StackItem,
) -> Result<(), StatePatternRefusal> {
    use StatePatternSymbol as S;
    let valid = match symbol {
        S::StateAsset | S::ReserveAsset | S::PredecessorProgram | S::FeeProgramDigest => {
            item.len() == 32
        }
        S::StateAmount => item
            .signed_le64_value(target)
            .is_some_and(|value| value > 0),
        S::FeeSponsorInputMax => item
            .script_number_value(target)
            .is_some_and(|value| (0..=255).contains(&value)),
        S::SponsorChangeProgram => {
            StackItem::encoded(target, EncodingClass::WitnessProgram, item.bytes().to_vec()).is_ok()
        }
        S::SponsorChangeVersion => item
            .script_number_value(target)
            .is_some_and(|value| (0..=16).contains(&value)),
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
            StatePatternId::StateCardinalityV1 => StatePatternOwner::FamilyCardinality,
            StatePatternId::StateInputRecognitionV1 => StatePatternOwner::PredecessorRecognition,
            StatePatternId::StateSponsorIsolationV1 => StatePatternOwner::SponsorIsolation,
            StatePatternId::StateIssuanceAbsenceV1 => StatePatternOwner::AbsenceRelations,
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

/// The five-component structural recipe, without a composed program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatePatternRecipe {
    components: Vec<StatePattern>,
    metadata: StatePatternMetadata,
    consumers: BTreeMap<StatePatternSymbol, StateConsumerRequirement>,
}

impl StatePatternRecipe {
    /// Derive all aggregate metadata from exactly the five compatible components.
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

    fn counts(&mut self) -> Result<(), StatePatternRefusal> {
        let shape = self.bindings.shape;
        self.instructions.extend([
            op(OpcodeId::InspectNumInputs),
            number(self.target, i64::from(shape.inputs()))?,
            op(OpcodeId::EqualVerify),
            op(OpcodeId::InspectNumOutputs),
            number(self.target, i64::from(shape.outputs()))?,
            op(OpcodeId::EqualVerify),
            number(self.target, i64::from(shape.sponsor_inputs))?,
            op(OpcodeId::ScriptNumToLe64),
        ]);
        self.symbol(StatePatternSymbol::FeeSponsorInputMax)?;
        self.instructions.extend([
            op(OpcodeId::ScriptNumToLe64),
            op(OpcodeId::LessThanOrEqual64),
            op(OpcodeId::Verify),
        ]);
        Ok(())
    }

    fn partition(&mut self) -> Result<(), StatePatternRefusal> {
        use StatePatternSymbol as S;
        self.counts()?;
        self.asset(OpcodeId::InspectInputAsset, 0, S::StateAsset)?;
        self.asset(OpcodeId::InspectOutputAsset, 0, S::StateAsset)?;
        let shape = self.bindings.shape;
        for position in 1..shape.inputs() {
            self.asset(OpcodeId::InspectInputAsset, position, S::ReserveAsset)?;
        }
        if shape.has_change() {
            self.asset(OpcodeId::InspectOutputAsset, 1, S::ReserveAsset)?;
            self.instructions.extend([
                number(self.target, 1)?,
                op(OpcodeId::InspectOutputScriptPubKey),
            ]);
            self.symbol(S::SponsorChangeVersion)?;
            self.instructions.push(op(OpcodeId::EqualVerify));
            self.symbol(S::SponsorChangeProgram)?;
            self.instructions.push(op(OpcodeId::EqualVerify));
        }
        if shape.has_fee() {
            let position = shape.outputs() - 1;
            self.asset(OpcodeId::InspectOutputAsset, position, S::ReserveAsset)?;
            self.instructions.extend([
                number(self.target, i64::from(position))?,
                op(OpcodeId::InspectOutputScriptPubKey),
                number(self.target, -1)?,
                op(OpcodeId::EqualVerify),
            ]);
            self.symbol(S::FeeProgramDigest)?;
            self.instructions.push(op(OpcodeId::EqualVerify));
        }
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
        self.instructions.extend([
            number(self.target, 0)?,
            op(OpcodeId::InspectInputScriptPubKey),
            number(self.target, 1)?,
            op(OpcodeId::EqualVerify),
        ]);
        self.symbol(StatePatternSymbol::PredecessorProgram)?;
        self.instructions.push(op(OpcodeId::EqualVerify));
        Ok(())
    }

    fn issuance(&mut self) -> Result<(), StatePatternRefusal> {
        for position in 0..self.bindings.shape.inputs() {
            self.instructions.extend([
                number(self.target, i64::from(position))?,
                op(OpcodeId::InspectInputIssuance),
                TapscriptInstruction::Push(StackItem::empty()),
                op(OpcodeId::EqualVerify),
            ]);
        }
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
    if id == StatePatternId::StateInputRecognitionV1 {
        emitter.recognition()?;
    } else {
        emitter.partition()?;
    }
    match id {
        StatePatternId::StateCoordinatorRoleV1 => emitter.instructions.extend([
            op(OpcodeId::PushCurrentInputIndex),
            number(target, 0)?,
            op(OpcodeId::EqualVerify),
        ]),
        StatePatternId::StateIssuanceAbsenceV1 => emitter.issuance()?,
        _ => (),
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
/// The sponsor check uses the existing value-field census before that comparison.
///
/// # Errors
/// Returns a typed refusal for changed bytes, sponsor-value reads or an invalid walk.
pub fn build_state_pattern(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StatePatternBindings,
    id: StatePatternId,
    fragment: TapscriptProgram,
) -> Result<StatePattern, StatePatternRefusal> {
    if id == StatePatternId::StateSponsorIsolationV1 && reads_a_value_field(&fragment) {
        return Err(StatePatternRefusal::SponsorValueRead);
    }
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
        metadata: structural_metadata(target, bindings.shape, id, &fragment),
        consumers,
        fragment,
    })
}

/// Walk each of the five components and derive their shared recipe metadata.
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
    shape: StateAnnouncementShape,
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
    metadata.disclosure.insert(StateDisclosure::AssetIdentities);
    if id == StatePatternId::StateInputRecognitionV1 {
        metadata
            .external
            .insert(StateExternalEvidenceRole::CurrentStateRootFreshness);
        metadata.sources.insert(Source::ExternalEvidence);
        metadata
            .disclosure
            .insert(StateDisclosure::PredecessorCommitment);
        metadata.residuals.extend([
            Residual::CurrentStateRootFreshness,
            Residual::SemanticMetadataAuthentication,
            Residual::PredecessorConstructorBinding,
        ]);
    } else {
        metadata.sources.insert(Source::RuntimeArchitectureBound);
        metadata.sources.extend([
            Source::AuthenticatedFamilyCensus,
            Source::AuthenticatedOutputObject,
        ]);
        metadata.disclosure.insert(StateDisclosure::PositionCensus);
        metadata.structural.extend(StateStructuralEvidence::ALL);
        metadata
            .residuals
            .insert(Residual::AuthenticatedFamilyRoles);
        metadata
            .residuals
            .insert(Residual::SuccessorConstructorAuthentication);
        if shape.has_change() || shape.has_fee() {
            metadata.disclosure.insert(StateDisclosure::SponsorPrograms);
        }
        if shape.sponsor_inputs > 0 || shape.has_change() || shape.has_fee() {
            metadata.residuals.insert(Residual::SubstrateConservation);
            metadata
                .external
                .insert(StateExternalEvidenceRole::SubstrateConservation);
            metadata.sources.insert(Source::ExternalEvidence);
        }
        if shape.has_fee() {
            metadata.evidence.insert(Evidence::FeeOutputForm);
        } else {
            metadata
                .evidence
                .insert(Evidence::FeelessTransactionAdmission);
        }
    }
    if id == StatePatternId::StateIssuanceAbsenceV1 {
        metadata.disclosure.insert(StateDisclosure::IssuanceAbsence);
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
