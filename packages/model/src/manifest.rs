//! Typed architecture-conformance contract.
//!
//! Implements `(´rule:verification:no-accumulator-manifest´)` and
//! `(´test:verification:no-accumulator-branches´)` (world check) against
//! the typed manifest in `tripod-architecture`.
//!
//! The normative architecture is the typed Rust declaration in the
//! `tripod-architecture` crate; `architecture.json` and
//! `architecture.toml` are generated derivatives. One may assume
//! naively that a stringly typed manifest view with substring checks
//! such as `contains("ATTESTATION_ROOT")` suffices, but that reproduces
//! exactly the drift the manifest exists to prevent; this module
//! instead maps the model's identifier enums onto the architecture's
//! stable IDs with exhaustive matches, so renames on either side break
//! conformance at compile time, and checks the hand-written policy
//! tables against the typed declarations.
//!
//! One may also assume naively that the exhaustive matches witness the
//! *correctness* of each pairing, but they only witness completeness: a
//! consistent transposition (mapping the model's `Burn` onto the
//! architecture's `redeem` in both directions) round-trips and passes
//! every match. The pairing is therefore welded twice more: each model
//! enum carries the architecture's stable numeric code as its
//! `repr(u16)` discriminant and compile-time assertions check every map
//! code-for-code, and `validate_architecture_conformance` checks every
//! pairing name-for-name, so the correspondence "the model's `Burn` is
//! the architecture's `burn`" is a structural fact of this module, not
//! a behavioral consequence of the downstream operations suite.

use std::collections::BTreeSet;

use architecture::ARCHITECTURE;

use crate::asset::Asset;
use crate::constants::Constants;
use crate::guard::Guard;
use crate::history::{BranchKind, DeltaKind, OpenFlowKind, RootEdge, TransitionCertificate};
use crate::object::{Meta, Tag};
use crate::policy::{RootUse, ValueFlowClass};
use crate::pool::PoolState;
use crate::scalar::Sat;
use crate::shape::ObjectKind;
use crate::world::World;

// Identifier correspondences. Each match is exhaustive over the model
// enum, and the reverse operation map is exhaustive over the
// architecture enum, so adding or renaming a variant on either side is
// a compile-time conformance failure. Exhaustiveness witnesses
// completeness only; the correctness of each pairing is welded by the
// shared numeric codes (compile-time, below) and by variant-name
// conformance (`validate_identifier_name_conformance`).

pub const fn branch_operation(branch: BranchKind) -> architecture::OperationId {
    match branch {
        BranchKind::CreateRequest => architecture::OperationId::CreateRequest,
        BranchKind::CancelRequest => architecture::OperationId::CancelRequest,
        BranchKind::AdmitDeposits => architecture::OperationId::AdmitDeposits,
        BranchKind::Cycle => architecture::OperationId::Cycle,
        BranchKind::SettleDistribution => architecture::OperationId::SettleDistribution,
        BranchKind::TransferLive => architecture::OperationId::TransferLive,
        BranchKind::TransferTimeLocked => architecture::OperationId::TransferTimeLocked,
        BranchKind::Redeem => architecture::OperationId::Redeem,
        BranchKind::ReceiptRelabel => architecture::OperationId::ReceiptRelabel,
        BranchKind::Burn => architecture::OperationId::Burn,
        BranchKind::CompactAsh => architecture::OperationId::CompactAsh,
        BranchKind::Clear => architecture::OperationId::Clear,
        BranchKind::AnnounceMaturity => architecture::OperationId::AnnounceMaturity,
    }
}

pub const fn operation_branch(operation: architecture::OperationId) -> BranchKind {
    match operation {
        architecture::OperationId::CreateRequest => BranchKind::CreateRequest,
        architecture::OperationId::CancelRequest => BranchKind::CancelRequest,
        architecture::OperationId::AdmitDeposits => BranchKind::AdmitDeposits,
        architecture::OperationId::Cycle => BranchKind::Cycle,
        architecture::OperationId::SettleDistribution => BranchKind::SettleDistribution,
        architecture::OperationId::TransferLive => BranchKind::TransferLive,
        architecture::OperationId::TransferTimeLocked => BranchKind::TransferTimeLocked,
        architecture::OperationId::Redeem => BranchKind::Redeem,
        architecture::OperationId::ReceiptRelabel => BranchKind::ReceiptRelabel,
        architecture::OperationId::Burn => BranchKind::Burn,
        architecture::OperationId::CompactAsh => BranchKind::CompactAsh,
        architecture::OperationId::Clear => BranchKind::Clear,
        architecture::OperationId::AnnounceMaturity => BranchKind::AnnounceMaturity,
    }
}

