//! Exact attestation indexer, canonical serializer, and exact rational
//! reduction.
//!
//! Implements `(´def:ledgers:burn-transaction´)`,
//! `(´def:ledgers:clear-entry´)`, `(´def:ledgers:attestation-context´)`,
//! `(´def:ledgers:attestation-term´)`,
//! `(´def:ledgers:attestation-query-result´)`,
//! `(´def:verification:reference-indexer´)`,
//! `(´rule:ledgers:sample-clear´)`, `(´rule:ledgers:group-credit´)`,
//! `(´rule:ledgers:query-attestation´)`, `(´def:ledgers:exact-rational´)`,
//! `(´rule:ledgers:reduce-attestation´)`,
//! `(´rule:ledgers:canonical-biguint´)`, `(´rule:ledgers:canonical-varint´)`,
//! `(´rule:ledgers:canonical-attestation-serialization´)`,
//! `(´def:ledgers:decode-error´)`, `(´rule:ledgers:decode-varint´)`,
//! `(´rule:ledgers:decode-biguint´)`,
//! `(´rule:ledgers:decode-attestation-query´)`,
//! `(´def:verification:canonical-block´)`,
//! `(´def:verification:validated-chain-view´)`,
//! `(´def:verification:indexer-checkpoint´)`, and
//! `(´rule:verification:indexer-reproject´)`.
//!
//! The raw burn log stores no sampled clear identifier. Clear
//! assignment is derived here relative to one canonical history. The
//! grouped term list remains the normative export; reduction is
//! optional.
//!
//! The indexer carries one validated canonical attestation event
//! sequence (genesis clearing first, strictly increasing canonical
//! order). For one address over that pre-indexed stream, a query is
//! one ordered traversal of burn and clear events and their records:
//! `O(burns + clearings + records)`, excluding big-integer arithmetic
//! and output allocation. All-address materialization may add
//! associative-map or sorting cost and never defines the
//! single-address query claim.
//!
//! The differential-conformance boundary has two attestation halves
//! and one audit-only companion (see [`crate::audit`]): the raw
//! burn/clear event-projection differential, the canonical
//! attestation-query differential, and a separate receipt-accounting
//! residue differential under the external-auditor role. Query
//! equality does not prove event-set equality.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{One, Zero};

use architecture::{ARCHITECTURE, semantic_hash};

use crate::asset::Asset;
use crate::guard::Guard;
use crate::history::{
    BranchKind, BurnProjection, BurnRecord, DeltaKind, History, TransitionCertificate,
};
use crate::scalar::{
    AttestationAddress, BlockHash, BlockHeight, CanonicalOrder, Sat, SchemaVersion, TxId,
};

// ´def:ledgers:burn-transaction´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnTransaction {
    pub txid: TxId,
    pub block_hash: BlockHash,
    pub order: CanonicalOrder,

    pub ash_value: Sat,
    pub records: Vec<BurnRecord>,
}

impl BurnTransaction {
    /// The attestation gate is a derivation from source data,
    /// Σ record.amount <= ash_value, not a mutable stored decision.
    pub fn records_accepted(&self) -> Result<bool, Guard> {
        let claimed = self
            .records
            .iter()
            .map(|record| record.amount)
            .try_fold(Sat::ZERO, |acc, amount| acc.checked_add(amount))?;

        Ok(claimed <= self.ash_value)
    }
}

// ´def:ledgers:clear-entry´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClearEntry {
    pub clear_id: ClearId,
    pub block_hash: BlockHash,
    pub order: CanonicalOrder,
    pub omega: Sat,
    pub y: Sat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClearId {
    Genesis([u8; 32]),
    Transaction(TxId),
}

// ´def:ledgers:attestation-event´
//
// Event recognition and query computation are separate claims and
// require separate witnesses. The canonical ordered event stream is
// the recognition object; the grouped query is derived from it in one
// pass.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttestationEventId {
    GenesisClear(ClearId),
    Burn(TxId),
    Clear(ClearId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderedAttestationEvent {
    pub order: CanonicalOrder,
    pub id: AttestationEventId,
}

// ´def:ledgers:recognized-attestation-event´
//
// The owned form carries its full payload so that raw recognition and
// projection differences are comparable independently of any grouped
// query result.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecognizedAttestationEvent {
    GenesisClear(ClearEntry),
    Burn(BurnTransaction),
    Clear(ClearEntry),
}

impl RecognizedAttestationEvent {
    pub fn order(&self) -> CanonicalOrder {
        match self {
            Self::GenesisClear(clear) | Self::Clear(clear) => clear.order,
            Self::Burn(burn) => burn.order,
        }
    }
}

// ´def:ledgers:attestation-event-snapshot´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttestationEventSnapshot {
    pub context: AttestationContext,
    pub events: Vec<RecognizedAttestationEvent>,
}

// ´def:ledgers:attestation-context´

/// The wire-schema version supported by this reference indexer and
/// canonical query codec.
pub const ATTESTATION_SCHEMA_VERSION: SchemaVersion = 13;

// ´rule:ledgers:expected-architecture-hash´
//
// The expected hash comes from the normative typed architecture, not
// from a caller-supplied string.

pub fn expected_architecture_manifest_hash() -> Result<[u8; 32], QueryValidationError> {
    semantic_hash(&ARCHITECTURE).map_err(|_| QueryValidationError::ArchitectureHashUnavailable)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestationContext {
    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],

    /// Semantic hash of the typed architecture manifest governing the
    /// reference indexer and canonical query format.
    pub architecture_manifest_hash: [u8; 32],

    pub checkpoint_block_hash: BlockHash,
    pub checkpoint_height: BlockHeight,

    pub schema_version: SchemaVersion,
}

// ´def:ledgers:attestation-term´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttestationTerm {
    pub clear_id: ClearId,
    pub clear_order: CanonicalOrder,
    pub aggregate_burn_amount: BigUint,
    pub omega: BigUint,
    pub y: BigUint,
}

// ´def:ledgers:attestation-query-result´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttestationQueryResult {
    pub context: AttestationContext,
    pub address: AttestationAddress,
    pub terms: Vec<AttestationTerm>,
}

// ´def:verification:canonical-block´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalBlock {
    pub height: BlockHeight,
    pub hash: BlockHash,

    /// None is permitted only for the first block in the verifier's
    /// retained chain prefix.
    pub parent_hash: Option<BlockHash>,
}

// ´def:verification:validated-chain-view´

/// A validated view of one canonical chain prefix.
///
/// `ValidatedChainView` verifies internal prefix consistency. It does
/// not replace Elements consensus/header validation: the deployment
/// verifier must still supply a canonical chain view derived from
/// validated block headers and functionary consensus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedChainView {
    context: AttestationContext,
    blocks: BTreeMap<BlockHeight, CanonicalBlock>,
}

/// Validate the semantic identity fields of an attestation context.
///
/// The schema must be the supported wire schema, the architecture
/// manifest hash must be this build's expected typed hash, and the
/// network and genesis identities must be nonzero.
///
/// Enforced at every indexer construction path — chain-view
/// construction, history ingestion, and checkpoint reconstruction —
/// so a typed context can never carry a wrong-architecture or
/// placeholder identity into query results that only fail later at
/// serialization.
fn validate_context_identity(context: &AttestationContext) -> Result<(), Guard> {
    if context.schema_version != ATTESTATION_SCHEMA_VERSION {
        return Err(Guard::UnsupportedSchema);
    }

    let expected = expected_architecture_manifest_hash().map_err(|_| Guard::BadConstant)?;

    if context.architecture_manifest_hash != expected {
        return Err(Guard::BadConstant);
    }

    if context.network_id == [0_u8; 32] || context.genesis_id == [0_u8; 32] {
        return Err(Guard::Domain);
    }

    Ok(())
}

