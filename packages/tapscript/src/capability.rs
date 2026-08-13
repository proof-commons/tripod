//! The compiler-to-target capability adapter.
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
//! # Support is not a boolean
//!
//! [`CapabilityAssessment`] separates five things a single Boolean
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

use compiler::target::{RequiredCapability, TargetRequirementSet};
use target_elements::{
    CapabilityContract, ElementsCapability, ElementsTarget, StaticCapabilityStatus,
    TargetEvidenceRequirementId,
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
/// The type is uninhabited, and that is the point. No backend proof
/// pattern has been approved, so there is no value of this type, so
/// [`CapabilityAssessment::CompleteBackendPattern`] cannot be
/// constructed — by anyone, including a future careless caller inside
/// this crate. Guide-8 §16.5 states that no complete pattern may be
/// claimed here; stating it in the type system rather than in a comment
/// means the prohibition cannot be forgotten. Approving the first real
/// pattern is the act of giving this enum its first variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackendPatternId {}

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
pub enum CapabilityAssessment {
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
    /// Unreachable: see [`BackendPatternId`].
    CompleteBackendPattern {
        /// The compiler capability, retained.
        required: RequiredCapability,
        /// The approved pattern.
        pattern: BackendPatternId,
    },
}

impl CapabilityAssessment {
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
            } => (primitives.clone(), structural.clone(), evidence.clone()),
            Self::BackendStructural {
                primitives,
                requirements,
                ..
            } => (primitives.clone(), requirements.clone(), BTreeSet::new()),
            Self::ExternalEvidenceRequired { evidence, .. } => {
                (BTreeSet::new(), BTreeSet::new(), evidence.clone())
            }
            // Unreachable: [`BackendPatternId`] is uninhabited, so this
            // variant has no value. The arm is written rather than
            // elided because a match behind a reference must still list
            // it, and it yields empty censuses rather than reading the
            // pattern — dereferencing a reference to an uninhabited
            // type is undefined behavior even where the code cannot
            // run.
            Self::CompleteBackendPattern { .. } => {
                (BTreeSet::new(), BTreeSet::new(), BTreeSet::new())
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

/// Assess one compiler capability against one validated target.
///
/// Pure and deterministic: the result depends on the target's
/// capability registry and on nothing else — no clock, no environment,
/// no interior state, and no node.
///
/// # What is checked, and what is not
///
/// The primitive prerequisites are checked against the target's
/// reviewed status for each. The evidence requirements a row names are
/// not checked for existence, because they cannot be absent: a
/// validated target definition carries the complete evidence census, so
/// there is no [`ElementsTarget`] in which one is missing.
#[must_use]
pub fn assess_capability(
    target: &ElementsTarget,
    required: RequiredCapability,
) -> CapabilityAssessment {
    let obligations = obligations(required);
    let contracts = target.definition().definition().capabilities();

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
        return CapabilityAssessment::Unsupported {
            required,
            reason: UnsupportedReason::ReviewedTargetPrimitivesUnsupported {
                primitives: unsupported,
            },
        };
    }

    if !missing.is_empty() {
        return CapabilityAssessment::MissingTargetPrimitives { required, missing };
    }

    let primitives = obligations.primitives.iter().copied().collect();
    let structural = obligations.structural.iter().copied().collect();
    let evidence = obligations.evidence.iter().copied().collect();

    match obligations.kind {
        ObligationKind::Pattern => CapabilityAssessment::BackendPatternRequired {
            required,
            primitives,
            structural,
            evidence,
        },
        ObligationKind::Structural => CapabilityAssessment::BackendStructural {
            required,
            primitives,
            requirements: structural,
        },
        ObligationKind::ExternalEvidence => {
            CapabilityAssessment::ExternalEvidenceRequired { required, evidence }
        }
    }
}

/// Exactly one assessment per required capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityAssessmentSet {
    assessments: BTreeMap<RequiredCapability, CapabilityAssessment>,
}

