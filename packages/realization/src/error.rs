use architecture::{AssetId, BoundId, ManifestError, ObjectId, OperationId, RootId};
use thiserror::Error;

use crate::{
    ConstructibilityAuthorization, ConstructibilityDependencyDeclaration, ConstructibilityNodeId,
    DisclosureDependencyDeclaration, DisclosureNodeId, ExprId, FactId,
    LifecycleDependencyDeclaration, LifecycleNodeId, ObservedObjectRef,
    RelationDependencyDeclaration, RelationId, RelationKind, RelationSubject, RepresentationMode,
    SemanticType,
};

/// Architecture field whose declared shape no longer matches a realization weld.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchitectureMismatchField {
    OperationKind,
    Authorization,
    Issuances,
    Reads,
    Writes,
    InputFamilies,
    OutputFamilies,
    AshInput,
    AshOutput,
    ReceiptInput,
    ReceiptOutput,
    StateInput,
    StateOutput,
    StateObject,
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
    /// Two operation regions claim the same object.
    #[error("operation regions overlap at {0:?}")]
    ObservedRegionOverlap(ObservedObjectRef),

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

    /// A requested operation is not part of this realization's scope.
    #[error("operation {0} is outside the realization scope")]
    OperationOutsideScope(OperationId),

    /// A realization's architecture binding does not equal the binding
    /// derived from the supplied architecture.
    #[error("realization architecture binding does not match the supplied architecture")]
    ArchitectureBindingMismatch,

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

    /// Checked cycle arithmetic overflowed.
    #[error("cycle arithmetic overflowed")]
    CycleOverflow,

    /// Announcement lead bounds do not describe a usable inclusive
    /// window: a zero minimum, or a minimum above the maximum.
    #[error("announcement lead bounds are invalid: minimum {minimum}, maximum {maximum}")]
    InvalidAnnouncementLeadBounds { minimum: u64, maximum: u64 },

    /// A required architecture-keyed cycle lead was not supplied.
    #[error("missing announcement lead bound {0}")]
    MissingAnnouncementLeadBound(BoundId),

    /// A cycle bound appeared where a cardinality magnitude is required.
    #[error("cycle bound {0} cannot supply a count")]
    CycleBoundUsedAsCount(BoundId),

    /// An asset declaration does not declare a singleton: it fixes no
    /// amount, fixes zero, or admits reissuance.
    ///
    /// The offending declaration travels with the refusal, because the
    /// caller's next question is which asset and which field, and a
    /// sentence naming neither would send it back to the manifest to
    /// find out.
    #[error(
        "asset {asset} does not declare a singleton: fixed amount {fixed_amount:?}, reissuable {reissuable}"
    )]
    InvalidSingletonDeclaration {
        asset: AssetId,
        fixed_amount: Option<u64>,
        reissuable: bool,
    },

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

    /// An expression predicate depends on a fact of another operation.
    #[error("fact {fact:?} belongs to an operation other than observed {operation}")]
    ForeignExpressionFact {
        fact: FactId,
        operation: OperationId,
    },

    /// An owner-set fact covers an observed input that commits no owner.
    #[error("fact {fact:?} covers an observed input without a committed owner")]
    UnderivableOwnerFact { fact: FactId },

    /// An expression result was requested before or outside complete
    /// evaluation.
    #[error("expression {expression:?} has no evaluated value")]
    UnknownEvaluatedExpression { expression: ExprId },

    /// One expression has too many ordered operands to record in the
    /// typed dependency edge.
    #[error("expression {expression:?} has too many operands")]
    TooManyExpressionOperands { expression: ExprId },

    /// An expression-predicate relation names an expression its
    /// operation does not declare (F4-001). Checked during derivation so
    /// a missing predicate target fails before evaluation.
    #[error(
        "relation {relation:?} names predicate expression {expression:?}, \
         which operation {operation} does not declare"
    )]
    UnknownPredicateExpression {
        operation: OperationId,
        relation: RelationId,
        expression: ExprId,
    },

    /// An expression-predicate relation names a non-boolean expression
    /// (F4-001).
    #[error(
        "relation {relation:?} predicate expression {expression:?} has type \
         {actual:?}, expected Bool"
    )]
    NonBooleanPredicateExpression {
        relation: RelationId,
        expression: ExprId,
        actual: SemanticType,
    },

    /// A declaration consumed an ordinary sponsor L-BTC amount, which
    /// sponsor-value opacity removes from the protocol read-set.
    #[error("sponsor-value opacity violated: a declaration reads a PLAIN_LBTC family amount")]
    SponsorValueRead,

    /// A scoped operation has no realization declaration in this tranche.
    #[error("operation {0} has no realization declaration")]
    UnsupportedOperationDeclaration(OperationId),

    /// One operation declaration was produced more than once.
    #[error("operation {0} is declared more than once")]
    DuplicateOperationDeclaration(OperationId),

    /// A declaration was returned for a different operation than
    /// requested.
    #[error("declaration for operation {requested} identifies itself as {declared}")]
    OperationDeclarationIdentityMismatch {
        requested: OperationId,
        declared: OperationId,
    },

    /// An operation declared an expression owned by another operation.
    #[error("operation {operation} declares foreign expression {expression:?}")]
    ForeignExpressionOwnership {
        operation: OperationId,
        expression: ExprId,
    },

    /// An operation declared a relation owned by another operation.
    #[error("operation {operation} declares foreign relation {relation:?}")]
    ForeignRelationOwnership {
        operation: OperationId,
        relation: RelationId,
    },

    /// A relation dependency named a relation owned by another
    /// operation.
    #[error("operation {operation} declares a dependency on foreign relation {relation:?}")]
    ForeignRelationDependency {
        operation: OperationId,
        relation: RelationId,
    },

    /// A proof alternative points at a relation other than the one
    /// carrying it.
    #[error("relation {relation:?} carries a proof alternative for {foreign:?}")]
    ForeignProofAlternativeBinding {
        relation: RelationId,
        foreign: RelationId,
    },

    /// An operation declared a constructibility node or edge endpoint
    /// owned by another operation.
    #[error("operation {operation} declares foreign constructibility node {node:?}")]
    ForeignConstructibilityOwnership {
        operation: OperationId,
        node: ConstructibilityNodeId,
    },

    /// An operation declared a disclosure node, edge endpoint, or seed
    /// owned by another operation.
    #[error("operation {operation} declares foreign disclosure node {node:?}")]
    ForeignDisclosureOwnership {
        operation: OperationId,
        node: DisclosureNodeId,
    },

    /// A disclosure node, edge endpoint, or seed reason named a
    /// relation that the operation never declared, so a declassification
    /// requirement would rest on a relation absent from the semantic
    /// census. Ownership validation alone cannot catch this: the
    /// phantom relation carries the right operation.
    #[error("disclosure names relation {relation:?}, which is not declared")]
    UnknownDisclosureRelation { relation: RelationId },

    /// An architecture operation needed by realization is absent.
    #[error("architecture operation {0} is missing")]
    MissingArchitectureOperation(OperationId),

    /// A realization weld disagrees with one architecture operation field.
    #[error("architecture operation {operation} mismatches realization field {field:?}")]
    ArchitectureOperationMismatch {
        operation: OperationId,
        field: ArchitectureMismatchField,
    },

    /// An architecture-declared object family has no realization
    /// relation of the required kind.
    ///
    /// The architecture owns the family's cardinality bounds and its
    /// asset, and realization is what exposes them as the semantic
    /// relations compiler analysis consumes: an unexposed family would
    /// leave those architecture facts owned by no relation at all.
    #[error("operation {operation} declares no {kind:?} relation for subject {subject:?}")]
    MissingArchitectureRelation {
        operation: OperationId,
        kind: RelationKind,
        subject: RelationSubject,
    },

    /// A realization relation disagrees with the architecture row that
    /// owns its values.
    #[error("relation {relation:?} disagrees with its architecture object family")]
    ArchitectureRelationMismatch { relation: RelationId },

    /// An operation declares a cardinality or recognition relation that
    /// its architecture row does not call for.
    ///
    /// The other direction of the family census. Without it an
    /// operation could add a family relation for an object family the
    /// architecture never declared, and every check keyed on the
    /// expected rows would still pass while the surplus row travelled
    /// on as if it were architecture-owned.
    #[error("relation {relation:?} is an architecture-family relation the architecture omits")]
    SurplusArchitectureRelation { relation: RelationId },

    /// A relation's declared identity does not describe its body.
    ///
    /// The identity is the stable semantic key everything downstream
    /// is filed under, and the body is what actually evaluates. When
    /// they disagree there are two answers to what the relation is,
    /// and no rule downstream can be trusted to have picked the same
    /// one as the rule before it.
    #[error("relation {declared:?} has a body whose semantic kind is {expected:?}")]
    RelationKindMismatch {
        declared: RelationId,
        expected: RelationKind,
    },

    /// A relation's declared subject is not the subject its body
    /// constrains.
    ///
    /// The kind may agree while the subject names another family,
    /// asset, root, projection, or lifecycle exit than the one the body
    /// constrains — a transposed input/output side being the sharpest
    /// case, since both sides exist and both are well typed. The body
    /// determines exactly one subject, so the expected one is reported
    /// alongside the declared one rather than left to the reader.
    #[error("relation {declared:?} has a body whose subject is {expected:?}")]
    RelationSubjectMismatch {
        declared: RelationId,
        expected: RelationSubject,
    },

    /// An object family named by an architecture operation row has no
    /// object specification.
    #[error("architecture object {0:?} is missing")]
    MissingArchitectureObject(ObjectId),

    /// One relation ID was declared more than once.
    #[error("relation {0:?} is declared more than once")]
    DuplicateRelation(RelationId),

    /// One relation referenced an undeclared dependency.
    #[error("relation {relation:?} references unknown dependency {dependency:?}")]
    UnknownRelationDependency {
        relation: RelationId,
        dependency: RelationId,
    },

    /// One typed relation dependency edge was declared more than once.
    #[error("relation dependency {0:?} is declared more than once")]
    DuplicateRelationDependency(RelationDependencyDeclaration),

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

    /// An observation contains the same object reference more than once.
    #[error("observed object {0:?} appears more than once")]
    DuplicateObservedObject(ObservedObjectRef),

    /// An observation reuses one object reference inside a partition.
    #[error("observed object reference is duplicated inside a partition")]
    DuplicateObservedReference,

    /// An observation lists one root effect more than once.
    #[error("observed root {0} appears more than once")]
    DuplicateObservedRoot(RootId),

    /// An observed reference appears on the wrong transaction side.
    #[error("observed reference {0:?} appears on the wrong side")]
    WrongObservedReferenceSide(ObservedObjectRef),

    /// Canonical-delta source or destination partitions overlap.
    #[error("observed canonical delta partition overlaps")]
    ObservedCanonicalPartitionOverlap,

    /// Two open flows claim one object reference (SR2-07).
    #[error("observed reference {0:?} is claimed by two open flows")]
    ObservedOpenFlowOverlap(ObservedObjectRef),

    /// A CPFP anchor was claimed by an open flow (SR2-07).
    ///
    /// The anchor family exists precisely to stand outside the
    /// open-value partition, so this is a malformed reference rather
    /// than an operation-specific policy question.
    #[error("observed CPFP anchor {0:?} cannot join an open flow")]
    AnchorInObservedOpenFlow(ObservedObjectRef),

    #[error("constructibility node {0:?} is declared more than once")]
    DuplicateConstructibilityNode(ConstructibilityNodeId),

    #[error("constructibility node {0:?} is unknown")]
    UnknownConstructibilityNode(ConstructibilityNodeId),

    #[error("constructibility dependency {0:?} is declared more than once")]
    DuplicateConstructibilityDependency(ConstructibilityDependencyDeclaration),

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
        path: Vec<ConstructibilityNodeId>,
    },

    /// An operation authorization derived from architecture cannot be
    /// built from the operation's own input rows (for example, a
    /// refund-key operation with no refund-key input).
    #[error("operation {0} authorization cannot be derived from its architecture inputs")]
    ConstructibilityAuthorizationMismatch(OperationId),

    /// A constructibility dependency requires a witness that the
    /// operation's authorization does not provide.
    #[error(
        "operation {operation} dependency {source_node:?} is unavailable under authorization {authorization:?}"
    )]
    ConstructibilityWitnessUnavailable {
        operation: OperationId,
        authorization: ConstructibilityAuthorization,
        source_node: ConstructibilityNodeId,
        path: Vec<ConstructibilityNodeId>,
    },

    #[error("sponsor-local dependency {source_node:?} escaped sponsor-only edge for {operation}")]
    SponsorDependencyEscaped {
        operation: OperationId,
        source_node: ConstructibilityNodeId,
        path: Vec<ConstructibilityNodeId>,
    },

    #[error(
        "constructibility dependency {source_node:?} belongs to another operation than {operation}"
    )]
    CrossOperationConstructibilityDependency {
        operation: OperationId,
        source_node: ConstructibilityNodeId,
        path: Vec<ConstructibilityNodeId>,
    },

    #[error("lifecycle node {0:?} is declared more than once")]
    DuplicateLifecycleNode(LifecycleNodeId),

    #[error("lifecycle node {0:?} is unknown")]
    UnknownLifecycleNode(LifecycleNodeId),

    #[error("lifecycle dependency {0:?} is declared more than once")]
    DuplicateLifecycleDependency(LifecycleDependencyDeclaration),

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

    /// A lifecycle `RequiresExit` edge is not shaped
    /// representation -> required-exit (F4-002): a reversed,
    /// representation-to-representation, or exit-to-exit edge.
    #[error(
        "lifecycle edge {source_node:?} -> {target_node:?} is not a \
         representation-to-required-exit edge"
    )]
    MalformedLifecycleEdge {
        source_node: LifecycleNodeId,
        target_node: LifecycleNodeId,
    },

    /// A lifecycle `RequiresExit` edge connects two different objects
    /// (F4-002).
    #[error("lifecycle edge {source_node:?} -> {target_node:?} crosses object families")]
    CrossObjectLifecycleEdge {
        source_node: LifecycleNodeId,
        target_node: LifecycleNodeId,
    },

    /// A lifecycle graph node no semantic relation declares (SR2-03).
    ///
    /// The lifecycle graph must be exactly what the `Representation`
    /// and `LifecycleExit` relations say it is. An orphan node is the
    /// residue of a lifecycle relation that was dropped while its graph
    /// path survived — the coherent omission that let a semantic
    /// obligation leave the relation census unnoticed.
    #[error("lifecycle node {0:?} is declared by no semantic relation")]
    UndeclaredLifecycleNode(LifecycleNodeId),

    /// A declared required exit that no allowed representation can
    /// reach, because the object carries no representation relation
    /// (SR2-03).
    #[error("lifecycle exit {object:?} -> {exit} has no allowed representation")]
    LifecycleExitWithoutRepresentation { object: ObjectId, exit: OperationId },

    #[error("disclosure node {0:?} is declared more than once")]
    DuplicateDisclosureNode(DisclosureNodeId),

    #[error("disclosure node {0:?} is unknown")]
    UnknownDisclosureNode(DisclosureNodeId),

    #[error("disclosure dependency {0:?} is declared more than once")]
    DuplicateDisclosureDependency(DisclosureDependencyDeclaration),

    #[error("representation/lifecycle mismatch for {object:?} {mode:?}")]
    RepresentationLifecycleMismatch {
        object: ObjectId,
        mode: RepresentationMode,
    },
}

