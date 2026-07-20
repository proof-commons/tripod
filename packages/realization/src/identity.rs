//! Stable first-party semantic keys.
//!
//! These values are complete typed keys. They are not graph positions,
//! source locations, declaration ordinals, or target identities.

use architecture::{AssetId, BoundId, ObjectId, OperationId, ProjectionId, RootId};

use crate::RepresentationMode;

/// Input or output side of a transaction family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionSide {
    Input,
    Output,
}

/// Stable identity of one primitive semantic observation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FactId {
    /// Cardinality of one declared object family.
    FamilyCount {
        operation: OperationId,
        side: TransactionSide,
        object: ObjectId,
    },

    /// Aggregate semantic amount of one declared object family.
    FamilyAmount {
        operation: OperationId,
        side: TransactionSide,
        object: ObjectId,
    },

    /// Owner set carried by one input family.
    InputOwners {
        operation: OperationId,
        object: ObjectId,
    },

    /// Signer set presented for one operation.
    Signers { operation: OperationId },

    /// Presence of one derived projection.
    ProjectionPresent {
        operation: OperationId,
        projection: ProjectionId,
    },

    /// Runtime value assigned to an architecture-owned bound.
    BoundValue { bound: BoundId },
}

/// Semantic relation family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationKind {
    Cardinality,
    Recognition,
    Authorization,
    Conservation,
    ClassClosure,
    OutputClosure,
    SponsorIsolation,
    RootPolicy,
    ProjectionPolicy,
    Constructibility,
    Lifecycle,
    Representation,
}

/// Typed subject distinguishing relations of one family.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationSubject {
    Operation,

    ObjectFamily {
        side: TransactionSide,
        object: ObjectId,
    },

    Asset {
        asset: AssetId,
    },

    Root {
        root: RootId,
    },

    Projection {
        projection: ProjectionId,
    },

    Sponsor,

    LifecycleExit {
        object: ObjectId,
        exit: OperationId,
    },

    Representation {
        object: ObjectId,
        mode: RepresentationMode,
    },
}

/// Stable identity of one target-independent semantic relation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationId {
    operation: OperationId,
    kind: RelationKind,
    subject: RelationSubject,
}

impl RelationId {
    /// Construct a complete typed relation key.
    #[must_use]
    pub const fn new(operation: OperationId, kind: RelationKind, subject: RelationSubject) -> Self {
        Self {
            operation,
            kind,
            subject,
        }
    }

    /// Owning architecture operation.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }

    /// Relation family.
    #[must_use]
    pub const fn kind(&self) -> RelationKind {
        self.kind
    }

    /// Typed relation subject.
    #[must_use]
    pub const fn subject(&self) -> &RelationSubject {
        &self.subject
    }
}

/// Target-independent proof family approved by realization semantics.
///
/// This is not a target instruction or backend pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProofKind {
    PublicArithmetic,
    ConfidentialConservation,
    SignerMembership,
    ManifestShape,
    PublicConstructibility,
}

/// Stable proof-alternative key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProofAlternativeId {
    relation: RelationId,
    proof: ProofKind,
}

impl ProofAlternativeId {
    /// Construct one proof-alternative key.
    #[must_use]
    pub const fn new(relation: RelationId, proof: ProofKind) -> Self {
        Self { relation, proof }
    }

    /// Relation discharged by this alternative.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    /// Target-independent proof family.
    #[must_use]
    pub const fn proof(&self) -> ProofKind {
        self.proof
    }
}
