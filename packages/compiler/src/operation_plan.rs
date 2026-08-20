//! The public compact-ASH target-operation plan (Guide-12 §7).
//!
//! The abstract requirement boundary in [`crate::target`] answers one
//! question — what capabilities does *some* target need — and a backend
//! that has to emit an operation needs a great deal more: the exact
//! operation scope, the relation identities, the activation cases, the
//! selected representation policy, the approved proof alternatives, the
//! source and constructibility requirements, the lifecycle obligations,
//! the abstract carriers, the layout requirements, the coverage
//! requirements, and the external evidence the analysis leaves open.
//! This module publishes exactly those facts for one operation, and
//! nothing else.
//!
//! # What this boundary speaks, and why it differs from `target`
//!
//! [`crate::target`] is deliberately realization-free: it serves a
//! target adapter whose package contract admits no realization
//! dependency, so it projects the evidence *role* and drops the
//! relation identity that raised it. This boundary cannot do that and
//! remain useful. Guide-12 §1.3 requires the compiler plan's relation
//! census to be *exactly equal* to the analyzed census and to the
//! backend's selected-proof census, and an equality between censuses
//! whose members have been anonymized is not an equality anyone can
//! check. So the plan speaks `realization::RelationId` — as §7.2's own
//! field list does — and its consumer is a backend that speaks relation
//! identities, not the capability-only adapter.
//!
//! # One authored source per semantic object
//!
//! Guide-12 §1.12 forbids a second authored source for one semantic
//! object, so the vocabulary this plan is written in is *re-exported*
//! rather than re-minted. A public twin of `LayoutRequirement` or
//! `SourceRequirement` would be a second place for the same obligation
//! to be described, free to drift from the one the analysis validator
//! checks. What is minted here is only what did not exist before: the
//! plan's own container rows, which join facts the analysis stores
//! separately.
//!
//! The analysis containers themselves stay crate-private. That is a
//! property of the module graph rather than an assertion: `analyzed`,
//! `analyzed_operation`, `placement`, and `coverage` are unexported
//! modules, so `ScopedAnalyzedProgram`, `AnalyzedOperation`,
//! `RelationCaseRequirements`, `PlacementCandidate`, and the coverage
//! graph projection are unnameable from outside this crate, and with
//! them every Petgraph handle the analysis holds.
//!
//! # What the plan does not carry
//!
//! No digest, and no field reserved for a future one (§1.10). No search
//! count, limit, or state total: those describe how the analysis ran,
//! not what it found. No target opcode, byte, leaf, stack index,
//! transaction position, control path, or target encoding (§1.2). No
//! filesystem path or environment value. And no observed target
//! verdict or accepted target projection: §19's coverage row spans the
//! whole pipeline, and the half a compiler owns is the *requirement*.
//! Reserving a field for the observation would be exactly the
//! speculative identity §1.10 refuses.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use realization::{
    ArchitectureBinding, ExternalEvidenceRequirement, RelationId, RepresentationMode,
    ScopedRealizationProjection,
};

pub use crate::{
    capability::RequiredCapability,
    carrier::{CarrierQuantification, CarrierRole},
    case::{ExecutionCaseId, SponsorCase},
    coverage::{
        CardinalityCeiling, CarrierAssignmentAlternative, CollateralPolicy, CollateralRequirement,
        CoverageBoundary, CoveragePurpose, CoverageRequirementId, EvidenceRole,
        NegativeCoverageRequirement, PositiveCoverageRequirement, ProjectionSubject,
        RelationMutation, SemanticProjectionRequirement,
    },
    input::CompilationScope,
    layout::LayoutRequirement,
    lifecycle::{LifecycleExitStatus, LifecycleRequirement},
    placement::{
        ActivationCondition, BackendStructuralRequirement, CompilerStaticRequirement,
        DischargeBoundary, PlacedCarrier, PlacementSearchLimits, RelationActivity, RelationCaseKey,
        SemanticScope,
    },
    requirement::{ProofDisposition, RepresentationSelection},
    source::{
        OperandId, OperandRole, RequiredSourceKind, RequirementActivation, SourceRequirement,
    },
    target::ExternalEvidenceRole,
};

use crate::{
    CompileError,
    analyzed::{AnalyzedProofPlan, ScopedAnalyzedProgram, analyze_scoped_program},
    analyzed_operation::AnalyzedOperation,
    input::BoundCompilerInput,
    layout::names_sponsor_amount,
    lifecycle::RepresentationChoiceId,
    target::canonical_census,
};

/// The object family whose representation the Phase-4 policy fixes.
const ASH: ObjectId = ObjectId::Ash;

/// The one operation this boundary plans (§7.2).
const PLANNED: OperationId = OperationId::CompactAsh;

