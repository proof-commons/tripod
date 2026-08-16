//! The wide-floor relation's domain, and every bound it depends on.
//!
//! # What is fixed here
//!
//! The relation is stated over one domain and one limb base, and every
//! bound the target schedule relies on is a consequence of those two
//! choices (Guide-10 `rule:guide10:wide-floor-relation`,
//! `candidate:guide10:derived-limbs`). Writing them as constants rather
//! than as literals scattered through a builder is what lets a test
//! compare a measured maximum against the bound it is supposed to
//! satisfy, instead of against a number somebody retyped.
//!
//! # Why the bounds are a typed census
//!
//! Guide 10 refuses target emission for any intermediate without a bound
//! (`rule:guide10:bound-proof-stage`). A prose bound cannot be checked;
//! [`WideFloorBound`] is the same statement as a value, so the oracle
//! reports one observation per member and a test walks the census rather
//! than a hand-written list that could omit the one intermediate nobody
//! thought about.

/// The limb base's exponent.
///
/// Twenty-six, because the widest partial product is then
/// `a0·b0 < 2^26 · 2^26 = 2^52`, which is inside the target's signed
/// fixed-width arithmetic with room for every carry the normalization
/// adds (Guide-10 `rule:guide10:derived-limb-bounds`).
pub const LIMB_BASE_BITS: u32 = 26;

/// The limb base itself.
pub const LIMB_BASE: u64 = 1 << LIMB_BASE_BITS;

/// The semantic amount domain's exponent.
pub const AMOUNT_BITS: u32 = 51;

/// The exclusive bound on every semantic amount.
pub const AMOUNT_DOMAIN: u64 = 1 << AMOUNT_BITS;

/// The exclusive bound on a low limb.
pub const LOW_LIMB_BOUND: u64 = LIMB_BASE;

/// The exclusive bound on a high limb.
///
/// An amount below `2^51` has a high limb below `2^(51-26) = 2^25`, so
/// the two limbs are of different widths and a schedule that treated
/// them alike would admit a high limb the domain does not contain.
pub const HIGH_LIMB_BOUND: u64 = 1 << (AMOUNT_BITS - LIMB_BASE_BITS);

/// How many limbs a product of two semantic amounts occupies.
///
/// `a·b < 2^102 < 2^104 = B^4`, so four limbs are enough and no fifth
/// exists to be omitted.
pub const PRODUCT_LIMBS: usize = 4;

/// The exclusive bound on any value a signed fixed-width target
/// operation may hold.
///
/// The target's arithmetic is signed, so the representable maximum is
/// `2^63 - 1`. Every intermediate below is shown strictly under `2^53`,
/// which leaves the schedule's correctness independent of how close the
/// bound is.
pub const SIGNED_INTERMEDIATE_BOUND: u64 = 1 << 63;

/// One intermediate the schedule holds, and the bound it satisfies.
///
/// Each member names a value the target actually computes. The exponent
/// is exclusive: an observation `v` of a member with exponent `e`
/// satisfies its bound exactly when `v < 2^e`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum WideFloorBound {
    /// Any semantic amount: `a`, `b`, `q`, `d`, or `r`.
    Amount,
    /// Any low limb.
    LowLimb,
    /// Any high limb.
    HighLimb,
    /// The low-by-low partial product, `a0·b0` or `q0·d0`.
    LowPartialProduct,
    /// Either cross partial product, `a0·b1` or `a1·b0`.
    CrossPartialProduct,
    /// The high-by-high partial product, `a1·b1` or `q1·d1`.
    HighPartialProduct,
    /// The first normalization's dividend: the low partial product plus,
    /// on the quotient side, the remainder's low limb.
    FirstDividend,
    /// The carry the first normalization produces.
    FirstCarry,
    /// The middle normalization's dividend: both cross products, the
    /// first carry, and, on the quotient side, the remainder's high limb.
    MiddleDividend,
    /// The carry the middle normalization produces.
    SecondCarry,
    /// The top normalization's dividend: the high partial product and the
    /// second carry.
    TopDividend,
    /// The top product limb.
    TopLimb,
}

impl WideFloorBound {
    /// The complete census.
    pub const ALL: &'static [Self] = &[
        Self::Amount,
        Self::LowLimb,
        Self::HighLimb,
        Self::LowPartialProduct,
        Self::CrossPartialProduct,
        Self::HighPartialProduct,
        Self::FirstDividend,
        Self::FirstCarry,
        Self::MiddleDividend,
        Self::SecondCarry,
        Self::TopDividend,
        Self::TopLimb,
    ];

    /// The exclusive power of two the intermediate stays under.
    #[must_use]
    #[expect(
        clippy::match_same_arms,
        reason = "three unrelated intermediates happen to share an exponent, and merging their arms would hide which derivation produced which"
    )]
    pub const fn exclusive_exponent(self) -> u32 {
        match self {
            Self::Amount => AMOUNT_BITS,
            Self::LowLimb | Self::FirstCarry => LIMB_BASE_BITS,
            Self::HighLimb | Self::TopLimb => AMOUNT_BITS - LIMB_BASE_BITS,
            // `a0·b0 < 2^26 · 2^26`.
            Self::LowPartialProduct => 2 * LIMB_BASE_BITS,
            // `a0·b1 < 2^26 · 2^25`.
            Self::CrossPartialProduct => AMOUNT_BITS,
            // `a1·b1 < 2^25 · 2^25`.
            Self::HighPartialProduct => 2 * (AMOUNT_BITS - LIMB_BASE_BITS),
            // `a0·b0 + r0 < 2^52 + 2^26 < 2^53`.
            Self::FirstDividend | Self::MiddleDividend => 53,
            // `floor(2^53 / 2^26) < 2^27`.
            Self::SecondCarry => LIMB_BASE_BITS + 1,
            // `a1·b1 + carry1 < 2^50 + 2^27 < 2^51`.
            Self::TopDividend => AMOUNT_BITS,
        }
    }

    /// The exclusive bound itself.
    #[must_use]
    pub const fn exclusive_maximum(self) -> u64 {
        1_u64 << self.exclusive_exponent()
    }

    /// Why the bound holds, stated so a reader can check the arithmetic
    /// rather than trust the exponent.
    #[must_use]
    pub const fn derivation(self) -> &'static str {
        match self {
            Self::Amount => "the relation's stated domain",
            Self::LowLimb => "a Euclidean remainder modulo the base",
            Self::HighLimb => "an amount below 2^51 divided by 2^26",
            Self::LowPartialProduct => "2^26 times 2^26",
            Self::CrossPartialProduct => "2^26 times 2^25",
            Self::HighPartialProduct => "2^25 times 2^25",
            Self::FirstDividend => "2^52 for the product, plus 2^26 for a remainder limb",
            Self::FirstCarry => "2^53 divided by 2^26, and the dividend is under 2^52 + 2^26",
            Self::MiddleDividend => {
                "two cross products under 2^51 each, a carry under 2^26, and a remainder limb \
                 under 2^25"
            }
            Self::SecondCarry => "a dividend under 2^53, divided by 2^26",
            Self::TopDividend => "2^50 for the product, plus a carry under 2^27",
            Self::TopLimb => "a dividend under 2^51, divided by 2^26",
        }
    }
}
