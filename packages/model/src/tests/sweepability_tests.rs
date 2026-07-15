//! Sweepability and sponsored-quiescence tests.
//!
//! Implements `´test:verification:sweepability´`.
//!
//! The full quiescence theorem excludes malformed open junk and
//! underfunded lost-refund requests from its zero-queue claim.

use super::advanced_fixtures::*;
use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn all_distributions_are_permissionlessly_collectable() {
    let (world, _cycle) = world_with_distribution(&[(ALICE, ALICE, 400), (BOB, BOB, 600)]);

    let (swept, report) = drive_shared_state_sweepability(&world, 100).unwrap();

    assert_eq!(report.entitlements, 0);
    assert_eq!(report.live_distributions, 0);

    check_invariant(&swept).unwrap();
}

#[test]
fn lost_owner_keys_do_not_block_distribution_cleanup() {
    let (world, _cycle) = world_with_distribution(&[(ALICE, ALICE, 500), (BOB, BOB, 500)]);

    // Scheduler carries no ALICE/BOB receipt-owner
    // authorization. Settlement is permissionless.
    let (swept, report) = drive_shared_state_sweepability(&world, 100).unwrap();

    assert_eq!(report.entitlements, 0);
    assert_eq!(report.live_distributions, 0);

    assert!(
        !find_receipts(&swept, ALICE, ReceiptClass::Live).is_empty()
            || !find_receipts(&swept, ALICE, ReceiptClass::TimeLocked).is_empty(),
    );
}

#[test]
fn ash_compacts_to_at_most_one_residual() {
    let mut world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    world = burn_from_owner(&world, ALICE, sat(50), Vec::new());

    world = give_live_receipt(&world, BOB, sat(50));

    world = burn_from_owner(&world, BOB, sat(50), Vec::new());

    let (swept, report) = drive_shared_state_sweepability(&world, 100).unwrap();

    assert!(report.ash_outputs <= 1);

    check_invariant(&swept).unwrap();
}

#[test]
fn full_quiescence_processes_requests_cycles_and_distributions() {
    let mut world = create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    world = create_request_for(&world, BOB, BOB, sat(200), Sat::ONE);

    let (quiet, report) = drive_sponsored_quiescence(&world, 1_000).unwrap();

    assert_eq!(report.admissible_requests, 0);

    assert_eq!(report.entitlements, 0);

    assert_eq!(report.live_distributions, 0);

    assert!(report.ash_outputs <= 1);

    check_invariant(&quiet).unwrap();
}

#[test]
fn wallets_are_inert_in_v13() {
    // `World::wallets` is reserved future work (´def:verification:wallets´):
    // no v13 operation reads or writes the map, so a full
    // sponsored-quiescence trace leaves it untouched.
    let mut world = create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    world = create_request_for(&world, BOB, BOB, sat(200), Sat::ONE);

    let (quiet, _report) = drive_sponsored_quiescence(&world, 1_000).unwrap();

    assert_eq!(quiet.wallets, world.wallets);
}