pub fn declared_asset(asset: Asset) -> Option<architecture::AssetId> {
    match asset {
        Asset::Lbtc => Some(architecture::AssetId::Lbtc),
        Asset::U => Some(architecture::AssetId::U),
        Asset::Ent => Some(architecture::AssetId::Ent),
        Asset::DistCtl => Some(architecture::AssetId::DistCtl),
        Asset::Pid => Some(architecture::AssetId::Pid),
        Asset::Pace => Some(architecture::AssetId::Pace),
        Asset::EntAuth => Some(architecture::AssetId::EntAuth),
        Asset::DistAuth => Some(architecture::AssetId::DistAuth),
        Asset::Foreign(_) => None,
    }
}

pub fn asset_of(declared: architecture::AssetId) -> Asset {
    match declared {
        architecture::AssetId::Lbtc => Asset::Lbtc,
        architecture::AssetId::U => Asset::U,
        architecture::AssetId::Ent => Asset::Ent,
        architecture::AssetId::DistCtl => Asset::DistCtl,
        architecture::AssetId::Pid => Asset::Pid,
        architecture::AssetId::Pace => Asset::Pace,
        architecture::AssetId::EntAuth => Asset::EntAuth,
        architecture::AssetId::DistAuth => Asset::DistAuth,
    }
}

/// The declared operation specification for a branch. Every branch
/// kind corresponds to exactly one declared operation; the typed
/// manifest validates complete operation coverage.
pub fn operation_spec(branch: BranchKind) -> &'static architecture::OperationSpec {
    ARCHITECTURE
        .operation(branch_operation(branch))
        .expect("every branch kind has a declared operation")
}

/// Transaction-shape classifier for each declared protocol object.
pub const fn object_kind_of(declared: architecture::ObjectId) -> ObjectKind {
    match declared {
        architecture::ObjectId::State => ObjectKind::State,
        architecture::ObjectId::Resv => ObjectKind::Resv,
        architecture::ObjectId::Pace => ObjectKind::Pace,
        architecture::ObjectId::EntitlementAuthority => ObjectKind::EntitlementAuthority,
        architecture::ObjectId::DistributionAuthority => ObjectKind::DistributionAuthority,
        architecture::ObjectId::ReceiptLive => ObjectKind::ReceiptLive,
        architecture::ObjectId::ReceiptTimeLocked => ObjectKind::ReceiptTimeLocked,
        architecture::ObjectId::DepositRequest => ObjectKind::DepositRequest,
        architecture::ObjectId::DepositEntitlement => ObjectKind::DepositEntitlement,
        architecture::ObjectId::DistributionControl => ObjectKind::DistributionControl,
        architecture::ObjectId::DistributionVault => ObjectKind::DistributionVault,
        architecture::ObjectId::Ash => ObjectKind::Ash,
        architecture::ObjectId::PlainLbtc => ObjectKind::PlainLbtc,
        architecture::ObjectId::CpfpAnchor => ObjectKind::CpfpAnchor,
    }
}

pub const fn tag_of(declared: architecture::TagId) -> Tag {
    match declared {
        architecture::TagId::Burn => Tag::Burn,
        architecture::TagId::Recon => Tag::Recon,
        architecture::TagId::Redeem => Tag::Redeem,
        architecture::TagId::Entitlement => Tag::Entitlement,
        architecture::TagId::DistributionControlClose => Tag::DistributionControlClose,
        architecture::TagId::DistributionResidue => Tag::DistributionResidue,
    }
}

pub const fn delta_kind_of(declared: architecture::DeltaKind) -> DeltaKind {
    match declared {
        architecture::DeltaKind::Issuance => DeltaKind::Issuance,
        architecture::DeltaKind::Destruction => DeltaKind::Destruction,
        architecture::DeltaKind::Lateral => DeltaKind::Lateral,
        architecture::DeltaKind::OwnerlessLateral => DeltaKind::OwnerlessLateral,
    }
}

pub(crate) const fn root_use_of(declared: architecture::RootUse) -> RootUse {
    match declared {
        architecture::RootUse::Forbidden => RootUse::Forbidden,
        architecture::RootUse::Succession => RootUse::Succession,
        architecture::RootUse::SuccessionOrTermination => RootUse::SuccessionOrTermination,
    }
}

