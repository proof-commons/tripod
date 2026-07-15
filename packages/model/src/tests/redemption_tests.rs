//! Redemption tests.
//!
//! Implements `´test:verification:redemption´`.
//!
//! The `pending_q_blocks_sealing_redemption` fixture must ensure no
//! other live receipts exist outside the chosen sealing receipt. The
//! fixture merges all live receipts after admission; the pre-admission
//! world is fully matured and relabeled so no operator/distribution
//! positions remain outstanding.

use super::advanced_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn partial_redemption_preserves_floor() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let receipt = find_receipts(&world, ALICE, ReceiptClass::Live)[0];

    let old_state = world.state().unwrap().1;

    let next = RedeemReceipt {
        receipt,
        signers: signers(&[ALICE]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    let new_state = next.state().unwrap().1;

    let old_y = old_state.y().unwrap();
    let new_y = new_state.y().unwrap();

    assert!(
        u128::from(new_state.omega.get()) * u128::from(old_y.get())
            >= u128::from(old_state.omega.get()) * u128::from(new_y.get()),
    );

    assert_eq!(old_state.y_l.checked_sub(new_state.y_l).unwrap(), sat(100),);

    check_invariant(&next).unwrap();
}

#[test]
fn time_locked_receipt_cannot_redeem() {
    let world = test_fixtures::world();

    let receipt = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];

    assert_eq!(
        RedeemReceipt {
            receipt,
            signers: signers(&[GENESIS_OWNER]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::ClassCross),
    );
}

#[test]
fn wrong_owner_cannot_redeem() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let receipt = find_receipts(&world, ALICE, ReceiptClass::Live)[0];

    assert_eq!(
        RedeemReceipt {
            receipt,
            signers: signers(&[BOB]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::BadSignature),
    );
}

#[test]
fn pending_q_blocks_sealing_redemption() {
    let mut world = mature_and_relabel_all(&test_fixtures::world());

    world = create_request_for(&world, ALICE, ALICE, sat(100), Sat::ONE);

    world = admit_all_requests(&world);

    let world = merge_live_receipts(&world, GENESIS_OWNER);

    let receipt = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    assert_eq!(receipt_value(&world, receipt), world.state().unwrap().1.y_l);

    assert_eq!(
        RedeemReceipt {
            receipt,
            signers: signers(&[GENESIS_OWNER]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::NoTrap),
    );
}

#[test]
fn sealing_redemption_terminates_resv_chain() {
    let (world, receipt) = sealing_world();

    let next = RedeemReceipt {
        receipt,
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    let state = next.state().unwrap().1;

    assert!(state.is_sealed().unwrap());
    assert_eq!(next.roots.resv, None);

    let certificate = next.history.transitions.last().unwrap();

    assert!(matches!(certificate.resv_edge, Some(RootEdge::Term { .. })));

    check_invariant(&next).unwrap();
}

#[test]
fn sealed_pool_rejects_further_redemption() {
    let (world, receipt) = sealing_world();

    let sealed = RedeemReceipt {
        receipt,
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    assert_eq!(sealed.active_resv(), Err(Guard::Sealed));
}

#[test]
fn sealed_pool_rejects_maturity_announcement() {
    let (world, receipt) = sealing_world();

    let sealed = RedeemReceipt {
        receipt,
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    assert_eq!(
        AnnounceMaturity {
            maturity_cycle: sealed.state().unwrap().1.cycle + 10,
            signers: signers(&[OPERATOR_KEY]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&sealed, next_order(&sealed)),
        Err(Guard::Sealed),
    );
}

fn receipt_value(world: &World, outpoint: OutPoint) -> Sat {
    world.utxo(outpoint).unwrap().value
}
