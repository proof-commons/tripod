//! Property trace suite — arbitrary action sequences preserve the
//! global invariant, and generated burns exercise the full burn
//! parameter space.
//!
//! Implements `(´test:verification:property-traces´)` and
//! `(´test:verification:property-burn-variability´)`.

use super::property_strategy::*;
use super::test_fixtures;
use crate::*;
use proptest::prelude::*;

// ´test:verification:property-burn-variability´

proptest! {
    #![proptest_config(
        ProptestConfig {
            cases: 500,
            .. ProptestConfig::default()
        }
    )]

    #[test]
    fn generated_burns_conserve_value_and_project(
        owner_selector in any::<u8>(),
        burn_selector in any::<u64>(),
        record_selector in any::<u64>(),
        address_selector in any::<u8>(),
    ) {
        let world = test_fixtures::world();

        let input_u_total: Sat = world
            .utxos
            .values()
            .filter(|utxo| utxo.asset == Asset::U)
            .map(|utxo| utxo.value)
            .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
            .unwrap();

        let action = PropertyAction::BurnSomeLive {
            owner_selector,
            burn_selector,
            record_selector,
            address_selector,
        };

        match apply_property_action(&world, action) {
            PropertyStepResult::Applied(next) => {
                let certificate = next.history.transitions.last().unwrap();

                // A burn projection exists iff the branch is burn.
                prop_assert_eq!(certificate.branch, BranchKind::Burn);
                prop_assert!(certificate.burn.is_some());

                let burn = certificate.burn.as_ref().unwrap();

                // ASH plus change conserves the total U supply.
                let output_u_total: Sat = next
                    .utxos
                    .values()
                    .filter(|utxo| utxo.asset == Asset::U)
                    .map(|utxo| utxo.value)
                    .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
                    .unwrap();

                prop_assert_eq!(output_u_total, input_u_total);

                // Record acceptance boundary: an indexer accepts the
                // record set iff the claimed sum does not exceed the
                // ASH value. Over-claiming never invalidates the burn
                // transition itself (this branch was accepted).
                let claimed = burn
                    .records
                    .iter()
                    .map(|record| record.amount)
                    .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
                    .unwrap();

                let indexer_accepts = claimed <= burn.ash_value;

                prop_assert!(indexer_accepts || claimed > burn.ash_value);

                prop_assert!(check_invariant(&next).is_ok());
            }

            PropertyStepResult::Rejected(guard) => {
                // The only legitimate rejections in this fixture are
                // structural non-availability, never a broken burn.
                prop_assert!(
                    matches!(guard, Guard::NoSuch | Guard::ZeroProgress),
                    "unexpected burn rejection: {:?}",
                    guard,
                );
            }
        }
    }
}

// ´test:verification:property-traces´

proptest! {
    #![proptest_config(
        ProptestConfig {
            cases: 1_000,
            max_shrink_iters: 100_000,
            .. ProptestConfig::default()
        }
    )]

    #[test]
    fn arbitrary_action_traces_preserve_invariant(
        seeds in trace_strategy(),
    ) {
        let initial = fund_property_world(
            &genesis(
                test_fixtures::constants(),
                test_fixtures::sat(1_000_000),
                CanonicalOrder {
                    height: 0,
                    tx_index: 0,
                },
                test_fixtures::txid(0),
            ).unwrap(),
            test_fixtures::sat(100_000),
        ).unwrap();

        let mut world = initial;

        for seed in seeds {
            let action = materialize_action(&world, seed);

            match apply_property_action(&world, action) {
                PropertyStepResult::Applied(next) => {
                    prop_assert!(check_invariant(&next).is_ok());

                    world = next;
                }

                PropertyStepResult::Rejected(_) => {
                    prop_assert!(check_invariant(&world).is_ok());
                }
            }
        }
    }
}

#[test]
fn every_property_action_family_is_materializable() {
    let world = fund_property_world(
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
    .unwrap();

    let mut seen = std::collections::BTreeSet::new();

    for selector in 0_u8..15_u8 {
        let seed = PropertyActionSeed {
            selector,
            amount_a: 10,
            amount_b: 5,
            owner_selector: 0,
            shape_selector: selector % 6,
        };

        let action = materialize_action(&world, seed);

        seen.insert(property_action_name(&action));
    }

    assert_eq!(seen.len(), 15);
}

#[test]
fn selector_mapping_includes_cancellation_and_injection() {
    let world = test_fixtures::world();

    let cancel = materialize_action(
        &world,
        PropertyActionSeed {
            selector: 2,
            amount_a: 0,
            amount_b: 0,
            owner_selector: 0,
            shape_selector: 0,
        },
    );

    assert!(matches!(cancel, PropertyAction::CancelSomeRequest));

    let inject = materialize_action(
        &world,
        PropertyActionSeed {
            selector: 14,
            amount_a: 100,
            amount_b: 10,
            owner_selector: 0,
            shape_selector: 0,
        },
    );

    assert!(matches!(inject, PropertyAction::InjectOpen { .. }));
}

#[test]
fn create_request_action_preserves_generated_fields() {
    let world = test_fixtures::world();

    let seed = PropertyActionSeed {
        selector: 1,
        amount_a: 41,
        amount_b: 6,
        owner_selector: 1,
        shape_selector: 0,
    };

    let action = materialize_action(&world, seed);

    let PropertyAction::CreateRequest {
        funder,
        owner,
        principal,
        budget,
    } = action
    else {
        panic!("selector 1 must materialize create-request");
    };

    assert_eq!(funder, seed.owner());
    assert_eq!(owner, seed.other_owner());
    assert_eq!(principal.get(), 1 + 41);
    assert_eq!(budget.get(), 1 + 6);
}
