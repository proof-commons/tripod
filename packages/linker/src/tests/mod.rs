//! Linker tests and their shared fixtures.
//!
//! # The bundle under test is a real one
//!
//! Every fixture below reaches the relocatable bundle the way an
//! external consumer would: derive the realization, bind the compiler
//! input, plan the compact-ASH target operation, and emit the bundle
//! through the backend's own public entry point. Nothing here
//! hand-assembles a bundle, because a hand-assembled bundle would let a
//! link succeed against an artifact no backend produced.
//!
//! # The symbol values are fixtures and say so
//!
//! The resolutions below are distinguishable, meaningless byte strings.
//! They are test material in the sense
//! `(´[ADR015-rule:security:test-material]´)` fixes: public, carrying no
//! secret, and standing for no real object. A production deployment's
//! values are not this linker's to invent (§1.10), and the demonstration
//! link says it is a fixture link rather than implying otherwise.

mod carrier_tests;
mod graph_tests;
mod link_tests;
mod public_api_tests;
mod relocation_tests;
mod symbol_tests;
mod taptree_tests;

use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::operation_plan::{
    PlacementSearchLimits, ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
};
use realization::{RealizationScope, derive};
use tapscript::{
    CandidateRelocatableTapscriptBundle, CompactAshSymbols, demonstration_policy,
    emit_candidate_bundle,
};
use target_elements::{ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript};

use crate::deployment::{LinkDeploymentParameters, SelfCommitmentStrategy};

/// The reviewed contract, unmodified.
fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// The validated compact-ASH plan, derived once and handed out by
/// clone.
fn compact_ash_plan() -> ValidatedTargetOperationPlan {
    static PLAN: LazyLock<ValidatedTargetOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let realization =
            derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
        let scope = CompilationScope::from_operations([OperationId::CompactAsh])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_compact_ash_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// The placeholder symbols the bundle is laid out against.
///
/// The backend's own fixture values, repeated here so the link is
/// resolving away exactly the placeholders the bundle recorded.
fn placeholder_symbols(target: &ReviewedElementsTapscriptDefinition) -> CompactAshSymbols {
    CompactAshSymbols::new(
        target,
        vec![0x11; 32],
        vec![0x22; 32],
        vec![0x33; 32],
        1,
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .expect("the placeholder symbols are the reviewed widths")
}

/// The resolved symbol values of the demonstration link.
///
/// Different from the placeholders in every field, and different in
/// *width* for the two witness programs, so a link that substituted
/// nothing and a link that patched bytes in place both fail rather than
/// pass by coincidence.
fn resolved_symbols(target: &ReviewedElementsTapscriptDefinition) -> CompactAshSymbols {
    CompactAshSymbols::new(
        target,
        vec![0xa1; 32],
        vec![0xa2; 32],
        vec![0xa3; 20],
        1,
        vec![0xa4; 32],
        0,
        vec![0xa5; 32],
    )
    .expect("the resolved symbols are the reviewed widths")
}

/// The demonstration relocatable bundle.
fn relocatable_bundle() -> CandidateRelocatableTapscriptBundle {
    let target = reviewed_target();
    emit_candidate_bundle(
        &target,
        &compact_ash_plan(),
        demonstration_policy(),
        placeholder_symbols(&target),
    )
    .expect("the demonstration bundle emits")
}

/// The demonstration deployment parameters under one strategy.
fn deployment(
    target: &ReviewedElementsTapscriptDefinition,
    strategy: SelfCommitmentStrategy,
) -> LinkDeploymentParameters {
    LinkDeploymentParameters::new(
        target,
        resolved_symbols(target),
        vec![0xa6; 32],
        strategy,
        NonZeroU32::new(8).expect("eight is nonzero"),
    )
    .expect("the demonstration deployment parameters are the reviewed widths")
}
