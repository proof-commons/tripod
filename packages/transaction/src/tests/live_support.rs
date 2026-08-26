//! Shared fixtures for the live-transfer tests (§12).
//!
//! # The linked live bundle is a real one
//!
//! Reached the way an external consumer would: derive the realization,
//! bind the compiler input, plan the live-transfer target operation,
//! derive a constructor per representation, emit through the backend's
//! own entry point, and link. Nothing here hand-assembles a constructor
//! or a bundle, because a hand-assembled one would let an ABI derive
//! against an artifact no backend and no linker produced.
//!
//! # The curve capability here is a fixture, and it is not arithmetic
//!
//! [`FixtureCurve`] answers both of
//! [`crate::live_taproot::LiveCurveCapability`]'s questions with a
//! tagged hash and a constant parity. It is *not* the taproot tweak, it
//! is not curve arithmetic at all, and no output key it returns is a
//! point of anything. That is deliberate and is the whole reason the
//! capability exists: this crate does not own the target's curve, its
//! own tests must not pretend to, and the real arithmetic belongs to the
//! package that does — where it is checked against the target's own
//! published vectors rather than against itself.
//!
//! What these tests can establish with a fixture capability is
//! everything the ABI is responsible for: which owner a program belongs
//! to, which position a family occupies, which bytes an owner is asked
//! to bind to, and which refusal a disagreement produces. What they
//! cannot establish is that a target would accept the spend, and no
//! assertion here says otherwise.
//!
//! # Every owner key is public test material
//!
//! Distinguishable byte strings at the approved encoding's exact width,
//! test material in the sense `(´[ADR015-rule:security:test-material]´)`
//! fixes: public, carrying no secret, standing for no party, and paired
//! with no private scalar anywhere in this crate.

use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::live_transfer_plan::{
    ValidatedLiveTransferOperationPlan, plan_live_transfer_target_operation,
};
use compiler::operation_plan::PlacementSearchLimits;
use linker::live_backend::LiveTransferRepresentationPlan;
use linker::{
    CandidateLinkedLiveTransferBundle, LiveLinkDeploymentParameters, OwnerParameter,
    link_live_candidate,
};
use realization::{RealizationScope, derive};
use tapscript::{
    CandidateRelocatableLiveTransferBundle, LiveTransferShapeSet, LiveTransferSymbols, OwnerKey,
    demonstration_live_shape_set, derive_live_receipt_constructor, emit_candidate_live_bundle,
    fee_bearing_live_shape_set, owner_key_encoding_closure, static_transfer_leaf_set,
};
use target_elements::ReviewedElementsTapscriptDefinition;

use super::reviewed_target;
use crate::bytes::{AssetField, AssetId, Outpoint, ValueField};
use crate::live_abi::{CandidateLiveTransferAbi, derive_live_transfer_abi};
use crate::live_taproot::{LiveCurveCapability, TweakedOutputKey};
use crate::taproot::{Digest32, OutputKeyParity, tagged_hash};
use crate::view::PublicOutputView;

/// The protocol asset the demonstration live deployment resolves.
pub(super) const LIVE_PROTOCOL_ASSET: [u8; 32] = [0xb1; 32];

/// The reserve asset the demonstration live deployment resolves.
pub(super) const LIVE_RESERVE_ASSET: [u8; 32] = [0xb2; 32];

/// The sponsor-change witness program the deployment resolves.
pub(super) const LIVE_SPONSOR_CHANGE_PROGRAM: [u8; 32] = [0xb4; 32];

/// The unspendable internal key the deployment resolves.
pub(super) const LIVE_INTERNAL_KEY: [u8; 32] = [0xb6; 32];

/// The first demonstration owner's published metadata bytes.
pub(super) const FIRST_OWNER: [u8; 32] = [0x11; 32];

/// The second demonstration owner's published metadata bytes.
pub(super) const SECOND_OWNER: [u8; 32] = [0x33; 32];

/// The published randomness a private test construction consumes.
pub(super) const PUBLISHED_RANDOMNESS: [u8; 32] = [0x7e; 32];

