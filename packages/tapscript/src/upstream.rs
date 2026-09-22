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
//!
//! # Guide 14's maturity half
//!
//! The maturity link binds two upstream values, and each of them
//! arrives as the type that validated it.
//! [`ValidatedMaturityAnnouncementOperationPlan`] is already a
//! parameter of this crate's own
//! [`crate::assess_maturity_announcement_program`] and
//! [`crate::project_maturity_carriers`], so a consumer of those
//! entries needs its name for the reason the compact half needed
//! [`ValidatedTargetOperationPlan`]. [`AnnouncementLeadBounds`] is the
//! realization's validated lead window: its constructor refuses a zero
//! minimum and an inverted pair, and its architecture-keyed resolution
//! refuses a magnitude the architecture does not carry, so a consumer
//! that restated the window as two integers would be discarding those
//! checks and keeping only their result. [`Cycle`] is named one step
//! down for the same reason — the window's accessors return it, and a
//! consumer that cannot write down a cycle cannot write down what the
//! window handed it.
//!
//! [`StateSingletonDeclaration`] is named for the third: the maturity
//! link takes the singleton's declared amount as the type that validated
//! it, resolved from the asset declaration that fixes the whole issuance
//! and forbids reissuance, rather than as an integer a deployment
//! supplies.
//!
//! [`StateMetadata`] is the semantic metadata
//! [`crate::CandidateStateConstructor::derive`] takes, so a consumer
//! that applies a constructor cannot write down what it must supply
//! without the name. [`EncodedStateMetadata`] is what
//! [`crate::CandidateStateConstructor::encoded_metadata`] hands back,
//! and a consumer retaining a concrete constructor beside its exact
//! metadata and nonce stores one. [`MaturityAnnouncementLifecycleClosure`]
//! is what [`ValidatedMaturityAnnouncementOperationPlan::lifecycle`]
//! returns, and a consumer that could not name it could not state the
//! exit obligations the plan leaves outstanding.
//!
//! # The carrier comparison's half
//!
//! A consumer comparing the plan with what a backend emitted writes down
//! one row per relation, per representation and per execution case, so
//! it needs the names of the five things such a row is made of.
//! [`MaturityAnnouncementRepresentationPlan`] keys the row, because the
//! comparison is per representation and a consumer that could not name a
//! mode would have to compare the union of both.
//! [`MaturityAnnouncementRepresentationProjection`] is what one of those
//! keys resolves to, and a function taking one as a parameter cannot be
//! written without its name. [`RelationId`] is the row's subject:
//! Guide-12 §1.3 requires the plan's relation census to be exactly equal
//! to the emitted one, and an equality between censuses whose members
//! cannot be named is not one anyone can check.
//! [`TargetRelationCase`] carries the per-case half of a relation's
//! discharge, [`RelationActivity`] is that case's vacuity disposition,
//! and [`DischargeBoundary`] is where the requirement is discharged — a
//! consumer that cannot write down a boundary cannot say which side of
//! one a relation was discharged on, which is the whole content of the
//! comparison. [`ExternalEvidenceRequirement`] is the exact premise an
//! external relation leaves open, named rather than projected to its
//! role, because a comparison that dropped the operation and asset it
//! carries could not tell two open premises apart.

pub use compiler::live_transfer_plan::{
    LiveTransferComposition, LiveTransferRepresentationPlan, LiveTransferRepresentationProjection,
    ValidatedLiveTransferOperationPlan,
};
pub use compiler::maturity_announcement_plan::{
    MaturityAnnouncementLifecycleClosure, MaturityAnnouncementRepresentationPlan,
    MaturityAnnouncementRepresentationProjection, ValidatedMaturityAnnouncementOperationPlan,
};
pub use compiler::operation_plan::{
    AbstractCarrierRequirement, CarrierAssignmentAlternative, CarrierQuantification, CarrierRole,
    DischargeBoundary, ExecutionCaseId, LifecycleRequirement, PlacedCarrier, RelationActivity,
    RelationCaseKey, RequiredCapability, RequiredSourceKind, TargetExecutionCase,
    TargetLifecycleStatus, TargetRelationCase, TargetRelationRequirement,
    ValidatedTargetOperationPlan,
};
pub use compiler::target::{ExternalEvidenceRole, RequiredCapability as TargetRequiredCapability};
pub use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, ExternalEvidenceRequirement, RelationId,
    STATE_METADATA_LAYOUT, StateMetadata, StateMetadataLayoutRow, StateMetadataRegionClass,
    StateSingletonDeclaration,
};
