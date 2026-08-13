//! Complete assembly validation of one scoped analyzed program
//! (Guide-7 §10.2, §13).
//!
//! The analyzed program is not trusted because its assembler produced
//! it. Every census, closure, and join it claims is re-derived here from
//! the bound input alone and compared for exact equality, so a value
//! that lost a relation-case, gained a placement, or changed one
//! coverage role after assembly is rejected rather than believed.
//!
//! Re-derivation runs through the production component stages. That is
//! the design rather than a shortcut: the components are independently
//! verified by their own oracles, and what no component can check for
//! itself is the *join* — that the assembled whole contains exactly
//! their results and nothing else. The independent census oracle that
//! restates those censuses without the components is a separate
//! obligation and lives elsewhere.
//!
//! Both directions of every equality are defects. Guide-7 §13.9 is
//! explicit that the validator rejects extra requirements and not only
//! missing ones, because a coverage projection that gained an obligation
//! states a demand nothing derived — and a validator that only checked
//! containment would accept it.
//!
//! The expensive re-derivation is separated from the comparison. The
//! expectations depend on the input and never on the program under test,
//! so one derivation can be checked against many candidate programs
//! without weakening anything: no corruption of a program can reach the
//! value it is compared against.

// One item-level allowance remains, on the crate-private root of the
// complete validator, for the reason given in `crate::analyzed`: §26.1
// exposes no public API that could call it until Phase 3.

use std::collections::{BTreeMap, BTreeSet};

use architecture::OperationId;
use realization::RelationId;

use crate::{
    CompileError,
    analyzed::{
        AnalyzedProofPlan, AnalyzedSource, ArchitectureScopeStatus, ScopedAnalyzedProgram,
        architecture_scope_status, lifecycle_completeness, revalidate_input,
        validate_assembly_closure,
    },
    analyzed_operation::{
        AnalyzedOperation, operation_execution_cases, validate_operation_factorization,
    },
    capability::CapabilityView,
    carrier::{CarrierEligibility, relation_case_eligibility},
    case::ExecutionCaseId,
    constructibility::build_constructibility_analysis,
    coverage::{
        CoverageBoundary, CoverageRequirementId, EvidenceRole, OperationCoverageProjection,
        RelationCoverageProjection,
    },
    coverage_graph::CoverageGraphProjection,
    foundation::{CompilerAnalysisFoundationProjection, analyze_foundation},
    input::BoundCompilerInput,
    layout::{LayoutRequirement, names_sponsor_amount},
    lifecycle::{CompilerLifecycleAnalysis, build_lifecycle_analysis},
    placement::{
        PlacementAssignment, PlacementCandidate, PlacementCandidateProjection,
        PlacementSearchLimits, RelationCaseKey, RelationCasePlan, classify_relation_cases,
        validate_placement,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::CompilerRelationAnalysis,
    requirement::{
        RelationRequirements, relation_requirements, validate_aggregate_closure,
        validate_lifecycle_closure, validate_sponsor_erasure,
    },
    source::{SourceRequirement, is_sponsor_amount_operand},
};

/// Which component of the retained typed source disagrees (§13.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalyzedSourceDefect {
    /// The bound architecture identity is not the input's.
    ArchitectureBinding,
    /// The retained realization projection is not the input's.
    RealizationProjection,
    /// The retained compilation scope is not the input's.
    CompilationScope,
}

/// Which foundation census disagrees (§13.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalyzedFoundationDefect {
    /// The relation node census is not the re-derived one.
    RelationCensus,
    /// The relation dependency census is not the re-derived one.
    RelationDependencies,
    /// The expression node census is not the re-derived one.
    ExpressionCensus,
    /// The expression dependency census is not the re-derived one.
    ExpressionDependencies,
    /// The relation census is not the retained source's own scoped
    /// relation census.
    SourceRelationCensus,
    /// The expression census is not the retained source's own scoped
    /// expression census.
    SourceExpressionCensus,
}

/// Which coverage obligation disagrees with the re-derived projection
/// (§10.2, §13.9).
///
/// Each variant names one rejection class of §10.2 rather than reporting
/// that two large values differ: a moved mutation, a changed evidence
/// role, and a rewritten collateral policy are different defects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalyzedCoverageDefect {
    /// The stored case census is not the operation's.
    CaseCensus,
    /// A re-derived relation-case has no stored coverage.
    MissingRelationCase,
    /// A stored relation-case is not re-derived.
    UnexpectedRelationCase,
    /// The stored activity is not the re-derived one.
    Activity,
    /// The stored boundaries are not the re-derived ones.
    Boundaries,
    /// A re-derived positive obligation is absent.
    MissingPositive,
    /// A stored positive obligation is not re-derived.
    UnexpectedPositive,
    /// A stored positive obligation's role, representation, or operands
    /// are not the re-derived ones.
    PositiveDetail,
    /// A re-derived negative obligation is absent.
    MissingNegative,
    /// A stored negative obligation is not re-derived.
    UnexpectedNegative,
    /// A stored negative obligation's role or mutation class is not the
    /// re-derived one.
    NegativeDetail,
    /// A stored negative obligation's collateral policy or dependency
    /// closure is not the re-derived one.
    Collateral,
    /// The stored carrier alternatives are not the re-derived ones.
    Carrier,
    /// A boundary's re-derived accepted projection is absent.
    MissingProjection,
    /// A stored accepted projection has no re-derived boundary.
    UnexpectedProjection,
    /// A stored accepted projection's subject, operands, or sources are
    /// not the re-derived ones.
    ProjectionDetail,
    /// The stored external evidence is not the re-derived set.
    ExternalEvidence,
    /// A target-execution role answers a compiler-static, structural, or
    /// external boundary.
    TargetExecutionRoleOutsideRuntime,
    /// A re-derived coverage symbol is absent from the stored graph.
    MissingDependencyNode,
    /// A stored coverage symbol is not re-derived.
    UnexpectedDependencyNode,
    /// A re-derived typed dependency is absent from the stored graph.
    MissingDependencyEdge,
    /// A stored typed dependency is not re-derived.
    UnexpectedDependencyEdge,
    /// The stored graph carries one coverage symbol more than once.
    DuplicateDependencyNode,
    /// The stored graph carries one typed dependency more than once.
    DuplicateDependencyEdge,
    /// The stored coverage symbols are not in canonical order.
    NoncanonicalDependencyNodeOrder,
    /// The stored typed dependencies are not in canonical order.
    NoncanonicalDependencyEdgeOrder,
}

