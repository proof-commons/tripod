//! Validated maturity-announcement contracts and their representation-specific rows.
//!
//! An announcement consumes and creates exactly one STATE. Its transition is a
//! symbolic field requirement, operator authorization remains external evidence,
//! and STATE succession remains a history duty. Neither a receipt conservation
//! proof nor owner-family authorization describes this operation.
//!
//! The source retains canonical declarations; each projection is derived from
//! them and checked against the announcement contract before publication.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId, ProjectionRule, RootId,
    RootUse,
};
use realization::{
    AvailabilityClass, CardinalityMaximum, ConstructibilityClass, ConstructibilityEdgeRole,
    ConstructibilityNodeId, Count, DisclosureNodeId, ExpectedCanonicalDelta,
    ExternalEvidenceRequirement, FactId, ObservedSide, Relation, RelationDeclaration, RelationId,
    RepresentationMode, RequirementStrength, WitnessRole,
};

use crate::{
    AnnouncementMetadataRequirement, CompileError, ConstructorContinuityRequirement,
    PublicRecoveryRequirement, RootHistoryRequirement, StateLawOperand,
    analyzed::{AnalyzedProofPlan, ScopedAnalyzedProgram, analyze_scoped_program},
    analyzed_operation::AnalyzedOperation,
    capability::census_enum,
    case::ExecutionCaseId,
    coverage_graph::{CoverageEdge, CoverageNodeId},
    input::BoundCompilerInput,
    layout::names_sponsor_amount,
    lifecycle::RepresentationChoiceId,
    operation_plan::{
        operation_relations, project_carriers, project_cases, project_coverage, project_lifecycle,
        project_relations,
    },
    sponsor_region::{ORDINARY_LBTC, ordinary_lbtc_role_of},
    target::canonical_census,
};
pub use crate::{
    capability::RequiredCapability,
    coverage::CoverageRequirementId,
    layout::LayoutRequirement,
    operation_plan::{
        AbstractCarrierRequirement, TargetCoverageRequirement, TargetExecutionCase,
        TargetLifecycleStatus, TargetOperationSource, TargetRelationRequirement,
    },
    placement::PlacementSearchLimits,
    sponsor_region::OrdinaryLbtcRole,
    target::ExternalEvidenceRole,
};

const PLANNED: OperationId = OperationId::AnnounceMaturity;
const STATE: ObjectId = ObjectId::State;
const STATE_ASSET: AssetId = AssetId::Pid;
const SPONSOR_ASSET: AssetId = AssetId::Lbtc;
const REQUIRED_PROJECTION: ProjectionId = ProjectionId::TransitionCertificate;
const IMPLEMENTED_EXIT: OperationId = PLANNED;
const OUTSTANDING_EXITS: [OperationId; 5] = [
    OperationId::AdmitDeposits,
    OperationId::Cycle,
    OperationId::Redeem,
    OperationId::ReceiptRelabel,
    OperationId::Clear,
];

census_enum! {
    /// The two STATE modes whose semantic facts remain public.
    pub enum MaturityAnnouncementRepresentationPlan {
        /// Explicit STATE facts.
        Explicit,
        /// Public STATE facts with an abstract public-opening requirement.
        PublicCommitted,
    }
}
impl MaturityAnnouncementRepresentationPlan {
    /// The realization mode selected by this plan.
    #[must_use]
    pub const fn mode(self) -> RepresentationMode {
        match self {
            Self::Explicit => RepresentationMode::Explicit,
            Self::PublicCommitted => RepresentationMode::PublicCommitted,
        }
    }
    /// The admitted plan for a realization mode, if any.
    #[must_use]
    pub const fn of(mode: RepresentationMode) -> Option<Self> {
        match mode {
            RepresentationMode::Explicit => Some(Self::Explicit),
            RepresentationMode::PublicCommitted => Some(Self::PublicCommitted),
            RepresentationMode::PrivateCommitted => None,
        }
    }
}

census_enum! {
    /// Which announcement contract failed before a plan could be returned.
    pub enum MaturityAnnouncementClause {
        /// Exactly one retained operation declaration and its complete census.
        Declaration,
        /// STATE recognition on both sides uses the protocol recognition asset.
        ProtocolObject,
        /// Exactly one STATE is consumed and created.
        ProtocolCardinality,
        /// Both sides admit only STATE and the sponsor family.
        ClassClosure,
        /// Both operator relations and the required witness agree.
        OperatorAuthorization,
        /// No canonical delta is expected.
        CanonicalDelta,
        /// The sponsor recognition identifies ordinary L-BTC.
        SponsorObject,
        /// Only the fee-sponsor open flow is admitted.
        SponsorFlow,
        /// The sponsor isolation relation is present exactly once.
        SponsorIsolation,
        /// At most one sponsor envelope is admitted.
        SponsorEnvelope,
        /// The external substrate requirement names L-BTC.
        SubstrateConservation,
        /// Optional sponsor inputs use the architecture maximum.
        SponsorInputs,
        /// Optional sponsor change has maximum one.
        SponsorChange,
        /// STATE succeeds and every other root is forbidden.
        RootPolicy,
        /// Only the transition certificate is required.
        CertificateProjection,
        /// Exactly the two public STATE modes are approved.
        RepresentationApproval,
        /// The exit closure follows the complete scoped lifecycle.
        LifecycleExits,
        /// The fifteen required facts are declared public.
        PublicFacts,
        /// Field dependencies and retained requirements agree.
        Transition,
    }
}

/// The declaration-approved STATE modes; both expose the transition facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementRepresentationPolicy {
    approved: BTreeSet<RepresentationMode>,
    admitted: BTreeSet<MaturityAnnouncementRepresentationPlan>,
    relation: RelationId,
}

impl MaturityAnnouncementRepresentationPolicy {
    /// Every mode approved by the declaration.
    #[must_use]
    pub const fn approved(&self) -> &BTreeSet<RepresentationMode> {
        &self.approved
    }
    /// Every plan admitted at this boundary.
    #[must_use]
    pub const fn admitted(&self) -> &BTreeSet<MaturityAnnouncementRepresentationPlan> {
        &self.admitted
    }
    /// The representation-policy relation.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }
}

/// One declared family count, retaining architecture-owned maxima.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementCardinality {
    minimum: Count,
    maximum: CardinalityMaximum,
}

impl MaturityAnnouncementCardinality {
    /// Read the minimum projection.
    #[must_use]
    pub const fn minimum(self) -> Count {
        self.minimum
    }

    /// Read the maximum projection.
    #[must_use]
    pub const fn maximum(self) -> CardinalityMaximum {
        self.maximum
    }

    /// Read the is optional projection.
    #[must_use]
    pub const fn is_optional(self) -> bool {
        self.minimum.is_zero()
    }
}

/// The optional sponsor region and its isolation contract, also required by announcements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementSponsorProjection {
    object: ObjectId,
    region: OrdinaryLbtcRole,
    open_flows: BTreeSet<OpenFlowKind>,
    envelope_maximum: Count,
    input_cardinality: MaturityAnnouncementCardinality,
    change_cardinality: MaturityAnnouncementCardinality,
    isolation: RelationId,
    multiplicity: RelationId,
    open_flow_policy: RelationId,
    substrate_conservation: RelationId,
}

