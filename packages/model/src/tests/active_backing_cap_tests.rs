//! Active-backing cap tests.
//!
//! Implements `´test:verification:active-backing-cap´` and
//! `´def:domains:active-backing-cap´`.
//!
//! The cap Ω + Q <= ACTIVE_BACKING_MAX bounds simultaneously active
//! backing plus deposit escrow, not cumulative deposit volume:
//! redemptions create headroom for later admission. Open L-BTC in the
//! global environment is not capped; admission enforces headroom at
//! consumption.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

fn cap_genesis(headroom: u64) -> World {
    genesis(
        test_fixtures::constants(),
        Sat::new(ACTIVE_BACKING_MAX - headroom).unwrap(),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        test_fixtures::txid(0),
    )
    .unwrap()
}

fn admission_to_cap_fixture() -> World {
    let world = cap_genesis(100);

    let funded = create_request_for(&world, ALICE, BOB, sat(100), Sat::ONE);

    admit_all_requests(&funded)
}

#[test]
fn genesis_at_active_backing_cap_succeeds() {
    let world = cap_genesis(0);

    assert_eq!(
        world.active_resv().unwrap().1.value,
        Sat::new(ACTIVE_BACKING_MAX).unwrap(),
    );

    check_invariant(&world).unwrap();
}

#[test]
fn genesis_above_active_backing_cap_fails() {
    let reserve = Sat::new(ACTIVE_BACKING_MAX + 1).unwrap();

    assert_eq!(
        genesis(
            test_fixtures::constants(),
            reserve,
            CanonicalOrder {
                height: 0,
                tx_index: 0,
            },
            test_fixtures::txid(0),
        ),
        Err(Guard::ActiveBackingCapExceeded),
    );
}

#[test]
fn admission_to_exact_active_backing_cap_succeeds() {
    let admitted = admission_to_cap_fixture();

    assert_eq!(
        admitted.active_resv().unwrap().1.value,
        Sat::new(ACTIVE_BACKING_MAX).unwrap(),
    );

    assert_eq!(
        admitted.active_backing().unwrap(),
        Sat::new(ACTIVE_BACKING_MAX).unwrap(),
    );

    check_invariant(&admitted).unwrap();
}

#[test]
fn admission_over_active_backing_cap_fails_and_leaves_request() {
    let world = cap_genesis(100);

    let requested = create_request_for(&world, ALICE, BOB, sat(101), Sat::ONE);

    let request = find_request(&requested, BOB);

    let result = AdmitDeposits {
        requests: vec![request],
        admission_reward: Sat::ZERO,
        reward_owner: RELAYER,
    }
    .apply(&requested, next_order(&requested));

    assert_eq!(result, Err(Guard::ActiveBackingCapExceeded));

    assert!(requested.utxos.contains_key(&request));

    check_invariant(&requested).unwrap();
}

#[test]
fn cycle_preserves_active_backing_cap() {
    let admitted = admission_to_cap_fixture();

    let cycled = run_forced_cycle(&admitted);

    assert_eq!(
        cycled.active_resv().unwrap().1.value,
        Sat::new(ACTIVE_BACKING_MAX).unwrap(),
    );

    let state = cycled.state().unwrap().1;

    assert_eq!(
        state.omega.checked_add(state.q).unwrap(),
        Sat::new(ACTIVE_BACKING_MAX).unwrap(),
    );

    check_invariant(&cycled).unwrap();
}

#[test]
fn redemption_creates_active_backing_headroom() {
    let cycled = run_forced_cycle(&admission_to_cap_fixture());

    let y_l = cycled.state().unwrap().1.y_l;

    let (receipt, owner) = cycled
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::U,
                Meta::Receipt {
                    owner,
                    class: ReceiptClass::Live,
                },
            ) if utxo.value <= y_l => Some((*outpoint, owner)),

            _ => None,
        })
        .expect("redeemable live receipt");

    let before = cycled.active_resv().unwrap().1.value;

    let redeemed = RedeemReceipt {
        receipt,
        signers: signers(&[owner]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&cycled, next_order(&cycled))
    .unwrap();

    let after = redeemed.active_resv().unwrap().1.value;

    assert!(after < before);

    check_invariant(&redeemed).unwrap();
}

#[test]
fn oversized_open_lbtc_is_inert_until_admission() {
    let mut world = test_fixtures::world();

    let headroom = Sat::new(ACTIVE_BACKING_MAX)
        .unwrap()
        .checked_sub(world.active_resv().unwrap().1.value)
        .unwrap();

    // The principal exceeds remaining pool headroom by one unit; the
    // gross value adds a one-unit admission budget on top.
    let oversized_principal = headroom.checked_add(Sat::ONE).unwrap();

    let gross = oversized_principal.checked_add(Sat::ONE).unwrap();

    world.adversary.lbtc = gross;

    let (injected, request) = test_fixtures::inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        gross,
        Meta::DepositRequest {
            pool_id: world.constants.pool_id,
            refund_key: ALICE,
            receipt_owner: BOB,
            deposit_principal: oversized_principal,
        },
    );

    // The open request itself does not violate the pool.
    check_invariant(&injected).unwrap();

    // Admission rejects it when attempting to exceed active backing
    // capacity.
    assert_eq!(
        AdmitDeposits {
            requests: vec![request],
            admission_reward: Sat::ZERO,
            reward_owner: RELAYER,
        }
        .apply(&injected, next_order(&injected)),
        Err(Guard::ActiveBackingCapExceeded),
    );
}

// ´test:verification:admission-capacity-plan´
//
// The capacity plan separates locally valid requests from the canonical
// batch that currently fits active-backing headroom, and reports whether
// the full locally valid set could eventually be admitted. These tests
// pin each notion against a small explicit headroom.

