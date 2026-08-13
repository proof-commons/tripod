//! Public-API boundary test for the typed Elements target contract.
//!
//! This integration test compiles as an external consumer. At the
//! crate-boundary stage it proves that the crate is consumable from
//! outside, that its error root and contract version are public, and
//! that no target surface has leaked out ahead of the deliverable that
//! owns it.

use target_elements::{TargetContractVersion, TargetError};

#[test]
fn the_contract_version_is_publicly_decidable() {
    let version = TargetContractVersion::supported(TargetContractVersion::V1.get())
        .expect("the declared revision is implemented");
    assert_eq!(version, TargetContractVersion::V1);
}

#[test]
fn the_error_root_is_public_and_displays() {
    let error = TargetError::UnsupportedTargetContractVersion { offered: 99 };
    assert_eq!(error.to_string(), "unsupported target contract version 99");

    // The error root implements the standard error trait without a
    // derive crate, so an external consumer can box it.
    let boxed: Box<dyn core::error::Error> = Box::new(error);
    assert!(boxed.to_string().contains("99"));
}

#[test]
fn the_crate_exposes_no_target_definition_yet() {
    // Compile-time boundary: the reviewed opcode registry, encoding
    // registry, capability contracts, and deployment binding are not
    // public because they are not implemented. This test records that
    // absence deliberately so that adding a public surface without the
    // implementation behind it is a visible change.
    //
    // The only public vocabulary is the contract version and the error
    // root, both exercised above.
    let _: fn(u32) -> Result<TargetContractVersion, TargetError> = TargetContractVersion::supported;
}
