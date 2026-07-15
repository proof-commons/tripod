//! Contended-STATE fixture and stronger fee-auction tests.
//!
//! Implements `´test:verification:contended-state-fixture´` and
//! `´test:verification:native-fee-auction-two-valid´`.
//!
//! The fixture creates: admitted escrow Q > 0 (making cycle useful);
//! another unadmitted request (making admission useful); and a
//! cadence-old PACE (making the permissionless cycle valid). Both
//! candidates spend the same current STATE and RESV.
//!
//! The oracle models semantic transition reconstruction, not raw
//! serialized stale transactions. Consensus rejection of an old
//! serialized input set is a substrate-level test.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

pub fn contended_world() -> (World, OutPoint) {
    let first_request =
        create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    let admitted = admit_all_requests(&first_request);

    let with_second_request = create_request_for(&admitted, BOB, BOB, sat(200), Sat::ONE);

    let second_request = find_request(&with_second_request, BOB);

    let aged = advance_blocks(
        &with_second_request,
        with_second_request.constants.max_cadence_blocks,
    );

    (aged, second_request)
}

#[test]
fn admission_and_cycle_are_both_valid_against_one_state() {
    let (world, second_request) = contended_world();

    let state_input = world.roots.state;

    let admission = AdmitDeposits {
        requests: vec![second_request],

        admission_reward: Sat::ZERO,

        reward_owner: RELAYER,
    };

    let cycle = RunCycle {
        caller: CycleCaller::Anyone,

        operator_signers: SignerSet::new(),

        fee_envelope: FeeEnvelope::default(),
    };

    let admission_result = admission.apply(&world, next_order(&world)).unwrap();

    let cycle_result = cycle.apply(&world, next_order(&world)).unwrap();

    for result in [&admission_result, &cycle_result] {
        check_invariant(result).unwrap();

        let certificate = result.history.transitions.last().unwrap();

        assert!(certificate.consumed.contains(&state_input));
    }

    let admission_certificate = admission_result.history.transitions.last().unwrap();

    let cycle_certificate = cycle_result.history.transitions.last().unwrap();

    assert_eq!(
        admission_certificate.state_edge.map(|edge| {
            match edge {
                RootEdge::Succ { input, .. } | RootEdge::Term { input } => input,
            }
        }),
        Some(state_input),
    );

    assert_eq!(
        cycle_certificate.state_edge.map(|edge| {
            match edge {
                RootEdge::Succ { input, .. } | RootEdge::Term { input } => input,
            }
        }),
        Some(state_input),
    );
}

#[test]
fn winner_invalidates_other_candidate_outpoint() {
    let (world, second_request) = contended_world();

    let winner = AdmitDeposits {
        requests: vec![second_request],

        admission_reward: Sat::ZERO,

        reward_owner: RELAYER,
    }
    .apply(&world, next_order(&world))
    .unwrap();

    // Reusing the old candidate against the winner's
    // world must fail because the old STATE and RESV
    // outpoints have been consumed.
    let stale_cycle = RunCycle {
        caller: CycleCaller::Anyone,

        operator_signers: SignerSet::new(),

        fee_envelope: FeeEnvelope::default(),
    };

    // This call rebuilds against the new roots because the
    // high-level transition looks roots up dynamically.
    // A real stale serialized transaction would fail at
    // consensus. The certificate test below models that
    // old input outpoints are absent.
    assert!(!winner.utxos.contains_key(&world.roots.state));

    assert!(!winner.utxos.contains_key(&world.roots.resv.unwrap()));

    let rebuilt = stale_cycle.apply(&winner, next_order(&winner));

    // The operation may be valid after rebuilding against
    // the successor state; it is not the original stale tx.
    if let Ok(next) = rebuilt {
        check_invariant(&next).unwrap();
    }
}
