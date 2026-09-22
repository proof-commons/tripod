//! A complete announcement leaf walked from its declared witness.
//!
//! Fixture substitutions remain unresolved consumers. The composed record keeps
//! every component obligation, including those requiring external authentication.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use compiler::operation_plan::RequiredSourceKind;
use target_elements::{
    ElementsCapability, LeafVersion, OpcodeId, ResourceDimension,
    ReviewedElementsTapscriptDefinition, StackValueType, TargetEvidenceRequirementId,
};
use thiserror::Error;

use crate::pattern::{fragment_prerequisites, number, op};
use crate::state_announcement::{
    StateAnnouncementId, StateAnnouncementRecipe, StateAnnouncementResidual,
    StateAnnouncementSymbol, StateAnnouncementWitness,
};
use crate::state_constructor::{
    StateConstructorRefusal, StateLeafRole, StateStaticLeaf, StateStaticNode, StateStaticSubtree,
};
use crate::state_operator::{
    StateOperatorDisclosure, StateOperatorPattern, StateOperatorPatternId, StateOperatorResidual,
    StateOperatorSymbol,
};
use crate::state_pattern::{
    StateDisclosure, StateExternalEvidenceRole, StatePatternConstructibility, StatePatternId,
    StatePatternRecipe, StatePatternResidual, StatePatternSymbol, StateStructuralEvidence,
};
use crate::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, StackItem, TapscriptError,
    TapscriptInstruction, TapscriptProgram, resource_projection, validate_program,
};

/// Stack adapters have their own ranges, separate from component identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateProgramAdapter {
    /// Rotate the requested cycle above the authenticated metadata.
    LeadWindow,
    /// Swap the requested cycle and predecessor metadata.
    CopyThrough,
    /// Rotate the successor prefix above the reconstructed metadata.
    SuccessorReconstruction,
    /// Drop the derived metadata and push canonical true.
    FinalTruth,
}

/// Every admitted component identity and every explicit adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateProgramComponent {
    /// Committed operator authorization.
    Operator(StateOperatorPatternId),
    /// Structural authentication; each fragment owns a distinct range.
    Structural(StatePatternId),
    /// Semantic authentication or transition.
    Semantic(StateAnnouncementId),
    /// A named stack adapter.
    Adapter(StateProgramAdapter),
}

/// Shared asset and amount consumers use their structural identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateProgramSymbol {
    /// Structural consumer, including shared asset and amount bindings.
    Structural(StatePatternSymbol),
    /// Semantic-only consumer.
    Semantic(StateAnnouncementSymbol),
    /// Committed operator key.
    Operator(StateOperatorSymbol),
}

/// One checked item and every composed instruction that pushes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateProgramConsumer {
    /// A fixture substitution, not a deployment resolution.
    pub item: StackItem,
    /// Nonempty instruction sites in the composed program.
    pub sites: BTreeSet<usize>,
}

/// Public information retained from each component family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateProgramDisclosure {
    /// Structural disclosure.
    Structural(StateDisclosure),
    /// Semantic disclosure.
    Semantic(StateAnnouncementWitness),
    /// Operator disclosure.
    Operator(StateOperatorDisclosure),
}

/// Outstanding obligations retain their component-family identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateProgramResidual {
    /// Structural obligation.
    Structural(StatePatternResidual),
    /// Semantic obligation.
    Semantic(StateAnnouncementResidual),
    /// Operator obligation.
    Operator(StateOperatorResidual),
}

/// Exact unions over the recipe, with adapter opcode evidence included.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateProgramMetadata {
    /// Authenticated sources required by any component.
    pub sources: BTreeSet<RequiredSourceKind>,
    /// All component and adapter evidence requirements.
    pub evidence: BTreeSet<TargetEvidenceRequirementId>,
    /// Disclosures from all three families.
    pub disclosure: BTreeSet<StateProgramDisclosure>,
    /// Residuals from all three families.
    pub residuals: BTreeSet<StateProgramResidual>,
    /// Conditional structural absence evidence.
    pub structural: BTreeSet<StateStructuralEvidence>,
    /// External roles retained from the structural recipe.
    pub external: BTreeSet<StateExternalEvidenceRole>,
}

