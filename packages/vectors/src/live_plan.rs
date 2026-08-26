//! The live-transfer substrate the safety evidence is stated against.
//!
//! §13.1 derives the canonical evidence plan from seven sources, and
//! three of them — the compiler coverage, the exact linked bundle, and
//! the exact candidate ABI — are artifacts somebody has to build. They
//! were built in a test file until now, which meant the evidence plan
//! could not name them: a substrate reachable only from a test is a
//! substrate nothing outside that test can be evidence about.
//!
//! # This is a demonstration deployment, and says so
//!
//! The owners are the published BIP-340 appendix keys, the protocol asset
//! is a constant of this module, and the internal key is the published
//! unspendable one. None of it is a deployment anybody operates. What it
//! is, is a *complete* candidate: a validated plan, a link over both
//! representation plans, and an ABI whose destination programs are the
//! programs the conformance package's own curve arithmetic determines.
//!
//! # Every secret here is published
//!
//! The signing scalars are the BIP-340 specification's own appendix
//! values, admitted under ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)` and by Guide-13 §1.10. They
//! authorize nothing on any network anyone uses, and no interface here
//! accepts one from a caller.

use std::num::{NonZeroU32, NonZeroU64};
use std::sync::OnceLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::live_transfer_plan::{
    LiveTransferComposition, ValidatedLiveTransferOperationPlan,
    plan_live_transfer_target_operation,
};
use compiler::operation_plan::PlacementSearchLimits;
use linker::live_backend::LiveTransferRepresentationPlan;
use linker::{
    CandidateLinkedLiveTransferBundle, LiveLinkDeploymentParameters, link_live_candidate,
};
use realization::{RealizationScope, derive};
use tapscript::{
    CandidateRelocatableLiveTransferBundle, LiveTransferShapeSet, LiveTransferSymbols, OwnerKey,
    demonstration_live_shape_set, derive_live_receipt_constructor_composing,
    emit_candidate_live_bundle, fee_bearing_live_shape_set, owner_key_encoding_closure,
    static_transfer_leaf_set,
};
use target_elements::{ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript};
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use target_elements_conformance::test_material::OwnerSigningMaterial;
use transaction::live_abi::{CandidateLiveTransferAbi, derive_live_transfer_abi};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;

/// The first published BIP-340 signing scalar.
pub const FIRST_SCALAR: [u8; FIELD_ELEMENT_BYTES] = [
    0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7,
    0x62, 0xE7, 0x16, 0x0F, 0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
];

/// The second published BIP-340 signing scalar.
pub const SECOND_SCALAR: [u8; FIELD_ELEMENT_BYTES] = [
    0xC9, 0x0F, 0xDA, 0xA2, 0x21, 0x68, 0xC2, 0x34, 0xC4, 0xC6, 0x62, 0x8B, 0x80, 0xDC, 0x1C, 0xD1,
    0x29, 0x02, 0x4E, 0x08, 0x8A, 0x67, 0xCC, 0x74, 0x02, 0x0B, 0xBE, 0xA6, 0x3B, 0x14, 0xE5, 0xC9,
];

/// The third published BIP-340 signing scalar.
///
/// Beside the two above and published on the same footing — BIP-340's
/// own appendix vector, a value with no secrecy to lose
/// (`[ADR015-rule:security:test-material]`). It differs from them in
/// PURPOSE rather than in kind: nothing is linked for this owner in the
/// demonstration deployment, and that is exactly what it is for. A
/// discharge asking what happens to a receipt whose owner metadata
/// names somebody the constructor never built for needs an owner the
/// constructor never built for, and reusing a linked one would stage a
/// candidate the recognition accepts.
pub const THIRD_SCALAR: [u8; FIELD_ELEMENT_BYTES] = [
    0x0B, 0x43, 0x2B, 0x26, 0x77, 0x93, 0x73, 0x81, 0xAE, 0xF0, 0x5B, 0xB0, 0x2A, 0x66, 0xEC, 0xD0,
    0x12, 0x77, 0x30, 0x62, 0xCF, 0x3F, 0xA2, 0x54, 0x9E, 0x44, 0xF5, 0x8E, 0xD2, 0x40, 0x17, 0x10,
];

/// The protocol asset this demonstration deployment resolves.
///
/// A constant, and deliberately not a value a target issued: the linked
/// programs push it as a literal, so a deployment that wanted the
/// target's own asset would have to be linked again after the issuance.
/// A run that does so is a run's business, not this module's.
pub const PROTOCOL_ASSET: [u8; 32] = [0xb1; 32];

