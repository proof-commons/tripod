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
    /// Two semantic fixtures claimed one identity.
    DuplicateSemanticFixture(SemanticFixtureId),
    /// A fixture's stated facts are not a model-valid compact-ASH world.
    InvalidSemanticFixture(SemanticFixtureId),
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
}

impl From<FixtureBundleRefusal> for VectorError {
    fn from(refusal: FixtureBundleRefusal) -> Self {
        Self::FixtureBundleUnavailable(refusal)
    }
}