/// Witness roles, ordered deepest first by the record's declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateProgramWitness {
    /// Successor compressed-key prefix.
    SuccessorOutputKeyPrefix,
    /// Successor representation nonce.
    SuccessorNonce,
    /// Requested announcement cycle.
    RequestedCycle,
    /// Authenticated static subtree root.
    StaticSubtreeRoot,
    /// Canonical predecessor metadata.
    PredecessorMetadata,
    /// Predecessor compressed-key prefix.
    PredecessorOutputKeyPrefix,
    /// The operator fragment's signature item.
    OperatorSignature,
}

/// The predecessor metadata transport retained by a composed announcement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateWitnessSchedule {
    /// Every canonical metadata byte, retained for historical replay only.
    WholeMetadata,
    /// The canonical variable region, with constants restored by the executing leaf.
    VariableMetadata,
}

impl StateWitnessSchedule {
    /// Every schedule, in declaration order.
    pub const ALL: [Self; 2] = [Self::WholeMetadata, Self::VariableMetadata];

    /// The stable diagnostic name of the transport.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WholeMetadata => "whole-metadata",
            Self::VariableMetadata => "variable-metadata",
        }
    }

    /// Whether the transport is retained only for replay and reconstruction.
    #[must_use]
    pub const fn is_replay_only(self) -> bool {
        matches!(self, Self::WholeMetadata)
    }
}

/// Refusals at the composed-program boundary.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum StateProgramRefusal {
    /// The composer holds no legalization for the requested schedule.
    #[error("announcement witness schedule {} has no legalization", .0.name())]
    ScheduleLegalizationUnavailable(StateWitnessSchedule),
    /// A supplied program changed the complete recipe.
    #[error("announcement program differs from its component recipe")]
    ComponentRecipe,
    /// Consumer sites must be complete, nonempty and agree on their item.
    #[error("incomplete or inconsistent announcement consumer census")]
    ConsumerCensus,
    /// The declared witness must walk to one canonical true and verified authority.
    #[error("announcement final-stack or signature contract failed")]
    InvalidContract,
    /// A projected dimension exceeds its target limit.
    #[error("announcement resource limit exceeded: {0:?}")]
    ResourceLimit(ResourceDimension),
    /// Typed construction or abstract execution failed.
    #[error(transparent)]
    Program(#[from] TapscriptError),
    /// The static subtree was refused.
    #[error(transparent)]
    Subtree(#[from] StateConstructorRefusal),
}

/// An immutable complete record admitted by recipe equality and abstract execution.
///
/// The retained witness schedule lets a consumer compose, replay or compare against
/// the schedule this record was composed under. A schedule for which the composer
/// holds no legalization is refused rather than composed as another schedule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateAnnouncementProgram {
    schedule: StateWitnessSchedule,
    program: TapscriptProgram,
    witness: Vec<(StateProgramWitness, StackValueType)>,
    precondition: AbstractStackState,
    execution: AbstractExecutionResult,
    prerequisites: BTreeSet<ElementsCapability>,
    resources: BTreeMap<ResourceDimension, u64>,
    metadata: StateProgramMetadata,
    consumers: BTreeMap<StateProgramSymbol, StateProgramConsumer>,
    components: BTreeMap<StateProgramComponent, Range<usize>>,
}

#[derive(Default)]
struct Assembly {
    instructions: Vec<TapscriptInstruction>,
    components: BTreeMap<StateProgramComponent, Range<usize>>,
    consumers: BTreeMap<StateProgramSymbol, StateProgramConsumer>,
}

impl Assembly {
    fn append(&mut self, id: StateProgramComponent, instructions: &[TapscriptInstruction]) {
        let start = self.instructions.len();
        self.instructions.extend_from_slice(instructions);
        self.components.insert(id, start..self.instructions.len());
    }

    fn consumer(
        &mut self,
        id: StateProgramComponent,
        symbol: StateProgramSymbol,
        sites: &BTreeSet<usize>,
    ) -> Result<(), StateProgramRefusal> {
        let range = self
            .components
            .get(&id)
            .ok_or(StateProgramRefusal::ComponentRecipe)?;
        if sites.is_empty() {
            return Err(StateProgramRefusal::ConsumerCensus);
        }
        for &site in sites {
            if site >= range.len() {
                return Err(StateProgramRefusal::ConsumerCensus);
            }
            let index = range.start + site;
            let TapscriptInstruction::Push(item) = &self.instructions[index] else {
                return Err(StateProgramRefusal::ConsumerCensus);
            };
            let consumer = self
                .consumers
                .entry(symbol)
                .or_insert_with(|| StateProgramConsumer {
                    item: item.clone(),
                    sites: BTreeSet::new(),
                });
            if &consumer.item != item {
                return Err(StateProgramRefusal::ConsumerCensus);
            }
            consumer.sites.insert(index);
        }
        Ok(())
    }

