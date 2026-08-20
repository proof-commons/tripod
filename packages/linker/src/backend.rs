//! The backend-owned types this crate's own public API hands out.
//!
//! The same narrow fix `tapscript::upstream` already makes one layer
//! down, for the same reason. Most of this crate's public surface is
//! expressed in the backend's vocabulary —
//! [`crate::CandidateLinkedBundle::layouts`] is keyed by
//! [`CompactAshShape`] and valued by [`ConcreteLayout`], its programs
//! and witness roles are keyed by [`LeafRole`], and its definition
//! census is keyed by [`BundleSymbol`] — and a downstream crate cannot
//! name a type whose defining package it does not depend on.
//!
//! Re-exporting is narrower than the alternative. Guide-12 §15.1 names
//! `linker` and `target-elements` as the transaction package's direct
//! dependencies; a consumer adding `tapscript` in order to write down
//! the type of a bundle accessor's return value would be reaching
//! around this crate rather than consuming its projection, and would
//! then be able to hand the transaction layer a layout that no link
//! produced.
//!
//! Nothing new is published here. Every item is already reachable
//! through this crate's public API; this module only gives it a name.

pub use tapscript::{
    AshRepresentationSelection, BundleSymbol, CandidateShapeSet, CompactAshShape,
    CompactAshShapeBounds, ConcreteLayout, ConcreteRelationPlacement, ConstructorAssumption,
    ExactTargetProjection, ExplicitValuePolicy, InputPlacement, InputRole, InternalKeyPolicy,
    KeyPathPolicy, LeafRole, OutputPlacement, OutputRole, ProgramRole, ResourceModel,
    ResourceObligation, SponsorChangePresence, StackItem, TapscriptProgram, WitnessComponent,
    WitnessRole,
};
