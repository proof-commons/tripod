//! Output staging, exact canonical-flow and issuance declarations, the
//! transaction builder, canonical conservation validators, atomic
//! commit, and the root-cursor update.
//!
//! # Proof boundary
//!
//! The kernel proves structural transaction validity: conservation,
//! exact witness partitioning, branch shape, root use, event
//! projection, and postconditions. It does not independently establish
//! owner/operator authorization. Normative operation constructors
//! validate abstract authorization (`SignerSet`) before invoking this
//! crate-private builder. Exact signature bytes and sighash behavior
//! remain compiler/deployment obligations. The module is therefore
//! crate-private; the public normative transition surface is the
//! operation-constructor set plus [`crate::transition::execute`].
//!
//! Implements `(´def:verification:output-reference´)`,
//! `(´def:verification:destruction-leg´)`,
//! `(´def:verification:canonical-flow´)`,
//! `(´def:verification:issuance-route´)`,
//! `(´def:verification:destruction´)`,
//! `(´def:verification:transition-builder´)`,
//! `(´rule:verification:transition-builder-constructor´)`,
//! `(´rule:verification:declare-issuance´)`,
//! `(´rule:verification:declare-canonical-flow´)`,
//! `(´rule:verification:apply-fee-envelope´)`,
//! `(´rule:verification:asset-delta´)`,
//! `(´def:verification:commit-result´)`,
//! `(´rule:verification:atomic-commit-pipeline´)`,
//! `(´rule:verification:canonical-witness-partition´)`,
//! `(´rule:verification:flow-destructions´)`,
//! `(´rule:verification:root-input-policy´)`,
//! `(´rule:verification:closed-asset-conservation´)`,
//! `(´rule:verification:issuance-authority´)`,
//! `(´rule:verification:open-asset-conservation´)`,
//! `(´rule:verification:root-cursor-update´)`, and
//! `(´rule:verification:distinct-outpoints´)`.
//!
//! Aggregate canonical-asset conservation is insufficient if the same
//! input or output can be cited by more than one witness declaration.
//! The kernel therefore uses exact canonical-flow partitions: every
//! consumed `U`, `ENT`, or `DIST_CTL` outpoint belongs to exactly one
//! flow declaration; every created output of those assets belongs to
//! exactly one flow or issuance declaration; no source or destination
//! may be cited twice; each flow's actual source value equals its
//! actual destination value plus explicitly tagged destruction; and
//! each issuance's amount equals the exact sum of its destination
//! outputs. This makes the witness relation object-exact, not merely
//! aggregate-correct.

use std::collections::{BTreeMap, BTreeSet};

use crate::asset::Asset;
use crate::certify::derive_transition_certificate;
use crate::fee::FeeEnvelope;
use crate::guard::Guard;
use crate::history::{BranchKind, BurnRecord, DeltaKind, RootEdge, TransitionCertificate};
use crate::object::{DataOutput, Meta, Tag, Utxo};
use crate::policy::{BranchPolicy, RootUse, branch_policy};
use crate::scalar::{CanonicalOrder, OutPoint, Sat};
use crate::shape::{validate_branch_shape_postcommit, validate_branch_shape_precommit};
use crate::signer::require_signer;
use crate::world::World;

// ´def:verification:output-reference´

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputRef(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingOutput {
    pub asset: Asset,
    pub value: Sat,
    pub meta: Meta,
}

// ´def:verification:destruction-leg´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DestructionLeg {
    pub tag: Tag,
    pub amount: Sat,
}

// ´def:verification:canonical-flow´

/// A flow may contain:
///
/// - destinations only: transfer, burn-to-ASH, ASH compaction, receipt
///   relabel;
/// - destruction only: entitlement close, control close, redemption;
/// - both: partial clear, terminal distribution settlement.
///
/// `movement_kind` is:
///
/// - `Some(Lateral)` for owner-controlled or bound-object movement;
/// - `Some(OwnerlessLateral)` for ASH-to-ASH movement;
/// - `None` when no current-state output exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalFlow {
    pub asset: Asset,

    // Every source is a consumed UTXO of `asset`.
    pub source_inputs: Vec<OutPoint>,

    // Every destination is a created output of `asset`.
    pub destination_outputs: Vec<OutputRef>,

    // The source value not routed to current-state outputs
    // is destroyed under these domain-separated tags.
    pub destructions: Vec<DestructionLeg>,

    // Used only to classify the derived certificate projection.
    pub movement_kind: Option<DeltaKind>,
}