    fn adapter(
        &mut self,
        target: &ReviewedElementsTapscriptDefinition,
        adapter: StateProgramAdapter,
    ) -> Result<(), StateProgramRefusal> {
        let instructions = match adapter {
            StateProgramAdapter::LeadWindow | StateProgramAdapter::SuccessorReconstruction => {
                vec![op(OpcodeId::Rotate)]
            }
            StateProgramAdapter::CopyThrough => vec![op(OpcodeId::Swap)],
            StateProgramAdapter::FinalTruth => vec![op(OpcodeId::Drop), number(target, 1)?],
        };
        self.append(StateProgramComponent::Adapter(adapter), &instructions);
        Ok(())
    }
}

fn assemble(
    target: &ReviewedElementsTapscriptDefinition,
    structural: &StatePatternRecipe,
    semantic: &StateAnnouncementRecipe,
    operator: &StateOperatorPattern,
) -> Result<Assembly, StateProgramRefusal> {
    use StateProgramComponent as C;
    let mut assembly = Assembly::default();
    assembly.append(
        C::Operator(operator.id()),
        operator.fragment().instructions(),
    );
    for (&symbol, sites) in operator.consumers() {
        assembly.consumer(
            C::Operator(operator.id()),
            StateProgramSymbol::Operator(symbol),
            sites,
        )?;
    }
    for component in structural.components() {
        let id = C::Structural(component.id());
        assembly.append(id, component.fragment().instructions());
        for (&symbol, consumer) in component.consumers() {
            for sites in consumer.sites.values() {
                assembly.consumer(id, StateProgramSymbol::Structural(symbol), sites)?;
            }
        }
    }
    for component in semantic.components() {
        let adapter = match component.id() {
            StateAnnouncementId::LeadWindow => Some(StateProgramAdapter::LeadWindow),
            StateAnnouncementId::CopyThrough => Some(StateProgramAdapter::CopyThrough),
            StateAnnouncementId::SuccessorReconstruction => {
                Some(StateProgramAdapter::SuccessorReconstruction)
            }
            _ => None,
        };
        if let Some(adapter) = adapter {
            assembly.adapter(target, adapter)?;
        }
        let id = C::Semantic(*component.id());
        assembly.append(id, component.fragment().instructions());
        for (&symbol, consumer) in component.consumers() {
            let symbol = symbol.structural().map_or(
                StateProgramSymbol::Semantic(symbol),
                StateProgramSymbol::Structural,
            );
            for sites in consumer.sites.values() {
                assembly.consumer(id, symbol, sites)?;
            }
        }
    }
    assembly.adapter(target, StateProgramAdapter::FinalTruth)?;
    Ok(assembly)
}

fn witness(
    semantic: &StateAnnouncementRecipe,
    operator: &StateOperatorPattern,
) -> Result<Vec<(StateProgramWitness, StackValueType)>, StateProgramRefusal> {
    use StateAnnouncementId as I;
    use StateAnnouncementWitness as W;
    use StateProgramWitness as P;
    let mut declaration = Vec::new();
    for (role, id, source) in [
        (
            P::SuccessorOutputKeyPrefix,
            I::SuccessorReconstruction,
            W::OutputKeyPrefix,
        ),
        (P::SuccessorNonce, I::CopyThrough, W::SuccessorNonce),
        (P::RequestedCycle, I::LeadWindow, W::RequestedCycle),
        (
            P::StaticSubtreeRoot,
            I::MetadataAuthentication,
            W::StaticSubtreeRoot,
        ),
        (
            P::PredecessorMetadata,
            I::MetadataAuthentication,
            W::PredecessorMetadata,
        ),
        (
            P::PredecessorOutputKeyPrefix,
            I::MetadataAuthentication,
            W::OutputKeyPrefix,
        ),
    ] {
        let component = semantic
            .components()
            .iter()
            .find(|component| component.id() == &id)
            .ok_or(StateProgramRefusal::ComponentRecipe)?;
        let index = id
            .witness()
            .iter()
            .position(|&w| w == source)
            .ok_or(StateProgramRefusal::ComponentRecipe)?;
        let value = component
            .precondition()
            .main()
            .get(index)
            .ok_or(StateProgramRefusal::ComponentRecipe)?;
        declaration.push((role, value.clone()));
    }
    let [signature] = operator.precondition().main() else {
        return Err(StateProgramRefusal::ComponentRecipe);
    };
    declaration.push((P::OperatorSignature, signature.clone()));
    Ok(declaration)
}

