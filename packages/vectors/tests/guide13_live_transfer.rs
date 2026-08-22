//! The live-transfer candidate, built with the real arithmetic
//! (Guide-13 §12).
//!
//! # What this file is for
//!
//! The transaction package derives §12's candidate ABI and records two
//! of the link's obligations as discharged: the destination constructor
//! table selects among the linked placements, and the taproot output key
//! is the tweak of each constructor's committed root. Its own tests
//! establish the first with certainty and the second only in shape,
//! because that crate deliberately owns no curve arithmetic and its
//! fixture capability performs none.
//!
//! This is where the second claim is made true. The capability wired in
//! here is the conformance package's own point arithmetic, the internal
//! key is the published unspendable one, and the owners are the
//! published BIP-340 test vectors' keys. So the destination programs
//! this transfer pays to are the programs the deployment's constructors
//! actually determine, computed by the package that owns the curve.
//!
//! # What it still does not establish
//!
//! That a target accepts a spend of them. §1.11 admits no verdict
//! without a run, the selected sighash profile is unreviewed, and the
//! message the owners sign below is *not* the target's sighash — see
//! [`test_binding`]. Every obligation about those stays where it was.
//!
//! # Every secret here is published
//!
//! The signing scalars are the BIP-340 specification's own appendix
//! values, admitted under ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)` and by Guide-13 §1.10.
//! They authorize nothing on any network anyone uses.

use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::live_transfer_plan::{
    ValidatedLiveTransferOperationPlan, plan_live_transfer_target_operation,
};
use compiler::operation_plan::PlacementSearchLimits;
use linker::live_backend::LiveTransferRepresentationPlan;
use linker::{
    CandidateLinkedLiveTransferBundle, LiveLinkDeploymentParameters, LiveLinkObligation,
    OwnerParameter, link_live_candidate,
};
use realization::{RealizationScope, derive};
use tapscript::{
    LiveTransferSymbols, OwnerKey, demonstration_live_shape_set, derive_live_receipt_constructor,
    emit_candidate_live_bundle, owner_key_encoding_closure, static_transfer_leaf_set,
};
use target_elements::{ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript};
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use target_elements_conformance::constructor::tagged::{Digest32, tagged_hash};
use target_elements_conformance::live_receipt_key::live_receipt_output_key;
use target_elements_conformance::owner_key_oracle::verify_owner_signature;
use target_elements_conformance::test_material::OwnerSigningMaterial;
use transaction::bytes::{AssetField, AssetId, Outpoint, Txid, ValueField};
use transaction::live_abi::{CandidateLiveTransferAbi, derive_live_transfer_abi};
use transaction::live_construct::{complete_live_transfer, finalize_live_transfer};
use transaction::live_private::PrivateConstructionNonClaim;
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::view::{PublicConstructionView, PublicOutputView};
use vectors::{OracleFixtureValues, OracleLiveCurve};

/// The first published BIP-340 signing scalar.
const FIRST_SCALAR: [u8; FIELD_ELEMENT_BYTES] = [
    0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7,
    0x62, 0xE7, 0x16, 0x0F, 0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
];

/// The second published BIP-340 signing scalar.
const SECOND_SCALAR: [u8; FIELD_ELEMENT_BYTES] = [
    0xC9, 0x0F, 0xDA, 0xA2, 0x21, 0x68, 0xC2, 0x34, 0xC4, 0xC6, 0x62, 0x8B, 0x80, 0xDC, 0x1C, 0xD1,
    0x29, 0x02, 0x4E, 0x08, 0x8A, 0x67, 0xCC, 0x74, 0x02, 0x0B, 0xBE, 0xA6, 0x3B, 0x14, 0xE5, 0xC9,
];

/// The protocol asset this demonstration deployment resolves.
const PROTOCOL_ASSET: [u8; 32] = [0xb1; 32];

/// The published randomness the private construction consumes.
const PUBLISHED_RANDOMNESS: [u8; 32] = [0x7e; 32];

/// The message an owner is asked to sign over the finalized bytes.
///
/// **Not the target's sighash, and not a candidate for one.** §1.7
/// leaves the digest to the target's own construction over a transaction
/// only a target-native run has, and requires the selected profile to be
/// observed from a finalized witness or recomputed from the exact
/// request rather than asserted by a builder. A 32-byte value minted
/// here would be that assertion.
///
/// What it is instead is a *binding* over the exact protected
/// serialization, under a tag belonging to this workspace and to no
/// target. It establishes what this file claims and nothing more: that
/// an owner can bind to the finalized bytes, that the binding is
/// checkable by the package that owns the curve, and that a signature
/// over other bytes does not check.
fn test_binding(protected: &[u8]) -> Digest32 {
    tagged_hash("Guide13/live-transfer-test-binding", protected)
}