fn request_principal(world: &World, outpoint: OutPoint) -> Sat {
    match world.utxos.get(&outpoint).expect("request utxo").meta {
        Meta::DepositRequest {
            deposit_principal, ..
        } => deposit_principal,

        _ => panic!("outpoint is not a deposit request"),
    }
}

#[test]
fn capacity_plan_admits_a_request_that_exactly_fits_headroom() {
    let world = create_request_for(&cap_genesis(100), ALICE, BOB, sat(100), Sat::ONE);

    let plan = admission_capacity_plan(&world).unwrap();

    assert!(plan.all_fit);
    assert_eq!(plan.locally_valid.len(), 1);
    assert_eq!(plan.fitting_batch.len(), 1);
}

#[test]
fn capacity_plan_rejects_a_request_one_above_headroom() {
    let world = create_request_for(&cap_genesis(100), ALICE, BOB, sat(101), Sat::ONE);

    let plan = admission_capacity_plan(&world).unwrap();

    assert!(!plan.all_fit);
    assert_eq!(plan.locally_valid.len(), 1);
    assert!(plan.fitting_batch.is_empty());

    let residuals = match classify_quiescence_eligibility(&world).unwrap() {
        QuiescenceEligibility::Residual(residuals) => residuals,
        QuiescenceEligibility::Eligible => panic!("expected an active-backing capacity residual"),
    };

    assert!(residuals.contains(&QuiescenceResidual::ActiveBackingCapacityBlocked));

    let report = lifecycle_report(&world).unwrap();

    assert!(residuals_match_report(&world, &report, &residuals).unwrap());
}

#[test]
fn capacity_plan_admits_a_fitting_subset_when_the_full_set_exceeds_headroom() {
    let mut world = create_request_for(&cap_genesis(100), ALICE, ALICE, sat(60), Sat::ONE);

    world = create_request_for(&world, BOB, BOB, sat(50), Sat::ONE);

    let plan = admission_capacity_plan(&world).unwrap();

    // 60 + 50 = 110 > 100: the full set does not fit, but a canonical
    // single-request batch does.
    assert!(!plan.all_fit);
    assert_eq!(plan.locally_valid.len(), 2);
    assert_eq!(plan.fitting_batch.len(), 1);
}

#[test]
fn capacity_plan_skips_an_oversized_request_and_keeps_smaller_ones() {
    let mut world = create_request_for(&cap_genesis(100), ALICE, ALICE, sat(110), Sat::ONE);

    world = create_request_for(&world, BOB, BOB, sat(40), Sat::ONE);

    world = create_request_for(&world, CAROL, CAROL, sat(50), Sat::ONE);

    let plan = admission_capacity_plan(&world).unwrap();

    // The 110-unit request never fits (110 > 100); 40 + 50 = 90 <= 100,
    // so both smaller requests are batched regardless of canonical order.
    assert!(!plan.all_fit);
    assert_eq!(plan.locally_valid.len(), 3);
    assert_eq!(plan.fitting_batch.len(), 2);

    let oversized = plan
        .locally_valid
        .iter()
        .copied()
        .find(|outpoint| request_principal(&world, *outpoint) == sat(110))
        .expect("oversized request is locally valid");

    assert!(!plan.fitting_batch.contains(&oversized));
}

#[test]
fn capacity_plan_bounds_the_fitting_batch_by_admission_batch_max() {
    let batch_max = test_fixtures::constants().admission_batch_max;

    let mut world = cap_genesis(1_000);

    for _ in 0..=batch_max {
        world = create_request_for(&world, ALICE, ALICE, sat(1), Sat::ONE);
    }

    let plan = admission_capacity_plan(&world).unwrap();

    // Every unit-principal request fits (total <= headroom), but the
    // batch is capped; the full-set proof is unaffected.
    assert!(plan.all_fit);
    assert_eq!(plan.locally_valid.len(), batch_max + 1);
    assert_eq!(plan.fitting_batch.len(), batch_max);
}

#[test]
fn malformed_and_capacity_blocked_requests_do_not_collapse() {
    let mut world = create_request_for(&cap_genesis(100), ALICE, BOB, sat(101), Sat::ONE);

    // A wrong-pool request is malformed junk: locally invalid, inert,
    // recoverable only by its refund key. It must not be confused with
    // the well-formed request blocked purely by capacity.
    world.adversary.lbtc = world.adversary.lbtc.checked_add(sat(5)).unwrap();

    let (world, _junk) = test_fixtures::inject_and_get_outpoint(
        &world,
        Asset::Lbtc,
        sat(5),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id + 1,
            refund_key: CAROL,
            receipt_owner: CAROL,
            deposit_principal: sat(2),
        },
    );

    let plan = admission_capacity_plan(&world).unwrap();

    // Only the well-formed oversized request is locally valid.
    assert_eq!(plan.locally_valid.len(), 1);
    assert!(!plan.all_fit);
    assert!(plan.fitting_batch.is_empty());

    let residuals = match classify_quiescence_eligibility(&world).unwrap() {
        QuiescenceEligibility::Residual(residuals) => residuals,
        QuiescenceEligibility::Eligible => panic!("expected a mixed residual set"),
    };

    assert!(residuals.contains(&QuiescenceResidual::ActiveBackingCapacityBlocked));
    assert!(residuals.contains(&QuiescenceResidual::UnderfundedLostRefundRequest));

    let report = lifecycle_report(&world).unwrap();

    assert!(residuals_match_report(&world, &report, &residuals).unwrap());
}
