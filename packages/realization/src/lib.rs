//! Target-independent typed semantic realization of the attestation contract.
//!
//! This crate sits between the finite architecture manifest and two
//! independent consumers:
//!
//! - executable-model conformance;
//! - future target-independent compiler analysis.
//!
//! The crate consumes validated typed architecture values. It does not
//! parse generated publications, model source, tests, documentation,
//! plans, target descriptions, or deployment artifacts.
//!
//! Phase 1 deliberately covers only:
//!
//! - compact ASH;
//! - live receipt transfer.
//!
//! A Phase-1 value is explicitly partial and cannot be accepted as a
//! complete protocol realization.
//!
//! # Boundary
//!
//! The single first-party dependency is `architecture`, and the only
//! input is a typed [`architecture::Architecture`] value. Nothing here
//! opens a file, reads the environment, or touches the network. Nothing
//! here names a target opcode, stack index, tapleaf, control block,
//! transaction position, or deployment network value.
//!
//! The crate publishes no realization hash and no generated realization
//! file. Typed values are consumed directly; a publication is added only
//! after its schema and identity policy are separately reviewed. Local
//! graph positions and arena handles are never semantic identity — the
//! stable keys in [`identity`] are.
//!
//! # Module map
//!
//! - [`scope`] — [`RealizationScope`], the explicit operation scope, and
//!   the [`CompleteRealizationScope`] wrapper that only a full-coverage
//!   scope can reach.
//! - [`binding`] — [`ArchitectureBinding`], the recorded tie from a
//!   realization to the architecture semantic hash it derived from.
//! - [`derive`](mod@derive) — [`derive`](fn@derive::derive), the one pure derivation,
//!   its output [`ScopedRealizationSpec`], and the stable
//!   [`ScopedRealizationProjection`].
//! - [`identity`] — stable first-party keys: [`FactId`], [`ExprId`],
//!   [`RelationId`], [`ProofAlternativeId`], and their component enums.
//! - [`domain`] — the checked semantic domains [`ProtocolAmount`] and
//!   [`Count`], plus [`RepresentationMode`]. A count is not an amount.
//! - [`value`] — evaluator runtime values: [`SemanticType`],
//!   [`SemanticValue`], [`OwnerId`].
//! - [`expression`] — typed expression declarations and their dependency
//!   graph projection.
//! - [`relation`] — typed semantic relations, their declarations, and the
//!   relation dependency graph projection.
//! - [`operation`] — per-operation declarations
//!   ([`OperationRealization`]) and their projection.
//! - [`observation`] — [`OperationObservation`], the primitive
//!   target-independent observation of one concrete operation, and
//!   [`validate_observation`].
//! - [`evaluate`] — relation evaluation over an observation:
//!   [`ConformanceReport`], [`RelationVerdict`], [`RelationStatus`],
//!   [`RelationFailure`], [`ExternalEvidenceRequirement`].
//! - [`constructibility`] — who can construct an operation, as a typed
//!   graph plus the [`ConstructibilityAuthorization`] discharge rule.
//! - [`lifecycle`] — representation-specific lifecycle requirements.
//! - [`declassification`] — dependency-derived disclosure analysis; the
//!   monotone least fixed point of seeded disclosure reasons.
//! - [`error`] — [`RealizationError`], the single error root.
//!
//! # Primary workflow
//!
//! One derivation, then two independent consumptions:
//!
//! 1. choose a scope — [`scope::RealizationScope::phase1_pilots`] or
//!    [`scope::RealizationScope::from_operations`];
//! 2. call [`derive::derive`] with a typed architecture and that scope.
//!    It validates the architecture, derives every declaration, builds
//!    and validates the graphs, and returns a value that is internally
//!    consistent by construction;
//! 3. either evaluate concrete observations against it with
//!    [`ScopedRealizationSpec::evaluate_operation`] (model conformance),
//!    or read its typed structure with
//!    [`ScopedRealizationSpec::project`], `relations`, `operations`, and
//!    [`ScopedRealizationSpec::constructibility_authorizations`]
//!    (compiler analysis).
//!
//! ```
//! use architecture::{ARCHITECTURE, OperationId};
//! use realization::{RealizationScope, derive};
//!
//! let spec = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
//!     .expect("the two pilots derive");
//!
//! // Scope is explicit and readable; operations come back in a stable order.
//! assert_eq!(spec.scope(), &RealizationScope::phase1_pilots());
//! let operations: Vec<OperationId> = spec.operations().map(|(id, _)| id).collect();
//! assert_eq!(
//!     operations,
//!     [OperationId::TransferLive, OperationId::CompactAsh],
//! );
//!
//! // The binding records which architecture this realization came from.
//! assert_eq!(
//!     spec.architecture().architecture_schema_version(),
//!     ARCHITECTURE.document.architecture_schema_version,
//! );
//!
//! // Outside the scope is a typed refusal, never a silent default.
//! assert!(spec.operation(OperationId::Burn).is_none());
//! assert!(matches!(
//!     spec.constructibility_authorizations(OperationId::Burn),
//!     Err(realization::RealizationError::OperationOutsideScope(
//!         OperationId::Burn
//!     )),
//! ));
//!
//! // The projection is the stable comparison surface: no graph handles.
//! let projection = spec.project();
//! assert_eq!(projection.operations.len(), 2);
//! assert_eq!(projection.relations.nodes.len(), spec.relations().count());
//! ```
//!
//! The README carries the worked conformance example — building an
//! [`OperationObservation`] and reading a [`ConformanceReport`] — plus
//! the full public-API tour and the error-handling guide.

