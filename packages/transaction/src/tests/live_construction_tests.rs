//! Live-transfer construction: form exactness, layouts, and candidate
//! bytes (§12.1, §12.2, §12.5, §12.8, §12.9).

use std::collections::BTreeSet;

use linker::live_backend::LiveTransferRepresentationPlan;

use super::live_support::{
    FIRST_OWNER, FixturePrivateValue, LIVE_PROTOCOL_ASSET, LIVE_RESERVE_ASSET,
    LIVE_SPONSOR_CHANGE_PROGRAM, PUBLISHED_RANDOMNESS, SECOND_OWNER, live_abi, owner, receipt_view,
    single_representation_live_abi, sponsor_view,
};
use super::{outpoint, reviewed_target, view};
use crate::bytes::{AssetField, AssetId, Outpoint, ValueField};
use crate::error::TransactionRefusal;
use crate::live_abi::{CandidateLiveTransferAbi, LiveTransactionForm};
use crate::live_construct::{LiveFinalization, finalize_live_transfer};
use crate::live_private::{
    ConfidentialConstructionModel, PrivateConstructionDemonstration, PrivateConstructionNonClaim,
    SelectedConstructionModel,
};
use crate::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use crate::sponsor::{SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest};
use crate::view::PublicConstructionView;

/// A sponsor adapter contributing one input and echoing what it signs.
struct FixtureSponsor {
    inputs: Vec<Outpoint>,
    fee: u64,
    change: Option<ValueField>,
}

/// The outpoint every fixture sponsor offers.
///
/// Named once because the offer and the view must agree about it; two
/// spellings could drift apart and the guard would then be refusing the
/// fixture rather than the defect it is for.
fn sponsor_coin() -> Outpoint {
    outpoint(0xdd, 2)
}

impl FixtureSponsor {
    fn new(fee: u64, change: Option<ValueField>) -> Self {
        Self {
            inputs: vec![sponsor_coin()],
            fee,
            change,
        }
    }

    /// The capability whose offer names no input at all.
    const fn empty(fee: u64) -> Self {
        Self {
            inputs: Vec::new(),
            fee,
            change: None,
        }
    }
}

impl SponsorCapability for FixtureSponsor {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new(self.inputs.iter().copied(), self.fee, self.change)
            .expect("the fixture offer names distinct outpoints")
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, LIVE_SPONSOR_CHANGE_PROGRAM.to_vec()))
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        Some(SponsorSignature::new(
            request.transaction().to_vec(),
            vec![vec![0x30, 0x44, 0x01], vec![0x02, 0x0a]],
        ))
    }
}

/// One destination for `bytes`'s owner at `amount`.
fn destination(bytes: &[u8], amount: u64) -> LiveReceiptDestination {
    LiveReceiptDestination::new(
        owner(bytes),
        ProtocolValue::new(amount).expect("the fixture amounts are positive"),
    )
}

/// The explicit two-in two-out request and the view that recognizes it.
fn explicit_fixture(
    abi: &CandidateLiveTransferAbi,
    form: RequestedForm,
    change: SponsorChangeRequest,
) -> (LiveTransferRequest, PublicConstructionView) {
    let first = outpoint(0xa1, 0);
    let second = outpoint(0xa2, 1);
    let view = view([
        receipt_view(
            abi,
            first,
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(400),
        ),
        receipt_view(
            abi,
            second,
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(600),
        ),
        // The coin the fixture sponsor offers, stated rather than
        // merely named: a sponsored build reads its sponsor inputs from
        // this same view.
        sponsor_view(sponsor_coin(), 130),
    ]);
    let request = LiveTransferRequest::new(
        [first, second],
        [
            destination(&SECOND_OWNER, 250),
            destination(&FIRST_OWNER, 750),
        ],
        LiveTransferRepresentationPlan::Explicit,
        form,
        change,
        None,
    )
    .expect("the fixture request validates");
    (request, view)
}

/// The sponsorless explicit finalization.
fn sponsorless() -> LiveFinalization {
    let abi = live_abi();
    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    );
    finalize_live_transfer(&reviewed_target(), &abi, &request, &view, None, None)
        .expect("the sponsorless explicit form finalizes")
}

// --- §12.5, arrow by arrow --------------------------------------------

#[test]
fn a_sponsored_request_with_no_capability_is_refused() {
    let abi = live_abi();
    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    );
    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &view, None, None).err(),
        Some(TransactionRefusal::LiveSponsorRequestedWithoutCapability),
    );
}

#[test]
fn a_sponsorless_request_offered_a_capability_is_refused() {
    let abi = live_abi();
    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    );
    let sponsor = FixtureSponsor::new(90, None);
    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &view,
            Some(&sponsor),
            None,
        )
        .err(),
        Some(TransactionRefusal::LiveSponsorCapabilityWithoutRequest),
    );
}

