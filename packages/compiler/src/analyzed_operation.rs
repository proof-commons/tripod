//! Factorized operation-local analysis (Guide-7 §8, §9, §10).
//!
//! Proof plans are global because one plan may select compatible
//! strategies across several operations. Everything below a fixed plan
//! is not: an operation's execution cases, relation-case dispositions,
//! carrier eligibility, placements, layout, and coverage name only that
//! operation's relations, so the analysis of a multi-operation scope is
//! the *collection* of per-operation factors rather than their product.
//!
//! That distinction is the whole point of this module. The feasible
//! placement set of one plan is a product over independent relation-case
//! obligations, so a two-operation scope's placement set is the product
//! of the two operations' sets — complete, exponential in the number of
//! operations, and carrying no information the factors do not already
//! carry. The canonical path here therefore never enumerates across
//! operations: every search is handed one operation's obligations, and
//! the work is a sum over operations rather than a product.
//!
//! Reuse rather than a second search is deliberate. The placement search
//! reads its obligation census from the eligible carrier sets it is
//! handed and validates it against the relation-case plans it is handed;
//! restricting both to one operation's execution cases restricts the
//! odometer to that operation's relation-cases and leaves the search
//! itself untouched. A parallel operation-local enumerator would be a
//! second implementation of the same policy, free to drift from the one
//! the exact validator checks.
//!
//! Nothing here is completed evidence. A retained external-evidence
//! requirement is not discharged evidence, a layout requirement is not
//! an emitted structure, a carrier assignment alternative is not a
//! target program, and an inactive relation-case is present in the
//! census with its vacuity stated rather than omitted.

// Two item-level allowances remain, and they are the same allowance
// twice: `analyze_operation` and `analyze_candidate_operations` are the
// variants that discard the placement search report. Both production
// callers — the assembler and the complete validator's re-derivation —
// keep the report, so only a caller that does not want it uses these.

use std::collections::{BTreeMap, BTreeSet};

use architecture::OperationId;
use realization::{ExternalEvidenceRequirement, RelationId};

use crate::{
    CompileError,
    carrier::{CarrierEligibility, CarrierRole, relation_case_eligibility},
    case::{ExecutionCase, ExecutionCaseId, case_operations, execution_cases},
    coverage::{
        CarrierAssignmentAlternative, OperationCoverageProjection, RelationCoverageProjection,
        analyze_placed_coverage,
    },
    coverage_graph::{CoverageGraphProjection, resolve_coverage_dependencies},
    layout::{LayoutRequirement, layout_requirements},
    placement::{
        ActivationCondition, BackendStructuralRequirement, CompilerStaticRequirement,
        DischargeBoundary, PlacedProofPlanCandidate, PlacementCandidate,
        PlacementCandidateProjection, PlacementSearchLimits, PlacementSearchReport,
        RelationActivity, RelationCaseKey, RelationCasePlan, classify_relation_cases,
        enumerate_feasible_placements, validate_placement,
    },
    proof::ProofPlanCandidate,
    relation::CompilerRelationAnalysis,
    requirement::{RelationRequirements, active_source_requirements},
    source::SourceRequirement,
};

/// Why one relation-case requirement bundle is not admissible.
///
/// A typed reason rather than a message: each variant names one
/// retention rule of Guide-7 §8.2–§8.6, so a failure says which
/// disposition's contract was broken instead of that something differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationCaseDefect {
    /// The bundle's key names another operation.
    ForeignOperation,
    /// The relation carries no requirement bundle under this plan.
    MissingRelationRequirements,
    /// The bundle's activity differs from its coverage projection's.
    ActivityMismatch,
    /// The bundle's boundaries differ from its coverage projection's.
    BoundaryMismatch,
    /// The bundle's carrier alternatives differ from its coverage
    /// projection's.
    CarrierMismatch,
    /// The bundle's evidence differs from its coverage projection's.
    EvidenceMismatch,
    /// An inactive relation-case retains an active source row.
    ActiveSourceOnInactiveCase,
    /// A relation-case with no runtime boundary retains a runtime
    /// carrier.
    CarrierOnNonRuntimeCase,
    /// An active runtime relation-case retains no carrier alternative.
    MissingRuntimeCarrier,
    /// A relation-case with no compiler-static boundary retains a
    /// compiler validation requirement, or one with that boundary
    /// retains none.
    CompilerRequirementMismatch,
    /// A relation-case with no backend-structural boundary retains a
    /// structural requirement, or one with that boundary retains none.
    StructuralRequirementMismatch,
    /// A relation-case with no external-evidence boundary retains a
    /// typed evidence requirement, or one with that boundary retains
    /// none.
    ExternalEvidenceMismatch,
    /// The bundle states a layout requirement the operation census does
    /// not contain.
    SurplusLayoutRequirement,
}