/// The reviewed contract, unmodified.
fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// One owner's canonical metadata from a published public key.
fn owner_key(bytes: &[u8]) -> OwnerKey {
    let target = reviewed_target();
    let closure = owner_key_encoding_closure(target.definition().authorization());
    OwnerKey::new(&closure, closure.approved(), bytes.to_vec())
        .expect("a published x-only key is the approved encoding at its exact width")
}

/// One published scalar's owner parameter.
fn owner(scalar: &[u8; FIELD_ELEMENT_BYTES]) -> OwnerParameter {
    OwnerParameter::new(owner_key(&signing_material(scalar).x_only_public_key()))
}

/// One published scalar's signing material.
fn signing_material(scalar: &[u8; FIELD_ELEMENT_BYTES]) -> OwnerSigningMaterial {
    OwnerSigningMaterial::from_published_scalar(scalar)
        .expect("the published vectors' scalars are in range")
}

/// The validated live-transfer plan.
fn live_transfer_plan() -> ValidatedLiveTransferOperationPlan {
    let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
    let realization =
        derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
    let scope = CompilationScope::from_operations([OperationId::TransferLive])
        .expect("a one-operation scope");
    let policy = AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
    let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

    plan_live_transfer_target_operation(
        &input,
        PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
    )
    .expect("the plan validates")
}

/// The live symbols one set of values resolves.
fn live_symbols(
    target: &ReviewedElementsTapscriptDefinition,
    protocol_asset: Vec<u8>,
    reserve_asset: Vec<u8>,
    sponsor_change: Vec<u8>,
    fee_digest: Vec<u8>,
) -> LiveTransferSymbols {
    LiveTransferSymbols::new(
        target,
        protocol_asset,
        reserve_asset,
        1,
        sponsor_change,
        0,
        fee_digest,
    )
    .expect("the symbols are the reviewed widths")
}

/// The demonstration link, over two published owners and both plans.
fn linked_bundle() -> CandidateLinkedLiveTransferBundle {
    let target = reviewed_target();
    let plan = live_transfer_plan();
    let shapes = demonstration_live_shape_set();
    let placeholders = live_symbols(
        &target,
        vec![0x5a; 32],
        vec![0x22; 32],
        vec![0x44; 20],
        vec![0x55; 32],
    );

    let mut bundles = Vec::with_capacity(4);
    for scalar in [FIRST_SCALAR, SECOND_SCALAR] {
        for representation in [
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ] {
            let constructor = derive_live_receipt_constructor(
                &target,
                &plan,
                representation,
                owner_key(&signing_material(&scalar).x_only_public_key()),
                shapes.clone(),
                static_transfer_leaf_set(representation, &shapes),
            )
            .expect("the constructor derives for a published owner");
            bundles.push(
                emit_candidate_live_bundle(&target, &plan, &constructor, placeholders.clone())
                    .expect("the bundle emits"),
            );
        }
    }

    let deployment = LiveLinkDeploymentParameters::new(
        &target,
        live_symbols(
            &target,
            PROTOCOL_ASSET.to_vec(),
            vec![0xb2; 32],
            vec![0xb4; 32],
            vec![0xb5; 32],
        ),
        // The published unspendable internal key, which is a point of
        // the target's curve. That matters here and did not in the
        // transaction crate's own tests: the tweak is real arithmetic,
        // and a key that lifted to nothing would have no output key at
        // all.
        UNSPENDABLE_INTERNAL_KEY.to_vec(),
        NonZeroU32::new(8).expect("eight is nonzero"),
    )
    .expect("the deployment parameters are the reviewed widths");

    link_live_candidate(&target, &bundles, &deployment).expect("the demonstration live link")
}

/// The candidate ABI, derived through the oracle's own arithmetic.
fn oracle_abi() -> (CandidateLiveTransferAbi, OracleLiveCurve) {
    let curve = OracleLiveCurve::new(reviewed_target());
    let abi = derive_live_transfer_abi(&reviewed_target(), &linked_bundle(), &curve)
        .expect("the ABI derives against real arithmetic");
    (abi, curve)
}

