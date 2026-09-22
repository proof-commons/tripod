//! Walked semantic STATE fragments with explicit witness schedules.
//!
//! Consumers remain unresolved after fixture substitution. Metadata is assumed
//! canonical only after predecessor construction has been authenticated; root
//! freshness and target-native evidence remain external obligations.
//! A single static leaf cannot contain its own root, so the witnessed root is
//! bound to the introspected program by the tweak relation and reused to make
//! the successor commit to the same static subtree.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::RequiredSourceKind;
use realization::{
    STATE_METADATA_BYTES, STATE_METADATA_DOMAIN, STATE_METADATA_VARIABLE_BYTES,
    STATE_METADATA_VARIABLE_RANGE, StateField,
};
use sha2::{Digest, Sha256};
use target_elements::{
    ElementsCapability, EncodingClass, FailureCause, OpcodeId, ResourceDimension,
    ReviewedElementsTapscriptDefinition, StackValueType, TargetEvidenceRequirementId,
};
use thiserror::Error;

use crate::capability::census_enum;
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::pattern::{fragment_prerequisites, number, op, require_explicit};
use crate::program::TapscriptProgram;
use crate::stack::{AbstractLimits, AbstractStackState, resource_projection, validate_program};
use crate::state_pattern::{StatePatternConstructibility, StatePatternSymbol};
use crate::state_program::StateWitnessSchedule;

census_enum! {
    /// Identities admitted after walking their witness schedules.
    pub enum StateAnnouncementId {
        /// Authenticate the predecessor metadata commitment.
        ///
        /// This is the only place the consumed object's identity is
        /// established. Structural recognition names no predecessor program:
        /// it reads the asset, the explicit amount and the script version, and
        /// discards the program itself. Identity comes from this component's
        /// equation instead, which introspects that program and verifies the
        /// tweak against the internal key and the root derived from the
        /// authenticated metadata, so the program is bound to the recipe that
        /// commits to it rather than to a constant. Support for an object
        /// built under a different constructor recipe would therefore have to
        /// enter here, as an authentication against that recipe, because no
        /// other component reads the predecessor's construction at all. No
        /// such object can be reached today, because §17.3 admits no
        /// migration between static subtrees and the guide's own status
        /// leaves that migration outstanding, which is where such an object
        /// would first come from.
        MetadataAuthentication,
        /// Require the unannounced maturity state.
        MaturityPredecessor,
        /// Enforce the inclusive lead window without unsigned overflow.
        LeadWindow,
        /// Derive the successor bytes from the semantic schema.
        CopyThrough,
        /// Authenticate output zero's constructor, asset and amount.
        SuccessorReconstruction,
    }
}

census_enum! {
    /// Semantic responsibilities of announcement components.
    pub enum StateAnnouncementOwner {
        /// Predecessor constructor binding.
        PredecessorMetadata,
        /// Maturity transition eligibility.
        UnannouncedMaturity,
        /// Architecture-owned announcement bounds.
        AnnouncementWindow,
        /// Preservation of unaffected semantic fields.
        MetadataTransition,
        /// Successor constructor binding.
        SuccessorMetadata,
    }
}

census_enum! {
    /// Exact consumers, never deployment definitions.
    pub enum StateAnnouncementSymbol {
        /// The x-only key in the constructor's internal-key-policy reference.
        InternalKey,
        /// `MATURITY_LEAD_MIN`, as an unsigned little-endian eight-byte item.
        MaturityLeadMin,
        /// `MATURITY_LEAD_MAX`, in the same encoding as the minimum.
        MaturityLeadMax,
        /// The same future linker reference as `StatePatternSymbol::StateAsset`.
        StateAsset,
        /// The same future linker reference as `StatePatternSymbol::StateAmount`.
        StateAmount,
        /// Constant metadata header, consumed only by the variable schedule.
        ///
        /// The census follows the schedule because a symbol consumed by no
        /// site would leave a defined link key nothing reads; the link refuses
        /// such a definition by design.
        MetadataHeader,
    }
}