/// Which exactness rule one stable projection vector broke.
///
/// Separated from the caller's own defect vocabulary so the same
/// exactness rules can be restated over any canonical vector while each
/// caller keeps naming the census it validates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanonicalCensusDefect {
    /// A re-derived member is absent from the stored vector.
    Missing,
    /// A stored member was not re-derived.
    Unexpected,
    /// The stored vector carries one member more than once.
    Duplicate,
    /// The stored vector is not in canonical ascending order.
    Order,
}

/// Validate one stable projection vector against its re-derived census.
///
/// A canonical projection vector is strictly ascending: sorted, and
/// carrying every member exactly once. Comparing the two as sets is
/// therefore not the whole check — a set erases both repetition and
/// order, so a corrupted `[n, n]` compares equal to a derived `[n]`,
/// and a duplicate paired with an omission passes while even preserving
/// the vector's length. The exact-census property this projection
/// claims has to be checked as the vector it claims to be.
fn validate_canonical_census<T: Ord>(
    stored: &[T],
    required: &[T],
) -> Result<(), CanonicalCensusDefect> {
    let carried = stored.iter().collect::<BTreeSet<_>>();
    let expected = required.iter().collect::<BTreeSet<_>>();

    // Membership first: a changed census is the more informative
    // diagnostic when both a member and its repetition are wrong.
    if expected.difference(&carried).next().is_some() {
        return Err(CanonicalCensusDefect::Missing);
    }

    if carried.difference(&expected).next().is_some() {
        return Err(CanonicalCensusDefect::Unexpected);
    }

    if stored.len() != carried.len() {
        return Err(CanonicalCensusDefect::Duplicate);
    }

    if !stored.is_sorted() {
        return Err(CanonicalCensusDefect::Order);
    }

    if stored != required {
        // Contents and repetition already agree, so only order can
        // differ — unless the re-derived census is itself noncanonical,
        // which this backstop refuses rather than accepts.
        return Err(CanonicalCensusDefect::Order);
    }

    Ok(())
}

/// Which component of one relation-case bundle disagrees with the
/// re-derived bundle (§13.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalyzedRelationCaseDefect {
    /// The stored activity or activation condition was not derived.
    Activity,
    /// The stored discharge boundaries were not derived.
    Boundaries,
    /// The stored active source rows were not derived — including a row
    /// activated through an object its own condition does not name.
    ActiveSources,
    /// The stored compiler-static or backend-structural requirements
    /// were not derived.
    StaticRequirements,
    /// The stored carrier alternatives were not derived.
    CarrierAssignments,
    /// The stored layout dependencies were not derived.
    LayoutRequirements,
    /// The stored external evidence was not derived.
    ExternalEvidence,
    /// The bundle's own copy of its coverage projection was not derived.
    Coverage,
}

/// The obligation census one operation factor's placements were searched
/// against.
///
/// Retained beside the expected factor because the exact placement
/// validator needs the relation-case plans and the eligible carrier sets
/// a placement was built from; a validator handed only the stored
/// placements could compare their shape and nothing else. Neither is a
/// search: both are derived directly from the relation analysis.
#[derive(Clone, Debug)]
pub struct OperationObligations {
    pub relation_case_plans: Vec<RelationCasePlan>,
    pub eligibility: Vec<CarrierEligibility>,
}

/// Everything one analyzed program is checked against, re-derived from
/// its bound input alone.
///
/// Nothing here is read from the program under validation, which is what
/// makes the comparison a check rather than a restatement.
#[derive(Debug)]
pub struct AnalyzedExpectations {
    pub source: AnalyzedSource,
    pub foundation: CompilerAnalysisFoundationProjection,
    pub relations: CompilerRelationAnalysis,
    pub lifecycle: CompilerLifecycleAnalysis,
    pub architecture_scope: ArchitectureScopeStatus,
    pub scope: BTreeSet<OperationId>,
    pub relation_census: BTreeSet<RelationId>,
    pub plans: BTreeMap<ProofPlanCandidate, AnalyzedProofPlan>,
    pub obligations: BTreeMap<(ProofPlanCandidate, OperationId), OperationObligations>,
}

