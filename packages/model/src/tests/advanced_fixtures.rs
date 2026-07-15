//! Additional scenario helpers for redemption, burn, and provenance
//! suites.
//!
//! Implements `´test:verification:advanced-fixtures´`.

use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

pub fn give_live_receipt(world: &World, owner: OwnerKey, value: Sat) -> World {
    let genesis_receipt = find_receipts(world, GENESIS_OWNER, ReceiptClass::Live)
        .into_iter()
        .next()
        .expect("genesis live receipt");

    let input_value = world.utxo(genesis_receipt).expect("receipt").value;

    assert!(value < input_value);

    let change = input_value.checked_sub(value).expect("change");

    TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![genesis_receipt],

        outputs: vec![
            ReceiptDestination { owner, value },
            ReceiptDestination {
                owner: GENESIS_OWNER,
                value: change,
            },
        ],

        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_order(world))
    .expect("give live receipt")
}

pub fn burn_from_owner(
    world: &World,
    owner: OwnerKey,
    burn_value: Sat,
    records: Vec<BurnRecord>,
) -> World {
    let receipt = find_receipts(world, owner, ReceiptClass::Live)
        .into_iter()
        .find(|outpoint| world.utxo(*outpoint).expect("receipt").value >= burn_value)
        .expect("sufficient live receipt");

    let receipt_value = world.utxo(receipt).expect("receipt").value;

    let change_value = receipt_value.checked_sub(burn_value).expect("burn change");

    let change = if change_value.is_zero() {
        Vec::new()
    } else {
        vec![ReceiptDestination {
            owner,
            value: change_value,
        }]
    };

    BurnReceipts {
        receipts: vec![receipt],
        signers: signers(&[owner]),

        ash_value: burn_value,
        change,
        records,

        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_order(world))
    .expect("burn")
}

pub fn mature_and_relabel_all(world: &World) -> World {
    let mut current = mature_world(world);

    let time_locked = find_receipts(&current, GENESIS_OWNER, ReceiptClass::TimeLocked);

    if !time_locked.is_empty() {
        current = RelabelReceipts {
            receipts: time_locked,
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&current, next_order(&current))
        .expect("relabel");
    }

    current
}

pub fn merge_live_receipts(world: &World, owner: OwnerKey) -> World {
    let receipts = find_receipts(world, owner, ReceiptClass::Live);

    if receipts.len() <= 1 {
        return world.clone();
    }

    let total = receipts
        .iter()
        .map(|outpoint| world.utxo(*outpoint).expect("receipt").value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .expect("receipt sum");

    TransferReceipts {
        class: ReceiptClass::Live,
        inputs: receipts,

        outputs: vec![ReceiptDestination {
            owner,
            value: total,
        }],

        signers: signers(&[owner]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_order(world))
    .expect("merge live receipts")
}

pub fn sealing_world() -> (World, OutPoint) {
    let mature = mature_and_relabel_all(&test_fixtures::world());

    let merged = merge_live_receipts(&mature, GENESIS_OWNER);

    let receipt = find_receipts(&merged, GENESIS_OWNER, ReceiptClass::Live)
        .into_iter()
        .next()
        .expect("sealing receipt");

    assert_eq!(
        merged.utxo(receipt).expect("receipt").value,
        merged.state().expect("state").1.y().expect("y"),
    );

    (merged, receipt)
}

pub fn chain_view_for_world(world: &World) -> ValidatedChainView {
    test_fixtures::chain_view_for_history(world)
}

pub fn world_with_terminal_residue() -> World {
    let (distributed, cycle) = world_with_distribution(&[(ALICE, ALICE, 1), (BOB, BOB, 2)]);

    let entitlements = find_all_entitlements_for_cycle(&distributed, cycle);

    settle_batch(&distributed, cycle, entitlements)
}