impl StateAnnouncementSymbol {
    /// Structural consumer shared by the two recognition directions.
    #[must_use]
    pub const fn structural(self) -> Option<StatePatternSymbol> {
        match self {
            Self::StateAsset => Some(StatePatternSymbol::StateAsset),
            Self::StateAmount => Some(StatePatternSymbol::StateAmount),
            _ => None,
        }
    }
}

census_enum! {
    /// Witness and authenticated intermediate roles, deepest first in schedules.
    pub enum StateAnnouncementWitness {
        /// The static subtree root authenticated against the consumed program.
        StaticSubtreeRoot,
        /// Canonical predecessor bytes, authenticated by its constructor.
        PredecessorMetadata,
        /// Requested cycle in the codec's big-endian eight-byte encoding.
        RequestedCycle,
        /// Big-endian four-byte representation nonce for the successor.
        SuccessorNonce,
        /// One compressed-key prefix byte, checked by tweak verification.
        OutputKeyPrefix,
        /// Derived successor metadata, never an independent free witness.
        DerivedSuccessorMetadata,
    }
}

/// Refusals before an identity can be admitted.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum StateAnnouncementRefusal {
    /// Emission or abstract execution failed.
    #[error(transparent)]
    Program(#[from] TapscriptError),
    /// Every consumer must occur exactly in the supplied binding census.
    #[error("missing or unused announcement consumer")]
    ConsumerCensus,
    /// A substitution has an invalid width or domain.
    #[error("invalid announcement consumer: {0:?}")]
    InvalidBinding(StateAnnouncementSymbol),
    /// Changed instructions cannot inherit an admitted identity.
    #[error("announcement fragment differs from its recipe")]
    FragmentMismatch,
    /// The walk did not establish exactly the declared postcondition.
    #[error("announcement stack contract was not established")]
    InvalidContract,
    /// All five compatible components are required in canonical order.
    #[error("incomplete or incompatible announcement recipe")]
    ComponentRecipe,
}

/// Checked substitutions that still require linker resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateAnnouncementBindings {
    schedule: StateWitnessSchedule,
    values: BTreeMap<StateAnnouncementSymbol, StackItem>,
}

impl StateAnnouncementBindings {
    /// Check the exact census, widths, singleton amount and ordered lead bounds.
    ///
    /// # Errors
    /// Refuses missing, unused or malformed substitutions.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        schedule: StateWitnessSchedule,
        values: BTreeMap<StateAnnouncementSymbol, StackItem>,
    ) -> Result<Self, StateAnnouncementRefusal> {
        use StateAnnouncementSymbol as S;
        let expected: BTreeSet<_> = S::ALL
            .iter()
            .copied()
            .filter(|symbol| {
                schedule == StateWitnessSchedule::VariableMetadata || *symbol != S::MetadataHeader
            })
            .collect();
        if values.keys().copied().collect::<BTreeSet<_>>() != expected {
            return Err(StateAnnouncementRefusal::ConsumerCensus);
        }
        for (&symbol, item) in &values {
            let valid = match symbol {
                S::InternalKey | S::StateAsset => item.len() == 32,
                S::MaturityLeadMin | S::MaturityLeadMax => item.len() == 8,
                S::StateAmount => item.signed_le64_value(target).is_some_and(|n| n > 0),
                S::MetadataHeader => item.len() == STATE_METADATA_VARIABLE_RANGE.start,
            };
            if !valid {
                return Err(StateAnnouncementRefusal::InvalidBinding(symbol));
            }
        }
        let read = |symbol| {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(values[&symbol].bytes());
            u64::from_le_bytes(bytes)
        };
        if read(S::MaturityLeadMin) == 0 || read(S::MaturityLeadMin) > read(S::MaturityLeadMax) {
            return Err(StateAnnouncementRefusal::InvalidBinding(S::MaturityLeadMin));
        }
        Ok(Self { schedule, values })
    }

    /// Schedule whose exact symbol census these bindings satisfy.
    #[must_use]
    pub const fn schedule(&self) -> StateWitnessSchedule {
        self.schedule
    }
}

