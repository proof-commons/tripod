//! The exact target assessment of one operation plan (Guide-12 §8).
//!
//! # Assessed against the plan, never against a list
//!
//! §8.1 puts the static assessment first, and it is an assessment of
//! what *the compiler required for this operation*. So the input is a
//! [`ValidatedTargetOperationPlan`] and the census is the plan's own:
//! every capability it publishes, every layout requirement it
//! publishes, and every external-evidence role it publishes. An
//! assessment built from a hand-written list of expected capability
//! families would answer for the families someone remembered, and a
//! requirement the plan added later would vanish without a signal.
//!
//! Every census is exact in both directions. A requirement with no
//! verdict and a verdict for nothing required are both typed failures,
//! never a shorter report.
//!
//! # Why an operation verdict can differ from the target-wide one
//!
//! [`crate::capability::assess_static_capability`] answers a different
//! question: what would *this target* oblige a backend to do about this
//! capability, for any analysis at all. This module answers what this
//! target obliges for *this operation*, and the two can honestly
//! differ — most often because an obligation the general answer calls a
//! programming problem is, for compact ASH, discharged by an approved
//! pattern, or discharged as a structural absence because the operation
//! has no such effect to prove.
//!
//! A verdict that differed silently would be a claim with no ground, so
//! every row carries both dispositions and a [`VerdictGround`] naming
//! why they differ where they do.
//!
//! # Fail closed
//!
//! §8.3 is not advice. [`OperationAssessmentSet::emission_admissible`]
//! refuses whenever any row has no accepted target proof left, and it
//! separately refuses the specific substitutions §8.3 names — a
//! conservation obligation discharged by a program, or any pattern that
//! reads a sponsor amount. Those are checked against the pattern census
//! itself rather than trusted, because a prohibition nobody evaluates
//! is a comment.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::{
    ExternalEvidenceRole, LayoutRequirement, RequiredCapability, RequiredSourceKind,
    ValidatedTargetOperationPlan,
};
use target_elements::{
    ElementsCapability, ReviewedElementsTapscriptDefinition, TargetEvidenceRequirementId,
};

use crate::capability::{
    AssessmentDisposition, BackendFoundationRequirement, BackendPatternId,
    ExternalEvidenceAssessment, StaticCapabilityAssessment, assess_evidence_role,
    assess_static_capability,
};
use crate::error::TapscriptError;
use crate::pattern::BackendPattern;

/// One requirement the plan publishes, whichever census it came from.
///
/// Three kinds in one key so the assessment is a single census that can
/// be counted, compared, and refused as a whole. Splitting them into
/// three unrelated results would let one be dropped while the other two
/// still looked complete.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationRequirement {
    /// An abstract capability the analysis requires of a target.
    Capability(RequiredCapability),
    /// A target-independent layout obligation on a future backend.
    Layout(LayoutRequirement),
    /// An external-evidence role the analysis leaves open.
    ExternalEvidence(ExternalEvidenceRole),
}

/// Why an operation verdict is what it is.
///
/// Every row states one, including the rows where the operation verdict
/// simply repeats the target-wide one — a ground that appeared only on
/// the interesting rows would leave a reader unable to tell "same
/// answer, same reason" from "nobody looked".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerdictGround {
    /// The target-wide assessment already answers it, unchanged.
    TargetWideAssessment,
    /// An approved pattern of this operation discharges it.
    ApprovedPattern,
    /// The operation has no such effect, and the exhaustive position
    /// census makes its absence structural (§8.1, §12.10).
    ///
    /// Distinct from an approved pattern: nothing here proves a root
    /// effect or a projected event *correct*. What is established is
    /// that the operation contains none, which is a different and
    /// weaker claim, and one that would be a lie if it were reported as
    /// a completed proof of the effect itself.
    StructuralAbsence,
    /// Only the target's own consensus rules discharge it.
    ExternalTargetClaim,
    /// A structural obligation the compiler and the ABI owe.
    CompilerOrAbiObligation,
    /// No approved pattern covers it, and none is claimed.
    NoApprovedPattern,
}