/// The exact typed source one operation plan was derived from (§7.2).
///
/// A complete typed source rather than a reference to one, for the same
/// reason the analysis retains one: a revision, a path, or an identity
/// alone would name a source this value cannot check, and every
/// consumer would have to trust that the named thing still says what it
/// said. These three values re-derive the whole analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetOperationSource {
    architecture: ArchitectureBinding,
    realization: ScopedRealizationProjection,
    compilation_scope: CompilationScope,
}

impl TargetOperationSource {
    /// The architecture identity the realization binds.
    #[must_use]
    pub const fn architecture(&self) -> &ArchitectureBinding {
        &self.architecture
    }

    /// The complete scoped realization projection the analysis read.
    #[must_use]
    pub const fn realization(&self) -> &ScopedRealizationProjection {
        &self.realization
    }

    /// The explicit compiler scope the analysis covered.
    #[must_use]
    pub const fn compilation_scope(&self) -> &CompilationScope {
        &self.compilation_scope
    }
}

/// Why the representation alternative set was narrowed (§1.7).
///
/// The distinction is the whole content of the Guide-11 ruling. An
/// alternative set of one was never a choice; an alternative set the
/// deployment narrowed is a choice that a later deployment may revisit,
/// and recording the two the same way would turn a policy decision into
/// an apparent semantic law.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RepresentationNarrowing {
    /// The realization approved exactly one alternative.
    SemanticNecessity,
    /// Several alternatives remained approved and policy selected one.
    DeploymentPolicy,
}

/// The explicit Phase-4 representation selection (§1.7, §7.3).
///
/// `PrivateCommitted` ASH is unsupported in this candidate and
/// `PublicCommitted` ASH is deferred against the three Guide-11
/// blockers. Neither is carried as a dormant branch: the approved set
/// records what the realization allows, the selection records what
/// Phase 4 chose, and the narrowing records that the difference is
/// deployment policy rather than semantic necessity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetRepresentationPolicy {
    selected: BTreeMap<ObjectId, RepresentationMode>,
    approved: BTreeMap<ObjectId, BTreeSet<RepresentationMode>>,
    narrowing: BTreeMap<ObjectId, RepresentationNarrowing>,
}

impl TargetRepresentationPolicy {
    /// The mode Phase 4 selected for each object family, in canonical
    /// order.
    pub fn selected(&self) -> impl Iterator<Item = (ObjectId, RepresentationMode)> + '_ {
        self.selected.iter().map(|(object, mode)| (*object, *mode))
    }

    /// The mode selected for one object family, if this operation
    /// decides it.
    #[must_use]
    pub fn selection(&self, object: ObjectId) -> Option<RepresentationMode> {
        self.selected.get(&object).copied()
    }

    /// Every realization-approved alternative for one object family,
    /// before the Phase-4 filter.
    #[must_use]
    pub fn approved(&self, object: ObjectId) -> Option<&BTreeSet<RepresentationMode>> {
        self.approved.get(&object)
    }

    /// Why one family's alternative set was narrowed to the selection.
    #[must_use]
    pub fn narrowing(&self, object: ObjectId) -> Option<RepresentationNarrowing> {
        self.narrowing.get(&object).copied()
    }
}

/// One execution case of the planned operation (§7.1).
///
/// Both dispositions are retained. An inactive relation is present in
/// the census with its vacuity stated rather than omitted, because a
/// case that simply dropped it would be indistinguishable from a case
/// that never knew about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetExecutionCase {
    /// The case's own typed identity.
    pub id: ExecutionCaseId,
    /// Relations active in this case.
    pub active_relations: BTreeSet<RelationId>,
    /// Relations present in this case with no obligation.
    pub vacuous_relations: BTreeSet<RelationId>,
}

/// Everything one relation owns inside one execution case (§7.1).
///
/// The plan's row, not the analysis's: the analysis row also carries
/// its coverage projection, which this boundary publishes separately
/// under its own requirement identity so that a coverage census can be
/// compared row by row against a later package's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetRelationCase {
    /// The relation-case census key.
    pub key: RelationCaseKey,
    /// Whether the relation is active here.
    pub activity: RelationActivity,
    /// The case-independent condition under which it is active.
    pub activation: ActivationCondition,
    /// Every boundary this relation-case discharges at.
    pub boundaries: BTreeSet<DischargeBoundary>,
    /// The relation's source rows active in this case.
    pub active_sources: BTreeSet<SourceRequirement>,
    /// Properties the compiler validated from typed input directly.
    pub compiler_requirements: BTreeSet<CompilerStaticRequirement>,
    /// Obligations on a future emitted bundle or ABI.
    pub structural_requirements: BTreeSet<BackendStructuralRequirement>,
    /// The layout this relation-case's own alternatives depend on.
    pub layout_requirements: BTreeSet<LayoutRequirement>,
    /// The external evidence this relation-case leaves open.
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
}

