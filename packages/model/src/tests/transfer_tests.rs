//! Transfer tests.
//!
//! Implements `´test:verification:transfer´`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn split_and_merge_preserve_class_value() {
    let world = test_fixtures::world();

    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    let value = world.utxo(input).unwrap().value;

    let first = sat(value.get() / 2);

    let second = value.checked_sub(first).unwrap();

    apply_checked(
        &world,
        &TransferReceipts {
            class: ReceiptClass::Live,

            inputs: vec![input],

            outputs: vec![
                ReceiptDestination {
                    owner: ALICE,
                    value: first,
                },
                ReceiptDestination {
                    owner: BOB,
                    value: second,
                },
            ],

            signers: signers(&[GENESIS_OWNER]),

            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );
}

#[test]
fn mixed_owner_transfer_requires_every_signer() {
    let world = test_fixtures::world();

    let genesis_input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    let split = apply_checked(
        &world,
        &TransferReceipts {
            class: ReceiptClass::Live,

            inputs: vec![genesis_input],

            outputs: vec![
                ReceiptDestination {
                    owner: ALICE,
                    value: sat(5),
                },
                ReceiptDestination {
                    owner: BOB,
                    value: sat(7),
                },
                ReceiptDestination {
                    owner: GENESIS_OWNER,
                    value: world
                        .utxo(genesis_input)
                        .unwrap()
                        .value
                        .checked_sub(sat(12))
                        .unwrap(),
                },
            ],

            signers: signers(&[GENESIS_OWNER]),

            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let alice = find_receipts(&split, ALICE, ReceiptClass::Live)[0];

    let bob = find_receipts(&split, BOB, ReceiptClass::Live)[0];

    assert_eq!(
        TransferReceipts {
            class: ReceiptClass::Live,

            inputs: vec![alice, bob],

            outputs: vec![ReceiptDestination {
                owner: CAROL,
                value: sat(12),
            }],

            signers: signers(&[ALICE]),

            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&split, next_order(&split)),
        Err(Guard::BadSignature),
    );

    apply_checked(
        &split,
        &TransferReceipts {
            class: ReceiptClass::Live,

            inputs: vec![alice, bob],

            outputs: vec![ReceiptDestination {
                owner: CAROL,
                value: sat(12),
            }],

            signers: signers(&[ALICE, BOB]),

            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&split),
    );
}

#[test]
fn time_locked_input_cannot_use_live_transfer_leaf() {
    let world = test_fixtures::world();

    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];

    assert_eq!(
        TransferReceipts {
            class: ReceiptClass::Live,

            inputs: vec![input],

            outputs: vec![ReceiptDestination {
                owner: ALICE,
                value: world.utxo(input).unwrap().value,
            }],

            signers: signers(&[GENESIS_OWNER]),

            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::ClassCross),
    );
}
