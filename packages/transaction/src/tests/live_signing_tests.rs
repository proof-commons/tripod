//! The finalization boundary, the signing requests, and §12.7's ten
//! rejections (§12.6, §12.7, §1.6, §1.7).
//!
//! # Every rejection has an executable negative
//!
//! Ten rejections, ten tests, each stating exactly one disagreement with
//! the finalized form. The signature bytes are fixture bytes: this crate
//! checks binding rather than cryptography, and the package that owns
//! the target's curve checks the rest.

use std::collections::BTreeSet;

use linker::live_backend::{LiveTransferRepresentationPlan, ProtectedDatum};

use super::live_support::{FIRST_OWNER, SECOND_OWNER, live_abi, owner, receipt_view};
use super::{outpoint, reviewed_target, view};
use crate::bytes::{
    AssetField, AssetId, InputWitness, NonceField, TargetOutput, TargetTransaction, ValueField,
};
use crate::error::TransactionRefusal;
use crate::live_construct::finalize_live_transfer;
use crate::live_finalize::{FinalizedFact, FinalizedLiveTransfer, LiveSigningRequest};
use crate::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use crate::live_signing::{LiveOwnerResponse, authorize_live_transfer};

/// The fixture signature bytes, which verify nothing.
const FIXTURE_SIGNATURE: [u8; 8] = [0x5c; 8];

/// One destination for `bytes`'s owner at `amount`.
fn destination(bytes: &[u8], amount: u64) -> LiveReceiptDestination {
    LiveReceiptDestination::new(
        owner(bytes),
        ProtocolValue::new(amount).expect("the fixture amounts are positive"),
    )
}

/// A finalized transfer consuming two receipts of two different owners.
fn two_owners() -> FinalizedLiveTransfer {
    let abi = live_abi();
    let first = outpoint(0xa1, 0);
    let second = outpoint(0xa2, 1);
    let stated = view([
        receipt_view(
            &abi,
            first,
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(400),
        ),
        receipt_view(
            &abi,
            second,
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(600),
        ),
    ]);
    let request = LiveTransferRequest::new(
        [first, second],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None)
        .expect("the fixture finalizes")
        .into_finalized()
}

