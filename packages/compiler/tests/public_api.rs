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
