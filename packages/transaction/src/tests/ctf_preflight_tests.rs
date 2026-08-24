//! Confidential-funding guide, Wave 0 preflight reproductions owned by
//! this crate.
//!
//! # What these tests are for
//!
//! The intermediate confidential-funding execution guide opens its
//! repository ground truth with four rows it calls PREFLIGHT FINDINGS:
//! claims about this tree that its own reading produced, stated as
//! hypotheses rather than as established defects. Wave 0's whole job is
//! to give each row a disposition by running code. Three of the four are
//! owned by this crate and are reproduced here.
//!
//! # Which form each test takes, and why
//!
//! Two forms appear below, and the difference is not stylistic.
//!
//! A row the guide states a REPAIR for gets a test that asserts the
//! repaired property and carries `#[ignore]` while this tree contradicts
//! it — the convention the Guide-13 reproductions in this workspace
//! already established. Such a test fails when it is run, and that
//! observed failure is the row's reproduction; the wave that repairs the
//! row removes the attribute rather than writing a new test. Running
//! them is `cargo test -p tripod-transaction -- --ignored`.
//!
//! A row that is a CHARACTERIZATION rather than a defect — the guide
//! asks what a role can and cannot do, not what is wrong with it — gets
//! ordinary running tests that record the measured limits. A running
//! test here is therefore not automatically a refuted row, which it is
//! in the Guide-13 files; the section headings below say which is which,
//! and no test carries a disposition it did not earn.
//!
//! Some assertions are written so that they survive the repair. Where
//! that was possible it is deliberate: a fact about the encoder, or a
//! refusal the repaired behaviour still owes, is worth keeping as a
//! standing guarantee rather than deleting on the day the row closes.
//!
//! # Nothing here implements anything
//!
//! Wave 0's exit is that nothing is implemented and every finding has a
//! disposition. These tests add no production code, change no
//! production behaviour, and mint no vocabulary.

use std::cell::RefCell;

use linker::live_backend::LiveTransferRepresentationPlan;

use super::live_support::{
    FIRST_OWNER, FixturePrivateValue, LIVE_PROTOCOL_ASSET, PUBLISHED_RANDOMNESS, SECOND_OWNER,
    live_abi, owner, receipt_view,
};
use super::{outpoint, reviewed_target, view};
use crate::bytes::{
    AssetField, AssetId, COMMITMENT_BYTES, InputWitness, NULL_PREFIX, NonceField, TargetInput,
    TargetOutput, TargetTransaction, VALUE_COMMITMENT_PREFIXES, ValueField,
};
use crate::error::TransactionRefusal;
use crate::live_construct::{LiveFinalization, finalize_live_transfer};
use crate::live_private::PrivateValueCapability;
use crate::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};

/// The asset every fixture output carries explicitly.
const FIXTURE_ASSET: [u8; 32] = [0xf1; 32];

/// A witness program the fixture outputs pay to.
const FIXTURE_PROGRAM: [u8; 32] = [0xf2; 32];

/// A nonempty input witness, so that the encoder writes the witness
/// section at all.
///
/// Without it `has_witness` is false, the flag byte is the witnessless
/// one, and there is no output-witness region to put a proof in — which
/// is a fact about this crate worth stating once here rather than
/// rediscovering in each test below.
fn nonnull_witness() -> InputWitness {
    InputWitness::new(vec![vec![0x30, 0x44, 0x01], vec![0x02, 0x0a]])
}

/// One output paying the fixture program, with `value` as its value
/// field.
///
/// The asset stays explicit and the nonce stays null in both cases, so
/// that the only difference between the two fixture transactions below
/// is the one the refusals are being asked about.
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

/// A one-in one-out transaction whose single output carries `value`.
fn fixture_transaction(value: ValueField) -> TargetTransaction {
    TargetTransaction::new(
        3,
        vec![TargetInput::new(outpoint(0xe1, 0), 0xffff_ffff)],
        vec![fixture_output(value)],
        0,
        vec![nonnull_witness()],
    )
    .expect("the fixture census is one witness per input")
}

/// The explicit form: an explicit asset and an explicit value.
fn explicit_form() -> TargetTransaction {
    fixture_transaction(ValueField::Explicit(1_000))
}

/// The guide's hybrid form: an explicit asset and a committed value.
///
/// The prefix is the first of the two the target admits. Which parity a
/// real commitment carries is a property of a point this crate does not
/// compute, and nothing here claims the bytes are one.
fn hybrid_form() -> TargetTransaction {
    let mut commitment = [0_u8; COMMITMENT_BYTES];
    commitment[0] = VALUE_COMMITMENT_PREFIXES[0];
    commitment[1..].copy_from_slice(&[0xc7; COMMITMENT_BYTES - 1]);
    fixture_transaction(ValueField::Commitment(commitment))
}