/// Validate one burn-record sequence against the kernel's canonical
/// form: contiguous ordinals from zero and nonzero amounts.
///
/// Mirrors `derive_burn_records` so a checkpoint or forged history
/// cannot smuggle a record shape the kernel could never emit.
fn validate_burn_records(records: &[BurnRecord]) -> Result<(), Guard> {
    for (position, record) in records.iter().enumerate() {
        if record.amount.is_zero() {
            return Err(Guard::Domain);
        }

        let expected_index = u32::try_from(position).map_err(|_| Guard::Overflow)?;

        if record.record_index != expected_index {
            return Err(Guard::WrongShape);
        }
    }

    Ok(())
}

/// Validate one burn payload against the facts every kernel-derived
/// burn guarantees: positive fresh ASH, kernel-shaped records, and a
/// record sum inside the `Sat` domain (so `records_accepted` cannot
/// fail at query time on an already-accepted payload).
///
/// Over-claiming (`Σ records > ash_value`) stays valid here: the
/// attestation gate is a credit verdict derived per query, not burn
/// provenance, so only arithmetic-domain failure rejects.
fn validate_burn_payload(ash_value: Sat, records: &[BurnRecord]) -> Result<(), Guard> {
    if ash_value.is_zero() {
        return Err(Guard::Domain);
    }

    validate_burn_records(records)?;

    Sat::checked_sum(records.iter().map(|record| record.amount))?;

    Ok(())
}

/// Validate the certificate/projection facts available at the
/// model-history boundary. This is defense in depth for histories
/// produced by the executable-model kernel; it is not independent raw
/// target-transaction recognition.
fn validate_model_burn_projection(
    certificate: &TransitionCertificate,
    burn: &BurnProjection,
) -> Result<(), Guard> {
    if certificate.branch != BranchKind::Burn
        || certificate.clear.is_some()
        || certificate.distribution_residue.is_some()
    {
        return Err(Guard::WrongShape);
    }

    validate_burn_payload(burn.ash_value, &burn.records)?;

    if !certificate.created.contains(&burn.ash_output) {
        return Err(Guard::WrongShape);
    }

    let mut saw_ash_lateral_destination = false;

    for delta in &certificate.canonical_deltas {
        if delta.asset != Asset::U {
            continue;
        }

        match delta.kind {
            DeltaKind::Lateral => {
                if delta.amount.is_zero()
                    || delta.authority_input.is_some()
                    || delta.source_inputs.is_empty()
                    || delta.destination_outputs.is_empty()
                    || delta.destruction_tag.is_some()
                {
                    return Err(Guard::WrongShape);
                }

                if !delta
                    .source_inputs
                    .iter()
                    .all(|source| certificate.consumed.contains(source))
                    || !delta
                        .destination_outputs
                        .iter()
                        .all(|destination| certificate.created.contains(destination))
                {
                    return Err(Guard::WrongShape);
                }

                if delta.destination_outputs.contains(&burn.ash_output) {
                    if delta.amount < burn.ash_value {
                        return Err(Guard::ValuePin);
                    }

                    saw_ash_lateral_destination = true;
                }
            }

            DeltaKind::Destruction => {
                return Err(Guard::WrongShape);
            }

            DeltaKind::Issuance | DeltaKind::OwnerlessLateral => {}
        }
    }

    if !saw_ash_lateral_destination {
        return Err(Guard::WrongShape);
    }

    Ok(())
}

/// A clear used for attestation is operational: genesis has E₀ > 0,
/// clear refuses a sealed state and leaves Y >= 1, and 𝗜₃ gives
/// Ω >= Y, so zero `omega` or zero `y` cannot correspond to a valid
/// clearing (the wire-level `validate_query` applies the same rule to
/// serialized terms).
fn validate_clear_payload(clear: &ClearEntry) -> Result<(), Guard> {
    validate_clear_values(clear.omega, clear.y)
}

/// By-value form serving both a stored [`ClearEntry`] and a raw
/// [`crate::history::ClearProjection`], so the operational-clear rule
/// has exactly one home.
fn validate_clear_values(omega: Sat, y: Sat) -> Result<(), Guard> {
    if omega.is_zero() || y.is_zero() {
        return Err(Guard::Domain);
    }

    Ok(())
}

/// Shared semantic validation of a complete indexer payload set:
/// context identity, event heights bounded by the checkpoint, kernel-
/// shaped burn records, and operational clear payloads.
///
/// `validate_event_index` establishes the structural table/stream
/// correspondence; this validator establishes that the payloads are
/// ones a validated chain and kernel-derived history could produce.
/// Every non-test construction path applies both, so any
/// `AttestationQueryResult` returned by a `ReferenceIndexer` passes
/// `validate_query` by construction.
fn validate_checkpoint_semantics(
    context: &AttestationContext,
    burns: &BTreeMap<TxId, BurnTransaction>,
    clears: &BTreeMap<ClearId, ClearEntry>,
    events: &[OrderedAttestationEvent],
) -> Result<(), Guard> {
    validate_context_identity(context)?;

    for event in events {
        if event.order.height > context.checkpoint_height {
            return Err(Guard::WrongCheckpoint);
        }
    }

    for burn in burns.values() {
        validate_burn_payload(burn.ash_value, &burn.records)?;
    }

    for clear in clears.values() {
        validate_clear_payload(clear)?;
    }

    Ok(())
}

impl ValidatedChainView {
    pub fn new(
        network_id: [u8; 32],
        genesis_id: [u8; 32],
        architecture_manifest_hash: [u8; 32],
        checkpoint_height: BlockHeight,
        checkpoint_block_hash: BlockHash,
        schema_version: SchemaVersion,
        blocks: impl IntoIterator<Item = CanonicalBlock>,
    ) -> Result<Self, Guard> {
        if schema_version != ATTESTATION_SCHEMA_VERSION {
            return Err(Guard::UnsupportedSchema);
        }

        let mut by_height = BTreeMap::new();

        for block in blocks {
            if by_height.insert(block.height, block).is_some() {
                return Err(Guard::DuplicateEvent);
            }
        }

        let checkpoint = by_height
            .get(&checkpoint_height)
            .ok_or(Guard::WrongCheckpoint)?;

        if checkpoint.hash != checkpoint_block_hash {
            return Err(Guard::WrongCheckpoint);
        }

        let context = AttestationContext {
            network_id,
            genesis_id,
            architecture_manifest_hash,
            checkpoint_block_hash,
            checkpoint_height,
            schema_version,
        };

        validate_context_identity(&context)?;

        Ok(Self {
            context,
            blocks: by_height,
        })
    }

    pub fn context(&self) -> AttestationContext {
        self.context
    }

    pub fn block_hash(&self, height: BlockHeight) -> Result<BlockHash, Guard> {
        if height > self.context.checkpoint_height {
            return Err(Guard::WrongCheckpoint);
        }

        self.blocks
            .get(&height)
            .map(|block| block.hash)
            .ok_or(Guard::WrongCheckpoint)
    }

    /// Validates a complete, contiguous canonical prefix from the
    /// realization genesis height through the selected checkpoint.
    ///
    /// The chain provider is responsible for validating block headers
    /// and consensus. This check ensures the view supplied to the
    /// indexer is internally contiguous and parent-linked.
    pub fn validate_prefix_from(&self, genesis_height: BlockHeight) -> Result<(), Guard> {
        if genesis_height > self.context.checkpoint_height {
            return Err(Guard::WrongCheckpoint);
        }

        let mut expected_height = genesis_height;
        let mut previous_hash: Option<BlockHash> = None;

        for block in self
            .blocks
            .range(genesis_height..=self.context.checkpoint_height)
            .map(|(_, block)| block)
        {
            if block.height != expected_height {
                return Err(Guard::HistoryOrder);
            }

            if let Some(previous_hash) = previous_hash
                && block.parent_hash != Some(previous_hash)
            {
                return Err(Guard::HistoryOrder);
            }

            previous_hash = Some(block.hash);

            if block.height != self.context.checkpoint_height {
                expected_height = expected_height.checked_add(1).ok_or(Guard::Overflow)?;
            }
        }

        if expected_height != self.context.checkpoint_height {
            return Err(Guard::HistoryOrder);
        }

        let checkpoint_hash = previous_hash.ok_or(Guard::WrongCheckpoint)?;

        if checkpoint_hash != self.context.checkpoint_block_hash {
            return Err(Guard::WrongCheckpoint);
        }

        Ok(())
    }
}

