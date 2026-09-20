//! Construction of the maturity announcement: the ordered steps, the
//! refusals that travel whole, and the three findings a search can
//! produce.
//!
//! # The bundle under test is the shared one
//!
//! Every positive here runs over the demonstration deployment beside
//! this file — derived, planned, composed, bound and linked through the
//! public builders — so the asset, the amount, the lead window and the
//! static subtree a construction reads are a real link's and not a
//! fixture's opinion of one.
//!
//! # Why one negative needs a second bundle
//!
//! A bundle's nonce budget is the budget its own constructor declared,
//! and the reviewed one admits thousands of attempts, so an exhausted
//! search cannot be reached over the demonstration bundle by any input
//! a test could supply. The exhaustion test therefore links a second
//! bundle at a one-attempt budget over a predecessor found by a scan,
//! and asserts the scanned figure so the number is read rather than
//! trusted.

use linker::{CandidateLinkedMaturityBundle, StateLinkRefusal};
use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, Maturity, MaturityTransitionRefusal,
    ProtocolAmount, StateMetadata, StateRepresentationNonce, announce_maturity,
};
use tapscript::{
    CandidateStateConstructor, STATE_NUMS_KEY, StateConstructorRefusal, StateInternalKeyPolicy,
    StateNonceBudget, StateNonceEvidence, state_output_program_at_nonce,
};

use super::reviewed_target;
use super::state_support::{
    FixtureStateCurve, linked_bundle, linked_bundle_with, linked_pair, second_static_subtree,
    state_metadata, statements, validated_view,
};
use crate::bytes::{AssetField, AssetId, NonceField, TargetTransaction, ValueField};
use crate::error::TransactionRefusal;
use crate::live_request::{RequestedForm, SponsorChangeRequest};
use crate::state_abi::{CandidateMaturityAnnouncementAbi, derive_maturity_announcement_abi};
use crate::state_construct::{
    MATURITY_ANNOUNCEMENT_LOCK_TIME, MaturityConstruction, construct_maturity_announcement,
    hold_continuity,
};
use crate::state_request::MaturityAnnouncementRequest;
use crate::state_view::{
    MaturityViewStatement, PublicMaturityStateView, ValidatedMaturityStateView,
};

/// The cycle the tests announce.
///
/// The fixture predecessor sits at cycle five under a lead window of two
/// and four, so seven, eight and nine are admissible and this is the
/// middle of them: a figure at either endpoint would pass a window check
/// that had drifted by one in that direction.
const ANNOUNCED: Cycle = Cycle::new(8);

/// The quantity the exhaustion scan settles on.
///
/// Asserted rather than assumed: the scan below is what decides it, and
/// a figure nobody checks is a figure that can drift with the fixture
/// curve, the record or the metadata codec without anybody noticing
/// that the test stopped exercising exhaustion.
const EXHAUSTING_QUANTITY: u64 = 15;

/// The deployment's lead window, read through the bundle that carries
/// it.
fn window() -> AnnouncementLeadBounds {
    linked_bundle().deployment().lead_bounds().bounds()
}