/// The reserve asset this demonstration deployment resolves.
///
/// The sponsor and fee roles' asset, and a constant for exactly the
/// reason [`PROTOCOL_ASSET`] is one: §10.7's isolation fragments push it
/// as a literal, so it is welded into the leaves and therefore into
/// every destination program the taptree commits.
///
/// It is separated from the protocol asset here because the two are not
/// knowable at the same time. A run learns the protocol asset from the
/// issuance it asked for, and learns the reserve from the answer to a
/// sponsor-funding request it deliberately sends without naming an
/// asset — what a development network uses as its reserve being the
/// network's own fact. A deployment is welded to BOTH, and two reserves
/// are two deployments exactly as two protocol assets are.
pub const RESERVE_ASSET: [u8; 32] = [0xb2; 32];

/// The fee-role program digest this demonstration deployment resolves.
///
/// §10.7's sponsor isolation ends by inspecting the fee output's
/// scriptPubKey and requiring its digest to equal this symbol, so like
/// both assets above it is pushed as a literal by the isolation
/// fragments and welded into the leaves.
///
/// It is a parameter for a DIFFERENT reason than the assets are, and the
/// difference is worth stating plainly. A run cannot know either asset
/// in advance: the protocol asset is whatever the issuance created and
/// the reserve is whatever the network answers a funding request in,
/// both learned from a chain. This digest is learned from nothing — the
/// fee role's program is target-structural, construction writes it, and
/// its digest is therefore computable before any node is started. It is
/// threaded anyway because a deployment is welded to its symbols and
/// this constant is welded into the demonstration's: the value here
/// hashes to no program at all, and moving it to the real digest would
/// move the demonstration's committed taptree and with it every live
/// run-of-record identity the plans cite. So the demonstration keeps
/// the constant it was linked with, and a deployment that actually
/// intends to spend a sponsored control supplies the digest of the fee
/// program IT constructs.
pub const FEE_PROGRAM_DIGEST: [u8; 32] = [0xb5; 32];