impl MaturityAnnouncementSponsorProjection {
    /// Read the object projection.
    #[must_use]
    pub const fn object(&self) -> ObjectId {
        self.object
    }

    /// Read the region projection.
    #[must_use]
    pub const fn region(&self) -> OrdinaryLbtcRole {
        self.region
    }

    /// Read the open flows projection.
    pub fn open_flows(&self) -> impl Iterator<Item = OpenFlowKind> + '_ {
        self.open_flows.iter().copied()
    }

    /// Read the envelope maximum projection.
    #[must_use]
    pub const fn envelope_maximum(&self) -> Count {
        self.envelope_maximum
    }

    /// Read the input cardinality projection.
    #[must_use]
    pub const fn input_cardinality(&self) -> MaturityAnnouncementCardinality {
        self.input_cardinality
    }

    /// Read the change cardinality projection.
    #[must_use]
    pub const fn change_cardinality(&self) -> MaturityAnnouncementCardinality {
        self.change_cardinality
    }

    /// Read the is optional projection.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        self.input_cardinality.is_optional()
    }

    /// Read the isolation projection.
    #[must_use]
    pub const fn isolation(&self) -> &RelationId {
        &self.isolation
    }

    /// Read the multiplicity projection.
    #[must_use]
    pub const fn multiplicity(&self) -> &RelationId {
        &self.multiplicity
    }

    /// Read the open flow policy projection.
    #[must_use]
    pub const fn open_flow_policy(&self) -> &RelationId {
        &self.open_flow_policy
    }

    /// Read the substrate conservation projection.
    #[must_use]
    pub const fn substrate_conservation(&self) -> &RelationId {
        &self.substrate_conservation
    }
}

/// The root-policy map; an announcement requires STATE succession.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementRootProjection {
    expected: BTreeMap<RootId, RootUse>,
    relation: RelationId,
}

impl MaturityAnnouncementRootProjection {
    /// Read the expected projection.
    pub fn expected(&self) -> impl Iterator<Item = (RootId, RootUse)> + '_ {
        self.expected.iter().map(|(root, use_)| (*root, *use_))
    }

    /// Read the root use projection.
    #[must_use]
    pub fn root_use(&self, root: RootId) -> Option<RootUse> {
        self.expected.get(&root).copied()
    }

    /// Read the relation projection.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }
}

/// The semantic projection policy; announcements require a transition certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementCertificateProjection {
    expected: BTreeMap<ProjectionId, ProjectionRule>,
    relation: RelationId,
}

impl MaturityAnnouncementCertificateProjection {
    /// Read the expected projection.
    pub fn expected(&self) -> impl Iterator<Item = (ProjectionId, ProjectionRule)> + '_ {
        self.expected.iter().map(|(id, rule)| (*id, *rule))
    }

    /// Read the rule projection.
    #[must_use]
    pub fn rule(&self, projection: ProjectionId) -> Option<ProjectionRule> {
        self.expected.get(&projection).copied()
    }

    /// Read the required projection.
    pub fn required(&self) -> impl Iterator<Item = ProjectionId> + '_ {
        self.rules(ProjectionRule::Required)
    }

    /// Read the forbidden projection.
    pub fn forbidden(&self) -> impl Iterator<Item = ProjectionId> + '_ {
        self.rules(ProjectionRule::Forbidden)
    }

    /// Read the relation projection.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    fn rules(&self, wanted: ProjectionRule) -> impl Iterator<Item = ProjectionId> + '_ {
        self.expected
            .iter()
            .filter(move |(_, rule)| **rule == wanted)
            .map(|(projection, _)| *projection)
    }
}

/// The implemented and outstanding exit sets shared by both STATE modes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementLifecycleClosure {
    implemented: BTreeSet<OperationId>,
    outstanding: BTreeSet<OperationId>,
}

impl MaturityAnnouncementLifecycleClosure {
    /// Read the implemented projection.
    pub fn implemented(&self) -> impl Iterator<Item = OperationId> + '_ {
        self.implemented.iter().copied()
    }

    /// Read the outstanding projection.
    pub fn outstanding(&self) -> impl Iterator<Item = OperationId> + '_ {
        self.outstanding.iter().copied()
    }

    /// Read the release complete projection.
    #[must_use]
    pub fn release_complete(&self) -> bool {
        self.outstanding.is_empty()
    }
}

/// Generic operation rows for one STATE mode; these retain obligations, not completed evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementRepresentationProjection {
    plan: MaturityAnnouncementRepresentationPlan,
    cases: BTreeMap<ExecutionCaseId, TargetExecutionCase>,
    relations: BTreeMap<RelationId, TargetRelationRequirement>,
    carriers: BTreeSet<AbstractCarrierRequirement>,
    layout: BTreeSet<LayoutRequirement>,
    coverage: BTreeMap<CoverageRequirementId, TargetCoverageRequirement>,
    capabilities: BTreeSet<RequiredCapability>,
    external_evidence: BTreeSet<ExternalEvidenceRole>,
    lifecycle: TargetLifecycleStatus,
}

impl MaturityAnnouncementRepresentationProjection {
    /// Read the plan projection.
    #[must_use]
    pub const fn plan(&self) -> MaturityAnnouncementRepresentationPlan {
        self.plan
    }

    /// Read the cases projection.
    pub fn cases(&self) -> impl Iterator<Item = &TargetExecutionCase> {
        self.cases.values()
    }

    /// Read the case projection.
    #[must_use]
    pub fn case(&self, id: &ExecutionCaseId) -> Option<&TargetExecutionCase> {
        self.cases.get(id)
    }

    /// Read the relations projection.
    pub fn relations(&self) -> impl Iterator<Item = &TargetRelationRequirement> {
        self.relations.values()
    }

    /// Read the relation projection.
    #[must_use]
    pub fn relation(&self, relation: &RelationId) -> Option<&TargetRelationRequirement> {
        self.relations.get(relation)
    }

    /// Read the carriers projection.
    pub fn carriers(&self) -> impl Iterator<Item = &AbstractCarrierRequirement> {
        self.carriers.iter()
    }

    /// Read the layout projection.
    pub fn layout(&self) -> impl Iterator<Item = &LayoutRequirement> {
        self.layout.iter()
    }

    /// Read the coverage projection.
    pub fn coverage(&self) -> impl Iterator<Item = &TargetCoverageRequirement> {
        self.coverage.values()
    }

    /// Read the coverage requirement projection.
    #[must_use]
    pub fn coverage_requirement(
        &self,
        id: &CoverageRequirementId,
    ) -> Option<&TargetCoverageRequirement> {
        self.coverage.get(id)
    }

    /// Read the capabilities projection.
    pub fn capabilities(&self) -> impl Iterator<Item = RequiredCapability> + '_ {
        self.capabilities.iter().copied()
    }

    /// Read the external evidence projection.
    pub fn external_evidence(&self) -> impl Iterator<Item = ExternalEvidenceRole> + '_ {
        self.external_evidence.iter().copied()
    }

    /// Read the lifecycle projection.
    #[must_use]
    pub const fn lifecycle(&self) -> &TargetLifecycleStatus {
        &self.lifecycle
    }
}