pub(crate) const fn value_flow_of(declared: architecture::ValueFlowClass) -> ValueFlowClass {
    match declared {
        architecture::ValueFlowClass::OwnerConsented => ValueFlowClass::OwnerConsented,
        architecture::ValueFlowClass::ImmutableDestination => ValueFlowClass::ImmutableDestination,
        architecture::ValueFlowClass::FormulaBoundPayout => ValueFlowClass::FormulaBoundPayout,
        architecture::ValueFlowClass::PreauthorizedServiceBudget => {
            ValueFlowClass::PreauthorizedServiceBudget
        }
        architecture::ValueFlowClass::OwnerlessTerminalSink => {
            ValueFlowClass::OwnerlessTerminalSink
        }
        architecture::ValueFlowClass::SponsorEnvelope => ValueFlowClass::SponsorEnvelope,
        architecture::ValueFlowClass::OwnerlessBoundSink => ValueFlowClass::OwnerlessBoundSink,
        architecture::ValueFlowClass::OwnerlessBoundMovement => {
            ValueFlowClass::OwnerlessBoundMovement
        }
    }
}

pub(crate) const fn open_flow_of(declared: architecture::OpenFlowKind) -> OpenFlowKind {
    match declared {
        architecture::OpenFlowKind::RequestCreation => OpenFlowKind::RequestCreation,
        architecture::OpenFlowKind::RequestRefund => OpenFlowKind::RequestRefund,
        architecture::OpenFlowKind::DepositAdmission => OpenFlowKind::DepositAdmission,
        architecture::OpenFlowKind::ReserveCarry => OpenFlowKind::ReserveCarry,
        architecture::OpenFlowKind::Redemption => OpenFlowKind::Redemption,
        architecture::OpenFlowKind::FeeSponsor => OpenFlowKind::FeeSponsor,
    }
}
// ´rule:verification:identifier-weld´
//
// Numeric weld of the identifier correspondences. Each fieldless model
// enum carries the architecture's stable code as its `repr(u16)`
// discriminant, and these assertions check every pairing
// code-for-code over the architecture's complete variant lists (the
// `ALL` slices are generated by the architecture's id macro, so they
// cannot omit a variant). Transposing a pair in a match arm, or
// mistyping a discriminant, is a build failure here, not a behavioral
// failure in a downstream operations test. `Asset` carries a payload
// variant and cannot be cast; its pairing is welded by name in
// `validate_identifier_name_conformance`, which also welds the
// discriminants themselves to the compiler-derived variant names.
const _: () = {
    let mut index = 0;
    while index < architecture::OperationId::ALL.len() {
        let operation = architecture::OperationId::ALL[index];
        assert!(operation_branch(operation) as u16 == operation.code());
        assert!(branch_operation(operation_branch(operation)).code() == operation.code());
        index += 1;
    }

    let mut index = 0;
    while index < architecture::ObjectId::ALL.len() {
        let object = architecture::ObjectId::ALL[index];
        assert!(object_kind_of(object) as u16 == object.code());
        index += 1;
    }

    let mut index = 0;
    while index < architecture::TagId::ALL.len() {
        let tag = architecture::TagId::ALL[index];
        assert!(tag_of(tag) as u16 == tag.code());
        index += 1;
    }

    let mut index = 0;
    while index < architecture::DeltaKind::ALL.len() {
        let kind = architecture::DeltaKind::ALL[index];
        assert!(delta_kind_of(kind) as u16 == kind.code());
        index += 1;
    }

    let mut index = 0;
    while index < architecture::RootUse::ALL.len() {
        let root_use = architecture::RootUse::ALL[index];
        assert!(root_use_of(root_use) as u16 == root_use.code());
        index += 1;
    }

    let mut index = 0;
    while index < architecture::ValueFlowClass::ALL.len() {
        let class = architecture::ValueFlowClass::ALL[index];
        assert!(value_flow_of(class) as u16 == class.code());
        index += 1;
    }

    let mut index = 0;
    while index < architecture::OpenFlowKind::ALL.len() {
        let kind = architecture::OpenFlowKind::ALL[index];
        assert!(open_flow_of(kind) as u16 == kind.code());
        index += 1;
    }
};
/// Calibrated runtime value for a declared finite bound.
pub fn bound_value(constants: &Constants, bound: architecture::BoundId) -> usize {
    match bound {
        architecture::BoundId::AdmissionBatchMax => constants.admission_batch_max,
        architecture::BoundId::SettlementBatchMax => constants.settlement_batch_max,
        architecture::BoundId::RelabelBatchMax => constants.relabel_batch_max,
        architecture::BoundId::AshBatchMax => constants.ash_batch_max,
        architecture::BoundId::BurnInputMax => constants.burn_input_max,
        architecture::BoundId::BurnChangeMax => constants.burn_change_max,
        architecture::BoundId::BurnRecordMax => constants.burn_record_max,
        architecture::BoundId::TransferInputMax => constants.transfer_input_max,
        architecture::BoundId::TransferOutputMax => constants.transfer_output_max,
        architecture::BoundId::FeeSponsorInputMax => constants.fee_sponsor_input_max,
    }
}