/// Everything one relation-case owns under one fixed proof plan
/// (Guide-7 §8.1).
///
/// The owning key and the boundary set stay explicit even where the
/// coverage projection restates them: the key is what makes the bundle
/// operation-owned, and the boundary distinctions are what keep a
/// compiler-static obligation from being read as target execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationCaseRequirements {
    pub key: RelationCaseKey,
    pub activity: RelationActivity,
    pub activation: ActivationCondition,
    pub boundaries: BTreeSet<DischargeBoundary>,
    /// The relation's source rows active in *this* case. Empty for an
    /// inactive relation-case, which requires nothing of any carrier.
    pub active_sources: BTreeSet<SourceRequirement>,
    pub compiler_requirements: BTreeSet<CompilerStaticRequirement>,
    pub structural_requirements: BTreeSet<BackendStructuralRequirement>,
    /// Every accepted inclusion-minimal carrier alternative, compressed
    /// out of the operation's feasible placements.
    pub carrier_assignments: BTreeSet<CarrierAssignmentAlternative>,
    /// The layout this relation-case's own alternatives and structural
    /// obligations depend on — a subset of the operation census.
    pub layout_requirements: BTreeSet<LayoutRequirement>,
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
    pub coverage: RelationCoverageProjection,
}

/// One operation's complete placement analysis under one fixed plan
/// (Guide-7 §9.6).
///
/// The intermediate stages are retained rather than discarded because
/// the exact placement validator needs the relation-case plans and the
/// eligible carrier sets the placements were built from; a validator
/// handed only the placements could check their shape and nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationPlacementAnalysis {
    pub operation: OperationId,
    pub execution_cases: Vec<ExecutionCase>,
    pub relation_case_plans: Vec<RelationCasePlan>,
    pub eligibility: Vec<CarrierEligibility>,
    pub placements: Vec<PlacementCandidate>,
    /// The operation-wide census over every eligible carrier.
    pub layout_requirements: Vec<LayoutRequirement>,
    pub search: PlacementSearchReport,
}

/// One operation's complete analysis factor under one fixed plan
/// (Guide-7 §9.2).
///
/// The proof plan is absent deliberately: it is global, the factor is
/// local, and a copy of the plan inside every factor would invite two
/// operations to disagree about which plan they were analyzed under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalyzedOperation {
    pub operation: OperationId,
    pub execution_cases: BTreeSet<ExecutionCaseId>,
    pub relation_cases: BTreeMap<RelationCaseKey, RelationCaseRequirements>,
    pub feasible_placements: BTreeSet<PlacementCandidateProjection>,
    pub layout_requirements: BTreeSet<LayoutRequirement>,
    pub coverage: OperationCoverageProjection,
    pub coverage_dependencies: CoverageGraphProjection,
}

/// The execution cases of one operation under one fixed plan (§9.4).
///
/// The complete case census is derived and validated first, then
/// restricted: the restriction is a projection of a validated whole,
/// not a separate derivation that could disagree with it.
///
/// # Errors
///
/// [`CompileError::AnalyzedOperationOutOfScope`] when the operation
/// carries no in-scope relation; any failure of
/// [`crate::case::execution_cases`].
pub fn operation_execution_cases(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    operation: OperationId,
) -> Result<Vec<ExecutionCase>, CompileError> {
    if !case_operations(relations).contains(&operation) {
        return Err(CompileError::AnalyzedOperationOutOfScope { operation });
    }

    Ok(execution_cases(relations, candidate)?
        .into_iter()
        .filter(|case| case.id.operation == operation)
        .collect())
}

