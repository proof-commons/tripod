//! The three wide-floor candidates, and which one was taken.
//!
//! # What is measured and what is not
//!
//! Exactly one candidate is emitted, so exactly one has measurements.
//! The other two are recorded as *lower bounds* in the same units, read
//! off what any complete schedule for them would have to do, and a lower
//! bound that already exceeds the taken candidate's measurement settles
//! the comparison without building the schedule
//! `(´[PLAN-rule:guide10:arithmetic-comparison-stage]´)`.
//!
//! [`ComparisonBasis`] keeps the two apart in the type, because a
//! projection recorded beside a measurement in an untyped table becomes
//! a measurement the first time somebody quotes the table.
//!
//! # No calibration
//!
//! These figures answer whether the prototype is plausible, which
//! candidate is smaller, and which target limit binds first. They select
//! no architecture bound, no production tree depth, and no operation
//! limit `(´[PLAN-rule:guide10:no-calibration]´)`.

use tapscript::instruction::TapscriptInstruction;
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::prototype_program::{PrototypeProgram, PrototypeResourceProjection};

use super::schedule;

/// Which wide-floor construction a comparison row is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum WideFloorCandidate {
    /// Candidate A: every limb derived by the target's own division by
    /// the base `(´[PLAN-candidate:guide10:derived-limbs]´)`.
    DerivedLimbs,
    /// Candidate B: limbs, product limbs, and carries supplied as
    /// witness and checked `(´[PLAN-candidate:guide10:witnessed-limbs]´)`.
    WitnessedLimbs,
    /// Candidate C: `q·d <= a·b < (q+1)·d`, with no remainder witness
    /// `(´[PLAN-candidate:guide10:sandwich]´)`.
    Sandwich,
}

impl WideFloorCandidate {
    /// The complete census.
    pub const ALL: &'static [Self] = &[Self::DerivedLimbs, Self::WitnessedLimbs, Self::Sandwich];
}

/// Whether a row's figures were measured or derived as a bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ComparisonBasis {
    /// Counted from an emitted program the reviewed contracts admitted.
    Measured,
    /// A lower bound any complete schedule for the candidate must meet.
    LowerBound,
}

/// How many of each reviewed primitive class a candidate performs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimitiveCounts {
    /// Fixed-width multiplications.
    pub multiplications: u64,
    /// Fixed-width divisions.
    pub divisions: u64,
    /// Fixed-width additions.
    pub additions: u64,
    /// Fixed-width comparisons.
    pub comparisons: u64,
    /// Verifications, of a success flag or of an equality.
    pub verifications: u64,
    /// Concatenations.
    pub concatenations: u64,
    /// Constant-width slices.
    pub slices: u64,
}

impl PrimitiveCounts {
    /// Counts the classes in one emitted program.
    #[must_use]
    pub fn of(program: &PrototypeProgram) -> Self {
        let mut counts = Self::default();
        for instruction in program.program().instructions() {
            let TapscriptInstruction::Opcode(id) = instruction else {
                continue;
            };
            match id {
                OpcodeId::Mul64 => counts.multiplications += 1,
                OpcodeId::Div64 => counts.divisions += 1,
                OpcodeId::Add64 | OpcodeId::Sub64 => counts.additions += 1,
                OpcodeId::LessThan64
                | OpcodeId::LessThanOrEqual64
                | OpcodeId::GreaterThan64
                | OpcodeId::GreaterThanOrEqual64 => counts.comparisons += 1,
                OpcodeId::Verify | OpcodeId::EqualVerify => counts.verifications += 1,
                OpcodeId::Concatenate => counts.concatenations += 1,
                OpcodeId::Substring => counts.slices += 1,
                _ => {}
            }
        }
        counts
    }
}

/// One candidate's row in the comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateComparison {
    candidate: WideFloorCandidate,
    basis: ComparisonBasis,
    witness_items: u64,
    witness_bytes: u64,
    counts: PrimitiveCounts,
    measured: Option<PrototypeResourceProjection>,
}

impl CandidateComparison {
    /// Which candidate.
    #[must_use]
    pub const fn candidate(&self) -> WideFloorCandidate {
        self.candidate
    }

    /// Whether the figures were measured or bounded.
    #[must_use]
    pub const fn basis(&self) -> ComparisonBasis {
        self.basis
    }

    /// How many items the witness carries.
    #[must_use]
    pub const fn witness_items(&self) -> u64 {
        self.witness_items
    }

    /// The witness's serialized width, control block and script
    /// included where the row is measured, and the stack items alone
    /// where it is bounded.
    #[must_use]
    pub const fn witness_bytes(&self) -> u64 {
        self.witness_bytes
    }

    /// The primitive counts.
    #[must_use]
    pub const fn counts(&self) -> PrimitiveCounts {
        self.counts
    }

    /// The measured resource projection, where one exists.
    #[must_use]
    pub const fn measured(&self) -> Option<PrototypeResourceProjection> {
        self.measured
    }
}

/// How many stack items each candidate's witness carries.
///
/// Candidate A witnesses the five amounts and nothing else. Candidate B
/// adds ten operand limbs, eight product limbs, and four carries, all of
/// which the target derives in candidate A. Candidate C drops the
/// remainder and keeps the four amounts.
const DERIVED_LIMB_WITNESS_ITEMS: u64 = 5;
/// Candidate B's witness: the amounts, every limb, and every carry.
const WITNESSED_LIMB_WITNESS_ITEMS: u64 = 5 + 10 + 8 + 4;
/// Candidate C's witness: the three amounts and the quotient.
const SANDWICH_WITNESS_ITEMS: u64 = 4;

