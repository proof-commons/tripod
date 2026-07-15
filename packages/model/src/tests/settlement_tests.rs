//! Distribution settlement tests.
//!
//! Implements `´test:verification:settlement´`.
//!
//! The cross-cycle test should construct both cycles inside one world
//! rather than using an outpoint from another world. A production
//! version should create two live distributions sequentially and
//! attempt to settle one with the other's entitlement.

use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use crate::*;

#[test]
fn partial_settlement_preserves_class_payability() {
    let (world, cycle) = world_with_distribution(&[(ALICE, ALICE, 400), (BOB, BOB, 600)]);

    let alice_entitlement = find_entitlement(&world, ALICE);

    let next = settle_batch(&world, cycle, vec![alice_entitlement]);

    check_invariant(&next).unwrap();

    let control = find_distribution_control(&next, cycle);

    let view = read_distribution_control(next.utxo(control).unwrap()).unwrap();

    assert_eq!(view.remaining_principal, sat(600));

    let vault = find_distribution_vault(&next, cycle).unwrap();

    assert_eq!(
        next.utxo(vault).unwrap().value,
        view.remaining_live_value
            .checked_add(view.remaining_time_locked_value)
            .unwrap(),
    );
}

#[test]
fn terminal_settlement_closes_control_and_vault() {
    let (world, cycle) = world_with_distribution(&[(ALICE, ALICE, 400), (BOB, BOB, 600)]);

    let entitlements = find_all_entitlements_for_cycle(&world, cycle);

    let next = settle_batch(&world, cycle, entitlements);

    assert!(next.utxos.values().all(|utxo| {
        !matches!(
            utxo.meta,
            Meta::DistributionControl {
                cycle: control_cycle,
                ..
            } if control_cycle == cycle
        ) && !matches!(
            utxo.meta,
            Meta::DistributionVault {
                cycle: vault_cycle,
            } if vault_cycle == cycle
        )
    }));

    let certificate = next.history.transitions.last().unwrap();

    assert!(certificate.distribution_residue.is_some());

    check_invariant(&next).unwrap();
}

#[test]
fn zero_draw_entitlements_are_retired_permissionlessly() {
    let (world, cycle) = world_with_distribution(&[(ALICE, ALICE, 1), (BOB, BOB, 2)]);

    let entitlements = find_all_entitlements_for_cycle(&world, cycle);

    let next = settle_batch(&world, cycle, entitlements);

    let alice_receipts = find_receipts(&next, ALICE, ReceiptClass::Live);

    let bob_receipts = find_receipts(&next, BOB, ReceiptClass::Live);

    assert_eq!(alice_receipts, [] as [u64; 0]);

    assert_eq!(bob_receipts, [] as [u64; 0]);

    assert!(next.utxos.values().all(|utxo| {
        !matches!(
            utxo.meta,
            Meta::DepositEntitlement {
                target_cycle,
                ..
            } if target_cycle == cycle
        )
    }));

    check_invariant(&next).unwrap();
}

#[test]
fn cross_cycle_entitlement_is_rejected() {
    // Both distributions are constructed inside one world, per the
    // spec note: an outpoint from a separate world would collide with
    // a valid entitlement under the deterministic outpoint sequence.
    let (first, first_cycle) = world_with_distribution(&[(ALICE, ALICE, 100)]);

    let with_second = create_request_for(&first, BOB, BOB, sat(100), Sat::ONE);

    let admitted = admit_all_requests(&with_second);

    let second = run_forced_cycle(&admitted);

    // BOB's entitlement targets the second distribution's cycle.
    let foreign_entitlement = find_entitlement(&second, BOB);

    let control = find_distribution_control(&second, first_cycle);

    let vault = find_distribution_vault(&second, first_cycle);

    assert_eq!(
        SettleDistribution {
            control,
            vault,
            entitlements: vec![foreign_entitlement],
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&second, next_order(&second)),
        Err(Guard::WrongTarget),
    );
}

#[test]
fn settlement_after_maturity_preserves_original_class() {
    let (distributed, cycle) = world_with_distribution(&[(ALICE, ALICE, 1_000)]);

    let matured = mature_world(&distributed);

    let entitlement = find_entitlement(&matured, ALICE);

    let settled = settle_batch(&matured, cycle, vec![entitlement]);

    assert_ne!(
        find_receipts(&settled, ALICE, ReceiptClass::TimeLocked),
        [] as [u64; 0]
    );

    check_invariant(&settled).unwrap();
}
