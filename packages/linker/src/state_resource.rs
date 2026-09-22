//! What the linked maturity leaf costs, measured rather than inherited.
//!
//! # Checked arithmetic, not the record's saturating projection
//!
//! The composed record carries a three-dimensional projection whose
//! totals saturate, and its own documentation calls them diagnostic. A
//! saturated total is visibly pinned, which is the right behaviour for a
//! figure nobody is closing over — but it cannot establish closure,
//! because a program whose cost overflowed reports exactly the figure a
//! program that did not would report. So the totals here are computed
//! again over the linked program with checked arithmetic, and the
//! record's projection stands beside them as the diagnostic it is. The
//! two are compared rather than assumed to agree: for every dimension
//! the projection carries, the checked total must equal it or the
//! projection must be pinned at the saturating maximum.
//!
//! # Exact tables, and no shape axis to index them by
//!
//! The live generation fits an affine model over a shape's counts
//! because its programs come one per shape. This leaf carries no count
//! bound at all, so there is no admitted shape set to index a table by
//! and nothing for a model to be fitted to. The tables here are exact
//! figures for the one linked program, and a coefficient invented for an
//! axis the leaf does not have would describe programs nobody emits.
//!
//! # The schedule and the final-stack walk stand beside the totals
//!
//! A resource figure says what a program costs, not whether it runs. The
//! walk from the record's own precondition and the final-stack rule are
//! therefore reported with the totals rather than in place of them, and
//! a linked program that fails the final-stack rule is refused here even
//! though its totals are perfectly computable.
//!
//! # What the totals do not cover is named rather than covered
//!
//! The native observations are derived from the linked program's own
//! primitives through the prerequisite adapter, and that adapter assigns
//! no capability to the streaming hash primitives, while the reviewed
//! contract's figures for them are per-opcode constants. Both are
//! visible facts of the emitters this module reads, and both would be
//! easy to paper over — by adding a capability the adapter does not
//! assign, or by charging a hash by its message length the contract does
//! not charge it by. Either would be inventing a measurement, so each is
//! carried as a named gap instead, recomputed from the emitter it comes
//! from.
//!
//! # Initial arguments and execution elements have different bounds
//!
//! Relay judges the seven initial arguments before the program runs. The
//! policy bound on those arguments is measured from the record's starting
//! stack beside the linked program's walk. The execution element bound is
//! enforced by the item constructor and is not a total here: the leaf's
//! 86- and 118-byte intermediates are legal execution elements. One
//! greatest-width figure cannot stand for both rules.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::pattern::fragment_prerequisites;
use tapscript::{
    AbstractExecutionResult, AbstractLimits, FinalStackDefect, StateAnnouncementProgram,
    StateProgramWitness, TapscriptInstruction, TapscriptProgram, final_stack_defects,
    program_stack_profile,
};
use target_elements::{
    ElementsCapability, OpcodeId, OpcodeResourceCost, OperandContract, PayloadWidth, ResourceBound,
    ResourceDimension, ReviewedElementsTapscriptDefinition, StackValueType,
};

use crate::state_error::StateLinkRefusal;
use crate::state_relocate::LinkedStateLeafProgram;

/// The streaming hash primitives, named once.
///
/// The three the prerequisite adapter returns no capability for, which
/// is the fact two of the gaps below are recomputed from.
const STREAMING_HASH: [OpcodeId; 3] = [
    OpcodeId::Sha256Initialize,
    OpcodeId::Sha256Update,
    OpcodeId::Sha256Finalize,
];

// --- Checked totals ----------------------------------------------------

/// The width fixed by one initial argument's declared type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitialArgumentWidth {
    /// The declaration fixes this many bytes.
    Exact(u64),
    /// The declaration does not fix one exact width.
    NoExactWidth(StackValueType),
}

/// The policy statement used for initial-argument admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitialArgumentBound {
    /// The reviewed policy states this bound.
    Stated(ResourceBound),
    /// The reviewed policy states no bound for this dimension.
    NoBoundStated,
}

/// The result of comparing one initial argument with policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitialArgumentOutcome {
    /// The exact width satisfies the stated bound.
    WithinBound,
    /// The exact width exceeds the stated maximum.
    OverBound,
    /// The declaration fixes no exact width to compare.
    NoExactWidth,
    /// There is no stated policy bound to compare against.
    NoBoundStated,
}

