//! Class-accounting corruption tests.
//!
//! Implements `´test:verification:receipt-class-accounting´`.

use super::corruption_fixtures::*;
use super::distribution_fixtures::mature_world;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn premature_t_to_l_relabel_breaks_pre_maturity_identity() {
    let world = test_fixtures::world();

    let time_locked = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];

    let old = *world.utxo(time_locked).unwrap();

    let corrupted = replace_utxo(
        &world,
        time_locked,
        Utxo {
            meta: Meta::Receipt {
                owner: GENESIS_OWNER,
                class: ReceiptClass::Live,
            },
            ..old
        },
    );

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::ReceiptAccountingPreMaturity),
    );
}

#[test]
fn post_maturity_time_locked_physical_receipt_is_accounted_live() {
    let world = mature_world(&test_fixtures::world());

    assert_eq!(world.state().unwrap().1.y_t, Sat::ZERO);

    assert_ne!(
        find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked),
        [] as [u64; 0]
    );

    check_invariant(&world).unwrap();
}