// ´def:verification:reference-indexer´

/// The genesis clear is sourced from the dedicated trusted-setup
/// projection carried by `History`, never from an ordinary transition.
///
/// The raw burn log retains every provenance-authenticated burn
/// transaction, including transactions whose records fail the
/// aggregate attestation gate: a genuine burn remains a genuine burn
/// even when its records over-claim. Only its attestation records are
/// rejected. Dropping the whole transaction would conflate burn-event
/// authentication with record-gate acceptance.
///
/// The index maps and the canonical event sequence are private: a
/// burn present in the map but absent from the event order (or the
/// reverse) is an unchecked correspondence, so the type owns the
/// relationship and every construction path validates it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceIndexer {
    /// The validated checkpoint context this index is bound to. The
    /// caller cannot supply query context; it is a property of the
    /// validated indexer state.
    context: AttestationContext,

    // Contains every provenance-authenticated burn,
    // including burns whose record set is rejected.
    burns: BTreeMap<TxId, BurnTransaction>,

    clears: BTreeMap<ClearId, ClearEntry>,

    /// Strict canonical order, genesis clearing first.
    events: Vec<OrderedAttestationEvent>,
}

// ´rule:verification:validate-event-index´
//
// The canonical event stream must satisfy:
//
// - the stream is nonempty and the genesis clearing is first;
// - events are strictly increasing by canonical order (which also
//   forbids two events at one order, and therefore a same-transaction
//   burn+clear pair);
// - every burn event refers to exactly one burn-table entry whose
//   payload order matches the event order;
// - every clear event refers to exactly one clear-table entry whose
//   payload order matches the event order;
// - no burn or clear table entry is absent from the event stream;
// - no event appears twice.

pub fn validate_event_index(
    burns: &BTreeMap<TxId, BurnTransaction>,
    clears: &BTreeMap<ClearId, ClearEntry>,
    events: &[OrderedAttestationEvent],
) -> Result<(), Guard> {
    // A valid index is never empty: the genesis clearing anchors the
    // total clear assignment and must exist as the first event. A
    // completely empty checkpoint would otherwise satisfy every
    // per-event rule and both census equalities vacuously.
    match events.first() {
        Some(first) if matches!(first.id, AttestationEventId::GenesisClear(_)) => {}
        _ => return Err(Guard::HistoryOrder),
    }

    let mut seen_burns = BTreeSet::new();
    let mut seen_clears = BTreeSet::new();

    let mut previous: Option<CanonicalOrder> = None;

    for (index, event) in events.iter().enumerate() {
        match event.id {
            AttestationEventId::GenesisClear(clear_id) => {
                if index != 0 {
                    return Err(Guard::HistoryOrder);
                }

                if !matches!(clear_id, ClearId::Genesis(_)) {
                    return Err(Guard::WrongShape);
                }

                let clear = clears.get(&clear_id).ok_or(Guard::NoSuch)?;

                if clear.clear_id != clear_id || clear.order != event.order {
                    return Err(Guard::HistoryOrder);
                }

                if !seen_clears.insert(clear_id) {
                    return Err(Guard::DuplicateEvent);
                }
            }

            AttestationEventId::Burn(txid) => {
                if index == 0 {
                    return Err(Guard::HistoryOrder);
                }

                let burn = burns.get(&txid).ok_or(Guard::NoSuch)?;

                if burn.txid != txid || burn.order != event.order {
                    return Err(Guard::HistoryOrder);
                }

                if !seen_burns.insert(txid) {
                    return Err(Guard::DuplicateEvent);
                }
            }

            AttestationEventId::Clear(clear_id) => {
                if index == 0 {
                    return Err(Guard::HistoryOrder);
                }

                if !matches!(clear_id, ClearId::Transaction(_)) {
                    return Err(Guard::WrongShape);
                }

                let clear = clears.get(&clear_id).ok_or(Guard::NoSuch)?;

                if clear.clear_id != clear_id || clear.order != event.order {
                    return Err(Guard::HistoryOrder);
                }

                if !seen_clears.insert(clear_id) {
                    return Err(Guard::DuplicateEvent);
                }
            }
        }

        if let Some(previous) = previous
            && event.order <= previous
        {
            return Err(Guard::HistoryOrder);
        }

        previous = Some(event.order);
    }

    if seen_burns.len() != burns.len() || seen_clears.len() != clears.len() {
        return Err(Guard::NoSuch);
    }

    Ok(())
}

// ´rule:verification:index-history´

impl ReferenceIndexer {
    pub fn context(&self) -> AttestationContext {
        self.context
    }

    /// Read-only view of the raw burn table. Mutation goes through the
    /// validated construction paths only.
    pub fn burns(&self) -> &BTreeMap<TxId, BurnTransaction> {
        &self.burns
    }

    /// Read-only view of the clear table.
    pub fn clears(&self) -> &BTreeMap<ClearId, ClearEntry> {
        &self.clears
    }

    /// Read-only view of the canonical ordered event sequence.
    pub fn events(&self) -> &[OrderedAttestationEvent] {
        &self.events
    }

    /// An opaque cache of this reference index, including its canonical
    /// event order. The returned value is a
    /// [`ModelIndexerCheckpoint`]: its fields are private, so a
    /// checkpoint can only originate from an already-projected reference
    /// index and can never be assembled from arbitrary caller rows.
    pub fn checkpoint(&self) -> ModelIndexerCheckpoint {
        ModelIndexerCheckpoint {
            context: self.context,
            burns: self.burns.clone(),
            clears: self.clears.clone(),
            events: self.events.clone(),
        }
    }

    /// The raw recognized event/projection sequence for the
    /// event-recognition differential. No grouping occurs here.
    pub fn event_snapshot(&self) -> Result<AttestationEventSnapshot, Guard> {
        let mut events = Vec::with_capacity(self.events.len());

        for event in &self.events {
            events.push(match event.id {
                AttestationEventId::GenesisClear(clear_id) => {
                    RecognizedAttestationEvent::GenesisClear(
                        *self.clears.get(&clear_id).ok_or(Guard::NoSuch)?,
                    )
                }

                AttestationEventId::Burn(txid) => RecognizedAttestationEvent::Burn(
                    self.burns.get(&txid).ok_or(Guard::NoSuch)?.clone(),
                ),

                AttestationEventId::Clear(clear_id) => RecognizedAttestationEvent::Clear(
                    *self.clears.get(&clear_id).ok_or(Guard::NoSuch)?,
                ),
            });
        }

        Ok(AttestationEventSnapshot {
            context: self.context,
            events,
        })
    }

    /// Synthetic constructor for deterministic unit tests.
    ///
    /// Production model projection code should use
    /// `from_model_history`.
    #[cfg(test)]
    pub(crate) fn empty_for_test(context: AttestationContext) -> Self {
        Self {
            context,
            burns: BTreeMap::new(),
            clears: BTreeMap::new(),
            events: Vec::new(),
        }
    }