// ´rule:verification:no-accumulator-manifest´

/// Validates the typed architecture manifest and checks the
/// implementation's hand-written policy tables against it.
pub fn validate_architecture_conformance() -> Result<(), Guard> {
    if architecture::validate_draft(&ARCHITECTURE).is_err() {
        return Err(Guard::BadAuthorization);
    }

    // The typed root set structurally excludes an attestation
    // accumulator: an attestation-root role is not representable, the
    // declared set is exactly the expected roots, and the
    // no-accumulator decision is closed.
    if ARCHITECTURE.roots.len() != architecture::RootId::ALL.len() {
        return Err(Guard::BadAuthorization);
    }

    for expected in architecture::RootId::ALL {
        if ARCHITECTURE.root(*expected).is_none() {
            return Err(Guard::BadAuthorization);
        }
    }

    if !ARCHITECTURE.decisions.iter().any(|decision| {
        decision.id == architecture::DecisionId::NoOnchainAttestationAccumulator
            && decision.status == architecture::DecisionStatus::Closed
    }) {
        return Err(Guard::BadAuthorization);
    }

    validate_residue_reader_conformance()?;
    validate_asset_authority_conformance()?;
    validate_identifier_name_conformance()?;

    // Branch root/projection policy, shape cardinalities, open-flow
    // allow-lists, value-flow classes, and data-output families are
    // derived directly from the typed manifest (`policy::branch_policy`,
    // `shape::shape_policy`, `kernel::branch_allows_open_flow`,
    // `policy::expected_value_flow_classes`), so no hand-written policy
    // tables remain to compare here.

    Ok(())
}

/// Historical residue is audit-only: no covenant operation, the
/// attestation indexer, and no consumer formula may read it.
fn validate_residue_reader_conformance() -> Result<(), Guard> {
    for quantity in [
        architecture::QuantityId::HistoricalLiveResidue,
        architecture::QuantityId::HistoricalTimeLockedResidue,
    ] {
        let spec = ARCHITECTURE.quantity(quantity).ok_or(Guard::NoSuch)?;

        if spec.allows(architecture::ReaderId::AttestationIndexer)
            || spec.allows(architecture::ReaderId::ConsumerFormula)
        {
            return Err(Guard::BadAuthorization);
        }

        for operation in architecture::OperationId::ALL {
            if spec.allows(architecture::ReaderId::Operation(*operation)) {
                return Err(Guard::BadAuthorization);
            }
        }

        if !spec.allows(architecture::ReaderId::InvariantChecker)
            || !spec.allows(architecture::ReaderId::ExternalAuditor)
        {
            return Err(Guard::BadAuthorization);
        }
    }

    Ok(())
}

fn validate_asset_authority_conformance() -> Result<(), Guard> {
    for declared in architecture::AssetId::ALL {
        let spec = ARCHITECTURE.asset(*declared).ok_or(Guard::NoSuch)?;

        let expected = spec.authority.map(asset_of);

        if asset_of(*declared).authority() != expected {
            return Err(Guard::MissingAuthority);
        }
    }

    Ok(())
}

