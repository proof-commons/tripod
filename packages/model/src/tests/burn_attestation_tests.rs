//! Burn and attestation tests.
//!
//! Implements `´test:verification:burn-attestation´`.
//!
//! The bare-tag test constructs a proof-only history object directly
//! and is therefore a low-level indexer test, not a valid-transition
//! test.

use std::collections::BTreeSet;

use num_bigint::BigUint;

use super::advanced_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use super::test_fixtures::{chain_view_for_history, txid};
use crate::kernel::{TxBuilder, movement_flow};
use crate::*;

/// The event-type anchor's negative case at the transition level: a
/// transaction presenting an ASH input cannot be certified as a burn.
/// (The indexer-side witness is
/// `compacted_ash_never_creates_another_burn_event`; this rejects the
/// transition itself.)
#[test]
fn burn_transaction_with_ash_input_is_rejected() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let burned = burn_from_owner(&world, ALICE, sat(100), Vec::new());

    let ash = find_ash(&burned)[0];

    let ash_value = burned.utxo(ash).unwrap().value;

    let mut tx = TxBuilder::new(&burned, BranchKind::Burn, next_order(&burned));

    tx.consume(ash).unwrap();

    let output = tx.emit(Asset::U, ash_value, Meta::Ash);

    tx.declare_flow(movement_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![ash],
        vec![output],
    ))
    .unwrap();

    let result = tx.finish();

    assert!(
        matches!(result, Err(Guard::WrongShape | Guard::WrongClass)),
        "burn with ASH input must be rejected, got {result:?}",
    );
}

#[test]
fn genuine_burn_creates_one_burn_projection() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let next = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100),
        }],
    );

    let certificate = next.history.transitions.last().unwrap();

    let burn = certificate.burn.as_ref().expect("burn projection");

    assert_eq!(burn.ash_value, sat(100));
    assert_eq!(burn.records.len(), 1);

    check_invariant(&next).unwrap();
}

#[test]
fn overclaiming_records_do_not_invalidate_burn() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let next = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![
            BurnRecord {
                record_index: 0,
                address: ADDRESS_A,
                amount: sat(70),
            },
            BurnRecord {
                record_index: 1,
                address: ADDRESS_B,
                amount: sat(50),
            },
        ],
    );

    let chain = chain_view_for_history(&next);

    let indexer =
        ReferenceIndexer::from_assumed_kernel_history(&next.history, &chain, [0_u8; 32]).unwrap();

    let burn = indexer.burns().values().next().expect("indexed burn");

    assert!(!burn.records_accepted().unwrap());

    assert!(indexer.group_all_credits_for_audit().unwrap().is_empty());

    check_invariant(&next).unwrap();
}

#[test]
fn underclaiming_records_is_a_valid_donation() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let next = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(80),
        }],
    );

    let indexer = ReferenceIndexer::from_assumed_kernel_history(
        &next.history,
        &chain_view_for_history(&next),
        [0_u8; 32],
    )
    .unwrap();

    let burn = indexer.burns().values().next().unwrap();

    assert!(burn.records_accepted().unwrap());

    let grouped = indexer.group_all_credits_for_audit().unwrap();

    assert_eq!(
        grouped.values().next().cloned().unwrap(),
        BigUint::from(80_u32),
    );
}

#[test]
fn compacted_ash_never_creates_another_burn_event() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let first = burn_from_owner(
        &world,
        ALICE,
        sat(50),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(50),
        }],
    );

    let second_world = give_live_receipt(&first, BOB, sat(50));

    let second = burn_from_owner(
        &second_world,
        BOB,
        sat(50),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_B,
            amount: sat(50),
        }],
    );

    let ash = find_ash(&second);

    assert_eq!(ash.len(), 2);

    let compacted = apply_checked(
        &second,
        &CompactAsh {
            ash_inputs: ash,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&second),
    );

    let burn_count = compacted
        .history
        .transitions
        .iter()
        .filter(|certificate| certificate.burn.is_some())
        .count();

    assert_eq!(burn_count, 2);

    assert!(compacted.history.transitions.last().unwrap().burn.is_none(),);
}

#[test]
fn duplicate_burn_record_indices_are_rejected() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let receipt = find_receipts(&world, ALICE, ReceiptClass::Live)[0];

    let result = BurnReceipts {
        receipts: vec![receipt],
        signers: signers(&[ALICE]),
        ash_value: sat(100),
        change: Vec::new(),
        records: vec![
            BurnRecord {
                record_index: 0,
                address: ADDRESS_A,
                amount: sat(40),
            },
            BurnRecord {
                record_index: 0,
                address: ADDRESS_B,
                amount: sat(60),
            },
        ],
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world));

    assert_eq!(result, Err(Guard::WrongShape));
}

#[test]
fn noncanonical_burn_record_index_is_rejected() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let receipt = find_receipts(&world, ALICE, ReceiptClass::Live)[0];

    let result = BurnReceipts {
        receipts: vec![receipt],
        signers: signers(&[ALICE]),
        ash_value: sat(100),
        change: Vec::new(),
        records: vec![BurnRecord {
            record_index: 1,
            address: ADDRESS_A,
            amount: sat(100),
        }],
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world));

    assert_eq!(result, Err(Guard::WrongShape));
}