/// Enumerate one operation's feasible placements (Guide-7 §9.6).
///
/// The obligation census handed to the search is one operation's
/// relation-cases, so the odometer runs over that operation's options
/// only and no combination across operations is ever materialized. The
/// search itself is the same exact enumerator a whole-scope call uses;
/// only its input is restricted.
///
/// # Errors
///
/// Any failure of [`operation_execution_cases`],
/// [`classify_relation_cases`],
/// [`crate::carrier::relation_case_eligibility`],
/// [`crate::layout::layout_requirements`],
/// [`enumerate_feasible_placements`], or
/// [`validate_operation_placements`].
pub fn analyze_operation_placements(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    operation: OperationId,
    limits: PlacementSearchLimits,
) -> Result<OperationPlacementAnalysis, CompileError> {
    let cases = operation_execution_cases(relations, candidate, operation)?;
    let plans = classify_relation_cases(relations, &cases)?;
    let eligibility = relation_case_eligibility(relations, &plans)?;
    let requirements = layout_requirements(relations, &plans, &eligibility)?;
    let placements = enumerate_feasible_placements(&plans, &eligibility, limits)?;

    let analysis = OperationPlacementAnalysis {
        operation,
        execution_cases: cases,
        relation_case_plans: plans,
        eligibility,
        placements: placements.candidates,
        layout_requirements: requirements,
        search: placements.search,
    };

    validate_operation_placements(&analysis)?;
    Ok(analysis)
}

/// Validate one operation factor's placements exactly (Guide-7 §9.7).
///
/// Every placement is re-checked by the exact whole-placement validator
/// rather than trusted because the enumerator produced it, and every
/// carried value is checked to belong to the factor's own operation.
///
/// # Errors
///
/// [`CompileError::ForeignOperationFactor`] when a case, relation-case,
/// eligible carrier set, or layout requirement belongs to another
/// operation; any failure of [`validate_placement`].
pub fn validate_operation_placements(
    analysis: &OperationPlacementAnalysis,
) -> Result<(), CompileError> {
    let operation = analysis.operation;
    let foreign = |found| CompileError::ForeignOperationFactor { operation, found };

    for case in &analysis.execution_cases {
        if case.id.operation != operation {
            return Err(foreign(case.id.operation));
        }
    }

    for plan in &analysis.relation_case_plans {
        if plan.relation.operation() != operation || plan.case.operation != operation {
            return Err(foreign(plan.relation.operation()));
        }
    }

    for entry in &analysis.eligibility {
        if entry.relation.operation() != operation || entry.case.operation != operation {
            return Err(foreign(entry.relation.operation()));
        }
    }

    for requirement in &analysis.layout_requirements {
        if layout_operation(requirement) != operation {
            return Err(foreign(layout_operation(requirement)));
        }
    }

    for placement in &analysis.placements {
        validate_placement(
            &analysis.relation_case_plans,
            &analysis.eligibility,
            placement,
        )?;
    }

    Ok(())
}

/// Analyze one operation completely under one fixed proof plan
/// (Guide-7 §9, §10).
///
/// # Errors
///
/// Any failure of [`analyze_operation_placements`],
/// [`crate::coverage::analyze_placed_coverage`],
/// [`crate::coverage_graph::resolve_coverage_dependencies`],
/// [`relation_case_requirements`], or [`validate_analyzed_operation`].
#[allow(dead_code)]
pub fn analyze_operation(
    relations: &CompilerRelationAnalysis,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
    candidate: &ProofPlanCandidate,
    operation: OperationId,
    limits: PlacementSearchLimits,
) -> Result<AnalyzedOperation, CompileError> {
    Ok(analyze_operation_reported(relations, requirements, candidate, operation, limits)?.0)
}

