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

use compiler::{AnalysisPolicy, CompilationScope, bind_input};

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
        AnalysisPolicy::Strict,
    )
    .unwrap();

    assert_eq!(
        bound.scope().operations(),
        [OperationId::TransferLive, OperationId::CompactAsh],
        "scope is canonical stable-code order",
    );
    assert_eq!(bound.policy(), AnalysisPolicy::Strict);
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
            AnalysisPolicy::Strict,
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
            AnalysisPolicy::Strict,
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
        bind_input(
            &mutated,
            phase1_realization(),
            scope,
            AnalysisPolicy::Strict,
        )
        .unwrap_err(),
        CompileError::ArchitectureBindingMismatch,
    );
}
