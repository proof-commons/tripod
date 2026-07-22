//! Bidirectional indexer-conformance tests.
//!
//! Implements `´test:verification:indexer-conformance´`.
//!
//! Event recognition and query computation are separate claims with
//! separate witnesses: the raw event/projection differential and the
//! canonical query-byte differential. Offsetting recognition errors in
//! the residue audit require the separate receipt-accounting
//! differential harness in [`crate::audit`]; attestation queries alone
//! cannot demonstrate them.

use num_bigint::BigUint;
use num_traits::One;

use super::advanced_fixtures::*;
use super::attestation_history_fixture::{accepted_burn, clear_entry, genesis_clear};
use super::scenario_fixtures::*;
use super::test_fixtures;
use super::test_fixtures::{
    TEST_GENESIS_ID, TEST_NETWORK_ID, block_hash, chain_view_for_history, test_manifest_hash, txid,
};
use crate::ledger::UntrustedIndexerFixture;
use crate::*;

fn indexed_burn_world() -> (World, ReferenceIndexer) {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let burned = burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100),
        }],
    );

    let chain = chain_view_for_history(&burned);

    let indexer =
        ReferenceIndexer::from_assumed_kernel_history(&burned.history, &chain, [0_u8; 32]).unwrap();

    (burned, indexer)
}

fn synthetic_indexer(clears: &[ClearEntry], burns: &[BurnTransaction]) -> ReferenceIndexer {
    let mut indexer = ReferenceIndexer::empty_for_test(test_fixtures::context(5, block_hash(5)));

    indexer.insert_clear_for_test(genesis_clear()).unwrap();

    for clear in clears {
        indexer.insert_clear_for_test(*clear).unwrap();
    }

    for burn in burns {
        indexer.insert_burn_for_test(burn.clone()).unwrap();
    }

    indexer
}

#[test]
fn over_recognition_is_detected() {
    let (_world, expected) = indexed_burn_world();

    let mut checkpoint = UntrustedIndexerFixture::from_indexer(&expected);

    // Within the checkpoint prefix: reconstruction validates event
    // heights against the checkpoint, so an over-recognized event
    // must sit inside the prefix to reach the differential at all.
    let forged_height = expected.context().checkpoint_height;

    let forged = BurnTransaction {
        txid: txid(99),
        block_hash: block_hash(1),
        order: CanonicalOrder {
            height: forged_height,
            tx_index: 99,
        },
        ash_value: sat(50),
        records: vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(50),
        }],
    };

    checkpoint.events.push(OrderedAttestationEvent {
        order: forged.order,
        id: AttestationEventId::Burn(forged.txid),
    });

    checkpoint.burns.insert(forged.txid, forged);

    let candidate = checkpoint.restore().unwrap();

    assert_eq!(
        compare_attestation_query(&expected, &candidate, ADDRESS_A),
        Err(DifferentialError::QueryMismatch),
    );

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventCountMismatch),
    );
}

#[test]
fn under_recognition_is_detected() {
    let (_world, expected) = indexed_burn_world();

    let mut checkpoint = UntrustedIndexerFixture::from_indexer(&expected);

    let txid_to_remove = *checkpoint.burns.keys().next().unwrap();

    checkpoint.burns.remove(&txid_to_remove);

    checkpoint
        .events
        .retain(|event| event.id != AttestationEventId::Burn(txid_to_remove));

    let candidate = checkpoint.restore().unwrap();

    assert_eq!(
        compare_attestation_query(&expected, &candidate, ADDRESS_A),
        Err(DifferentialError::QueryMismatch),
    );

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventCountMismatch),
    );
}

#[test]
fn identical_event_sets_produce_identical_query() {
    let (_world, expected) = indexed_burn_world();

    let candidate = expected.clone();

    compare_attestation_query(&expected, &candidate, ADDRESS_A).unwrap();

    compare_attestation_events(
        &expected.event_snapshot().unwrap(),
        &candidate.event_snapshot().unwrap(),
    )
    .unwrap();

    compare_attestation_indexers(&expected, &candidate, [ADDRESS_A, ADDRESS_B]).unwrap();
}

