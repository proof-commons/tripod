//! The typed live-transfer request and its two censuses (§12.3, §12.4).

use std::collections::BTreeSet;

use linker::live_backend::LiveTransferRepresentationPlan;

use super::live_support::{FIRST_OWNER, PUBLISHED_RANDOMNESS, SECOND_OWNER, owner};
use super::outpoint;
use crate::error::TransactionRefusal;
use crate::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SelectableRequestFacet, SponsorChangeRequest, UnselectableRequestFacet,
};

/// One destination for `bytes`'s owner at `amount`.
fn destination(bytes: &[u8], amount: u64) -> LiveReceiptDestination {
    LiveReceiptDestination::new(
        owner(bytes),
        ProtocolValue::new(amount).expect("the fixture amounts are positive"),
    )
}

/// The explicit request consuming two receipts and creating two.
fn explicit_request() -> Result<LiveTransferRequest, TransactionRefusal> {
    LiveTransferRequest::new(
        [outpoint(0x01, 0), outpoint(0x02, 0)],
        [
            destination(&FIRST_OWNER, 300),
            destination(&SECOND_OWNER, 700),
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
}

// --- The two censuses -------------------------------------------------

#[test]
fn the_selectable_census_is_the_eight_choices_the_guide_admits() {
    assert_eq!(SelectableRequestFacet::ALL.len(), 8);
    let distinct: BTreeSet<_> = SelectableRequestFacet::ALL.iter().copied().collect();
    assert_eq!(distinct.len(), SelectableRequestFacet::ALL.len());
}

#[test]
fn the_unselectable_census_is_the_fourteen_choices_the_guide_refuses() {
    // §12.3's second bullet list, counted: predecessor owner, class, and
    // asset; output class and asset; constructor bytes; target program;
    // coordinator; family positions; witness order; target fee role;
    // issuance; destruction; specialized projection.
    assert_eq!(UnselectableRequestFacet::ALL.len(), 14);
    let distinct: BTreeSet<_> = UnselectableRequestFacet::ALL.iter().copied().collect();
    assert_eq!(distinct.len(), UnselectableRequestFacet::ALL.len());
}

#[test]
fn a_plain_request_selects_the_five_mandatory_choices_and_no_more() {
    let request = explicit_request().expect("the fixture request validates");
    assert_eq!(
        request.selected(),
        BTreeSet::from([
            SelectableRequestFacet::ReceiptInputOutpoints,
            SelectableRequestFacet::OrderedDestinationEntries,
            SelectableRequestFacet::DestinationOwners,
            SelectableRequestFacet::DestinationSemanticValues,
            SelectableRequestFacet::RepresentationPlan,
        ]),
    );
}

#[test]
fn a_private_sponsored_request_selects_all_eight() {
    let request = LiveTransferRequest::new(
        [outpoint(0x01, 0)],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::PrivateCommitted,
        RequestedForm::Sponsored,
        SponsorChangeRequest::Requested,
        Some(PublicTestRandomness::from_published_bytes(
            PUBLISHED_RANDOMNESS,
        )),
    )
    .expect("the fixture request validates");

    assert_eq!(
        request.selected(),
        SelectableRequestFacet::ALL.iter().copied().collect(),
    );
}

// --- Canonical receipt ordering ---------------------------------------

#[test]
fn the_receipt_family_is_in_canonical_outpoint_order_by_construction() {
    let request = LiveTransferRequest::new(
        [outpoint(0x09, 1), outpoint(0x02, 7), outpoint(0x09, 0)],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    let ordered: Vec<_> = request.receipts().iter().copied().collect();
    assert_eq!(
        ordered,
        vec![outpoint(0x02, 7), outpoint(0x09, 0), outpoint(0x09, 1)],
    );
}

#[test]
fn a_duplicate_receipt_outpoint_is_refused_rather_than_collapsed() {
    let repeated = outpoint(0x05, 2);
    let result = LiveTransferRequest::new(
        [repeated, outpoint(0x06, 0), repeated],
        [destination(&FIRST_OWNER, 1_000)],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    );
    assert_eq!(
        result,
        Err(TransactionRefusal::DuplicateReceiptOutpoint(repeated)),
    );
}

#[test]
fn an_empty_receipt_family_and_an_empty_destination_census_are_both_refused() {
    assert_eq!(
        LiveTransferRequest::new(
            [],
            [destination(&FIRST_OWNER, 1_000)],
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        ),
        Err(TransactionRefusal::EmptyReceiptSelection),
    );
    assert_eq!(
        LiveTransferRequest::new(
            [outpoint(0x01, 0)],
            [],
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        ),
        Err(TransactionRefusal::EmptyDestinationCensus),
    );
}

// --- Request order is presentation order, not identity ----------------

#[test]
fn two_destinations_of_one_owner_and_one_value_are_two_receipts() {
    // The difference between a destination census and a receipt
    // selection: a repeated outpoint is a family nobody has, and a
    // repeated destination entry is an even split.
    let request = LiveTransferRequest::new(
        [outpoint(0x01, 0)],
        [
            destination(&FIRST_OWNER, 500),
            destination(&FIRST_OWNER, 500),
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("an even split is an ordinary transfer");

    assert_eq!(request.destinations().len(), 2);
    assert_eq!(request.destination_owners().len(), 1);
    let multiset = request.destination_multiset();
    assert_eq!(multiset.len(), 1);
    assert_eq!(multiset.values().copied().collect::<Vec<_>>(), vec![2]);
    assert_eq!(request.destination_total(), Some(1_000));
}

#[test]
fn reordering_the_destinations_changes_the_presentation_and_not_the_semantics() {
    let one = LiveTransferRequest::new(
        [outpoint(0x01, 0)],
        [
            destination(&FIRST_OWNER, 300),
            destination(&SECOND_OWNER, 700),
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");
    let other = LiveTransferRequest::new(
        [outpoint(0x01, 0)],
        [
            destination(&SECOND_OWNER, 700),
            destination(&FIRST_OWNER, 300),
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the fixture request validates");

    // The ABI presentation differs, which is what every owner signature
    // commits to.
    assert_ne!(one.destinations(), other.destinations());
    // The semantic projection does not.
    assert_eq!(one.destination_multiset(), other.destination_multiset());
    assert_eq!(one.destination_owners(), other.destination_owners());
}

// --- The destination type ---------------------------------------------

#[test]
fn a_destination_worth_nothing_is_not_a_destination() {
    assert_eq!(
        ProtocolValue::new(0),
        Err(TransactionRefusal::DestinationValueIsZero),
    );
    assert_eq!(ProtocolValue::new(1).map(ProtocolValue::amount), Ok(1),);
}

#[test]
fn a_destination_is_an_owner_and_a_value_and_two_of_them_differ_only_in_those() {
    let one = destination(&FIRST_OWNER, 500);
    assert_eq!(one.owner(), &owner(&FIRST_OWNER));
    assert_eq!(one.value().amount(), 500);
    assert_ne!(one, destination(&SECOND_OWNER, 500));
    assert_ne!(one, destination(&FIRST_OWNER, 501));
    assert_eq!(one, destination(&FIRST_OWNER, 500));
}

// --- The representation's randomness rule -----------------------------

#[test]
fn an_explicit_request_offering_published_randomness_is_refused() {
    assert_eq!(
        LiveTransferRequest::new(
            [outpoint(0x01, 0)],
            [destination(&FIRST_OWNER, 1_000)],
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            Some(PublicTestRandomness::from_published_bytes(
                PUBLISHED_RANDOMNESS,
            )),
        ),
        Err(TransactionRefusal::PublicTestRandomnessWithoutPrivateForm),
    );
}

#[test]
fn a_private_request_offering_none_is_refused() {
    assert_eq!(
        LiveTransferRequest::new(
            [outpoint(0x01, 0)],
            [destination(&FIRST_OWNER, 1_000)],
            LiveTransferRepresentationPlan::PrivateCommitted,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        ),
        Err(TransactionRefusal::PrivateFormWithoutPublicTestRandomness),
    );
}

// --- §12.5's first arrow, at the request ------------------------------

#[test]
fn a_sponsorless_request_asking_for_sponsor_change_is_refused() {
    assert_eq!(
        LiveTransferRequest::new(
            [outpoint(0x01, 0)],
            [destination(&FIRST_OWNER, 1_000)],
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::Requested,
            None,
        ),
        Err(TransactionRefusal::SponsorChangeWithoutSponsoredForm),
    );
}
