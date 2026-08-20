//! The compiler-to-target capability and evidence-role adapter.
//!
//! # What an assessment is
//!
//! One compiler capability names something an approved analysis
//! requires of some target. One assessment states what *this* reviewed
//! Elements tapscript contract would oblige a future backend to do
//! about it. It is a statement of obligations, never of progress: no
//! assessment in this module says that a pattern has been written, that
//! a program has been emitted, or that any evidence has been produced.
//!
//! # Every assessment here is static
//!
//! The input is the *reviewed static target contract*
//! ([`ReviewedElementsTapscriptDefinition`]), and nothing else. No
//! function in this module reads a network identity, a genesis
//! identity, an activation declaration, or a deployment resource
//! override, and none accepts a value carrying them: Guide-9 §1.3 keeps
//! the static contract, the deployment declaration, and target-native
//! evidence apart, and a function that took a deployment binding and
//! then ignored it would collapse the first two of those into a false
//! impression of binding-aware assessment (second review, R2-N02).
//! Guide-9 §7.2 defers a deployment-aware assessment until a consumer
//! for one exists; when it arrives it will be a different function, over
//! a different input, returning a different type.
//!
//! # Two censuses, both carried
//!
//! The compiler publishes what an analysis requires of a target as two
//! censuses: abstract capabilities, and external-evidence *roles*. The
//! adapter assesses both, and [`TargetAssessmentSet`] carries both.
//! Consuming only the capabilities would drop half of the boundary
//! silently, and a compiler evidence role added later would vanish
//! without any signal (second review, R2-C05). Both mappings are
//! exhaustive matches with no wildcard arm, so a new member of either
//! census stops this crate compiling until its disposition is stated.
//!
//! # Support is not a boolean
//!
//! [`StaticCapabilityAssessment`] separates five things a single Boolean
//! would merge:
//!
//! - the reviewed contract states a needed primitive does not exist;
//! - a needed primitive exists on paper but the review did not
//!   establish it;
//! - the primitives are reviewed, and a backend proof pattern nobody
//!   has written is still required;
//! - the obligation is not a target program at all, but a structural
//!   fact the compiler and the transaction ABI owe;
//! - the obligation is discharged by the target's own consensus rules
//!   and by nothing a program can do.
//!
//! Collapsing any pair of those would let an assessment read as
//! progress that has not happened, which is the failure this whole
//! design is shaped to prevent.
//!
//! # Assessment factoring
//!
//! Guide-8 Appendix E.1 sketches `BackendPatternRequired` with
//! primitives and evidence, and `BackendStructural` with structural
//! requirements alone. Both carry more here, and §16.3 anticipates the
//! choice. A pattern obligation whose structural prerequisites were
//! dropped would look like a pure programming problem when it is not,
//! and a structural obligation whose primitive prerequisites were
//! dropped would stop responding to the target losing them — the
//! non-weakening property in §16.4 is exactly that an assessment
//! degrades when the target does. `BackendStructural` still carries no
//! evidence: a structural obligation is discharged by the compiler and
//! the ABI, and giving it an evidence field would invite precisely the
//! collapse §16.4 forbids.

use std::collections::{BTreeMap, BTreeSet};

use compiler::target::{ExternalEvidenceRole, RequiredCapability, TargetRequirementSet};
use target_elements::{
    CapabilityContract, ElementsCapability, ReviewedElementsTapscriptDefinition,
    StaticCapabilityStatus, TargetEvidenceRequirementId, ValidatedTargetDefinition,
};

use crate::error::TapscriptError;

/// A structural obligation the compiler and the transaction ABI owe.
///
/// None of these is a completed target program, and none of them is
/// something a target opcode can establish. They are the facts a
/// backend proof would be built on top of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackendFoundationRequirement {
    /// Protocol object families occupy a canonical, checkable layout.
    CanonicalFamilyLayout,
    /// The census of a family's members is complete, not a sample.
    CompleteFamilyCensus,
    /// Protocol families are complete and pairwise disjoint.
    CompleteAndDisjointProtocolFamilies,
    /// The fee-sponsor region is separated from protocol regions by
    /// role, never by guessing from an asset or an amount.
    ProtocolSponsorRegionSeparation,
    /// One canonical position carries the whole-transaction checks.
    CanonicalCoordinator,
    /// A root constructor's predecessor and successor bind
    /// continuously.
    RootConstructorContinuity,
    /// Projected events have a fixed, checkable shape.
    ProjectionShape,
    /// Every datum a construction needs is public.
    PublicConstructionData,
    /// The permissionless path needs no secret at all.
    SecretFreePermissionlessPath,
}

impl BackendFoundationRequirement {
    /// The complete census of structural obligations, in canonical
    /// order.
    pub const ALL: &'static [Self] = &[
        Self::CanonicalFamilyLayout,
        Self::CompleteFamilyCensus,
        Self::CompleteAndDisjointProtocolFamilies,
        Self::ProtocolSponsorRegionSeparation,
        Self::CanonicalCoordinator,
        Self::RootConstructorContinuity,
        Self::ProjectionShape,
        Self::PublicConstructionData,
        Self::SecretFreePermissionlessPath,
    ];
}

