//! The typed relation between a primitive's operands and its
//! successful results.
//!
//! # Success is a relation, not one output vector
//!
//! Several reviewed primitives have more than one valid successful
//! stack result. An input value arrives as an eight-byte explicit
//! payload or a thirty-two-byte confidential one; an output nonce
//! arrives explicit, confidential, or absent; a program arrives as a
//! witness program with its version or as a script digest with a
//! negative marker; an input carries an issuance or it does not; and
//! the relative-timelock check leaves its operand exactly where it
//! found it.
//!
//! A single result vector can state one of those and no more. The rest
//! then survive only as prose, and a comment describing an alternative
//! the type cannot represent is not a contract — a downstream stack
//! scheduler reading the type would compute one depth and the target
//! would produce another.
//!
//! # Every form states its own arithmetic
//!
//! Each successful form here states how many operands it consumes and
//! exactly what it pushes, so the resulting depth follows from the
//! type rather than from a convention. That matters because the
//! convention is not uniform: most primitives consume every operand
//! they name, and [`SuccessContract::RetainsOperands`] exists because
//! at least one inspects its operand and leaves it in place.

use std::collections::BTreeSet;

use crate::opcode::StackValueType;

/// What selects one successful form of a primitive.
///
/// The conditions are properties of the *target value being read*, not
/// of the program: whether the field it found was explicit, blinded,
/// or absent, whether the program was a witness program, whether the
/// input carried an issuance. A program cannot choose among them, so a
/// backend must be able to handle every form a primitive admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SuccessCondition {
    /// The primitive has one successful form.
    Always,
    /// The field read was carried in the clear.
    ExplicitEncoding,
    /// The field read was carried as a blinded commitment.
    ConfidentialEncoding,
    /// The field read was absent.
    NullEncoding,
    /// The program read was a witness program.
    WitnessProgram,
    /// The program read was not a witness program, so a digest stands
    /// in for it.
    NonWitnessProgram,
    /// The input read carried an issuance.
    IssuancePresent,
    /// The input read carried no issuance.
    IssuanceAbsent,
    /// The key offered was the recognized encoding, and the nonempty
    /// signature verified against it.
    ///
    /// The ordinary signature success, named rather than left as
    /// `Always` because it is now one of two: the target has a second
    /// successful form that verifies nothing at all.
    RecognizedKeyVerifiedSignature,
    /// The key offered was a nonempty key of an unrecognized form, so
    /// the check succeeded without verifying anything.
    ///
    /// The target's forward-compatibility rule. It is a success and
    /// must be modeled as one: a caller that treated an unrecognized
    /// key as a rejection would believe a spend fails that in fact
    /// stands, which is the more dangerous of the two errors.
    UnknownKeyTypeUnverified,
}

impl SuccessCondition {
    /// The complete census of success conditions.
    pub const ALL: &'static [Self] = &[
        Self::Always,
        Self::ExplicitEncoding,
        Self::ConfidentialEncoding,
        Self::NullEncoding,
        Self::WitnessProgram,
        Self::NonWitnessProgram,
        Self::IssuancePresent,
        Self::IssuanceAbsent,
        Self::RecognizedKeyVerifiedSignature,
        Self::UnknownKeyTypeUnverified,
    ];
}

