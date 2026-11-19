//! Typed relation operands and authenticatable source requirements
//! (Guide-3 Tranche C).
//!
//! A relation cannot be planned safely from its ID and proof kind
//! alone: the planner must know which facts each proof consumes and
//! how those facts are authenticated. This module derives, for every
//! relation and approved proof alternative, the exact operand census
//! and one source-requirement row per operand — typed, canonically
//! sorted, and free of any sponsor amount: sponsor erasure means the
//! value is absent, not merely secret, so an amount operand over the
//! sponsor family is a structural compile error.

use realization::{AvailabilityClass, RepresentationMode};

/// Closed semantic fields of Burn and Clear observations. Values such as a
/// record ordinal or amount are operands, never part of this identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BurnClearCoordinate {
    DataOutputSchema(architecture::DataOutputKind),
    DataOutputFamilyCount(architecture::DataOutputKind),
    DataOutputTag(architecture::DataOutputKind),
    DataOutputAsset(architecture::DataOutputKind),
    DataOutputCondition(architecture::DataOutputKind),
    RecordOrdinal,
    RecordAddress,
    DataOutputScalar(architecture::DataOutputKind),
    RecordFreshAsh,
    DestructionFlow,
    TransactionContext,
    CertificateContext,
    BurnEventType,
    ClearEventType,
    BurnFreshAmountEvent,
    ClearDestroyedAmountEvent,
    ClearOmegaEvent,
    ClearResidualEvent,
}

impl BurnClearCoordinate {
    /// Stable, injective subject code, separate from realization fact codes.
    #[must_use]
    pub const fn subject_code(self) -> [u16; 2] {
        match self {
            Self::DataOutputSchema(kind) => [13, kind.code()],
            Self::DataOutputFamilyCount(kind) => [14, kind.code()],
            Self::DataOutputTag(kind) => [15, kind.code()],
            Self::DataOutputAsset(kind) => [16, kind.code()],
            Self::DataOutputCondition(kind) => [17, kind.code()],
            Self::RecordOrdinal => [18, 0],
            Self::RecordAddress => [19, 0],
            Self::DataOutputScalar(kind) => [20, kind.code()],
            Self::RecordFreshAsh => [21, 0],
            Self::DestructionFlow => [22, 0],
            Self::TransactionContext => [23, 0],
            Self::CertificateContext => [24, 0],
            Self::BurnEventType => [25, 0],
            Self::ClearEventType => [26, 0],
            Self::BurnFreshAmountEvent => [27, 0],
            Self::ClearDestroyedAmountEvent => [28, 0],
            Self::ClearOmegaEvent => [29, 0],
            Self::ClearResidualEvent => [30, 0],
        }
    }
}

/// Stable identity of one relation operand.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperandId {
    relation: realization::RelationId,
    role: OperandRole,
}

impl OperandId {
    #[must_use]
    pub const fn new(relation: realization::RelationId, role: OperandRole) -> Self {
        Self { relation, role }
    }

    #[must_use]
    pub const fn relation(&self) -> &realization::RelationId {
        &self.relation
    }

    #[must_use]
    pub const fn role(&self) -> &OperandRole {
        &self.role
    }
}

/// Compiler-owned semantic operand role. Never a target position.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperandRole {
    ObjectFamilyMembers {
        side: realization::TransactionSide,
        object: architecture::ObjectId,
    },
    ObjectFamilyCount {
        side: realization::TransactionSide,
        object: architecture::ObjectId,
    },
    ObjectFamilyAmount {
        side: realization::TransactionSide,
        object: architecture::ObjectId,
    },
    ObjectFamilyOwners {
        object: architecture::ObjectId,
    },

    ProtocolSignerSet,
    SponsorSignerSet,
    SponsorRegion,

    CanonicalPartition,
    OpenFlowSet,
    RootEffects,
    ProjectionSet,

    RuntimeBound {
        bound: architecture::BoundId,
    },

    Representation {
        object: architecture::ObjectId,
    },

    ConstructibilityCase,
    LifecycleGraph,

    ExpressionResult {
        expression: realization::ExprId,
    },

    ExternalEvidence {
        requirement: realization::ExternalEvidenceRequirement,
    },

    /// One Burn/Clear primitive or report anchor, without a target position.
    BurnClear(BurnClearCoordinate),
}

/// How one operand's source must be authenticated. The source proof
/// obligation, not proof completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequiredSourceKind {
    AuthenticatedInputObject,
    AuthenticatedOutputObject,
    AuthenticatedFamilyCensus,
    AuthenticatedConsensusValue,
    AuthenticatedCommitmentRelation,
    AuthenticatedTransitionCertificate,

    RuntimeArchitectureBound,
    DerivedExpression,

    InputOwnerWitness,
    OperatorWitness,
    RefundKeyWitness,
    SponsorLocalWitness,

    PublicConstructionData,
    ExternalEvidence,
}

/// When one source requirement is active.
///
/// A representation condition names the object it belongs to as well as
/// the mode. An execution case fixes a mode per object, so a condition
/// carrying a mode alone would be answered by any object that happened
/// to select it — a requirement about one object family would activate
/// because an unrelated family was committed the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequirementActivation {
    Always,
    WhenSponsorPresent,
    WhenRepresentation {
        object: architecture::ObjectId,
        mode: RepresentationMode,
    },
    WhenSideRepresentation {
        side: realization::TransactionSide,
        object: architecture::ObjectId,
        mode: RepresentationMode,
    },
}

/// One canonical source-requirement row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRequirement {
    pub operand: OperandId,
    pub source: RequiredSourceKind,
    pub availability: AvailabilityClass,
    pub activation: RequirementActivation,
}

/// True for the erased sponsor region's amount: structurally absent
/// from the compiler, never an allowable private fact.
///
/// Tests the object family alone, which is sound only because
/// [`crate::sponsor_region::validate_decidable_sponsor_regions`] refuses
/// every scope in which ordinary L-BTC is claimed by a protocol flow.
/// Inside an accepted analysis the family and the sponsor region
/// coincide, so this is the erased region and not merely a family that
/// sometimes holds it. Lifting that gate means giving this predicate the
/// operation's role, because a protocol-region L-BTC amount is a readable
/// amount that protocol relations are entitled to require.
pub const fn is_sponsor_amount_operand(role: &OperandRole) -> bool {
    matches!(
        role,
        OperandRole::ObjectFamilyAmount {
            object: crate::sponsor_region::ORDINARY_LBTC,
            ..
        }
    )
}

// These vocabulary interpreters belong to the statically retained attestation
// owner. The source census algorithm is shared through its lowering wrapper.
pub use crate::engine_attestation::source::{
    derive_source_requirements, proof_capabilities, proof_external_evidence, relation_operands,
};