/// Whether every initial argument has established relay admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitialArgumentVerdict {
    /// Every initial argument has an exact admitted width.
    Admitted,
    /// At least one initial argument has not established admission.
    Refused,
}

/// One initial argument in the record's deepest-first order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitialArgumentAssessment {
    position: usize,
    role: StateProgramWitness,
    width: InitialArgumentWidth,
    outcome: InitialArgumentOutcome,
}

impl InitialArgumentAssessment {
    /// The argument's deepest-first position.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// The record's name for this argument.
    #[must_use]
    pub const fn role(&self) -> StateProgramWitness {
        self.role
    }

    /// The width its declared type fixes, or the type that fixes none.
    #[must_use]
    pub const fn width(&self) -> &InitialArgumentWidth {
        &self.width
    }

    /// The argument's comparison with policy.
    #[must_use]
    pub const fn outcome(&self) -> InitialArgumentOutcome {
        self.outcome
    }
}

/// Policy admission of every initial argument of one linked program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateInitialArgumentAdmission {
    bound: InitialArgumentBound,
    arguments: Vec<InitialArgumentAssessment>,
    over_bound_positions: BTreeSet<usize>,
    verdict: InitialArgumentVerdict,
}

impl StateInitialArgumentAdmission {
    /// The policy statement compared with the arguments.
    #[must_use]
    pub const fn bound(&self) -> InitialArgumentBound {
        self.bound
    }

    /// Every argument, deepest first, including non-exact declarations.
    #[must_use]
    pub fn arguments(&self) -> &[InitialArgumentAssessment] {
        &self.arguments
    }

    /// Positions whose exact widths exceed a stated maximum.
    #[must_use]
    pub const fn over_bound_positions(&self) -> &BTreeSet<usize> {
        &self.over_bound_positions
    }

    /// Whether every initial argument establishes admission.
    #[must_use]
    pub const fn verdict(&self) -> InitialArgumentVerdict {
        self.verdict
    }
}

/// Compare initial arguments in deepest-first order with one stated bound.
///
/// Equality with a maximum is admitted. An unbounded statement admits
/// every exact width. A declaration without one exact width never
/// establishes admission, including under an unbounded statement.
#[must_use]
pub fn compare_initial_argument_widths(
    arguments: &[(StateProgramWitness, InitialArgumentWidth)],
    bound: ResourceBound,
) -> StateInitialArgumentAdmission {
    compare_initial_arguments(arguments, InitialArgumentBound::Stated(bound))
}

/// Compare against either a stated policy bound or an explicit absence.
#[must_use]
pub(crate) fn compare_initial_arguments(
    arguments: &[(StateProgramWitness, InitialArgumentWidth)],
    bound: InitialArgumentBound,
) -> StateInitialArgumentAdmission {
    let arguments: Vec<_> = arguments
        .iter()
        .enumerate()
        .map(|(position, (role, width))| {
            let outcome = match (width, bound) {
                (InitialArgumentWidth::NoExactWidth(_), _) => InitialArgumentOutcome::NoExactWidth,
                (InitialArgumentWidth::Exact(_), InitialArgumentBound::NoBoundStated) => {
                    InitialArgumentOutcome::NoBoundStated
                }
                (
                    InitialArgumentWidth::Exact(width),
                    InitialArgumentBound::Stated(ResourceBound::Maximum(maximum)),
                ) if *width > maximum => InitialArgumentOutcome::OverBound,
                (InitialArgumentWidth::Exact(_), InitialArgumentBound::Stated(_)) => {
                    InitialArgumentOutcome::WithinBound
                }
            };
            InitialArgumentAssessment {
                position,
                role: *role,
                width: width.clone(),
                outcome,
            }
        })
        .collect();
    let over_bound_positions = arguments
        .iter()
        .filter(|argument| argument.outcome == InitialArgumentOutcome::OverBound)
        .map(|argument| argument.position)
        .collect();
    let verdict = if matches!(bound, InitialArgumentBound::Stated(_))
        && arguments
            .iter()
            .all(|argument| argument.outcome == InitialArgumentOutcome::WithinBound)
    {
        InitialArgumentVerdict::Admitted
    } else {
        InitialArgumentVerdict::Refused
    };
    StateInitialArgumentAdmission {
        bound,
        arguments,
        over_bound_positions,
        verdict,
    }
}