#[test]
fn an_empty_sponsor_capability_never_downgrades_a_sponsored_request() {
    // The rule §12.5 states in its own sentence. The sponsorless form is
    // buildable for exactly these counts, so a downgrade would have
    // succeeded quietly and returned a transaction the caller did not
    // ask for.
    let abi = live_abi();
    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    );
    let empty = FixtureSponsor::empty(90);
    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &view,
            Some(&empty),
            None
        )
        .err(),
        Some(TransactionRefusal::EmptyLiveSponsorOffer),
    );

    // And the form it would have downgraded to does build, which is what
    // makes the refusal a choice rather than an accident.
    let (sponsorless_request, sponsorless_view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    );
    assert!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &sponsorless_request,
            &sponsorless_view,
            None,
            None,
        )
        .is_ok()
    );
}

#[test]
fn a_sponsored_request_asking_for_change_that_the_capability_offers_none_of_is_refused() {
    let abi = live_abi();
    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::Requested,
    );
    let sponsor = FixtureSponsor::new(90, None);
    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &view,
            Some(&sponsor),
            None,
        )
        .err(),
        Some(TransactionRefusal::SponsorChangeRequestedWithoutDestination),
    );
}

#[test]
fn a_sponsored_request_not_asking_for_change_that_is_offered_one_is_refused() {
    let abi = live_abi();
    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    );
    let sponsor = FixtureSponsor::new(90, Some(ValueField::Explicit(40)));
    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &view,
            Some(&sponsor),
            None,
        )
        .err(),
        Some(TransactionRefusal::SponsorChangeOfferedWithoutRequest),
    );
}

#[test]
fn the_sponsored_request_reports_the_sponsored_form_and_the_sponsorless_the_other() {
    // The equivalence's last term: the form the request asked for is the
    // form the report states, in both directions.
    let abi = live_abi();
    assert_eq!(
        sponsorless().report().form(),
        LiveTransactionForm::Sponsorless
    );

    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::Requested,
    );
    let sponsor = FixtureSponsor::new(90, Some(ValueField::Explicit(40)));
    let built = finalize_live_transfer(
        &reviewed_target(),
        &abi,
        &request,
        &view,
        Some(&sponsor),
        None,
    )
    .expect("the sponsored form finalizes");
    assert_eq!(built.report().form(), LiveTransactionForm::Sponsored);
    assert!(built.report().shape().sponsored());
}

// --- The layouts, as built --------------------------------------------

#[test]
fn the_receipt_family_is_sorted_and_the_coordinator_is_input_zero() {
    let finalization = sponsorless();
    let finalized = finalization.finalized();

    let outpoints: Vec<_> = finalized
        .protected()
        .inputs()
        .iter()
        .map(crate::bytes::TargetInput::outpoint)
        .collect();
    let mut sorted = outpoints.clone();
    sorted.sort_unstable();
    assert_eq!(outpoints, sorted);
    assert_eq!(finalized.receipts()[0].position(), 0);
    assert_eq!(finalized.receipts().len(), 2);
    assert_eq!(finalized.sponsor_inputs(), &[]);
}

#[test]
fn the_destinations_are_in_typed_request_order_and_not_in_owner_order() {
    // §12.2: request order is ABI presentation order. The fixture asks
    // for the second owner first, and the outputs come out that way,
    // which a builder sorting by owner or by amount would not produce.
    let abi = live_abi();
    let finalization = sponsorless();
    let outputs = finalization.finalized().outputs().outputs();

    let second_program = abi
        .destinations()
        .get(
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the second owner has a constructor")
        .instance()
        .program()
        .to_vec();
    let first_program = abi
        .destinations()
        .get(
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the first owner has a constructor")
        .instance()
        .program()
        .to_vec();

    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0].program(), second_program.as_slice());
    assert_eq!(outputs[0].value(), ValueField::Explicit(250));
    assert_eq!(outputs[1].program(), first_program.as_slice());
    assert_eq!(outputs[1].value(), ValueField::Explicit(750));
    for output in outputs {
        assert_eq!(
            output.asset(),
            AssetField::Explicit(AssetId::from_internal(LIVE_PROTOCOL_ASSET)),
        );
    }
}

#[test]
fn the_sponsored_form_places_change_then_the_fee_role_after_the_destinations() {
    let abi = live_abi();
    let (request, view) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::Requested,
    );
    let sponsor = FixtureSponsor::new(90, Some(ValueField::Explicit(40)));
    let built = finalize_live_transfer(
        &reviewed_target(),
        &abi,
        &request,
        &view,
        Some(&sponsor),
        None,
    )
    .expect("the sponsored form finalizes");

    let census = built.finalized().outputs();
    let outputs = census.outputs();
    assert_eq!(outputs.len(), 4);
    assert_eq!(census.destination_range(), (0, 2));
    assert_eq!(census.sponsor_change_position(), Some(2));
    assert_eq!(census.fee_position(), Some(3));

    // The sponsor's two roles carry the reserve asset, and the fee role
    // is the target's own structural one.
    let reserve = AssetField::Explicit(AssetId::from_internal(LIVE_RESERVE_ASSET));
    assert_eq!(outputs[2].asset(), reserve);
    assert_eq!(outputs[2].value(), ValueField::Explicit(40));
    assert_eq!(outputs[3].asset(), reserve);
    assert_eq!(outputs[3].value(), ValueField::Explicit(90));
    assert!(outputs[3].is_fee());
    // And the sponsor's own amount is nowhere in the report (§1.9).
    assert_eq!(built.report().created_total(), 1_000);
}

