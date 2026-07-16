//! Property tests for adversarial open-asset injection.
//!
//! Implements `(´test:verification:property-open-asset-immunity´)`.
//!
//! The generated malformed open objects shrink to minimal
//! counterexamples: every accepted injection preserves the pool
//! invariant, and every rejected injection leaves the world unchanged.

use super::test_fixtures;
use crate::*;

use proptest::prelude::*;

// ´test:verification:property-open-asset-immunity´

proptest! {
    #![proptest_config(
        ProptestConfig {
            cases: 500,
            max_shrink_iters: 100_000,
            .. ProptestConfig::default()
        }
    )]

    #[test]
    fn arbitrary_open_injection_never_breaks_pool_invariant(
        amount in 0_u64..=1_000_000,
        shape_selector in 0_u8..6_u8,
        owner_selector in any::<u8>(),
    ) {
        let mut world = test_fixtures::world();

        world.adversary.lbtc = Sat::new(TWO_51 - 1).unwrap();

        world
            .adversary
            .foreign
            .insert(0xF001, Sat::new(TWO_51 - 1).unwrap());

        let seed = PropertyActionSeed {
            selector: 14,
            amount_a: amount,
            amount_b: amount / 2,
            owner_selector,
            shape_selector,
        };

        let action = materialize_action(&world, seed);

        let is_injection = matches!(action, PropertyAction::InjectOpen { .. });

        prop_assert!(is_injection);

        match apply_property_action(&world, action) {
            PropertyStepResult::Applied(next) => {
                prop_assert!(check_invariant(&next).is_ok());
            }

            PropertyStepResult::Rejected(_) => {
                prop_assert!(check_invariant(&world).is_ok());
            }
        }
    }
}
