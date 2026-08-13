//! The complete scoped analyzed program (Guide-7 §5, §6, §11, §12, §17).
//!
//! This module assembles the internal analyses into one value that is
//! complete *for its explicit compiler scope*. Every word of that
//! qualification is load-bearing, and the types below keep the separate
//! completeness dimensions apart rather than collapsing them into one
//! boolean: the analysis is complete for the operations it was asked
//! about, the realization does not cover every architecture operation,
//! not every lifecycle exit lies inside the compiler scope, no target
//! is selected, and the external evidence the analysis requires is
//! required rather than obtained.
//!
//! Two storage rulings shape the principal type. Proof plans stay
//! global, because one plan may select compatible strategies across
//! several operations, and they are keyed by the complete typed plan —
//! never by a search rank, a vector position, or a chosen "best" plan,
//! none of which are properties of the analysis. Everything below a
//! fixed plan is stored per operation as the factors of a product that
//! is never materialized: the placement semantics remain the product
//! over operations, while the analyzed value stores only the factors.
//!
//! The execution report is the third separation. Search limits and
//! state counts describe how the analysis was carried out, not what it
//! found, so they are retained beside the semantic result and excluded
//! from the stable projection: a larger sufficient limit must change
//! the report and leave the projection untouched.

// Two item-level allowances remain. The stable projection (§12) is
// built by nothing inside the analysis, because the assembly stores
// the values themselves and §26.1 exposes no API that projects them;
// and `analyze_scoped_program` is the crate-private root of the whole
// pipeline, which by the same ruling has no non-test caller until a
// target package becomes the first honest consumer. Everything between
// those two is reached from the root and carries no allowance.

use std::collections::{BTreeMap, BTreeSet};

use architecture::OperationId;
use realization::{
    ArchitectureBinding, ExternalEvidenceRequirement, RelationId, ScopedRealizationProjection,
};

use crate::{
    CompileError,
    analyzed_operation::{AnalyzedOperation, analyze_candidate_operations_reported},
    analyzed_validate::validate_scoped_analyzed_program,
    capability::CapabilityView,
    constructibility::build_constructibility_analysis,
    foundation::{CompilerAnalysisFoundationProjection, analyze_foundation},
    input::{BoundCompilerInput, CompilationScope, ProofSearchLimits},
    lifecycle::{LifecycleExitStatus, LifecycleRequirement, build_lifecycle_analysis},
    placement::{PlacementSearchLimits, PlacementSearchReport},
    proof::{ProofPlanCandidate, ProofSearchReport, enumerate_feasible_plans},
    requirement::{RelationRequirements, relation_requirements},
};

/// The exact typed source one analyzed program was derived from
/// (Guide-7 §6.1).
///
/// A complete typed source, never a reference to one. A path, a
/// revision, or an architecture hash alone would name a source the
/// analyzed value cannot check, so anything reading this binding would
/// have to trust that the named thing still says what it said. The
/// three retained values — the architecture identity, the complete
/// scoped realization projection, and the explicit compiler scope — are
/// together enough to re-derive the whole analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalyzedSource {
    pub architecture: ArchitectureBinding,
    pub realization: ScopedRealizationProjection,
    pub compilation_scope: CompilationScope,
}

/// How much of the architecture the compiler scope covers (§5.3).
///
/// The missing set is derived from the validated architecture operation
/// census the input retained, never from a written-down list: a
/// hard-coded remainder would keep claiming the same eleven operations
/// after the architecture grew a twelfth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArchitectureScopeStatus {
    Complete,
    Partial { missing: BTreeSet<OperationId> },
}

/// Whether every lifecycle exit one plan requires lies inside the
/// compiler scope (§5.4).
///
/// Incompleteness is expected and valid for a partial pilot scope. It
/// is stated rather than tolerated silently: an object whose required
/// exit is analyzed by nobody has a future obligation, and dropping the
/// obligation would turn a partial analysis into an apparently complete
/// one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleCompleteness {
    Complete,
    Incomplete {
        obligations: BTreeSet<LifecycleRequirement>,
    },
}

