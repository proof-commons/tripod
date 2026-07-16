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

proptest! {
    #![proptest_config(
        ProptestConfig {
            cases: 250,
            max_shrink_iters: 50_000,
            .. ProptestConfig::default()
        }
    )]

    #[test]
    fn maintenance_discharge_matches_declared_preconditions(
        seeds in trace_strategy(),
    ) {
        let initial = property_genesis();

        let busy = drive_property_seed_trace(&initial, seeds);

        let before = lifecycle_report(&busy).unwrap();

        match classify_quiescence_eligibility(&busy).unwrap() {
            QuiescenceEligibility::Eligible => {
                let (quiet, after) = drive_sponsored_quiescence(&busy, 10_000)
                    .map_err(|error| {
                        TestCaseError::fail(format!(
                            "eligible world failed sponsored quiescence: {error:?}"
                        ))
                    })?;

                prop_assert!(check_invariant(&quiet).is_ok());

                prop_assert_eq!(after.admissible_requests, 0);
                prop_assert_eq!(after.entitlements, 0);
                prop_assert_eq!(after.live_distributions, 0);
                prop_assert!(after.ash_outputs <= 1);

                prop_assert!(after.entitlements <= before.entitlements);
                prop_assert!(after.live_distributions <= before.live_distributions);
            }

            QuiescenceEligibility::Residual(residuals) => {
                prop_assert!(
                    residuals_match_report(&busy, &before, &residuals).unwrap(),
                    "residual classification did not match lifecycle report: \
                     {:?} vs {:?}",
                    residuals,
                    before,
                );
            }
        }
    }
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

    let (quiet, report) = drive_sponsored_quiescence(&world, 10_000).unwrap();

    check_invariant(&quiet).unwrap();

    assert_eq!(report.admissible_requests, 0);
    assert_eq!(report.entitlements, 0);
    assert_eq!(report.live_distributions, 0);
    assert!(report.ash_outputs <= 1);
}
