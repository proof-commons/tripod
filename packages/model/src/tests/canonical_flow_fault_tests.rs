//! Exact canonical-flow fault tests (kernel structural).
//!
//! Implements `´test:verification:canonical-flow-faults´` and
//! `´test:verification:kernel-structural-validity´`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::kernel::{CanonicalFlow, TxBuilder, destruction_flow};
use crate::*;

#[test]
fn omitted_canonical_source_is_rejected() {
    let world = test_fixtures::world();

    let receipt = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    let receipt_value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::TransferLive, next_order(&world));

    tx.consume(receipt).unwrap();

    tx.emit(
        Asset::U,
        receipt_value,
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    assert_eq!(tx.finish(), Err(Guard::CanonicalDeltaMismatch));
}

#[test]
fn duplicated_destination_is_rejected() {
    let world = test_fixtures::world();

    let receipt = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::TransferLive, next_order(&world));

    tx.consume(receipt).unwrap();

    let output = tx.emit(
        Asset::U,
        value,
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    // The duplicate destination is rejected at declaration time; the
    // undeclared flow then leaves the output unwitnessed at commit.
    let declared = tx.declare_flow(CanonicalFlow {
        asset: Asset::U,
        source_inputs: vec![receipt],
        destination_outputs: vec![output, output],
        destructions: Vec::new(),
        movement_kind: Some(DeltaKind::Lateral),
    });

    assert!(matches!(
        declared,
        Err(Guard::DuplicateOutputIndex | Guard::CanonicalDeltaMismatch)
    ));

    assert!(matches!(
        tx.finish(),
        Err(Guard::DuplicateOutputIndex | Guard::CanonicalDeltaMismatch)
    ));
}

#[test]
fn wrong_destruction_tag_for_branch_is_rejected() {
    let world = test_fixtures::world();

    let receipt = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::Burn, next_order(&world));

    tx.consume(receipt).unwrap();

    tx.declare_flow(destruction_flow(Asset::U, vec![receipt], Tag::Recon, value))
        .unwrap();

    assert_eq!(tx.finish(), Err(Guard::BadAuthorization));
}
