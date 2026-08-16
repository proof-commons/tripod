#![doc = include_str!("../README.md")]
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
pub use conformance::{
    ConformanceProjectionError, ModelConformanceObservation, observe_compact_ash,
    observe_live_transfer, unresolved_model_evidence,
};
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
