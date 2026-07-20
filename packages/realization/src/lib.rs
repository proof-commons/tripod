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
pub mod declassification;
pub mod derive;
pub mod domain;
pub mod error;
pub mod evaluate;
pub mod expression;
pub mod identity;
pub mod observation;
pub mod operation;
pub mod relation;
pub mod scope;
pub mod value;

mod declarations;
mod validate;

pub use binding::ArchitectureBinding;
pub use declassification::{
    DeclassificationAnalysis, DisclosureEdge, DisclosureNode, DisclosureReason,
};
pub use derive::{ScopedRealizationSpec, derive};
pub use domain::{Count, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE, ProtocolAmount, RepresentationMode};
pub use error::{ArchitectureMismatchField, RealizationError};
pub use evaluate::{ConformanceReport, RelationStatus, RelationVerdict, evaluate_operation};
pub use expression::{
    DependencyEdge, EvaluatedExpressions, ExpressionDeclaration, ExpressionNode, FactValues,
    build_expression_graph, evaluate_expressions,
};
pub use identity::{
    ExprId, ExpressionRole, FactId, ProofAlternativeId, ProofKind, RelationId, RelationKind,
    RelationSubject, TransactionSide,
};
pub use observation::{
    ObservedAsset, ObservedObject, ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow,
    ObservedRootEffect, ObservedSide, OperationObservation,
};
pub use operation::OperationRealization;
pub use relation::{CardinalityMaximum, Relation, RelationDeclaration, RelationEdge};
pub use scope::{CompleteRealizationScope, RealizationScope};
pub use value::{OwnerId, SemanticType, SemanticValue};

#[cfg(test)]
mod tests;