/// How wide one witnessed amount, limb, or carry is on the wire.
///
/// Eight canonical bytes and the one-byte compact-size length every
/// witness item carries.
const WITNESS_ITEM_BYTES: u64 = 9;

/// The complete §20.2 comparison.
///
/// # Errors
///
/// `None` when the reviewed contract does not admit the emitted
/// candidate, in which case there is nothing to compare against.
#[must_use]
pub fn comparison(
    target: &ReviewedElementsTapscriptDefinition,
) -> Option<Vec<CandidateComparison>> {
    let emitted = PrototypeProgram::wide_floor(target).ok()?;
    let counts = PrimitiveCounts::of(&emitted);
    let resources = emitted.resources();

    Some(vec![
        CandidateComparison {
            candidate: WideFloorCandidate::DerivedLimbs,
            basis: ComparisonBasis::Measured,
            witness_items: DERIVED_LIMB_WITNESS_ITEMS,
            witness_bytes: resources.witness_bytes(),
            counts,
            measured: Some(resources),
        },
        CandidateComparison {
            candidate: WideFloorCandidate::WitnessedLimbs,
            basis: ComparisonBasis::LowerBound,
            witness_items: WITNESSED_LIMB_WITNESS_ITEMS,
            witness_bytes: WITNESSED_LIMB_WITNESS_ITEMS * WITNESS_ITEM_BYTES,
            // Every partial product still has to be recomputed and
            // compared, so the multiplications do not go away; each of
            // the five amounts additionally needs its recomposition
            // proved. The divisions do go away, which is the candidate's
            // one advantage, and every witnessed limb then needs two
            // bound comparisons the derived form gets for free.
            counts: PrimitiveCounts {
                multiplications: counts.multiplications + 5,
                divisions: 0,
                additions: counts.additions + 5,
                comparisons: counts.comparisons + 2 * 10,
                verifications: counts.verifications + 5 + 8 + 4,
                concatenations: counts.concatenations,
                slices: counts.slices,
            },
            measured: None,
        },
        CandidateComparison {
            candidate: WideFloorCandidate::Sandwich,
            basis: ComparisonBasis::LowerBound,
            witness_items: SANDWICH_WITNESS_ITEMS,
            witness_bytes: SANDWICH_WITNESS_ITEMS * WITNESS_ITEM_BYTES,
            // Three wide products instead of two, so half again as many
            // partial products, normalizations, and carries; plus the
            // increment of the quotient under its own flag, and two
            // four-limb orderings, each of which is a cascade of
            // comparisons rather than the single byte equality the
            // limbs admit.
            counts: PrimitiveCounts {
                multiplications: counts.multiplications * 3 / 2,
                divisions: counts.divisions * 3 / 2,
                additions: counts.additions * 3 / 2 + 1,
                comparisons: counts.comparisons + 8,
                verifications: counts.verifications * 3 / 2,
                concatenations: counts.concatenations * 3 / 2,
                slices: counts.slices * 3 / 2,
            },
            measured: None,
        },
    ])
}

/// Which candidate the comparison selects.
///
/// Candidate A, and not by preference. The reviewed substrate carries a
/// Euclidean fixed-width division, so a limb costs one division and one
/// verified flag and is correct by the primitive's own contract; the
/// witnessed form pays five times the witness for values it then has to
/// bound-check, and the sandwich pays half again as much arithmetic to
/// avoid a witness worth nine bytes
/// `(´[PLAN-rule:guide10:arithmetic-comparison-stage]´)`.
#[must_use]
pub const fn selected() -> WideFloorCandidate {
    WideFloorCandidate::DerivedLimbs
}

/// Why each candidate was or was not taken.
#[must_use]
pub const fn disposition(candidate: WideFloorCandidate) -> &'static str {
    match candidate {
        WideFloorCandidate::DerivedLimbs => {
            "taken: the reviewed division primitive derives both limbs of an amount in one \
             operation under one verified flag, so no limb, carry, or product limb is witnessed \
             and the underconstrained-witness rows of the threat matrix have nothing to attack"
        }
        WideFloorCandidate::WitnessedLimbs => {
            "not implemented: it removes the divisions and pays for them with five times the \
             witness and twenty limb-bound comparisons, and Guide 10 admits it only where the \
             derived form is too large, too deep, or hard to schedule, which the measured row \
             shows it is not"
        }
        WideFloorCandidate::Sandwich => {
            "not implemented: a shorter mathematical statement is not a simpler target proof. It \
             needs three wide products where the relation needs two, an incremented quotient \
             under its own overflow flag, and two four-limb orderings where the limb comparison \
             is one byte equality"
        }
    }
}

/// How many instructions the emitted candidate occupies.
///
/// A convenience for the comparison's readers; the schedule module owns
/// the boundary between the pattern and its standalone framing.
#[must_use]
pub fn emitted_instruction_count(target: &ReviewedElementsTapscriptDefinition) -> Option<usize> {
    schedule::instructions(target).map(|emitted| emitted.len())
}
