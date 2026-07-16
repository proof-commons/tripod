//! Kernel structural tests.
//!
//! Implements `´test:verification:kernel´` and
//! `´test:verification:kernel-structural-validity´`.
//!
//! These tests exercise the crate-private transaction builder
//! directly. They prove structural transaction validity — not
//! authorized protocol transitions: owner/operator authorization is
//! validated by the operation constructors, which are tested
//! separately.

use super::test_fixtures::*;
use crate::kernel::{IssuanceDeclaration, OpenFlow, TxBuilder, movement_flow};
use crate::*;

fn one_output_transfer(
    world: &World,
    receipt: OutPoint,
    owner: OwnerKey,
    order: CanonicalOrder,
) -> TxBuilder<'_> {
    let receipt_utxo = *world.utxo(receipt).unwrap();

    let mut tx = TxBuilder::new(world, BranchKind::TransferLive, order);

    tx.consume(receipt).unwrap();

    let output = tx.emit(
        Asset::U,
        receipt_utxo.value,
        Meta::Receipt {
            owner,
            class: ReceiptClass::Live,
        },
    );

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![output],
    ))
    .unwrap();

    tx
}

#[test]
fn genesis_satisfies_invariant() {
    let world = world();

    check_invariant(&world).unwrap();

    assert_eq!(world.roots.resv, Some(world.history.genesis.resv_out));
}

#[test]
fn pure_transition_error_leaves_world_unchanged() {
    let world = world();

    let before = world.clone();

    let transfer = TransferReceipts {
        class: ReceiptClass::Live,

        inputs: Vec::new(),

        outputs: Vec::new(),

        signers: SignerSet::new(),

        fee_envelope: FeeEnvelope::default(),
    };

    assert_eq!(transfer.apply(&world, order(1)), Err(Guard::Domain));

    assert_eq!(world, before);
}

#[test]
fn valid_exact_flow_commits() {
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let tx = one_output_transfer(&world, receipt, ALICE, order(1));

    let result = tx.finish().unwrap();

    check_invariant(&result.world).unwrap();
}

#[test]
fn duplicate_source_witness_is_rejected() {
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::TransferLive, order(1));

    tx.consume(receipt).unwrap();

    let output_a = tx.emit(
        Asset::U,
        value,
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    let output_b = tx.emit(
        Asset::U,
        value,
        Meta::Receipt {
            owner: BOB,
            class: ReceiptClass::Live,
        },
    );

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![output_a],
    ))
    .unwrap();

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![output_b],
    ))
    .unwrap();

    assert_eq!(tx.finish(), Err(Guard::CanonicalDeltaMismatch));
}

#[test]
fn unwitnessed_canonical_output_is_rejected() {
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::TransferLive, order(1));

    tx.consume(receipt).unwrap();

    let witnessed = tx.emit(
        Asset::U,
        value,
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    tx.emit(
        Asset::U,
        Sat::ONE,
        Meta::Receipt {
            owner: BOB,
            class: ReceiptClass::Live,
        },
    );

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![witnessed],
    ))
    .unwrap();

    assert_eq!(tx.finish(), Err(Guard::CanonicalDeltaMismatch));
}

#[test]
fn issuance_without_authority_is_rejected() {
    let world = world();

    let mut tx = TxBuilder::new(&world, BranchKind::TransferLive, order(1));

    let output = tx.emit(
        Asset::U,
        sat(10),
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    assert_eq!(
        tx.declare_issuance(IssuanceDeclaration {
            asset: Asset::U,
            authority_asset: Asset::Pace,
            authority_input: world.roots.pace,
            amount: sat(10),
            destination_outputs: vec![output],
        }),
        Err(Guard::MissingAuthority),
    );
}

#[test]
fn issuance_must_exhaust_destination_outputs() {
    let world = world();

    let mut tx = TxBuilder::new(&world, BranchKind::Cycle, order(1));

    tx.consume(world.roots.pace).unwrap();

    let output = tx.emit(
        Asset::U,
        sat(9),
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    assert_eq!(
        tx.declare_issuance(IssuanceDeclaration {
            asset: Asset::U,
            authority_asset: Asset::Pace,
            authority_input: world.roots.pace,
            amount: sat(10),
            destination_outputs: vec![output],
        }),
        Err(Guard::BadIssuance),
    );
}

#[test]
fn output_cannot_be_funded_twice() {
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::TransferLive, order(1));

    tx.consume(receipt).unwrap();

    tx.consume(world.roots.pace).unwrap();

    let output = tx.emit(
        Asset::U,
        value,
        Meta::Receipt {
            owner: ALICE,
            class: ReceiptClass::Live,
        },
    );

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![output],
    ))
    .unwrap();

    tx.declare_issuance(IssuanceDeclaration {
        asset: Asset::U,
        authority_asset: Asset::Pace,
        authority_input: world.roots.pace,
        amount: value,
        destination_outputs: vec![output],
    })
    .unwrap();

    assert_eq!(tx.finish(), Err(Guard::CanonicalDeltaMismatch));
}