/// `transaction`'s own bytes with each output's proof pair replaced.
///
/// The encoder writes two empty length prefixes per output and offers no
/// way to write anything else, so the fixture bytes are assembled by
/// dropping that region and writing the offered proofs in its place.
/// Both payloads stay short enough for a one-byte length prefix, which
/// is why no compact-size helper is needed.
fn with_output_proofs(
    transaction: &TargetTransaction,
    surjection: &[u8],
    rangeproof: &[u8],
) -> Vec<u8> {
    assert!(
        surjection.len() < 0xfd && rangeproof.len() < 0xfd,
        "the fixture proofs stay inside the one-byte length prefix",
    );
    let surjection_length = u8::try_from(surjection.len()).expect("the fixture proof is short");
    let rangeproof_length = u8::try_from(rangeproof.len()).expect("the fixture proof is short");
    let mut bytes = transaction.encode();
    let written = 2 * transaction.outputs().len();
    bytes.truncate(bytes.len() - written);
    for _ in transaction.outputs() {
        bytes.push(surjection_length);
        bytes.extend_from_slice(surjection);
        bytes.push(rangeproof_length);
        bytes.extend_from_slice(rangeproof);
    }
    bytes
}

// --- Row: the two proof-field refusals ---------------------------------
//
// The guide's row: `RangeProofRefused` and `SurjectionProofRefused` both
// currently fire on any nonempty proof field, and must become
// form-conditional — refusing a rangeproof where the profile forbids
// one, and a surjection proof always for the hybrid form.

/// The rangeproof refusal must consult the output's value form.
///
/// The repaired property in one test, both halves at once, because a
/// refusal that fired for neither form would satisfy half of it and be
/// worse than today. The hybrid form the guide's §7 fixes — explicit
/// asset, committed value, empty surjection field — REQUIRES a
/// rangeproof at the target, so bytes carrying one must decode; the
/// explicit form forbids one, so bytes carrying one must still refuse.
///
/// Ignored while the tree refuses both: the observed failure is this
/// row's reproduction.
#[test]
#[ignore = "confidential-funding preflight: the refusal is unconditional today; run with -- --ignored"]
fn the_rangeproof_refusal_consults_the_output_value_form() {
    let hybrid = hybrid_form();
    let hybrid_bytes = with_output_proofs(&hybrid, &[], &[0xab; 64]);

    assert_eq!(
        TargetTransaction::decode(&hybrid_bytes).err(),
        None,
        "the hybrid form's mandatory rangeproof must decode",
    );

    let explicit = explicit_form();
    let explicit_bytes = with_output_proofs(&explicit, &[], &[0xab; 64]);

    assert_eq!(
        TargetTransaction::decode(&explicit_bytes).err(),
        Some(TransactionRefusal::RangeProofRefused),
        "an explicit value forbids a rangeproof, so this one is still refused",
    );
}

/// A rangeproof is refused for the explicit form.
///
/// The other half of the row's claim, observed rather than inferred: the
/// refusal the hybrid form meets is the same one the explicit form
/// meets, which is what "fires on any nonempty proof field" means. It is
/// also a standing guarantee — an explicit value forbids a rangeproof
/// after the repair exactly as it does today — so this half runs while
/// the half above stays ignored.
#[test]
fn a_rangeproof_is_refused_for_the_explicit_form() {
    let explicit = explicit_form();
    let bytes = with_output_proofs(&explicit, &[], &[0xab; 64]);

    assert_eq!(
        TargetTransaction::decode(&bytes).err(),
        Some(TransactionRefusal::RangeProofRefused),
        "an explicit value admits no rangeproof",
    );
}

/// A surjection proof is refused for the hybrid form.
///
/// A standing guarantee rather than a defect: the target requires the
/// surjection field to be EMPTY where a confidential value is paired
/// with an explicit asset, so the repaired behaviour owes this refusal
/// exactly as today's unconditional one delivers it. The test therefore
/// runs, and is expected to keep running once the row closes.
///
/// What it does not establish is that the refusal is form-aware. It is
/// not, today, and the ignored test above is where that is recorded.
#[test]
fn a_surjection_proof_is_refused_for_the_hybrid_form() {
    let hybrid = hybrid_form();
    let bytes = with_output_proofs(&hybrid, &[0xcd; 32], &[0xab; 64]);

    assert_eq!(
        TargetTransaction::decode(&bytes).err(),
        Some(TransactionRefusal::SurjectionProofRefused),
        "an explicit asset admits no surjection proof",
    );
}