/// The reviewed contract, unmodified.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the reviewed contract
/// does not validate, which is a defect in the target definition and not
/// in anything this module states.
pub fn reviewed_target() -> Result<ReviewedElementsTapscriptDefinition, VectorError> {
    reviewed_elements_tapscript().map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// One published scalar's signing material.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the scalar is out of
/// range, which the two published constants are not.
pub fn signing_material(
    scalar: &[u8; FIELD_ELEMENT_BYTES],
) -> Result<OwnerSigningMaterial, VectorError> {
    OwnerSigningMaterial::from_published_scalar(scalar)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// One owner's canonical metadata from a published public key.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the bytes are not the
/// approved encoding at its exact width.
pub fn owner_key(bytes: &[u8]) -> Result<OwnerKey, VectorError> {
    let target = reviewed_target()?;
    let closure = owner_key_encoding_closure(target.definition().authorization());
    OwnerKey::new(&closure, closure.approved(), bytes.to_vec())
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// One published scalar's owner metadata.
///
/// # Errors
///
/// Whatever [`owner_key`] and [`signing_material`] refuse.
pub fn published_owner(scalar: &[u8; FIELD_ELEMENT_BYTES]) -> Result<OwnerKey, VectorError> {
    owner_key(&signing_material(scalar)?.x_only_public_key())
}

/// The validated live-transfer operation plan.
///
/// The first of §13.1's seven sources, and the one every other source is
/// checked against: the linked bundle is emitted for it, the ABI is
/// derived from that link, and every §15 row that names a relation
/// resolves against the coverage this plan publishes.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the realization, the
/// scope, the bound input, or the plan itself does not validate.
pub fn live_transfer_plan() -> Result<ValidatedLiveTransferOperationPlan, VectorError> {
    static CACHED: OnceLock<Result<ValidatedLiveTransferOperationPlan, VectorError>> =
        OnceLock::new();
    CACHED.get_or_init(derive_live_transfer_plan).clone()
}

/// The plan derivation proper, run once behind the cache.
fn derive_live_transfer_plan() -> Result<ValidatedLiveTransferOperationPlan, VectorError> {
    let limit = |value: u64| NonZeroU64::new(value).ok_or(VectorError::LiveSubstrateUnavailable);
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let scope = CompilationScope::from_operations([OperationId::TransferLive])
        .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let policy = AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000)?, limit(10_000)?));
    let input = bind_input(&ARCHITECTURE, realization, scope, policy)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)?;

    plan_live_transfer_target_operation(
        &input,
        PlacementSearchLimits::new(limit(10_000_000)?, limit(1_000_000)?),
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// The live symbols one set of values resolves.
fn live_symbols(
    target: &ReviewedElementsTapscriptDefinition,
    protocol_asset: Vec<u8>,
    reserve_asset: Vec<u8>,
    sponsor_change: Vec<u8>,
    fee_digest: Vec<u8>,
) -> Result<LiveTransferSymbols, VectorError> {
    LiveTransferSymbols::new(
        target,
        protocol_asset,
        reserve_asset,
        1,
        sponsor_change,
        0,
        fee_digest,
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// The demonstration link, over two published owners and both plans.
///
/// The second of §13.1's seven sources. Four constructors — two owners
/// times two representation plans — emitted, linked, and placed by the
/// deterministic taptree.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a constructor, an
/// emission, the deployment parameters, or the link refuses.
pub fn demonstration_live_bundle() -> Result<CandidateLinkedLiveTransferBundle, VectorError> {
    static CACHED: OnceLock<Result<CandidateLinkedLiveTransferBundle, VectorError>> =
        OnceLock::new();
    CACHED.get_or_init(link_demonstration_bundle).clone()
}

/// The link proper, run once behind the cache.
fn link_demonstration_bundle() -> Result<CandidateLinkedLiveTransferBundle, VectorError> {
    link_live_bundle_for_asset(PROTOCOL_ASSET, RESERVE_ASSET, FEE_PROGRAM_DIGEST)
}

/// The demonstration link, over one stated pair of assets.
///
/// The constant-asset link above is this function at [`PROTOCOL_ASSET`]
/// and [`RESERVE_ASSET`], and the parameters exist for one reason: a
/// target-native run does not get to choose either asset. §14.3's
/// materialization funds real coins, and on a disposable chain the
/// protocol asset is whatever the issuance step created while the
/// reserve is whatever the network already pays in — values the run
/// learns *after* it starts, from two different answers. The linked
/// programs push both as literals, so a run against target-supplied
/// assets has to link again once it knows them, and the constructors,
/// their committed trees, and therefore the destination programs all
/// move with them.
///
/// That is a fact about the deployment rather than a workaround: a
/// deployment is welded to its assets, and two pairs are two
/// deployments. The reserve's own arrival is the sharper case, because
/// nothing asks for it: the sponsor-funding request deliberately names
/// no asset, and the executor reports which one it funded in.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a constructor, an
/// emission, the deployment parameters, or the link refuses.
pub fn link_live_bundle_for_asset(
    protocol_asset: [u8; 32],
    reserve_asset: [u8; 32],
    fee_program_digest: [u8; 32],
) -> Result<CandidateLinkedLiveTransferBundle, VectorError> {
    link_live_bundle_for_vocabulary(
        LiveShapeVocabulary::Demonstration,
        protocol_asset,
        reserve_asset,
        fee_program_digest,
    )
}

/// The linked bundle of one stated vocabulary over one stated pair of
/// assets.
///
/// The deployment parameters are the same for both vocabularies — same
/// symbols, same internal key, same depth — and only the shape set
/// differs. That is the point rather than an economy: the fee-bearing
/// candidate is the demonstration candidate plus one member, so anything
/// that moves between them moved because of the member.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a constructor, an
/// emission, the deployment parameters, or the link refuses.
pub fn link_live_bundle_for_vocabulary(
    vocabulary: LiveShapeVocabulary,
    protocol_asset: [u8; 32],
    reserve_asset: [u8; 32],
    fee_program_digest: [u8; 32],
) -> Result<CandidateLinkedLiveTransferBundle, VectorError> {
    link_live_bundle_composing(
        vocabulary,
        LiveTransferComposition::HomogeneousExplicit,
        protocol_asset,
        reserve_asset,
        fee_program_digest,
    )
}

/// The linked bundle of one vocabulary and one seated composition.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a constructor, an
/// emission, the deployment parameters, or the link refuses.
pub fn link_live_bundle_composing(
    vocabulary: LiveShapeVocabulary,
    composition: LiveTransferComposition,
    protocol_asset: [u8; 32],
    reserve_asset: [u8; 32],
    fee_program_digest: [u8; 32],
) -> Result<CandidateLinkedLiveTransferBundle, VectorError> {
    let target = reviewed_target()?;
    let bundles = relocatable_live_bundles_composing(vocabulary, composition)?;
    let deployment = live_deployment_for_asset(protocol_asset, reserve_asset, fee_program_digest)?;
    link_live_candidate(&target, &bundles, &deployment)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// The four relocatable bundles the demonstration link is taken over.
///
/// Two published owners times both representation plans, emitted against
/// placeholder symbols and not yet resolved to any deployment. Exposed
/// because they are the linker's own *input*: the §15.7 linker rows are
/// discharged by handing this list, or a single change of it, back to the
/// entry point that consumes it, and a discharge built from a bundle
/// nothing else uses would be evidence about a private fixture rather
/// than about the deployment.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a constructor or an
/// emission refuses.
pub fn relocatable_live_bundles() -> Result<Vec<CandidateRelocatableLiveTransferBundle>, VectorError>
{
    relocatable_live_bundles_for(LiveShapeVocabulary::Demonstration)
}

/// Which live-transfer shape vocabulary a deployment emits programs for.
///
/// Two candidates, not one candidate with a switch. The whole reason the
/// fee-bearing form is a second vocabulary rather than a widening of the
/// first is that a candidate's shape set becomes one coordinator leaf per
/// shape, those leaves tweak the taproot output key, and that key is the
/// destination program every recorded fixture digest was taken over. A
/// ceremony that does not intend a fee-bearing transfer therefore keeps
/// the demonstration vocabulary and keeps its digests, byte for byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveShapeVocabulary {
    /// The Phase-5 demonstration set, with no sponsorless fee-bearing
    /// member.
    Demonstration,
    /// The same counts, also emitting the sponsorless form that pays its
    /// own fee.
    FeeBearing,
}

impl LiveShapeVocabulary {
    /// The shape set this vocabulary names.
    #[must_use]
    pub fn shape_set(self) -> LiveTransferShapeSet {
        match self {
            Self::Demonstration => demonstration_live_shape_set(),
            Self::FeeBearing => fee_bearing_live_shape_set(),
        }
    }
}

/// The relocatable bundles of one stated vocabulary.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a constructor or an
/// emission refuses.
pub fn relocatable_live_bundles_for(
    vocabulary: LiveShapeVocabulary,
) -> Result<Vec<CandidateRelocatableLiveTransferBundle>, VectorError> {
    relocatable_live_bundles_composing(vocabulary, LiveTransferComposition::HomogeneousExplicit)
}

/// The relocatable bundles of one vocabulary, one of whose keys a stated
/// CROSSING composition substitutes at.
///
/// # A crossing deployment substitutes; it does not widen
///
/// A deployment's destination table is keyed by the plan a coin is
/// RECOGNIZED under, and a crossing constructor recognizes coins of its
/// consumed side. So a crossing deployment is the homogeneous one with
/// exactly one of its two keys' constructors replaced — the consumed
/// side's — and the other key left holding the ordinary homogeneous
/// constructor, which is what the crossing transfer's own destinations
/// are then paid to. No key is added, no table widens, and a coin
/// created by a crossing transfer is spendable by an ordinary
/// homogeneous one afterwards, which is the property that makes the
/// crossing a step in a lifecycle rather than a cul-de-sac.
///
/// A homogeneous composition substitutes nothing and reproduces
/// [`relocatable_live_bundles_for`] exactly, which is why that function
/// is this one rather than a sibling of it.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a constructor or an
/// emission refuses.
pub fn relocatable_live_bundles_composing(
    vocabulary: LiveShapeVocabulary,
    composition: LiveTransferComposition,
) -> Result<Vec<CandidateRelocatableLiveTransferBundle>, VectorError> {
    let target = reviewed_target()?;
    let plan = live_transfer_plan()?;
    let shapes = vocabulary.shape_set();
    let placeholders = live_symbols(
        &target,
        vec![0x5a; 32],
        vec![0x22; 32],
        vec![0x44; 20],
        vec![0x55; 32],
    )?;

    let mut bundles = Vec::with_capacity(4);
    for scalar in [FIRST_SCALAR, SECOND_SCALAR] {
        for representation in [
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ] {
            // The crossing constructor stands at its CONSUMED side's
            // key and nowhere else. Every other key keeps the
            // homogeneous constructor it always held, so a crossing
            // deployment differs from the demonstration one at exactly
            // one of its four bundles per owner-and-plan pair.
            let seated = if composition.crosses() && representation == composition.consumed() {
                composition
            } else {
                LiveTransferComposition::homogeneous(representation)
            };
            let constructor = derive_live_receipt_constructor_composing(
                &target,
                &plan,
                seated,
                published_owner(&scalar)?,
                shapes.clone(),
                static_transfer_leaf_set(representation, &shapes),
            )
            .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
            bundles.push(
                emit_candidate_live_bundle(&target, &plan, &constructor, placeholders.clone())
                    .map_err(|_| VectorError::LiveSubstrateUnavailable)?,
            );
        }
    }
    Ok(bundles)
}

/// The deployment parameters the demonstration link resolves against.
///
/// The counterpart of [`relocatable_live_bundles`], and exposed for the
/// same reason: a symbol census collected against a deployment nobody
/// else uses would not be the census the deployment's own link builds.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the symbols or the
/// parameters refuse.
pub fn live_deployment_for_asset(
    protocol_asset: [u8; 32],
    reserve_asset: [u8; 32],
    fee_program_digest: [u8; 32],
) -> Result<LiveLinkDeploymentParameters, VectorError> {
    let target = reviewed_target()?;
    LiveLinkDeploymentParameters::new(
        &target,
        live_symbols(
            &target,
            protocol_asset.to_vec(),
            reserve_asset.to_vec(),
            vec![0xb4; 32],
            fee_program_digest.to_vec(),
        )?,
        UNSPENDABLE_INTERNAL_KEY.to_vec(),
        NonZeroU32::new(8).ok_or(VectorError::LiveSubstrateUnavailable)?,
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// The candidate ABI, derived through the oracle's own arithmetic.
///
/// The third of §13.1's seven sources. The curve capability is the
/// conformance package's point arithmetic, so the destination programs
/// are the ones the deployment's constructors determine rather than the
/// ones the builder hoped they would be.
///
/// # The derivation is memoized, and that changes nothing about it
///
/// The three artifacts are pure functions of constants in this file, so
/// two calls cannot differ; deriving them again per caller cost tens of
/// seconds across an evidence census that asks for the ABI once per
/// staged case. The cache holds the *result*, refusals included, so a
/// substrate that failed to build keeps failing rather than being retried
/// into a different answer.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the link or the ABI
/// derivation refuses.
pub fn demonstration_live_abi() -> Result<CandidateLiveTransferAbi, VectorError> {
    static CACHED: OnceLock<Result<CandidateLiveTransferAbi, VectorError>> = OnceLock::new();
    CACHED
        .get_or_init(|| live_abi_for_asset(PROTOCOL_ASSET, RESERVE_ASSET, FEE_PROGRAM_DIGEST))
        .clone()
}

/// The candidate ABI over one stated pair of assets.
///
/// The run-time counterpart of [`demonstration_live_abi`], for the reason
/// [`link_live_bundle_for_asset`] states. Deliberately not memoized: a
/// run links once for the pair it was given, and a cache keyed by
/// nothing would hand the second pair the first one's programs.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the link or the ABI
/// derivation refuses.
pub fn live_abi_for_asset(
    protocol_asset: [u8; 32],
    reserve_asset: [u8; 32],
    fee_program_digest: [u8; 32],
) -> Result<CandidateLiveTransferAbi, VectorError> {
    live_abi_for_vocabulary(
        LiveShapeVocabulary::Demonstration,
        protocol_asset,
        reserve_asset,
        fee_program_digest,
    )
}

/// The candidate ABI of one stated vocabulary over one stated pair of
/// assets.
///
/// Not memoized, for the reason [`live_abi_for_asset`] gives and one
/// more: the vocabulary is a second key, and a cache holding one of the
/// two candidates would hand a fee-bearing ceremony the demonstration
/// candidate's programs — which is exactly the confusion the two
/// vocabularies exist to prevent.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the link or the ABI
/// derivation refuses.
pub fn live_abi_for_vocabulary(
    vocabulary: LiveShapeVocabulary,
    protocol_asset: [u8; 32],
    reserve_asset: [u8; 32],
    fee_program_digest: [u8; 32],
) -> Result<CandidateLiveTransferAbi, VectorError> {
    live_abi_composing(
        vocabulary,
        LiveTransferComposition::HomogeneousExplicit,
        protocol_asset,
        reserve_asset,
        fee_program_digest,
    )
}

/// The candidate ABI of one vocabulary and one seated composition.
///
/// A crossing composition seats its crossing constructor at the key its
/// consumed side is recognized under, so this ABI's destination table
/// holds the crossing constructor at one key and the ordinary
/// homogeneous one at the other. Both are needed and for different
/// reasons: the crossing key is what a spent receipt resolves through,
/// and the other is what this transfer's own destinations are paid to.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when the link or the ABI
/// derivation refuses.
pub fn live_abi_composing(
    vocabulary: LiveShapeVocabulary,
    composition: LiveTransferComposition,
    protocol_asset: [u8; 32],
    reserve_asset: [u8; 32],
    fee_program_digest: [u8; 32],
) -> Result<CandidateLiveTransferAbi, VectorError> {
    let target = reviewed_target()?;
    let curve = OracleLiveCurve::new(reviewed_target()?);
    derive_live_transfer_abi(
        &target,
        &link_live_bundle_composing(
            vocabulary,
            composition,
            protocol_asset,
            reserve_asset,
            fee_program_digest,
        )?,
        &curve,
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

#[cfg(test)]
mod tests {
    use super::{
        FEE_PROGRAM_DIGEST, FIRST_SCALAR, LiveShapeVocabulary, PROTOCOL_ASSET, RESERVE_ASSET,
        SECOND_SCALAR, demonstration_live_abi, demonstration_live_bundle, live_abi_for_vocabulary,
        live_transfer_plan, published_owner,
    };
    use std::collections::BTreeSet;

    #[test]
    fn the_three_artifact_sources_build() {
        // §13.1 names seven sources and three of them are artifacts. A
        // plan that stopped deriving, a link that stopped linking, or an
        // ABI that stopped resolving would leave the evidence plan
        // stating requirements about nothing.
        let plan = live_transfer_plan().expect("the live plan validates");
        assert_eq!(plan.operation(), architecture::OperationId::TransferLive);
        let bundle = demonstration_live_bundle().expect("the demonstration links");
        assert_ne!(bundle.constructors().len(), 0);
        let abi = demonstration_live_abi().expect("the ABI derives");
        assert_eq!(abi.destinations().entries().len(), 4);
    }

    #[test]
    fn the_fee_bearing_deployment_carries_the_shape_a_self_paid_fee_needs() {
        // The construction path is only reachable with a node, so the
        // thing that would send a ceremony to a real target to learn
        // `UnsupportedLiveShape` is asked here instead: does a candidate
        // exist for one receipt in, one receipt out, no sponsor, and a
        // fee? Selection matches on exactly these terms, so a shape
        // answering them is what stands between the ceremony and a
        // typed stop that costs a node run to observe.
        let abi = live_abi_for_vocabulary(
            LiveShapeVocabulary::FeeBearing,
            PROTOCOL_ASSET,
            RESERVE_ASSET,
            FEE_PROGRAM_DIGEST,
        )
        .expect("the fee-bearing ABI derives");

        let selected = abi
            .shapes()
            .values()
            .find(|candidate| {
                let shape = candidate.shape();
                shape.receipt_inputs() == 1
                    && shape.receipt_outputs() == 1
                    && shape.sponsor_inputs() == 0
                    && candidate.sponsor_change_position().is_none()
                    && candidate.fee_position().is_some()
            })
            .expect("the fee-bearing candidate emits a sponsorless one-to-one shape with a fee");

        // The fee sits immediately after the single destination, and the
        // exact output count leaves no position over. Both are what the
        // covenant's own family census asserts, so a disagreement here
        // is a disagreement the emitted program would have carried.
        assert_eq!(selected.fee_position(), Some(1));
        assert_eq!(selected.shape().outputs(), 2);
        assert_eq!(selected.shape().inputs(), 1);
    }

    #[test]
    fn the_demonstration_deployment_still_offers_no_such_shape() {
        // The converse, and the reason the two vocabularies are separate
        // deployments rather than one widened set: nothing that links
        // against the demonstration candidate can accidentally select a
        // fee-bearing shape, so no existing ceremony changes what it
        // builds.
        let abi = demonstration_live_abi().expect("the ABI derives");

        assert!(
            !abi.shapes().values().any(|candidate| {
                candidate.shape().sponsor_inputs() == 0 && candidate.fee_position().is_some()
            }),
            "the demonstration deployment gained a sponsorless fee-bearing shape"
        );
    }

    #[test]
    fn the_two_published_owners_are_distinct() {
        let owners: BTreeSet<_> = [FIRST_SCALAR, SECOND_SCALAR]
            .into_iter()
            .map(|scalar| published_owner(&scalar).expect("a published owner"))
            .collect();
        assert_eq!(owners.len(), 2);
    }
}
