//! Compiler analysis unit tests.
//!
//! Fixtures derive the real Phase-1 pilot realization for positive
//! coverage; negative coverage uses synthetic declarations a validated
//! realization could never carry, through the crate-private builders.

mod analyzed_corruption_tests;
mod analyzed_operation_tests;
mod analyzed_program_oracle_tests;
mod analyzed_program_tests;
mod carrier_tests;
mod case_tests;
mod constructibility_tests;
mod coverage_graph_tests;
mod coverage_oracle_tests;
mod coverage_tests;
mod disclosure_tests;
mod expression_tests;
mod fold_tests;
mod layout_tests;
mod lifecycle_tests;
mod operation_plan_tests;
mod oracle_tests;
mod pilot_coverage_tests;
mod pilot_placement_tests;
mod pilot_program_tests;
mod placement_oracle_tests;
mod placement_tests;
mod proof_tests;
mod relation_tests;
mod requirement_tests;
mod source_tests;
mod target_tests;

use architecture::OperationId;

use crate::{AnalysisPolicy, BoundCompilerInput, CompilationScope, ProofSearchLimits, bind_input};

pub const fn test_policy() -> AnalysisPolicy {
    AnalysisPolicy::strict(ProofSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(10_000).expect("nonzero"),
    ))
}

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
        test_policy(),
    )
    .expect("bind input")
}

// --- foundation aggregate (Guide-2 Tranche F) ---

#[test]
fn repeated_foundation_analysis_is_equal() {
    for scope in [
        vec![OperationId::CompactAsh],
        vec![OperationId::TransferLive],
        vec![OperationId::CompactAsh, OperationId::TransferLive],
    ] {
        let input = bound_input(&scope);
        let first = crate::foundation::analyze_foundation(&input)
            .expect("first analysis")
            .project();
        let second = crate::foundation::analyze_foundation(&input)
            .expect("second analysis")
            .project();

        assert_eq!(first, second, "scope {scope:?}");
    }
}

#[test]
fn foundation_projection_equals_its_component_projections() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let foundation = crate::foundation::analyze_foundation(&input).expect("analysis");
    let projection = foundation.project();

    assert_eq!(projection.relations, foundation.relations.project());
    assert_eq!(projection.expressions, foundation.expressions.project());

    // The pilots declare no expressions, so the folded projection is
    // empty — and every relation in compiler scope is represented.
    assert_eq!(
        projection.expressions.nodes,
        [] as [crate::expression::CompilerExpressionNodeProjection; 0]
    );
    assert_eq!(
        projection.relations.nodes.len(),
        input.realization().project().relations.nodes.len(),
    );
}