impl CapabilityAssessmentSet {
    /// Every assessment, in canonical compiler-capability order.
    pub fn assessments(&self) -> impl Iterator<Item = (RequiredCapability, &CapabilityAssessment)> {
        self.assessments
            .iter()
            .map(|(capability, assessment)| (*capability, assessment))
    }

    /// One capability's assessment, if the set covers it.
    #[must_use]
    pub fn assessment(&self, required: RequiredCapability) -> Option<&CapabilityAssessment> {
        self.assessments.get(&required)
    }

    /// The stable comparison form of the whole set.
    ///
    /// A vector in canonical compiler-capability order, not a set: the
    /// order is part of the contract, and a set comparison would accept
    /// a projection that named one capability twice.
    #[must_use]
    pub fn projection(&self) -> Vec<AssessmentProjection> {
        self.assessments
            .values()
            .map(CapabilityAssessment::projection)
            .collect()
    }

    /// Build a set and check it against the census it must cover.
    ///
    /// Crate-private and taking a vector rather than a map, so that
    /// both failures are real branches rather than assertions about a
    /// container that already made them impossible.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::DuplicateCapabilityAssessment`] when one
    /// capability is assessed twice;
    /// [`TapscriptError::CapabilityAssessmentCensusMismatch`] when the
    /// assessed keys are not exactly `required`, in either direction.
    pub(crate) fn assemble(
        required: &BTreeSet<RequiredCapability>,
        assessed: Vec<(RequiredCapability, CapabilityAssessment)>,
    ) -> Result<Self, TapscriptError> {
        let mut assessments = BTreeMap::new();

        for (capability, assessment) in assessed {
            if assessments.insert(capability, assessment).is_some() {
                return Err(TapscriptError::DuplicateCapabilityAssessment(capability));
            }
        }

        let missing: Vec<_> = required
            .iter()
            .filter(|capability| !assessments.contains_key(capability))
            .copied()
            .collect();
        let unexpected: Vec<_> = assessments
            .keys()
            .filter(|capability| !required.contains(capability))
            .copied()
            .collect();

        if missing.is_empty() && unexpected.is_empty() {
            Ok(Self { assessments })
        } else {
            Err(TapscriptError::CapabilityAssessmentCensusMismatch {
                missing,
                unexpected,
            })
        }
    }

    /// Assess an explicit capability census against one target.
    fn of_census(
        target: &ElementsTarget,
        required: &BTreeSet<RequiredCapability>,
    ) -> Result<Self, TapscriptError> {
        let assessed = required
            .iter()
            .map(|capability| (*capability, assess_capability(target, *capability)))
            .collect();

        Self::assemble(required, assessed)
    }
}

/// Assess everything one validated analysis requires of this target.
///
/// The published requirement census and the assessment census are equal
/// in both directions, exactly. No required capability may vanish from
/// the result, and no assessment may appear that nothing required.
///
/// # Errors
///
/// [`TapscriptError::DuplicateCapabilityAssessment`] or
/// [`TapscriptError::CapabilityAssessmentCensusMismatch`] if the two
/// censuses ever disagree.
pub fn assess_requirements(
    target: &ElementsTarget,
    requirements: &TargetRequirementSet,
) -> Result<CapabilityAssessmentSet, TapscriptError> {
    CapabilityAssessmentSet::of_census(target, &requirements.capabilities().collect())
}

/// Assess the complete compiler capability census against this target.
///
/// The upper bound on any requirement set: an analysis can require a
/// subset of this and never anything outside it. Stated as its own
/// entry point because the interesting question about a newly reviewed
/// target is what it obliges across the whole census, before any
/// particular analysis narrows it.
///
/// # Errors
///
/// As [`assess_requirements`].
pub fn assess_complete_census(
    target: &ElementsTarget,
) -> Result<CapabilityAssessmentSet, TapscriptError> {
    CapabilityAssessmentSet::of_census(target, &RequiredCapability::ALL.iter().copied().collect())
}
