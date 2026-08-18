//! The wide-floor prototype's case matrix.
//!
//! # What a row is
//!
//! One compound-prototype fixture: the exact script the spend executes,
//! the exact five-item witness it carries, the bare leaf that script
//! lives in, and the outcome the reviewed target is required to reach.
//! Every expected outcome is authored from this package's own wide-floor
//! oracle — never from an executor's answer
//! `(´[PLAN-rule:guide10:independent-oracles]´)`.
//!
//! # Where a mutation lives
//!
//! In the witness, and nowhere else. The wide-floor pattern reads no
//! transaction field, derives no output, and hashes nothing: everything
//! it decides, it decides from five stack items. That is what makes this
//! matrix a plain enumeration where the constructor's had to reason
//! about which half of a construction a mutation could live in.
//!
//! One row is the exception, and deliberately: the unchecked-flag row
//! mutates the *script*, because a schedule that fails to verify a
//! success flag is not a witness a caller could supply. Guide 10 admits
//! that row as a static or a target rejection
//! `(´[PLAN-tab:guide10:wide-floor-threats]´)`; it is stated as both, refused by
//! the emitter and refused by the target.
//!
//! # What this matrix cannot state, and why
//!
//! Three threat rows are substitutions of `a`, `b`, or `d` by an
//! enclosing binding — the operation that authenticates those amounts
//! before handing them to this pattern. A standalone pattern has no
//! enclosing binding, so the rows are residuals rather than cases: they
//! belong to whatever operation composes this proof, and stating them
//! here would be claiming coverage of a boundary this program does not
//! have.
//!
//! Two more — a wrong limb and a wrong carry — have no witness to
//! mutate. The selected candidate derives every limb and every carry
//! from the amounts by the target's own division, so there is no
//! caller-supplied limb to corrupt. The threat is answered by the
//! construction rather than by a case, and recording a case that could
//! not fail would be padding.
//!
//! # A mock cannot satisfy any of this
//!
//! Every row's outcome is a target verdict, and these rows are answered
//! by a native run: `check-target-elements-prototypes` drives this matrix
//! through the reviewed executor and the prototype gate reads the result,
//! which is why every claim here is now required rather than unresolved.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use tapscript::instruction::TapscriptInstruction;
use tapscript::program::TapscriptProgram;
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::tree::{FixtureTapTree, construct};
use crate::fixture::{ExpectedResourceObservation, ResourceExpectation};
use crate::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeCaseId, PrototypeClaim,
    PrototypeConstruction, PrototypeRelation,
};
use crate::prototype_program::PrototypeProgram;
use crate::wide_floor::domain::{AMOUNT_DOMAIN, HIGH_LIMB_BOUND, LIMB_BASE};
use crate::wide_floor::oracle::{WideFloorInstance, WideFloorWitness};
use crate::wide_floor::schedule;

/// Why the matrix could not be authored.
///
/// Every variant is a defect in this package rather than in a target:
/// the matrix is built from the reviewed contract and this package's own
/// oracle, and either both agree or something here is wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum WideFloorMatrixDefect {
    /// The prototype program is not admitted against the reviewed
    /// contract, so there is no script to state.
    ProgramNotAdmitted,
    /// A vector the matrix states has no coherent instance, which means
    /// the oracle and the vector disagree about the domain.
    InstanceNotAvailable,
    /// The bare leaf determines no output key.
    LeafNotConstructible,
    /// The flag-unchecked script variant is not expressible.
    VariantNotExpressible,
}

impl fmt::Display for WideFloorMatrixDefect {
    /// The defect, spelled for a command's typed diagnostic.
    ///
    /// As for the constructor matrix: every spelling names a part of
    /// this package that did not determine a row, and none describes a
    /// target. A command reporting one is reporting that this
    /// repository's contract and its oracle disagree
    /// `(´[ADR010-rule:output:data-classification]´)`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::ProgramNotAdmitted => {
                "the wide-floor prototype's program is not admitted against the reviewed \
                 contract, so there is no script to state"
            }
            Self::InstanceNotAvailable => {
                "a vector the matrix states has no coherent instance, so the oracle and the \
                 vector disagree about the domain"
            }
            Self::LeafNotConstructible => "the bare leaf determines no output key",
            Self::VariantNotExpressible => "the flag-unchecked script variant is not expressible",
        };
        formatter.write_str(text)
    }
}

