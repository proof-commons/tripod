//! Request-lifecycle tests.
//!
//! Implements `´test:verification:request-lifecycle´`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn conforming_request_can_be_created() {
    let world = test_fixtures::world();

    let next = create_request_for(&world, ALICE, BOB, sat(900), sat(100));

    let request = find_request(&next, BOB);

    let view =
        validate_request_for_admission(&next.constants, next.utxo(request).unwrap()).unwrap();

    assert_eq!(view.deposit_principal, sat(900));

    assert_eq!(view.admission_budget, sat(100));

    check_invariant(&next).unwrap();
}

#[test]
fn malformed_open_request_is_inert_but_not_admissible() {
    let mut world = test_fixtures::world();

    world.adversary.lbtc = sat(100);

    let outpoint = world.next_outpoint;

    let next = inject_open_object(
        &world,
        Asset::Lbtc,
        sat(100),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id,
            refund_key: ALICE,
            receipt_owner: BOB,
            deposit_principal: sat(100),
        },
    )
    .unwrap();

    check_invariant(&next).unwrap();

    assert_eq!(
        validate_request_for_admission(&next.constants, next.utxo(outpoint).unwrap()),
        Err(Guard::PartitionPin),
    );
}

#[test]
fn request_cancellation_refunds_full_request_value() {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));

    let request = find_request(&world, BOB);

    let request_value = world.utxo(request).unwrap().value;

    let next = apply_checked(
        &world,
        &CancelRequest {
            request,
            signers: signers(&[ALICE]),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let refund_total = next
        .utxos
        .values()
        .filter_map(|utxo| match (utxo.asset, utxo.meta) {
            (Asset::Lbtc, Meta::PlainLbtc { owner }) if owner == ALICE => Some(utxo.value),

            _ => None,
        })
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();

    assert!(refund_total >= request_value);

    assert!(!next.utxos.contains_key(&request));

    check_invariant(&next).unwrap();
}

#[test]
fn wrong_refund_signer_cannot_cancel() {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));

    let request = find_request(&world, BOB);

    assert_eq!(
        CancelRequest {
            request,
            signers: signers(&[BOB]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::BadSignature),
    );
}
