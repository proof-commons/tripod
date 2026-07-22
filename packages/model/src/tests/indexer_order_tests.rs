//! Canonical event-order and one-pass query tests.
//!
//! Implements `´test:verification:indexer-event-order´`.
//!
//! The slow oracle re-implements the scan-per-burn semantics that the
//! one-pass merge replaced. It is test-only; its independence from the
//! production traversal is what makes the equivalence useful evidence.

use std::collections::BTreeMap;

use num_bigint::BigUint;
use num_traits::Zero;

use super::attestation_history_fixture::{accepted_burn, clear_entry, genesis_clear};
use super::scenario_fixtures::{ADDRESS_A, ADDRESS_B, sat};
use super::test_fixtures::{block_hash, context, txid};
use crate::ledger::UntrustedIndexerFixture;
use crate::*;

fn most_recent_clear_before_slow(
    indexer: &ReferenceIndexer,
    order: CanonicalOrder,
) -> Result<ClearEntry, Guard> {
    indexer
        .clears()
        .values()
        .filter(|clear| clear.order < order)
        .max_by_key(|clear| (clear.order, clear.clear_id))
        .copied()
        .ok_or(Guard::NoSuch)
}

fn slow_terms_for_address(
    indexer: &ReferenceIndexer,
    address: AttestationAddress,
) -> Result<Vec<AttestationTerm>, Guard> {
    let mut grouped: BTreeMap<(AttestationAddress, ClearId), BigUint> = BTreeMap::new();

    for burn in indexer.burns().values() {
        if !burn.records_accepted()? {
            continue;
        }

        let clear = most_recent_clear_before_slow(indexer, burn.order)?;

        for record in &burn.records {
            let entry = grouped
                .entry((record.address, clear.clear_id))
                .or_insert_with(BigUint::zero);

            *entry += BigUint::from(record.amount.get());
        }
    }

    let mut terms = Vec::new();

    for ((term_address, clear_id), amount) in grouped {
        if term_address != address || amount.is_zero() {
            continue;
        }

        let clear = indexer.clears().get(&clear_id).ok_or(Guard::NoSuch)?;

        terms.push(AttestationTerm {
            clear_id,
            clear_order: clear.order,
            aggregate_burn_amount: amount,
            omega: BigUint::from(clear.omega.get()),
            y: BigUint::from(clear.y.get()),
        });
    }

    terms.sort_by_key(|term| (term.clear_order, term.clear_id));

    Ok(terms)
}

/// Multiple clearings, several burns between clearings, several
/// records per burn, under-claiming, over-claiming, and two addresses.
fn rich_indexer() -> ReferenceIndexer {
    let mut indexer = ReferenceIndexer::empty_for_test(context(9, block_hash(9)));

    indexer.insert_clear_for_test(genesis_clear()).unwrap();

    // Burn before any ordinary clear: credited to genesis.
    indexer
        .insert_burn_for_test(accepted_burn(1, 1, 0, 10, ADDRESS_A))
        .unwrap();

    indexer
        .insert_clear_for_test(clear_entry(2, 2, 0, 1_200, 1_000))
        .unwrap();

    // Two burns between the first and second ordinary clearings, one
    // with several records across two addresses.
    indexer
        .insert_burn_for_test(BurnTransaction {
            txid: txid(3),
            block_hash: block_hash(3),
            order: CanonicalOrder {
                height: 3,
                tx_index: 0,
            },
            ash_value: sat(100),
            records: vec![
                BurnRecord {
                    record_index: 0,
                    address: ADDRESS_A,
                    amount: sat(30),
                },
                BurnRecord {
                    record_index: 1,
                    address: ADDRESS_B,
                    amount: sat(50),
                },
            ],
        })
        .unwrap();

    // Under-claiming burn: valid donation of the difference.
    indexer
        .insert_burn_for_test(BurnTransaction {
            txid: txid(4),
            block_hash: block_hash(3),
            order: CanonicalOrder {
                height: 3,
                tx_index: 1,
            },
            ash_value: sat(40),
            records: vec![BurnRecord {
                record_index: 0,
                address: ADDRESS_A,
                amount: sat(25),
            }],
        })
        .unwrap();

    indexer
        .insert_clear_for_test(clear_entry(5, 5, 0, 1_300, 900))
        .unwrap();

    // Over-claiming burn: the event stays in the raw stream, the
    // records contribute nothing.
    indexer
        .insert_burn_for_test(BurnTransaction {
            txid: txid(6),
            block_hash: block_hash(6),
            order: CanonicalOrder {
                height: 6,
                tx_index: 0,
            },
            ash_value: sat(10),
            records: vec![BurnRecord {
                record_index: 0,
                address: ADDRESS_A,
                amount: sat(11),
            }],
        })
        .unwrap();

    indexer
        .insert_burn_for_test(accepted_burn(7, 6, 1, 7, ADDRESS_B))
        .unwrap();

    indexer
}

