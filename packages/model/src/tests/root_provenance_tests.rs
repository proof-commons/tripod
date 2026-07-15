//! Root-provenance corruption tests.
//!
//! Implements `´test:verification:root-provenance´`.

use super::advanced_fixtures;
use super::scenario_fixtures::*;
use super::test_fixtures;
use super::test_fixtures::{external_lbtc_world, inject_and_get_outpoint};
use crate::*;

#[test]
fn decoy_resv_does_not_change_active_cursor() {
    let mut world = external_lbtc_world(2_000_000);

    let active = world.roots.resv.unwrap();

    let active_value = world.utxo(active).unwrap().value;

    let (next, decoy) = inject_and_get_outpoint(&world, Asset::Lbtc, active_value, Meta::Resv);

    world = next;

    assert_eq!(world.roots.resv, Some(active));
    assert_ne!(world.roots.resv, Some(decoy));

    check_invariant(&world).unwrap();
}

#[test]
fn weld_exempt_clear_preserves_resv_provenance() {
    let mut world = test_fixtures::world();

    world = advanced_fixtures::give_live_receipt(&world, ALICE, sat(100));

    world = advanced_fixtures::burn_from_owner(&world, ALICE, sat(100), Vec::new());

    let old_resv = world.roots.resv;

    let next = ClearAsh {
        ash_inputs: find_ash(&world),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    assert_eq!(next.roots.resv, old_resv);

    let certificate = next.history.transitions.last().unwrap();

    assert!(certificate.state_edge.is_some());
    assert!(certificate.resv_edge.is_none());

    check_invariant(&next).unwrap();
}

#[test]
fn cursor_corruption_is_detected() {
    let mut world = test_fixtures::world();

    world.adversary.lbtc = sat(2_000_000);

    let active_value = world.active_resv().unwrap().1.value;

    let (mut corrupted, decoy) =
        inject_and_get_outpoint(&world, Asset::Lbtc, active_value, Meta::Resv);

    corrupted.roots.resv = Some(decoy);

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::ResvSuccession),
    );
}