/// STATE identity, side closure, and exact counts; no receipt-value sum is implied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementStateProjection {
    object: ObjectId,
    asset: AssetId,
    input_recognition: RelationId,
    output_recognition: RelationId,
    input_cardinality_relation: RelationId,
    output_cardinality_relation: RelationId,
    input_cardinality: MaturityAnnouncementCardinality,
    output_cardinality: MaturityAnnouncementCardinality,
    input_closure: RelationId,
    output_closure: RelationId,
    admitted: BTreeSet<ObjectId>,
    forbidden: BTreeSet<ObjectId>,
}

impl MaturityAnnouncementStateProjection {
    /// The protocol object recognized on both sides.
    #[must_use]
    pub const fn object(&self) -> &ObjectId {
        &self.object
    }
    /// The asset used to recognize STATE.
    #[must_use]
    pub const fn asset(&self) -> &AssetId {
        &self.asset
    }
    /// The declaration-owned input recognition identity.
    #[must_use]
    pub const fn input_recognition(&self) -> &RelationId {
        &self.input_recognition
    }
    /// The declaration-owned output recognition identity.
    #[must_use]
    pub const fn output_recognition(&self) -> &RelationId {
        &self.output_recognition
    }
    /// The declaration-owned input count identity.
    #[must_use]
    pub const fn input_cardinality_relation(&self) -> &RelationId {
        &self.input_cardinality_relation
    }
    /// The declaration-owned output count identity.
    #[must_use]
    pub const fn output_cardinality_relation(&self) -> &RelationId {
        &self.output_cardinality_relation
    }
    /// The exact input count.
    #[must_use]
    pub const fn input_cardinality(&self) -> &MaturityAnnouncementCardinality {
        &self.input_cardinality
    }
    /// The exact output count.
    #[must_use]
    pub const fn output_cardinality(&self) -> &MaturityAnnouncementCardinality {
        &self.output_cardinality
    }
    /// The input-family closure relation.
    #[must_use]
    pub const fn input_closure(&self) -> &RelationId {
        &self.input_closure
    }
    /// The output-family closure relation.
    #[must_use]
    pub const fn output_closure(&self) -> &RelationId {
        &self.output_closure
    }
    /// Both admitted object families.
    #[must_use]
    pub const fn admitted(&self) -> &BTreeSet<ObjectId> {
        &self.admitted
    }
    /// The complement of the admitted families in the object census.
    #[must_use]
    pub const fn forbidden(&self) -> &BTreeSet<ObjectId> {
        &self.forbidden
    }
}

/// Operator authorization joins two relations to one required witness and unresolved evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementOperatorProjection {
    authorization: RelationId,
    constructibility: RelationId,
    class: ConstructibilityClass,
    witness: ConstructibilityNodeId,
    evidence: ExternalEvidenceRequirement,
}

impl MaturityAnnouncementOperatorProjection {
    /// The operator-authorization relation.
    #[must_use]
    pub const fn authorization(&self) -> &RelationId {
        &self.authorization
    }
    /// The operator constructibility relation.
    #[must_use]
    pub const fn constructibility(&self) -> &RelationId {
        &self.constructibility
    }
    /// The declared authorization class.
    #[must_use]
    pub const fn class(&self) -> &ConstructibilityClass {
        &self.class
    }
    /// The required operator witness with its availability.
    #[must_use]
    pub const fn witness(&self) -> &ConstructibilityNodeId {
        &self.witness
    }
    /// The external operator evidence still required.
    #[must_use]
    pub const fn evidence(&self) -> &ExternalEvidenceRequirement {
        &self.evidence
    }
}

/// The empty canonical-delta policy; a STATE update introduces no conservation relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAnnouncementCanonicalProjection {
    expected: BTreeSet<ExpectedCanonicalDelta>,
    relation: RelationId,
}

impl MaturityAnnouncementCanonicalProjection {
    /// The declaration-owned expected delta set.
    #[must_use]
    pub const fn expected(&self) -> &BTreeSet<ExpectedCanonicalDelta> {
        &self.expected
    }
    /// The canonical-delta policy relation.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }
}

/// A complete checked announcement plan, constructible only through the validating entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedMaturityAnnouncementOperationPlan {
    operation: OperationId,
    source: TargetOperationSource,
    representation: MaturityAnnouncementRepresentationPolicy,
    state: MaturityAnnouncementStateProjection,
    operator: MaturityAnnouncementOperatorProjection,
    canonical: MaturityAnnouncementCanonicalProjection,
    sponsor: MaturityAnnouncementSponsorProjection,
    roots: MaturityAnnouncementRootProjection,
    certificate: MaturityAnnouncementCertificateProjection,
    lifecycle: MaturityAnnouncementLifecycleClosure,
    constructibility: ConstructorContinuityRequirement,
    public_facts: [FactId; 15],
    transition: AnnouncementMetadataRequirement,
    root_history: RootHistoryRequirement,
    public_recovery: PublicRecoveryRequirement,
    representations: BTreeMap<
        MaturityAnnouncementRepresentationPlan,
        MaturityAnnouncementRepresentationProjection,
    >,
}

impl ValidatedMaturityAnnouncementOperationPlan {
    /// The fixed announcement operation.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }
    /// The complete typed source used by the analysis.
    #[must_use]
    pub const fn source(&self) -> &TargetOperationSource {
        &self.source
    }
    /// Both declaration-approved modes.
    #[must_use]
    pub const fn representation(&self) -> &MaturityAnnouncementRepresentationPolicy {
        &self.representation
    }
    /// STATE recognition, side counts, and family closure.
    #[must_use]
    pub const fn state(&self) -> &MaturityAnnouncementStateProjection {
        &self.state
    }
    /// Operator authorization and its outstanding evidence.
    #[must_use]
    pub const fn operator(&self) -> &MaturityAnnouncementOperatorProjection {
        &self.operator
    }
    /// The empty expected canonical flow.
    #[must_use]
    pub const fn canonical(&self) -> &MaturityAnnouncementCanonicalProjection {
        &self.canonical
    }
    /// The optional sponsor envelope.
    #[must_use]
    pub const fn sponsor(&self) -> &MaturityAnnouncementSponsorProjection {
        &self.sponsor
    }
    /// STATE succession and other-root absence.
    #[must_use]
    pub const fn roots(&self) -> &MaturityAnnouncementRootProjection {
        &self.roots
    }
    /// Transition-certificate presence and other-projection absence.
    #[must_use]
    pub const fn certificate(&self) -> &MaturityAnnouncementCertificateProjection {
        &self.certificate
    }
    /// Implemented and outstanding exit sets.
    #[must_use]
    pub const fn lifecycle(&self) -> &MaturityAnnouncementLifecycleClosure {
        &self.lifecycle
    }
    /// Required constructor continuity, retained without claiming discharge.
    #[must_use]
    pub const fn constructibility(&self) -> &ConstructorContinuityRequirement {
        &self.constructibility
    }
    /// The fifteen public fact keys retained from the requirement module.
    #[must_use]
    pub const fn public_facts(&self) -> &[FactId; 15] {
        &self.public_facts
    }
    /// Six field laws and their separate admissibility requirements.
    #[must_use]
    pub const fn transition(&self) -> &AnnouncementMetadataRequirement {
        &self.transition
    }
    /// Required succession and every history check.
    #[must_use]
    pub const fn root_history(&self) -> &RootHistoryRequirement {
        &self.root_history
    }
    /// Public reconstruction inputs and duties.
    #[must_use]
    pub const fn public_recovery(&self) -> &PublicRecoveryRequirement {
        &self.public_recovery
    }
    /// Every admitted representation's projection, in census order.
    pub fn representations(
        &self,
    ) -> impl Iterator<Item = &MaturityAnnouncementRepresentationProjection> {
        self.representations.values()
    }
    /// The projection of one admitted STATE mode.
    #[must_use]
    pub fn projection(
        &self,
        plan: MaturityAnnouncementRepresentationPlan,
    ) -> Option<&MaturityAnnouncementRepresentationProjection> {
        self.representations.get(&plan)
    }
}