/// One item a successful form leaves on the stack.
///
/// # Why a result is not always a type
///
/// Most primitives push values they compute, and the contract can name
/// what those values are: a digest, a truth value, a fixed-width
/// integer. The ordinary stack operations push nothing of their own.
/// [`OpcodeId::Duplicate`] pushes whatever was already there, and what
/// that is depends entirely on the program, not on the primitive.
///
/// Naming a type for such a result would be a claim about the caller's
/// stack that this contract has no way to know. Naming the *operand*
/// instead states exactly what the target does — the item is carried
/// through unchanged — and lets a stack validator resolve the type from
/// the state it actually has (Guide-10 `rule:guide10:primitive-admission`).
///
/// [`OpcodeId::Duplicate`]: crate::opcode::OpcodeId::Duplicate
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResultValue {
    /// A value the primitive computes, stated as its type.
    Computed(StackValueType),
    /// A copy of one declared operand, named by its deepest-first
    /// index in the operand list.
    ///
    /// The item is carried through byte for byte. It is a copy rather
    /// than a move because the same operand may be named more than
    /// once, which is exactly what a duplicating primitive does.
    OperandCopy(usize),
    /// A value the primitive computes whose exact width the target
    /// takes from the *value* of one declared operand.
    ///
    /// # Why a width has to be stated this way
    ///
    /// The reviewed slice primitive returns exactly as many bytes as
    /// its length operand asks for. Stated as a plain
    /// [`Self::Computed`] byte string the result is a width the
    /// contract does not fix, and a consumer that then wants to feed
    /// the slice into a width-constrained position has no ground to
    /// stand on: it would have to assume a width the contract never
    /// claimed. The reviewed target has no such freedom — it produces
    /// the requested width or it aborts — so the contract states the
    /// dependency instead of losing it.
    ///
    /// # Deliberately not a width calculus
    ///
    /// This names one operand whose value *is* the width, and nothing
    /// more: no arithmetic over widths, no symbolic expressions, no
    /// relation between two operands. A consumer that cannot settle the
    /// named operand's value falls back to [`Self::unsettled_type`],
    /// which is the same unconstrained result the contract stated
    /// before, so the extension never narrows a result a consumer could
    /// not already justify.
    ComputedWidthFromOperand {
        /// The deepest-first index of the operand whose numeric value
        /// is the result's width in bytes.
        width_operand: usize,
        /// The type the result carries wherever that operand's value is
        /// not settled.
        ///
        /// It must admit every width the settled form can take: it is
        /// the honest answer for a consumer that knows nothing about
        /// the operand, and every narrower answer this variant permits
        /// lies inside it.
        unsettled: StackValueType,
    },
}

impl ResultValue {
    /// The type this result carries when nothing about the operands is
    /// settled.
    ///
    /// An operand copy has none: its type is whatever the caller's
    /// stack held, which this contract does not know.
    #[must_use]
    pub const fn unsettled_type(&self) -> Option<&StackValueType> {
        match self {
            Self::Computed(value)
            | Self::ComputedWidthFromOperand {
                unsettled: value, ..
            } => Some(value),
            Self::OperandCopy(_) => None,
        }
    }

    /// The declared-operand index this result names, if any.
    #[must_use]
    pub const fn named_operand(&self) -> Option<usize> {
        match self {
            Self::OperandCopy(index)
            | Self::ComputedWidthFromOperand {
                width_operand: index,
                ..
            } => Some(*index),
            Self::Computed(_) => None,
        }
    }
}

impl From<StackValueType> for ResultValue {
    fn from(value: StackValueType) -> Self {
        Self::Computed(value)
    }
}

/// What one successful form does to the stack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuccessStackEffect {
    consumed_operands: usize,
    results: Vec<ResultValue>,
}

impl SuccessStackEffect {
    /// States one successful form's stack arithmetic.
    ///
    /// `results` is in push order, so its last element ends up on top.
    #[must_use]
    pub fn new(consumed_operands: usize, results: Vec<StackValueType>) -> Self {
        Self::resolving(
            consumed_operands,
            results.into_iter().map(ResultValue::Computed).collect(),
        )
    }

    /// States one successful form that can carry operands through.
    ///
    /// The general form. [`Self::new`] is the common case where every
    /// result is a value the primitive computed.
    #[must_use]
    pub const fn resolving(consumed_operands: usize, results: Vec<ResultValue>) -> Self {
        Self {
            consumed_operands,
            results,
        }
    }

    /// How many of the declared operands this form consumes.
    #[must_use]
    pub const fn consumed_operands(&self) -> usize {
        self.consumed_operands
    }

    /// How many of `declared` operands this form leaves in place.
    #[must_use]
    pub const fn retained_operands(&self, declared: usize) -> usize {
        declared.saturating_sub(self.consumed_operands)
    }

    /// The items pushed, in push order.
    #[must_use]
    pub fn results(&self) -> &[ResultValue] {
        &self.results
    }

