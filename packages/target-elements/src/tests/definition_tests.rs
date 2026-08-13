//! Tests for the typed target-contract version.

use crate::definition::TargetContractVersion;
use crate::error::TargetError;

#[test]
fn the_first_contract_revision_is_supported() {
    let version = TargetContractVersion::supported(1).expect("V1 is implemented");
    assert_eq!(version, TargetContractVersion::V1);
    assert_eq!(version.get(), 1);
}

#[test]
fn an_unimplemented_contract_revision_is_refused() {
    // Refusal is the point: a consumer that cannot interpret a
    // revision must not proceed on a guess.
    for offered in [0_u32, 2, u32::MAX] {
        assert_eq!(
            TargetContractVersion::supported(offered),
            Err(TargetError::UnsupportedTargetContractVersion { offered }),
        );
    }
}

#[test]
fn the_supported_census_is_exact_and_duplicate_free() {
    // The expected census is written out here rather than derived from
    // the constant under test.
    let expected = [TargetContractVersion::V1];
    assert_eq!(TargetContractVersion::SUPPORTED, &expected);

    let mut seen = std::collections::BTreeSet::new();
    for version in TargetContractVersion::SUPPORTED {
        assert!(seen.insert(*version), "duplicate supported revision");
    }
    assert_eq!(seen.len(), TargetContractVersion::SUPPORTED.len());
}

#[test]
fn the_error_root_displays_its_offered_version() {
    let error = TargetError::UnsupportedTargetContractVersion { offered: 7 };
    assert_eq!(error.to_string(), "unsupported target contract version 7");
}