/// The tag the fixture capability derives its stand-in keys under.
///
/// Deliberately not one of the target's tags. A fixture that hashed
/// under `TapTweak/elements` would produce a value indistinguishable at
/// a glance from a real output key, and the point of the fixture is that
/// it is distinguishable.
const FIXTURE_KEY_TAG: &str = "Guide13/live-fixture-output-key";

/// The tag the fixture capability derives its stand-in commitments
/// under.
const FIXTURE_COMMITMENT_TAG: &str = "Guide13/live-fixture-value-commitment";

/// A curve capability that performs no curve arithmetic.
///
/// See this module's own documentation. Every value it returns is a
/// tagged hash of its inputs: reproducible, distinguishable, and not a
/// point.
pub(super) struct FixtureCurve;

impl LiveCurveCapability for FixtureCurve {
    fn owner_key_is_a_curve_point(&self, owner: &[u8]) -> bool {
        // The fixture's rule: any offering of the approved width is
        // accepted, and the empty one is not. Enough to exercise the
        // refusal path without claiming to decide curve membership.
        !owner.is_empty()
    }

    fn output_key(&self, internal_key: &[u8], merkle_root: &Digest32) -> Option<TweakedOutputKey> {
        let mut preimage = Vec::with_capacity(internal_key.len() + merkle_root.len());
        preimage.extend_from_slice(internal_key);
        preimage.extend_from_slice(merkle_root);
        Some(TweakedOutputKey::new(
            tagged_hash(FIXTURE_KEY_TAG, &preimage),
            OutputKeyParity::Even,
        ))
    }
}

/// A curve capability that decides no owner key is a point.
pub(super) struct RefusingCurve;

impl LiveCurveCapability for RefusingCurve {
    fn owner_key_is_a_curve_point(&self, _owner: &[u8]) -> bool {
        false
    }

    fn output_key(
        &self,
        _internal_key: &[u8],
        _merkle_root: &Digest32,
    ) -> Option<TweakedOutputKey> {
        None
    }
}

/// A confidential value capability that performs no commitment
/// arithmetic.
///
/// The same disclaimer as [`FixtureCurve`]: the 33 bytes it returns are
/// a prefix and a tagged hash, not a Pedersen commitment. It exercises
/// the field form and the ordering rule and establishes nothing about
/// what a target would read.
pub(super) struct FixturePrivateValue;

impl crate::live_private::PrivateValueCapability for FixturePrivateValue {
    fn value_commitment(
        &self,
        asset: AssetId,
        value: crate::live_request::ProtocolValue,
        randomness: &crate::live_request::PublicTestRandomness,
        position: u16,
    ) -> Option<[u8; crate::bytes::COMMITMENT_BYTES]> {
        let mut preimage = Vec::with_capacity(74);
        preimage.extend_from_slice(asset.internal());
        preimage.extend_from_slice(&value.amount().to_be_bytes());
        preimage.extend_from_slice(randomness.bytes());
        preimage.extend_from_slice(&position.to_be_bytes());

        let digest = tagged_hash(FIXTURE_COMMITMENT_TAG, &preimage);
        let mut commitment = [0_u8; crate::bytes::COMMITMENT_BYTES];
        // The first of the two confidential value prefixes. Which of the
        // two a real commitment carries depends on the point's parity,
        // and neither has been exercised against the target.
        commitment[0] = crate::bytes::VALUE_COMMITMENT_PREFIXES[0];
        commitment[1..].copy_from_slice(&digest);
        Some(commitment)
    }
}