/// A stable key naming one approved complete backend proof pattern.
///
/// # Why these variants exist and no others
///
/// The type was uninhabited through Guide 8 and Guide 11, and that was
/// the point: no backend proof pattern had been approved, so no value
/// of this type existed and
/// [`StaticCapabilityAssessment::CompleteBackendPattern`] could not be
/// constructed by anyone. Guide-12 §8.4 is what admits the first
/// variants, and it admits them one at a time: a variant appears here
/// exactly where [`crate::pattern::operation_patterns`] carries a
/// complete [`crate::pattern::BackendPattern`] record for it — semantic
/// owner, target prerequisites, typed instruction fragment, stack
/// contract, failure behaviour, ABI assumptions, source requirements,
/// resource formula, positive and negative vectors, and the operation
/// evidence its correctness depends on.
///
/// Every one of these was earned by walking its fragment through the
/// abstract validator and requiring the resulting success, non-aborting
/// failure, and abort sets to be exactly the pattern's claim. A pattern
/// that could not be scheduled, or whose walk did not settle its claim,
/// has no variant here at all — not a variant marked provisional, which
/// would be an identity for something that does not exist (§1.10).
///
/// # What a variant does not claim
///
/// It does not claim a node ran anything. The vectors behind these are
/// abstract vectors over the reviewed primitive contracts;
/// relation-indexed target evidence is a later layer's obligation and
/// remains outstanding for every one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackendPatternId {
    /// §10.3: the coordinator leaf spends the canonical anchor and
    /// aborts anywhere else.
    CompactAshCoordinatorRoleV1,
    /// §10.3, §12.7: a member leaf lies in the shape's member range.
    CompactAshMemberRoleV1,
    /// §12.1, §12.2: an ASH object is the linked asset and program, in
    /// the explicit form, inside the semantic amount domain.
    CompactAshObjectRecognitionV1,
    /// §12.3: the target's own input and output counts are exactly the
    /// shape's.
    CompactAshShapeV1,
    /// §12.4: the exact explicit sum, every arithmetic flag consumed,
    /// compared byte for byte with the successor's amount.
    CompactAshExplicitSumV1,
    /// §12.5, §12.6, §12.10: every admitted position accounted for by
    /// role, which is what makes root and specialized-event absence
    /// structural.
    CompactAshCanonicalPartitionV1,
    /// §12.9: the sponsor region is exactly the suffix, carries the
    /// reserve asset, and is never read for an amount.
    CompactAshSponsorIsolationV1,
    /// §12.8: the emitted protocol leaves carry no authorization or
    /// cadence primitive at all.
    CompactAshPermissionlessPathV1,
}

impl BackendPatternId {
    /// The complete census of approved patterns, in canonical order.
    pub const ALL: &'static [Self] = &[
        Self::CompactAshCoordinatorRoleV1,
        Self::CompactAshMemberRoleV1,
        Self::CompactAshObjectRecognitionV1,
        Self::CompactAshShapeV1,
        Self::CompactAshExplicitSumV1,
        Self::CompactAshCanonicalPartitionV1,
        Self::CompactAshSponsorIsolationV1,
        Self::CompactAshPermissionlessPathV1,
    ];
}

/// Why the reviewed target contract cannot support a capability.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnsupportedReason {
    /// The contract states that a primitive this capability needs does
    /// not exist.
    ///
    /// A reviewed negative fact, distinct from a primitive the review
    /// merely left incomplete: no amount of further review turns this
    /// into a prerequisite that is met.
    ReviewedTargetPrimitivesUnsupported {
        /// The primitives the contract states are unsupported.
        primitives: BTreeSet<ElementsCapability>,
    },
}

/// What this target obliges a backend to do about one compiler
/// capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticCapabilityAssessment {
    /// The reviewed contract rules the capability out.
    Unsupported {
        /// The compiler capability, retained.
        required: RequiredCapability,
        /// What the contract states.
        reason: UnsupportedReason,
    },

    /// Prerequisites the review did not establish are in the way.
    MissingTargetPrimitives {
        /// The compiler capability, retained.
        required: RequiredCapability,
        /// The prerequisites that are not reviewed.
        missing: BTreeSet<ElementsCapability>,
    },

    /// The prerequisites hold; a backend proof pattern is still owed.
    BackendPatternRequired {
        /// The compiler capability, retained.
        required: RequiredCapability,
        /// Target primitives the pattern would be built from.
        primitives: BTreeSet<ElementsCapability>,
        /// Structural facts the pattern would rest on.
        structural: BTreeSet<BackendFoundationRequirement>,
        /// Target evidence the pattern's correctness would depend on.
        evidence: BTreeSet<TargetEvidenceRequirementId>,
    },

    /// The obligation is structural, not a target program.
    BackendStructural {
        /// The compiler capability, retained.
        required: RequiredCapability,
        /// Target primitives the structure would be checked with.
        primitives: BTreeSet<ElementsCapability>,
        /// The structural facts owed.
        requirements: BTreeSet<BackendFoundationRequirement>,
    },

    /// Only the target's own rules can discharge it.
    ExternalEvidenceRequired {
        /// The compiler capability, retained.
        required: RequiredCapability,
        /// The evidence a deployment must produce.
        evidence: BTreeSet<TargetEvidenceRequirementId>,
    },

    /// An approved complete backend pattern establishes it.
    ///
    /// The censuses are carried here for the same reason they are
    /// carried on the two backend dispositions: a discharged obligation
    /// whose prerequisites were dropped would stop responding to the
    /// target losing them, and an assessment that degrades when the
    /// target does is the whole non-weakening property.
    CompleteBackendPattern {
        /// The compiler capability, retained.
        required: RequiredCapability,
        /// The approved pattern.
        pattern: BackendPatternId,
        /// Target primitives the pattern is built from.
        primitives: BTreeSet<ElementsCapability>,
        /// Structural facts the pattern rests on.
        structural: BTreeSet<BackendFoundationRequirement>,
        /// Target evidence the pattern's correctness depends on.
        evidence: BTreeSet<TargetEvidenceRequirementId>,
    },
}