/// Re-derive everything one analyzed program claims (§13).
///
/// The same component stages the assembler runs, run again from the
/// input. The result is a complete expected analysis, not a summary of
/// one.
///
/// # Errors
///
/// Any failure of [`revalidate_input`], [`analyze_foundation`],
/// [`build_constructibility_analysis`], [`build_lifecycle_analysis`],
/// [`enumerate_feasible_plans`], [`relation_requirements`],
/// [`crate::analyzed_operation::analyze_candidate_operations`],
/// [`operation_execution_cases`], [`classify_relation_cases`], or
/// [`relation_case_eligibility`];
/// [`CompileError::DuplicateAnalyzedProofPlan`] when the exact search
/// offers one typed plan twice.
pub fn derive_expectations(
    input: &BoundCompilerInput,
    limits: PlacementSearchLimits,
) -> Result<AnalyzedExpectations, CompileError> {
    revalidate_input(input)?;

    let foundation = analyze_foundation(input)?;
    let constructibility = build_constructibility_analysis(input)?;
    let lifecycle = build_lifecycle_analysis(input, &foundation.relations)?;
    let feasible = enumerate_feasible_plans(input, &CapabilityView::Unconstrained)?;

    let mut plans = BTreeMap::new();
    let mut obligations = BTreeMap::new();

    for candidate in &feasible.candidates {
        let requirements = relation_requirements(
            input,
            &foundation.relations,
            &constructibility,
            &lifecycle,
            candidate,
        )?;
        let operations = crate::analyzed_operation::analyze_candidate_operations(
            &foundation.relations,
            &requirements,
            candidate,
            limits,
        )?;

        for operation in operations.keys() {
            obligations.insert(
                (candidate.clone(), *operation),
                operation_obligations(&foundation.relations, candidate, *operation)?,
            );
        }

        let analyzed = AnalyzedProofPlan {
            proof_plan: candidate.clone(),
            lifecycle: lifecycle_completeness(candidate),
            relation_requirements: requirements,
            operations,
        };

        if plans.insert(candidate.clone(), analyzed).is_some() {
            return Err(CompileError::DuplicateAnalyzedProofPlan);
        }
    }

    let relation_census = foundation
        .relations
        .graph
        .node_weights()
        .map(|node| node.source.id.clone())
        .collect();

    Ok(AnalyzedExpectations {
        source: AnalyzedSource {
            architecture: input.architecture_binding().clone(),
            realization: input.realization().project(),
            compilation_scope: input.scope().clone(),
        },
        foundation: foundation.project(),
        architecture_scope: architecture_scope_status(input),
        scope: input.scope().operations().iter().copied().collect(),
        relation_census,
        relations: foundation.relations,
        lifecycle,
        plans,
        obligations,
    })
}

/// One operation's carrier obligation census, without a placement
/// search.
fn operation_obligations(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    operation: OperationId,
) -> Result<OperationObligations, CompileError> {
    let cases = operation_execution_cases(relations, candidate, operation)?;
    let relation_case_plans = classify_relation_cases(relations, &cases)?;
    let eligibility = relation_case_eligibility(relations, &relation_case_plans)?;

    Ok(OperationObligations {
        relation_case_plans,
        eligibility,
    })
}

/// Validate one complete assembled analyzed program (§13.1–§13.13).
///
/// # Errors
///
/// Any failure of [`derive_expectations`] or
/// [`validate_against_expectations`].
// The second crate-private root, for the same reason as
// [`crate::analyzed::analyze_scoped_program`]: the complete validator
// is reachable from the assembler's own step 12 and from the pilot
// acceptance analyses, and from no public API until Phase 3 gives it
// one.
#[allow(dead_code)]
pub fn validate_scoped_analyzed_program(
    input: &BoundCompilerInput,
    limits: PlacementSearchLimits,
    program: &ScopedAnalyzedProgram,
) -> Result<(), CompileError> {
    let expected = derive_expectations(input, limits)?;

    validate_against_expectations(input, &expected, program)
}

/// Validate one assembled program against an independent re-derivation
/// of its input (§13.1–§13.13, in order).
///
/// # Errors
///
/// The typed rejection of the first §13 subsection the program fails.
pub fn validate_against_expectations(
    input: &BoundCompilerInput,
    expected: &AnalyzedExpectations,
    program: &ScopedAnalyzedProgram,
) -> Result<(), CompileError> {
    revalidate_input(input)?;

    // §13.12 first, deliberately out of the section order. Sponsor
    // erasure is a structural precondition rather than a census
    // question: an assembled value that names the erased amount is
    // rejected for naming it, and diagnosing it instead as whichever
    // census the same field happens to participate in would report the
    // wrong defect.
    validate_program_sponsor_opacity(program)?;

    validate_source(expected, program)?;
    validate_foundation(expected, program)?;
    validate_proof_plan_census(expected, program)?;

    for (key, analyzed) in &program.proof_plans {
        validate_analyzed_plan(expected, key, analyzed)?;
    }

    validate_assembly_closure(input, program)
}

// --- §13.1 source validation ---

