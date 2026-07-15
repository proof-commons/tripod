//! Distribution-bijection corruption tests.
//!
//! Implements `´test:verification:distribution-bijection´`.

use super::corruption_fixtures::*;
use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn orphan_vault_is_rejected() {
    let world = test_fixtures::world();

    let corrupted = add_closed_utxo(
        &world,
        Utxo {
            asset: Asset::U,
            value: sat(10),
            meta: Meta::DistributionVault { cycle: 999 },
        },
    );

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::DistributionPayability),
    );
}

#[test]
fn duplicate_vault_is_rejected() {
    let (world, cycle) = world_with_distribution(&[(ALICE, ALICE, 1_000)]);

    let vault = find_distribution_vault(&world, cycle).unwrap();

    let duplicate = *world.utxo(vault).unwrap();

    let corrupted = add_closed_utxo(&world, duplicate);

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::DistributionPayability),
    );
}

#[test]
fn control_without_matching_entitlement_is_rejected() {
    let (world, cycle) = world_with_distribution(&[(ALICE, ALICE, 1_000)]);

    let entitlement = find_entitlement(&world, ALICE);

    let mut corrupted = world;

    corrupted.utxos.remove(&entitlement);

    assert!(matches!(
        check_invariant(&corrupted),
        Err(InvariantError::EntitlementLifecycle)
    ));

    let _ = cycle;
}

#[test]
fn vault_value_mismatch_is_rejected() {
    let (world, cycle) = world_with_distribution(&[(ALICE, ALICE, 1_000)]);

    let vault = find_distribution_vault(&world, cycle).unwrap();

    let old = *world.utxo(vault).unwrap();

    let corrupted = replace_utxo(
        &world,
        vault,
        Utxo {
            value: old.value.checked_sub(Sat::ONE).unwrap(),
            ..old
        },
    );

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::DistributionPayability),
    );
}
