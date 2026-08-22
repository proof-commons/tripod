//! Typed refusals, separated by the layer that produced them.
//!
//! Guide-12 §1.5 keeps construction failure and target rejection
//! distinct, and the separation only survives if the failures are
//! themselves distinguishable. [`FixtureBundleRefusal`] therefore wraps
//! each producing layer's own error rather than flattening them into a
//! message, so a caller can tell a compiler refusal from a link refusal
//! from an ABI refusal without reading prose.

use compiler::CompileError;
use linker::LinkRefusal;
use realization::{RealizationError, RelationId};
use tapscript::bundle::BundleRefusal;
use tapscript::error::TapscriptError;
use target_elements::TargetError;
use transaction::TransactionRefusal;

use crate::fixture::SemanticFixtureId;
use crate::materialize::TargetVectorId;

/// A refusal encountered while building the fixture linked bundle.
///
/// Each variant names the layer that refused. §1.5's report contract
/// asks for exactly this: a claim about the target requires a target,
/// and a refusal from the linker must not be filed as one.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixtureBundleRefusal {
    /// The reviewed target contract did not validate.
    Target(Vec<TargetError>),
    /// The realization did not derive.
    Realization(RealizationError),
    /// The compiler refused to bind input or to plan the operation.
    Compile(CompileError),
    /// A deployment or placeholder symbol was not the reviewed width.
    Symbols(TapscriptError),
    /// The backend refused to emit the candidate bundle.
    Emission(BundleRefusal),
    /// The linker refused to link.
    Link(LinkRefusal),
    /// The ABI refused to derive from the linked bundle.
    Abi(TransactionRefusal),
    /// The taproot tweak of this bundle's own tree is not a key.
    ///
    /// Reachable only for a ceremony-bound bundle, whose pin is derived
    /// rather than stated. It carries no cause: the oracle's defect
    /// vocabulary is that package's own, and restating it here would be
    /// a second authored spelling of a distinction this package neither
    /// owns nor acts on `(´[PLAN-rule:guide12-exec:typed-source]´)`.
    Tweak,
}

