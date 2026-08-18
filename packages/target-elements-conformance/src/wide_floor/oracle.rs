//! The exact host reference for the wide-floor relation.
//!
//! # What this is for
//!
//! A wide-floor fixture states an exact witness and an exact verdict.
//! Something has to decide what `q` and `r` are, and it must not be the
//! target schedule: an expectation produced by the schedule builder
//! would agree with the schedule builder however wrong both were
//! `(´[PLAN-rule:guide10:independent-oracles]´)` and
//! `(´[PLAN-rule:guide10:wide-floor-host]´)`.
//!
//! So this computes the relation directly, in `u128`, from the
//! definition — `q = product / d`, `r = product % d` — and separately
//! reports every value the target schedule will hold on the way, so a
//! test can compare the two limb by limb and carry by carry rather than
//! only at the answer.
//!
//! # Fused normalization, and why it is the same normalization
//!
//! Guide 10 states the quotient side in two stages: normalize `q·d` into
//! limbs `u0..u3`, then add `r`'s limbs through a second carry cascade
//! `(´[PLAN-rule:guide10:remainder-addition]´)`. This module folds the two into
//! one cascade, adding `r0` to the first dividend and `r1` to the middle
//! one. Both compute the canonical base-`B` representation of the same
//! integer `q·d + r`, and that representation is unique, so the limbs
//! agree; what the fusion removes is three of the six divisions the
//! target would otherwise perform. The staged form is implemented
//! independently in [`super::normalizer`] and the two are compared, so
//! the equivalence is a checked property here rather than an assertion
//! `(´[PLAN-rule:guide10:limb-oracle-stage]´)`.
//!
//! # The oracle is not the witness
//!
//! Host-generated `q` and `r` are witnesses, not trusted answers
//! `(´[PLAN-rule:guide10:wide-floor-relation]´)`. [`WideFloorWitness`] is
//! therefore a plain record with no coherence requirement at all: a
//! mutation row states one the relation does not hold for, which is
//! exactly the case the target must refuse.

use num_bigint::BigUint;

use super::domain::{
    AMOUNT_DOMAIN, HIGH_LIMB_BOUND, LIMB_BASE, LOW_LIMB_BOUND, PRODUCT_LIMBS,
    SIGNED_INTERMEDIATE_BOUND, WideFloorBound,
};

/// How wide one semantic amount is, as the target carries it.
pub const AMOUNT_BYTES: usize = 8;

/// Why the oracle refuses to state an instance.
///
/// Every variant is a statement about the *inputs*, not about a target:
/// the oracle answers only for arguments the relation is defined over,
/// and a caller wanting a case outside them wants a witness rather than
/// an instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum WideFloorDefect {
    /// An amount is not below the stated domain.
    AmountOutOfDomain {
        /// The offending value.
        value: u64,
    },
    /// The divisor is zero, so no quotient exists.
    ZeroDivisor,
    /// An intermediate would not fit the target's signed fixed-width
    /// arithmetic, which would make the schedule unsound rather than
    /// merely wrong.
    IntermediateOverflow {
        /// The offending value.
        value: u64,
    },
}

/// One amount's two limbs.
///
/// The decomposition is `x = low + high·B`, which is exactly what the
/// target's Euclidean division by `B` produces: the remainder is the low
/// limb and the quotient is the high one. Nothing witnesses a limb, so
/// there is no limb bound for a caller to violate — the bounds below are
/// consequences of the amount's own domain
/// `(´[PLAN-candidate:guide10:derived-limbs]´)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AmountLimbs {
    low: u64,
    high: u64,
}

impl AmountLimbs {
    /// The limbs of one in-domain amount.
    ///
    /// # Errors
    ///
    /// [`WideFloorDefect::AmountOutOfDomain`] when the amount is not
    /// below `2^51`.
    pub const fn of(amount: u64) -> Result<Self, WideFloorDefect> {
        if amount >= AMOUNT_DOMAIN {
            return Err(WideFloorDefect::AmountOutOfDomain { value: amount });
        }
        Ok(Self {
            low: amount % LIMB_BASE,
            high: amount / LIMB_BASE,
        })
    }

