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
    // Before destination elision: 3a4f2f086f8171ecfbfa9a5f2b2f8c720d3c18e5882d0b1f46f5a2c3c0b0acbe.
    0x53, 0x4b, 0x0b, 0xd7, 0xeb, 0x62, 0x74, 0xc2, 0x64, 0x78, 0xc9, 0x65, 0x4a, 0x1a, 0xd1, 0xc0,
    0x9a, 0x20, 0x2c, 0xcb, 0xc2, 0x1e, 0x9d, 0x21, 0x27, 0x7f, 0x90, 0xf2, 0x76, 0xd4, 0x8d, 0xf8,
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
/// # This is the fixture's pinned program, and a test says so
///
/// The fixture bundle pins this exact key as
/// `vectors::bundle::PINNED_PROGRAM`, stated there as a literal
/// because that package must not depend on this one — the vectors
/// package contract §16.2 fixes the direction, and this package
/// dev-depends on vectors, so the reverse edge would close a cycle.
/// The literal is therefore kept honest from this side: the
/// cross-check in the reference-oracle tests recomputes the key here
/// and asserts equality with the fixture's constant, which is what
/// makes the fixture pin derived rather than declared.
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
    // Before destination elision: 5a75951341de4ba06716242e047ca4ef5c60ce869f4a6378608df7450917bced.
    0x1b, 0x26, 0xef, 0x7f, 0x1f, 0xe5, 0x39, 0x71, 0xa7, 0x07, 0x0e, 0xc0, 0x3c, 0xd8, 0x92, 0xbf,
    0xcf, 0x67, 0x3e, 0xac, 0x35, 0x09, 0x46, 0x33, 0xec, 0x30, 0xab, 0x2b, 0xc6, 0x27, 0xc5, 0xf8,
];

/// The parity of [`FIXTURE_REFERENCE_OUTPUT_KEY`]'s implicit y
/// coordinate, as the reference implementation reports it.
///
/// A control block carries this bit and an x-only program cannot state
/// it, so a spend that guesses it wrong fails.
///
/// This is **even**, for the tree the fixture bundle links today. It is
/// derived with the key and moves with it: an earlier tree yielded an
/// odd key, and before that a revision of the fixture *declared* even
/// without deriving it and had its control blocks rejected by the
/// reference verifier for every leaf while every hash in the path was
/// right. That the value has now come back round to even is a
/// coincidence of the current tree and not a return to that mistake —
/// what makes it right is that the cross-check below recomputes it.
/// The fixture carries this bit as `vectors::bundle::PINNED_PARITY`,
/// and the cross-check asserts the two agree.
pub const FIXTURE_REFERENCE_OUTPUT_KEY_PARITY_BIT: u8 = 0;