impl StaticCapabilityAssessment {
    /// The compiler capability this assessment answers.
    ///
    /// Every variant carries it, so no disposition can lose the
    /// requirement it was reached from.
    #[must_use]
    pub const fn required(&self) -> RequiredCapability {
        match self {
            Self::Unsupported { required, .. }
            | Self::MissingTargetPrimitives { required, .. }
            | Self::BackendPatternRequired { required, .. }
            | Self::BackendStructural { required, .. }
            | Self::ExternalEvidenceRequired { required, .. }
            | Self::CompleteBackendPattern { required, .. } => *required,
        }
    }

    /// This assessment's disposition, without its detail.
    #[must_use]
    pub const fn disposition(&self) -> AssessmentDisposition {
        match self {
            Self::Unsupported { .. } => AssessmentDisposition::Unsupported,
            Self::MissingTargetPrimitives { .. } => AssessmentDisposition::MissingTargetPrimitives,
            Self::BackendPatternRequired { .. } => AssessmentDisposition::BackendPatternRequired,
            Self::BackendStructural { .. } => AssessmentDisposition::BackendStructural,
            Self::ExternalEvidenceRequired { .. } => {
                AssessmentDisposition::ExternalEvidenceRequired
            }
            Self::CompleteBackendPattern { .. } => AssessmentDisposition::CompleteBackendPattern,
        }
    }

    /// The stable comparison form of this assessment.
    ///
    /// It carries the compiler capability, the disposition, and the
    /// three obligation censuses in canonical order. It carries no
    /// target program, program or transaction position, test result,
    /// digest, source revision, or execution status, because none of
    /// those exists and no field here could hold one honestly.
    ///
    /// The primitive census means what the disposition says it means:
    /// the unsupported primitives under `Unsupported`, the unmet ones
    /// under `MissingTargetPrimitives`, and the required ones under the
    /// two backend dispositions. That is deliberate — the projection is
    /// read together with the disposition or not at all.
    #[must_use]
    pub fn projection(&self) -> AssessmentProjection {
        let (primitives, structural, evidence) = match self {
            Self::Unsupported {
                reason: UnsupportedReason::ReviewedTargetPrimitivesUnsupported { primitives },
                ..
            }
            | Self::MissingTargetPrimitives {
                missing: primitives,
                ..
            } => (primitives.clone(), BTreeSet::new(), BTreeSet::new()),
            Self::BackendPatternRequired {
                primitives,
                structural,
                evidence,
                ..
            }
            | Self::CompleteBackendPattern {
                primitives,
                structural,
                evidence,
                ..
            } => (primitives.clone(), structural.clone(), evidence.clone()),
            Self::BackendStructural {
                primitives,
                requirements,
                ..
            } => (primitives.clone(), requirements.clone(), BTreeSet::new()),
            Self::ExternalEvidenceRequired { evidence, .. } => {
                (BTreeSet::new(), BTreeSet::new(), evidence.clone())
            }
        };

        AssessmentProjection {
            required: self.required(),
            disposition: self.disposition(),
            primitives: primitives.into_iter().collect(),
            structural: structural.into_iter().collect(),
            evidence: evidence.into_iter().collect(),
        }
    }
}

/// One assessment's disposition, without its detail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssessmentDisposition {
    /// The reviewed contract rules the capability out.
    Unsupported,
    /// Prerequisites the review did not establish are in the way.
    MissingTargetPrimitives,
    /// A backend proof pattern is owed.
    BackendPatternRequired,
    /// A structural fact is owed.
    BackendStructural,
    /// Target evidence is owed.
    ExternalEvidenceRequired,
    /// An approved complete pattern establishes it.
    CompleteBackendPattern,
}

