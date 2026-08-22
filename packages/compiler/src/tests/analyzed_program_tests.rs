//! Scoped analyzed-program assembly tests (Guide-7 §5, §6, §11, §12,
//! §17; §21 Wave 3).
//!
//! Each assertion below is checked against something derived
//! independently of the assembled value: the plan set against a fresh
//! exact search, the operation factors against the wave-2 factor
//! analysis, the missing architecture operations against the typed
//! architecture census, and the lifecycle obligations against the
//! realization's own declared exits. A program compared only with
//! itself would agree with itself.
//!
//! Nothing here claims completed evidence. The external-evidence
//! assertions state that requirements are retained and unresolved,
//! which is the only honest claim a target-independent analysis can
//! make about them.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

use architecture::OperationId;
use realization::ExternalEvidenceRequirement;

use super::{bound_input, phase1_realization, test_policy};
use crate::{
    AnalysisPolicy, BoundCompilerInput, CompilationScope, ProofSearchLimits,
    analyzed::{
        AnalyzedProofPlanProjection, ArchitectureScopeStatus, LifecycleCompleteness,
        ScopedAnalyzedProgram, analyze_scoped_program,
    },
    analyzed_operation::analyze_candidate_operations,
    bind_input,
    capability::CapabilityView,
    constructibility::build_constructibility_analysis,
    lifecycle::{LifecycleExitStatus, build_lifecycle_analysis},
    placement::PlacementSearchLimits,
    proof::enumerate_feasible_plans,
    relation::{build_relation_analysis, build_relation_graph},
    requirement::relation_requirements,
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// One scope analyzed end to end, with the input it was analyzed from.
struct Analyzed {
    input: BoundCompilerInput,
    program: ScopedAnalyzedProgram,
}

fn analyze(operations: &[OperationId]) -> Analyzed {
    let input = bound_input(operations);
    let program = analyze_scoped_program(&input, limits()).expect("scoped analyzed program");

    Analyzed { input, program }
}

// Each scope is analyzed once and shared: the assembly runs a proof
// search and one placement search per plan and operation, so a
// per-assertion analysis would pay for the whole pipeline repeatedly.
static COMPACT_ASH: LazyLock<Analyzed> = LazyLock::new(|| analyze(&[OperationId::CompactAsh]));
static TRANSFER_LIVE: LazyLock<Analyzed> = LazyLock::new(|| analyze(&[OperationId::TransferLive]));
static COMBINED: LazyLock<Analyzed> =
    LazyLock::new(|| analyze(&[OperationId::CompactAsh, OperationId::TransferLive]));

fn pilots() -> [&'static Analyzed; 2] {
    [&COMPACT_ASH, &TRANSFER_LIVE]
}

impl Analyzed {
    fn scope(&self) -> BTreeSet<OperationId> {
        self.input.scope().operations().iter().copied().collect()
    }
}

// --- complete source binding (§6.1) ---

#[test]
fn the_analyzed_source_binds_the_exact_typed_source() {
    for analyzed in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        let source = &analyzed.program.source;

        assert_eq!(&source.architecture, analyzed.input.architecture_binding());
        assert_eq!(source.realization, analyzed.input.realization().project());
        assert_eq!(&source.compilation_scope, analyzed.input.scope());

        // The binding retains the *complete* scoped realization, which
        // the compiler scope may narrow but never extend: a
        // single-operation scope analyzes a subset of the relations its
        // source declares, and the source states the whole of what it
        // was derived from rather than the part that was used.
        assert_ne!(
            source.realization.relations.nodes,
            [] as [realization::RelationDeclaration; 0]
        );
        assert!(
            analyzed.program.foundation.relations.nodes.len()
                <= source.realization.relations.nodes.len(),
        );
    }

    // The two-operation scope is the realization's own scope, so there
    // the two censuses coincide exactly.
    assert_eq!(
        COMBINED.program.foundation.relations.nodes.len(),
        COMBINED.program.source.realization.relations.nodes.len(),
    );
}

// --- exact proof-plan map (§11.1, §11.3, §11.4) ---

#[test]
fn the_proof_plan_map_is_the_exact_feasible_set() {
    for analyzed in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        // A fresh exact search, not a read-back of the analyzed keys.
        let expected = enumerate_feasible_plans(&analyzed.input, &CapabilityView::Unconstrained)
            .expect("feasible plans")
            .candidates
            .into_iter()
            .collect::<BTreeSet<_>>();

        assert!(!expected.is_empty());
        assert_eq!(
            analyzed
                .program
                .proof_plans
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected,
        );

        // The key is the plan identity: the retained field restates it
        // rather than offering a second answer.
        for (plan, analysis) in &analyzed.program.proof_plans {
            assert_eq!(&analysis.proof_plan, plan);
        }
    }
}

