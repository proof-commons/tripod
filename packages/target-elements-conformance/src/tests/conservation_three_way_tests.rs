//! The third leg of the Guide 11 §7.4 three-way comparison.
//!
//! # What these vectors are
//!
//! Every value below was **observed**, on a real `elementsregtest` node,
//! by the Wave-7 conservation run: the openings are what the target
//! reported for outputs it had just created, and the commitments are what
//! those transactions actually carry. They are pinned here so the
//! comparison is a standing test rather than a one-time observation on a
//! host nobody else has.
//!
//! # The comparison runs one way
//!
//! The oracle predicts a commitment from the openings, and the prediction
//! is checked against the observed bytes. Nothing here rewrites an
//! expectation to match an observation: the oracle's recipe is Wave 6's,
//! written before any of these bytes existed, and a disagreement would be
//! reported rather than absorbed `(´[PLAN-rule:guide11-exec:three-way]´)`.
//!
//! # Byte order, established rather than assumed
//!
//! The node prints BOTH an asset identifier and a blinding factor
//! reversed from the order the arithmetic reads them in: each is a
//! `uint256` whose printed form is the reverse of its stored bytes. The
//! convention was established by trying the four combinations against a
//! real observation, not assumed -- and only one reproduces the target's
//! commitment, which is what makes it a fact about the target rather than
//! a fitting parameter.
//!
//! This is why a naive comparison of these strings fails, and why the
//! failure is worth naming: it is a convention fault rather than a
//! different point, which is exactly the distinction `PointMismatch`
//! separates.

use crate::commitment_oracle::commitment::blinded_commitment;
use crate::commitment_oracle::vector::{CommitmentSource, ThreeWayComparison, compare_points};

/// One output the target created, as the target described it.
struct ObservedOpening {
    /// The row the output came from.
    row: &'static str,
    /// The asset identifier, in the order the node prints it.
    printed_asset: &'static str,
    /// The amount the node reported.
    amount: u64,
    /// The value blinding factor, as the node prints it.
    amount_blinder: &'static str,
    /// The asset blinding factor, as the node prints it.
    asset_blinder: &'static str,
    /// The value commitment the transaction actually carries.
    observed_commitment: &'static str,
}

/// The openings observed by the Wave-7 conservation run.
const OBSERVED: &[ObservedOpening] = &[
    ObservedOpening {
        row: "confidential-to-confidential-balanced",
        printed_asset: "b2e15d0d7a0c94e4e2ce0fe6e8691b9e451377f6e46e8045a86f7c4b5d4f0f23",
        amount: 10_000_000,
        amount_blinder: "7485c3c1373d74622d718e1a090140eb7bd6fae50cc7894574b86c1a517326b9",
        asset_blinder: "279c5df4f63be14a25c00f9b2051c268d7b7f092615ee01bdc62bb6ea85dd16e",
        observed_commitment: "08d91926a75ab4320d7d4b83f3dfb999bd69add91081c292dab2ae83559580e3a7",
    },
    ObservedOpening {
        row: "confidential-to-explicit-with-private-change",
        printed_asset: "b2e15d0d7a0c94e4e2ce0fe6e8691b9e451377f6e46e8045a86f7c4b5d4f0f23",
        amount: 5_000_000,
        amount_blinder: "a175461c8b5703c23e2a711d032338b28c6b32443f881f3c44648ff837e574b7",
        asset_blinder: "317b317d51f8cec315b4c2d80477fa9d784a89289269c9c1cb56b77125c51031",
        observed_commitment: "0851933b91dc2fb18fed2bb592644bdfc9185fbe17aa1c21ae1b2ab9cf21769b13",
    },
    ObservedOpening {
        row: "hidden-confidential-output",
        printed_asset: "b2e15d0d7a0c94e4e2ce0fe6e8691b9e451377f6e46e8045a86f7c4b5d4f0f23",
        amount: 5_000_000,
        amount_blinder: "0d4b47843d8442f8d04935ff03e6aa4399f8cddbe99274f6fada5effdfa4c3bd",
        asset_blinder: "80dda1333346ebd2918289a7440dcd323858e42952ec2b77d9a9bfac31c44a01",
        observed_commitment: "0849e232ade124da823b1a049f95d1fa5faf9d9570086675a9c35df672cdbffeae",
    },
];

/// Thirty-two bytes from sixty-four hex digits.
fn bytes32(text: &str) -> [u8; 32] {
    let raw = hex(text);
    let mut value = [0_u8; 32];
    value.copy_from_slice(&raw);
    value
}

/// The bytes sixty-four hex digits name.
fn hex(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "hex has whole bytes");
    (0..text.len() / 2)
        .map(|index| u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).expect("hex digit"))
        .collect()
}

/// One printed `uint256`, in the order the arithmetic reads it.
///
/// Applies to asset identifiers and to blinding factors alike: the node
/// prints both reversed from the order they are consumed in.
fn internal_order(printed: &str) -> [u8; 32] {
    let mut value = bytes32(printed);
    value.reverse();
    value
}

#[test]
fn the_oracle_predicts_every_observed_commitment() {
    for opening in OBSERVED {
        let predicted = blinded_commitment(
            &internal_order(opening.printed_asset),
            opening.amount,
            &internal_order(opening.asset_blinder),
            &internal_order(opening.amount_blinder),
        )
        .unwrap_or_else(|defect| {
            panic!("the oracle refused the {} opening: {defect:?}", opening.row)
        });

        let observed = hex(opening.observed_commitment);
        if let Err(mismatch) = compare_points(&predicted, &observed) {
            panic!(
                "the oracle and the target disagree on {}: {mismatch:?}\n  \
                 predicted {}\n  observed  {}",
                opening.row,
                predicted
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                opening.observed_commitment,
            );
        }
    }
}

#[test]
fn the_three_way_comparison_records_the_target_leg_as_agreeing() {
    // The comparison interface Wave 6 wrote, with the leg Wave 6 could
    // not supply. The construction-library leg stays absent and is
    // reported as absent rather than as agreement: no third-party library
    // built these transactions, the node did.
    let opening = &OBSERVED[0];
    let predicted = blinded_commitment(
        &internal_order(opening.printed_asset),
        opening.amount,
        &internal_order(opening.asset_blinder),
        &internal_order(opening.amount_blinder),
    )
    .expect("the oracle predicts the observed opening");

    let comparison = ThreeWayComparison {
        construction_library: None,
        target_introspection: Some(hex(opening.observed_commitment)),
    };
    let outcome = comparison.against(&predicted);

    assert_eq!(outcome.agreed, vec![CommitmentSource::TargetIntrospection]);
    assert!(outcome.disagreed.is_empty());
    assert_eq!(outcome.absent, vec![CommitmentSource::ConstructionLibrary]);
    assert!(outcome.supplied_legs_agree());
    // Not complete, and it must not claim to be: one leg never arrived.
    assert!(!outcome.complete_and_agreeing());
}
