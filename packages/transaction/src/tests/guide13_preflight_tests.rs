//! Guide-13 preflight reproductions owned by this crate.
//!
//! # What these tests are for
//!
//! Guide 13 §3 opens with a register of review hypotheses that are "not
//! current-tree facts until reproduced". Each test below reaches the
//! exact branch one row names, so the row can be disposed on evidence
//! rather than on a reading of the source.
//!
//! # Why they assert the repaired behaviour
//!
//! A test that asserted today's behaviour would pass now and fail once
//! the defect is fixed, which is the wrong way round. Each test states
//! the property the row's repair must establish, so it fails while the
//! defect stands and passes once the repair lands. Confirmed rows are
//! therefore `#[ignore]`d: the reproduction is committed and replayable
//! with `--ignored`, and the ordinary lane stays green until the repair
//! wave removes the attribute.
//!
//! Nothing here is a repair. Guide 13 wave 0 disposes rows; later waves
//! change behaviour.

use linker::backend::SponsorChangePresence;
use target_elements::TransactionForm;

use crate::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, ValueField, WITNESS_FLAG,
};
use crate::construct::construct;
use crate::request::CompactAshRequest;
use crate::sponsor::{SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest};
use crate::tests::{
    CLOSED_ASSET, PINNED_PROGRAM, RESERVE_ASSET, SPONSOR_CHANGE_PROGRAM, ash_view, candidate_abi,
    outpoint, reviewed_target, view,
};
use crate::view::{PublicConstructionView, PublicOutputView};

// --- G13-R02: the superfluous witness record -------------------------

/// A transaction whose every witness is empty.
///
/// Built directly rather than through the construction pipeline: the
/// pipeline installs a script-path stack, and this row is about the
/// encoding of a transaction that carries no witness at all.
fn all_empty_witness_transaction() -> TargetTransaction {
    let mut program = vec![0x51, 0x20];
    program.extend_from_slice(&PINNED_PROGRAM);
    let inputs = vec![
        TargetInput::new(outpoint(0xaa, 0), 0xffff_ffff),
        TargetInput::new(outpoint(0xbb, 1), 0xffff_ffff),
    ];
    let outputs = vec![TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(CLOSED_ASSET)),
        ValueField::Explicit(300),
        NonceField::Null,
        program,
    )];
    let witnesses = vec![InputWitness::default(); inputs.len()];
    TargetTransaction::new(3, inputs, outputs, 0, witnesses)
        .expect("the fixture census is one witness per input")
}

/// The same transaction re-serialized with the witness flag set and an
/// entirely empty witness section written out.
///
/// The section is the one the target's serializer would emit: four
/// empty length prefixes per input — issuance proof, inflation-keys
/// proof, an empty stack count, and the peg-in stack — and two per
/// output for the surjection and range proofs.
fn witness_flagged_all_empty_bytes(transaction: &TargetTransaction) -> Vec<u8> {
    let mut bytes = transaction.encode_without_witness();
    // The flag byte sits immediately after the four version bytes.
    bytes[4] = WITNESS_FLAG;
    for _ in transaction.inputs() {
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    }
    for _ in transaction.outputs() {
        bytes.extend_from_slice(&[0x00, 0x00]);
    }
    bytes
}

/// `G13-R02`: a successful decode must reproduce its own input bytes.
///
/// The canonical rule the encoder already follows is that a transaction
/// whose every witness is empty is serialized without the witness
/// section. A decoder that accepts the witness-flagged all-empty form
/// admits a second byte string for one typed transaction, because
/// re-encoding emits the witnessless form instead.
///
/// The assertion is written as the round-trip law rather than as "the
/// decode fails", so that it also holds once the decoder refuses the
/// forged bytes outright.
#[test]
fn a_decoded_transaction_re_encodes_to_the_bytes_it_came_from() {
    let transaction = all_empty_witness_transaction();
    let forged = witness_flagged_all_empty_bytes(&transaction);

    // The forged string really is a second encoding of the same typed
    // value, not a malformed one: it differs from the canonical bytes.
    assert_ne!(
        forged,
        transaction.encode(),
        "the fixture must offer bytes the encoder would not have produced"
    );

    match TargetTransaction::decode(&forged) {
        // Repaired: the superfluous witness record is refused.
        Err(_) => {}
        Ok(decoded) => assert_eq!(
            decoded.encode(),
            forged,
            "a transaction the decoder accepted must re-encode to exactly the bytes it decoded"
        ),
    }
}

/// The canonical witnessless form decodes and round-trips.
///
/// Committed unignored as the control for the row above: it establishes
/// that the fixture and the round-trip law are right about the form the
/// encoder does produce, so a failure of the ignored test is about the
/// superfluous record and not about the fixture.
#[test]
fn the_canonical_witnessless_form_round_trips() {
    let transaction = all_empty_witness_transaction();
    let canonical = transaction.encode();
    let decoded = TargetTransaction::decode(&canonical).expect("the canonical form decodes");
    assert_eq!(decoded, transaction);
    assert_eq!(decoded.encode(), canonical);
}

// --- G13-R13: a sponsored request that builds the sponsorless form ---

/// A sponsor adapter that offers no inputs at all.
///
/// It declares a fee and holds a change destination, so the only thing
/// separating it from the ordinary fixture sponsor is an empty offer.
struct EmptyOfferSponsor {
    fee: u64,
}

