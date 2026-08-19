//! Asset generator derivation.
//!
//! The recipe the reviewed target uses, restated here so the oracle can
//! predict a generator without asking the target for one. Every step
//! cites the vendored source it was read from, at merged tip `78499c2`
//! under ADR-018.
//!
//! ```text
//! generator(asset) = map(sha256("1st generation: " ‖ asset))
//!                  + map(sha256("2nd generation: " ‖ asset))
//! ```
//!
//! `map` is the Fouque and Tibouchi indifferentiable construction, which
//! the library names after Shallue and van de Woestijne. The reviewed
//! contract in `target-elements` states the shape of this recipe — two
//! tagged hashes, two curve maps, one point addition — and this module
//! is what performs it `(´[PLAN-rule:guide11:generator-recipe]´)`.

use num_bigint::BigUint;
use num_traits::One;
use sha2::{Digest, Sha256};

use super::curve::{
    self, CURVE_CONSTANT, FIELD_ELEMENT_BYTES, Group, PREFIXED_POINT_BYTES, Point,
    PointEncodingDefect,
};

/// The byte a generator encoding exclusive-ors the squareness of `y`
/// into, so `0x0a` marks a square y and `0x0b` a non-square one.
///
/// Source: `src/secp256k1/src/modules/generator/main_impl.h:88`.
pub const GENERATOR_PREFIX_BASE: u8 = 11;

/// The tagged prefix of the first generation, sixteen bytes.
///
/// Source: `src/secp256k1/src/modules/generator/main_impl.h:205`.
const FIRST_GENERATION: &[u8; 16] = b"1st generation: ";

/// The tagged prefix of the second generation, sixteen bytes.
///
/// Source: `src/secp256k1/src/modules/generator/main_impl.h:206`.
const SECOND_GENERATION: &[u8; 16] = b"2nd generation: ";

/// `-c`, where `c` is a square root of `-3`.
///
/// Source: `src/secp256k1/src/modules/generator/main_impl.h:132`.
const NEGATED_C: &str = "F5D2D456CAF80E20DCC88F3D586869D339E092EA25EB132B8272D850E32A03DD";

/// `d = (c - 1) / 2`, the x coordinate the map degenerates to.
///
/// Source: `src/secp256k1/src/modules/generator/main_impl.h:133`.
const MAP_D: &str = "851695D49A83F8EF919BB86153CBCB16630FB68AED0A766A3EC693D68E6AFA40";

/// Why an asset identifier does not yield a generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GeneratorDefect {
    /// The asset identifier is not thirty-two bytes.
    WrongIdentifierWidth {
        /// The width the recipe consumes.
        expected: usize,
        /// The width supplied.
        found: usize,
    },
    /// A generation's digest is not below the field modulus.
    ///
    /// The library folds this into a return code its consensus callers
    /// assert on, so a caller here is told rather than handed a point
    /// derived from an out-of-range element.
    DigestOutOfFieldRange {
        /// Which generation's digest fell out of range, one or two.
        generation: u8,
    },
    /// The two mapped points are each other's negation, so their sum has
    /// no coordinates.
    ///
    /// No reviewed asset identifier produces this. It is named rather
    /// than folded into another variant so that an oracle which ever met
    /// it would say so instead of reporting a range failure.
    IdentitySum,
}

fn constant(hexadecimal: &str) -> BigUint {
    BigUint::parse_bytes(hexadecimal.as_bytes(), 16)
        .expect("the published map constants are well formed")
}

/// The tagged digest of one generation, as a field element.
fn generation_digest(prefix: &[u8; 16], asset_id: &[u8; FIELD_ELEMENT_BYTES]) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(prefix);
    hasher.update(asset_id);
    BigUint::from_bytes_be(&hasher.finalize())
}

