//! Curve and field arithmetic for the confidential-value oracle.
//!
//! # Why this is not `constructor::curve`
//!
//! The taproot constructor already carries public-point arithmetic, and
//! this module deliberately does not extend it. That module states a
//! narrow charter — public taproot values, an x-only encoding, and the
//! oddness of `y` — and says in as many words that it would rather stop
//! existing than grow a capability quietly. The confidential encodings
//! need a different set: quadratic residuosity rather than oddness, a
//! point built from both coordinates, negation, and a curve map. Adding
//! those there would blur a boundary the target itself draws, since the
//! disagreement between the two parity conventions is precisely the
//! reviewed blocker the target review named.
//!
//! # Public data only
//!
//! Nothing here holds or produces a secret. A blinding factor reaching
//! this module is test material stated in the open
//! `(´[PLAN-rule:guide10:public-data]´)`, so the arithmetic is
//! plainly written rather than constant-time. A module that ever needed
//! to be constant-time would be a different module with a different
//! review.

use std::sync::LazyLock;

use num_bigint::BigUint;
use num_traits::{One, Zero};

/// The field the curve is defined over, `2^256 - 2^32 - 977`.
static FIELD_MODULUS: LazyLock<BigUint> =
    LazyLock::new(|| parse("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F"));

/// The order of the generator's subgroup.
static GROUP_ORDER: LazyLock<BigUint> =
    LazyLock::new(|| parse("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141"));

/// The conventional base point, which the blinding term multiplies.
static BASE_POINT: LazyLock<Point> = LazyLock::new(|| Point {
    x: parse("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"),
    y: parse("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"),
});

/// The curve's constant term: the equation is `y^2 = x^3 + 7`.
pub const CURVE_CONSTANT: u32 = 7;

/// How many bytes a field element occupies on the wire.
pub const FIELD_ELEMENT_BYTES: usize = 32;

/// How many bytes a prefixed point encoding occupies.
pub const PREFIXED_POINT_BYTES: usize = 1 + FIELD_ELEMENT_BYTES;

/// Reads one of the published constants.
///
/// Total by construction: every caller passes a literal from the
/// published curve parameters, so a defect in one is a defect in this
/// file rather than a condition a caller could handle.
fn parse(hexadecimal: &str) -> BigUint {
    BigUint::parse_bytes(hexadecimal.as_bytes(), 16)
        .expect("the published curve parameters are well formed")
}

/// The field modulus.
#[must_use]
pub fn field_modulus() -> &'static BigUint {
    &FIELD_MODULUS
}

/// The order of the generator's subgroup.
#[must_use]
pub fn group_order() -> &'static BigUint {
    &GROUP_ORDER
}

/// The conventional base point.
#[must_use]
pub fn base_point() -> Point {
    BASE_POINT.clone()
}

/// One point of the curve, other than the identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point {
    x: BigUint,
    y: BigUint,
}

impl Point {
    /// States a point from both coordinates, without checking it.
    ///
    /// Used by the curve map, whose output is on the curve by
    /// construction, and by tests that state a point directly.
    #[must_use]
    pub const fn from_coordinates(x: BigUint, y: BigUint) -> Self {
        Self { x, y }
    }

    /// The x coordinate.
    #[must_use]
    pub const fn x(&self) -> &BigUint {
        &self.x
    }

    /// The y coordinate.
    #[must_use]
    pub const fn y(&self) -> &BigUint {
        &self.y
    }

    /// The x coordinate as the target serializes it.
    #[must_use]
    pub fn x_bytes(&self) -> [u8; FIELD_ELEMENT_BYTES] {
        field_bytes(&self.x)
    }

    /// Whether the y coordinate is a quadratic residue.
    ///
    /// This is the bit the confidential encodings record, and it is not
    /// the oddness of `y`.
    #[must_use]
    pub fn has_square_y(&self) -> bool {
        is_square(&self.y)
    }