/// A finalized transfer whose two receipts belong to one owner.
fn one_owner_two_inputs() -> FinalizedLiveTransfer {
    let abi = live_abi();
    let first = outpoint(0xa1, 0);
    let second = outpoint(0xa2, 1);
    let stated = view([
        receipt_view(
            &abi,
            first,
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(400),
        ),
        receipt_view(
            &abi,
            second,
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(600),
        ),
    ]);
    let request = LiveTransferRequest::new(
        [first, second],
        [destination(&SECOND_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None)
        .expect("the fixture finalizes")
        .into_finalized()
}

/// Every conforming response, keyed by the position it answers.
fn conforming(finalized: &FinalizedLiveTransfer) -> Vec<(u16, LiveOwnerResponse)> {
    finalized
        .signing_requests()
        .iter()
        .map(|request| {
            (
                request.input(),
                LiveOwnerResponse::to(request, FIXTURE_SIGNATURE.to_vec()),
            )
        })
        .collect()
}

/// The signing request for one input.
fn request_for(finalized: &FinalizedLiveTransfer, input: u16) -> LiveSigningRequest {
    finalized
        .signing_request(input)
        .expect("the fixture positions are receipt inputs")
}

// --- The finalization boundary (§12.6) --------------------------------

#[test]
fn a_finalized_form_settles_all_ten_items() {
    let finalized = two_owners();
    assert_eq!(FinalizedFact::ALL.len(), 10);
    assert_eq!(
        finalized.settled(),
        FinalizedFact::ALL.iter().copied().collect::<BTreeSet<_>>(),
    );
}

#[test]
fn the_protected_bytes_carry_no_witness_and_are_the_message_preimage() {
    // §1.7: the request carries the preimage, not a digest. A 32-byte
    // value here would be this crate asserting the profile it intended.
    let finalized = two_owners();
    assert_eq!(
        finalized.protected_bytes(),
        finalized.protected().encode_without_witness(),
    );
    assert_ne!(finalized.protected_bytes().len(), 32);
    for witness in finalized.protected().witnesses() {
        assert!(witness.is_null());
    }
}

#[test]
fn every_signing_request_carries_the_same_bytes_and_the_protected_data_census() {
    let finalized = two_owners();
    let requests = finalized.signing_requests();
    assert_eq!(requests.len(), 2);

    for request in &requests {
        assert_eq!(request.protected_bytes(), finalized.protected_bytes());
        assert_eq!(request.protected_data(), finalized.protected_data());
        assert_eq!(request.protected_data().len(), 15);
        assert!(
            request
                .protected_data()
                .contains(&ProtectedDatum::ReceiptInputs)
        );
        assert!(
            request
                .protected_data()
                .contains(&ProtectedDatum::DestinationEntries)
        );
        assert_eq!(
            request.required_dimensions(),
            finalized.required_dimensions()
        );
        assert_ne!(request.required_dimensions().len(), 0);
    }

    // One request per input, and the two name different owners and
    // different leaves at the two positions.
    assert_ne!(requests[0].owner(), requests[1].owner());
    assert_ne!(requests[0].leaf(), requests[1].leaf());
    assert_eq!(
        finalized.signing_request(9),
        Err(TransactionRefusal::ReceiptPositionOutsideFamily { position: 9 }),
    );
}

// --- §1.6's three levels ----------------------------------------------

#[test]
fn two_owners_of_two_inputs_report_two_of_each() {
    let finalized = two_owners();
    let responses = conforming(&finalized);
    let authorized =
        authorize_live_transfer(finalized, responses).expect("the complete owner set authorizes");

    let levels = authorized.levels();
    assert_eq!(levels.distinct_semantic_owners().len(), 2);
    assert_eq!(levels.receipt_inputs(), 2);
    assert_eq!(levels.owner_signatures(), 2);
    assert!(!levels.some_owner_holds_several_inputs());
}

#[test]
fn one_owner_of_two_inputs_is_one_owner_and_two_signatures() {
    // §1.6's own warning: three signatures from one owner are not three
    // independent owners, and a report that carried only a count would
    // have said they were.
    let finalized = one_owner_two_inputs();
    let responses = conforming(&finalized);
    let authorized =
        authorize_live_transfer(finalized, responses).expect("the complete owner set authorizes");

    let levels = authorized.levels();
    assert_eq!(levels.distinct_semantic_owners().len(), 1);
    assert_eq!(levels.receipt_inputs(), 2);
    assert_eq!(levels.owner_signatures(), 2);
    assert!(levels.some_owner_holds_several_inputs());
}

#[test]
fn the_assembled_witness_is_the_signature_the_leaf_and_the_control_block() {
    let finalized = two_owners();
    let scripts: Vec<_> = finalized
        .receipts()
        .iter()
        .map(|record| {
            (
                record.leaf_script().to_vec(),
                record.control_block().to_vec(),
            )
        })
        .collect();
    let responses = conforming(&finalized);
    let authorized = authorize_live_transfer(finalized, responses).expect("the owners authorize");

    assert_eq!(authorized.witnesses().len(), 2);
    for (position, witness) in authorized.witnesses() {
        let (script, control) = &scripts[usize::from(*position)];
        assert_eq!(
            witness,
            &InputWitness::new(vec![
                FIXTURE_SIGNATURE.to_vec(),
                script.clone(),
                control.clone(),
            ]),
        );
    }
}

// --- §12.7's ten rejections -------------------------------------------

#[test]
fn a_missing_owner_is_refused() {
    let finalized = two_owners();
    let mut responses = conforming(&finalized);
    responses.pop();
    assert_eq!(
        authorize_live_transfer(finalized, responses).err(),
        Some(TransactionRefusal::OwnerResponseMissing { input: 1 }),
    );
}

#[test]
fn a_duplicate_response_is_refused() {
    let finalized = two_owners();
    let mut responses = conforming(&finalized);
    responses.push(responses[0].clone());
    assert_eq!(
        authorize_live_transfer(finalized, responses).err(),
        Some(TransactionRefusal::OwnerResponseDuplicated { input: 0 }),
    );
}

#[test]
fn an_unexpected_signer_is_refused() {
    let finalized = two_owners();
    let mut responses = conforming(&finalized);
    let stray = LiveOwnerResponse::from_parts(
        7,
        owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::Explicit,
        finalized.required_dimensions().clone(),
        finalized.protected_bytes().to_vec(),
        FIXTURE_SIGNATURE.to_vec(),
    );
    responses.push((7, stray));
    assert_eq!(
        authorize_live_transfer(finalized, responses).err(),
        Some(TransactionRefusal::UnexpectedSigner { input: 7 }),
    );
}

#[test]
fn a_response_from_the_wrong_owner_is_refused() {
    let finalized = two_owners();
    let request = request_for(&finalized, 0);
    let impostor = LiveOwnerResponse::from_parts(
        0,
        // The other owner of this very transfer, which is the closest a
        // wrong owner can get to a right one.
        owner(&SECOND_OWNER),
        request.representation(),
        request.required_dimensions().clone(),
        request.protected_bytes().to_vec(),
        FIXTURE_SIGNATURE.to_vec(),
    );
    let responses = vec![
        (0, impostor),
        (
            1,
            LiveOwnerResponse::to(&request_for(&finalized, 1), FIXTURE_SIGNATURE.to_vec()),
        ),
    ];
    assert_eq!(
        authorize_live_transfer(finalized, responses).err(),
        Some(TransactionRefusal::ResponseFromWrongOwner { input: 0 }),
    );
}

#[test]
fn a_response_answering_the_wrong_input_is_refused() {
    let finalized = two_owners();
    // The response to input one, offered at input zero.
    let misfiled = LiveOwnerResponse::to(&request_for(&finalized, 1), FIXTURE_SIGNATURE.to_vec());
    let responses = vec![(0, misfiled)];
    assert_eq!(
        authorize_live_transfer(finalized, responses).err(),
        Some(TransactionRefusal::ResponseForWrongInput { input: 0 }),
    );
}

#[test]
fn a_response_under_another_sighash_profile_is_refused() {
    let finalized = two_owners();
    let request = request_for(&finalized, 0);
    let mut narrower = request.required_dimensions().clone();
    let dropped = *narrower
        .iter()
        .next()
        .expect("the selected profile requires at least one dimension");
    narrower.remove(&dropped);

    let response = LiveOwnerResponse::from_parts(
        0,
        request.owner().clone(),
        request.representation(),
        narrower,
        request.protected_bytes().to_vec(),
        FIXTURE_SIGNATURE.to_vec(),
    );
    assert_eq!(
        authorize_live_transfer(finalized, vec![(0, response)]).err(),
        Some(TransactionRefusal::ResponseUnderWrongSighashProfile { input: 0 }),
    );
}

#[test]
fn a_response_bound_to_other_bytes_is_refused() {
    let finalized = two_owners();
    let request = request_for(&finalized, 0);
    let mut other = request.protected_bytes().to_vec();
    other.push(0x00);

    let response = LiveOwnerResponse::from_parts(
        0,
        request.owner().clone(),
        request.representation(),
        request.required_dimensions().clone(),
        other,
        FIXTURE_SIGNATURE.to_vec(),
    );
    assert_eq!(
        authorize_live_transfer(finalized, vec![(0, response)]).err(),
        Some(TransactionRefusal::ResponseBoundToDifferentBytes { input: 0 }),
    );
}

#[test]
fn an_output_changed_after_signing_is_refused() {
    let finalized = two_owners();
    let protected = finalized.protected().clone();
    let mut outputs = protected.outputs().to_vec();
    outputs[0] = TargetOutput::new(
        outputs[0].asset(),
        ValueField::Explicit(999),
        NonceField::Null,
        outputs[0].program().to_vec(),
    );
    let mutated = TargetTransaction::new(
        protected.version(),
        protected.inputs().to_vec(),
        outputs,
        protected.lock_time(),
        protected.witnesses().to_vec(),
    )
    .expect("the mutated transaction assembles");

    let responses = conforming(&finalized);
    let authorized = authorize_live_transfer(finalized, responses).expect("the owners authorize");
    assert_eq!(
        authorized.check_offered(&mutated),
        Err(TransactionRefusal::OutputMutatedAfterSigning { position: 0 }),
    );
    // And the transaction that was actually finalized still passes.
    assert_eq!(authorized.check_offered(&protected), Ok(()));
}

#[test]
fn an_input_added_after_signing_is_refused() {
    let finalized = two_owners();
    let protected = finalized.protected().clone();
    let mut inputs = protected.inputs().to_vec();
    inputs.push(crate::bytes::TargetInput::new(
        outpoint(0xfe, 0),
        crate::live_construct::LIVE_TRANSFER_SEQUENCE,
    ));
    let mut witnesses = protected.witnesses().to_vec();
    witnesses.push(InputWitness::default());
    let extended = TargetTransaction::new(
        protected.version(),
        inputs,
        protected.outputs().to_vec(),
        protected.lock_time(),
        witnesses,
    )
    .expect("the extended transaction assembles");

    let responses = conforming(&finalized);
    let authorized = authorize_live_transfer(finalized, responses).expect("the owners authorize");
    assert_eq!(
        authorized.check_offered(&extended),
        Err(TransactionRefusal::InputExtendedAfterSigning {
            finalized: 2,
            offered: 3,
        }),
    );
}

#[test]
fn an_output_removed_after_signing_is_refused() {
    let abi = live_abi();
    let first = outpoint(0xa1, 0);
    let stated = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::Explicit,
        ValueField::Explicit(1_000),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [
            destination(&FIRST_OWNER, 400),
            destination(&SECOND_OWNER, 600),
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");
    let finalized = finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None)
        .expect("the fixture finalizes")
        .into_finalized();

    let protected = finalized.protected().clone();
    let mut outputs = protected.outputs().to_vec();
    outputs.pop();
    let shortened = TargetTransaction::new(
        protected.version(),
        protected.inputs().to_vec(),
        outputs,
        protected.lock_time(),
        protected.witnesses().to_vec(),
    )
    .expect("the shortened transaction assembles");

    let responses = conforming(&finalized);
    let authorized = authorize_live_transfer(finalized, responses).expect("the owner authorizes");
    assert_eq!(
        authorized.check_offered(&shortened),
        Err(TransactionRefusal::OutputOmittedAfterSigning { position: 1 }),
    );
}

#[test]
fn a_version_or_locktime_moved_after_signing_is_refused() {
    // Protected data the same signature covers (§1.7), so a change to
    // either is reported rather than passing because the outputs
    // happened to match.
    let finalized = two_owners();
    let protected = finalized.protected().clone();
    let retimed = TargetTransaction::new(
        protected.version(),
        protected.inputs().to_vec(),
        protected.outputs().to_vec(),
        protected.lock_time().saturating_add(1),
        protected.witnesses().to_vec(),
    )
    .expect("the retimed transaction assembles");

    let responses = conforming(&finalized);
    let authorized = authorize_live_transfer(finalized, responses).expect("the owners authorize");
    assert_eq!(
        authorized.check_offered(&retimed),
        Err(TransactionRefusal::OutputMutatedAfterSigning { position: 0 }),
    );
}

// --- No partial owner set proceeds ------------------------------------

#[test]
fn no_partial_owner_set_produces_an_authorized_transfer() {
    // Every proper subset of the responses is refused, and there is no
    // constructor for an authorized transfer that does not run through
    // this function.
    let finalized = two_owners();
    let complete = conforming(&finalized);
    for skipped in 0..complete.len() {
        let partial: Vec<_> = complete
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != skipped)
            .map(|(_, response)| response.clone())
            .collect();
        assert!(authorize_live_transfer(finalized.clone(), partial).is_err());
    }
    assert!(authorize_live_transfer(finalized, complete).is_ok());
}

#[test]
fn the_asset_of_every_destination_is_the_protocol_asset_the_link_resolved() {
    let finalized = two_owners();
    for output in finalized.outputs().outputs() {
        assert_eq!(
            output.asset(),
            AssetField::Explicit(AssetId::from_internal(
                super::live_support::LIVE_PROTOCOL_ASSET
            )),
        );
    }
}
