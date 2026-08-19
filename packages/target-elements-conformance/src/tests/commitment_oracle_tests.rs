//! What the commitment oracle is checked against.
//!
//! # Two derivations, and a published third
//!
//! Every expected byte string below was computed twice before it was
//! pinned: once by this crate's oracle, and once by a separate
//! implementation written independently in Python from the same
//! source-cited recipe. A value entered this file only where both agreed
//! byte for byte `(´[PLAN-rule:guide11:two-derivations]´)`.
//!
//! Better than either, the vendored curve library carries its own
//! published fixed vectors, and they are reproduced here as the outside
//! opinion an oracle checked only against itself would lack:
//!
//! - `src/secp256k1/src/modules/generator/tests_impl.h:52-84` — the
//!   curve map for small `t`, stated there to match an independent SAGE
//!   program;
//! - `:109-142` — the generator of every asset identifier that is
//!   thirty-one zero bytes followed by `i`, for `i` from one to
//!   thirty-two, which is where the two assets used below come from;
//! - `:175-178` and `:325-329` — one point under both prefix
//!   conventions.
//!
//! # The `two_g` fixture does not encode `2G`
//!
//! The upstream fixture named `two_g` carries prefix `0x0b`, and it is a
//! parse-and-serialize round trip rather than an assertion about the
//! doubled base point. Parsing takes the point whose y is a quadratic
//! residue and then negates it when the prefix is odd, so `0x0b`
//! followed by that x names the *negation* of `2G`. The y of `2G` itself
//! is a square, so `2G` encodes as `0x0a`. The distinction is pinned
//! below so a later wave does not read the fixture's name as a claim.
//!
//! All material here is public test data under ADR-015.

use num_bigint::BigUint;

use crate::commitment_oracle::commitment::{
    COMMITMENT_PREFIX_BASE, CommitmentDefect, ScalarDefect, commitment, is_semantic_amount,
    parse_commitment,
};
use crate::commitment_oracle::curve::{
    self, Point, PointEncodingDefect, encode_prefixed_point, is_square,
};
use crate::commitment_oracle::generator::{
    GENERATOR_PREFIX_BASE, GeneratorDefect, asset_generator, curve_map, parse_generator,
    serialized_asset_generator,
};
use crate::commitment_oracle::vector::{
    CommitmentSource, PointMismatch, PublicCommitmentVector, ThreeWayComparison, compare_points,
};

/// One asset identifier the vendored tests publish a generator for.
fn asset(last: u8) -> [u8; 32] {
    let mut id = [0_u8; 32];
    id[31] = last;
    id
}

const ASSET_A: u8 = 1;
const ASSET_B: u8 = 2;

/// A blinding factor of one: thirty-one zero bytes then `0x01`.
fn blind_one() -> [u8; 32] {
    let mut blind = [0_u8; 32];
    blind[31] = 1;
    blind
}

fn blind_zero() -> [u8; 32] {
    [0_u8; 32]
}

fn blind_full() -> [u8; 32] {
    [0x42_u8; 32]
}

fn bytes(hexadecimal: &str) -> Vec<u8> {
    assert!(hexadecimal.len() % 2 == 0, "hex must be byte aligned");
    (0..hexadecimal.len() / 2)
        .map(|index| {
            u8::from_str_radix(&hexadecimal[index * 2..index * 2 + 2], 16)
                .expect("test vector hex is well formed")
        })
        .collect()
}

