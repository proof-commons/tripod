//! The compiler's single error root.
//!
//! Analysis failures are typed and never silently weakened: an
//! analysis that cannot complete returns an error rather than dropping
//! a relation, weakening an authorization, or falling back to a
//! partial result.
//!
//! Only the input-boundary variants exist so far. They are the
//! failures the analyzed input boundary must be able to report, and
//! each one is exercised by the public-API test. The enum is
//! deliberately `non_exhaustive`, so the later analysis stages extend
//! this vocabulary without a breaking change and without this file
//! having to guess their shapes now.

use architecture::OperationId;
use thiserror::Error;

use crate::relation::RelationCycleComponent;

/// A typed compilation failure.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompileError {
    /// The realization was built against a realization schema this
    /// compiler does not support.
    ///
    /// Reserved: the typed realization currently exposes no schema
    /// value the binder can observe, so no code path constructs this
    /// today. It exists so the failure has a stable home when a
    /// realization schema is introduced; it is not implemented
    /// evidence.
    #[error("unsupported realization schema {schema}")]
    UnsupportedRealizationSchema {
        /// The schema version carried by the offered realization.
        schema: u32,
    },

    /// The offered realization did not validate under its owner's
    /// validator. The compiler re-runs that validator rather than
    /// trusting the value it was handed.
    #[error("realization failed its owner's validation")]
    InvalidRealization,

    /// The realization binds an architecture identity other than the
    /// one the compilation was requested against.
    #[error("realization architecture binding does not match the requested architecture")]
    ArchitectureBindingMismatch,

    /// The requested compilation scope names an operation the offered
    /// realization does not declare.
    #[error("requested scope operation {operation:?} is absent from the realization")]
    IncompleteRealizationScope {
        /// The operation named by the scope but missing from the input.
        operation: OperationId,
    },

    /// The requested compilation scope names one operation more than
    /// once. Scope is a set; a repeated member is a caller defect, not
    /// a value to normalize away silently.
    #[error("requested scope names operation {operation:?} more than once")]
    DuplicateScopeOperation {
        /// The repeated operation.
        operation: OperationId,
    },

    /// The requested compilation scope names no operation at all.
    #[error("requested compilation scope is empty")]
    EmptyCompilationScope,

    /// One relation ID occurred more than once in the scoped source.
    #[error("relation {relation:?} occurs more than once in the scoped source")]
    DuplicateRelation {
        /// The repeated realization-owned relation.
        relation: realization::RelationId,
    },

    /// A dependency edge names a relation absent from the scoped graph.
    #[error("relation dependency endpoint {relation:?} is absent from the scoped graph")]
    UnknownRelationDependencyEndpoint {
        /// The missing stable relation ID (never a graph index).
        relation: realization::RelationId,
    },

    /// One identical dependency declaration occurred more than once.
    #[error("relation dependency {prerequisite:?} -> {dependent:?} is declared more than once")]
    DuplicateRelationDependency {
        /// The prerequisite relation.
        prerequisite: realization::RelationId,
        /// The dependent relation.
        dependent: realization::RelationId,
        /// The repeated edge kind.
        edge: realization::RelationEdge,
    },

    /// A source dependency crosses the requested compiler boundary.
    ///
    /// A crossing is rejected, never silently dropped: an explicit
    /// import design may later admit operation-independent
    /// dependencies, but no crossing is accepted implicitly.
    #[error(
        "relation dependency {prerequisite:?} -> {dependent:?} escapes the requested compiler scope"
    )]
    RelationDependencyEscapesScope {
        /// The prerequisite relation.
        prerequisite: realization::RelationId,
        /// The dependent relation.
        dependent: realization::RelationId,
    },

    /// The scoped relation dependencies contain a cycle.
    #[error("relation dependencies contain {} cyclic component(s)", components.len())]
    RelationDependencyCycle {
        /// Canonically ordered cyclic components in stable IDs.
        components: Vec<RelationCycleComponent>,
    },

    /// The graph census does not equal the expected scoped census.
    #[error(
        "relation census mismatch: {} missing, {} unexpected",
        missing.len(),
        unexpected.len()
    )]
    RelationCensusMismatch {
        /// Expected relations absent from the graph, sorted.
        missing: Vec<realization::RelationId>,
        /// Graph relations absent from the expectation, sorted.
        unexpected: Vec<realization::RelationId>,
    },

    /// A proof alternative names a relation other than its owner.
    #[error("proof alternative on {relation:?} is bound to {alternative_relation:?}")]
    ForeignProofAlternative {
        /// The relation carrying the alternative.
        relation: realization::RelationId,
        /// The relation the alternative actually names.
        alternative_relation: realization::RelationId,
    },

    /// One expression ID occurred more than once in the scoped source.
    #[error("expression {expression:?} occurs more than once in the scoped source")]
    DuplicateExpression {
        /// The repeated realization-owned expression.
        expression: realization::ExprId,
    },

    /// One identical expression dependency occurred more than once.
    #[error("expression dependency {dependency:?} -> {consumer:?} is declared more than once")]
    DuplicateExpressionDependency {
        /// The dependency expression.
        dependency: realization::ExprId,
        /// The consuming expression.
        consumer: realization::ExprId,
    },

    /// A dependency edge names an expression absent from the source.
    #[error("expression dependency endpoint {expression:?} is absent from the scoped source")]
    UnknownExpressionDependencyEndpoint {
        /// The missing stable expression ID (never a graph index).
        expression: realization::ExprId,
    },

    /// An in-scope expression depends on another operation's expression.
    #[error("expression dependency {dependency:?} of {consumer:?} escapes the compiler scope")]
    ExpressionDependencyEscapesScope {
        /// The out-of-scope dependency.
        dependency: realization::ExprId,
        /// The in-scope consumer.
        consumer: realization::ExprId,
    },

    /// The scoped expression dependencies contain a cycle.
    #[error("expression dependencies contain {} cyclic component(s)", components.len())]
    ExpressionDependencyCycle {
        /// Canonically ordered cyclic components in stable IDs.
        components: Vec<crate::expression::ExpressionCycleComponent>,
    },

    /// The graph census does not equal the expected scoped closure.
    #[error(
        "expression census mismatch: {} missing, {} unexpected",
        missing.len(),
        unexpected.len()
    )]
    ExpressionCensusMismatch {
        /// Expected expressions absent from the graph, sorted.
        missing: Vec<realization::ExprId>,
        /// Graph expressions absent from the expectation, sorted.
        unexpected: Vec<realization::ExprId>,
    },

    /// A relation predicate names an expression absent from the graph.
    #[error("relation {relation:?} names unknown predicate expression {expression:?}")]
    UnknownPredicateExpression {
        /// The relation carrying the predicate.
        relation: realization::RelationId,
        /// The missing predicate expression.
        expression: realization::ExprId,
    },

    /// A relation predicate expression is not boolean.
    #[error("predicate expression {expression:?} of {relation:?} has type {actual:?}")]
    NonBooleanPredicateExpression {
        /// The relation carrying the predicate.
        relation: realization::RelationId,
        /// The predicate expression.
        expression: realization::ExprId,
        /// The expression's actual semantic type.
        actual: realization::SemanticType,
    },

    /// A relation predicate belongs to another operation.
    #[error("predicate expression {expression:?} of {relation:?} belongs to another operation")]
    PredicateExpressionOutsideScope {
        /// The relation carrying the predicate.
        relation: realization::RelationId,
        /// The foreign predicate expression.
        expression: realization::ExprId,
    },

    /// Constant folding met operands of incompatible semantic types.
    #[error("constant fold of {expression:?} expected {expected:?}, found {actual:?}")]
    ConstantFoldTypeMismatch {
        /// The expression being folded.
        expression: realization::ExprId,
        /// The expected semantic type.
        expected: realization::SemanticType,
        /// The actual semantic type met.
        actual: realization::SemanticType,
    },

    /// A closed checked sum overflowed its integer domain.
    #[error("constant fold of {expression:?} overflowed")]
    ConstantFoldOverflow {
        /// The overflowing expression.
        expression: realization::ExprId,
    },

    /// A closed checked amount sum left the protocol-amount domain.
    #[error("constant fold of {expression:?} produced out-of-domain amount {value}")]
    ConstantFoldAmountOutOfDomain {
        /// The out-of-domain expression.
        expression: realization::ExprId,
        /// The offending total.
        value: u64,
    },

    /// Folding met an operand absent from the analysis graph.
    #[error("constant fold of {expression:?} is missing operand {operand:?}")]
    ConstantFoldMissingOperand {
        /// The expression being folded.
        expression: realization::ExprId,
        /// The absent operand.
        operand: realization::ExprId,
    },

    /// A relation operand derived no authenticatable source row.
    #[error("relation {relation:?} operand {operand:?} has no source requirement")]
    MissingSourceRequirement {
        /// The relation being planned.
        relation: realization::RelationId,
        /// The sourceless operand.
        operand: crate::source::OperandId,
    },

    /// A source row failed authentication classification.
    #[error("relation {relation:?} operand {operand:?} has no authenticated source")]
    UnauthenticatedSourceRequirement {
        /// The relation being planned.
        relation: realization::RelationId,
        /// The unauthenticated operand.
        operand: crate::source::OperandId,
    },

    /// A witness-bearing source has no constructibility counterpart.
    #[error("relation {relation:?} operand {operand:?} mismatches constructibility")]
    SourceConstructibilityMismatch {
        /// The relation being planned.
        relation: realization::RelationId,
        /// The mismatched operand.
        operand: crate::source::OperandId,
    },

    /// An individual sponsor amount reached compiler analysis.
    ///
    /// Sponsor erasure means the value is structurally absent, not
    /// merely secret; any occurrence is a defect, never data.
    #[error("compiler analysis attempted to read an erased sponsor value")]
    SponsorValueRead,
}
