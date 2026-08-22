//! Public-point arithmetic on the curve the target's taproot rules use.
//!
//! # Public data only, by construction
//!
//! Nothing here holds or derives a secret scalar. The scalars this
//! module multiplies by are a taproot tweak, which is a hash of public
//! data that anybody verifying the output recomputes
//! `(´[PLAN-rule:guide10:public-data]´)`, and — since Guide-13 §14.3
//! asked for a test-only owner-signing capability — a signing scalar
//! handed in by [`crate::test_material`], which publishes it. There is
//! no key generation and no nonce here.
//!
//! # The property that has to hold, and the one that only looked like it
//!
//! What matters is that no *secret* scalar reaches this arithmetic, and
//! that is unchanged: the taproot tweak is public by construction, and
//! the signing scalar is a published fixture under ADR-015's
//! test-material rule, labelled test-only at its own definition and
//! authorizing nothing on any network. Guide-13 §1.10 forbids a
//! first-party production interface for owner private keys, and none
//! exists — there is no route by which a production secret could arrive
//! at this function, because there is no production caller that holds
//! one.
//!
//! The earlier wording said the tweak was the only scalar multiplied
//! here, and stating the *consequence* rather than the property was
//! what made the sentence go stale the moment a second public scalar
//! appeared. So it is stated as the property now, and the property is
//! also why the implementation stays deliberately unhurried and plainly
//! written rather than constant-time: there is no secret whose timing
//! could leak. A module that ever needed to be constant-time would be a
//! different module with a different review, and this one would have to
//! stop existing rather than grow that capability quietly.
//!
//! # Why first-party rather than a curve library
//!
//! The oracle exists to be *independent* of the implementation it
//! checks `(´[PLAN-rule:guide10:independent-oracles]´)`. It is written
//! from the published curve parameters and the standard affine
//! formulas, using only arithmetic the workspace already carries. A
//! library shared with the thing under test would weaken exactly the
//! property the oracle is for.
//!
//! # What is not claimed
//!
//! No claim is made that a point here has no known discrete logarithm,
//! that the arithmetic resists any side channel, or that this module is
//! suitable for anything but recomputing public taproot values.

use std::sync::LazyLock;

use num_bigint::BigUint;
use num_traits::{One, Zero};

/// The field the curve is defined over, `2^256 - 2^32 - 977`.
static FIELD_MODULUS: LazyLock<BigUint> =
    LazyLock::new(|| parse("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F"));

/// The order of the generator's subgroup.
static GROUP_ORDER: LazyLock<BigUint> =
    LazyLock::new(|| parse("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141"));

/// The generator's affine coordinates.
static GENERATOR: LazyLock<AffinePoint> = LazyLock::new(|| AffinePoint {
    x: parse("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"),
    y: parse("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"),
});

/// The curve's constant term: the equation is `y^2 = x^3 + 7`.
const CURVE_CONSTANT: u32 = 7;

/// How many bytes a field element occupies on the wire.
pub const FIELD_ELEMENT_BYTES: usize = 32;

/// The prefix byte an uncompressed point encoding carries.
pub const UNCOMPRESSED_PREFIX: u8 = 0x04;

/// Reads one of the published constants.
///
/// Total by construction: every caller passes a literal from the
/// published curve parameters, and a defect in one is a defect in this
/// file rather than a condition a caller could handle.
fn parse(hexadecimal: &str) -> BigUint {
    BigUint::parse_bytes(hexadecimal.as_bytes(), 16)
        .expect("the published curve parameters are well formed")
}

/// One point of the curve, other than the identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AffinePoint {
    x: BigUint,
    y: BigUint,
}

/// One point of the curve, the identity included.
///
/// The identity is represented rather than made unrepresentable because
/// point addition genuinely produces it, and a sum that silently became
/// some other point would be a wrong answer where the honest answer is
/// "this construction has no output key".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurvePoint {
    /// The identity, which has no affine coordinates and no x-only
    /// encoding.
    Identity,
    /// An ordinary point.
    Affine(AffinePoint),
}

