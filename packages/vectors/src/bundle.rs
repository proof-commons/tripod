//! The one exact linked bundle and ABI every fixture binds to.
//!
//! `(´[PLAN-pkg:vectors:contract]´)` scopes this package to "one exact linked
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
//!
//! Most of those constants are arbitrary patterns, and one is not.
//! [`PINNED_PROGRAM`] and [`PINNED_PARITY`] are *derived* facts about
//! this bundle rather than choices — the taproot output key of its own
//! committed tree, and that key's parity. Their derivation chain, and
//! the cross-check that keeps them from drifting away from the tree,
//! are stated on the constants themselves.

use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{
    AnalysisPolicy, BoundCompilerInput, CompilationScope, ProofSearchLimits, bind_input,
};
use compiler::operation_plan::{
    PlacementSearchLimits, ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
};
use linker::{CandidateLinkedBundle, LinkDeploymentParameters, SelfCommitmentStrategy};
use realization::{RealizationScope, derive};
use tapscript::{CompactAshSymbols, demonstration_policy, emit_candidate_bundle};
use target_elements::{ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript};
use target_elements_conformance::constructor::tagged;
use target_elements_conformance::constructor::tree as oracle_tree;
use transaction::{
    AshInstanceOrigin, CandidateTransactionAbi, OutputKeyParity, PinnedAshInstance, commit_tree,
    derive_candidate_abi,
};

use crate::error::FixtureBundleRefusal;

/// The disposable closed protocol asset.
pub const CLOSED_ASSET: [u8; 32] = [0xa1; 32];
/// The disposable reserve asset the sponsor region uses.
pub const RESERVE_ASSET: [u8; 32] = [0xa2; 32];
/// The disposable sponsor-change program payload.
///
/// # Why this one really is a choice, unlike the fee digest
///
/// The change output is a witness program, and
/// `OP_INSPECTOUTPUTSCRIPTPUBKEY` pushes a witness program's *payload*
/// and its version rather than a hash of the whole script. So what the
/// emitted coordinator compares against is the thirty-two byte payload
/// of a version-zero program, and any thirty-two bytes name one. A
/// deployment picks it.
///
/// [`fee_program_digest`] is not like that, and the difference is worth
/// keeping straight because the two symbols look alike: a fee output
/// has no witness program at all, so the target falls back to the
/// SHA-256 of the whole script under a version of -1, and the value is
/// then determined rather than chosen. Making this constant "consistent"
/// with that one would break a check that currently passes.
pub const SPONSOR_CHANGE_PROGRAM: [u8; 32] = [0xa4; 32];
/// The meaningless taproot internal key the fixture link commits to.
pub const INTERNAL_KEY: [u8; 32] = [0xa6; 32];
/// The fee-program digest the resolved symbols carry.
///
/// # This value is derived, not chosen, and an earlier revision chose it
///
/// It is the SHA-256 of the *empty* program, because that is what the
/// target's fee role is. `OP_INSPECTOUTPUTSCRIPTPUBKEY` pushes a
/// witness program and its version for an output that has one, and for
/// an output that does not it pushes the SHA-256 of the whole
/// `scriptPubKey` under a version of -1 (`pushspk` and
/// `GetOutputScriptPubKeysSHA256` in `src/script/interpreter.cpp`). The
/// fee role's whole identity is its empty program, so the emitted
/// coordinator's check against this digest is a check against
/// SHA-256 of nothing.
///
/// An earlier revision carried `[0xa5; 32]`, an arbitrary pattern, and
/// nothing noticed while no sponsored transaction was ever built: the
/// fee fragment is emitted only for a sponsored shape. The first four
/// sponsored rows put to a live node were refused
/// `Script failed an OP_EQUALVERIFY operation`, which is that
/// comparison failing. This is the same class of defect Wave 11 found
/// in `CLOSED_ASSET` — a value the target determines, written down as
/// though it were a deployment's to pick.
///
/// The digest is computed rather than stated, through the conformance
/// package's own SHA-256, so it cannot drift from the rule above.
#[must_use]
pub fn fee_program_digest() -> [u8; 32] {
    tagged::sha256(&[])
}
/// The pinned output program every fixture ASH input pays to.
///
/// # This value is derived, not chosen
///
/// It is the BIP-341-style taproot output key of this very bundle: the
/// x-only key obtained by tweaking [`INTERNAL_KEY`] by the merkle root
/// of the committed tree that `transaction::commit_tree` builds over
/// the linked bundle below. An earlier revision carried a fixed byte
/// pattern here, which is not a curve point at all and therefore not
/// the tweak of any key by any root; an output created at that program
/// was unspendable by construction, because no control block can
/// satisfy a taproot commitment check against a program that is not a
/// key.
///
/// # Where the derivation happens, and what re-checks it
///
/// Not here. This package computes merkle roots and refuses the curve
/// arithmetic that turns one into an output key — that refusal is the
/// entire content of the `PinnedOutputKeyUnverifiedAgainstTree`
/// obligation, and importing an oracle to discharge it inside the
/// fixture would be the substrate marking its own homework. The value
/// is instead stated as a literal whose provenance is the reference
/// vector `FIXTURE_REFERENCE_OUTPUT_KEY` in the conformance package's
/// reference module, recomputed there through the adopted reference
/// bindings from this bundle's own internal key and root.
///
/// That package's reference-oracle cross-checks recompute the key and
/// assert it equals this constant, so the two cannot drift apart: a
/// change to the tree moves the derived key and fails that comparison
/// rather than silently leaving an unspendable program pinned here.
/// The dependency runs one way only — conformance dev-depends on this
/// package, never the reverse, per the vectors package contract in
/// §16.2 — so the agreement is enforced from the side that already
/// owns the curve arithmetic.
///
/// # What this still does not establish
///
/// Nothing about spendability. `PinnedOutputKeyUnverifiedAgainstTree`
/// remains outstanding and this constant does not discharge it:
/// discharging needs the funding ceremony against a real node — an
/// output that node actually created at this program, and a spend it
/// accepted. What a derived pin buys is narrower: the fixture is no
/// longer unspendable *by construction*, so that ceremony is able to
/// run at all.
pub const PINNED_PROGRAM: [u8; 32] = [
    0x5a, 0x75, 0x95, 0x13, 0x41, 0xde, 0x4b, 0xa0, 0x67, 0x16, 0x24, 0x2e, 0x04, 0x7c, 0xa4, 0xef,
    0x5c, 0x60, 0xce, 0x86, 0x9f, 0x4a, 0x63, 0x78, 0x60, 0x8d, 0xf7, 0x45, 0x09, 0x17, 0xbc, 0xed,
];

