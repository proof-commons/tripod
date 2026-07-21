//! Residue noninterference tests.
//!
//! Implements `´test:verification:residue-noninterference´` and the
//! receipt-accounting audit-projection boundary
//! `´test:verification:residue-audit-differential´`.
//!
//! The audit-boundary test demonstrates the exact intended boundary:
//! the audit identity fails while the monetary floor is unchanged.
//! The audit-projection tests witness that residue is compared by the
//! separate receipt-accounting differential — never by the
//! attestation indexer — and that offsetting residue events with
//! equal totals are still a projection mismatch.

use num_bigint::BigUint;
use num_traits::Zero;

use super::advanced_fixtures;
use super::advanced_fixtures::world_with_terminal_residue;
use super::scenario_fixtures::*;
use super::test_fixtures;
use super::test_fixtures::chain_view_for_history;
use crate::*;

#[test]
fn residue_does_not_change_floor_or_monetary_queries() {
    let world = world_with_terminal_residue();

    let floor_before = floor_terms(&world).unwrap();

    let payout_before = redemption_payout(&world, sat(10)).unwrap();

    let issuance_before = cycle_issuance_query(&world).unwrap();

    let mut perturbed = world;

    perturb_residue_projection(&mut perturbed.history, sat(7), sat(11));

    assert_eq!(floor_terms(&perturbed).unwrap(), floor_before);

    assert_eq!(
        redemption_payout(&perturbed, sat(10)).unwrap(),
        payout_before,
    );

    assert_eq!(cycle_issuance_query(&perturbed).unwrap(), issuance_before);
}

// The perturbation targets a *legitimate* settlement residue: the
// ingestion boundary rejects a residue projection on any other
// branch outright, so residue noninterference is claimed exactly
// where a residue can exist at all.
#[test]
fn residue_does_not_change_attestation_query() {
    let world =
        advanced_fixtures::give_live_receipt(&world_with_terminal_residue(), ALICE, sat(100));

    let burned = advanced_fixtures::burn_from_owner(
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

    let before_indexer =
        ReferenceIndexer::from_model_history(&burned.history, &chain, [0_u8; 32]).unwrap();

    let before_query = before_indexer.query(ADDRESS_A).unwrap();

    let mut perturbed = burned;

    let residue = perturbed
        .history
        .transitions
        .iter_mut()
        .find_map(|certificate| certificate.distribution_residue.as_mut())
        .unwrap();

    residue.live_residue = sat(13);
    residue.time_locked_residue = sat(17);

    let after_indexer =
        ReferenceIndexer::from_model_history(&perturbed.history, &chain, [0_u8; 32]).unwrap();

    let after_query = after_indexer.query(ADDRESS_A).unwrap();

    assert_eq!(before_query, after_query);
}

#[test]
fn residue_corruption_changes_only_audit_reconciliation() {
    let world = world_with_terminal_residue();

    assert!(check_invariant(&world).is_ok());

    let mut perturbed = world.clone();

    perturb_residue_projection(&mut perturbed.history, sat(1), Sat::ZERO);

    assert!(matches!(
        check_invariant(&perturbed),
        Err(InvariantError::ReceiptAccountingPreMaturity
            | InvariantError::ReceiptAccountingPostMaturity)
    ));

    assert_eq!(floor_terms(&perturbed), floor_terms(&world));
}

fn synthetic_residue_event(nonce: u8, live: Sat, time_locked: Sat) -> ResidueAuditEvent {
    ResidueAuditEvent {
        txid: test_fixtures::txid(nonce),
        order: test_fixtures::later_order(100 + u64::from(nonce), 0),
        cycle: 1,
        control_input: 0,
        vault_input: None,
        live_residue: live,
        time_locked_residue: time_locked,
    }
}

/// The canonical F30 residue regression: an extra and a missing
/// residue of equal value preserve both aggregate totals, yet the
/// exact event-sequence comparison detects the substitution.
#[test]
fn offsetting_residue_recognition_is_detected() {
    let base = receipt_accounting_audit(&world_with_terminal_residue()).unwrap();

    let mut expected = base.clone();

    expected.residue_events = vec![
        synthetic_residue_event(1, sat(4), sat(6)),
        synthetic_residue_event(2, sat(3), sat(7)),
    ];
    expected.historical_live_residue = BigUint::from(7_u64);
    expected.historical_time_locked_residue = BigUint::from(13_u64);

    let mut candidate = base;

    candidate.residue_events = vec![synthetic_residue_event(3, sat(7), sat(13))];
    candidate.historical_live_residue = BigUint::from(7_u64);
    candidate.historical_time_locked_residue = BigUint::from(13_u64);

    // Aggregate totals match on both axes.
    assert_eq!(
        expected.historical_live_residue,
        candidate.historical_live_residue,
    );

    assert_eq!(
        expected.historical_time_locked_residue,
        candidate.historical_time_locked_residue,
    );

    // The exact residue event sequence does not.
    assert_eq!(
        compare_receipt_accounting_audit(&expected, &candidate),
        Err(DifferentialError::ReceiptAccountingProjectionMismatch),
    );
}

/// Historical totals are derived from the exact residue event list,
/// never accepted independently.
#[test]
fn audit_totals_are_derived_from_the_event_sequence() {
    let projection = receipt_accounting_audit(&world_with_terminal_residue()).unwrap();

    let live = projection
        .residue_events
        .iter()
        .fold(BigUint::zero(), |acc, event| {
            acc + BigUint::from(event.live_residue.get())
        });

    let time_locked = projection
        .residue_events
        .iter()
        .fold(BigUint::zero(), |acc, event| {
            acc + BigUint::from(event.time_locked_residue.get())
        });

    assert_eq!(live, projection.historical_live_residue);

    assert_eq!(time_locked, projection.historical_time_locked_residue);
}

/// The reader firewall, seen from the audit side: perturbing residue
/// changes the receipt-accounting audit projection while the
/// companion test `residue_does_not_change_attestation_query` shows
/// the attestation indexer remains residue-blind.
#[test]
fn residue_perturbation_changes_audit_projection() {
    let world = world_with_terminal_residue();

    let expected = receipt_accounting_audit(&world).unwrap();

    let mut perturbed = world;

    perturb_residue_projection(&mut perturbed.history, sat(7), sat(11));

    let candidate = receipt_accounting_audit(&perturbed).unwrap();

    assert_eq!(
        compare_receipt_accounting_audit(&expected, &candidate),
        Err(DifferentialError::ReceiptAccountingProjectionMismatch),
    );
}