/// The outpoint of `index` of a transaction whose identifier is `byte`
/// repeated.
fn outpoint(byte: u8, index: u32) -> Outpoint {
    Outpoint::new(Txid::from_internal([byte; 32]), index).expect("the fixture index is in range")
}

/// One destination for a published owner at `amount`.
fn destination(scalar: &[u8; FIELD_ELEMENT_BYTES], amount: u64) -> LiveReceiptDestination {
    LiveReceiptDestination::new(
        owner(scalar),
        ProtocolValue::new(amount).expect("the fixture amounts are positive"),
    )
}

/// A public view of one owner's live receipt.
fn receipt_view(
    abi: &CandidateLiveTransferAbi,
    point: Outpoint,
    scalar: &[u8; FIELD_ELEMENT_BYTES],
    representation: LiveTransferRepresentationPlan,
    value: ValueField,
) -> PublicOutputView {
    PublicOutputView::new(
        point,
        AssetField::Explicit(AssetId::from_internal(PROTOCOL_ASSET)),
        value,
        abi.destinations()
            .get(&owner(scalar), representation)
            .expect("a published owner has a linked constructor")
            .instance()
            .program()
            .to_vec(),
    )
}

// --- The output key the ABI derived is the one the oracle computes ----

#[test]
fn every_destination_program_is_the_tweak_of_its_own_committed_root() {
    // `TaprootOutputKeyUndischarged`, closed with the arithmetic rather
    // than in shape. The builder hashed each constructor's committed
    // tree and asked the capability for the key; this recomputes the key
    // from that same root through the oracle's entry point and compares
    // the program byte for byte.
    let (abi, _curve) = oracle_abi();
    assert_eq!(abi.destinations().entries().len(), 4);

    for constructor in abi.destinations().entries().values() {
        let instance = constructor.instance();
        let internal = <[u8; 32]>::try_from(instance.internal_key())
            .expect("the linked internal key is the reviewed width");
        let expected = live_receipt_output_key(&internal, &instance.tree().merkle_root())
            .expect("the committed root determines an output key");

        assert_eq!(instance.output_key().key(), expected.key());
        assert_eq!(instance.output_key().parity().bit(), expected.parity());
        assert_eq!(instance.program(), expected.program());
    }

    // And the ABI records the obligation as discharged rather than
    // carried, which is the claim the comparison above is what makes
    // true.
    assert!(
        abi.inherited_link_obligations()
            .discharged()
            .contains(&LiveLinkObligation::TaprootOutputKeyUndischarged)
    );
    assert_eq!(abi.inherited_link_obligations().carried(), &BTreeSet::new());
}

#[test]
fn four_constructors_determine_four_different_programs() {
    let (abi, _curve) = oracle_abi();
    let programs: BTreeSet<_> = abi
        .destinations()
        .entries()
        .values()
        .map(|constructor| constructor.instance().program().to_vec())
        .collect();
    assert_eq!(programs.len(), 4);
}

// --- A multi-owner transfer, signed by published owners ---------------

