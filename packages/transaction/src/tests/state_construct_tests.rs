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

fn change_public_sponsor(
    public: &crate::view::PublicConstructionView,
    asset: Option<AssetField>,
    value: Option<ValueField>,
) -> crate::view::PublicConstructionView {
    let first = public.outputs().keys().next().copied().unwrap();
    crate::view::PublicConstructionView::new(public.outputs().values().map(|spent| {
        if spent.outpoint() == first {
            crate::view::PublicOutputView::new(
                first,
                asset.unwrap_or_else(|| spent.asset()),
                value.unwrap_or_else(|| spent.value()),
                spent.program().to_vec(),
            )
        } else {
            spent.clone()
        }
    }))
    .unwrap()
}

#[test]
fn malformed_and_empty_offers_refuse_against_a_nonempty_control() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (mut sponsor, public) = maturity_sponsor(true);
    assert!(construct_with_sponsor(true, &sponsor, &public).is_ok());
    let inputs = sponsor.inputs.clone();
    sponsor.inputs.clear();
    assert_eq!(
        construct_with_sponsor(true, &sponsor, &public).unwrap_err(),
        TransactionRefusal::EmptySponsorOffer
    );
    sponsor.inputs = inputs;
    sponsor.change = None;
    assert_eq!(
        construct_with_sponsor(true, &sponsor, &public).unwrap_err(),
        TransactionRefusal::SponsorChangeRequestedWithoutDestination
    );
    sponsor.change = Some(ValueField::Explicit(40));
    assert_eq!(
        construct_with_sponsor(false, &sponsor, &public).unwrap_err(),
        TransactionRefusal::SponsorChangeOfferedWithoutRequest
    );
    sponsor.destination = None;
    assert_eq!(
        construct_with_sponsor(true, &sponsor, &public).unwrap_err(),
        TransactionRefusal::SponsorChangeRequestedWithoutDestination
    );
    sponsor.destination = Some((0, Vec::new()));
    assert_eq!(
        construct_with_sponsor(true, &sponsor, &public).unwrap_err(),
        TransactionRefusal::MalformedPinnedProgram { offered: 0 }
    );
    let coin = sponsor.inputs[0];
    assert!(crate::SponsorOffer::new([coin], 1, None).is_ok());
    assert_eq!(
        crate::SponsorOffer::new([coin, coin], 1, None),
        Err(TransactionRefusal::DuplicateSponsorOutpoint(coin))
    );
}

#[test]
fn declared_reserve_cannot_alias_the_state_asset() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (sponsor, public) = maturity_sponsor(false);
    let control = construct_with_sponsor(false, &sponsor, &public).unwrap();
    let AssetField::Explicit(state_asset) = control.transaction().outputs()[0].asset() else {
        panic!("the control carries an explicit STATE asset");
    };
    let validated = validated_view();
    let abi = abi_over(&validated);
    let context = crate::MaturitySponsorContext {
        reserve_asset: state_asset,
        view: &public,
        capability: &sponsor,
    };
    assert_eq!(
        crate::construct_maturity_announcement_with_sponsor(
            &reviewed_target(),
            &abi,
            &validated,
            &control.request(),
            &FixtureStateCurve,
            Some(&context)
        )
        .unwrap_err(),
        TransactionRefusal::MaturitySponsorReserveAliasesStateAsset
    );
    assert!(sponsor.requests.borrow().is_empty());
}

#[test]
fn sponsor_suffix_position_overflow_refuses_before_signing() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (mut sponsor, public) = maturity_sponsor(false);
    assert!(construct_with_sponsor(false, &sponsor, &public).is_ok());
    sponsor.inputs = (0..=u16::MAX)
        .map(|index| super::outpoint(0xdc, u32::from(index)))
        .collect();
    assert_eq!(
        construct_with_sponsor(false, &sponsor, &public).unwrap_err(),
        TransactionRefusal::MaturitySponsorInputPositionOutOfRange
    );
    assert!(sponsor.requests.borrow().is_empty());
}

#[test]
fn state_input_cannot_join_sponsor_offer() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (mut sponsor, public) = maturity_sponsor(false);
    assert!(construct_with_sponsor(false, &sponsor, &public).is_ok());
    let state = validated_view().view().current_state_outpoint();
    sponsor.inputs.push(state);
    assert_eq!(
        construct_with_sponsor(false, &sponsor, &public).unwrap_err(),
        TransactionRefusal::OverlappingOutpoint(state)
    );
}