    /// Test-only insertion that maintains the canonical event order
    /// and re-validates the index. Genesis clearing is recognized from
    /// the clear-id variant.
    #[cfg(test)]
    pub(crate) fn insert_clear_for_test(&mut self, clear: ClearEntry) -> Result<(), Guard> {
        let id = match clear.clear_id {
            ClearId::Genesis(_) => AttestationEventId::GenesisClear(clear.clear_id),
            ClearId::Transaction(_) => AttestationEventId::Clear(clear.clear_id),
        };

        if self.clears.insert(clear.clear_id, clear).is_some() {
            return Err(Guard::DuplicateEvent);
        }

        self.insert_event_for_test(OrderedAttestationEvent {
            order: clear.order,
            id,
        })
    }

    /// Test-only insertion that maintains the canonical event order
    /// and re-validates the index.
    #[cfg(test)]
    pub(crate) fn insert_burn_for_test(&mut self, burn: BurnTransaction) -> Result<(), Guard> {
        let order = burn.order;
        let txid = burn.txid;

        if self.burns.insert(txid, burn).is_some() {
            return Err(Guard::DuplicateEvent);
        }

        self.insert_event_for_test(OrderedAttestationEvent {
            order,
            id: AttestationEventId::Burn(txid),
        })
    }

    #[cfg(test)]
    fn insert_event_for_test(&mut self, event: OrderedAttestationEvent) -> Result<(), Guard> {
        self.events.push(event);

        self.events.sort_by_key(|event| event.order);

        validate_event_index(&self.burns, &self.clears, &self.events)
    }

    /// Project attestation events from history previously produced by
    /// the trusted executable-model kernel.
    ///
    /// This constructor validates internal certificate/event
    /// consistency, but it does not independently recognize or
    /// authenticate arbitrary caller-authored history. In particular,
    /// a model [`History`] does not carry enough data to prove every
    /// consumed object was a live receipt, that no ASH input was
    /// consumed, or that the ASH output value came from target
    /// consensus data. Independent deployment event recognition must
    /// derive events from validated target transactions.
    ///
    /// The history may contain transitions after the checkpoint. They
    /// are validated for strict order, txid uniqueness, and local
    /// projection/certificate consistency but are not indexed: the
    /// `ReferenceIndexer` is bound to one prefix.
    pub fn from_model_history(
        history: &History,
        chain: &ValidatedChainView,
        genesis_clear_id: [u8; 32],
    ) -> Result<Self, Guard> {
        // ValidatedChainView::new already establishes this, but the
        // indexer is an independent consumer and fails closed rather
        // than trusting the chain view's construction path.
        validate_context_identity(&chain.context())?;

        chain.validate_prefix_from(history.genesis.order.height)?;

        if history.genesis.order.height > chain.context().checkpoint_height {
            return Err(Guard::WrongCheckpoint);
        }

        let genesis_block_hash = chain.block_hash(history.genesis.order.height)?;

        let mut indexer = Self {
            context: chain.context(),
            burns: BTreeMap::new(),
            clears: BTreeMap::new(),
            events: Vec::new(),
        };

        let genesis_clear_id = ClearId::Genesis(genesis_clear_id);

        if indexer
            .clears
            .insert(
                genesis_clear_id,
                ClearEntry {
                    clear_id: genesis_clear_id,
                    block_hash: genesis_block_hash,
                    order: history.genesis.order,
                    omega: history.genesis.omega,
                    y: history.genesis.y,
                },
            )
            .is_some()
        {
            return Err(Guard::DuplicateEvent);
        }

        indexer.events.push(OrderedAttestationEvent {
            order: history.genesis.order,
            id: AttestationEventId::GenesisClear(genesis_clear_id),
        });

        let mut previous_order = history.genesis.order;

        // Seeded with the genesis txid so a transition cannot reuse it;
        // must stay in agreement with `replay_root_history`.
        let mut seen_txids = BTreeSet::from([history.genesis.txid]);

        for certificate in &history.transitions {
            if certificate.order <= previous_order {
                return Err(Guard::HistoryOrder);
            }

            previous_order = certificate.order;

            if !seen_txids.insert(certificate.txid) {
                return Err(Guard::DuplicateEvent);
            }

            // Event type comes from the complete transition, never
            // from a projection alone: the kernel attaches a burn
            // projection exactly on the burn branch, a clear
            // projection exactly on the clear branch, and a residue
            // projection only on a terminal distribution settlement.
            // A certificate pairing a projection with any other
            // branch is forged and the whole history is rejected.
            // (This also makes a same-transaction burn+clear pair
            // unrepresentable here, matching the kernel.)
            if certificate.burn.is_some() != (certificate.branch == BranchKind::Burn) {
                return Err(Guard::WrongShape);
            }

            if certificate.clear.is_some() != (certificate.branch == BranchKind::Clear) {
                return Err(Guard::WrongShape);
            }

            if certificate.distribution_residue.is_some()
                && certificate.branch != BranchKind::SettleDistribution
            {
                return Err(Guard::WrongShape);
            }

            // Applied here so forged payloads after the checkpoint are
            // also rejected; the indexed prefix is re-validated as a
            // whole by `validate_checkpoint_semantics` below.
            if let Some(burn) = &certificate.burn {
                validate_model_burn_projection(certificate, burn)?;
            }

            if let Some(clear) = &certificate.clear {
                validate_clear_values(clear.omega, clear.y)?;
            }

            // The ReferenceIndexer is bound to one prefix. Events
            // after the checkpoint are not part of this index.
            if certificate.order.height > indexer.context.checkpoint_height {
                continue;
            }

            let block_hash = chain.block_hash(certificate.order.height)?;

            if let Some(burn) = &certificate.burn {
                let entry = BurnTransaction {
                    txid: certificate.txid,
                    block_hash,
                    order: certificate.order,

                    ash_value: burn.ash_value,

                    records: burn.records.clone(),
                };

                if indexer.burns.insert(certificate.txid, entry).is_some() {
                    return Err(Guard::DuplicateEvent);
                }

                indexer.events.push(OrderedAttestationEvent {
                    order: certificate.order,
                    id: AttestationEventId::Burn(certificate.txid),
                });
            }

            if let Some(clear) = certificate.clear {
                let clear_id = ClearId::Transaction(certificate.txid);

                if indexer
                    .clears
                    .insert(
                        clear_id,
                        ClearEntry {
                            clear_id,
                            block_hash,
                            order: certificate.order,
                            omega: clear.omega,
                            y: clear.y,
                        },
                    )
                    .is_some()
                {
                    return Err(Guard::DuplicateEvent);
                }

                indexer.events.push(OrderedAttestationEvent {
                    order: certificate.order,
                    id: AttestationEventId::Clear(clear_id),
                });
            }
        }

        validate_event_index(&indexer.burns, &indexer.clears, &indexer.events)?;

        // The loop above validates transition payloads, but the
        // genesis clear is inserted directly from the model history
        // projection: the shared semantic validator closes that gap
        // (zero genesis omega/y fails here, not at query time) and
        // keeps every non-test construction path on the same
        // constructor invariant as checkpoint reconstruction.
        validate_checkpoint_semantics(
            &indexer.context,
            &indexer.burns,
            &indexer.clears,
            &indexer.events,
        )?;

        Ok(indexer)
    }

    // ´rule:ledgers:sample-clear´
    //
    // The genesis clear is ordered strictly before every burn, and
    // the event stream is strictly increasing, so `current_clear` is
    // always the most recent clearing strictly before a later burn.
    // No per-burn clear scan occurs.

    // ´rule:ledgers:group-credit´
    //
    // This preserves the distinction: a burn event existing is not the
    // same as its burn records being accepted.

