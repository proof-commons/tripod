//! Abstract stack validation over the reviewed primitive contracts.
//!
//! # The question this answers
//!
//! Given a typed initial stack and a typed program, what successful,
//! non-aborting failure, and aborting states can the sequence produce?
//! It is a statement about the reviewed contracts, not about a node: no
//! function here executes anything, and nothing it returns is evidence
//! that a target behaves as the contracts say.
//!
//! # Three outcomes, never one Boolean
//!
//! A successful state, a state reached by a failure that pushed a false
//! and carried on, and an abort are three different things. Their stack
//! depths differ exactly on the paths a backend has to handle safely:
//! the arithmetic primitives leave their operands in place and push a
//! false *above* them, so a caller that treated failure as "the program
//! stopped" would schedule the wrong depth precisely where it matters.
//!
//! # Alternatives are kept, never chosen
//!
//! Several primitives have more than one successful form, selected by a
//! property of the target value that was read — whether a value was
//! explicit or blinded, whether an input carried an issuance. A program
//! cannot decide that, and neither can this validator: every compatible
//! alternative is retained. Taking the first would silently commit to
//! one shape of a stack the target may produce in another.
//!
//! # Two failure causes the abstract state decides
//!
//! A cause the abstract state settles is not recorded as something the
//! target might do. Insufficient operands is decided here — the state
//! says how deep the stack is — and so is an operand of the wrong
//! width, once every operand's abstract type fixes one width. Both are
//! reported as validation failures of the program instead. Every other
//! declared cause is recorded, because nothing in the abstract state
//! rules it out.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

use target_elements::{
    EncodingClass, FailureCause, FailureOutcome, OpcodeId, OpcodeSpec, PayloadWidth, ResourceBound,
    ResourceDimension, ReviewedElementsTapscriptDefinition, StackValueType, SuccessCase,
};

use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::program::{MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptProgram};

/// One abstract stack, main and alternate.
///
/// The alternate stack is represented even though no reviewed primitive
/// touches it. Leaving it out would make its depth an untracked axis
/// that a later primitive could start moving without anything noticing.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbstractStackState {
    main: Vec<StackValueType>,
    alternate: Vec<StackValueType>,
}

impl AbstractStackState {
    /// States one abstract stack.
    ///
    /// Both vectors are deepest first, so the last element of `main` is
    /// the top of the stack.
    #[must_use]
    pub const fn new(main: Vec<StackValueType>, alternate: Vec<StackValueType>) -> Self {
        Self { main, alternate }
    }

    /// States one abstract stack with an empty alternate stack.
    #[must_use]
    pub const fn from_main(main: Vec<StackValueType>) -> Self {
        Self::new(main, Vec::new())
    }

    /// The main stack, deepest first.
    #[must_use]
    pub fn main(&self) -> &[StackValueType] {
        &self.main
    }

    /// The alternate stack, deepest first.
    #[must_use]
    pub fn alternate(&self) -> &[StackValueType] {
        &self.alternate
    }

    /// The combined depth of both stacks.
    ///
    /// Combined because the target bounds them together: a program that
    /// moved items to the alternate stack would not thereby be allowed
    /// more of them.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.main.len().saturating_add(self.alternate.len())
    }
}

/// What a program can produce.
///
/// The three sets are kept apart deliberately. A consumer that wanted
/// one answer would have to decide which of them it meant, and that
/// decision belongs to the consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbstractExecutionResult {
    success: BTreeSet<AbstractStackState>,
    nonaborting_failure: BTreeSet<AbstractStackState>,
    aborts: BTreeSet<FailureCause>,
}

impl AbstractExecutionResult {
    /// The states the program reaches without any failure on the way.
    #[must_use]
    pub const fn success(&self) -> &BTreeSet<AbstractStackState> {
        &self.success
    }

    /// The states the program reaches through at least one failure that
    /// pushed a false and carried on.
    #[must_use]
    pub const fn nonaborting_failure(&self) -> &BTreeSet<AbstractStackState> {
        &self.nonaborting_failure
    }

    /// The causes on which the program can end evaluation.
    #[must_use]
    pub const fn aborts(&self) -> &BTreeSet<FailureCause> {
        &self.aborts
    }

    /// Whether the program has no surviving state at all, so every path
    /// through it aborts.
    #[must_use]
    pub fn always_aborts(&self) -> bool {
        self.success.is_empty() && self.nonaborting_failure.is_empty()
    }
}

