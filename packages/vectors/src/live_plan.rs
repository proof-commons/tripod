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
    ValidatedLiveTransferOperationPlan, plan_live_transfer_target_operation,
};
use compiler::operation_plan::PlacementSearchLimits;
use linker::live_backend::LiveTransferRepresentationPlan;
use linker::{
    CandidateLinkedLiveTransferBundle, LiveLinkDeploymentParameters, link_live_candidate,
};
use realization::{RealizationScope, derive};
use tapscript::{
    LiveTransferSymbols, OwnerKey, demonstration_live_shape_set, derive_live_receipt_constructor,
    emit_candidate_live_bundle, owner_key_encoding_closure, static_transfer_leaf_set,
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

/// The protocol asset this demonstration deployment resolves.
///
/// A constant, and deliberately not a value a target issued: the linked
/// programs push it as a literal, so a deployment that wanted the
/// target's own asset would have to be linked again after the issuance.
/// A run that does so is a run's business, not this module's.
pub const PROTOCOL_ASSET: [u8; 32] = [0xb1; 32];

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
    let target = reviewed_target()?;
    let plan = live_transfer_plan()?;
    let shapes = demonstration_live_shape_set();
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
            let constructor = derive_live_receipt_constructor(
                &target,
                &plan,
                representation,
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

    let deployment = LiveLinkDeploymentParameters::new(
        &target,
        live_symbols(
            &target,
            PROTOCOL_ASSET.to_vec(),
            vec![0xb2; 32],
            vec![0xb4; 32],
            vec![0xb5; 32],
        )?,
        UNSPENDABLE_INTERNAL_KEY.to_vec(),
        NonZeroU32::new(8).ok_or(VectorError::LiveSubstrateUnavailable)?,
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)?;

    link_live_candidate(&target, &bundles, &deployment)
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
        .get_or_init(|| {
            let target = reviewed_target()?;
            let curve = OracleLiveCurve::new(reviewed_target()?);
            derive_live_transfer_abi(&target, &demonstration_live_bundle()?, &curve)
                .map_err(|_| VectorError::LiveSubstrateUnavailable)
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::{
        FIRST_SCALAR, SECOND_SCALAR, demonstration_live_abi, demonstration_live_bundle,
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
    fn the_two_published_owners_are_distinct() {
        let owners: BTreeSet<_> = [FIRST_SCALAR, SECOND_SCALAR]
            .into_iter()
            .map(|scalar| published_owner(&scalar).expect("a published owner"))
            .collect();
        assert_eq!(owners.len(), 2);
    }
}
