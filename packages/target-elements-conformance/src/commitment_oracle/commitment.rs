//! Pedersen commitment construction.
//!
//! The reviewed relation, with both terms positive:
//!
//! ```text
//! C = r · G + v · H_A
//! ```
//!
//! where `r` is the blinding scalar read as thirty-two big-endian bytes,
//! `G` is the conventional base point, `v` is the amount, and `H_A` is
//! the asset generator. Source:
//! `src/secp256k1/src/modules/generator/pedersen_impl.h:42-49` for the
//! relation and `main_impl.h:308-335` for the domain rules.

use num_bigint::BigUint;

use super::curve::{
    self, FIELD_ELEMENT_BYTES, Group, PREFIXED_POINT_BYTES, Point, PointEncodingDefect,
};
use super::generator::{GeneratorDefect, asset_generator};

/// The byte a commitment encoding exclusive-ors the squareness of `y`
/// into, so `0x08` marks a square y and `0x09` a non-square one.
///
/// Source: `src/secp256k1/src/modules/generator/main_impl.h:277-278`.
pub const COMMITMENT_PREFIX_BASE: u8 = 9;

/// The width of the opening scalar.
pub const SCALAR_BYTES: usize = FIELD_ELEMENT_BYTES;

/// The exclusive upper bound of the semantic amount domain, `2^51`.
///
/// This is the protocol's bound, not the target's. The reviewed target
/// admits the full unsigned sixty-four bit range at this layer and
/// leaves the range to its caller, so the two facts are kept apart:
/// [`is_semantic_amount`] answers the protocol question and
/// [`commitment`] computes regardless.
pub const SEMANTIC_AMOUNT_BOUND: u64 = 1 << 51;

/// Whether an amount lies in the protocol's semantic domain.
///
/// A false answer is not a statement that the target would refuse the
/// amount. It would not: the commitment layer takes any `u64`.
#[must_use]
pub const fn is_semantic_amount(amount: u64) -> bool {
    amount < SEMANTIC_AMOUNT_BOUND
}

/// Why a byte string is not an opening scalar the target accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ScalarDefect {
    /// The scalar is not thirty-two bytes.
    WrongWidth {
        /// The width the relation requires.
        expected: usize,
        /// The width supplied.
        found: usize,
    },
    /// The scalar is at or above the group order, which the commit
    /// routine refuses without computing.
    AtOrAboveGroupOrder,
}

/// Why a commitment could not be constructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CommitmentDefect {
    /// The blinding factor is not an admitted scalar.
    Scalar(ScalarDefect),
    /// The asset identifier does not yield a generator.
    Generator(GeneratorDefect),
    /// The relation lands on the identity, which the target refuses
    /// rather than encoding.
    ///
    /// This is how a zero amount under a zero blinder is rejected.
    IdentityResult,
}

/// Reads an opening scalar.
///
/// Zero is admitted deliberately: an explicit amount enters the target's
/// balance as a commitment under a zero blinder.
///
/// # Errors
///
/// [`ScalarDefect`] when the width or the group domain check fails.
pub fn read_scalar(blinding_factor: &[u8]) -> Result<BigUint, ScalarDefect> {
    if blinding_factor.len() != SCALAR_BYTES {
        return Err(ScalarDefect::WrongWidth {
            expected: SCALAR_BYTES,
            found: blinding_factor.len(),
        });
    }
    let scalar = BigUint::from_bytes_be(blinding_factor);
    if &scalar >= curve::group_order() {
        return Err(ScalarDefect::AtOrAboveGroupOrder);
    }
    Ok(scalar)
}

/// The commitment point for one amount under one blinder and generator.
///
/// # Errors
///
/// [`CommitmentDefect`] when the scalar is refused or the relation lands
/// on the identity.
pub fn commitment_point(
    blinding_factor: &[u8],
    amount: u64,
    generator: &Point,
) -> Result<Point, CommitmentDefect> {
    let scalar = read_scalar(blinding_factor).map_err(CommitmentDefect::Scalar)?;

    let blinded = curve::multiply_point(&scalar, &curve::base_point());
    let valued = curve::multiply_point(&BigUint::from(amount), generator);

    match curve::add(&blinded, &valued) {
        Group::Affine(point) => Ok(point),
        Group::Identity => Err(CommitmentDefect::IdentityResult),
    }
}

/// The serialized commitment for one amount under one blinder and asset.
///
/// # Errors
///
/// [`CommitmentDefect`] when the asset, the scalar, or the result is
/// refused.
pub fn commitment(
    asset_id: &[u8],
    amount: u64,
    blinding_factor: &[u8],
) -> Result<[u8; PREFIXED_POINT_BYTES], CommitmentDefect> {
    let generator = asset_generator(asset_id).map_err(CommitmentDefect::Generator)?;
    let point = commitment_point(blinding_factor, amount, &generator)?;
    Ok(curve::encode_prefixed_point(&point, COMMITMENT_PREFIX_BASE))
}

/// The blinded generator of one asset.
///
/// # Why a blinded generator is needed at all
///
/// [`asset_generator`] derives the generator an *explicit* asset names.
/// A confidential output does not use it: the asset is itself blinded,
/// and the generator the value commitment is taken against is
///
/// ```text
/// H' = H(asset) + assetblinder · G
/// ```
///
/// Without this, the oracle can predict a commitment only for an output
/// whose asset is in the clear — and the balanced confidential rows of
/// the §8.4 matrix have no such output. The three-way comparison of §7.4
/// would then have nothing to compare on exactly the rows it matters
/// most for.
///
/// # Errors
///
/// [`CommitmentDefect`] when the asset, the scalar, or the sum is
/// refused.
pub fn blinded_asset_generator(
    asset_id: &[u8],
    asset_blinding_factor: &[u8],
) -> Result<Point, CommitmentDefect> {
    let generator = asset_generator(asset_id).map_err(CommitmentDefect::Generator)?;
    let scalar = read_scalar(asset_blinding_factor).map_err(CommitmentDefect::Scalar)?;

    let offset = curve::multiply_point(&scalar, &curve::base_point());
    match curve::add(&Group::Affine(generator), &offset) {
        Group::Affine(point) => Ok(point),
        Group::Identity => Err(CommitmentDefect::IdentityResult),
    }
}

/// The serialized value commitment of one fully confidential output.
///
/// Both blinders are the output's own: the asset blinder determines the
/// generator, and the value blinder the commitment taken against it. This
/// is what the target's own openings — the amount and the two blinding
/// factors it reports for an output it created — are checked against.
///
/// # Errors
///
/// [`CommitmentDefect`] when the asset, either scalar, or a result is
/// refused.
pub fn blinded_commitment(
    asset_id: &[u8],
    amount: u64,
    asset_blinding_factor: &[u8],
    value_blinding_factor: &[u8],
) -> Result<[u8; PREFIXED_POINT_BYTES], CommitmentDefect> {
    let generator = blinded_asset_generator(asset_id, asset_blinding_factor)?;
    let point = commitment_point(value_blinding_factor, amount, &generator)?;
    Ok(curve::encode_prefixed_point(&point, COMMITMENT_PREFIX_BASE))
}

/// Reads a serialized commitment.
///
/// # Errors
///
/// [`PointEncodingDefect`] when the encoding is not a canonical
/// commitment.
pub fn parse_commitment(encoded: &[u8]) -> Result<Point, PointEncodingDefect> {
    curve::decode_prefixed_point(encoded, COMMITMENT_PREFIX_BASE)
}