/// Read the same exact declaration forms that the witness ABI reads.
fn declared_initial_width(declared: &StackValueType) -> InitialArgumentWidth {
    let width = match declared {
        StackValueType::Bytes { minimum, maximum } if minimum == maximum => Some(*minimum),
        StackValueType::Encoded(class) => match class.v1_shape().payload() {
            PayloadWidth::Absent => Some(0),
            PayloadWidth::Exact(width) => Some(width.get()),
            PayloadWidth::Bounded { minimum, maximum } if minimum == maximum.get() => Some(minimum),
            PayloadWidth::Bounded { .. } => None,
        },
        _ => None,
    };
    width
        .and_then(|width| u64::try_from(width).ok())
        .map_or_else(
            || InitialArgumentWidth::NoExactWidth(declared.clone()),
            InitialArgumentWidth::Exact,
        )
}

/// Pair the walked starting stack with the record's role order.
fn initial_arguments(
    record: &StateAnnouncementProgram,
) -> Vec<(StateProgramWitness, InitialArgumentWidth)> {
    record
        .witness()
        .iter()
        .zip(record.precondition().main())
        .map(|((role, _), declared)| (*role, declared_initial_width(declared)))
        .collect()
}

/// Every dimension this module measures over one linked program.
///
/// The greatest initial-argument width is present only when every
/// initial argument declares one exact width. A non-exact declaration
/// leaves that dimension absent rather than supplying an invented peak.
/// [`ResourceDimension::InitialWitnessItemBytes`] names that policy
/// measurement separately from the execution element bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateLinkedResourceTotals {
    totals: BTreeMap<ResourceDimension, u64>,
    instructions: u64,
}

impl StateLinkedResourceTotals {
    /// One dimension's checked total, where this module measures it.
    #[must_use]
    pub fn total(&self, dimension: ResourceDimension) -> Option<u64> {
        self.totals.get(&dimension).copied()
    }

    /// Every measured dimension, in canonical order.
    #[must_use]
    pub const fn totals(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.totals
    }

    /// How many instructions the linked program carries.
    #[must_use]
    pub const fn instructions(&self) -> u64 {
        self.instructions
    }
}

/// Measure the linked program in every dimension a program fixes.
///
/// # Which dimensions are measured, and why the rest are not
///
/// Six of the roster's eleven are properties of this program and the
/// witness it declares, and each is computed from the reviewed
/// contract's own figures: the script bytes are the encoder's own count,
/// the operation cost and the validation budget are checked sums of the
/// per-primitive figures, the initial stack is the declared witness's
/// depth, the greatest initial-argument width is read from the same
/// precondition the linked program walks from, and the peak stack is
/// the abstract walk's own profile. The width total is present only
/// when each argument declares an exact width.
///
/// The other five are absent because no program fixes them, and saying
/// so is the point of naming them. The transaction weight and the
/// package limit are figures about a transaction and a relay decision,
/// not about a leaf. The witness bytes are the spend's, and the record
/// declares its witness as roles and width ranges rather than bytes. The
/// stack element width is a per-item bound the item constructor enforces
/// at construction rather than a total anything sums. The control path
/// depth is a property of the tree the leaf is committed in, which the
/// deployment caps and the tree bite measures.
///
/// # Errors
///
/// [`StateLinkRefusal::ResourceTotalOverflow`] when one dimension's
/// checked sum does not fit.
pub fn measure_state_totals(
    target: &ReviewedElementsTapscriptDefinition,
    record: &StateAnnouncementProgram,
    program: &TapscriptProgram,
) -> Result<StateLinkedResourceTotals, StateLinkRefusal> {
    let mut operation_cost: u64 = 0;
    let mut validation_budget: u64 = 0;

    for instruction in program.instructions() {
        let TapscriptInstruction::Opcode(id) = instruction else {
            continue;
        };
        let cost = primitive_cost(target, *id);
        operation_cost = checked(
            operation_cost,
            cost.operation_cost(),
            ResourceDimension::OperationCost,
        )?;
        validation_budget = checked(
            validation_budget,
            cost.validation_budget(),
            ResourceDimension::ValidationBudget,
        )?;
    }

    let profile = program_stack_profile(
        target,
        program,
        record.precondition(),
        AbstractLimits::for_target(target),
    );
    let peak = checked(
        profile.peak_main(),
        profile.peak_alternate(),
        ResourceDimension::PeakStackItems,
    )?;

    // Two width conversions, not the saturating arithmetic this module
    // refuses: an instruction count is bounded by the program's own
    // limit and a declared witness by the walk's depth bound, so neither
    // can reach the fallback on any host this builds for. The form is
    // the program's own.
    let instructions = u64::try_from(program.len()).unwrap_or(u64::MAX);
    let initial = u64::try_from(record.precondition().depth()).unwrap_or(u64::MAX);
    let arguments = initial_arguments(record);
    let greatest_initial_width = arguments
        .iter()
        .map(|(_, width)| match width {
            InitialArgumentWidth::Exact(width) => Some(*width),
            InitialArgumentWidth::NoExactWidth(_) => None,
        })
        .collect::<Option<Vec<_>>>()
        .and_then(|widths| widths.into_iter().max());
    let mut totals = BTreeMap::from([
        (
            ResourceDimension::ScriptBytes,
            program.encoded_length(target),
        ),
        (ResourceDimension::OperationCost, operation_cost),
        (ResourceDimension::ValidationBudget, validation_budget),
        (ResourceDimension::InitialStackItems, initial),
        (ResourceDimension::PeakStackItems, peak),
    ]);
    if let Some(width) = greatest_initial_width {
        totals.insert(ResourceDimension::InitialWitnessItemBytes, width);
    }

    Ok(StateLinkedResourceTotals {
        totals,
        instructions,
    })
}

