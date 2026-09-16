//! Exact maturity-announcement requirements assessed against the static target.
//!
//! The plan-only assessment retains its standing obligations. A complete STATE
//! announcement record can replace individual rows, while registry restrictions
//! and external evidence remain visible. Carrier projection names emitted work
//! without claiming linked closure or target acceptance.

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
    /// hashing nor fee-role recognition has a registry entry. Those and x-only
    /// encoding map to the empty slice; record-backed composition is assessed
    /// separately and does not change this registry projection.
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

use crate::state_announcement::StateAnnouncementId as Semantic;
use crate::state_operator::StateOperatorPatternId;
use crate::state_pattern::StatePatternId as Structural;
use crate::state_program::{
    StateAnnouncementProgram, StateProgramComponent as Component, StateProgramMetadata,
};

const AUTHENTICATION: &[Component] = &[
    Component::Semantic(Semantic::MetadataAuthentication),
    Component::Semantic(Semantic::SuccessorReconstruction),
];
const PARTITION: &[Component] = &[
    Component::Structural(Structural::StateCardinalityV1),
    Component::Structural(Structural::StateSponsorIsolationV1),
];
const OPERATOR: Component = Component::Operator(StateOperatorPatternId::OperatorAuthorizationV1);

/// Component identities performing each group; an identity never overrides registry status.
///
/// Authentication composes tagged hashes and derives the x-only coordinate from
/// the witnessed parity prefix and introspected output key on both sides.
pub(crate) const fn group_components(group: MaturityCapabilityGroup) -> &'static [Component] {
    use MaturityCapabilityGroup as G;
    match group {
        // The coordinator verifies the introspected current input is zero.
        G::CurrentInputIndex => &[Component::Structural(Structural::StateCoordinatorRoleV1)],
        // The complete partition checks counts and isolates the fee role.
        G::InputAndOutputCount | G::FeeRoleRecognition => PARTITION,
        // Predecessor recognition checks the singleton asset and amount.
        G::InputAssetAndValueInspection => {
            &[Component::Structural(Structural::StateInputRecognitionV1)]
        }
        // Both constructors inspect programs, compare bytes, slice and hash metadata,
        // reconstruct tagged trees, derive x-only keys, and verify tweaks. The path
        // is walked and resource projected; execution and resource capabilities
        // must nevertheless occur in the prerequisite census.
        G::InputAndOutputProgramInspection
        | G::ByteEquality
        | G::ByteSlicingAndConcatenation
        | G::StreamingSha256
        | G::TapleafAndTapbranchHashing
        | G::TweakVerification
        | G::XOnlyKeyEncoding
        | G::ScriptPathExecution
        | G::ResourceLimits => AUTHENTICATION,
        // Checked lead bounds implement arithmetic; transaction header requirements
        // remain pending when their primitives are absent from the record.
        G::CheckedArithmeticAndComparisons | G::TransactionVersionAndLockTime => {
            &[Component::Semantic(Semantic::LeadWindow)]
        }
        // The operator checks the signature; unreviewed sighash entries still block.
        G::SignatureVerification | G::SelectedSighashSemantics => &[OPERATOR],
        // No component can replace the substrate's external consensus claim.
        G::TargetTransactionConservation => &[],
    }
}

/// Total abstract-capability mapping, including capabilities absent from this plan.
pub(crate) const fn capability_components(capability: RequiredCapability) -> &'static [Component] {
    use RequiredCapability as C;
    match capability {
        // Recognition needs both the predecessor and reconstructed successor.
        C::AuthenticatedObjectRecognition => &[
            Component::Structural(Structural::StateInputRecognitionV1),
            Component::Semantic(Semantic::SuccessorReconstruction),
        ],
        // The partition establishes family counts and canonical/open-flow regions.
        C::AuthenticatedFamilyCardinality
        | C::AuthenticatedCanonicalPartition
        | C::AuthenticatedOpenFlowPartition => PARTITION,
        // Authentication binds both constructors; any absent registry prerequisite
        // still prevents this broader root-effects capability from completing.
        C::AuthenticatedRootEffects | C::PublicConstructibility => AUTHENTICATION,
        // The semantic transition produces the certificate's successor metadata.
        C::AuthenticatedProjectionSet => &[Component::Semantic(Semantic::CopyThrough)],
        // The window implements checked public arithmetic.
        C::ExactPublicAmountArithmetic => &[Component::Semantic(Semantic::LeadWindow)],
        // The committed operator fragment supplies the in-script signature check.
        C::OperatorAuthorization => &[OPERATOR],
        // Announcement has no owner/refund gate or script conservation proof.
        C::OwnerAuthorization
        | C::RefundAuthorization
        | C::ConfidentialValueConservation
        | C::WholeTransactionValueConservation => &[],
    }
}

