//! The four serialization changes of the confidential-funding guide's
//! freeze rule, each with its round-trip test.
//!
//! # What a round-trip test means here
//!
//! The encoding law the guide states is exact: decoding a candidate's
//! bytes and re-encoding them reproduces those bytes, including nonempty
//! range proofs and empty surjection proofs. Every test below that calls
//! itself a round trip checks that identity on real bytes rather than
//! checking that two typed values compare equal — the law is about bytes,
//! and a comparison of two values built by the same constructor could not
//! have caught the census bug it exists to prevent.
//!
//! # The four changes, and where each is checked
//!
//! The output-witness census, the witness-flag predicate, the
//! superfluous-section condition, and the form-conditional proof refusals
//! are four separate rows of the guide and are checked in four separate
//! sections below. The form-conditional row's *forbidding* half already
//! has standing tests in the preflight file and is not restated here; its
//! *requiring* half is new with this wave and is here.
//!
//! # Nothing here computes a commitment
//!
//! The commitments below are byte strings with an admitted prefix and
//! filler after it. Whether they are points is a question about
//! arithmetic this file does not do and does not need: the serializer's
//! rules are about prefixes, lengths, and positions, and a real point
//! would test the same bytes more slowly.

use crate::bytes::{
    AssetField, AssetId, COMMITMENT_BYTES, InputWitness, NULL_PREFIX, NonceField, OutputWitness,
    TargetInput, TargetOutput, TargetTransaction, VALUE_COMMITMENT_PREFIXES, ValueField,
};
use crate::error::TransactionRefusal;

use super::outpoint;

/// The asset every fixture output carries explicitly.
const FIXTURE_ASSET: [u8; 32] = [0xf1; 32];

/// A witness program the fixture outputs pay to.
const FIXTURE_PROGRAM: [u8; 32] = [0xf2; 32];

/// A range proof, as a byte string of the right shape and no meaning.
const FIXTURE_RANGE_PROOF: [u8; 96] = [0xab; 96];

/// A committed value field carrying the admitted prefix at `parity`.
fn committed(parity: usize) -> ValueField {
    let mut commitment = [0xc7_u8; COMMITMENT_BYTES];
    commitment[0] = VALUE_COMMITMENT_PREFIXES[parity];
    ValueField::Commitment(commitment)
}

/// A nonempty input witness, so the encoder writes the section.
fn nonnull_witness() -> InputWitness {
    InputWitness::new(vec![vec![0x30, 0x44, 0x01], vec![0x02, 0x0a]])
}

/// One output paying the fixture program, with `value` as its value.
fn fixture_output(value: ValueField) -> TargetOutput {
    let mut program = vec![0x51, 0x20];
    program.extend_from_slice(&FIXTURE_PROGRAM);
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(FIXTURE_ASSET)),
        value,
        NonceField::Null,
        program,
    )
}

/// A one-in two-out transaction carrying these values and witnesses.
fn transaction(
    values: [ValueField; 2],
    witness: InputWitness,
    output_witnesses: [OutputWitness; 2],
) -> TargetTransaction {
    TargetTransaction::with_output_witnesses(
        3,
        vec![TargetInput::new(outpoint(0xe1, 0), 0xffff_ffff)],
        values.into_iter().map(fixture_output).collect(),
        0,
        vec![witness],
        output_witnesses.into(),
    )
    .expect("the fixture censuses are one per input and one per output")
}

/// The hybrid candidate: two committed values, two range proofs, and no
/// input witness at all, which is the shape a funding transaction has
/// before anything signs it.
fn hybrid_candidate() -> TargetTransaction {
    transaction(
        [committed(0), committed(1)],
        InputWitness::default(),
        [
            OutputWitness::range_proof_only(FIXTURE_RANGE_PROOF.to_vec()),
            OutputWitness::range_proof_only(FIXTURE_RANGE_PROOF.to_vec()),
        ],
    )
}

/// The explicit candidate: two explicit values and no proofs anywhere.
fn explicit_candidate() -> TargetTransaction {
    transaction(
        [ValueField::Explicit(600), ValueField::Explicit(400)],
        nonnull_witness(),
        [OutputWitness::empty(), OutputWitness::empty()],
    )
}

// --- Row one: the output-witness census --------------------------------

/// A candidate carrying range proofs round-trips its exact bytes.
///
/// The guide's exit condition for this wave, at the serialization layer:
/// a proof-finalized candidate's bytes decode and re-encode to themselves.
/// Both proofs are checked by value after the round trip, because a
/// decoder that read the vector positionally but bound it to the wrong
/// output would still produce a byte-identical re-encoding.
#[test]
fn a_proof_bearing_candidate_round_trips_exact_bytes() {
    let candidate = hybrid_candidate();
    let bytes = candidate.encode();
    let decoded = TargetTransaction::decode(&bytes).expect("the hybrid candidate decodes");

    assert_eq!(decoded.encode(), bytes, "the bytes survive the round trip");
    assert_eq!(decoded, candidate, "and so does every typed member");
    assert_eq!(
        decoded.output_witnesses().len(),
        decoded.outputs().len(),
        "the output-witness census is one entry per output",
    );
    for witness in decoded.output_witnesses() {
        assert_eq!(
            witness.range_proof(),
            FIXTURE_RANGE_PROOF,
            "each output carries its own range proof",
        );
        assert!(
            witness.surjection_proof().is_empty(),
            "and an empty surjection proof, which the hybrid form requires",
        );
    }
}

