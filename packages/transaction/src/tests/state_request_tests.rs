//! The typed maturity-announcement request and its two censuses (§12.3,
//! §2.3).

use std::collections::BTreeSet;

use realization::Cycle;
use tapscript::StateProgramWitness;

use super::reviewed_target;
use super::state_support::{
    FixtureLiveCurve, FixtureScriptPathVerifier, FixtureStateCurve, fixture_sign, validated_view,
};
use crate::error::TransactionRefusal;
use crate::live_request::{RequestedForm, SponsorChangeRequest};
use crate::operator_right::OperatorRightRegistry;
use crate::operator_signing::{OPERATOR_SIGHASH_TYPE_BYTE, OperatorSigningResponse};
use crate::state_abi::derive_maturity_announcement_abi;
use crate::state_construct::construct_maturity_announcement;
use crate::state_finalize::{FinalizedMaturityAnnouncement, finalize_maturity_announcement};
use crate::state_request::{
    MaturityAnnouncementRequest, MaturityRequestDecodingRefusal, SelectableMaturityRequestFacet,
    UnselectableMaturityRequestFacet,
};
use crate::state_signing::{OperatorSigningStarted, SubmitReadyMaturityAnnouncement};

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
    // The semantic request remains separate from its funding context.
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

fn finalized_request() -> FinalizedMaturityAnnouncement {
    let target = reviewed_target();
    let view = validated_view();
    let abi = derive_maturity_announcement_abi(&target, &view).unwrap();
    let request = MaturityAnnouncementRequest::new(
        Cycle::new(8),
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .unwrap();
    finalize_maturity_announcement(
        construct_maturity_announcement(&target, &abi, &view, &request, &FixtureStateCurve)
            .unwrap(),
    )
}

fn bound_request(finalized: &FinalizedMaturityAnnouncement) -> SubmitReadyMaturityAnnouncement<'_> {
    let target = reviewed_target();
    let state = OperatorSigningStarted::open(finalized, &target, &FixtureLiveCurve).unwrap();
    let binding = state.request().binding();
    let signature = fixture_sign(
        binding.key().bytes(),
        state.request().message().with_vector_grown(),
    );
    let response = OperatorSigningResponse::new(
        state.request().input_index(),
        signature.to_vec(),
        OPERATOR_SIGHASH_TYPE_BYTE,
        state.protected_bytes().to_vec(),
        binding.key().clone(),
        binding.deployment().clone(),
        binding.capability_revision(),
    );
    let mut registry = OperatorRightRegistry::default();
    let right = registry
        .issue(state.construction_right_scope(), state.protected_bytes())
        .unwrap();
    state
        .authorize(&mut registry, right, [response], &FixtureScriptPathVerifier)
        .unwrap()
        .bind_for_submission(&target)
        .unwrap()
}

#[test]
fn cycle_decoder_inverts_the_bound_witness_writer_and_refuses_wrong_widths() {
    let finalized = finalized_request();
    let ready = bound_request(&finalized);
    let construction = finalized.construction();
    let abi = construction.abi();
    let role = abi
        .witness_roles()
        .iter()
        .find(|record| record.role() == StateProgramWitness::RequestedCycle)
        .unwrap();
    let encoded = &ready.witness().stack()[role.position()];
    assert_eq!(
        MaturityAnnouncementRequest::decode_announced_cycle(abi, encoded),
        Ok(construction.request().announced_cycle())
    );
    // Truncation is the subject answering the malformed-cycle-encoding row
    // in packages/vectors/src/maturity_safety.rs; extension checks the other direction.
    assert_eq!(
        MaturityAnnouncementRequest::decode_announced_cycle(abi, &encoded[..encoded.len() - 1]),
        Err(MaturityRequestDecodingRefusal::CycleWidth {
            declared: 8,
            offered: 7,
        })
    );
    let mut extended = encoded.clone();
    extended.push(0);
    assert_eq!(
        MaturityAnnouncementRequest::decode_announced_cycle(abi, &extended),
        Err(MaturityRequestDecodingRefusal::CycleWidth {
            declared: 8,
            offered: 9,
        })
    );
}

#[test]
fn cycle_decoder_admits_the_whole_unsigned_domain() {
    let finalized = finalized_request();
    let abi = finalized.construction().abi();
    // state_announcement.rs: reverse_cycle reverses all eight bytes and
    // biases unsigned order into signed order; window uses 64-bit
    // comparisons. Width is the encoding boundary, not signed-number range.
    for value in [0, (1_u64 << 63) - 1, 1_u64 << 63, u64::MAX] {
        assert_eq!(
            MaturityAnnouncementRequest::decode_announced_cycle(abi, &value.to_be_bytes()),
            Ok(Cycle::new(value))
        );
    }
}

fn every_decoding_refusal_is_censused(refusal: MaturityRequestDecodingRefusal) {
    match refusal {
        MaturityRequestDecodingRefusal::CycleWidth { .. } => {}
    }
}

#[test]
fn the_decoding_refusal_vocabulary_has_no_wildcard_guard() {
    every_decoding_refusal_is_censused(MaturityRequestDecodingRefusal::CycleWidth {
        declared: 8,
        offered: 7,
    });
}