/// Why a 32-byte value is not the x coordinate of a curve point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointDecodingDefect {
    /// The value is not below the field modulus, so it is not a field
    /// element at all.
    NotAFieldElement,
    /// The value is a field element, but `x^3 + 7` is not a square, so
    /// no point on the curve has it as an x coordinate. Roughly half of
    /// all field elements are like this.
    NotOnCurve,
}

impl AffinePoint {
    /// The x coordinate, as the target serializes it.
    #[must_use]
    pub fn x_only_bytes(&self) -> [u8; FIELD_ELEMENT_BYTES] {
        field_bytes(&self.x)
    }

    /// The point's uncompressed encoding: a prefix byte, then both
    /// coordinates.
    ///
    /// The one place this package needs a y coordinate on the wire. It
    /// exists because the published unspendable internal key is derived
    /// as a digest of the generator in exactly this encoding, and that
    /// derivation is recomputed rather than trusted.
    #[must_use]
    pub fn uncompressed_bytes(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(1 + 2 * FIELD_ELEMENT_BYTES);
        encoded.push(UNCOMPRESSED_PREFIX);
        encoded.extend_from_slice(&field_bytes(&self.x));
        encoded.extend_from_slice(&field_bytes(&self.y));
        encoded
    }

    /// Whether the y coordinate is even.
    ///
    /// The target's x-only encoding drops the y coordinate entirely and
    /// implies the even one, so this is the bit a control block has to
    /// carry for a verifier to recover the point that was actually
    /// committed to.
    #[must_use]
    pub fn has_even_y(&self) -> bool {
        !self.y.bit(0)
    }

    /// The parity bit the target's control block carries: 0 for an even
    /// y coordinate, 1 for an odd one.
    #[must_use]
    pub fn parity_bit(&self) -> u8 {
        u8::from(!self.has_even_y())
    }
}

/// The generator.
#[must_use]
pub fn generator() -> AffinePoint {
    GENERATOR.clone()
}

/// The order of the generator's subgroup.
///
/// Exposed so that a caller reducing a scalar modulo the order uses the
/// same published constant this module's own overflow rule uses. A
/// second transcription of the parameter would be a second chance to
/// get one nibble wrong, and the census already records what one wrong
/// nibble cost.
#[must_use]
pub fn group_order() -> &'static BigUint {
    &GROUP_ORDER
}

/// Whether a 32-byte scalar is a valid multiplier.
///
/// Anything at or above the group order is not, and nothing else is
/// excluded. The target reads a tweak as a scalar and refuses it on
/// overflow alone, so overflow is the whole of the rule here as well
/// `(´[PLAN-rule:guide10:tweak-totality]´)`.
///
/// # Zero is a multiplier
///
/// It takes the generator to the identity, and the identity is the
/// additive unit, so `P + 0G = P`: a zero tweak leaves the key it is
/// added to exactly where it was. Refusing it here would have named a
/// rule the target does not have and would have reported a
/// construction the target accepts as one with no output key at all.
///
/// What genuinely has no output key is a *sum* that is the identity,
/// and that is decided after the addition — by looking at the sum,
/// where the fact is — rather than guessed at from one operand.
#[must_use]
pub fn is_valid_scalar(scalar: &[u8; FIELD_ELEMENT_BYTES]) -> bool {
    BigUint::from_bytes_be(scalar) < *GROUP_ORDER
}

/// The point whose x coordinate is `x` and whose y coordinate is even.
///
/// This is the target's rule for reading an x-only key: the encoding
/// names one coordinate and the even y is implied.
///
/// # Errors
///
/// [`PointDecodingDefect`] when the value is not a field element, or is
/// one that no curve point has as an x coordinate.
pub fn lift_x(x: &[u8; FIELD_ELEMENT_BYTES]) -> Result<AffinePoint, PointDecodingDefect> {
    let x = BigUint::from_bytes_be(x);
    if x >= *FIELD_MODULUS {
        return Err(PointDecodingDefect::NotAFieldElement);
    }

    let square = (x.modpow(&BigUint::from(3_u32), &FIELD_MODULUS) + BigUint::from(CURVE_CONSTANT))
        % &*FIELD_MODULUS;
    // The modulus is congruent to 3 modulo 4, so a square root — where
    // one exists — is the (p+1)/4 power. Where one does not, the same
    // power is some other element, which is why the result is squared
    // back and checked rather than trusted.
    let exponent = (&*FIELD_MODULUS + BigUint::one()) >> 2_u32;
    let root = square.modpow(&exponent, &FIELD_MODULUS);
    if root.modpow(&BigUint::from(2_u32), &FIELD_MODULUS) != square {
        return Err(PointDecodingDefect::NotOnCurve);
    }

    let y = if root.bit(0) {
        &*FIELD_MODULUS - &root
    } else {
        root
    };
    Ok(AffinePoint { x, y })
}

