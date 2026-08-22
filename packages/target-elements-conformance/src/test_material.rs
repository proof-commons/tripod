//! The test-only owner-signing capability (Guide-13 §1.10, §14.3).
//!
//! # What this is for
//!
//! The canonical primitive census records a gap in as many words: it
//! contains no signature over a transaction sighash, because such a
//! signature depends on the transaction the executor materializes and
//! so cannot be a static fixture. Every owner-authorization case in
//! §9.2 and §19.3 is on the far side of that gap. This module is the
//! capability §14.3 lists as an input to target materialization, and
//! closing the gap is the whole of its purpose.
//!
//! # Every value here is a published fixture, and says so
//!
//! The scalars below are public disposable test-network material under
//! ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)`, admitted for exactly
//! this use by Guide-13 §1.10. They are the published BIP-340 test
//! vectors' own secret keys, which have been in the specification's
//! appendix since it was written. They authorize nothing on any network
//! anyone uses, they are reproducible from the specification rather
//! than generated here, and they must not be reused outside these
//! tests.
//!
//! # What this is not
//!
//! Guide-13 §1.10 refuses a first-party production interface for owner
//! private keys, production signing nonces, or wallet seeds, and this
//! module is not one. It is not the shape of one either: there is no
//! key generation, no randomness, no storage, no serialization of a
//! scalar, and no constructor that takes a secret from anywhere a
//! production secret could come from. [`OwnerSigningMaterial`] is built
//! from a scalar the caller has already published and refuses to hand
//! that scalar back out. Wiring a real key through it would require
//! publishing the key first, which is the point.
//!
//! Anything Guide-13 would need beyond that — a production signer, a
//! nonce policy, multi-party blinding — needs the separate ADR-015
//! future-secret design before it lands, and none of it is reachable
//! from here.
//!
//! # Why the signature is over a message this module is handed
//!
//! §1.7 requires the selected profile to be observed from the finalized
//! witness or recomputed from the exact signing request, and neither is
//! something a fixture can do for itself. The digest a signature
//! commits to is the target's construction, over a transaction only a
//! target-native run has. So this module signs a 32-byte message it is
//! given and takes no view on where the message came from: it closes
//! the "no signature is expressible" gap without inventing the sighash
//! the review has not reached.
//!
//! That division is also what makes the mutation cases honest. A
//! mutation case is one message signed and a different transaction
//! offered, and it is the target that decides the two disagree.
//!
//! # Why first-party arithmetic
//!
//! The same reason the constructor and confidential oracles give. The
//! target materializes signatures with the secp256k1 library the node
//! vendors in-tree, so a Rust binding to that library would make the
//! fixture and the thing checking it one opinion wearing two hats. What
//! this module needs is two scalar multiplications, three tagged
//! hashes, and arithmetic modulo the group order, all of it over
//! published constants, and [`crate::constructor::curve`] already
//! carries the point arithmetic.
//!
//! The check that the arithmetic is right is not internal consistency.
//! It is that this module reproduces the published BIP-340 vectors'
//! signatures byte for byte from their published secret keys and
//! auxiliary values, which is a comparison against an artifact nothing
//! in this repository produced.

use num_bigint::BigUint;
use num_traits::Zero as _;

use crate::constructor::curve::{
    CurvePoint, FIELD_ELEMENT_BYTES, generator, group_order, multiply_point,
};
use crate::constructor::tagged::{Digest32, tagged_hash};

/// The tag the signature scheme derives its nonce under.
///
/// Provenance: BIP-340's own tag, unsuffixed. The taproot *tree* tags
/// this target uses carry an `/elements` suffix and these do not: the
/// suffixed tags belong to the target's own taptree rules, where the
/// signature scheme is the upstream one the vendored library
/// implements. Getting this wrong would produce a well-formed signature
/// no verifier accepts, and the vector comparison is what catches it.
const NONCE_TAG: &str = "BIP0340/nonce";

/// The tag the auxiliary value is masked under.
const AUXILIARY_TAG: &str = "BIP0340/aux";

