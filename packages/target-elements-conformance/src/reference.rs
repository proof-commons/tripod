//! Reference vectors minted against the adopted reference
//! implementation.
//!
//! # Claim class
//!
//! Every value in this module is **reference-implementation
//! conformance, not independent evidence**. It was computed with the
//! `elements` and `secp256k1-zkp` crates, which bind the same C library
//! the node vendors in-tree. Agreement with them says the first-party
//! computation matches the target's own implementation; it does not say
//! either one matches the specification, because the two are not
//! independent opinions. The first-party oracles in
//! [`crate::constructor`] and [`crate::commitment_oracle`] retain the
//! independence claim unqualified, and nothing here replaces them
//! `(´[PLAN-rule:guide10:independent-oracles]´)`.
//!
//! # Why the constants live in the library and the checks live in tests
//!
//! The cross-checks need the first-party fixtures, which arrive through
//! test-only dependencies, so they are `#[cfg(test)]`. The *results*
//! they pin are durable artifacts a later wave reads, so they are here.
//!
//! # Everything below is public disposable test material
//!
//! The fixture internal key and the values derived from it belong to a
//! disposable development chain and authorize nothing
//! `(´[ADR015-rule:security:test-material]´)`.

/// How many bytes an x-only key or a digest occupies.
pub const REFERENCE_KEY_BYTES: usize = 32;

/// The merkle root of the fixture bundle's committed tree.
///
/// Computed first-party by `transaction::commit_tree` over the linked
/// fixture bundle, and confirmed byte-for-byte by folding every leaf's
/// path through the reference crate's own branch hashing. Pinned so
/// that a change to the tree fails a comparison here rather than
/// silently producing a different output key downstream.
pub const FIXTURE_MERKLE_ROOT: [u8; REFERENCE_KEY_BYTES] = [
    0xc0, 0x45, 0x45, 0x15, 0xc9, 0xe1, 0x50, 0x34, 0x28, 0x76, 0x15, 0xfd, 0xc2, 0xb4, 0x33, 0xca,
    0xd2, 0xd5, 0x29, 0xf3, 0x66, 0xe6, 0x90, 0x24, 0x75, 0xc6, 0xb7, 0x2a, 0xd3, 0x03, 0xa0, 0xb3,
];

/// The BIP-341-style taproot output key of the fixture instance.
///
/// # What this is
///
/// The x-only key obtained by tweaking the fixture bundle's internal
/// key (`vectors::bundle::INTERNAL_KEY`) by [`FIXTURE_MERKLE_ROOT`],
/// computed through `secp256k1-zkp` because the first-party
/// construction side deliberately does not compute it: `transaction`
/// owns the merkle root and refuses the curve arithmetic that would
/// turn it into an output key, which is the whole content of the
/// `PinnedOutputKeyUnverifiedAgainstTree` obligation.
///
/// # What this does NOT discharge
///
/// It does **not** discharge that obligation. Discharging it requires
/// the funding ceremony against a real node: an output actually created
/// at this program, and a spend the node accepts. What this pin buys is
/// narrower and still worth having — a reference value for the key, so
/// that a drift in tree construction changes a compared constant here
/// instead of surfacing for the first time as an unspendable output in
/// a later wave.
pub const FIXTURE_REFERENCE_OUTPUT_KEY: [u8; REFERENCE_KEY_BYTES] = [
    0x71, 0xc8, 0x38, 0xe9, 0x0d, 0xeb, 0x1c, 0x76, 0x55, 0x8a, 0x68, 0x8e, 0x8e, 0x5a, 0x08, 0xc6,
    0x94, 0xe9, 0xca, 0x95, 0x3d, 0x13, 0xb4, 0xb7, 0x84, 0xd6, 0xc4, 0x56, 0x69, 0x31, 0x75, 0xfe,
];

/// The parity of [`FIXTURE_REFERENCE_OUTPUT_KEY`]'s implicit y
/// coordinate, as the reference implementation reports it.
///
/// A control block carries this bit and an x-only program cannot state
/// it, so a spend that guesses it wrong fails. The fixture bundle pins
/// a parity of its own choosing rather than deriving one; the test
/// lane records whether the two agree instead of assuming they do.
pub const FIXTURE_REFERENCE_OUTPUT_KEY_PARITY_BIT: u8 = 1;