/// The stable comparison form of one assessment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssessmentProjection {
    required: RequiredCapability,
    disposition: AssessmentDisposition,
    primitives: Vec<ElementsCapability>,
    structural: Vec<BackendFoundationRequirement>,
    evidence: Vec<TargetEvidenceRequirementId>,
}

impl AssessmentProjection {
    /// The compiler capability this projection answers.
    #[must_use]
    pub const fn required(&self) -> RequiredCapability {
        self.required
    }

    /// The disposition reached.
    #[must_use]
    pub const fn disposition(&self) -> AssessmentDisposition {
        self.disposition
    }

    /// The target primitives, in canonical order.
    #[must_use]
    pub fn primitives(&self) -> &[ElementsCapability] {
        &self.primitives
    }

    /// The structural obligations, in canonical order.
    #[must_use]
    pub fn structural(&self) -> &[BackendFoundationRequirement] {
        &self.structural
    }

    /// The target evidence requirements, in canonical order.
    #[must_use]
    pub fn evidence(&self) -> &[TargetEvidenceRequirementId] {
        &self.evidence
    }
}

/// Which kind of obligation one mapping row states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObligationKind {
    Pattern,
    Structural,
    ExternalEvidence,
}

/// One row of the production mapping.
struct TargetObligations {
    kind: ObligationKind,
    primitives: &'static [ElementsCapability],
    structural: &'static [BackendFoundationRequirement],
    evidence: &'static [TargetEvidenceRequirementId],
}