/// Validate the retained typed source and the architecture-scope status.
fn validate_source(
    expected: &AnalyzedExpectations,
    program: &ScopedAnalyzedProgram,
) -> Result<(), CompileError> {
    let defect = |defect| CompileError::AnalyzedSourceBindingMismatch { defect };

    if program.source.architecture != expected.source.architecture {
        return Err(defect(AnalyzedSourceDefect::ArchitectureBinding));
    }

    if program.source.compilation_scope != expected.source.compilation_scope {
        return Err(defect(AnalyzedSourceDefect::CompilationScope));
    }

    if program.source.realization != expected.source.realization {
        return Err(defect(AnalyzedSourceDefect::RealizationProjection));
    }

    let stated = match &program.architecture_scope {
        ArchitectureScopeStatus::Complete => BTreeSet::new(),
        ArchitectureScopeStatus::Partial { missing } => missing.clone(),
    };
    let required = match &expected.architecture_scope {
        ArchitectureScopeStatus::Complete => BTreeSet::new(),
        ArchitectureScopeStatus::Partial { missing } => missing.clone(),
    };

    if stated == required {
        return Ok(());
    }

    Err(CompileError::AnalyzedArchitectureScopeStatusMismatch {
        missing: required.difference(&stated).copied().collect(),
        unexpected: stated.difference(&required).copied().collect(),
    })
}

// --- §13.2 foundation validation ---

/// Validate the foundation censuses against a fresh analysis and against
/// the retained source's own scoped censuses.
fn validate_foundation(
    expected: &AnalyzedExpectations,
    program: &ScopedAnalyzedProgram,
) -> Result<(), CompileError> {
    let defect = |defect| CompileError::AnalyzedFoundationMismatch { defect };

    if program.foundation.relations.nodes != expected.foundation.relations.nodes {
        return Err(defect(AnalyzedFoundationDefect::RelationCensus));
    }

    if program.foundation.relations.edges != expected.foundation.relations.edges {
        return Err(defect(AnalyzedFoundationDefect::RelationDependencies));
    }

    if program.foundation.expressions.nodes != expected.foundation.expressions.nodes {
        return Err(defect(AnalyzedFoundationDefect::ExpressionCensus));
    }

    if program.foundation.expressions.edges != expected.foundation.expressions.edges {
        return Err(defect(AnalyzedFoundationDefect::ExpressionDependencies));
    }

    // The join no component stage can check for itself: the analyzed
    // foundation is the *scoped restriction* of the source the program
    // retains, so a source that lost a declaration and a foundation that
    // kept its analysis of it disagree here.
    let scope = &expected.scope;
    let declared = program
        .source
        .realization
        .relations
        .nodes
        .iter()
        .filter(|declaration| scope.contains(&declaration.id.operation()))
        .map(|declaration| declaration.id.clone())
        .collect::<BTreeSet<_>>();
    let analyzed = program
        .foundation
        .relations
        .nodes
        .iter()
        .map(|node| node.source.id.clone())
        .collect::<BTreeSet<_>>();

    if declared != analyzed {
        return Err(defect(AnalyzedFoundationDefect::SourceRelationCensus));
    }

    let expressions = program
        .source
        .realization
        .expressions
        .nodes
        .iter()
        .filter(|declaration| {
            crate::expression::expression_operation(&declaration.id)
                .is_none_or(|operation| scope.contains(&operation))
        })
        .count();

    if expressions != program.foundation.expressions.nodes.len() {
        return Err(defect(AnalyzedFoundationDefect::SourceExpressionCensus));
    }

    Ok(())
}

// --- §13.3 proof-plan census ---

/// Validate the plan map against a fresh exact feasible search.
fn validate_proof_plan_census(
    expected: &AnalyzedExpectations,
    program: &ScopedAnalyzedProgram,
) -> Result<(), CompileError> {
    let stored = program.proof_plans.keys().collect::<BTreeSet<_>>();
    let required = expected.plans.keys().collect::<BTreeSet<_>>();

    if stored != required {
        return Err(CompileError::AnalyzedProofPlanCensusMismatch {
            missing: required.difference(&stored).count(),
            unexpected: stored.difference(&required).count(),
        });
    }

    // The key is the plan identity. A retained field offering a second
    // answer would let one entry be read two ways, and the map cannot
    // hold the same typed plan twice, so this is the only shape a
    // duplicated or altered plan can take once assembly has finished.
    for (key, analyzed) in &program.proof_plans {
        if &analyzed.proof_plan != key {
            return Err(CompileError::AnalyzedProofPlanKeyMismatch);
        }
    }

    Ok(())
}

// --- per-plan validation ---

/// Validate one analyzed plan completely (§13.4–§13.10, §13.13).
fn validate_analyzed_plan(
    expected: &AnalyzedExpectations,
    key: &ProofPlanCandidate,
    analyzed: &AnalyzedProofPlan,
) -> Result<(), CompileError> {
    let required = &expected.plans[key];

    validate_relation_requirements(expected, analyzed, required)?;
    validate_operation_census(expected, analyzed)?;

    // Before the factorization conditions, because a stored product
    // member fails the per-factor placement census for a reason that
    // would misname it: the defect is that the combined product was
    // stored at all, not that one factor lost an obligation.
    for (operation, factor) in &analyzed.operations {
        reject_combined_product(*operation, factor)?;
    }

    // §13.6, §13.7, §13.13: the wave-2 factorization validator owns the
    // case, relation-case, and cross-operation conditions, and re-derives
    // each census from the relation analysis rather than from the factor.
    validate_operation_factorization(
        &expected.relations,
        &analyzed.proof_plan,
        &analyzed.operations,
    )?;

    for (operation, factor) in &analyzed.operations {
        let obligations = &expected.obligations[&(key.clone(), *operation)];
        let expected_factor = &required.operations[operation];

        validate_relation_case_closure(*operation, factor, expected_factor)?;
        validate_placement_closure(*operation, obligations, factor, expected_factor)?;
        validate_coverage_closure(*operation, &factor.coverage, &expected_factor.coverage)?;
        validate_coverage_graph(
            *operation,
            &factor.coverage_dependencies,
            &expected_factor.coverage_dependencies,
        )?;
    }

    validate_lifecycle_status(analyzed, required)
}