/// Unionable semantic dependency metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateAnnouncementMetadata {
    /// Authenticated inputs required by these fragments.
    pub sources: BTreeSet<RequiredSourceKind>,
    /// Reviewed primitive and encoding evidence requirements.
    pub evidence: BTreeSet<TargetEvidenceRequirementId>,
    /// Public witness roles disclosed by these components.
    pub disclosure: BTreeSet<StateAnnouncementWitness>,
    /// Obligations left outside this pre-link abstract walk.
    pub residuals: BTreeSet<StateAnnouncementResidual>,
}

census_enum! {
    /// Obligations these fragments do not claim to discharge.
    pub enum StateAnnouncementResidual {
        /// Fixture substitution does not establish deployment resolution.
        UnresolvedConsumers,
        /// Abstract contracts do not establish target-native behavior.
        TargetNativeEvidence,
        /// Authenticated construction must supply canonical predecessor metadata.
        CanonicalPredecessorConstruction,
        /// Chain evidence establishes current-root freshness.
        CurrentRootFreshness,
        /// Nonce leastness belongs to the constructor's host search.
        RepresentationNonceLeastness,
        /// The NUMS policy retains its discrete-log and preimage assumptions.
        InternalKeyAssumptions,
    }
}

/// Consumer sites grouped by semantic component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateAnnouncementConsumer {
    /// The unresolved symbol read at every recorded push.
    pub symbol: StateAnnouncementSymbol,
    /// Instruction indices local to each component.
    pub sites: BTreeMap<StateAnnouncementId, BTreeSet<usize>>,
}

/// One semantic record admitted only by its canonical walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateAnnouncementPattern {
    id: StateAnnouncementId,
    bindings: StateAnnouncementBindings,
    fragment: TapscriptProgram,
    precondition: AbstractStackState,
    success: BTreeSet<AbstractStackState>,
    nonaborting: BTreeSet<AbstractStackState>,
    aborts: BTreeSet<FailureCause>,
    prerequisites: BTreeSet<ElementsCapability>,
    resources: BTreeMap<ResourceDimension, u64>,
    metadata: StateAnnouncementMetadata,
    consumers: BTreeMap<StateAnnouncementSymbol, StateAnnouncementConsumer>,
}

/// A union over the five components, without a composed leaf.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateAnnouncementRecipe {
    schedule: StateWitnessSchedule,
    components: Vec<StateAnnouncementPattern>,
    metadata: StateAnnouncementMetadata,
    consumers: BTreeMap<StateAnnouncementSymbol, StateAnnouncementConsumer>,
}

const fn width(bytes: usize) -> StackValueType {
    StackValueType::Bytes {
        minimum: bytes,
        maximum: bytes,
    }
}

impl StateAnnouncementId {
    /// Witness schedule, with authenticated intermediates distinguished by role.
    #[must_use]
    pub fn witness(self) -> Vec<StateAnnouncementWitness> {
        use StateAnnouncementWitness as W;
        match self {
            Self::MetadataAuthentication => vec![
                W::StaticSubtreeRoot,
                W::PredecessorMetadata,
                W::OutputKeyPrefix,
            ],
            Self::MaturityPredecessor => vec![W::PredecessorMetadata],
            Self::LeadWindow => vec![W::PredecessorMetadata, W::RequestedCycle],
            Self::CopyThrough => vec![
                W::SuccessorNonce,
                W::StaticSubtreeRoot,
                W::RequestedCycle,
                W::PredecessorMetadata,
            ],
            Self::SuccessorReconstruction => vec![
                W::StaticSubtreeRoot,
                W::DerivedSuccessorMetadata,
                W::OutputKeyPrefix,
            ],
        }
    }

    /// The semantic responsibility of this component.
    #[must_use]
    pub const fn owner(self) -> StateAnnouncementOwner {
        use StateAnnouncementOwner as O;
        match self {
            Self::MetadataAuthentication => O::PredecessorMetadata,
            Self::MaturityPredecessor => O::UnannouncedMaturity,
            Self::LeadWindow => O::AnnouncementWindow,
            Self::CopyThrough => O::MetadataTransition,
            Self::SuccessorReconstruction => O::SuccessorMetadata,
        }
    }

