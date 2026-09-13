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

/// One predecessor STATE metadata field read on the STATE input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateField {
    /// Pool backing quantity.
    Omega,
    /// Live receipt quantity.
    YL,
    /// Time-locked receipt quantity.
    YT,
    /// Pending entitlement quantity.
    Q,
    /// Current cycle ordinal.
    Cycle,
    /// Maturity status.
    Maturity,
}

impl StateField {
    /// Every field, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::Omega,
        Self::YL,
        Self::YT,
        Self::Q,
        Self::Cycle,
        Self::Maturity,
    ];

    /// Return the stable kebab-case name of this field.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Omega => "omega",
            Self::YL => "y-l",
            Self::YT => "y-t",
            Self::Q => "q",
            Self::Cycle => "cycle",
            Self::Maturity => "maturity",
        }
    }
}

/// One consensus lead input to an announcement observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnnouncementLeadBound {
    /// Minimum announcement lead.
    Minimum,
    /// Maximum announcement lead.
    Maximum,
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

    /// One predecessor metadata field read on the STATE input.
    /// The STATE family amount remains PID value, not metadata.
    StateField {
        operation: OperationId,
        field: StateField,
    },

    /// Announced cycle carried by the public announcement request.
    RequestedAnnouncementCycle { operation: OperationId },

    /// Consensus lead input carried by the observation, not an
    /// architecture-owned cardinality bound.
    AnnouncementLead {
        operation: OperationId,
        bound: AnnouncementLeadBound,
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
///
/// # Every member has a body
///
/// A member of this vocabulary names a family of `Relation` bodies, and
/// the owner validator derives the member from the body. A member with
/// no body variant could therefore only ever be declared by a relation
/// that means something else, so members are added with their bodies
/// and removed when they lose them
/// `(´[PLAN-rule:guide10:relation-identity]´)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationKind {
    Cardinality,
    Recognition,
    Authorization,
    Conservation,
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
    /// A relation whose content is one owned boolean expression.
    ///
    /// The body already existed; the identity did not, so an
    /// expression-bearing relation had to borrow a kind describing
    /// something else. It is named here so that it can be declared
    /// honestly, not because expression-bearing production scope has
    /// begun.
    ExpressionPredicate,
}

/// Typed subject distinguishing relations of one family.
///
/// # Exactly one subject per body
///
/// The owner validator derives a relation's subject from its body as a
/// function, so each member here is the subject of some body rather
/// than one admissible presentation among several. A body constraining
/// a whole transaction side is subjected to that side; a body fixing an
/// operation-wide policy is subjected to the operation
/// `(´[PLAN-rule:guide11-exec:relation-subject]´)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationSubject {
    Operation,

    ObjectFamily {
        side: TransactionSide,
        object: ObjectId,
    },

    /// A whole transaction side.
    ///
    /// The subject of a body that constrains which families may appear
    /// on one side at all. Such a body names no single family — naming
    /// one of the families it admits would file the closure under a
    /// member of its own result — so the side itself is the subject.
    TransactionSide {
        side: TransactionSide,
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
