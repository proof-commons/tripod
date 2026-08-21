//! The complete vector matrix of Guide-12 §18.
//!
//! §18 names every vector class the first compact-ASH bundle report must
//! carry. §1.11 forbids truncating that matrix, which is only checkable
//! if the matrix is a countable object rather than a reading of prose.
//! This module is that object: one entry per named class, each carrying
//! the three facts a later wave cannot infer on its own — whether the
//! class expects acceptance, which layer the mutation is applied at, and
//! which layer is expected to produce the verdict.
//!
//! The last of those is the load-bearing one. §1.5 keeps construction
//! failure and target rejection distinct, and a class that does not say
//! in advance which boundary it expects can be "passed" by a failure at
//! any boundary at all. Recording the expectation here, before anything
//! executes, is what makes Wave 11 and Wave 12 falsifiable rather than
//! self-confirming.
//!
//! Nothing in this module observes anything. It is a specification
//! transcribed into types, and it deliberately carries no fixture, no
//! bundle, no bytes, and no verdict.

use core::fmt::{self, Display};

/// The §18 table a class is drawn from.
///
/// The families partition the matrix exactly: every class belongs to one
/// table, and the per-family counts recomputed in this module's tests
/// must sum to the whole.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VectorFamily {
    /// §18.1 — positive semantic cases.
    PositiveSemantic,
    /// §18.2 — cardinality mutations.
    Cardinality,
    /// §18.3 — input mutations.
    Input,
    /// §18.4 — output mutations.
    Output,
    /// §18.5 — canonical-partition mutations.
    CanonicalPartition,
    /// §18.6 — authorization and constructibility mutations.
    Constructibility,
    /// §18.7 — sponsor mutations.
    Sponsor,
    /// §18.8 — root and projection mutations.
    Absence,
    /// §18.9 — representation mutations.
    Representation,
    /// §18.10 — constructor and linker mutations.
    Linker,
    /// §18.11 — ABI mutations.
    Abi,
    /// §18.12 — resource and infrastructure cases.
    Resource,
}

impl VectorFamily {
    /// Every family, in §18 order.
    pub const ALL: &'static [Self] = &[
        Self::PositiveSemantic,
        Self::Cardinality,
        Self::Input,
        Self::Output,
        Self::CanonicalPartition,
        Self::Constructibility,
        Self::Sponsor,
        Self::Absence,
        Self::Representation,
        Self::Linker,
        Self::Abi,
        Self::Resource,
    ];

    /// The §18 subsection this family transcribes.
    #[must_use]
    pub const fn section(self) -> &'static str {
        match self {
            Self::PositiveSemantic => "18.1",
            Self::Cardinality => "18.2",
            Self::Input => "18.3",
            Self::Output => "18.4",
            Self::CanonicalPartition => "18.5",
            Self::Constructibility => "18.6",
            Self::Sponsor => "18.7",
            Self::Absence => "18.8",
            Self::Representation => "18.9",
            Self::Linker => "18.10",
            Self::Abi => "18.11",
            Self::Resource => "18.12",
        }
    }
}

impl Display for VectorFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.section())
    }
}

/// What a class expects the run to establish.
///
/// Three outcomes, not two. `(´[PLAN-rule:vectors:execution]´)`
/// forbids counting an
/// infrastructure failure as an expected rejection, so a class whose
/// subject *is* the infrastructure needs its own polarity rather than a
/// negative one with a caveat attached.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VectorPolarity {
    /// The complete transaction is expected to be accepted, and its
    /// projection to match the expected compact-ASH projection.
    Positive,
    /// The run is expected to be refused at the class's stated boundary.
    Negative,
    /// The class's subject is the executor or transport, and the run is
    /// expected to produce a typed infrastructure failure and no target
    /// verdict at all.
    Infrastructure,
}

impl VectorPolarity {
    /// Every polarity.
    pub const ALL: &'static [Self] = &[Self::Positive, Self::Negative, Self::Infrastructure];

