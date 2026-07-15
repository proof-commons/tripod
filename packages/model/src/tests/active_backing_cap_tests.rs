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
