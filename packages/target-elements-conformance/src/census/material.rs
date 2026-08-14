//! The public vectors and public test values the census is stated
//! against.
//!
//! # Disposable test-network material
//!
//! Everything here is public. The keys, points, scalars, and signatures
//! are published test vectors or values derived from published ones by
//! elementary public facts; the assets, amounts, identifiers, and
//! programs are arbitrary development values that name nothing and
//! authorize nothing on any network anyone uses. Under ADR-015's
//! disposable test-network material rule this is fixture data, not
//! secret material, and it must not be reused outside these tests
//! (Guide-9 §10.5, §17.2).
//!
//! No signing scalar appears. The one place a secret would be needed —
//! a signature over a transaction sighash — is deliberately absent
//! rather than supplied: that signature depends on the transaction the
//! executor materializes, so no static fixture can carry it, and the
//! census records the gap instead of inventing a key to close it.
//!
//! # Where the vectors come from
//!
//! - the digests are the published SHA-256 vectors;
//! - the signature vector is published BIP-340 test vector 0, of which
//!   only the public key, the message, and the signature are used;
//! - the curve points and scalars are the generator, the published
//!   scalar-multiplication vectors of the target's own public test
//!   suite, and the published BIP-341 key-tweak vectors;
//! - the malformed keys are the published invalid-key vectors.
//!
//! None of them is produced by this repository's code, which is the
//! point: an expectation the production encoder generated would be the
//! encoder agreeing with itself (Guide-9 §10.6, §14.1, §21.3).

/// One hexadecimal digit's value, refused at compile time if it is not
/// one.
const fn nibble(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        b'a'..=b'f' => digit - b'a' + 10,
        _ => panic!("a published vector must be written in lowercase hexadecimal"),
    }
}

/// A thirty-two byte vector, decoded at compile time.
const fn hex32(text: &[u8; 64]) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    let mut index = 0;
    while index < 32 {
        bytes[index] = (nibble(text[index * 2]) << 4) | nibble(text[index * 2 + 1]);
        index += 1;
    }
    bytes
}

/// A thirty-three byte vector, decoded at compile time.
const fn hex33(text: &[u8; 66]) -> [u8; 33] {
    let mut bytes = [0_u8; 33];
    let mut index = 0;
    while index < 33 {
        bytes[index] = (nibble(text[index * 2]) << 4) | nibble(text[index * 2 + 1]);
        index += 1;
    }
    bytes
}

/// A sixty-four byte vector, decoded at compile time.
const fn hex64(text: &[u8; 128]) -> [u8; 64] {
    let mut bytes = [0_u8; 64];
    let mut index = 0;
    while index < 64 {
        bytes[index] = (nibble(text[index * 2]) << 4) | nibble(text[index * 2 + 1]);
        index += 1;
    }
    bytes
}

/// One small scalar, most significant byte first.
const fn small_scalar(value: u8) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    bytes[31] = value;
    bytes
}

// -- Published SHA-256 vectors ---------------------------------------

