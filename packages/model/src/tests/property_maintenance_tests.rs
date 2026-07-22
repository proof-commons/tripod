//! Periodic maintenance property — sponsored quiescence after an
//! arbitrary busy trace.
//!
//! Implements `(´test:verification:property-maintenance´)` and
//! `(´rule:verification:maintenance-progress´)` (deterministic loop
//! rejection).
//!
//! Each action is materialized against the evolving world by the
//! state-aware seed driver. The property is non-vacuous: an eligible
//! world **must** fully discharge under the sponsored driver; a
//! blocked world must match its explicitly classified residuals. No
//! generic `if let Ok` escape hatch is permitted.

use super::property_strategy::*;
use super::test_fixtures;
use crate::*;
use proptest::prelude::*;

fn property_genesis() -> World {
    fund_property_world(
        &genesis(
            test_fixtures::constants(),
            test_fixtures::sat(1_000_000),
            CanonicalOrder {
                height: 0,
                tx_index: 0,
            },
            test_fixtures::txid(0),
        )
        .unwrap(),
        test_fixtures::sat(100_000),
    )
    .unwrap()
}

// ´test:verification:property-maintenance´

/// The property's whole body, over one stated busy trace.
///
/// It is a named function rather than an inline proptest body so that a
/// particular trace can be handed to it directly. A generator seed
/// replays a trace only by asking the same random number generator for
/// it again; a trace written down replays because it is written down.
fn maintenance_reaches_a_typed_outcome(
    seeds: Vec<PropertyActionSeed>,
) -> Result<(), TestCaseError> {
    let initial = property_genesis();

    let busy = drive_property_seed_trace(&initial, seeds);

    let before = lifecycle_report(&busy).unwrap();

    // Every world — eligible or blocked — must reach a fixpoint: the
    // driver makes all permissionless progress and never stalls on a
    // capacity-blocked request.
    let outcome = drive_sponsored_quiescence(&busy, 10_000)
        .map_err(|error| TestCaseError::fail(format!("sponsored quiescence failed: {error:?}")))?;

    prop_assert!(check_invariant(&outcome.world).is_ok());

    // The precondition classifier inspects the *initial* world and
    // names only the two blocking classes. The driver can neither
    // create nor clear a blocking residual (it never redeems to
    // restore headroom and never unseals a pool), so a world is
    // eligible exactly when the terminal outcome carries no blocking
    // residual. Non-blocking residuals (junk, refund-locked
    // requests) may remain even on an eligible world, so this is
    // strictly weaker than full discharge.
    let eligible = matches!(
        classify_quiescence_eligibility(&busy).unwrap(),
        QuiescenceEligibility::Eligible,
    );
    let has_blocking = outcome
        .residuals
        .contains(&QuiescenceResidual::SealedPoolWithPendingRequests)
        || outcome
            .residuals
            .contains(&QuiescenceResidual::ActiveBackingCapacityBlocked);
    prop_assert_eq!(eligible, !has_blocking);

    if outcome.is_fully_discharged() {
        prop_assert_eq!(outcome.report.admissible_requests, 0);
        prop_assert_eq!(outcome.report.entitlements, 0);
        prop_assert_eq!(outcome.report.live_distributions, 0);
        prop_assert!(outcome.report.ash_outputs <= 1);

        prop_assert!(outcome.report.entitlements <= before.entitlements);
        prop_assert!(outcome.report.live_distributions <= before.live_distributions);
    } else {
        prop_assert!(
            residuals_match_report(&outcome.world, &outcome.report, &outcome.residuals,).unwrap(),
            "residual classification did not match lifecycle report: \
             {:?} vs {:?}",
            outcome.residuals,
            outcome.report,
        );
    }

    Ok(())
}

proptest! {
    #![proptest_config(
        ProptestConfig {
            cases: 250,
            max_shrink_iters: 50_000,
            .. ProptestConfig::default()
        }
    )]

    #[test]
    fn maintenance_drives_every_world_to_a_typed_outcome(
        seeds in trace_strategy(),
    ) {
        maintenance_reaches_a_typed_outcome(seeds)?;
    }
}

/// The busy trace the maintenance property shrank to, written down as
/// the actions it is.
///
/// It used to be kept as a generator seed in the regression file beside
/// this suite, which stated nothing a reader could read and could have
/// been swapped for another value without any check noticing. The trace
/// it replays is twenty-three actions long and every field of every one
/// of them is here.
#[test]
fn the_shrunken_busy_trace_reaches_a_typed_outcome() {
    // One action of the shrunken trace. Every seed in it leaves the
    // second amount, the owner and the shape at zero.
    const fn action(selector: u8, amount_a: u64) -> PropertyActionSeed {
        PropertyActionSeed {
            selector,
            amount_a,
            amount_b: 0,
            owner_selector: 0,
            shape_selector: 0,
        }
    }

    let seeds = vec![
        action(13, 0),
        action(0, 355),
        action(4, 0),
        action(0, 99),
        action(4, 0),
        action(0, 2_251_799_813_685_246),
        action(4, 0),
        action(0, 3_683),
        action(0, 0),
        action(4, 0),
        action(1, 0),
        action(0, 9),
        action(4, 2_251_799_813_685_247),
        action(0, 65_535),
        action(4, 0),
        action(0, 2_251_799_813_685_246),
        action(3, 0),
        action(4, 0),
        action(0, 9),
        action(4, 63),
        action(1, 0),
        action(0, 63),
        action(4, 10_601),
    ];

    maintenance_reaches_a_typed_outcome(seeds)
        .expect("the shrunken trace still reaches a typed outcome");
}

// (´rule:verification:maintenance-progress´)
//
// A deliberately broken scheduler repeats an empty cycle, which is a
// valid transition that makes no maintenance progress. The driver must
// reject the loop immediately instead of silently consuming the step
// budget.

struct BrokenNoProgressScheduler;

impl MaintenanceScheduler for BrokenNoProgressScheduler {
    fn next_action(&self, _world: &World) -> Result<Option<MaintenanceAction>, Guard> {
        Ok(Some(MaintenanceAction::RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
        }))
    }

    fn fee_envelope_for(
        &self,
        _world: &World,
        _action: &MaintenanceAction,
    ) -> Result<FeeEnvelope, Guard> {
        Ok(FeeEnvelope::default())
    }
}

#[test]
fn broken_scheduler_loop_is_rejected_before_step_limit() {
    // Q = 0, so the repeated cycle is an empty cycle: valid, but no
    // maintenance progress.
    let world = test_fixtures::world();

    assert_eq!(
        drive_quiescence_with_scheduler(&world, &BrokenNoProgressScheduler, 10_000, true),
        Err(Guard::ZeroProgress),
    );
}

#[test]
fn deterministic_scheduler_discharges_eligible_fixture() {
    // Deterministic non-vacuity witness: the standard fixture is
    // eligible and must fully discharge.
    let world = test_fixtures::world();

    assert_eq!(
        classify_quiescence_eligibility(&world).unwrap(),
        QuiescenceEligibility::Eligible,
    );

    let outcome = drive_sponsored_quiescence(&world, 10_000).unwrap();

    check_invariant(&outcome.world).unwrap();

    assert!(outcome.is_fully_discharged());
    assert_eq!(outcome.report.admissible_requests, 0);
    assert_eq!(outcome.report.entitlements, 0);
    assert_eq!(outcome.report.live_distributions, 0);
    assert!(outcome.report.ash_outputs <= 1);
}