// ´def:verification:issuance-route´

/// A zero supply delta does not require an issuance declaration. Empty
/// cycles still consume and recreate PACE and DIST_AUTH as roots, but
/// declare no issuance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuanceDeclaration {
    pub asset: Asset,
    pub authority_asset: Asset,
    pub authority_input: OutPoint,
    pub amount: Sat,

    pub destination_outputs: Vec<OutputRef>,
}

// ´def:verification:destruction´
//
// Derived from flows for conservation checking and certificate
// projections; not declared directly by branch code.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DestructionDeclaration {
    pub asset: Asset,
    pub amount: Sat,
    pub tag: Tag,

    pub source_inputs: Vec<OutPoint>,
}

// ´def:verification:open-flow´
//
// Open-value flow roles (`OpenFlowKind`) live in [`crate::history`]
// because the derived `OpenFlowProjection` is certificate metadata;
// the kind is re-exported here beside the builder-side declaration.
//
// L-BTC conservation alone does not identify the semantic purpose of
// an L-BTC output. The kernel therefore applies the same
// exact-partition discipline to open L-BTC that it applies to
// canonical closed assets. This does not make L-BTC a closed protocol
// asset: arbitrary external L-BTC remains inert in the global
// environment. The flow declarations apply only to L-BTC actually
// consumed and created by one accepted transaction.

pub use crate::history::OpenFlowKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenFlow {
    pub kind: OpenFlowKind,

    // Every source is a consumed L-BTC UTXO.
    pub source_inputs: Vec<OutPoint>,

    // Every destination is a created L-BTC output.
    pub destination_outputs: Vec<OutputRef>,

    // This flow's exact contribution to the chain fee.
    pub fee: Sat,
}

// ´def:verification:transition-builder´

pub struct TxBuilder<'a> {
    base: &'a World,
    branch: BranchKind,
    order: CanonicalOrder,

    consumed: BTreeMap<OutPoint, Utxo>,
    outputs: Vec<PendingOutput>,
    data_outputs: Vec<DataOutput>,

    issuances: Vec<IssuanceDeclaration>,
    flows: Vec<CanonicalFlow>,
    open_flows: Vec<OpenFlow>,

    chain_fee: Sat,
}

// ´rule:verification:transition-builder-constructor´

impl<'a> TxBuilder<'a> {
    pub fn new(base: &'a World, branch: BranchKind, order: CanonicalOrder) -> Self {
        Self {
            base,
            branch,
            order,

            consumed: BTreeMap::new(),
            outputs: Vec::new(),
            data_outputs: Vec::new(),

            issuances: Vec::new(),
            flows: Vec::new(),
            open_flows: Vec::new(),

            chain_fee: Sat::ZERO,
        }
    }

    pub fn consume(&mut self, outpoint: OutPoint) -> Result<Utxo, Guard> {
        if self.consumed.contains_key(&outpoint) {
            return Err(Guard::DuplicateInput);
        }

        let utxo = *self.base.utxo(outpoint)?;

        self.consumed.insert(outpoint, utxo);

        Ok(utxo)
    }

    pub fn emit(&mut self, asset: Asset, value: Sat, meta: Meta) -> OutputRef {
        let output_ref = OutputRef(self.outputs.len());

        self.outputs.push(PendingOutput { asset, value, meta });

        output_ref
    }

    pub fn emit_data(&mut self, output: DataOutput) {
        self.data_outputs.push(output);
    }

    // ´rule:verification:declare-issuance´
    //
    // A zero supply delta has no issuance declaration. An empty cycle
    // still consumes and recreates PACE and DIST_AUTH, but declares no
    // issuance.

    pub fn declare_issuance(&mut self, declaration: IssuanceDeclaration) -> Result<(), Guard> {
        if declaration.amount.is_zero() {
            return Err(Guard::BadIssuance);
        }

        let expected_authority = declaration.asset.authority().ok_or(Guard::BadIssuance)?;

        if declaration.authority_asset != expected_authority {
            return Err(Guard::BadIssuance);
        }

        let authority_input = self
            .consumed
            .get(&declaration.authority_input)
            .ok_or(Guard::MissingAuthority)?;

        if authority_input.asset != expected_authority || authority_input.value != Sat::ONE {
            return Err(Guard::MissingAuthority);
        }

        if self
            .issuances
            .iter()
            .any(|issuance| issuance.asset == declaration.asset)
        {
            return Err(Guard::BadIssuance);
        }

        let mut seen_outputs = BTreeSet::new();

        let mut routed = Sat::ZERO;

        for output_ref in &declaration.destination_outputs {
            if !seen_outputs.insert(*output_ref) {
                return Err(Guard::DuplicateOutputIndex);
            }

            let output = self
                .outputs
                .get(output_ref.0)
                .ok_or(Guard::MissingOutputIndex)?;

            if output.asset != declaration.asset {
                return Err(Guard::WrongAsset);
            }

            routed = routed.checked_add(output.value)?;
        }

        if routed != declaration.amount {
            return Err(Guard::BadIssuance);
        }

        self.issuances.push(declaration);

        Ok(())
    }