    /// The low limb, below `2^26`.
    #[must_use]
    pub const fn low(self) -> u64 {
        self.low
    }

    /// The high limb, below `2^25`.
    #[must_use]
    pub const fn high(self) -> u64 {
        self.high
    }

    /// The amount the two limbs recompose to.
    #[must_use]
    pub const fn recompose(self) -> u64 {
        self.low + self.high * LIMB_BASE
    }

    /// Whether both limbs are inside the bounds the domain forces.
    #[must_use]
    pub const fn within_bounds(self) -> bool {
        self.low < LOW_LIMB_BOUND && self.high < HIGH_LIMB_BOUND
    }
}

/// One base-`B` normalization, with every value the target holds.
///
/// The three coefficients are the raw partial-product sums; the three
/// dividends are what the target actually divides; the carries are what
/// each division returns above its remainder; and the limbs are the
/// canonical answer. A test compares the target's stack against these
/// rather than against the answer alone, so a schedule that reached the
/// right limbs through a wrong carry is still a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedProduct {
    coefficients: [u64; 3],
    dividends: [u64; 3],
    carries: [u64; 2],
    limbs: [u64; PRODUCT_LIMBS],
}

impl NormalizedProduct {
    /// Normalizes `x·y + addend`, where the addend's limbs enter the
    /// first two dividends.
    ///
    /// The addend is the remainder on the quotient side and absent on
    /// the product side, which is the only difference between the two
    /// passes the target performs.
    fn of(x: AmountLimbs, y: AmountLimbs, addend: Option<AmountLimbs>) -> Self {
        let (low_addend, high_addend) = addend.map_or((0, 0), |limbs| (limbs.low, limbs.high));

        let low_product = x.low * y.low;
        let cross = x.low * y.high + x.high * y.low;
        let high_product = x.high * y.high;

        let first = low_product + low_addend;
        let (limb0, carry0) = (first % LIMB_BASE, first / LIMB_BASE);

        let middle = cross + high_addend + carry0;
        let (limb1, carry1) = (middle % LIMB_BASE, middle / LIMB_BASE);

        let top = high_product + carry1;
        let (limb2, limb3) = (top % LIMB_BASE, top / LIMB_BASE);

        Self {
            coefficients: [low_product, cross, high_product],
            dividends: [first, middle, top],
            carries: [carry0, carry1],
            limbs: [limb0, limb1, limb2, limb3],
        }
    }

    /// The raw partial-product coefficients, before any carry.
    #[must_use]
    pub const fn coefficients(&self) -> &[u64; 3] {
        &self.coefficients
    }

    /// What the target divides at each of the three normalization steps.
    #[must_use]
    pub const fn dividends(&self) -> &[u64; 3] {
        &self.dividends
    }

    /// The two carries the normalization propagates.
    #[must_use]
    pub const fn carries(&self) -> &[u64; 2] {
        &self.carries
    }

    /// The canonical base-`B` limbs, least significant first.
    #[must_use]
    pub const fn limbs(&self) -> &[u64; PRODUCT_LIMBS] {
        &self.limbs
    }

    /// The value the limbs represent.
    #[must_use]
    pub fn value(&self) -> u128 {
        let base = u128::from(LIMB_BASE);
        self.limbs.iter().rev().fold(0_u128, |accumulated, limb| {
            accumulated * base + u128::from(*limb)
        })
    }
}

/// One complete witness, as the target receives it.
///
/// Deliberately unconstrained. A witness is what a caller supplies, and
/// every mutation row in the threat matrix is a witness the relation
/// does not hold for `(´[PLAN-tab:guide10:wide-floor-threats]´)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WideFloorWitness {
    /// The first authenticated factor.
    pub a: u64,
    /// The second authenticated factor.
    pub b: u64,
    /// The claimed quotient.
    pub q: u64,
    /// The authenticated divisor.
    pub d: u64,
    /// The claimed remainder.
    pub r: u64,
}