/// Name weld of the identifier correspondences: every map must pair
/// variants of equal (compiler-derived) name. Together with the
/// numeric weld this closes the transposition surface entirely — a
/// consistent transposition would now have to swap the model's variant
/// names themselves, which is a rename visible at every use site, not
/// a silent mistranscription.
fn validate_identifier_name_conformance() -> Result<(), Guard> {
    fn welded<M: std::fmt::Debug, A: std::fmt::Debug>(model: M, declared: A) -> bool {
        format!("{model:?}") == format!("{declared:?}")
    }

    for operation in architecture::OperationId::ALL {
        if !welded(operation_branch(*operation), *operation) {
            return Err(Guard::BadAuthorization);
        }
    }

    for asset in architecture::AssetId::ALL {
        if !welded(asset_of(*asset), *asset) {
            return Err(Guard::BadAuthorization);
        }
    }

    for object in architecture::ObjectId::ALL {
        if !welded(object_kind_of(*object), *object) {
            return Err(Guard::BadAuthorization);
        }
    }

    for tag in architecture::TagId::ALL {
        if !welded(tag_of(*tag), *tag) {
            return Err(Guard::BadAuthorization);
        }
    }

    for kind in architecture::DeltaKind::ALL {
        if !welded(delta_kind_of(*kind), *kind) {
            return Err(Guard::BadAuthorization);
        }
    }

    for root_use in architecture::RootUse::ALL {
        if !welded(root_use_of(*root_use), *root_use) {
            return Err(Guard::BadAuthorization);
        }
    }

    for class in architecture::ValueFlowClass::ALL {
        if !welded(value_flow_of(*class), *class) {
            return Err(Guard::BadAuthorization);
        }
    }

    for kind in architecture::OpenFlowKind::ALL {
        if !welded(open_flow_of(*kind), *kind) {
            return Err(Guard::BadAuthorization);
        }
    }

    Ok(())
}

// ´rule:verification:manifest-delta-conformance´
//
// The manifest declares, per operation, the asset-specific canonical
// deltas together with their activation conditions and destruction
// tags. Postcommit validation compares the certificate's actual delta
// set against the manifest's active expected set.

