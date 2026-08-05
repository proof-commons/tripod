//! Compiler analysis unit tests.
//!
//! Fixtures derive the real Phase-1 pilot realization for positive
//! coverage; negative coverage uses synthetic declarations a validated
//! realization could never carry, through the crate-private builders.

mod expression_tests;
mod relation_tests;

use architecture::OperationId;

use crate::{AnalysisPolicy, BoundCompilerInput, CompilationScope, bind_input};

pub fn phase1_realization() -> realization::ScopedRealizationSpec {
    realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::phase1_pilots(),
    )
    .expect("phase-1 pilots derive")
}

pub fn bound_input(operations: &[OperationId]) -> BoundCompilerInput {
    bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        CompilationScope::from_operations(operations.iter().copied()).expect("scope"),
        AnalysisPolicy::Strict,
    )
    .expect("bind input")
}