// --- §13.4 relation requirement census ---

/// Validate one plan's relation-indexed requirements.
fn validate_relation_requirements(
    expected: &AnalyzedExpectations,
    analyzed: &AnalyzedProofPlan,
    required: &AnalyzedProofPlan,
) -> Result<(), CompileError> {
    // Delegated to the wave-1 closures, which state the capability,
    // source, and evidence defects precisely: an item the aggregate
    // carries with no owning relation, and an owned item the aggregate
    // lost, are different failures with different diagnoses.
    validate_aggregate_closure(&analyzed.proof_plan, &analyzed.relation_requirements)?;

    // §13.4 and §13.10: the plan's lifecycle rows are exactly the rows
    // its selected representations require, each with its own exit
    // status, and every one is owned by an in-scope lifecycle relation.
    // Delegated here rather than at §13.10 so a rewritten row is
    // diagnosed as the lifecycle defect it is, before the bundle
    // comparison reports it as one more field that was not derived.
    validate_lifecycle_closure(
        &expected.lifecycle,
        &analyzed.proof_plan,
        &analyzed.relation_requirements,
    )?;

    for bundle in analyzed.relation_requirements.values() {
        validate_sponsor_erasure(bundle)?;
    }

    let stored = analyzed
        .relation_requirements
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    if stored != expected.relation_census {
        return Err(CompileError::AnalyzedRelationRequirementCensusMismatch {
            missing: expected
                .relation_census
                .difference(&stored)
                .cloned()
                .collect(),
            unexpected: stored
                .difference(&expected.relation_census)
                .cloned()
                .collect(),
        });
    }

    for (relation, bundle) in &analyzed.relation_requirements {
        validate_relation_requirement(relation, bundle, &required.relation_requirements[relation])?;
    }

    Ok(())
}

/// Compare one relation's requirement bundle field by field.
fn validate_relation_requirement(
    relation: &RelationId,
    stored: &RelationRequirements,
    required: &RelationRequirements,
) -> Result<(), CompileError> {
    if stored.relation != *relation || stored.proof != required.proof {
        return Err(CompileError::AnalyzedProofDispositionMismatch {
            relation: relation.clone(),
        });
    }

    if let Some(capability) = symmetric_difference(
        &stored.required_capabilities,
        &required.required_capabilities,
    ) {
        return Err(CompileError::AnalyzedCapabilityClosureMismatch {
            relation: Some(relation.clone()),
            capability: *capability,
        });
    }

    if let Some(row) =
        symmetric_difference(&stored.source_requirements, &required.source_requirements)
    {
        return Err(CompileError::AnalyzedSourceClosureMismatch {
            relation: Some(relation.clone()),
            operand: row.operand.clone(),
        });
    }

    if let Some(requirement) =
        symmetric_difference(&stored.external_evidence, &required.external_evidence)
    {
        return Err(CompileError::AnalyzedEvidenceClosureMismatch {
            relation: Some(relation.clone()),
            requirement: requirement.clone(),
        });
    }

    if stored.representation != required.representation || stored.lifecycle != required.lifecycle {
        return Err(CompileError::AnalyzedRelationRequirementMismatch {
            relation: relation.clone(),
        });
    }

    Ok(())
}

/// The first member of two sets' symmetric difference, in canonical
/// order.
fn symmetric_difference<'a, T: Ord>(
    left: &'a BTreeSet<T>,
    right: &'a BTreeSet<T>,
) -> Option<&'a T> {
    left.difference(right)
        .next()
        .or_else(|| right.difference(left).next())
}

// --- §13.5 operation-factor census ---

/// Validate that one plan's factors are exactly the compilation scope.
fn validate_operation_census(
    expected: &AnalyzedExpectations,
    analyzed: &AnalyzedProofPlan,
) -> Result<(), CompileError> {
    let stored = analyzed.operations.keys().copied().collect::<BTreeSet<_>>();

    if stored == expected.scope {
        return Ok(());
    }

    Err(CompileError::AnalyzedOperationCensusMismatch {
        missing: expected.scope.difference(&stored).copied().collect(),
        unexpected: stored.difference(&expected.scope).copied().collect(),
    })
}

// --- §13.7 relation-case closure ---

