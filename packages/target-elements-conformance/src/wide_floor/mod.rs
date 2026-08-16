//! The exact wide-floor prototype: oracle, bounds, and target schedule.
//!
//! # The relation
//!
//! ```text
//! a·b = q·d + r,   0 <= r < d,   0 <= a,b,q < 2^51,   0 < d < 2^51
//! ```
//!
//! Those conditions imply `q = floor(a·b / d)`, and the target proof
//! establishes the complete relation rather than the floor alone
//! (Guide-10 `rule:guide10:wide-floor-relation`).
//!
//! # Three independent opinions
//!
//! [`oracle`] computes `q` and `r` in `u128` from the definition and
//! reports every intermediate the target will hold. [`normalizer`]
//! reimplements the limb cascade the way Guide 10 stages it, plus a
//! plain arbitrary-precision digit extraction that does no carry
//! reasoning at all. A target-native verdict is the fourth opinion and
//! belongs to a run rather than to this module
//! (`rule:guide10:independent-oracles`).
//!
//! # What is not here yet
//!
//! The target schedule. This module is the reference the schedule will
//! be compared against, and it is deliberately complete before any
//! instruction is emitted (`rule:guide10:arithmetic-oracle-stage`).

pub mod domain;
pub mod normalizer;
pub mod oracle;

pub use domain::{
    AMOUNT_BITS, AMOUNT_DOMAIN, HIGH_LIMB_BOUND, LIMB_BASE, LIMB_BASE_BITS, LOW_LIMB_BOUND,
    PRODUCT_LIMBS, SIGNED_INTERMEDIATE_BOUND, WideFloorBound,
};
pub use normalizer::{StagedNormalization, limbs_of, staged_product, staged_quotient_side};
pub use oracle::{
    AMOUNT_BYTES, AmountLimbs, NormalizedProduct, WideFloorDefect, WideFloorInstance,
    WideFloorWitness,
};
