//! Receipt-relabel tests.
//!
//! Implements `´test:verification:receipt-relabel´`.
//!
//! The constructor-level tests prove API safety: the public
//! `RelabelReceipts` constructor cannot express a malformed output
//! mapping. The kernel-structural fault tests below additionally
//! inject malformed relabel transactions through the crate-private
//! builder and confirm the postcommit owner/value-multiset bijection
//! validator rejects them.

use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::kernel::{CanonicalFlow, TxBuilder};
use crate::*;

#[test]
fn relabel_requires_completed_maturity() {
    let world = test_fixtures::world();

    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];

    assert_eq!(
        RelabelReceipts {
            receipts: vec![input],
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::MaturityNotComplete),
    );
}

#[test]
fn relabel_preserves_owner_and_value() {
    let world = mature_world(&test_fixtures::world());

    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];

    let value = world.utxo(input).unwrap().value;

    let next = RelabelReceipts {
        receipts: vec![input],
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    let outputs = find_receipts(&next, GENESIS_OWNER, ReceiptClass::Live);

    assert!(
        outputs
            .iter()
            .any(|outpoint| next.utxo(*outpoint).unwrap().value == value),
    );

    check_invariant(&next).unwrap();
}

#[test]
fn relabel_destinations_are_unrepresentable_in_the_public_api() {
    let world = mature_world(&test_fixtures::world());

    let inputs = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked);

    if inputs.len() < 2 {
        return;
    }

    // The public transition constructor does not expose
    // custom relabel destinations, so an owner/value
    // redistribution attempt is unrepresentable.
    let next = RelabelReceipts {
        receipts: inputs,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    check_invariant(&next).unwrap();
}

// Kernel-structural relabel fault injection. A mature world holds two
// time-locked receipts with distinct owners and values; malformed
// relabel transactions are built directly through the crate-private
// builder so the postcommit multiset-bijection validator is actually
// exercised.

fn split_time_locked_world(first: OwnerKey, second: OwnerKey) -> (World, Vec<(OutPoint, Sat)>) {
    let world = test_fixtures::world();

    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];

    let value = world.utxo(input).unwrap().value;

    assert!(value.get() >= 4, "fixture receipt too small to split");

    let first_value = Sat::new(value.get() / 3).unwrap();

    let second_value = value.checked_sub(first_value).unwrap();

    let transferred = TransferReceipts {
        class: ReceiptClass::TimeLocked,
        inputs: vec![input],
        outputs: vec![
            ReceiptDestination {
                owner: first,
                value: first_value,
            },
            ReceiptDestination {
                owner: second,
                value: second_value,
            },
        ],
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    let mature = mature_world(&transferred);

    let mut receipts = Vec::new();

    for owner in [first, second] {
        for outpoint in find_receipts(&mature, owner, ReceiptClass::TimeLocked) {
            if !receipts.iter().any(|(existing, _)| *existing == outpoint) {
                receipts.push((outpoint, mature.utxo(outpoint).unwrap().value));
            }
        }
    }

    (mature, receipts)
}

/// Builds a relabel transaction with explicit live-receipt outputs and
/// one combined lateral flow, bypassing the public constructor.
fn build_relabel_with_outputs(
    world: &World,
    inputs: &[(OutPoint, Sat)],
    outputs: &[(OwnerKey, Sat)],
) -> Result<(), Guard> {
    let (state_outpoint, state) = world.state()?;

    let mut tx = TxBuilder::new(world, BranchKind::ReceiptRelabel, next_order(world));

    tx.consume(state_outpoint)?;

    tx.emit(Asset::Pid, Sat::ONE, Meta::State(state));

    let mut sources = Vec::new();

    for (outpoint, _) in inputs {
        tx.consume(*outpoint)?;

        sources.push(*outpoint);
    }

    let mut destinations = Vec::new();

    for (owner, value) in outputs {
        destinations.push(tx.emit(
            Asset::U,
            *value,
            Meta::Receipt {
                owner: *owner,
                class: ReceiptClass::Live,
            },
        ));
    }

    // The aggregate flow conserves value, so kernel conservation
    // passes; only the branch postcondition can reject redistribution.
    tx.declare_flow(CanonicalFlow {
        asset: Asset::U,
        source_inputs: sources,
        destination_outputs: destinations,
        destructions: Vec::new(),
        movement_kind: Some(DeltaKind::Lateral),
    })?;

    tx.finish().map(|_| ())
}

