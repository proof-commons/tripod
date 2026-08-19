//! Tests for the typed target-contract version.

use crate::definition::{TargetContractVersion, reviewed_elements_tapscript};
use crate::error::TargetError;

#[test]
fn the_second_contract_revision_is_supported() {
    let version = TargetContractVersion::supported(2).expect("V2 is implemented");
    assert_eq!(version, TargetContractVersion::V2);
    assert_eq!(version.get(), 2);
}

#[test]
fn an_unimplemented_contract_revision_is_refused() {
    // Refusal is the point: a consumer that cannot interpret a
    // revision must not proceed on a guess. Revision one is in this
    // list because the crate names it without implementing it, which
    // is the sharpest case: the number exists, so refusing it has to
    // be a decision rather than an accident of the number being
    // unfamiliar.
    for offered in [0_u32, 1, 3, u32::MAX] {
        assert_eq!(
            TargetContractVersion::supported(offered),
            Err(TargetError::UnsupportedTargetContractVersion { offered }),
        );
    }
}

#[test]
fn the_reviewed_contract_is_the_second_revision() {
    // The census and the success algebra both changed, so the reviewed
    // contract is V2 and says so. A reviewed contract still claiming V1
    // would be the silent widening the revision exists to prevent
    // (´[PLAN-rule:guide10:target-version]´).
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(reviewed.definition().version(), TargetContractVersion::V2);
    assert_ne!(reviewed.definition().version(), TargetContractVersion::V1);
}

#[test]
fn the_historical_revision_keeps_its_number_and_is_not_advertised() {
    // V1 is not renumbered, not aliased to V2, and not removed: a
    // consumer pinned to revision one is pinned to what revision one
    // described, and the only way to keep that promise is for the
    // number to stay attached to the same value. What it is not is
    // supported. This crate implements one validator, applying one
    // census and one operand and success algebra, and that is V2's.
    // Advertising V1 under it meant either refusing a genuine V1
    // contract for primitives it never had, or accepting a V2 body
    // that called itself V1.
    assert_eq!(TargetContractVersion::V1.get(), 1);
    assert_ne!(TargetContractVersion::V1, TargetContractVersion::V2);
    assert!(!TargetContractVersion::SUPPORTED.contains(&TargetContractVersion::V1));
}

#[test]
fn the_supported_census_is_exact_and_duplicate_free() {
    // The expected census is written out here rather than derived from
    // the constant under test.
    let expected = [TargetContractVersion::V2];
    assert_eq!(TargetContractVersion::SUPPORTED, &expected);

    let mut seen = std::collections::BTreeSet::new();
    for version in TargetContractVersion::SUPPORTED {
        assert!(seen.insert(*version), "duplicate supported revision");
    }
}
