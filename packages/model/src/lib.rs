//! Executable verification model for Peer attestation.
//!
//! # Public-data boundary
//!
//! This crate accepts and exposes transparent public semantic state. It
//! does not accept production private keys, credentials, signing nonces,
//! blinding factors, private openings, or wallet-private state. `OwnerKey`
//! and `SignerSet` are public abstract authorization identities.
//! Cryptographic signing and secret custody remain outside the model
//! (ADR-015).
//!
//! Consolidated executable-model foundation, written as a pure
//! `World -> Result<World, Guard>` state machine. It is normative only
//! after it compiles and the generated seeded/property suites pass;
//! until then, it is the implementation target corresponding to the
//! frozen architecture manifest.
//!
//! Module map (each module carries the spec labels it implements):
//!
//! - [`scalar`] — identifiers and exact scalar types `(´def:verification:identifiers´)`,
//!   `(´def:domains:sat´)`, `(´def:domains:ratio´)`, checked arithmetic
//!   `(´rule:domains:checked-arithmetic´)`.
//! - [`asset`] — assets, classes, and maturity `(´def:architecture:assets´)`,
//!   `(´def:architecture:receipt-class´)`, `(´def:state:maturity´)`.
//! - [`pool`] — pool state `(´def:state:pool-state´)`.
//! - [`object`] — UTXO metadata and objects `(´def:objects:metadata´)`,
//!   `(´def:verification:utxo´)`, domain-separated data outputs
//!   `(´def:objects:destruction-tags´)`, `(´def:verification:data-output´)`.
//! - [`constants`] — configuration and finite bounds `(´def:domains:constants´)`.
//! - [`guard`] — guard and invariant errors `(´def:verification:guard´)`,
//!   `(´def:verification:invariant-error´)`.
//! - [`signer`] — signer abstraction `(´def:verification:signer-set´)`.
//! - [`history`] — transition kinds, root edges, canonical deltas,
//!   transition certificates, derived projections, and canonical chain
//!   history `(´def:verification:branch-kind´)` through `(´def:verification:history´)`.
//! - [`world`] — root cursors, wallets, adversarial environment, and the
//!   complete pure `World` `(´def:verification:root-cursor´)`,
//!   `(´def:verification:wallets´)`, `(´def:verification:external-budget´)`,
//!   `(´def:verification:world´)`.
//! - [`recognition`] — state projection classifier and branch-specific
//!   validators/readers `(´def:recognition:state-class´)` through
//!   `(´def:recognition:ash-view´)`.
//! - [`fee`] — fee envelope `(´def:auction:fee-envelope´)`,
//!   `(´rule:auction:fee-envelope-validation´)`.
//! - [`policy`] — branch policy `(´def:verification:branch-policy´)`.
//! - `kernel` (crate-private) — output staging, exact canonical-flow and issuance
//!   declarations, the transaction builder, conservation validators,
//!   exact witness partition, atomic commit, and root-cursor update
//!   `(´def:verification:transition-builder´)` through
//!   `(´rule:verification:root-cursor-update´)`.
//! - [`shape`] — operation-independent branch-shape validation
//!   `(´def:verification:object-kind´)` through
//!   `(´rule:verification:branch-event-projections´)`.
//! - `certify` (crate-private) — transition-certificate derivation
//!   `(´rule:verification:derive-transition-certificate´)` and its helper
//!   rules (root shapes, root edges, canonical deltas, projections,
//!   data-output validation).
//! - [`genesis`](mod@genesis) — trusted-setup genesis constructor
//!   `(´protocol:state:genesis´)`.
//! - [`invariant`] — full invariant checker over the state and history
//!   `(´def:verification:invariant-checker´)` and its component rules.
//! - [`ops`] — operation bodies (requests, admission, cycle,
//!   settlement, transfer, redemption, relabel, burn, ASH maintenance,
//!   maturity announcement, adversarial injection).
//! - [`ledger`] — exact attestation indexer, canonical serializer and
//!   decoder, reorg-aware checkpoint, and the split event/query
//!   differential-conformance comparisons
//!   `(´def:ledgers:burn-transaction´)` through
//!   `(´rule:verification:indexer-reproject´)`.
//! - [`audit`] — receipt-accounting audit projection and residue
//!   differential under the external-auditor role
//!   `(´def:verification:residue-audit-event´)` through
//!   `(´rule:verification:compare-receipt-accounting-audit´)`.
//! - [`quiescence`] — quiescence/lifecycle reports, audit helpers, and
//!   the residue-reader policy
//!   `(´def:verification:quiescence-report´)` through
//!   `(´rule:verification:residue-readers´)`.
//! - [`maintenance`] — sponsored-maintenance actions, scheduler
//!   interface, and the sweepability driver
//!   `(´def:verification:maintenance-action´)` through
//!   `(´rule:verification:apply-maintenance-action´)`.
//! - [`transition`] — atomic transition API `(´rule:verification:pure-transition´)`.
//!
//! # Trust boundary: a transparent reference model
//!
//! This crate is deliberately a **transparent reference model**, not a
//! capability-safe state machine. The public API makes the following
//! contract, and downstream crates (the future `vectors` and
//! `compiler` evidence machinery in particular) must rely on exactly
//! this — no more:
//!
//! - **[`World`] is intentionally inspectable and corruptible.** Its
//!   state fields are public so auditors, fault harnesses, and
//!   corruption fixtures can construct, examine, and deliberately
//!   damage model states. Public mutability is a feature of the proof
//!   surface, not an encapsulation failure.
//! - **[`execute`] is the recommended, invariant-wrapped transition
//!   path.** It applies a declared operation and re-checks the global
//!   invariant on the result. [`Transition::apply`] is also public and
//!   performs the operation's own guard checks but not the wrapper's
//!   invariant re-check; a **valid-world precondition** is part of its
//!   contract — applying a transition to a hand-corrupted world has no
//!   defined semantic meaning beyond what the individual guards catch.
//! - **The sealed [`Transition`] trait closes the operation
//!   vocabulary, not the state space.** External crates cannot add new
//!   transition kinds, but they can freely assemble or mutate worlds
//!   without going through `execute`. Rust does not — and this crate
//!   does not claim to — prevent that.
//! - **Low-level kernel transaction construction is crate-private.**
//!   `TxBuilder` and the staging/commit machinery cannot be reached
//!   from outside, so every *declared-operation* path does flow
//!   through the operation constructors.
//! - **Evidence rule.** A world is *model-valid evidence* only when
//!   every *protocol* state change in it was produced by
//!   [`genesis`](fn@genesis::genesis) followed by a chain of
//!   successful [`execute`] calls. Any other protocol change — direct
//!   construction, public field mutation, raw `apply` on corrupted
//!   state — is corruption evidence for fault/differential testing and
//!   must be labeled as such by the harness that produced it.
//!
//!   The rule governs protocol transitions, not the environment they
//!   run in (S4). Two things evolve a world without the protocol
//!   acting: block age advancing, and externally funded open objects
//!   arriving. Both are *substrate* facts. The chain moves time and
//!   third parties send coins whether or not this protocol does
//!   anything, so neither carries a transition certificate — there was
//!   no transition to certify, and recording one would assert that the
//!   protocol did something it did not.
//!
//!   Consequently the model must be safe under *arbitrary* substrate
//!   movement, not merely under the movement a test happens to script:
//!   no invariant may depend on how far time advanced, in what order
//!   external funds arrived, or on that history being final. A
//!   reorganization can rewrite substrate history, and the model must
//!   survive it. That is tractable here because no operation reveals
//!   private data as a condition of acting, so a reorg changes which
//!   history is current without having disclosed anything that cannot
//!   be undone.
//!
//!   A harness may therefore advance block age or inject an externally
//!   funded open object without invalidating the evidence, and must
//!   still route every protocol operation through [`execute`].
//!
//! The generated `generated/declassification.json` is likewise a
//! **provisional, derivative publication** (see [`artifacts`]): the
//! planned compiler derives declassification from the typed
//! realization dependency graph and never ingests that JSON.