impl SponsorCapability for EmptyOfferSponsor {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new([], self.fee, None)
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, SPONSOR_CHANGE_PROGRAM.to_vec()))
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        Some(SponsorSignature::new(
            request.transaction().to_vec(),
            vec![vec![0x30, 0x44, 0x01], vec![0x02, 0x0a]],
        ))
    }
}

/// `G13-R13`: a sponsored request never yields the sponsorless form.
///
/// Shape selection reads the number of sponsor outpoints the capability
/// actually offered rather than the request's own flag, so an empty
/// offer selects a sponsorless shape: the declared sponsor fee has no
/// position to occupy, no signing request is issued, and the report
/// names a form the caller did not ask for.
///
/// Stated as an equivalence between what was asked and what was built,
/// so a repair that refuses the empty offer also satisfies it.
#[test]
#[ignore = "G13-R13: confirmed, repair pending"]
fn a_sponsored_request_does_not_build_the_sponsorless_form() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let chain = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = EmptyOfferSponsor { fee: 30 };

    match construct(&target, &abi, &request, &chain, Some(&sponsor)) {
        // Repaired: an empty sponsor offer is refused.
        Err(_) => {}
        Ok(built) => assert_eq!(
            built.report().form(),
            TransactionForm::Sponsored,
            "a request that asked for a sponsor suffix must not be answered with the sponsorless form"
        ),
    }
}

/// The control: the same request with one sponsor outpoint offered.
///
/// Unignored, so the ignored test above is known to be about the empty
/// offer rather than about the fixture's ability to build a sponsored
/// transaction at all.
#[test]
fn one_offered_sponsor_outpoint_builds_the_sponsored_form() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_outpoint = outpoint(0xdd, 2);
    let mut sponsor_program = vec![0x00, 0x14];
    sponsor_program.extend_from_slice(&[0xd0; 20]);
    let chain = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        PublicOutputView::new(
            sponsor_outpoint,
            AssetField::Explicit(AssetId::from_internal(RESERVE_ASSET)),
            ValueField::Explicit(90),
            sponsor_program,
        ),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = OneInputSponsor { fee: 90 };

    let built = construct(&target, &abi, &request, &chain, Some(&sponsor))
        .expect("the sponsored form constructs");
    assert_eq!(built.report().form(), TransactionForm::Sponsored);
    assert_eq!(
        built.report().shape().sponsor_change(),
        SponsorChangePresence::Absent
    );
}

/// A sponsor adapter contributing exactly one outpoint and no change.
struct OneInputSponsor {
    fee: u64,
}

impl SponsorCapability for OneInputSponsor {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new([outpoint(0xdd, 2)], self.fee, None)
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, SPONSOR_CHANGE_PROGRAM.to_vec()))
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        Some(SponsorSignature::new(
            request.transaction().to_vec(),
            vec![vec![0x30, 0x44, 0x01], vec![0x02, 0x0a]],
        ))
    }
}

// --- G13-R16: silently collapsed duplicates --------------------------

/// `G13-R16`: declaration order must not decide the constructed amount.
///
/// Two contradictory public views of one outpoint both enter the public
/// construction boundary; the map keyed by outpoint keeps whichever
/// arrived last. The surviving view is the one the constructor believes
/// about that output's amount, so the successor amount of the finished
/// transaction depends on the order the caller happened to list the two
/// statements in.
///
/// The repair makes the constructor fallible, at which point both
/// orders refuse and the first arm of the match carries the test.
#[test]
#[ignore = "G13-R16: confirmed, repair pending"]
fn a_contradictory_public_view_does_not_depend_on_declaration_order() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");

    let amount_when = |views: [PublicOutputView; 3]| {
        construct(
            &target,
            &abi,
            &request,
            &PublicConstructionView::new(views),
            None,
        )
        .map(|built| built.report().successor_amount())
        .map_err(|_| ())
    };

    let low = ash_view(&target, first, 120);
    let high = ash_view(&target, first, 900);
    let other = ash_view(&target, second, 180);

    let low_last = amount_when([high.clone(), low.clone(), other.clone()]);
    let high_last = amount_when([low, high, other]);

    match (low_last, high_last) {
        // Repaired: a contradictory pair is refused in either order.
        (Err(()), Err(())) => {}
        (low_last, high_last) => assert_eq!(
            low_last, high_last,
            "which of two contradictory views the constructor believes must not be decided by the order they were listed in"
        ),
    }
}

/// `G13-R16`: a duplicated sponsor outpoint is not a smaller offer.
///
/// The offer collects into a set, so a caller naming one outpoint twice
/// is silently normalized into a request for one input rather than
/// refused. The assertion is on the census the caller stated.
///
/// The repair makes the constructor fallible; the repair wave replaces
/// this assertion with the refusal it then returns.
#[test]
#[ignore = "G13-R16: confirmed, repair pending"]
fn a_duplicated_sponsor_outpoint_is_not_silently_collapsed() {
    let repeated: Outpoint = outpoint(0xdd, 2);
    let offer = SponsorOffer::new([repeated, repeated], 90, None);
    assert_eq!(
        offer.inputs().len(),
        2,
        "a sponsor offer naming one outpoint twice must not be normalized into a one-input offer"
    );
}