/// Everything one relation requires of a backend (§7.1, §7.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetRelationRequirement {
    /// The relation's realization-owned identity.
    pub relation: RelationId,
    /// The approved proof alternative, or the reason there is none.
    pub proof: ProofDisposition,
    /// The relation's own object-specific representation decision.
    pub representation: Option<RepresentationSelection>,
    /// Every abstract capability this relation requires.
    pub required_capabilities: BTreeSet<RequiredCapability>,
    /// The relation's complete source-requirement set across cases.
    pub source_requirements: BTreeSet<SourceRequirement>,
    /// The external evidence this relation leaves open.
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
    /// The lifecycle obligations this relation's representation implies.
    pub lifecycle: BTreeSet<LifecycleRequirement>,
    /// One row per execution case, including the vacuous ones.
    pub cases: BTreeMap<ExecutionCaseId, TargetRelationCase>,
}

/// The abstract carrier obligation of one relation-case (§7.1).
///
/// Abstract: a carrier role and a quantification, never a tapleaf, a
/// script position, or a witness index. Selecting among the
/// alternatives is a backend decision under its own typed policy, and
/// the compiler states every one it accepts rather than pre-selecting.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbstractCarrierRequirement {
    /// The relation-case that must be carried.
    pub relation_case: RelationCaseKey,
    /// Every inclusion-minimal assignment the analysis accepts.
    pub alternatives: BTreeSet<CarrierAssignmentAlternative>,
}

/// Which half of the coverage contract one requirement states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetCoverageObligation {
    /// The boundary must accept the valid subject.
    Positive(PositiveCoverageRequirement),
    /// The boundary must reject one focused mutation.
    Negative(NegativeCoverageRequirement),
}

/// One relation-indexed coverage requirement (§19).
///
/// The requirement half of §19's row. The observed target verdict and
/// the accepted target projection are absent by ruling: they are
/// observations an evidence package makes, and a compiler field
/// reserved for them would assert a future artifact exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetCoverageRequirement {
    /// Relation, case, boundary, and purpose.
    pub id: CoverageRequirementId,
    /// Whether the relation is active in this case.
    pub activity: RelationActivity,
    /// The condition under which the relation is active at all.
    pub activation: ActivationCondition,
    /// The approved proof alternative behind this requirement.
    pub proof: ProofDisposition,
    /// Which artifact answers this requirement.
    pub role: EvidenceRole,
    /// The positive or negative obligation itself.
    pub obligation: TargetCoverageObligation,
    /// Every carrier assignment the relation-case admits.
    pub carrier: BTreeSet<CarrierAssignmentAlternative>,
    /// The accepted-projection comparison of this requirement's
    /// boundary, where one applies.
    pub projection: Option<SemanticProjectionRequirement>,
    /// The external evidence this requirement depends on.
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
}

/// The planned operation's lifecycle status (§5.7).
///
/// `release_complete` is derived from the obligation set, never
/// asserted: a status that could be set independently of the
/// obligations it summarizes is a place for two answers to disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetLifecycleStatus {
    implemented: BTreeSet<OperationId>,
    outstanding: BTreeSet<LifecycleRequirement>,
}

impl TargetLifecycleStatus {
    /// Every required exit this compiler scope implements.
    pub fn implemented(&self) -> impl Iterator<Item = OperationId> + '_ {
        self.implemented.iter().copied()
    }

    /// Every required exit declared outside the compiler scope.
    pub fn outstanding(&self) -> impl Iterator<Item = &LifecycleRequirement> {
        self.outstanding.iter()
    }

    /// Whether every required exit is implemented in scope.
    ///
    /// False for the Phase-4 candidate, whose `clear` exit is
    /// outstanding.
    #[must_use]
    pub fn release_complete(&self) -> bool {
        self.outstanding.is_empty()
    }
}

/// The validated public plan for one target operation (§7.2).
///
/// Every field is private and there is no public constructor, no
/// `Default`, and no builder: the sole route to a value of this type is
/// [`plan_compact_ash_target_operation`], which runs the complete
/// analysis, its own independent validator, the Phase-4 policy filter,
/// and then the complete re-derivation validation of the assembled
/// plan. A plan assembled from arbitrary fields would be a request
/// rather than an analysis, and nothing downstream could tell the two
/// apart once they shared a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedTargetOperationPlan {
    operation: OperationId,
    source: TargetOperationSource,
    representation: TargetRepresentationPolicy,
    cases: BTreeMap<ExecutionCaseId, TargetExecutionCase>,
    relations: BTreeMap<RelationId, TargetRelationRequirement>,
    carriers: BTreeSet<AbstractCarrierRequirement>,
    layout: BTreeSet<LayoutRequirement>,
    coverage: BTreeMap<CoverageRequirementId, TargetCoverageRequirement>,
    capabilities: BTreeSet<RequiredCapability>,
    external_evidence: BTreeSet<ExternalEvidenceRole>,
    lifecycle: TargetLifecycleStatus,
}

impl ValidatedTargetOperationPlan {
    /// The operation this plan is about — always `compact-ash`.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }

    /// The complete typed source the plan was derived from.
    #[must_use]
    pub const fn source(&self) -> &TargetOperationSource {
        &self.source
    }

    /// The explicit Phase-4 representation policy.
    #[must_use]
    pub const fn representation(&self) -> &TargetRepresentationPolicy {
        &self.representation
    }

    /// Every execution case, in canonical order.
    pub fn cases(&self) -> impl Iterator<Item = &TargetExecutionCase> {
        self.cases.values()
    }

    /// One execution case by identity.
    #[must_use]
    pub fn case(&self, id: &ExecutionCaseId) -> Option<&TargetExecutionCase> {
        self.cases.get(id)
    }

    /// The exact relation census, in canonical order.
    pub fn relations(&self) -> impl Iterator<Item = &TargetRelationRequirement> {
        self.relations.values()
    }

    /// One relation's requirements by identity.
    #[must_use]
    pub fn relation(&self, relation: &RelationId) -> Option<&TargetRelationRequirement> {
        self.relations.get(relation)
    }

    /// The exact abstract carrier census, in canonical order.
    pub fn carriers(&self) -> impl Iterator<Item = &AbstractCarrierRequirement> {
        self.carriers.iter()
    }

    /// The exact layout census, in canonical order.
    pub fn layout(&self) -> impl Iterator<Item = &LayoutRequirement> {
        self.layout.iter()
    }

    /// The exact coverage census, in canonical order.
    pub fn coverage(&self) -> impl Iterator<Item = &TargetCoverageRequirement> {
        self.coverage.values()
    }

    /// One coverage requirement by identity.
    #[must_use]
    pub fn coverage_requirement(
        &self,
        id: &CoverageRequirementId,
    ) -> Option<&TargetCoverageRequirement> {
        self.coverage.get(id)
    }

    /// Every abstract capability this operation requires.
    pub fn capabilities(&self) -> impl Iterator<Item = RequiredCapability> + '_ {
        self.capabilities.iter().copied()
    }

    /// Every external-evidence role this operation leaves open.
    pub fn external_evidence(&self) -> impl Iterator<Item = ExternalEvidenceRole> + '_ {
        self.external_evidence.iter().copied()
    }

    /// The operation's lifecycle status.
    #[must_use]
    pub const fn lifecycle(&self) -> &TargetLifecycleStatus {
        &self.lifecycle
    }
}