pub(crate) fn validate_manifest_delta_conformance(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(), Guard> {
    let spec = operation_spec(certificate.branch);

    // The manifest declares delta *families*: an operation may emit
    // several flows of the same family (relabel declares one lateral
    // flow per receipt), so conformance compares signature sets, not
    // multiset counts.
    let mut expected: BTreeSet<(Asset, DeltaKind, Option<Tag>)> = BTreeSet::new();

    for delta in spec.canonical_deltas {
        if delta_condition_holds(delta.condition, before, after, certificate)? {
            expected.insert((
                asset_of(delta.asset),
                delta_kind_of(delta.kind),
                delta.destruction_tag.map(tag_of),
            ));
        }
    }

    let actual = certificate
        .canonical_deltas
        .iter()
        .map(|delta| (delta.asset, delta.kind, delta.destruction_tag))
        .collect::<BTreeSet<_>>();

    if expected != actual {
        return Err(Guard::CanonicalDeltaMismatch);
    }

    Ok(())
}

/// Evaluates a manifest delta condition against an actual transition.
pub(crate) fn delta_condition_holds(
    condition: architecture::DeltaCondition,
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<bool, Guard> {
    match condition {
        architecture::DeltaCondition::Always => Ok(true),

        architecture::DeltaCondition::PositiveAdmittedPrincipal => {
            let (old, new) = state_pair(before, after, certificate)?;

            Ok(new.q > old.q)
        }

        architecture::DeltaCondition::PositiveCycleIssuance => {
            let (old, new) = state_pair(before, after, certificate)?;

            Ok(new.y()? > old.y()?)
        }

        architecture::DeltaCondition::PositiveCyclePrincipal => {
            let (old, _new) = state_pair(before, after, certificate)?;

            Ok(!old.q.is_zero())
        }

        architecture::DeltaCondition::DistributionContinues => {
            Ok(certificate.distribution_residue.is_none())
        }

        architecture::DeltaCondition::DistributionTerminates => {
            Ok(certificate.distribution_residue.is_some())
        }

        architecture::DeltaCondition::PositiveSettlementUOutput => {
            Ok(!created_u_total(after, certificate)?.is_zero())
        }

        architecture::DeltaCondition::PositiveDistributionResidue => {
            match &certificate.distribution_residue {
                Some(residue) => Ok(!residue
                    .live_residue
                    .checked_add(residue.time_locked_residue)?
                    .is_zero()),

                None => Ok(false),
            }
        }

        architecture::DeltaCondition::PositiveAshResidual => {
            Ok(!created_ash_total(after, certificate)?.is_zero())
        }
    }
}

fn state_pair(
    before: &World,
    after: &World,
    certificate: &TransitionCertificate,
) -> Result<(PoolState, PoolState), Guard> {
    let edge = certificate.state_edge.ok_or(Guard::RootSuccession)?;

    let (input, output) = match edge {
        RootEdge::Succ { input, output } => (input, output),

        RootEdge::Term { .. } => {
            return Err(Guard::RootSuccession);
        }
    };

    let old = match before.utxo(input)?.meta {
        Meta::State(state) => state,
        _ => return Err(Guard::WrongShape),
    };

    let new = match after.utxo(output)?.meta {
        Meta::State(state) => state,
        _ => return Err(Guard::WrongShape),
    };

    Ok((old, new))
}

fn created_u_total(after: &World, certificate: &TransitionCertificate) -> Result<Sat, Guard> {
    let mut total = Sat::ZERO;

    for outpoint in &certificate.created {
        let utxo = after.utxo(*outpoint)?;

        if utxo.asset == Asset::U {
            total = total.checked_add(utxo.value)?;
        }
    }

    Ok(total)
}

fn created_ash_total(after: &World, certificate: &TransitionCertificate) -> Result<Sat, Guard> {
    let mut total = Sat::ZERO;

    for outpoint in &certificate.created {
        let utxo = after.utxo(*outpoint)?;

        if utxo.asset == Asset::U && utxo.meta == Meta::Ash {
            total = total.checked_add(utxo.value)?;
        }
    }

    Ok(total)
}

/// Every declared finite bound must be calibrated to a nonzero runtime
/// value that dominates the architecture's cardinality minima.
///
/// The minima come from the typed architecture (every input, output,
/// and data-output minimum whose maximum references the bound), not
/// from hardcoded constants: `ash_batch_max = 1` is nonzero yet makes
/// `compact-ash` (minimum two ASH inputs) unsatisfiable, so a nonzero
/// check alone admits constants under which a declared operation is
/// impossible.
pub fn validate_bound_conformance(constants: &Constants) -> Result<(), Guard> {
    for bound in ARCHITECTURE.bounds {
        let runtime =
            u64::try_from(bound_value(constants, bound.id)).map_err(|_| Guard::BadConstant)?;

        if runtime == 0 {
            return Err(Guard::BadConstant);
        }

        if runtime < architecture::manifest_minimum_for_bound(&ARCHITECTURE, bound.id) {
            return Err(Guard::BadConstant);
        }
    }

    Ok(())
}

/// The runtime constants must equal each bound's authority.
///
/// That authority is the deployment profile's calibration for
/// calibrated bounds and the manifest's declared fixed value for fixed
/// bounds: a deployment may not run with limits that differ from the
/// evidence it published.
pub fn validate_profile_bound_conformance(
    constants: &Constants,
    profile: &architecture::DeploymentProfile,
) -> Result<(), Guard> {
    validate_bounds_against_authority(ARCHITECTURE.bounds, constants, profile)
}

pub(crate) fn validate_bounds_against_authority(
    bounds: &[architecture::BoundSpec],
    constants: &Constants,
    profile: &architecture::DeploymentProfile,
) -> Result<(), Guard> {
    for bound in bounds {
        let runtime =
            u64::try_from(bound_value(constants, bound.id)).map_err(|_| Guard::BadConstant)?;

        if !bound.requires_deployment_calibration {
            // A fixed bound's value is the manifest declaration itself;
            // a missing declaration is a defect, not a free constant.
            let fixed = bound.default_value.ok_or(Guard::BadConstant)?;

            if runtime != fixed {
                return Err(Guard::BadConstant);
            }

            continue;
        }

        let calibrated = profile
            .calibrated_bounds
            .iter()
            .find(|calibration| calibration.bound == bound.id)
            .ok_or(Guard::BadConstant)?;

        if runtime != calibrated.value {
            return Err(Guard::BadConstant);
        }
    }

    Ok(())
}

// ´test:verification:no-accumulator-branches´ (world check)

/// Structural world check over declared object metadata forms.
///
/// Every UTXO must carry one of the declared metadata forms. The match
/// is exhaustive, so an attestation-accumulator object cannot be
/// introduced without extending [`Meta`] and revisiting this check.
pub fn assert_no_attestation_singleton(world: &World) -> Result<(), Guard> {
    for utxo in world.utxos.values() {
        match utxo.meta {
            Meta::State(_)
            | Meta::Resv
            | Meta::Pace
            | Meta::EntitlementAuthority
            | Meta::DistributionAuthority
            | Meta::Receipt { .. }
            | Meta::DepositRequest { .. }
            | Meta::DepositEntitlement { .. }
            | Meta::DistributionControl { .. }
            | Meta::DistributionVault { .. }
            | Meta::Ash
            | Meta::PlainLbtc { .. }
            | Meta::CpfpAnchor
            | Meta::ForeignShape(_) => {}
        }
    }

    Ok(())
}
