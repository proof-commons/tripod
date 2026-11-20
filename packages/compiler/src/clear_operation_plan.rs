//! The validated public explicit Clear target-operation plan.
//!
//! This facade reads the complete attestation analysis. Public planning runs
//! its private assembly and validator. All published censuses are requirements,
//! not observations of an emitted transaction or an authenticated event report.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    ExprId, ExpressionRole, FactId, Relation, RelationId, RelationKind, RelationSubject,
    RepresentationMode, StateField, TransactionSide,
};

pub use crate::{
    capability::RequiredCapability,
    case::ExecutionCaseId,
    coverage::CoverageRequirementId,
    layout::LayoutRequirement,
    lifecycle::RepresentationChoiceId,
    operation_plan::{
        AbstractCarrierRequirement, TargetCoverageRequirement, TargetExecutionCase,
        TargetLifecycleStatus, TargetOperationSource, TargetRelationRequirement,
    },
    placement::PlacementSearchLimits,
    target::ExternalEvidenceRole,
};

use crate::{
    CompileError,
    analyzed::{AnalyzedProofPlan, ScopedAnalyzedProgram, analyze_scoped_program},
    analyzed_operation::AnalyzedOperation,
    operation_plan::{
        operation_relations, project_carriers, project_cases, project_coverage, project_lifecycle,
        project_relations,
    },
    target::canonical_census,
};

const PLANNED: OperationId = OperationId::Clear;

/// The assignment obligation for one stored STATE field.
///
/// The YL decrement is owned by the closed clamp/assignment law. The five
/// copies carry their existing equality expression identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearFieldRequirement {
    field: StateField,
    relation: RelationId,
    input: FactId,
    output: FactId,
    predicate: Option<ExprId>,
}

impl ClearFieldRequirement {
    /// The stored field governed by this row.
    #[must_use]
    pub const fn field(&self) -> StateField {
        self.field
    }

    /// The assignment relation owning this field.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    /// The predecessor STATE fact.
    #[must_use]
    pub const fn input(&self) -> &FactId {
        &self.input
    }

    /// The successor STATE fact.
    #[must_use]
    pub const fn output(&self) -> &FactId {
        &self.output
    }

    /// The equality predicate for a copy; absent for the YL decrement.
    #[must_use]
    pub const fn predicate(&self) -> Option<&ExprId> {
        self.predicate.as_ref()
    }
}

#[cfg(test)]
impl ClearFieldRequirement {
    pub(crate) const fn set_predicate(&mut self, predicate: Option<ExprId>) {
        self.predicate = predicate;
    }
}

/// The Clear selections used to project one explicit plan.
///
/// These are operation-owned choices. A `TransferLive` selection in the same
/// scope cannot supply one that Clear omitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearRepresentationPolicy {
    selected: BTreeMap<RepresentationChoiceId, RepresentationMode>,
}

impl ClearRepresentationPolicy {
    /// The operation-owned selections in canonical order.
    pub fn selected(&self) -> impl Iterator<Item = (&RepresentationChoiceId, RepresentationMode)> {
        self.selected.iter().map(|(choice, mode)| (choice, *mode))
    }

    /// The mode selected for one Clear choice.
    #[must_use]
    pub fn selection(&self, choice: &RepresentationChoiceId) -> Option<RepresentationMode> {
        self.selected.get(choice).copied()
    }
}

/// Clear's complete explicit requirement projection.
///
/// Fields are private. There is no public constructor or unchecked builder.
/// The public constructor validates both the analysis and this facade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedClearOperationPlan {
    operation: OperationId,
    source: TargetOperationSource,
    representation: ClearRepresentationPolicy,
    fields: BTreeMap<StateField, ClearFieldRequirement>,
    cases: BTreeMap<ExecutionCaseId, TargetExecutionCase>,
    relations: BTreeMap<RelationId, TargetRelationRequirement>,
    carriers: BTreeSet<AbstractCarrierRequirement>,
    layout: BTreeSet<LayoutRequirement>,
    coverage: BTreeMap<CoverageRequirementId, TargetCoverageRequirement>,
    capabilities: BTreeSet<RequiredCapability>,
    external_evidence: BTreeSet<ExternalEvidenceRole>,
    lifecycle: TargetLifecycleStatus,
}