// --- Recognition, and its refusals ------------------------------------

#[test]
fn a_receipt_with_no_public_view_is_refused() {
    let abi = live_abi();
    let (request, _) = explicit_fixture(
        &abi,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    );
    let empty = view([]);
    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &empty, None, None).err(),
        Some(TransactionRefusal::MissingPublicReceiptView(outpoint(
            0xa1, 0
        ))),
    );
}

#[test]
fn a_receipt_whose_program_no_constructor_produces_is_not_a_live_receipt() {
    let abi = live_abi();
    let first = outpoint(0xa1, 0);
    let foreign = view([crate::view::PublicOutputView::new(
        first,
        AssetField::Explicit(AssetId::from_internal(LIVE_PROTOCOL_ASSET)),
        ValueField::Explicit(1_000),
        vec![0x51, 0x20, 0xee],
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &foreign, None, None).err(),
        Some(TransactionRefusal::ReceiptInputIsNotALiveReceipt(first)),
    );
}

#[test]
fn a_receipt_carrying_another_asset_is_refused() {
    let abi = live_abi();
    let first = outpoint(0xa1, 0);
    let wrong = view([crate::view::PublicOutputView::new(
        first,
        AssetField::Explicit(AssetId::from_internal(LIVE_RESERVE_ASSET)),
        ValueField::Explicit(1_000),
        abi.destinations()
            .get(
                &owner(&FIRST_OWNER),
                LiveTransferRepresentationPlan::Explicit,
            )
            .expect("the first owner has a constructor")
            .instance()
            .program()
            .to_vec(),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &wrong, None, None).err(),
        Some(TransactionRefusal::ReceiptInputCarriesForeignAsset(first)),
    );
}

#[test]
fn an_explicit_request_over_a_committed_receipt_value_is_refused() {
    let abi = live_abi();
    let first = outpoint(0xa1, 0);
    let committed = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::Explicit,
        ValueField::Commitment([0x08; 33]),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &committed, None, None).err(),
        Some(TransactionRefusal::ReceiptInputValueFormRefused(first)),
    );
}

#[test]
fn the_explicit_plan_closes_conservation_locally() {
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
        [destination(&FIRST_OWNER, 999)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None).err(),
        Some(TransactionRefusal::LiveConservationFailed {
            consumed: 1_000,
            created: 999,
        }),
    );
}

#[test]
fn no_admitted_shape_takes_a_count_outside_the_candidate_bounds() {
    let abi = live_abi();
    let outpoints = [
        outpoint(0xb1, 0),
        outpoint(0xb2, 0),
        outpoint(0xb3, 0),
        outpoint(0xb4, 0),
    ];
    let stated = view(outpoints.iter().map(|point| {
        receipt_view(
            &abi,
            *point,
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(250),
        )
    }));
    let request = LiveTransferRequest::new(
        outpoints,
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None).err(),
        Some(TransactionRefusal::UnsupportedLiveShape {
            receipt_inputs: 4,
            destinations: 1,
            sponsor_inputs: 0,
            sponsor_change: false,
        }),
    );
}

#[test]
fn an_outpoint_in_both_regions_is_refused_before_the_sort() {
    let abi = live_abi();
    let shared = outpoint(0xdd, 2);
    let stated = view([receipt_view(
        &abi,
        shared,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::Explicit,
        ValueField::Explicit(1_000),
    )]);
    let request = LiveTransferRequest::new(
        [shared],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");
    let sponsor = FixtureSponsor::new(90, None);

    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &stated,
            Some(&sponsor),
            None,
        )
        .err(),
        Some(TransactionRefusal::SponsorOverlapsReceiptFamily(shared)),
    );
}

#[test]
fn a_sponsor_input_the_caller_cannot_show_is_refused() {
    // The offer names an outpoint and the view says nothing about it.
    // The refusal is about the view rather than about the coin, because
    // that is all the builder knows: it has not seen an asset to call
    // foreign.
    let abi = live_abi();
    let (request, _) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    );
    let receipts_only = view([
        receipt_view(
            &abi,
            outpoint(0xa1, 0),
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(400),
        ),
        receipt_view(
            &abi,
            outpoint(0xa2, 1),
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(600),
        ),
    ]);
    let sponsor = FixtureSponsor::new(90, None);
    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &receipts_only,
            Some(&sponsor),
            None,
        )
        .err(),
        Some(TransactionRefusal::MissingPublicSponsorView(sponsor_coin())),
    );
}