/// One requirement's operation verdict.
///
/// Both dispositions are retained. A row that reported only the
/// operation verdict would lose the fact that the target's general
/// answer was weaker, which is what makes the non-weakening property
/// checkable: an assessment must degrade when the target does, and it
/// cannot be seen to degrade if the target's own answer is not there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationVerdict {
    requirement: OperationRequirement,
    target_wide: Option<AssessmentDisposition>,
    operation: AssessmentDisposition,
    ground: VerdictGround,
    patterns: BTreeSet<BackendPatternId>,
    primitives: BTreeSet<ElementsCapability>,
    structural: BTreeSet<BackendFoundationRequirement>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl OperationVerdict {
    /// The requirement this verdict answers.
    #[must_use]
    pub const fn requirement(&self) -> &OperationRequirement {
        &self.requirement
    }

    /// What the target-wide assessment says, where one applies.
    ///
    /// `None` for a layout requirement: the target-wide mapping answers
    /// capabilities and evidence roles, and there is no general answer
    /// for a layout obligation to compare against.
    #[must_use]
    pub const fn target_wide(&self) -> Option<AssessmentDisposition> {
        self.target_wide
    }

    /// What this operation's assessment says.
    #[must_use]
    pub const fn operation(&self) -> AssessmentDisposition {
        self.operation
    }

    /// Why.
    #[must_use]
    pub const fn ground(&self) -> VerdictGround {
        self.ground
    }

    /// The approved patterns that contribute to this verdict.
    ///
    /// Possibly several, and possibly some where the verdict is not
    /// `CompleteBackendPattern`: a pattern can establish part of an
    /// obligation without establishing all of it, and recording the
    /// contribution without overstating the verdict is the honest
    /// shape.
    #[must_use]
    pub const fn patterns(&self) -> &BTreeSet<BackendPatternId> {
        &self.patterns
    }

    /// The target primitives this verdict rests on.
    #[must_use]
    pub const fn primitives(&self) -> &BTreeSet<ElementsCapability> {
        &self.primitives
    }

    /// The structural obligations this verdict rests on.
    #[must_use]
    pub const fn structural(&self) -> &BTreeSet<BackendFoundationRequirement> {
        &self.structural
    }

    /// The target evidence this verdict depends on.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Whether this row still leaves an accepted target proof.
    ///
    /// The three admitting states are the ones that name who discharges
    /// the obligation: an approved pattern, the compiler and the ABI,
    /// or the target's own rules. The other three name the absence of
    /// one.
    #[must_use]
    pub const fn admits_emission(&self) -> bool {
        matches!(
            self.operation,
            AssessmentDisposition::CompleteBackendPattern
                | AssessmentDisposition::BackendStructural
                | AssessmentDisposition::ExternalEvidenceRequired
        )
    }
}

/// Why emission is refused (§8.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmissionRefusal {
    /// Some requirement has no accepted target proof.
    NoAcceptedProof {
        /// Every unproved requirement, in canonical census order.
        requirements: Vec<OperationRequirement>,
    },
    /// A conservation obligation was discharged by a program.
    ///
    /// §8.3 names this exactly: whole-transaction balance must not be
    /// substituted for object closure, and a conservation row reaching
    /// `CompleteBackendPattern` is that substitution.
    ConservationDischargedByPattern {
        /// The capability that was over-claimed.
        capability: RequiredCapability,
    },
    /// An admitted pattern reads a sponsor amount.
    ///
    /// §8.3 and §1.6. Checked against the pattern census rather than
    /// assumed, because the prohibition is about what the emitted
    /// instructions do.
    PatternReadsSponsorAmount {
        /// The offending pattern.
        pattern: BackendPatternId,
    },
    /// A verdict claims a pattern the census does not carry.
    UnbackedPatternClaim {
        /// The claimed pattern.
        pattern: BackendPatternId,
    },
}

/// The complete assessment of one operation plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationAssessmentSet {
    verdicts: BTreeMap<OperationRequirement, OperationVerdict>,
}

impl OperationAssessmentSet {
    /// Every verdict, in canonical census order.
    pub fn verdicts(&self) -> impl Iterator<Item = &OperationVerdict> {
        self.verdicts.values()
    }

    /// One requirement's verdict, if the census covers it.
    #[must_use]
    pub fn verdict(&self, requirement: &OperationRequirement) -> Option<&OperationVerdict> {
        self.verdicts.get(requirement)
    }

