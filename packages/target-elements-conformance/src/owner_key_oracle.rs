//! The independent owner-key and signature oracle (Guide-13 §1.8).
//!
//! # The two questions the adapter cannot answer
//!
//! `tapscript` decides §1.8's first two negatives by walking the emitted
//! fragment: an empty key literal leaves the signature primitive no
//! successful form, and a key of an unrecognized width reopens the
//! target's forward-compatibility path. The remaining four keep the
//! approved encoding at its exact width, so no property of the emitted
//! bytes separates them from a correct leaf — a malformed key, another
//! owner's key, a signature under another key, and a signature over
//! another message are all, to an abstract walk, the same fragment.
//!
//! Deciding them needs curve arithmetic, and the adapter holds none and
//! takes no dependency that would give it any. It is right that it does
//! not: a curve library inside the package that emits the programs would
//! make the emitter and the thing checking it one opinion wearing two
//! hats. So the answers live here, over
//! [`crate::constructor::curve`]'s own first-party arithmetic.
//!
//! # What this discharges
//!
//! `OwnerKeyResidual::CurvePointMembership` says in as many words that
//! an accepted owner encoding is not a claim that the bytes are a point.
//! [`owner_key_point`] is where that claim is made or refused, so the
//! residual stops being an open question for any key an oracle has seen
//! and remains one only for keys nothing has checked.
//!
//! # What it does not establish
//!
//! Nothing here is target evidence. A signature this module verifies is
//! a signature over the thirty-two byte message it was handed, and §1.7
//! requires the message to be the digest of the finalized transaction
//! under the selected profile — which is the target's construction, over
//! a transaction only a target-native run has, under a profile whose
//! review is incomplete. This module closes the "no owner negative is
//! executable" gap. It does not close §1.11's.
//!
//! # Why the verification is written here rather than borrowed
//!
//! The same reason [`crate::constructor::curve`] gives. The target
//! verifies with the secp256k1 library the node vendors in-tree; a Rust
//! binding to that library would be the implementation checking itself.
//! What verification needs is two scalar multiplications, one point
//! addition, and one tagged hash over published constants, all of which
//! the curve module already carries — and the check that the arithmetic
//! is right is not internal consistency but that it accepts the
//! published BIP-340 vectors' own signatures and refuses every mutation
//! of them.

use num_bigint::BigUint;

use tapscript::{OwnerKey, OwnerKeyEncodingClosure, owner_key_encoding_closure};
use target_elements::{
    EncodingClass, EncodingSpec, PayloadWidth, ReviewedElementsTapscriptDefinition,
};

use crate::constructor::curve::{
    AffinePoint, CurvePoint, FIELD_ELEMENT_BYTES, PointDecodingDefect, add, generator, group_order,
    is_valid_scalar, lift_x, multiply_point,
};
use crate::constructor::tagged::tagged_hash;
use crate::test_material::{CHALLENGE_TAG, SIGNATURE_BYTES, scalar_bytes};

/// Why an offered owner key is not a point of the target's curve.
///
/// Width and curve membership are separate members because they are
/// separate findings: the first is a question §7.2's own gate answers
/// and the second is the one it explicitly does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerKeyPointDefect {
    /// The approved encoding fixes no single width to check against.
    ///
    /// A guard on the *target*, mirroring the one §7.2's gate carries:
    /// "wrong width" is meaningful only against an exact width, and an
    /// approved class that stopped fixing one would leave this oracle
    /// comparing against nothing.
    ApprovedEncodingFixesNoWidth {
        /// The width the approved class admits.
        width: PayloadWidth,
    },
    /// The offering is not the approved encoding's exact width.
    NotTheApprovedWidth {
        /// How many bytes were offered.
        offered: usize,
        /// How many the approved encoding fixes.
        required: usize,
    },
    /// The bytes are the right width and are not a curve point.
    NotACurvePoint {
        /// Which of the two ways it fails.
        defect: PointDecodingDefect,
    },
}

/// The exact width the approved owner-key encoding fixes.
const fn approved_width(closure: &OwnerKeyEncodingClosure) -> Result<usize, OwnerKeyPointDefect> {
    match closure.approved().v1_shape().payload() {
        PayloadWidth::Exact(width) => Ok(width.get()),
        width => Err(OwnerKeyPointDefect::ApprovedEncodingFixesNoWidth { width }),
    }
}

/// The point an offered owner key names, or why it names none.
///
/// The width is read from the reviewed contract's own approved encoding
/// rather than written down here, so an oracle and a gate cannot come to
/// disagree about what the approved width is.
///
/// # Errors
///
/// [`OwnerKeyPointDefect::ApprovedEncodingFixesNoWidth`] when the target
/// stops fixing one; [`OwnerKeyPointDefect::NotTheApprovedWidth`] for an
/// offering of another width, the empty one included; and
/// [`OwnerKeyPointDefect::NotACurvePoint`] for bytes of the right width
/// that no curve point has as an x coordinate.
pub fn offered_key_point(
    target: &ReviewedElementsTapscriptDefinition,
    offered: &[u8],
) -> Result<AffinePoint, OwnerKeyPointDefect> {
    let closure = owner_key_encoding_closure(target.definition().authorization());
    let required = approved_width(&closure)?;

    if offered.len() != required {
        return Err(OwnerKeyPointDefect::NotTheApprovedWidth {
            offered: offered.len(),
            required,
        });
    }

    let x = <[u8; FIELD_ELEMENT_BYTES]>::try_from(offered).map_err(|_| {
        // Reachable only if the approved encoding's width ever stops
        // being the curve's field width, which would make the whole
        // question a different one. Reported as a width mismatch rather
        // than asserted away.
        OwnerKeyPointDefect::NotTheApprovedWidth {
            offered: offered.len(),
            required: FIELD_ELEMENT_BYTES,
        }
    })?;

    lift_x(&x).map_err(|defect| OwnerKeyPointDefect::NotACurvePoint { defect })
}