/// The work a validation may do before it gives up.
///
/// Exhaustion returns a typed error and no partial result: a truncated
/// state set would be indistinguishable from a complete one and would
/// understate what the program can produce.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(
    clippy::struct_field_names,
    reason = "every field is a maximum, and dropping the prefix would leave `states` and `instructions` reading as counts"
)]
pub struct AbstractLimits {
    maximum_states: NonZeroU64,
    maximum_stack_depth: u64,
    maximum_instructions: NonZeroU64,
    maximum_result_alternatives: NonZeroU64,
}

/// The states one validation may visit before the default limit gives
/// up.
const DEFAULT_MAXIMUM_STATES: u64 = 4_096;

/// The alternatives one result may carry under the default limit.
const DEFAULT_MAXIMUM_RESULT_ALTERNATIVES: u64 = 1_024;

impl AbstractLimits {
    /// The default limits for one reviewed target.
    ///
    /// The stack depth comes from the target's own consensus bound, so
    /// the validator refuses exactly what the target would. The others
    /// are first-party work bounds: the target enforces no script size
    /// in the reviewed domain, and it has no notion of an abstract state
    /// at all.
    #[must_use]
    pub fn for_target(target: &ReviewedElementsTapscriptDefinition) -> Self {
        let depth = target
            .definition()
            .resources()
            .consensus()
            .bounds()
            .get(&ResourceDimension::PeakStackItems)
            .copied()
            .and_then(ResourceBound::maximum)
            .unwrap_or(u64::MAX);

        Self {
            maximum_states: nonzero(DEFAULT_MAXIMUM_STATES),
            maximum_stack_depth: depth,
            maximum_instructions: nonzero(MAXIMUM_PROGRAM_INSTRUCTIONS),
            maximum_result_alternatives: nonzero(DEFAULT_MAXIMUM_RESULT_ALTERNATIVES),
        }
    }

    /// Narrows the state budget.
    ///
    /// Narrowing only: every setter takes the smaller of the two
    /// figures, so a caller can ask for less work than the target
    /// permits and can never ask for more.
    #[must_use]
    pub fn with_maximum_states(self, maximum: NonZeroU64) -> Self {
        Self {
            maximum_states: self.maximum_states.min(maximum),
            ..self
        }
    }

    /// Narrows the stack-depth bound.
    #[must_use]
    pub fn with_maximum_stack_depth(self, maximum: u64) -> Self {
        Self {
            maximum_stack_depth: self.maximum_stack_depth.min(maximum),
            ..self
        }
    }

    /// Narrows the instruction bound.
    #[must_use]
    pub fn with_maximum_instructions(self, maximum: NonZeroU64) -> Self {
        Self {
            maximum_instructions: self.maximum_instructions.min(maximum),
            ..self
        }
    }

    /// Narrows the result-alternative bound.
    #[must_use]
    pub fn with_maximum_result_alternatives(self, maximum: NonZeroU64) -> Self {
        Self {
            maximum_result_alternatives: self.maximum_result_alternatives.min(maximum),
            ..self
        }
    }

    /// The states one validation may visit.
    #[must_use]
    pub const fn maximum_states(self) -> u64 {
        self.maximum_states.get()
    }

    /// The greatest combined stack depth a state may reach.
    #[must_use]
    pub const fn maximum_stack_depth(self) -> u64 {
        self.maximum_stack_depth
    }

    /// The instructions one program may carry.
    #[must_use]
    pub const fn maximum_instructions(self) -> u64 {
        self.maximum_instructions.get()
    }

    /// The alternatives one result may carry.
    #[must_use]
    pub const fn maximum_result_alternatives(self) -> u64 {
        self.maximum_result_alternatives.get()
    }
}

/// A limit as a nonzero figure.
fn nonzero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap_or(NonZeroU64::MIN)
}

/// Admits one more visited state against the state budget.
///
/// Checked before the increment, following the compiler's search
/// counter: a budgeted counter can then never pass its maximum, so it
/// can never reach the point where the increment itself would wrap.
const fn admit_state(visited: &mut u64, maximum: u64) -> Result<(), u64> {
    if *visited >= maximum {
        return Err(maximum);
    }
    *visited += 1;
    Ok(())
}

/// What one instruction does to one incoming state.
#[derive(Clone, Debug, Default)]
struct Transfer {
    success: Vec<AbstractStackState>,
    nonaborting_failure: Vec<AbstractStackState>,
    aborts: BTreeSet<FailureCause>,
}