#[test]
fn the_plan_set_carries_every_realization_approved_representation() {
    // The unconstrained view retains both approved modes of each pilot,
    // so no plan set here is the target-pruned subset of another.
    let modes = |analyzed: &Analyzed| {
        analyzed
            .program
            .proof_plans
            .keys()
            .flat_map(|plan| plan.representations.values().copied())
            .collect::<BTreeSet<_>>()
    };

    assert_eq!(
        modes(&COMPACT_ASH),
        BTreeSet::from([
            realization::RepresentationMode::Explicit,
            realization::RepresentationMode::PublicCommitted,
        ]),
    );
    assert_eq!(
        modes(&TRANSFER_LIVE),
        BTreeSet::from([
            realization::RepresentationMode::Explicit,
            realization::RepresentationMode::PrivateCommitted,
        ]),
    );
}

// --- exact operation factors (§9.3, §11.2) ---

#[test]
fn every_plan_stores_exactly_the_scope_operation_factors() {
    for analyzed in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        let relations = build_relation_analysis(&analyzed.input).expect("relations");
        let constructibility =
            build_constructibility_analysis(&analyzed.input).expect("constructibility");
        let lifecycle =
            build_lifecycle_analysis(&analyzed.input, &relations).expect("lifecycle analysis");

        for (plan, analysis) in &analyzed.program.proof_plans {
            assert_eq!(
                analysis.operations.keys().copied().collect::<BTreeSet<_>>(),
                analyzed.scope(),
            );

            // The stored factors equal the wave-2 factor analysis of
            // the same plan, derived here independently.
            let requirements = relation_requirements(
                &analyzed.input,
                &relations,
                &constructibility,
                &lifecycle,
                plan,
            )
            .expect("relation requirements");

            assert_eq!(analysis.relation_requirements, requirements);
            assert_eq!(
                analysis.operations,
                analyze_candidate_operations(&relations, &requirements, plan, limits())
                    .expect("operation factors"),
            );

            for (operation, factor) in &analysis.operations {
                assert_eq!(factor.operation, *operation);
                assert!(!factor.feasible_placements.is_empty());
                assert!(!factor.relation_cases.is_empty());
            }
        }
    }
}

// --- partial architecture status (§5.3) ---

#[test]
fn the_pilot_scopes_report_a_partial_architecture() {
    // The expectation is derived from the typed architecture census,
    // never from a written-down remainder.
    let census = architecture::ARCHITECTURE
        .operations
        .iter()
        .map(|operation| operation.id)
        .collect::<BTreeSet<_>>();

    for analyzed in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        let expected = census
            .difference(&analyzed.scope())
            .copied()
            .collect::<BTreeSet<_>>();

        let ArchitectureScopeStatus::Partial { missing } = &analyzed.program.architecture_scope
        else {
            panic!("the pilot scopes do not cover the architecture");
        };

        assert_eq!(missing, &expected);
        assert!(missing.is_disjoint(&analyzed.scope()));
    }

    // The two-operation pilot scope leaves eleven of the thirteen
    // architecture operations unanalyzed.
    let ArchitectureScopeStatus::Partial { missing } = &COMBINED.program.architecture_scope else {
        panic!("the combined scope does not cover the architecture");
    };

    assert_eq!(missing.len(), 11);
    assert_eq!(census.len(), 13);
}

// --- lifecycle incompleteness (§5.4) ---

/// The exits one analyzed scope declares outside the compiler scope,
/// per plan.
fn outstanding_exits(analyzed: &Analyzed) -> BTreeSet<OperationId> {
    let mut exits = BTreeSet::new();

    for analysis in analyzed.program.proof_plans.values() {
        let LifecycleCompleteness::Incomplete { obligations } = &analysis.lifecycle else {
            panic!("a pilot plan claims a complete deployment lifecycle");
        };

        for obligation in obligations {
            assert_eq!(
                obligation.status,
                LifecycleExitStatus::DeclaredOutsideCompilerScope,
            );
            exits.insert(obligation.exit);
        }
    }

    exits
}

