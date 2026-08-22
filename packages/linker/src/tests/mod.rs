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
mod guide13_reproductions;
mod link_tests;
mod live_link_tests;
mod live_taptree_tests;
mod public_api_tests;
mod relocation_tests;
mod symbol_tests;
mod taptree_tests;

use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::live_transfer_plan::{
    LiveTransferRepresentationPlan, ValidatedLiveTransferOperationPlan,
    plan_live_transfer_target_operation,
};
use compiler::operation_plan::{
    PlacementSearchLimits, ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
};
use realization::{RealizationScope, derive};
use tapscript::{
    CandidateRelocatableLiveTransferBundle, CandidateRelocatableTapscriptBundle, CompactAshSymbols,
    LiveTransferSymbols, OwnerKey, demonstration_live_shape_set, demonstration_policy,
    derive_live_receipt_constructor, emit_candidate_bundle, emit_candidate_live_bundle,
    owner_key_encoding_closure, static_transfer_leaf_set,
};
use target_elements::{ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript};

use crate::deployment::{LinkDeploymentParameters, SelfCommitmentStrategy};
use crate::live_deployment::LiveLinkDeploymentParameters;

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
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .expect("the placeholder symbols are the reviewed widths")
}

/// The resolved symbol values of the demonstration link.
///
/// Different from the placeholders in every field, and different in
/// *width* for the sponsor-change witness program, so a link that
/// substituted nothing and a link that patched bytes in place both fail
/// rather than pass by coincidence.
fn resolved_symbols(target: &ReviewedElementsTapscriptDefinition) -> CompactAshSymbols {
    CompactAshSymbols::new(
        target,
        vec![0xa1; 32],
        vec![0xa2; 32],
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

// --- Guide-13 live-transfer fixtures ----------------------------------

/// The validated live-transfer plan, derived once and handed out by
/// clone.
///
/// The derivation is the whole scoped analysis, its validator, the
/// Phase-5 representation filter, and an independent re-derivation of the
/// assembled plan. A clone of the finished value is the same value by
/// construction, so nothing about what is tested changes — only how many
/// times the compiler is asked for it.
fn live_transfer_plan() -> ValidatedLiveTransferOperationPlan {
    static PLAN: LazyLock<ValidatedLiveTransferOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let realization =
            derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
        let scope = CompilationScope::from_operations([OperationId::TransferLive])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_live_transfer_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// The placeholder live symbols the bundles are laid out against.
///
/// The backend's own fixture values, repeated here so a link is resolving
/// away exactly the placeholders the bundles recorded.
fn placeholder_live_symbols(target: &ReviewedElementsTapscriptDefinition) -> LiveTransferSymbols {
    LiveTransferSymbols::new(
        target,
        vec![0x5a; 32],
        vec![0x22; 32],
        1,
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .expect("the placeholder symbols are the reviewed widths")
}

/// The resolved live symbol values of the demonstration link.
///
/// Different from the placeholders in every field, and different in
/// *width* for the sponsor-change witness program, so a link that
/// substituted nothing and a link that patched bytes in place both fail
/// rather than pass by coincidence. The destination program version is
/// left at one because it is the version a taproot output is read at
/// under the reviewed contract, and a different value would be a claim
/// about a different target rather than a different deployment.
fn resolved_live_symbols(target: &ReviewedElementsTapscriptDefinition) -> LiveTransferSymbols {
    LiveTransferSymbols::new(
        target,
        vec![0xb1; 32],
        vec![0xb2; 32],
        1,
        vec![0xb4; 32],
        0,
        vec![0xb5; 32],
    )
    .expect("the resolved symbols are the reviewed widths")
}

/// The demonstration owner's canonical metadata.
///
/// Public test material `(´[ADR015-rule:security:test-material]´)`: a
/// distinguishable byte string at the approved encoding's exact width,
/// standing for no party and paired with no secret. §1.10 gives
/// [`OwnerKey`] no way to hold a private key, and nothing here would have
/// one to offer.
fn live_owner() -> OwnerKey {
    owner_key(&[0x11; 32])
}

/// One owner's canonical metadata from fixture bytes.
fn owner_key(bytes: &[u8]) -> OwnerKey {
    let target = reviewed_target();
    let closure = owner_key_encoding_closure(target.definition().authorization());
    OwnerKey::new(&closure, closure.approved(), bytes.to_vec())
        .expect("the fixture is the approved encoding at its exact width")
}

/// One representation's relocatable bundle for one owner.
fn single_live_bundle(
    representation: LiveTransferRepresentationPlan,
    owner: &[u8],
) -> CandidateRelocatableLiveTransferBundle {
    let target = reviewed_target();
    let shapes = demonstration_live_shape_set();
    let leaves = static_transfer_leaf_set(representation, &shapes);
    let constructor = derive_live_receipt_constructor(
        &target,
        &live_transfer_plan(),
        representation,
        owner_key(owner),
        shapes,
        leaves,
    )
    .expect("the demonstration constructor derives");

    emit_candidate_live_bundle(
        &target,
        &live_transfer_plan(),
        &constructor,
        placeholder_live_symbols(&target),
    )
    .expect("the demonstration bundle emits")
}

/// Both representations' relocatable bundles for the demonstration owner.
///
/// Two bundles rather than one, because §11.3 keeps the representations'
/// leaf sets disjoint and each emission produces a bundle for one
/// constructor. Linking both is what makes §11.5's per-plan comparison a
/// comparison at all.
fn live_bundles() -> Vec<CandidateRelocatableLiveTransferBundle> {
    vec![
        single_live_bundle(LiveTransferRepresentationPlan::Explicit, &[0x11; 32]),
        single_live_bundle(
            LiveTransferRepresentationPlan::PrivateCommitted,
            &[0x11; 32],
        ),
    ]
}

/// The demonstration live deployment parameters.
fn live_deployment(target: &ReviewedElementsTapscriptDefinition) -> LiveLinkDeploymentParameters {
    live_deployment_at_depth(target, 8)
}

/// The demonstration live deployment parameters at one depth policy.
fn live_deployment_at_depth(
    target: &ReviewedElementsTapscriptDefinition,
    depth: u32,
) -> LiveLinkDeploymentParameters {
    LiveLinkDeploymentParameters::new(
        target,
        resolved_live_symbols(target),
        vec![0xb6; 32],
        NonZeroU32::new(depth).expect("the fixture depths are nonzero"),
    )
    .expect("the demonstration deployment parameters are the reviewed widths")
}