    /// How many requirements reached each support state.
    ///
    /// The census §8.2 asks for, as a count per state. A state with no
    /// rows is absent rather than reported as zero, so the map's keys
    /// are the states this operation actually reached.
    #[must_use]
    pub fn state_census(&self) -> BTreeMap<AssessmentDisposition, usize> {
        let mut census = BTreeMap::new();

        for verdict in self.verdicts.values() {
            *census.entry(verdict.operation).or_insert(0) += 1;
        }
        census
    }

    /// How many requirements reached each ground.
    #[must_use]
    pub fn ground_census(&self) -> BTreeMap<VerdictGround, usize> {
        let mut census = BTreeMap::new();

        for verdict in self.verdicts.values() {
            *census.entry(verdict.ground).or_insert(0) += 1;
        }
        census
    }

    /// The total number of requirements assessed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.verdicts.len()
    }

    /// Whether the census is empty, which a validated plan's never is.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.verdicts.is_empty()
    }

    /// Whether a backend may emit for this operation (§8.3).
    ///
    /// # Errors
    ///
    /// [`EmissionRefusal::NoAcceptedProof`] when any requirement has no
    /// accepted target proof left;
    /// [`EmissionRefusal::ConservationDischargedByPattern`] when a
    /// conservation obligation was answered by a program;
    /// [`EmissionRefusal::PatternReadsSponsorAmount`] when an admitted
    /// pattern introspects a sponsor amount; and
    /// [`EmissionRefusal::UnbackedPatternClaim`] when a verdict names a
    /// pattern the census does not carry.
    pub fn emission_admissible(
        &self,
        patterns: &BTreeMap<BackendPatternId, BackendPattern>,
    ) -> Result<(), EmissionRefusal> {
        for verdict in self.verdicts.values() {
            for pattern in &verdict.patterns {
                if !patterns.contains_key(pattern) {
                    return Err(EmissionRefusal::UnbackedPatternClaim { pattern: *pattern });
                }
            }

            // §8.3: whole-transaction balance must not be substituted
            // for object closure, so a conservation obligation may
            // never be discharged by a program.
            if let OperationRequirement::Capability(
                capability @ (RequiredCapability::WholeTransactionValueConservation
                | RequiredCapability::ConfidentialValueConservation),
            ) = verdict.requirement
                && verdict.operation == AssessmentDisposition::CompleteBackendPattern
            {
                return Err(EmissionRefusal::ConservationDischargedByPattern { capability });
            }
        }

        // §8.3 and §1.6: no admitted pattern may read a sponsor amount.
        // The sponsor region is the one place a value introspection
        // would be the erased amount, and the isolation pattern is the
        // only pattern that touches it — so the check is on that
        // pattern's own prerequisite census, derived from the
        // instructions it schedules.
        if let Some(isolation) = patterns.get(&BackendPatternId::CompactAshSponsorIsolationV1)
            && isolation
                .prerequisites()
                .contains(&ElementsCapability::InputValueInspection)
        {
            return Err(EmissionRefusal::PatternReadsSponsorAmount {
                pattern: BackendPatternId::CompactAshSponsorIsolationV1,
            });
        }

        let unproved = self
            .verdicts
            .values()
            .filter(|verdict| !verdict.admits_emission())
            .map(|verdict| verdict.requirement.clone())
            .collect::<Vec<_>>();

        if unproved.is_empty() {
            Ok(())
        } else {
            Err(EmissionRefusal::NoAcceptedProof {
                requirements: unproved,
            })
        }
    }
}

/// Assess every requirement one validated operation plan publishes.
///
/// # Errors
///
/// [`TapscriptError::DuplicateOperationRequirement`] if the plan ever
/// publishes one requirement twice. Both censuses are exact by
/// construction otherwise: the verdicts are built from the plan's own
/// iterators, so there is no route by which a verdict exists for
/// something the plan did not publish.
pub fn assess_operation_plan(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedTargetOperationPlan,
) -> Result<OperationAssessmentSet, TapscriptError> {
    let mut verdicts: BTreeMap<OperationRequirement, OperationVerdict> = BTreeMap::new();

    let mut admit = |verdict: OperationVerdict| -> Result<(), TapscriptError> {
        if verdicts
            .insert(verdict.requirement.clone(), verdict)
            .is_some()
        {
            return Err(TapscriptError::DuplicateOperationRequirement);
        }
        Ok(())
    };

    for capability in plan.capabilities() {
        admit(capability_verdict(target, capability))?;
    }
    for requirement in plan.layout() {
        admit(layout_verdict(requirement.clone()))?;
    }
    for role in plan.external_evidence() {
        admit(evidence_verdict(role))?;
    }

    Ok(OperationAssessmentSet { verdicts })
}

