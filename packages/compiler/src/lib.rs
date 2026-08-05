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
//! and [`BoundCompilerInput`], P2-004) are implemented. The canonical
//! relation graph, constant folding, proof planning, disclosure and
//! fact-source analysis, constructibility, lifecycle, placement,
//! layout, and coverage requirements are separate deliverables and are
//! not implemented here. No partial analysis is exposed in the
//! meantime, so nothing in this crate can be mistaken for a completed
//! one.

#![forbid(unsafe_code)]

mod capability;
pub mod error;
mod expression;
mod fold;
mod foundation;
pub mod input;
mod relation;
mod source;

pub use error::CompileError;
pub use expression::ExpressionCycleComponent;
pub use input::{AnalysisPolicy, BoundCompilerInput, CompilationScope, bind_input};
pub use relation::RelationCycleComponent;
pub use source::OperandId;

#[cfg(test)]
mod tests;