// --- Row: the protected preimage ---------------------------------------
//
// The guide's row: `FinalizedLiveTransfer::protected_bytes` is
// `encode_without_witness()`, which omits the output-witness vector the
// target's `SIGHASH_ALL` covers — the empty-vector case `G11-W11-06`
// already diagnosed on the target side. What is reproduced here is the
// first-party half only.

/// The private form's protected preimage is not the witnessless
/// serialization.
///
/// The witnessless serialization is exactly the empty-output-witness
/// case the recorded target diagnosis names: a signer over those bytes
/// hashes the empty string where consensus hashes one entry per output,
/// so the signature is reported complete and the spend is invalid. The
/// repaired private preimage carries the output-witness vector, and
/// therefore differs from the witnessless bytes.
///
/// Ignored while the two are equal: the observed failure is this row's
/// first-party half. The target half is already established by the
/// recorded runnable diagnosis and is cited rather than rerun.
#[test]
#[ignore = "confidential-funding preflight: the preimage is the witnessless form today; run with -- --ignored"]
fn the_private_protected_preimage_is_not_the_witnessless_serialization() {
    let built = private_finalization_with(&FixturePrivateValue);
    let finalized = built.finalized();

    assert_ne!(
        finalized.protected_bytes(),
        finalized.protected().encode_without_witness().as_slice(),
        "the bytes an owner binds to must not be the empty-output-witness form",
    );
}

/// What the witnessless serialization omits, counted.
///
/// A fact about the encoder rather than about the preimage, so it stays
/// true after the row closes. It is what makes the omission concrete:
/// the difference between the two serializations is the whole witness
/// section, and two bytes of every output's share of it are the
/// surjection and range proof fields that carry nothing today and would
/// carry the proofs a confidential funding transaction depends on.
#[test]
fn the_witnessless_serialization_omits_the_whole_witness_section() {
    let transaction = hybrid_form();
    let full = transaction.encode();
    let stripped = transaction.encode_without_witness();

    // Each input's witness costs two empty issuance proofs, a stack, and
    // an empty peg-in field; each output's costs two empty proof fields.
    let stack_bytes: usize = nonnull_witness()
        .stack()
        .iter()
        .map(|item| 1 + item.len())
        .sum();
    let per_input = 2 + 1 + stack_bytes + 1;
    let per_output = 2;

    assert_eq!(
        full.len() - stripped.len(),
        transaction.inputs().len() * per_input + transaction.outputs().len() * per_output,
        "the two serializations differ by exactly the witness section",
    );
    assert_eq!(
        full[full.len() - per_output..],
        [NULL_PREFIX, NULL_PREFIX],
        "the output-witness region the stripped form omits is the proof fields",
    );
}

// --- Row: the per-output confidential value role -----------------------
//
// The guide's row is a CHARACTERIZATION, not a defect: it names five
// limits of `PrivateValueCapability` and concludes that the concept
// replaces the role rather than extending it. The tests below therefore
// run, and record the limits as measured facts. The wave that retires
// the role from private complete-transaction claims replaces them.

/// Everything one call of the confidential value role was told.
///
/// The whole argument list, so that the record is a complete description
/// of what crossed the boundary rather than a selection from it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedCall {
    asset: AssetId,
    amount: u64,
    randomness: [u8; 32],
    position: u16,
}

/// A confidential value capability that records what it is asked and
/// answers with a constant.
///
/// The constant is the point: it depends on no argument at all, so a
/// construction that accepts its answers has checked nothing about the
/// values the commitments are supposed to carry.
struct RecordingPrivateValue {
    calls: RefCell<Vec<RecordedCall>>,
}

impl RecordingPrivateValue {
    fn recording() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<RecordedCall> {
        self.calls.borrow().clone()
    }
}

impl PrivateValueCapability for RecordingPrivateValue {
    fn value_commitment(
        &self,
        asset: AssetId,
        value: ProtocolValue,
        randomness: &PublicTestRandomness,
        position: u16,
    ) -> Option<[u8; COMMITMENT_BYTES]> {
        self.calls.borrow_mut().push(RecordedCall {
            asset,
            amount: value.amount(),
            randomness: *randomness.bytes(),
            position,
        });

        let mut commitment = [0x5c_u8; COMMITMENT_BYTES];
        commitment[0] = VALUE_COMMITMENT_PREFIXES[0];
        Some(commitment)
    }
}