/// The operation verdict for one abstract capability.
fn capability_verdict(
    target: &ReviewedElementsTapscriptDefinition,
    capability: RequiredCapability,
) -> OperationVerdict {
    let assessment = assess_static_capability(target, capability);
    let projection = assessment.projection();
    let target_wide = projection.disposition();
    let mut primitives: BTreeSet<ElementsCapability> =
        projection.primitives().iter().copied().collect();
    let structural: BTreeSet<BackendFoundationRequirement> =
        projection.structural().iter().copied().collect();
    let evidence: BTreeSet<TargetEvidenceRequirementId> =
        projection.evidence().iter().copied().collect();

    let (operation, ground, patterns) = capability_operation_verdict(capability, &assessment);

    // The aggregate pattern narrows an introspected amount to a
    // fixed-width operand before any arithmetic touches it, so slicing
    // is a prerequisite of *this* technique even though it is not a
    // prerequisite of exact arithmetic in general. It is recorded on
    // the operation row rather than pushed back into the target-wide
    // mapping, because the target-wide answer is about the capability
    // and this is about the pattern that discharges it here.
    if capability == RequiredCapability::ExactPublicAmountArithmetic {
        primitives.insert(ElementsCapability::ByteStringSlicing);
        primitives.insert(ElementsCapability::BooleanVerification);
    }

    OperationVerdict {
        requirement: OperationRequirement::Capability(capability),
        target_wide: Some(target_wide),
        operation,
        ground,
        patterns,
        primitives,
        structural,
        evidence,
    }
}

/// Which pattern, if any, discharges one capability for this operation.
///
/// Exhaustive with no wildcard arm: a capability added to the compiler
/// census stops this crate compiling until this operation's answer for
/// it is stated, which is the same mechanism the target-wide mapping
/// uses.
fn capability_operation_verdict(
    capability: RequiredCapability,
    assessment: &StaticCapabilityAssessment,
) -> (
    AssessmentDisposition,
    VerdictGround,
    BTreeSet<BackendPatternId>,
) {
    use AssessmentDisposition as D;
    use BackendPatternId as P;
    use VerdictGround as G;

    // A capability whose prerequisites the reviewed target does not
    // establish is never upgraded by a pattern. A pattern built on a
    // primitive the target does not have is not a pattern, and letting
    // an approved identity override a missing prerequisite is exactly
    // the weakening §8.3 forbids — it is also what makes the
    // non-weakening property hold: degrade the target and every one of
    // these rows degrades with it.
    let disposition = assessment.disposition();
    if matches!(disposition, D::Unsupported | D::MissingTargetPrimitives) {
        return (disposition, G::TargetWideAssessment, BTreeSet::new());
    }

    match capability {
        RequiredCapability::AuthenticatedObjectRecognition => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshObjectRecognitionV1]),
        ),

        RequiredCapability::AuthenticatedFamilyCardinality => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([
                P::CompactAshShapeV1,
                P::CompactAshCoordinatorRoleV1,
                P::CompactAshMemberRoleV1,
            ]),
        ),

        RequiredCapability::AuthenticatedCanonicalPartition => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([
                P::CompactAshCanonicalPartitionV1,
                P::CompactAshObjectRecognitionV1,
                P::CompactAshExplicitSumV1,
            ]),
        ),

        RequiredCapability::AuthenticatedOpenFlowPartition => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshSponsorIsolationV1, P::CompactAshShapeV1]),
        ),

        // Compact ASH creates no root and projects no specialized
        // event, so there is no effect here for a pattern to prove
        // correct. What the exhaustive position census establishes is
        // that none is present — a structural absence, which §8.1
        // admits and which stays represented in the census rather than
        // being dropped. Reporting it as a completed pattern would
        // claim a proof of something that does not occur.
        RequiredCapability::AuthenticatedRootEffects
        | RequiredCapability::AuthenticatedProjectionSet => (
            D::BackendStructural,
            G::StructuralAbsence,
            BTreeSet::from([P::CompactAshCanonicalPartitionV1, P::CompactAshShapeV1]),
        ),

        RequiredCapability::ExactPublicAmountArithmetic => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshExplicitSumV1]),
        ),

        // The permissionless pattern establishes the secret-free half
        // by audit of the emitted instructions. It does not establish
        // that every datum a construction needs is public, which is not
        // a target program at all, so the row stays structural and the
        // pattern is recorded as contributing rather than discharging.
        RequiredCapability::PublicConstructibility => (
            D::BackendStructural,
            G::CompilerOrAbiObligation,
            BTreeSet::from([P::CompactAshPermissionlessPathV1]),
        ),

        RequiredCapability::ConfidentialValueConservation
        | RequiredCapability::WholeTransactionValueConservation => (
            D::ExternalEvidenceRequired,
            G::ExternalTargetClaim,
            BTreeSet::new(),
        ),

        // Compact ASH is permissionless, so no authorization capability
        // should reach this assessment at all. If one does, it is not
        // discharged: §12.8 forbids a signature check in a compact-ASH
        // protocol leaf, so there is no pattern here and the row fails
        // closed.
        RequiredCapability::OwnerAuthorization
        | RequiredCapability::OperatorAuthorization
        | RequiredCapability::RefundAuthorization => (
            D::BackendPatternRequired,
            G::NoApprovedPattern,
            BTreeSet::new(),
        ),
    }
}