    /// One ordered traversal of the canonical event stream for one
    /// address: `O(E + R_a)` where `E = burns + clears` and `R_a` is
    /// the number of records inspected while filtering for the
    /// address, excluding arbitrary-precision arithmetic and output
    /// allocation.
    fn terms_for_address(
        &self,
        address: AttestationAddress,
    ) -> Result<Vec<AttestationTerm>, Guard> {
        let mut current_clear: Option<&ClearEntry> = None;

        let mut terms: Vec<AttestationTerm> = Vec::new();

        for event in &self.events {
            match event.id {
                AttestationEventId::GenesisClear(clear_id)
                | AttestationEventId::Clear(clear_id) => {
                    current_clear = Some(self.clears.get(&clear_id).ok_or(Guard::NoSuch)?);
                }

                AttestationEventId::Burn(txid) => {
                    let burn = self.burns.get(&txid).ok_or(Guard::NoSuch)?;

                    if !burn.records_accepted()? {
                        continue;
                    }

                    let clear = current_clear.ok_or(Guard::NoSuch)?;

                    let amount = burn
                        .records
                        .iter()
                        .filter(|record| record.address == address)
                        .map(|record| record.amount)
                        .try_fold(Sat::ZERO, |acc, amount| acc.checked_add(amount))?;

                    if amount.is_zero() {
                        continue;
                    }

                    if clear.y.is_zero() {
                        return Err(Guard::Domain);
                    }

                    if let Some(last) = terms.last_mut()
                        && last.clear_id == clear.clear_id
                    {
                        last.aggregate_burn_amount += BigUint::from(amount.get());
                    } else {
                        terms.push(AttestationTerm {
                            clear_id: clear.clear_id,
                            clear_order: clear.order,
                            aggregate_burn_amount: BigUint::from(amount.get()),
                            omega: BigUint::from(clear.omega.get()),
                            y: BigUint::from(clear.y.get()),
                        });
                    }
                }
            }
        }

        Ok(terms)
    }

    /// All-address audit grouping: one canonical event traversal plus
    /// associative-map insertion. This helper is audit tooling; it is
    /// never the normative query path and its all-address aggregation
    /// cost does not define the complexity of a single-address query.
    pub fn group_all_credits_for_audit(
        &self,
    ) -> Result<BTreeMap<(AttestationAddress, ClearId), BigUint>, Guard> {
        let mut grouped = BTreeMap::new();

        let mut current_clear: Option<ClearId> = None;

        for event in &self.events {
            match event.id {
                AttestationEventId::GenesisClear(clear_id)
                | AttestationEventId::Clear(clear_id) => {
                    current_clear = Some(clear_id);
                }

                AttestationEventId::Burn(txid) => {
                    let burn = self.burns.get(&txid).ok_or(Guard::NoSuch)?;

                    if !burn.records_accepted()? {
                        continue;
                    }

                    let clear_id = current_clear.ok_or(Guard::NoSuch)?;

                    for record in &burn.records {
                        let key = (record.address, clear_id);

                        let entry = grouped.entry(key).or_insert_with(BigUint::zero);

                        *entry += BigUint::from(record.amount.get());
                    }
                }
            }
        }

        Ok(grouped)
    }

    // ´rule:ledgers:query-attestation´
    //
    // Terms are emitted in strict clear order by the ordered
    // traversal; no sort and no all-address grouping is needed.

    pub fn query(&self, address: AttestationAddress) -> Result<AttestationQueryResult, Guard> {
        Ok(AttestationQueryResult {
            context: self.context,
            address,
            terms: self.terms_for_address(address)?,
        })
    }
}

// ´def:ledgers:exact-rational´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactRational {
    numerator: BigUint,
    denominator: BigUint,
}

impl ExactRational {
    #[must_use]
    pub fn numerator(&self) -> &BigUint {
        &self.numerator
    }

    #[must_use]
    pub fn denominator(&self) -> &BigUint {
        &self.denominator
    }
}

// ´rule:ledgers:reduce-attestation´

impl AttestationQueryResult {
    fn reduce_validated(&self) -> ExactRational {
        let mut numerator = BigUint::zero();

        let mut denominator = BigUint::one();

        for term in &self.terms {
            let term_numerator = &term.aggregate_burn_amount * &term.omega;

            let term_denominator = &term.y;

            numerator = numerator * term_denominator + term_numerator * &denominator;

            denominator *= term_denominator;

            let gcd = numerator.gcd(&denominator);

            if !gcd.is_zero() {
                numerator /= &gcd;
                denominator /= gcd;
            }
        }

        if numerator.is_zero() {
            ExactRational {
                numerator: BigUint::zero(),
                denominator: BigUint::one(),
            }
        } else {
            ExactRational {
                numerator,
                denominator,
            }
        }
    }

    pub fn try_reduce(&self) -> Result<ExactRational, QueryValidationError> {
        validate_query(self)?;
        Ok(self.reduce_validated())
    }
}

// ´rule:ledgers:canonical-biguint´
//
// Zero is encoded as a canonical zero-length magnitude.
// (`BigUint::to_bytes_be` yields `[0]` for zero, which would violate
// the minimal-encoding rule, so zero is special-cased.)

pub fn encode_biguint(value: &BigUint, output: &mut Vec<u8>) -> Result<(), EncodeError> {
    if value.is_zero() {
        encode_varint(0, output);
        return Ok(());
    }

    let bytes = value.to_bytes_be();

    let length = u64::try_from(bytes.len()).map_err(|_| EncodeError::LengthOverflow)?;

    encode_varint(length, output);

    output.extend_from_slice(&bytes);

    Ok(())
}

// ´rule:ledgers:canonical-varint´
//
// A decoder must reject nonminimal encodings.

pub fn encode_varint(value: u64, output: &mut Vec<u8>) {
    let mut remaining = value;

    loop {
        let mut byte = (remaining & 0x7f) as u8;

        remaining >>= 7;

        if remaining != 0 {
            byte |= 0x80;
        }

        output.push(byte);

        if remaining == 0 {
            break;
        }
    }
}

// ´rule:ledgers:canonical-attestation-serialization´
//
// A canonical decoder verifies:
//
// - domain and schema;
// - fixed field widths;
// - minimal varints;
// - minimal integer encodings;
// - strict term ordering;
// - no duplicate clear ids;
// - nonzero `y`;
// - checkpoint context.

pub const ATTESTATION_QUERY_DOMAIN: &[u8] = b"tripod/query/v13";

// ´def:ledgers:query-validation-error´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryValidationError {
    UnsupportedSchema,
    ArchitectureManifestMismatch,
    ArchitectureHashUnavailable,
    ZeroNetworkId,
    ZeroGenesisId,

    DuplicateTerm,
    OutOfOrderTerm,

    ZeroAggregate,
    ZeroClearOmega,
    ZeroDenominator,

    TermAfterCheckpoint,

    LengthOverflow,
}

// ´def:ledgers:encode-error´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    InvalidQuery(QueryValidationError),
    LengthOverflow,
}

// ´rule:ledgers:validate-attestation-query´
//
// One shared semantic validator used by the serializer, the decoder,
// and the differential-indexer comparison.
//
// `ZeroClearOmega` rejects a term with `omega == 0`. A canonical clear
// used for attestation is operational: genesis has E₀ > 0, clear
// refuses a sealed state and leaves Y >= 1, and 𝗜₃ gives Ω >= Y, so a
// term with zero omega cannot correspond to a valid clearing.

