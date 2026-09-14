//! Exact maturity-announcement requirements assessed against the static target.
//!
//! No maturity STATE pattern exists here. Compact and live patterns are not
//! admissible as maturity completion claims: neither the public entry point nor
//! its implementation accepts or consults a pilot pattern. Layout rows remain
//! routing and authentication obligations, never proof that STATE is implemented.

use std::collections::{BTreeMap, BTreeSet};

use compiler::maturity_announcement_plan::{
    MaturityAnnouncementRepresentationPlan as Representation,
    MaturityAnnouncementRepresentationProjection, ValidatedMaturityAnnouncementOperationPlan,
};
use compiler::operation_plan::{
    ExternalEvidenceRole, LayoutRequirement, RequiredCapability, RequiredSourceKind,
};
use target_elements::{
    ElementsCapability, ReviewedElementsTapscriptDefinition, StaticCapabilityStatus,
    TargetEvidenceRequirementId, ValidatedTargetDefinition,
};

use crate::capability::{
    AssessmentDisposition, BackendFoundationRequirement, assess_evidence_role,
    assess_validated_capability, census_enum,
};
use crate::{TapscriptError, VerdictGround};

census_enum! {
    /// The eighteen target capability groups required by the announcement.
    pub enum MaturityCapabilityGroup {
        /// Current-input index.
        CurrentInputIndex,
        /// Input count and output count.
        InputAndOutputCount,
        /// Input asset and value inspection.
        InputAssetAndValueInspection,
        /// Input and output program inspection.
        InputAndOutputProgramInspection,
        /// Transaction version and lock-time inspection where required.
        TransactionVersionAndLockTime,
        /// Byte equality and verifying equality.
        ByteEquality,
        /// Checked fixed-width arithmetic and comparisons.
        CheckedArithmeticAndComparisons,
        /// Byte slicing and concatenation.
        ByteSlicingAndConcatenation,
        /// Streaming SHA-256.
        StreamingSha256,
        /// Tapleaf and tapbranch hashing support or exact compositional equivalents.
        TapleafAndTapbranchHashing,
        /// Tweak verification.
        TweakVerification,
        /// X-only key encoding.
        XOnlyKeyEncoding,
        /// Signature verification.
        SignatureVerification,
        /// Selected sighash semantics.
        SelectedSighashSemantics,
        /// Script-path execution.
        ScriptPathExecution,
        /// Target transaction conservation.
        TargetTransactionConservation,
        /// Fee-role recognition.
        FeeRoleRecognition,
        /// Resource limits.
        ResourceLimits,
    }
}

impl MaturityCapabilityGroup {
    /// The registry capabilities required by this group, in mapping order.
    ///
    /// An encoding contract is not a capability entry. Neither tagged-tree
    /// hashing nor fee-role recognition has a registry entry or a reviewed
    /// exact composition here; those and x-only encoding map to the empty slice.
    #[must_use]
    pub const fn target_capabilities(self) -> &'static [ElementsCapability] {
        use ElementsCapability as P;
        match self {
            Self::CurrentInputIndex => &[P::CurrentInputIndexInspection],
            Self::InputAndOutputCount => &[P::InputCountInspection, P::OutputCountInspection],
            Self::InputAssetAndValueInspection => {
                &[P::InputAssetInspection, P::InputValueInspection]
            }
            Self::InputAndOutputProgramInspection => {
                &[P::InputProgramInspection, P::OutputProgramInspection]
            }
            Self::TransactionVersionAndLockTime => &[
                P::TransactionVersionInspection,
                P::TransactionLockTimeInspection,
            ],
            Self::ByteEquality => &[P::ByteStringEquality, P::BooleanVerification],
            Self::CheckedArithmeticAndComparisons => &[
                P::SignedFixedWidthArithmetic,
                P::SignedFixedWidthComparison,
                P::BooleanVerification,
            ],
            Self::ByteSlicingAndConcatenation => {
                &[P::ByteStringSlicing, P::ByteStringConcatenation]
            }
            Self::StreamingSha256 => &[P::StreamingSha256],
            Self::TweakVerification => &[P::TweakVerification],
            Self::SignatureVerification => &[P::SignatureVerification],
            Self::SelectedSighashSemantics => {
                &[P::OutputCommittingSighash, P::InputCommitmentControl]
            }
            Self::ScriptPathExecution => &[P::TapscriptExecution, P::RequiredLeafVersion],
            Self::TargetTransactionConservation => &[P::ConfidentialValueConservation],
            Self::ResourceLimits => &[P::ConsensusResourceLimits, P::PolicyResourceLimits],
            Self::TapleafAndTapbranchHashing
            | Self::XOnlyKeyEncoding
            | Self::FeeRoleRecognition => &[],
        }
    }
}