    /// Exact starting stack for this standalone fragment.
    #[must_use]
    pub fn precondition(self, schedule: StateWitnessSchedule) -> AbstractStackState {
        use StateAnnouncementWitness as W;
        AbstractStackState::from_main(
            self.witness()
                .iter()
                .map(|role| {
                    width(match role {
                        W::PredecessorMetadata
                            if self == Self::MetadataAuthentication
                                && schedule == StateWitnessSchedule::VariableMetadata =>
                        {
                            STATE_METADATA_VARIABLE_BYTES
                        }
                        W::PredecessorMetadata | W::DerivedSuccessorMetadata => {
                            STATE_METADATA_BYTES
                        }
                        W::StaticSubtreeRoot => 32,
                        W::RequestedCycle => 8,
                        W::SuccessorNonce => 4,
                        W::OutputKeyPrefix => 1,
                    })
                })
                .collect(),
        )
    }

    fn postcondition(self, schedule: StateWitnessSchedule) -> AbstractStackState {
        if self == Self::LeadWindow {
            self.precondition(schedule)
        } else if matches!(self, Self::MetadataAuthentication | Self::CopyThrough) {
            AbstractStackState::from_main(vec![width(32), width(STATE_METADATA_BYTES)])
        } else {
            AbstractStackState::from_main(vec![width(STATE_METADATA_BYTES)])
        }
    }
}

/// Derive a field slice from the codec's typed field order and widths.
fn field_slice(wanted: StateField) -> (i64, i64) {
    let mut start = i64::try_from(STATE_METADATA_DOMAIN.len()).unwrap_or(20) + 4;
    for &field in StateField::ALL {
        let length = match field {
            StateField::Maturity => 9,
            StateField::Omega
            | StateField::YL
            | StateField::YT
            | StateField::Q
            | StateField::Cycle => 8,
        };
        if field == wanted {
            return (start, length);
        }
        start += length;
    }
    (start, 0)
}

struct Emitter<'a> {
    target: &'a ReviewedElementsTapscriptDefinition,
    bindings: &'a StateAnnouncementBindings,
    instructions: Vec<TapscriptInstruction>,
    sites: BTreeMap<StateAnnouncementSymbol, BTreeSet<usize>>,
}

