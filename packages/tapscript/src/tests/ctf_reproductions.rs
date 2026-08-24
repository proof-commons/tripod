//! Confidential-funding guide, Wave 0 preflight reproductions owned by
//! this crate.
//!
//! One of the intermediate confidential-funding guide's four preflight
//! rows lands here: the dimension the owner profile maps
//! [`ProtectedDatum::ProofFields`] to. The guide reads the tree as
//! mapping it to [`SighashDimension::SpentOutputs`] alone — naming the
//! input side's anchoring and not the created outputs' proofs — and
//! states the repair as widening the coverage to name the created
//! outputs' dimension as well.
//!
//! The convention is the one the Guide-13 reproductions in this crate
//! established: a test that asserts the repaired property and carries
//! `#[ignore]` is a row Wave 0 CONFIRMED, and the observed failure of
//! running it is that row's reproduction. Run them with
//! `cargo test -p tripod-tapscript -- --ignored`.
//!
//! The running test beside it is deliberately written so that it
//! survives the repair: the spent-output anchoring the guide's repair
//! keeps is stated as a membership rather than as an equality, so
//! widening the coverage does not falsify it.
//!
//! Nothing here implements anything. Wave 0 adds no production code.

use std::collections::BTreeSet;

use target_elements::SighashDimension;

use crate::authorization::{DimensionRole, ProtectedDatum, selected_owner_profile};

/// Every dimension the selected profile's coverage map associates with
/// `datum`.
///
/// Read through the public coverage iterator rather than through
/// `carrier`, which answers with one dimension by signature. Collecting
/// the pairs is what lets this file state "includes" and "does not
/// include" separately, and it is what keeps both tests below readable
/// against a repaired coverage that names more than one dimension per
/// datum.
fn carriers_of(datum: ProtectedDatum) -> BTreeSet<SighashDimension> {
    selected_owner_profile()
        .coverage()
        .filter(|(covered, _)| *covered == datum)
        .map(|(_, dimension)| dimension)
        .collect()
}

/// The proof fields are carried by the created outputs' dimension as
/// well as the spent outputs'.
///
/// The guide's repair, stated as the property rather than as an edit.
/// The target rule the concept verified is that `SIGHASH_ALL`
/// incorporates the serialized output set and the hash of the
/// output-witness vector, so a transfer's own created outputs carry
/// proof fields the owner's signature commits to. A coverage map that
/// names only the spent outputs declares less protection than the
/// target actually gives, and a later narrowing of the profile would be
/// argued against the declaration rather than against the rule.
///
/// Ignored while the coverage names one dimension: the observed failure
/// is this row's reproduction.
#[test]
#[ignore = "confidential-funding preflight: the coverage names the spent outputs alone today; run with -- --ignored"]
fn the_proof_fields_are_carried_by_the_created_outputs_dimension_as_well() {
    let carriers = carriers_of(ProtectedDatum::ProofFields);

    assert!(
        carriers.contains(&SighashDimension::AllOutputs),
        "the created outputs' proof fields are protected data too, and were carried by {carriers:?}",
    );
}

/// The proof fields are carried by the spent outputs' dimension.
///
/// A standing guarantee rather than a defect: the input side's
/// anchoring is real, the guide's repair widens the coverage rather
/// than moving it, and this membership is owed afterwards exactly as it
/// is met today. Written as a membership for that reason.
#[test]
fn the_proof_fields_are_carried_by_the_spent_outputs_dimension() {
    let carriers = carriers_of(ProtectedDatum::ProofFields);

    assert!(
        carriers.contains(&SighashDimension::SpentOutputs),
        "the spent outputs anchor the input side's proofs, and were carried by {carriers:?}",
    );
}

/// The created outputs' dimension is one the profile requires.
///
/// The measurement that says the repair is a coverage change and not a
/// profile change: the dimension the proof fields must also land on is
/// already required by the selected profile, so widening the coverage
/// asks nothing new of the target and lands no datum on a refused
/// dimension. A row whose repair had needed a refused dimension would
/// be a different and much larger finding.
#[test]
fn the_created_outputs_dimension_is_already_required() {
    assert_eq!(
        selected_owner_profile().role(SighashDimension::AllOutputs),
        Some(DimensionRole::Required),
        "the profile already commits to the whole output list",
    );
}