#[test]
fn mismatched_contexts_cannot_be_compared() {
    let (_world, expected) = indexed_burn_world();

    let candidate = ReferenceIndexer::empty_for_test(test_fixtures::context(5, block_hash(6)));

    assert_eq!(
        compare_attestation_query(&expected, &candidate, ADDRESS_A),
        Err(DifferentialError::ContextMismatch),
    );

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::ContextMismatch),
    );
}

// The canonical F30 regression: an offsetting event-recognition error
// leaves canonical query bytes identical, so only the raw event
// differential can witness it.
#[test]
fn offsetting_burn_recognition_is_detected() {
    let expected = synthetic_indexer(
        &[],
        &[
            accepted_burn(1, 1, 0, 40, ADDRESS_A),
            accepted_burn(2, 1, 1, 60, ADDRESS_A),
        ],
    );

    let candidate = synthetic_indexer(&[], &[accepted_burn(3, 1, 0, 100, ADDRESS_A)]);

    // The aggregate query happens to match.
    compare_attestation_query(&expected, &candidate, ADDRESS_A).unwrap();

    // Raw recognition does not.
    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventCountMismatch),
    );

    // The combined conformance comparison therefore fails.
    assert_eq!(
        compare_attestation_indexers(&expected, &candidate, [ADDRESS_A]),
        Err(DifferentialError::EventCountMismatch),
    );
}

// The event comparison is not reducible to query comparison: a missing
// clear is invisible to every empty query.
#[test]
fn missing_clear_with_empty_query_is_detected() {
    let expected = synthetic_indexer(&[clear_entry(9, 1, 0, 1_200, 1_000)], &[]);

    let candidate = synthetic_indexer(&[], &[]);

    compare_attestation_query(&expected, &candidate, ADDRESS_B).unwrap();

    assert!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        )
        .is_err()
    );
}

#[test]
fn event_payload_mismatches_are_detected() {
    // Same event identity and order, different ASH value and records.
    let expected = synthetic_indexer(&[], &[accepted_burn(1, 1, 0, 40, ADDRESS_A)]);

    let candidate = synthetic_indexer(&[], &[accepted_burn(1, 1, 0, 60, ADDRESS_A)]);

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventPayloadMismatch),
    );

    // Same identity and order, different block hash.
    let mut moved = accepted_burn(1, 1, 0, 40, ADDRESS_A);

    moved.block_hash = block_hash(77);

    let candidate = synthetic_indexer(&[], &[moved]);

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventPayloadMismatch),
    );

    // Same clear identity and order, different omega.
    let expected = synthetic_indexer(&[clear_entry(9, 1, 0, 1_200, 1_000)], &[]);

    let candidate = synthetic_indexer(&[clear_entry(9, 1, 0, 1_300, 1_000)], &[]);

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventPayloadMismatch),
    );

    // Same clear identity and order, different y.
    let candidate = synthetic_indexer(&[clear_entry(9, 1, 0, 1_200, 900)], &[]);

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventPayloadMismatch),
    );
}

#[test]
fn event_kind_and_order_mismatches_are_detected() {
    // Equal counts and orders, different kinds at one position.
    let expected = synthetic_indexer(&[clear_entry(9, 1, 0, 1_200, 1_000)], &[]);

    let candidate = synthetic_indexer(&[], &[accepted_burn(9, 1, 0, 40, ADDRESS_A)]);

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventKindMismatch),
    );

    // Equal counts and kinds, different canonical orders.
    let expected = synthetic_indexer(&[], &[accepted_burn(1, 1, 0, 40, ADDRESS_A)]);

    let candidate = synthetic_indexer(&[], &[accepted_burn(1, 1, 1, 40, ADDRESS_A)]);

    assert_eq!(
        compare_attestation_events(
            &expected.event_snapshot().unwrap(),
            &candidate.event_snapshot().unwrap(),
        ),
        Err(DifferentialError::EventOrderMismatch),
    );
}

/// An independent test double sharing the expected event snapshot but
/// computing a wrong query. Two `ReferenceIndexer` instances cannot
/// exercise this direction because they share one query algorithm.
struct WrongAggregateCandidate(ReferenceIndexer);