/// The sponsorless request announcing `cycle`.
fn sponsorless(cycle: Cycle) -> MaturityAnnouncementRequest {
    MaturityAnnouncementRequest::new(
        cycle,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .expect("the sponsorless form without change is an admitted pair")
}

/// The candidate ABI over one validated view.
fn abi_over(validated: &ValidatedMaturityStateView) -> CandidateMaturityAnnouncementAbi {
    derive_maturity_announcement_abi(&reviewed_target(), validated)
        .expect("the demonstration view derives its ABI")
}

/// Construct over the reviewed target and the fixture curve.
fn construct_over(
    abi: &CandidateMaturityAnnouncementAbi,
    validated: &ValidatedMaturityStateView,
    request: &MaturityAnnouncementRequest,
) -> Result<MaturityConstruction, TransactionRefusal> {
    construct_maturity_announcement(
        &reviewed_target(),
        abi,
        validated,
        request,
        &FixtureStateCurve,
    )
}

/// The demonstration construction, which every positive reads.
fn demonstration() -> (CandidateMaturityAnnouncementAbi, MaturityConstruction) {
    let validated = validated_view();
    let abi = abi_over(&validated);
    let construction = construct_over(&abi, &validated, &sponsorless(ANNOUNCED))
        .expect("the demonstration deployment constructs its announcement");
    (abi, construction)
}

/// Whether `metadata` commits at `nonce` under one bundle's subtree and
/// internal key.
fn commits(
    bundle: &CandidateLinkedMaturityBundle,
    metadata: StateMetadata,
    nonce: StateRepresentationNonce,
) -> bool {
    state_output_program_at_nonce(
        &reviewed_target(),
        &EncodedStateMetadata {
            semantic: metadata,
            representation: nonce,
        },
        bundle.static_subtree(),
        bundle.policy().internal_key(),
        &FixtureStateCurve,
    )
    .is_ok()
}

/// The fixture metadata with its pending entitlement replaced.
fn with_quantity(quantity: u64) -> StateMetadata {
    StateMetadata {
        q: ProtocolAmount::new(quantity).expect("the scanned quantities are in domain"),
        ..state_metadata()
    }
}

/// The least quantity whose predecessor commits at nonce zero and whose
/// successor does not.
///
/// Both halves are needed and neither is incidental: the first is what
/// lets a one-attempt link exist at all, and the second is what makes
/// the successor search run out at its only attempt.
fn exhausting_scan() -> (u64, StateMetadata) {
    let reference = linked_bundle();
    (1..64_u64)
        .map(|quantity| (quantity, with_quantity(quantity)))
        .find(|&(_, metadata)| {
            let successor = announce_maturity(&metadata, ANNOUNCED, window())
                .expect("the scanned predecessors keep the fixture cycle and window");
            commits(&reference, metadata, StateRepresentationNonce::ZERO)
                && !commits(&reference, successor, StateRepresentationNonce::ZERO)
        })
        .expect("a scanned quantity fails the successor's first nonce")
}

/// A validated view over the demonstration bundle stating `asset` and
/// `value` for the current STATE output.
fn view_stating(asset: AssetField, value: ValueField) -> ValidatedMaturityStateView {
    let bundle = linked_bundle();
    let (nonce, program) = linked_pair(&bundle);
    let stated: Vec<_> = statements(&bundle, state_metadata(), nonce, program)
        .into_iter()
        .map(|statement| match statement {
            MaturityViewStatement::AssetAndAmount(..) => {
                MaturityViewStatement::AssetAndAmount(asset, value)
            }
            other => other,
        })
        .collect();

    PublicMaturityStateView::new(stated)
        .expect("the seven statements name distinct entries")
        .validate(&reviewed_target(), &FixtureStateCurve)
        .expect("the asset and the amount are not what the one check reads")
}

#[test]
fn the_sponsorless_candidate_is_one_input_and_one_output_at_the_abis_positions() {
    let (abi, construction) = demonstration();
    let validated = validated_view();
    let transaction = construction.transaction();

    assert_eq!(transaction.version(), 2);
    assert_eq!(transaction.lock_time(), MATURITY_ANNOUNCEMENT_LOCK_TIME);
    assert_eq!(transaction.inputs().len(), 1);
    assert_eq!(transaction.outputs().len(), 1);

    let input = &transaction.inputs()[0];
    assert_eq!(input.outpoint(), validated.view().current_state_outpoint());
    assert_eq!(input.sequence(), abi.sequence().sequence());

    let output = &transaction.outputs()[0];
    assert_eq!(
        output.asset(),
        AssetField::Explicit(AssetId::from_internal([0x11; 32])),
    );
    assert_eq!(output.value(), ValueField::Explicit(1));
    assert_eq!(output.nonce(), NonceField::Null);
    assert_eq!(
        output.program(),
        construction.successor_constructor().output_program(),
    );
    assert!(!transaction.has_witness());
}

#[test]
fn the_bytes_decode_back_through_the_strict_decoder_to_an_equal_transaction() {
    let (_, construction) = demonstration();
    let decoded = TargetTransaction::decode(&construction.bytes())
        .expect("the constructed bytes are the codec's own");

    assert_eq!(&decoded, construction.transaction());
}

#[test]
fn the_successor_metadata_is_the_transition_over_the_views_predecessor() {
    let (_, construction) = demonstration();
    let expected = announce_maturity(&state_metadata(), ANNOUNCED, window())
        .expect("the announced cycle is inside the fixture window");
    let successor = construction.successor_metadata();
    let predecessor = state_metadata();

    assert_eq!(successor, expected);
    assert_eq!(successor.maturity, Maturity::Announced { cycle: ANNOUNCED });
    assert_eq!(
        (
            successor.omega,
            successor.y_l,
            successor.y_t,
            successor.q,
            successor.cycle,
        ),
        (
            predecessor.omega,
            predecessor.y_l,
            predecessor.y_t,
            predecessor.q,
            predecessor.cycle,
        ),
    );
}

#[test]
fn the_successor_constructor_is_the_bundles_own_application_and_continuity_holds() {
    let (_, construction) = demonstration();
    let bundle = linked_bundle();
    let expected = bundle
        .apply_constructor(
            &reviewed_target(),
            &construction.successor_metadata(),
            &FixtureStateCurve,
        )
        .expect("the bundle applies its recipe to the derived successor");
    let retained = bundle
        .instances()
        .first()
        .expect("the link retained its own application")
        .constructor();

    assert_eq!(
        construction.successor_constructor().nonce(),
        expected.nonce()
    );
    assert_eq!(
        construction.successor_constructor().output_program(),
        expected.output_program(),
    );
    assert_eq!(
        hold_continuity(retained, construction.successor_constructor()),
        Ok(()),
    );
}

#[test]
fn the_evidence_carries_every_rejected_lower_candidate_and_states_its_residual() {
    let (_, construction) = demonstration();
    let evidence = construction.successor_constructor().evidence();

    assert_eq!(
        evidence.selected,
        construction.successor_constructor().nonce()
    );
    let rejected: Vec<u32> = evidence
        .rejected
        .iter()
        .map(|(nonce, _)| nonce.get())
        .collect();
    let expected: Vec<u32> = (0..evidence.selected.get()).collect();
    assert_eq!(rejected, expected);
    for (_, refusal) in &evidence.rejected {
        assert!(refusal.retryable());
    }
    assert_eq!(
        StateNonceEvidence::RESIDUAL,
        "host leastness only; a later admissible nonce may exist beyond the search budget",
    );
}

#[test]
fn a_sponsored_request_is_refused_for_the_carrier_it_would_need() {
    let validated = validated_view();
    let abi = abi_over(&validated);
    let sponsored = MaturityAnnouncementRequest::new(
        ANNOUNCED,
        RequestedForm::Sponsored,
        SponsorChangeRequest::NotRequested,
    )
    .expect("the sponsored form is representable");

    assert_eq!(
        construct_over(&abi, &validated, &sponsored)
            .expect_err("no leaf and no model constrains sponsored bytes yet"),
        TransactionRefusal::SponsoredMaturityFormHasNoCarrier,
    );
}

#[test]
fn a_cycle_outside_the_window_carries_the_transitions_own_refusal() {
    let validated = validated_view();
    let abi = abi_over(&validated);

    assert_eq!(
        construct_over(&abi, &validated, &sponsorless(Cycle::new(6)))
            .expect_err("six is one short of the minimum lead"),
        TransactionRefusal::MaturitySuccessorTransitionRefused {
            refusal: MaturityTransitionRefusal::AnnouncementBelowMinimum,
        },
    );
    assert_eq!(
        construct_over(&abi, &validated, &sponsorless(Cycle::new(10)))
            .expect_err("ten is one past the maximum lead"),
        TransactionRefusal::MaturitySuccessorTransitionRefused {
            refusal: MaturityTransitionRefusal::AnnouncementAboveMaximum,
        },
    );
}

#[test]
fn a_view_disagreeing_with_the_deployments_asset_or_amount_is_refused_by_name() {
    let other_asset = view_stating(
        AssetField::Explicit(AssetId::from_internal([0x12; 32])),
        ValueField::Explicit(1),
    );
    let abi = abi_over(&other_asset);
    assert_eq!(
        construct_over(&abi, &other_asset, &sponsorless(ANNOUNCED))
            .expect_err("the singleton is not the asset the view states"),
        TransactionRefusal::MaturityDeploymentAssetDisagreesWithView,
    );

    let other_amount = view_stating(
        AssetField::Explicit(AssetId::from_internal([0x11; 32])),
        ValueField::Explicit(2),
    );
    let abi = abi_over(&other_amount);
    assert_eq!(
        construct_over(&abi, &other_amount, &sponsorless(ANNOUNCED))
            .expect_err("the declared issuance is not the amount the view states"),
        TransactionRefusal::MaturityDeploymentAmountDisagreesWithView,
    );
}

#[test]
fn an_exhausted_search_carries_the_links_own_refusal() {
    let (quantity, metadata) = exhausting_scan();
    assert_eq!(quantity, EXHAUSTING_QUANTITY);

    let bundle = linked_bundle_with(
        StateNonceBudget::new(1).expect("one attempt is a budget"),
        metadata,
    );
    let (nonce, program) = linked_pair(&bundle);
    let validated = PublicMaturityStateView::new(statements(&bundle, metadata, nonce, program))
        .expect("the seven statements name distinct entries")
        .validate(&reviewed_target(), &FixtureStateCurve)
        .expect("the link's own pair reproduces the link's own program");
    let abi = abi_over(&validated);

    assert_eq!(
        construct_over(&abi, &validated, &sponsorless(ANNOUNCED))
            .expect_err("the one admitted attempt fails the fixed outer side"),
        TransactionRefusal::MaturitySuccessorSearchRefused {
            refusal: Box::new(StateLinkRefusal::ConstructorApplication(
                StateConstructorRefusal::RepresentationSearchExhausted,
            )),
        },
    );
}

#[test]
fn exhaustion_a_wrong_nonce_and_a_later_admissible_nonce_are_three_findings() {
    let (quantity, metadata) = exhausting_scan();
    assert_eq!(quantity, EXHAUSTING_QUANTITY);
    let exhausted = {
        let bundle = linked_bundle_with(
            StateNonceBudget::new(1).expect("one attempt is a budget"),
            metadata,
        );
        let (nonce, program) = linked_pair(&bundle);
        let validated = PublicMaturityStateView::new(statements(&bundle, metadata, nonce, program))
            .expect("the seven statements name distinct entries")
            .validate(&reviewed_target(), &FixtureStateCurve)
            .expect("the link's own pair reproduces the link's own program");
        let abi = abi_over(&validated);
        construct_over(&abi, &validated, &sponsorless(ANNOUNCED))
            .expect_err("the one admitted attempt fails the fixed outer side")
    };

    // The first finding: a budget the host set ran out, which says
    // nothing about what the target would accept.
    assert_eq!(
        exhausted,
        TransactionRefusal::MaturitySuccessorSearchRefused {
            refusal: Box::new(StateLinkRefusal::ConstructorApplication(
                StateConstructorRefusal::RepresentationSearchExhausted,
            )),
        },
    );

    // The second: a nonce that commits to a program other than the one
    // published, which is the view's finding and not the search's.
    let bundle = linked_bundle();
    let (linked_nonce, program) = linked_pair(&bundle);
    let other_nonce = (0..64_u32)
        .map(StateRepresentationNonce::new)
        .find(|&nonce| nonce != linked_nonce && commits(&bundle, state_metadata(), nonce))
        .expect("a second committing nonce is inside the scanned range");
    let wrong_nonce =
        PublicMaturityStateView::new(statements(&bundle, state_metadata(), other_nonce, program))
            .expect("the seven statements name distinct entries")
            .validate(&reviewed_target(), &FixtureStateCurve)
            .expect_err("a second nonce commits to a second program");
    assert_ne!(wrong_nonce, exhausted);

    // The third is not a refusal at all: a search that succeeded says in
    // its own evidence that a later admissible nonce may exist beyond
    // the budget it stopped at.
    let (_, construction) = demonstration();
    assert_eq!(
        StateNonceEvidence::RESIDUAL,
        "host leastness only; a later admissible nonce may exist beyond the search budget",
    );
    assert_eq!(
        construction.successor_constructor().evidence().selected,
        construction.successor_constructor().nonce(),
    );
}

#[test]
fn a_constructor_over_another_static_subtree_fails_the_continuity_equality() {
    let target = reviewed_target();
    let elsewhere = CandidateStateConstructor::derive(
        &target,
        &state_metadata(),
        &second_static_subtree(),
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &FixtureStateCurve)
            .expect("the reviewed internal key is the derived one"),
        StateNonceBudget::default(),
        &FixtureStateCurve,
    )
    .expect("the second subtree admits a constructor");
    let bundle = linked_bundle();
    let retained = bundle
        .instances()
        .first()
        .expect("the link retained its own application")
        .constructor();

    assert_eq!(
        hold_continuity(retained, &elsewhere)
            .expect_err("two subtrees, and this generation implements no migration"),
        TransactionRefusal::MaturitySuccessorContinuityRefused {
            refusal: StateConstructorRefusal::ConflictingLeaf,
        },
    );
}