#[test]
fn one_pass_query_matches_slow_oracle() {
    let indexer = rich_indexer();

    for address in [ADDRESS_A, ADDRESS_B, AttestationAddress([99_u8; 32])] {
        assert_eq!(
            indexer.query(address).unwrap().terms,
            slow_terms_for_address(&indexer, address).unwrap(),
        );
    }
}

#[test]
fn overclaiming_burn_stays_in_raw_stream_but_credits_nothing() {
    let indexer = rich_indexer();

    assert!(
        indexer
            .events()
            .iter()
            .any(|event| event.id == AttestationEventId::Burn(txid(6)))
    );

    let snapshot = indexer.event_snapshot().unwrap();

    assert!(snapshot.events.iter().any(
        |event| matches!(event, RecognizedAttestationEvent::Burn(burn) if burn.txid == txid(6)),
    ));

    // No term for the over-claimed clear window's ADDRESS_A record.
    let terms = indexer.query(ADDRESS_A).unwrap().terms;

    assert!(
        terms
            .iter()
            .all(|term| term.clear_id != ClearId::Transaction(txid(5)))
    );
}

#[test]
fn genesis_clear_is_the_first_event() {
    let indexer = rich_indexer();

    assert!(matches!(
        indexer.events().first().unwrap().id,
        AttestationEventId::GenesisClear(ClearId::Genesis(_)),
    ));

    // A burn before any ordinary clear uses genesis, and a query never
    // fails merely because no ordinary clear exists.
    let mut only_genesis = ReferenceIndexer::empty_for_test(context(1, block_hash(1)));

    only_genesis.insert_clear_for_test(genesis_clear()).unwrap();

    only_genesis
        .insert_burn_for_test(accepted_burn(1, 1, 0, 10, ADDRESS_A))
        .unwrap();

    let terms = only_genesis.query(ADDRESS_A).unwrap().terms;

    assert_eq!(terms.len(), 1);

    assert!(matches!(terms[0].clear_id, ClearId::Genesis(_)));
}

#[test]
fn same_block_clear_before_burn_is_sampled() {
    let mut indexer = ReferenceIndexer::empty_for_test(context(5, block_hash(5)));

    indexer.insert_clear_for_test(genesis_clear()).unwrap();

    let clear = clear_entry(1, 5, 1, 1_200, 1_000);

    indexer.insert_clear_for_test(clear).unwrap();

    indexer
        .insert_burn_for_test(accepted_burn(2, 5, 2, 10, ADDRESS_A))
        .unwrap();

    let terms = indexer.query(ADDRESS_A).unwrap().terms;

    assert_eq!(terms[0].clear_id, clear.clear_id);
}

#[test]
fn same_block_burn_before_clear_uses_previous_clear() {
    let mut indexer = ReferenceIndexer::empty_for_test(context(5, block_hash(5)));

    indexer.insert_clear_for_test(genesis_clear()).unwrap();

    indexer
        .insert_burn_for_test(accepted_burn(2, 5, 1, 10, ADDRESS_A))
        .unwrap();

    indexer
        .insert_clear_for_test(clear_entry(1, 5, 2, 1_200, 1_000))
        .unwrap();

    let terms = indexer.query(ADDRESS_A).unwrap().terms;

    assert!(matches!(terms[0].clear_id, ClearId::Genesis(_)));
}

// Event-index completeness faults: every corrupted correspondence must
// fail `validate_event_index` on reconstruction.

fn valid_checkpoint() -> UntrustedIndexerFixture {
    UntrustedIndexerFixture::from_indexer(&rich_indexer())
}

#[test]
fn burn_in_map_but_absent_from_events_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint
        .events
        .retain(|event| event.id != AttestationEventId::Burn(txid(3)));

    assert!(checkpoint.check().is_err());
}

#[test]
fn event_referencing_missing_burn_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint.burns.remove(&txid(3));

    assert!(checkpoint.check().is_err());
}

#[test]
fn clear_in_map_but_absent_from_events_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint
        .events
        .retain(|event| event.id != AttestationEventId::Clear(ClearId::Transaction(txid(2))));

    assert!(checkpoint.check().is_err());
}