    /// Whether a class of this polarity may contribute a target verdict.
    ///
    /// False for [`Self::Infrastructure`], which is the whole point of
    /// giving it a polarity of its own.
    #[must_use]
    pub const fn yields_target_verdict(self) -> bool {
        matches!(self, Self::Positive | Self::Negative)
    }
}

/// The layer a mutation is applied at, per
/// `(´[PLAN-rule:vectors:mutations]´)`.
///
/// Positive classes mutate nothing and carry `None` instead.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MutationLayer {
    /// A semantic fact of the request or the predecessor world.
    SemanticFact,
    /// The ABI's classification or role layout.
    AbiLayout,
    /// A field of the concrete target transaction.
    TargetTransaction,
    /// A witness item, signature, control block, or proof.
    WitnessProof,
    /// The linked constructor, an emitted program, or the taptree.
    LinkedConstructorProgram,
}

impl MutationLayer {
    /// Every mutation layer.
    pub const ALL: &'static [Self] = &[
        Self::SemanticFact,
        Self::AbiLayout,
        Self::TargetTransaction,
        Self::WitnessProof,
        Self::LinkedConstructorProgram,
    ];
}

/// The layer a class expects to produce its verdict, per §1.5.
///
/// §1.5 enumerates exactly these eleven and forbids inferring one from
/// what a test hoped for. A class names its boundary in advance, and a
/// run that refuses at a different one is a finding, not a pass.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceBoundary {
    /// The typed semantic request is refused.
    SemanticRequestRejection,
    /// The compiler refuses to produce a plan.
    CompilerPlanRejection,
    /// The backend refuses to emit.
    BackendEmissionRejection,
    /// The linker refuses to link.
    LinkerRejection,
    /// The ABI or the constructor refuses the shape.
    AbiConstructionRejection,
    /// The executor or its transport fails; no target verdict exists.
    ExecutorInfrastructureFailure,
    /// The target refuses before any script runs.
    ConsensusRejectionBeforeScript,
    /// The covenant script path refuses.
    ScriptPathRejection,
    /// The target accepts for consensus and relay policy refuses.
    RelayPolicyRejection,
    /// The target accepts the complete transaction.
    AcceptedTransaction,
    /// The target accepts and the report layer refuses the projection.
    ReportSemanticProjectionRejection,
}

impl EvidenceBoundary {
    /// Every boundary, in §1.5 order.
    pub const ALL: &'static [Self] = &[
        Self::SemanticRequestRejection,
        Self::CompilerPlanRejection,
        Self::BackendEmissionRejection,
        Self::LinkerRejection,
        Self::AbiConstructionRejection,
        Self::ExecutorInfrastructureFailure,
        Self::ConsensusRejectionBeforeScript,
        Self::ScriptPathRejection,
        Self::RelayPolicyRejection,
        Self::AcceptedTransaction,
        Self::ReportSemanticProjectionRejection,
    ];

    /// Whether reaching this boundary required a complete target
    /// transaction to have been materialized and executed.
    ///
    /// §1.5's core distinction: a claim that the target refused needs
    /// something for the target to have refused.
    #[must_use]
    pub const fn requires_target_execution(self) -> bool {
        matches!(
            self,
            Self::ConsensusRejectionBeforeScript
                | Self::ScriptPathRejection
                | Self::RelayPolicyRejection
                | Self::AcceptedTransaction
                | Self::ReportSemanticProjectionRejection
        )
    }

    /// Whether this boundary is a pre-target refusal.
    #[must_use]
    pub const fn is_pre_target(self) -> bool {
        matches!(
            self,
            Self::SemanticRequestRejection
                | Self::CompilerPlanRejection
                | Self::BackendEmissionRejection
                | Self::LinkerRejection
                | Self::AbiConstructionRejection
        )
    }
}

/// One named class of the §18 matrix.
///
/// The name is the guide's own wording, normalized to an identifier, so
/// that a reader can check the transcription against §18 line by line
/// without trusting a paraphrase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VectorClass {
    family: VectorFamily,
    name: &'static str,
    polarity: VectorPolarity,
    mutation: Option<MutationLayer>,
    boundary: EvidenceBoundary,
}