/// The parity of [`PINNED_PROGRAM`]'s implicit y coordinate.
///
/// Derived with the key itself and carried beside it for the same
/// reason, under the same cross-check. A control block states this bit
/// and an x-only program cannot, so a fixture that guessed it would
/// produce control blocks a verifier rejects for every leaf even when
/// every hash in the path is right — which is exactly what an earlier
/// revision did by declaring even parity against a derivation that
/// yields odd.
pub const PINNED_PARITY: OutputKeyParity = OutputKeyParity::Even;

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
    pin: PinnedAshInstance,
    closed_asset: [u8; 32],
    reserve_asset: [u8; 32],
    provenance: PinProvenance,
}

/// Where a bundle's pinned output key came from.
///
/// # Why the two are told apart in the type
///
/// A pin is either the literal this module states — cross-checked from
/// the far side of the §16.2 boundary, and the only pin the canonical
/// fixtures use — or one derived here for a closed asset a ceremony
/// observed, which no literal could have anticipated. They are not
/// interchangeable: the first carries the reference oracle's agreement
/// and the second carries none until a target accepts a spend at it.
///
/// Keeping them apart in the type is what stops a report from citing
/// the cross-check for a bundle the cross-check never saw.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum PinProvenance {
    /// [`PINNED_PROGRAM`], the literal the reference oracle cross-checks.
    CanonicalLiteral,
    /// Derived here, for a closed asset a funding ceremony observed.
    DerivedForObservedAsset,
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
    #[must_use]
    pub const fn pin(&self) -> &PinnedAshInstance {
        &self.pin
    }

    /// Where that pin came from.
    #[must_use]
    pub const fn pin_provenance(&self) -> PinProvenance {
        self.provenance
    }

    /// The closed protocol asset this bundle's programs are linked
    /// against.
    ///
    /// # Why a bundle has to be able to say this
    ///
    /// Every leaf substitutes the closed asset into its own bytes, so
    /// the asset is not a parameter of a run against a fixed bundle —
    /// it is part of what the bundle *is*. A ceremony that issued some
    /// other asset has not funded these programs, and this accessor is
    /// what lets a planner notice that rather than discover it as a
    /// script failure.
    #[must_use]
    pub const fn closed_asset(&self) -> [u8; 32] {
        self.closed_asset
    }

    /// The reserve asset this bundle's programs are linked against.
    ///
    /// # Why the reserve is part of the bundle too
    ///
    /// For the same reason the closed asset is, and it was found the
    /// same way. The emitted coordinator introspects every sponsor
    /// input's asset and requires it to equal this value, and requires
    /// the fee output to carry it as well. So a sponsored transaction
    /// built against a bundle linked at one reserve and funded with
    /// another is refused by the candidate's own program, and this
    /// accessor is what lets a planner notice rather than discover it
    /// as a script failure.
    ///
    /// A sponsorless bundle carries the value anyway: the symbol is
    /// substituted into all twelve leaves whether or not a shape uses
    /// it, so the pin is a function of it in either case.
    #[must_use]
    pub const fn reserve_asset(&self) -> [u8; 32] {
        self.reserve_asset
    }
}

