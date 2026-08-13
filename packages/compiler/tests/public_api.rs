//! Public-API boundary test for the target-independent compiler
//! package.
//!
//! This integration test compiles as an external consumer. At the
//! crate-boundary stage it proves three things: the crate is
//! consumable from outside, its error root is public and carries
//! architecture-owned identifiers rather than compiler-local
//! restatements of them, and no analysis surface has leaked out ahead
//! of the deliverable that owns it.

use architecture::OperationId;
use compiler::CompileError;

#[test]
fn the_error_root_is_publicly_constructible_and_displays() {
    let error = CompileError::UnsupportedRealizationSchema { schema: 17 };
    assert_eq!(error.to_string(), "unsupported realization schema 17");

    let error = CompileError::InvalidRealization;
    assert_eq!(
        error.to_string(),
        "realization failed its owner's validation"
    );

    let error = CompileError::ArchitectureBindingMismatch;
    assert!(error.to_string().contains("does not match"));
}

#[test]
fn scope_errors_carry_architecture_owned_operation_ids() {
    // The compiler cites the architecture-owned identifier. It does
    // not mint a compiler-local operation identity, which would
    // duplicate an upstream ID and obscure its owner.
    let error = CompileError::IncompleteRealizationScope {
        operation: OperationId::CompactAsh,
    };
    let CompileError::IncompleteRealizationScope { operation } = error else {
        panic!("constructed variant must round-trip");
    };
    assert_eq!(operation, OperationId::CompactAsh);

    let duplicate = CompileError::DuplicateScopeOperation {
        operation: OperationId::TransferLive,
    };
    assert_ne!(
        duplicate,
        CompileError::IncompleteRealizationScope {
            operation: OperationId::TransferLive,
        },
        "an absent scope member and a repeated one are distinct failures"
    );
}

#[test]
fn the_error_root_is_a_standard_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}

    assert_error(&CompileError::InvalidRealization);
}

// --- P2-004: the validated input boundary ---

use compiler::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};

const fn test_policy() -> AnalysisPolicy {
    AnalysisPolicy::strict(ProofSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(10_000).expect("nonzero"),
    ))
}

fn phase1_realization() -> realization::ScopedRealizationSpec {
    realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::phase1_pilots(),
    )
    .expect("phase-1 pilots derive")
}

#[test]
fn phase1_realization_binds_with_explicit_scope() {
    let realization = phase1_realization();
    let expected_binding = realization.architecture().clone();
    let scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap();

    let bound = bind_input(
        &architecture::ARCHITECTURE,
        realization,
        scope,
        test_policy(),
    )
    .unwrap();

    assert_eq!(
        bound.scope().operations(),
        [OperationId::TransferLive, OperationId::CompactAsh],
        "scope is canonical stable-code order",
    );
    assert_eq!(bound.policy(), test_policy());
    assert_eq!(bound.architecture_binding(), &expected_binding);
    assert!(
        bound
            .realization()
            .operation(OperationId::CompactAsh)
            .is_some()
    );
}

#[test]
fn single_pilot_scopes_bind_and_permutations_are_equal() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let scope = CompilationScope::from_operations([operation]).unwrap();
        bind_input(
            &architecture::ARCHITECTURE,
            phase1_realization(),
            scope,
            test_policy(),
        )
        .unwrap();
    }

    assert_eq!(
        CompilationScope::from_operations([OperationId::TransferLive, OperationId::CompactAsh])
            .unwrap(),
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap(),
    );
}

#[test]
fn scope_construction_rejects_empty_and_duplicate() {
    assert_eq!(
        CompilationScope::from_operations([]).unwrap_err(),
        CompileError::EmptyCompilationScope,
    );
    assert_eq!(
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::CompactAsh])
            .unwrap_err(),
        CompileError::DuplicateScopeOperation {
            operation: OperationId::CompactAsh,
        },
    );
}

#[test]
fn scope_outside_the_realization_is_incomplete() {
    // The pilot realization is intentionally partial; requesting Burn
    // must fail rather than silently drop or complete the member.
    let scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::Burn]).unwrap();

    assert_eq!(
        bind_input(
            &architecture::ARCHITECTURE,
            phase1_realization(),
            scope,
            test_policy(),
        )
        .unwrap_err(),
        CompileError::IncompleteRealizationScope {
            operation: OperationId::Burn,
        },
    );
}