#[test]
fn a_sponsor_input_carrying_an_asset_that_is_not_the_reserve_is_refused() {
    // Both sides of the guard, against one another. The deployment's
    // reserve is welded into the leaves and pushed as a literal by
    // §10.7's isolation fragments, and the fee output this build writes
    // names it; a sponsor input holding anything else funds that output
    // in an asset it does not hold.
    //
    // The two runs differ in the sponsor view's asset and in nothing
    // else — same request, same offer, same receipts — so the refusal
    // is attributable to the asset and to no other difference.
    let abi = live_abi();
    let (request, admitted) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    );
    let sponsor = FixtureSponsor::new(90, None);
    assert!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &admitted,
            Some(&sponsor),
            None,
        )
        .is_ok(),
        "the reserve-asset sponsor input is the admitted case",
    );

    let foreign = view([
        receipt_view(
            &abi,
            outpoint(0xa1, 0),
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(400),
        ),
        receipt_view(
            &abi,
            outpoint(0xa2, 1),
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(600),
        ),
        crate::view::PublicOutputView::new(
            sponsor_coin(),
            // The protocol asset, which is exactly the near miss a
            // deployment welded to two assets can produce: a coin of
            // the right chain and the wrong role.
            AssetField::Explicit(AssetId::from_internal(LIVE_PROTOCOL_ASSET)),
            ValueField::Explicit(130),
            LIVE_SPONSOR_CHANGE_PROGRAM.to_vec(),
        ),
    ]);
    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &foreign,
            Some(&sponsor),
            None,
        )
        .err(),
        Some(TransactionRefusal::LiveSponsorInputCarriesForeignAsset(
            sponsor_coin()
        )),
    );
}

#[test]
fn a_sponsor_input_whose_value_form_the_plan_does_not_read_is_refused() {
    // The asymmetry this closes: a receipt's value form has been gated
    // against the representation plan since the plan existed, and the
    // sponsor side had no counterpart and no prose saying why. So an
    // EXPLICIT request accepted a COMMITTED sponsor value and built a
    // candidate no reader of the request expected.
    //
    // The two runs differ in the sponsor view's VALUE FORM and in
    // nothing else — same request, same offer, same receipts, same
    // reserve asset — so the refusal is attributable to the form.
    let abi = live_abi();
    let (request, admitted) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    );
    let sponsor = FixtureSponsor::new(90, None);
    assert!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &admitted,
            Some(&sponsor),
            None,
        )
        .is_ok(),
        "the explicit sponsor value is the admitted case under an explicit plan",
    );

    let committed = view([
        receipt_view(
            &abi,
            outpoint(0xa1, 0),
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(400),
        ),
        receipt_view(
            &abi,
            outpoint(0xa2, 1),
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(600),
        ),
        crate::view::PublicOutputView::new(
            sponsor_coin(),
            // The RESERVE asset, unchanged, so the asset guard has
            // nothing to say and the form guard is what answers. ADR-015
            // public disposable test material.
            AssetField::Explicit(AssetId::from_internal(LIVE_RESERVE_ASSET)),
            ValueField::Commitment([0x08; 33]),
            LIVE_SPONSOR_CHANGE_PROGRAM.to_vec(),
        ),
    ]);
    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &committed,
            Some(&sponsor),
            None,
        )
        .err(),
        Some(TransactionRefusal::LiveSponsorInputValueFormRefused(
            sponsor_coin()
        )),
        "a committed sponsor value under an explicit plan is not refused for its form",
    );
}

// --- The request's plan against the link's (§12.4) ---------------------

#[test]
fn a_private_request_against_an_explicit_only_link_names_the_absent_plan() {
    // A one-representation link is legitimate, and the request selects
    // its plan without ever seeing the ABI. The two disagreeing is a
    // reachable state, and what it deserves is the refusal that says so:
    // the plan the request asked for was never linked.
    //
    // The receipt view is built from the explicit-only ABI's own
    // program, so nothing about the receipt is wrong. Only the selected
    // plan is, which is what makes the refusal attributable.
    let abi = single_representation_live_abi(LiveTransferRepresentationPlan::Explicit);
    assert_eq!(
        abi.representations(),
        &BTreeSet::from([LiveTransferRepresentationPlan::Explicit]),
    );

    let first = outpoint(0xc7, 0);
    let stated = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::Explicit,
        ValueField::Commitment([0x09; 33]),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [
            destination(&FIRST_OWNER, 600),
            destination(&SECOND_OWNER, 400),
        ],
        LiveTransferRepresentationPlan::PrivateCommitted,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        Some(PublicTestRandomness::from_published_bytes(
            PUBLISHED_RANDOMNESS,
        )),
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(
            &reviewed_target(),
            &abi,
            &request,
            &stated,
            None,
            Some(&FixturePrivateValue),
        )
        .err(),
        Some(TransactionRefusal::RepresentationNotLinked),
    );
}

#[test]
fn an_explicit_request_against_a_private_only_link_names_it_too() {
    // The mirror, so that the check is about the disagreement rather
    // than about one plan being the privileged one.
    let abi = single_representation_live_abi(LiveTransferRepresentationPlan::PrivateCommitted);
    assert_eq!(
        abi.representations(),
        &BTreeSet::from([LiveTransferRepresentationPlan::PrivateCommitted]),
    );

    let first = outpoint(0xc8, 0);
    let stated = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::PrivateCommitted,
        ValueField::Explicit(1_000),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [
            destination(&FIRST_OWNER, 600),
            destination(&SECOND_OWNER, 400),
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None).err(),
        Some(TransactionRefusal::RepresentationNotLinked),
    );
}