/// One checked addition, named by the dimension it is a total of.
fn checked(total: u64, units: u64, dimension: ResourceDimension) -> Result<u64, StateLinkRefusal> {
    total
        .checked_add(units)
        .ok_or(StateLinkRefusal::ResourceTotalOverflow { dimension })
}

/// The reviewed contract's cost of one primitive.
///
/// # Panics
///
/// Panics only if the reviewed contract states no contract for a
/// primitive the program carries, which the target validator's own
/// completeness check cannot arrange: a definition omitting one is
/// refused before it becomes a reviewed contract.
fn primitive_cost(
    target: &ReviewedElementsTapscriptDefinition,
    id: OpcodeId,
) -> OpcodeResourceCost {
    target
        .definition()
        .opcodes()
        .get(&id)
        .expect("the reviewed contract states a contract for every primitive")
        .resources()
}

// --- Native observations -----------------------------------------------

/// What the linked program's own primitives observe, and what they do
/// not.
///
/// Absent is stated against the reviewed contract's complete capability
/// census rather than left implicit, because an unlisted capability and
/// an unreachable one read the same in a set that carries only what is
/// present.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateNativeObservations {
    present: BTreeSet<ElementsCapability>,
    absent: BTreeSet<ElementsCapability>,
}

impl StateNativeObservations {
    /// Every capability some primitive of the linked program needs.
    #[must_use]
    pub const fn present(&self) -> &BTreeSet<ElementsCapability> {
        &self.present
    }

    /// Every capability of the census none of them maps to.
    #[must_use]
    pub const fn absent(&self) -> &BTreeSet<ElementsCapability> {
        &self.absent
    }
}

/// Read the observations off the linked program.
fn observe(program: &TapscriptProgram) -> StateNativeObservations {
    let present = fragment_prerequisites(program);
    let absent = ElementsCapability::ALL
        .iter()
        .copied()
        .filter(|capability| !present.contains(capability))
        .collect();

    StateNativeObservations { present, absent }
}

// --- What the measurement does not establish ---------------------------