/// The Fouque and Tibouchi map from a field element to the curve.
///
/// Transcribed operation by operation from
/// `src/secp256k1/src/modules/generator/main_impl.h:94-202`. The
/// published algorithm takes the sign of `y` from the Jacobi symbol of
/// `t`; this implementation follows the library in taking it from the
/// oddness of `t` instead, which the source justifies by noting that the
/// rest of the map uses only `t^2`, so any criterion that negates with
/// `t` will do (`:196-200`).
///
/// The output is on the curve by construction, including in the
/// degenerate case `t = 0`, where the joint denominator vanishes and the
/// map yields the point at `d`.
///
/// # Panics
///
/// Never, for any field element. The map is proven to place at least one
/// of its three candidate x coordinates on the curve, and the panic
/// guards that proof rather than a caller's input: reaching it would
/// mean the transcription of the algorithm is wrong, which is a defect
/// here and not a condition a caller could handle.
#[must_use]
pub fn curve_map(t: &BigUint) -> Point {
    let t = t % curve::field_modulus();
    let negated_c = constant(NEGATED_C);
    let map_d = constant(MAP_D);
    let one = BigUint::one();

    // wd = t^2
    let mut wd = curve::multiply(&t, &t);
    // x1 = -c * t^2
    let mut x1 = curve::multiply(&negated_c, &wd);
    // x3d = -3 * t^2
    let x3d = curve::subtract(
        &BigUint::from(0_u32),
        &curve::multiply(&BigUint::from(3_u32), &wd),
    );
    // wd = 1 + b + t^2
    wd = curve::add_field(&wd, &BigUint::from(CURVE_CONSTANT + 1));
    // jinv = 1 / (wd * x3d), which the library defines as zero when the
    // product vanishes.
    let jinv = curve::invert(&curve::multiply(&wd, &x3d));

    // x1 = d + (-c * t^2 * x3d) / j
    x1 = curve::multiply(&x1, &x3d);
    x1 = curve::multiply(&x1, &jinv);
    x1 = curve::add_field(&x1, &map_d);
    // x2 = -(x1 + 1)
    let x2 = curve::subtract(&BigUint::from(0_u32), &curve::add_field(&x1, &one));
    // x3 = 1 + wd^3 / j
    let wd_cubed = curve::multiply(&curve::multiply(&wd, &wd), &wd);
    let x3 = curve::add_field(&curve::multiply(&wd_cubed, &jinv), &one);

    // The first candidate whose cubic is a square wins, in order.
    let mut chosen = None;
    for candidate in [x1, x2, x3] {
        let cubic = curve::add_field(
            &curve::multiply(&curve::multiply(&candidate, &candidate), &candidate),
            &BigUint::from(CURVE_CONSTANT),
        );
        if let Some(root) = curve::square_root(&cubic) {
            chosen = Some(Point::from_coordinates(candidate, root));
            break;
        }
    }
    // Total by construction: the map is proven to place at least one of
    // the three candidates on the curve.
    let point = chosen.expect("the curve map always yields a curve point");

    if t.bit(0) { point.negate() } else { point }
}

/// The generator of one asset.
///
/// Source: `src/secp256k1/src/modules/generator/main_impl.h:204-248`.
///
/// # Errors
///
/// [`GeneratorDefect`] when the identifier is the wrong width, or when a
/// generation's digest is not below the field modulus.
pub fn asset_generator(asset_id: &[u8]) -> Result<Point, GeneratorDefect> {
    let asset_id: &[u8; FIELD_ELEMENT_BYTES] =
        asset_id
            .try_into()
            .map_err(|_| GeneratorDefect::WrongIdentifierWidth {
                expected: FIELD_ELEMENT_BYTES,
                found: asset_id.len(),
            })?;

    let mut accumulated = Group::Identity;
    for (generation, prefix) in [(1_u8, FIRST_GENERATION), (2_u8, SECOND_GENERATION)] {
        let t = generation_digest(prefix, asset_id);
        if &t >= curve::field_modulus() {
            return Err(GeneratorDefect::DigestOutOfFieldRange { generation });
        }
        accumulated = curve::add(&accumulated, &Group::Affine(curve_map(&t)));
    }

    // The sum of two independent map outputs is the identity only if
    // they are each other's negation, which no reviewed asset produces.
    match accumulated {
        Group::Affine(point) => Ok(point),
        Group::Identity => Err(GeneratorDefect::IdentitySum),
    }
}

/// The serialized generator of one asset.
///
/// # Errors
///
/// [`GeneratorDefect`] as [`asset_generator`].
pub fn serialized_asset_generator(
    asset_id: &[u8],
) -> Result<[u8; PREFIXED_POINT_BYTES], GeneratorDefect> {
    Ok(curve::encode_prefixed_point(
        &asset_generator(asset_id)?,
        GENERATOR_PREFIX_BASE,
    ))
}

/// Reads a serialized generator.
///
/// # Errors
///
/// [`PointEncodingDefect`] when the encoding is not a canonical
/// generator.
pub fn parse_generator(encoded: &[u8]) -> Result<Point, PointEncodingDefect> {
    curve::decode_prefixed_point(encoded, GENERATOR_PREFIX_BASE)
}