#[test]
fn an_owner_the_linked_plan_has_no_constructor_for_is_still_its_own_refusal() {
    // The refusal the new one must not have swallowed. Here the plan is
    // linked and one destination owner has no constructor in it, which
    // is a different fault with a different repair.
    let abi = live_abi();
    let (_, stated) = explicit_fixture(
        &abi,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    );
    let stranger = [0x3b_u8; 32];
    let request = LiveTransferRequest::new(
        [outpoint(0xa1, 0), outpoint(0xa2, 1)],
        [destination(&stranger, 250), destination(&FIRST_OWNER, 750)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None).err(),
        Some(TransactionRefusal::DestinationOwnerHasNoConstructor {
            owner: owner(&stranger),
        }),
    );
}

// --- The private test construction (§12.8) ----------------------------

#[test]
fn the_private_construction_records_the_central_public_fixture_model() {
    let abi = live_abi();
    let first = outpoint(0xc1, 0);
    let stated = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::PrivateCommitted,
        ValueField::Commitment([0x09; 33]),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [
            destination(&FIRST_OWNER, 600),
            destination(&SECOND_OWNER, 400),
        ],
        LiveTransferRepresentationPlan::PrivateCommitted,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        Some(PublicTestRandomness::from_published_bytes(
            PUBLISHED_RANDOMNESS,
        )),
    )
    .expect("the fixture request validates");

    let built = finalize_live_transfer(
        &reviewed_target(),
        &abi,
        &request,
        &stated,
        None,
        Some(&FixturePrivateValue),
    )
    .expect("the private form finalizes");

    let model = built
        .report()
        .construction_model()
        .expect("a private construction records its model");
    assert_eq!(
        model.model(),
        ConfidentialConstructionModel::CentralPublicFixtureConstruction,
    );
    assert_eq!(
        model.demonstrates(),
        &BTreeSet::from([
            PrivateConstructionDemonstration::TargetFeasibility,
            PrivateConstructionDemonstration::SemanticEquivalence,
        ]),
    );
    assert!(
        model
            .non_claims()
            .contains(&PrivateConstructionNonClaim::NotAProductionMultiOwnerPrivacyProtocol)
    );
    assert_eq!(model.non_claims().len(), 4);

    // The destination values are commitments and the asset stays
    // explicit (§6.3), and no public subtotal of the consumed amounts is
    // reported (§6.4).
    for output in built.finalized().outputs().outputs() {
        assert!(matches!(output.value(), ValueField::Commitment(_)));
        assert_eq!(
            output.asset(),
            AssetField::Explicit(AssetId::from_internal(LIVE_PROTOCOL_ASSET)),
        );
    }
    assert_eq!(built.report().consumed_total(), None);
}

#[test]
fn the_private_output_order_follows_the_request_and_not_the_amount() {
    // §12.2's last sentence. The larger destination is asked for first,
    // and its commitment lands at position zero; a builder ordering by
    // amount or by opening would have produced the other order.
    let abi = live_abi();
    let first = outpoint(0xc1, 0);
    let stated = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::PrivateCommitted,
        ValueField::Commitment([0x09; 33]),
    )]);

    let build = |entries: [LiveReceiptDestination; 2]| {
        let request = LiveTransferRequest::new(
            [first],
            entries,
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
            Some(&FixturePrivateValue),
        )
        .expect("the private form finalizes")
    };

    let descending = build([
        destination(&FIRST_OWNER, 900),
        destination(&SECOND_OWNER, 100),
    ]);
    let ascending = build([
        destination(&SECOND_OWNER, 100),
        destination(&FIRST_OWNER, 900),
    ]);

    // The two builds place the same two owners in opposite positions,
    // which is only possible if the order came from the request.
    let programs = |built: &LiveFinalization| {
        built
            .finalized()
            .outputs()
            .outputs()
            .iter()
            .map(|output| output.program().to_vec())
            .collect::<Vec<_>>()
    };
    let one = programs(&descending);
    let other = programs(&ascending);
    assert_ne!(one, other);
    assert_eq!(one[0], other[1]);
    assert_eq!(one[1], other[0]);
}

#[test]
fn a_private_request_with_no_confidential_capability_is_refused() {
    let abi = live_abi();
    let first = outpoint(0xc1, 0);
    let stated = view([receipt_view(
        &abi,
        first,
        &owner(&FIRST_OWNER),
        LiveTransferRepresentationPlan::PrivateCommitted,
        ValueField::Commitment([0x09; 33]),
    )]);
    let request = LiveTransferRequest::new(
        [first],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::PrivateCommitted,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        Some(PublicTestRandomness::from_published_bytes(
            PUBLISHED_RANDOMNESS,
        )),
    )
    .expect("the fixture request validates");

    assert_eq!(
        finalize_live_transfer(&reviewed_target(), &abi, &request, &stated, None, None).err(),
        Some(TransactionRefusal::PrivateValueCapabilityAbsent),
    );
}

