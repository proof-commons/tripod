//! Admission tests.
//!
//! Implements `´test:verification:admission´`.
//!
//! The ceiling fixture directly mutates state and is therefore
//! suitable only as a low-level branch test. Property tests should
//! construct near-ceiling states through valid transitions or a
//! trusted fixture builder.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn admission_preserves_exact_principal_partition() {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));

    let request = find_request(&world, BOB);

    let old_state = world.state().unwrap().1;

    let old_resv = world.active_resv().unwrap().1.value;

    let next = apply_checked(
        &world,
        &AdmitDeposits {
            requests: vec![request],
            admission_reward: sat(40),
            reward_owner: RELAYER,
        },
        next_order(&world),
    );

    let new_state = next.state().unwrap().1;

    let new_resv = next.active_resv().unwrap().1.value;

    assert_eq!(new_state.q.checked_sub(old_state.q).unwrap(), sat(900));

    assert_eq!(new_resv.checked_sub(old_resv).unwrap(), sat(900));

    let entitlement = find_entitlement(&next, BOB);

    assert_eq!(next.utxo(entitlement).unwrap().value, sat(900));

    let certificate = next.history.transitions.last().unwrap();

    assert_eq!(certificate.chain_fee, sat(60));
}

#[test]
fn admission_creates_one_entitlement_per_request() {
    let mut world = test_fixtures::world();

    world = create_request_for(&world, ALICE, BOB, sat(3), sat(1));

    world = create_request_for(&world, ALICE, BOB, sat(5), sat(1));

    let requests = world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| {
            if matches!(
                utxo.meta,
                Meta::DepositRequest {
                    receipt_owner,
                    ..
                } if receipt_owner == BOB
            ) {
                Some(*outpoint)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let next = apply_checked(
        &world,
        &AdmitDeposits {
            requests,
            admission_reward: Sat::ZERO,
            reward_owner: RELAYER,
        },
        next_order(&world),
    );

    let entitlements = next
        .utxos
        .values()
        .filter(|utxo| {
            matches!(
                (utxo.asset, utxo.meta),
                (
                    Asset::Ent,
                    Meta::DepositEntitlement {
                        owner,
                        ..
                    },
                ) if owner == BOB
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(entitlements.len(), 2);

    let mut values = entitlements
        .iter()
        .map(|utxo| utxo.value)
        .collect::<Vec<_>>();

    values.sort();

    assert_eq!(values, vec![sat(3), sat(5)]);
}

#[test]
fn admission_rejects_cross_pool_request() {
    let mut world = test_fixtures::world();

    world.adversary.lbtc = sat(100);

    let request = world.next_outpoint;

    world = inject_open_object(
        &world,
        Asset::Lbtc,
        sat(100),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id + 1,
            refund_key: ALICE,
            receipt_owner: BOB,
            deposit_principal: sat(90),
        },
    )
    .unwrap();

    assert_eq!(
        AdmitDeposits {
            requests: vec![request],
            admission_reward: Sat::ZERO,
            reward_owner: RELAYER,
        }
        .apply(&world, next_order(&world)),
        Err(Guard::WrongPool),
    );
}

#[test]
fn admission_ceiling_failure_leaves_requests_unspent() {
    // Reachable near-cap fixture: valid genesis just below the
    // reserve cap plus a request whose principal exceeds the
    // remaining headroom.
    let world = genesis(
        test_fixtures::constants(),
        Sat::new(ACTIVE_BACKING_MAX - 10).unwrap(),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        test_fixtures::txid(0),
    )
    .unwrap();

    let world = create_request_for(&world, ALICE, BOB, sat(20), sat(1));

    let request = find_request(&world, BOB);

    assert_eq!(
        AdmitDeposits {
            requests: vec![request],
            admission_reward: Sat::ZERO,
            reward_owner: RELAYER,
        }
        .apply(&world, next_order(&world)),
        Err(Guard::ActiveBackingCapExceeded),
    );

    assert!(world.utxos.contains_key(&request));
}

/// The admission reward is admitter-chosen only **within** the
/// preauthorized budget: one unit past the budget boundary is
/// rejected, and the exact boundary is accepted.
#[test]
fn admission_reward_exceeding_budget_is_rejected() {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));

    let request = find_request(&world, BOB);

    assert_eq!(
        AdmitDeposits {
            requests: vec![request],
            admission_reward: sat(101),
            reward_owner: RELAYER,
        }
        .apply(&world, next_order(&world)),
        Err(Guard::PartitionPin),
    );

    // The exact boundary is within the authorized envelope.
    let _next = apply_checked(
        &world,
        &AdmitDeposits {
            requests: vec![request],
            admission_reward: sat(100),
            reward_owner: RELAYER,
        },
        next_order(&world),
    );
}