#[test]
fn event_referencing_missing_clear_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint.clears.remove(&ClearId::Transaction(txid(2)));

    assert!(checkpoint.check().is_err());
}

#[test]
fn duplicate_event_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    let last = *checkpoint.events.last().unwrap();

    checkpoint.events.push(last);

    assert!(checkpoint.check().is_err());
}

#[test]
fn non_increasing_event_order_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    let length = checkpoint.events.len();

    checkpoint.events.swap(length - 1, length - 2);

    assert!(checkpoint.check().is_err());
}

#[test]
fn ordinary_event_before_genesis_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint.events.swap(0, 1);

    assert!(checkpoint.check().is_err());
}

#[test]
fn two_events_at_one_canonical_order_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    let order = checkpoint.events.last().unwrap().order;

    let forged = BurnTransaction {
        txid: txid(50),
        block_hash: block_hash(6),
        order,
        ash_value: sat(1),
        records: Vec::new(),
    };

    checkpoint.events.push(OrderedAttestationEvent {
        order,
        id: AttestationEventId::Burn(forged.txid),
    });

    checkpoint.burns.insert(forged.txid, forged);

    assert!(checkpoint.check().is_err());
}

// Genesis-anchor faults: a checkpoint with no genesis clearing (or a
// duplicated one) must never validate. An empty index in particular is
// not vacuously valid — the genesis clearing anchors the total clear
// assignment.

#[test]
fn completely_empty_checkpoint_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint.burns.clear();
    checkpoint.clears.clear();
    checkpoint.events.clear();

    assert!(checkpoint.check().is_err());
}

#[test]
fn clear_map_without_any_event_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint.burns.clear();
    checkpoint.events.clear();

    assert!(checkpoint.check().is_err());
}

#[test]
fn event_stream_without_genesis_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    // Drop the genesis clearing from both the event stream and the
    // clear table, keeping every ordinary event self-consistent.
    checkpoint
        .events
        .retain(|event| !matches!(event.id, AttestationEventId::GenesisClear(_)));
    checkpoint
        .clears
        .retain(|clear_id, _| !matches!(clear_id, ClearId::Genesis(_)));

    assert!(checkpoint.check().is_err());
}

#[test]
fn ordinary_clear_as_first_event_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    // Remove only the genesis event (the table entry stays), so the
    // stream now begins with an ordinary event.
    checkpoint
        .events
        .retain(|event| !matches!(event.id, AttestationEventId::GenesisClear(_)));

    assert!(checkpoint.check().is_err());
}

#[test]
fn genesis_event_referencing_missing_clear_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    // The genesis event stays in the stream, but its clear-table entry
    // is gone: the event references a missing clear.
    checkpoint
        .clears
        .retain(|clear_id, _| !matches!(clear_id, ClearId::Genesis(_)));

    assert!(checkpoint.check().is_err());
}

#[test]
fn duplicated_genesis_event_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    // The same genesis event twice: the second occurrence sits at a
    // non-zero index and duplicates the seen clear id.
    let genesis = *checkpoint
        .events
        .first()
        .expect("valid checkpoint has a genesis event");
    checkpoint.events.insert(1, genesis);

    assert!(checkpoint.check().is_err());
}

#[test]
fn second_distinct_genesis_clear_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    let second_id = ClearId::Genesis([0xEE; 32]);
    let second = ClearEntry {
        clear_id: second_id,
        block_hash: block_hash(0),
        order: CanonicalOrder {
            height: 0,
            tx_index: 1,
        },
        omega: sat(1),
        y: sat(1),
    };

    checkpoint.clears.insert(second_id, second);
    checkpoint.events.insert(
        1,
        OrderedAttestationEvent {
            order: second.order,
            id: AttestationEventId::GenesisClear(second_id),
        },
    );

    assert!(checkpoint.check().is_err());
}

#[test]
fn burn_payload_order_differing_from_event_order_is_rejected() {
    let mut checkpoint = valid_checkpoint();

    checkpoint.burns.get_mut(&txid(3)).unwrap().order = CanonicalOrder {
        height: 3,
        tx_index: 7,
    };

    assert!(checkpoint.check().is_err());
}

#[test]
fn valid_checkpoint_round_trips() {
    let indexer = rich_indexer();

    let rebuilt = UntrustedIndexerFixture::from_indexer(&indexer).restore().unwrap();

    assert_eq!(rebuilt, indexer);

    assert_eq!(
        rebuilt.query(ADDRESS_A).unwrap(),
        indexer.query(ADDRESS_A).unwrap(),
    );
}
