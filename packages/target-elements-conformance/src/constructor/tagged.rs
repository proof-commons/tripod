//! The target's tagged hash, and the tags the constructor uses.
//!
//! # A tag is a domain, not a label
//!
//! Every hash the taproot rules take is prefixed by the double of a tag
//! hash, so a preimage accepted under one tag cannot be replayed under
//! another. Getting a tag wrong therefore does not produce a slightly
//! wrong answer that a test might tolerate: it produces a value from a
//! different domain entirely.
//!
//! # The tags are the target's, not Bitcoin's
//!
//! The reviewed target uses its own tag strings, which differ from the
//! upstream Bitcoin ones by an `/elements` suffix. They are transcribed
//! here from a reading of the target's own source, with the provenance
//! recorded beside each constant, because a constructor built on the
//! Bitcoin tags would compute a well-formed output program that the
//! target does not accept and no amount of internal consistency would
//! reveal it.

use sha2::{Digest, Sha256};

/// How many bytes one digest occupies.
pub const DIGEST_BYTES: usize = 32;

/// One 32-byte digest.
pub type Digest32 = [u8; DIGEST_BYTES];

/// The tag the target hashes a leaf under.
///
/// Provenance: `src/script/interpreter.cpp:550`,
/// `HASHER_TAPLEAF_ELEMENTS = TaggedHash("TapLeaf/elements")`.
pub const TAP_LEAF_TAG: &str = "TapLeaf/elements";

/// The tag the target hashes a branch under.
///
/// Provenance: `src/script/interpreter.cpp:551`,
/// `HASHER_TAPBRANCH_ELEMENTS = TaggedHash("TapBranch/elements")`.
pub const TAP_BRANCH_TAG: &str = "TapBranch/elements";

/// The tag the target hashes an output tweak under.
///
/// Provenance: `src/pubkey.cpp:246`,
/// `HASHER_TAPTWEAK_ELEMENTS = TaggedHash("TapTweak/elements")`.
pub const TAP_TWEAK_TAG: &str = "TapTweak/elements";

/// The tagged hash of `message` under `tag`.
///
/// The construction is the published one: the tag's own digest twice,
/// then the message. Prefixing with a fixed-width pair rather than the
/// tag's bytes is what makes the domain separation independent of the
/// tag's length.
#[must_use]
pub fn tagged_hash(tag: &str, message: &[u8]) -> Digest32 {
    let prefix = sha256(tag.as_bytes());
    let mut hasher = Sha256::new();
    hasher.update(prefix);
    hasher.update(prefix);
    hasher.update(message);
    hasher.finalize().into()
}

/// The plain digest of `message`.
#[must_use]
pub fn sha256(message: &[u8]) -> Digest32 {
    Sha256::digest(message).into()
}

/// The target's variable-width length prefix.
///
/// The leaf preimage frames its script with this, so a script and a
/// longer one that begins with it hash differently. Only the widths a
/// script can actually reach are written; anything larger is beyond
/// what a leaf can carry.
#[must_use]
pub fn compact_size(value: usize) -> Vec<u8> {
    if value < 0xfd {
        return vec![u8::try_from(value).unwrap_or(0)];
    }
    if value <= 0xffff {
        let mut bytes = vec![0xfd];
        bytes.extend_from_slice(&u16::try_from(value).unwrap_or(u16::MAX).to_le_bytes());
        return bytes;
    }
    let mut bytes = vec![0xfe];
    bytes.extend_from_slice(&u32::try_from(value).unwrap_or(u32::MAX).to_le_bytes());
    bytes
}