#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_const_for_fn,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod binding;
pub mod constructibility;
pub mod declassification;
pub mod derive;
pub mod domain;
pub mod error;
pub mod evaluate;
pub mod expression;
pub mod identity;
pub mod lifecycle;
pub mod observation;
pub mod operation;
pub mod relation;
pub mod scope;
pub mod value;

mod declarations;
mod validate;

pub use binding::ArchitectureBinding;
pub(crate) use constructibility::validate_constructibility;
pub use constructibility::{
    AvailabilityClass, ConstructibilityAuthorization, ConstructibilityDependencyDeclaration,
    ConstructibilityDependencyProjection, ConstructibilityEdge, ConstructibilityEdgeRole,
    ConstructibilityGraphProjection, ConstructibilityNode, ConstructibilityNodeId,
    RequirementStrength, WitnessRole,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use constructibility::{build_constructibility_graph, project_constructibility_graph};
pub use declassification::{
    DeclassificationAnalysis, DisclosureDependencyDeclaration, DisclosureDependencyProjection,
    DisclosureEdge, DisclosureGraphProjection, DisclosureNode, DisclosureNodeId, DisclosureReason,
    DisclosureSeed, InitialVisibility, phase1_declassification,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use declassification::{
    analysis_from_reason_map, disclosure_reasons_by_node, project_disclosure_graph,
};
pub use derive::{
    ScopedRealizationProjection, ScopedRealizationSpec, derive, project_scoped_realization,
};
pub use domain::{Count, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE, ProtocolAmount, RepresentationMode};
pub use error::{ArchitectureMismatchField, RealizationError};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use evaluate::evaluate_operation;
pub use evaluate::{
    ConformanceReport, ExternalEvidenceRequirement, RelationFailure, RelationStatus,
    RelationVerdict,
};
pub use expression::{
    DependencyEdge, EvaluatedExpressions, ExpressionDeclaration, ExpressionDependencyProjection,
    ExpressionGraphProjection, ExpressionNode, FactValues,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use expression::{
    build_expression_graph, evaluate_expressions, project_expression_graph,
};
pub use identity::{
    ExprId, ExpressionRole, FactId, ProofAlternativeId, ProofKind, RelationId, RelationKind,
    RelationSubject, TransactionSide,
};
pub(crate) use lifecycle::require_lifecycle_exit;
pub use lifecycle::{
    LifecycleDependencyDeclaration, LifecycleDependencyProjection, LifecycleEdge,
    LifecycleGraphProjection, LifecycleNode, LifecycleNodeId,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use lifecycle::{build_lifecycle_graph, project_lifecycle_graph};
pub use observation::{
    ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition, ObservedDestructionLeg,
    ObservedFlowRole, ObservedIssuance, ObservedObject, ObservedObjectKind, ObservedObjectRef,
    ObservedOpenFlow, ObservedRootEffect, ObservedRootEffectKind, ObservedSide, ObservedValue,
    OperationObservation, validate_observation,
};
pub use operation::{OperationRealization, OperationRealizationProjection};
pub use relation::{
    CardinalityMaximum, ConstructibilityClass, ExpectedCanonicalDelta, Relation,
    RelationDeclaration, RelationDependencyDeclaration, RelationDependencyProjection, RelationEdge,
    RelationGraphProjection,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use relation::{build_relation_graph, project_relation_graph};
pub use scope::{CompleteRealizationScope, RealizationScope};
#[cfg(test)]
pub(crate) use validate::constructibility_authorizations;
pub use value::{OwnerId, SemanticType, SemanticValue};

#[cfg(test)]
mod tests;
