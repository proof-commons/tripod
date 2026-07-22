//! ASH compaction and clear tests.
//!
//! Implements `´test:verification:ash-clear´`.

use super::advanced_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

fn world_with_two_ash() -> World {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let first = burn_from_owner(&world, ALICE, sat(50), Vec::new());

    let with_bob = give_live_receipt(&first, BOB, sat(50));

    burn_from_owner(&with_bob, BOB, sat(50), Vec::new())
}

#[test]
fn compaction_preserves_ash_value_and_reduces_count() {
    let world = world_with_two_ash();

    let ash = find_ash(&world);

    let total = ash
        .iter()
        .map(|outpoint| world.utxo(*outpoint).unwrap().value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();

    let next = apply_checked(
        &world,
        &CompactAsh {
            ash_inputs: ash,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let next_ash = find_ash(&next);

    assert_eq!(next_ash.len(), 1);
    assert_eq!(next.utxo(next_ash[0]).unwrap().value, total);

    assert!(next.history.transitions.last().unwrap().burn.is_none(),);
}

#[test]
fn clear_decrements_y_l_and_ash_equally() {
    let world = world_with_two_ash();

    let old_state = world.state().unwrap().1;

    let ash = find_ash(&world);

    let ash_total = ash
        .iter()
        .map(|outpoint| world.utxo(*outpoint).unwrap().value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();

    let next = apply_checked(
        &world,
        &ClearAsh {
            ash_inputs: ash,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let new_state = next.state().unwrap().1;

    let cleared = old_state.y_l.checked_sub(new_state.y_l).unwrap();

    assert!(cleared <= ash_total);

    let remaining_ash = find_ash(&next)
        .iter()
        .map(|outpoint| next.utxo(*outpoint).unwrap().value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();

    assert_eq!(ash_total.checked_sub(cleared).unwrap(), remaining_ash);

    let certificate = next.history.transitions.last().unwrap();

    assert!(certificate.clear.is_some());
    assert!(certificate.burn.is_none());
}

#[test]
fn clear_never_reaches_zero_total_supply() {
    let (world, receipt) = sealing_world();

    let burned = apply_checked(
        &world,
        &BurnReceipts {
            receipts: vec![receipt],
            signers: signers(&[GENESIS_OWNER]),
            ash_value: world.utxo(receipt).unwrap().value,
            change: Vec::new(),
            records: Vec::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let ash = find_ash(&burned);

    let next = apply_checked(
        &burned,
        &ClearAsh {
            ash_inputs: ash,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&burned),
    );

    assert_eq!(next.state().unwrap().1.y(), Ok(Sat::ONE));

    assert_eq!(find_ash(&next).len(), 1);
}

#[test]
fn zero_progress_clear_is_rejected() {
    let (world, receipt) = sealing_world();

    let burned = apply_checked(
        &world,
        &BurnReceipts {
            receipts: vec![receipt],
            signers: signers(&[GENESIS_OWNER]),
            ash_value: world.utxo(receipt).unwrap().value,
            change: Vec::new(),
            records: Vec::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let first = apply_checked(
        &burned,
        &ClearAsh {
            ash_inputs: find_ash(&burned),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&burned),
    );

    assert_eq!(
        ClearAsh {
            ash_inputs: find_ash(&first),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&first, next_order(&first)),
        Err(Guard::ZeroProgress),
    );
}

/// A single ash whose value exceeds what clear can consume partially
/// clears: the clamp (`min(ash, y_l, y - 1)`) fires, the residual is
/// re-emitted as ASH, and nothing strands. This witnesses the clamp
/// directly rather than by absence of an older behaviour.
#[test]
fn oversized_single_ash_partially_clears_with_residual() {
    let (world, receipt) = sealing_world();

    let y = world.state().unwrap().1.y().unwrap();

    // Burn the entire supply into one ash object: its value `y`
    // exceeds the clear capacity `y - 1`.
    let burned = apply_checked(
        &world,
        &BurnReceipts {
            receipts: vec![receipt],
            signers: signers(&[GENESIS_OWNER]),
            ash_value: y,
            change: Vec::new(),
            records: Vec::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let ash = find_ash(&burned);

    assert_eq!(ash.len(), 1);
    assert_eq!(burned.utxo(ash[0]).unwrap().value, y);

    let next = apply_checked(
        &burned,
        &ClearAsh {
            ash_inputs: ash,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&burned),
    );

    // Partial clear: exactly `y - 1` cleared, supply floor preserved.
    let state = next.state().unwrap().1;

    assert_eq!(state.y(), Ok(Sat::ONE));

    // The residual is re-emitted as ASH — not stranded, not dropped.
    let residual = find_ash(&next);

    assert_eq!(residual.len(), 1);
    assert_eq!(next.utxo(residual[0]).unwrap().value, Sat::ONE);

    let certificate = next.history.transitions.last().unwrap();

    assert!(certificate.clear.is_some());
    assert!(certificate.burn.is_none());
}