impl RealizationError {
    /// A payload-free, stable variant identifier for tracing.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn trace_kind(&self) -> &'static str {
        match self {
            Self::ArchitectureValidationFailed { errors } => errors
                .first()
                .map_or("architecture_validation_failed", ManifestError::name),
            Self::ArchitectureHashUnavailable => "architecture_hash_unavailable",
            Self::ObservedRegionOverlap(_) => "observed_region_overlap",
            Self::EmptyScope => "empty_scope",
            Self::DuplicateScopeOperation(_) => "duplicate_scope_operation",
            Self::OperationOutsideArchitecture(_) => "operation_outside_architecture",
            Self::OperationOutsideScope(_) => "operation_outside_scope",
            Self::ArchitectureBindingMismatch => "architecture_binding_mismatch",
            Self::IncompleteScope { .. } => "incomplete_scope",
            Self::AmountOutOfDomain { .. } => "amount_out_of_domain",
            Self::AmountOverflow => "amount_overflow",
            Self::AmountUnderflow => "amount_underflow",
            Self::CountOverflow => "count_overflow",
            Self::CycleOverflow => "cycle_overflow",
            Self::InvalidAnnouncementLeadBounds { .. } => "invalid_announcement_lead_bounds",
            Self::MissingAnnouncementLeadBound(_) => "missing_announcement_lead_bound",
            Self::CycleBoundUsedAsCount(_) => "cycle_bound_used_as_count",
            Self::InvalidSingletonDeclaration { .. } => "invalid_singleton_declaration",
            Self::DuplicateExpression(_) => "duplicate_expression",
            Self::UnknownExpressionDependency { .. } => "unknown_expression_dependency",
            Self::ExpressionDependencyCycle { .. } => "expression_dependency_cycle",
            Self::ExpressionTypeMismatch { .. } => "expression_type_mismatch",
            Self::ExpressionOperandTypeMismatch { .. } => "expression_operand_type_mismatch",
            Self::BinaryOperandTypeMismatch { .. } => "binary_operand_type_mismatch",
            Self::InvalidSumType(_) => "invalid_sum_type",
            Self::InvalidOrderedType(_) => "invalid_ordered_type",
            Self::FactExpressionIdentityMismatch { .. } => "fact_expression_identity_mismatch",
            Self::DuplicateFactValue(_) => "duplicate_fact_value",
            Self::FactTypeMismatch { .. } => "fact_type_mismatch",
            Self::MissingFactValue(_) => "missing_fact_value",
            Self::ForeignExpressionFact { .. } => "foreign_expression_fact",
            Self::UnderivableOwnerFact { .. } => "underivable_owner_fact",
            Self::UnknownEvaluatedExpression { .. } | Self::UnknownRelation(_) => {
                "internal_invariant"
            }
            Self::TooManyExpressionOperands { .. } => "too_many_expression_operands",
            Self::UnknownPredicateExpression { .. } => "unknown_predicate_expression",
            Self::NonBooleanPredicateExpression { .. } => "non_boolean_predicate_expression",
            Self::SponsorValueRead => "sponsor_value_read",
            Self::UnsupportedOperationDeclaration(_) => "unsupported_operation_declaration",
            Self::DuplicateOperationDeclaration(_) => "duplicate_operation_declaration",
            Self::OperationDeclarationIdentityMismatch { .. } => {
                "operation_declaration_identity_mismatch"
            }
            Self::ForeignExpressionOwnership { .. } => "foreign_expression_ownership",
            Self::ForeignRelationOwnership { .. } => "foreign_relation_ownership",
            Self::ForeignRelationDependency { .. } => "foreign_relation_dependency",
            Self::ForeignProofAlternativeBinding { .. } => "foreign_proof_alternative_binding",
            Self::ForeignConstructibilityOwnership { .. } => "foreign_constructibility_ownership",
            Self::ForeignDisclosureOwnership { .. } => "foreign_disclosure_ownership",
            Self::UnknownDisclosureRelation { .. } => "unknown_disclosure_relation",
            Self::MissingArchitectureOperation(_) => "missing_architecture_operation",
            Self::ArchitectureOperationMismatch { .. } => "architecture_operation_mismatch",
            Self::MissingArchitectureRelation { .. } => "missing_architecture_relation",
            Self::ArchitectureRelationMismatch { .. } => "architecture_relation_mismatch",
            Self::SurplusArchitectureRelation { .. } => "surplus_architecture_relation",
            Self::RelationKindMismatch { .. } => "relation_kind_mismatch",
            Self::RelationSubjectMismatch { .. } => "relation_subject_mismatch",
            Self::MissingArchitectureObject(_) => "missing_architecture_object",
            Self::DuplicateRelation(_) => "duplicate_relation",
            Self::UnknownRelationDependency { .. } => "unknown_relation_dependency",
            Self::DuplicateRelationDependency(_) => "duplicate_relation_dependency",
            Self::RelationDependencyCycle { .. } => "relation_dependency_cycle",
            Self::MissingBoundValue(_) => "missing_bound_value",
            Self::UnknownObservedObject(_) => "unknown_observed_object",
            Self::DuplicateObservedObject(_) => "duplicate_observed_object",
            Self::DuplicateObservedReference => "duplicate_observed_reference",
            Self::DuplicateObservedRoot(_) => "duplicate_observed_root",
            Self::WrongObservedReferenceSide(_) => "wrong_observed_reference_side",
            Self::ObservedCanonicalPartitionOverlap => "observed_canonical_partition_overlap",
            Self::ObservedOpenFlowOverlap(_) => "observed_open_flow_overlap",
            Self::AnchorInObservedOpenFlow(_) => "anchor_in_observed_open_flow",
            Self::DuplicateConstructibilityNode(_) => "duplicate_constructibility_node",
            Self::UnknownConstructibilityNode(_) => "unknown_constructibility_node",
            Self::DuplicateConstructibilityDependency(_) => "duplicate_constructibility_dependency",
            Self::ConstructibilityCycle { .. } => "constructibility_cycle",
            Self::MissingConstructibilityOperation(_) => "missing_constructibility_operation",
            Self::PermissionlessPrivateDependency { .. } => "permissionless_private_dependency",
            Self::ConstructibilityAuthorizationMismatch(_) => {
                "constructibility_authorization_mismatch"
            }
            Self::ConstructibilityWitnessUnavailable { .. } => {
                "constructibility_witness_unavailable"
            }
            Self::SponsorDependencyEscaped { .. } => "sponsor_dependency_escaped",
            Self::CrossOperationConstructibilityDependency { .. } => {
                "cross_operation_constructibility_dependency"
            }
            Self::DuplicateLifecycleNode(_) => "duplicate_lifecycle_node",
            Self::UnknownLifecycleNode(_) => "unknown_lifecycle_node",
            Self::DuplicateLifecycleDependency(_) => "duplicate_lifecycle_dependency",
            Self::LifecycleCycle { .. } => "lifecycle_cycle",
            Self::MissingLifecycleRepresentation { .. } => "missing_lifecycle_representation",
            Self::MissingLifecycleExitNode { .. } => "missing_lifecycle_exit_node",
            Self::MissingLifecyclePath { .. } => "missing_lifecycle_path",
            Self::MalformedLifecycleEdge { .. } => "malformed_lifecycle_edge",
            Self::CrossObjectLifecycleEdge { .. } => "cross_object_lifecycle_edge",
            Self::UndeclaredLifecycleNode(_) => "undeclared_lifecycle_node",
            Self::LifecycleExitWithoutRepresentation { .. } => {
                "lifecycle_exit_without_representation"
            }
            Self::DuplicateDisclosureNode(_) => "duplicate_disclosure_node",
            Self::UnknownDisclosureNode(_) => "unknown_disclosure_node",
            Self::DuplicateDisclosureDependency(_) => "duplicate_disclosure_dependency",
            Self::RepresentationLifecycleMismatch { .. } => "representation_lifecycle_mismatch",
        }
    }

    pub(crate) fn trace_failed(&self) -> bool {
        matches!(
            self,
            Self::ArchitectureHashUnavailable
                | Self::UnknownEvaluatedExpression { .. }
                | Self::UnknownRelation(_)
        )
    }

    pub(crate) fn trace_defect_count(&self) -> Option<u64> {
        match self {
            Self::ArchitectureValidationFailed { errors } => Some(crate::trace_count(errors.len())),
            _ => None,
        }
    }
}
