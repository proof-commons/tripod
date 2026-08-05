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
//! The crate boundary and the validated input boundary ([`bind_input`]
//! and [`BoundCompilerInput`], P2-004) are the public surface.
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
//! - an independent exhaustive proof-search oracle.
//!
//! Every stage above the input boundary is crate-private. The
//! following remain absent:
//!
//! - execution-case placement;
//! - concrete target-independent layout requirements;
//! - relation-indexed coverage requirements;
//! - a complete pilot analyzed program;
//! - a public complete-analysis result;
//! - a compiler-plan identity;
//! - target program emission.
//!
//! No public value produced by the crate today can be mistaken for a
//! completed compiler analysis.

#![forbid(unsafe_code)]

mod capability;
mod case;
mod constructibility;
mod disclosure;
pub mod error;
mod expression;
mod fold;
mod foundation;
pub mod input;
mod lifecycle;
mod placement;
mod proof;
mod relation;
mod source;

pub use error::CompileError;
pub use expression::ExpressionCycleComponent;
pub use input::{
    AnalysisPolicy, BoundCompilerInput, CompilationScope, ProofSearchLimits, bind_input,
};
pub use relation::RelationCycleComponent;
pub use source::OperandId;

#[cfg(test)]
mod tests;
