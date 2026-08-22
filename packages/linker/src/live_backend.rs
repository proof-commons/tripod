//! The backend-owned types this crate's live-transfer API hands out.
//!
//! The same narrow fix [`crate::backend`] makes for the compact-ASH
//! surface, made once more for the live-transfer one, and for the same
//! reason. Most of §11's public surface is expressed in the backend's
//! vocabulary — [`crate::CandidateLinkedLiveTransferBundle::constructors`]
//! is keyed by an owner and a [`LiveTransferRepresentationPlan`],
//! [`crate::LiveAbiHandoff::family_ranges`] is keyed by
//! [`LiveTransferShape`] and valued by [`CompleteFamilyRanges`], and its
//! witness census is keyed by [`LiveTransferLeafRole`] — and a
//! downstream crate cannot name a type whose defining package it does
//! not depend on.
//!
//! Kept separate from [`crate::backend`] rather than appended to it,
//! because the two surfaces are separate: a consumer of the compact-ASH
//! projection has no business naming a live-transfer shape, and a module
//! that published both would let it.
//!
//! Nothing new is published here. Every item is already reachable
//! through this crate's public API; this module only gives it a name.

pub use tapscript::upstream::{
    ExternalEvidenceRole, LiveTransferRepresentationPlan, ValidatedLiveTransferOperationPlan,
};
pub use tapscript::{
    CandidateTransferLifecycle, CompleteFamilyRanges, LiveFamily, LiveFamilyRange, LiveInputFamily,
    LiveOutputFamily, LiveProgramRole, LiveTransferLeafRole, LiveTransferShape,
    LiveTransferShapeBounds, LiveTransferShapeSet, OwnerKey, OwnerProfileDisposition,
    OwnerSighashProfile, ProtectedDatum, RecognitionResidual,
};
