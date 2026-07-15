//! Canonical-closure corruption tests.
//!
//! Implements `´test:verification:canonical-closure´`.

use super::corruption_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn duplicate_pace_root_is_rejected() {
    let world = test_fixtures::world();

    let corrupted = add_closed_utxo(
        &world,
        Utxo {
            asset: Asset::Pace,
            value: Sat::ONE,
            meta: Meta::Pace,
        },
    );

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::IdentityAuthority),
    );
}

#[test]
fn canonical_u_under_foreign_shape_is_rejected() {
    let world = test_fixtures::world();

    let corrupted = add_closed_utxo(
        &world,
        Utxo {
            asset: Asset::U,
            value: Sat::ONE,
            meta: Meta::ForeignShape(7),
        },
    );

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::CanonicalClosure),
    );
}

#[test]
fn canonical_ent_under_wrong_shape_is_rejected() {
    let world = test_fixtures::world();

    let corrupted = add_closed_utxo(
        &world,
        Utxo {
            asset: Asset::Ent,
            value: Sat::ONE,
            meta: Meta::Ash,
        },
    );

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::CanonicalClosure),
    );
}

#[test]
fn malformed_open_lbtc_remains_inert() {
    let mut world = test_fixtures::world();

    world.adversary.lbtc = sat(100);

    let next = inject_open_object(&world, Asset::Lbtc, sat(100), Meta::ForeignShape(8)).unwrap();

    check_invariant(&next).unwrap();
}
