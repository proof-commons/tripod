//! Transaction tests and their shared fixtures.
//!
//! # The bundle under test is a real one
//!
//! Every fixture below reaches the linked bundle the way an external
//! consumer would: derive the realization, bind the compiler input,
//! plan the compact-ASH target operation, emit the bundle through the
//! backend's public entry point, and link it. Nothing here
//! hand-assembles a bundle or an ABI, because a hand-assembled one
//! would let a construction succeed against an artifact no backend and
//! no linker produced.
//!
//! # The pinned instance is a fixture and says so
//!
//! The pin below is a public, meaningless byte string — test material
//! in the sense `(´[ADR015-rule:security:test-material]´)` fixes. It
//! carries no secret and stands for no deployed object. It could not be
//! anything else: a real pin is the taproot output key of a real
//! constructor instance, and this crate deliberately does not compute
//! one. Its origin is recorded as synthetic, which is what makes every
//! report built on it carry §15.9's five disclaimers.
//!
//! # Exact bytes are checked against hand computation
//!
//! The encoding tests state their expected bytes as literal field
//! groups written out from the reviewed target's own serializer, and
//! the taproot tests state digests computed outside this crate. An
//! encoder compared only with itself would agree however wrong it was,
//! which is the whole reason those two files look the way they do.

mod abi_tests;
mod construction_tests;
mod encoding_tests;
mod guide13_preflight_tests;
mod taproot_tests;

use std::num::{NonZeroU8, NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::operation_plan::{
    PlacementSearchLimits, ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
};
use linker::backend::{CompactAshShape, SponsorChangePresence};
use linker::{CandidateLinkedBundle, LinkDeploymentParameters, SelfCommitmentStrategy};
use realization::{RealizationScope, derive};
use tapscript::{CompactAshSymbols, demonstration_policy, emit_candidate_bundle};
use target_elements::{ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript};

use crate::abi::{CandidateTransactionAbi, derive_candidate_abi};
use crate::bytes::{AssetField, Outpoint, Txid, ValueField};
use crate::taproot::{AshInstanceOrigin, OutputKeyParity, PinnedAshInstance};
use crate::view::{PublicConstructionView, PublicOutputView};

/// The reviewed contract, unmodified.
fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// The validated compact-ASH plan, derived once and handed out by
/// clone.
fn compact_ash_plan() -> ValidatedTargetOperationPlan {
    static PLAN: LazyLock<ValidatedTargetOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let realization =
            derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
        let scope = CompilationScope::from_operations([OperationId::CompactAsh])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_compact_ash_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// The closed protocol asset the demonstration deployment resolves.
const CLOSED_ASSET: [u8; 32] = [0xa1; 32];

/// The reserve asset the demonstration deployment resolves.
const RESERVE_ASSET: [u8; 32] = [0xa2; 32];

/// The sponsor-change witness program the deployment resolves.
const SPONSOR_CHANGE_PROGRAM: [u8; 32] = [0xa4; 32];

/// The unspendable internal key the deployment resolves.
const INTERNAL_KEY: [u8; 32] = [0xa6; 32];

/// The pinned ASH witness program of the fixture instance.
const PINNED_PROGRAM: [u8; 32] = [0xcc; 32];

/// The demonstration linked bundle, linked once and cloned.
fn linked_bundle() -> CandidateLinkedBundle {
    static BUNDLE: LazyLock<CandidateLinkedBundle> = LazyLock::new(|| {
        let target = reviewed_target();
        let placeholders = CompactAshSymbols::new(
            &target,
            vec![0x11; 32],
            vec![0x22; 32],
            vec![0x44; 20],
            0,
            vec![0x55; 32],
        )
        .expect("the placeholder symbols are the reviewed widths");
        let bundle = emit_candidate_bundle(
            &target,
            &compact_ash_plan(),
            demonstration_policy(),
            placeholders,
        )
        .expect("the demonstration bundle emits");

        let resolved = CompactAshSymbols::new(
            &target,
            CLOSED_ASSET.to_vec(),
            RESERVE_ASSET.to_vec(),
            SPONSOR_CHANGE_PROGRAM.to_vec(),
            0,
            vec![0xa5; 32],
        )
        .expect("the resolved symbols are the reviewed widths");
        let deployment = LinkDeploymentParameters::new(
            &target,
            resolved,
            INTERNAL_KEY.to_vec(),
            SelfCommitmentStrategy::IdentityIntrospection,
            NonZeroU32::new(8).expect("eight is nonzero"),
        )
        .expect("the demonstration deployment parameters are the reviewed widths");

        linker::link_candidate(&target, &bundle, &deployment).expect("the demonstration link")
    });
    BUNDLE.clone()
}

/// The fixture pin: a public, meaningless program and an even parity.
fn pin() -> PinnedAshInstance {
    PinnedAshInstance::new(
        &PINNED_PROGRAM,
        OutputKeyParity::Even,
        target_elements::LeafVersion::TAPSCRIPT,
        INTERNAL_KEY.to_vec(),
        AshInstanceOrigin::SyntheticTestFunding,
    )
    .expect("the fixture program is the reviewed taproot width")
}

/// The demonstration candidate ABI.
fn candidate_abi() -> CandidateTransactionAbi {
    derive_candidate_abi(&reviewed_target(), &linked_bundle(), pin())
        .expect("the demonstration ABI derives")
}

/// The shape with `ash` ASH inputs, `sponsors` sponsors, and change or
/// not.
fn shape(ash: u8, sponsors: u8, change: bool) -> CompactAshShape {
    let bounds =
        linker::backend::CompactAshShapeBounds::new(NonZeroU8::new(4).expect("four is nonzero"), 1)
            .expect("the demonstration bounds");
    CompactAshShape::new(
        bounds,
        NonZeroU8::new(ash).expect("the fixture batch is nonzero"),
        sponsors,
        if change {
            SponsorChangePresence::Present
        } else {
            SponsorChangePresence::Absent
        },
    )
    .expect("the fixture shape is admitted")
}

/// The outpoint of `index` of a transaction whose identifier is `byte`
/// repeated.
fn outpoint(byte: u8, index: u32) -> Outpoint {
    Outpoint::new(Txid::from_internal([byte; 32]), index).expect("the fixture index is in range")
}

/// A public view of `outpoint` carrying the closed asset, `amount`, and
/// the pinned program.
fn ash_view(
    target: &ReviewedElementsTapscriptDefinition,
    outpoint: Outpoint,
    amount: u64,
) -> PublicOutputView {
    PublicOutputView::new(
        outpoint,
        AssetField::Explicit(crate::bytes::AssetId::from_internal(CLOSED_ASSET)),
        ValueField::Explicit(amount),
        pin()
            .output_script(target)
            .expect("the pinned program builds a script"),
    )
}

/// A public view of a sponsor input carrying the reserve asset and an
/// admitted program class.
fn sponsor_view(outpoint: Outpoint, amount: u64) -> PublicOutputView {
    let mut program = vec![0x00, 0x14];
    program.extend_from_slice(&[0xd0; 20]);
    PublicOutputView::new(
        outpoint,
        AssetField::Explicit(crate::bytes::AssetId::from_internal(RESERVE_ASSET)),
        ValueField::Explicit(amount),
        program,
    )
}

/// The public view holding exactly these outputs.
fn view(entries: impl IntoIterator<Item = PublicOutputView>) -> PublicConstructionView {
    PublicConstructionView::new(entries).expect("the fixture views name distinct outpoints")
}