/// Total layout mapping; source routing is separately checked against metadata.
pub(crate) const fn layout_components(layout: &LayoutRequirement) -> &'static [Component] {
    match layout {
        // Counts and disjointness are established by the structural partition.
        LayoutRequirement::AuthenticateFamilyCensus { .. }
        | LayoutRequirement::CompleteAndDisjointFamilies { .. } => PARTITION,
        // The sponsor partition recognizes every sponsor and fee position.
        LayoutRequirement::IsolateSponsorRegion { .. } => {
            &[Component::Structural(Structural::StateSponsorIsolationV1)]
        }
        // Representation is authenticated on both constructor endpoints.
        LayoutRequirement::EnforceRepresentation { .. } => AUTHENTICATION,
        // A coordinator remains a compiler/ABI obligation in the standing result.
        LayoutRequirement::CanonicalCoordinator { .. } => {
            &[Component::Structural(Structural::StateCoordinatorRoleV1)]
        }
        // Source availability follows the record's source census, not a guessed fragment.
        // An operator-gated announcement cannot establish a secret-free path.
        LayoutRequirement::MakeSourceAvailable { .. }
        | LayoutRequirement::SecretFreeOperationPath { .. } => &[],
    }
}

/// Assess the complete record without weakening target or external obligations.
/// Evidence is unioned metadata: the record exposes no component-indexed evidence.
///
/// # Errors
/// Returns the same exact-census and group-agreement failures as plan assessment.
pub fn assess_maturity_announcement_program(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    program: &StateAnnouncementProgram,
) -> Result<MaturityAssessmentSet, TapscriptError> {
    assess_maturity_program_parts(
        target.validated(),
        plan,
        &program.components().keys().copied().collect(),
        program.prerequisites(),
        program.metadata(),
    )
}

/// Decisions over admitted record parts, also allowing component-removal tests.
pub(crate) fn assess_maturity_program_parts(
    target: &ValidatedTargetDefinition,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    components: &BTreeSet<Component>,
    prerequisites: &BTreeSet<ElementsCapability>,
    metadata: &StateProgramMetadata,
) -> Result<MaturityAssessmentSet, TapscriptError> {
    let mut assessment = assess_validated_maturity_plan(target, plan)?;
    for projection in assessment.projections.values_mut() {
        for row in projection.values_mut() {
            if record_backs_row(row, target, components, prerequisites, metadata) {
                row.operation = AssessmentDisposition::CompleteBackendPattern;
                row.ground = VerdictGround::ApprovedPattern;
                row.evidence.extend(&metadata.evidence);
            }
        }
    }
    assessment.check_group_agreement()?;
    Ok(assessment)
}

fn record_backs_row(
    row: &MaturityVerdict,
    target: &ValidatedTargetDefinition,
    components: &BTreeSet<Component>,
    prerequisites: &BTreeSet<ElementsCapability>,
    metadata: &StateProgramMetadata,
) -> bool {
    use AssessmentDisposition as D;
    let carrying = match &row.requirement {
        MaturityRequirement::Group(group) => {
            if row.operation == D::ExternalEvidenceRequired {
                return false;
            }
            group_components(*group)
        }
        MaturityRequirement::Capability(capability)
            if row.operation == D::BackendPatternRequired =>
        {
            capability_components(*capability)
        }
        MaturityRequirement::Layout(LayoutRequirement::MakeSourceAvailable { source, .. }) => {
            return row.operation != D::ExternalEvidenceRequired
                && metadata.sources.contains(&source.source);
        }
        MaturityRequirement::Layout(layout) if row.operation == D::BackendPatternRequired => {
            layout_components(layout)
        }
        _ => return false,
    };
    !carrying.is_empty()
        && carrying
            .iter()
            .all(|component| components.contains(component))
        && row.primitives.iter().all(|primitive| {
            prerequisites.contains(primitive)
                && target
                    .definition()
                    .capabilities()
                    .get(primitive)
                    .is_some_and(|contract| contract.status() == StaticCapabilityStatus::Reviewed)
        })
}

/// Exact replacement sets for one representation; unchanged rows retain obligations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityRecordCensus {
    /// Rows promoted to an approved complete pattern by this record.
    pub completed: BTreeSet<MaturityRequirement>,
    /// Rows retaining their standing result.
    pub remaining: BTreeSet<MaturityRequirement>,
}