    /// The types among the pushed items.
    ///
    /// An operand carried through has no type here, and is absent
    /// rather than guessed at: its type is whatever the caller's stack
    /// held, which this contract does not know.
    ///
    /// A result whose width the target takes from an operand reports
    /// its unsettled type, which is the widest form it can take. A
    /// consumer reading this method learns exactly what it learned
    /// before the width relation existed; narrowing is available only
    /// to a consumer that walks [`Self::results`] and can settle the
    /// operand.
    #[must_use]
    pub fn computed_types(&self) -> Vec<StackValueType> {
        self.results
            .iter()
            .filter_map(|result| result.unsettled_type().cloned())
            .collect()
    }

    /// The greatest declared-operand index this form names, if any.
    #[must_use]
    pub fn deepest_named_operand(&self) -> Option<usize> {
        self.results
            .iter()
            .filter_map(ResultValue::named_operand)
            .max()
    }

    /// The change this form makes to the main stack's depth.
    ///
    /// The depth after a successful execution is the depth before it
    /// plus this number. Stating it here rather than deriving it at
    /// each call site is the point of the type: the two contributing
    /// numbers are not the same for every form of every primitive.
    #[must_use]
    pub fn depth_change(&self) -> i64 {
        let pushed = i64::try_from(self.results.len()).unwrap_or(i64::MAX);
        let consumed = i64::try_from(self.consumed_operands).unwrap_or(i64::MAX);
        pushed - consumed
    }
}

/// One successful form, and what selects it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuccessCase {
    condition: SuccessCondition,
    effect: SuccessStackEffect,
}

impl SuccessCase {
    /// States one successful form.
    #[must_use]
    pub const fn new(condition: SuccessCondition, effect: SuccessStackEffect) -> Self {
        Self { condition, effect }
    }

    /// What selects this form.
    #[must_use]
    pub const fn condition(&self) -> SuccessCondition {
        self.condition
    }

    /// What this form does to the stack.
    #[must_use]
    pub const fn effect(&self) -> &SuccessStackEffect {
        &self.effect
    }
}

/// The complete successful behavior of one primitive.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SuccessContract {
    /// One successful form that consumes operands and pushes results.
    Fixed {
        /// How many of the declared operands are consumed.
        consumed_operands: usize,
        /// The items pushed, in push order.
        results: Vec<StackValueType>,
    },
    /// One successful form that inspects its operands and leaves every
    /// one of them in place.
    ///
    /// Distinct from [`Self::Fixed`] with a zero count, because that
    /// would read as a primitive that happens to take no operands
    /// rather than one that deliberately does not remove the operands
    /// it reads.
    RetainsOperands {
        /// The items pushed above the retained operands, in push
        /// order. Often empty.
        results: Vec<StackValueType>,
    },
    /// Several successful forms, each selected by a condition on the
    /// target value that was read.
    Alternatives {
        /// The forms, one per condition.
        cases: Vec<SuccessCase>,
    },
    /// One successful form whose results are stated against the
    /// operands rather than as plain types.
    ///
    /// Two reviewed shapes need this. The ordinary stack operations
    /// push the operands themselves, in a new order and possibly more
    /// than once, so they cannot be stated as [`Self::Fixed`] without
    /// inventing types for items whose types the caller chose. The
    /// slice primitive pushes a value it computed, but of a width its
    /// operand names, which [`Self::Fixed`] cannot state either.
    ///
    /// Both are the same fact about the target: what the primitive
    /// leaves behind is a function of what it was handed, and a
    /// contract that dropped the dependency would understate it.
    OperandResolved {
        /// How many of the declared operands are consumed.
        consumed_operands: usize,
        /// The items pushed, in push order.
        results: Vec<ResultValue>,
    },
}

/// Why a success contract does not describe a coherent relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuccessContractDefect {
    /// The contract offers alternatives but names none, so it states
    /// no successful behavior at all.
    NoCases,
    /// Two alternatives claim the same condition, so the contract does
    /// not say which one the target produces.
    DuplicateCondition(SuccessCondition),
    /// A form consumes more operands than the primitive declares.
    ConsumesMoreThanDeclared,
    /// A contract retains operands the primitive does not declare, so
    /// there is nothing for it to retain.
    NothingToRetain,
    /// A result names an operand position the primitive does not
    /// declare, so there is no item for it to carry through.
    ResultNamesNoOperand,
}