/// The operation verdict for one layout requirement.
///
/// Exhaustive over the layout vocabulary and, inside the source-routing
/// variant, over the compiler's fact-source kinds. Every discrimination
/// here is on a compiler-owned value; nothing reads an
/// architecture-owned payload, which keeps the target package from
/// deciding anything about semantic identity (§1.2).
fn layout_verdict(requirement: LayoutRequirement) -> OperationVerdict {
    use AssessmentDisposition as D;
    use BackendFoundationRequirement as S;
    use BackendPatternId as P;
    use VerdictGround as G;

    let (operation, ground, patterns, structural) = match &requirement {
        // The coordinator's anchor is bound by the index comparison the
        // coordinator leaf opens with.
        LayoutRequirement::CanonicalCoordinator { .. } => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshCoordinatorRoleV1, P::CompactAshMemberRoleV1]),
            BTreeSet::from([S::CanonicalCoordinator]),
        ),

        // A family census is authenticated by pinning the target's own
        // counts to the shape and then visiting every position.
        LayoutRequirement::AuthenticateFamilyCensus { .. } => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshShapeV1, P::CompactAshObjectRecognitionV1]),
            BTreeSet::from([S::CompleteFamilyCensus, S::CanonicalFamilyLayout]),
        ),

        LayoutRequirement::CompleteAndDisjointFamilies { .. } => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([
                P::CompactAshCanonicalPartitionV1,
                P::CompactAshSponsorIsolationV1,
            ]),
            BTreeSet::from([
                S::CompleteAndDisjointProtocolFamilies,
                S::ProtocolSponsorRegionSeparation,
            ]),
        ),

        LayoutRequirement::IsolateSponsorRegion { .. } => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshSponsorIsolationV1, P::CompactAshShapeV1]),
            BTreeSet::from([S::ProtocolSponsorRegionSeparation]),
        ),

        // The selected representation is enforced by the prefix
        // comparison every recognition fragment opens its value check
        // with.
        LayoutRequirement::EnforceRepresentation { .. } => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshObjectRecognitionV1]),
            BTreeSet::new(),
        ),

        LayoutRequirement::SecretFreeOperationPath { .. } => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshPermissionlessPathV1]),
            BTreeSet::from([S::SecretFreePermissionlessPath]),
        ),

        LayoutRequirement::MakeSourceAvailable { source, .. } => routing_verdict(source.source),
    };

    OperationVerdict {
        requirement: OperationRequirement::Layout(requirement),
        target_wide: None,
        operation,
        ground,
        patterns,
        primitives: BTreeSet::new(),
        structural,
        evidence: BTreeSet::new(),
    }
}