/// What this target would oblige a backend to do about one capability.
///
/// The match is exhaustive with no wildcard arm, so a capability added
/// to the compiler census stops this crate compiling until its
/// disposition is stated here. That compile failure is the mechanism
/// Guide-8 §15.2 asks for; a wildcard arm would replace it with a
/// silent default, and a silent default in a mapping whose whole
/// purpose is to be complete is worse than no mapping.
#[expect(
    clippy::too_many_lines,
    reason = "one table, one row per compiler capability; splitting it would hide the census"
)]
#[expect(
    clippy::match_same_arms,
    reason = "confidential and whole-transaction conservation are distinct claims whose \
              obligations coincide today; merging the arms would make a future divergence \
              a restructuring rather than an edit, and would delete the reason each is \
              external evidence"
)]
const fn obligations(required: RequiredCapability) -> TargetObligations {
    use BackendFoundationRequirement as S;
    use ElementsCapability as P;
    use TargetEvidenceRequirementId as R;

    match required {
        // Primitives can read an input's or an output's asset and
        // program. That an object *is* a protocol object of a given
        // family is a protocol judgement the target has no vocabulary
        // for, so the pattern is still owed.
        RequiredCapability::AuthenticatedObjectRecognition => TargetObligations {
            kind: ObligationKind::Pattern,
            primitives: &[
                P::InputAssetInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputProgramInspection,
            ],
            structural: &[],
            evidence: &[
                R::EncodingSemantics,
                R::InputIntrospectionSemantics,
                R::OutputIntrospectionSemantics,
            ],
        },

        // A count opcode reports how many inputs and outputs there are.
        // It does not report how many of them belong to a family, and
        // no opcode does: that follows from a canonical layout and a
        // complete census, which the compiler and the ABI owe.
        RequiredCapability::AuthenticatedFamilyCardinality => TargetObligations {
            kind: ObligationKind::Structural,
            primitives: &[
                P::InputCountInspection,
                P::OutputCountInspection,
                P::CurrentInputIndexInspection,
            ],
            structural: &[
                S::CanonicalFamilyLayout,
                S::CompleteFamilyCensus,
                S::CanonicalCoordinator,
            ],
            evidence: &[],
        },

        // Partitioning the whole transaction needs the counts, both
        // sides' assets, values, and programs, and a canonical position
        // from which the partition is checked once rather than per
        // input.
        RequiredCapability::AuthenticatedCanonicalPartition => TargetObligations {
            kind: ObligationKind::Pattern,
            primitives: &[
                P::InputCountInspection,
                P::OutputCountInspection,
                P::InputAssetInspection,
                P::InputValueInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputValueInspection,
                P::OutputProgramInspection,
            ],
            structural: &[
                S::CanonicalFamilyLayout,
                S::CompleteAndDisjointProtocolFamilies,
                S::CanonicalCoordinator,
            ],
            evidence: &[
                R::EncodingSemantics,
                R::InputIntrospectionSemantics,
                R::OutputIntrospectionSemantics,
                R::TransactionIntrospectionSemantics,
            ],
        },

        // Deliberately no value-inspection primitive. The open flow is
        // separated from the fee-sponsor region by role — by family and
        // by ABI position — and never by reading a sponsor amount,
        // testing a sponsor subtotal, or requiring sponsor positivity.
        // Listing a value primitive here would make an amount a
        // prerequisite of the partition and quietly reintroduce exactly
        // the sponsor visibility the erasure exists to remove.
        RequiredCapability::AuthenticatedOpenFlowPartition => TargetObligations {
            kind: ObligationKind::Pattern,
            primitives: &[
                P::InputCountInspection,
                P::OutputCountInspection,
                P::CurrentInputIndexInspection,
                P::InputAssetInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputProgramInspection,
            ],
            structural: &[
                S::CanonicalFamilyLayout,
                S::CompleteFamilyCensus,
                S::ProtocolSponsorRegionSeparation,
            ],
            evidence: &[
                R::EncodingSemantics,
                R::InputIntrospectionSemantics,
                R::OutputIntrospectionSemantics,
                R::TransactionIntrospectionSemantics,
                R::ConfidentialValueConservation,
            ],
        },

        // Binding a successor to its predecessor needs the outpoint,
        // both sides' assets and programs, and the hashing and tweak
        // checks a commitment would be rebuilt with. Guide 8 claims no
        // constructor continuity; the pattern is owed in full.
        RequiredCapability::AuthenticatedRootEffects => TargetObligations {
            kind: ObligationKind::Pattern,
            primitives: &[
                P::InputOutpointInspection,
                P::InputAssetInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputProgramInspection,
                P::StreamingSha256,
                P::TweakVerification,
            ],
            structural: &[S::RootConstructorContinuity],
            evidence: &[
                R::EncodingSemantics,
                R::OutputIntrospectionSemantics,
                R::InputIntrospectionSemantics,
                R::StreamingHashSemantics,
                R::EllipticCurveSemantics,
            ],
        },

        RequiredCapability::AuthenticatedProjectionSet => TargetObligations {
            kind: ObligationKind::Pattern,
            primitives: &[P::OutputCountInspection, P::OutputProgramInspection],
            structural: &[S::ProjectionShape],
            evidence: &[
                R::EncodingSemantics,
                R::OutputIntrospectionSemantics,
                R::TransactionIntrospectionSemantics,
            ],
        },

        // Sixty-four-bit arithmetic exists and is reviewed. That is not
        // wide floor arithmetic, and it is not a checked-overflow
        // discipline either: every success flag has to be tested by a
        // pattern nobody has written.
        RequiredCapability::ExactPublicAmountArithmetic => TargetObligations {
            kind: ObligationKind::Pattern,
            primitives: &[
                P::SignedFixedWidthArithmetic,
                P::SignedFixedWidthComparison,
                P::ScriptNumberConversion,
                P::ExplicitValueInspection,
            ],
            structural: &[],
            evidence: &[
                R::EncodingSemantics,
                R::ArithmeticSemantics,
                R::ComparisonSemantics,
                R::ConversionSemantics,
            ],
        },

        // No primitive demonstrates conservation, so none is listed.
        // Naming the target's own `ConfidentialValueConservation`
        // capability as a *primitive* prerequisite would turn an
        // external consensus claim into a question of primitive
        // availability, which §16.4 forbids in as many words.
        RequiredCapability::ConfidentialValueConservation => TargetObligations {
            kind: ObligationKind::ExternalEvidence,
            primitives: &[],
            structural: &[],
            evidence: &[R::ConfidentialValueConservation],
        },

        // The three authorizations share one mapping. The adapter does
        // not know which operation has an owner, an operator, or a
        // refund path — those are protocol facts — so it must not
        // differentiate them by inventing one.
        RequiredCapability::OwnerAuthorization
        | RequiredCapability::OperatorAuthorization
        | RequiredCapability::RefundAuthorization => TargetObligations {
            kind: ObligationKind::Pattern,
            primitives: &[
                P::SignatureVerification,
                P::OutputCommittingSighash,
                P::InputCommitmentControl,
            ],
            structural: &[],
            evidence: &[R::SignatureSemantics, R::SighashSemantics],
        },

        // Not reducible to an opcode, and specifically not reducible to
        // signature availability: a signature check proves someone
        // authorized something, which is the opposite of the claim that
        // anyone can construct the operation with public data alone.
        RequiredCapability::PublicConstructibility => TargetObligations {
            kind: ObligationKind::Structural,
            primitives: &[],
            structural: &[
                S::CanonicalFamilyLayout,
                S::PublicConstructionData,
                S::SecretFreePermissionlessPath,
            ],
            evidence: &[],
        },

        // The target's consensus rules conserve value across the whole
        // transaction, or they do not. A protocol-local subtotal or a
        // sponsor-positivity test is a different and weaker claim, and
        // substituting one here is named in §16.4 as a prohibited
        // weakening.
        RequiredCapability::WholeTransactionValueConservation => TargetObligations {
            kind: ObligationKind::ExternalEvidence,
            primitives: &[],
            structural: &[],
            evidence: &[R::ConfidentialValueConservation],
        },
    }
}

/// Assess one compiler capability against the reviewed static target.
///
/// Pure and deterministic: the result depends on the reviewed
/// contract's capability registry and on nothing else — no clock, no
/// environment, no interior state, no deployment binding, and no node.
///
/// # What is checked, and what is not
///
/// The primitive prerequisites are checked against the contract's
/// reviewed status for each. The evidence requirements a row names are
/// not checked for existence, because they cannot be absent: a
/// validated target definition carries the complete evidence census, so
/// there is no definition in which one is missing.
#[must_use]
pub fn assess_static_capability(
    target: &ReviewedElementsTapscriptDefinition,
    required: RequiredCapability,
) -> StaticCapabilityAssessment {
    assess_validated_capability(target.validated(), required)
}