impl Emitter<'_> {
    fn ops(&mut self, ids: &[OpcodeId]) {
        self.instructions.extend(ids.iter().copied().map(op));
    }

    fn raw(&mut self, bytes: &[u8]) -> Result<(), StateAnnouncementRefusal> {
        self.instructions
            .push(TapscriptInstruction::Push(StackItem::new(
                self.target,
                bytes.to_vec(),
            )?));
        Ok(())
    }

    fn num(&mut self, value: i64) -> Result<(), StateAnnouncementRefusal> {
        self.instructions.push(number(self.target, value)?);
        Ok(())
    }

    fn slice(&mut self, start: i64, length: i64) -> Result<(), StateAnnouncementRefusal> {
        self.num(start)?;
        self.num(length)?;
        self.ops(&[OpcodeId::Substring]);
        Ok(())
    }

    fn symbol(&mut self, symbol: StateAnnouncementSymbol) {
        self.sites
            .entry(symbol)
            .or_default()
            .insert(self.instructions.len());
        self.instructions.push(TapscriptInstruction::Push(
            self.bindings.values[&symbol].clone(),
        ));
    }

    fn tag(&mut self, name: &[u8]) -> Result<(), StateAnnouncementRefusal> {
        let digest = Sha256::digest(name);
        self.raw(&[digest.as_slice(), digest.as_slice()].concat())
    }

    fn hash_metadata(&mut self) -> Result<(), StateAnnouncementRefusal> {
        use OpcodeId as O;
        self.tag(b"TapLeaf/elements")?;
        self.raw(&[0xc4, 90, 0x4c, 86])?;
        self.ops(&[
            O::Concatenate,
            O::Sha256Initialize,
            O::Swap,
            O::Sha256Update,
        ]);
        self.raw(&[0x00, 0x69])?;
        self.ops(&[O::Sha256Finalize]);
        self.tag(b"TapBranch/elements")?;
        self.ops(&[O::Sha256Initialize, O::Swap, O::Sha256Update]);
        // The packed root and metadata sit directly beneath the hash context.
        self.ops(&[O::CopyOver]);
        self.slice(0, 32)?;
        self.ops(&[O::Sha256Finalize]);
        self.tag(b"TapTweak/elements")?;
        self.ops(&[O::Sha256Initialize]);
        self.symbol(StateAnnouncementSymbol::InternalKey);
        self.ops(&[O::Sha256Update, O::Swap, O::Sha256Finalize]);
        Ok(())
    }

    fn authenticate(
        &mut self,
        inspect: OpcodeId,
        retain_root: bool,
    ) -> Result<(), StateAnnouncementRefusal> {
        use OpcodeId as O;
        // Packing keeps both protected items within the registry's three-item reach.
        self.ops(&[O::Rotate, O::Rotate, O::Concatenate, O::Swap]);
        self.num(0)?;
        self.ops(&[inspect]);
        self.num(1)?;
        self.ops(&[O::EqualVerify, O::Concatenate]);
        // The witness supplies only parity; the coordinate comes from introspection.
        self.slice(0, 33)?;
        self.ops(&[O::Swap, O::Duplicate]);
        self.slice(32, i64::try_from(STATE_METADATA_BYTES).unwrap_or(86))?;
        self.hash_metadata()?;
        self.ops(&[O::Rotate, O::Swap]);
        self.symbol(StateAnnouncementSymbol::InternalKey);
        self.ops(&[O::TweakVerify]);
        if retain_root {
            self.ops(&[O::Duplicate]);
            self.slice(0, 32)?;
            self.ops(&[O::Swap]);
        }
        self.slice(32, i64::try_from(STATE_METADATA_BYTES).unwrap_or(86))
    }

    fn reverse_cycle(&mut self) -> Result<(), StateAnnouncementRefusal> {
        use OpcodeId as O;
        self.raw(&[])?;
        for offset in (0..8).rev() {
            self.ops(&[O::CopyOver]);
            self.slice(offset, 1)?;
            self.ops(&[O::Concatenate]);
        }
        self.ops(&[O::RemoveSecond]);
        self.slice(0, 8)?;
        // Bias unsigned order into signed order: x maps to x - 2^63.
        self.raw(&[0, 0, 0, 0, 0, 0, 0, 0x80])?;
        self.ops(&[O::BitwiseXor]);
        self.slice(0, 8)
    }

    fn add_lead(
        &mut self,
        symbol: StateAnnouncementSymbol,
    ) -> Result<(), StateAnnouncementRefusal> {
        use OpcodeId as O;
        self.symbol(symbol);
        self.raw(&[255, 255, 255, 255, 255, 255, 255, 127])?;
        self.ops(&[O::BitwiseAnd]);
        self.slice(0, 8)?;
        self.ops(&[O::Add64, O::Verify]);
        // Two nonnegative 2^62 contributions encode the lead's high bit.
        // Every intermediate rises monotonically; overflow is exactly u64 overflow.
        for _ in 0..2 {
            self.symbol(symbol);
            self.slice(7, 1)?;
            self.raw(&[0, 0, 0])?;
            self.ops(&[O::Concatenate]);
            self.slice(0, 4)?;
            self.ops(&[O::Le32ToLe64]);
            self.num(128)?;
            self.ops(&[O::ScriptNumToLe64, O::Div64, O::Verify, O::Swap, O::Drop]);
            self.raw(&(1_i64 << 62).to_le_bytes())?;
            self.ops(&[O::Mul64, O::Verify, O::Add64, O::Verify]);
        }
        Ok(())
    }

    fn window(&mut self) -> Result<(), StateAnnouncementRefusal> {
        use OpcodeId as O;
        for (symbol, compare) in [
            (
                StateAnnouncementSymbol::MaturityLeadMin,
                O::LessThanOrEqual64,
            ),
            (
                StateAnnouncementSymbol::MaturityLeadMax,
                O::GreaterThanOrEqual64,
            ),
        ] {
            self.ops(&[O::DuplicateTwo, O::Swap]);
            let (start, length) = field_slice(StateField::Cycle);
            self.slice(start, length)?;
            self.reverse_cycle()?;
            self.add_lead(symbol)?;
            self.ops(&[O::Swap]);
            self.reverse_cycle()?;
            self.ops(&[compare, O::Verify]);
        }
        Ok(())
    }

    /// Carry the retained root because the registry reaches only three items deep.
    fn copy_through(&mut self) -> Result<(), StateAnnouncementRefusal> {
        use OpcodeId as O;
        let (start, _) = field_slice(StateField::Maturity);
        self.slice(0, start)?;
        self.ops(&[O::Swap]);
        self.raw(&[1])?;
        self.ops(&[
            O::Swap,
            O::Concatenate,
            O::Concatenate,
            O::Rotate,
            O::Concatenate,
        ]);
        self.raw(&[0; 8])?;
        self.ops(&[O::Concatenate]);
        self.slice(0, i64::try_from(STATE_METADATA_BYTES).unwrap_or(86))
    }

    fn successor(&mut self) -> Result<(), StateAnnouncementRefusal> {
        for (inspect, encoding, symbol) in [
            (
                OpcodeId::InspectOutputAsset,
                EncodingClass::ExplicitAsset,
                StateAnnouncementSymbol::StateAsset,
            ),
            (
                OpcodeId::InspectOutputValue,
                EncodingClass::ExplicitValue,
                StateAnnouncementSymbol::StateAmount,
            ),
        ] {
            self.num(0)?;
            self.ops(&[inspect]);
            self.instructions
                .extend(require_explicit(self.target, encoding)?);
            self.symbol(symbol);
            self.ops(&[OpcodeId::EqualVerify]);
        }
        self.authenticate(OpcodeId::InspectOutputScriptPubKey, false)
    }
}