    /// The point's negation, which shares its x coordinate.
    #[must_use]
    pub fn negate(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: subtract(&BigUint::zero(), &self.y),
        }
    }

    /// Whether the point satisfies the curve equation.
    #[must_use]
    pub fn is_on_curve(&self) -> bool {
        let left = multiply(&self.y, &self.y);
        let right = add_field(
            &multiply(&multiply(&self.x, &self.x), &self.x),
            &BigUint::from(CURVE_CONSTANT),
        );
        left == right
    }
}

/// One point of the curve, the identity included.
///
/// The identity is represented rather than made unrepresentable because
/// addition genuinely produces it, and the target refuses a commitment
/// that lands there rather than encoding some other point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Group {
    /// The identity, which has no coordinates and no encoding.
    Identity,
    /// An ordinary point.
    Affine(Point),
}

impl Group {
    /// The ordinary point, where this is not the identity.
    #[must_use]
    pub const fn affine(&self) -> Option<&Point> {
        match self {
            Self::Identity => None,
            Self::Affine(point) => Some(point),
        }
    }
}

/// Whether a field element is a quadratic residue.
///
/// By Euler's criterion. Zero is reported as a square, matching the
/// library's own predicate; no point of this curve has a zero y
/// coordinate, since that would be a point of order two and the group
/// has prime order.
#[must_use]
pub fn is_square(value: &BigUint) -> bool {
    let value = value % &*FIELD_MODULUS;
    if value.is_zero() {
        return true;
    }
    let exponent = (&*FIELD_MODULUS - BigUint::one()) >> 1_u32;
    value.modpow(&exponent, &FIELD_MODULUS).is_one()
}

/// The square root the library computes, where one exists.
///
/// The modulus is congruent to 3 modulo 4, so the `(p+1)/4` power is a
/// square root whenever the input has one. That root is itself always a
/// square, because `(p+1)/4` is even — which is what makes the parity
/// prefix recoverable from the x coordinate alone.
#[must_use]
pub fn square_root(value: &BigUint) -> Option<BigUint> {
    let value = value % &*FIELD_MODULUS;
    let exponent = (&*FIELD_MODULUS + BigUint::one()) >> 2_u32;
    let root = value.modpow(&exponent, &FIELD_MODULUS);
    (multiply(&root, &root) == value).then_some(root)
}

/// The sum of two points.
#[must_use]
pub fn add(left: &Group, right: &Group) -> Group {
    let (left, right) = match (left, right) {
        (Group::Identity, other) | (other, Group::Identity) => return other.clone(),
        (Group::Affine(left), Group::Affine(right)) => (left, right),
    };

    if left.x == right.x {
        // Points sharing an x coordinate are equal or are each other's
        // negation; the negation case sums to the identity.
        if left.y != right.y || left.y.is_zero() {
            return Group::Identity;
        }
        return double(left);
    }

    let numerator = subtract(&right.y, &left.y);
    let denominator = subtract(&right.x, &left.x);
    let slope = multiply(&numerator, &invert(&denominator));
    complete(left, right, &slope)
}

/// The double of one point.
fn double(point: &Point) -> Group {
    if point.y.is_zero() {
        return Group::Identity;
    }
    let numerator = multiply(&BigUint::from(3_u32), &multiply(&point.x, &point.x));
    let denominator = multiply(&BigUint::from(2_u32), &point.y);
    let slope = multiply(&numerator, &invert(&denominator));
    complete(point, point, &slope)
}

/// The chord-and-tangent result once the slope is known.
fn complete(left: &Point, right: &Point, slope: &BigUint) -> Group {
    let x = subtract(&subtract(&multiply(slope, slope), &left.x), &right.x);
    let y = subtract(&multiply(slope, &subtract(&left.x, &x)), &left.y);
    Group::Affine(Point { x, y })
}