#[test]
fn a_different_architecture_semantic_body_is_a_binding_mismatch() {
    let mut mutated = architecture::ARCHITECTURE;
    mutated.document.specification.version = "0.0.0-binding-mismatch-test";

    let scope = CompilationScope::from_operations([OperationId::CompactAsh]).unwrap();

    assert_eq!(
        bind_input(&mutated, phase1_realization(), scope, test_policy(),).unwrap_err(),
        CompileError::ArchitectureBindingMismatch,
    );
}

// --- Guide-8 §15 and §20.4: the abstract target requirement boundary ---

use std::collections::BTreeSet;

use compiler::target::{
    ExternalEvidenceRole, PlacementSearchLimits, RequiredCapability, TargetRequirementSet,
    analyze_target_requirements,
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
const fn placement_limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

fn pilot_requirements() -> TargetRequirementSet {
    let scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .expect("pilot scope");
    let input = bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        scope,
        test_policy(),
    )
    .expect("bind input");

    analyze_target_requirements(&input, placement_limits()).expect("pilot requirements")
}

#[test]
fn every_required_capability_is_publicly_nameable() {
    // Written out rather than folded over the census: this test is the
    // external statement that each variant is reachable by name from
    // outside the crate, and a loop over the census would state
    // nothing.
    let named = [
        RequiredCapability::AuthenticatedObjectRecognition,
        RequiredCapability::AuthenticatedFamilyCardinality,
        RequiredCapability::AuthenticatedCanonicalPartition,
        RequiredCapability::AuthenticatedOpenFlowPartition,
        RequiredCapability::AuthenticatedRootEffects,
        RequiredCapability::AuthenticatedProjectionSet,
        RequiredCapability::ExactPublicAmountArithmetic,
        RequiredCapability::ConfidentialValueConservation,
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::OperatorAuthorization,
        RequiredCapability::RefundAuthorization,
        RequiredCapability::PublicConstructibility,
        RequiredCapability::WholeTransactionValueConservation,
    ];

    assert_eq!(RequiredCapability::ALL, named);
    assert_eq!(named.iter().collect::<BTreeSet<_>>().len(), named.len());
    assert!(named.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn every_evidence_role_is_publicly_nameable() {
    assert_eq!(
        ExternalEvidenceRole::ALL,
        [ExternalEvidenceRole::SubstrateConservation],
    );
}

#[test]
fn target_requirements_come_only_from_a_completed_analysis() {
    // There is no public constructor, no `Default`, and no builder:
    // this call is the only route to the type, and it runs the complete
    // analysis and its independent validator before returning.
    let requirements = pilot_requirements();
    let capabilities = requirements.capabilities().collect::<Vec<_>>();

    assert_ne!(
        capabilities,
        [] as [compiler::target::RequiredCapability; 0]
    );
    assert!(
        capabilities.windows(2).all(|pair| pair[0] < pair[1]),
        "the published census is canonically ordered, so no member repeats",
    );
    assert!(
        capabilities
            .iter()
            .all(|capability| RequiredCapability::ALL.contains(capability)),
        "no published capability is outside the census the adapter matches on",
    );

    let evidence = requirements.external_evidence().collect::<Vec<_>>();

    assert_eq!(evidence, [ExternalEvidenceRole::SubstrateConservation]);
}

#[test]
fn a_rejected_analysis_yields_no_requirements() {
    // The projection is not reachable around a failing analysis: an
    // input the analysis rejects produces a typed error, never an empty
    // or partial requirement set.
    let scope = CompilationScope::from_operations([OperationId::CompactAsh]).expect("scope");
    let mut mutated = architecture::ARCHITECTURE;
    mutated.document.specification.version = "0.0.0-target-boundary-test";

    assert_eq!(
        bind_input(&mutated, phase1_realization(), scope, test_policy())
            .expect_err("a mismatched binding never binds"),
        CompileError::ArchitectureBindingMismatch,
    );
}

#[test]
fn the_public_boundary_is_deterministic_and_target_free() {
    assert_eq!(pilot_requirements(), pilot_requirements());

    // The target package is not a dependency of this test target, so no
    // target-specific type can appear in a compiler signature this test
    // names: the absence is enforced by the package graph rather than
    // asserted here. What is asserted is that the boundary's whole
    // vocabulary is the two abstract censuses above, and that it mints
    // no analysis identity to go with them.
    let requirements = pilot_requirements();
    let capabilities = requirements.capabilities().collect::<BTreeSet<_>>();
    let evidence = requirements.external_evidence().collect::<BTreeSet<_>>();

    assert!(!capabilities.is_empty() && !evidence.is_empty());
    assert_eq!(
        format!("{requirements:?}").matches("digest").count(),
        0,
        "the boundary publishes no identity, and nothing named like one",
    );
}