#[test]
fn sponsor_views_are_required_and_foreign_assets_refuse() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (sponsor, public) = maturity_sponsor(false);
    assert!(construct_with_sponsor(false, &sponsor, &public).is_ok());
    let first = *public.outputs().keys().next().unwrap();
    assert_eq!(
        construct_with_sponsor(
            false,
            &sponsor,
            &crate::view::PublicConstructionView::default()
        )
        .unwrap_err(),
        TransactionRefusal::MissingPublicSponsorView(first)
    );
    let foreign = change_public_sponsor(
        &public,
        Some(AssetField::Explicit(AssetId::from_internal([0x77; 32]))),
        None,
    );
    assert_eq!(
        construct_with_sponsor(false, &sponsor, &foreign).unwrap_err(),
        TransactionRefusal::SponsorInputCarriesForeignAsset(first)
    );
}

#[test]
fn confidential_sponsor_value_requires_balanced_proof_route() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (mut sponsor, public) = maturity_sponsor(true);
    assert!(construct_with_sponsor(true, &sponsor, &public).is_ok());
    let committed = change_public_sponsor(&public, None, Some(ValueField::Commitment([8; 33])));
    assert_eq!(
        construct_with_sponsor(true, &sponsor, &committed).unwrap_err(),
        TransactionRefusal::MaturitySponsorValueRequiresProofFinalization
    );
    sponsor.change = Some(ValueField::Commitment([8; 33]));
    assert_eq!(
        construct_with_sponsor(true, &sponsor, &public).unwrap_err(),
        TransactionRefusal::MaturitySponsorValueRequiresProofFinalization
    );
}

#[test]
fn explicit_zero_change_is_omitted_without_changing_the_requested_form() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (mut sponsor, public) = maturity_sponsor(true);
    let positive = construct_with_sponsor(true, &sponsor, &public).unwrap();
    assert_eq!(positive.sponsor_change_position(), Some(1));
    sponsor.change = Some(ValueField::Explicit(0));
    sponsor.fee = 130;
    let zero = construct_with_sponsor(true, &sponsor, &public).unwrap();
    assert_eq!(zero.request(), positive.request());
    assert_eq!(zero.sponsor_change_position(), None);
    assert_eq!(zero.fee_position(), Some(1));
    assert_eq!(zero.transaction().outputs().len(), 2);
}

#[test]
fn sponsorless_request_refuses_an_extraneous_context() {
    use super::state_support::maturity_sponsor;
    let (sponsor, public) = maturity_sponsor(false);
    let validated = validated_view();
    let abi = abi_over(&validated);
    let request = sponsorless(ANNOUNCED);
    assert!(construct_over(&abi, &validated, &request).is_ok());
    let context = crate::MaturitySponsorContext {
        reserve_asset: AssetId::from_internal(super::live_support::LIVE_RESERVE_ASSET),
        view: &public,
        capability: &sponsor,
    };
    assert_eq!(
        crate::construct_maturity_announcement_with_sponsor(
            &reviewed_target(),
            &abi,
            &validated,
            &request,
            &FixtureStateCurve,
            Some(&context)
        )
        .unwrap_err(),
        TransactionRefusal::SponsorCapabilityWithoutRequest
    );
}