impl ValidatedClearOperationPlan {
    /// The planned operation.
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }

    /// The complete typed source used for analysis.
    #[must_use]
    pub const fn source(&self) -> &TargetOperationSource {
        &self.source
    }

    /// Clear's explicit representation selections.
    #[must_use]
    pub const fn representation(&self) -> &ClearRepresentationPolicy {
        &self.representation
    }

    /// The six stored STATE assignment requirements.
    pub fn fields(&self) -> impl Iterator<Item = &ClearFieldRequirement> {
        self.fields.values()
    }

    /// One stored field's assignment requirement.
    #[must_use]
    pub fn field(&self, field: StateField) -> Option<&ClearFieldRequirement> {
        self.fields.get(&field)
    }

    /// All execution cases in canonical order.
    pub fn cases(&self) -> impl Iterator<Item = &TargetExecutionCase> {
        self.cases.values()
    }

    /// One execution case by identity.
    #[must_use]
    pub fn case(&self, id: &ExecutionCaseId) -> Option<&TargetExecutionCase> {
        self.cases.get(id)
    }

    /// All Clear relation requirements in canonical order.
    pub fn relations(&self) -> impl Iterator<Item = &TargetRelationRequirement> {
        self.relations.values()
    }

    /// One relation by realization-owned identity.
    #[must_use]
    pub fn relation(&self, id: &RelationId) -> Option<&TargetRelationRequirement> {
        self.relations.get(id)
    }

    /// All abstract carrier requirements.
    pub fn carriers(&self) -> impl Iterator<Item = &AbstractCarrierRequirement> {
        self.carriers.iter()
    }

    /// All layout requirements.
    pub fn layout(&self) -> impl Iterator<Item = &LayoutRequirement> {
        self.layout.iter()
    }

    /// All coverage requirements.
    pub fn coverage(&self) -> impl Iterator<Item = &TargetCoverageRequirement> {
        self.coverage.values()
    }

    /// One coverage requirement by typed identity.
    #[must_use]
    pub fn coverage_requirement(
        &self,
        id: &CoverageRequirementId,
    ) -> Option<&TargetCoverageRequirement> {
        self.coverage.get(id)
    }

    /// Every abstract capability Clear requires.
    pub fn capabilities(&self) -> impl Iterator<Item = RequiredCapability> + '_ {
        self.capabilities.iter().copied()
    }

    /// Every external evidence role Clear leaves open.
    pub fn external_evidence(&self) -> impl Iterator<Item = ExternalEvidenceRole> + '_ {
        self.external_evidence.iter().copied()
    }

    /// Clear's selected-mode lifecycle status.
    #[must_use]
    pub const fn lifecycle(&self) -> &TargetLifecycleStatus {
        &self.lifecycle
    }
}

const fn choice(object: ObjectId) -> RepresentationChoiceId {
    RepresentationChoiceId {
        operation: PLANNED,
        object,
        side: None,
    }
}

fn selected_choices(candidate: &AnalyzedProofPlan) -> ClearRepresentationPolicy {
    ClearRepresentationPolicy {
        selected: candidate
            .proof_plan
            .representations
            .iter()
            .filter(|(choice, _)| choice.operation == PLANNED)
            .map(|(choice, mode)| (choice.clone(), *mode))
            .collect(),
    }
}

fn explicit(candidate: &AnalyzedProofPlan) -> bool {
    let modes = &candidate.proof_plan.representations;
    modes.get(&choice(ObjectId::State)) == Some(&RepresentationMode::Explicit)
        && modes.get(&choice(ObjectId::Ash)) == Some(&RepresentationMode::Explicit)
}

fn factors(analyzed: &ScopedAnalyzedProgram) -> Vec<(&AnalyzedProofPlan, &AnalyzedOperation)> {
    analyzed
        .proof_plans
        .values()
        .filter(|candidate| explicit(candidate))
        .filter_map(|candidate| {
            candidate
                .operations
                .get(&PLANNED)
                .map(|factor| (candidate, factor))
        })
        .collect()
}

fn capability_census(
    relations: &BTreeMap<RelationId, TargetRelationRequirement>,
) -> Result<BTreeSet<RequiredCapability>, CompileError> {
    let present = relations
        .values()
        .flat_map(|row| row.required_capabilities.iter().copied())
        .collect::<BTreeSet<_>>();
    canonical_census(&present, RequiredCapability::ALL, |capability| {
        CompileError::NoncanonicalCapabilityCensus { capability }
    })
    .map(|rows| rows.into_iter().collect())
}