/// The census is one entry per output or it is a refusal.
///
/// The input side's rule, applied to the side that gained one. A vector
/// of a different length would bind a proof to the wrong output and still
/// serialize.
#[test]
fn the_output_witness_census_refuses_a_wrong_length() {
    let refusal = TargetTransaction::with_output_witnesses(
        3,
        vec![TargetInput::new(outpoint(0xe1, 0), 0xffff_ffff)],
        vec![fixture_output(committed(0)), fixture_output(committed(1))],
        0,
        vec![InputWitness::default()],
        vec![OutputWitness::range_proof_only(
            FIXTURE_RANGE_PROOF.to_vec(),
        )],
    )
    .expect_err("one witness for two outputs is not a census");

    assert_eq!(
        refusal,
        TransactionRefusal::OutputWitnessCensusMismatch {
            outputs: 2,
            output_witnesses: 1,
        },
        "the refusal names both counts",
    );
}

/// An explicit candidate's bytes are unchanged by the census existing.
///
/// A regression guard on a settled lane rather than a new property. The
/// encoder used to write two `NULL_PREFIX` bytes per output; it now
/// writes two empty length prefixes, and those are the same two bytes. A
/// wave that changed the explicit lane's bytes while adding a
/// confidential one would have moved a signature's preimage under every
/// existing owner.
#[test]
fn explicit_candidate_bytes_are_unchanged_by_the_census() {
    let candidate = explicit_candidate();
    let bytes = candidate.encode();
    let tail = 2 * candidate.outputs().len();

    assert_eq!(
        bytes[bytes.len() - tail..],
        vec![NULL_PREFIX; tail],
        "an all-empty output-witness vector is still two zero bytes per output",
    );
    assert_eq!(
        TargetTransaction::decode(&bytes)
            .expect("the explicit candidate decodes")
            .encode(),
        bytes,
        "and the explicit lane's round-trip law is untouched",
    );
}

// --- Row two: the witness-flag predicate --------------------------------

/// A candidate with null input witnesses and range proofs is serialized
/// with its witness section.
///
/// The predicate's whole reason. A confidential funding transaction has
/// nothing signed yet, so every input witness is null; under the old
/// conjunction the flag byte would have been the witnessless one and the
/// proofs would not have been written at all. What the test compares is
/// the two serializations: they must differ, and the difference must
/// contain the proof.
#[test]
fn the_witness_flag_follows_an_output_proof_with_no_input_witness() {
    let candidate = hybrid_candidate();

    assert!(
        candidate.witnesses().iter().all(InputWitness::is_null),
        "the candidate is unsigned, so no input witness carries anything",
    );
    assert!(
        candidate.has_witness(),
        "and it still has a witness, because its outputs carry proofs",
    );

    let full = candidate.encode();
    let stripped = candidate.encode_without_witness();
    assert_ne!(full, stripped, "the two serializations diverge");
    assert!(
        full.windows(FIXTURE_RANGE_PROOF.len())
            .any(|window| window == FIXTURE_RANGE_PROOF),
        "and the full one contains the range proof",
    );
    assert!(
        !stripped
            .windows(FIXTURE_RANGE_PROOF.len())
            .any(|window| window == FIXTURE_RANGE_PROOF),
        "while the stripped one is exactly the case that omits it",
    );
}

/// A candidate with neither kind of witness still has none.
///
/// The predicate is a disjunction, not an assertion that everything has a
/// witness. An explicit unsigned transaction is serialized without the
/// section exactly as before.
#[test]
fn a_candidate_with_neither_witness_has_none() {
    let candidate = transaction(
        [ValueField::Explicit(600), ValueField::Explicit(400)],
        InputWitness::default(),
        [OutputWitness::empty(), OutputWitness::empty()],
    );

    assert!(!candidate.has_witness(), "nothing carries anything");
    assert_eq!(
        candidate.encode(),
        candidate.encode_without_witness(),
        "so the two serializations are one",
    );
}

// --- Row three: the superfluous-section condition -----------------------

/// A flagged section carrying only an output proof is not superfluous.
///
/// The condition's repair, checked on the bytes the encoder actually
/// writes: every input witness is null, so the old condition would have
/// refused this byte string, and the transaction it holds is one whose
/// proofs are the only thing in its witness section.
#[test]
fn a_flagged_section_carrying_only_an_output_proof_is_admitted() {
    let bytes = hybrid_candidate().encode();
    let decoded = TargetTransaction::decode(&bytes)
        .expect("a section carrying proofs is not a superfluous one");

    assert!(
        decoded.witnesses().iter().all(InputWitness::is_null),
        "the admitted section's input witnesses are all null",
    );
    assert_eq!(decoded.encode(), bytes, "and the bytes round-trip");
}