pub fn validate_query(query: &AttestationQueryResult) -> Result<(), QueryValidationError> {
    if query.context.schema_version != ATTESTATION_SCHEMA_VERSION {
        return Err(QueryValidationError::UnsupportedSchema);
    }

    let expected_hash = expected_architecture_manifest_hash()?;

    if query.context.architecture_manifest_hash != expected_hash {
        return Err(QueryValidationError::ArchitectureManifestMismatch);
    }

    // The full context-identity rule from indexer construction: a
    // placeholder network or genesis identity must not serialize,
    // deserialize, or pass the differential comparison.
    if query.context.network_id == [0_u8; 32] {
        return Err(QueryValidationError::ZeroNetworkId);
    }

    if query.context.genesis_id == [0_u8; 32] {
        return Err(QueryValidationError::ZeroGenesisId);
    }

    u64::try_from(query.terms.len()).map_err(|_| QueryValidationError::LengthOverflow)?;

    let mut seen = BTreeSet::new();

    let mut previous: Option<(CanonicalOrder, ClearId)> = None;

    for term in &query.terms {
        if term.aggregate_burn_amount.is_zero() {
            return Err(QueryValidationError::ZeroAggregate);
        }

        if term.omega.is_zero() {
            return Err(QueryValidationError::ZeroClearOmega);
        }

        if term.y.is_zero() {
            return Err(QueryValidationError::ZeroDenominator);
        }

        if term.clear_order.height > query.context.checkpoint_height {
            return Err(QueryValidationError::TermAfterCheckpoint);
        }

        // The duplicate check precedes the ordering check so that
        // duplicate terms reliably report `DuplicateTerm` rather than
        // the ordering violation they also imply.
        if !seen.insert(term.clear_id) {
            return Err(QueryValidationError::DuplicateTerm);
        }

        let key = (term.clear_order, term.clear_id);

        if let Some(previous) = previous
            && key <= previous
        {
            return Err(QueryValidationError::OutOfOrderTerm);
        }

        previous = Some(key);

        let integer_lengths = [
            term.aggregate_burn_amount.to_bytes_be().len(),
            term.omega.to_bytes_be().len(),
            term.y.to_bytes_be().len(),
        ];

        for length in integer_lengths {
            u64::try_from(length).map_err(|_| QueryValidationError::LengthOverflow)?;
        }
    }

    Ok(())
}

fn query_error_to_encode(error: QueryValidationError) -> EncodeError {
    match error {
        QueryValidationError::LengthOverflow => EncodeError::LengthOverflow,

        other => EncodeError::InvalidQuery(other),
    }
}

fn query_error_to_decode(error: QueryValidationError) -> DecodeError {
    match error {
        QueryValidationError::UnsupportedSchema => DecodeError::UnsupportedSchema,

        QueryValidationError::ArchitectureManifestMismatch => {
            DecodeError::ArchitectureManifestMismatch
        }

        QueryValidationError::ArchitectureHashUnavailable => {
            DecodeError::ArchitectureHashUnavailable
        }

        QueryValidationError::ZeroNetworkId => DecodeError::ZeroNetworkId,

        QueryValidationError::ZeroGenesisId => DecodeError::ZeroGenesisId,

        QueryValidationError::DuplicateTerm => DecodeError::DuplicateTerm,

        QueryValidationError::OutOfOrderTerm => DecodeError::OutOfOrderTerm,

        QueryValidationError::ZeroAggregate => DecodeError::ZeroAggregate,

        QueryValidationError::ZeroClearOmega => DecodeError::ZeroClearOmega,

        QueryValidationError::ZeroDenominator => DecodeError::ZeroDenominator,

        QueryValidationError::TermAfterCheckpoint => DecodeError::TermAfterCheckpoint,

        QueryValidationError::LengthOverflow => DecodeError::LengthOverflow,
    }
}

fn encode_query_unchecked(query: &AttestationQueryResult) -> Result<Vec<u8>, EncodeError> {
    let mut output = Vec::new();

    output.extend_from_slice(ATTESTATION_QUERY_DOMAIN);

    output.extend_from_slice(&query.context.network_id);

    output.extend_from_slice(&query.context.genesis_id);

    output.extend_from_slice(&query.context.architecture_manifest_hash);

    output.extend_from_slice(&query.context.checkpoint_block_hash.0);

    output.extend_from_slice(&query.context.checkpoint_height.to_be_bytes());

    output.extend_from_slice(&query.context.schema_version.to_be_bytes());

    output.extend_from_slice(&query.address.0);

    let term_count = u64::try_from(query.terms.len()).map_err(|_| EncodeError::LengthOverflow)?;

    encode_varint(term_count, &mut output);

    for term in &query.terms {
        match term.clear_id {
            ClearId::Genesis(id) => {
                output.push(0);
                output.extend_from_slice(&id);
            }

            ClearId::Transaction(txid) => {
                output.push(1);
                output.extend_from_slice(&txid.0);
            }
        }

        output.extend_from_slice(&term.clear_order.height.to_be_bytes());

        output.extend_from_slice(&term.clear_order.tx_index.to_be_bytes());

        encode_biguint(&term.aggregate_burn_amount, &mut output)?;

        encode_biguint(&term.omega, &mut output)?;

        encode_biguint(&term.y, &mut output)?;
    }

    Ok(output)
}

/// Test-only structural encoder for constructing semantically invalid
/// bytes in decoder-malformation tests. Not exported from the crate.
#[cfg(test)]
pub(crate) fn serialize_query_unchecked_for_test(
    query: &AttestationQueryResult,
) -> Result<Vec<u8>, EncodeError> {
    encode_query_unchecked(query)
}

/// The canonical serializer cannot emit bytes that the shared semantic
/// validator would reject.
pub fn serialize_query(query: &AttestationQueryResult) -> Result<Vec<u8>, EncodeError> {
    validate_query(query).map_err(query_error_to_encode)?;

    encode_query_unchecked(query)
}

// ´def:ledgers:decode-error´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEnd,
    WrongDomain,

    UnsupportedSchema,
    ArchitectureManifestMismatch,
    ArchitectureHashUnavailable,
    ZeroNetworkId,
    ZeroGenesisId,

    NonMinimalVarint,
    NonMinimalInteger,

    DuplicateTerm,
    OutOfOrderTerm,

    ZeroAggregate,
    ZeroClearOmega,
    ZeroDenominator,

    TermAfterCheckpoint,

    LengthOverflow,
    TrailingBytes,
}

// ´rule:ledgers:decode-varint´
//
// The `shift == 63 && payload > 1` check prevents a ten-byte varint
// from silently truncating above `u64::MAX`.

pub fn decode_varint(input: &[u8], cursor: &mut usize) -> Result<u64, DecodeError> {
    let start = *cursor;

    let mut value = 0_u64;

    let mut shift = 0_u32;

    loop {
        let byte = *input.get(*cursor).ok_or(DecodeError::UnexpectedEnd)?;

        *cursor += 1;

        let payload = u64::from(byte & 0x7f);

        if shift > 63 {
            return Err(DecodeError::NonMinimalVarint);
        }

        if shift == 63 && payload > 1 {
            return Err(DecodeError::NonMinimalVarint);
        }

        value |= payload
            .checked_shl(shift)
            .ok_or(DecodeError::NonMinimalVarint)?;

        if byte & 0x80 == 0 {
            break;
        }

        shift = shift.checked_add(7).ok_or(DecodeError::NonMinimalVarint)?;
    }

    let mut canonical = Vec::new();

    encode_varint(value, &mut canonical);

    if input.get(start..*cursor) != Some(canonical.as_slice()) {
        return Err(DecodeError::NonMinimalVarint);
    }

    Ok(value)
}

// ´rule:ledgers:decode-biguint´
//
// Zero remains the unique zero-length encoding.

pub fn decode_biguint(input: &[u8], cursor: &mut usize) -> Result<BigUint, DecodeError> {
    let encoded_length = decode_varint(input, cursor)?;

    let length = usize::try_from(encoded_length).map_err(|_| DecodeError::LengthOverflow)?;

    let end = (*cursor)
        .checked_add(length)
        .ok_or(DecodeError::UnexpectedEnd)?;

    let bytes = input.get(*cursor..end).ok_or(DecodeError::UnexpectedEnd)?;

    *cursor = end;

    if length > 0 && bytes[0] == 0 {
        return Err(DecodeError::NonMinimalInteger);
    }

    Ok(BigUint::from_bytes_be(bytes))
}