/// A resource expectation that compares nothing.
///
/// What one of these costs is a measurement the native run takes. An
/// invented figure would fail an honest executor over a number no
/// contract states.
const fn recorded_only() -> ExpectedResourceObservation {
    ExpectedResourceObservation {
        script_bytes: ResourceExpectation::RecordedOnly,
        initial_stack_items: ResourceExpectation::RecordedOnly,
        peak_stack_items: ResourceExpectation::RecordedOnly,
        peak_altstack_items: ResourceExpectation::RecordedOnly,
        maximum_element_bytes: ResourceExpectation::RecordedOnly,
        validation_budget_used: ResourceExpectation::RecordedOnly,
        transaction_weight: ResourceExpectation::RecordedOnly,
    }
}

/// Everything the matrix shares: the emitted script and its bare leaf.
struct Recipe {
    script: Vec<u8>,
    unchecked_flag_script: Vec<u8>,
}

impl Recipe {
    /// The recipe the reviewed contract determines.
    fn resolve(
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<Self, WideFloorMatrixDefect> {
        let emitted = PrototypeProgram::wide_floor(target)
            .map_err(|_| WideFloorMatrixDefect::ProgramNotAdmitted)?;
        Ok(Self {
            script: emitted.encode(target),
            unchecked_flag_script: unchecked_flag_variant(target)?,
        })
    }

    /// One row over the stated script and witness.
    ///
    /// The leaf, the tree, and the consumed input's program are what the
    /// script and the published internal key determine; a fixture
    /// stating anything else would describe a spend of an output its own
    /// leaf does not commit to.
    fn row(
        script: &[u8],
        name: &str,
        claims: &[PrototypeClaim],
        witness: Vec<Vec<u8>>,
        expected: ExpectedPrototypeOutcome,
    ) -> Result<CompoundPrototypeFixture, WideFloorMatrixDefect> {
        let leaf = FixtureTapTree::leaf(script.to_vec());
        let built = construct(&UNSPENDABLE_INTERNAL_KEY, &leaf, &leaf)
            .map_err(|_| WideFloorMatrixDefect::LeafNotConstructible)?;
        Ok(CompoundPrototypeFixture {
            case: PrototypeCaseId {
                relation: PrototypeRelation::WideFloorRelation,
                name: name.to_owned(),
            },
            claims: claims.iter().copied().collect(),
            target_contract_version: 0,
            script: script.to_vec(),
            initial_stack: witness,
            construction: PrototypeConstruction {
                internal_key: UNSPENDABLE_INTERNAL_KEY,
                tree: leaf.clone(),
                executing_leaf: leaf,
                control: Some(built.control_block().to_vec()),
                predecessor_program: built.output_program().to_vec(),
                // The pattern reads no transaction field, so it requires
                // no output of any role.
                outputs: Vec::new(),
            },
            expected,
            expected_resources: recorded_only(),
        })
    }
}

/// The emitted schedule with one multiplication's success flag left
/// unverified.
///
/// The first verification following a fixed-width multiplication is
/// removed and nothing else changes, which is exactly the omission
/// Guide 10 names. The result is refused twice over: the emitter will
/// not admit it, because its schedule leaves a state a failed
/// multiplication reaches; and a target executing it divides a
/// one-byte truth value as though it were a fixed-width amount.
fn unchecked_flag_variant(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<u8>, WideFloorMatrixDefect> {
    let instructions =
        schedule::instructions(target).ok_or(WideFloorMatrixDefect::VariantNotExpressible)?;
    let at = instructions
        .windows(2)
        .position(|pair| {
            pair[0] == TapscriptInstruction::Opcode(OpcodeId::Mul64)
                && pair[1] == TapscriptInstruction::Opcode(OpcodeId::Verify)
        })
        .ok_or(WideFloorMatrixDefect::VariantNotExpressible)?;
    let mut mutated = instructions;
    mutated.remove(at + 1);
    TapscriptProgram::new(mutated)
        .map(|program| program.encode(target))
        .map_err(|_| WideFloorMatrixDefect::VariantNotExpressible)
}

/// One witness, with one item replaced by exact bytes.
fn with_item(witness: &WideFloorWitness, index: usize, bytes: Vec<u8>) -> Vec<Vec<u8>> {
    let mut items = witness.encode();
    if let Some(slot) = items.get_mut(index) {
        *slot = bytes;
    }
    items
}

/// The solved instance for one vector, or a defect naming the vector as
/// outside the oracle's domain.
fn solve(
    first: u64,
    second: u64,
    divisor: u64,
) -> Result<WideFloorInstance, WideFloorMatrixDefect> {
    WideFloorInstance::solve(first, second, divisor)
        .map_err(|_| WideFloorMatrixDefect::InstanceNotAvailable)
}

/// The accepting rows: every vector Guide 10 requires the matrix to
/// carry, with the exact quotient and remainder the oracle states.
fn accepting_vectors() -> Vec<(&'static str, [u64; 3], &'static [PrototypeClaim])> {
    use PrototypeClaim as K;
    const EXACT: &[PrototypeClaim] = &[K::WideFloorExactDivisionObserved];
    const NONZERO: &[PrototypeClaim] = &[K::WideFloorNonzeroRemainderObserved];
    const LIMBS: &[PrototypeClaim] = &[K::WideFloorLimbDerivationObserved];
    const EXACT_AND_LIMBS: &[PrototypeClaim] = &[
        K::WideFloorExactDivisionObserved,
        K::WideFloorLimbDerivationObserved,
    ];
    const NONZERO_AND_LIMBS: &[PrototypeClaim] = &[
        K::WideFloorNonzeroRemainderObserved,
        K::WideFloorLimbDerivationObserved,
    ];

    vec![
        // Zero and one, at both ends of the relation.
        ("zero_numerator", [0, 1, 1], EXACT),
        ("one_times_one_over_one", [1, 1, 1], EXACT),
        // Exact division, with the remainder gone.
        ("exact_division", [6, 7, 42], EXACT),
        (
            "exact_division_over_the_base",
            [LIMB_BASE, 3, LIMB_BASE],
            EXACT,
        ),
        // A remainder at each end of its range.
        ("remainder_one", [7, 3, 4], NONZERO),
        ("remainder_one_below_the_divisor", [7, 2, 5], NONZERO),
        (
            "ordinary_nonzero_remainder",
            [123_456_789, 987_654_321, 1_000_003],
            NONZERO,
        ),
        // The divisor's own two ends.
        ("divisor_one", [AMOUNT_DOMAIN - 1, 1, 1], EXACT_AND_LIMBS),
        (
            "maximum_divisor",
            [AMOUNT_DOMAIN - 1, AMOUNT_DOMAIN - 1, AMOUNT_DOMAIN - 1],
            EXACT_AND_LIMBS,
        ),
        // A quotient of zero, and the largest quotient the domain holds.
        ("quotient_zero", [1, 1, AMOUNT_DOMAIN - 1], NONZERO),
        (
            "largest_accepted_quotient",
            [AMOUNT_DOMAIN - 1, 1, 1],
            EXACT_AND_LIMBS,
        ),
        // The base's boundaries, which are the limb derivation's own.
        (
            "operand_one_below_the_base",
            [LIMB_BASE - 1, LIMB_BASE - 1, 7],
            LIMBS,
        ),
        ("operand_at_the_base", [LIMB_BASE, LIMB_BASE, 7], LIMBS),
        (
            "operand_one_above_the_base",
            [LIMB_BASE + 1, LIMB_BASE + 1, 7],
            LIMBS,
        ),
        (
            "operand_one_below_the_high_limb_bound",
            [HIGH_LIMB_BOUND - 1, HIGH_LIMB_BOUND - 1, 3],
            LIMBS,
        ),
        (
            "operand_at_the_high_limb_bound",
            [HIGH_LIMB_BOUND, HIGH_LIMB_BOUND, 3],
            LIMBS,
        ),
        // Maximal carry propagation: every limb of both operands at its
        // own maximum, which is what drives each normalization's
        // dividend to the top of its bound.
        (
            "maximal_carry_propagation",
            [AMOUNT_DOMAIN - 1, AMOUNT_DOMAIN - 2, AMOUNT_DOMAIN - 1],
            NONZERO_AND_LIMBS,
        ),
        (
            "domain_maximum_one_below",
            [AMOUNT_DOMAIN - 2, AMOUNT_DOMAIN - 2, AMOUNT_DOMAIN - 1],
            NONZERO_AND_LIMBS,
        ),
    ]
}

/// The complete §22.6 wide-floor matrix.
///
/// # Errors
///
/// [`WideFloorMatrixDefect`] when the reviewed contract and this
/// package's oracle do not between them determine every row.
#[expect(
    clippy::too_many_lines,
    reason = "the matrix is one enumeration and splitting it would hide which rows exist"
)]
pub fn wide_floor_matrix(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<CompoundPrototypeFixture>, WideFloorMatrixDefect> {
    use ExpectedPrototypeOutcome::{Accepted, Rejected};
    use PrototypeClaim as K;

    let recipe = Recipe::resolve(target)?;
    let script = recipe.script.clone();
    let mut rows = Vec::new();

    for (name, [first, second, divisor], claims) in accepting_vectors() {
        let instance = solve(first, second, divisor)?;
        rows.push(Recipe::row(
            &script,
            name,
            claims,
            instance.witness().encode(),
            Accepted,
        )?);
    }

    // The representative instance every witness mutation is stated
    // against: an ordinary vector whose quotient and remainder are both
    // nonzero and whose limbs are all distinct, so a mutation cannot
    // coincide with the value it replaced.
    let base = solve(123_456_789, 987_654_321, 1_000_003)?;
    let witness = base.witness();

    // -- The quotient, one step either way ----------------------------

    rows.push(Recipe::row(
        &script,
        "quotient_one_too_small",
        &[K::WideFloorUnderQuotientRejectedObserved],
        WideFloorWitness {
            q: witness.q - 1,
            ..witness
        }
        .encode(),
        Rejected,
    )?);
    rows.push(Recipe::row(
        &script,
        "quotient_one_too_large",
        &[K::WideFloorOverQuotientRejectedObserved],
        WideFloorWitness {
            q: witness.q + 1,
            ..witness
        }
        .encode(),
        Rejected,
    )?);

    // -- The remainder's bound ----------------------------------------
    //
    // Both rows keep the equation balanced: the quotient is reduced by
    // one and the divisor is added back into the remainder, so `a·b`
    // still equals `q·d + r` exactly. What fails is `r < d`, which is
    // the condition that makes the quotient the floor rather than merely
    // *a* quotient (´[PLAN-rule:guide10:wide-floor-relation]´).
    for (name, extra) in [
        ("remainder_equals_the_divisor", 0_u64),
        ("remainder_above_the_divisor", 1),
    ] {
        rows.push(Recipe::row(
            &script,
            name,
            &[K::WideFloorRemainderBoundObserved],
            WideFloorWitness {
                q: witness.q - 1,
                r: witness.r + witness.d + extra,
                ..witness
            }
            .encode(),
            Rejected,
        )?);
    }

    // -- The divisor --------------------------------------------------

    rows.push(Recipe::row(
        &script,
        "zero_divisor",
        &[K::WideFloorZeroDivisorRejectedObserved],
        WideFloorWitness {
            d: 0,
            q: 0,
            r: 0,
            ..witness
        }
        .encode(),
        Rejected,
    )?);

    // -- The domain, at and past each end -----------------------------

    for (name, index) in [
        ("negative_first_factor", 0_usize),
        ("negative_divisor", 3),
        ("negative_remainder", 4),
    ] {
        rows.push(Recipe::row(
            &script,
            name,
            &[K::WideFloorOperandDomainObserved],
            with_item(&witness, index, (-1_i64).to_le_bytes().to_vec()),
            Rejected,
        )?);
    }

    for (name, index) in [
        ("first_factor_at_the_domain_bound", 0_usize),
        ("second_factor_at_the_domain_bound", 1),
        ("quotient_at_the_domain_bound", 2),
        ("divisor_at_the_domain_bound", 3),
    ] {
        rows.push(Recipe::row(
            &script,
            name,
            &[K::WideFloorOperandDomainObserved],
            with_item(&witness, index, AMOUNT_DOMAIN.to_le_bytes().to_vec()),
            Rejected,
        )?);
    }

    // -- The encoding -------------------------------------------------

    let canonical = witness.a.to_le_bytes();
    for (name, bytes) in [
        ("malformed_seven_byte_operand", canonical[..7].to_vec()),
        ("malformed_nine_byte_operand", {
            let mut wide = canonical.to_vec();
            wide.push(0);
            wide
        }),
        ("byte_reversed_operand", {
            let mut reversed = canonical.to_vec();
            reversed.reverse();
            reversed
        }),
    ] {
        rows.push(Recipe::row(
            &script,
            name,
            &[K::WideFloorOperandEncodingObserved],
            with_item(&witness, 0, bytes),
            Rejected,
        )?);
    }

    // -- The witness's shape ------------------------------------------

    {
        // The right five values in the wrong places: the two factors
        // exchanged with the quotient and the divisor.
        let reordered = WideFloorWitness {
            a: witness.q,
            b: witness.d,
            q: witness.a,
            d: witness.b,
            r: witness.r,
        };
        rows.push(Recipe::row(
            &script,
            "witness_items_reordered",
            &[K::WideFloorWitnessOrderObserved],
            reordered.encode(),
            Rejected,
        )?);
    }

    {
        // The quotient and the remainder exchanged, which is the
        // reordering an honest caller is most likely to make.
        let swapped = WideFloorWitness {
            q: witness.r,
            r: witness.q,
            ..witness
        };
        rows.push(Recipe::row(
            &script,
            "quotient_and_remainder_exchanged",
            &[K::WideFloorWitnessOrderObserved],
            swapped.encode(),
            Rejected,
        )?);
    }

    {
        // One amount standing in for another, which is the nearest this
        // candidate comes to a duplicated limb: there is no witnessed
        // limb to duplicate, so the duplication happens one level up.
        let duplicated = WideFloorWitness {
            q: witness.a,
            ..witness
        };
        rows.push(Recipe::row(
            &script,
            "duplicated_amount_in_place_of_the_quotient",
            &[K::WideFloorWitnessOrderObserved],
            duplicated.encode(),
            Rejected,
        )?);
    }

    for (name, items) in [
        ("witness_item_omitted", {
            let mut short = witness.encode();
            short.pop();
            short
        }),
        ("additional_witness_item", {
            let mut long = witness.encode();
            long.push(witness.r.to_le_bytes().to_vec());
            long
        }),
    ] {
        rows.push(Recipe::row(
            &script,
            name,
            &[K::WideFloorWitnessOrderObserved],
            items,
            Rejected,
        )?);
    }

    // -- The schedule itself ------------------------------------------

    rows.push(Recipe::row(
        &recipe.unchecked_flag_script,
        "arithmetic_flag_left_unchecked",
        &[K::WideFloorUncheckedFlagRejectedObserved],
        witness.encode(),
        Rejected,
    )?);

    // Every row is stated against the reviewed revision, set once here
    // rather than repeated at each row where one could drift.
    let version = target.definition().version().get();
    for row in &mut rows {
        row.target_contract_version = version;
    }

    Ok(rows)
}

/// Which cases bear on each wide-floor claim.
///
/// The inverse of the fixtures' own claim sets, computed rather than
/// restated `(´[PLAN-rule:guide10:claim-coverage]´)`.
#[must_use]
pub fn bearing_cases(
    matrix: &[CompoundPrototypeFixture],
) -> BTreeMap<PrototypeClaim, BTreeSet<PrototypeCaseId>> {
    let mut bearing: BTreeMap<PrototypeClaim, BTreeSet<PrototypeCaseId>> = BTreeMap::new();
    for fixture in matrix {
        for claim in &fixture.claims {
            bearing
                .entry(*claim)
                .or_default()
                .insert(fixture.case.clone());
        }
    }
    bearing
}

/// What the matrix deliberately does not state, and why.
///
/// Enumerable rather than inferred from an absence, which is the same
/// discipline the claim registry follows: a reader can see exactly which
/// corners of the threat matrix have no case here
/// `(´[PLAN-rule:guide10:claim-registry]´)`.
#[must_use]
pub const fn residual_threats() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "proof over a substituted first factor",
            "the substitution is refused by whatever operation authenticates the amount before \
             handing it to this pattern, and a standalone pattern has no such enclosing binding",
        ),
        (
            "proof over a substituted second factor",
            "as above: an enclosing-binding rejection, not a rejection this program can make",
        ),
        (
            "proof over a substituted divisor",
            "as above: an enclosing-binding rejection, not a rejection this program can make",
        ),
        (
            "a low or high limb above its bound",
            "no limb is witnessed. Both limbs of every amount are the target's own Euclidean \
             division by the base, so the bound is the primitive's contract rather than a check a \
             caller could evade",
        ),
        (
            "a wrong first or second carry",
            "no carry is witnessed, for the same reason: each carry is the quotient the same \
             division returns above its remainder",
        ),
        (
            "an omitted or duplicated product limb",
            "no product limb is witnessed. The nearest expressible row is a duplicated amount, \
             which is stated",
        ),
    ]
}
