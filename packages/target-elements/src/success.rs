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

/// What one successful form does to the stack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuccessStackEffect {
    consumed_operands: usize,
    results: Vec<StackValueType>,
}

impl SuccessStackEffect {
    /// States one successful form's stack arithmetic.
    ///
    /// `results` is in push order, so its last element ends up on top.
    #[must_use]
    pub const fn new(consumed_operands: usize, results: Vec<StackValueType>) -> Self {
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
    pub fn results(&self) -> &[StackValueType] {
        &self.results
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
        }
    }

    /// Every result type the contract can push, across every form.
    #[must_use]
    pub fn result_types(&self) -> Vec<StackValueType> {
        match self {
            Self::Fixed { results, .. } | Self::RetainsOperands { results } => results.clone(),
            Self::Alternatives { cases } => cases
                .iter()
                .flat_map(|case| case.effect().results().iter().cloned())
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
        }
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
        }
        None
    }
}