impl VectorClass {
    const fn new(
        family: VectorFamily,
        name: &'static str,
        polarity: VectorPolarity,
        mutation: Option<MutationLayer>,
        boundary: EvidenceBoundary,
    ) -> Self {
        Self {
            family,
            name,
            polarity,
            mutation,
            boundary,
        }
    }

    /// The §18 table this class comes from.
    #[must_use]
    pub const fn family(&self) -> VectorFamily {
        self.family
    }

    /// The class's stable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What the class expects the run to establish.
    #[must_use]
    pub const fn polarity(&self) -> VectorPolarity {
        self.polarity
    }

    /// The layer the class's mutation is applied at, if it mutates.
    #[must_use]
    pub const fn mutation(&self) -> Option<MutationLayer> {
        self.mutation
    }

    /// The layer the class expects to produce its verdict.
    #[must_use]
    pub const fn boundary(&self) -> EvidenceBoundary {
        self.boundary
    }
}

impl Display for VectorClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.family.section(), self.name)
    }
}

use EvidenceBoundary as B;
use MutationLayer as L;
use VectorFamily as F;
use VectorPolarity as P;

const fn positive(family: VectorFamily, name: &'static str) -> VectorClass {
    VectorClass::new(family, name, P::Positive, None, B::AcceptedTransaction)
}

const fn negative(
    family: VectorFamily,
    name: &'static str,
    mutation: MutationLayer,
    boundary: EvidenceBoundary,
) -> VectorClass {
    VectorClass::new(family, name, P::Negative, Some(mutation), boundary)
}

const fn infrastructure(family: VectorFamily, name: &'static str) -> VectorClass {
    VectorClass::new(
        family,
        name,
        P::Infrastructure,
        None,
        B::ExecutorInfrastructureFailure,
    )
}

/// §18.1 — positive semantic cases.
pub const POSITIVE_SEMANTIC: &[VectorClass] = &[
    positive(F::PositiveSemantic, "minimum-two-ash-inputs"),
    positive(F::PositiveSemantic, "three-ash-inputs"),
    positive(F::PositiveSemantic, "candidate-maximum"),
    positive(F::PositiveSemantic, "values-at-fixed-width-boundaries"),
    positive(
        F::PositiveSemantic,
        "values-summing-to-two-pow-51-minus-one",
    ),
    positive(F::PositiveSemantic, "canonical-input-order-normalization"),
    positive(F::PositiveSemantic, "sponsorless-consensus-transaction"),
    positive(F::PositiveSemantic, "sponsored-transaction"),
    positive(F::PositiveSemantic, "one-sponsor-input"),
    positive(F::PositiveSemantic, "multiple-sponsor-inputs"),
    positive(F::PositiveSemantic, "sponsor-change-present"),
    positive(F::PositiveSemantic, "sponsor-change-absent"),
    positive(F::PositiveSemantic, "exact-successor-amount"),
    positive(F::PositiveSemantic, "repeated-execution-equal-report-bytes"),
];

