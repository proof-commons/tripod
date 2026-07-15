//! Open-asset immunity tests.
//!
//! Implements `´test:verification:open-asset-immunity´`.

use super::test_fixtures::*;
use crate::*;

#[test]
fn malformed_this_pool_request_is_inert() {
    let world = external_lbtc_world(10_000);

    let (next, request_outpoint) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(100),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id,
            refund_key: ALICE,
            receipt_owner: BOB,
            deposit_principal: sat(100),
        },
    );

    check_invariant(&next).unwrap();

    assert_eq!(
        validate_request_for_admission(&next.constants, next.utxo(request_outpoint).unwrap()),
        Err(Guard::PartitionPin),
    );
}

#[test]
fn cross_pool_request_is_inert() {
    let world = external_lbtc_world(10_000);

    let (next, request_outpoint) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(100),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id.wrapping_add(1),
            refund_key: ALICE,
            receipt_owner: BOB,
            deposit_principal: sat(90),
        },
    );

    check_invariant(&next).unwrap();

    assert_eq!(
        validate_request_for_admission(&next.constants, next.utxo(request_outpoint).unwrap()),
        Err(Guard::WrongPool),
    );
}

#[test]
fn decoy_resv_does_not_replace_active_resv() {
    let mut world = external_lbtc_world(2_000_000);

    let active = world.roots.resv.unwrap();

    let active_value = world.utxo(active).unwrap().value;

    let (next, decoy) = inject_and_get_outpoint(&world, Asset::Lbtc, active_value, Meta::Resv);

    world = next;

    check_invariant(&world).unwrap();

    assert_eq!(world.roots.resv, Some(active));

    assert_ne!(world.roots.resv, Some(decoy));
}

#[test]
fn zero_value_anchor_junk_is_inert() {
    let world = external_lbtc_world(1);

    let (next, _anchor) = inject_and_get_outpoint(&world, Asset::Lbtc, Sat::ZERO, Meta::CpfpAnchor);

    check_invariant(&next).unwrap();
}

#[test]
fn foreign_state_shaped_object_is_inert() {
    let mut world = world();

    world.adversary.foreign.insert(42, sat(10));

    let fake_state = PoolState {
        omega: Sat::ZERO,
        y_l: Sat::ZERO,
        y_t: Sat::ZERO,
        q: Sat::ZERO,
        cycle: 999,
        maturity: Maturity::Complete,
    };

    let (next, _fake) = inject_and_get_outpoint(
        &world,
        Asset::Foreign(42),
        Sat::ONE,
        Meta::State(fake_state),
    );

    check_invariant(&next).unwrap();

    let (state_outpoint, state) = next.state().unwrap();

    assert_eq!(state_outpoint, next.roots.state);

    assert_ne!(state, fake_state);
}