#[test]
fn compact_ash_retains_its_clear_lifecycle_obligation() {
    assert_eq!(
        outstanding_exits(&COMPACT_ASH),
        BTreeSet::from([OperationId::Clear]),
    );
}

#[test]
fn live_transfer_retains_its_burn_and_redeem_obligations() {
    assert_eq!(
        outstanding_exits(&TRANSFER_LIVE),
        BTreeSet::from([OperationId::Burn, OperationId::Redeem]),
    );
}

#[test]
fn the_combined_scope_retains_every_outstanding_exit() {
    assert_eq!(
        outstanding_exits(&COMBINED),
        BTreeSet::from([OperationId::Burn, OperationId::Clear, OperationId::Redeem]),
    );
}

// --- unresolved external evidence (§5.5, §11.5) ---

#[test]
fn required_external_evidence_is_the_union_over_plans_and_relations() {
    for analyzed in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        let mut union = BTreeSet::new();

        for analysis in analyzed.program.proof_plans.values() {
            for bundle in analysis.relation_requirements.values() {
                union.extend(bundle.external_evidence.iter().cloned());
            }
        }

        assert!(!union.is_empty());
        assert_eq!(analyzed.program.required_external_evidence, union);
    }
}

#[test]
fn the_pilots_retain_unresolved_substrate_conservation_evidence() {
    for analyzed in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        let operations = analyzed
            .program
            .required_external_evidence
            .iter()
            .filter_map(|requirement| match requirement {
                ExternalEvidenceRequirement::SubstrateConservation { operation, .. } => {
                    Some(*operation)
                }
                // A different class with a different owner: the
                // substrate obligation is one per operation, where the
                // confidential one belongs only to the operations whose
                // plans hold a protocol amount as a commitment. Folding
                // them together would make this assertion pass for the
                // wrong reason.
                ExternalEvidenceRequirement::ConfidentialValueConservation { .. } => None,
            })
            .collect::<BTreeSet<_>>();

        // One whole-transaction substrate obligation per analyzed
        // operation, retained as a requirement: the analyzed program
        // carries no verdict, report, or discharge status for any of
        // them, and the type it stores has no place to put one.
        assert_eq!(operations, analyzed.scope());
    }
}

// --- no combined placement product (§9.1, §12.3, §20.2) ---

#[test]
fn the_analyzed_program_stores_placement_factors_only() {
    let combined = &COMBINED;
    let projection = combined.program.project();

    for analysis in projection.proof_plans.values() {
        assert_eq!(analysis.operations.len(), 2);

        for (operation, factor) in &analysis.operations {
            // Every placement is one operation's: a stored combination
            // across operations would name a second operation's
            // relation-case here.
            for placement in &factor.feasible_placements {
                for key in placement.assignments.keys() {
                    assert_eq!(key.relation.operation(), *operation);
                    assert_eq!(key.case.operation, *operation);
                }

                for requirement in &placement.layout_requirements {
                    assert!(factor.layout_requirements.contains(requirement));
                }
            }
        }

        // The stored placement content is the two factors, and the
        // combined product they denote is far larger than what is
        // stored — the point of storing factors at all.
        let stored = analysis
            .operations
            .values()
            .map(|factor| factor.feasible_placements.len())
            .collect::<Vec<_>>();
        let sum = stored.iter().sum::<usize>();
        let product = stored.iter().product::<usize>();

        assert_eq!(stored.len(), 2);
        assert!(stored.iter().all(|count| *count > 1));
        assert!(sum < product);
    }
}

#[test]
fn the_stored_placements_equal_the_operation_factors() {
    let combined = &COMBINED;
    let relations = build_relation_analysis(&combined.input).expect("relations");
    let constructibility =
        build_constructibility_analysis(&combined.input).expect("constructibility");
    let lifecycle =
        build_lifecycle_analysis(&combined.input, &relations).expect("lifecycle analysis");
    let plan = combined
        .program
        .proof_plans
        .keys()
        .next()
        .expect("an analyzed plan");

    let requirements = relation_requirements(
        &combined.input,
        &relations,
        &constructibility,
        &lifecycle,
        plan,
    )
    .expect("relation requirements");
    let factors = analyze_candidate_operations(&relations, &requirements, plan, limits())
        .expect("operation factors");
    let analysis = &combined.program.proof_plans[plan];

    for (operation, factor) in &factors {
        assert_eq!(
            analysis.operations[operation].feasible_placements,
            factor.feasible_placements,
        );
        assert_eq!(
            analysis.operations[operation].layout_requirements,
            factor.layout_requirements,
        );
    }
}