/// Which pattern routes one kind of fact to the carrier that reads it.
///
/// Exhaustive over the compiler's fact-source vocabulary. The kinds
/// this plan never produces are answered anyway, and answered fail
/// closed: an authorization witness reaching a compact-ASH carrier
/// would contradict §12.8, and a row that quietly defaulted to
/// "discharged" is how such a contradiction would go unnoticed.
fn routing_verdict(
    source: RequiredSourceKind,
) -> (
    AssessmentDisposition,
    VerdictGround,
    BTreeSet<BackendPatternId>,
    BTreeSet<BackendFoundationRequirement>,
) {
    use AssessmentDisposition as D;
    use BackendFoundationRequirement as S;
    use BackendPatternId as P;
    use RequiredSourceKind as K;
    use VerdictGround as G;

    match source {
        K::AuthenticatedInputObject | K::AuthenticatedOutputObject => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([
                P::CompactAshObjectRecognitionV1,
                P::CompactAshCanonicalPartitionV1,
            ]),
            BTreeSet::from([S::CanonicalFamilyLayout]),
        ),

        K::AuthenticatedFamilyCensus => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshShapeV1, P::CompactAshSponsorIsolationV1]),
            BTreeSet::from([S::CompleteFamilyCensus]),
        ),

        K::AuthenticatedConsensusValue => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([P::CompactAshExplicitSumV1, P::CompactAshObjectRecognitionV1]),
            BTreeSet::new(),
        ),

        // The transition certificate is the canonical partition, the
        // open-flow set, the root effects, and the projection set. For
        // compact ASH the first two are proved by the position census
        // and the last two are absences it makes structural, so the row
        // is discharged by the partition pattern together with the
        // count the shape pattern pins.
        K::AuthenticatedTransitionCertificate => (
            D::CompleteBackendPattern,
            G::ApprovedPattern,
            BTreeSet::from([
                P::CompactAshCanonicalPartitionV1,
                P::CompactAshShapeV1,
                P::CompactAshSponsorIsolationV1,
            ]),
            BTreeSet::from([S::CompleteAndDisjointProtocolFamilies]),
        ),

        K::PublicConstructionData => (
            D::BackendStructural,
            G::CompilerOrAbiObligation,
            BTreeSet::from([P::CompactAshPermissionlessPathV1]),
            BTreeSet::from([S::PublicConstructionData]),
        ),

        K::ExternalEvidence => (
            D::ExternalEvidenceRequired,
            G::ExternalTargetClaim,
            BTreeSet::new(),
            BTreeSet::new(),
        ),

        // A commitment relation is the confidential representation,
        // which the Phase-4 filter already excluded; a runtime bound
        // and a derived expression are facts no emitted fragment
        // computes; and every witness source is a secret reaching a
        // carrier, which a permissionless operation must not have. All
        // of them fail closed with no pattern, so a plan that started
        // producing one refuses emission rather than passing.
        K::AuthenticatedCommitmentRelation
        | K::RuntimeArchitectureBound
        | K::DerivedExpression
        | K::InputOwnerWitness
        | K::OperatorWitness
        | K::RefundKeyWitness
        | K::SponsorLocalWitness => (
            D::BackendPatternRequired,
            G::NoApprovedPattern,
            BTreeSet::new(),
            BTreeSet::new(),
        ),
    }
}

/// The operation verdict for one external-evidence role.
fn evidence_verdict(role: ExternalEvidenceRole) -> OperationVerdict {
    let assessment = assess_evidence_role(role);
    let evidence = match &assessment {
        ExternalEvidenceAssessment::TargetEvidenceRequired { evidence, .. } => evidence.clone(),
    };

    OperationVerdict {
        requirement: OperationRequirement::ExternalEvidence(role),
        target_wide: Some(AssessmentDisposition::ExternalEvidenceRequired),
        operation: AssessmentDisposition::ExternalEvidenceRequired,
        ground: VerdictGround::ExternalTargetClaim,
        patterns: BTreeSet::new(),
        primitives: BTreeSet::new(),
        structural: BTreeSet::new(),
        evidence,
    }
}