/// The hexadecimal form of a byte string, for a readable failure.
fn shown(value: &[u8]) -> String {
    use std::fmt::Write as _;

    value.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

fn big(hexadecimal: &str) -> BigUint {
    BigUint::parse_bytes(hexadecimal.as_bytes(), 16).expect("test constant is well formed")
}

// --- the published curve-map vectors ---------------------------------------

#[test]
fn the_curve_map_reproduces_the_published_degenerate_point() {
    // tests_impl.h:54 - t = 0, where the joint denominator vanishes and
    // the map yields the point at d.
    let point = curve_map(&BigUint::from(0_u32));
    assert_eq!(
        shown(&point.x_bytes()),
        "851695d49a83f8ef919bb86153cbcb16630fb68aed0a766a3ec693d68e6afa40"
    );
    assert_eq!(
        shown(&curve::field_bytes(point.y())),
        "4218f20ae6c646b363db68605822fb14264ca8d2587fdd6fbc750d587e76a7ee"
    );
    assert!(point.is_on_curve());
}

#[test]
fn the_curve_map_takes_the_sign_of_y_from_the_oddness_of_t() {
    // tests_impl.h:56-57 - t = 1 and t = -1 share an x coordinate and
    // carry negated y coordinates. The rule is the oddness of t, not its
    // Jacobi symbol, so the odd input is the one whose y is flipped away
    // from the square root the field returns.
    let positive = curve_map(&BigUint::from(1_u32));
    let negative = curve_map(&(curve::field_modulus() - BigUint::from(1_u32)));

    assert_eq!(positive.x_bytes(), negative.x_bytes());
    assert_eq!(
        shown(&positive.x_bytes()),
        "edd1fd3e327ce90cc7a3542614289aee9682003e9cf7dcc9cf2ca9743be5aa0c"
    );
    assert_eq!(
        shown(&curve::field_bytes(positive.y())),
        "0225f529ee75acafccfc456026c5e46bf80237a33924655a16f90e88085ed52a"
    );
    assert_eq!(
        shown(&curve::field_bytes(negative.y())),
        "fdda0ad6118a53503303ba9fd93a1b9407fdc85cc6db9aa5e906f176f7a12705"
    );

    // The root the field returns is itself always a square, so the even
    // input keeps it and the odd input does not.
    assert!(!positive.has_square_y());
    assert!(negative.has_square_y());
    assert_eq!(positive, negative.negate());
}

// --- the published generator vectors ---------------------------------------

#[test]
fn the_generators_reproduce_the_published_vectors_for_two_distinct_assets() {
    // tests_impl.h:111-112 - entries one and two of the published table.
    assert_eq!(
        shown(&serialized_asset_generator(&asset(ASSET_A)).expect("asset A has a generator")),
        "0b806cd8edd6c153e34aa9b9a08755c4be4718b1efb26cb93ffdd99e1b21f2af8e"
    );
    assert_eq!(
        shown(&serialized_asset_generator(&asset(ASSET_B)).expect("asset B has a generator")),
        "0ad91b15ec47a811f4aa189561d13f5c4d4e81f10dc7dc551f4fea9b84610314c4"
    );

    // The published table states both coordinates; the prefix above is
    // only the squareness of y, so the x coordinates are checked too.
    let first = asset_generator(&asset(ASSET_A)).expect("asset A has a generator");
    assert_eq!(
        shown(&curve::field_bytes(first.y())),
        "c7062208cc649a031bdc1a339d01f1154bcd0dcafe0b875d62f35f7328673006"
    );
    assert!(first.is_on_curve());
    assert!(!first.has_square_y());
}

#[test]
fn distinct_assets_have_distinct_generators() {
    let first = asset_generator(&asset(ASSET_A)).expect("asset A has a generator");
    let second = asset_generator(&asset(ASSET_B)).expect("asset B has a generator");
    assert_ne!(first, second);
    assert_ne!(first.x_bytes(), second.x_bytes());
}

// --- the two prefix conventions over one point ------------------------------

#[test]
fn one_point_encodes_differently_under_the_two_conventions() {
    // tests_impl.h:175-178 and :325-329 carry prefix 0x0b and 0x09 over
    // the x coordinate of the doubled base point. Both name the negation
    // of 2G, because an odd prefix negates the square-y point.
    let base = curve::base_point();
    let doubled = match curve::add(
        &curve::Group::Affine(base.clone()),
        &curve::Group::Affine(base),
    ) {
        curve::Group::Affine(point) => point,
        curve::Group::Identity => panic!("the base point is not its own negation"),
    };
    let published_x = "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";
    assert_eq!(shown(&doubled.x_bytes()), published_x);

    // 2G's own y is a square, so it takes the even-valued prefix under
    // both conventions.
    assert!(doubled.has_square_y());
    assert_eq!(
        shown(&encode_prefixed_point(&doubled, GENERATOR_PREFIX_BASE)),
        format!("0a{published_x}")
    );
    assert_eq!(
        shown(&encode_prefixed_point(&doubled, COMMITMENT_PREFIX_BASE)),
        format!("08{published_x}")
    );

    // The published fixtures are the negation, and they round-trip.
    let negated = doubled.negate();
    assert_eq!(
        shown(&encode_prefixed_point(&negated, GENERATOR_PREFIX_BASE)),
        format!("0b{published_x}")
    );
    assert_eq!(
        shown(&encode_prefixed_point(&negated, COMMITMENT_PREFIX_BASE)),
        format!("09{published_x}")
    );
    assert_eq!(
        parse_generator(&bytes(&format!("0b{published_x}"))).expect("published generator parses"),
        negated
    );
    assert_eq!(
        parse_commitment(&bytes(&format!("09{published_x}"))).expect("published commitment parses"),
        negated
    );
}

// --- the required commitment vectors ---------------------------------------

/// Every pinned commitment, as inputs and the expected encoding.
fn pinned_commitments() -> Vec<(&'static str, u8, u64, [u8; 32], &'static str)> {
    vec![
        // Amount zero under a blinder of one is exactly the base point,
        // which is the cleanest check that the terms are not swapped.
        (
            "amount zero",
            ASSET_A,
            0,
            blind_one(),
            "0879be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        ),
        (
            "amount one",
            ASSET_A,
            1,
            blind_one(),
            "08db1d086d75683cb608731feb1b7ca0479458af6c4e46dad55951533cfb5264ff",
        ),
        (
            "amount at the semantic bound less one",
            ASSET_A,
            (1_u64 << 51) - 1,
            blind_one(),
            "084bd29bf28762316374cbd0c5ac910ff47029bdee6fae15158a767dbb073299f6",
        ),
        // A zero blinder leaves the asset generator itself, which is how
        // an explicit amount joins the target's balance.
        (
            "zero blinding factor",
            ASSET_A,
            1,
            blind_zero(),
            "09806cd8edd6c153e34aa9b9a08755c4be4718b1efb26cb93ffdd99e1b21f2af8e",
        ),
        (
            "full-width blinding factor",
            ASSET_A,
            1,
            blind_full(),
            "09c48314cd41d2accfacf9935e4fef06f3739b7974f86c42fb8d40da1fb1625940",
        ),
        (
            "wrong asset, same amount and blinder",
            ASSET_B,
            1,
            blind_full(),
            "08711825033c27ccbb028fa39b257f767e80417bf313f12d3d9599e91c14353dd9",
        ),
        // The target admits the full unsigned range at this layer even
        // though the protocol's semantic domain stops at 2^51.
        (
            "amount above the semantic domain",
            ASSET_A,
            u64::MAX,
            blind_full(),
            "08b235f13b66d280b439c64f5a796bc974059e481c79df47632106fad8d2ba07ae",
        ),
    ]
}

#[test]
fn every_pinned_commitment_reproduces_exactly() {
    for (name, asset_last, amount, blind, expected) in pinned_commitments() {
        let produced = commitment(&asset(asset_last), amount, &blind)
            .unwrap_or_else(|defect| panic!("{name}: refused with {defect:?}"));
        assert_eq!(shown(&produced), expected, "{name}");
    }
}

#[test]
fn both_output_parities_occur_among_the_pinned_vectors() {
    let prefixes: Vec<u8> = pinned_commitments()
        .into_iter()
        .map(|(_, asset_last, amount, blind, _)| {
            commitment(&asset(asset_last), amount, &blind).expect("pinned vector commits")[0]
        })
        .collect();
    assert!(prefixes.contains(&0x08), "no square-y commitment pinned");
    assert!(
        prefixes.contains(&0x09),
        "no non-square-y commitment pinned"
    );
}

#[test]
fn a_zero_blinder_over_amount_one_is_the_asset_generator_itself() {
    let generator = asset_generator(&asset(ASSET_A)).expect("asset A has a generator");
    let committed =
        commitment(&asset(ASSET_A), 1, &blind_zero()).expect("a zero blinder is admitted");
    // The same point under the two conventions: 0x0b and 0x09 both
    // report a non-square y.
    assert_eq!(committed[1..], generator.x_bytes());
    assert_eq!(committed[0], 0x09);
    assert_eq!(
        serialized_asset_generator(&asset(ASSET_A)).expect("asset A has a generator")[0],
        0x0b
    );
}

#[test]
fn a_byte_reversed_blinder_is_a_different_commitment() {
    let mut reversed = blind_one();
    reversed.reverse();
    let straight = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");
    let flipped = commitment(&asset(ASSET_A), 1, &reversed).expect("commits");
    assert_ne!(straight, flipped);
    assert_eq!(
        shown(&flipped),
        "098317a743c90f41ea2dd8f3f3307d1027b97b338afd2088aab81365e444188a45"
    );
}

// --- the scalar and amount domains ------------------------------------------

#[test]
fn a_zero_scalar_is_admitted_and_the_identity_result_is_not() {
    // Zero is admitted deliberately, and consensus relies on it.
    assert!(commitment(&asset(ASSET_A), 1, &blind_zero()).is_ok());
    // Amount zero under a zero blinder lands on the identity, which the
    // target refuses rather than encoding.
    assert_eq!(
        commitment(&asset(ASSET_A), 0, &blind_zero()),
        Err(CommitmentDefect::IdentityResult)
    );
}

#[test]
fn a_scalar_at_or_above_the_group_order_is_refused() {
    let order = curve::group_order().clone();
    let at_order = curve::field_bytes(&order);
    assert_eq!(
        commitment(&asset(ASSET_A), 1, &at_order),
        Err(CommitmentDefect::Scalar(ScalarDefect::AtOrAboveGroupOrder))
    );

    let above = curve::field_bytes(&(order + BigUint::from(1_u32)));
    assert_eq!(
        commitment(&asset(ASSET_A), 1, &above),
        Err(CommitmentDefect::Scalar(ScalarDefect::AtOrAboveGroupOrder))
    );

    // One below the order is still admitted, so the boundary is exact.
    let below = curve::field_bytes(&(curve::group_order() - BigUint::from(1_u32)));
    assert!(commitment(&asset(ASSET_A), 1, &below).is_ok());
}

#[test]
fn a_wrong_width_scalar_is_refused_before_any_arithmetic() {
    for width in [0_usize, 31, 33, 64] {
        let blind = vec![0x11_u8; width];
        assert_eq!(
            commitment(&asset(ASSET_A), 1, &blind),
            Err(CommitmentDefect::Scalar(ScalarDefect::WrongWidth {
                expected: 32,
                found: width,
            })),
            "width {width}"
        );
    }
}

#[test]
fn a_wrong_width_asset_identifier_is_refused() {
    for width in [0_usize, 31, 33] {
        let id = vec![0x07_u8; width];
        assert_eq!(
            commitment(&id, 1, &blind_one()),
            Err(CommitmentDefect::Generator(
                GeneratorDefect::WrongIdentifierWidth {
                    expected: 32,
                    found: width,
                }
            )),
            "width {width}"
        );
    }
}

#[test]
fn the_semantic_amount_domain_is_narrower_than_what_the_oracle_computes() {
    assert!(is_semantic_amount(0));
    assert!(is_semantic_amount((1_u64 << 51) - 1));
    assert!(!is_semantic_amount(1_u64 << 51));
    assert!(!is_semantic_amount(u64::MAX));

    // Outside the semantic domain the oracle still computes, because the
    // target does: the bound is the protocol's, not the target's.
    assert!(commitment(&asset(ASSET_A), 1_u64 << 51, &blind_one()).is_ok());
    assert!(commitment(&asset(ASSET_A), u64::MAX, &blind_one()).is_ok());
}

// --- malformed encodings ----------------------------------------------------

#[test]
fn a_one_bit_mutation_of_the_x_coordinate_is_refused() {
    let mut mutated = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");
    mutated[32] ^= 0x01;
    assert_eq!(
        parse_commitment(&mutated),
        Err(PointEncodingDefect::XNotOnCurve)
    );
}

#[test]
fn a_one_bit_mutation_of_the_prefix_names_the_negated_point() {
    // The distinction matters: a flipped x is refused, while a flipped
    // parity bit is a well-formed encoding of a different point. Nothing
    // in the encoding binds the prefix to the commitment that was meant.
    let original = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");
    let mut mutated = original;
    mutated[0] ^= 0x01;

    let intended = parse_commitment(&original).expect("the original parses");
    let observed = parse_commitment(&mutated).expect("the parity flip still parses");
    assert_eq!(observed, intended.negate());
    assert_ne!(observed, intended);
}

#[test]
fn a_compressed_public_key_prefix_is_not_a_commitment_prefix() {
    // The encoding domains do not meet: 0x02 and 0x03 record the oddness
    // of y and belong to the curve primitives, not to a commitment.
    let commitment_bytes = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");
    for prefix in [0x02_u8, 0x03, 0x04, 0x0a, 0x0b, 0x00] {
        let mut malformed = commitment_bytes;
        malformed[0] = prefix;
        assert_eq!(
            parse_commitment(&malformed),
            Err(PointEncodingDefect::UnknownPrefix(prefix)),
            "prefix {prefix:#04x}"
        );
    }
    // And the generator prefixes refuse the commitment ones in turn.
    for prefix in [0x08_u8, 0x09] {
        let mut malformed = commitment_bytes;
        malformed[0] = prefix;
        assert_eq!(
            parse_generator(&malformed),
            Err(PointEncodingDefect::UnknownPrefix(prefix))
        );
    }
}

#[test]
fn an_x_only_encoding_is_not_a_commitment() {
    // An x-only key is thirty-two bytes and carries no prefix at all.
    let commitment_bytes = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");
    assert_eq!(
        parse_commitment(&commitment_bytes[1..]),
        Err(PointEncodingDefect::WrongWidth {
            expected: 33,
            found: 32,
        })
    );
}

#[test]
fn trailing_bytes_are_refused() {
    let commitment_bytes = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");
    let mut extended = commitment_bytes.to_vec();
    extended.push(0x00);
    assert_eq!(
        parse_commitment(&extended),
        Err(PointEncodingDefect::WrongWidth {
            expected: 33,
            found: 34,
        })
    );
}

#[test]
fn an_x_coordinate_at_or_above_the_field_modulus_is_refused() {
    let mut malformed = [0_u8; 33];
    malformed[0] = 0x08;
    malformed[1..].copy_from_slice(&curve::field_bytes(curve::field_modulus()));
    assert_eq!(
        parse_commitment(&malformed),
        Err(PointEncodingDefect::XNotAFieldElement)
    );
}

// --- the vector type and the comparison hook --------------------------------

#[test]
fn a_public_vector_verifies_against_the_recipe() {
    let vector = PublicCommitmentVector {
        asset_id: asset(ASSET_A),
        amount: 1,
        blinding_factor: blind_one(),
        expected_generator: bytes(
            "0b806cd8edd6c153e34aa9b9a08755c4be4718b1efb26cb93ffdd99e1b21f2af8e",
        ),
        expected_commitment: bytes(
            "08db1d086d75683cb608731feb1b7ca0479458af6c4e46dad55951533cfb5264ff",
        ),
    };
    assert_eq!(vector.verify(), Ok(()));
}

#[test]
fn a_vector_whose_pinned_commitment_drifted_is_refused() {
    let mut vector = PublicCommitmentVector {
        asset_id: asset(ASSET_A),
        amount: 1,
        blinding_factor: blind_one(),
        expected_generator: bytes(
            "0b806cd8edd6c153e34aa9b9a08755c4be4718b1efb26cb93ffdd99e1b21f2af8e",
        ),
        expected_commitment: bytes(
            "08db1d086d75683cb608731feb1b7ca0479458af6c4e46dad55951533cfb5264ff",
        ),
    };
    // A wrong amount must not be absorbed by recomputation.
    vector.amount = 2;
    assert!(matches!(
        vector.verify(),
        Err(crate::commitment_oracle::vector::VectorDefect::CommitmentMismatch(_))
    ));
}

#[test]
fn the_comparison_separates_a_parity_difference_from_a_different_point() {
    let expected = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");

    assert_eq!(compare_points(&expected, &expected), Ok(()));

    let mut parity = expected;
    parity[0] ^= 0x01;
    assert_eq!(
        compare_points(&expected, &parity),
        Err(PointMismatch::ParityPrefix {
            expected: 0x08,
            found: 0x09,
        })
    );

    let other = commitment(&asset(ASSET_B), 1, &blind_one()).expect("commits");
    assert!(matches!(
        compare_points(&expected, &other),
        Err(PointMismatch::Coordinate { .. })
    ));

    assert_eq!(
        compare_points(&expected, &expected[1..]),
        Err(PointMismatch::Width {
            expected: 33,
            found: 32,
        })
    );
}

#[test]
fn an_absent_leg_is_reported_as_absent_rather_than_as_agreement() {
    let expected = commitment(&asset(ASSET_A), 1, &blind_one()).expect("commits");

    let nothing_supplied = ThreeWayComparison::default().against(&expected);
    assert!(nothing_supplied.supplied_legs_agree());
    assert!(!nothing_supplied.complete_and_agreeing());
    assert_eq!(
        nothing_supplied.absent,
        vec![
            CommitmentSource::ConstructionLibrary,
            CommitmentSource::TargetIntrospection
        ]
    );

    let both_agree = ThreeWayComparison {
        construction_library: Some(expected.to_vec()),
        target_introspection: Some(expected.to_vec()),
    }
    .against(&expected);
    assert!(both_agree.complete_and_agreeing());

    let mut wrong = expected;
    wrong[0] ^= 0x01;
    let one_disagrees = ThreeWayComparison {
        construction_library: Some(expected.to_vec()),
        target_introspection: Some(wrong.to_vec()),
    }
    .against(&expected);
    assert!(!one_disagrees.supplied_legs_agree());
    assert_eq!(one_disagrees.disagreed.len(), 1);
    assert_eq!(
        one_disagrees.disagreed[0].source,
        CommitmentSource::TargetIntrospection
    );
}

// --- arithmetic sanity ------------------------------------------------------

#[test]
fn the_square_root_the_field_returns_is_itself_a_square() {
    // This is what makes the parity prefix recoverable: the even-valued
    // prefix always names the square-y point.
    for value in [1_u32, 2, 3, 4, 9, 16, 1_234_567] {
        let candidate = BigUint::from(value);
        if let Some(root) = curve::square_root(&candidate) {
            assert!(is_square(&root), "root of {value} is not itself a square");
            assert_eq!(curve::multiply(&root, &root), candidate);
        }
    }
}

#[test]
fn the_published_map_constants_are_what_they_claim() {
    // negc is the negation of a square root of -3, and d is (c-1)/2.
    let negated_c = big("F5D2D456CAF80E20DCC88F3D586869D339E092EA25EB132B8272D850E32A03DD");
    let c = curve::subtract(&BigUint::from(0_u32), &negated_c);
    assert_eq!(
        curve::multiply(&c, &c),
        curve::subtract(&BigUint::from(0_u32), &BigUint::from(3_u32))
    );

    let d = big("851695D49A83F8EF919BB86153CBCB16630FB68AED0A766A3EC693D68E6AFA40");
    let halved = curve::multiply(
        &curve::subtract(&c, &BigUint::from(1_u32)),
        &curve::invert(&BigUint::from(2_u32)),
    );
    assert_eq!(d, halved);
}

#[test]
fn the_base_point_is_on_the_curve_and_has_a_square_y() {
    let base = curve::base_point();
    assert!(base.is_on_curve());
    // Which is why a commitment that equals the base point takes 0x08.
    assert!(base.has_square_y());
    let reconstructed = Point::from_coordinates(base.x().clone(), base.y().clone());
    assert_eq!(reconstructed, base);
}
