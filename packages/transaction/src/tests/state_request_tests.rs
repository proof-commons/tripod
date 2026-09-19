//! The typed maturity-announcement request and its two censuses (§12.3,
//! §2.3).

use std::collections::BTreeSet;

use realization::Cycle;

use crate::error::TransactionRefusal;
use crate::live_request::{RequestedForm, SponsorChangeRequest};
use crate::state_request::{
    MaturityAnnouncementRequest, SelectableMaturityRequestFacet, UnselectableMaturityRequestFacet,
};

// --- The two censuses -------------------------------------------------

#[test]
fn the_selectable_census_is_the_three_choices_the_guide_admits() {
    assert_eq!(SelectableMaturityRequestFacet::ALL.len(), 3);
    let distinct: BTreeSet<_> = SelectableMaturityRequestFacet::ALL
        .iter()
        .copied()
        .collect();
    assert_eq!(distinct.len(), SelectableMaturityRequestFacet::ALL.len());
}

#[test]
fn a_request_is_built_from_exactly_the_three_selectable_choices_and_reads_them_back() {
    // The census is three, the constructor takes three, and the three
    // read back: that is the whole of what a caller selects.
    let request = MaturityAnnouncementRequest::new(
        Cycle::new(41),
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .expect("a sponsorless request taking no change is admitted");

    assert_eq!(request.announced_cycle(), Cycle::new(41));
    assert_eq!(request.requested_form(), RequestedForm::Sponsorless);
    assert_eq!(request.sponsor_change(), SponsorChangeRequest::NotRequested);
}

#[test]
fn the_unselectable_census_is_the_fifteen_choices_the_two_lists_refuse() {
    // §12.3's twelve bullets and §2.3's thirteen, with the ten pairs
    // that name one thing counted once: the predecessor STATE input and
    // its metadata, the successor metadata and program, the operator
    // identity, the static subtree, the internal key, the representation
    // nonce, the fee role, and the control block.
    assert_eq!(UnselectableMaturityRequestFacet::ALL.len(), 15);
    let distinct: BTreeSet<_> = UnselectableMaturityRequestFacet::ALL
        .iter()
        .copied()
        .collect();
    assert_eq!(distinct.len(), UnselectableMaturityRequestFacet::ALL.len());

    for facet in UnselectableMaturityRequestFacet::ALL {
        assert_ne!(facet.origin(), "", "{facet:?} names no origin");
    }
}

// --- The two forms ----------------------------------------------------

#[test]
fn the_sponsored_form_is_representable_and_reads_back_as_sponsored() {
    // A request that could not ask for the sponsored form could not be
    // refused for asking, which is why the arm is representable before
    // anything emits it.
    let request = MaturityAnnouncementRequest::new(
        Cycle::new(7),
        RequestedForm::Sponsored,
        SponsorChangeRequest::Requested,
    )
    .expect("a sponsored request taking change is admitted");

    assert_eq!(request.requested_form(), RequestedForm::Sponsored);
    assert!(request.requested_form().sponsored());
    assert_eq!(request.sponsor_change(), SponsorChangeRequest::Requested);
}

#[test]
fn two_requests_differing_only_in_the_form_are_unequal() {
    let sponsorless = MaturityAnnouncementRequest::new(
        Cycle::new(12),
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .expect("the sponsorless fixture is admitted");
    let sponsored = MaturityAnnouncementRequest::new(
        Cycle::new(12),
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    )
    .expect("the sponsored fixture is admitted");

    assert_ne!(sponsorless, sponsored);
    assert_eq!(sponsorless.announced_cycle(), sponsored.announced_cycle());
    assert_eq!(sponsorless.sponsor_change(), sponsored.sponsor_change());
}

#[test]
fn a_sponsorless_request_asking_for_sponsor_change_is_refused() {
    // The one contradiction the request settles alone: no sponsor
    // region, so no residual for a change output to carry.
    assert_eq!(
        MaturityAnnouncementRequest::new(
            Cycle::new(3),
            RequestedForm::Sponsorless,
            SponsorChangeRequest::Requested,
        ),
        Err(TransactionRefusal::SponsorChangeWithoutSponsoredForm),
    );
}