/// One private finalization driven by `capability`.
fn private_finalization_with(capability: &dyn PrivateValueCapability) -> LiveFinalization {
    let abi = live_abi();
    let first = outpoint(0xc1, 0);
    let stated = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::PrivateCommitted,
        ValueField::Commitment([0x09; COMMITMENT_BYTES]),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [
            LiveReceiptDestination::new(
                owner(&FIRST_OWNER),
                ProtocolValue::new(600).expect("the fixture amounts are positive"),
            ),
            LiveReceiptDestination::new(
                owner(&SECOND_OWNER),
                ProtocolValue::new(400).expect("the fixture amounts are positive"),
            ),
        ],
        LiveTransferRepresentationPlan::PrivateCommitted,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        Some(PublicTestRandomness::from_published_bytes(
            PUBLISHED_RANDOMNESS,
        )),
    )
    .expect("the fixture request validates");

    finalize_live_transfer(
        &reviewed_target(),
        &abi,
        &request,
        &stated,
        None,
        Some(capability),
    )
    .expect("the private form finalizes")
}

/// The confidential value role is told four things and no opening.
///
/// The recorded tuple is the complete argument list, so what it does
/// NOT contain is as much of the measurement as what it does: no
/// predecessor outpoint, no predecessor opening, no other destination,
/// no blinder, and no proof request. One call per destination, in
/// request order, at consecutive positions.
#[test]
fn the_confidential_value_role_is_told_four_things_and_no_opening() {
    let capability = RecordingPrivateValue::recording();
    let built = private_finalization_with(&capability);
    let calls = capability.calls();

    assert_eq!(calls.len(), 2, "one call per destination and no more");
    assert_eq!(
        calls[0].amount, 600,
        "the first destination, in request order"
    );
    assert_eq!(calls[1].amount, 400, "the second destination");
    for call in &calls {
        assert_eq!(
            call.asset,
            AssetId::from_internal(LIVE_PROTOCOL_ASSET),
            "the protocol asset, explicit",
        );
        assert_eq!(
            call.randomness, PUBLISHED_RANDOMNESS,
            "the published randomness the request carried",
        );
    }
    assert_eq!(
        calls[1].position,
        calls[0].position + 1,
        "the positions are the request's own order, consecutive",
    );

    assert_eq!(
        built.finalized().outputs().outputs().len(),
        2,
        "the sponsorless private form has exactly the two destinations",
    );
}

/// Nothing checks that the per-output commitments balance.
///
/// The recording capability answers every call with one constant
/// commitment that depends on no argument. Two destinations of different
/// amounts therefore receive identical value fields, which no set of
/// openings could produce, and construction accepts it: there is no
/// place in the private path where a blinder sum, a conservation, or a
/// cross-destination relation is checked.
///
/// The consumed total is absent for the same reason, which is the
/// existing rule that a public subtotal of private values is refused —
/// so the private path closes no conservation of its own either.
#[test]
fn nothing_checks_that_the_per_output_commitments_balance() {
    let capability = RecordingPrivateValue::recording();
    let built = private_finalization_with(&capability);
    let outputs = built.finalized().outputs().outputs();

    let first = match outputs[0].value() {
        ValueField::Commitment(commitment) => commitment,
        ValueField::Explicit(_) => panic!("the private form builds committed value fields"),
    };
    let second = match outputs[1].value() {
        ValueField::Commitment(commitment) => commitment,
        ValueField::Explicit(_) => panic!("the private form builds committed value fields"),
    };

    assert_eq!(
        first, second,
        "two destinations of different amounts took the same value field",
    );
    assert_eq!(
        built.report().consumed_total(),
        None,
        "no public subtotal of private values is reported",
    );
}

/// The private form fixes a null nonce and can carry no proof.
///
/// Two of the five limits at once, both structural. Every output the
/// private path builds takes the null nonce, and the output type has no
/// proof member at all — so serializing the finalized transaction with a
/// witness present writes the two empty proof fields per output that the
/// encoder writes unconditionally, and there is nowhere for a rangeproof
/// to go.
#[test]
fn the_private_form_fixes_a_null_nonce_and_can_carry_no_proof() {
    let built = private_finalization_with(&FixturePrivateValue);
    let protected = built.finalized().protected();

    for output in protected.outputs() {
        assert_eq!(
            output.nonce(),
            NonceField::Null,
            "the private path fixes the null nonce",
        );
    }

    // The same transaction with a witness present, which is the only
    // serialization that writes the output-witness region at all.
    let witnessed = TargetTransaction::new(
        protected.version(),
        protected.inputs().to_vec(),
        protected.outputs().to_vec(),
        protected.lock_time(),
        vec![nonnull_witness(); protected.inputs().len()],
    )
    .expect("the census is one witness per input");
    let bytes = witnessed.encode();
    let tail = 2 * witnessed.outputs().len();

    assert_eq!(
        bytes[bytes.len() - tail..],
        vec![NULL_PREFIX; tail],
        "every output's proof pair is empty and no API offers another value",
    );
}