#[test]
fn relabel_owner_value_redistribution_is_rejected() {
    let (world, receipts) = split_time_locked_world(ALICE, BOB);

    assert!(receipts.len() >= 2);

    let (_, alice_value) = receipts[0];
    let (_, bob_value) = receipts[1];

    // Move one unit of value from Alice's receipt to Bob's. The
    // aggregate flow still conserves total U.
    let outputs = [
        (ALICE, alice_value.checked_sub(Sat::ONE).unwrap()),
        (BOB, bob_value.checked_add(Sat::ONE).unwrap()),
    ];

    assert_eq!(
        build_relabel_with_outputs(&world, &receipts, &outputs),
        Err(Guard::RecipientPin),
    );
}

#[test]
fn relabel_owner_swap_is_rejected() {
    let (world, receipts) = split_time_locked_world(ALICE, BOB);

    let (_, alice_value) = receipts[0];
    let (_, bob_value) = receipts[1];

    // Values preserved but owners crossed: the owner/value multiset
    // differs (values are distinct by fixture construction).
    assert_ne!(alice_value, bob_value);

    let outputs = [(ALICE, bob_value), (BOB, alice_value)];

    assert_eq!(
        build_relabel_with_outputs(&world, &receipts, &outputs),
        Err(Guard::RecipientPin),
    );
}

#[test]
fn relabel_same_owner_wrong_per_input_values_is_rejected() {
    let (world, receipts) = split_time_locked_world(ALICE, ALICE);

    let total = receipts
        .iter()
        .map(|(_, value)| *value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();

    // Same owner, same total, wrong per-receipt values.
    let outputs = [
        (ALICE, Sat::ONE),
        (ALICE, total.checked_sub(Sat::ONE).unwrap()),
    ];

    assert_ne!(receipts[0].1, Sat::ONE);

    assert_eq!(
        build_relabel_with_outputs(&world, &receipts, &outputs),
        Err(Guard::RecipientPin),
    );
}

#[test]
fn relabel_merge_into_one_output_is_rejected() {
    let (world, receipts) = split_time_locked_world(ALICE, BOB);

    let total = receipts
        .iter()
        .map(|(_, value)| *value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();

    // Two inputs mapped to one output: rejected by the required
    // input/output receipt-count equality.
    let outputs = [(ALICE, total)];

    assert_eq!(
        build_relabel_with_outputs(&world, &receipts, &outputs),
        Err(Guard::WrongShape),
    );
}

#[test]
fn relabel_extra_live_output_is_rejected() {
    let (world, receipts) = split_time_locked_world(ALICE, BOB);

    let (_, alice_value) = receipts[0];
    let (_, bob_value) = receipts[1];

    // Three outputs from two inputs.
    let outputs = [
        (ALICE, alice_value.checked_sub(Sat::ONE).unwrap()),
        (ALICE, Sat::ONE),
        (BOB, bob_value),
    ];

    assert_eq!(
        build_relabel_with_outputs(&world, &receipts, &outputs),
        Err(Guard::WrongShape),
    );
}

#[test]
fn relabel_correct_multiset_is_accepted_through_the_builder() {
    // Sanity witness that the fault harness itself is well-formed:
    // the exact owner/value multiset commits successfully.
    let (world, receipts) = split_time_locked_world(ALICE, BOB);

    let outputs = [(ALICE, receipts[0].1), (BOB, receipts[1].1)];

    build_relabel_with_outputs(&world, &receipts, &outputs).unwrap();
}