/// Analyze one operation and retain its placement search report.
///
/// The report is diagnostic provenance rather than analysis: it is
/// returned beside the factor instead of inside it, so a consumer that
/// records limits and state counts can do so without those counts ever
/// entering a value intended for stable comparison.
///
/// # Errors
///
/// Any failure of [`analyze_operation`].
pub fn analyze_operation_reported(
    relations: &CompilerRelationAnalysis,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
    candidate: &ProofPlanCandidate,
    operation: OperationId,
    limits: PlacementSearchLimits,
) -> Result<(AnalyzedOperation, PlacementSearchReport), CompileError> {
    let placement = analyze_operation_placements(relations, candidate, operation, limits)?;
    let search = placement.search;

    // The operation-local placed value exists only to reuse the exact
    // coverage stage, which is already per-operation inside. It carries
    // the same global proof plan, so nothing here can select a
    // different one.
    let placed = PlacedProofPlanCandidate {
        proof_plan: candidate.clone(),
        execution_cases: placement.execution_cases.clone(),
        relation_case_plans: placement.relation_case_plans.clone(),
        feasible_placements: placement.placements.clone(),
        layout_requirements: placement.layout_requirements.clone(),
    };

    let mut coverage = analyze_placed_coverage(relations, &placed)?;
    let graph = resolve_coverage_dependencies(&mut coverage, relations)?;

    let covered = coverage.operations.keys().copied().collect::<BTreeSet<_>>();

    if covered != BTreeSet::from([operation]) {
        let found = covered
            .into_iter()
            .find(|covered| *covered != operation)
            .unwrap_or(operation);

        return Err(CompileError::ForeignOperationFactor { operation, found });
    }

    let projection = coverage
        .operations
        .get(&operation)
        .ok_or(CompileError::ForeignOperationFactor {
            operation,
            found: operation,
        })?
        .project();

    let relation_cases = relation_case_requirements(&placement, requirements, &projection)?;

    let analyzed = AnalyzedOperation {
        operation,
        execution_cases: placement
            .execution_cases
            .iter()
            .map(|case| case.id.clone())
            .collect(),
        relation_cases,
        feasible_placements: placement
            .placements
            .iter()
            .map(PlacementCandidate::project)
            .collect(),
        layout_requirements: placement.layout_requirements.iter().cloned().collect(),
        coverage: projection,
        coverage_dependencies: graph.project(),
    };

    validate_analyzed_operation(relations, candidate, &analyzed)?;
    Ok((analyzed, search))
}

/// Analyze every operation of one scope under one fixed plan.
///
/// A sum over operations, never a product: each factor is analyzed on
/// its own obligations and stored beside the others.
///
/// # Errors
///
/// Any failure of [`analyze_operation`] for any operation, or of
/// [`validate_operation_factorization`] on the assembled factors.
// Both production callers — the assembler and the complete validator's
// re-derivation — want the placement search reports, so both take the
// reported form and this projection of it has test callers only. It is
// kept because a test that only asserts about factors should not have
// to name and discard a report it does not read.
#[allow(dead_code)]
pub fn analyze_candidate_operations(
    relations: &CompilerRelationAnalysis,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
    candidate: &ProofPlanCandidate,
    limits: PlacementSearchLimits,
) -> Result<BTreeMap<OperationId, AnalyzedOperation>, CompileError> {
    Ok(analyze_candidate_operations_reported(relations, requirements, candidate, limits)?.0)
}

/// One scope's operation factors beside their placement search
/// reports.
///
/// The two travel as a pair rather than as one value: the factors are
/// the analysis, the reports are provenance about how it was found, and
/// a type joining them permanently would put search counts inside the
/// value the analyzed program compares.
pub type ReportedOperationAnalyses = (
    BTreeMap<OperationId, AnalyzedOperation>,
    BTreeMap<OperationId, PlacementSearchReport>,
);

/// Analyze every operation of one scope and retain their placement
/// search reports.
///
/// # Errors
///
/// Any failure of [`analyze_candidate_operations`].
pub fn analyze_candidate_operations_reported(
    relations: &CompilerRelationAnalysis,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
    candidate: &ProofPlanCandidate,
    limits: PlacementSearchLimits,
) -> Result<ReportedOperationAnalyses, CompileError> {
    let mut factors = BTreeMap::new();
    let mut reports = BTreeMap::new();

    for operation in case_operations(relations) {
        let (analyzed, search) =
            analyze_operation_reported(relations, requirements, candidate, operation, limits)?;

        factors.insert(operation, analyzed);
        reports.insert(operation, search);
    }

    validate_operation_factorization(relations, candidate, &factors)?;
    Ok((factors, reports))
}

