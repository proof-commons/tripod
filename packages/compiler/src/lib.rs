//! Target-independent compilation analysis of a validated the attestation realization.
//!
//! This crate analyzes a validated realization into a deterministic,
//! target-independent compilation plan. It emits no target program.
//!
//! # Boundary
//!
//! The compiler consumes typed values only. It does not consume
//! generated architecture, realization, or declassification
//! publications; model source, labels, or tests; plans, ADR, or target
//! reference prose; target bytecode or disassembly; or environment and
//! filesystem state. Nothing in this crate opens a file or reads the
//! environment, and that absence is a contract rather than an
//! accident.
//!
//! No target opcode, stack index, tapleaf, transaction position, or
//! target byte enters compiler core. Concrete positions are backend
//! output.
//!
//! # Identity
//!
//! This crate mints no public compiler digest. Under the recorded
//! identity policy an analysis identity activates only once a real
//! cross-process, cached, or published consumer exists; until then
//! typed comparison is the boundary, and a field reserved for a future
//! digest would itself be a speculative identity.
//!
//! # State
//!
//! The crate boundary, the validated input boundary ([`bind_input`]
//! and [`BoundCompilerInput`], P2-004), and the abstract target
//! requirement boundary ([`target`], Guide-8 §15) are the public
//! surface. The last of those publishes what an analysis requires of
//! some target — abstract capabilities and external-evidence roles —
//! and nothing about how the analysis reached them.
//!
//! These internal analysis stages are implemented:
//!
//! - validated typed input binding;
//! - canonical scoped relation DAG;
//! - canonical scoped expression DAG;
//! - checked constant folding;
//! - proof-obligation classification;
//! - exact feasible-plan enumeration;
//! - source requirements;
//! - constructibility analysis;
//! - disclosure analysis;
//! - representation lifecycle analysis;
//! - an independent exhaustive proof-search oracle;
//! - typed execution cases derived from feasible proof plans;
//! - relation discharge classification per execution case;
//! - abstract carrier roles and carrier eligibility;
//! - exact feasible placement enumeration with explicit limits;
//! - target-independent layout requirements;
//! - an independent exhaustive placement oracle;
//! - relation-indexed coverage requirements with typed mutation classes;
//! - typed coverage dependencies with two-pass symbol resolution and a
//!   forbidden-cycle SCC policy;
//! - an independent relation-by-case coverage oracle;
//! - relation-indexed requirement bundles with exact aggregate closure;
//! - complete scoped analyzed programs for the pilot scope, factorized
//!   per operation, with a corruption-resistant assembly validator and
//!   an independent assembly census oracle;
//! - the abstract target requirement projection, derivable only from a
//!   completely validated analyzed program.
//!
//! Every stage above the input boundary is crate-private, and the
//! target boundary publishes a projection of the result rather than the
//! result. The following remain absent:
//!
//! - a public complete-analysis result;
//! - a compiler-plan identity;
//! - a target capability adapter;
//! - any target-specific type;
//! - target program emission.
//!
//! No public value produced by the crate today can be mistaken for a
//! completed compiler analysis.
//!
//! # Module map
//!
//! Only three modules are public, and that is the point: everything
//! between the input boundary and the target boundary is crate-private.
//!
//! - [`input`] — the validated input boundary: [`CompilationScope`],
//!   [`ProofSearchLimits`], [`AnalysisPolicy`], [`BoundCompilerInput`],
//!   and [`bind_input`].
//! - [`target`] — the abstract target requirement boundary:
//!   [`target::RequiredCapability`], [`target::ExternalEvidenceRole`],
//!   [`target::PlacementSearchLimits`], [`target::TargetRequirementSet`],
//!   and [`target::analyze_target_requirements`].
//! - [`error`] — [`CompileError`], the single error root for both.
//!
//! Three further items reach the crate root: [`ExpressionCycleComponent`]
//! and [`RelationCycleComponent`], carried by the two cycle-diagnostic
//! error variants, and [`OperandId`].
//!
//! # Primary workflow
//!
//! Two calls, in this order, with no route between them:
//!
//! 1. [`input::bind_input`] — hand it a typed architecture, a validated
//!    [`realization::ScopedRealizationSpec`], a
//!    [`input::CompilationScope`], and an [`input::AnalysisPolicy`]. It
//!    re-runs the realization owner's validation, requires the
//!    realization's architecture binding to equal the architecture you
//!    passed, and requires every scope member to be declared by the
//!    realization. What comes back is an immutable
//!    [`input::BoundCompilerInput`].
//! 2. [`target::analyze_target_requirements`] — hand it that bound input
//!    and [`target::PlacementSearchLimits`]. It runs the complete scoped
//!    analysis and its independent validator, then returns an opaque
//!    read-only [`target::TargetRequirementSet`]. There is no partial
//!    result: a search that ran out of budget is a typed failure, never
//!    a smaller requirement set.
//!
//! ```
//! use std::num::NonZeroU64;
//!
//! use architecture::{ARCHITECTURE, OperationId};
//! use compiler::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
//! use compiler::target::{PlacementSearchLimits, analyze_target_requirements};
//!
//! let realization = realization::derive(
//!     &ARCHITECTURE,
//!     realization::RealizationScope::phase1_pilots(),
//! )
//! .expect("phase-1 pilots derive");
//!
//! let scope =
//!     CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
//!         .expect("nonempty, no duplicates");
//!
//! let policy = AnalysisPolicy::strict(ProofSearchLimits::new(
//!     NonZeroU64::new(1_000_000).unwrap(),
//!     NonZeroU64::new(10_000).unwrap(),
//! ));
//!
//! let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("bindable");
//!
//! let requirements = analyze_target_requirements(
//!     &input,
//!     PlacementSearchLimits::new(
//!         NonZeroU64::new(10_000_000).unwrap(),
//!         NonZeroU64::new(1_000_000).unwrap(),
//!     ),
//! )
//! .expect("the pilot analysis completes");
//!
//! assert!(requirements.capabilities().next().is_some());
//! // Typed comparison is the whole comparison mechanism: no digest is minted.
//! assert_eq!(requirements, requirements.clone());
//! ```
//!
//! The README carries the public-API tour, the error-handling guide, and
//! the record of what stays crate-private and why.

#![forbid(unsafe_code)]

mod analyzed;
mod analyzed_operation;
mod analyzed_validate;
mod capability;
mod carrier;
mod case;
mod constructibility;
mod coverage;
mod coverage_graph;
mod disclosure;
pub mod error;
mod expression;
mod fold;
mod foundation;
pub mod input;
mod layout;
mod lifecycle;
mod placement;
mod proof;
mod relation;
mod requirement;
mod search_counter;
mod source;
mod sponsor_region;
pub mod target;

pub use error::CompileError;
pub use expression::ExpressionCycleComponent;
pub use input::{
    AnalysisPolicy, BoundCompilerInput, CompilationScope, ProofSearchLimits, bind_input,
};
pub use relation::RelationCycleComponent;
pub use source::OperandId;

#[cfg(test)]
mod tests;