fn emit<'a>(
    target: &'a ReviewedElementsTapscriptDefinition,
    bindings: &'a StateAnnouncementBindings,
    id: StateAnnouncementId,
) -> Result<Emitter<'a>, StateAnnouncementRefusal> {
    let mut emitter = Emitter {
        target,
        bindings,
        instructions: Vec::new(),
        sites: BTreeMap::new(),
    };
    match id {
        StateAnnouncementId::MetadataAuthentication => {
            if bindings.schedule() == StateWitnessSchedule::VariableMetadata {
                use OpcodeId as O;
                emitter.ops(&[O::Swap]);
                emitter.symbol(StateAnnouncementSymbol::MetadataHeader);
                emitter.ops(&[O::Swap, O::Concatenate]);
                emitter.raw(&[0; 8])?;
                emitter.ops(&[O::Concatenate, O::Swap]);
            }
            emitter.authenticate(OpcodeId::InspectInputScriptPubKey, true)?;
        }
        StateAnnouncementId::MaturityPredecessor => {
            emitter.ops(&[OpcodeId::Duplicate]);
            emitter.slice(field_slice(StateField::Maturity).0, 1)?;
            emitter.raw(&[0])?;
            emitter.ops(&[OpcodeId::EqualVerify]);
        }
        StateAnnouncementId::LeadWindow => emitter.window()?,
        StateAnnouncementId::CopyThrough => emitter.copy_through()?,
        StateAnnouncementId::SuccessorReconstruction => emitter.successor()?,
    }
    Ok(emitter)
}

/// Emit a semantic fragment using checked unresolved consumers.
///
/// # Errors
/// Propagates instruction, literal and resource admission failures.
pub fn state_announcement_fragment(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StateAnnouncementBindings,
    id: StateAnnouncementId,
) -> Result<TapscriptProgram, StateAnnouncementRefusal> {
    Ok(TapscriptProgram::new(
        emit(target, bindings, id)?.instructions,
    )?)
}