// --- stable projection (§12.2, §12.3) ---

#[test]
fn the_projection_carries_the_semantic_value_without_the_execution_report() {
    let analyzed = &COMPACT_ASH;
    let projection = analyzed.program.project();

    assert_eq!(projection.source, analyzed.program.source);
    assert_eq!(projection.foundation, analyzed.program.foundation);
    assert_eq!(
        projection.architecture_scope,
        analyzed.program.architecture_scope,
    );
    assert_eq!(
        projection.required_external_evidence,
        analyzed.program.required_external_evidence,
    );
    assert_eq!(
        projection.proof_plans.keys().collect::<Vec<_>>(),
        analyzed.program.proof_plans.keys().collect::<Vec<_>>(),
    );

    // The report exists on the assembled value and has no projected
    // counterpart: the search visited states, and no count of them
    // reaches the stable value.
    assert!(
        analyzed
            .program
            .execution_report
            .proof_search
            .states_visited
            > 0
    );
    assert!(
        analyzed
            .program
            .execution_report
            .operation_placement_search
            .values()
            .all(|report| report.states_visited > 0),
    );

    for (plan, analysis) in &projection.proof_plans {
        let source = &analyzed.program.proof_plans[plan];

        assert_eq!(
            analysis,
            &AnalyzedProofPlanProjection {
                relation_requirements: source.relation_requirements.clone(),
                operations: source.operations.clone(),
                lifecycle: source.lifecycle.clone(),
            },
        );
    }
}

// --- determinism (§12.5) ---

#[test]
fn repeated_analysis_projects_equally() {
    for operations in [
        vec![OperationId::CompactAsh],
        vec![OperationId::TransferLive],
    ] {
        let input = bound_input(&operations);
        let first = analyze_scoped_program(&input, limits()).expect("first analysis");
        let second = analyze_scoped_program(&input, limits()).expect("second analysis");

        assert_eq!(first.project(), second.project(), "{operations:?}");
    }
}

#[test]
fn a_permuted_scope_order_projects_equally() {
    // The scope is canonical by construction, so the two orders are the
    // same scope: the projection may not depend on how it was written.
    let forward = analyze(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let reversed = analyze(&[OperationId::TransferLive, OperationId::CompactAsh]);

    assert_eq!(forward.program.project(), reversed.program.project());
}

#[test]
fn permuted_relation_declarations_assemble_equally() {
    // The orchestrator derives its relations from the realization, so
    // the permutation is applied to the declarations the assembly
    // stages consume and the assembled per-plan value is compared.
    for analyzed in pilots() {
        let source = analyzed.input.realization().project();
        let mut nodes = source.relations.nodes;
        let mut edges = source.relations.edges;

        nodes.reverse();
        edges.reverse();

        let relations = build_relation_graph(analyzed.input.scope().operations(), &nodes, &edges)
            .expect("permuted relation graph");
        let constructibility =
            build_constructibility_analysis(&analyzed.input).expect("constructibility");
        let lifecycle =
            build_lifecycle_analysis(&analyzed.input, &relations).expect("lifecycle analysis");

        for (plan, analysis) in &analyzed.program.proof_plans {
            let requirements = relation_requirements(
                &analyzed.input,
                &relations,
                &constructibility,
                &lifecycle,
                plan,
            )
            .expect("relation requirements");
            let operations =
                analyze_candidate_operations(&relations, &requirements, plan, limits())
                    .expect("operation factors");

            assert_eq!(
                analysis.project(),
                AnalyzedProofPlanProjection {
                    relation_requirements: requirements,
                    operations,
                    lifecycle: analysis.lifecycle.clone(),
                },
            );
        }
    }
}

// --- sufficient-limit invariance (§6.3, §12.5) ---

fn generous_input(operations: &[OperationId]) -> BoundCompilerInput {
    bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        CompilationScope::from_operations(operations.iter().copied()).expect("scope"),
        AnalysisPolicy::strict(ProofSearchLimits::new(
            std::num::NonZeroU64::new(100_000_000).expect("nonzero"),
            std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
        )),
    )
    .expect("bind input")
}