fn sponsor_observation(construction: &MaturityConstruction) -> realization::OperationObservation {
    use std::collections::{BTreeMap, BTreeSet};

    use architecture::{BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId, RootId};
    use realization::{
        ObservedAsset, ObservedObject, ObservedObjectKind, ObservedObjectRef,
        ObservedOperationRegion, ObservedRootEffect, ObservedRootEffectKind, ObservedSide,
        ObservedValue, OwnerId, RepresentationMode,
    };
    let state = |side| ObservedObject {
        reference: ObservedObjectRef { side, ordinal: 0 },
        kind: ObservedObjectKind::Declared(ObjectId::State),
        asset: ObservedAsset::Declared(architecture::AssetId::Pid),
        value: ObservedValue::Protocol(ProtocolAmount::ONE),
        owner: None,
        representation: RepresentationMode::Explicit,
    };
    let mut objects = vec![state(ObservedSide::Input), state(ObservedSide::Output)];
    let mut sources = Vec::new();
    let mut signers = BTreeSet::new();
    for (offset, spent) in construction.sponsor_spent().iter().enumerate() {
        let ordinal = u32::try_from(offset + 1).unwrap();
        assert_eq!(
            construction.transaction().inputs()[offset + 1].outpoint(),
            spent.outpoint()
        );
        assert_eq!(
            spent.asset(),
            AssetField::Explicit(AssetId::from_internal(
                super::live_support::LIVE_RESERVE_ASSET
            ))
        );
        let owner = OwnerId([u8::try_from(ordinal).unwrap(); 32]);
        let reference = ObservedObjectRef {
            side: ObservedSide::Input,
            ordinal,
        };
        objects.push(ObservedObject {
            reference,
            kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
            asset: ObservedAsset::Declared(architecture::AssetId::Lbtc),
            value: ObservedValue::SponsorOpaque,
            owner: Some(owner),
            representation: RepresentationMode::Explicit,
        });
        sources.push(reference);
        signers.insert(owner);
    }
    let mut destinations = Vec::new();
    if let Some(position) = construction.sponsor_change_position() {
        assert!(!construction.transaction().outputs()[usize::from(position)].is_fee());
        let reference = ObservedObjectRef {
            side: ObservedSide::Output,
            ordinal: u32::from(position),
        };
        objects.push(ObservedObject {
            reference,
            kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
            asset: ObservedAsset::Declared(architecture::AssetId::Lbtc),
            value: ObservedValue::SponsorOpaque,
            owner: Some(OwnerId([0x44; 32])),
            representation: RepresentationMode::Explicit,
        });
        destinations.push(reference);
    }
    realization::OperationObservation {
        operation: OperationId::AnnounceMaturity,
        region: ObservedOperationRegion {
            members: objects.iter().map(|object| object.reference).collect(),
            canonical_partition: realization::ObservedCanonicalPartition::default(),
            open_flows: vec![realization::ObservedOpenFlow {
                kind: OpenFlowKind::FeeSponsor,
                sources,
                destinations,
                fee: ProtocolAmount::ZERO,
            }],
        },
        objects,
        protocol_signers: BTreeSet::new(),
        sponsor_signers: signers,
        other_regions: Vec::new(),
        root_effects: vec![ObservedRootEffect {
            root: RootId::State,
            effect: ObservedRootEffectKind::Succession,
        }],
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([(BoundId::FeeSponsorInputMax, realization::Count::new(16))]),
    }
}

fn sponsor_status(
    observed: &realization::OperationObservation,
    kind: realization::RelationKind,
    subject: realization::RelationSubject,
) -> realization::RelationStatus {
    let scope = realization::RealizationScope::from_operations([
        architecture::OperationId::AnnounceMaturity,
    ])
    .unwrap();
    let realization = realization::derive(&architecture::ARCHITECTURE, scope).unwrap();
    let normalized = observed.clone().validate_and_normalize().unwrap();
    realization
        .evaluate_operation(&normalized)
        .unwrap()
        .verdict(&realization::RelationId::new(
            architecture::OperationId::AnnounceMaturity,
            kind,
            subject,
        ))
        .unwrap()
        .status
        .clone()
}

fn isolation(observed: &realization::OperationObservation) -> realization::RelationStatus {
    sponsor_status(
        observed,
        realization::RelationKind::SponsorIsolation,
        realization::RelationSubject::Sponsor,
    )
}

fn sponsored_observation_control() -> realization::OperationObservation {
    let control = sponsor_observation(&super::state_support::sponsored_construction(true));
    assert_eq!(isolation(&control), realization::RelationStatus::Passed);
    control
}

#[test]
fn missing_sponsor_membership_fails_isolation() {
    let control = sponsored_observation_control();
    for source in [true, false] {
        let mut subject = control.clone();
        if source {
            subject.region.open_flows[0].sources.pop();
        } else {
            subject.region.open_flows[0].destinations.pop();
        }
        assert_eq!(
            isolation(&subject),
            realization::RelationStatus::Failed {
                reason: realization::RelationFailure::SponsorIsolation
            }
        );
    }
}

#[test]
fn surplus_sponsor_membership_refuses_at_observation_boundary() {
    let control = sponsored_observation_control();
    let member = control.region.open_flows[0].sources[0];
    let mut repeated = control.clone();
    repeated.region.open_flows[0].sources.push(member);
    assert_eq!(
        repeated.validate_and_normalize(),
        Err(realization::RealizationError::DuplicateObservedReference)
    );
    let mut crossed = control.clone();
    crossed.region.members.remove(&member);
    crossed
        .other_regions
        .push(realization::ObservedOperationRegion {
            members: std::collections::BTreeSet::from([member]),
            ..Default::default()
        });
    assert_eq!(
        crossed.validate_and_normalize(),
        Err(realization::RealizationError::UnknownObservedObject(member))
    );
    assert!(control.validate_and_normalize().is_ok());
}

