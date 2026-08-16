//! The wide-floor oracle, the independent normalizer, and the bounds.

use num_bigint::BigUint;

use crate::wide_floor::domain::{
    AMOUNT_DOMAIN, HIGH_LIMB_BOUND, LIMB_BASE, LOW_LIMB_BOUND, WideFloorBound,
};
use crate::wide_floor::normalizer::{limbs_of, staged_product, staged_quotient_side};
use crate::wide_floor::oracle::{AmountLimbs, WideFloorDefect, WideFloorInstance};

/// The boundary values Guide 10 requires every fixed vector to include
/// (`tbl:guide10:wide-floor-threats`).
fn boundary_values() -> Vec<u64> {
    vec![
        0,
        1,
        LIMB_BASE - 1,
        LIMB_BASE,
        LIMB_BASE + 1,
        HIGH_LIMB_BOUND - 1,
        HIGH_LIMB_BOUND,
        LOW_LIMB_BOUND - 1,
        AMOUNT_DOMAIN - 2,
        AMOUNT_DOMAIN - 1,
    ]
}

/// A deterministic spread over the domain, so the properties below are
/// not answered by ten hand-picked numbers alone.
///
/// A multiplicative walk rather than a random generator: the vectors are
/// the same on every run and on every machine, which is what makes a
/// failure reproducible without a recorded seed (Guide-10 §19).
fn generated_values(count: u64) -> Vec<u64> {
    let mut values = Vec::new();
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    for _ in 0..count {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        values.push(state % AMOUNT_DOMAIN);
    }
    values
}

#[test]
fn the_oracle_solves_the_relation_over_the_boundary_vectors() {
    for a in boundary_values() {
        for b in boundary_values() {
            for d in boundary_values() {
                if d == 0 {
                    assert_eq!(
                        WideFloorInstance::solve(a, b, d),
                        Err(WideFloorDefect::ZeroDivisor)
                    );
                    continue;
                }
                let Ok(instance) = WideFloorInstance::solve(a, b, d) else {
                    // The only refusal left is a quotient outside the
                    // domain, which is a stated limit rather than a
                    // defect: `a·b/d` can exceed `2^51`.
                    assert!(
                        u128::from(a) * u128::from(b) / u128::from(d) >= u128::from(AMOUNT_DOMAIN)
                    );
                    continue;
                };
                let witness = instance.witness();
                assert_eq!(
                    u128::from(a) * u128::from(b),
                    u128::from(witness.q) * u128::from(d) + u128::from(witness.r)
                );
                assert!(witness.r < d);
                assert!(instance.agrees_with_arbitrary_precision());
                assert!(instance.limbs_agree());
            }
        }
    }
}

#[test]
fn the_two_normalizations_represent_the_two_sides_of_the_relation() {
    for a in generated_values(64) {
        for b in [1, 3, LIMB_BASE + 7, AMOUNT_DOMAIN - 1] {
            for d in [1, 2, LIMB_BASE - 1, AMOUNT_DOMAIN - 1] {
                let Ok(instance) = WideFloorInstance::solve(a, b, d) else {
                    continue;
                };
                assert_eq!(instance.product().value(), u128::from(a) * u128::from(b));
                assert_eq!(instance.quotient_side().value(), instance.product().value());
                assert!(instance.limbs_agree());
            }
        }
    }
}

#[test]
fn the_fused_cascade_agrees_with_the_staged_one() {
    for a in generated_values(48) {
        for d in [1, 5, LIMB_BASE, AMOUNT_DOMAIN - 1] {
            let Ok(instance) = WideFloorInstance::solve(a, AMOUNT_DOMAIN - 3, d) else {
                continue;
            };
            let [_, _, q_limbs, d_limbs, r_limbs] = instance.amount_limbs();
            let staged = staged_quotient_side(q_limbs, d_limbs, r_limbs);
            assert_eq!(staged.sum_limbs(), instance.quotient_side().limbs());

            let (product_limbs, _) = staged_product(q_limbs, d_limbs);
            assert_eq!(&product_limbs, staged.product_limbs());
        }
    }
}

#[test]
fn digit_extraction_is_a_third_opinion_on_every_limb() {
    for a in generated_values(48) {
        for b in [2, LIMB_BASE + 1, AMOUNT_DOMAIN - 1] {
            for d in [1, 7, AMOUNT_DOMAIN - 1] {
                let Ok(instance) = WideFloorInstance::solve(a, b, d) else {
                    continue;
                };
                let product = BigUint::from(a) * BigUint::from(b);
                let extracted =
                    limbs_of(&product).expect("a product of two amounts has four limbs");
                assert_eq!(&extracted, instance.product().limbs());
                assert_eq!(&extracted, instance.quotient_side().limbs());
            }
        }
    }
}