/// The validated live-transfer plan, derived once and handed out by
/// clone.
fn live_transfer_plan() -> ValidatedLiveTransferOperationPlan {
    static PLAN: LazyLock<ValidatedLiveTransferOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let realization =
            derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
        let scope = CompilationScope::from_operations([OperationId::TransferLive])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_live_transfer_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// One owner's canonical metadata from fixture bytes.
pub(super) fn owner_key(bytes: &[u8]) -> OwnerKey {
    let target = reviewed_target();
    let closure = owner_key_encoding_closure(target.definition().authorization());
    OwnerKey::new(&closure, closure.approved(), bytes.to_vec())
        .expect("the fixture is the approved encoding at its exact width")
}

/// One owner's link parameter from fixture bytes.
pub(super) fn owner(bytes: &[u8]) -> OwnerParameter {
    OwnerParameter::new(owner_key(bytes))
}

/// One representation's relocatable bundle for one owner.
fn single_live_bundle(
    representation: LiveTransferRepresentationPlan,
    owner: &[u8],
) -> CandidateRelocatableLiveTransferBundle {
    live_bundle_over(representation, owner, demonstration_live_shape_set())
}

/// One representation's relocatable bundle over a NAMED shape set.
///
/// The shape set is a parameter because the demonstration set carries no
/// fee-bearing member and a self-paying candidate needs one. Passing it
/// keeps the two deployments one function apart rather than two copies,
/// so a change to how a bundle is emitted cannot reach one and miss the
/// other.
fn live_bundle_over(
    representation: LiveTransferRepresentationPlan,
    owner: &[u8],
    shapes: LiveTransferShapeSet,
) -> CandidateRelocatableLiveTransferBundle {
    let target = reviewed_target();
    let leaves = static_transfer_leaf_set(representation, &shapes);
    let constructor = derive_live_receipt_constructor(
        &target,
        &live_transfer_plan(),
        representation,
        owner_key(owner),
        shapes,
        leaves,
    )
    .expect("the demonstration constructor derives");

    emit_candidate_live_bundle(
        &target,
        &live_transfer_plan(),
        &constructor,
        placeholder_live_symbols(&target),
    )
    .expect("the demonstration bundle emits")
}

/// The placeholder live symbols the bundles are laid out against.
fn placeholder_live_symbols(target: &ReviewedElementsTapscriptDefinition) -> LiveTransferSymbols {
    LiveTransferSymbols::new(
        target,
        vec![0x5a; 32],
        vec![0x22; 32],
        1,
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .expect("the placeholder symbols are the reviewed widths")
}

/// The resolved live symbol values of the demonstration link.
fn resolved_live_symbols(target: &ReviewedElementsTapscriptDefinition) -> LiveTransferSymbols {
    LiveTransferSymbols::new(
        target,
        LIVE_PROTOCOL_ASSET.to_vec(),
        LIVE_RESERVE_ASSET.to_vec(),
        1,
        LIVE_SPONSOR_CHANGE_PROGRAM.to_vec(),
        0,
        vec![0xb5; 32],
    )
    .expect("the resolved symbols are the reviewed widths")
}

/// The demonstration linked live bundle, linked once and cloned.
///
/// Two owners across both representations, so that the destination
/// constructor table is a table rather than one row and an owner without
/// a constructor is a case a test can state.
pub(super) fn linked_live_bundle() -> CandidateLinkedLiveTransferBundle {
    static BUNDLE: LazyLock<CandidateLinkedLiveTransferBundle> = LazyLock::new(|| {
        let target = reviewed_target();
        let bundles = vec![
            single_live_bundle(LiveTransferRepresentationPlan::Explicit, &FIRST_OWNER),
            single_live_bundle(
                LiveTransferRepresentationPlan::PrivateCommitted,
                &FIRST_OWNER,
            ),
            single_live_bundle(LiveTransferRepresentationPlan::Explicit, &SECOND_OWNER),
            single_live_bundle(
                LiveTransferRepresentationPlan::PrivateCommitted,
                &SECOND_OWNER,
            ),
        ];
        let deployment = LiveLinkDeploymentParameters::new(
            &target,
            resolved_live_symbols(&target),
            LIVE_INTERNAL_KEY.to_vec(),
            NonZeroU32::new(8).expect("eight is nonzero"),
        )
        .expect("the demonstration deployment parameters are the reviewed widths");

        link_live_candidate(&target, &bundles, &deployment).expect("the demonstration live link")
    });
    BUNDLE.clone()
}

/// The demonstration candidate live-transfer ABI.
pub(super) fn live_abi() -> CandidateLiveTransferAbi {
    derive_live_transfer_abi(&reviewed_target(), &linked_live_bundle(), &FixtureCurve)
        .expect("the demonstration live ABI derives")
}

/// The candidate ABI whose shape set admits a sponsorless self-paid fee.
///
/// A SEPARATE deployment rather than a widened demonstration one, for
/// the reason the fee-bearing vocabulary is separate everywhere else:
/// admitting the fee axis into the demonstration bounds would unroll
/// extra members, move the committed taptree, and move every recorded
/// demonstration digest with it.
pub(super) fn fee_bearing_live_abi() -> CandidateLiveTransferAbi {
    static BUNDLE: LazyLock<CandidateLinkedLiveTransferBundle> = LazyLock::new(|| {
        let target = reviewed_target();
        let bundles = vec![
            live_bundle_over(
                LiveTransferRepresentationPlan::Explicit,
                &FIRST_OWNER,
                fee_bearing_live_shape_set(),
            ),
            live_bundle_over(
                LiveTransferRepresentationPlan::Explicit,
                &SECOND_OWNER,
                fee_bearing_live_shape_set(),
            ),
        ];
        let deployment = LiveLinkDeploymentParameters::new(
            &target,
            resolved_live_symbols(&target),
            LIVE_INTERNAL_KEY.to_vec(),
            NonZeroU32::new(8).expect("eight is nonzero"),
        )
        .expect("the fee-bearing deployment parameters are the reviewed widths");

        link_live_candidate(&target, &bundles, &deployment).expect("the fee-bearing live link")
    });
    derive_live_transfer_abi(&reviewed_target(), &BUNDLE.clone(), &FixtureCurve)
        .expect("the fee-bearing live ABI derives")
}

/// An ABI over a link that carries one representation and not the other.
///
/// Both owners, one plan. A link like this is legitimate rather than
/// defective — the linker reports which plans a candidate carries
/// precisely because a deployment may choose one — and it is the only
/// way to reach a request whose selected plan the ABI has no
/// constructor for.
pub(super) fn single_representation_live_abi(
    representation: LiveTransferRepresentationPlan,
) -> CandidateLiveTransferAbi {
    let target = reviewed_target();
    let bundles = vec![
        single_live_bundle(representation, &FIRST_OWNER),
        single_live_bundle(representation, &SECOND_OWNER),
    ];
    let deployment = LiveLinkDeploymentParameters::new(
        &target,
        resolved_live_symbols(&target),
        LIVE_INTERNAL_KEY.to_vec(),
        NonZeroU32::new(8).expect("eight is nonzero"),
    )
    .expect("the demonstration deployment parameters are the reviewed widths");
    let linked =
        link_live_candidate(&target, &bundles, &deployment).expect("a one-representation link");

    derive_live_transfer_abi(&target, &linked, &FixtureCurve)
        .expect("the one-representation live ABI derives")
}

/// A public view of a sponsor coin at `outpoint`, in the reserve asset.
///
/// The counterpart of [`receipt_view`] for the sponsor region. A
/// sponsored construction offers coins from the same view its receipts
/// come from, and until the reserve-asset guard existed no live fixture
/// ever stated one — every sponsored fixture named an outpoint in its
/// offer and showed the builder nothing about it.
pub(super) fn sponsor_view(outpoint: Outpoint, amount: u64) -> PublicOutputView {
    PublicOutputView::new(
        outpoint,
        AssetField::Explicit(AssetId::from_internal(LIVE_RESERVE_ASSET)),
        ValueField::Explicit(amount),
        LIVE_SPONSOR_CHANGE_PROGRAM.to_vec(),
    )
}

/// A public view of one owner's live receipt at `outpoint`.
pub(super) fn receipt_view(
    abi: &CandidateLiveTransferAbi,
    outpoint: Outpoint,
    owner: &OwnerParameter,
    representation: LiveTransferRepresentationPlan,
    value: ValueField,
) -> PublicOutputView {
    let program = abi
        .destinations()
        .get(owner, representation)
        .expect("the fixture owner has a linked constructor")
        .instance()
        .program()
        .to_vec();
    PublicOutputView::new(
        outpoint,
        AssetField::Explicit(AssetId::from_internal(LIVE_PROTOCOL_ASSET)),
        value,
        program,
    )
}