/// Assess one compiler capability against a merely validated contract.
///
/// Crate-private, and deliberately so. The mapping is one function —
/// duplicating it for a second trust state would be two mappings that
/// could disagree — but *who may call it with what* is the trust
/// boundary: a caller-assembled definition can be locally valid without
/// being the reviewed Elements contract, and only the reviewed wrapper
/// reaches the public entry point above. Inside the crate this is how
/// the non-weakening fixtures assess a deliberately degraded contract:
/// they build one the target validator accepts, and it can never
/// impersonate the reviewed one at the package boundary because no
/// public path takes it.
///
/// This function must never become public. Making it public would hand
/// back exactly the impersonation the reviewed wrapper exists to
/// prevent (second review, R2-N01).
#[must_use]
pub(crate) fn assess_validated_capability(
    target: &ValidatedTargetDefinition,
    required: RequiredCapability,
) -> StaticCapabilityAssessment {
    let obligations = obligations(required);
    let contracts = target.definition().capabilities();

    let mut unsupported = BTreeSet::new();
    let mut missing = BTreeSet::new();

    for primitive in obligations.primitives {
        match contracts.get(primitive).map(CapabilityContract::status) {
            Some(StaticCapabilityStatus::Reviewed) => {}
            Some(StaticCapabilityStatus::Unsupported) => {
                unsupported.insert(*primitive);
            }
            // `Incomplete`, and — because the status enum is
            // non-exhaustive and a registry entry is theoretically
            // absent — anything else. Fail closed: a status this
            // version does not recognize has not established anything,
            // and reading an unknown status as satisfied is the one
            // mistake here that would be silent.
            _ => {
                missing.insert(*primitive);
            }
        }
    }

    // A reviewed negative fact outranks an incomplete one. Reporting
    // "not yet established" for something the contract says does not
    // exist would invite a reader to wait for a review that can never
    // land.
    if !unsupported.is_empty() {
        return StaticCapabilityAssessment::Unsupported {
            required,
            reason: UnsupportedReason::ReviewedTargetPrimitivesUnsupported {
                primitives: unsupported,
            },
        };
    }

    if !missing.is_empty() {
        return StaticCapabilityAssessment::MissingTargetPrimitives { required, missing };
    }

    let primitives = obligations.primitives.iter().copied().collect();
    let structural = obligations.structural.iter().copied().collect();
    let evidence = obligations.evidence.iter().copied().collect();

    match obligations.kind {
        ObligationKind::Pattern => StaticCapabilityAssessment::BackendPatternRequired {
            required,
            primitives,
            structural,
            evidence,
        },
        ObligationKind::Structural => StaticCapabilityAssessment::BackendStructural {
            required,
            primitives,
            requirements: structural,
        },
        ObligationKind::ExternalEvidence => {
            StaticCapabilityAssessment::ExternalEvidenceRequired { required, evidence }
        }
    }
}

/// What one compiler external-evidence role obliges of this target.
///
/// One variant today, because one disposition is true today: every role
/// the compiler publishes is discharged by the target's own consensus
/// rules and by nothing a program could do. It is an enum rather than a
/// struct so that a role whose obligation is *not* target evidence — a
/// role a backend pattern could discharge, say — is added as a new
/// disposition rather than by widening this one until it means nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalEvidenceAssessment {
    /// Only target evidence a deployment must produce discharges it.
    TargetEvidenceRequired {
        /// The compiler evidence role, retained.
        role: ExternalEvidenceRole,
        /// The target evidence requirements it lands on.
        evidence: BTreeSet<TargetEvidenceRequirementId>,
    },
}

impl ExternalEvidenceAssessment {
    /// The compiler evidence role this assessment answers.
    ///
    /// Every variant carries it, so the role cannot be lost on the way
    /// through the adapter — which is the whole traceability the census
    /// exists for.
    #[must_use]
    pub const fn role(&self) -> ExternalEvidenceRole {
        match self {
            Self::TargetEvidenceRequired { role, .. } => *role,
        }
    }

    /// This assessment's disposition, without its detail.
    #[must_use]
    pub const fn disposition(&self) -> EvidenceAssessmentDisposition {
        match self {
            Self::TargetEvidenceRequired { .. } => {
                EvidenceAssessmentDisposition::TargetEvidenceRequired
            }
        }
    }

    /// The stable comparison form of this assessment.
    ///
    /// The role, the disposition, and the target evidence census in
    /// canonical order. It carries no result, no report, no deployment,
    /// and no digest, because none of those exists here.
    #[must_use]
    pub fn projection(&self) -> EvidenceAssessmentProjection {
        let evidence = match self {
            Self::TargetEvidenceRequired { evidence, .. } => evidence.clone(),
        };

        EvidenceAssessmentProjection {
            role: self.role(),
            disposition: self.disposition(),
            evidence: evidence.into_iter().collect(),
        }
    }
}