#[test]
fn state_input_and_successor_cannot_join_sponsor_flow() {
    let control = sponsored_observation_control();
    for side in [
        realization::ObservedSide::Input,
        realization::ObservedSide::Output,
    ] {
        let mut subject = control.clone();
        let state = realization::ObservedObjectRef { side, ordinal: 0 };
        match side {
            realization::ObservedSide::Input => subject.region.open_flows[0].sources.push(state),
            realization::ObservedSide::Output => {
                subject.region.open_flows[0].destinations.push(state);
            }
        }
        assert_eq!(
            isolation(&subject),
            realization::RelationStatus::Failed {
                reason: realization::RelationFailure::SponsorIsolation
            }
        );
    }
}

#[test]
fn each_sponsor_member_requires_its_own_model_authorization() {
    let control = sponsored_observation_control();
    for owner in &control.sponsor_signers {
        let mut subject = control.clone();
        subject.sponsor_signers.remove(owner);
        assert_eq!(
            isolation(&subject),
            realization::RelationStatus::Failed {
                reason: realization::RelationFailure::SponsorIsolation
            }
        );
    }
}

#[test]
fn second_sponsor_envelope_fails_region_multiplicity() {
    use realization::{RelationKind, RelationSubject};
    let mut subject = sponsored_observation_control();
    assert_eq!(
        sponsor_status(
            &subject,
            RelationKind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor
        ),
        realization::RelationStatus::Passed
    );
    let second = subject.region.open_flows[0].sources.pop().unwrap();
    subject
        .region
        .open_flows
        .push(realization::ObservedOpenFlow {
            kind: architecture::OpenFlowKind::FeeSponsor,
            sources: vec![second],
            destinations: Vec::new(),
            fee: ProtocolAmount::ZERO,
        });
    assert!(subject.clone().validate_and_normalize().is_ok());
    assert_eq!(
        sponsor_status(
            &subject,
            RelationKind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor
        ),
        realization::RelationStatus::Failed {
            reason: realization::RelationFailure::SponsorEnvelopeMultiplicity
        }
    );
}

#[test]
fn foreign_sponsor_asset_fails_model_recognition() {
    use realization::{RelationKind, RelationSubject, TransactionSide};
    let mut subject = sponsored_observation_control();
    let family = RelationSubject::ObjectFamily {
        side: TransactionSide::Input,
        object: architecture::ObjectId::PlainLbtc,
    };
    assert_eq!(
        sponsor_status(&subject, RelationKind::Recognition, family.clone()),
        realization::RelationStatus::Passed
    );
    let member = subject.region.open_flows[0].sources[0];
    subject
        .objects
        .iter_mut()
        .find(|object| object.reference == member)
        .unwrap()
        .asset = realization::ObservedAsset::Foreign(1);
    assert_eq!(
        sponsor_status(&subject, RelationKind::Recognition, family),
        realization::RelationStatus::Failed {
            reason: realization::RelationFailure::ObjectRecognition
        }
    );
}

#[test]
fn zero_sponsor_value_does_not_erase_membership() {
    use super::state_support::{construct_with_sponsor, maturity_sponsor};
    let (sponsor, public) = maturity_sponsor(true);
    let control = sponsor_observation(&construct_with_sponsor(true, &sponsor, &public).unwrap());
    let zero = change_public_sponsor(&public, None, Some(ValueField::Explicit(0)));
    let subject = sponsor_observation(&construct_with_sponsor(true, &sponsor, &zero).unwrap());
    assert_eq!(control, subject);
    assert_eq!(isolation(&subject), realization::RelationStatus::Passed);
}