/// The multiple `scalar · point`, by double and add.
#[must_use]
pub fn multiply_point(scalar: &BigUint, point: &Point) -> Group {
    let mut total = Group::Identity;
    let mut addend = Group::Affine(point.clone());

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
#[must_use]
pub fn field_bytes(value: &BigUint) -> [u8; FIELD_ELEMENT_BYTES] {
    let mut bytes = [0_u8; FIELD_ELEMENT_BYTES];
    let encoded = value.to_bytes_be();
    // Total by construction: every value reaching here is reduced
    // modulo the field, so it never needs more than 32 bytes.
    let offset = FIELD_ELEMENT_BYTES.saturating_sub(encoded.len());
    bytes[offset..].copy_from_slice(&encoded);
    bytes
}

/// Why a byte string is not the encoding of a confidential point.
///
/// Every variant names a check the target itself performs, in the order
/// it performs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PointEncodingDefect {
    /// The encoding is not thirty-three bytes.
    ///
    /// Both a short encoding and one carrying trailing bytes land here:
    /// the target reads a fixed width and there is no framing that would
    /// let a longer string mean the same point.
    WrongWidth {
        /// The width the field requires.
        expected: usize,
        /// The width supplied.
        found: usize,
    },
    /// The prefix is not one of the two this field admits.
    UnknownPrefix(u8),
    /// The x coordinate is not below the field modulus.
    XNotAFieldElement,
    /// No point of the curve has that x coordinate.
    XNotOnCurve,
}

/// Encodes a point under a quadratic-residue prefix convention.
///
/// `base` is the byte the encoder exclusive-ors the squareness of `y`
/// into: eleven for a generator and nine for a value commitment, so a
/// square `y` yields `0x0a` and `0x08` respectively.
#[must_use]
pub fn encode_prefixed_point(point: &Point, base: u8) -> [u8; PREFIXED_POINT_BYTES] {
    let mut encoded = [0_u8; PREFIXED_POINT_BYTES];
    encoded[0] = base ^ u8::from(point.has_square_y());
    encoded[1..].copy_from_slice(&point.x_bytes());
    encoded
}

/// Reads a point encoded under a quadratic-residue prefix convention.
///
/// The prefix selects between the two y values sharing the x
/// coordinate: the even-valued prefix names the point whose y is a
/// quadratic residue, and the odd-valued one names its negation.
///
/// # Errors
///
/// [`PointEncodingDefect`] when the width, prefix, field range, or curve
/// membership check fails.
pub fn decode_prefixed_point(encoded: &[u8], base: u8) -> Result<Point, PointEncodingDefect> {
    if encoded.len() != PREFIXED_POINT_BYTES {
        return Err(PointEncodingDefect::WrongWidth {
            expected: PREFIXED_POINT_BYTES,
            found: encoded.len(),
        });
    }
    let prefix = encoded[0];
    if prefix & 0xFE != base & 0xFE {
        return Err(PointEncodingDefect::UnknownPrefix(prefix));
    }

    let x = BigUint::from_bytes_be(&encoded[1..]);
    if x >= *FIELD_MODULUS {
        return Err(PointEncodingDefect::XNotAFieldElement);
    }

    let square = add_field(
        &multiply(&multiply(&x, &x), &x),
        &BigUint::from(CURVE_CONSTANT),
    );
    let root = square_root(&square).ok_or(PointEncodingDefect::XNotOnCurve)?;

    // The root the field returns is itself a square, so it is the y the
    // even-valued prefix names.
    let point = Point { x, y: root };
    Ok(if prefix & 1 == 1 {
        point.negate()
    } else {
        point
    })
}

/// The product of two field elements.
#[must_use]
pub fn multiply(left: &BigUint, right: &BigUint) -> BigUint {
    (left * right) % &*FIELD_MODULUS
}

/// The sum of two field elements.
#[must_use]
pub fn add_field(left: &BigUint, right: &BigUint) -> BigUint {
    (left + right) % &*FIELD_MODULUS
}

/// The difference of two field elements.
#[must_use]
pub fn subtract(left: &BigUint, right: &BigUint) -> BigUint {
    ((left + &*FIELD_MODULUS) - (right % &*FIELD_MODULUS)) % &*FIELD_MODULUS
}

/// The multiplicative inverse of a field element.
///
/// By Fermat's little theorem. Zero has no inverse and the modular power
/// returns zero for it, which is the behavior the curve map relies on:
/// the library's own inversion is documented to return zero there, and
/// the map's degenerate case is defined in terms of it.
#[must_use]
pub fn invert(value: &BigUint) -> BigUint {
    value.modpow(&(&*FIELD_MODULUS - BigUint::from(2_u32)), &FIELD_MODULUS)
}