/// One evidence assessment's disposition, without its detail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceAssessmentDisposition {
    /// Target evidence a deployment must produce is owed.
    TargetEvidenceRequired,
}

/// The stable comparison form of one evidence assessment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceAssessmentProjection {
    role: ExternalEvidenceRole,
    disposition: EvidenceAssessmentDisposition,
    evidence: Vec<TargetEvidenceRequirementId>,
}

impl EvidenceAssessmentProjection {
    /// The compiler evidence role this projection answers.
    #[must_use]
    pub const fn role(&self) -> ExternalEvidenceRole {
        self.role
    }

    /// The disposition reached.
    #[must_use]
    pub const fn disposition(&self) -> EvidenceAssessmentDisposition {
        self.disposition
    }

    /// The target evidence requirements, in canonical order.
    #[must_use]
    pub fn evidence(&self) -> &[TargetEvidenceRequirementId] {
        &self.evidence
    }
}

/// What this target obliges about one compiler evidence role.
///
/// Exhaustive with no wildcard arm, for the same reason the capability
/// mapping is: a role added to the compiler census stops this crate
/// compiling until its target obligation is stated here. That compile
/// failure is the mechanism the second review asks for in R2-C05, and a
/// wildcard arm would replace it with a silent default.
///
/// The function takes no target. The obligation a role lands on is a
/// property of the role and of this target contract's evidence
/// vocabulary, not of any one contract value: the target validator
/// refuses a definition whose evidence registry is not the complete
/// census, so there is no validated contract in which the requirement
/// below is absent. Taking a target here and not reading it would be
/// the same defect as R2-N02 in miniature.
#[must_use]
pub fn assess_evidence_role(role: ExternalEvidenceRole) -> ExternalEvidenceAssessment {
    let evidence: &[TargetEvidenceRequirementId] = match role {
        // The compiler says the substrate itself must conserve value
        // across a transaction; the target names that claim as
        // whole-transaction conservation over its confidential and
        // explicit value classes. Neither an opcode nor a backend
        // pattern discharges it, so no primitive and no structural
        // obligation appears here — listing one would turn an external
        // consensus claim into a question of primitive availability.
        ExternalEvidenceRole::SubstrateConservation => {
            &[TargetEvidenceRequirementId::ConfidentialValueConservation]
        }
    };

    ExternalEvidenceAssessment::TargetEvidenceRequired {
        role,
        evidence: evidence.iter().copied().collect(),
    }
}

/// Build one census map and check it against the census it must cover.
///
/// Shared by both halves of [`TargetAssessmentSet`], because both owe
/// the same three checks — no member assessed twice, no required member
/// unassessed, no assessment nothing required — and two hand-written
/// copies of them would be two places for one of the three to go
/// missing.
fn assemble_census<K: Copy + Ord, V>(
    required: &BTreeSet<K>,
    assessed: Vec<(K, V)>,
    duplicate: impl Fn(K) -> TapscriptError,
    mismatch: impl Fn(Vec<K>, Vec<K>) -> TapscriptError,
) -> Result<BTreeMap<K, V>, TapscriptError> {
    let mut assessments = BTreeMap::new();

    for (key, assessment) in assessed {
        if assessments.insert(key, assessment).is_some() {
            return Err(duplicate(key));
        }
    }

    let missing: Vec<_> = required
        .iter()
        .filter(|key| !assessments.contains_key(key))
        .copied()
        .collect();
    let unexpected: Vec<_> = assessments
        .keys()
        .filter(|key| !required.contains(key))
        .copied()
        .collect();

    if missing.is_empty() && unexpected.is_empty() {
        Ok(assessments)
    } else {
        Err(mismatch(missing, unexpected))
    }
}

/// Exactly one assessment per required capability and per evidence role.
///
/// The compiler publishes two censuses, so the adapter answers two.
/// Both are exact in both directions; neither may be dropped, and
/// neither may quietly answer for the other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetAssessmentSet {
    capabilities: BTreeMap<RequiredCapability, StaticCapabilityAssessment>,
    external_evidence: BTreeMap<ExternalEvidenceRole, ExternalEvidenceAssessment>,
}

impl TargetAssessmentSet {
    /// Every capability assessment, in canonical census order.
    pub fn capability_assessments(
        &self,
    ) -> impl Iterator<Item = (RequiredCapability, &StaticCapabilityAssessment)> {
        self.capabilities
            .iter()
            .map(|(capability, assessment)| (*capability, assessment))
    }

    /// One capability's assessment, if the set covers it.
    #[must_use]
    pub fn capability_assessment(
        &self,
        required: RequiredCapability,
    ) -> Option<&StaticCapabilityAssessment> {
        self.capabilities.get(&required)
    }

    /// Every evidence-role assessment, in canonical census order.
    pub fn evidence_assessments(
        &self,
    ) -> impl Iterator<Item = (ExternalEvidenceRole, &ExternalEvidenceAssessment)> {
        self.external_evidence
            .iter()
            .map(|(role, assessment)| (*role, assessment))
    }