fn metadata(
    target: &ReviewedElementsTapscriptDefinition,
    structural: &StatePatternRecipe,
    semantic: &StateAnnouncementRecipe,
    operator: &StateOperatorPattern,
    assembly: &Assembly,
) -> StateProgramMetadata {
    let s = structural.metadata();
    let a = semantic.metadata();
    let mut result = StateProgramMetadata::default();
    result.sources.extend(s.sources());
    result.sources.extend(&a.sources);
    result.sources.extend(operator.sources());
    result.evidence.extend(s.evidence());
    result.evidence.extend(&a.evidence);
    result.evidence.extend(operator.evidence());
    result.disclosure.extend(
        s.disclosure()
            .iter()
            .copied()
            .map(StateProgramDisclosure::Structural),
    );
    result.disclosure.extend(
        a.disclosure
            .iter()
            .copied()
            .map(StateProgramDisclosure::Semantic),
    );
    result.disclosure.extend(
        operator
            .disclosure()
            .iter()
            .copied()
            .map(StateProgramDisclosure::Operator),
    );
    result.residuals.extend(
        s.residuals()
            .iter()
            .copied()
            .map(StateProgramResidual::Structural),
    );
    result.residuals.extend(
        a.residuals
            .iter()
            .copied()
            .map(StateProgramResidual::Semantic),
    );
    result.residuals.extend(
        operator
            .residuals()
            .iter()
            .copied()
            .map(StateProgramResidual::Operator),
    );
    result.structural.extend(s.structural());
    result.external.extend(s.external());
    for (id, range) in &assembly.components {
        if matches!(id, StateProgramComponent::Adapter(_)) {
            for instruction in &assembly.instructions[range.clone()] {
                if let TapscriptInstruction::Opcode(opcode) = instruction {
                    result
                        .evidence
                        .extend(target.definition().opcodes()[opcode].evidence());
                }
            }
        }
    }
    result
}

/// Emit the exact complete recipe in component order.
///
/// # Errors
/// Refuses schedules without a legalization, incompatible shared consumers or
/// invalid typed instructions.
#[must_use = "composition can refuse the requested schedule or recipe"]
pub fn state_announcement_program(
    target: &ReviewedElementsTapscriptDefinition,
    structural: &StatePatternRecipe,
    semantic: &StateAnnouncementRecipe,
    operator: &StateOperatorPattern,
    schedule: StateWitnessSchedule,
) -> Result<TapscriptProgram, StateProgramRefusal> {
    if schedule == StateWitnessSchedule::VariableMetadata {
        return Err(StateProgramRefusal::ScheduleLegalizationUnavailable(
            schedule,
        ));
    }
    Ok(TapscriptProgram::new(
        assemble(target, structural, semantic, operator)?.instructions,
    )?)
}

