//! Attestation bound conformance: the claims the containment and
//! seigniorage corrections rest on.
//!
//! Two classes of test live here, and they assert opposite things.
//!
//! The counterexamples pin *negative* results: a legal transition
//! sequence reaches a state that a withdrawn Attestation claim declared
//! impossible. They must keep passing, because the claim they refute
//! must not come back.
//!
//! The transition lemmas pin the *positive* per-step facts the
//! pre-maturity capacity proof telescopes. The proof's analytic steps
//! — the logarithm inequality and the deposit maximization — are
//! calculus, true independently of this protocol, and are proved in
//! the paper rather than re-verified here. What is protocol-specific
//! is exactly what is asserted below, and all of it is rational.
//!
//! Nothing here verifies the real-valued Attestation theorem. These are
//! integer-domain conformance tests: they establish that the
//! implemented transitions have the algebraic properties the proof
//! relies on, and that rounding moves in the conservative direction.

use super::advanced_fixtures::*;
use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

fn state_of(world: &World) -> PoolState {
    world.state().expect("state").1
}

/// The genesis floor factor is `1/(1-zeta)`. For `zeta = n/d` the
/// comparison `phi > 1/(1-zeta)` is exactly `omega*(d-n) > y*d`,
/// evaluated in `u128` so no rounding enters the assertion itself.
fn floor_over_genesis_factor(world: &World, multiple: u128) -> bool {
    let state = state_of(world);

    let zeta = world.constants.zeta;

    let numerator = u128::from(zeta.numerator());
    let denominator = u128::from(zeta.denominator());

    let omega = u128::from(state.omega.get());
    let y = u128::from(state.y().expect("y").get());

    omega * (denominator - numerator) > multiple * y * denominator
}

/// The time-locked slack `T - (1-zeta)*Y`, scaled by the split
/// denominator so it stays integral. Genesis makes it zero; the class
/// rules must never let it fall below zero before maturity.
fn time_locked_slack(world: &World) -> i128 {
    let state = state_of(world);

    let zeta = world.constants.zeta;

    let numerator = i128::from(zeta.numerator());
    let denominator = i128::from(zeta.denominator());

    let y_t = i128::from(state.y_t.get());
    let y = i128::from(state.y().expect("y").get());

    y_t * denominator - (denominator - numerator) * y
}