/// Build one operation's relation-case requirement bundles (§8).
///
/// # Errors
///
/// [`CompileError::InvalidRelationCaseRequirements`] for the first
/// bundle breaking one of the §8.2–§8.6 retention rules.
pub fn relation_case_requirements(
    placement: &OperationPlacementAnalysis,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
    coverage: &OperationCoverageProjection,
) -> Result<BTreeMap<RelationCaseKey, RelationCaseRequirements>, CompileError> {
    let mut bundles = BTreeMap::new();

    for plan in &placement.relation_case_plans {
        let key = RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        };
        let defect = |defect| CompileError::InvalidRelationCaseRequirements {
            relation: key.relation.clone(),
            case: key.case.clone(),
            defect,
        };

        let owned = requirements
            .get(&plan.relation)
            .ok_or_else(|| defect(RelationCaseDefect::MissingRelationRequirements))?;
        let projection = coverage
            .requirements
            .get(&key)
            .ok_or_else(|| defect(RelationCaseDefect::MissingRelationRequirements))?;

        let carrier_assignments = projection.carrier.clone();
        let layout =
            relation_case_layout(&key, &carrier_assignments, &placement.layout_requirements);

        let bundle = RelationCaseRequirements {
            key: key.clone(),
            activity: plan.activity,
            activation: plan.activation,
            boundaries: plan.boundaries.clone(),
            active_sources: relation_case_sources(plan, owned),
            compiler_requirements: plan.compiler_requirements.iter().cloned().collect(),
            structural_requirements: plan.structural_requirements.iter().cloned().collect(),
            carrier_assignments,
            layout_requirements: layout,
            external_evidence: plan.external_evidence.clone(),
            coverage: projection.clone(),
        };

        validate_relation_case_requirements(
            placement.operation,
            &placement.layout_requirements,
            &bundle,
        )?;
        bundles.insert(key, bundle);
    }

    Ok(bundles)
}

/// Validate one relation-case bundle against its disposition's
/// retention rules (§8.2–§8.6).
///
/// # Errors
///
/// [`CompileError::InvalidRelationCaseRequirements`] naming the typed
/// rule the bundle breaks.
pub fn validate_relation_case_requirements(
    operation: OperationId,
    census: &[LayoutRequirement],
    bundle: &RelationCaseRequirements,
) -> Result<(), CompileError> {
    let defect = |defect| CompileError::InvalidRelationCaseRequirements {
        relation: bundle.key.relation.clone(),
        case: bundle.key.case.clone(),
        defect,
    };

    if bundle.key.relation.operation() != operation || bundle.key.case.operation != operation {
        return Err(defect(RelationCaseDefect::ForeignOperation));
    }

    // The restated fields are checked against the coverage projection
    // rather than assumed to agree with it: a bundle whose explicit
    // boundary set disagreed with its own coverage would report two
    // different dispositions for one relation-case.
    if bundle.activity != bundle.coverage.activity {
        return Err(defect(RelationCaseDefect::ActivityMismatch));
    }

    if bundle.boundaries != bundle.coverage.boundaries {
        return Err(defect(RelationCaseDefect::BoundaryMismatch));
    }

    if bundle.carrier_assignments != bundle.coverage.carrier {
        return Err(defect(RelationCaseDefect::CarrierMismatch));
    }

    if bundle.external_evidence != bundle.coverage.external_evidence {
        return Err(defect(RelationCaseDefect::EvidenceMismatch));
    }

    let runtime = bundle
        .boundaries
        .contains(&DischargeBoundary::RuntimeCarrier);
    let inactive = bundle.activity == RelationActivity::Vacuous;

    // §8.6: an inactive relation-case is present, states its vacuity,
    // and requires nothing of any carrier.
    if inactive && !bundle.active_sources.is_empty() {
        return Err(defect(RelationCaseDefect::ActiveSourceOnInactiveCase));
    }

    if inactive && !bundle.carrier_assignments.is_empty() {
        return Err(defect(RelationCaseDefect::CarrierOnNonRuntimeCase));
    }

    // §8.3–§8.5: a compiler-static, structural, or externally evidenced
    // relation-case never receives a runtime carrier.
    if !runtime && !bundle.carrier_assignments.is_empty() {
        return Err(defect(RelationCaseDefect::CarrierOnNonRuntimeCase));
    }

    // §8.2: an active runtime relation-case retains every feasible
    // alternative, so retaining none would be an unplaceable obligation
    // reported as analyzed.
    if runtime && !inactive && bundle.carrier_assignments.is_empty() {
        return Err(defect(RelationCaseDefect::MissingRuntimeCarrier));
    }

    if bundle
        .boundaries
        .contains(&DischargeBoundary::CompilerStatic)
        == bundle.compiler_requirements.is_empty()
    {
        return Err(defect(RelationCaseDefect::CompilerRequirementMismatch));
    }

    if bundle
        .boundaries
        .contains(&DischargeBoundary::BackendStructural)
        == bundle.structural_requirements.is_empty()
    {
        return Err(defect(RelationCaseDefect::StructuralRequirementMismatch));
    }

    if bundle
        .boundaries
        .contains(&DischargeBoundary::ExternalEvidence)
        == bundle.external_evidence.is_empty()
    {
        return Err(defect(RelationCaseDefect::ExternalEvidenceMismatch));
    }

    let present = census.iter().collect::<BTreeSet<_>>();

    for requirement in &bundle.layout_requirements {
        if !present.contains(requirement) {
            return Err(defect(RelationCaseDefect::SurplusLayoutRequirement));
        }
    }

    Ok(())
}

