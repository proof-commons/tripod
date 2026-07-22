//! Cycle tests.
//!
//! Implements `´test:verification:cycle´`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::shape::issuance_projection;
use crate::*;

fn admitted_world(principal: u64) -> World {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(principal), sat(1));

    admit_all_requests(&world)
}

#[test]
fn empty_cycle_advances_without_issuance() {
    let world = advance_blocks(&test_fixtures::world(), 100);

    let next = apply_checked(
        &world,
        &RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let old_state = world.state().unwrap().1;

    let new_state = next.state().unwrap().1;

    assert_eq!(new_state.cycle, old_state.cycle + 1);

    assert_eq!(new_state.q, Sat::ZERO);

    let certificate = next.history.transitions.last().unwrap();

    assert!(
        issuance_projection(certificate, Asset::U)
            .unwrap()
            .is_none()
    );

    assert!(
        issuance_projection(certificate, Asset::DistCtl)
            .unwrap()
            .is_none()
    );
}

#[test]
fn cycle_creates_distribution_for_nonzero_q() {
    let world = advance_blocks(&admitted_world(1_000), 100);

    let next = apply_checked(
        &world,
        &RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let cycle = next.state().unwrap().1.cycle;

    let control = find_distribution_control(&next, cycle);

    let control_view = read_distribution_control(next.utxo(control).unwrap()).unwrap();

    assert_eq!(control_view.principal, sat(1_000));

    let certificate = next.history.transitions.last().unwrap();

    assert_eq!(
        issuance_projection(certificate, Asset::DistCtl)
            .unwrap()
            .unwrap()
            .amount,
        Sat::ONE,
    );
}

#[test]
fn cadence_band_is_enforced() {
    let world = test_fixtures::world();

    assert_eq!(
        RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::CadenceTooEarly),
    );

    let operator_band = advance_blocks(&world, 10);

    assert_eq!(
        RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&operator_band, next_order(&operator_band)),
        Err(Guard::CadenceOperatorOnly),
    );
}

#[test]
fn maturity_cycle_converts_even_when_empty() {
    let mut world = test_fixtures::world();

    let announce = AnnounceMaturity {
        maturity_cycle: 10,
        signers: signers(&[OPERATOR_KEY]),
        fee_envelope: FeeEnvelope::default(),
    };

    world = apply_checked(&world, &announce, next_order(&world));

    for _ in 0..10 {
        world = advance_blocks(&world, 100);

        world = apply_checked(
            &world,
            &RunCycle {
                caller: CycleCaller::Anyone,
                operator_signers: SignerSet::new(),
                fee_envelope: FeeEnvelope::default(),
            },
            next_order(&world),
        );
    }

    let state = world.state().unwrap().1;

    assert_eq!(state.cycle, 10);

    assert_eq!(state.maturity, Maturity::Complete);

    assert_eq!(state.y_t, Sat::ZERO);
}

#[test]
fn maturity_lead_bounds_are_enforced() {
    let world = test_fixtures::world();

    assert_eq!(
        AnnounceMaturity {
            maturity_cycle: 9,
            signers: signers(&[OPERATOR_KEY]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::MaturityLeadTooShort),
    );

    assert_eq!(
        AnnounceMaturity {
            maturity_cycle: 1_001,
            signers: signers(&[OPERATOR_KEY]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::MaturityLeadTooLong),
    );
}

/// After maturity completion, a cycle with Q > 0 issues its entire
/// operator fee share live-class: the live/time-locked split (fee
/// share included) is bootstrapping-only.
#[test]
fn post_maturity_fee_share_is_entirely_live_class() {
    let mut world = test_fixtures::world();

    world = apply_checked(
        &world,
        &AnnounceMaturity {
            maturity_cycle: 10,
            signers: signers(&[OPERATOR_KEY]),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    for _ in 0..10 {
        world = run_forced_cycle(&world);
    }

    assert_eq!(world.state().unwrap().1.maturity, Maturity::Complete);

    // A fresh deposit gives the post-maturity cycle a positive Q.
    world = create_request_for(&world, ALICE, BOB, sat(1_000), sat(1));

    world = admit_all_requests(&world);

    assert!(!world.state().unwrap().1.q.is_zero());

    let next = run_forced_cycle(&world);

    let state = next.state().unwrap().1;

    assert_eq!(state.maturity, Maturity::Complete);
    assert_eq!(state.y_t, Sat::ZERO);

    // Every receipt created by this cycle — the operator fee share
    // included — is live-class.
    let certificate = next.history.transitions.last().unwrap();

    let mut created_receipts = 0;

    for outpoint in &certificate.created {
        if let Meta::Receipt { class, .. } = next.utxo(*outpoint).unwrap().meta {
            created_receipts += 1;

            assert_eq!(class, ReceiptClass::Live);
        }
    }

    assert!(created_receipts > 0, "cycle with Q > 0 issues receipts");

    check_invariant(&next).unwrap();
}

/// D1 regression, corruption-style: an announced maturity cycle that
/// is not strictly in the future violates the maturity clause and
/// reports its own reason, not the accounting clause's.
#[test]
fn announced_cycle_not_in_future_reports_maturity_coherence() {
    let world = test_fixtures::world();

    let (state_outpoint, state) = world.state().unwrap();

    let corrupted = super::corruption_fixtures::replace_utxo(
        &world,
        state_outpoint,
        Utxo {
            asset: Asset::Pid,
            value: Sat::ONE,
            meta: Meta::State(PoolState {
                maturity: Maturity::Announced { cycle: state.cycle },
                ..state
            }),
        },
    );

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::MaturityCoherence),
    );
}
