//! Kernel structural tests for open-value flows.
//!
//! Implements `´test:verification:open-flow-partition´` and
//! `´test:verification:kernel-structural-validity´`.

use super::test_fixtures::*;
use crate::kernel::{OpenFlow, TxBuilder, movement_flow};
use crate::*;

#[test]
fn lbtc_input_must_belong_to_exactly_one_open_flow() {
    let mut world = external_lbtc_world(1_000);

    let (next, input) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(100),
        Meta::PlainLbtc { owner: ALICE },
    );

    world = next;

    let mut tx = TxBuilder::new(&world, BranchKind::CreateRequest, order(1));

    tx.consume(input).unwrap();

    let _request = tx.emit(
        Asset::Lbtc,
        sat(90),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id,
            refund_key: ALICE,
            receipt_owner: ALICE,
            deposit_principal: sat(80),
        },
    );

    tx.set_chain_fee(sat(10));

    // No open flow declaration.
    assert_eq!(tx.finish(), Err(Guard::OpenFlowMismatch));

    let mut tx = TxBuilder::new(&world, BranchKind::CreateRequest, order(1));

    tx.consume(input).unwrap();

    let request = tx.emit(
        Asset::Lbtc,
        sat(90),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id,
            refund_key: ALICE,
            receipt_owner: ALICE,
            deposit_principal: sat(80),
        },
    );

    tx.declare_open_flow(OpenFlow {
        kind: OpenFlowKind::RequestCreation,
        source_inputs: vec![input],
        destination_outputs: vec![request],
        fee: sat(10),
    })
    .unwrap();

    tx.set_chain_fee(sat(10));

    tx.finish().unwrap();
}

#[test]
fn lbtc_output_cannot_be_claimed_by_two_open_flows() {
    let mut world = external_lbtc_world(1_000);

    let (next, input) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(100),
        Meta::PlainLbtc { owner: ALICE },
    );

    world = next;

    let mut tx = TxBuilder::new(&world, BranchKind::CreateRequest, order(1));

    tx.consume(input).unwrap();

    let request = tx.emit(
        Asset::Lbtc,
        sat(100),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id,
            refund_key: ALICE,
            receipt_owner: ALICE,
            deposit_principal: sat(90),
        },
    );

    for _ in 0..2 {
        tx.declare_open_flow(OpenFlow {
            kind: OpenFlowKind::RequestCreation,
            source_inputs: vec![input],
            destination_outputs: vec![request],
            fee: Sat::ZERO,
        })
        .unwrap();
    }

    assert_eq!(tx.finish(), Err(Guard::OpenFlowMismatch));
}

#[test]
fn zero_value_anchor_does_not_require_open_flow() {
    let world = world();

    let mut tx = TxBuilder::new(&world, BranchKind::Cycle, order(1));

    // This incomplete cycle fails for missing roots,
    // but not because the anchor lacks an open flow.
    tx.emit(Asset::Lbtc, Sat::ZERO, Meta::CpfpAnchor);

    assert_ne!(tx.finish(), Err(Guard::OpenFlowMismatch));
}

/// The kernel enforces at-most-one generic fee-sponsor envelope per
/// transaction; a second envelope is rejected as `SponsorMismatch`,
/// not merged.
#[test]
fn two_sponsor_envelopes_in_one_transaction_are_rejected() {
    let mut world = external_lbtc_world(1_000);

    let (next, first_sponsor) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(10),
        Meta::PlainLbtc { owner: ALICE },
    );

    world = next;

    let (next, second_sponsor) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(10),
        Meta::PlainLbtc { owner: ALICE },
    );

    world = next;

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let receipt_value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::Burn, order(1));

    tx.consume(receipt).unwrap();

    let ash = tx.emit(Asset::U, receipt_value, Meta::Ash);

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![ash],
    ))
    .unwrap();

    for sponsor in [first_sponsor, second_sponsor] {
        tx.apply_fee_envelope(&FeeEnvelope {
            inputs: vec![sponsor],
            signers: std::iter::once(ALICE).collect(),
            chain_fee: sat(10),
            change: None,
        })
        .unwrap();
    }

    assert_eq!(tx.finish(), Err(Guard::SponsorMismatch));
}