/// The published digest of the empty message.
pub const SHA256_EMPTY: [u8; 32] =
    hex32(b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

/// The published digest of the three-byte message `abc`.
pub const SHA256_ABC: [u8; 32] =
    hex32(b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");

/// The published digest of the fifty-six byte two-block message.
pub const SHA256_TWO_BLOCK: [u8; 32] =
    hex32(b"248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");

/// The three-byte message of the second published vector.
pub const MESSAGE_ABC: &[u8] = b"abc";

/// The fifty-six byte message of the third published vector.
///
/// Fifty-six bytes is the length at which SHA-256's padding no longer
/// fits in the message's own block, so the digest exercises a second
/// compression the one-block vectors do not reach.
pub const MESSAGE_TWO_BLOCK: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

// -- Published BIP-340 signature vector -------------------------------

/// The public key of published signature vector zero.
pub const SIGNATURE_VECTOR_KEY: [u8; 32] =
    hex32(b"f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9");

/// The message of published signature vector zero.
pub const SIGNATURE_VECTOR_MESSAGE: [u8; 32] = [0_u8; 32];

/// The signature of published signature vector zero.
pub const SIGNATURE_VECTOR_SIGNATURE: [u8; 64] = hex64(
    b"e907831f80848d1069a5371b402410364bdf1c5f8307b0084c55f1ce2dba821525f66a4a85ea8b71e482a74f382d2ce5ebeee8fdb2172f477df4900d310536c0",
);

/// The signature vector's signature with its last byte flipped.
///
/// A non-empty signature that does not verify, which the reviewed
/// contract says aborts rather than pushing a false. Derived from the
/// published vector by an operation stated here rather than by signing
/// anything.
pub fn mutated_signature() -> Vec<u8> {
    let mut bytes = SIGNATURE_VECTOR_SIGNATURE.to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0xff;
    }
    bytes
}

// -- Published curve vectors ------------------------------------------

/// The compressed generator point.
pub const GENERATOR: [u8; 33] =
    hex33(b"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");

/// The scalar one, which multiplies the generator to itself.
pub const SCALAR_ONE: [u8; 32] = small_scalar(1);

/// The scalar ten of the published scalar-multiplication vector.
pub const SCALAR_TEN: [u8; 32] = small_scalar(10);

/// The published product of the generator and ten, an odd-parity point.
pub const GENERATOR_TIMES_TEN: [u8; 33] =
    hex33(b"03a0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7");

/// The scalar of the second published scalar-multiplication vector.
pub const SCALAR_VECTOR: [u8; 32] =
    hex32(b"e0f47c124f228b97bbdc0e4398aac9788869b9fbbc193d5323fdad9570609de6");

/// The published product of the generator and that scalar.
pub const GENERATOR_TIMES_SCALAR: [u8; 33] =
    hex33(b"032c0158d0f6df4881e99e65fbea21f27321d817f79ad39e08eaf4f16f1419bb0c");

/// A published scalar above the curve order, which no relation admits.
pub const SCALAR_ABOVE_ORDER: [u8; 32] =
    hex32(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364143");

/// A published thirty-three byte value whose x coordinate is not on the
/// curve.
pub const POINT_OFF_CURVE: [u8; 33] =
    hex33(b"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81700");

/// A published thirty-three byte value carrying an unknown prefix.
pub const KEY_UNKNOWN_PREFIX: [u8; 33] =
    hex33(b"0979be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");

// -- Published key-tweak vectors --------------------------------------

/// The internal key of the published tweak vector with an even result.
pub const TWEAK_EVEN_INTERNAL: [u8; 32] =
    hex32(b"93478e9488f956df2396be2ce6c5cced75f900dfa18e7dabd2428aae78451820");

/// The tweak of the published tweak vector with an even result.
pub const TWEAK_EVEN_TWEAK: [u8; 32] =
    hex32(b"e2cfc1c640f9a69605ac7550295cb9169ede0ad96a560abd6ec8dd76303f4f5c");

/// The tweaked key of the published tweak vector with an even result.
///
/// The parity byte is the published control block's own parity bit: the
/// vector states the tweaked key x-only, and the block says which of the
/// two points it is.
pub const TWEAK_EVEN_RESULT: [u8; 33] =
    hex33(b"0238e4e558c6c939d9ee0b953766d9f934969319dcc84561ab6e05eedd205f16f1");

/// The internal key of the published tweak vector with an odd result.
pub const TWEAK_ODD_INTERNAL: [u8; 32] =
    hex32(b"187791b6f712a8ea41c8ecdd0ee77fab3e85263b37e1ec18a3651926b3a6cf27");

/// The tweak of the published tweak vector with an odd result.
pub const TWEAK_ODD_TWEAK: [u8; 32] =
    hex32(b"ce08db99c02d29ccebcf4b7b4b7d88111a5f1eaa23a1216340876338f26976d2");

/// The tweaked key of the published tweak vector with an odd result.
pub const TWEAK_ODD_RESULT: [u8; 33] =
    hex33(b"03f8c9791817c6781858c5e7c8314cd36d2b0c22b0c138e756bfe3a6fd3e39bc00");

// -- Development transaction values -----------------------------------
//
// The census transaction states no asset, no outpoint, and no program:
// which asset a development network issues, which outputs an executor
// funded, and what program commits to the fixture's own leaf are all
// facts about a deployment rather than about the target. The values
// that *were* stated here for those fields have been removed rather
// than left unused, so that nothing reads as available material.
