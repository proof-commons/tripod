//! Kernel structural tests for open-value flows.
//!
//! Implements `´test:verification:open-flow-partition´` and
//! `(´test:verification:kernel-structural-validity´)`.

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

// Sponsor-value opacity (F2-006): open-flow membership is
// family-based, never value-based. A zero-valued ordinary L-BTC
// output is permitted but pointless — value zero is only meaningful
// for CPFP anchoring (or another colored-output scheme outside this
// protocol) — so the first-party builder omits known zero change and
// the kernel neither requires nor forbids the shape.

fn burn_with_sponsor(world: &World, sponsor: OutPoint) -> TxBuilder<'_> {
    let receipt = find_live_receipt(world, GENESIS_OWNER);
    let receipt_value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(world, BranchKind::Burn, order(1));
    tx.consume(receipt).unwrap();
    let ash = tx.emit(Asset::U, receipt_value, Meta::Ash);
    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![ash],
    ))
    .unwrap();
    tx.consume(sponsor).unwrap();

    tx
}

#[test]
fn builder_omits_a_zero_valued_sponsor_change() {
    let mut world = external_lbtc_world(1_000);
    let (next, sponsor) = inject_and_get_outpoint(
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

    // A requested zero change is normalized away, so the declared fee
    // must balance the whole envelope.
    tx.apply_fee_envelope(&FeeEnvelope {
        inputs: vec![sponsor],
        signers: std::iter::once(ALICE).collect(),
        chain_fee: sat(10),
        change: Some(FeeChange {
            owner: ALICE,
            value: Sat::ZERO,
        }),
    })
    .unwrap();

    let result = tx.finish().unwrap();
    assert!(
        !result
            .world
            .utxos
            .values()
            .any(|utxo| utxo.asset == Asset::Lbtc && utxo.value.is_zero()),
        "the builder must not emit a zero-valued ordinary L-BTC output"
    );
}

#[test]
fn claimed_zero_valued_sponsor_change_is_accepted() {
    // Low-level construction (not the first-party builder) may carry
    // an explicit zero change output; family-based membership claims
    // it and the envelope balances, so the kernel accepts.
    let mut world = external_lbtc_world(1_000);
    let (next, sponsor) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(10),
        Meta::PlainLbtc { owner: ALICE },
    );
    world = next;

    let mut tx = burn_with_sponsor(&world, sponsor);
    let change = tx.emit(Asset::Lbtc, Sat::ZERO, Meta::PlainLbtc { owner: ALICE });
    tx.declare_open_flow(OpenFlow {
        kind: OpenFlowKind::FeeSponsor,
        source_inputs: vec![sponsor],
        destination_outputs: vec![change],
        fee: sat(10),
    })
    .unwrap();
    tx.set_chain_fee(sat(10));

    tx.finish().unwrap();
}

#[test]
fn unclaimed_zero_valued_plain_lbtc_output_is_an_open_flow_fault() {
    // Opacity is not omission: a zero-valued ordinary output outside
    // every flow fails exact membership — as an open-flow fault, not
    // a domain rejection of the value.
    let mut world = external_lbtc_world(1_000);
    let (next, sponsor) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(10),
        Meta::PlainLbtc { owner: ALICE },
    );
    world = next;

    let mut tx = burn_with_sponsor(&world, sponsor);
    let _unclaimed = tx.emit(Asset::Lbtc, Sat::ZERO, Meta::PlainLbtc { owner: ALICE });
    tx.declare_open_flow(OpenFlow {
        kind: OpenFlowKind::FeeSponsor,
        source_inputs: vec![sponsor],
        destination_outputs: vec![],
        fee: sat(10),
    })
    .unwrap();
    tx.set_chain_fee(sat(10));

    assert_eq!(tx.finish(), Err(Guard::OpenFlowMismatch));
}

#[test]
fn an_anchor_cannot_be_claimed_by_an_open_flow() {
    // Anchor identity is structural: the anchor stands outside the
    // partition by family and can neither satisfy nor join a sponsor
    // role.
    let mut world = external_lbtc_world(1_000);
    let (next, sponsor) = inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(10),
        Meta::PlainLbtc { owner: ALICE },
    );
    world = next;

    let mut tx = burn_with_sponsor(&world, sponsor);
    let anchor = tx.emit(Asset::Lbtc, Sat::ZERO, Meta::CpfpAnchor);
    tx.declare_open_flow(OpenFlow {
        kind: OpenFlowKind::FeeSponsor,
        source_inputs: vec![sponsor],
        destination_outputs: vec![anchor],
        fee: sat(10),
    })
    .unwrap();
    tx.set_chain_fee(sat(10));

    assert_eq!(tx.finish(), Err(Guard::OpenFlowMismatch));
}