/// Admit a complete recipe only after walking its declared witness.
///
/// The abstract stack tracks widths, so exact recipe equality also proves the
/// final item is the canonical true literal, not just another one-byte value.
///
/// # Errors
/// Refuses schedules without a legalization, recipe changes, inconsistent
/// consumers, contract or resource failures.
#[must_use = "record admission can refuse the requested schedule or recipe"]
pub fn build_state_announcement_program(
    target: &ReviewedElementsTapscriptDefinition,
    structural: &StatePatternRecipe,
    semantic: &StateAnnouncementRecipe,
    operator: &StateOperatorPattern,
    schedule: StateWitnessSchedule,
    program: TapscriptProgram,
) -> Result<StateAnnouncementProgram, StateProgramRefusal> {
    if schedule == StateWitnessSchedule::VariableMetadata {
        return Err(StateProgramRefusal::ScheduleLegalizationUnavailable(
            schedule,
        ));
    }
    let assembly = assemble(target, structural, semantic, operator)?;
    if assembly.instructions != program.instructions() {
        return Err(StateProgramRefusal::ComponentRecipe);
    }
    let witness = witness(semantic, operator)?;
    let precondition =
        AbstractStackState::from_main(witness.iter().map(|(_, value)| value.clone()).collect());
    let execution = validate_program(
        target,
        &program,
        &precondition,
        AbstractLimits::for_target(target),
    )?;
    let expected = AbstractStackState::from_main(vec![StackValueType::Bytes {
        minimum: 1,
        maximum: 1,
    }]);
    if execution.success() != &BTreeSet::from([expected])
        || !execution.nonaborting_failure().is_empty()
        || execution.signature_forms() != operator.execution().signature_forms()
    {
        return Err(StateProgramRefusal::InvalidContract);
    }
    let resources = resource_projection(target, &program);
    for (&dimension, &charged) in &resources {
        if target
            .definition()
            .resources()
            .consensus()
            .bounds()
            .get(&dimension)
            .and_then(|bound| bound.maximum())
            .is_some_and(|maximum| charged > maximum)
        {
            return Err(StateProgramRefusal::ResourceLimit(dimension));
        }
    }
    Ok(StateAnnouncementProgram {
        schedule,
        prerequisites: fragment_prerequisites(&program),
        metadata: metadata(target, structural, semantic, operator, &assembly),
        program,
        witness,
        precondition,
        execution,
        resources,
        consumers: assembly.consumers,
        components: assembly.components,
    })
}

/// Build the production subtree with the single complete announcement leaf.
///
/// No support leaf is needed because the recipe carries every consumer inside
/// the announcement program. An incomplete consumer census is refused.
///
/// # Errors
/// Refuses missing or inconsistent consumer sites or an invalid static leaf.
pub fn production_static_subtree(
    target: &ReviewedElementsTapscriptDefinition,
    program: &StateAnnouncementProgram,
) -> Result<StateStaticSubtree, StateProgramRefusal> {
    if program.consumers.is_empty()
        || program.consumers.values().any(|consumer| {
            consumer.sites.is_empty()
                || consumer.sites.iter().any(|&site| {
                    program.program.instructions().get(site)
                        != Some(&TapscriptInstruction::Push(consumer.item.clone()))
                })
        })
    {
        return Err(StateProgramRefusal::ConsumerCensus);
    }
    Ok(StateStaticSubtree::new(
        target,
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role: StateLeafRole::Announcement,
                version: LeafVersion::TAPSCRIPT.get(),
                program: program.program.clone(),
            },
        }),
    )?)
}

impl StateAnnouncementProgram {
    /// The witness schedule this record was composed under.
    #[must_use]
    pub const fn schedule(&self) -> StateWitnessSchedule {
        self.schedule
    }

    /// Exact composed instructions.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }
    /// Declared witness roles and types, deepest first.
    #[must_use]
    pub const fn witness(&self) -> &Vec<(StateProgramWitness, StackValueType)> {
        &self.witness
    }
    /// The complete declared starting stack.
    #[must_use]
    pub const fn precondition(&self) -> &AbstractStackState {
        &self.precondition
    }
    /// Walked success, failure, abort and signature contracts.
    #[must_use]
    pub const fn execution(&self) -> &AbstractExecutionResult {
        &self.execution
    }
    /// Capabilities derived from the complete program.
    #[must_use]
    pub const fn prerequisites(&self) -> &BTreeSet<ElementsCapability> {
        &self.prerequisites
    }
    /// Projected resources checked against the target.
    #[must_use]
    pub const fn resources(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.resources
    }
    /// Exact component unions and adapter evidence.
    #[must_use]
    pub const fn metadata(&self) -> &StateProgramMetadata {
        &self.metadata
    }
    /// Complete consumer census in composed instruction coordinates.
    #[must_use]
    pub const fn consumers(&self) -> &BTreeMap<StateProgramSymbol, StateProgramConsumer> {
        &self.consumers
    }
    /// Component and adapter ranges; every range is disjoint.
    #[must_use]
    pub const fn components(&self) -> &BTreeMap<StateProgramComponent, Range<usize>> {
        &self.components
    }
    /// Checked substitutions never establish deployment resolution.
    #[must_use]
    pub const fn constructibility(&self) -> StatePatternConstructibility {
        StatePatternConstructibility::CheckedUnresolvedConsumers
    }
}