/// Validate one complete operation factor (Guide-7 §9.3, §9.5, §10.1).
///
/// The censuses are recomputed from the relation analysis and the
/// candidate rather than read back out of the factor, so a factor that
/// dropped a relation-case is caught here instead of agreeing with
/// itself.
///
/// # Errors
///
/// [`CompileError::AnalyzedOperationOutOfScope`] when the factor names
/// an operation carrying no in-scope relation;
/// [`CompileError::ForeignOperationFactor`] when any carried value
/// belongs to another operation;
/// [`CompileError::RelationCaseCensusMismatch`] when the relation-case
/// or coverage census is not exactly the operation's relations crossed
/// with its cases;
/// [`CompileError::AnalyzedOperationProofPlanMismatch`] when a case
/// fixes a representation the candidate did not select;
/// [`CompileError::PlacementCensusMismatch`] when a placement does not
/// carry exactly the active runtime relation-cases; any failure of
/// [`validate_relation_case_requirements`].
pub fn validate_analyzed_operation(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    analyzed: &AnalyzedOperation,
) -> Result<(), CompileError> {
    let operation = analyzed.operation;

    if !case_operations(relations).contains(&operation) {
        return Err(CompileError::AnalyzedOperationOutOfScope { operation });
    }

    let foreign = |found| CompileError::ForeignOperationFactor { operation, found };
    let census = analyzed
        .layout_requirements
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    // §9.4: the cases are exactly the candidate's cases for this
    // operation, and each one fixes exactly the representations the
    // global plan selected — a placement that changed a representation
    // would have changed the plan.
    let expected_cases = operation_execution_cases(relations, candidate, operation)?
        .into_iter()
        .map(|case| case.id)
        .collect::<BTreeSet<_>>();

    if analyzed.execution_cases != expected_cases {
        return Err(CompileError::ExecutionCaseCensusMismatch {
            missing: expected_cases
                .difference(&analyzed.execution_cases)
                .cloned()
                .collect(),
            unexpected: analyzed
                .execution_cases
                .difference(&expected_cases)
                .cloned()
                .collect(),
        });
    }

    for case in &analyzed.execution_cases {
        if case.operation != operation {
            return Err(foreign(case.operation));
        }

        for (object, mode) in &case.representations {
            let choice = crate::lifecycle::RepresentationChoiceId {
                operation,
                object: *object,
            };

            if candidate.representations.get(&choice) != Some(mode) {
                return Err(CompileError::AnalyzedOperationProofPlanMismatch { operation });
            }
        }
    }

    // §9.5: every operation relation crossed with every operation case,
    // exactly once, with all five dispositions present.
    let expected_keys = expected_relation_cases(relations, operation, &analyzed.execution_cases);
    let carried = analyzed
        .relation_cases
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    if carried != expected_keys {
        return Err(CompileError::RelationCaseCensusMismatch {
            missing: expected_keys.difference(&carried).cloned().collect(),
            unexpected: carried.difference(&expected_keys).cloned().collect(),
        });
    }

    for bundle in analyzed.relation_cases.values() {
        validate_relation_case_requirements(operation, &census, bundle)?;
    }

    // §10.1: coverage keys are the operation relation-case keys.
    if analyzed.coverage.operation != operation {
        return Err(foreign(analyzed.coverage.operation));
    }

    let covered = analyzed
        .coverage
        .requirements
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    if covered != expected_keys {
        return Err(CompileError::RelationCaseCensusMismatch {
            missing: expected_keys.difference(&covered).cloned().collect(),
            unexpected: covered.difference(&expected_keys).cloned().collect(),
        });
    }

    for requirement in &analyzed.layout_requirements {
        if layout_operation(requirement) != operation {
            return Err(foreign(layout_operation(requirement)));
        }
    }

    validate_operation_placement_census(analyzed)?;
    validate_coverage_locality(analyzed)?;
    Ok(())
}