// ´rule:ledgers:decode-attestation-query´
//
// The decoder validates schema and architecture hash early for fast
// rejection; structural checks (minimal varints, minimal integers,
// clear-id variants, truncation) run during parsing; the shared
// `validate_query` call is the authoritative semantic check.

pub fn deserialize_query(input: &[u8]) -> Result<AttestationQueryResult, DecodeError> {
    let mut cursor = 0_usize;

    let domain = input
        .get(0..ATTESTATION_QUERY_DOMAIN.len())
        .ok_or(DecodeError::UnexpectedEnd)?;

    if domain != ATTESTATION_QUERY_DOMAIN {
        return Err(DecodeError::WrongDomain);
    }

    cursor += ATTESTATION_QUERY_DOMAIN.len();

    fn take<const N: usize>(input: &[u8], cursor: &mut usize) -> Result<[u8; N], DecodeError> {
        let end = cursor.checked_add(N).ok_or(DecodeError::UnexpectedEnd)?;

        let slice = input.get(*cursor..end).ok_or(DecodeError::UnexpectedEnd)?;

        let mut bytes = [0_u8; N];
        bytes.copy_from_slice(slice);

        *cursor = end;
        Ok(bytes)
    }

    let network_id = take::<32>(input, &mut cursor)?;

    let genesis_id = take::<32>(input, &mut cursor)?;

    let architecture_manifest_hash = take::<32>(input, &mut cursor)?;

    let checkpoint_block_hash = BlockHash(take::<32>(input, &mut cursor)?);

    let checkpoint_height = u64::from_be_bytes(take::<8>(input, &mut cursor)?);

    let schema_version = u32::from_be_bytes(take::<4>(input, &mut cursor)?);

    if schema_version != ATTESTATION_SCHEMA_VERSION {
        return Err(DecodeError::UnsupportedSchema);
    }

    let expected_hash = expected_architecture_manifest_hash().map_err(query_error_to_decode)?;

    if architecture_manifest_hash != expected_hash {
        return Err(DecodeError::ArchitectureManifestMismatch);
    }

    let address = AttestationAddress(take::<32>(input, &mut cursor)?);

    let encoded_term_count = decode_varint(input, &mut cursor)?;

    let term_count =
        usize::try_from(encoded_term_count).map_err(|_| DecodeError::LengthOverflow)?;

    // Every term requires substantially more than one byte. This cheap
    // check prevents an attacker from claiming a huge term count and
    // forcing a large allocation before the decoder reaches EOF.
    if term_count > input.len() {
        return Err(DecodeError::UnexpectedEnd);
    }

    let mut terms = Vec::with_capacity(term_count.min(1_024));

    for _ in 0..term_count {
        let variant = *input.get(cursor).ok_or(DecodeError::UnexpectedEnd)?;

        cursor += 1;

        let id_bytes = take::<32>(input, &mut cursor)?;

        let clear_id = match variant {
            0 => ClearId::Genesis(id_bytes),

            1 => ClearId::Transaction(TxId(id_bytes)),

            _ => {
                return Err(DecodeError::WrongDomain);
            }
        };

        let height = u64::from_be_bytes(take::<8>(input, &mut cursor)?);

        let tx_index = u32::from_be_bytes(take::<4>(input, &mut cursor)?);

        let clear_order = CanonicalOrder { height, tx_index };

        let aggregate_burn_amount = decode_biguint(input, &mut cursor)?;

        let omega = decode_biguint(input, &mut cursor)?;

        let y = decode_biguint(input, &mut cursor)?;

        terms.push(AttestationTerm {
            clear_id,
            clear_order,
            aggregate_burn_amount,
            omega,
            y,
        });
    }

    if cursor != input.len() {
        return Err(DecodeError::TrailingBytes);
    }

    let query = AttestationQueryResult {
        context: AttestationContext {
            network_id,
            genesis_id,
            architecture_manifest_hash,
            checkpoint_block_hash,
            checkpoint_height,
            schema_version,
        },
        address,
        terms,
    };

    validate_query(&query).map_err(query_error_to_decode)?;

    Ok(query)
}

// ´def:verification:indexer-checkpoint´

/// An opaque cache of a reference index already projected from an
/// assumed kernel-produced model trace.
///
/// It is **not** an independent target-chain event recognizer and is
/// **not** deployment event evidence. Its fields are private, so a
/// checkpoint can only be obtained from an existing [`ReferenceIndexer`]
/// via [`ReferenceIndexer::checkpoint`]; there is no public constructor
/// and no public conversion that promotes arbitrary recognized rows into
/// a query-capable reference index. The following therefore does not
/// compile:
///
/// ```compile_fail
/// use model::{ModelIndexerCheckpoint, ReferenceIndexer};
///
/// // The cache fields are private; arbitrary recognized rows cannot be
/// // promoted into a query-capable reference index.
/// let checkpoint = ModelIndexerCheckpoint {
///     context: todo!(),
///     burns: Default::default(),
///     clears: Default::default(),
///     events: Default::default(),
/// };
/// let _indexer: ReferenceIndexer = checkpoint.into();
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelIndexerCheckpoint {
    context: AttestationContext,

    burns: BTreeMap<TxId, BurnTransaction>,

    clears: BTreeMap<ClearId, ClearEntry>,

    /// Strict canonical order, genesis clearing first.
    events: Vec<OrderedAttestationEvent>,
}

// ´rule:verification:indexer-reproject´
//
// A reorg produces a fresh reference index with a different canonical
// burn/clear set and event order; its checkpoint is a new opaque cache.
// Because the cache can only be created from an already-validated
// reference index, restoring it needs no re-validation: the private
// `restore` rebuilds the query-capable index for the read-only
// accessors below. There is no public arbitrary-row reconstruction.

impl ModelIndexerCheckpoint {
    /// The validated checkpoint context this cache is bound to.
    #[must_use]
    pub const fn context(&self) -> AttestationContext {
        self.context
    }

    /// The raw recognized event/projection sequence for the
    /// event-recognition differential.
    pub fn event_snapshot(&self) -> Result<AttestationEventSnapshot, Guard> {
        self.restore().event_snapshot()
    }

    /// The canonical attestation query for one address, computed from
    /// the cached index.
    pub fn query(&self, address: AttestationAddress) -> Result<AttestationQueryResult, Guard> {
        self.restore().query(address)
    }

    /// Rebuild the query-capable reference index from this opaque cache.
    ///
    /// Private on purpose: the cache is only ever created from an
    /// already-valid [`ReferenceIndexer`], so this is the trusted
    /// restoration path for the read-only accessors, not a public
    /// arbitrary-row constructor.
    fn restore(&self) -> ReferenceIndexer {
        ReferenceIndexer {
            context: self.context,
            burns: self.burns.clone(),
            clears: self.clears.clone(),
            events: self.events.clone(),
        }
    }
}

// ´def:verification:indexer-snapshot´

/// A publicly constructible diagnostic projection of a reference index.
///
/// Suitable for diagnostics and comparisons only. Caller-authored values
/// are untrusted, there is no conversion back to a query-capable
/// [`ReferenceIndexer`], and it makes no claim of chain provenance. It is
/// deliberately distinct from [`ModelIndexerCheckpoint`], which is the
/// opaque cache the reference index itself produces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexerDiagnosticSnapshot {
    pub context: AttestationContext,

    pub burns: BTreeMap<TxId, BurnTransaction>,

    pub clears: BTreeMap<ClearId, ClearEntry>,

    pub events: Vec<OrderedAttestationEvent>,
}

impl From<&ReferenceIndexer> for IndexerDiagnosticSnapshot {
    fn from(indexer: &ReferenceIndexer) -> Self {
        Self {
            context: indexer.context,
            burns: indexer.burns.clone(),
            clears: indexer.clears.clone(),
            events: indexer.events.clone(),
        }
    }
}