/// The point one accepted owner key names.
///
/// The residual §7.2 leaves open, discharged. An [`OwnerKey`] has
/// already passed the encoding gate, so the only question left is the
/// one the gate said it could not answer.
///
/// # Errors
///
/// Any failure of [`offered_key_point`], which for a value of this type
/// is [`OwnerKeyPointDefect::NotACurvePoint`] unless the reviewed
/// contract's approved encoding has changed underneath it.
pub fn owner_key_point(
    target: &ReviewedElementsTapscriptDefinition,
    owner: &OwnerKey,
) -> Result<AffinePoint, OwnerKeyPointDefect> {
    offered_key_point(target, owner.bytes())
}

/// Why an offered owner signature does not verify.
///
/// Each member names a condition of the scheme rather than a policy of
/// this module, so a caller reading one learns what the specification
/// refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignatureRejection {
    /// The offering is not the signature encoding's exact width.
    NotTheSignatureWidth {
        /// How many bytes were offered.
        offered: usize,
        /// How many the reviewed encoding fixes.
        required: usize,
    },
    /// The reviewed signature encoding fixes no single width.
    SignatureEncodingFixesNoWidth {
        /// The width it admits.
        width: PayloadWidth,
    },
    /// The key the signature is required to verify against is not a
    /// curve point.
    KeyIsNotACurvePoint {
        /// Why not.
        defect: OwnerKeyPointDefect,
    },
    /// The signature's commitment half is not a curve point.
    CommitmentIsNotACurvePoint {
        /// Why not.
        defect: PointDecodingDefect,
    },
    /// The signature's response half is not below the group order.
    ResponseOutOfRange,
    /// The verification equation does not hold.
    ///
    /// The one that carries the content: the signature is well formed,
    /// the key is a point, and this signature was not taken by that key
    /// over that message.
    CommitmentMismatch,
}

/// The exact width the reviewed signature encoding fixes.
fn signature_width(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<usize, SignatureRejection> {
    match target
        .definition()
        .encodings()
        .get(&EncodingClass::SchnorrSignature)
        .map_or(PayloadWidth::Absent, EncodingSpec::payload)
    {
        PayloadWidth::Exact(width) => Ok(width.get()),
        width => Err(SignatureRejection::SignatureEncodingFixesNoWidth { width }),
    }
}

/// Whether one signature verifies for one key over one message.
///
/// The verification equation is stated as `s·G = R + e·P`, with `R`
/// recovered from the signature's commitment half by the target's own
/// x-only rule. That is the specification's `R' = s·G − e·P` with the
/// terms rearranged so no point negation is needed: recovering `R`
/// through the even-y lift already fixes both the parity the
/// specification checks and the x coordinate it compares.
///
/// # Errors
///
/// [`SignatureRejection::NotTheSignatureWidth`] and
/// [`SignatureRejection::SignatureEncodingFixesNoWidth`] for an offering
/// the reviewed encoding does not admit;
/// [`SignatureRejection::KeyIsNotACurvePoint`] and
/// [`SignatureRejection::CommitmentIsNotACurvePoint`] for a key or
/// commitment that names no point;
/// [`SignatureRejection::ResponseOutOfRange`] for a response at or above
/// the group order; and [`SignatureRejection::CommitmentMismatch`] when
/// the equation does not hold.
pub fn verify_owner_signature(
    target: &ReviewedElementsTapscriptDefinition,
    public_key: &[u8],
    message: &[u8; FIELD_ELEMENT_BYTES],
    signature: &[u8],
) -> Result<(), SignatureRejection> {
    let required = signature_width(target)?;
    if signature.len() != required || signature.len() != SIGNATURE_BYTES {
        return Err(SignatureRejection::NotTheSignatureWidth {
            offered: signature.len(),
            required,
        });
    }

    let point = offered_key_point(target, public_key)
        .map_err(|defect| SignatureRejection::KeyIsNotACurvePoint { defect })?;

    let (commitment, response) = signature.split_at(FIELD_ELEMENT_BYTES);
    let commitment = half(commitment);
    let response = half(response);

    let commitment_point = lift_x(&commitment)
        .map_err(|defect| SignatureRejection::CommitmentIsNotACurvePoint { defect })?;
    if !is_valid_scalar(&response) {
        return Err(SignatureRejection::ResponseOutOfRange);
    }

    let mut preimage = Vec::with_capacity(3 * FIELD_ELEMENT_BYTES);
    preimage.extend_from_slice(&commitment);
    preimage.extend_from_slice(&point.x_only_bytes());
    preimage.extend_from_slice(message);
    let challenge = BigUint::from_bytes_be(&tagged_hash(CHALLENGE_TAG, &preimage)) % group_order();

    let left = multiply_point(&response, &generator());
    let right = add(
        &CurvePoint::Affine(commitment_point),
        &multiply_point(&scalar_bytes(&challenge), &point),
    );

    if left == right {
        Ok(())
    } else {
        Err(SignatureRejection::CommitmentMismatch)
    }
}

/// One thirty-two byte half of a signature.
///
/// Total for every caller: the width was checked before the split, and
/// each half of a sixty-four byte offering is thirty-two bytes.
fn half(bytes: &[u8]) -> [u8; FIELD_ELEMENT_BYTES] {
    let mut half = [0_u8; FIELD_ELEMENT_BYTES];
    let width = half.len().min(bytes.len());
    half[..width].copy_from_slice(&bytes[..width]);
    half
}