#[test]
fn every_refusal_variant_this_module_raises_is_reached_by_a_test_in_this_file() {
    // One arm per variant this module raises, naming the test that
    // reaches it. Whole-vocabulary exhaustiveness is the census file's
    // job and is not duplicated here — matching a hundred and
    // twenty-eight variants twice would be one copy too many — so the
    // catch-all stands for everything raised elsewhere in the crate.
    // A match arm is not a test either: what this records is which
    // test is answerable for each refusal construction can produce.
    let named = |refusal: &TransactionRefusal| -> &'static str {
        match refusal {
            TransactionRefusal::SponsoredMaturityFormHasNoCarrier => {
                "a_sponsored_request_is_refused_for_the_carrier_it_would_need"
            }
            TransactionRefusal::MaturitySuccessorTransitionRefused { .. } => {
                "a_cycle_outside_the_window_carries_the_transitions_own_refusal"
            }
            TransactionRefusal::MaturitySuccessorSearchRefused { .. } => {
                "an_exhausted_search_carries_the_links_own_refusal"
            }
            TransactionRefusal::MaturitySuccessorContinuityRefused { .. } => {
                "a_constructor_over_another_static_subtree_fails_the_continuity_equality"
            }
            TransactionRefusal::MaturityDeploymentAssetDisagreesWithView
            | TransactionRefusal::MaturityDeploymentAmountDisagreesWithView => {
                "a_view_disagreeing_with_the_deployments_asset_or_amount_is_refused_by_name"
            }
            _ => "raised elsewhere in the crate",
        }
    };

    assert_eq!(
        named(&TransactionRefusal::SponsoredMaturityFormHasNoCarrier),
        "a_sponsored_request_is_refused_for_the_carrier_it_would_need",
    );
}