/// Validate one factor's relation-case bundles against the re-derived
/// bundles.
///
/// The wave-2 validator proves each bundle is internally consistent with
/// its own coverage projection; that is a different question from
/// whether the bundle is the one the relation, case, and plan imply. A
/// bundle whose active source rows were rewritten agrees with itself.
fn validate_relation_case_closure(
    operation: OperationId,
    factor: &AnalyzedOperation,
    required: &AnalyzedOperation,
) -> Result<(), CompileError> {
    for (key, bundle) in &factor.relation_cases {
        // The key census is the wave-2 validator's; a bundle with no
        // re-derived counterpart cannot reach this point.
        let Some(expected) = required.relation_cases.get(key) else {
            continue;
        };

        let defect = |defect| CompileError::AnalyzedRelationCaseMismatch {
            operation,
            relation: key.relation.clone(),
            case: key.case.clone(),
            defect,
        };

        if bundle.activity != expected.activity || bundle.activation != expected.activation {
            return Err(defect(AnalyzedRelationCaseDefect::Activity));
        }

        if bundle.boundaries != expected.boundaries {
            return Err(defect(AnalyzedRelationCaseDefect::Boundaries));
        }

        if bundle.active_sources != expected.active_sources {
            return Err(defect(AnalyzedRelationCaseDefect::ActiveSources));
        }

        if bundle.compiler_requirements != expected.compiler_requirements
            || bundle.structural_requirements != expected.structural_requirements
        {
            return Err(defect(AnalyzedRelationCaseDefect::StaticRequirements));
        }

        if bundle.carrier_assignments != expected.carrier_assignments {
            return Err(defect(AnalyzedRelationCaseDefect::CarrierAssignments));
        }

        if bundle.layout_requirements != expected.layout_requirements {
            return Err(defect(AnalyzedRelationCaseDefect::LayoutRequirements));
        }

        if bundle.external_evidence != expected.external_evidence {
            return Err(defect(AnalyzedRelationCaseDefect::ExternalEvidence));
        }

        if bundle.coverage != expected.coverage {
            return Err(defect(AnalyzedRelationCaseDefect::Coverage));
        }
    }

    Ok(())
}

// --- §13.8 carrier and placement closure ---

/// Reject a factor storing a member of the combined placement product.
///
/// A placement naming two operations' obligations is a product member
/// rather than a factor. Checked before the factorization conditions,
/// which would diagnose the same value as a factor that lost an
/// obligation.
fn reject_combined_product(
    operation: OperationId,
    factor: &AnalyzedOperation,
) -> Result<(), CompileError> {
    for placement in &factor.feasible_placements {
        let operations = placement
            .assignments
            .keys()
            .map(|key| key.relation.operation())
            .collect::<BTreeSet<_>>();

        if operations.len() > 1 {
            return Err(CompileError::UnexpectedCombinedPlacementProduct { operation });
        }
    }

    Ok(())
}

/// Validate one factor's placements and layout census exactly.
fn validate_placement_closure(
    operation: OperationId,
    obligations: &OperationObligations,
    factor: &AnalyzedOperation,
    required: &AnalyzedOperation,
) -> Result<(), CompileError> {
    for placement in &factor.feasible_placements {
        validate_placement(
            &obligations.relation_case_plans,
            &obligations.eligibility,
            &restore_placement(placement),
        )?;
    }

    if factor.feasible_placements != required.feasible_placements {
        return Err(CompileError::AnalyzedPlacementFactorMismatch {
            operation,
            missing: required
                .feasible_placements
                .difference(&factor.feasible_placements)
                .count(),
            unexpected: factor
                .feasible_placements
                .difference(&required.feasible_placements)
                .count(),
        });
    }

    if factor.layout_requirements != required.layout_requirements {
        return Err(CompileError::AnalyzedLayoutCensusMismatch { operation });
    }

    Ok(())
}

/// The stored projection read back as the value the exact placement
/// validator checks.
///
/// A projection is the canonical form of a placement rather than a
/// lossy one, so the round trip loses nothing: the assignment order and
/// carrier order the validator expects are the canonical orders the
/// projection's containers already impose.
fn restore_placement(projection: &PlacementCandidateProjection) -> PlacementCandidate {
    PlacementCandidate {
        assignments: projection
            .assignments
            .iter()
            .map(|(key, carriers)| PlacementAssignment {
                relation: key.relation.clone(),
                case: key.case.clone(),
                carriers: carriers.iter().cloned().collect(),
            })
            .collect(),
        layout_requirements: projection.layout_requirements.iter().cloned().collect(),
    }
}

// --- §13.9 and §10.2 coverage closure ---

/// Validate one factor's coverage against an independently re-derived
/// projection.
///
/// Exact equality in both directions. An extra requirement is rejected
/// for the same reason a missing one is: it states a demand no relation,
/// case, plan, placement, layout, or dependency closure produced.
fn validate_coverage_closure(
    operation: OperationId,
    stored: &OperationCoverageProjection,
    required: &OperationCoverageProjection,
) -> Result<(), CompileError> {
    let defect = |relation: Option<&RelationId>, case: Option<&ExecutionCaseId>, defect| {
        CompileError::AnalyzedCoverageCensusMismatch {
            operation,
            relation: relation.cloned(),
            case: case.cloned(),
            defect,
        }
    };

    if stored.operation != operation || stored.cases != required.cases {
        return Err(defect(None, None, AnalyzedCoverageDefect::CaseCensus));
    }

    for (key, projection) in &stored.requirements {
        validate_coverage_roles(operation, key, projection)?;
    }

    let carried = stored.requirements.keys().collect::<BTreeSet<_>>();
    let expected = required.requirements.keys().collect::<BTreeSet<_>>();

    if let Some(key) = expected.difference(&carried).next() {
        return Err(defect(
            Some(&key.relation),
            Some(&key.case),
            AnalyzedCoverageDefect::MissingRelationCase,
        ));
    }

    if let Some(key) = carried.difference(&expected).next() {
        return Err(defect(
            Some(&key.relation),
            Some(&key.case),
            AnalyzedCoverageDefect::UnexpectedRelationCase,
        ));
    }

    for (key, projection) in &stored.requirements {
        validate_relation_coverage(operation, key, projection, &required.requirements[key])?;
    }

    Ok(())
}