    /// One evidence role's assessment, if the set covers it.
    #[must_use]
    pub fn evidence_assessment(
        &self,
        role: ExternalEvidenceRole,
    ) -> Option<&ExternalEvidenceAssessment> {
        self.external_evidence.get(&role)
    }

    /// The stable comparison form of the capability census.
    ///
    /// A vector in canonical compiler-capability order, not a set: the
    /// order is part of the contract, and a set comparison would accept
    /// a projection that named one capability twice.
    #[must_use]
    pub fn capability_projection(&self) -> Vec<AssessmentProjection> {
        self.capabilities
            .values()
            .map(StaticCapabilityAssessment::projection)
            .collect()
    }

    /// The stable comparison form of the evidence-role census.
    ///
    /// A vector in canonical role order, for the same reason.
    #[must_use]
    pub fn evidence_projection(&self) -> Vec<EvidenceAssessmentProjection> {
        self.external_evidence
            .values()
            .map(ExternalEvidenceAssessment::projection)
            .collect()
    }

    /// Build a set and check both halves against the censuses they
    /// must cover.
    ///
    /// Crate-private and taking vectors rather than maps, so that every
    /// failure is a real branch rather than an assertion about a
    /// container that already made it impossible.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::DuplicateCapabilityAssessment`] or
    /// [`TapscriptError::DuplicateEvidenceAssessment`] when one member
    /// is assessed twice;
    /// [`TapscriptError::CapabilityAssessmentCensusMismatch`] or
    /// [`TapscriptError::EvidenceAssessmentCensusMismatch`] when the
    /// assessed keys are not exactly the required ones, in either
    /// direction.
    pub(crate) fn assemble(
        required_capabilities: &BTreeSet<RequiredCapability>,
        capability_assessments: Vec<(RequiredCapability, StaticCapabilityAssessment)>,
        required_roles: &BTreeSet<ExternalEvidenceRole>,
        evidence_assessments: Vec<(ExternalEvidenceRole, ExternalEvidenceAssessment)>,
    ) -> Result<Self, TapscriptError> {
        let capabilities = assemble_census(
            required_capabilities,
            capability_assessments,
            TapscriptError::DuplicateCapabilityAssessment,
            |missing, unexpected| TapscriptError::CapabilityAssessmentCensusMismatch {
                missing,
                unexpected,
            },
        )?;
        let external_evidence = assemble_census(
            required_roles,
            evidence_assessments,
            TapscriptError::DuplicateEvidenceAssessment,
            |missing, unexpected| TapscriptError::EvidenceAssessmentCensusMismatch {
                missing,
                unexpected,
            },
        )?;

        Ok(Self {
            capabilities,
            external_evidence,
        })
    }

    /// Assess two explicit censuses against the reviewed contract.
    fn of_censuses(
        target: &ReviewedElementsTapscriptDefinition,
        required_capabilities: &BTreeSet<RequiredCapability>,
        required_roles: &BTreeSet<ExternalEvidenceRole>,
    ) -> Result<Self, TapscriptError> {
        let capabilities = required_capabilities
            .iter()
            .map(|capability| (*capability, assess_static_capability(target, *capability)))
            .collect();
        let evidence = required_roles
            .iter()
            .map(|role| (*role, assess_evidence_role(*role)))
            .collect();

        Self::assemble(
            required_capabilities,
            capabilities,
            required_roles,
            evidence,
        )
    }
}

/// Assess everything one validated analysis requires of this target.
///
/// Both published censuses are answered, and both are equal to their
/// assessment census in both directions, exactly. No required
/// capability and no external-evidence role may vanish from the result,
/// and no assessment may appear that nothing required.
///
/// # Errors
///
/// [`TapscriptError::DuplicateCapabilityAssessment`],
/// [`TapscriptError::CapabilityAssessmentCensusMismatch`],
/// [`TapscriptError::DuplicateEvidenceAssessment`], or
/// [`TapscriptError::EvidenceAssessmentCensusMismatch`] if a census
/// ever disagrees with its assessments.
pub fn assess_requirements(
    target: &ReviewedElementsTapscriptDefinition,
    requirements: &TargetRequirementSet,
) -> Result<TargetAssessmentSet, TapscriptError> {
    TargetAssessmentSet::of_censuses(
        target,
        &requirements.capabilities().collect(),
        &requirements.external_evidence().collect(),
    )
}

/// Assess the complete compiler censuses against this target.
///
/// The upper bound on any requirement set: an analysis can require a
/// subset of these and never anything outside them. Stated as its own
/// entry point because the interesting question about a newly reviewed
/// target is what it obliges across the whole vocabulary, before any
/// particular analysis narrows it.
///
/// # Errors
///
/// As [`assess_requirements`].
pub fn assess_complete_census(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<TargetAssessmentSet, TapscriptError> {
    TargetAssessmentSet::of_censuses(
        target,
        &RequiredCapability::ALL.iter().copied().collect(),
        &ExternalEvidenceRole::ALL.iter().copied().collect(),
    )
}