impl WideFloorWitness {
    /// The witness stack, deepest item first.
    ///
    /// The order is the consumption order reversed: the schedule checks
    /// and packs from the top down, and it needs the divisor immediately
    /// below the remainder so that `r < d` is decidable without reaching
    /// past the third item `(´[PLAN-rule:guide10:stack-schedule]´)`.
    #[must_use]
    pub fn encode(&self) -> Vec<Vec<u8>> {
        [self.a, self.b, self.q, self.d, self.r]
            .iter()
            .map(|value| value.to_le_bytes().to_vec())
            .collect()
    }
}

/// One coherent instance of the relation, with every intermediate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WideFloorInstance {
    witness: WideFloorWitness,
    a_limbs: AmountLimbs,
    b_limbs: AmountLimbs,
    q_limbs: AmountLimbs,
    d_limbs: AmountLimbs,
    r_limbs: AmountLimbs,
    product: NormalizedProduct,
    quotient_side: NormalizedProduct,
}

impl WideFloorInstance {
    /// Solves the relation exactly for in-domain `a`, `b`, and `d`.
    ///
    /// # Errors
    ///
    /// [`WideFloorDefect::AmountOutOfDomain`] when an argument or the
    /// resulting quotient is not below `2^51`,
    /// [`WideFloorDefect::ZeroDivisor`] when the divisor is zero, and
    /// [`WideFloorDefect::IntermediateOverflow`] when any value the
    /// target would hold does not fit its signed fixed-width
    /// arithmetic.
    pub fn solve(first: u64, second: u64, divisor: u64) -> Result<Self, WideFloorDefect> {
        if divisor == 0 {
            return Err(WideFloorDefect::ZeroDivisor);
        }
        for value in [first, second, divisor] {
            if value >= AMOUNT_DOMAIN {
                return Err(WideFloorDefect::AmountOutOfDomain { value });
            }
        }

        // The exact relation, computed from its definition and from
        // nothing the target schedule does.
        let product = u128::from(first) * u128::from(second);
        let wide_divisor = u128::from(divisor);
        let wide_quotient = product / wide_divisor;
        let wide_remainder = product % wide_divisor;

        let quotient = u64::try_from(wide_quotient)
            .map_err(|_| WideFloorDefect::AmountOutOfDomain { value: u64::MAX })?;
        let remainder = u64::try_from(wide_remainder)
            .map_err(|_| WideFloorDefect::AmountOutOfDomain { value: u64::MAX })?;
        if quotient >= AMOUNT_DOMAIN {
            return Err(WideFloorDefect::AmountOutOfDomain { value: quotient });
        }

        let a_limbs = AmountLimbs::of(first)?;
        let b_limbs = AmountLimbs::of(second)?;
        let q_limbs = AmountLimbs::of(quotient)?;
        let d_limbs = AmountLimbs::of(divisor)?;
        let r_limbs = AmountLimbs::of(remainder)?;

        let instance = Self {
            witness: WideFloorWitness {
                a: first,
                b: second,
                q: quotient,
                d: divisor,
                r: remainder,
            },
            a_limbs,
            b_limbs,
            q_limbs,
            d_limbs,
            r_limbs,
            product: NormalizedProduct::of(a_limbs, b_limbs, None),
            quotient_side: NormalizedProduct::of(q_limbs, d_limbs, Some(r_limbs)),
        };

        for bound in WideFloorBound::ALL.iter().copied() {
            let observed = instance.observed(bound);
            if observed >= SIGNED_INTERMEDIATE_BOUND {
                return Err(WideFloorDefect::IntermediateOverflow { value: observed });
            }
        }

        Ok(instance)
    }

    /// The witness a spend would carry.
    #[must_use]
    pub const fn witness(&self) -> WideFloorWitness {
        self.witness
    }

    /// The exact quotient.
    #[must_use]
    pub const fn quotient(&self) -> u64 {
        self.witness.q
    }

    /// The exact remainder.
    #[must_use]
    pub const fn remainder(&self) -> u64 {
        self.witness.r
    }