/// How the analysis was carried out (§6.3).
///
/// Diagnostic provenance, deliberately outside the stable projection.
/// Limits may turn a complete result into a typed complexity failure;
/// they may never turn one successful complete result into a different
/// successful one, so nothing here can reach the semantic value.
///
/// The per-operation placement counts are accumulated across the plan
/// set. A count is a statement about work, and the work of analyzing
/// one operation is the sum of the work its plans required; keying the
/// report by plan as well would make a diagnostic aggregate as large as
/// the analysis it describes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisExecutionReport {
    pub proof_search_limits: ProofSearchLimits,
    pub placement_search_limits: PlacementSearchLimits,
    pub proof_search: ProofSearchReport,
    pub operation_placement_search: BTreeMap<OperationId, PlacementSearchReport>,
}

/// One globally fixed proof plan, analyzed completely (§11.2).
///
/// The operations map holds the factors of the placement product, not
/// its members: a two-operation scope stores two factors, and the
/// combined placements remain exactly their product without ever being
/// enumerated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalyzedProofPlan {
    pub proof_plan: ProofPlanCandidate,
    pub relation_requirements: BTreeMap<RelationId, RelationRequirements>,
    pub operations: BTreeMap<OperationId, AnalyzedOperation>,
    pub lifecycle: LifecycleCompleteness,
}

/// The complete analysis of one compiler scope (§11.1).
///
/// The plan map keys are complete typed plans, and the whole feasible
/// set is retained: Phase 2 selects no preferred plan, representation,
/// placement, target, or backend, because selection needs a policy no
/// reviewed boundary has yet stated.
///
/// Fields are reachable inside this crate and nowhere else — the module
/// is crate-private and unexported, so no external caller can assemble
/// a purported complete analysis out of independent fields. Assembly
/// runs through [`analyze_scoped_program`] alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedAnalyzedProgram {
    pub source: AnalyzedSource,
    pub foundation: CompilerAnalysisFoundationProjection,
    pub architecture_scope: ArchitectureScopeStatus,
    pub proof_plans: BTreeMap<ProofPlanCandidate, AnalyzedProofPlan>,
    pub required_external_evidence: BTreeSet<ExternalEvidenceRequirement>,
    pub execution_report: AnalysisExecutionReport,
}

/// The stable projection of one analyzed plan (§12.1).
///
/// The typed plan itself is absent: the projection's map key carries it
/// exactly once, and a second copy inside the value would be a place
/// for two answers to the same question to disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct AnalyzedProofPlanProjection {
    pub relation_requirements: BTreeMap<RelationId, RelationRequirements>,
    pub operations: BTreeMap<OperationId, AnalyzedOperation>,
    pub lifecycle: LifecycleCompleteness,
}

/// The stable projection of one analyzed program (§12.1–§12.4).
///
/// Typed semantic values in canonical containers. The execution report
/// is excluded entirely, and with it the search limits, the state and
/// candidate counts, and every other artifact of how the search walked
/// its options. So is the cross-operation placement product, which is
/// not stored to begin with.
///
/// The source binding needs no separate projection type: it already
/// holds the architecture identity, the *projected* realization, and
/// the explicit scope, none of which carry order, handles, or counts.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ScopedAnalyzedProgramProjection {
    pub source: AnalyzedSource,
    pub foundation: CompilerAnalysisFoundationProjection,
    pub architecture_scope: ArchitectureScopeStatus,
    pub proof_plans: BTreeMap<ProofPlanCandidate, AnalyzedProofPlanProjection>,
    pub required_external_evidence: BTreeSet<ExternalEvidenceRequirement>,
}

impl AnalyzedProofPlan {
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> AnalyzedProofPlanProjection {
        AnalyzedProofPlanProjection {
            relation_requirements: self.relation_requirements.clone(),
            operations: self.operations.clone(),
            lifecycle: self.lifecycle.clone(),
        }
    }
}

impl ScopedAnalyzedProgram {
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> ScopedAnalyzedProgramProjection {
        ScopedAnalyzedProgramProjection {
            source: self.source.clone(),
            foundation: self.foundation.clone(),
            architecture_scope: self.architecture_scope.clone(),
            proof_plans: self
                .proof_plans
                .iter()
                .map(|(plan, analyzed)| (plan.clone(), analyzed.project()))
                .collect(),
            required_external_evidence: self.required_external_evidence.clone(),
        }
    }
}