/// A published plan requirement or one of the additional target groups.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaturityRequirement {
    /// An abstract capability published by this representation.
    Capability(RequiredCapability),
    /// A layout row published by this representation.
    Layout(LayoutRequirement),
    /// An external-evidence role published by this representation.
    ExternalEvidence(ExternalEvidenceRole),
    /// A representation-independent target capability group.
    Group(MaturityCapabilityGroup),
}

/// One requirement's disposition, ground, and target obligations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityVerdict {
    requirement: MaturityRequirement,
    target_wide: Option<AssessmentDisposition>,
    operation: AssessmentDisposition,
    ground: VerdictGround,
    primitives: BTreeSet<ElementsCapability>,
    structural: BTreeSet<BackendFoundationRequirement>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl MaturityVerdict {
    /// The requirement this verdict answers.
    #[must_use]
    pub const fn requirement(&self) -> &MaturityRequirement {
        &self.requirement
    }

    /// The target-wide answer; absent for layout and group rows.
    #[must_use]
    pub const fn target_wide(&self) -> Option<AssessmentDisposition> {
        self.target_wide
    }

    /// The maturity-announcement disposition.
    #[must_use]
    pub const fn operation(&self) -> AssessmentDisposition {
        self.operation
    }

    /// Why this disposition applies.
    #[must_use]
    pub const fn ground(&self) -> VerdictGround {
        self.ground
    }

    /// Required target capabilities; conservation names a consensus claim.
    #[must_use]
    pub const fn primitives(&self) -> &BTreeSet<ElementsCapability> {
        &self.primitives
    }

    /// Structural obligations retained from the capability mapping.
    #[must_use]
    pub const fn structural(&self) -> &BTreeSet<BackendFoundationRequirement> {
        &self.structural
    }

    /// The target evidence still required, including operator discharge semantics.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    #[cfg(test)]
    pub(crate) fn restate(
        &mut self,
        requirement: MaturityRequirement,
        operation: AssessmentDisposition,
    ) {
        self.requirement = requirement;
        self.operation = operation;
    }
}

/// Both exact representation censuses; no unchecked public construction exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAssessmentSet {
    projections: BTreeMap<Representation, BTreeMap<MaturityRequirement, MaturityVerdict>>,
}

impl MaturityAssessmentSet {
    /// One representation's complete verdict map.
    #[must_use]
    pub fn projection(
        &self,
        representation: Representation,
    ) -> Option<&BTreeMap<MaturityRequirement, MaturityVerdict>> {
        self.projections.get(&representation)
    }

    /// Every verdict of one representation, in requirement order.
    pub fn verdicts(
        &self,
        representation: Representation,
    ) -> impl Iterator<Item = &MaturityVerdict> {
        self.projection(representation)
            .into_iter()
            .flat_map(BTreeMap::values)
    }

    /// One requirement's verdict in one representation.
    #[must_use]
    pub fn verdict(
        &self,
        representation: Representation,
        requirement: &MaturityRequirement,
    ) -> Option<&MaturityVerdict> {
        self.projection(representation)?.get(requirement)
    }

    /// Nonzero row counts by disposition in one representation.
    #[must_use]
    pub fn state_census(
        &self,
        representation: Representation,
    ) -> BTreeMap<AssessmentDisposition, usize> {
        let mut census = BTreeMap::new();
        for row in self.verdicts(representation) {
            *census.entry(row.operation).or_insert(0) += 1;
        }
        census
    }