/// Return exact replacement membership, independently for each representation.
///
/// # Errors
/// Returns assessment census or agreement failures.
pub fn maturity_announcement_record_census(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    program: &StateAnnouncementProgram,
) -> Result<BTreeMap<Representation, MaturityRecordCensus>, TapscriptError> {
    let assessment = assess_maturity_announcement_program(target, plan, program)?;
    Ok(plan
        .representations()
        .map(|projection| {
            let (completed, remaining) = assessment
                .verdicts(projection.plan())
                .partition::<Vec<_>, _>(|row| {
                    row.operation == AssessmentDisposition::CompleteBackendPattern
                });
            (
                projection.plan(),
                MaturityRecordCensus {
                    completed: completed
                        .into_iter()
                        .map(|row| row.requirement.clone())
                        .collect(),
                    remaining: remaining
                        .into_iter()
                        .map(|row| row.requirement.clone())
                        .collect(),
                },
            )
        })
        .collect())
}

use compiler::operation_plan::TargetRelationRequirement;
use realization::{
    ExternalEvidenceRequirement, RelationId, RelationKind, RelationSubject, TransactionSide,
};

/// Emitted code or a named external obligation, never a target-acceptance claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityCarrier {
    /// This component occurs in the admitted announcement recipe.
    Emitted(Component),
    /// Realization's exact outstanding requirement, including operation and asset.
    External(ExternalEvidenceRequirement),
}

/// A relation that could not be assigned a present, unambiguous carrier.
/// The existing adapter error root has no relation-bearing variant.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("maturity carrier refused for {relation:?}: {reason:?}")]
pub struct MaturityCarrierRefusal {
    /// Exact relation identity, retained on every refusal path.
    pub relation: RelationId,
    /// Why no complete projection can be published.
    pub reason: MaturityCarrierRefusalReason,
}

/// Closed reasons for refusing one carrier decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityCarrierRefusalReason {
    /// No announcement component implements this kind and subject.
    Unmapped,
    /// The carrying component is absent from the supplied component set.
    MissingComponent(Component),
    /// One carrier cannot represent multiple external requirements.
    MultipleExternalRequirements,
    /// The representations do not publish the same carrier for this relation.
    RepresentationDisagreement,
}

/// One carrier per relation, with identical projections for the two STATE modes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityCarrierProjection {
    projections: BTreeMap<Representation, BTreeMap<RelationId, MaturityCarrier>>,
}

impl MaturityCarrierProjection {
    /// The complete relation map for one representation.
    #[must_use]
    pub fn projection(
        &self,
        representation: Representation,
    ) -> Option<&BTreeMap<RelationId, MaturityCarrier>> {
        self.projections.get(&representation)
    }
}

/// Project every relation to an emitted component or realization's external claim.
///
/// Operator authorization is emitted because the fragment verifies the signature
/// in-script; the transaction membership producer supplies its separate membership
/// half. Operator constructibility retains its own external requirement. This
/// projection establishes neither target enforcement nor linked or ABI closure.
///
/// # Errors
/// Names any unplaceable relation, absent component, or representation disagreement.
pub fn project_maturity_carriers(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    program: &StateAnnouncementProgram,
) -> Result<MaturityCarrierProjection, MaturityCarrierRefusal> {
    project_maturity_carrier_parts(plan, &program.components().keys().copied().collect())
}