#[test]
fn no_model_other_than_the_expected_one_may_be_recorded() {
    assert_eq!(ConfidentialConstructionModel::ALL.len(), 4);
    for model in ConfidentialConstructionModel::ALL {
        let recorded = SelectedConstructionModel::record(*model);
        if *model == ConfidentialConstructionModel::EXPECTED {
            assert!(recorded.is_ok());
        } else {
            assert_eq!(
                recorded.err(),
                Some(TransactionRefusal::ConfidentialConstructionModelNotAdmitted),
            );
        }
    }
}

// --- End to end, to candidate bytes (§12.9) ---------------------------

#[test]
fn a_sponsorless_explicit_transfer_reaches_candidate_bytes() {
    let target = reviewed_target();
    let abi = live_abi();
    let (request, stated) = explicit_fixture(
        &abi,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    );
    let finalization = finalize_live_transfer(&target, &abi, &request, &stated, None, None)
        .expect("the sponsorless explicit form finalizes");
    let report = finalization.report().clone();
    let finalized = finalization.into_finalized();

    let responses: Vec<_> = finalized
        .signing_requests()
        .iter()
        .map(|signing| {
            (
                signing.input(),
                crate::live_signing::LiveOwnerResponse::to(signing, vec![0x5c; 8]),
            )
        })
        .collect();
    let authorized = crate::live_signing::authorize_live_transfer(finalized, responses)
        .expect("both owners authorize");
    let built = crate::live_construct::complete_live_transfer(&target, authorized, report, None)
        .expect("the candidate completes");

    // A complete transaction, its witnesses attached, and bytes that
    // decode back to it.
    assert_eq!(built.transaction().inputs().len(), 2);
    assert_eq!(built.transaction().outputs().len(), 2);
    assert!(built.transaction().has_witness());
    for witness in built.transaction().witnesses() {
        assert_eq!(witness.stack().len(), 3);
    }

    let bytes = built.bytes();
    assert_ne!(bytes.len(), 0);
    assert_eq!(
        crate::bytes::TargetTransaction::decode(&bytes).as_ref(),
        Ok(built.transaction()),
    );

    // The report says what was built, and the §1.6 census keeps its two
    // levels apart.
    assert_eq!(built.report().form(), LiveTransactionForm::Sponsorless);
    assert_eq!(built.report().owners().receipt_inputs(), 2);
    assert_eq!(built.report().owners().distinct_semantic_owners().len(), 2);
    assert_eq!(built.report().consumed_total(), Some(1_000));
    assert_eq!(built.report().created_total(), 1_000);
    assert_eq!(built.report().construction_model(), None);

    // The protected serialization the owners bound to is the same
    // transaction without its witness, so the only thing that moved
    // after the boundary is what the profile excludes (§12.6).
    assert_eq!(
        built.transaction().encode_without_witness(),
        built.authorized().finalized().protected_bytes(),
    );
    assert_eq!(
        built.authorized().check_offered(built.transaction()),
        Ok(())
    );
}

#[test]
fn a_sponsored_transfer_reaches_candidate_bytes_with_the_sponsors_own_witness() {
    let target = reviewed_target();
    let abi = live_abi();
    let (request, stated) = explicit_fixture(
        &abi,
        RequestedForm::Sponsored,
        SponsorChangeRequest::Requested,
    );
    let sponsor = FixtureSponsor::new(90, Some(ValueField::Explicit(40)));
    let finalization =
        finalize_live_transfer(&target, &abi, &request, &stated, Some(&sponsor), None)
            .expect("the sponsored form finalizes");
    let report = finalization.report().clone();
    let finalized = finalization.into_finalized();

    let responses: Vec<_> = finalized
        .signing_requests()
        .iter()
        .map(|signing| {
            (
                signing.input(),
                crate::live_signing::LiveOwnerResponse::to(signing, vec![0x5c; 8]),
            )
        })
        .collect();
    let authorized = crate::live_signing::authorize_live_transfer(finalized, responses)
        .expect("both owners authorize");
    let built =
        crate::live_construct::complete_live_transfer(&target, authorized, report, Some(&sponsor))
            .expect("the sponsored candidate completes");

    assert_eq!(built.transaction().inputs().len(), 3);
    assert_eq!(built.transaction().outputs().len(), 4);
    // The two receipt inputs carry the three-item script-path witness;
    // the sponsor's own is the sponsor's, and §1.9 keeps it outside the
    // owner responses.
    assert_eq!(built.transaction().witnesses()[0].stack().len(), 3);
    assert_eq!(built.transaction().witnesses()[1].stack().len(), 3);
    assert_eq!(built.transaction().witnesses()[2].stack().len(), 2);
    assert_eq!(built.authorized().witnesses().len(), 2);

    let bytes = built.bytes();
    assert_eq!(
        crate::bytes::TargetTransaction::decode(&bytes).as_ref(),
        Ok(built.transaction()),
    );
}