#[test]
fn two_published_owners_authorize_one_finalized_transfer() {
    let target = reviewed_target();
    let (abi, _curve) = oracle_abi();
    let first = outpoint(0xa1, 0);
    let second = outpoint(0xa2, 1);
    let view = PublicConstructionView::new([
        receipt_view(
            &abi,
            first,
            &FIRST_SCALAR,
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(400),
        ),
        receipt_view(
            &abi,
            second,
            &SECOND_SCALAR,
            LiveTransferRepresentationPlan::Explicit,
            ValueField::Explicit(600),
        ),
    ])
    .expect("the fixture views name distinct outpoints");

    let request = LiveTransferRequest::new(
        [first, second],
        [
            destination(&SECOND_SCALAR, 250),
            destination(&FIRST_SCALAR, 750),
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .expect("the request validates");

    let finalization = finalize_live_transfer(&target, &abi, &request, &view, None, None)
        .expect("the transfer finalizes");
    let report = finalization.report().clone();
    let finalized = finalization.into_finalized();

    // Every owner signs the same finalized protected transaction (§12.7),
    // and the message is the binding this file defines rather than a
    // sighash anybody minted.
    let auxiliary = [0_u8; FIELD_ELEMENT_BYTES];
    let message = test_binding(finalized.protected_bytes());
    let responses: Vec<_> = finalized
        .signing_requests()
        .iter()
        .map(|signing| {
            let scalar = if signing.owner() == &owner(&FIRST_SCALAR) {
                FIRST_SCALAR
            } else {
                SECOND_SCALAR
            };
            let signature = signing_material(&scalar)
                .sign(&message, &auxiliary)
                .expect("a published owner signs");
            (
                signing.input(),
                LiveOwnerResponse::to(signing, signature.to_vec()),
            )
        })
        .collect();

    let authorized =
        authorize_live_transfer(finalized, responses).expect("both published owners authorize");
    assert_eq!(authorized.levels().distinct_semantic_owners().len(), 2);
    assert_eq!(authorized.levels().owner_signatures(), 2);

    // Each collected signature verifies against the owner the input
    // authenticates, through the package that owns the curve. The
    // builder checked binding; this checks the arithmetic agrees.
    for (position, witness) in authorized.witnesses() {
        let record = authorized
            .finalized()
            .receipts()
            .iter()
            .find(|record| record.position() == *position)
            .expect("every witness answers a receipt");
        assert_eq!(
            verify_owner_signature(
                &target,
                record.owner().key().bytes(),
                &message,
                &witness.stack()[0],
            ),
            Ok(()),
        );
    }

    let built =
        complete_live_transfer(&target, authorized, report, None).expect("the candidate completes");
    assert_eq!(built.transaction().inputs().len(), 2);
    assert_eq!(built.transaction().outputs().len(), 2);
    assert_eq!(
        transaction::bytes::TargetTransaction::decode(&built.bytes()).as_ref(),
        Ok(built.transaction()),
    );
}

// --- The private construction, with real commitments ------------------

#[test]
fn the_private_construction_builds_target_value_commitments() {
    // §12.8's central public-fixture model, performed: the openings are
    // hashes of randomness the caller published, the commitments are the
    // oracle's own Pedersen points, and the asset stays explicit as §6.3
    // requires. It demonstrates target feasibility, and the report says
    // in the same breath what it does not demonstrate.
    let target = reviewed_target();
    let (abi, _curve) = oracle_abi();
    let point = outpoint(0xc1, 0);
    let view = PublicConstructionView::new([receipt_view(
        &abi,
        point,
        &FIRST_SCALAR,
        LiveTransferRepresentationPlan::PrivateCommitted,
        ValueField::Commitment([0x09; 33]),
    )])
    .expect("the fixture view names one outpoint");

    let request = LiveTransferRequest::new(
        [point],
        [
            destination(&FIRST_SCALAR, 600),
            destination(&SECOND_SCALAR, 400),
        ],
        LiveTransferRepresentationPlan::PrivateCommitted,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        Some(PublicTestRandomness::from_published_bytes(
            PUBLISHED_RANDOMNESS,
        )),
    )
    .expect("the request validates");

    let built = finalize_live_transfer(
        &target,
        &abi,
        &request,
        &view,
        None,
        Some(&OracleFixtureValues),
    )
    .expect("the private form finalizes with real commitments");

    let outputs = built.finalized().outputs().outputs();
    assert_eq!(outputs.len(), 2);
    let mut commitments = BTreeSet::new();
    for output in outputs {
        let ValueField::Commitment(commitment) = output.value() else {
            panic!("a private destination carries a value commitment");
        };
        // One of the two confidential value prefixes, which is what
        // makes the field the target's confidential form rather than an
        // opaque blob (§12.8's field-form non-claim covers whether the
        // target reads it back).
        assert!(transaction::bytes::VALUE_COMMITMENT_PREFIXES.contains(&commitment[0]));
        assert!(
            target_elements_conformance::commitment_oracle::commitment::parse_commitment(
                &commitment
            )
            .is_ok()
        );
        commitments.insert(commitment);
        assert_eq!(
            output.asset(),
            AssetField::Explicit(AssetId::from_internal(PROTOCOL_ASSET)),
        );
    }
    // Two destinations, two distinct commitments: the blinder is a
    // function of the position, so equal values would still commit
    // apart.
    assert_eq!(commitments.len(), 2);

    let model = built
        .report()
        .construction_model()
        .expect("a private build records its model");
    assert!(
        model
            .non_claims()
            .contains(&PrivateConstructionNonClaim::NotAProductionMultiOwnerPrivacyProtocol)
    );
    assert!(
        model
            .non_claims()
            .contains(&PrivateConstructionNonClaim::FieldFormSettledOnlyOnTheTarget)
    );
    assert_eq!(built.report().consumed_total(), None);
}