impl AttestationQueryProvider for WrongAggregateCandidate {
    fn context(&self) -> AttestationContext {
        self.0.context()
    }

    fn query(&self, address: AttestationAddress) -> Result<AttestationQueryResult, Guard> {
        let mut query = self.0.query(address)?;

        for term in &mut query.terms {
            term.aggregate_burn_amount += BigUint::one();
        }

        Ok(query)
    }
}

impl IndependentAttestationIndexer for WrongAggregateCandidate {
    fn event_snapshot(&self) -> Result<AttestationEventSnapshot, Guard> {
        self.0.event_snapshot()
    }
}

#[test]
fn query_mismatch_with_equal_event_snapshot_is_detected() {
    let (_world, expected) = indexed_burn_world();

    let candidate = WrongAggregateCandidate(expected.clone());

    // Raw event recognition agrees...
    compare_attestation_events(
        &expected.event_snapshot().unwrap(),
        &candidate.event_snapshot().unwrap(),
    )
    .unwrap();

    // ...but the canonical query bytes do not.
    assert_eq!(
        compare_attestation_query(&expected, &candidate, ADDRESS_A),
        Err(DifferentialError::QueryMismatch),
    );

    assert_eq!(
        compare_attestation_indexers(&expected, &candidate, [ADDRESS_A]),
        Err(DifferentialError::QueryMismatch),
    );
}

#[test]
fn query_uses_indexer_bound_context() {
    let world = indexed_burn_world().0;

    let chain = chain_view_for_history(&world);

    let indexer =
        ReferenceIndexer::from_assumed_kernel_history(&world.history, &chain, [0_u8; 32]).unwrap();

    let query = indexer.query(ADDRESS_A).unwrap();

    assert_eq!(query.context, chain.context());
}

#[test]
fn checkpoint_hash_mismatch_is_rejected() {
    let blocks = vec![CanonicalBlock {
        height: 0,
        hash: block_hash(0),
        parent_hash: None,
    }];

    assert_eq!(
        ValidatedChainView::new(
            TEST_NETWORK_ID,
            TEST_GENESIS_ID,
            test_manifest_hash(),
            0,
            block_hash(99),
            ATTESTATION_SCHEMA_VERSION,
            blocks,
        ),
        Err(Guard::WrongCheckpoint),
    );
}

#[test]
fn noncontiguous_chain_view_is_rejected() {
    let view = ValidatedChainView::new(
        TEST_NETWORK_ID,
        TEST_GENESIS_ID,
        test_manifest_hash(),
        2,
        block_hash(2),
        ATTESTATION_SCHEMA_VERSION,
        vec![
            CanonicalBlock {
                height: 0,
                hash: block_hash(0),
                parent_hash: None,
            },
            CanonicalBlock {
                height: 2,
                hash: block_hash(2),
                parent_hash: Some(block_hash(0)),
            },
        ],
    )
    .unwrap();

    assert_eq!(view.validate_prefix_from(0), Err(Guard::HistoryOrder));
}

#[test]
fn parent_hash_mismatch_is_rejected() {
    let view = ValidatedChainView::new(
        TEST_NETWORK_ID,
        TEST_GENESIS_ID,
        test_manifest_hash(),
        1,
        block_hash(1),
        ATTESTATION_SCHEMA_VERSION,
        vec![
            CanonicalBlock {
                height: 0,
                hash: block_hash(0),
                parent_hash: None,
            },
            CanonicalBlock {
                height: 1,
                hash: block_hash(1),
                parent_hash: Some(block_hash(99)),
            },
        ],
    )
    .unwrap();

    assert_eq!(view.validate_prefix_from(0), Err(Guard::HistoryOrder));
}

#[test]
fn unsupported_attestation_schema_is_rejected() {
    assert_eq!(
        ValidatedChainView::new(
            TEST_NETWORK_ID,
            TEST_GENESIS_ID,
            test_manifest_hash(),
            0,
            block_hash(0),
            ATTESTATION_SCHEMA_VERSION + 1,
            vec![CanonicalBlock {
                height: 0,
                hash: block_hash(0),
                parent_hash: None,
            }],
        ),
        Err(Guard::UnsupportedSchema),
    );
}