/// Validates a program against the reviewed contracts.
///
/// The instruction index in a returned diagnostic is ephemeral context
/// for a reader: it names where in this program the defect is, and it
/// is not an identity, not stable across an edit, and carried in no
/// projection.
///
/// # Errors
///
/// [`TapscriptError::StackUnderflow`] and
/// [`TapscriptError::StackTypeMismatch`] when an instruction cannot be
/// applied to a state that reaches it, and
/// [`TapscriptError::InstructionLimitExceeded`],
/// [`TapscriptError::AbstractStateLimitExceeded`],
/// [`TapscriptError::StackLimitExceeded`], or
/// [`TapscriptError::ResultAlternativeLimitExceeded`] when the work
/// exceeds the configured budget, in which case there is no partial
/// result.
pub fn validate_program(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
    initial: &AbstractStackState,
    limits: AbstractLimits,
) -> Result<AbstractExecutionResult, TapscriptError> {
    let offered = u64::try_from(program.len()).unwrap_or(u64::MAX);
    if offered > limits.maximum_instructions() {
        return Err(TapscriptError::InstructionLimitExceeded {
            maximum: limits.maximum_instructions(),
        });
    }

    // A state is live together with whether it was reached through a
    // non-aborting failure. The same stack shape can be reached both
    // ways, and the two are different findings about the program.
    let mut live: BTreeSet<(AbstractStackState, bool)> = BTreeSet::new();
    live.insert((initial.clone(), false));
    let mut aborts: BTreeSet<FailureCause> = BTreeSet::new();
    let mut visited = 0_u64;
    check_depth(initial, limits)?;

    for (index, instruction) in program.instructions().iter().enumerate() {
        let mut next: BTreeSet<(AbstractStackState, bool)> = BTreeSet::new();
        for (state, failed) in &live {
            let transfer = step(target, instruction, state, index, limits)?;
            aborts.extend(transfer.aborts);
            for reached in transfer.success {
                admit(&mut next, (reached, *failed), &mut visited, limits)?;
            }
            for reached in transfer.nonaborting_failure {
                admit(&mut next, (reached, true), &mut visited, limits)?;
            }
        }
        live = next;
    }

    let mut success = BTreeSet::new();
    let mut nonaborting_failure = BTreeSet::new();
    for (state, failed) in live {
        if failed {
            nonaborting_failure.insert(state);
        } else {
            success.insert(state);
        }
    }

    for alternatives in [success.len(), nonaborting_failure.len()] {
        if u64::try_from(alternatives).unwrap_or(u64::MAX) > limits.maximum_result_alternatives() {
            return Err(TapscriptError::ResultAlternativeLimitExceeded {
                maximum: limits.maximum_result_alternatives(),
            });
        }
    }

    Ok(AbstractExecutionResult {
        success,
        nonaborting_failure,
        aborts,
    })
}

/// Records one reached state against the state budget.
fn admit(
    next: &mut BTreeSet<(AbstractStackState, bool)>,
    reached: (AbstractStackState, bool),
    visited: &mut u64,
    limits: AbstractLimits,
) -> Result<(), TapscriptError> {
    admit_state(visited, limits.maximum_states())
        .map_err(|maximum| TapscriptError::AbstractStateLimitExceeded { maximum })?;
    next.insert(reached);
    Ok(())
}

/// Refuses a state deeper than the target admits.
fn check_depth(state: &AbstractStackState, limits: AbstractLimits) -> Result<(), TapscriptError> {
    if u64::try_from(state.depth()).unwrap_or(u64::MAX) > limits.maximum_stack_depth() {
        return Err(TapscriptError::StackLimitExceeded {
            maximum: limits.maximum_stack_depth(),
        });
    }
    Ok(())
}

/// Applies one instruction to one incoming state.
fn step(
    target: &ReviewedElementsTapscriptDefinition,
    instruction: &TapscriptInstruction,
    state: &AbstractStackState,
    index: usize,
    limits: AbstractLimits,
) -> Result<Transfer, TapscriptError> {
    match instruction {
        TapscriptInstruction::Push(item) => {
            let mut main = state.main().to_vec();
            main.push(literal_type(item));
            let reached = AbstractStackState::new(main, state.alternate().to_vec());
            check_depth(&reached, limits)?;
            Ok(Transfer {
                success: vec![reached],
                ..Transfer::default()
            })
        }
        TapscriptInstruction::Opcode(id) => apply_opcode(target, *id, state, index, limits),
    }
}

/// The abstract type of one pushed literal.
const fn literal_type(item: &StackItem) -> StackValueType {
    if item.is_empty() {
        StackValueType::Empty
    } else {
        StackValueType::Bytes {
            minimum: item.len(),
            maximum: item.len(),
        }
    }
}