/// §18.2 — cardinality mutations.
pub const CARDINALITY: &[VectorClass] = &[
    negative(
        F::Cardinality,
        "zero-ash-inputs",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Cardinality,
        "one-ash-input",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Cardinality,
        "one-above-candidate-ash-maximum",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Cardinality,
        "zero-ash-outputs",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Cardinality,
        "two-ash-outputs",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Cardinality,
        "sponsor-count-above-candidate-maximum",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Cardinality,
        "two-sponsor-change-outputs",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Cardinality,
        "duplicate-target-fee-roles",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Cardinality,
        "missing-required-target-fee-role",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Cardinality,
        "unexpected-target-only-output",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
];

/// §18.3 — input mutations.
pub const INPUT: &[VectorClass] = &[
    negative(
        F::Input,
        "duplicate-ash-outpoint",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Input,
        "outpoint-claimed-as-ash-and-sponsor",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Input,
        "wrong-asset-at-one-ash-input",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "correct-asset-under-wrong-constructor",
        L::LinkedConstructorProgram,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "correct-constructor-under-wrong-representation",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "foreign-open-asset-with-ash-shaped-metadata",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "unrecognized-closed-asset-input",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "sponsor-member-in-ash-range",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "ash-member-in-sponsor-suffix",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "noncanonical-ash-ordering",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "wrong-coordinator",
        L::LinkedConstructorProgram,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "two-coordinator-leaves",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "no-coordinator",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "member-leaf-at-input-zero",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Input,
        "coordinator-leaf-at-member-index",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
];

/// §18.4 — output mutations.
pub const OUTPUT: &[VectorClass] = &[
    // The mutation takes one unit off the successor and leaves the
    // closed asset short of what the inputs carry — its own module
    // already says `preserves_value_balance` is false for this arm,
    // and Elements checks per-asset conservation before it runs any
    // script. So the covenant is never reached: block validation
    // answers `bad-txns-in-ne-out` at the consensus layer before a
    // script-path verdict could exist, which is what the live run
    // observed verbatim. Section 19.2 requires the intended carrier to
    // have executed for a case to discharge its relation, and the
    // covenant script is that carrier here, so an arm answered before
    // it runs answers its class without discharging the conservation
    // requirement — which stays outstanding, as Wave 13d recorded when
    // it found the arm's script-path claim wrong.
    negative(
        F::Output,
        "successor-one-below-the-sum",
        L::SemanticFact,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Output,
        "successor-one-above-the-sum",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "successor-zero",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "successor-at-or-above-two-pow-51",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "wrong-output-asset",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "confidential-closed-asset",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "wrong-constructor",
        L::LinkedConstructorProgram,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "ordinary-wallet-u-output",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "second-u-output",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "sponsor-change-carrying-u",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "fee-role-carrying-u",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Output,
        "successor-at-wrong-role",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "sponsor-change-and-fee-exchanged",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    negative(
        F::Output,
        "malformed-fee-role",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Output,
        "unclaimed-output",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
];

/// §18.5 — canonical-partition mutations.
pub const CANONICAL_PARTITION: &[VectorClass] = &[
    negative(
        F::CanonicalPartition,
        "omit-one-ash-source",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "cite-one-source-twice",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "cite-one-destination-twice",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "claim-successor-through-two-flows",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "add-a-destruction",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "add-issuance",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "change-movement-kind-to-owner-controlled-lateral",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "leave-one-canonical-object-unwitnessed",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "route-part-of-u-into-an-undeclared-object",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::CanonicalPartition,
        "shorten-successor-and-grow-another-output",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
];

/// §18.6 — authorization and constructibility mutations.
pub const CONSTRUCTIBILITY: &[VectorClass] = &[
    negative(
        F::Constructibility,
        "hidden-owner-signature",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Constructibility,
        "hidden-operator-signature",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Constructibility,
        "sponsor-input-without-valid-authorization",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Constructibility,
        "one-sponsor-authorization-omitted",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Constructibility,
        "output-set-changed-after-sponsor-signing",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Constructibility,
        "public-ash-amount-unavailable",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Constructibility,
        "public-amount-copied-from-another-object",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
    negative(
        F::Constructibility,
        "constructor-uses-creator-private-state",
        L::LinkedConstructorProgram,
        B::CompilerPlanRejection,
    ),
    negative(
        F::Constructibility,
        "permissionless-constructor-accesses-owner-state",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Constructibility,
        "protocol-path-depends-on-sponsor-amount",
        L::LinkedConstructorProgram,
        B::CompilerPlanRejection,
    ),
];

/// §18.7 — sponsor mutations.
///
/// Two entries here are positive. §1.6 requires sponsor opacity to
/// survive lowering, so changing a sponsor denomination while the
/// protocol projection stays fixed must still be *accepted* with an
/// unchanged projection; recording it as a rejection would invert the
/// property it exists to witness.
pub const SPONSOR: &[VectorClass] = &[
    negative(
        F::Sponsor,
        "sponsor-protocol-reference-overlap",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Sponsor,
        "two-sponsor-regions",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Sponsor,
        "sponsor-change-outside-its-role",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    negative(
        F::Sponsor,
        "ordinary-l-btc-substituted-for-fee-role",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Sponsor,
        "fee-role-substituted-for-sponsor-change",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Sponsor,
        "foreign-sponsor-asset",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Sponsor,
        "sponsor-member-left-unclassified",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    VectorClass::new(
        F::Sponsor,
        "sponsor-denomination-changed-with-projection-fixed",
        P::Positive,
        Some(L::TargetTransaction),
        B::AcceptedTransaction,
    ),
    negative(
        F::Sponsor,
        "zero-valued-sponsor-member-with-exact-roles",
        L::TargetTransaction,
        B::RelayPolicyRejection,
    ),
    negative(
        F::Sponsor,
        "balanced-theft-attempt",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    VectorClass::new(
        F::Sponsor,
        "confidential-sponsor-values-where-policy-permits",
        P::Positive,
        Some(L::TargetTransaction),
        B::AcceptedTransaction,
    ),
    negative(
        F::Sponsor,
        "report-publishes-sponsor-amount-or-opening",
        L::SemanticFact,
        B::ReportSemanticProjectionRejection,
    ),
];

/// §18.8 — root and projection mutations.
pub const ABSENCE: &[VectorClass] = &[
    negative(
        F::Absence,
        "add-state-input",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-resv-input",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-pace-input",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-authority-input",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-root-shaped-output",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-burn-record",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-tag-burn",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-clear-destruction",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "add-residue-output",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Absence,
        "claim-burn-projection",
        L::SemanticFact,
        B::ReportSemanticProjectionRejection,
    ),
    negative(
        F::Absence,
        "claim-clear-projection",
        L::SemanticFact,
        B::ReportSemanticProjectionRejection,
    ),
    negative(
        F::Absence,
        "omit-semantic-transition-certificate",
        L::SemanticFact,
        B::ReportSemanticProjectionRejection,
    ),
    negative(
        F::Absence,
        "introduce-false-certificate-membership",
        L::SemanticFact,
        B::ReportSemanticProjectionRejection,
    ),
];

/// §18.9 — representation mutations.
pub const REPRESENTATION: &[VectorClass] = &[
    negative(
        F::Representation,
        "private-ash-under-explicit-only-policy",
        L::SemanticFact,
        B::CompilerPlanRejection,
    ),
    negative(
        F::Representation,
        "public-committed-ash-without-authenticated-opening",
        L::SemanticFact,
        B::CompilerPlanRejection,
    ),
    negative(
        F::Representation,
        "wrong-amount-encoding",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Representation,
        "noncanonical-explicit-amount",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    negative(
        F::Representation,
        "confidential-u",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    negative(
        F::Representation,
        "representation-changed-without-plan-update",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Representation,
        "output-representation-differs-from-abi",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Representation,
        "public-fact-available-only-in-creator-memory",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
];

/// §18.10 — constructor and linker mutations.
pub const LINKER: &[VectorClass] = &[
    negative(
        F::Linker,
        "missing-coordinator-program",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "missing-member-program",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "extra-escape-leaf",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "key-path-escape",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "wrong-internal-key",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "wrong-leaf-version",
        L::LinkedConstructorProgram,
        B::ScriptPathRejection,
    ),
    negative(
        F::Linker,
        "wrong-tree-order",
        L::LinkedConstructorProgram,
        B::ScriptPathRejection,
    ),
    negative(
        F::Linker,
        "stale-constructor-from-another-candidate",
        L::LinkedConstructorProgram,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Linker,
        "unresolved-symbol",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "ambiguous-symbol",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "relocation-omitted",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "relocation-applied-twice",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    negative(
        F::Linker,
        "wrong-u-asset-substituted",
        L::LinkedConstructorProgram,
        B::ScriptPathRejection,
    ),
    negative(
        F::Linker,
        "candidate-bound-relocation-disagrees-with-abi",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Linker,
        "unique-relation-carrier-unreachable",
        L::LinkedConstructorProgram,
        B::BackendEmissionRejection,
    ),
    negative(
        F::Linker,
        "source-order-dependent-tree",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
];

/// §18.11 — ABI mutations.
pub const ABI: &[VectorClass] = &[
    negative(
        F::Abi,
        "family-overlap",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "family-gap",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "wrong-total-input-count",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "wrong-total-output-count",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    // The transaction version selects TRUC relay eligibility and
    // nothing else. No emitted program inspects it — InspectVersion sits
    // in the opcode vocabulary and the capability map and at no
    // emission site in tapscript's pattern module — no signature covers
    // the sponsorless form either, and the ABI's own type doc records
    // the choice as a policy one: selecting TopologyRestricted "is a
    // policy choice rather than a consensus requirement: consensus
    // admits the sponsorless form at either version." The relay floor
    // is no rescue either, since the vectors pay a thousand satoshis
    // against a chain run at zero min-relay fee, so no floor refuses
    // the flip to the standard version — so the target has no rule to
    // refuse it by and accepting it is correct. What fixes the field is
    // the safe constructor, which writes the ABI's own version at
    // assembly and offers no request field for another. The boundary is
    // therefore the constructor's, like wrong-sequence and the other
    // rows whose mutation only first-party code catches.
    negative(
        F::Abi,
        "wrong-transaction-version",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    // The sequence field is an ABI convention and nothing else. No
    // emitted program inspects it, no signature covers an ASH input, and
    // a final-minus-one sequence engages neither a relative timelock nor
    // replaceability — so the target has no rule to refuse it by and
    // accepting it is correct. What fixes the field is the safe
    // constructor, which writes the ABI's sequence and offers no request
    // field for another. The boundary is therefore the constructor's,
    // like the two other rows whose mutation only first-party code
    // catches.
    negative(
        F::Abi,
        "wrong-sequence",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "witness-item-reorder",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Abi,
        "control-path-from-another-program",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    negative(
        F::Abi,
        "caller-chooses-target-program",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "caller-supplies-successor-amount",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "duplicate-signing-request",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "unexpected-signature",
        L::WitnessProof,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "target-bytes-changed-after-abi-validation",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "bundle-paired-with-another-abi",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    negative(
        F::Abi,
        "raw-transaction-bypasses-safe-constructor",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
];

/// §18.12 — resource and infrastructure cases.
///
/// The eight infrastructure entries are the reason [`VectorPolarity`]
/// has three arms: `(´[PLAN-rule:vectors:execution]´)` says an
/// infrastructure
/// failure never counts as an expected rejection, so these classes must
/// be unable to contribute a target verdict by construction.
pub const RESOURCE: &[VectorClass] = &[
    positive(F::Resource, "program-at-candidate-maximum"),
    positive(F::Resource, "witness-at-candidate-maximum"),
    positive(F::Resource, "deepest-candidate-control-path"),
    positive(F::Resource, "maximum-candidate-sponsor-shape"),
    negative(
        F::Resource,
        "consensus-acceptance-with-policy-rejection",
        L::TargetTransaction,
        B::RelayPolicyRejection,
    ),
    positive(F::Resource, "policy-acceptance-where-claimed"),
    infrastructure(F::Resource, "executor-timeout"),
    infrastructure(F::Resource, "malformed-response"),
    infrastructure(F::Resource, "oversized-record"),
    infrastructure(F::Resource, "unterminated-record"),
    infrastructure(F::Resource, "wrong-environment"),
    infrastructure(F::Resource, "wrong-provenance"),
    infrastructure(F::Resource, "target-infrastructure-failure"),
    VectorClass::new(
        F::Resource,
        "resource-prediction-mismatch",
        P::Negative,
        None,
        B::ReportSemanticProjectionRejection,
    ),
    infrastructure(
        F::Resource,
        "infrastructure-response-carrying-target-observation",
    ),
];

/// Every §18 family's classes, in §18 order.
pub const FAMILIES: &[(VectorFamily, &[VectorClass])] = &[
    (F::PositiveSemantic, POSITIVE_SEMANTIC),
    (F::Cardinality, CARDINALITY),
    (F::Input, INPUT),
    (F::Output, OUTPUT),
    (F::CanonicalPartition, CANONICAL_PARTITION),
    (F::Constructibility, CONSTRUCTIBILITY),
    (F::Sponsor, SPONSOR),
    (F::Absence, ABSENCE),
    (F::Representation, REPRESENTATION),
    (F::Linker, LINKER),
    (F::Abi, ABI),
    (F::Resource, RESOURCE),
];

/// Every class of the matrix, in §18 order.
#[must_use]
pub fn all_classes() -> Vec<VectorClass> {
    FAMILIES
        .iter()
        .flat_map(|(_, classes)| classes.iter().copied())
        .collect()
}

/// The per-family class counts, recomputed from the tables themselves.
#[must_use]
pub fn family_census() -> std::collections::BTreeMap<VectorFamily, usize> {
    FAMILIES
        .iter()
        .map(|(family, classes)| (*family, classes.len()))
        .collect()
}

/// The total number of named §18 classes.
#[must_use]
pub fn class_count() -> usize {
    FAMILIES.iter().map(|(_, classes)| classes.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::{
        EvidenceBoundary, FAMILIES, MutationLayer, VectorFamily, VectorPolarity, all_classes,
        class_count, family_census,
    };
    use std::collections::BTreeSet;

    /// The per-family counts read directly off Guide-12 §18's bullet
    /// lists. Transcribed once, here, so that the tables above are
    /// checked against the guide rather than against themselves.
    const GUIDE_COUNTS: &[(VectorFamily, usize)] = &[
        (VectorFamily::PositiveSemantic, 14),
        (VectorFamily::Cardinality, 10),
        (VectorFamily::Input, 15),
        (VectorFamily::Output, 15),
        (VectorFamily::CanonicalPartition, 10),
        (VectorFamily::Constructibility, 10),
        (VectorFamily::Sponsor, 12),
        (VectorFamily::Absence, 13),
        (VectorFamily::Representation, 8),
        (VectorFamily::Linker, 16),
        (VectorFamily::Abi, 15),
        (VectorFamily::Resource, 15),
    ];

    #[test]
    fn every_family_matches_the_guide_count() {
        let census = family_census();
        assert_eq!(census.len(), VectorFamily::ALL.len());
        for (family, expected) in GUIDE_COUNTS {
            assert_eq!(
                census.get(family).copied(),
                Some(*expected),
                "family {family} disagrees with Guide-12 §{}",
                family.section()
            );
        }
    }

    #[test]
    fn the_total_census_is_exact() {
        let expected: usize = GUIDE_COUNTS.iter().map(|(_, count)| *count).sum();
        assert_eq!(expected, 153, "the guide's own bullets sum to 153");
        assert_eq!(class_count(), 153);
        assert_eq!(all_classes().len(), 153);
    }

    #[test]
    fn class_names_are_unique_within_the_whole_matrix() {
        let names: BTreeSet<&str> = all_classes().iter().map(super::VectorClass::name).collect();
        assert_eq!(
            names.len(),
            class_count(),
            "two classes share a name, so a coverage row could be double-counted"
        );
    }

    #[test]
    fn every_class_sits_in_the_family_that_lists_it() {
        for (family, classes) in FAMILIES {
            for class in *classes {
                assert_eq!(
                    class.family(),
                    *family,
                    "{class} is filed under the wrong table"
                );
            }
        }
    }

    #[test]
    fn the_polarity_census_is_exact() {
        let classes = all_classes();
        let positive = classes
            .iter()
            .filter(|class| class.polarity() == VectorPolarity::Positive)
            .count();
        let negative = classes
            .iter()
            .filter(|class| class.polarity() == VectorPolarity::Negative)
            .count();
        let infrastructure = classes
            .iter()
            .filter(|class| class.polarity() == VectorPolarity::Infrastructure)
            .count();

        // 14 from §18.1, two sponsor-opacity cases from §18.7, and five
        // resource cases from §18.12 that claim acceptance.
        assert_eq!(positive, 21);
        assert_eq!(infrastructure, 8);
        assert_eq!(negative, 124);
        assert_eq!(positive + negative + infrastructure, 153);
    }

    #[test]
    fn a_positive_class_expects_acceptance() {
        for class in all_classes() {
            if class.polarity() == VectorPolarity::Positive {
                assert_eq!(
                    class.boundary(),
                    EvidenceBoundary::AcceptedTransaction,
                    "{class} claims acceptance but names another boundary"
                );
            }
        }
    }

    #[test]
    fn an_infrastructure_class_yields_no_target_verdict() {
        for class in all_classes() {
            if class.polarity() == VectorPolarity::Infrastructure {
                assert!(!class.polarity().yields_target_verdict());
                assert_eq!(
                    class.boundary(),
                    EvidenceBoundary::ExecutorInfrastructureFailure,
                    "{class} is infrastructure but names a target boundary"
                );
                assert!(!class.boundary().requires_target_execution());
            }
        }
    }

    #[test]
    fn no_negative_class_hides_behind_the_infrastructure_boundary() {
        for class in all_classes() {
            assert!(
                !(class.polarity() == VectorPolarity::Negative
                    && class.boundary() == EvidenceBoundary::ExecutorInfrastructureFailure),
                "{class} would count an infrastructure failure as an expected rejection"
            );
        }
    }

    #[test]
    fn every_boundary_is_either_pre_target_or_target_executed() {
        for boundary in EvidenceBoundary::ALL {
            let pre = boundary.is_pre_target();
            let executed = boundary.requires_target_execution();
            let infrastructure = *boundary == EvidenceBoundary::ExecutorInfrastructureFailure;
            assert_eq!(
                usize::from(pre) + usize::from(executed) + usize::from(infrastructure),
                1,
                "{boundary:?} falls into no single §1.5 class"
            );
        }
        assert_eq!(EvidenceBoundary::ALL.len(), 11, "§1.5 names eleven layers");
    }

    #[test]
    fn every_mutating_class_names_a_layer_and_every_positive_one_may_not_need_it() {
        for class in all_classes() {
            if class.polarity() == VectorPolarity::Negative
                && class.boundary() != EvidenceBoundary::ReportSemanticProjectionRejection
            {
                assert!(
                    class.mutation().is_some(),
                    "{class} is a rejection claim with no named mutation layer"
                );
            }
        }
        assert_eq!(MutationLayer::ALL.len(), 5);
    }

    #[test]
    fn every_mutation_layer_and_every_boundary_is_actually_exercised() {
        let classes = all_classes();
        for layer in MutationLayer::ALL {
            assert!(
                classes.iter().any(|class| class.mutation() == Some(*layer)),
                "no §18 class mutates at {layer:?}, so the layer is unreachable"
            );
        }
        // Two §1.5 layers are deliberately unexercised by §18: the
        // semantic-request and backend-emission boundaries. The first is
        // covered by §18.6's construction refusals, which the guide files
        // at the construction boundary; the second appears once, at
        // §18.10's unreachable-carrier class. Recording which boundaries
        // the matrix leaves empty is the point of this assertion.
        let unexercised: BTreeSet<EvidenceBoundary> = EvidenceBoundary::ALL
            .iter()
            .copied()
            .filter(|boundary| !classes.iter().any(|class| class.boundary() == *boundary))
            .collect();
        assert_eq!(
            unexercised,
            BTreeSet::from([EvidenceBoundary::SemanticRequestRejection]),
            "the set of §1.5 boundaries §18 never reaches has changed"
        );
    }
}
