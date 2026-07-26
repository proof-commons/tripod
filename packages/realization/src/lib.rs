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
pub use evaluate::{ConformanceReport, RelationFailure, RelationStatus, RelationVerdict};
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
    ObservedIssuance, ObservedObject, ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow,
    ObservedRootEffect, ObservedSide, ObservedValue, OperationObservation, validate_observation,
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