// -------------------------------------------------------------------------
// The private lane's own entry point (rule:guide-ctf-exec:per-output-retirement)
// -------------------------------------------------------------------------

/// A materializer collaborator that is never reached.
///
/// Every test below refuses before any materialization is attempted,
/// and that is the property under test: a private finalization that
/// disagreed with its request or its openings must say so before it
/// spends any proof work, because a refusal produced after the fact
/// would be attributable to two things.
struct UnreachedMaterializer;

impl crate::live_materialize::ConfidentialProofMaterializer for UnreachedMaterializer {
    fn origin(&self) -> crate::live_materialize::CommitmentOrigin {
        unreachable!("no test below reaches the materializer")
    }

    fn value_commitment(
        &self,
        _explicit_asset: AssetId,
        _semantic_amount: u64,
        _value_blinder: &[u8; crate::live_materialize::SCALAR_BYTES],
    ) -> Option<crate::live_materialize::MaterializerCommitment> {
        unreachable!("no test below reaches the materializer")
    }

    fn nonce_commitment(
        &self,
        _nonce_input: &[u8; crate::live_materialize::SCALAR_BYTES],
    ) -> Option<[u8; crate::bytes::COMMITMENT_BYTES]> {
        unreachable!("no test below reaches the materializer")
    }

    fn range_proof(
        &self,
        _request: &crate::live_materialize::RangeproofRequest<'_>,
    ) -> Option<crate::live_materialize::MaterializedRangeproof> {
        unreachable!("no test below reaches the materializer")
    }
}

impl crate::live_materialize::IndependentCommitmentCheck for UnreachedMaterializer {
    fn origin(&self) -> crate::live_materialize::CommitmentOrigin {
        unreachable!("no test below reaches the checker")
    }

    fn recompute(
        &self,
        _explicit_asset: AssetId,
        _semantic_amount: u64,
        _value_blinder: &[u8; crate::live_materialize::SCALAR_BYTES],
    ) -> Option<crate::live_materialize::IndependentCommitment> {
        unreachable!("no test below reaches the checker")
    }

    fn solve_balancing_blinder(
        &self,
        _input_blinder_sum: &[u8; crate::live_materialize::SCALAR_BYTES],
        _other_blinders: &[[u8; crate::live_materialize::SCALAR_BYTES]],
    ) -> Option<[u8; crate::live_materialize::SCALAR_BYTES]> {
        unreachable!("no test below reaches the checker")
    }
}

/// An empty frozen view, for the refusals that fire before lookup.
fn empty_fixtures() -> crate::live_materialize::FrozenConfidentialFixtureView {
    crate::live_materialize::FrozenConfidentialFixtureView::new(std::collections::BTreeMap::new())
}

/// One opening reference into a fixture nothing resolves.
fn opening(index: usize) -> crate::live_materialize::FixtureOpeningReference {
    crate::live_materialize::FixtureOpeningReference::new(
        "unresolved".to_owned(),
        [0x11; 32],
        index,
    )
}

/// One value commitment of the admitted parity `prefix`.
///
/// A private predecessor's value field IS a commitment — receipt
/// recognition refuses an explicit one under the private plan, which is
/// how this fixture found out it was wrong the first time — so the two
/// spent outputs carry the two admitted prefixes rather than amounts.
/// The bytes after the prefix are meaningless public test material: no
/// test below reaches a point where they are opened.
fn commitment(prefix: u8) -> [u8; crate::bytes::COMMITMENT_BYTES] {
    let mut bytes = [0x5a_u8; crate::bytes::COMMITMENT_BYTES];
    bytes[0] = prefix;
    bytes
}

/// The private two-in two-out request and the view that recognizes it.
fn private_fixture(
    abi: &CandidateLiveTransferAbi,
    form: RequestedForm,
) -> (LiveTransferRequest, PublicConstructionView) {
    let first = outpoint(0xb1, 0);
    let second = outpoint(0xb2, 1);
    let stated = view([
        receipt_view(
            abi,
            first,
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::PrivateCommitted,
            ValueField::Commitment(commitment(0x08)),
        ),
        receipt_view(
            abi,
            second,
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::PrivateCommitted,
            ValueField::Commitment(commitment(0x09)),
        ),
    ]);
    let request = LiveTransferRequest::new(
        [first, second],
        [
            destination(&FIRST_OWNER, 1_200),
            destination(&SECOND_OWNER, 800),
        ],
        LiveTransferRepresentationPlan::PrivateCommitted,
        form,
        SponsorChangeRequest::NotRequested,
        Some(PublicTestRandomness::from_published_bytes(
            PUBLISHED_RANDOMNESS,
        )),
    )
    .expect("the fixture request validates");
    (request, stated)
}