/// A flagged section carrying nothing at all is still superfluous.
///
/// The half of the old law that survives, and the reason the repair is a
/// widening rather than a weakening: a byte string whose two witness
/// vectors are both empty has one spelling, and it is the witnessless
/// one.
#[test]
fn a_flagged_section_carrying_nothing_is_still_superfluous() {
    let candidate = transaction(
        [ValueField::Explicit(600), ValueField::Explicit(400)],
        InputWitness::default(),
        [OutputWitness::empty(), OutputWitness::empty()],
    );

    // The flagged spelling of a transaction the encoder writes without a
    // section: the same bytes with the flag set and the empty section
    // appended, which is where the target writes it — after the lock
    // time, not before it.
    let mut flagged = candidate.encode_without_witness();
    flagged[4] = 0x01;
    for _ in candidate.inputs() {
        // Two empty issuance proofs, an empty stack, an empty peg-in.
        flagged.extend_from_slice(&[NULL_PREFIX; 4]);
    }
    for _ in candidate.outputs() {
        // An empty surjection proof and an empty range proof.
        flagged.extend_from_slice(&[NULL_PREFIX; 2]);
    }

    assert_eq!(
        TargetTransaction::decode(&flagged).err(),
        Some(TransactionRefusal::SuperfluousWitnessRecord),
        "a section carrying nothing has no business being flagged",
    );
}

// --- Row four: the form-conditional proof refusals ----------------------

/// The decoder refuses a dropped range proof.
///
/// The requiring half of the form-conditional rule. A committed value
/// with no range proof is a transaction the target refuses as invalid, so
/// a decoder that returned it would be handing back an unacceptable
/// transaction and calling it well formed. The bytes are the candidate's
/// own with one output's proof emptied, so nothing else about them
/// changed.
#[test]
fn the_decoder_refuses_a_dropped_range_proof() {
    let dropped = transaction(
        [committed(0), committed(1)],
        nonnull_witness(),
        [
            OutputWitness::range_proof_only(FIXTURE_RANGE_PROOF.to_vec()),
            OutputWitness::empty(),
        ],
    );

    assert_eq!(
        TargetTransaction::decode(&dropped.encode()).err(),
        Some(TransactionRefusal::RangeProofRequired { output: 1 }),
        "the refusal names the output whose proof was dropped",
    );
}

/// The refusal names the first output that dropped one.
///
/// Position matters: a report naming output zero for a proof dropped at
/// output one would send a reader to the wrong field.
#[test]
fn the_dropped_range_proof_refusal_names_its_output() {
    let dropped = transaction(
        [committed(0), committed(1)],
        nonnull_witness(),
        [
            OutputWitness::empty(),
            OutputWitness::range_proof_only(FIXTURE_RANGE_PROOF.to_vec()),
        ],
    );

    assert_eq!(
        TargetTransaction::decode(&dropped.encode()).err(),
        Some(TransactionRefusal::RangeProofRequired { output: 0 }),
        "the first offending position, not the last",
    );
}

/// An explicit output carrying a range proof is still refused.
///
/// The forbidding half, restated on the census's own encoder rather than
/// on hand-spliced bytes. The preflight file establishes the row; this
/// checks that the repair did not open the other direction while closing
/// this one.
#[test]
fn an_explicit_output_carrying_a_range_proof_is_refused() {
    let wrong = transaction(
        [ValueField::Explicit(600), ValueField::Explicit(400)],
        nonnull_witness(),
        [
            OutputWitness::range_proof_only(FIXTURE_RANGE_PROOF.to_vec()),
            OutputWitness::empty(),
        ],
    );

    assert_eq!(
        TargetTransaction::decode(&wrong.encode()).err(),
        Some(TransactionRefusal::RangeProofRefused),
        "an explicit value admits no range proof",
    );
}

/// A surjection proof is refused whatever the value form.
///
/// The asymmetry the guide records: the surjection refusal was already
/// the form-conditional answer for every form this workspace builds, and
/// only the range-proof half moved. Both forms are checked so that the
/// claim is measured rather than asserted.
#[test]
fn a_surjection_proof_is_refused_for_either_value_form() {
    for values in [
        [committed(0), committed(1)],
        [ValueField::Explicit(600), ValueField::Explicit(400)],
    ] {
        let carrying = transaction(
            values,
            nonnull_witness(),
            [
                OutputWitness::new(vec![0xcd; 32], FIXTURE_RANGE_PROOF.to_vec()),
                OutputWitness::empty(),
            ],
        );

        assert_eq!(
            TargetTransaction::decode(&carrying.encode()).err(),
            Some(TransactionRefusal::SurjectionProofRefused),
            "an explicit asset admits no surjection proof under either value form",
        );
    }
}