/// Test-only untrusted raw index rows, used to exercise the ingestion
/// gate without a public arbitrary-row constructor.
///
/// [`Self::check`] returns a consistency result — not a query-capable
/// [`ReferenceIndexer`] — so rejection tests prove the gate closes
/// without recreating the public provenance problem.
/// [`Self::restore`] additionally rebuilds a query-capable index for the
/// few in-crate tests that must exercise query semantics on a crafted
/// payload; it has no public counterpart.
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct UntrustedIndexerFixture {
    pub context: AttestationContext,
    pub burns: BTreeMap<TxId, BurnTransaction>,
    pub clears: BTreeMap<ClearId, ClearEntry>,
    pub events: Vec<OrderedAttestationEvent>,
}

#[cfg(test)]
impl UntrustedIndexerFixture {
    pub(crate) fn from_indexer(indexer: &ReferenceIndexer) -> Self {
        Self {
            context: indexer.context,
            burns: indexer.burns.clone(),
            clears: indexer.clears.clone(),
            events: indexer.events.clone(),
        }
    }

    pub(crate) fn check(&self) -> Result<(), Guard> {
        validate_event_index(&self.burns, &self.clears, &self.events)?;
        validate_checkpoint_semantics(&self.context, &self.burns, &self.clears, &self.events)
    }

    pub(crate) fn restore(&self) -> Result<ReferenceIndexer, Guard> {
        self.check()?;
        Ok(ReferenceIndexer {
            context: self.context,
            burns: self.burns.clone(),
            clears: self.clears.clone(),
            events: self.events.clone(),
        })
    }
}

// ´def:verification:differential-error´
//
// `Guard::InvariantFailure` loses the distinction between event
// recognition and query computation; the differential harness reports
// the precise failing claim.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifferentialError {
    ContextMismatch,
    EventSnapshotFailure,
    EventCountMismatch,
    EventOrderMismatch,
    EventKindMismatch,
    EventPayloadMismatch,
    QueryValidationFailure,
    QueryMismatch,
    ReceiptAccountingProjectionMismatch,
}

impl fmt::Display for DifferentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ContextMismatch => "attestation contexts differ",
            Self::EventSnapshotFailure => "candidate could not produce an event snapshot",
            Self::EventCountMismatch => "recognized attestation event counts differ",
            Self::EventOrderMismatch => "recognized attestation event orders differ",
            Self::EventKindMismatch => "recognized attestation event kinds differ",
            Self::EventPayloadMismatch => "recognized attestation event payloads differ",
            Self::QueryValidationFailure => "an indexer produced a semantically invalid query",
            Self::QueryMismatch => "canonical attestation query bytes differ",
            Self::ReceiptAccountingProjectionMismatch => {
                "receipt-accounting audit projections differ"
            }
        })
    }
}

impl std::error::Error for DifferentialError {}

// ´def:verification:attestation-query-provider´

/// The candidate side of the query differential.
///
/// A separately implemented indexer exposes its checkpoint context and
/// canonical query; the comparison then proves query computation and
/// serialization given the candidate's recognized event set — never
/// event recognition itself.
pub trait AttestationQueryProvider {
    fn context(&self) -> AttestationContext;

    fn query(&self, address: AttestationAddress) -> Result<AttestationQueryResult, Guard>;
}

impl AttestationQueryProvider for ReferenceIndexer {
    fn context(&self) -> AttestationContext {
        ReferenceIndexer::context(self)
    }

    fn query(&self, address: AttestationAddress) -> Result<AttestationQueryResult, Guard> {
        ReferenceIndexer::query(self, address)
    }
}

/// The candidate side of the combined conformance comparison: raw
/// recognized events plus canonical queries.
pub trait IndependentAttestationIndexer: AttestationQueryProvider {
    fn event_snapshot(&self) -> Result<AttestationEventSnapshot, Guard>;
}

impl IndependentAttestationIndexer for ReferenceIndexer {
    fn event_snapshot(&self) -> Result<AttestationEventSnapshot, Guard> {
        ReferenceIndexer::event_snapshot(self)
    }
}

// ´rule:verification:compare-attestation-events´
//
// Event recognition and query computation are separate claims and
// require separate witnesses: a candidate can recognize the wrong
// events yet accidentally compute the right aggregate, and offsetting
// recognition errors cancel invisibly in any grouped comparison. This
// comparison therefore matches the exact recognized event sequence —
// identity, order, block hash, ASH value, burn-record sequence, and
// clear `(Ω, Y)` — with no grouping.

pub fn compare_attestation_events(
    expected: &AttestationEventSnapshot,
    candidate: &AttestationEventSnapshot,
) -> Result<(), DifferentialError> {
    if expected.context != candidate.context {
        return Err(DifferentialError::ContextMismatch);
    }

    if expected.events.len() != candidate.events.len() {
        return Err(DifferentialError::EventCountMismatch);
    }

    for (expected_event, candidate_event) in expected.events.iter().zip(&candidate.events) {
        if expected_event.order() != candidate_event.order() {
            return Err(DifferentialError::EventOrderMismatch);
        }

        let same_kind = matches!(
            (expected_event, candidate_event),
            (
                RecognizedAttestationEvent::GenesisClear(_),
                RecognizedAttestationEvent::GenesisClear(_),
            ) | (
                RecognizedAttestationEvent::Burn(_),
                RecognizedAttestationEvent::Burn(_),
            ) | (
                RecognizedAttestationEvent::Clear(_),
                RecognizedAttestationEvent::Clear(_),
            )
        );

        if !same_kind {
            return Err(DifferentialError::EventKindMismatch);
        }

        if expected_event != candidate_event {
            return Err(DifferentialError::EventPayloadMismatch);
        }
    }

    Ok(())
}

// ´rule:verification:compare-indexers´
//
// This comparison proves query computation and canonical
// serialization given the candidate's recognized event set. It does
// not prove event recognition; `compare_attestation_events` carries
// that claim separately.

pub fn compare_attestation_query(
    expected: &impl AttestationQueryProvider,
    candidate: &impl AttestationQueryProvider,
    address: AttestationAddress,
) -> Result<(), DifferentialError> {
    if expected.context() != candidate.context() {
        return Err(DifferentialError::ContextMismatch);
    }

    let expected_query = expected
        .query(address)
        .map_err(|_| DifferentialError::QueryValidationFailure)?;

    let candidate_query = candidate
        .query(address)
        .map_err(|_| DifferentialError::QueryValidationFailure)?;

    // The comparison fails closed if either indexer produces a
    // semantically invalid query object.
    let expected_bytes =
        serialize_query(&expected_query).map_err(|_| DifferentialError::QueryValidationFailure)?;

    let candidate_bytes =
        serialize_query(&candidate_query).map_err(|_| DifferentialError::QueryValidationFailure)?;

    if expected_bytes != candidate_bytes {
        return Err(DifferentialError::QueryMismatch);
    }

    Ok(())
}

// ´rule:verification:compare-attestation-indexers´
//
// Combined conformance: neither comparison subsumes the other. For a
// complete finite fixture, compare every address appearing in the
// union of both raw event snapshots, plus addresses with expected zero
// results. For deployment evidence the candidate must be a separately
// implemented tool, not another `ReferenceIndexer`.

pub fn compare_attestation_indexers(
    expected: &ReferenceIndexer,
    candidate: &impl IndependentAttestationIndexer,
    addresses: impl IntoIterator<Item = AttestationAddress>,
) -> Result<(), DifferentialError> {
    let expected_snapshot = expected
        .event_snapshot()
        .map_err(|_| DifferentialError::EventSnapshotFailure)?;

    let candidate_snapshot = candidate
        .event_snapshot()
        .map_err(|_| DifferentialError::EventSnapshotFailure)?;

    compare_attestation_events(&expected_snapshot, &candidate_snapshot)?;

    for address in addresses {
        compare_attestation_query(expected, candidate, address)?;
    }

    Ok(())
}