/// The openings for the fixture above, with `inputs` and `destinations`
/// entries rather than the two-and-two the request needs.
fn openings(inputs: usize, destinations: usize) -> crate::live_construct::PrivateLiveOpenings {
    crate::live_construct::PrivateLiveOpenings::new(
        (0..inputs)
            .map(|index| crate::live_construct::PrivateInputOpening {
                opening: opening(index),
                explicit_amount: 1_000,
                zero_asset_blinder: [0_u8; crate::live_materialize::SCALAR_BYTES],
            })
            .collect(),
        (0..destinations)
            .map(|index| crate::live_construct::PrivateDestinationOpening {
                fixture: opening(index),
                role: if index == 0 {
                    crate::live_materialize::ConfidentialOutputRole::Primary
                } else {
                    crate::live_materialize::ConfidentialOutputRole::Balancing
                },
            })
            .collect(),
        crate::live_materialize::NonProtocolFundingRegion::default(),
        crate::live_materialize::ConfidentialMaterializationProfiles {
            reproducibility_contract: target_elements::ReproducibilityContract::ByteIdentity,
            custody_profile:
                crate::live_materialize::ConfidentialCustodyProfile::CentralPublicFixtures,
            materializer_profile:
                crate::live_materialize::ConfidentialMaterializerProfile::GuideCtfDeterministicV1,
            proof_profile:
                crate::live_materialize::ConfidentialProofProfile::ExplicitAssetRangeproofV1,
            nonce_profile:
                crate::live_materialize::ConfidentialNonceProfile::DeterministicDerivedV1,
            order_profile: crate::live_materialize::ConfidentialOrderProfile::FixtureFixedOrder,
            retry_profile: crate::live_materialize::ConfidentialRetryProfile::NoRetry,
        },
    )
}

#[test]
fn the_private_entry_point_refuses_an_explicit_request_by_its_own_name() {
    // Not RepresentationNotLinked and not a capability refusal: the two
    // lanes have two entry points now, and a caller standing at the
    // wrong one is told which one it is at.
    let abi = live_abi();
    let (request, stated) = explicit_fixture(
        &abi,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    );
    assert_eq!(
        crate::live_construct::finalize_private_live_transfer(
            &abi,
            &request,
            &stated,
            &openings(2, 2),
            &empty_fixtures(),
            &UnreachedMaterializer,
            &UnreachedMaterializer,
        )
        .err(),
        Some(
            TransactionRefusal::PrivateFinalizationIsNotTheExplicitLane {
                representation: LiveTransferRepresentationPlan::Explicit,
            }
        ),
    );
}

#[test]
fn a_sponsored_private_request_refuses_on_the_absent_signer_rather_than_building() {
    // The candidate is buildable and would be unauthorizable, which is
    // the shape that produces a refusal attributable to the wrong
    // thing. So the lane stops at the form.
    let abi = live_abi();
    let (request, stated) = private_fixture(&abi, RequestedForm::Sponsored);
    assert_eq!(
        crate::live_construct::finalize_private_live_transfer(
            &abi,
            &request,
            &stated,
            &openings(2, 2),
            &empty_fixtures(),
            &UnreachedMaterializer,
            &UnreachedMaterializer,
        )
        .err(),
        Some(TransactionRefusal::PrivateFinalizationIsSponsorless),
    );
}

#[test]
fn openings_that_do_not_cover_the_request_are_refused_on_both_sides() {
    let abi = live_abi();
    let (request, stated) = private_fixture(&abi, RequestedForm::Sponsorless);

    // One opening short on the input side.
    assert_eq!(
        crate::live_construct::finalize_private_live_transfer(
            &abi,
            &request,
            &stated,
            &openings(1, 2),
            &empty_fixtures(),
            &UnreachedMaterializer,
            &UnreachedMaterializer,
        )
        .err(),
        Some(TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest {
            offered: 1,
            required: 2,
        }),
    );

    // And one over on the destination side. Both directions, because a
    // check written as "at least as many" would pass the second.
    assert_eq!(
        crate::live_construct::finalize_private_live_transfer(
            &abi,
            &request,
            &stated,
            &openings(2, 3),
            &empty_fixtures(),
            &UnreachedMaterializer,
            &UnreachedMaterializer,
        )
        .err(),
        Some(TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest {
            offered: 3,
            required: 2,
        }),
    );
}

#[test]
fn a_materializer_refusal_arrives_as_the_materializers_own_word() {
    // The fixture handle resolves to nothing, so the materializer
    // refuses at its fixture stage. What matters here is that the
    // refusal is WRAPPED rather than flattened: the caller gets the
    // materializer's census member and not a re-spelling of it.
    let abi = live_abi();
    let (request, stated) = private_fixture(&abi, RequestedForm::Sponsorless);
    let refusal = crate::live_construct::finalize_private_live_transfer(
        &abi,
        &request,
        &stated,
        &openings(2, 2),
        &empty_fixtures(),
        &UnreachedMaterializer,
        &UnreachedMaterializer,
    )
    .expect_err("an unresolvable fixture handle refuses");
    assert!(
        matches!(
            refusal,
            TransactionRefusal::PrivateMaterializationRefused(_)
        ),
        "the materializer's refusal was not carried through: {refusal:?}",
    );
}