    /// The limbs of each amount, in the order the schedule packs them.
    #[must_use]
    pub const fn amount_limbs(&self) -> [AmountLimbs; 5] {
        [
            self.a_limbs,
            self.b_limbs,
            self.q_limbs,
            self.d_limbs,
            self.r_limbs,
        ]
    }

    /// The normalization of `a·b`.
    #[must_use]
    pub const fn product(&self) -> &NormalizedProduct {
        &self.product
    }

    /// The normalization of `q·d + r`.
    #[must_use]
    pub const fn quotient_side(&self) -> &NormalizedProduct {
        &self.quotient_side
    }

    /// The greatest value of one bounded intermediate class this
    /// instance reaches, across both normalizations.
    ///
    /// This is the measurement a bound test compares against
    /// [`WideFloorBound::exclusive_maximum`], which is what makes the
    /// range proof machine-checked rather than asserted
    /// `(´[PLAN-rule:guide10:bound-proof-stage]´)`.
    #[must_use]
    pub fn observed(&self, bound: WideFloorBound) -> u64 {
        let sides = [&self.product, &self.quotient_side];
        let over = |pick: fn(&NormalizedProduct) -> u64| -> u64 {
            sides.iter().map(|side| pick(side)).max().unwrap_or(0)
        };
        match bound {
            WideFloorBound::Amount => [
                self.witness.a,
                self.witness.b,
                self.witness.q,
                self.witness.d,
                self.witness.r,
            ]
            .into_iter()
            .max()
            .unwrap_or(0),
            WideFloorBound::LowLimb => self
                .amount_limbs()
                .into_iter()
                .map(AmountLimbs::low)
                .max()
                .unwrap_or(0),
            WideFloorBound::HighLimb => self
                .amount_limbs()
                .into_iter()
                .map(AmountLimbs::high)
                .max()
                .unwrap_or(0),
            WideFloorBound::LowPartialProduct => over(|side| side.coefficients[0]),
            // The cross coefficient is the *sum* of the two cross
            // products, so the bound on one of them is read off the
            // larger half rather than off the sum.
            WideFloorBound::CrossPartialProduct => {
                let halves = [
                    self.a_limbs.low * self.b_limbs.high,
                    self.a_limbs.high * self.b_limbs.low,
                    self.q_limbs.low * self.d_limbs.high,
                    self.q_limbs.high * self.d_limbs.low,
                ];
                halves.into_iter().max().unwrap_or(0)
            }
            WideFloorBound::HighPartialProduct => over(|side| side.coefficients[2]),
            WideFloorBound::FirstDividend => over(|side| side.dividends[0]),
            WideFloorBound::MiddleDividend => over(|side| side.dividends[1]),
            WideFloorBound::TopDividend => over(|side| side.dividends[2]),
            WideFloorBound::FirstCarry => over(|side| side.carries[0]),
            WideFloorBound::SecondCarry => over(|side| side.carries[1]),
            WideFloorBound::TopLimb => over(|side| side.limbs[3]),
        }
    }

    /// Whether the two normalizations agree limb for limb.
    ///
    /// The relation the target proves, stated as the comparison the
    /// target performs.
    #[must_use]
    pub fn limbs_agree(&self) -> bool {
        self.product.limbs == self.quotient_side.limbs
    }

    /// The relation restated over arbitrary-precision arithmetic.
    ///
    /// A second opinion on the `u128` answer, so that a later change to
    /// the domain cannot silently outgrow the primitive width the oracle
    /// happens to use today `(´[PLAN-rule:guide10:wide-floor-host]´)`.
    #[must_use]
    pub fn agrees_with_arbitrary_precision(&self) -> bool {
        let product = BigUint::from(self.witness.a) * BigUint::from(self.witness.b);
        let expected = BigUint::from(self.witness.q) * BigUint::from(self.witness.d)
            + BigUint::from(self.witness.r);
        product == expected && BigUint::from(self.witness.r) < BigUint::from(self.witness.d)
    }
}