    /// Nonzero row counts by ground in one representation.
    #[must_use]
    pub fn ground_census(&self, representation: Representation) -> BTreeMap<VerdictGround, usize> {
        let mut census = BTreeMap::new();
        for row in self.verdicts(representation) {
            *census.entry(row.ground).or_insert(0) += 1;
        }
        census
    }

    /// Total verdict rows across both representations, including repeated groups.
    #[must_use]
    pub fn len(&self) -> usize {
        self.projections.values().map(BTreeMap::len).sum()
    }

    /// Whether there are no verdict rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.projections.values().all(BTreeMap::is_empty)
    }

    /// Verify equality of the full group verdicts, including their grounds.
    ///
    /// # Errors
    /// Returns a typed group disagreement for an unequal or absent group row.
    pub fn check_group_agreement(&self) -> Result<(), TapscriptError> {
        for group in MaturityCapabilityGroup::ALL {
            let requirement = MaturityRequirement::Group(*group);
            let explicit = self.verdict(Representation::Explicit, &requirement);
            if explicit.is_none()
                || explicit != self.verdict(Representation::PublicCommitted, &requirement)
            {
                return Err(TapscriptError::MaturityGroupDisagreement { group: *group });
            }
        }
        Ok(())
    }
}

/// Assess each published requirement and each target group for both STATE modes.
///
/// No maturity pattern is established. Compact and live patterns are not
/// admissible maturity completion claims; no pattern argument or lookup exists.
/// Operator evidence remains external and names signature and sighash semantics
/// as its discharge path, without establishing the operator identity or profile.
///
/// # Errors
/// Returns duplicate-requirement, exact-census, or group-disagreement errors
/// if the assembled verdicts disagree with the plan and the group census.
pub fn assess_maturity_announcement_plan(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<MaturityAssessmentSet, TapscriptError> {
    assess_validated_maturity_plan(target.validated(), plan)
}

/// The shared core permits degraded-target tests without a public trust bypass.
pub(crate) fn assess_validated_maturity_plan(
    target: &ValidatedTargetDefinition,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<MaturityAssessmentSet, TapscriptError> {
    let rows = plan.representations().flat_map(|projection| {
        requirements(projection)
            .map(move |requirement| (projection.plan(), requirement_verdict(target, requirement)))
    });
    from_rows(plan, rows)
}

fn requirements(
    projection: &MaturityAnnouncementRepresentationProjection,
) -> impl Iterator<Item = MaturityRequirement> + '_ {
    projection
        .capabilities()
        .map(MaturityRequirement::Capability)
        .chain(
            projection
                .layout()
                .cloned()
                .map(MaturityRequirement::Layout),
        )
        .chain(
            projection
                .external_evidence()
                .map(MaturityRequirement::ExternalEvidence),
        )
        .chain(
            MaturityCapabilityGroup::ALL
                .iter()
                .copied()
                .map(MaturityRequirement::Group),
        )
}

/// Validate independently supplied rows, used by production and corruption tests.
pub(crate) fn from_rows(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    rows: impl IntoIterator<Item = (Representation, MaturityVerdict)>,
) -> Result<MaturityAssessmentSet, TapscriptError> {
    let mut assessment = MaturityAssessmentSet {
        projections: BTreeMap::new(),
    };
    for (representation, row) in rows {
        if assessment
            .projections
            .entry(representation)
            .or_default()
            .insert(row.requirement.clone(), row)
            .is_some()
        {
            return Err(TapscriptError::DuplicateOperationRequirement);
        }
    }
    for projection in plan.representations() {
        let expected = requirements(projection).collect::<BTreeSet<_>>();
        let actual = assessment
            .verdicts(projection.plan())
            .map(|row| row.requirement.clone())
            .collect::<BTreeSet<_>>();
        if expected != actual {
            return Err(TapscriptError::MaturityAssessmentCensusMismatch {
                representation: projection.plan(),
                missing: expected.difference(&actual).cloned().collect(),
                unexpected: actual.difference(&expected).cloned().collect(),
            });
        }
    }
    assessment.check_group_agreement()?;
    Ok(assessment)
}

fn requirement_verdict(
    target: &ValidatedTargetDefinition,
    requirement: MaturityRequirement,
) -> MaturityVerdict {
    use AssessmentDisposition as D;
    let mut row = MaturityVerdict {
        requirement,
        target_wide: None,
        operation: D::BackendPatternRequired,
        ground: VerdictGround::NoApprovedPattern,
        primitives: BTreeSet::new(),
        structural: BTreeSet::new(),
        evidence: BTreeSet::new(),
    };
    match &row.requirement {
        MaturityRequirement::Capability(capability) => {
            let assessment = assess_validated_capability(target, *capability).projection();
            row.target_wide = Some(assessment.disposition());
            row.operation = match assessment.disposition() {
                D::CompleteBackendPattern => D::BackendPatternRequired,
                disposition => disposition,
            };
            row.primitives.extend(assessment.primitives());
            row.structural.extend(assessment.structural());
            row.evidence.extend(assessment.evidence());
            // A family count still needs a STATE authentication pattern.
            if *capability == RequiredCapability::AuthenticatedFamilyCardinality
                && row.operation == D::BackendStructural
            {
                row.operation = D::BackendPatternRequired;
            }
        }
        MaturityRequirement::Layout(layout) => row.operation = layout_disposition(layout),
        MaturityRequirement::ExternalEvidence(role) => {
            row.operation = D::ExternalEvidenceRequired;
            row.target_wide = Some(D::ExternalEvidenceRequired);
            row.evidence
                .extend(assess_evidence_role(*role).projection().evidence());
        }
        MaturityRequirement::Group(group) => {
            row.primitives.extend(group.target_capabilities());
            row.operation = group_disposition(target, *group);
            for capability in &row.primitives {
                if let Some(contract) = target.definition().capabilities().get(capability) {
                    row.evidence.extend(contract.evidence());
                }
            }
        }
    }
    row.ground = match row.operation {
        D::BackendPatternRequired => VerdictGround::NoApprovedPattern,
        D::BackendStructural => VerdictGround::CompilerOrAbiObligation,
        D::ExternalEvidenceRequired => VerdictGround::ExternalTargetClaim,
        _ => VerdictGround::TargetWideAssessment,
    };
    row
}

const fn layout_disposition(layout: &LayoutRequirement) -> AssessmentDisposition {
    use AssessmentDisposition as D;
    match layout {
        LayoutRequirement::CanonicalCoordinator { .. } => D::BackendStructural,
        LayoutRequirement::MakeSourceAvailable { source, .. } => match source.source {
            RequiredSourceKind::PublicConstructionData => D::BackendStructural,
            RequiredSourceKind::ExternalEvidence => D::ExternalEvidenceRequired,
            _ => D::BackendPatternRequired,
        },
        LayoutRequirement::AuthenticateFamilyCensus { .. }
        | LayoutRequirement::CompleteAndDisjointFamilies { .. }
        | LayoutRequirement::IsolateSponsorRegion { .. }
        | LayoutRequirement::EnforceRepresentation { .. }
        | LayoutRequirement::SecretFreeOperationPath { .. } => D::BackendPatternRequired,
    }
}

fn group_disposition(
    target: &ValidatedTargetDefinition,
    group: MaturityCapabilityGroup,
) -> AssessmentDisposition {
    use AssessmentDisposition as D;
    // As in the pilot mapping, conservation is target evidence, not a
    // primitive-availability test. Its Incomplete registry row is an external
    // consensus claim and cannot be replaced with a script conservation proof.
    if group == MaturityCapabilityGroup::TargetTransactionConservation {
        return D::ExternalEvidenceRequired;
    }
    let mut missing = group.target_capabilities().is_empty();
    for capability in group.target_capabilities() {
        match target
            .definition()
            .capabilities()
            .get(capability)
            .map(target_elements::CapabilityContract::status)
        {
            Some(StaticCapabilityStatus::Reviewed) => {}
            Some(StaticCapabilityStatus::Unsupported) => return D::Unsupported,
            _ => missing = true,
        }
    }
    if missing {
        D::MissingTargetPrimitives
    } else if matches!(
        group,
        MaturityCapabilityGroup::ScriptPathExecution | MaturityCapabilityGroup::ResourceLimits
    ) {
        D::BackendStructural
    } else {
        D::BackendPatternRequired
    }
}