/// The tag the challenge is taken under.
///
/// Shared with [`crate::owner_key_oracle`], which needs the same
/// challenge to verify what this module signs. Sharing it is not the
/// oracle checking itself: what grounds both is that this module
/// reproduces the published vectors' signatures byte for byte and the
/// oracle accepts those same published signatures, so a wrong tag here
/// fails against the specification's own artifacts rather than being
/// agreed to twice.
pub(crate) const CHALLENGE_TAG: &str = "BIP0340/challenge";

/// How many bytes one signature occupies.
pub const SIGNATURE_BYTES: usize = 64;

/// Why a signing request cannot be served.
///
/// Each member names a condition of the scheme rather than a policy of
/// this module, so a caller reading one learns what the specification
/// refused rather than what this file decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TestSigningDefect {
    /// The scalar is zero or is not below the group order, so it is not
    /// a key.
    ScalarOutOfRange,
    /// The derived nonce reduced to zero.
    ///
    /// Vanishingly unlikely and stated anyway: the specification names
    /// the condition, and a signer that reached it and carried on would
    /// publish the scalar it was signing with.
    NonceOutOfRange,
    /// A scalar multiplication produced the identity, which has no
    /// x-only encoding.
    ///
    /// Unreachable for a scalar this type admits, and typed rather than
    /// asserted because "unreachable" is a claim about the range check
    /// above and not about the arithmetic below it.
    DegenerateResult,
}

/// One owner's published test signing material (§14.3).
///
/// The scalar is private and there is no accessor for it, no `Debug`
/// that could print it, and no serialization. That is not because it is
/// secret — it is published, and the fixture that supplies it says so
/// out loud — but because a type that handed scalars back would be the
/// beginning of the production interface §1.10 refuses, and the useful
/// time to not have that shape is before anybody needs it.
#[derive(Clone, PartialEq, Eq)]
pub struct OwnerSigningMaterial {
    scalar: BigUint,
    public_key: [u8; FIELD_ELEMENT_BYTES],
}

impl OwnerSigningMaterial {
    /// Build the material for one published scalar.
    ///
    /// # Errors
    ///
    /// [`TestSigningDefect::ScalarOutOfRange`] when the scalar is zero
    /// or at least the group order;
    /// [`TestSigningDefect::DegenerateResult`] when its multiple of the
    /// generator has no x-only encoding.
    pub fn from_published_scalar(
        scalar: &[u8; FIELD_ELEMENT_BYTES],
    ) -> Result<Self, TestSigningDefect> {
        let value = BigUint::from_bytes_be(scalar);

        if value.is_zero() || value >= *group_order() {
            return Err(TestSigningDefect::ScalarOutOfRange);
        }

        let public_key = x_only(&multiply_point(scalar, &generator()))?;

        Ok(Self {
            scalar: value,
            public_key,
        })
    }

    /// The x-only public key this material authorizes for.
    #[must_use]
    pub const fn x_only_public_key(&self) -> [u8; FIELD_ELEMENT_BYTES] {
        self.public_key
    }