#[test]
fn every_limb_recomposes_to_its_amount() {
    for value in boundary_values().into_iter().chain(generated_values(128)) {
        let limbs = AmountLimbs::of(value).expect("a boundary value is in domain");
        assert_eq!(limbs.recompose(), value);
        assert!(limbs.within_bounds());
        assert!(limbs.low() < LOW_LIMB_BOUND);
        assert!(limbs.high() < HIGH_LIMB_BOUND);
    }
}

#[test]
fn an_amount_at_the_domain_bound_has_no_limbs() {
    for value in [AMOUNT_DOMAIN, AMOUNT_DOMAIN + 1, u64::MAX] {
        assert_eq!(
            AmountLimbs::of(value),
            Err(WideFloorDefect::AmountOutOfDomain { value })
        );
    }
}

#[test]
fn the_oracle_refuses_an_operand_outside_the_domain() {
    for (a, b, d) in [
        (AMOUNT_DOMAIN, 1, 1),
        (1, AMOUNT_DOMAIN, 1),
        (1, 1, AMOUNT_DOMAIN),
    ] {
        assert!(matches!(
            WideFloorInstance::solve(a, b, d),
            Err(WideFloorDefect::AmountOutOfDomain { .. })
        ));
    }
}

#[test]
fn a_quotient_outside_the_domain_is_refused_rather_than_truncated() {
    // The largest product over the smallest nonzero divisor.
    let a = AMOUNT_DOMAIN - 1;
    assert!(matches!(
        WideFloorInstance::solve(a, a, 1),
        Err(WideFloorDefect::AmountOutOfDomain { .. })
    ));
}

#[test]
fn every_bound_in_the_census_holds_over_every_vector() {
    let mut worst = std::collections::BTreeMap::new();
    let vectors: Vec<u64> = boundary_values()
        .into_iter()
        .chain(generated_values(96))
        .collect();
    for a in &vectors {
        for b in &vectors {
            for d in [1_u64, 2, LIMB_BASE - 1, LIMB_BASE, AMOUNT_DOMAIN - 1] {
                let Ok(instance) = WideFloorInstance::solve(*a, *b, d) else {
                    continue;
                };
                for bound in WideFloorBound::ALL.iter().copied() {
                    let observed = instance.observed(bound);
                    assert!(
                        observed < bound.exclusive_maximum(),
                        "{bound:?} observed {observed}, bound {} ({})",
                        bound.exclusive_maximum(),
                        bound.derivation()
                    );
                    let entry = worst.entry(bound).or_insert(0_u64);
                    *entry = (*entry).max(observed);
                }
            }
        }
    }
    // Every member was exercised: a census entry no vector reaches is a
    // bound nothing checked.
    for bound in WideFloorBound::ALL {
        assert!(worst.contains_key(bound), "{bound:?} was never observed");
    }
}

#[test]
fn the_bound_census_is_a_set_and_its_exponents_are_representable() {
    let unique: std::collections::BTreeSet<WideFloorBound> =
        WideFloorBound::ALL.iter().copied().collect();
    assert_eq!(unique.len(), WideFloorBound::ALL.len());
    for bound in WideFloorBound::ALL.iter().copied() {
        assert!(bound.exclusive_exponent() < 63);
        assert!(!bound.derivation().is_empty());
    }
}

#[test]
fn the_witness_encodes_five_canonical_fixed_width_amounts() {
    let instance = WideFloorInstance::solve(123_456_789, 987_654_321, 1_000_003)
        .expect("an ordinary instance");
    let encoded = instance.witness().encode();
    assert_eq!(encoded.len(), 5);
    for item in &encoded {
        assert_eq!(item.len(), 8);
    }
    let witness = instance.witness();
    assert_eq!(encoded[0], witness.a.to_le_bytes().to_vec());
    assert_eq!(encoded[1], witness.b.to_le_bytes().to_vec());
    assert_eq!(encoded[2], witness.q.to_le_bytes().to_vec());
    assert_eq!(encoded[3], witness.d.to_le_bytes().to_vec());
    assert_eq!(encoded[4], witness.r.to_le_bytes().to_vec());
}