/// Validate the evidence roles one relation-case's coverage assigns
/// (§10.2).
///
/// A rule of its own rather than a consequence of the comparison: a
/// target-execution role at a compiler-static, backend-structural, or
/// external boundary claims that running the operation discharges an
/// obligation nobody will run, and that is a defect whether or not the
/// re-derivation happens to differ elsewhere.
fn validate_coverage_roles(
    operation: OperationId,
    key: &RelationCaseKey,
    projection: &RelationCoverageProjection,
) -> Result<(), CompileError> {
    let misplaced = |id: &CoverageRequirementId, role: &EvidenceRole| {
        *role == EvidenceRole::TargetExecution && id.boundary != CoverageBoundary::RuntimeCarrier
    };

    let found = projection
        .positive
        .iter()
        .any(|requirement| misplaced(&requirement.id, &requirement.role))
        || projection
            .negative
            .iter()
            .any(|requirement| misplaced(&requirement.id, &requirement.role));

    if !found {
        return Ok(());
    }

    Err(CompileError::AnalyzedCoverageCensusMismatch {
        operation,
        relation: Some(key.relation.clone()),
        case: Some(key.case.clone()),
        defect: AnalyzedCoverageDefect::TargetExecutionRoleOutsideRuntime,
    })
}

/// Compare one relation-case's coverage projection component by
/// component.
fn validate_relation_coverage(
    operation: OperationId,
    key: &RelationCaseKey,
    stored: &RelationCoverageProjection,
    required: &RelationCoverageProjection,
) -> Result<(), CompileError> {
    let defect = |defect| CompileError::AnalyzedCoverageCensusMismatch {
        operation,
        relation: Some(key.relation.clone()),
        case: Some(key.case.clone()),
        defect,
    };

    if stored.activity != required.activity {
        return Err(defect(AnalyzedCoverageDefect::Activity));
    }

    if stored.boundaries != required.boundaries {
        return Err(defect(AnalyzedCoverageDefect::Boundaries));
    }

    validate_positive_coverage(stored, required).map_err(defect)?;
    validate_negative_coverage(stored, required).map_err(defect)?;

    if stored.carrier != required.carrier {
        return Err(defect(AnalyzedCoverageDefect::Carrier));
    }

    validate_projection_coverage(stored, required).map_err(defect)?;

    if stored.external_evidence != required.external_evidence {
        return Err(defect(AnalyzedCoverageDefect::ExternalEvidence));
    }

    Ok(())
}

/// Compare the positive obligations, distinguishing a changed census
/// from a changed obligation.
fn validate_positive_coverage(
    stored: &RelationCoverageProjection,
    required: &RelationCoverageProjection,
) -> Result<(), AnalyzedCoverageDefect> {
    let carried = stored
        .positive
        .iter()
        .map(|requirement| &requirement.id)
        .collect::<BTreeSet<_>>();
    let expected = required
        .positive
        .iter()
        .map(|requirement| &requirement.id)
        .collect::<BTreeSet<_>>();

    if expected.difference(&carried).next().is_some() {
        return Err(AnalyzedCoverageDefect::MissingPositive);
    }

    if carried.difference(&expected).next().is_some() {
        return Err(AnalyzedCoverageDefect::UnexpectedPositive);
    }

    if stored.positive == required.positive {
        return Ok(());
    }

    Err(AnalyzedCoverageDefect::PositiveDetail)
}

/// Compare the negative obligations, distinguishing a changed census, a
/// changed obligation, and a rewritten collateral policy.
fn validate_negative_coverage(
    stored: &RelationCoverageProjection,
    required: &RelationCoverageProjection,
) -> Result<(), AnalyzedCoverageDefect> {
    let carried = stored
        .negative
        .iter()
        .map(|requirement| &requirement.id)
        .collect::<BTreeSet<_>>();
    let expected = required
        .negative
        .iter()
        .map(|requirement| &requirement.id)
        .collect::<BTreeSet<_>>();

    if expected.difference(&carried).next().is_some() {
        return Err(AnalyzedCoverageDefect::MissingNegative);
    }

    if carried.difference(&expected).next().is_some() {
        return Err(AnalyzedCoverageDefect::UnexpectedNegative);
    }

    if stored.negative == required.negative {
        return Ok(());
    }

    let collateral = |projection: &RelationCoverageProjection| {
        projection
            .negative
            .iter()
            .map(|requirement| (requirement.id.clone(), requirement.collateral.clone()))
            .collect::<BTreeMap<_, _>>()
    };

    if collateral(stored) == collateral(required) {
        return Err(AnalyzedCoverageDefect::NegativeDetail);
    }

    Err(AnalyzedCoverageDefect::Collateral)
}

/// Compare the accepted projections boundary by boundary.
fn validate_projection_coverage(
    stored: &RelationCoverageProjection,
    required: &RelationCoverageProjection,
) -> Result<(), AnalyzedCoverageDefect> {
    let carried = stored.projections.keys().collect::<BTreeSet<_>>();
    let expected = required.projections.keys().collect::<BTreeSet<_>>();

    if expected.difference(&carried).next().is_some() {
        return Err(AnalyzedCoverageDefect::MissingProjection);
    }

    if carried.difference(&expected).next().is_some() {
        return Err(AnalyzedCoverageDefect::UnexpectedProjection);
    }

    if stored.projections == required.projections {
        return Ok(());
    }

    Err(AnalyzedCoverageDefect::ProjectionDetail)
}