    /// Sign one 32-byte message.
    ///
    /// The auxiliary value is a parameter rather than randomness. The
    /// scheme masks the scalar with it before deriving the nonce, and a
    /// value chosen here would make the signature unreproducible from
    /// the fixture's own stated inputs, which §1.10 requires of test
    /// material where the materializer permits it. It is also what lets
    /// the published vectors be reproduced exactly.
    ///
    /// # Errors
    ///
    /// [`TestSigningDefect::NonceOutOfRange`] when the derived nonce
    /// reduces to zero; [`TestSigningDefect::DegenerateResult`] when a
    /// scalar multiplication produces the identity.
    pub fn sign(
        &self,
        message: &[u8; FIELD_ELEMENT_BYTES],
        auxiliary: &[u8; FIELD_ELEMENT_BYTES],
    ) -> Result<[u8; SIGNATURE_BYTES], TestSigningDefect> {
        let order = group_order();

        // The scheme signs with whichever of the scalar and its
        // negation has an even-y public key, because the x-only
        // encoding names no other.
        let effective = self.even_y_scalar()?;

        let masked = xor(
            &scalar_bytes(&effective),
            &tagged_hash(AUXILIARY_TAG, auxiliary),
        );
        let mut nonce_preimage = Vec::with_capacity(3 * FIELD_ELEMENT_BYTES);
        nonce_preimage.extend_from_slice(&masked);
        nonce_preimage.extend_from_slice(&self.public_key);
        nonce_preimage.extend_from_slice(message);

        let nonce = BigUint::from_bytes_be(&tagged_hash(NONCE_TAG, &nonce_preimage)) % order;

        if nonce.is_zero() {
            return Err(TestSigningDefect::NonceOutOfRange);
        }

        let commitment = multiply_point(&scalar_bytes(&nonce), &generator());
        let commitment_x = x_only(&commitment)?;
        let nonce = if has_even_y(&commitment)? {
            nonce
        } else {
            order - nonce
        };

        let mut challenge_preimage = Vec::with_capacity(3 * FIELD_ELEMENT_BYTES);
        challenge_preimage.extend_from_slice(&commitment_x);
        challenge_preimage.extend_from_slice(&self.public_key);
        challenge_preimage.extend_from_slice(message);

        let challenge =
            BigUint::from_bytes_be(&tagged_hash(CHALLENGE_TAG, &challenge_preimage)) % order;
        let response = (nonce + challenge * effective) % order;

        let mut signature = [0_u8; SIGNATURE_BYTES];
        signature[..FIELD_ELEMENT_BYTES].copy_from_slice(&commitment_x);
        signature[FIELD_ELEMENT_BYTES..].copy_from_slice(&scalar_bytes(&response));
        Ok(signature)
    }

    /// The scalar whose public key has an even y coordinate.
    fn even_y_scalar(&self) -> Result<BigUint, TestSigningDefect> {
        let point = multiply_point(&scalar_bytes(&self.scalar), &generator());

        Ok(if has_even_y(&point)? {
            self.scalar.clone()
        } else {
            group_order() - &self.scalar
        })
    }
}

/// The x-only encoding of one point, or a typed defect for the
/// identity.
fn x_only(point: &CurvePoint) -> Result<[u8; FIELD_ELEMENT_BYTES], TestSigningDefect> {
    match point {
        CurvePoint::Affine(affine) => Ok(affine.x_only_bytes()),
        CurvePoint::Identity => Err(TestSigningDefect::DegenerateResult),
    }
}

/// Whether one point's y coordinate is even.
fn has_even_y(point: &CurvePoint) -> Result<bool, TestSigningDefect> {
    match point {
        CurvePoint::Affine(affine) => Ok(affine.has_even_y()),
        CurvePoint::Identity => Err(TestSigningDefect::DegenerateResult),
    }
}

/// One scalar as the scheme's 32 big-endian bytes.
///
/// Total for every value this module produces: each is reduced modulo
/// the group order, which is below `2^256`, so it never needs more than
/// thirty-two bytes and the leading ones stay zero.
pub(crate) fn scalar_bytes(value: &BigUint) -> [u8; FIELD_ELEMENT_BYTES] {
    let mut bytes = [0_u8; FIELD_ELEMENT_BYTES];
    let encoded = value.to_bytes_be();
    let offset = FIELD_ELEMENT_BYTES.saturating_sub(encoded.len());
    bytes[offset..].copy_from_slice(&encoded);
    bytes
}

/// The byte-wise exclusive or of two 32-byte values.
fn xor(left: &[u8; FIELD_ELEMENT_BYTES], right: &Digest32) -> [u8; FIELD_ELEMENT_BYTES] {
    let mut result = [0_u8; FIELD_ELEMENT_BYTES];

    for (slot, (left, right)) in result.iter_mut().zip(left.iter().zip(right.iter())) {
        *slot = left ^ right;
    }

    result
}