impl SuccessContract {
    /// The successful forms, normalized to cases.
    ///
    /// [`Self::Fixed`] and [`Self::RetainsOperands`] have one form
    /// apiece and report it under [`SuccessCondition::Always`], so a
    /// consumer can walk every primitive's forms the same way instead
    /// of matching the shape at each call site.
    #[must_use]
    pub fn cases(&self) -> Vec<SuccessCase> {
        match self {
            Self::Fixed {
                consumed_operands,
                results,
            } => vec![SuccessCase::new(
                SuccessCondition::Always,
                SuccessStackEffect::new(*consumed_operands, results.clone()),
            )],
            Self::RetainsOperands { results } => vec![SuccessCase::new(
                SuccessCondition::Always,
                SuccessStackEffect::new(0, results.clone()),
            )],
            Self::Alternatives { cases } => cases.clone(),
            Self::OperandResolved {
                consumed_operands,
                results,
            } => vec![SuccessCase::new(
                SuccessCondition::Always,
                SuccessStackEffect::resolving(*consumed_operands, results.clone()),
            )],
        }
    }

    /// Every result type the contract can push, across every form.
    #[must_use]
    pub fn result_types(&self) -> Vec<StackValueType> {
        match self {
            Self::Fixed { results, .. } | Self::RetainsOperands { results } => results.clone(),
            Self::Alternatives { cases } => cases
                .iter()
                .flat_map(|case| case.effect().computed_types())
                .collect(),
            Self::OperandResolved { results, .. } => results
                .iter()
                .filter_map(|result| result.unsettled_type().cloned())
                .collect(),
        }
    }

    /// The first way the contract fails to describe a coherent
    /// relation over `declared_operands` operands.
    #[must_use]
    pub fn defect(&self, declared_operands: usize) -> Option<SuccessContractDefect> {
        match self {
            Self::Fixed {
                consumed_operands, ..
            } => (*consumed_operands > declared_operands)
                .then_some(SuccessContractDefect::ConsumesMoreThanDeclared),
            Self::RetainsOperands { .. } => {
                (declared_operands == 0).then_some(SuccessContractDefect::NothingToRetain)
            }
            Self::Alternatives { cases } => Self::alternatives_defect(cases, declared_operands),
            Self::OperandResolved {
                consumed_operands,
                results,
            } => Self::operand_resolved_defect(*consumed_operands, results, declared_operands),
        }
    }

    /// The first defect in an operand-resolved form.
    ///
    /// A named operand that the primitive does not declare is the
    /// dangerous one: a stack validator resolving it would read past
    /// the operands it checked, so the contract is refused here rather
    /// than left for the validator to survive.
    fn operand_resolved_defect(
        consumed_operands: usize,
        results: &[ResultValue],
        declared_operands: usize,
    ) -> Option<SuccessContractDefect> {
        if consumed_operands > declared_operands {
            return Some(SuccessContractDefect::ConsumesMoreThanDeclared);
        }
        results
            .iter()
            .filter_map(ResultValue::named_operand)
            .any(|index| index >= declared_operands)
            .then_some(SuccessContractDefect::ResultNamesNoOperand)
    }

    /// The first defect among a set of alternative forms.
    fn alternatives_defect(
        cases: &[SuccessCase],
        declared_operands: usize,
    ) -> Option<SuccessContractDefect> {
        if cases.is_empty() {
            return Some(SuccessContractDefect::NoCases);
        }

        let mut seen: BTreeSet<SuccessCondition> = BTreeSet::new();
        for case in cases {
            if !seen.insert(case.condition()) {
                return Some(SuccessContractDefect::DuplicateCondition(case.condition()));
            }
            if case.effect().consumed_operands() > declared_operands {
                return Some(SuccessContractDefect::ConsumesMoreThanDeclared);
            }
            if case
                .effect()
                .deepest_named_operand()
                .is_some_and(|index| index >= declared_operands)
            {
                return Some(SuccessContractDefect::ResultNamesNoOperand);
            }
        }
        None
    }
}