fn metadata(
    target: &ReviewedElementsTapscriptDefinition,
    id: StateAnnouncementId,
    fragment: &TapscriptProgram,
) -> StateAnnouncementMetadata {
    use RequiredSourceKind as S;
    use StateAnnouncementResidual as R;
    use TargetEvidenceRequirementId as E;
    let mut result = StateAnnouncementMetadata {
        sources: BTreeSet::from([S::AuthenticatedInputObject, S::PublicConstructionData]),
        evidence: BTreeSet::from([
            E::TapscriptExecutionDomain,
            E::LeafVersionActivation,
            E::PushEncodingSemantics,
            E::EncodingSemantics,
            E::ConsensusResourceLimits,
        ]),
        disclosure: id.witness().into_iter().collect(),
        residuals: BTreeSet::from([
            R::UnresolvedConsumers,
            R::TargetNativeEvidence,
            R::CanonicalPredecessorConstruction,
            R::CurrentRootFreshness,
            R::RepresentationNonceLeastness,
            R::InternalKeyAssumptions,
        ]),
    };
    if id == StateAnnouncementId::LeadWindow {
        result.sources.insert(S::RuntimeArchitectureBound);
    }
    if id == StateAnnouncementId::SuccessorReconstruction {
        result.sources.insert(S::AuthenticatedOutputObject);
    }
    for instruction in fragment.instructions() {
        if let TapscriptInstruction::Opcode(opcode) = instruction {
            result
                .evidence
                .extend(target.definition().opcodes()[opcode].evidence());
        }
    }
    result
}

/// Admit a record only after checking the canonical fragment and walking it.
///
/// # Errors
/// Refuses changed instructions or a walk with a different postcondition.
pub fn build_state_announcement_pattern(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StateAnnouncementBindings,
    id: StateAnnouncementId,
    fragment: TapscriptProgram,
) -> Result<StateAnnouncementPattern, StateAnnouncementRefusal> {
    let emitted = emit(target, bindings, id)?;
    if fragment.instructions() != emitted.instructions {
        return Err(StateAnnouncementRefusal::FragmentMismatch);
    }
    let precondition = id.precondition(bindings.schedule());
    let execution = validate_program(
        target,
        &fragment,
        &precondition,
        AbstractLimits::for_target(target),
    )?;
    if execution.success() != &BTreeSet::from([id.postcondition(bindings.schedule())])
        || !execution.nonaborting_failure().is_empty()
        || !execution.signature_forms().is_empty()
    {
        return Err(StateAnnouncementRefusal::InvalidContract);
    }
    let consumers = emitted
        .sites
        .into_iter()
        .map(|(symbol, sites)| {
            (
                symbol,
                StateAnnouncementConsumer {
                    symbol,
                    sites: BTreeMap::from([(id, sites)]),
                },
            )
        })
        .collect();
    Ok(StateAnnouncementPattern {
        id,
        bindings: bindings.clone(),
        precondition,
        consumers,
        success: execution.success().clone(),
        nonaborting: execution.nonaborting_failure().clone(),
        aborts: execution.aborts().clone(),
        prerequisites: fragment_prerequisites(&fragment),
        resources: resource_projection(target, &fragment),
        metadata: metadata(target, id, &fragment),
        fragment,
    })
}

