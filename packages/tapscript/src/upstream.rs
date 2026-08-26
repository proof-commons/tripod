//! The compiler-owned types this crate's own public API hands out.
//!
//! Not a convenience. Several of this crate's public signatures return
//! compiler types by value or by reference —
//! [`crate::CandidateRelocatableTapscriptBundle::plan`] returns a
//! [`ValidatedTargetOperationPlan`], and its placement census is keyed
//! by [`RelationCaseKey`] — and a downstream crate cannot name a type
//! whose defining package it does not depend on. Without this module a
//! consumer could call those methods but could not write down the type
//! of what it got back, so it could not store one in a field or return
//! one from a function of its own.
//!
//! Re-exporting is the narrow fix, and it is narrower than the
//! alternative: Guide-12 §14.1 admits exactly `tapscript` and
//! `target-elements` as the linker's direct dependencies, so a
//! downstream crate adding `compiler` to name these types would be
//! reaching around that boundary rather than consuming this crate's
//! projection of it.
//!
//! Nothing new is published here. Every item is already reachable
//! through this crate's public API; this module only gives it a name.
//!
//! # Guide 13's half
//!
//! [`crate::CandidateRelocatableLiveTransferBundle::plan`] returns a
//! [`ValidatedLiveTransferOperationPlan`] and
//! [`crate::CandidateRelocatableLiveTransferBundle::representation`]
//! returns a [`LiveTransferRepresentationPlan`], so the same argument
//! applies to them for the same reason. The per-representation
//! projection is named too, because §11.5's carrier closure compares
//! *per plan*: a consumer that could not write down the type of one
//! representation's projection would have to compare the union, which is
//! precisely the comparison §11.5 forbids.

pub use compiler::live_transfer_plan::{
    LiveTransferComposition, LiveTransferRepresentationPlan, LiveTransferRepresentationProjection,
    ValidatedLiveTransferOperationPlan,
};
pub use compiler::operation_plan::{
    AbstractCarrierRequirement, CarrierAssignmentAlternative, CarrierQuantification, CarrierRole,
    ExecutionCaseId, LifecycleRequirement, PlacedCarrier, RelationCaseKey, RequiredCapability,
    RequiredSourceKind, TargetExecutionCase, TargetLifecycleStatus, TargetRelationRequirement,
    ValidatedTargetOperationPlan,
};
pub use compiler::target::{ExternalEvidenceRole, RequiredCapability as TargetRequiredCapability};
