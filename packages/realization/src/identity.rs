//! Stable first-party semantic keys.
//!
//! These values are complete typed keys. They are not graph positions,
//! source locations, declaration ordinals, target identities, matrix
//! positions, or backend handles.

use architecture::{AssetId, BoundId, ObjectId, OperationId, ProjectionId, RootId};

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

    /// Owners committed by one input family.
    InputOwners {
        operation: OperationId,
        object: ObjectId,
    },

    /// Signers presented for one operation.
    Signers { operation: OperationId },

    /// Presence of one derived projection.
    ProjectionPresent {
        operation: OperationId,
        projection: ProjectionId,
    },

    /// Runtime value assigned to an architecture-owned cardinality
    /// bound.
    BoundValue { bound: BoundId },

    /// Whether every observed member of one object family passed
    /// architecture-owned object/asset recognition.
    ///
    /// The later model adapter derives this from primitive observed
    /// objects. It must not accept a caller-authored assertion as
    /// evidence.
    FamilyRecognized {
        operation: OperationId,
        side: TransactionSide,
        object: ObjectId,
    },

    /// Whether one declared sponsor region is isolated from protocol
    /// value.
    ///
    /// As with `FamilyRecognized`, the observation adapter derives
    /// this from primitive flows and object families rather than
    /// accepting a pre-decided boolean from an untrusted caller.
    SponsorIsolated { operation: OperationId },

    /// Whether the operation used any owner or operator secret.
    ///
    /// Used by constructibility relations for permissionless
    /// operations. Sponsor-owner witnesses are modeled separately.
    ProtocolSecretUsed { operation: OperationId },
}

/// Semantic relation family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationKind {
    Cardinality,
    Recognition,
    Authorization,
    Conservation,
    ClassClosure,
    AllowedObjectFamilies,
    CanonicalDeltaPolicy,
    OpenFlowPolicy,
    SponsorEnvelopeMultiplicity,
    SponsorIsolation,
    RootPolicy,
    ProjectionPolicy,
    Constructibility,
    Lifecycle,
    Representation,
    SubstrateConservation,
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

/// Semantic role of an expression owned by one relation.
///
/// Roles are semantic names, not arena positions. Adding an unrelated
/// expression therefore does not renumber existing expression IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressionRole {
    /// Final boolean predicate of the relation.
    Predicate,

    /// Lower cardinality or value bound.
    Minimum,

    /// Upper cardinality or value bound.
    Maximum,

    /// Exact expected value.
    Expected,

    /// Aggregate input-side term.
    InputTotal,

    /// Aggregate output-side term.
    OutputTotal,

    /// Activation condition of a conditional relation.
    Condition,

    /// Authorization requirement.
    RequiredOwners,

    /// Presented authorization evidence.
    PresentedSigners,
}

/// Stable identity of one expression.
///
/// A primitive fact expression uses the fact's complete key directly.
/// Relation-owned constants and derived expressions use the complete
/// relation key plus a typed semantic role.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExprId {
    Fact(FactId),

    Relation {
        relation: RelationId,
        role: ExpressionRole,
    },
}

impl ExprId {
    /// Stable ID of a primitive fact expression.
    #[must_use]
    pub const fn fact(fact: FactId) -> Self {
        Self::Fact(fact)
    }

    /// Stable ID of a relation-owned expression.
    #[must_use]
    pub const fn relation(relation: RelationId, role: ExpressionRole) -> Self {
        Self::Relation { relation, role }
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
    /// Exact whole-transaction substrate value conservation.
    ///
    /// Not `PublicArithmetic`: the realization boundary has erased the
    /// sponsor values a public computation would need. Not
    /// `ConfidentialConservation` either: explicit and confidential
    /// L-BTC both rely on target-wide substrate conservation while
    /// their target mechanisms differ. The compiler maps this abstract
    /// requirement to model-kernel exact conservation for model
    /// evidence or to a reviewed exact target proof.
    SubstrateConservation,
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
