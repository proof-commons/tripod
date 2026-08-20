//! The one exact linked bundle and ABI every fixture binds to.
//!
//! `pkg:vectors:contract` scopes this package to "one exact linked
//! bundle and ABI", and Guide-12 §17.2 makes target materialization a
//! function of the bundle, the ABI, the typed request, and the public
//! input view. This module owns the first two so that every fixture in
//! the crate is bound to the same pair rather than to a pair each test
//! happens to rebuild.
//!
//! # Why this is library code and not a test helper
//!
//! The transaction and linker crates build the same demonstration bundle
//! inside `#[cfg(test)]` modules, which is right for them: the bundle is
//! an input to their own tests. Here it is the *substrate a later wave
//! executes against a real node*, and a substrate reachable only from a
//! cfg-gated module cannot be executed by anything. It is therefore
//! public, and the recipe is stated once here rather than a third time.
//!
//! # Everything here is public disposable test material
//!
//! Every byte constant below is a fixed public value with no
//! provenance in production material. None authorizes anything, none is
//! secret, and all of it belongs to a disposable development chain
//! `(´[ADR015-rule:security:test-material]´)`. The internal key is a
//! meaningless 32-byte pattern; §11.3's unspendability obligation is
//! outstanding and this module does not discharge it.

use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::operation_plan::{
    PlacementSearchLimits, ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
};
use linker::{CandidateLinkedBundle, LinkDeploymentParameters, SelfCommitmentStrategy};
use realization::{RealizationScope, derive};
use tapscript::{CompactAshSymbols, demonstration_policy, emit_candidate_bundle};
use target_elements::{ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript};
use transaction::{
    AshInstanceOrigin, CandidateTransactionAbi, OutputKeyParity, PinnedAshInstance,
    derive_candidate_abi,
};

use crate::error::FixtureBundleRefusal;

/// The disposable closed protocol asset.
pub const CLOSED_ASSET: [u8; 32] = [0xa1; 32];
/// The disposable reserve asset the sponsor region uses.
pub const RESERVE_ASSET: [u8; 32] = [0xa2; 32];
/// The disposable sponsor-change program digest.
pub const SPONSOR_CHANGE_PROGRAM: [u8; 32] = [0xa4; 32];
/// The meaningless taproot internal key the fixture link commits to.
pub const INTERNAL_KEY: [u8; 32] = [0xa6; 32];
/// The disposable fee-program digest the resolved symbols carry.
pub const FEE_PROGRAM: [u8; 32] = [0xa5; 32];
/// The pinned output program every fixture ASH input pays to.
pub const PINNED_PROGRAM: [u8; 32] = [0xcc; 32];

/// The one bundle-and-ABI pair every fixture in this crate binds to.
///
/// Cloned out of a process-wide cache, because deriving it runs the
/// compiler's placement search and the value is a pure function of the
/// constants above.
#[derive(Clone, Debug)]
pub struct FixtureBundle {
    target: ReviewedElementsTapscriptDefinition,
    plan: ValidatedTargetOperationPlan,
    bundle: CandidateLinkedBundle,
    abi: CandidateTransactionAbi,
}

impl FixtureBundle {
    /// The reviewed target contract, unmodified.
    #[must_use]
    pub const fn target(&self) -> &ReviewedElementsTapscriptDefinition {
        &self.target
    }

    /// The validated compact-ASH operation plan.
    ///
    /// Every relation and coverage census this package states is
    /// recomputed from here, never restated from prose.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedTargetOperationPlan {
        &self.plan
    }

    /// The candidate linked bundle.
    #[must_use]
    pub const fn linked(&self) -> &CandidateLinkedBundle {
        &self.bundle
    }

    /// The candidate transaction ABI derived from that bundle.
    #[must_use]
    pub const fn abi(&self) -> &CandidateTransactionAbi {
        &self.abi
    }

    /// The pinned ASH instance the ABI was derived against.
    ///
    /// # Errors
    ///
    /// [`FixtureBundleRefusal::Abi`] when the fixture program is not the
    /// reviewed taproot width, which the constants above make
    /// unreachable but which is not asserted away.
    pub fn pin(&self) -> Result<PinnedAshInstance, FixtureBundleRefusal> {
        fixture_pin(&self.target)
    }
}