#[test]
fn burn_projection_is_derived_from_data_outputs() {
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::Burn, order(1));

    tx.consume(receipt).unwrap();

    let ash = tx.emit(Asset::U, value, Meta::Ash);

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![ash],
    ))
    .unwrap();

    tx.emit_data(DataOutput::BurnRecord {
        record_index: 0,
        address: ADDRESS_A,
        amount: value,
    });

    let result = tx.finish().unwrap();

    let burn = result
        .world
        .history
        .transitions
        .last()
        .unwrap()
        .burn
        .as_ref()
        .unwrap();

    assert_eq!(
        burn.records,
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: value,
        }],
    );
}

#[test]
fn low_level_duplicate_burn_payload_indices_are_rejected() {
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::Burn, order(1));

    tx.consume(receipt).unwrap();

    let ash = tx.emit(Asset::U, value, Meta::Ash);

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![ash],
    ))
    .unwrap();

    for amount in [sat(40), sat(60)] {
        tx.emit_data(DataOutput::BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount,
        });
    }

    assert_eq!(tx.finish(), Err(Guard::WrongShape));
}

// (´rule:verification:declared-cardinality´) — manifest-driven static
// minima and maxima at the kernel shape gate.

#[test]
fn below_minimum_receipt_inputs_are_rejected_by_the_manifest_minimum() {
    // A transfer with no receipt inputs or outputs violates the
    // manifest's declared minimum of one receipt in and one out.
    let world = world();

    let tx = TxBuilder::new(&world, BranchKind::TransferLive, order(1));

    assert_eq!(tx.finish(), Err(Guard::WrongShape));
}

#[test]
fn omitting_a_manifest_required_output_is_rejected() {
    // Request creation must emit exactly one deposit request; the
    // manifest's output minimum rejects its omission even when the
    // open-flow accounting balances.
    let mut world = world();

    world.adversary.lbtc = sat(1_000);

    let world = inject_open_object(
        &world,
        Asset::Lbtc,
        sat(1_000),
        Meta::PlainLbtc { owner: ALICE },
    )
    .unwrap();

    let source = world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (Asset::Lbtc, Meta::PlainLbtc { owner }) if owner == ALICE => Some(*outpoint),
            _ => None,
        })
        .unwrap();

    let mut tx = TxBuilder::new(&world, BranchKind::CreateRequest, order(1));

    tx.consume(source).unwrap();

    let change = tx.emit(Asset::Lbtc, sat(1_000), Meta::PlainLbtc { owner: ALICE });

    tx.declare_open_flow(OpenFlow {
        kind: OpenFlowKind::RequestCreation,
        source_inputs: vec![source],
        destination_outputs: vec![change],
        fee: Sat::ZERO,
    })
    .unwrap();

    assert_eq!(tx.finish(), Err(Guard::WrongShape));
}

#[test]
fn exceeding_a_manifest_output_maximum_is_rejected() {
    // Burn emits exactly one ASH output; a second violates the
    // manifest's exact maximum.
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let mut tx = TxBuilder::new(&world, BranchKind::Burn, order(1));

    tx.consume(receipt).unwrap();

    let first = tx.emit(Asset::U, sat(1), Meta::Ash);
    let second = tx.emit(Asset::U, value.checked_sub(sat(1)).unwrap(), Meta::Ash);

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![first, second],
    ))
    .unwrap();

    assert_eq!(tx.finish(), Err(Guard::Domain));
}
