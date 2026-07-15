//! Property tests for exact canonical-flow witness faults (kernel
//! structural).
//!
//! Implements `´test:verification:property-canonical-flow-faults´` and
//! `´test:verification:kernel-structural-validity´`.
//!
//! Each fault kind builds its malformed transaction in a dedicated
//! helper rather than continuing after a declaration error, so the
//! builder state at `finish()` is unambiguous. Every fault path ends in
//! rejection, and the minimal failing fault shrinks cleanly.

use super::test_fixtures::*;
use crate::kernel::{CanonicalFlow, CommitResult, OutputRef, TxBuilder, movement_flow};
use crate::*;

use proptest::prelude::*;

fn transfer_builder(world: &World) -> (TxBuilder<'_>, OutPoint, OutputRef) {
    let receipt = find_live_receipt(world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(world, BranchKind::TransferLive, order(1));

    tx.consume(receipt).unwrap();

    let output = tx.emit(
        Asset::U,
        value,
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    (tx, receipt, output)
}

/// Missing source witness: the consumed receipt is cited by no flow.
fn missing_source_witness(world: &World) -> Result<CommitResult, Guard> {
    let (tx, _receipt, _output) = transfer_builder(world);

    tx.finish()
}

/// The same source is cited by two flows. Declaration-time rejection is
/// itself the property; the unfinished builder must then fail to
/// commit.
fn duplicate_source_witness(world: &World) -> Result<CommitResult, Guard> {
    let (mut tx, receipt, output) = transfer_builder(world);

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![output],
    ))
    .unwrap();

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        Vec::new(),
    ))?;

    tx.finish()
}

/// The same destination is cited twice inside one flow.
fn duplicate_destination_witness(world: &World) -> Result<CommitResult, Guard> {
    let (mut tx, receipt, output) = transfer_builder(world);

    tx.declare_flow(CanonicalFlow {
        asset: Asset::U,
        source_inputs: vec![receipt],
        destination_outputs: vec![output, output],
        destructions: Vec::new(),
        movement_kind: Some(DeltaKind::Lateral),
    })?;

    tx.finish()
}

// ´test:verification:property-canonical-flow-faults´

proptest! {
    #![proptest_config(
        ProptestConfig {
            cases: 250,
            max_shrink_iters: 50_000,
            .. ProptestConfig::default()
        }
    )]

    #[test]
    fn duplicate_or_missing_flow_witnesses_are_rejected(
        fault_kind in 0_u8..3_u8,
    ) {
        let world = world();

        let result = match fault_kind {
            0 => missing_source_witness(&world),
            1 => duplicate_source_witness(&world),
            _ => duplicate_destination_witness(&world),
        };

        prop_assert!(result.is_err());
    }
}
