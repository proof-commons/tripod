//! Executable verification model for Peer attestation.
//!
//! Consolidated executable-model foundation, written as a pure
//! `World -> Result<World, Guard>` state machine. It is normative only
//! after it compiles and the generated seeded/property suites pass;
//! until then, it is the implementation target corresponding to the
//! frozen architecture manifest.
//!
//! Module map (each module carries the spec labels it implements):
//!
//! - [`scalar`] — identifiers and exact scalar types `´def:verification:identifiers´`,
//!   `´def:domains:sat´`, `´def:domains:ratio´`, checked arithmetic
//!   `´rule:domains:checked-arithmetic´`.
//! - [`asset`] — assets, classes, and maturity `´def:architecture:assets´`,
//!   `´def:architecture:receipt-class´`, `´def:state:maturity´`.
//! - [`pool`] — pool state `´def:state:pool-state´`.
//! - [`object`] — UTXO metadata and objects `´def:objects:metadata´`,
//!   `´def:verification:utxo´`, domain-separated data outputs
//!   `´def:objects:destruction-tags´`, `´def:verification:data-output´`.
//! - [`constants`] — configuration and finite bounds `´def:domains:constants´`.
//! - [`guard`] — guard and invariant errors `´def:verification:guard´`,
//!   `´def:verification:invariant-error´`.
//! - [`signer`] — signer abstraction `´def:verification:signer-set´`.
//! - [`history`] — transition kinds, root edges, canonical deltas,
//!   transition certificates, derived projections, and canonical chain
//!   history `´def:verification:branch-kind´` through `´def:verification:history´`.
//! - [`world`] — root cursors, wallets, adversarial environment, and the
//!   complete pure `World` `´def:verification:root-cursor´`,
//!   `´def:verification:wallets´`, `´def:verification:external-budget´`,
//!   `´def:verification:world´`.
//! - [`recognition`] — state projection classifier and branch-specific
//!   validators/readers `´def:recognition:state-class´` through
//!   `´def:recognition:ash-view´`.
//! - [`fee`] — fee envelope `´def:auction:fee-envelope´`,
//!   `´rule:auction:fee-envelope-validation´`.
//! - `kernel` (crate-private) — output staging, exact canonical-flow and issuance
//!   declarations, the transaction builder, conservation validators,
//!   exact witness partition, atomic commit, and root-cursor update
//!   `´def:verification:transition-builder´` through
//!   `´rule:verification:root-cursor-update´`.
//! - `certify` (crate-private) — transition-certificate derivation
//!   `´rule:verification:derive-transition-certificate´` and its helper
//!   rules (root shapes, root edges, canonical deltas, projections,
//!   data-output validation).
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
//! - **Evidence rule.** A world is *model-valid evidence* only when it
//!   was produced by [`genesis`](fn@genesis::genesis) followed by a
//!   chain of successful [`execute`] calls. Any other world — direct construction, public
//!   field mutation, raw `apply` on corrupted state — is corruption
//!   evidence for fault/differential testing and must be labeled as
//!   such by the harness that produced it.
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

pub mod asset;
pub mod constants;
pub mod fee;
pub mod guard;
pub mod history;
pub mod object;
pub mod pool;
pub mod queries;
pub mod recognition;
pub mod scalar;
pub mod signer;
pub mod world;

pub use asset::{Asset, Maturity, ReceiptClass};
pub use constants::Constants;
pub use fee::{FeeChange, FeeEnvelope, validate_fee_envelope};
pub use guard::{Guard, InvariantError};
pub use history::{
    BranchKind, BurnProjection, BurnRecord, CanonicalDelta, ClearProjection, DeltaKind,
    DistributionResidueProjection, GenesisProjection, History, OpenFlowKind, OpenFlowProjection,
    RootEdge, TransitionCertificate,
};
pub use object::{DataOutput, Meta, Tag, Utxo};
pub use pool::PoolState;
pub use queries::{cycle_issuance_query, floor_terms, redemption_payout};
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
pub use signer::{SignerSet, require_signer};
pub use world::{ExternalBudget, RootCursor, Wallets, World};