fn evidence_census(
    relations: &BTreeMap<RelationId, TargetRelationRequirement>,
) -> Result<BTreeSet<ExternalEvidenceRole>, CompileError> {
    let present = relations
        .values()
        .flat_map(|row| row.external_evidence.iter())
        .map(ExternalEvidenceRole::of)
        .collect::<BTreeSet<_>>();
    canonical_census(&present, ExternalEvidenceRole::ALL, |role| {
        CompileError::NoncanonicalEvidenceRoleCensus { role }
    })
    .map(|rows| rows.into_iter().collect())
}

const fn field_identity(field: StateField) -> RelationId {
    RelationId::new(
        PLANNED,
        RelationKind::ClearAssignment,
        RelationSubject::StateField { field },
    )
}

const fn field_requirement(field: StateField, predicate: Option<ExprId>) -> ClearFieldRequirement {
    ClearFieldRequirement {
        field,
        relation: field_identity(field),
        input: FactId::StateField {
            operation: PLANNED,
            side: TransactionSide::Input,
            field,
        },
        output: FactId::StateField {
            operation: PLANNED,
            side: TransactionSide::Output,
            field,
        },
        predicate,
    }
}

fn project_fields(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<BTreeMap<StateField, ClearFieldRequirement>, CompileError> {
    let mut fields = BTreeMap::new();
    for declaration in &analyzed.source.realization.relations.nodes {
        if declaration.id.operation() != PLANNED {
            continue;
        }
        let Relation::ClearAssignment { field, expression } = &declaration.relation else {
            continue;
        };
        if declaration.id != field_identity(*field)
            || fields
                .insert(*field, field_requirement(*field, expression.clone()))
                .is_some()
        {
            return Err(CompileError::TargetPlanRelationRequirementMismatch);
        }
    }
    Ok(fields)
}

fn expected_fields() -> BTreeMap<StateField, ClearFieldRequirement> {
    StateField::ALL
        .iter()
        .map(|field| {
            let predicate = (*field != StateField::YL)
                .then(|| ExprId::relation(field_identity(*field), ExpressionRole::Predicate));
            (*field, field_requirement(*field, predicate))
        })
        .collect()
}

fn canonical_relation_ids() -> BTreeSet<RelationId> {
    use RelationKind as K;
    let operation = |kind| RelationId::new(PLANNED, kind, RelationSubject::Operation);
    let mut ids = BTreeSet::from([
        operation(K::Authorization),
        operation(K::CanonicalDeltaPolicy),
        operation(K::ClearClamp),
        operation(K::ClearFloor),
        operation(K::ClearFlow),
        operation(K::ClearBinding),
        operation(K::DataOutputs),
        operation(K::OpenFlowPolicy),
        operation(K::RootPolicy),
        operation(K::ProjectionPolicy),
        operation(K::Constructibility),
        RelationId::new(PLANNED, K::SponsorIsolation, RelationSubject::Sponsor),
        RelationId::new(
            PLANNED,
            K::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
        ),
        RelationId::new(
            PLANNED,
            K::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ),
    ]);
    for side in [TransactionSide::Input, TransactionSide::Output] {
        ids.insert(RelationId::new(
            PLANNED,
            K::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
        ));
        for object in [ObjectId::State, ObjectId::Ash, ObjectId::PlainLbtc] {
            for kind in [K::Cardinality, K::Recognition] {
                ids.insert(RelationId::new(
                    PLANNED,
                    kind,
                    RelationSubject::ObjectFamily { side, object },
                ));
            }
        }
    }
    for object in [ObjectId::State, ObjectId::Ash] {
        ids.insert(RelationId::new(
            PLANNED,
            K::Representation,
            RelationSubject::Representation { object },
        ));
    }
    for field in StateField::ALL {
        ids.insert(field_identity(*field));
    }
    for exit in [OperationId::CompactAsh, OperationId::Clear] {
        ids.insert(RelationId::new(
            PLANNED,
            K::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::Ash,
                exit,
            },
        ));
    }
    for exit in [
        OperationId::AdmitDeposits,
        OperationId::Cycle,
        OperationId::Redeem,
        OperationId::ReceiptRelabel,
        OperationId::Clear,
        OperationId::AnnounceMaturity,
    ] {
        ids.insert(RelationId::new(
            PLANNED,
            K::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::State,
                exit,
            },
        ));
    }
    ids
}