    // ´rule:verification:declare-canonical-flow´

    pub fn declare_flow(&mut self, flow: CanonicalFlow) -> Result<(), Guard> {
        if !matches!(flow.asset, Asset::U | Asset::Ent | Asset::DistCtl) {
            return Err(Guard::WrongAsset);
        }

        if flow.source_inputs.is_empty() {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        let mut source_set = BTreeSet::new();

        let mut source_total = Sat::ZERO;

        for source in &flow.source_inputs {
            if !source_set.insert(*source) {
                return Err(Guard::DuplicateInput);
            }

            let utxo = self
                .consumed
                .get(source)
                .ok_or(Guard::CanonicalDeltaMismatch)?;

            if utxo.asset != flow.asset {
                return Err(Guard::WrongAsset);
            }

            source_total = source_total.checked_add(utxo.value)?;
        }

        let mut destination_set = BTreeSet::new();

        let mut destination_total = Sat::ZERO;

        for destination in &flow.destination_outputs {
            if !destination_set.insert(*destination) {
                return Err(Guard::DuplicateOutputIndex);
            }

            let output = self
                .outputs
                .get(destination.0)
                .ok_or(Guard::MissingOutputIndex)?;

            if output.asset != flow.asset {
                return Err(Guard::WrongAsset);
            }

            destination_total = destination_total.checked_add(output.value)?;
        }

        let mut destruction_total = Sat::ZERO;

        for destruction in &flow.destructions {
            if destruction.amount.is_zero() {
                return Err(Guard::ZeroProgress);
            }

            destruction_total = destruction_total.checked_add(destruction.amount)?;
        }

        if source_total != destination_total.checked_add(destruction_total)? {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        if destination_total.is_zero() && flow.movement_kind.is_some() {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        if !destination_total.is_zero() && flow.movement_kind.is_none() {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        for destruction in &flow.destructions {
            self.emit_data(DataOutput::Destruction {
                tag: destruction.tag,
                asset: flow.asset,
                amount: destruction.amount,
            });
        }

        self.flows.push(flow);

        Ok(())
    }

    // ´rule:verification:declare-open-flow´

    pub fn declare_open_flow(&mut self, flow: OpenFlow) -> Result<(), Guard> {
        if flow.source_inputs.is_empty() {
            return Err(Guard::ZeroProgress);
        }

        let mut source_set = BTreeSet::new();
        let mut source_total = Sat::ZERO;

        for source in &flow.source_inputs {
            if !source_set.insert(*source) {
                return Err(Guard::DuplicateInput);
            }

            let utxo = self.consumed.get(source).ok_or(Guard::ValuePin)?;

            if utxo.asset != Asset::Lbtc {
                return Err(Guard::WrongAsset);
            }

            source_total = source_total.checked_add(utxo.value)?;
        }

        let mut destination_set = BTreeSet::new();
        let mut destination_total = Sat::ZERO;

        for destination in &flow.destination_outputs {
            if !destination_set.insert(*destination) {
                return Err(Guard::DuplicateOutputIndex);
            }

            let output = self
                .outputs
                .get(destination.0)
                .ok_or(Guard::MissingOutputIndex)?;

            if output.asset != Asset::Lbtc {
                return Err(Guard::WrongAsset);
            }

            destination_total = destination_total.checked_add(output.value)?;
        }

        if source_total != destination_total.checked_add(flow.fee)? {
            return Err(Guard::OpenFlowMismatch);
        }

        self.open_flows.push(flow);
        Ok(())
    }

    pub fn set_chain_fee(&mut self, chain_fee: Sat) {
        self.chain_fee = chain_fee;
    }

    // ´rule:verification:apply-fee-envelope´
    //
    // The generic fee envelope accepts ordinary L-BTC wallet inputs
    // only. It cannot consume the active reserve or any
    // `RESV_SPK`-shaped object. A branch may use at most one generic
    // fee envelope in v13; branch-specific review checks this.

    pub fn apply_fee_envelope(&mut self, envelope: &FeeEnvelope) -> Result<(), Guard> {
        if envelope.inputs.is_empty() {
            if !envelope.chain_fee.is_zero() || envelope.change.is_some() {
                return Err(Guard::FeeMismatch);
            }

            return Ok(());
        }

        if envelope.inputs.len() > self.base.constants.fee_sponsor_input_max {
            return Err(Guard::Domain);
        }

        ensure_distinct(&envelope.inputs)?;

        let mut source_inputs = Vec::new();

        for input in &envelope.inputs {
            let utxo = self.consume(*input)?;

            let owner = match (utxo.asset, utxo.meta) {
                (Asset::Lbtc, Meta::PlainLbtc { owner }) => owner,

                _ => {
                    return Err(Guard::SponsorMismatch);
                }
            };

            require_signer(&envelope.signers, owner)?;

            source_inputs.push(*input);
        }

        let mut destination_outputs = Vec::new();

        if let Some(change) = envelope.change {
            if change.value.is_zero() {
                return Err(Guard::ZeroProgress);
            }

            destination_outputs.push(self.emit(
                Asset::Lbtc,
                change.value,
                Meta::PlainLbtc {
                    owner: change.owner,
                },
            ));
        }

        self.declare_open_flow(OpenFlow {
            kind: OpenFlowKind::FeeSponsor,
            source_inputs,
            destination_outputs,
            fee: envelope.chain_fee,
        })?;

        self.chain_fee = self.chain_fee.checked_add(envelope.chain_fee)?;

        Ok(())
    }

    /// This declaration-time check is a convenience for the public
    /// branch constructor. It is not the only check: certificate
    /// derivation independently revalidates the staged payloads,
    /// because `emit_data()` remains available to low-level and fault
    /// tests.
    pub fn emit_burn_record(&mut self, record: BurnRecord) -> Result<(), Guard> {
        let current_count = self
            .data_outputs
            .iter()
            .filter(|output| matches!(output, DataOutput::BurnRecord { .. }))
            .count();

        if current_count >= self.base.constants.burn_record_max {
            return Err(Guard::Domain);
        }

        if record.amount.is_zero() {
            return Err(Guard::Domain);
        }

        let expected_index = u32::try_from(current_count).map_err(|_| Guard::Overflow)?;

        if record.record_index != expected_index {
            return Err(Guard::WrongShape);
        }

        self.emit_data(DataOutput::BurnRecord {
            record_index: record.record_index,
            address: record.address,
            amount: record.amount,
        });

        Ok(())
    }
}

// ´rule:verification:asset-delta´

fn sum_asset_values<'a, I>(asset: Asset, values: I) -> Result<Sat, Guard>
where
    I: IntoIterator<Item = &'a Utxo>,
{
    values
        .into_iter()
        .filter(|utxo| utxo.asset == asset)
        .map(|utxo| utxo.value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
}

fn sum_pending_asset_values<'a, I>(asset: Asset, values: I) -> Result<Sat, Guard>
where
    I: IntoIterator<Item = &'a PendingOutput>,
{
    values
        .into_iter()
        .filter(|output| output.asset == asset)
        .map(|output| output.value)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
}

// ´def:verification:commit-result´

#[derive(Debug, PartialEq, Eq)]
pub struct CommitResult {
    pub world: World,
    pub output_map: BTreeMap<OutputRef, OutPoint>,
}

// ´rule:verification:atomic-commit-pipeline´

impl TxBuilder<'_> {
    /// The transition certificate is derived after actual input
    /// consumption and output allocation. Branch code does not author
    /// root edges or event projections.
    pub fn finish(self) -> Result<CommitResult, Guard> {
        let TxBuilder {
            base,
            branch,
            order,

            consumed,
            outputs,
            data_outputs,

            issuances,
            flows,
            open_flows,

            chain_fee,
        } = self;

        let policy = branch_policy(branch);

        // The exact witness partition is checked before root-input
        // policy so that a mis-witnessed canonical object is always
        // reported as a canonical-delta fault, independent of which
        // roots the transaction touches.
        validate_exact_canonical_witness_partition(&consumed, &outputs, &issuances, &flows)?;

        validate_root_inputs(base, &consumed, policy)?;

        let destructions = flows_to_destructions(&flows);

        validate_closed_asset_deltas(&consumed, &outputs, &issuances, &destructions)?;

        validate_exact_open_flow_partition(branch, &consumed, &outputs, &open_flows, chain_fee)?;

        // The aggregate L-BTC check remains as a redundant checksum.
        // The open-flow partition proves the semantic decomposition.
        validate_open_asset_conservation(&consumed, &outputs, chain_fee)?;

        validate_branch_shape_precommit(base, branch, &consumed, &outputs, &data_outputs)?;

        let mut next_world = base.clone();

        for outpoint in consumed.keys() {
            next_world.utxos.remove(outpoint);
        }

        let txid = next_world.next_txid()?;

        let mut output_map = BTreeMap::new();

        for (index, pending) in outputs.iter().cloned().enumerate() {
            let output_ref = OutputRef(index);
            let outpoint = next_world.next_outpoint()?;

            next_world.utxos.insert(
                outpoint,
                Utxo {
                    asset: pending.asset,
                    value: pending.value,
                    meta: pending.meta,
                },
            );

            output_map.insert(output_ref, outpoint);
        }

        let certificate = derive_transition_certificate(
            base,
            &next_world,
            txid,
            order,
            branch,
            &consumed,
            &output_map,
            &data_outputs,
            &issuances,
            &flows,
            &open_flows,
            chain_fee,
        )?;

        validate_fee_sponsor_flows(&certificate, &open_flows)?;

        update_root_cursors_from_certificate(&mut next_world, &certificate, policy)?;

        validate_branch_shape_postcommit(base, &next_world, &certificate)?;

        next_world.history.transitions.push(certificate);

        Ok(CommitResult {
            world: next_world,
            output_map,
        })
    }
}

// ´rule:verification:canonical-witness-partition´
//
// This prevents:
//
// - one source input from witnessing two flows;
// - one output from being funded twice;
// - a canonical source from being omitted;
// - a canonical output from being unwitnessed;
// - an issuance output from also being claimed by a lateral movement.

fn validate_exact_canonical_witness_partition(
    inputs: &BTreeMap<OutPoint, Utxo>,
    outputs: &[PendingOutput],
    issuances: &[IssuanceDeclaration],
    flows: &[CanonicalFlow],
) -> Result<(), Guard> {
    let value_assets = [Asset::U, Asset::Ent, Asset::DistCtl];

    let mut source_uses: BTreeMap<OutPoint, usize> = BTreeMap::new();

    let mut destination_uses: BTreeMap<OutputRef, usize> = BTreeMap::new();

    for flow in flows {
        for source in &flow.source_inputs {
            *source_uses.entry(*source).or_insert(0) += 1;
        }

        for destination in &flow.destination_outputs {
            *destination_uses.entry(*destination).or_insert(0) += 1;
        }
    }

    for issuance in issuances {
        for destination in &issuance.destination_outputs {
            *destination_uses.entry(*destination).or_insert(0) += 1;
        }
    }

    for (outpoint, utxo) in inputs {
        if value_assets.contains(&utxo.asset)
            && source_uses.get(outpoint).copied().unwrap_or(0) != 1
        {
            return Err(Guard::CanonicalDeltaMismatch);
        }
    }

    for (index, output) in outputs.iter().enumerate() {
        if value_assets.contains(&output.asset) {
            let output_ref = OutputRef(index);

            if destination_uses.get(&output_ref).copied().unwrap_or(0) != 1 {
                return Err(Guard::CanonicalDeltaMismatch);
            }
        }
    }

    for (source, count) in source_uses {
        if count != 1 {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        let utxo = inputs.get(&source).ok_or(Guard::CanonicalDeltaMismatch)?;

        if !value_assets.contains(&utxo.asset) {
            return Err(Guard::CanonicalDeltaMismatch);
        }
    }

    for (output_ref, count) in destination_uses {
        if count != 1 {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        let output = outputs.get(output_ref.0).ok_or(Guard::MissingOutputIndex)?;

        if !value_assets.contains(&output.asset) {
            return Err(Guard::CanonicalDeltaMismatch);
        }
    }

    Ok(())
}

// ´rule:verification:flow-destructions´
//
// The exact-flow partition guarantees the same source set is not used
// by another flow.

pub fn flows_to_destructions(flows: &[CanonicalFlow]) -> Vec<DestructionDeclaration> {
    flows
        .iter()
        .flat_map(|flow| {
            flow.destructions
                .iter()
                .map(|destruction| DestructionDeclaration {
                    asset: flow.asset,
                    amount: destruction.amount,
                    tag: destruction.tag,
                    source_inputs: flow.source_inputs.clone(),
                })
        })
        .collect()
}

// ´rule:verification:open-flow-policy´
//
// Admission's request-funded fee is carried by `DepositAdmission`; it
// does not use a generic fee-sponsor flow in the v13 core.

pub fn branch_allows_open_flow(branch: BranchKind, kind: OpenFlowKind) -> bool {
    // Derived from the typed manifest's per-operation open-flow
    // declarations; no hand-written allow-list remains.
    crate::manifest::operation_spec(branch)
        .open_flows
        .iter()
        .any(|declared| crate::manifest::open_flow_of(*declared) == kind)
}

// ´rule:verification:exact-open-flow-partition´
//
// Zero-value CPFP anchors are exempt from open-flow partitioning
// because they carry no L-BTC value.

fn validate_exact_open_flow_partition(
    branch: BranchKind,
    inputs: &BTreeMap<OutPoint, Utxo>,
    outputs: &[PendingOutput],
    open_flows: &[OpenFlow],
    chain_fee: Sat,
) -> Result<(), Guard> {
    let mut source_uses: BTreeMap<OutPoint, usize> = BTreeMap::new();

    let mut destination_uses: BTreeMap<OutputRef, usize> = BTreeMap::new();

    let mut fee_total = Sat::ZERO;

    for flow in open_flows {
        if !branch_allows_open_flow(branch, flow.kind) {
            return Err(Guard::BadAuthorization);
        }

        fee_total = fee_total.checked_add(flow.fee)?;

        for source in &flow.source_inputs {
            *source_uses.entry(*source).or_insert(0) += 1;
        }

        for destination in &flow.destination_outputs {
            *destination_uses.entry(*destination).or_insert(0) += 1;
        }
    }

    for (outpoint, utxo) in inputs {
        if utxo.asset == Asset::Lbtc
            && !utxo.value.is_zero()
            && source_uses.get(outpoint).copied().unwrap_or(0) != 1
        {
            return Err(Guard::OpenFlowMismatch);
        }
    }

    // Coverage faults — source and destination — are reported before
    // the aggregate fee equation, so a missing flow surfaces as an
    // open-flow fault rather than a fee mismatch.
    for (index, output) in outputs.iter().enumerate() {
        if output.asset == Asset::Lbtc && !output.value.is_zero() {
            let output_ref = OutputRef(index);

            if destination_uses.get(&output_ref).copied().unwrap_or(0) != 1 {
                return Err(Guard::OpenFlowMismatch);
            }
        }

        if output.asset == Asset::Lbtc
            && output.value.is_zero()
            && !matches!(output.meta, Meta::CpfpAnchor)
        {
            return Err(Guard::Domain);
        }
    }

    if fee_total != chain_fee {
        return Err(Guard::FeeMismatch);
    }

    for (source, count) in source_uses {
        if count != 1 {
            return Err(Guard::OpenFlowMismatch);
        }

        let utxo = inputs.get(&source).ok_or(Guard::OpenFlowMismatch)?;

        if utxo.asset != Asset::Lbtc {
            return Err(Guard::WrongAsset);
        }
    }

    for (destination, count) in destination_uses {
        if count != 1 {
            return Err(Guard::OpenFlowMismatch);
        }

        let output = outputs
            .get(destination.0)
            .ok_or(Guard::MissingOutputIndex)?;

        if output.asset != Asset::Lbtc || output.value.is_zero() {
            return Err(Guard::OpenFlowMismatch);
        }
    }

    Ok(())
}

// ´rule:verification:sponsor-envelope-postcondition´
//
// Open-flow role data is proof-only and is not an on-chain event. This
// validation runs precommit over the builder's declarations rather
// than storing `open_flow_kinds` as certificate metadata.

fn validate_fee_sponsor_flows(
    certificate: &TransitionCertificate,
    open_flows: &[OpenFlow],
) -> Result<(), Guard> {
    let sponsor_fee = open_flows
        .iter()
        .filter(|flow| flow.kind == OpenFlowKind::FeeSponsor)
        .map(|flow| flow.fee)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let other_fee = open_flows
        .iter()
        .filter(|flow| flow.kind != OpenFlowKind::FeeSponsor)
        .map(|flow| flow.fee)
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    if sponsor_fee.checked_add(other_fee)? != certificate.chain_fee {
        return Err(Guard::FeeMismatch);
    }

    Ok(())
}

// ´rule:verification:flow-helper´

pub fn movement_flow(
    asset: Asset,
    kind: DeltaKind,
    source_inputs: Vec<OutPoint>,
    destination_outputs: Vec<OutputRef>,
) -> CanonicalFlow {
    CanonicalFlow {
        asset,
        source_inputs,
        destination_outputs,
        destructions: Vec::new(),
        movement_kind: Some(kind),
    }
}

pub fn destruction_flow(
    asset: Asset,
    source_inputs: Vec<OutPoint>,
    tag: Tag,
    amount: Sat,
) -> CanonicalFlow {
    CanonicalFlow {
        asset,
        source_inputs,
        destination_outputs: Vec::new(),
        destructions: vec![DestructionLeg { tag, amount }],
        movement_kind: None,
    }
}

/// Retained for kernel structural tests that declare a combined
/// movement-plus-destruction flow directly.
#[cfg_attr(not(test), allow(dead_code))]
pub fn mixed_flow(
    asset: Asset,
    kind: DeltaKind,
    source_inputs: Vec<OutPoint>,
    destination_outputs: Vec<OutputRef>,
    destructions: Vec<DestructionLeg>,
) -> CanonicalFlow {
    CanonicalFlow {
        asset,
        source_inputs,
        destination_outputs,
        destructions,
        movement_kind: Some(kind),
    }
}

// ´rule:verification:root-input-policy´
//
// Branch-specific validation additionally forbids every `RESV_SPK`-shaped
// input on weld-exempt STATE branches, including inactive decoys.

fn validate_root_inputs(
    world: &World,
    consumed: &BTreeMap<OutPoint, Utxo>,
    policy: BranchPolicy,
) -> Result<(), Guard> {
    validate_one_root_use(world.roots.state, consumed, policy.state)?;

    validate_optional_root_use(world.roots.resv, consumed, policy.resv)?;

    validate_one_root_use(world.roots.pace, consumed, policy.pace)?;

    validate_one_root_use(
        world.roots.entitlement_authority,
        consumed,
        policy.entitlement_authority,
    )?;

    validate_one_root_use(
        world.roots.distribution_authority,
        consumed,
        policy.distribution_authority,
    )?;

    Ok(())
}

fn validate_one_root_use(
    root: OutPoint,
    consumed: &BTreeMap<OutPoint, Utxo>,
    use_policy: RootUse,
) -> Result<(), Guard> {
    let present = consumed.contains_key(&root);

    match use_policy {
        RootUse::Forbidden if present => Err(Guard::RootSuccession),

        RootUse::Forbidden => Ok(()),

        RootUse::Succession | RootUse::SuccessionOrTermination if present => Ok(()),

        RootUse::Succession | RootUse::SuccessionOrTermination => Err(Guard::RootSuccession),
    }
}

fn validate_optional_root_use(
    root: Option<OutPoint>,
    consumed: &BTreeMap<OutPoint, Utxo>,
    use_policy: RootUse,
) -> Result<(), Guard> {
    match root {
        Some(root) => validate_one_root_use(root, consumed, use_policy),

        None => match use_policy {
            RootUse::Forbidden => Ok(()),

            RootUse::Succession | RootUse::SuccessionOrTermination => Err(Guard::Sealed),
        },
    }
}

// ´rule:verification:closed-asset-conservation´

fn validate_closed_asset_deltas(
    inputs: &BTreeMap<OutPoint, Utxo>,
    outputs: &[PendingOutput],
    issuances: &[IssuanceDeclaration],
    destructions: &[DestructionDeclaration],
) -> Result<(), Guard> {
    let closed_assets = [
        Asset::U,
        Asset::Ent,
        Asset::DistCtl,
        Asset::Pid,
        Asset::Pace,
        Asset::EntAuth,
        Asset::DistAuth,
    ];

    for asset in closed_assets {
        let input_total = sum_asset_values(asset, inputs.values())?;

        let output_total = sum_pending_asset_values(asset, outputs.iter())?;

        let issued = issuances
            .iter()
            .filter(|issuance| issuance.asset == asset)
            .map(|issuance| issuance.amount)
            .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

        let destroyed = destructions
            .iter()
            .filter(|destruction| destruction.asset == asset)
            .map(|destruction| destruction.amount)
            .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

        let lhs = input_total.checked_add(issued)?;

        let rhs = output_total.checked_add(destroyed)?;

        if lhs != rhs {
            return Err(Guard::CanonicalDeltaMismatch);
        }

        if asset.is_singleton_root_asset() && (!issued.is_zero() || !destroyed.is_zero()) {
            return Err(Guard::BadIssuance);
        }
    }

    validate_issuance_authorities(inputs, outputs, issuances)?;

    Ok(())
}

// ´rule:verification:issuance-authority´
//
// The emitted script must additionally bind each issuance to the correct
// authority token through Elements issuance introspection.

fn validate_issuance_authorities(
    inputs: &BTreeMap<OutPoint, Utxo>,
    outputs: &[PendingOutput],
    issuances: &[IssuanceDeclaration],
) -> Result<(), Guard> {
    for issuance in issuances {
        let expected_authority = issuance.asset.authority().ok_or(Guard::BadIssuance)?;

        if issuance.authority_asset != expected_authority {
            return Err(Guard::BadIssuance);
        }

        let authority_input = inputs
            .get(&issuance.authority_input)
            .ok_or(Guard::MissingAuthority)?;

        if authority_input.asset != expected_authority || authority_input.value != Sat::ONE {
            return Err(Guard::MissingAuthority);
        }

        let successor_count = outputs
            .iter()
            .filter(|output| output.asset == expected_authority && output.value == Sat::ONE)
            .count();

        if successor_count != 1 {
            return Err(Guard::RootSuccession);
        }
    }

    Ok(())
}

// ´rule:verification:open-asset-conservation´

fn validate_open_asset_conservation(
    inputs: &BTreeMap<OutPoint, Utxo>,
    outputs: &[PendingOutput],
    fee: Sat,
) -> Result<(), Guard> {
    let lbtc_in = sum_asset_values(Asset::Lbtc, inputs.values())?;

    let lbtc_out = sum_pending_asset_values(Asset::Lbtc, outputs.iter())?;

    if lbtc_in != lbtc_out.checked_add(fee)? {
        return Err(Guard::FeeMismatch);
    }

    let mut foreign_ids = BTreeSet::new();

    for input in inputs.values() {
        if let Asset::Foreign(id) = input.asset {
            foreign_ids.insert(id);
        }
    }

    for output in outputs {
        if let Asset::Foreign(id) = output.asset {
            foreign_ids.insert(id);
        }
    }

    for id in foreign_ids {
        let asset = Asset::Foreign(id);

        let input_total = sum_asset_values(asset, inputs.values())?;

        let output_total = sum_pending_asset_values(asset, outputs.iter())?;

        if input_total != output_total {
            return Err(Guard::CanonicalDeltaMismatch);
        }
    }

    Ok(())
}

// ´rule:verification:root-cursor-update´

pub fn update_root_cursors_from_certificate(
    world: &mut World,
    certificate: &TransitionCertificate,
    policy: BranchPolicy,
) -> Result<(), Guard> {
    match certificate.state_edge {
        Some(RootEdge::Succ { output, .. }) => {
            world.roots.state = output;
        }

        Some(RootEdge::Term { .. }) => {
            return Err(Guard::RootSuccession);
        }

        None => {
            if policy.state != RootUse::Forbidden {
                return Err(Guard::RootSuccession);
            }
        }
    }

    match certificate.resv_edge {
        Some(RootEdge::Succ { output, .. }) => {
            world.roots.resv = Some(output);
        }

        Some(RootEdge::Term { .. }) => {
            world.roots.resv = None;
        }

        None => {
            if policy.resv != RootUse::Forbidden {
                return Err(Guard::RootSuccession);
            }
        }
    }

    if let Some(RootEdge::Succ { output, .. }) = certificate.pace_edge {
        world.roots.pace = output;
        world.pace_age_blocks = 0;
    }

    if let Some(RootEdge::Succ { output, .. }) = certificate.entitlement_authority_edge {
        world.roots.entitlement_authority = output;
    }

    if let Some(RootEdge::Succ { output, .. }) = certificate.distribution_authority_edge {
        world.roots.distribution_authority = output;
    }

    Ok(())
}

// ´rule:verification:distinct-outpoints´

pub fn ensure_distinct(outpoints: &[OutPoint]) -> Result<(), Guard> {
    let mut set = BTreeSet::new();

    for outpoint in outpoints {
        if !set.insert(*outpoint) {
            return Err(Guard::DuplicateInput);
        }
    }

    Ok(())
}