/// Applies one reviewed primitive to one incoming state.
fn apply_opcode(
    target: &ReviewedElementsTapscriptDefinition,
    id: OpcodeId,
    state: &AbstractStackState,
    index: usize,
    limits: AbstractLimits,
) -> Result<Transfer, TapscriptError> {
    let spec = opcode(target, id);
    let stack = spec.stack();
    let operands = stack.operands();

    if state.main().len() < operands.len() {
        return Err(TapscriptError::StackUnderflow { instruction: index });
    }

    // Operands are declared deepest first, so the declared list aligns
    // with the top of the stack.
    let base = state.main().len() - operands.len();
    let mut widths_decided = true;
    for (offset, declared) in operands.iter().enumerate() {
        let actual = &state.main()[base + offset];
        if !accepts(target, declared, actual) {
            return Err(TapscriptError::StackTypeMismatch {
                instruction: index,
                expected: declared.clone(),
                actual: actual.clone(),
            });
        }
        widths_decided &= is_exact_width(target, actual);
    }

    let mut transfer = Transfer::default();

    // Every compatible alternative is retained. The condition selecting
    // one is a property of the target value that was read, which no
    // abstract state can settle.
    for case in stack.success().cases() {
        let reached = apply_case(state, &case);
        check_depth(&reached, limits)?;
        transfer.success.push(reached);
    }

    for effect in stack.failure().effects() {
        match effect.outcome() {
            FailureOutcome::AbortEvaluation => {
                if !statically_excluded(effect.cause(), widths_decided) {
                    transfer.aborts.insert(effect.cause());
                }
            }
            FailureOutcome::ConsumeOperandsPushFalse => {
                let reached = with_false(state, base);
                check_depth(&reached, limits)?;
                transfer.nonaborting_failure.push(reached);
            }
            FailureOutcome::RetainOperandsPushFalse => {
                let reached = with_false(state, state.main().len());
                check_depth(&reached, limits)?;
                transfer.nonaborting_failure.push(reached);
            }
            // An outcome this crate has not been taught is recorded as
            // ending evaluation: it is the only outcome that claims no
            // surviving stack shape, so it cannot overstate what the
            // target leaves behind.
            _ => {
                transfer.aborts.insert(effect.cause());
            }
        }
    }

    Ok(transfer)
}

/// Whether the abstract state settles this cause by itself.
///
/// Insufficient operands is settled by the depth, which the caller has
/// already checked. An operand of the wrong width is settled once every
/// operand's abstract type fixes one width, and not otherwise.
const fn statically_excluded(cause: FailureCause, widths_decided: bool) -> bool {
    match cause {
        FailureCause::StackUnderflow => true,
        FailureCause::InvalidOperandWidth => widths_decided,
        _ => false,
    }
}

/// The state one successful form leaves behind.
fn apply_case(state: &AbstractStackState, case: &SuccessCase) -> AbstractStackState {
    let effect = case.effect();
    let mut main = state.main().to_vec();
    // Each form states how many of the declared operands it consumes,
    // which is what makes a retaining form different from a consuming
    // one with no results rather than a special case here.
    main.truncate(main.len().saturating_sub(effect.consumed_operands()));
    main.extend(effect.results().iter().cloned());
    AbstractStackState::new(main, state.alternate().to_vec())
}

/// The state a non-aborting failure leaves behind, truncated to `keep`.
fn with_false(state: &AbstractStackState, keep: usize) -> AbstractStackState {
    let mut main = state.main().to_vec();
    main.truncate(keep);
    main.push(StackValueType::Empty);
    AbstractStackState::new(main, state.alternate().to_vec())
}

/// The reviewed contract of one primitive.
///
/// Total by construction: the target validator refuses a definition
/// that omits a contract for any member of the reviewed census.
fn opcode(target: &ReviewedElementsTapscriptDefinition, id: OpcodeId) -> &'_ OpcodeSpec {
    target
        .definition()
        .opcodes()
        .get(&id)
        .expect("the reviewed contract states a contract for every primitive")
}