#[test]
fn bare_burn_tag_without_burn_projection_attests_nothing() {
    let mut world = test_fixtures::world();

    // Open data can exist in the global chain environment,
    // but the reference indexer reads only derived BurnProjection values.
    world.history.transitions.push(TransitionCertificate {
        txid: txid(99),
        order: next_order(&world),
        branch: BranchKind::CompactAsh,

        consumed: BTreeSet::new(),
        created: BTreeSet::new(),

        state_edge: None,
        resv_edge: None,
        pace_edge: None,
        entitlement_authority_edge: None,
        distribution_authority_edge: None,

        canonical_partition: CertifiedCanonicalPartition::default(),
        open_flows: Vec::new(),

        chain_fee: Sat::ZERO,

        burn: None,
        clear: None,
        distribution_residue: None,
    });

    let indexer = ReferenceIndexer::from_assumed_kernel_history(
        &world.history,
        &chain_view_for_history(&world),
        [0_u8; 32],
    )
    .unwrap();

    assert!(indexer.burns().is_empty());
}

#[test]
fn indexer_excludes_events_after_checkpoint() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let world = give_live_receipt(&world, BOB, sat(50));

    // Burn A at the genesis height.
    let after_first = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100),
        }],
    );

    // Burn B in the next block, after the checkpoint.
    let receipt = find_receipts(&after_first, BOB, ReceiptClass::Live)[0];

    let full = apply_checked(
        &after_first,
        &BurnReceipts {
            receipts: vec![receipt],
            signers: signers(&[BOB]),
            ash_value: sat(50),
            change: Vec::new(),
            records: vec![BurnRecord {
                record_index: 0,
                address: ADDRESS_B,
                amount: sat(50),
            }],
            fee_envelope: FeeEnvelope::default(),
        },
        next_block_order(&after_first),
    );

    let genesis_height = full.history.genesis.order.height;

    let checkpoint_chain = ValidatedChainView::new(
        test_fixtures::TEST_NETWORK_ID,
        test_fixtures::TEST_GENESIS_ID,
        test_fixtures::test_manifest_hash(),
        genesis_height,
        test_fixtures::block_hash(u8::try_from(genesis_height % 256).unwrap()),
        ATTESTATION_SCHEMA_VERSION,
        vec![CanonicalBlock {
            height: genesis_height,
            hash: test_fixtures::block_hash(u8::try_from(genesis_height % 256).unwrap()),
            parent_hash: None,
        }],
    )
    .unwrap();

    let prefix_indexer =
        ReferenceIndexer::from_assumed_kernel_history(&full.history, &checkpoint_chain, [0_u8; 32])
            .unwrap();

    assert_eq!(prefix_indexer.burns().len(), 1);

    let indexed_burn = prefix_indexer.burns().values().next().unwrap();

    assert_eq!(indexed_burn.records[0].address, ADDRESS_A);

    let full_indexer = ReferenceIndexer::from_assumed_kernel_history(
        &full.history,
        &chain_view_for_history(&full),
        [0_u8; 32],
    )
    .unwrap();

    assert_eq!(full_indexer.burns().len(), 2);
}

#[test]
fn duplicate_transition_txid_is_rejected_by_indexer() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let next = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100),
        }],
    );

    let mut forged = next;

    // Reuse the txid of an existing transition under a later order.
    let mut duplicate = forged.history.transitions.last().unwrap().clone();

    duplicate.order.tx_index += 1;

    forged.history.transitions.push(duplicate);

    assert_eq!(
        ReferenceIndexer::from_assumed_kernel_history(
            &forged.history,
            &chain_view_for_history(&forged),
            [0_u8; 32],
        ),
        Err(Guard::DuplicateEvent),
    );
}

#[test]
fn invalid_semantic_context_is_rejected_at_chain_view_construction() {
    let block = CanonicalBlock {
        height: 1,
        hash: test_fixtures::block_hash(1),
        parent_hash: None,
    };

    let build = |network_id, genesis_id, manifest_hash| {
        ValidatedChainView::new(
            network_id,
            genesis_id,
            manifest_hash,
            1,
            test_fixtures::block_hash(1),
            ATTESTATION_SCHEMA_VERSION,
            vec![block],
        )
    };

    // A wrong architecture hash, a zero network identity, and a zero
    // genesis identity each fail at construction, not at some later
    // serialization boundary.
    assert_eq!(
        build(
            test_fixtures::TEST_NETWORK_ID,
            test_fixtures::TEST_GENESIS_ID,
            [0xAB_u8; 32],
        ),
        Err(Guard::BadConstant),
    );

    assert_eq!(
        build(
            [0_u8; 32],
            test_fixtures::TEST_GENESIS_ID,
            test_fixtures::test_manifest_hash(),
        ),
        Err(Guard::Domain),
    );

    assert_eq!(
        build(
            test_fixtures::TEST_NETWORK_ID,
            [0_u8; 32],
            test_fixtures::test_manifest_hash(),
        ),
        Err(Guard::Domain),
    );

    build(
        test_fixtures::TEST_NETWORK_ID,
        test_fixtures::TEST_GENESIS_ID,
        test_fixtures::test_manifest_hash(),
    )
    .unwrap();
}

#[test]
fn indexer_query_result_always_passes_semantic_validation() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let next = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100),
        }],
    );

    let indexer = ReferenceIndexer::from_assumed_kernel_history(
        &next.history,
        &chain_view_for_history(&next),
        [0_u8; 32],
    )
    .unwrap();

    let query = indexer.query(ADDRESS_A).unwrap();

    validate_query(&query).unwrap();
}

#[test]
fn transition_reusing_genesis_txid_is_rejected_by_indexer() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let next = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100),
        }],
    );

    let mut forged = next;

    forged.history.transitions.last_mut().unwrap().txid = forged.history.genesis.txid;

    assert_eq!(
        ReferenceIndexer::from_assumed_kernel_history(
            &forged.history,
            &chain_view_for_history(&forged),
            [0_u8; 32],
        ),
        Err(Guard::DuplicateEvent),
    );
}
