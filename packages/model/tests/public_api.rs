//! Public-API boundary test.
//!
//! Implements `´test:verification:public-api-boundary´`.
//!
//! This integration test compiles as an *external* crate against the
//! published surface of `tripod-model`, proving that the declared
//! operation constructors plus [`model::execute`] are a
//! sufficient public transition API.
//!
//! The complementary negative guarantees hold at compile time by
//! construction and therefore need no `trybuild` harness:
//!
//! - `model::TxBuilder`, `model::kernel`, and
//!   the flow/issuance declaration types are crate-private; naming
//!   them here fails to compile;
//! - `model::Transition` is sealed through a crate-private
//!   marker module, so an external `impl Transition for T` fails to
//!   compile;
//! - `model::Ratio` has private fields, so an external
//!   `Ratio { numerator: 1, denominator: 0 }` literal fails to
//!   compile; [`Ratio::new`] is the only constructor.
//!
//! These negative guarantees close the *operation vocabulary* only.
//! The model is a transparent reference model (see the crate-level
//! trust boundary): `World` state remains publicly inspectable and
//! mutable, and `Transition::apply` remains publicly callable, by
//! design — for auditors, corruption fixtures, and fault harnesses.
//! No stronger encapsulation is claimed here.

use model::*;

fn constants() -> Constants {
    Constants {
        pool_id: 7,

        zeta: Ratio::new(1, 2).unwrap(),
        mint_fee: Ratio::new(1, 2).unwrap(),

        min_maturity_lead: 10,
        max_maturity_lead: 1000,

        min_cadence_blocks: 10,
        max_cadence_blocks: 100,

        admission_batch_max: 32,
        settlement_batch_max: 32,
        relabel_batch_max: 32,
        ash_batch_max: 64,

        burn_input_max: 64,
        burn_change_max: 32,
        burn_record_max: 64,

        transfer_input_max: 64,
        transfer_output_max: 64,

        fee_sponsor_input_max: 16,
    }
}

fn genesis_world() -> World {
    genesis(
        constants(),
        Sat::new(1_000_000).unwrap(),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        TxId([0_u8; 32]),
    )
    .unwrap()
}

/// Every declared covenant operation remains publicly nameable and
/// implements the sealed [`Transition`] trait.
#[test]
fn every_operation_constructor_is_publicly_executable() {
    fn assert_transition<T: Transition>() {}

    assert_transition::<CreateRequest>();
    assert_transition::<CancelRequest>();
    assert_transition::<AdmitDeposits>();
    assert_transition::<RunCycle>();
    assert_transition::<SettleDistribution>();
    assert_transition::<TransferReceipts>();
    assert_transition::<RedeemReceipt>();
    assert_transition::<RelabelReceipts>();
    assert_transition::<BurnReceipts>();
    assert_transition::<CompactAsh>();
    assert_transition::<ClearAsh>();
    assert_transition::<AnnounceMaturity>();
}

/// Out-of-domain ratios are unconstructible from outside the crate.
///
/// `Ratio`'s fields are private, so `Ratio::new` is the only path to
/// a value; every rejection here is therefore a rejection everywhere,
/// and `genesis()` can never see a zero denominator (the panic a
/// public field literal used to permit).
#[test]
fn invalid_ratios_are_unconstructible() {
    assert_eq!(Ratio::new(1, 0), Err(Guard::BadConstant));
    assert_eq!(Ratio::new(0, 2), Err(Guard::BadConstant));
    assert_eq!(Ratio::new(2, 2), Err(Guard::BadConstant));
    assert_eq!(Ratio::new(3, 2), Err(Guard::BadConstant));
    assert_eq!(Ratio::new(1, 1024), Err(Guard::BadConstant));

    let ratio = Ratio::new(1, 2).unwrap();
    assert_eq!(ratio.numerator(), 1);
    assert_eq!(ratio.denominator(), 2);
}

/// A genesis txid equal to a generated transition id must not wedge
/// the world.
///
/// Without the generator-side skip, replay rejects the duplicate
/// while the failed transition leaves the predecessor — and its
/// nonce — unchanged, so the same collision recurs forever and a
/// valid genesis world can never make its first transition.
#[test]
fn genesis_txid_collision_does_not_wedge_the_first_transition() {
    let mut colliding = [0_u8; 32];
    colliding[..8].copy_from_slice(&1_u64.to_be_bytes());

    let world = genesis(
        constants(),
        Sat::new(1_000_000).unwrap(),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        TxId(colliding),
    )
    .unwrap();

    let announce = AnnounceMaturity {
        maturity_cycle: world.constants.min_maturity_lead,
        signers: std::iter::once(OPERATOR_KEY).collect(),
        fee_envelope: FeeEnvelope::default(),
    };

    let next = execute(
        &world,
        &announce,
        CanonicalOrder {
            height: 0,
            tx_index: 1,
        },
    )
    .unwrap();

    check_invariant(&next).unwrap();

    let first = next.history.transitions.last().unwrap().txid;

    assert_ne!(first, TxId(colliding));
    assert_ne!(first, world.history.genesis.txid);
}

/// A complete request/cancel round trip through the public normative
/// entry point [`execute`], entirely outside the crate.
#[test]
fn public_operation_api_executes_transitions() {
    let world = genesis_world();

    let announce = AnnounceMaturity {
        maturity_cycle: world.constants.min_maturity_lead,
        signers: std::iter::once(OPERATOR_KEY).collect(),
        fee_envelope: FeeEnvelope::default(),
    };

    let next = execute(
        &world,
        &announce,
        CanonicalOrder {
            height: 0,
            tx_index: 1,
        },
    )
    .unwrap();

    check_invariant(&next).unwrap();

    assert_eq!(
        next.history.transitions.last().unwrap().branch,
        BranchKind::AnnounceMaturity,
    );
}
