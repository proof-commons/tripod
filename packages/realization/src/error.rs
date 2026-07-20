use architecture::{ManifestError, OperationId};
use thiserror::Error;

use crate::{ExprId, FactId, SemanticType};

/// Failure while constructing or validating target-independent
/// realization values.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RealizationError {
    /// The architecture failed its own structural validation.
    #[error("architecture validation failed with {} error(s)", errors.len())]
    ArchitectureValidationFailed { errors: Vec<ManifestError> },

    /// The architecture semantic hash could not be derived.
    #[error("architecture semantic hash is unavailable")]
    ArchitectureHashUnavailable,

    /// A realization scope contained no operations.
    #[error("realization scope is empty")]
    EmptyScope,

    /// One operation occurred more than once in a scope declaration.
    #[error("operation {0} occurs more than once in the realization scope")]
    DuplicateScopeOperation(OperationId),

    /// A scoped operation is absent from the supplied architecture.
    #[error("operation {0} is outside the supplied architecture")]
    OperationOutsideArchitecture(OperationId),

    /// A complete realization was requested from a partial scope.
    #[error(
        "realization scope is incomplete; {} architecture operation(s) are missing",
        missing.len()
    )]
    IncompleteScope { missing: Vec<OperationId> },

    /// A protocol amount was outside the realization amount domain.
    #[error("protocol amount {value} is outside the domain 0 <= v < 2^51")]
    AmountOutOfDomain { value: u64 },

    /// Checked protocol-amount addition overflowed.
    #[error("checked protocol-amount arithmetic overflowed")]
    AmountOverflow,

    /// Checked protocol-amount subtraction underflowed.
    #[error("checked protocol-amount arithmetic underflowed")]
    AmountUnderflow,

    /// Checked count arithmetic overflowed.
    #[error("checked count arithmetic overflowed")]
    CountOverflow,

    /// One expression ID was declared more than once.
    #[error("expression {0:?} is declared more than once")]
    DuplicateExpression(ExprId),

    /// One expression referenced an undeclared dependency.
    #[error("expression {expression:?} references unknown dependency {dependency:?}")]
    UnknownExpressionDependency {
        expression: ExprId,
        dependency: ExprId,
    },

    /// Declared expression dependencies contain a cycle.
    #[error(
        "expression dependency graph contains a cycle involving {} expression(s)",
        nodes.len()
    )]
    ExpressionDependencyCycle { nodes: Vec<ExprId> },

    /// One expression's declared type differs from its node type.
    #[error("expression {expression:?} has type {actual:?}, expected {expected:?}")]
    ExpressionTypeMismatch {
        expression: ExprId,
        expected: SemanticType,
        actual: SemanticType,
    },

    /// One expression operand has the wrong semantic type.
    #[error("expression operand {expression:?} has type {actual:?}, expected {expected:?}")]
    ExpressionOperandTypeMismatch {
        expression: ExprId,
        expected: SemanticType,
        actual: SemanticType,
    },

    /// Two operands of one binary expression have incompatible types.
    #[error(
        "binary operands {left:?} and {right:?} have incompatible types {left_type:?} and {right_type:?}"
    )]
    BinaryOperandTypeMismatch {
        left: ExprId,
        right: ExprId,
        left_type: SemanticType,
        right_type: SemanticType,
    },

    /// A checked sum was declared over a nonnumeric type.
    #[error("checked sum is not defined for semantic type {0:?}")]
    InvalidSumType(SemanticType),

    /// An ordered comparison was declared over an unordered type.
    #[error("ordered comparison is not defined for semantic type {0:?}")]
    InvalidOrderedType(SemanticType),

    /// A fact expression's ID does not equal the fact's stable ID.
    #[error("fact expression {expression:?} does not use the fact's stable identity {fact:?}")]
    FactExpressionIdentityMismatch { expression: ExprId, fact: FactId },

    /// One primitive fact value was supplied more than once.
    #[error("fact {0:?} was supplied more than once")]
    DuplicateFactValue(FactId),

    /// A primitive fact value has the wrong type.
    #[error("fact {fact:?} has value type {actual:?}, expected {expected:?}")]
    FactTypeMismatch {
        fact: FactId,
        expected: SemanticType,
        actual: SemanticType,
    },

    /// A required primitive fact value was absent.
    #[error("fact {0:?} has no supplied value")]
    MissingFactValue(FactId),

    /// An expression result was requested before or outside complete
    /// evaluation.
    #[error("expression {expression:?} has no evaluated value")]
    UnknownEvaluatedExpression { expression: ExprId },

    /// One expression has too many ordered operands to record in the
    /// typed dependency edge.
    #[error("expression {expression:?} has too many operands: {count}")]
    TooManyExpressionOperands { expression: ExprId, count: usize },
}