fn canonical_pin() -> Result<PinnedAshInstance, FixtureBundleRefusal> {
    PinnedAshInstance::new(
        &PINNED_PROGRAM,
        PINNED_PARITY,
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
    build_at(CLOSED_ASSET, RESERVE_ASSET, PinProvenance::CanonicalLiteral)
}

/// The same bundle, linked against a closed asset a ceremony observed.
///
/// # Why this exists at all, and what Wave 11 found
///
/// The canonical bundle pins [`CLOSED_ASSET`], a chosen constant. Every
/// one of the twelve linked leaves substitutes that constant into its
/// own bytes — the linker reports `ClosedAsset` among the symbols it
/// substituted on all twelve — so the committed tree, and therefore the
/// taproot output key the ASH inputs pay to, are functions of it.
///
/// An Elements asset identifier is not a value anyone chooses. It is
/// derived from the issuing input's outpoint and the contract hash, so
/// no issuance can produce the chosen constant: a regtest node asked to
/// issue one answered with a value derived from that outpoint and that
/// contract hash, as every issuance must, and no such derivation lands
/// on a repeated byte. The canonical fixtures are therefore unfundable
/// on any real chain — not because the constructor is wrong, but
/// because the asset they name cannot be minted.
///
/// So a run against a real node links a *second* bundle, at the asset
/// the ceremony actually issued, and executes that one. The canonical
/// bundle is untouched: it keeps its literal pin, its reference-oracle
/// cross-check, and its byte-stable fixtures.
///
/// # Why the pin is derived here rather than stated
///
/// A literal cannot anticipate an asset the chain had not yet chosen.
/// The key is therefore computed from this bundle's own internal key
/// and the merkle root of its own committed tree, through the
/// conformance package's first-party constructor oracle — the same
/// arithmetic the reference cross-check performs for the canonical
/// literal, reached across the §16.2 edge Wave 11 takes.
///
/// That is not the substrate marking its own homework. The oracle
/// computes a key; it does not decide whether an output at that key is
/// spendable. Only the target decides that, and it decides it by
/// accepting or refusing a script-path spend whose control block
/// commits to this very tree. A wrong key is refused for every leaf.
///
/// # Errors
///
/// [`FixtureBundleRefusal`] naming the layer that refused, exactly as
/// [`fixture_bundle`] does, plus a refusal from the tree commitment or
/// the tweak when the derived key is not a point this contract admits.
/// # Why the reserve asset is observed too, and not only the closed one
///
/// Wave 11 needed one observed asset because the sponsorless shapes
/// name one. A sponsored shape names two: the coordinator requires
/// every sponsor input to carry the reserve asset and requires the fee
/// output to carry it as well. Neither can be a chosen constant on a
/// real chain, and for the *reserve* there is a further constraint the
/// closed asset does not have — a fee this target's mempool weighs is
/// one paid in the chain's own policy asset, so the reserve has to be
/// the asset the chain already uses as a reserve rather than a second
/// asset a ceremony issued.
///
/// So both come from the target: the closed asset is what the issuance
/// step chose, and the reserve is what the sponsor-funding step
/// reported paying in.
pub fn ceremony_bundle(
    closed_asset: [u8; 32],
    reserve_asset: [u8; 32],
) -> Result<FixtureBundle, FixtureBundleRefusal> {
    build_at(
        closed_asset,
        reserve_asset,
        PinProvenance::DerivedForObservedAsset,
    )
}

/// Derive the pinned instance for a linked bundle, from its own tree.
fn derived_pin(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedBundle,
) -> Result<PinnedAshInstance, FixtureBundleRefusal> {
    let tree = commit_tree(target, bundle).map_err(FixtureBundleRefusal::Abi)?;
    let root = tree.merkle_root();
    let tweak = oracle_tree::tweak(&INTERNAL_KEY, &root);
    let (key, parity_bit) =
        oracle_tree::tweaked_key(&INTERNAL_KEY, &tweak).map_err(|_| FixtureBundleRefusal::Tweak)?;
    let parity = if parity_bit == 0 {
        OutputKeyParity::Even
    } else {
        OutputKeyParity::Odd
    };
    PinnedAshInstance::new(
        &key,
        parity,
        target_elements::LeafVersion::TAPSCRIPT,
        INTERNAL_KEY.to_vec(),
        AshInstanceOrigin::SyntheticTestFunding,
    )
    .map_err(FixtureBundleRefusal::Abi)
}

/// The bound compiler input every compact-ASH analysis in this crate
/// runs over.
///
/// One spelling of the recipe. A second caller that rebound the input
/// for itself could differ from this one in a scope or a search limit,
/// and the two analyses would then be about slightly different programs
/// while both calling themselves the candidate's.
///
/// # Errors
///
/// [`FixtureBundleRefusal::Realization`] where the realization does not
/// derive, and [`FixtureBundleRefusal::Compile`] where the scope or the
/// binding is refused.
pub fn bind_compact_ash_input() -> Result<BoundCompilerInput, FixtureBundleRefusal> {
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .map_err(FixtureBundleRefusal::Realization)?;
    let scope = CompilationScope::from_operations([OperationId::CompactAsh])
        .map_err(FixtureBundleRefusal::Compile)?;
    let policy = AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
    bind_input(&ARCHITECTURE, realization, scope, policy).map_err(FixtureBundleRefusal::Compile)
}

fn build_at(
    closed_asset: [u8; 32],
    reserve_asset: [u8; 32],
    provenance: PinProvenance,
) -> Result<FixtureBundle, FixtureBundleRefusal> {
    let target = reviewed_elements_tapscript().map_err(FixtureBundleRefusal::Target)?;

    let input = bind_compact_ash_input()?;
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
        closed_asset.to_vec(),
        reserve_asset.to_vec(),
        SPONSOR_CHANGE_PROGRAM.to_vec(),
        0,
        fee_program_digest().to_vec(),
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

    let pin = match provenance {
        PinProvenance::CanonicalLiteral => canonical_pin()?,
        PinProvenance::DerivedForObservedAsset => derived_pin(&target, &bundle)?,
    };
    let abi =
        derive_candidate_abi(&target, &bundle, pin.clone()).map_err(FixtureBundleRefusal::Abi)?;

    Ok(FixtureBundle {
        target,
        plan,
        bundle,
        abi,
        pin,
        closed_asset,
        reserve_asset,
        provenance,
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
            super::fee_program_digest(),
            super::PINNED_PROGRAM,
        ] {
            assert!(seen.insert(constant), "two fixture constants collide");
        }
        assert_ne!(CLOSED_ASSET, RESERVE_ASSET);
        assert_ne!(PINNED_PROGRAM, CLOSED_ASSET);
    }
}