/// Projection over record parts allows absence tests without counterfeit records.
pub(crate) fn project_maturity_carrier_parts(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    components: &BTreeSet<Component>,
) -> Result<MaturityCarrierProjection, MaturityCarrierRefusal> {
    let mut projections = BTreeMap::new();
    let mut baseline: Option<BTreeMap<RelationId, MaturityCarrier>> = None;
    for projection in plan.representations() {
        let rows = projection
            .relations()
            .map(|row| {
                maturity_relation_carrier(plan, row, components)
                    .map(|carrier| (row.relation.clone(), carrier))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        if let Some(first) = &baseline {
            for relation in first.keys().chain(rows.keys()) {
                if first.get(relation) != rows.get(relation) {
                    return Err(MaturityCarrierRefusal {
                        relation: relation.clone(),
                        reason: MaturityCarrierRefusalReason::RepresentationDisagreement,
                    });
                }
            }
        } else {
            baseline = Some(rows.clone());
        }
        projections.insert(projection.plan(), rows);
    }
    Ok(MaturityCarrierProjection { projections })
}

/// Decide one relation, preserving its identity on every refusal.
pub(crate) fn maturity_relation_carrier(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    row: &TargetRelationRequirement,
    components: &BTreeSet<Component>,
) -> Result<MaturityCarrier, MaturityCarrierRefusal> {
    let refuse = |reason| MaturityCarrierRefusal {
        relation: row.relation.clone(),
        reason,
    };
    if row.external_evidence.len() > 1 {
        return Err(refuse(
            MaturityCarrierRefusalReason::MultipleExternalRequirements,
        ));
    }
    if &row.relation != plan.operator().authorization()
        && let Some(external) = row.external_evidence.first()
    {
        return Ok(MaturityCarrier::External(external.clone()));
    }
    let component = relation_component(plan, &row.relation)
        .ok_or_else(|| refuse(MaturityCarrierRefusalReason::Unmapped))?;
    if !components.contains(&component) {
        return Err(refuse(MaturityCarrierRefusalReason::MissingComponent(
            component,
        )));
    }
    Ok(MaturityCarrier::Emitted(component))
}

/// Total kind/subject decision: unknown combinations refuse instead of guessing.
/// Structural arms are coordinator structure; semantic/operator arms are announcement
/// leaf work. Authentication implements constructor checks inside that leaf, without
/// asserting the separate linked-constructor carrier class is closed.
fn relation_component(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    relation: &RelationId,
) -> Option<Component> {
    use RelationKind as K;
    use RelationSubject as S;
    if relation.operation() != plan.operation() {
        return None;
    }
    match (relation.kind(), relation.subject()) {
        // Both STATE counts use the complete structural partition.
        (K::Cardinality, S::ObjectFamily { object, .. }) if object == plan.state().object() => {
            Some(Component::Structural(Structural::StateCardinalityV1))
        }
        // Sponsor counts and recognition use the isolated suffix, never STATE values.
        (K::Cardinality | K::Recognition, S::ObjectFamily { object, .. })
            if *object == plan.sponsor().object() =>
        {
            Some(Component::Structural(Structural::StateSponsorIsolationV1))
        }
        // Input recognition binds the consumed singleton's asset, amount and program.
        (
            K::Recognition,
            S::ObjectFamily {
                side: TransactionSide::Input,
                object,
            },
        ) if object == plan.state().object() => {
            Some(Component::Structural(Structural::StateInputRecognitionV1))
        }
        // Output recognition and representation authenticate the successor constructor.
        (
            K::Recognition,
            S::ObjectFamily {
                side: TransactionSide::Output,
                object,
            },
        )
        | (K::Representation, S::Representation { object })
            if object == plan.state().object() =>
        {
            Some(Component::Semantic(Semantic::SuccessorReconstruction))
        }
        // Family closure and empty canonical deltas exclude foreign families and issuance.
        (K::AllowedObjectFamilies, S::TransactionSide { .. })
        | (K::CanonicalDeltaPolicy, S::Operation) => {
            Some(Component::Structural(Structural::StateIssuanceAbsenceV1))
        }
        // Sponsor envelope and open-flow policies share the isolated partition.
        (K::SponsorIsolation | K::SponsorEnvelopeMultiplicity, S::Sponsor)
        | (K::OpenFlowPolicy, S::Operation) => {
            Some(Component::Structural(Structural::StateSponsorIsolationV1))
        }
        // Authorization verifies the committed operator key in the announcement leaf.
        (K::Authorization, S::Operation) => Some(OPERATOR),
        // Root continuity authenticates the predecessor; constructibility without
        // external evidence uses the same authentication endpoint.
        (K::RootPolicy | K::Constructibility, S::Operation) => {
            Some(Component::Semantic(Semantic::MetadataAuthentication))
        }
        // The certificate binds the copied fields and requested successor maturity.
        (K::ProjectionPolicy, S::Operation) => Some(Component::Semantic(Semantic::CopyThrough)),
        // This operation's exit is checked against the inclusive announcement window.
        (K::Lifecycle, S::LifecycleExit { object, exit })
            if object == plan.state().object() && *exit == plan.operation() =>
        {
            Some(Component::Semantic(Semantic::LeadWindow))
        }
        // Other declared exits retain the predecessor maturity eligibility check;
        // this carrier assignment does not claim implementations of those exits.
        (K::Lifecycle, S::LifecycleExit { object, exit })
            if object == plan.state().object()
                && plan
                    .lifecycle()
                    .outstanding()
                    .any(|candidate| candidate == *exit) =>
        {
            Some(Component::Semantic(Semantic::MaturityPredecessor))
        }
        // Unsupported kinds or subjects have no announcement carrier.
        _ => None,
    }
}
