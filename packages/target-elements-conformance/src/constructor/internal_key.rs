//! The prototype's internal key, and why it is the one it is.
//!
//! # The requirement
//!
//! A constructor instance is spendable by its key path if anybody knows
//! the internal key's private scalar. The prototype's whole subject is
//! the script path, so the internal key must be a point nobody can
//! claim: not an operator key, not a release key, not a test signing
//! key, not a generated-and-discarded key, and not a deployment
//! parameter anybody can change
//! (Guide-10 `rule:guide10:internal-key`).
//!
//! # What is claimed, and what is not
//!
//! What is claimed: the constant below is a deterministic, published,
//! publicly recomputable point, and no private scalar for it is
//! recorded anywhere in this repository because none was ever
//! generated.
//!
//! What is *not* claimed: that no such scalar exists, or that none can
//! be found. That is a residual cryptographic assumption resting on the
//! difficulty of the discrete logarithm and on the derivation below
//! being a preimage-resistant hash of a fixed string. The prototype
//! does not prove discrete-log hardness and does not pretend to.

use crate::constructor::curve::FIELD_ELEMENT_BYTES;

/// The prototype's internal key: the published unspendable point.
///
/// # Derivation
///
/// This is the x coordinate of the point conventionally called `H`,
/// published in BIP-341 as the standard nothing-up-my-sleeve internal
/// key. Its x coordinate is the SHA-256 digest of the *uncompressed*
/// encoding of the curve's generator:
///
/// ```text
/// H_x = SHA256(0x04 || G_x || G_y)
/// ```
///
/// The uncompressed encoding, not the compressed one: the digest of
/// `0x02 || G_x` is a different value and not this point. The
/// distinction is recorded because it is invisible in the constant and
/// a reader reconstructing the derivation from memory can land on the
/// wrong one.
///
/// The derivation is recomputed and checked against this constant by a
/// test in this package, so the constant cannot drift from the rule
/// that produced it, and a reader need not take the derivation on
/// trust.
///
/// Anybody who could produce a private scalar for `H` would have found
/// a discrete logarithm of a hash output, which is the residual
/// assumption named in this module's documentation.
pub const UNSPENDABLE_INTERNAL_KEY: [u8; FIELD_ELEMENT_BYTES] = [
    0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
    0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
];