/// Derive and validate the complete announcement operation plan.
///
/// The full scoped analysis must succeed before either representation can be
/// published. Requirements remain symbolic and external evidence remains open.
///
/// # Errors
/// Returns typed analysis errors on exhausted searches, an out-of-scope error
/// when the announcement is absent, or a contract/census error when declarations,
/// retained requirements, factors, or representation semantics disagree.
pub fn plan_maturity_announcement_target_operation(
    input: &BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<ValidatedMaturityAnnouncementOperationPlan, CompileError> {
    if !input.scope().operations().contains(&PLANNED) {
        return Err(CompileError::TargetOperationOutOfScope { operation: PLANNED });
    }
    let analyzed = analyze_scoped_program(input, placement_limits)?;
    let plan = derive_plan(&analyzed)?;
    validate_maturity_announcement_plan(&analyzed, &plan)?;
    Ok(plan)
}

pub(crate) fn derive_plan(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<ValidatedMaturityAnnouncementOperationPlan, CompileError> {
    validate_declaration(analyzed)?;
    let mut representations = BTreeMap::new();
    for mode in MaturityAnnouncementRepresentationPlan::ALL.iter().copied() {
        representations.insert(mode, derive_representation(analyzed, mode)?);
    }
    Ok(ValidatedMaturityAnnouncementOperationPlan {
        operation: PLANNED,
        source: TargetOperationSource::of(analyzed),
        representation: derive_representation_policy(analyzed)?,
        state: derive_state(analyzed)?,
        operator: derive_operator(analyzed)?,
        canonical: derive_canonical(analyzed)?,
        sponsor: derive_sponsor(analyzed)?,
        roots: derive_roots(analyzed)?,
        certificate: derive_certificate(analyzed)?,
        lifecycle: derive_lifecycle_closure(&representations)?,
        representations,
        constructibility: ConstructorContinuityRequirement::REQUIRED,
        public_facts: AnnouncementMetadataRequirement::public_facts(),
        transition: AnnouncementMetadataRequirement::required(),
        root_history: RootHistoryRequirement::REQUIRED,
        public_recovery: PublicRecoveryRequirement::REQUIRED,
    })
}

const fn defect(clause: MaturityAnnouncementClause) -> CompileError {
    CompileError::MaturityAnnouncementContractDefect { clause }
}

fn declarations(analyzed: &ScopedAnalyzedProgram) -> impl Iterator<Item = &RelationDeclaration> {
    analyzed
        .source
        .realization
        .relations
        .nodes
        .iter()
        .filter(|declaration| declaration.id.operation() == PLANNED)
}

fn sole<T>(
    analyzed: &ScopedAnalyzedProgram,
    clause: MaturityAnnouncementClause,
    read: impl Fn(&Relation) -> Option<T>,
) -> Result<(RelationId, T), CompileError> {
    let mut found = declarations(analyzed).filter_map(|declaration| {
        read(&declaration.relation).map(|value| (declaration.id.clone(), value))
    });

    let Some(first) = found.next() else {
        return Err(defect(clause));
    };

    if found.next().is_some() {
        return Err(defect(clause));
    }

    Ok(first)
}

fn cardinality(
    analyzed: &ScopedAnalyzedProgram,
    clause: MaturityAnnouncementClause,
    wanted_side: ObservedSide,
    wanted_object: ObjectId,
) -> Result<(RelationId, MaturityAnnouncementCardinality), CompileError> {
    sole(analyzed, clause, |relation| match relation {
        Relation::Cardinality {
            side,
            object,
            minimum,
            maximum,
        } if *side == wanted_side && *object == wanted_object => {
            Some(MaturityAnnouncementCardinality {
                minimum: *minimum,
                maximum: *maximum,
            })
        }
        _ => None,
    })
}

fn recognition(
    analyzed: &ScopedAnalyzedProgram,
    clause: MaturityAnnouncementClause,
    wanted_side: ObservedSide,
    wanted_asset: AssetId,
) -> Result<(RelationId, ObjectId), CompileError> {
    sole(analyzed, clause, |relation| match relation {
        Relation::Recognition {
            side,
            object,
            asset,
        } if *side == wanted_side && *asset == wanted_asset => Some(*object),
        _ => None,
    })
}

fn closure(
    analyzed: &ScopedAnalyzedProgram,
    wanted_side: ObservedSide,
) -> Result<(RelationId, BTreeSet<ObjectId>), CompileError> {
    sole(
        analyzed,
        MaturityAnnouncementClause::ClassClosure,
        |relation| match relation {
            Relation::AllowedObjectFamilies { side, allowed } if *side == wanted_side => {
                Some(allowed.clone())
            }
            _ => None,
        },
    )
}

fn derive_state(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<MaturityAnnouncementStateProjection, CompileError> {
    let clause = MaturityAnnouncementClause::ProtocolObject;
    let (input_recognition, input_object) =
        recognition(analyzed, clause, ObservedSide::Input, STATE_ASSET)?;
    let (output_recognition, output_object) =
        recognition(analyzed, clause, ObservedSide::Output, STATE_ASSET)?;
    if input_object != STATE || output_object != STATE {
        return Err(defect(clause));
    }
    let (input_closure, input_families) = closure(analyzed, ObservedSide::Input)?;
    let (output_closure, output_families) = closure(analyzed, ObservedSide::Output)?;
    let admitted = BTreeSet::from([STATE, ORDINARY_LBTC]);
    if input_families != admitted || output_families != admitted {
        return Err(defect(MaturityAnnouncementClause::ClassClosure));
    }
    let clause = MaturityAnnouncementClause::ProtocolCardinality;
    let (input_cardinality_relation, input_cardinality) =
        cardinality(analyzed, clause, ObservedSide::Input, STATE)?;
    let (output_cardinality_relation, output_cardinality) =
        cardinality(analyzed, clause, ObservedSide::Output, STATE)?;
    let exact = MaturityAnnouncementCardinality {
        minimum: Count::ONE,
        maximum: CardinalityMaximum::Exact(Count::ONE),
    };
    if input_cardinality != exact || output_cardinality != exact {
        return Err(defect(clause));
    }
    Ok(MaturityAnnouncementStateProjection {
        object: input_object,
        asset: STATE_ASSET,
        input_recognition,
        output_recognition,
        input_cardinality_relation,
        output_cardinality_relation,
        input_cardinality,
        output_cardinality,
        input_closure,
        output_closure,
        forbidden: ObjectId::ALL
            .iter()
            .copied()
            .filter(|object| !admitted.contains(object))
            .collect(),
        admitted,
    })
}

fn derive_operator(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<MaturityAnnouncementOperatorProjection, CompileError> {
    let clause = MaturityAnnouncementClause::OperatorAuthorization;
    let (authorization, ()) = sole(analyzed, clause, |relation| match relation {
        Relation::OperatorAuthorization => Some(()),
        _ => None,
    })?;
    let (constructibility, class) = sole(analyzed, clause, |relation| match relation {
        Relation::Constructibility { class } => Some(*class),
        _ => None,
    })?;
    if class != ConstructibilityClass::Operator {
        return Err(defect(clause));
    }
    let expected = ConstructibilityNodeId::Witness {
        operation: PLANNED,
        role: WitnessRole::OperatorAuthorization,
        availability: AvailabilityClass::Operator,
    };
    let graph = &analyzed.source.realization.constructibility;
    let mut witnesses = graph.nodes.iter().filter(|node| node.id == expected);
    let witness = witnesses.next().ok_or_else(|| defect(clause))?.id.clone();
    if witnesses.next().is_some() {
        return Err(defect(clause));
    }
    let mut edges = graph.edges.iter().filter(|edge| {
        edge.source == witness && edge.target == ConstructibilityNodeId::Operation(PLANNED)
    });
    let edge = edges.next().ok_or_else(|| defect(clause))?;
    if edges.next().is_some()
        || edge.edge.role != ConstructibilityEdgeRole::RequiredWitness
        || edge.edge.strength != RequirementStrength::Required
    {
        return Err(defect(clause));
    }
    let evidence = ExternalEvidenceRequirement::OperatorAuthorization { operation: PLANNED };
    for plan in analyzed.proof_plans.values() {
        for relation in [&authorization, &constructibility] {
            let row = plan
                .relation_requirements
                .get(relation)
                .ok_or_else(|| defect(clause))?;
            if row.external_evidence != BTreeSet::from([evidence.clone()])
                || !matches!(&row.proof, crate::requirement::ProofDisposition::ExternalEvidence { requirement, .. }
                    if requirement == &evidence)
            {
                return Err(defect(clause));
            }
        }
    }
    Ok(MaturityAnnouncementOperatorProjection {
        authorization,
        constructibility,
        class,
        witness,
        evidence,
    })
}

fn derive_canonical(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<MaturityAnnouncementCanonicalProjection, CompileError> {
    let clause = MaturityAnnouncementClause::CanonicalDelta;
    let (relation, expected) = sole(analyzed, clause, |relation| match relation {
        Relation::CanonicalDeltaPolicy { expected } => Some(expected.clone()),
        _ => None,
    })?;
    if !expected.is_empty() {
        return Err(defect(clause));
    }
    Ok(MaturityAnnouncementCanonicalProjection { expected, relation })
}

fn derive_sponsor(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<MaturityAnnouncementSponsorProjection, CompileError> {
    for side in [ObservedSide::Input, ObservedSide::Output] {
        let (_, object) = recognition(
            analyzed,
            MaturityAnnouncementClause::SponsorObject,
            side,
            SPONSOR_ASSET,
        )?;
        if object != ORDINARY_LBTC {
            return Err(defect(MaturityAnnouncementClause::SponsorObject));
        }
    }
    let (open_flow_policy, open_flows) = sole(
        analyzed,
        MaturityAnnouncementClause::SponsorFlow,
        |relation| match relation {
            Relation::OpenFlowPolicy { allowed } => Some(allowed.clone()),
            _ => None,
        },
    )?;

    if open_flows != BTreeSet::from([OpenFlowKind::FeeSponsor]) {
        return Err(defect(MaturityAnnouncementClause::SponsorFlow));
    }

    let region = ordinary_lbtc_role_of(&open_flows);

    if region != OrdinaryLbtcRole::SponsorRegion {
        return Err(defect(MaturityAnnouncementClause::SponsorFlow));
    }

    let (isolation, ()) = sole(
        analyzed,
        MaturityAnnouncementClause::SponsorIsolation,
        |relation| match relation {
            Relation::SponsorIsolation => Some(()),
            _ => None,
        },
    )?;
    let (multiplicity, envelope_maximum) = sole(
        analyzed,
        MaturityAnnouncementClause::SponsorEnvelope,
        |relation| match relation {
            Relation::SponsorEnvelopeMultiplicity { maximum } => Some(*maximum),
            _ => None,
        },
    )?;

    if envelope_maximum != Count::ONE {
        return Err(defect(MaturityAnnouncementClause::SponsorEnvelope));
    }

    let (substrate_conservation, substrate_asset) = sole(
        analyzed,
        MaturityAnnouncementClause::SubstrateConservation,
        |relation| match relation {
            Relation::SubstrateConservation { asset } => Some(*asset),
            _ => None,
        },
    )?;

    if substrate_asset != SPONSOR_ASSET {
        return Err(defect(MaturityAnnouncementClause::SubstrateConservation));
    }

    let (_, input_cardinality) = cardinality(
        analyzed,
        MaturityAnnouncementClause::SponsorInputs,
        ObservedSide::Input,
        ORDINARY_LBTC,
    )?;
    let (_, change_cardinality) = cardinality(
        analyzed,
        MaturityAnnouncementClause::SponsorChange,
        ObservedSide::Output,
        ORDINARY_LBTC,
    )?;

    if !input_cardinality.is_optional()
        || input_cardinality.maximum != CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax)
    {
        return Err(defect(MaturityAnnouncementClause::SponsorInputs));
    }

    if !change_cardinality.is_optional()
        || change_cardinality.maximum != CardinalityMaximum::Exact(Count::ONE)
    {
        return Err(defect(MaturityAnnouncementClause::SponsorChange));
    }

    Ok(MaturityAnnouncementSponsorProjection {
        object: ORDINARY_LBTC,
        region,
        open_flows,
        envelope_maximum,
        input_cardinality,
        change_cardinality,
        isolation,
        multiplicity,
        open_flow_policy,
        substrate_conservation,
    })
}

fn derive_roots(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<MaturityAnnouncementRootProjection, CompileError> {
    let (relation, expected) = sole(
        analyzed,
        MaturityAnnouncementClause::RootPolicy,
        |relation| match relation {
            Relation::RootPolicy { expected } => Some(expected.clone()),
            _ => None,
        },
    )?;
    let projection = MaturityAnnouncementRootProjection { expected, relation };

    if projection.expected
        != RootId::ALL
            .iter()
            .map(|root| {
                (
                    *root,
                    if *root == RootId::State {
                        RootUse::Succession
                    } else {
                        RootUse::Forbidden
                    },
                )
            })
            .collect()
    {
        return Err(defect(MaturityAnnouncementClause::RootPolicy));
    }

    Ok(projection)
}

fn derive_certificate(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<MaturityAnnouncementCertificateProjection, CompileError> {
    let (relation, expected) = sole(
        analyzed,
        MaturityAnnouncementClause::CertificateProjection,
        |relation| match relation {
            Relation::ProjectionPolicy { expected } => Some(expected.clone()),
            _ => None,
        },
    )?;

    let contract = ProjectionId::ALL.iter().all(|projection| {
        let rule = if *projection == REQUIRED_PROJECTION {
            ProjectionRule::Required
        } else {
            ProjectionRule::Forbidden
        };

        expected.get(projection) == Some(&rule)
    });

    if !contract {
        return Err(defect(MaturityAnnouncementClause::CertificateProjection));
    }

    Ok(MaturityAnnouncementCertificateProjection { expected, relation })
}

fn derive_representation_policy(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<MaturityAnnouncementRepresentationPolicy, CompileError> {
    let clause = MaturityAnnouncementClause::RepresentationApproval;
    let (relation, approved) = sole(analyzed, clause, |relation| match relation {
        Relation::Representation { object, allowed } if *object == STATE => Some(allowed.clone()),
        _ => None,
    })?;
    let admitted: BTreeSet<_> = MaturityAnnouncementRepresentationPlan::ALL
        .iter()
        .copied()
        .collect();
    if approved != admitted.iter().map(|plan| plan.mode()).collect() {
        return Err(defect(clause));
    }
    Ok(MaturityAnnouncementRepresentationPolicy {
        approved,
        admitted,
        relation,
    })
}

fn derive_lifecycle_closure(
    representations: &BTreeMap<
        MaturityAnnouncementRepresentationPlan,
        MaturityAnnouncementRepresentationProjection,
    >,
) -> Result<MaturityAnnouncementLifecycleClosure, CompileError> {
    let mut closures = representations
        .values()
        .map(|projection| exit_closure(&projection.lifecycle));

    let Some(closure) = closures.next() else {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    };

    if closures.any(|other| other != closure) {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }

    if closure.implemented != BTreeSet::from([IMPLEMENTED_EXIT])
        || closure.outstanding != OUTSTANDING_EXITS.into_iter().collect()
    {
        return Err(defect(MaturityAnnouncementClause::LifecycleExits));
    }

    Ok(closure)
}

fn exit_closure(status: &TargetLifecycleStatus) -> MaturityAnnouncementLifecycleClosure {
    MaturityAnnouncementLifecycleClosure {
        implemented: status.implemented().collect(),
        outstanding: status
            .outstanding()
            .map(|requirement| requirement.exit)
            .collect(),
    }
}

fn factors(
    analyzed: &ScopedAnalyzedProgram,
    representation: MaturityAnnouncementRepresentationPlan,
) -> Vec<(&AnalyzedProofPlan, &AnalyzedOperation)> {
    analyzed
        .proof_plans
        .values()
        .filter(|plan| selects(plan, representation))
        .filter_map(|plan| plan.operations.get(&PLANNED).map(|factor| (plan, factor)))
        .collect()
}

fn selects(
    plan: &AnalyzedProofPlan,
    representation: MaturityAnnouncementRepresentationPlan,
) -> bool {
    plan.proof_plan
        .representations
        .get(&RepresentationChoiceId {
            operation: PLANNED,
            object: STATE,
        })
        .is_some_and(|mode| *mode == representation.mode())
}

fn derive_representation(
    analyzed: &ScopedAnalyzedProgram,
    representation: MaturityAnnouncementRepresentationPlan,
) -> Result<MaturityAnnouncementRepresentationProjection, CompileError> {
    let mut derived = Vec::new();

    for (plan, factor) in factors(analyzed, representation) {
        let candidate = project_factor(representation, plan, factor)?;

        if !derived.contains(&candidate) {
            derived.push(candidate);
        }
    }

    let mut derived = derived.into_iter();
    let projection =
        derived
            .next()
            .ok_or(CompileError::MissingMaturityAnnouncementRepresentation {
                operation: PLANNED,
                representation,
            })?;

    if derived.next().is_some() {
        return Err(CompileError::AmbiguousTargetOperationPlan { operation: PLANNED });
    }

    Ok(projection)
}

fn project_factor(
    representation: MaturityAnnouncementRepresentationPlan,
    plan: &AnalyzedProofPlan,
    factor: &AnalyzedOperation,
) -> Result<MaturityAnnouncementRepresentationProjection, CompileError> {
    let relations = operation_relations(factor);
    let relation_requirements = project_relations(plan, factor, &relations)?;

    Ok(MaturityAnnouncementRepresentationProjection {
        plan: representation,
        cases: project_cases(factor),
        carriers: project_carriers(factor),
        layout: factor.layout_requirements.clone(),
        coverage: project_coverage(plan, factor)?,
        capabilities: capability_census(&relation_requirements)?,
        external_evidence: evidence_census(&relation_requirements)?,
        lifecycle: project_lifecycle(plan, &relations),
        relations: relation_requirements,
    })
}

fn capability_census(
    relations: &BTreeMap<RelationId, TargetRelationRequirement>,
) -> Result<BTreeSet<RequiredCapability>, CompileError> {
    let present = relations
        .values()
        .flat_map(|requirement| requirement.required_capabilities.iter().copied())
        .collect::<BTreeSet<_>>();

    canonical_census(&present, RequiredCapability::ALL, |capability| {
        CompileError::NoncanonicalCapabilityCensus { capability }
    })
    .map(|census| census.into_iter().collect())
}

fn evidence_census(
    relations: &BTreeMap<RelationId, TargetRelationRequirement>,
) -> Result<BTreeSet<ExternalEvidenceRole>, CompileError> {
    let present = relations
        .values()
        .flat_map(|requirement| requirement.external_evidence.iter())
        .map(ExternalEvidenceRole::of)
        .collect::<BTreeSet<_>>();

    canonical_census(&present, ExternalEvidenceRole::ALL, |role| {
        CompileError::NoncanonicalEvidenceRoleCensus { role }
    })
    .map(|census| census.into_iter().collect())
}

fn validate_against_factor(
    projection: &MaturityAnnouncementRepresentationProjection,
    candidate: &AnalyzedProofPlan,
    factor: &AnalyzedOperation,
) -> Result<(), CompileError> {
    let relations = operation_relations(factor);
    let published = projection
        .relations
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    if let Some(relation) = relations
        .difference(&published)
        .chain(published.difference(&relations))
        .next()
    {
        return Err(CompileError::TargetPlanRelationCensusMismatch {
            relation: relation.clone(),
        });
    }

    if projection.cases.keys().cloned().collect::<BTreeSet<_>>() != factor.execution_cases
        || project_cases(factor) != projection.cases
    {
        return Err(CompileError::TargetPlanCaseCensusMismatch);
    }

    if project_carriers(factor) != projection.carriers {
        return Err(CompileError::TargetPlanCarrierCensusMismatch);
    }

    if factor.layout_requirements != projection.layout {
        return Err(CompileError::TargetPlanLayoutCensusMismatch);
    }

    let requirements = project_relations(candidate, factor, &relations)?;

    if requirements != projection.relations {
        return Err(CompileError::TargetPlanRelationRequirementMismatch);
    }

    if project_coverage(candidate, factor)? != projection.coverage {
        return Err(CompileError::TargetPlanCoverageCensusMismatch);
    }

    if capability_census(&requirements)? != projection.capabilities {
        return Err(CompileError::TargetPlanCapabilityCensusMismatch);
    }

    if evidence_census(&requirements)? != projection.external_evidence {
        return Err(CompileError::TargetPlanEvidenceCensusMismatch);
    }

    if project_lifecycle(candidate, &relations) != projection.lifecycle {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }

    Ok(())
}

fn validate_sponsor_erasure(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<(), CompileError> {
    let mut published = plan.representations.values().flat_map(|projection| {
        projection
            .layout
            .iter()
            .chain(projection.relations.values().flat_map(|relation| {
                relation
                    .cases
                    .values()
                    .flat_map(|case| case.layout_requirements.iter())
            }))
            .chain(
                projection
                    .carriers
                    .iter()
                    .flat_map(|carrier| carrier.alternatives.iter())
                    .flat_map(|alternative| alternative.layout.iter()),
            )
    });

    if published.any(names_sponsor_amount) {
        return Err(CompileError::SponsorValueRead);
    }

    Ok(())
}

// The operation projection owns identity; its graphs own the declarations.
fn validate_declaration(analyzed: &ScopedAnalyzedProgram) -> Result<(), CompileError> {
    if !analyzed
        .source
        .compilation_scope
        .operations()
        .contains(&PLANNED)
    {
        return Err(CompileError::TargetOperationOutOfScope { operation: PLANNED });
    }
    let source = &analyzed.source.realization;
    let mut declared = source
        .operations
        .iter()
        .filter(|row| row.operation == PLANNED);
    if declared.next().is_none() || declared.next().is_some() {
        return Err(defect(MaturityAnnouncementClause::Declaration));
    }
    let relations: BTreeSet<_> = declarations(analyzed).map(|row| row.id.clone()).collect();
    let edges: Vec<_> = source
        .relations
        .edges
        .iter()
        .filter(|edge| edge.source.operation() == PLANNED || edge.target.operation() == PLANNED)
        .collect();
    if relations.len() != 26
        || declarations(analyzed).count() != 26
        || edges.len() != 23
        || edges.iter().copied().collect::<BTreeSet<_>>().len() != 23
        || edges
            .iter()
            .any(|edge| !relations.contains(&edge.source) || !relations.contains(&edge.target))
    {
        return Err(defect(MaturityAnnouncementClause::Declaration));
    }
    Ok(())
}

pub(crate) fn validate_maturity_announcement_plan(
    analyzed: &ScopedAnalyzedProgram,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<(), CompileError> {
    validate_declaration(analyzed)?;
    if plan.operation != PLANNED {
        return Err(CompileError::TargetOperationOutOfScope {
            operation: plan.operation,
        });
    }
    if !plan.source.is_analyzed_source(analyzed) {
        return Err(CompileError::TargetPlanSourceMismatch);
    }
    validate_contract_projections(analyzed, plan)?;
    validate_retained_requirements(analyzed, plan)?;
    let admitted: BTreeSet<_> = MaturityAnnouncementRepresentationPlan::ALL
        .iter()
        .copied()
        .collect();
    if plan
        .representations
        .keys()
        .copied()
        .collect::<BTreeSet<_>>()
        != admitted
    {
        return Err(defect(MaturityAnnouncementClause::RepresentationApproval));
    }
    for (representation, projection) in &plan.representations {
        if projection.plan != *representation {
            return Err(defect(MaturityAnnouncementClause::RepresentationApproval));
        }
        let factors = factors(analyzed, *representation);
        if factors.is_empty() {
            return Err(CompileError::MissingMaturityAnnouncementRepresentation {
                operation: PLANNED,
                representation: *representation,
            });
        }
        for (candidate, factor) in factors {
            validate_against_factor(projection, candidate, factor)?;
            validate_declaration_census(analyzed, projection, factor)?;
        }
    }
    // Every retained factor must belong to an admitted group; filtering is not admission.
    for candidate in analyzed.proof_plans.values() {
        if !candidate.operations.contains_key(&PLANNED)
            || !admitted.iter().any(|mode| selects(candidate, *mode))
        {
            return Err(defect(MaturityAnnouncementClause::RepresentationApproval));
        }
    }
    if plan.lifecycle != derive_lifecycle_closure(&plan.representations)? {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }
    validate_representation_equivalence(plan)?;
    validate_sponsor_erasure(plan)
}

fn validate_contract_projections(
    analyzed: &ScopedAnalyzedProgram,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<(), CompileError> {
    use MaturityAnnouncementClause as Clause;
    for (equal, clause) in [
        (
            plan.representation == derive_representation_policy(analyzed)?,
            Clause::RepresentationApproval,
        ),
        (plan.state == derive_state(analyzed)?, Clause::ClassClosure),
        (
            plan.operator == derive_operator(analyzed)?,
            Clause::OperatorAuthorization,
        ),
        (
            plan.canonical == derive_canonical(analyzed)?,
            Clause::CanonicalDelta,
        ),
        (
            plan.sponsor == derive_sponsor(analyzed)?,
            Clause::SponsorIsolation,
        ),
        (plan.roots == derive_roots(analyzed)?, Clause::RootPolicy),
        (
            plan.certificate == derive_certificate(analyzed)?,
            Clause::CertificateProjection,
        ),
    ] {
        if !equal {
            return Err(defect(clause));
        }
    }
    Ok(())
}

fn validate_retained_requirements(
    analyzed: &ScopedAnalyzedProgram,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<(), CompileError> {
    let facts: BTreeSet<_> = plan.public_facts.iter().cloned().collect();
    let declared: BTreeSet<_> = analyzed
        .source
        .realization
        .disclosure
        .nodes
        .iter()
        .filter_map(|node| match node {
            DisclosureNodeId::Fact(id) if announcement_fact(id) => Some(id.clone()),
            _ => None,
        })
        .collect();
    let public: BTreeSet<_> = analyzed
        .source
        .realization
        .declassification
        .required_public
        .iter()
        .filter(|(fact, reasons)| announcement_fact(fact) && !reasons.is_empty())
        .map(|(fact, _)| fact.clone())
        .collect();
    if plan.public_facts != AnnouncementMetadataRequirement::public_facts()
        || declared != facts
        || public != facts
    {
        return Err(defect(MaturityAnnouncementClause::PublicFacts));
    }
    if plan.constructibility != ConstructorContinuityRequirement::REQUIRED
        || plan.transition != AnnouncementMetadataRequirement::required()
        || plan.root_history != RootHistoryRequirement::REQUIRED
        || plan.public_recovery != PublicRecoveryRequirement::REQUIRED
    {
        return Err(defect(MaturityAnnouncementClause::Transition));
    }
    for row in &plan.transition.fields {
        if row.validate().is_err()
            || !facts.contains(&row.input)
            || !facts.contains(&row.output)
            || row.law.operands.iter().any(|operand| match operand {
                StateLawOperand::Fact(fact) => !facts.contains(fact),
                StateLawOperand::PublishedParameter(_) => true,
            })
        {
            return Err(defect(MaturityAnnouncementClause::Transition));
        }
    }
    let succession = &plan.root_history.succession;
    if succession.root_relation != plan.roots.relation
        || succession.certificate_relation != plan.certificate.relation
        || succession.input_fields.as_ref()
            != Some(
                &plan
                    .transition
                    .fields
                    .each_ref()
                    .map(|row| row.input.clone()),
            )
        || succession.output_fields.as_ref()
            != Some(
                &plan
                    .transition
                    .fields
                    .each_ref()
                    .map(|row| row.output.clone()),
            )
    {
        return Err(defect(MaturityAnnouncementClause::Transition));
    }
    Ok(())
}

const fn announcement_fact(fact: &FactId) -> bool {
    matches!(
        fact,
        FactId::StateField {
            operation: PLANNED,
            ..
        } | FactId::RequestedAnnouncementCycle { operation: PLANNED }
            | FactId::AnnouncementLead {
                operation: PLANNED,
                ..
            }
            | FactId::FamilyCount {
                operation: PLANNED,
                ..
            }
            | FactId::FamilyAmount {
                operation: PLANNED,
                ..
            }
            | FactId::InputOwners {
                operation: PLANNED,
                ..
            }
            | FactId::Signers { operation: PLANNED }
            | FactId::ProjectionPresent {
                operation: PLANNED,
                ..
            }
            | FactId::FamilyRecognized {
                operation: PLANNED,
                ..
            }
            | FactId::SponsorIsolated { operation: PLANNED }
            | FactId::ProtocolSecretUsed { operation: PLANNED }
    )
}

// Compare against authored declarations directly, including vacuous prerequisites.
fn validate_declaration_census(
    analyzed: &ScopedAnalyzedProgram,
    projection: &MaturityAnnouncementRepresentationProjection,
    factor: &AnalyzedOperation,
) -> Result<(), CompileError> {
    let declared: BTreeSet<_> = declarations(analyzed).map(|row| row.id.clone()).collect();
    let published = projection.relations.keys().cloned().collect();
    if let Some(relation) = declared.symmetric_difference(&published).next() {
        return Err(CompileError::TargetPlanRelationCensusMismatch {
            relation: relation.clone(),
        });
    }
    let edges: BTreeSet<_> = analyzed
        .source
        .realization
        .relations
        .edges
        .iter()
        .filter(|edge| edge.source.operation() == PLANNED)
        .map(|edge| (edge.source.clone(), edge.target.clone()))
        .collect();
    let mut projected_edges = BTreeSet::new();
    let mut count = 0;
    for edge in &factor.coverage_dependencies.edges {
        if edge.edge != CoverageEdge::RelationPrerequisite {
            continue;
        }
        let (CoverageNodeId::RelationCase(source), CoverageNodeId::RelationCase(target)) =
            (&edge.source, &edge.target)
        else {
            return Err(CompileError::TargetPlanCoverageCensusMismatch);
        };
        if source.case != target.case || !projection.cases.contains_key(&source.case) {
            return Err(CompileError::TargetPlanCoverageCensusMismatch);
        }
        projected_edges.insert((
            source.case.clone(),
            source.relation.clone(),
            target.relation.clone(),
        ));
        count += 1;
    }
    let expected: BTreeSet<_> = projection
        .cases
        .keys()
        .flat_map(|case| {
            edges
                .iter()
                .map(move |(source, target)| (case.clone(), source.clone(), target.clone()))
        })
        .collect();
    if projected_edges != expected || count != expected.len() {
        return Err(CompileError::TargetPlanCoverageCensusMismatch);
    }
    Ok(())
}

// Typed mode-bearing rows may change their selected mode, but no proof family changes.
fn validate_representation_equivalence(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<(), CompileError> {
    let reference = plan
        .projection(MaturityAnnouncementRepresentationPlan::Explicit)
        .ok_or(CompileError::MissingMaturityAnnouncementRepresentation {
            operation: PLANNED,
            representation: MaturityAnnouncementRepresentationPlan::Explicit,
        })?;
    for projection in plan.representations.values() {
        if semantic_cases(projection) != semantic_cases(reference)
            || projection.capabilities != reference.capabilities
            || projection.external_evidence != reference.external_evidence
            || semantic_carriers(projection) != semantic_carriers(reference)
        {
            return Err(CompileError::TargetPlanRelationRequirementMismatch);
        }
        let relations = semantic_relations(projection);
        if relations != semantic_relations(reference) {
            return Err(CompileError::TargetPlanRelationRequirementMismatch);
        }
        if semantic_layout(&projection.layout) != semantic_layout(&reference.layout) {
            return Err(CompileError::TargetPlanLayoutCensusMismatch);
        }
    }
    Ok(())
}

fn semantic_relations(
    projection: &MaturityAnnouncementRepresentationProjection,
) -> BTreeMap<RelationId, TargetRelationRequirement> {
    let mut relations = projection.relations.clone();
    for row in relations.values_mut() {
        if let Some(selection) = &mut row.representation {
            selection.mode = RepresentationMode::Explicit;
        }
        row.lifecycle = row
            .lifecycle
            .iter()
            .cloned()
            .map(|mut requirement| {
                requirement.representation = RepresentationMode::Explicit;
                requirement
            })
            .collect();
        row.cases = row
            .cases
            .values()
            .cloned()
            .map(|mut case| {
                normalize_case_mode(&mut case);
                (case.key.case.clone(), case)
            })
            .collect();
    }
    relations
}

fn normalize_case_mode(case: &mut crate::operation_plan::TargetRelationCase) {
    use crate::placement::{BackendStructuralRequirement, CompilerStaticRequirement};
    normalize_case_id(&mut case.key.case);
    case.compiler_requirements = case
        .compiler_requirements
        .iter()
        .cloned()
        .map(|mut row| {
            if let CompilerStaticRequirement::RepresentationSelection { selected, .. } = &mut row {
                *selected = RepresentationMode::Explicit;
            }
            row
        })
        .collect();
    case.structural_requirements = case
        .structural_requirements
        .iter()
        .cloned()
        .map(|mut row| {
            if let BackendStructuralRequirement::EncodeAndAuthenticateRepresentation {
                representation,
                ..
            } = &mut row
            {
                *representation = RepresentationMode::Explicit;
            }
            row
        })
        .collect();
    case.layout_requirements = semantic_layout(&case.layout_requirements);
}

fn normalize_case_id(case: &mut ExecutionCaseId) {
    if let Some(mode) = case.representations.get_mut(&STATE) {
        *mode = RepresentationMode::Explicit;
    }
}

fn semantic_cases(
    projection: &MaturityAnnouncementRepresentationProjection,
) -> BTreeMap<ExecutionCaseId, TargetExecutionCase> {
    projection
        .cases
        .values()
        .cloned()
        .map(|mut case| {
            normalize_case_id(&mut case.id);
            (case.id.clone(), case)
        })
        .collect()
}

fn semantic_carriers(
    projection: &MaturityAnnouncementRepresentationProjection,
) -> BTreeSet<AbstractCarrierRequirement> {
    projection
        .carriers
        .iter()
        .cloned()
        .map(|mut row| {
            normalize_case_id(&mut row.relation_case.case);
            row.alternatives = row
                .alternatives
                .into_iter()
                .map(|mut alternative| {
                    alternative.layout = semantic_layout(&alternative.layout);
                    alternative
                })
                .collect();
            row
        })
        .collect()
}

fn semantic_layout(layout: &BTreeSet<LayoutRequirement>) -> BTreeSet<LayoutRequirement> {
    layout
        .iter()
        .cloned()
        .map(|mut row| {
            match &mut row {
                LayoutRequirement::EnforceRepresentation { representation, .. } => {
                    *representation = RepresentationMode::Explicit;
                }
                LayoutRequirement::MakeSourceAvailable { case, .. }
                | LayoutRequirement::IsolateSponsorRegion { case, .. }
                | LayoutRequirement::SecretFreeOperationPath { case, .. } => {
                    normalize_case_id(case);
                }
                LayoutRequirement::AuthenticateFamilyCensus { .. }
                | LayoutRequirement::CompleteAndDisjointFamilies { .. }
                | LayoutRequirement::CanonicalCoordinator { .. } => {}
            }
            row
        })
        .collect()
}