/// What the totals and the observations leave open.
///
/// Closed, and each variant a fact recomputed from the emitter it comes
/// from rather than a caution written down. A gap the tree does not show
/// would be this module inventing a limitation, exactly as a capability
/// the adapter does not assign would be it inventing a measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateResourceGap {
    /// The prerequisite adapter maps the streaming hash primitives to no
    /// capability at all, so the leaf schedules a hash whose capability
    /// the observations report as absent. An absent capability is
    /// therefore not evidence that the program does not need it.
    Sha256PrimitivesCarryNoCapability,

    /// The domains the leaf separates its hashes by arrive as pushed
    /// literals, and the observations are derived from primitives alone.
    /// No observation here names a hash domain, so nothing in this
    /// measurement says which domain a digest was taken in.
    HashDomainSeparationIsNotObserved,

    /// The reviewed contract states the streaming hash primitives' cost
    /// as a per-opcode constant, so the totals charge the same for a
    /// hash over a long message as for one over a short message. The
    /// figures are the target's; a length-dependent charge would be a
    /// measurement this module made up.
    StreamingHashCostIsPerOpcodeNotPerByte,

    /// A primitive that charges the validation budget charges it only on
    /// the paths its operand contract admits a charge on — the
    /// signature positions admit the empty item, which verifies nothing
    /// and costs nothing. The checked total is therefore the greatest
    /// the linked program can charge rather than what a given spend
    /// does.
    ValidationBudgetIsAPerPrimitiveMaximum,
}

/// Which gaps this linked program actually stands in.
///
/// Recomputed from the program and the reviewed contract, so a leaf that
/// stopped hashing would stop carrying the hash gaps rather than carry
/// them out of habit.
fn gaps(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> BTreeSet<StateResourceGap> {
    let mut gaps = BTreeSet::new();
    let hashes: Vec<OpcodeId> = STREAMING_HASH
        .into_iter()
        .filter(|id| schedules(program, *id))
        .collect();

    if !hashes.is_empty() {
        gaps.insert(StateResourceGap::Sha256PrimitivesCarryNoCapability);
        gaps.insert(StateResourceGap::HashDomainSeparationIsNotObserved);
    }
    if hashes.iter().any(|id| {
        let cost = primitive_cost(target, *id);
        cost.operation_cost() == 0 && cost.validation_budget() == 0
    }) {
        gaps.insert(StateResourceGap::StreamingHashCostIsPerOpcodeNotPerByte);
    }
    if program.instructions().iter().any(|instruction| {
        let TapscriptInstruction::Opcode(id) = instruction else {
            return false;
        };
        primitive_cost(target, *id).validation_budget() > 0
            && admits_an_empty_signature(target, *id)
    }) {
        gaps.insert(StateResourceGap::ValidationBudgetIsAPerPrimitiveMaximum);
    }

    gaps
}

/// Whether the program schedules one primitive.
fn schedules(program: &TapscriptProgram, id: OpcodeId) -> bool {
    program
        .instructions()
        .iter()
        .any(|instruction| matches!(instruction, TapscriptInstruction::Opcode(scheduled) if *scheduled == id))
}

/// Whether one primitive's signature position admits the empty item.
///
/// # Panics
///
/// Panics only under [`primitive_cost`]'s own condition, which the
/// reviewed contract's completeness excludes.
fn admits_an_empty_signature(target: &ReviewedElementsTapscriptDefinition, id: OpcodeId) -> bool {
    target
        .definition()
        .opcodes()
        .get(&id)
        .expect("the reviewed contract states a contract for every primitive")
        .stack()
        .operands()
        .iter()
        .any(|operand| {
            matches!(
                operand,
                OperandContract::Signature {
                    empty_allowed: true,
                    ..
                }
            )
        })
}

// --- The measurement ---------------------------------------------------

/// Everything this module establishes about one linked leaf.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateLinkedResources {
    totals: StateLinkedResourceTotals,
    initial_argument_admission: Box<StateInitialArgumentAdmission>,
    schedule: AbstractExecutionResult,
    final_stack: Vec<FinalStackDefect>,
    observations: StateNativeObservations,
    gaps: BTreeSet<StateResourceGap>,
    diagnostic: BTreeMap<ResourceDimension, u64>,
}

impl StateLinkedResources {
    /// The checked totals.
    #[must_use]
    pub const fn totals(&self) -> &StateLinkedResourceTotals {
        &self.totals
    }

    /// Each initial argument's admission under the reviewed relay policy.
    #[must_use]
    pub const fn initial_argument_admission(&self) -> &StateInitialArgumentAdmission {
        &self.initial_argument_admission
    }