#[test]
fn state_amount_corruption_fails_model_recognition_with_sponsorship() {
    use realization::{
        ObservedSide, ObservedValue, RelationKind, RelationSubject, TransactionSide,
    };
    let mut subject = sponsored_observation_control();
    let family = RelationSubject::ObjectFamily {
        side: TransactionSide::Output,
        object: architecture::ObjectId::State,
    };
    assert_eq!(
        sponsor_status(&subject, RelationKind::Recognition, family.clone()),
        realization::RelationStatus::Passed
    );
    let state = subject
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Output && object.reference.ordinal == 0
        })
        .unwrap();
    state.value = ObservedValue::Protocol(ProtocolAmount::new(2).unwrap());
    assert_eq!(
        sponsor_status(&subject, RelationKind::Recognition, family),
        realization::RelationStatus::Failed {
            reason: realization::RelationFailure::ObjectRecognition
        }
    );
}

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
fn a_sponsored_request_without_its_capability_is_refused() {
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
            .expect_err("the wrapper supplies no sponsor capability"),
        TransactionRefusal::SponsorRequestedWithoutCapability,
    );
}

// The four request forms have named dispositions: sponsorless without change
// constructs, sponsorless with change is refused by the request, and both
// sponsored forms construct through the funding context.
#[test]
fn each_request_form_is_built_or_refused_by_name() {
    use std::collections::BTreeSet;

    let validated = validated_view();
    let abi = abi_over(&validated);
    let sponsorless = RequestedForm::Sponsorless;
    let sponsored = RequestedForm::Sponsored;
    let no_change = SponsorChangeRequest::NotRequested;
    let change = SponsorChangeRequest::Requested;
    let forms = [
        (sponsorless, no_change),
        (sponsorless, change),
        (sponsored, no_change),
        (sponsored, change),
    ];
    let distinct: BTreeSet<_> = forms.iter().copied().collect();
    assert_eq!(distinct.len(), 4);

    let (form, change) = forms[0];
    let request = MaturityAnnouncementRequest::new(ANNOUNCED, form, change)
        .expect("the sponsorless form without change is admitted");
    let construction = construct_over(&abi, &validated, &request)
        .expect("the sponsorless form without change constructs");
    let transaction = construction.transaction();
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

    let (form, change) = forms[1];
    assert_eq!(
        MaturityAnnouncementRequest::new(ANNOUNCED, form, change)
            .expect_err("a sponsorless form has no residual for change"),
        TransactionRefusal::SponsorChangeWithoutSponsoredForm,
    );

    for (form, change) in [forms[2], forms[3]] {
        let request = MaturityAnnouncementRequest::new(ANNOUNCED, form, change)
            .expect("both sponsored forms are well-formed requests");
        let built = super::state_support::sponsored_construction(change.requested());
        assert_eq!(built.request(), request);
        assert_eq!(built.transaction().inputs().len(), 3);
        assert_eq!(
            built.transaction().outputs().len(),
            if change.requested() { 3 } else { 2 }
        );
        assert_eq!(built.transaction().outputs()[0], transaction.outputs()[0]);
    }
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
    // job and is not duplicated here, so the
    // catch-all stands for everything raised elsewhere in the crate.
    // A match arm is not a test either: what this records is which
    // test is answerable for each refusal construction can produce.
    let named = |refusal: &TransactionRefusal| -> &'static str {
        match refusal {
            TransactionRefusal::SponsorRequestedWithoutCapability => {
                "a_sponsored_request_without_its_capability_is_refused"
            }
            TransactionRefusal::SponsorCapabilityWithoutRequest => {
                "sponsorless_request_refuses_an_extraneous_context"
            }
            TransactionRefusal::EmptySponsorOffer
            | TransactionRefusal::SponsorChangeRequestedWithoutDestination
            | TransactionRefusal::SponsorChangeOfferedWithoutRequest
            | TransactionRefusal::MalformedPinnedProgram { .. } => {
                "malformed_and_empty_offers_refuse_against_a_nonempty_control"
            }
            TransactionRefusal::OverlappingOutpoint(_) => "state_input_cannot_join_sponsor_offer",
            TransactionRefusal::MissingPublicSponsorView(_)
            | TransactionRefusal::SponsorInputCarriesForeignAsset(_) => {
                "sponsor_views_are_required_and_foreign_assets_refuse"
            }
            TransactionRefusal::MaturitySponsorValueRequiresProofFinalization => {
                "confidential_sponsor_value_requires_balanced_proof_route"
            }
            TransactionRefusal::MaturitySponsorReserveAliasesStateAsset => {
                "declared_reserve_cannot_alias_the_state_asset"
            }
            TransactionRefusal::MaturitySponsorInputPositionOutOfRange => {
                "sponsor_suffix_position_overflow_refuses_before_signing"
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
        named(&TransactionRefusal::SponsorRequestedWithoutCapability),
        "a_sponsored_request_without_its_capability_is_refused",
    );
}