/// Whether an operand of type `actual` satisfies a declared operand.
///
/// The same type always satisfies itself. Otherwise one of the two must
/// be an unconstrained byte string — a literal a program pushed, or an
/// operand the contract declares as bytes — and then the question is
/// whether the widths fit.
fn accepts(
    target: &ReviewedElementsTapscriptDefinition,
    declared: &StackValueType,
    actual: &StackValueType,
) -> bool {
    if declared == actual {
        return true;
    }

    let literal = matches!(actual, StackValueType::Bytes { .. } | StackValueType::Empty);
    let unconstrained = matches!(declared, StackValueType::Bytes { .. });
    if !literal && !unconstrained {
        return false;
    }

    let admissible = width_ranges(target, declared);
    width_ranges(target, actual)
        .into_iter()
        .all(|range| admissible.iter().any(|wider| contains(*wider, range)))
}

/// Whether one inclusive width range contains another.
const fn contains(wider: (usize, usize), range: (usize, usize)) -> bool {
    wider.0 <= range.0 && range.1 <= wider.1
}

/// Whether a type fixes exactly one width.
fn is_exact_width(target: &ReviewedElementsTapscriptDefinition, value: &StackValueType) -> bool {
    let ranges = width_ranges(target, value);
    ranges.len() == 1 && ranges.iter().all(|(minimum, maximum)| minimum == maximum)
}

/// The inclusive width ranges a stack type admits.
fn width_ranges(
    target: &ReviewedElementsTapscriptDefinition,
    value: &StackValueType,
) -> BTreeSet<(usize, usize)> {
    let mut ranges = BTreeSet::new();
    match value {
        // The target's truth values are the empty item and a one-byte
        // one.
        StackValueType::Bool => {
            ranges.insert((0, 1));
        }
        StackValueType::Empty => {
            ranges.insert((0, 0));
        }
        StackValueType::ScriptNumber => {
            ranges.insert(payload_range(target, EncodingClass::ScriptNumber));
        }
        StackValueType::Bytes { minimum, maximum } => {
            ranges.insert((*minimum, *maximum));
        }
        StackValueType::SignedFixedWidth { bytes, .. }
        | StackValueType::UnsignedFixedWidth { bytes, .. } => {
            ranges.insert((bytes.get(), bytes.get()));
        }
        StackValueType::Encoded(class) => {
            ranges.insert(whole_field_range(target, *class));
        }
        StackValueType::EncodedPayload(class) => {
            ranges.insert(payload_range(target, *class));
        }
        StackValueType::EncodingPrefix(_) | StackValueType::EncodingPrefixAlternatives(_) => {
            ranges.insert((1, 1));
        }
        StackValueType::EncodedPayloadAlternatives(classes) => {
            for class in classes {
                ranges.insert(payload_range(target, *class));
            }
        }
        // A type this crate has not been taught admits no width it can
        // reason about, so nothing satisfies it and nothing it produces
        // satisfies anything else.
        _ => {}
    }
    ranges
}

/// The width range of one encoding class's payload.
fn payload_range(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> (usize, usize) {
    let Some(spec) = target.definition().encodings().get(&class) else {
        return (0, 0);
    };
    match spec.payload() {
        PayloadWidth::Absent => (0, 0),
        PayloadWidth::Exact(exact) => (exact.get(), exact.get()),
        PayloadWidth::Bounded { minimum, maximum } => (minimum, maximum.get()),
    }
}

/// The width range of one whole encoded field, its prefix included.
fn whole_field_range(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> (usize, usize) {
    let (minimum, maximum) = payload_range(target, class);
    let prefix = target
        .definition()
        .encodings()
        .get(&class)
        .map_or(0, |spec| usize::from(!spec.prefixes().is_empty()));
    (minimum + prefix, maximum + prefix)
}

/// The resource cost one program can charge, by dimension.
///
/// A projection rather than a bound: it states what the reviewed
/// contracts say the primitives in this program can consume, and
/// nothing here compares it against a deployment.
#[must_use]
pub fn resource_projection(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> BTreeMap<ResourceDimension, u64> {
    let mut totals: BTreeMap<ResourceDimension, u64> = BTreeMap::new();
    for instruction in program.instructions() {
        let TapscriptInstruction::Opcode(id) = instruction else {
            continue;
        };
        let cost = opcode(target, *id).resources();
        // Saturating: these are diagnostic totals, and a saturated one
        // is visibly pinned where a wrapped one would read as small and
        // honest.
        for (dimension, units) in [
            (ResourceDimension::ScriptBytes, cost.script_bytes()),
            (ResourceDimension::OperationCost, cost.operation_cost()),
            (
                ResourceDimension::ValidationBudget,
                cost.validation_budget(),
            ),
        ] {
            let total = totals.entry(dimension).or_insert(0);
            *total = total.saturating_add(units);
        }
    }
    totals
}
