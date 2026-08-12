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

    /// One constructibility node occurred more than once.
    #[error("constructibility node {node:?} occurs more than once")]
    DuplicateConstructibilityNode {
        /// The repeated node.
        node: realization::ConstructibilityNodeId,
    },

    /// One constructibility dependency occurred more than once.
    #[error("constructibility dependency at {node:?} is declared more than once")]
    DuplicateConstructibilityDependency {
        /// The dependency's source node.
        node: realization::ConstructibilityNodeId,
    },

    /// A constructibility edge names an absent node.
    #[error("constructibility node {node:?} is absent from the scoped graph")]
    UnknownConstructibilityNode {
        /// The missing stable node ID (never a graph index).
        node: realization::ConstructibilityNodeId,
    },

    /// The scoped constructibility dependencies contain a cycle.
    #[error("constructibility dependencies contain {} cyclic component(s)", components.len())]
    ConstructibilityCycle {
        /// Canonically ordered cyclic components in stable IDs.
        components: Vec<Vec<realization::ConstructibilityNodeId>>,
    },

    /// A constructibility dependency crosses operations.
    #[error("constructibility node {node:?} crosses into operation {operation:?}")]
    ConstructibilityCrossOperationDependency {
        /// The operation being analyzed.
        operation: OperationId,
        /// The foreign node.
        node: realization::ConstructibilityNodeId,
    },

    /// A permissionless case depends on a private availability.
    #[error("permissionless {operation:?} requires private node {node:?}")]
    PermissionlessPrivateDependency {
        /// The permissionless operation.
        operation: OperationId,
        /// The private dependency.
        node: realization::ConstructibilityNodeId,
        /// A stable-ID path to the operation.
        path: Vec<realization::ConstructibilityNodeId>,
    },

    /// A required witness is unavailable under an authorization case.
    #[error("witness {node:?} of {operation:?} is unavailable under its authorization")]
    ConstructibilityWitnessUnavailable {
        /// The operation being analyzed.
        operation: OperationId,
        /// The authorization case that cannot discharge the witness.
        authorization: realization::ConstructibilityAuthorization,
        /// The unavailable dependency.
        node: realization::ConstructibilityNodeId,
        /// A stable-ID path to the operation.
        path: Vec<realization::ConstructibilityNodeId>,
    },

    /// A sponsor-local dependency escaped its optional sponsor subtree.
    #[error("sponsor-local node {node:?} of {operation:?} escapes its optional subtree")]
    SponsorDependencyEscaped {
        /// The operation being analyzed.
        operation: OperationId,
        /// The escaping sponsor-local node.
        node: realization::ConstructibilityNodeId,
        /// A stable-ID escape path.
        path: Vec<realization::ConstructibilityNodeId>,
    },

    /// An in-scope lifecycle exit has no representation variable.
    #[error("no representation choice for {object:?} of {operation:?}")]
    MissingRepresentationChoice {
        /// The operation owning the lifecycle relation.
        operation: OperationId,
        /// The object without a representation relation.
        object: architecture::ObjectId,
    },

    /// A supported representation cannot reach a required exit.
    #[error("{object:?} under {representation:?} cannot reach exit {exit:?}")]
    MissingLifecyclePath {
        /// The lifecycle object.
        object: architecture::ObjectId,
        /// The representation without the path.
        representation: realization::RepresentationMode,
        /// The unreachable required exit.
        exit: OperationId,
    },

    /// An added disclosure carries no typed reason.
    #[error("disclosure of {fact:?} has no typed reason")]
    MissingDisclosureReason {
        /// The reasonless disclosed fact.
        fact: realization::FactId,
    },

    /// A permissionless operation requires a private fact.
    #[error("permissionless {operation:?} requires private fact {fact:?}")]
    PermissionlessPrivateFact {
        /// The permissionless operation.
        operation: OperationId,
        /// The private fact.
        fact: realization::FactId,
    },

    /// A fact was disclosed with no proof or representation cause.
    #[error("unexpected disclosure of {fact:?}")]
    UnexpectedDisclosure {
        /// The unexpectedly disclosed fact.
        fact: realization::FactId,
    },

    /// A runtime relation declares no approved proof alternative.
    #[error("relation {relation:?} has no proof alternative")]
    MissingProofAlternative {
        /// The alternativeless relation.
        relation: realization::RelationId,
    },

    /// An externally evidenced relation does not declare exactly its
    /// one realization-approved proof class.
    ///
    /// The defect is an extra, missing, or wrong alternative, so the
    /// compiler refuses to plan the relation rather than manufacturing
    /// the class or selecting an arbitrary member of the set.
    #[error("external-evidence relation {relation:?} has unexpected proof alternatives")]
    InvalidExternalEvidenceProofAlternatives {
        /// The externally evidenced relation.
        relation: realization::RelationId,
    },

    /// No complete assignment satisfies every hard constraint.
    ///
    /// Compiler planning failure under the supplied capability and
    /// constructibility policy — never target evidence.
    #[error("no feasible proof plan; {} relation(s) blocked", blocked_relations.len())]
    NoFeasibleProofPlan {
        /// The blocking relations, canonically sorted.
        blocked_relations: Vec<realization::RelationId>,
    },

    /// The exact search exceeded its explicit state limit.
    ///
    /// No partial result is returned: neither infeasibility nor
    /// optimality is claimed on exhaustion.
    #[error("proof search exceeded {maximum} states")]
    ProofSearchStateLimitExceeded {
        /// The configured maximum.
        maximum: u64,
    },

    /// The exact search exceeded its explicit candidate limit.
    #[error("proof search exceeded {maximum} candidates")]
    ProofCandidateLimitExceeded {
        /// The configured maximum.
        maximum: u64,
    },

    /// One execution-case identity occurred more than once.
    ///
    /// Cases are a semantic equivalence class census: a repeated
    /// identity means one class would be planned twice, so it is a
    /// defect rather than a value to deduplicate silently.
    #[error("execution case {case:?} occurs more than once")]
    DuplicateExecutionCase {
        /// The repeated case identity.
        case: crate::case::ExecutionCaseId,
    },

    /// The derived case census does not equal the census the relations
    /// and the selected proof plan require.
    #[error(
        "execution case census mismatch: {} missing, {} unexpected",
        missing.len(),
        unexpected.len()
    )]
    ExecutionCaseCensusMismatch {
        /// Required cases absent from the derivation, sorted.
        missing: Vec<crate::case::ExecutionCaseId>,
        /// Derived cases absent from the requirement, sorted.
        unexpected: Vec<crate::case::ExecutionCaseId>,
    },

    /// One relation was planned twice in the same execution case.
    #[error("relation {relation:?} is planned twice in case {case:?}")]
    DuplicateRelationCasePlan {
        /// The twice-planned relation.
        relation: realization::RelationId,
        /// The case carrying both plans.
        case: crate::case::ExecutionCaseId,
    },

    /// The relation-case census is not exactly every in-scope relation
    /// crossed with every applicable execution case.
    ///
    /// A vacuous, compiler-static, structural, or externally evidenced
    /// disposition is stated explicitly; it is never a reason for a
    /// relation to leave the census.
    #[error(
        "relation-case census mismatch: {} missing, {} unexpected",
        missing.len(),
        unexpected.len()
    )]
    RelationCaseCensusMismatch {
        /// Required relation-case pairs absent from the analysis.
        missing: Vec<crate::placement::RelationCaseKey>,
        /// Analyzed pairs absent from the requirement.
        unexpected: Vec<crate::placement::RelationCaseKey>,
    },

    /// The operation has no input family a typed cardinality minimum
    /// guarantees is present, so no coordinator anchor exists.
    ///
    /// A global coordinator is never conjured from nowhere: without a
    /// mandatory input family the operation's transaction-global
    /// relations have no carrier, and inventing one would claim
    /// enforcement no structure supports.
    #[error("operation {operation:?} has no mandatory input family to anchor a coordinator")]
    MissingCanonicalCoordinator {
        /// The operation without a coordinator anchor.
        operation: OperationId,
    },

    /// Every candidate carrier of an unconditionally active relation is
    /// anchored in a family that may be absent.
    ///
    /// The optional sponsor family is the pilots' instance: it may
    /// carry its own conditional relations, never an unconditional one
    /// on its own.
    #[error("unconditional relation {relation:?} has only optional carriers in case {case:?}")]
    UnconditionalRelationOnOptionalCarrier {
        /// The unconditionally active relation.
        relation: realization::RelationId,
        /// The case in which every candidate is optional.
        case: crate::case::ExecutionCaseId,
    },

    /// No candidate carrier can receive every active source row of an
    /// active runtime relation.
    #[error("relation {relation:?} has no eligible carrier in case {case:?}")]
    NoEligibleCarrier {
        /// The uncarried relation.
        relation: realization::RelationId,
        /// The case in which no carrier is eligible.
        case: crate::case::ExecutionCaseId,
    },

    /// A carrier dependency or structural obligation has no layout
    /// requirement.
    ///
    /// A carrier that cannot see its operands does not discharge its
    /// relation, so an unrouted source is a defect rather than an
    /// omission a backend may fill in later by assumption.
    #[error("relation {relation:?} lacks a layout requirement in case {case:?}")]
    MissingLayoutRequirement {
        /// The relation whose dependency is unstated.
        relation: realization::RelationId,
        /// The case carrying the unstated dependency.
        case: crate::case::ExecutionCaseId,
    },

    /// The carried relation-cases are not exactly the relation-cases
    /// whose plan states a runtime requirement.
    ///
    /// A relation-case never disappears from the census because it is
    /// vacuous, and a compiler-static, backend-structural, or externally
    /// evidenced relation-case never enters it: a runtime carrier for
    /// one of those would report an unfinished obligation as target
    /// execution.
    #[error(
        "placement census mismatch: {} missing, {} unexpected",
        missing.len(),
        unexpected.len()
    )]
    PlacementCensusMismatch {
        /// Runtime relation-cases absent from the placement, sorted.
        missing: Vec<crate::placement::RelationCaseKey>,
        /// Placed relation-cases absent from the requirement, sorted.
        unexpected: Vec<crate::placement::RelationCaseKey>,
    },

    /// Every eligible carrier of a family- or transaction-global
    /// relation is a local one.
    ///
    /// A per-member role observes one member, so it cannot establish a
    /// property of a complete family or of the whole transaction; a
    /// global relation may not hide on a local-only carrier.
    #[error("global relation {relation:?} has only local carriers in case {case:?}")]
    GlobalRelationHasOnlyLocalCarrier {
        /// The global relation.
        relation: realization::RelationId,
        /// The case in which every eligible carrier is local.
        case: crate::case::ExecutionCaseId,
    },

    /// A placement assigned a carrier the obligation does not permit.
    ///
    /// The carrier is ineligible, is a non-runtime role, cannot
    /// discharge the obligation's semantic scope or multiplicity, or is
    /// an optional sponsor carrier of an unconditional relation.
    #[error("relation {relation:?} may not be placed on {carrier:?} in case {case:?}")]
    UnpermittedCarrierPlacement {
        /// The relation being placed.
        relation: realization::RelationId,
        /// The case carrying the placement.
        case: crate::case::ExecutionCaseId,
        /// The impermissible carrier role.
        carrier: crate::carrier::CarrierRole,
    },

    /// One placement assigned the same carrier role twice to one
    /// obligation.
    ///
    /// A repeated entry is a defect rather than a value to normalize
    /// away: a stable projection collapses it into a set, so a raw
    /// invalid assignment and a valid one would project identically and
    /// duplication policy would stop being enforceable.
    #[error("relation {relation:?} places carrier {carrier:?} twice in case {case:?}")]
    DuplicatePlacedCarrier {
        /// The relation whose assignment repeats a carrier.
        relation: realization::RelationId,
        /// The case carrying the repeated assignment.
        case: crate::case::ExecutionCaseId,
        /// The twice-assigned carrier role.
        carrier: crate::carrier::CarrierRole,
    },

    /// One placement assigned a carrier set that is not one exact
    /// retained option of the obligation.
    ///
    /// Enumeration and validation describe the same candidate language:
    /// an exactly-one, every-member, or at-least-one obligation carries
    /// one inclusion-minimal choice, and a deliberate-duplication
    /// obligation carries exactly the complete admitted set. A superset
    /// is neither more enforcement nor a second valid answer.
    #[error("relation {relation:?} has a noncanonical carrier assignment in case {case:?}")]
    NonCanonicalCarrierAssignment {
        /// The relation whose assignment is not a retained option.
        relation: realization::RelationId,
        /// The case carrying the noncanonical assignment.
        case: crate::case::ExecutionCaseId,
    },

    /// A placement states a layout requirement no carrier it selected
    /// depends on.
    ///
    /// Placement-local layout is exactly what the selected carriers
    /// need. A surplus, repeated, or foreign requirement would record an
    /// obligation against a future backend that this placement never
    /// justified, so it is rejected rather than tolerated as harmless
    /// extra structure.
    #[error("placement states {} unrelated layout requirement(s)", unexpected.len())]
    UnexpectedLayoutRequirement {
        /// The stated requirements no selected carrier depends on,
        /// sorted.
        unexpected: Vec<crate::layout::LayoutRequirement>,
    },

    /// The exact placement search exceeded its explicit state limit.
    ///
    /// No partial result is returned: a truncated search proves neither
    /// that the retained placements are complete nor that the remaining
    /// ones are infeasible.
    #[error("placement search exceeded {maximum} states")]
    PlacementSearchStateLimitExceeded {
        /// The configured maximum.
        maximum: u64,
    },

    /// The exact placement search exceeded its explicit candidate limit.
    #[error("placement search exceeded {maximum} candidates")]
    PlacementCandidateLimitExceeded {
        /// The configured maximum.
        maximum: u64,
    },

    /// One coverage requirement identity occurs more than once.
    ///
    /// A repeated identity means one obligation would be stated twice
    /// and answered once, so it is a defect rather than a value to
    /// deduplicate silently.
    #[error("coverage requirement {requirement:?} occurs more than once")]
    DuplicateCoverageRequirement {
        /// The repeated requirement identity.
        requirement: crate::coverage::CoverageRequirementId,
    },

    /// The covered relation-cases are not exactly the planned ones.
    ///
    /// No relation may leave the coverage census because it is
    /// inactive, compiler-static, backend-structural, or externally
    /// evidenced: those are dispositions coverage states, never reasons
    /// to omit a relation.
    #[error(
        "coverage census mismatch: {} missing, {} unexpected",
        missing.len(),
        unexpected.len()
    )]
    CoverageCensusMismatch {
        /// Planned relation-cases absent from the coverage, sorted.
        missing: Vec<crate::placement::RelationCaseKey>,
        /// Covered relation-cases absent from the plans, sorted.
        unexpected: Vec<crate::placement::RelationCaseKey>,
    },

    /// An active relation-case states no positive coverage at one of
    /// its own evidence boundaries.
    #[error("relation {relation:?} has no positive coverage at {boundary:?} in case {case:?}")]
    MissingPositiveCoverage {
        /// The uncovered relation.
        relation: realization::RelationId,
        /// The case carrying the gap.
        case: crate::case::ExecutionCaseId,
        /// The boundary with no positive requirement.
        boundary: crate::coverage::CoverageBoundary,
    },

    /// An active relation-case states no negative coverage at one of
    /// its own evidence boundaries.
    ///
    /// A relation with no required rejection is a relation no evidence
    /// package can distinguish from an unenforced one.
    #[error("relation {relation:?} has no negative coverage at {boundary:?} in case {case:?}")]
    MissingNegativeCoverage {
        /// The uncovered relation.
        relation: realization::RelationId,
        /// The case carrying the gap.
        case: crate::case::ExecutionCaseId,
        /// The boundary with no negative requirement.
        boundary: crate::coverage::CoverageBoundary,
    },

    /// An inactive relation-case states no inactive-valid coverage.
    #[error("relation {relation:?} has no inactive-valid coverage in case {case:?}")]
    MissingInactiveCoverage {
        /// The inactive relation.
        relation: realization::RelationId,
        /// The case in which it never activates.
        case: crate::case::ExecutionCaseId,
    },

    /// An inactive relation-case demands a focused rejection.
    ///
    /// A target rejecting a mutation of a relation the case never
    /// activates proves nothing about that relation, so the
    /// requirement would be evidence of the wrong thing.
    #[error("inactive relation {relation:?} demands a rejection in case {case:?}")]
    UnexpectedCoverageRejection {
        /// The inactive relation.
        relation: realization::RelationId,
        /// The case in which it never activates.
        case: crate::case::ExecutionCaseId,
    },

    /// An active runtime relation-case states no carrier obligation.
    #[error("relation {relation:?} has no carrier coverage in case {case:?}")]
    MissingCarrierCoverage {
        /// The uncarried relation.
        relation: realization::RelationId,
        /// The case carrying the gap.
        case: crate::case::ExecutionCaseId,
    },

    /// A relation-case that no carrier executes states one anyway.
    ///
    /// A compiler-static, backend-structural, externally evidenced, or
    /// inactive relation-case with a runtime carrier requirement would
    /// report an unfinished obligation as target execution.
    #[error("relation {relation:?} may not require a runtime carrier in case {case:?}")]
    UnexpectedRuntimeCarrierCoverage {
        /// The relation no carrier executes.
        relation: realization::RelationId,
        /// The case carrying the impermissible requirement.
        case: crate::case::ExecutionCaseId,
    },

    /// Runtime-target acceptance is claimed with no accepted semantic
    /// projection to compare.
    ///
    /// A report that says "accepted" without naming what was accepted
    /// cannot distinguish a relation that held from one that was never
    /// evaluated.
    #[error("relation {relation:?} has no accepted projection in case {case:?}")]
    MissingProjectionCoverage {
        /// The relation with no projection comparison.
        relation: realization::RelationId,
        /// The case carrying the gap.
        case: crate::case::ExecutionCaseId,
    },

    /// A selectable carrier assignment is uncovered, or its layout
    /// dependencies are.
    #[error("relation {relation:?} has an uncovered assignment in case {case:?}")]
    MissingCoverageLayoutRequirement {
        /// The relation whose assignment is uncovered.
        relation: realization::RelationId,
        /// The case carrying the uncovered assignment.
        case: crate::case::ExecutionCaseId,
    },

    /// Coverage references a layout requirement the plan does not
    /// state.
    ///
    /// Coverage requires what the *selected* assignment depends on; a
    /// reference outside the plan's own census is a dependency nothing
    /// derived.
    #[error("relation {relation:?} references an unstated layout requirement in case {case:?}")]
    UnexpectedCoverageLayoutRequirement {
        /// The over-referencing relation.
        relation: realization::RelationId,
        /// The case carrying the reference.
        case: crate::case::ExecutionCaseId,
    },

    /// An allowed representation mode is covered by no plan in the set.
    ///
    /// Each candidate fixes one mode, so the mode census is a property
    /// of the complete feasible plan set rather than of any one plan.
    #[error("relation {relation:?} has no coverage for representation {representation:?}")]
    MissingRepresentationCoverage {
        /// The representation relation.
        relation: realization::RelationId,
        /// The uncovered mode.
        representation: realization::RepresentationMode,
    },

    /// A coverage dependency names a symbol the definition census does
    /// not define.
    ///
    /// The two-pass resolution completes the census first, so an
    /// unresolved reference is a missing definition rather than a
    /// forward reference the order happened to reach early.
    #[error("coverage dependency names undefined symbol {symbol:?}")]
    UnknownCoverageSymbol {
        /// The unresolved coverage symbol.
        symbol: crate::coverage_graph::CoverageNodeId,
    },

    /// The same typed coverage dependency is declared twice.
    ///
    /// One dependency stated twice would be counted twice by anything
    /// that walks the edges, so it is a defect rather than a value to
    /// deduplicate silently.
    ///
    /// The endpoints are boxed: a coverage symbol is a complete typed
    /// value, and two of them inline would make every fallible
    /// compiler result carry that width.
    #[error(
        "coverage dependency {prerequisite:?} -> {dependent:?} ({edge:?}) is declared more than once"
    )]
    DuplicateCoverageDependency {
        /// The prerequisite symbol.
        prerequisite: Box<crate::coverage_graph::CoverageNodeId>,
        /// The dependent symbol.
        dependent: Box<crate::coverage_graph::CoverageNodeId>,
        /// The repeated edge role.
        edge: crate::coverage_graph::CoverageEdge,
    },

    /// The coverage dependencies contain a cycle.
    ///
    /// Coverage cycles are forbidden: an accepted cycle would need a
    /// typed resolution strategy that does not exist, and finding a
    /// strongly connected component never authorizes one.
    #[error("coverage dependency cycle in {} component(s)", components.len())]
    CoverageDependencyCycle {
        /// Canonically normalized cyclic components.
        components: Vec<crate::coverage_graph::CoverageCycleComponent>,
    },

    /// A coverage dependency crosses two operations.
    ///
    /// Coverage is stored and validated per operation, so a dependency
    /// spanning two of them is a claim neither operation's analysis
    /// owns. The endpoints are boxed for the same reason as
    /// [`Self::DuplicateCoverageDependency`].
    #[error("coverage dependency {prerequisite:?} -> {dependent:?} crosses two operations")]
    CrossOperationCoverageDependency {
        /// The prerequisite symbol.
        prerequisite: Box<crate::coverage_graph::CoverageNodeId>,
        /// The dependent symbol.
        dependent: Box<crate::coverage_graph::CoverageNodeId>,
    },

    /// One proof-plan candidate was placed more than once.
    ///
    /// The variant names no plan: until an admitted plan identity
    /// exists, a plan may not be referred to by vector position, search
    /// order, candidate number, or digest, and the complete typed plan
    /// value is too large to be a diagnostic field.
    #[error("a proof-plan candidate was placed more than once")]
    DuplicatePlacedProofPlan,

    /// The placed proof-plan set is not exactly the offered set.
    ///
    /// Execution-case union equality is too weak to prove this: two
    /// plans may select different proofs while fixing the same
    /// representations, so their case identities coincide and dropping
    /// one leaves the union untouched.
    ///
    /// The counts are the diagnostic because the alternative is not: a
    /// complete typed plan is too large to carry here, and a digest,
    /// vector position, or candidate number would name a plan by
    /// something that is not its identity.
    #[error("placed proof-plan census mismatch: {missing} missing, {unexpected} unexpected")]
    PlacedProofPlanCensusMismatch {
        /// Offered plans absent from the placed set.
        missing: usize,
        /// Placed plans absent from the offered set.
        unexpected: usize,
    },
}
