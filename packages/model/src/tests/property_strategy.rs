//! Property-seed strategies for the trace and maintenance suites.
//!
//! Implements `´test:verification:property-seed-strategy´`.
//!
//! Amounts are full-width `u64` values biased toward useful small
//! amounts and critical boundaries, so Proptest can shrink cleanly
//! while still visiting near-ceiling arithmetic.

use crate::{PropertyActionSeed, TWO_51};
use proptest::prelude::*;

// ´test:verification:property-seed-strategy´

fn amount_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        6 => 0_u64..=65_536_u64,
        1 => Just(0),
        1 => Just(1),
        1 => Just(2),
        1 => Just(63),
        1 => Just(64),
        1 => Just(65_535),
        1 => Just(65_536),
        1 => Just(TWO_51 - 2),
        1 => Just(TWO_51 - 1),
    ]
}

pub fn action_seed_strategy() -> impl Strategy<Value = PropertyActionSeed> {
    (
        0_u8..15_u8,
        amount_strategy(),
        amount_strategy(),
        any::<u8>(),
        0_u8..6_u8,
    )
        .prop_map(
            |(selector, amount_a, amount_b, owner_selector, shape_selector)| PropertyActionSeed {
                selector,
                amount_a,
                amount_b,
                owner_selector,
                shape_selector,
            },
        )
}

pub fn trace_strategy() -> impl Strategy<Value = Vec<PropertyActionSeed>> {
    prop::collection::vec(action_seed_strategy(), 1..256)
}