    /// The walk the linked program was admitted by.
    #[must_use]
    pub const fn schedule(&self) -> &AbstractExecutionResult {
        &self.schedule
    }

    /// What the final-stack walk found, which an admitted program leaves
    /// empty.
    ///
    /// Retained although a nonempty walk refuses, so that a reader sees
    /// the walk ran rather than inferring it from the absence of a
    /// refusal.
    #[must_use]
    pub fn final_stack(&self) -> &[FinalStackDefect] {
        &self.final_stack
    }

    /// What the linked program observes natively, and what it does not.
    #[must_use]
    pub const fn observations(&self) -> &StateNativeObservations {
        &self.observations
    }

    /// What this measurement does not establish.
    #[must_use]
    pub const fn gaps(&self) -> &BTreeSet<StateResourceGap> {
        &self.gaps
    }

    /// The record's own saturating projection, beside the checked
    /// totals.
    #[must_use]
    pub const fn diagnostic(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.diagnostic
    }
}

/// Measure one linked leaf exactly, beside its schedule and its gaps.
///
/// The comparison against the record's projection admits exactly two
/// outcomes per dimension: equal figures, or a projection pinned at the
/// saturating maximum where the checked arithmetic would have refused.
/// The tests observe the first for every dimension the record carries
/// and never the second, which is what makes the pinned case a stated
/// admission rather than a hole.
///
/// # Errors
///
/// [`StateLinkRefusal::ResourceTotalOverflow`] from the totals,
/// [`StateLinkRefusal::LinkedProgramDoesNotSchedule`] when the
/// final-stack walk cannot schedule the program,
/// [`StateLinkRefusal::LinkedProgramFailsTheFinalStackRule`] when that
/// walk finds a defect, and
/// [`StateLinkRefusal::InitialArgumentNotAdmitted`] when a non-replay
/// schedule has an initial argument policy does not admit, and
/// [`StateLinkRefusal::ResourceProjectionDisagreement`] when a checked
/// total and an unpinned diagnostic figure differ.
pub fn measure_state_resources(
    target: &ReviewedElementsTapscriptDefinition,
    record: &StateAnnouncementProgram,
    linked: &LinkedStateLeafProgram,
) -> Result<StateLinkedResources, StateLinkRefusal> {
    let program = linked.program();
    let totals = measure_state_totals(target, record, program)?;

    let final_stack = final_stack_defects(target, program, record.precondition())
        .map_err(|cause| StateLinkRefusal::LinkedProgramDoesNotSchedule { cause })?;
    if !final_stack.is_empty() {
        return Err(StateLinkRefusal::LinkedProgramFailsTheFinalStackRule {
            defects: final_stack,
        });
    }

    let bound = target
        .definition()
        .resources()
        .policy()
        .bounds()
        .get(&ResourceDimension::InitialWitnessItemBytes)
        .copied()
        .map_or(
            InitialArgumentBound::NoBoundStated,
            InitialArgumentBound::Stated,
        );
    let initial_argument_admission = compare_initial_arguments(&initial_arguments(record), bound);
    if !record.schedule().is_replay_only()
        && let Some(argument) = initial_argument_admission
            .arguments()
            .iter()
            .find(|argument| argument.outcome() != InitialArgumentOutcome::WithinBound)
    {
        return Err(StateLinkRefusal::InitialArgumentNotAdmitted {
            position: argument.position(),
            role: argument.role(),
            width: argument.width().clone(),
            bound,
        });
    }

    let diagnostic = record.resources().clone();
    for (&dimension, &projected) in &diagnostic {
        // A dimension the projection carries and this module does not
        // measure is a disagreement rather than an agreement: not
        // measured is not the same as equal.
        let measured = totals.total(dimension).unwrap_or_default();
        if measured != projected && projected != u64::MAX {
            return Err(StateLinkRefusal::ResourceProjectionDisagreement {
                dimension,
                diagnostic: projected,
                checked: measured,
            });
        }
    }

    Ok(StateLinkedResources {
        totals,
        initial_argument_admission: Box::new(initial_argument_admission),
        schedule: linked.execution().clone(),
        final_stack,
        observations: observe(program),
        gaps: gaps(target, program),
        diagnostic,
    })
}