impl StateAnnouncementRecipe {
    /// Union exactly five compatible records in semantic order.
    ///
    /// # Errors
    /// Refuses a missing, reordered or differently substituted component.
    pub fn new(
        components: Vec<StateAnnouncementPattern>,
    ) -> Result<Self, StateAnnouncementRefusal> {
        if components.iter().map(|c| c.id).collect::<Vec<_>>() != StateAnnouncementId::ALL
            || components
                .windows(2)
                .any(|pair| pair[0].bindings != pair[1].bindings)
        {
            return Err(StateAnnouncementRefusal::ComponentRecipe);
        }
        let schedule = components[0].bindings.schedule();
        let mut metadata = StateAnnouncementMetadata::default();
        let mut consumers: BTreeMap<_, StateAnnouncementConsumer> = BTreeMap::new();
        for component in &components {
            metadata.sources.extend(&component.metadata.sources);
            metadata.evidence.extend(&component.metadata.evidence);
            metadata.disclosure.extend(&component.metadata.disclosure);
            metadata.residuals.extend(&component.metadata.residuals);
            for (&symbol, requirement) in &component.consumers {
                consumers
                    .entry(symbol)
                    .or_insert_with(|| StateAnnouncementConsumer {
                        symbol,
                        sites: BTreeMap::new(),
                    })
                    .sites
                    .extend(requirement.sites.clone());
            }
        }
        Ok(Self {
            schedule,
            components,
            metadata,
            consumers,
        })
    }
}

/// Walk the five fragments and union their dependency records.
///
/// # Errors
/// Propagates the first emission, execution or recipe refusal.
pub fn state_announcement_patterns(
    target: &ReviewedElementsTapscriptDefinition,
    bindings: &StateAnnouncementBindings,
) -> Result<StateAnnouncementRecipe, StateAnnouncementRefusal> {
    StateAnnouncementRecipe::new(
        StateAnnouncementId::ALL
            .iter()
            .map(|&id| {
                let fragment = state_announcement_fragment(target, bindings, id)?;
                build_state_announcement_pattern(target, bindings, id, fragment)
            })
            .collect::<Result<_, _>>()?,
    )
}

impl StateAnnouncementPattern {
    /// Admitted semantic identity.
    #[must_use]
    pub const fn id(&self) -> &StateAnnouncementId {
        &self.id
    }

    /// Canonical instructions.
    #[must_use]
    pub const fn fragment(&self) -> &TapscriptProgram {
        &self.fragment
    }

    /// Declared starting stack.
    #[must_use]
    pub const fn precondition(&self) -> &AbstractStackState {
        &self.precondition
    }

    /// Successful stack states.
    #[must_use]
    pub const fn success(&self) -> &BTreeSet<AbstractStackState> {
        &self.success
    }

    /// Non-aborting failure states.
    #[must_use]
    pub const fn nonaborting_failure(&self) -> &BTreeSet<AbstractStackState> {
        &self.nonaborting
    }

    /// Named abort causes.
    #[must_use]
    pub const fn aborts(&self) -> &BTreeSet<FailureCause> {
        &self.aborts
    }

    /// Capabilities derived from instructions.
    #[must_use]
    pub const fn prerequisites(&self) -> &BTreeSet<ElementsCapability> {
        &self.prerequisites
    }

    /// Projected target resources.
    #[must_use]
    pub const fn resources(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.resources
    }

    /// Component dependency metadata.
    #[must_use]
    pub const fn metadata(&self) -> &StateAnnouncementMetadata {
        &self.metadata
    }

    /// Unresolved consumer sites.
    #[must_use]
    pub const fn consumers(&self) -> &BTreeMap<StateAnnouncementSymbol, StateAnnouncementConsumer> {
        &self.consumers
    }
}

impl StateAnnouncementRecipe {
    /// Witness schedule shared by all five component bindings.
    #[must_use]
    pub const fn schedule(&self) -> StateWitnessSchedule {
        self.schedule
    }

    /// Ordered component records.
    #[must_use]
    pub const fn components(&self) -> &Vec<StateAnnouncementPattern> {
        &self.components
    }

    /// Unioned dependency metadata.
    #[must_use]
    pub const fn metadata(&self) -> &StateAnnouncementMetadata {
        &self.metadata
    }

    /// Unioned consumer sites.
    #[must_use]
    pub const fn consumers(&self) -> &BTreeMap<StateAnnouncementSymbol, StateAnnouncementConsumer> {
        &self.consumers
    }
}

impl StateAnnouncementPattern {
    /// Construction still requires authenticated consumer resolution.
    #[must_use]
    pub const fn constructibility(&self) -> StatePatternConstructibility {
        StatePatternConstructibility::CheckedUnresolvedConsumers
    }
}
