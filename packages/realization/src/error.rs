use architecture::{BoundId, ManifestError, ObjectId, OperationId};
use thiserror::Error;

use crate::{
    ConstructibilityNodeId, DisclosureNodeId, ExprId, FactId, LifecycleNodeId, ObservedObjectRef,
    RelationId, RepresentationMode, SemanticType,
};

/// Architecture field whose declared shape no longer matches a realization weld.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchitectureMismatchField {
    Authorization,
    InputFamilies,
    OutputFamilies,
    AshInput,
    AshOutput,
    ReceiptInput,
    ReceiptOutput,
    SponsorInput,
    SponsorOutput,
    Bounds,
    CanonicalDelta,
    CanonicalDeltas,
    DataOutputs,
    OpenFlows,
    ValueFlows,
    RootPolicy,
    ProjectionPolicy,
    Witnesses,
}

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

    /// Declared expression dependencies contain at least one cycle.
    #[error("expression dependency graph contains one or more cycles")]
    ExpressionDependencyCycle { components: Vec<Vec<ExprId>> },

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
    #[error("expression {expression:?} has too many operands")]
    TooManyExpressionOperands { expression: ExprId },

    /// A scoped operation has no realization declaration in this tranche.
    #[error("operation {0} has no realization declaration")]
    UnsupportedOperationDeclaration(OperationId),

    /// One operation declaration was produced more than once.
    #[error("operation {0} is declared more than once")]
    DuplicateOperationDeclaration(OperationId),

    /// An architecture operation needed by realization is absent.
    #[error("architecture operation {0} is missing")]
    MissingArchitectureOperation(OperationId),

    /// A realization weld disagrees with one architecture operation field.
    #[error("architecture operation {operation} mismatches realization field {field:?}")]
    ArchitectureOperationMismatch {
        operation: OperationId,
        field: ArchitectureMismatchField,
    },

    /// One relation ID was declared more than once.
    #[error("relation {0:?} is declared more than once")]
    DuplicateRelation(RelationId),

    /// One relation referenced an undeclared dependency.
    #[error("relation {relation:?} references unknown dependency {dependency:?}")]
    UnknownRelationDependency {
        relation: RelationId,
        dependency: RelationId,
    },

    /// Declared relation dependencies contain at least one cycle.
    #[error("relation dependency graph contains one or more cycles")]
    RelationDependencyCycle { components: Vec<Vec<RelationId>> },

    /// A relation result was requested outside the relation graph.
    #[error("relation {0:?} is unknown")]
    UnknownRelation(RelationId),

    /// A required runtime bound value was absent from an observation.
    #[error("bound {0} has no supplied runtime value")]
    MissingBoundValue(BoundId),

    /// A flow or relation referenced an unknown observed object.
    #[error("observed object {0:?} is unknown")]
    UnknownObservedObject(ObservedObjectRef),

    #[error("constructibility node {0:?} is declared more than once")]
    DuplicateConstructibilityNode(ConstructibilityNodeId),

    #[error("constructibility node {0:?} is unknown")]
    UnknownConstructibilityNode(ConstructibilityNodeId),

    #[error("constructibility graph contains one or more cycles")]
    ConstructibilityCycle {
        components: Vec<Vec<ConstructibilityNodeId>>,
    },

    #[error("operation {0} has no constructibility node")]
    MissingConstructibilityOperation(OperationId),

    #[error("permissionless operation {operation} depends on private source {source_node:?}")]
    PermissionlessPrivateDependency {
        operation: OperationId,
        source_node: ConstructibilityNodeId,
    },

    #[error("sponsor-local dependency {source_node:?} escaped sponsor-only edge for {operation}")]
    SponsorDependencyEscaped {
        operation: OperationId,
        source_node: ConstructibilityNodeId,
    },

    #[error("lifecycle node {0:?} is declared more than once")]
    DuplicateLifecycleNode(LifecycleNodeId),

    #[error("lifecycle node {0:?} is unknown")]
    UnknownLifecycleNode(LifecycleNodeId),

    #[error("lifecycle graph contains one or more cycles")]
    LifecycleCycle {
        components: Vec<Vec<LifecycleNodeId>>,
    },

    #[error("missing lifecycle representation {object:?} {mode:?}")]
    MissingLifecycleRepresentation {
        object: ObjectId,
        mode: RepresentationMode,
    },

    #[error("missing lifecycle exit node {object:?} {exit}")]
    MissingLifecycleExitNode { object: ObjectId, exit: OperationId },

    #[error("missing lifecycle path {object:?} {mode:?} -> {exit}")]
    MissingLifecyclePath {
        object: ObjectId,
        mode: RepresentationMode,
        exit: OperationId,
    },

    #[error("disclosure node {0:?} is declared more than once")]
    DuplicateDisclosureNode(DisclosureNodeId),

    #[error("disclosure node {0:?} is unknown")]
    UnknownDisclosureNode(DisclosureNodeId),

    #[error("disclosure graph contains one or more cycles")]
    DisclosureCycle {
        components: Vec<Vec<DisclosureNodeId>>,
    },

    #[error("representation/lifecycle mismatch for {object:?} {mode:?}")]
    RepresentationLifecycleMismatch {
        object: ObjectId,
        mode: RepresentationMode,
    },
}