fn project_factor(
    analyzed: &ScopedAnalyzedProgram,
    candidate: &AnalyzedProofPlan,
    factor: &AnalyzedOperation,
) -> Result<ValidatedClearOperationPlan, CompileError> {
    let relations = operation_relations(factor);
    let canonical = canonical_relation_ids();
    if let Some(relation) = relations.symmetric_difference(&canonical).next() {
        return Err(CompileError::TargetPlanRelationCensusMismatch {
            relation: relation.clone(),
        });
    }
    let requirements = project_relations(candidate, factor, &relations)?;
    Ok(ValidatedClearOperationPlan {
        operation: PLANNED,
        source: TargetOperationSource::of(analyzed),
        representation: selected_choices(candidate),
        fields: project_fields(analyzed)?,
        cases: project_cases(factor),
        carriers: project_carriers(factor),
        layout: factor.layout_requirements.clone(),
        coverage: project_coverage(candidate, factor)?,
        capabilities: capability_census(&requirements)?,
        external_evidence: evidence_census(&requirements)?,
        lifecycle: project_lifecycle(candidate, &relations),
        relations: requirements,
    })
}

/// Plan one explicit Clear from an owner-bound compiler input.
///
/// The complete analyzed program and the Clear facade are both validated
/// before this constructor returns. Its report anchors remain obligations,
/// not evidence that a report was produced or authenticated.
///
/// # Errors
///
/// Returns any analysis, scope, selection, ambiguity or plan-validation
/// refusal raised by the complete compiler pipeline.
pub fn plan_clear_target_operation(
    input: &crate::BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<ValidatedClearOperationPlan, CompileError> {
    let operation_code = PLANNED.code();
    let scope_operation_count = crate::instrumentation::count(input.scope().operations().len());
    let span = tracing::info_span!(
        target: "compiler::plan",
        "plan_target_operation",
        operation_code,
        family = "clear",
        scope_operation_count
    );
    let _entered = span.enter();
    let result = (|| {
        let analyzed = analyze_scoped_program(input, placement_limits)?;
        let proof_plan_count = if tracing::enabled!(target: "compiler::plan", tracing::Level::INFO)
        {
            crate::instrumentation::count(analyzed.proof_plans.len())
        } else {
            0
        };
        let plan = derive_clear_plan(&analyzed)?;
        validate_clear_plan(&analyzed, &plan)?;
        Ok((plan, proof_plan_count))
    })();
    crate::instrumentation::plan_completion(&result, operation_code, "clear");
    result.map(|(plan, _)| plan)
}

/// Assemble the Clear facade from a complete validated analysis.
///
/// # Errors
///
/// Returns a typed scope, selection, ambiguity or projection error.
pub(crate) fn derive_clear_plan(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<ValidatedClearOperationPlan, CompileError> {
    if !analyzed
        .source
        .compilation_scope
        .operations()
        .contains(&PLANNED)
    {
        return Err(CompileError::TargetOperationOutOfScope { operation: PLANNED });
    }
    let mut distinct = Vec::new();
    for (candidate, factor) in factors(analyzed) {
        let projected = project_factor(analyzed, candidate, factor)?;
        if !distinct.contains(&projected) {
            distinct.push(projected);
        }
    }
    let mut distinct = distinct.into_iter();
    let plan = distinct
        .next()
        .ok_or(CompileError::NoAdmissiblePhase4Plan { operation: PLANNED })?;
    if distinct.next().is_some() {
        return Err(CompileError::AmbiguousTargetOperationPlan { operation: PLANNED });
    }
    Ok(plan)
}

/// Check every published Clear projection against a re-projection of every
/// selected analyzed factor through the constructor's projection functions;
/// the literal field table is its independent expectation.
///
/// # Errors
///
/// Returns the typed mismatch for the first changed census or selection.
pub(crate) fn validate_clear_plan(
    analyzed: &ScopedAnalyzedProgram,
    plan: &ValidatedClearOperationPlan,
) -> Result<(), CompileError> {
    if plan.operation != PLANNED {
        return Err(CompileError::TargetOperationOutOfScope {
            operation: plan.operation,
        });
    }
    if !plan.source.is_analyzed_source(analyzed) {
        return Err(CompileError::TargetPlanSourceMismatch);
    }
    if plan.fields != expected_fields() || plan.fields != project_fields(analyzed)? {
        return Err(CompileError::TargetPlanRelationRequirementMismatch);
    }
    for object in [ObjectId::State, ObjectId::Ash] {
        if plan.representation.selection(&choice(object)) != Some(RepresentationMode::Explicit) {
            return Err(CompileError::TargetPlanRepresentationMismatch { object });
        }
    }
    validate_sponsor_erasure(plan)?;
    let factors = factors(analyzed);
    if factors.is_empty() {
        return Err(CompileError::NoAdmissiblePhase4Plan { operation: PLANNED });
    }
    for (candidate, factor) in factors {
        if plan.representation != selected_choices(candidate) {
            return Err(CompileError::TargetPlanRepresentationMismatch {
                object: ObjectId::State,
            });
        }
        validate_factor(plan, candidate, factor)?;
    }
    Ok(())
}

fn validate_factor(
    plan: &ValidatedClearOperationPlan,
    candidate: &AnalyzedProofPlan,
    factor: &AnalyzedOperation,
) -> Result<(), CompileError> {
    let relations = operation_relations(factor);
    let published = plan.relations.keys().cloned().collect::<BTreeSet<_>>();
    if let Some(relation) = relations.symmetric_difference(&published).next() {
        return Err(CompileError::TargetPlanRelationCensusMismatch {
            relation: relation.clone(),
        });
    }
    if plan.cases != project_cases(factor) {
        return Err(CompileError::TargetPlanCaseCensusMismatch);
    }
    if plan.carriers != project_carriers(factor) {
        return Err(CompileError::TargetPlanCarrierCensusMismatch);
    }
    if plan.layout != factor.layout_requirements {
        return Err(CompileError::TargetPlanLayoutCensusMismatch);
    }
    let requirements = project_relations(candidate, factor, &relations)?;
    if plan.relations != requirements {
        return Err(CompileError::TargetPlanRelationRequirementMismatch);
    }
    if plan.coverage != project_coverage(candidate, factor)? {
        return Err(CompileError::TargetPlanCoverageCensusMismatch);
    }
    if plan.capabilities != capability_census(&requirements)? {
        return Err(CompileError::TargetPlanCapabilityCensusMismatch);
    }
    if plan.external_evidence != evidence_census(&requirements)? {
        return Err(CompileError::TargetPlanEvidenceCensusMismatch);
    }
    if plan.lifecycle != project_lifecycle(candidate, &relations) {
        return Err(CompileError::TargetPlanLifecycleMismatch);
    }
    Ok(())
}

fn validate_sponsor_erasure(plan: &ValidatedClearOperationPlan) -> Result<(), CompileError> {
    let layouts = plan
        .layout
        .iter()
        .chain(plan.carriers.iter().flat_map(|row| {
            row.alternatives
                .iter()
                .flat_map(|alternative| alternative.layout.iter())
        }))
        .chain(plan.relations.values().flat_map(|relation| {
            relation
                .cases
                .values()
                .flat_map(|case| case.layout_requirements.iter())
        }));
    if layouts.into_iter().any(crate::layout::names_sponsor_amount) {
        return Err(CompileError::SponsorValueRead);
    }
    Ok(())
}

#[cfg(test)]
impl ValidatedClearOperationPlan {
    pub(crate) fn fields_mut(&mut self) -> &mut BTreeMap<StateField, ClearFieldRequirement> {
        &mut self.fields
    }

    pub(crate) fn relations_mut(&mut self) -> &mut BTreeMap<RelationId, TargetRelationRequirement> {
        &mut self.relations
    }

    pub(crate) fn carriers_mut(&mut self) -> &mut BTreeSet<AbstractCarrierRequirement> {
        &mut self.carriers
    }

    pub(crate) fn coverage_mut(
        &mut self,
    ) -> &mut BTreeMap<CoverageRequirementId, TargetCoverageRequirement> {
        &mut self.coverage
    }

    pub(crate) fn layout_mut(&mut self) -> &mut BTreeSet<LayoutRequirement> {
        &mut self.layout
    }

    pub(crate) fn lifecycle_mut(&mut self) -> &mut TargetLifecycleStatus {
        &mut self.lifecycle
    }
}