fn fixture_pin(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<PinnedAshInstance, FixtureBundleRefusal> {
    let _ = target;
    PinnedAshInstance::new(
        &PINNED_PROGRAM,
        OutputKeyParity::Even,
        target_elements::LeafVersion::TAPSCRIPT,
        INTERNAL_KEY.to_vec(),
        AshInstanceOrigin::SyntheticTestFunding,
    )
    .map_err(FixtureBundleRefusal::Abi)
}

fn limit(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap_or(NonZeroU64::MIN)
}

fn build() -> Result<FixtureBundle, FixtureBundleRefusal> {
    let target = reviewed_elements_tapscript().map_err(FixtureBundleRefusal::Target)?;

    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .map_err(FixtureBundleRefusal::Realization)?;
    let scope = CompilationScope::from_operations([OperationId::CompactAsh])
        .map_err(FixtureBundleRefusal::Compile)?;
    let policy = AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
    let input = bind_input(&ARCHITECTURE, realization, scope, policy)
        .map_err(FixtureBundleRefusal::Compile)?;
    let plan = plan_compact_ash_target_operation(
        &input,
        PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
    )
    .map_err(FixtureBundleRefusal::Compile)?;

    // The placeholder symbols are deliberately a different width and a
    // different pattern from the resolved ones, so a link that failed to
    // relocate would produce a program the resolved widths reject rather
    // than one that merely looks unchanged.
    let placeholders = CompactAshSymbols::new(
        &target,
        vec![0x11; 32],
        vec![0x22; 32],
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .map_err(FixtureBundleRefusal::Symbols)?;
    let emitted = emit_candidate_bundle(&target, &plan, demonstration_policy(), placeholders)
        .map_err(FixtureBundleRefusal::Emission)?;

    let resolved = CompactAshSymbols::new(
        &target,
        CLOSED_ASSET.to_vec(),
        RESERVE_ASSET.to_vec(),
        SPONSOR_CHANGE_PROGRAM.to_vec(),
        0,
        FEE_PROGRAM.to_vec(),
    )
    .map_err(FixtureBundleRefusal::Symbols)?;
    let deployment = LinkDeploymentParameters::new(
        &target,
        resolved,
        INTERNAL_KEY.to_vec(),
        SelfCommitmentStrategy::IdentityIntrospection,
        NonZeroU32::new(8).unwrap_or(NonZeroU32::MIN),
    )
    .map_err(FixtureBundleRefusal::Link)?;

    let bundle = linker::link_candidate(&target, &emitted, &deployment)
        .map_err(FixtureBundleRefusal::Link)?;

    let pin = fixture_pin(&target)?;
    let abi = derive_candidate_abi(&target, &bundle, pin).map_err(FixtureBundleRefusal::Abi)?;

    Ok(FixtureBundle {
        target,
        plan,
        bundle,
        abi,
    })
}

/// The fixture bundle, derived once per process and handed out by clone.
///
/// # Errors
///
/// A [`FixtureBundleRefusal`] naming the layer that refused. The
/// constants this module states make every one of them unreachable, but
/// none is asserted away: a refusal here is a real finding about the
/// layers below, and swallowing it would hide exactly the kind of
/// regression this substrate exists to notice.
pub fn fixture_bundle() -> Result<FixtureBundle, FixtureBundleRefusal> {
    static BUNDLE: LazyLock<Result<FixtureBundle, FixtureBundleRefusal>> = LazyLock::new(build);
    BUNDLE.clone()
}

#[cfg(test)]
mod tests {
    use super::{CLOSED_ASSET, PINNED_PROGRAM, RESERVE_ASSET, fixture_bundle};
    use linker::LinkedArtifactStatus;
    use transaction::{AbiObligation, AbiStatus};

    #[test]
    fn the_fixture_bundle_builds_and_is_the_same_bundle_every_time() {
        let first = fixture_bundle().expect("the fixture bundle builds");
        let second = fixture_bundle().expect("the fixture bundle builds");
        assert_eq!(first.linked(), second.linked());
        assert_eq!(first.abi(), second.abi());
        assert_eq!(first.plan(), second.plan());
    }

    #[test]
    fn the_bundle_and_abi_are_candidates_and_stay_candidates() {
        // §1.9 keeps the candidate and final states distinct, and this
        // package must not be the place either quietly graduates.
        // The linked bundle is a `Prototype`, and `derive_candidate_abi`
        // accepts nothing else — an ABI may only be derived from an
        // artifact that has not yet claimed to be operation-proven.
        // Recording the actual status here rather than the one it would
        // be flattering to claim is the whole point of the assertion.
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        assert_eq!(fixture.linked().status(), LinkedArtifactStatus::Prototype);
        assert_eq!(fixture.abi().status(), AbiStatus::Candidate);
    }

    #[test]
    fn the_abi_still_carries_its_outstanding_obligations() {
        // The fixtures carry these obligations forward; nothing in this
        // wave discharges one, and a wave that silently emptied the set
        // would be claiming verification it did not perform.
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        let outstanding = fixture.abi().outstanding_obligations();
        assert!(outstanding.holds(AbiObligation::PinnedOutputKeyUnverifiedAgainstTree));
        assert!(outstanding.count().get() >= 1);
    }

    #[test]
    fn the_fixture_constants_are_distinct_from_one_another() {
        // A collision would make one mutation indistinguishable from
        // another — a wrong-asset vector that used the right asset.
        let mut seen = std::collections::BTreeSet::new();
        for constant in [
            super::CLOSED_ASSET,
            super::RESERVE_ASSET,
            super::SPONSOR_CHANGE_PROGRAM,
            super::INTERNAL_KEY,
            super::FEE_PROGRAM,
            super::PINNED_PROGRAM,
        ] {
            assert!(seen.insert(constant), "two fixture constants collide");
        }
        assert_ne!(CLOSED_ASSET, RESERVE_ASSET);
        assert_ne!(PINNED_PROGRAM, CLOSED_ASSET);
    }
}