// The Rust in this crate is a transcription of the normative executable
// model. To keep the source textually aligned with the frozen manifest,
// purely stylistic pedantic/nursery lints are allowed crate-wide;
// every correctness, suspicious, and perf lint remains in force.
#![allow(
    clippy::cast_possible_truncation,
    clippy::collapsible_match,
    clippy::comparison_chain,
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::manual_contains,
    clippy::manual_let_else,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    clippy::use_self
)]

pub mod artifacts;
pub mod asset;
pub mod audit;
pub(crate) mod certify;
pub mod conformance;
pub mod constants;
pub mod fee;
pub mod genesis;
pub mod guard;
pub mod history;
pub mod invariant;
pub(crate) mod kernel;
pub mod ledger;
pub mod maintenance;
pub mod manifest;
pub mod object;
pub mod ops;
pub mod policy;
pub mod pool;
pub mod property;
pub mod queries;
pub mod quiescence;
pub mod recognition;
pub mod scalar;
pub mod shape;
pub mod signer;
pub mod transition;
pub mod world;

#[cfg(test)]
mod tests;

pub use asset::{Asset, Maturity, ReceiptClass};
pub use audit::{
    ReceiptAccountingAuditProjection, ResidueAuditEvent, compare_receipt_accounting_audit,
    receipt_accounting_audit,
};
pub use conformance::{ConformanceProjectionError, observe_compact_ash, observe_live_transfer};
pub use constants::Constants;
pub use fee::{FeeChange, FeeEnvelope, validate_fee_envelope};
pub use genesis::{GENESIS_OWNER, OPERATOR_KEY, genesis};
pub use guard::{Guard, InvariantError};
pub use history::{
    BranchKind, BurnProjection, BurnRecord, CanonicalDelta, CanonicalDeltaFamily,
    CertifiedCanonicalFlow, CertifiedCanonicalPartition, CertifiedDestructionLeg,
    CertifiedIssuance, ClearProjection, DeltaKind, DistributionResidueProjection,
    GenesisProjection, History, OpenFlowKind, OpenFlowProjection, RootEdge, TransitionCertificate,
};
pub use invariant::{AccountingFold, check_invariant, clause_of};
// The low-level transaction builder and its flow/issuance declaration
// types are deliberately crate-private (`crate::kernel`): they prove
// structural transaction validity but do not establish owner/operator
// authorization, so they must not be reachable as a public
// construction path. The public normative transition surface is the
// operation-constructor set re-exported from [`ops`] plus
// [`transition::execute`]. Kernel structural unit tests import the
// crate-private path directly.
pub use ledger::{
    ATTESTATION_QUERY_DOMAIN, ATTESTATION_SCHEMA_VERSION, AttestationContext, AttestationEventId,
    AttestationEventSnapshot, AttestationQueryProvider, AttestationQueryResult, AttestationTerm,
    BurnTransaction, CanonicalBlock, ClearEntry, ClearId, DecodeError, DifferentialError,
    EncodeError, ExactRational, IndependentAttestationIndexer, IndexerDiagnosticSnapshot,
    ModelIndexerCheckpoint, OrderedAttestationEvent, QueryValidationError,
    RecognizedAttestationEvent, ReferenceIndexer, ValidatedChainView, compare_attestation_events,
    compare_attestation_indexers, compare_attestation_query, decode_biguint, decode_varint,
    deserialize_query, encode_biguint, encode_varint, expected_architecture_manifest_hash,
    serialize_query, validate_event_index, validate_query,
};
pub use maintenance::{
    AdmissionCapacityPlan, DeterministicMaintenanceScheduler, MaintenanceAction, MaintenanceMode,
    MaintenanceScheduler, MaintenanceSponsor, StateCandidate, admission_capacity_plan,
    apply_maintenance_action, capacity_admissible_request_batch, drive_quiescence_with_scheduler,
    drive_shared_state_sweepability, drive_shared_state_to_fixpoint, drive_sponsored_quiescence,
    maintenance_phase_potential, next_model_order,
};
pub use manifest::{
    assert_no_attestation_singleton, asset_of, bound_value, branch_operation, declared_asset,
    operation_branch, validate_architecture_conformance, validate_bound_conformance,
    validate_profile_bound_conformance,
};
pub use object::{DataOutput, Meta, Tag, Utxo};
pub use ops::{
    AdmitDeposits, AnnounceMaturity, BurnReceipts, CancelRequest, ClearAsh, CompactAsh,
    CreateRequest, CycleCaller, ReceiptDestination, RedeemReceipt, RelabelReceipts, RunCycle,
    SettleDistribution, TransferReceipts, inject_open_object,
};
pub use policy::{
    BranchPolicy, RootUse, ValueFlowClass, branch_policy, expected_value_flow_classes,
};
pub use pool::PoolState;
pub use property::{
    PROPERTY_ADDRESS_A, PROPERTY_ADDRESS_B, PROPERTY_OWNER_A, PROPERTY_OWNER_B, PROPERTY_OWNER_C,
    PROPERTY_SPONSOR, PropertyAction, PropertyActionSeed, PropertyStepResult,
    apply_property_action, drive_property_seed_trace, drive_property_trace, fund_property_world,
    materialize_action, property_action_name,
};
pub use queries::{cycle_issuance_query, floor_terms, redemption_payout};
pub use quiescence::{
    ProtocolObservable, QuantityId, QuiescenceEligibility, QuiescenceOutcome, QuiescenceReport,
    QuiescenceResidual, assert_protocol_noninterference, assert_residue_noninterference,
    assert_residue_reader_policy, classify_quiescence_eligibility, lifecycle_report,
    perturb_residue_projection, protocol_observable, quantity_reads_residue, quiescence_residuals,
    residuals_match_report, shared_state_is_swept,
};
pub use recognition::{
    CanonicalObject, DistributionControlView, EntitlementView, ReceiptView, RequestView,
    StateClass, classify_state_object, read_ash, read_distribution_control, read_entitlement,
    read_receipt, validate_request_for_admission,
};
pub use scalar::{
    ACTIVE_BACKING_MAX, AttestationAddress, BlockHash, BlockHeight, CanonicalOrder, Cycle,
    OutPoint, OwnerKey, Ratio, Sat, SchemaVersion, TWO_51, TxId, TxIndex, checked_active_backing,
    checked_add_to_map, checked_sum_sats, floor_mul_div, floor_ratio,
};
pub use shape::{ObjectKind, ShapePolicy};
pub use signer::{SignerSet, require_signer};
pub use transition::{
    ExecutedTransition, ExecutionBindingError, Transition, bind_execution, execute, execute_bound,
};
pub use world::{ExternalBudget, RootCursor, Wallets, World};
