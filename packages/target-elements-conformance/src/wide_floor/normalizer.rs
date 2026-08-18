//! An independent limb normalizer, written the way Guide 10 states it.
//!
//! # Why a second normalizer exists
//!
//! The oracle folds the remainder into the product normalization,
//! because the target then performs three divisions on the quotient side
//! instead of six. Guide 10 states that side in two stages instead:
//! normalize `q·d` into `u0..u3`, then add `r`'s limbs through a second
//! carry cascade `(´[PLAN-rule:guide10:remainder-addition]´)`. Reimplementing the
//! staged form here and comparing the two is how the fusion is checked
//! rather than argued `(´[PLAN-rule:guide10:limb-oracle-stage]´)`.
//!
//! Nothing here is used to build a target program. It exists to disagree
//! with the oracle if the oracle is wrong, which is a purpose a helper
//! shared with the builder could not serve.
//!
//! # A third opinion
//!
//! [`limbs_of`] extracts base-`B` digits from an arbitrary-precision
//! integer without any carry reasoning at all. Three implementations
//! that agree on the limbs of a value — the fused cascade, the staged
//! cascade, and plain digit extraction — is the comparison Guide 10 asks
//! stage A2 for.

use num_bigint::BigUint;
use num_traits::ToPrimitive;

use super::domain::{LIMB_BASE, PRODUCT_LIMBS};
use super::oracle::AmountLimbs;

/// The staged normalization of `q·d`, then `+ r`, exactly as stated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StagedNormalization {
    product_limbs: [u64; PRODUCT_LIMBS],
    product_carries: [u64; 2],
    sum_limbs: [u64; PRODUCT_LIMBS],
    sum_carries: [u64; 3],
}

impl StagedNormalization {
    /// The product limbs `u0..u3`, before the remainder is added.
    #[must_use]
    pub const fn product_limbs(&self) -> &[u64; PRODUCT_LIMBS] {
        &self.product_limbs
    }

    /// The two carries the product normalization propagates.
    #[must_use]
    pub const fn product_carries(&self) -> &[u64; 2] {
        &self.product_carries
    }

    /// The limbs `z0..z3` of `q·d + r`.
    #[must_use]
    pub const fn sum_limbs(&self) -> &[u64; PRODUCT_LIMBS] {
        &self.sum_limbs
    }

    /// The three carries the remainder addition propagates.
    #[must_use]
    pub const fn sum_carries(&self) -> &[u64; 3] {
        &self.sum_carries
    }
}

/// Normalizes `x·y` into four base-`B` limbs and two carries.
///
/// Written from the coefficient definitions rather than shared with the
/// oracle, so the two can disagree.
#[must_use]
pub const fn staged_product(x: AmountLimbs, y: AmountLimbs) -> ([u64; PRODUCT_LIMBS], [u64; 2]) {
    let coefficient0 = x.low() * y.low();
    let coefficient1 = x.low() * y.high() + x.high() * y.low();
    let coefficient2 = x.high() * y.high();

    let limb0 = coefficient0 % LIMB_BASE;
    let carry0 = coefficient0 / LIMB_BASE;

    let middle = coefficient1 + carry0;
    let limb1 = middle % LIMB_BASE;
    let carry1 = middle / LIMB_BASE;

    let top = coefficient2 + carry1;
    let limb2 = top % LIMB_BASE;
    let limb3 = top / LIMB_BASE;

    ([limb0, limb1, limb2, limb3], [carry0, carry1])
}

/// The complete staged quotient side: `q·d` normalized, then `r` added.
#[must_use]
pub const fn staged_quotient_side(
    quotient: AmountLimbs,
    divisor: AmountLimbs,
    remainder: AmountLimbs,
) -> StagedNormalization {
    let (product_limbs, product_carries) = staged_product(quotient, divisor);

    let sum0 = product_limbs[0] + remainder.low();
    let limb0 = sum0 % LIMB_BASE;
    let carry0 = sum0 / LIMB_BASE;

    let sum1 = product_limbs[1] + remainder.high() + carry0;
    let limb1 = sum1 % LIMB_BASE;
    let carry1 = sum1 / LIMB_BASE;

    let sum2 = product_limbs[2] + carry1;
    let limb2 = sum2 % LIMB_BASE;
    let carry2 = sum2 / LIMB_BASE;

    let limb3 = product_limbs[3] + carry2;

    StagedNormalization {
        product_limbs,
        product_carries,
        sum_limbs: [limb0, limb1, limb2, limb3],
        sum_carries: [carry0, carry1, carry2],
    }
}

/// The base-`B` digits of an arbitrary-precision value, least
/// significant first.
///
/// No carry reasoning at all: repeated division by the base. `None` when
/// the value needs more than the four limbs the domain admits, which is
/// itself a fact a caller may want to observe rather than a panic.
#[must_use]
pub fn limbs_of(value: &BigUint) -> Option<[u64; PRODUCT_LIMBS]> {
    let base = BigUint::from(LIMB_BASE);
    let mut remaining = value.clone();
    let mut limbs = [0_u64; PRODUCT_LIMBS];
    for limb in &mut limbs {
        *limb = (&remaining % &base).to_u64()?;
        remaining /= &base;
    }
    if remaining == BigUint::from(0_u32) {
        Some(limbs)
    } else {
        None
    }
}