/// Compare one factor's typed coverage dependency graph.
fn validate_coverage_graph(
    operation: OperationId,
    stored: &CoverageGraphProjection,
    required: &CoverageGraphProjection,
) -> Result<(), CompileError> {
    let defect = |defect| CompileError::AnalyzedCoverageCensusMismatch {
        operation,
        relation: None,
        case: None,
        defect,
    };

    validate_canonical_census(&stored.nodes, &required.nodes).map_err(|census| {
        defect(match census {
            CanonicalCensusDefect::Missing => AnalyzedCoverageDefect::MissingDependencyNode,
            CanonicalCensusDefect::Unexpected => AnalyzedCoverageDefect::UnexpectedDependencyNode,
            CanonicalCensusDefect::Duplicate => AnalyzedCoverageDefect::DuplicateDependencyNode,
            CanonicalCensusDefect::Order => AnalyzedCoverageDefect::NoncanonicalDependencyNodeOrder,
        })
    })?;

    validate_canonical_census(&stored.edges, &required.edges).map_err(|census| {
        defect(match census {
            CanonicalCensusDefect::Missing => AnalyzedCoverageDefect::MissingDependencyEdge,
            CanonicalCensusDefect::Unexpected => AnalyzedCoverageDefect::UnexpectedDependencyEdge,
            CanonicalCensusDefect::Duplicate => AnalyzedCoverageDefect::DuplicateDependencyEdge,
            CanonicalCensusDefect::Order => AnalyzedCoverageDefect::NoncanonicalDependencyEdgeOrder,
        })
    })?;

    Ok(())
}

// --- §13.10 lifecycle closure ---

/// Validate one plan's lifecycle rows and its stated completeness.
fn validate_lifecycle_status(
    analyzed: &AnalyzedProofPlan,
    required: &AnalyzedProofPlan,
) -> Result<(), CompileError> {
    if analyzed.lifecycle != lifecycle_completeness(&analyzed.proof_plan)
        || analyzed.lifecycle != required.lifecycle
    {
        return Err(CompileError::AnalyzedLifecycleStatusMismatch);
    }

    Ok(())
}

// --- §13.12 sponsor opacity ---

/// Traverse the complete assembled value for any erased sponsor amount.
///
/// Sponsor family recognition, cardinality, owner authorization,
/// protocol disjointness, envelope multiplicity, and substrate
/// conservation all remain expressible. The individual amount does not —
/// not as an operand, a source row, a routed layout requirement, a
/// projected comparison, or an evidence subject.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`] on any occurrence.
pub fn validate_program_sponsor_opacity(
    program: &ScopedAnalyzedProgram,
) -> Result<(), CompileError> {
    for plan in program.proof_plans.values() {
        for bundle in plan.relation_requirements.values() {
            validate_sponsor_erasure(bundle)?;
        }

        for row in &plan.proof_plan.source_requirements {
            reject_sponsor_source(row)?;
        }

        for factor in plan.operations.values() {
            validate_factor_sponsor_opacity(factor)?;
        }
    }

    Ok(())
}

/// Traverse one operation factor for any erased sponsor amount.
fn validate_factor_sponsor_opacity(factor: &AnalyzedOperation) -> Result<(), CompileError> {
    for requirement in &factor.layout_requirements {
        reject_sponsor_layout(requirement)?;
    }

    for placement in &factor.feasible_placements {
        for requirement in &placement.layout_requirements {
            reject_sponsor_layout(requirement)?;
        }
    }

    for bundle in factor.relation_cases.values() {
        for row in &bundle.active_sources {
            reject_sponsor_source(row)?;
        }

        for requirement in &bundle.layout_requirements {
            reject_sponsor_layout(requirement)?;
        }
    }

    for projection in factor.coverage.requirements.values() {
        validate_coverage_sponsor_opacity(projection)?;
    }

    Ok(())
}

/// Traverse one relation-case's coverage projection for any erased
/// sponsor amount.
fn validate_coverage_sponsor_opacity(
    projection: &RelationCoverageProjection,
) -> Result<(), CompileError> {
    for requirement in &projection.positive {
        for operand in &requirement.operands {
            if is_sponsor_amount_operand(operand.role()) {
                return Err(CompileError::SponsorValueRead);
            }
        }
    }

    for alternative in &projection.carrier {
        for requirement in &alternative.layout {
            reject_sponsor_layout(requirement)?;
        }
    }

    for requirement in projection.projections.values() {
        for operand in &requirement.operands {
            if is_sponsor_amount_operand(operand.role()) {
                return Err(CompileError::SponsorValueRead);
            }
        }

        for row in &requirement.sources {
            reject_sponsor_source(row)?;
        }
    }

    Ok(())
}

/// Reject one source row naming the erased sponsor amount.
const fn reject_sponsor_source(row: &SourceRequirement) -> Result<(), CompileError> {
    if is_sponsor_amount_operand(row.operand.role()) {
        return Err(CompileError::SponsorValueRead);
    }

    Ok(())
}

/// Reject one layout requirement routing the erased sponsor amount.
const fn reject_sponsor_layout(requirement: &LayoutRequirement) -> Result<(), CompileError> {
    if names_sponsor_amount(requirement) {
        return Err(CompileError::SponsorValueRead);
    }

    Ok(())
}
