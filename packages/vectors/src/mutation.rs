//! Focused mutations of an accepted transaction, for the negative half.
//!
//! Guide-12 §19.2's negative requirements need a transaction the target
//! refuses, and the safe constructor cannot produce one: refusing to
//! emit an invalid transaction is what makes it safe. So a negative
//! vector is made the only other way it can be — by taking a
//! transaction the target *accepted* and changing exactly one thing
//! about it.
//!
//! # Why the un-mutated sibling is the whole argument
//!
//! A refusal only means something if it is attributable. Given a
//! transaction the target took and the same transaction with one
//! structural change, a refusal of the second is caused by that change
//! or by nothing at all. That argument fails the moment the surgery
//! introduces a second difference — a re-encoding that normalizes a
//! field, a witness that shifts, a length prefix that moves — because
//! then the refusal has two possible causes and the vector establishes
//! neither.
//!
//! So this module refuses to mutate a transaction it cannot reproduce.
//! [`round_trips`] decodes and re-encodes with no change at all and
//! compares bytes; every mutation runs it first, and a vector that does
//! not survive it is refused rather than mutated. That check is the
//! reason the rest of the module is allowed to describe its output as
//! "one change".
//!
//! # Nothing here decides what a refusal proves
//!
//! A mutation names the §18 class it stages and nothing more. Whether
//! the layer the target answered at is the layer that class expects is
//! a comparison made against an observed transcript, never here, and a
//! mutation that produced an acceptance is a finding this module has no
//! opinion about.

use transaction::{AssetField, AssetId, TargetTransaction};

use crate::error::VectorError;
use crate::materialize::{MaterializedTargetVector, TargetVectorId};

/// Whether a transaction survives a decode and re-encode unchanged.
///
/// The precondition of every mutation here. It is not a claim about
/// this workspace's encoder in the abstract — it is a property of these
/// exact bytes, asked about each vector rather than assumed once,
/// because a transaction carrying a field the decoder normalized would
/// pass in general and fail on the one row that mattered.
///
/// # Errors
///
/// [`VectorError::UnmutatableVector`] when the bytes do not decode, or
/// decode to something that re-encodes differently.
pub fn round_trips(vector: &MaterializedTargetVector) -> Result<TargetTransaction, VectorError> {
    decode_exactly(vector.bytes(), vector.id())
}

/// The same check, against bytes a run observed rather than a vector
/// this package materialized.
///
/// # Errors
///
/// [`VectorError::UnmutatableVector`] on either failure.
pub fn decode_exactly(bytes: &[u8], id: TargetVectorId) -> Result<TargetTransaction, VectorError> {
    let decoded =
        TargetTransaction::decode(bytes).map_err(|_| VectorError::UnmutatableVector(id))?;
    if decoded.encode() != bytes {
        return Err(VectorError::UnmutatableVector(id));
    }
    Ok(decoded)
}

/// Which outputs carry one explicit asset.
///
/// The successor is the closed-asset output the covenant settles, and
/// finding it by asset rather than by position keeps a mutation from
/// depending on a layout the ABI is free to rearrange.
#[must_use]
pub fn outputs_carrying(transaction: &TargetTransaction, asset: [u8; 32]) -> Vec<usize> {
    let wanted = AssetField::Explicit(AssetId::from_internal(asset));
    transaction
        .outputs()
        .iter()
        .enumerate()
        .filter(|(_, output)| output.asset() == wanted)
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::round_trips;
    use crate::bundle::fixture_bundle;
    use crate::plan::derive_evidence_plan;

    #[test]
    fn every_materialized_vector_survives_a_decode_and_re_encode() {
        // The soundness gate of the whole negative half, checked against
        // every vector this plan holds bytes for. A vector that failed
        // here could still be mutated, and the mutation would carry a
        // second difference nobody chose — so the refusal it produced
        // would be attributable to nothing in particular.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = derive_evidence_plan(&bundle).expect("the evidence plan derives");
        assert!(
            !plan.target_cases().is_empty(),
            "the plan materialized nothing to check",
        );
        for subject in plan.target_cases() {
            let vector = subject.subject();
            let decoded = round_trips(vector)
                .unwrap_or_else(|_| panic!("{:?} does not round-trip", vector.id()));
            // And the decode is not vacuous: it read the shape the
            // vector claims, so a decoder that returned an empty
            // transaction could not pass this.
            assert_eq!(
                decoded.inputs().len(),
                usize::from(vector.id().ash_inputs()) + usize::from(vector.id().sponsors()),
                "{:?} decoded to another input census",
                vector.id(),
            );
        }
    }
}
