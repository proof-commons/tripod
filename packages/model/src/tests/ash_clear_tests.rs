//! ASH compaction and clear tests.
//!
//! Implements `´test:verification:ash-clear´`.

use super::advanced_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

pub(super) fn world_with_two_ash() -> World {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let first = burn_from_owner(&world, ALICE, sat(50), Vec::new());

    let with_bob = give_live_receipt(&first, BOB, sat(50));

    burn_from_owner(&with_bob, BOB, sat(50), Vec::new())
}

pub(super) fn world_with_three_ash() -> World {
    let world = give_live_receipt(&world_with_two_ash(), CAROL, sat(30));
    burn_from_owner(&world, CAROL, sat(30), Vec::new())
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
fn clear_changes_only_the_stored_y_l_and_the_derived_total() {
    let world = world_with_two_ash();

    let old_state = world.state().unwrap().1;
    let PoolState {
        omega: old_omega,
        y_l: old_y_l,
        y_t: old_y_t,
        q: old_q,
        cycle: old_cycle,
        maturity: old_maturity,
    } = old_state;
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
    let PoolState {
        omega,
        y_l,
        y_t,
        q,
        cycle,
        maturity,
    } = new_state;
    let remaining_ash = find_ash(&next)
        .iter()
        .map(|outpoint| next.utxo(*outpoint).unwrap().value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();
    let cleared = old_y_l.checked_sub(y_l).unwrap();

    assert!(!cleared.is_zero());
    assert_eq!(omega, old_omega);
    assert_eq!(y_t, old_y_t);
    assert_eq!(q, old_q);
    assert_eq!(cycle, old_cycle);
    assert_eq!(maturity, old_maturity);
    assert_eq!(cleared, ash_total.checked_sub(remaining_ash).unwrap());
    assert_eq!(
        old_state
            .y()
            .unwrap()
            .checked_sub(new_state.y().unwrap())
            .unwrap(),
        cleared,
    );
    assert_eq!(ash_total.checked_sub(cleared).unwrap(), remaining_ash);
    assert_eq!(
        new_state.is_sealed().unwrap(),
        old_state.is_sealed().unwrap()
    );
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

fn assert_executed_clear_agrees(world: &World, expected_x: u64, expected_residual: u64) {
    use super::state_metadata_conformance_tests::project;

    check_invariant(world).unwrap();
    let (input, predecessor) = world.state().unwrap();
    let ash_inputs = find_ash(world);
    let family: Vec<_> = ash_inputs
        .iter()
        .map(|&outpoint| {
            let value = world.utxo(outpoint).unwrap().value;
            (
                outpoint,
                realization::ProtocolAmount::new(value.get()).unwrap(),
            )
        })
        .collect();
    let executed = execute_bound(
        world,
        ClearAsh {
            ash_inputs,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(world),
    )
    .unwrap();
    check_invariant(executed.after()).unwrap();
    let (output, successor) = executed.after().state().unwrap();
    let realized = realization::clear_ash(
        &project(predecessor),
        &family,
        world.constants.ash_batch_max,
    )
    .unwrap();
    // Expectations come from the scenario, not the realization result.
    assert_eq!(predecessor.y_l.get() - successor.y_l.get(), expected_x);
    assert_eq!(realized.clear_amount.get(), expected_x);
    assert_eq!(realized.residual.get(), expected_residual);
    assert_eq!(realized.successor, project(successor));
    for field in realization::StateField::ALL {
        match field {
            realization::StateField::Omega => assert_eq!(successor.omega, predecessor.omega),
            realization::StateField::YL => {
                assert_eq!(successor.y_l.get(), predecessor.y_l.get() - expected_x);
            }
            realization::StateField::YT => assert_eq!(successor.y_t, predecessor.y_t),
            realization::StateField::Q => assert_eq!(successor.q, predecessor.q),
            realization::StateField::Cycle => assert_eq!(successor.cycle, predecessor.cycle),
            realization::StateField::Maturity => {
                assert_eq!(successor.maturity, predecessor.maturity);
            }
        }
    }
    assert!(successor.y().unwrap() >= Sat::ONE);
    assert_eq!(
        successor.y().unwrap().get(),
        predecessor.y().unwrap().get() - expected_x
    );
    assert_ne!(input, output);
    assert!(!executed.after().utxos.contains_key(&input));
    assert_eq!(
        executed.certificate().state_edge,
        Some(RootEdge::Succ { input, output })
    );
    let residuals = find_ash(executed.after());
    if expected_residual == 0 {
        assert_eq!(residuals, [] as [u64; 0]);
    } else {
        assert_eq!(residuals.len(), 1);
        assert_eq!(
            executed.after().utxo(residuals[0]).unwrap().value.get(),
            expected_residual
        );
    }
}

#[test]
fn executed_full_clear_successor_agrees() {
    let world = world_with_two_ash();
    assert_eq!(find_ash(&world).len(), 2);
    assert_executed_clear_agrees(&world, 100, 0);
}

#[test]
fn executed_partial_clear_successor_agrees() {
    let (world, receipt) = sealing_world();
    let supply = world.state().unwrap().1.y().unwrap();
    let burned = apply_checked(
        &world,
        &BurnReceipts {
            receipts: vec![receipt],
            signers: signers(&[GENESIS_OWNER]),
            ash_value: supply,
            change: Vec::new(),
            records: Vec::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );
    assert_executed_clear_agrees(&burned, supply.get() - 1, 1);
}

#[test]
fn executed_ash_live_tie_successor_agrees() {
    let world = test_fixtures::world();
    let live = world.state().unwrap().1.y_l;
    let y = world.state().unwrap().1.y().unwrap();
    assert!(y.get() - 1 > live.get());
    let burned = burn_from_owner(&world, GENESIS_OWNER, live, Vec::new());
    assert_executed_clear_agrees(&burned, live.get(), 0);
}

#[test]
fn executed_ash_floor_tie_successor_agrees() {
    let (world, _) = sealing_world();
    let supply = world.state().unwrap().1.y().unwrap();
    let capacity = supply.checked_sub(Sat::ONE).unwrap();
    assert!(world.state().unwrap().1.y_l > capacity);
    let burned = burn_from_owner(&world, GENESIS_OWNER, capacity, Vec::new());
    assert_executed_clear_agrees(&burned, capacity.get(), 0);
}

#[test]
fn executed_three_way_tie_successor_agrees() {
    // The ordinary fixture's locked supply exceeds one. A valid two-unit
    // genesis with its existing half split reaches YL=YT=B=1 honestly.
    let world = genesis(
        test_fixtures::constants(),
        sat(2),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        test_fixtures::txid(0),
    )
    .unwrap();
    let burned = burn_from_owner(&world, GENESIS_OWNER, Sat::ONE, Vec::new());
    let state = burned.state().unwrap().1;
    assert_eq!((state.y_l.get(), state.y_t.get()), (1, 1));
    assert_executed_clear_agrees(&burned, 1, 0);
}

#[test]
fn executed_clear_preserves_nontrivial_metadata() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let with_bob = give_live_receipt(&world, BOB, sat(37));
    let receipt = find_receipts(&with_bob, BOB, ReceiptClass::Live)[0];
    // Burn preserves YL, so the genesis half split leaves YL=YT. Redemption
    // supplies a reachable predecessor whose copied quantities are distinct.
    let world = apply_checked(
        &with_bob,
        &RedeemReceipt {
            receipt,
            signers: signers(&[BOB]),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&with_bob),
    );
    let aged = advance_blocks(&world, world.constants.max_cadence_blocks);
    let cycled = apply_checked(
        &aged,
        &RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&aged),
    );
    let (announced, _) = super::distribution_fixtures::announce_at_minimum_lead(&cycled);
    let requested = create_request_for(&announced, ALICE, BOB, sat(1234), Sat::ONE);
    let admitted = admit_all_requests(&requested);
    let burned = burn_from_owner(&admitted, ALICE, sat(50), Vec::new());
    let state = burned.state().unwrap().1;
    assert!(!state.q.is_zero());
    assert_ne!(state.cycle, 0);
    assert!(matches!(state.maturity, Maturity::Announced { .. }));
    assert_eq!(
        std::collections::BTreeSet::from([
            state.omega.get(),
            state.y_l.get(),
            state.y_t.get(),
            state.q.get(),
            state.cycle,
        ])
        .len(),
        5
    );
    assert_executed_clear_agrees(&burned, 50, 0);
}

#[test]
fn clear_world_lookup_and_recognition_remain_outside_helper_domain() {
    let world = world_with_two_ash();
    let ash = find_ash(&world);
    for (input, expected) in [
        (u64::MAX, Guard::NoSuch),
        (world.state().unwrap().0, Guard::WrongAsset),
    ] {
        let request = ClearAsh {
            ash_inputs: vec![input],
            fee_envelope: FeeEnvelope::default(),
        };
        assert_eq!(request.apply(&world, next_order(&world)), Err(expected));
    }
    let mut wrong_shape = world.clone();
    wrong_shape.utxos.get_mut(&ash[0]).unwrap().meta = Meta::State(world.state().unwrap().1);
    let request = ClearAsh {
        ash_inputs: vec![ash[0]],
        fee_envelope: FeeEnvelope::default(),
    };
    assert_eq!(
        request.apply(&wrong_shape, next_order(&wrong_shape)),
        Err(Guard::WrongShape)
    );
    assert_executed_clear_agrees(&world, 100, 0);
}