/// The sum of two points.
#[must_use]
pub fn add(left: &CurvePoint, right: &CurvePoint) -> CurvePoint {
    let (left, right) = match (left, right) {
        (CurvePoint::Identity, other) | (other, CurvePoint::Identity) => return other.clone(),
        (CurvePoint::Affine(left), CurvePoint::Affine(right)) => (left, right),
    };

    if left.x == right.x {
        // Two points sharing an x coordinate are equal or are each
        // other's negation; the negation case sums to the identity.
        if left.y != right.y || left.y.is_zero() {
            return CurvePoint::Identity;
        }
        return double(left);
    }

    let numerator = subtract(&right.y, &left.y);
    let denominator = subtract(&right.x, &left.x);
    let slope = multiply(&numerator, &invert(&denominator));
    complete(left, right, &slope)
}

/// The double of one point.
fn double(point: &AffinePoint) -> CurvePoint {
    if point.y.is_zero() {
        return CurvePoint::Identity;
    }
    let numerator = multiply(&BigUint::from(3_u32), &multiply(&point.x, &point.x));
    let denominator = multiply(&BigUint::from(2_u32), &point.y);
    let slope = multiply(&numerator, &invert(&denominator));
    complete(point, point, &slope)
}

/// The third intersection, reflected: the chord-and-tangent result once
/// the slope is known.
fn complete(left: &AffinePoint, right: &AffinePoint, slope: &BigUint) -> CurvePoint {
    let x = subtract(&subtract(&multiply(slope, slope), &left.x), &right.x);
    let y = subtract(&multiply(slope, &subtract(&left.x, &x)), &left.y);
    CurvePoint::Affine(AffinePoint { x, y })
}

/// The multiple `scalar · point`, by double and add.
#[must_use]
pub fn multiply_point(scalar: &[u8; FIELD_ELEMENT_BYTES], point: &AffinePoint) -> CurvePoint {
    let scalar = BigUint::from_bytes_be(scalar);
    let mut total = CurvePoint::Identity;
    let mut addend = CurvePoint::Affine(point.clone());

    // Least significant bit first, doubling the addend as it goes.
    for bit in 0..scalar.bits() {
        if scalar.bit(bit) {
            total = add(&total, &addend);
        }
        addend = add(&addend, &addend);
    }
    total
}

/// One field element as the target's 32 big-endian bytes.
fn field_bytes(value: &BigUint) -> [u8; FIELD_ELEMENT_BYTES] {
    let mut bytes = [0_u8; FIELD_ELEMENT_BYTES];
    let encoded = value.to_bytes_be();
    // Total by construction: every value reaching here is reduced
    // modulo the field, so it never needs more than 32 bytes.
    let offset = FIELD_ELEMENT_BYTES.saturating_sub(encoded.len());
    bytes[offset..].copy_from_slice(&encoded);
    bytes
}

/// The product of two field elements.
fn multiply(left: &BigUint, right: &BigUint) -> BigUint {
    (left * right) % &*FIELD_MODULUS
}

/// The difference of two field elements.
fn subtract(left: &BigUint, right: &BigUint) -> BigUint {
    ((left + &*FIELD_MODULUS) - right) % &*FIELD_MODULUS
}

/// The multiplicative inverse of a field element.
///
/// By Fermat's little theorem, which needs no extended Euclid and no
/// branch on the operand. Zero has no inverse and the modular power
/// returns zero for it, which propagates into an identity result rather
/// than a wrong point: the callers reach it only where the addition
/// formulas have already ruled the case out.
fn invert(value: &BigUint) -> BigUint {
    value.modpow(&(&*FIELD_MODULUS - BigUint::from(2_u32)), &FIELD_MODULUS)
}