fn clear_all_ash(world: &World) -> World {
    let ash_inputs = find_ash(world);

    if ash_inputs.is_empty() {
        return world.clone();
    }

    apply_checked(
        world,
        &ClearAsh {
            ash_inputs,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(world),
    )
}

/// Only the principal reaches the pool; the budget covers admission
/// and does not enter `omega`, so the arithmetic below is in principal.
///
/// Issuance happens at the cycle boundary and settlement then routes
/// the minted receipts to their owners. Settlement moves no supply, so
/// the pool arithmetic is fixed by the cycle; it is run here only so a
/// burnable receipt exists.
fn deposit_and_cycle(world: &World, principal: Sat) -> World {
    let requested = create_request_for(world, ALICE, BOB, principal, sat(1_000));

    let cycled = run_forced_cycle(&admit_all_requests(&requested));

    let cycle = state_of(&cycled).cycle;

    let entitlements = find_all_entitlements_for_cycle(&cycled, cycle);

    if entitlements.is_empty() {
        return cycled;
    }

    settle_batch(&cycled, cycle, entitlements)
}

// ── Counterexamples ─────────────────────────────────────────────────

/// The withdrawn claim: `phi <= 1/(1-zeta)` throughout bootstrapping.
///
/// Before external deposits the bound does hold, and is attained by
/// burning the whole live class. One deposit and one further live burn
/// carry the floor above it while the pool is still pre-maturity, so
/// no deposit-independent bootstrapping ceiling exists.
#[test]
fn no_fixed_bootstrap_floor_ceiling() {
    let world = test_fixtures::world();

    // Burn the entire genesis live class. The floor reaches the
    // genesis factor exactly and does not pass it: this is the
    // pre-external-deposit regime, where the retained bound is true.
    let burned = burn_from_owner(&world, GENESIS_OWNER, sat(500_000), Vec::new());

    let settled = clear_all_ash(&burned);

    let state = state_of(&settled);

    assert_eq!(state.y_l, sat(0));
    assert_eq!(state.y_t, sat(500_000));
    assert_eq!(state.omega, sat(1_000_000));

    assert!(
        !floor_over_genesis_factor(&settled, 1),
        "before external deposits the genesis factor is a real bound"
    );

    // One external deposit. Issuance preserves the floor, so this step
    // alone changes nothing about the ratio.
    let deposited = deposit_and_cycle(&settled, sat(1_000_000));

    let after_deposit = state_of(&deposited);

    assert_eq!(after_deposit.omega, sat(2_000_000));
    assert_eq!(after_deposit.y_l, sat(250_000));
    assert_eq!(after_deposit.y_t, sat(750_000));

    // phi is exactly the genesis factor here: 2_000_000 / 1_000_000.
    assert!(!floor_over_genesis_factor(&deposited, 1));

    // Burning any of the newly minted live class now carries the floor
    // past the genesis factor, still before maturity.
    let burned_again = burn_from_owner(&deposited, BOB, sat(100_000), Vec::new());

    let settled_again = clear_all_ash(&burned_again);

    let final_state = state_of(&settled_again);

    // phi = 2_000_000 / 900_000 = 20/9, against a claimed ceiling of 2.
    assert_eq!(final_state.omega, sat(2_000_000));
    assert_eq!(final_state.y().expect("y"), sat(900_000));

    assert_eq!(final_state.maturity, Maturity::Unannounced);

    assert!(
        floor_over_genesis_factor(&settled_again, 1),
        "a pre-maturity state exists with phi above the genesis factor"
    );
}

/// The step the withdrawn lifetime envelope rested on: that
/// post-maturity burns settle "at worst" at the genesis factor.
///
/// Maturity converts the time-locked class, removing the supply floor
/// that bounded the ratio. The floor then passes any fixed multiple of
/// the genesis factor.
#[test]
fn post_maturity_floor_passes_any_fixed_multiple_of_the_genesis_factor() {
    // Relabelling leaves the converted class as its own receipt, so
    // the two live receipts are merged before a single large burn.
    let world = merge_live_receipts(
        &mature_and_relabel_all(&test_fixtures::world()),
        GENESIS_OWNER,
    );

    let state = state_of(&world);

    assert_eq!(state.y_t, sat(0), "maturity removes the supply floor");

    let burned = burn_from_owner(&world, GENESIS_OWNER, sat(900_000), Vec::new());

    let settled = clear_all_ash(&burned);

    assert!(
        floor_over_genesis_factor(&settled, 4),
        "post-maturity the floor is not bounded by the genesis factor"
    );
}

// ── Transition lemmas ───────────────────────────────────────────────

/// Settlement pinning is conservative: a burn is credited at the floor
/// committed by the most recent *preceding* clear, and because the
/// floor ratchets that never exceeds the floor prevailing when the
/// burn settles.
///
/// This is the step the capacity proof needs, and the only part of it
/// that depends on the protocol rather than on calculus.
#[test]
fn settlement_pinned_credit_never_exceeds_the_settling_floor() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100_000));

    let burned = burn_from_owner(
        &world,
        ALICE,
        sat(100_000),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100_000),
        }],
    );

    let settled = clear_all_ash(&burned);

    let indexer = ReferenceIndexer::from_assumed_kernel_history(
        &settled.history,
        &chain_view_for_world(&settled),
        [0_u8; 32],
    )
    .expect("indexer");

    let query = indexer.query(ADDRESS_A).expect("query");

    assert!(!query.terms.is_empty(), "the burn must credit something");

    let state = state_of(&settled);

    let omega_now = u128::from(state.omega.get());
    let y_now = u128::from(state.y().expect("y").get());

    for term in &query.terms {
        let term_omega = u128::try_from(&term.omega).expect("omega in domain");
        let term_y = u128::try_from(&term.y).expect("y in domain");

        // term_omega/term_y <= omega_now/y_now, by cross-multiplication.
        assert!(
            term_omega * y_now <= omega_now * term_y,
            "a pinned term valued a burn above the floor it settled at"
        );
    }
}

/// The class rules keep `T >= (1-zeta)*Y` before maturity. Deposits
/// preserve the slack exactly in the real-valued model; with integer
/// issuance the floors can only add to it, never subtract.
#[test]
fn deposits_never_reduce_the_time_locked_slack() {
    let world = test_fixtures::world();

    assert_eq!(time_locked_slack(&world), 0, "genesis starts at equality");

    let mut current = world;

    for _ in 0..3 {
        let before = time_locked_slack(&current);

        current = deposit_and_cycle(&current, sat(400_000));

        let after = time_locked_slack(&current);

        assert!(
            after >= before,
            "a deposit reduced the time-locked slack: {before} -> {after}"
        );

        assert!(after >= 0, "the class rules were violated");
    }
}

/// Live burns and live redemptions reduce supply without touching the
/// time-locked class, so both strictly increase the slack. This is the
/// other half of why the slack never goes negative before maturity.
#[test]
fn live_burns_increase_the_time_locked_slack() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100_000));

    let before = time_locked_slack(&world);

    let settled = clear_all_ash(&burn_from_owner(&world, ALICE, sat(100_000), Vec::new()));

    let after = time_locked_slack(&settled);

    assert!(after > before, "a live burn must increase the slack");

    assert_eq!(state_of(&settled).y_t, state_of(&world).y_t);
}