/// Plan the compact-ASH target operation from one bound input.
///
/// The complete analysis runs first, including its own corruption-
/// resistant assembly validator; the Phase-4 policy of §7.3 then
/// filters the retained feasible plans to those selecting `Explicit`
/// ASH; the surviving alternatives are projected onto the operation and
/// required to agree exactly; the plan is assembled; and the assembled
/// value is validated by an independent re-derivation before it is
/// returned. A caller receives a complete validated plan or a typed
/// failure. There is no partial result and no route to the analyzed
/// program itself.
///
/// # Errors
///
/// Any failure of the scoped analysis or its validator;
/// [`CompileError::TargetOperationOutOfScope`] when the bound scope
/// does not analyze `compact-ash`;
/// [`CompileError::NoAdmissiblePhase4Plan`] when the policy filter
/// retains nothing; [`CompileError::AmbiguousTargetOperationPlan`] when
/// the surviving alternatives disagree about the operation;
/// [`CompileError::DuplicateTargetCoverageRequirement`] when one
/// coverage identity is claimed twice; and any defect the plan
/// validator raises.
pub fn plan_compact_ash_target_operation(
    input: &BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<ValidatedTargetOperationPlan, CompileError> {
    let analyzed = analyze_scoped_program(input, placement_limits)?;
    let plan = derive_compact_ash_plan(&analyzed)?;

    validate_target_operation_plan(&analyzed, &plan)?;

    Ok(plan)
}

/// Derive the plan from a complete analyzed program.
///
/// Crate-private and reachable only below
/// [`plan_compact_ash_target_operation`], so the validated-analysis
/// precondition is a property of the call graph rather than a comment.
///
/// # Errors
///
/// [`CompileError::TargetOperationOutOfScope`],
/// [`CompileError::NoAdmissiblePhase4Plan`],
/// [`CompileError::AmbiguousTargetOperationPlan`],
/// [`CompileError::DuplicateTargetCoverageRequirement`], or a census
/// defect raised while canonicalizing the capability and evidence-role
/// censuses.
pub(crate) fn derive_compact_ash_plan(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<ValidatedTargetOperationPlan, CompileError> {
    if !analyzed
        .source
        .compilation_scope
        .operations()
        .contains(&PLANNED)
    {
        return Err(CompileError::TargetOperationOutOfScope { operation: PLANNED });
    }

    // §7.3: every feasible plan is retained by the analysis; the
    // Phase-4 policy filters, and no feasible plan is removed because
    // it is inconvenient.
    let admissible = analyzed
        .proof_plans
        .values()
        .filter(|plan| selects_explicit_ash(plan))
        .collect::<Vec<_>>();

    if admissible.is_empty() {
        return Err(CompileError::NoAdmissiblePhase4Plan { operation: PLANNED });
    }

    // The filter is stated over the whole scope's plans, and the plans
    // of a multi-operation scope also differ in choices this operation
    // does not make. Projecting each survivor onto the operation and
    // requiring exact agreement is stronger than a tie-break: it proves
    // the plan is a property of the operation rather than of which
    // alternative happened to be chosen.
    let mut derived = Vec::new();

    for plan in admissible {
        let candidate = project_operation(analyzed, plan)?;

        if !derived.contains(&candidate) {
            derived.push(candidate);
        }
    }

    let mut derived = derived.into_iter();
    let plan = derived
        .next()
        .ok_or(CompileError::NoAdmissiblePhase4Plan { operation: PLANNED })?;

    if derived.next().is_some() {
        return Err(CompileError::AmbiguousTargetOperationPlan { operation: PLANNED });
    }

    Ok(plan)
}

/// Whether one analyzed plan selects the Phase-4 ASH representation.
///
/// The choice variable is read by its typed identity — the planned
/// operation and the ASH object family — never by scanning for a mode
/// some other family happened to select.
fn selects_explicit_ash(plan: &AnalyzedProofPlan) -> bool {
    plan.proof_plan
        .representations
        .get(&RepresentationChoiceId {
            operation: PLANNED,
            object: ASH,
        })
        .is_some_and(|mode| *mode == RepresentationMode::Explicit)
}

/// Project one admissible analyzed plan onto the planned operation.
///
/// # Errors
///
/// [`CompileError::TargetOperationOutOfScope`] when the plan analyzed
/// no factor for the operation;
/// [`CompileError::DuplicateTargetCoverageRequirement`] on a repeated
/// coverage identity; a census defect from
/// [`crate::target::canonical_census`].
fn project_operation(
    analyzed: &ScopedAnalyzedProgram,
    plan: &AnalyzedProofPlan,
) -> Result<ValidatedTargetOperationPlan, CompileError> {
    let operation = plan
        .operations
        .get(&PLANNED)
        .ok_or(CompileError::TargetOperationOutOfScope { operation: PLANNED })?;

    let relations = operation_relations(operation);
    let cases = project_cases(operation);
    let relation_requirements = project_relations(plan, operation, &relations)?;
    let carriers = project_carriers(operation);
    let coverage = project_coverage(plan, operation)?;

    let capabilities = relation_requirements
        .values()
        .flat_map(|requirement| requirement.required_capabilities.iter().copied())
        .collect::<BTreeSet<_>>();
    let capabilities = canonical_census(&capabilities, RequiredCapability::ALL, |capability| {
        CompileError::NoncanonicalCapabilityCensus { capability }
    })?;

    let evidence = relation_requirements
        .values()
        .flat_map(|requirement| requirement.external_evidence.iter())
        .map(ExternalEvidenceRole::of)
        .collect::<BTreeSet<_>>();
    let evidence = canonical_census(&evidence, ExternalEvidenceRole::ALL, |role| {
        CompileError::NoncanonicalEvidenceRoleCensus { role }
    })?;

    Ok(ValidatedTargetOperationPlan {
        operation: PLANNED,
        source: TargetOperationSource {
            architecture: analyzed.source.architecture.clone(),
            realization: analyzed.source.realization.clone(),
            compilation_scope: analyzed.source.compilation_scope.clone(),
        },
        representation: project_representation(analyzed, plan),
        cases,
        relations: relation_requirements,
        carriers,
        layout: operation.layout_requirements.clone(),
        coverage,
        capabilities: capabilities.into_iter().collect(),
        external_evidence: evidence.into_iter().collect(),
        lifecycle: project_lifecycle(plan, &relations),
    })
}

/// The exact relation census of one operation factor.
///
/// Derived from the relation-case census the analysis stores, so a
/// relation that lost every case is absent here for the same reason it
/// is absent there — and the plan validator proves both against the
/// analyzed program independently.
fn operation_relations(operation: &AnalyzedOperation) -> BTreeSet<RelationId> {
    operation
        .relation_cases
        .keys()
        .map(|key| key.relation.clone())
        .collect()
}

/// Project the operation's execution cases with their relation split.
fn project_cases(operation: &AnalyzedOperation) -> BTreeMap<ExecutionCaseId, TargetExecutionCase> {
    let mut cases = operation
        .execution_cases
        .iter()
        .map(|id| {
            (
                id.clone(),
                TargetExecutionCase {
                    id: id.clone(),
                    active_relations: BTreeSet::new(),
                    vacuous_relations: BTreeSet::new(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    for (key, requirements) in &operation.relation_cases {
        let Some(case) = cases.get_mut(&key.case) else {
            continue;
        };

        match requirements.activity {
            RelationActivity::Active => case.active_relations.insert(key.relation.clone()),
            RelationActivity::Vacuous => case.vacuous_relations.insert(key.relation.clone()),
        };
    }

    cases
}

/// Project one relation requirement per relation in the census.
///
/// # Errors
///
/// [`CompileError::TargetOperationOutOfScope`] when a relation of the
/// operation carries no requirement bundle under the plan.
fn project_relations(
    plan: &AnalyzedProofPlan,
    operation: &AnalyzedOperation,
    relations: &BTreeSet<RelationId>,
) -> Result<BTreeMap<RelationId, TargetRelationRequirement>, CompileError> {
    let mut projected = BTreeMap::new();

    for relation in relations {
        let bundle = plan.relation_requirements.get(relation).ok_or_else(|| {
            CompileError::TargetPlanRelationCensusMismatch {
                relation: relation.clone(),
            }
        })?;

        let cases = operation
            .relation_cases
            .iter()
            .filter(|(key, _)| &key.relation == relation)
            .map(|(key, requirements)| {
                (
                    key.case.clone(),
                    TargetRelationCase {
                        key: key.clone(),
                        activity: requirements.activity,
                        activation: requirements.activation,
                        boundaries: requirements.boundaries.clone(),
                        active_sources: requirements.active_sources.clone(),
                        compiler_requirements: requirements.compiler_requirements.clone(),
                        structural_requirements: requirements.structural_requirements.clone(),
                        layout_requirements: requirements.layout_requirements.clone(),
                        external_evidence: requirements.external_evidence.clone(),
                    },
                )
            })
            .collect();

        projected.insert(
            relation.clone(),
            TargetRelationRequirement {
                relation: relation.clone(),
                proof: bundle.proof.clone(),
                representation: bundle.representation,
                required_capabilities: bundle.required_capabilities.clone(),
                source_requirements: bundle.source_requirements.clone(),
                external_evidence: bundle.external_evidence.clone(),
                lifecycle: bundle.lifecycle.clone(),
                cases,
            },
        );
    }

    Ok(projected)
}

/// Project the abstract carrier census of one operation factor.
///
/// A relation-case with no accepted assignment contributes no row: an
/// empty alternative set would read as a carrier obligation nothing can
/// discharge, when the analysis means there is no runtime obligation at
/// all.
fn project_carriers(operation: &AnalyzedOperation) -> BTreeSet<AbstractCarrierRequirement> {
    operation
        .relation_cases
        .iter()
        .filter(|(_, requirements)| !requirements.carrier_assignments.is_empty())
        .map(|(key, requirements)| AbstractCarrierRequirement {
            relation_case: key.clone(),
            alternatives: requirements.carrier_assignments.clone(),
        })
        .collect()
}

/// Project the relation-indexed coverage census of one operation.
///
/// # Errors
///
/// [`CompileError::DuplicateTargetCoverageRequirement`] when two rows
/// claim one coverage identity — two answers to one question.
fn project_coverage(
    plan: &AnalyzedProofPlan,
    operation: &AnalyzedOperation,
) -> Result<BTreeMap<CoverageRequirementId, TargetCoverageRequirement>, CompileError> {
    let mut coverage = BTreeMap::new();

    for (key, projection) in &operation.coverage.requirements {
        let proof = plan
            .relation_requirements
            .get(&key.relation)
            .map(|bundle| bundle.proof.clone())
            .ok_or_else(|| CompileError::TargetPlanRelationCensusMismatch {
                relation: key.relation.clone(),
            })?;
        let activation = operation
            .relation_cases
            .get(key)
            .map(|requirements| requirements.activation)
            .ok_or_else(|| CompileError::TargetPlanRelationCensusMismatch {
                relation: key.relation.clone(),
            })?;

        let rows = projection
            .positive
            .iter()
            .map(|requirement| {
                (
                    requirement.id.clone(),
                    requirement.role.clone(),
                    TargetCoverageObligation::Positive(requirement.clone()),
                )
            })
            .chain(projection.negative.iter().map(|requirement| {
                (
                    requirement.id.clone(),
                    requirement.role.clone(),
                    TargetCoverageObligation::Negative(requirement.clone()),
                )
            }));

        for (id, role, obligation) in rows {
            let projected = projection.projections.get(&id.boundary).cloned();
            let row = TargetCoverageRequirement {
                activity: projection.activity,
                activation,
                proof: proof.clone(),
                role,
                obligation,
                carrier: projection.carrier.clone(),
                projection: projected,
                external_evidence: projection.external_evidence.clone(),
                id: id.clone(),
            };

            if coverage.insert(id.clone(), row).is_some() {
                return Err(CompileError::DuplicateTargetCoverageRequirement { requirement: id });
            }
        }
    }

    Ok(coverage)
}

/// Project the explicit Phase-4 representation policy (§1.7, §7.3).
///
/// The approved set is read from the realization's own lifecycle
/// choices rather than from the selection: a policy that reported its
/// own choice as the only approved alternative would record every
/// narrowing as semantic necessity.
fn project_representation(
    analyzed: &ScopedAnalyzedProgram,
    plan: &AnalyzedProofPlan,
) -> TargetRepresentationPolicy {
    let mut selected = BTreeMap::new();
    let mut approved: BTreeMap<ObjectId, BTreeSet<RepresentationMode>> = BTreeMap::new();
    let mut narrowing = BTreeMap::new();

    for (choice, mode) in &plan.proof_plan.representations {
        if choice.operation != PLANNED {
            continue;
        }

        selected.insert(choice.object, *mode);
        approved
            .entry(choice.object)
            .or_default()
            .extend(approved_modes(analyzed, choice.object));
    }

    for (object, modes) in &approved {
        narrowing.insert(
            *object,
            if modes.len() > 1 {
                RepresentationNarrowing::DeploymentPolicy
            } else {
                RepresentationNarrowing::SemanticNecessity
            },
        );
    }

    TargetRepresentationPolicy {
        selected,
        approved,
        narrowing,
    }
}

/// The realization-approved representation alternatives of one object
/// family in the planned operation.
///
/// Read from the retained realization projection, which is the authored
/// source of the alternative set. Reading it from the compiler's own
/// restatement of the same fact would leave the policy unable to
/// disagree with the analysis, and disagreement is exactly what the
/// oracle checks for.
fn approved_modes(
    analyzed: &ScopedAnalyzedProgram,
    object: ObjectId,
) -> BTreeSet<RepresentationMode> {
    analyzed
        .source
        .realization
        .relations
        .nodes
        .iter()
        .filter(|declaration| declaration.id.operation() == PLANNED)
        .filter_map(|declaration| match &declaration.relation {
            realization::Relation::Representation {
                object: declared,
                allowed,
            } if *declared == object => Some(allowed.iter().copied()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// Project the operation's lifecycle status (§5.7).
fn project_lifecycle(
    plan: &AnalyzedProofPlan,
    relations: &BTreeSet<RelationId>,
) -> TargetLifecycleStatus {
    let owned = relations
        .iter()
        .filter_map(|relation| plan.relation_requirements.get(relation))
        .flat_map(|bundle| bundle.lifecycle.iter().cloned())
        .collect::<BTreeSet<_>>();

    TargetLifecycleStatus {
        implemented: owned
            .iter()
            .filter(|requirement| {
                requirement.status == LifecycleExitStatus::AvailableInCompilerScope
            })
            .map(|requirement| requirement.exit)
            .collect(),
        outstanding: owned
            .into_iter()
            .filter(|requirement| {
                requirement.status == LifecycleExitStatus::DeclaredOutsideCompilerScope
            })
            .collect(),
    }
}

/// Validate one assembled plan against the analyzed program (§7.2).
///
/// The independent assembly validator. The plan is not trusted because
/// its assembler produced it: every census it claims is re-derived here
/// from the analyzed program directly and compared for exact equality
/// in both directions, so a plan that lost a relation, gained a
/// carrier, changed a layout requirement, or rewrote one coverage row
/// after assembly is rejected rather than believed.
///
/// Re-derivation reads the analyzed program rather than the plan's own
/// construction, which is what makes this a check and not a restatement.
///
/// # Errors
///
/// [`CompileError::TargetPlanRelationCensusMismatch`],
/// [`CompileError::TargetPlanCaseCensusMismatch`],
/// [`CompileError::TargetPlanCarrierCensusMismatch`],
/// [`CompileError::TargetPlanLayoutCensusMismatch`],
/// [`CompileError::TargetPlanCoverageCensusMismatch`],
/// [`CompileError::TargetPlanRepresentationMismatch`],
/// [`CompileError::TargetPlanLifecycleMismatch`], or
/// [`CompileError::SponsorValueRead`] when a published layout
/// requirement names an erased sponsor amount.
pub(crate) fn validate_target_operation_plan(
    analyzed: &ScopedAnalyzedProgram,
    plan: &ValidatedTargetOperationPlan,
) -> Result<(), CompileError> {
    if plan.operation != PLANNED {
        return Err(CompileError::TargetOperationOutOfScope {
            operation: plan.operation,
        });
    }

    // The source is the analysis's own, component by component.
    if plan.source.architecture != analyzed.source.architecture
        || plan.source.realization != analyzed.source.realization
        || plan.source.compilation_scope != analyzed.source.compilation_scope
    {
        return Err(CompileError::TargetPlanSourceMismatch);
    }

    // §1.7: the published policy really is the explicit selection.
    if plan.representation.selection(ASH) != Some(RepresentationMode::Explicit) {
        return Err(CompileError::TargetPlanRepresentationMismatch { object: ASH });
    }

    // The analyzed factors this plan may have come from. Every one of
    // them must agree with the published censuses, which is stronger
    // than checking the one the assembler happened to read.
    let factors = analyzed
        .proof_plans
        .values()
        .filter(|candidate| selects_explicit_ash(candidate))
        .filter_map(|candidate| {
            candidate
                .operations
                .get(&PLANNED)
                .map(|operation| (candidate, operation))
        })
        .collect::<Vec<_>>();

    if factors.is_empty() {
        return Err(CompileError::NoAdmissiblePhase4Plan { operation: PLANNED });
    }

    for (candidate, operation) in factors {
        validate_against_factor(plan, candidate, operation)?;
    }

    // §1.6: no published layout requirement may name an erased sponsor
    // amount, wherever it is published from.
    for requirement in plan
        .layout
        .iter()
        .chain(plan.relations.values().flat_map(|relation| {
            relation
                .cases
                .values()
                .flat_map(|case| case.layout_requirements.iter())
        }))
        .chain(
            plan.carriers
                .iter()
                .flat_map(|carrier| carrier.alternatives.iter())
                .flat_map(|alternative| alternative.layout.iter()),
        )
    {
        if names_sponsor_amount(requirement) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    Ok(())
}

/// Compare one published plan against one analyzed operation factor.
///
/// Both directions of every equality are defects: a missing member is a
/// completeness defect and an unexpected one is an unowned requirement.
///
/// # Errors
///
/// The census-mismatch variants listed by
/// [`validate_target_operation_plan`].
fn validate_against_factor(
    plan: &ValidatedTargetOperationPlan,
    candidate: &AnalyzedProofPlan,
    operation: &AnalyzedOperation,
) -> Result<(), CompileError> {
    let relations = operation_relations(operation);
    let published = plan.relations.keys().cloned().collect::<BTreeSet<_>>();

    if let Some(relation) = relations
        .difference(&published)
        .chain(published.difference(&relations))
        .next()
    {
        return Err(CompileError::TargetPlanRelationCensusMismatch {
            relation: relation.clone(),
        });
    }

    let cases = plan.cases.keys().cloned().collect::<BTreeSet<_>>();

    if cases != operation.execution_cases {
        return Err(CompileError::TargetPlanCaseCensusMismatch);
    }

    if project_cases(operation) != plan.cases {
        return Err(CompileError::TargetPlanCaseCensusMismatch);
    }

    if project_carriers(operation) != plan.carriers {
        return Err(CompileError::TargetPlanCarrierCensusMismatch);
    }

    if operation.layout_requirements != plan.layout {
        return Err(CompileError::TargetPlanLayoutCensusMismatch);
    }

    if project_relations(candidate, operation, &relations)? != plan.relations {
        return Err(CompileError::TargetPlanRelationRequirementMismatch);
    }

    if project_coverage(candidate, operation)? != plan.coverage {
        return Err(CompileError::TargetPlanCoverageCensusMismatch);
    }

    if project_lifecycle(candidate, &relations) != plan.lifecycle {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }

    Ok(())
}

/// Test-only corruption handles.
///
/// The published fields are private and there is no public route to
/// them, which is the property §7.4 requires. The corruption oracles
/// nevertheless have to damage a validated plan the way a defect in the
/// join could, so the handles exist for tests alone and are compiled
/// out of every other build.
#[cfg(test)]
impl ValidatedTargetOperationPlan {
    pub(crate) const fn source_mut(&mut self) -> &mut TargetOperationSource {
        &mut self.source
    }

    pub(crate) const fn representation_mut(&mut self) -> &mut TargetRepresentationPolicy {
        &mut self.representation
    }

    pub(crate) const fn cases_mut(
        &mut self,
    ) -> &mut BTreeMap<ExecutionCaseId, TargetExecutionCase> {
        &mut self.cases
    }

    pub(crate) const fn relations_mut(
        &mut self,
    ) -> &mut BTreeMap<RelationId, TargetRelationRequirement> {
        &mut self.relations
    }

    pub(crate) const fn carriers_mut(&mut self) -> &mut BTreeSet<AbstractCarrierRequirement> {
        &mut self.carriers
    }

    pub(crate) const fn layout_mut(&mut self) -> &mut BTreeSet<LayoutRequirement> {
        &mut self.layout
    }

    pub(crate) const fn coverage_mut(
        &mut self,
    ) -> &mut BTreeMap<CoverageRequirementId, TargetCoverageRequirement> {
        &mut self.coverage
    }

    pub(crate) const fn lifecycle_mut(&mut self) -> &mut TargetLifecycleStatus {
        &mut self.lifecycle
    }
}

#[cfg(test)]
impl TargetRepresentationPolicy {
    pub(crate) const fn selected_mut(&mut self) -> &mut BTreeMap<ObjectId, RepresentationMode> {
        &mut self.selected
    }
}

#[cfg(test)]
impl TargetLifecycleStatus {
    pub(crate) fn clear_outstanding(&mut self) {
        self.outstanding.clear();
    }
}