#[test]
fn larger_sufficient_limits_project_equally() {
    let operations = [OperationId::CompactAsh];
    let modest = bound_input(&operations);
    let generous = generous_input(&operations);

    // Both configurations are sufficient; only the headroom differs.
    assert_ne!(
        modest.policy().proof_search_limits,
        generous.policy().proof_search_limits,
    );

    let larger = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(100_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
    );

    let first = analyze_scoped_program(&modest, limits()).expect("modest analysis");
    let second = analyze_scoped_program(&generous, larger).expect("generous analysis");

    assert_eq!(first.project(), second.project());

    // The reports record the difference the semantic value must not.
    assert_ne!(first.execution_report, second.execution_report);
    assert_eq!(
        first.execution_report.proof_search,
        second.execution_report.proof_search,
    );
    assert_eq!(
        first.execution_report.operation_placement_search,
        second.execution_report.operation_placement_search,
    );
    assert_ne!(
        first.execution_report.placement_search_limits,
        second.execution_report.placement_search_limits,
    );
}

// --- rejected assemblies ---

#[test]
fn an_out_of_scope_realization_operation_is_rejected() {
    // A scope naming an operation the realization does not declare is
    // refused at binding, so no analyzed program can exist for it.
    let error = bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        CompilationScope::from_operations([OperationId::Burn]).expect("scope"),
        test_policy(),
    )
    .expect_err("the pilots do not declare burn");

    assert_eq!(
        error,
        crate::CompileError::IncompleteRealizationScope {
            operation: OperationId::Burn,
        },
    );
}

#[test]
fn a_corrupted_architecture_scope_status_is_rejected() {
    let analyzed = &COMPACT_ASH;
    let mut program = analyzed.program.clone();

    program.architecture_scope = ArchitectureScopeStatus::Complete;

    let error = crate::analyzed::validate_assembly_closure(&analyzed.input, &program)
        .expect_err("a partial scope may not claim architecture completeness");

    let crate::CompileError::AnalyzedArchitectureScopeStatusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("the architecture-scope status defect is not reported as one");
    };

    assert_eq!(missing.len(), 12);
    assert_eq!(unexpected, [] as [OperationId; 0]);
}

#[test]
fn a_dropped_evidence_requirement_is_rejected() {
    let analyzed = &COMPACT_ASH;
    let mut program = analyzed.program.clone();
    let dropped = program
        .required_external_evidence
        .iter()
        .next()
        .expect("an evidence requirement")
        .clone();

    program.required_external_evidence.remove(&dropped);

    let error = crate::analyzed::validate_assembly_closure(&analyzed.input, &program)
        .expect_err("the top-level union may not omit a relation's requirement");

    let crate::CompileError::AnalyzedEvidenceClosureMismatch {
        relation,
        requirement,
    } = error
    else {
        panic!("the evidence closure defect is not reported as one");
    };

    assert_eq!(requirement, dropped);
    assert!(relation.is_some());
}

#[test]
fn an_unowned_evidence_requirement_is_rejected() {
    let analyzed = &COMPACT_ASH;
    let mut program = analyzed.program.clone();

    program
        .required_external_evidence
        .insert(ExternalEvidenceRequirement::SubstrateConservation {
            operation: OperationId::Burn,
            asset: architecture::AssetId::Lbtc,
        });

    let error = crate::analyzed::validate_assembly_closure(&analyzed.input, &program)
        .expect_err("an evidence requirement no relation owns is a defect");

    assert!(matches!(
        error,
        crate::CompileError::AnalyzedEvidenceClosureMismatch { relation: None, .. },
    ));
}

/// The per-plan operation factors are the whole placement content, so
/// the analyzed program's own storage is a sum over operations rather
/// than a product — measured here on the combined scope.
#[test]
fn combined_scope_storage_is_a_sum_over_operations() {
    let mut stored = BTreeMap::new();

    for (plan, analysis) in &COMBINED.program.proof_plans {
        for (operation, factor) in &analysis.operations {
            stored.insert((plan.clone(), *operation), factor.feasible_placements.len());
        }
    }

    assert!(!stored.is_empty());
    assert!(stored.values().all(|count| *count > 1));
}

/// The combined scope's stored placement content, stated as measured
/// sizes: two factors of 216 per plan, never their 46,656-member
/// product.
#[test]
fn the_combined_scope_stores_two_factors_rather_than_their_product() {
    for analysis in COMBINED.program.proof_plans.values() {
        let factors = analysis
            .operations
            .values()
            .map(|factor| factor.feasible_placements.len())
            .collect::<Vec<_>>();

        assert_eq!(factors, vec![216, 216]);
        assert_eq!(factors.iter().sum::<usize>(), 432);
        assert_eq!(factors.iter().product::<usize>(), 46_656);
    }
}