/// Validate one factor's placement census against its own bundles
/// (§9.7).
///
/// The required census is read out of the relation-case bundles rather
/// than out of the placements: a placement that dropped an obligation
/// and a bundle set that never stated one are different defects, and
/// comparing the placements only with each other could distinguish
/// neither.
fn validate_operation_placement_census(analyzed: &AnalyzedOperation) -> Result<(), CompileError> {
    let runtime = analyzed
        .relation_cases
        .values()
        .filter(|bundle| bundle.activity == RelationActivity::Active)
        .filter(|bundle| {
            bundle
                .boundaries
                .contains(&DischargeBoundary::RuntimeCarrier)
        })
        .map(|bundle| bundle.key.clone())
        .collect::<BTreeSet<_>>();

    for placement in &analyzed.feasible_placements {
        let assigned = placement
            .assignments
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();

        if assigned != runtime {
            return Err(CompileError::PlacementCensusMismatch {
                missing: runtime.difference(&assigned).cloned().collect(),
                unexpected: assigned.difference(&runtime).cloned().collect(),
            });
        }

        for requirement in &placement.layout_requirements {
            if !analyzed.layout_requirements.contains(requirement) {
                return Err(CompileError::UnexpectedLayoutRequirement {
                    unexpected: vec![requirement.clone()],
                });
            }
        }
    }

    Ok(())
}