/// A refusal encountered while assembling or checking evidence.
///
/// The census variants carry both numbers rather than a boolean, because
/// a census that disagrees is a finding a reader has to be able to size.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum VectorError {
    /// The fixture bundle could not be built at all.
    FixtureBundleUnavailable(FixtureBundleRefusal),
    /// The relation census derived from the plan is not the one stated.
    RelationCensusMismatch {
        /// The number the plan itself publishes.
        derived: usize,
        /// The number this package recomputed.
        recomputed: usize,
    },
    /// The execution-case census disagrees.
    CaseCensusMismatch {
        /// The number the plan itself publishes.
        derived: usize,
        /// The number this package recomputed.
        recomputed: usize,
    },
    /// The relation-case census disagrees.
    RelationCaseCensusMismatch {
        /// The number the plan itself publishes.
        derived: usize,
        /// The number this package recomputed.
        recomputed: usize,
    },
    /// A relation appears in the coverage set but not in the plan.
    UnexpectedRelation(RelationId),
    /// A planned relation has no coverage requirement at all.
    MissingRelation(RelationId),
    /// Two coverage requirements claimed one identity.
    DuplicateCoverageRequirement,
    /// A vector could not be decoded and re-encoded to its own bytes,
    /// so no mutation of it would carry exactly one difference.
    UnmutatableVector(TargetVectorId),
    /// A negative coverage requirement's evidence role names no place
    /// its refusal could be observed.
    ///
    /// Refused rather than filed under whichever column looked closest:
    /// a requirement this package cannot place is one it cannot honestly
    /// count, and a default arm would make the negative census add up
    /// while meaning nothing.
    UnclassifiableNegativeRequirement,
    /// Canonical first-party evidence could not be built or validated.
    ///
    /// The first-party module owns the refusal vocabulary for both the
    /// canonical case and the validator run, so this error preserves
    /// that typed cause rather than filing it as a coverage-role
    /// classification failure.
    FirstPartyEvidenceRefused {
        /// Which first-party precondition failed.
        cause: crate::first_party::FirstPartyEvidenceRefusal,
    },
    /// Two semantic fixtures claimed one identity.
    DuplicateSemanticFixture(SemanticFixtureId),
    /// A fixture's stated facts are not a model-valid compact-ASH world.
    InvalidSemanticFixture(SemanticFixtureId),
    /// A fixture's stated facts do not witness the class it is filed
    /// under.
    ///
    /// Distinct from [`Self::InvalidSemanticFixture`], and the
    /// difference is the whole of `G13-R10`: the world may be perfectly
    /// valid and still fail to exhibit the property its §18.1 class name
    /// asserts, in which case a run of the row is evidence about
    /// something other than what it is filed as.
    FixtureContradictsItsClass {
        /// The row whose facts and name disagree.
        fixture: SemanticFixtureId,
        /// The class it is filed under.
        class: &'static str,
    },
    /// The expected semantic result could not be derived from the
    /// realization layer's own domain.
    ExpectationNotDerivable {
        /// The fixture whose expectation failed.
        fixture: SemanticFixtureId,
        /// The realization layer's own refusal.
        cause: RealizationError,
    },
    /// A target vector could not be materialized.
    TargetMaterializationFailed {
        /// The vector that failed.
        vector: TargetVectorId,
        /// The transaction layer's own refusal.
        cause: TransactionRefusal,
    },
    /// The materialized transaction does not carry the shape the
    /// fixture's own facts imply.
    MaterializedShapeMismatch(TargetVectorId),
    /// The successor amount the constructor settled on is not the one
    /// the realization layer's arithmetic derived.
    SuccessorAmountMismatch {
        /// The vector that disagreed.
        vector: TargetVectorId,
        /// The realization layer's derivation.
        expected: u64,
        /// What the constructor settled on.
        settled: u64,
    },
    /// A named §18 class was claimed by no case, or by more than one.
    MatrixCoverageMismatch {
        /// The class name in question.
        class: &'static str,
    },
    /// The ceremony supplied a number of coins this vector cannot take.
    FundingCardinalityMismatch {
        /// The vector the coins were offered to.
        vector: TargetVectorId,
        /// How many ASH inputs its shape carries.
        wanted: usize,
        /// How many coins were offered.
        supplied: usize,
    },
    /// The ceremony supplied one coin twice for a single vector.
    DuplicateFundedOutpoint(TargetVectorId),
    /// A declared intended violation matched no requirement, or several.
    ///
    /// The declaration and the published plan disagree about what
    /// exists: either the relation no longer publishes that mutation
    /// class, or it publishes more than one and the arm cannot say which
    /// it meant. Both are answered by fixing the declaration, never by
    /// taking whichever row came first.
    NegativeLinkUnresolved {
        /// The semantic mutation class the arm declared.
        class: &'static str,
    },
    /// A declaration resolved, and the requirement it found describes
    /// something else.
    ///
    /// The relation and the mutation class agreed and one of the other
    /// six §4.1 statements did not, which means the link was established
    /// by two fields while the rest of the declaration was wrong about
    /// the row. Answered by fixing whichever side is stale, never by
    /// dropping the statement that disagreed.
    NegativeLinkContradicted {
        /// The semantic mutation class the arm declared.
        class: &'static str,
        /// Which stated expectation the published requirement denies.
        expectation: crate::violation::ContradictedExpectation,
    },
    /// A withheld class changes several target fields at once.
    ///
    /// The ABI-validation boundary pins one field, so a class that moves
    /// more than one cannot be settled there. Answered by narrowing the
    /// class or by giving the boundary a second thing to pin, never by
    /// picking whichever field the reading happened to reach first.
    AbiValidationFieldUnderdetermined {
        /// The class whose declaration names several fields.
        class: crate::mutation::NegativeMutation,
    },
    /// Funding cut for one vector was offered to another.
    FundingNamesAnotherVector {
        /// The vector being materialized.
        wanted: TargetVectorId,
        /// The vector the funding names.
        supplied: TargetVectorId,
    },
}

impl From<FixtureBundleRefusal> for VectorError {
    fn from(refusal: FixtureBundleRefusal) -> Self {
        Self::FixtureBundleUnavailable(refusal)
    }
}