/// Analyze one bound compiler input completely (Guide-7 §17).
///
/// The thirteen orchestration steps run in order and no stage failure
/// becomes a partial result: a complexity failure, a closure mismatch,
/// or an invalid factor returns a typed error and no analyzed program
/// at all. The canonical analysis uses
/// [`CapabilityView::Unconstrained`], so the retained plan set is every
/// realization-approved plan rather than the subset some unnamed target
/// happens to support.
///
/// # Errors
///
/// Any failure of [`revalidate_input`], [`analyze_foundation`],
/// [`build_constructibility_analysis`], [`build_lifecycle_analysis`],
/// [`enumerate_feasible_plans`], [`relation_requirements`],
/// [`analyze_candidate_operations_reported`], or
/// [`validate_scoped_analyzed_program`];
/// [`CompileError::DuplicateAnalyzedProofPlan`] when the exact search
/// offers one typed plan twice.
// The crate-private root of the whole analysis. Guide-7 §26.1 keeps
// the complete analyzed program unexposed until a target package is
// the first honest consumer, so in a non-test build this entry point
// has no caller — everything it reaches does, which is why the
// allowance belongs here and nowhere below it.
#[allow(dead_code)]
pub fn analyze_scoped_program(
    input: &BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<ScopedAnalyzedProgram, CompileError> {
    // 1: the bound input is re-checked rather than trusted.
    revalidate_input(input)?;

    // 2 to 4: the foundation, and the two analyses the relation-indexed
    // requirements are derived against.
    let foundation = analyze_foundation(input)?;
    let constructibility = build_constructibility_analysis(input)?;
    let lifecycle = build_lifecycle_analysis(input, &foundation.relations)?;

    // 5: the exact unconstrained feasible plan set.
    let feasible = enumerate_feasible_plans(input, &CapabilityView::Unconstrained)?;

    let mut proof_plans = BTreeMap::new();
    let mut placement_search: BTreeMap<OperationId, PlacementSearchReport> = BTreeMap::new();

    for candidate in &feasible.candidates {
        // 6: relation-indexed requirements, closed against the
        // candidate's own aggregates.
        let requirements = relation_requirements(
            input,
            &foundation.relations,
            &constructibility,
            &lifecycle,
            candidate,
        )?;

        // 7: one validated factor per operation — a sum over
        // operations, never a product.
        let (operations, reports) = analyze_candidate_operations_reported(
            &foundation.relations,
            &requirements,
            candidate,
            placement_limits,
        )?;

        for (operation, report) in reports {
            let entry = placement_search.entry(operation).or_default();
            *entry = accumulate(*entry, report);
        }

        // 9: this plan's lifecycle completeness.
        let analyzed = AnalyzedProofPlan {
            proof_plan: candidate.clone(),
            lifecycle: lifecycle_completeness(candidate),
            relation_requirements: requirements,
            operations,
        };

        if proof_plans.insert(candidate.clone(), analyzed).is_some() {
            return Err(CompileError::DuplicateAnalyzedProofPlan);
        }
    }

    // 8: how much of the architecture this scope covers.
    let architecture_scope = architecture_scope_status(input);

    // 10: the external evidence every plan and relation requires.
    let required_external_evidence = evidence_union(&proof_plans);

    // 11: construct.
    let program = ScopedAnalyzedProgram {
        source: AnalyzedSource {
            architecture: input.architecture_binding().clone(),
            realization: input.realization().project(),
            compilation_scope: input.scope().clone(),
        },
        foundation: foundation.project(),
        architecture_scope,
        proof_plans,
        required_external_evidence,
        execution_report: AnalysisExecutionReport {
            proof_search_limits: input.policy().proof_search_limits,
            placement_search_limits: placement_limits,
            proof_search: feasible.search,
            operation_placement_search: placement_search,
        },
    };

    // 12: the assembled value is validated, not trusted because this
    // function produced it.
    //
    // The complete validator, not the narrow closure check. The narrow
    // check tests architecture-scope status and evidence closure; the
    // complete one independently re-derives source, foundation, proof
    // plans, relation requirements, operation factors, placements,
    // coverage, lifecycle, sponsor opacity, and the execution report,
    // and it delegates to the narrow check itself. Running only the
    // narrow check here meant the production constructor did not
    // perform the corruption-resistant assembly validation this package
    // documents — a defect in the joins between component stages would
    // be caught by whichever test called the full validator, and not by
    // the analysis entry point.
    validate_scoped_analyzed_program(input, placement_limits, &program)?;

    // 13.
    Ok(program)
}

/// Re-check the bound input's own invariants (§17 step 1).
///
/// The realization was validated by its owner when the input was bound
/// and the bound value is immutable, so this stage re-checks what the
/// analyzed program is about to claim: that the scope is canonical and
/// nonempty, that every scope operation is declared by the realization,
/// and that the retained architecture identity is the one the
/// realization projects.
///
/// # Errors
///
/// [`CompileError::EmptyCompilationScope`] for an empty scope;
/// [`CompileError::DuplicateScopeOperation`] for a repeated member;
/// [`CompileError::IncompleteRealizationScope`] when the scope names an
/// operation the realization does not declare;
/// [`CompileError::ArchitectureBindingMismatch`] when the projected
/// realization binds another architecture identity.
pub fn revalidate_input(input: &BoundCompilerInput) -> Result<(), CompileError> {
    let operations = input.scope().operations();

    if operations.is_empty() {
        return Err(CompileError::EmptyCompilationScope);
    }

    if let Some(duplicate) = operations
        .windows(2)
        .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
    {
        return Err(CompileError::DuplicateScopeOperation {
            operation: duplicate,
        });
    }

    for operation in operations {
        if input.realization().operation(*operation).is_none() {
            return Err(CompileError::IncompleteRealizationScope {
                operation: *operation,
            });
        }
    }

    if &input.realization().project().architecture != input.architecture_binding() {
        return Err(CompileError::ArchitectureBindingMismatch);
    }

    Ok(())
}

/// Derive the architecture-scope status (§5.3, §17 step 8).
///
/// The remainder of the validated architecture census after the
/// compiler scope is removed. An empty remainder is
/// [`ArchitectureScopeStatus::Complete`]; anything else names exactly
/// what is not analyzed.
#[must_use]
pub fn architecture_scope_status(input: &BoundCompilerInput) -> ArchitectureScopeStatus {
    let missing = missing_architecture_operations(input);

    if missing.is_empty() {
        ArchitectureScopeStatus::Complete
    } else {
        ArchitectureScopeStatus::Partial { missing }
    }
}

/// The architecture operations outside the compiler scope.
fn missing_architecture_operations(input: &BoundCompilerInput) -> BTreeSet<OperationId> {
    let scope = input
        .scope()
        .operations()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    input
        .architecture_operations()
        .difference(&scope)
        .copied()
        .collect()
}

/// Derive one plan's lifecycle completeness (§5.4, §17 step 9).
///
/// Every requirement whose exit is declared outside the compiler scope
/// is retained as an explicit obligation. The candidate's own lifecycle
/// rows are the source: the requirement stage already proved they are
/// exactly the rows the plan's selected representations imply.
#[must_use]
pub fn lifecycle_completeness(candidate: &ProofPlanCandidate) -> LifecycleCompleteness {
    let obligations = candidate
        .lifecycle
        .iter()
        .filter(|requirement| {
            requirement.status == LifecycleExitStatus::DeclaredOutsideCompilerScope
        })
        .cloned()
        .collect::<BTreeSet<_>>();

    if obligations.is_empty() {
        LifecycleCompleteness::Complete
    } else {
        LifecycleCompleteness::Incomplete { obligations }
    }
}

/// The external evidence every plan and relation requires (§11.5).
fn evidence_union(
    plans: &BTreeMap<ProofPlanCandidate, AnalyzedProofPlan>,
) -> BTreeSet<ExternalEvidenceRequirement> {
    plans.values().flat_map(plan_evidence).collect()
}

/// The external evidence one plan's relations require.
fn plan_evidence(plan: &AnalyzedProofPlan) -> BTreeSet<ExternalEvidenceRequirement> {
    plan.relation_requirements
        .values()
        .flat_map(|bundle| bundle.external_evidence.iter().cloned())
        .collect()
}

/// Validate the closures assembly itself asserts (§5.3, §11.5).
///
/// Deliberately narrow. The complete assembly validator re-derives
/// every component census independently; this stage checks only the two
/// claims the constructor makes that no component stage can check for
/// it — that the stated architecture-scope status is the one the
/// architecture census and the scope imply, and that the top-level
/// evidence set is exactly the union over plans and over the relations
/// within them.
///
/// Both directions of the evidence equality are defects. A missing
/// requirement understates what a later boundary must supply; an
/// unowned one asserts an obligation no relation introduced.
///
/// # Errors
///
/// [`CompileError::AnalyzedArchitectureScopeStatusMismatch`] when the
/// stated status is not the derived one;
/// [`CompileError::AnalyzedEvidenceClosureMismatch`] on the first
/// evidence difference in canonical order.
pub fn validate_assembly_closure(
    input: &BoundCompilerInput,
    program: &ScopedAnalyzedProgram,
) -> Result<(), CompileError> {
    let expected = missing_architecture_operations(input);
    let stated = match &program.architecture_scope {
        ArchitectureScopeStatus::Complete => BTreeSet::new(),
        ArchitectureScopeStatus::Partial { missing } => missing.clone(),
    };

    if stated != expected {
        return Err(CompileError::AnalyzedArchitectureScopeStatusMismatch {
            missing: expected.difference(&stated).copied().collect(),
            unexpected: stated.difference(&expected).copied().collect(),
        });
    }

    for plan in program.proof_plans.values() {
        let owned = plan_evidence(plan);

        // The relation-owned union and the plan's own aggregate agree,
        // and both are contained in the top-level set.
        if let Some(requirement) = owned.difference(&plan.proof_plan.external_evidence).next() {
            return Err(CompileError::AnalyzedEvidenceClosureMismatch {
                relation: evidence_owner(plan, requirement),
                requirement: requirement.clone(),
            });
        }

        if let Some(requirement) = plan.proof_plan.external_evidence.difference(&owned).next() {
            return Err(CompileError::AnalyzedEvidenceClosureMismatch {
                relation: None,
                requirement: requirement.clone(),
            });
        }

        if let Some(requirement) = owned.difference(&program.required_external_evidence).next() {
            return Err(CompileError::AnalyzedEvidenceClosureMismatch {
                relation: evidence_owner(plan, requirement),
                requirement: requirement.clone(),
            });
        }
    }

    let union = evidence_union(&program.proof_plans);

    if let Some(requirement) = program.required_external_evidence.difference(&union).next() {
        return Err(CompileError::AnalyzedEvidenceClosureMismatch {
            relation: None,
            requirement: requirement.clone(),
        });
    }

    Ok(())
}

/// The first relation of one plan owning an evidence requirement, in
/// canonical order.
fn evidence_owner(
    plan: &AnalyzedProofPlan,
    requirement: &ExternalEvidenceRequirement,
) -> Option<RelationId> {
    plan.relation_requirements
        .iter()
        .find(|(_, bundle)| bundle.external_evidence.contains(requirement))
        .map(|(relation, _)| relation.clone())
}

/// Accumulate one operation's placement search counts across plans.
///
/// Saturating rather than wrapping: an implausible overflow should
/// leave a diagnostic count pinned at its maximum, never wrap around to
/// a small number that reads as little work done.
pub const fn accumulate(
    total: PlacementSearchReport,
    report: PlacementSearchReport,
) -> PlacementSearchReport {
    PlacementSearchReport {
        states_visited: total.states_visited.saturating_add(report.states_visited),
        complete_assignments: total
            .complete_assignments
            .saturating_add(report.complete_assignments),
        feasible_placements: total
            .feasible_placements
            .saturating_add(report.feasible_placements),
        retained_options: total
            .retained_options
            .saturating_add(report.retained_options),
    }
}