/// Validate the factorization conditions (Guide-7 §9.8).
///
/// Conditions 6 to 8 are structural rather than checkable data: the
/// proof plan is a parameter of the analysis rather than a factor
/// output, the factors are compared against it here so a placement
/// cannot have changed it, and no shared target resource model exists
/// in Phase 2 for a placement to consume.
///
/// # Errors
///
/// [`CompileError::AnalyzedOperationCensusMismatch`] when the factor
/// keys are not exactly the scope operations;
/// [`CompileError::CrossOperationRelationDependency`] when a relation
/// dependency crosses two operations;
/// [`CompileError::ForeignOperationFactor`] when a factor carries a
/// value another operation owns; any failure of
/// [`validate_analyzed_operation`].
pub fn validate_operation_factorization(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    factors: &BTreeMap<OperationId, AnalyzedOperation>,
) -> Result<(), CompileError> {
    let expected = case_operations(relations);
    let carried = factors.keys().copied().collect::<BTreeSet<_>>();

    if carried != expected {
        return Err(CompileError::AnalyzedOperationCensusMismatch {
            missing: expected.difference(&carried).copied().collect(),
            unexpected: carried.difference(&expected).copied().collect(),
        });
    }

    // Condition 2: a relation dependency spanning two operations would
    // make one factor's validity depend on another's.
    for edge in relations.project().edges {
        let crate::relation::AnalysisNodeId::SourceRelation(prerequisite) = edge.source;
        let crate::relation::AnalysisNodeId::SourceRelation(dependent) = edge.target;

        if prerequisite.operation() != dependent.operation() {
            return Err(CompileError::CrossOperationRelationDependency {
                prerequisite,
                dependent,
            });
        }
    }

    for (operation, analyzed) in factors {
        if analyzed.operation != *operation {
            return Err(CompileError::ForeignOperationFactor {
                operation: *operation,
                found: analyzed.operation,
            });
        }

        validate_analyzed_operation(relations, candidate, analyzed)?;

        // Condition 3: no carrier spans operations.
        for placement in &analyzed.feasible_placements {
            for carrier in placement.assignments.values().flatten() {
                if let Some(carried) = carrier_operation(&carrier.carrier)
                    && carried != *operation
                {
                    return Err(CompileError::ForeignOperationFactor {
                        operation: *operation,
                        found: carried,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Validate that no coverage dependency leaves the factor (§9.8).
fn validate_coverage_locality(analyzed: &AnalyzedOperation) -> Result<(), CompileError> {
    let operation = analyzed.operation;

    for node in &analyzed.coverage_dependencies.nodes {
        if node.id.operation() != operation {
            return Err(CompileError::ForeignOperationFactor {
                operation,
                found: node.id.operation(),
            });
        }
    }

    for edge in &analyzed.coverage_dependencies.edges {
        if edge.source.operation() != operation || edge.target.operation() != operation {
            return Err(CompileError::CrossOperationCoverageDependency {
                prerequisite: Box::new(edge.source.clone()),
                dependent: Box::new(edge.target.clone()),
            });
        }
    }

    Ok(())
}

/// The relation-case census one operation requires (§9.5).
fn expected_relation_cases(
    relations: &CompilerRelationAnalysis,
    operation: OperationId,
    cases: &BTreeSet<ExecutionCaseId>,
) -> BTreeSet<RelationCaseKey> {
    let mut expected = BTreeSet::new();

    for node in relations.graph.node_weights() {
        if node.source.id.operation() != operation {
            continue;
        }

        for case in cases {
            expected.insert(RelationCaseKey {
                relation: node.source.id.clone(),
                case: case.clone(),
            });
        }
    }

    expected
}

/// The source rows one relation-case actually requires (§8.2, §8.6).
///
/// Two independent activation axes meet here. A source row states when
/// *it* is active — a sponsor-local witness exists only where the
/// sponsor region does — while the relation states when its obligation
/// exists at all. The row axis does not subsume the relation axis: a
/// relation over the optional sponsor family owns rows whose own
/// activation is unconditional, such as the architecture bound its
/// cardinality cites, and those rows are active in the sponsorless case
/// by their own test while the relation they belong to is not.
///
/// An inactive relation-case therefore states no active source. It
/// requires nothing of any carrier, so a row retained here would be an
/// obligation with no obligation behind it — and the runtime
/// requirement the same relation-case would otherwise carry is already
/// absent for exactly that reason.
fn relation_case_sources(
    plan: &RelationCasePlan,
    owned: &RelationRequirements,
) -> BTreeSet<SourceRequirement> {
    if plan.activity == RelationActivity::Vacuous {
        return BTreeSet::new();
    }

    active_source_requirements(owned, &plan.case)
}

/// The layout one relation-case depends on.
///
/// Two sources, both restricted to this relation-case: the layout its
/// own accepted carrier alternatives depend on, and the census entries
/// its own relation owns in its own case. A requirement of another
/// relation-case belongs to the operation census, never to this bundle.
fn relation_case_layout(
    key: &RelationCaseKey,
    assignments: &BTreeSet<CarrierAssignmentAlternative>,
    census: &[LayoutRequirement],
) -> BTreeSet<LayoutRequirement> {
    let mut requirements = assignments
        .iter()
        .flat_map(|alternative| alternative.layout.iter().cloned())
        .collect::<BTreeSet<_>>();

    for requirement in census {
        if requirement.relation() != Some(&key.relation) {
            continue;
        }

        if layout_case(requirement).is_none_or(|case| *case == key.case) {
            requirements.insert(requirement.clone());
        }
    }

    requirements
}

/// The operation one layout requirement belongs to.
const fn layout_operation(requirement: &LayoutRequirement) -> OperationId {
    match requirement {
        LayoutRequirement::CanonicalCoordinator { operation, .. } => *operation,
        LayoutRequirement::AuthenticateFamilyCensus { relation, .. }
        | LayoutRequirement::CompleteAndDisjointFamilies { relation, .. }
        | LayoutRequirement::MakeSourceAvailable { relation, .. }
        | LayoutRequirement::IsolateSponsorRegion { relation, .. }
        | LayoutRequirement::EnforceRepresentation { relation, .. }
        | LayoutRequirement::SecretFreeOperationPath { relation, .. } => relation.operation(),
    }
}

/// The execution case one layout requirement names, where it names one.
const fn layout_case(requirement: &LayoutRequirement) -> Option<&ExecutionCaseId> {
    match requirement {
        LayoutRequirement::MakeSourceAvailable { case, .. }
        | LayoutRequirement::IsolateSponsorRegion { case, .. }
        | LayoutRequirement::SecretFreeOperationPath { case, .. } => Some(case),
        LayoutRequirement::AuthenticateFamilyCensus { .. }
        | LayoutRequirement::CompleteAndDisjointFamilies { .. }
        | LayoutRequirement::CanonicalCoordinator { .. }
        | LayoutRequirement::EnforceRepresentation { .. } => None,
    }
}

/// The operation one carrier role names, where it names one.
///
/// A family-anchored role names an object rather than an operation: it
/// belongs to whichever operation's relation-case selected it, so it
/// can never span two.
const fn carrier_operation(carrier: &CarrierRole) -> Option<OperationId> {
    match carrier {
        CarrierRole::OperationGlobal { operation, .. }
        | CarrierRole::BackendStructural { operation } => Some(*operation),
        CarrierRole::ExternalEvidence { requirement } => Some(requirement.operation()),
        CarrierRole::EveryInputFamilyMember { .. } | CarrierRole::InputFamilyCoordinator { .. } => {
            None
        }
    }
}
