//! Typed coverage dependency tests (Guide-6 Tranche E).
//!
//! The pilots exercise the derivation end to end; the synthetic graphs
//! exercise the closure and the SCC policy on shapes a derived coverage
//! analysis cannot produce. The rejection paths go through the graph
//! builder directly, because a resolver that only ever sees its own
//! well-formed output proves nothing about what it rejects.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use realization::{
    RelationDependencyProjection, RelationEdge, RelationId, RelationKind, RelationSubject,
    TransactionSide,
};

use super::bound_input;
use crate::{
    CompileError,
    capability::CapabilityView,
    case::{ExecutionCaseId, SponsorCase},
    coverage::{
        CollateralPolicy, PlanCoverageAnalysis, RelationCoveragePlan, analyze_placed_coverage,
    },
    coverage_graph::{
        CoverageDefinitionCensus, CoverageDependency, CoverageDependencyGraph, CoverageEdge,
        CoverageEdgeClass, CoverageNode, CoverageNodeId, analyze_coverage_dependencies,
        build_coverage_graph, coverage_definition_census, derive_coverage_dependencies,
        resolve_coverage_dependencies,
    },
    placement::{
        PlacementSearchLimits, RelationActivity, RelationCaseKey, classify_relation_cases,
        place_feasible_proof_plans,
    },
    proof::enumerate_feasible_plans,
    relation::CompilerRelationAnalysis,
};

// --- pilot fixtures ---

/// One pilot's placed coverage across its complete feasible plan set.
struct Pilot {
    operation: OperationId,
    relations: CompilerRelationAnalysis,
    coverage: Vec<PlanCoverageAnalysis>,
}

/// Generous limits: a truncated pilot search would hide a defect
/// rather than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

fn place(operation: OperationId) -> Pilot {
    let input = bound_input(&[operation]);
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates;
    let placed = place_feasible_proof_plans(&relations, &candidates, limits())
        .expect("placement analysis")
        .placed;

    assert_ne!(
        placed,
        [] as [crate::placement::PlacedProofPlanCandidate; 0]
    );

    let coverage = placed
        .iter()
        .map(|entry| analyze_placed_coverage(&relations, entry).expect("placed coverage"))
        .collect();

    Pilot {
        operation,
        relations,
        coverage,
    }
}

fn pilots() -> [Pilot; 2] {
    [
        place(OperationId::CompactAsh),
        place(OperationId::TransferLive),
    ]
}

impl Pilot {
    fn graphs(&self) -> Vec<(&PlanCoverageAnalysis, CoverageDependencyGraph)> {
        self.coverage
            .iter()
            .map(|analysis| {
                let graph = analyze_coverage_dependencies(analysis, &self.relations)
                    .expect("coverage dependencies resolve");

                (analysis, graph)
            })
            .collect()
    }
}

/// The coverage plan of one relation-case, across a whole analysis.
fn plan_of<'a>(
    analysis: &'a PlanCoverageAnalysis,
    key: &RelationCaseKey,
) -> Option<&'a RelationCoveragePlan> {
    analysis.plans().find(|plan| &plan.key() == key)
}

// --- synthetic fixtures ---

/// A synthetic execution case of one operation.
///
/// No representation is fixed: the synthetic graphs exercise the
/// closure and the cycle policy, which no representation choice
/// changes.
fn synthetic_case(operation: OperationId) -> ExecutionCaseId {
    ExecutionCaseId {
        operation,
        sponsor: SponsorCase::Absent,
        representations: BTreeMap::new(),
    }
}

/// A synthetic relation-case, distinguished by relation kind.
fn synthetic_key(operation: OperationId, kind: RelationKind) -> RelationCaseKey {
    RelationCaseKey {
        relation: RelationId::new(operation, kind, RelationSubject::Operation),
        case: synthetic_case(operation),
    }
}

fn synthetic_node(key: &RelationCaseKey, activity: RelationActivity) -> CoverageNode {
    CoverageNode {
        id: CoverageNodeId::RelationCase(key.clone()),
        activity: Some(activity),
    }
}

fn prerequisite(source: &RelationCaseKey, target: &RelationCaseKey) -> CoverageDependency {
    CoverageDependency {
        source: CoverageNodeId::RelationCase(source.clone()),
        target: CoverageNodeId::RelationCase(target.clone()),
        edge: CoverageEdge::RelationPrerequisite,
    }
}

/// Four synthetic relation-cases of one operation.
fn synthetic_keys(operation: OperationId) -> [RelationCaseKey; 4] {
    [
        synthetic_key(operation, RelationKind::Cardinality),
        synthetic_key(operation, RelationKind::Recognition),
        synthetic_key(operation, RelationKind::Authorization),
        synthetic_key(operation, RelationKind::Conservation),
    ]
}

fn synthetic_census(nodes: &[CoverageNode]) -> CoverageDefinitionCensus {
    let mut census = CoverageDefinitionCensus::new();

    for node in nodes {
        census.define_once(node.clone()).expect("distinct symbol");
    }

    census
}

/// A synthetic graph over active relation-cases.
fn synthetic_graph(
    keys: &[RelationCaseKey],
    edges: &[CoverageDependency],
) -> Result<CoverageDependencyGraph, CompileError> {
    let nodes = keys
        .iter()
        .map(|key| synthetic_node(key, RelationActivity::Active))
        .collect::<Vec<_>>();

    build_coverage_graph(&synthetic_census(&nodes), edges)
}

// --- pilot resolution ---

#[test]
fn pilot_coverage_dependencies_resolve_and_are_acyclic() {
    for pilot in pilots() {
        for (analysis, graph) in pilot.graphs() {
            let projection = graph.project();

            assert!(
                graph.cyclic_components().is_empty(),
                "{:?}",
                pilot.operation
            );
            assert_eq!(graph.validate_acyclic(), Ok(()));

            // Every relation-case of the coverage census is a symbol,
            // including the inactive ones: a relation absent from the
            // graph would be a relation whose dependencies nobody
            // resolved.
            let symbols = projection
                .nodes
                .iter()
                .filter_map(|node| match &node.id {
                    CoverageNodeId::RelationCase(key) => Some(key.clone()),
                    _ => None,
                })
                .collect::<BTreeSet<_>>();

            assert_eq!(symbols, analysis.keys(), "{:?}", pilot.operation);
            assert!(!projection.edges.is_empty(), "{:?}", pilot.operation);

            // Every symbol kind the vocabulary defines is reached by the
            // real pilots, so no variant is carried untested.
            let carriers = projection
                .nodes
                .iter()
                .filter(|node| matches!(node.id, CoverageNodeId::Carrier { .. }))
                .count();
            let layouts = projection
                .nodes
                .iter()
                .filter(|node| matches!(node.id, CoverageNodeId::Layout(_)))
                .count();
            let evidence = projection
                .nodes
                .iter()
                .filter(|node| matches!(node.id, CoverageNodeId::ExternalEvidence(_)))
                .count();

            assert!(carriers > 0, "{:?}", pilot.operation);
            assert!(layouts > 0, "{:?}", pilot.operation);
            assert!(evidence > 0, "{:?}", pilot.operation);
        }
    }
}

#[test]
fn pilot_dependencies_stay_within_one_operation_and_one_case() {
    for pilot in pilots() {
        for (_, graph) in pilot.graphs() {
            for dependency in &graph.project().edges {
                assert_eq!(
                    dependency.source.operation(),
                    dependency.target.operation(),
                    "{:?}",
                    pilot.operation
                );

                if dependency.edge != CoverageEdge::RelationPrerequisite {
                    continue;
                }

                let source = dependency.source.relation_case().expect("relation-case");
                let target = dependency.target.relation_case().expect("relation-case");

                assert_eq!(source.case, target.case, "{:?}", pilot.operation);
            }
        }
    }
}

#[test]
fn repeated_dependency_analysis_is_equal() {
    for pilot in pilots() {
        for analysis in &pilot.coverage {
            let first = analyze_coverage_dependencies(analysis, &pilot.relations)
                .expect("first analysis")
                .project();
            let second = analyze_coverage_dependencies(analysis, &pilot.relations)
                .expect("second analysis")
                .project();

            assert_eq!(first, second, "{:?}", pilot.operation);
        }
    }
}

#[test]
fn a_permuted_insertion_order_projects_equally() {
    for pilot in pilots() {
        for analysis in &pilot.coverage {
            let census = coverage_definition_census(analysis).expect("census");
            let edges = derive_coverage_dependencies(
                analysis,
                &census,
                &pilot.source_relation_dependencies(),
            )
            .expect("dependencies");
            let mut permuted = edges.clone();
            permuted.reverse();

            let canonical = build_coverage_graph(&census, &edges).expect("canonical graph");
            let reversed = build_coverage_graph(&census, &permuted).expect("permuted graph");

            assert_eq!(
                canonical.project(),
                reversed.project(),
                "{:?}",
                pilot.operation
            );
        }
    }
}

impl Pilot {
    /// The pilot's realization relation dependency edges.
    fn source_relation_dependencies(&self) -> Vec<RelationDependencyProjection> {
        self.relations
            .project()
            .edges
            .into_iter()
            .map(|edge| {
                let crate::relation::AnalysisNodeId::SourceRelation(source) = edge.source;
                let crate::relation::AnalysisNodeId::SourceRelation(target) = edge.target;
                let crate::relation::CompilerRelationEdge::SourceDependency(kind) = edge.edge;

                RelationDependencyProjection {
                    source,
                    target,
                    edge: kind,
                }
            })
            .collect()
    }
}

// --- real dependency collateral ---

#[test]
fn sponsor_recognition_blocks_sponsor_cardinality() {
    for pilot in pilots() {
        let recognition = RelationId::new(
            pilot.operation,
            RelationKind::Recognition,
            RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        );
        let cardinality = RelationId::new(
            pilot.operation,
            RelationKind::Cardinality,
            RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        );
        let mut sponsored = 0_usize;

        for (analysis, graph) in pilot.graphs() {
            for plan in analysis.plans() {
                if plan.relation != recognition || plan.activity != RelationActivity::Active {
                    continue;
                }

                sponsored += 1;

                let collateral = graph.dependency_collateral(&plan.key());
                let dependent = RelationCaseKey {
                    relation: cardinality.clone(),
                    case: plan.case.clone(),
                };

                assert!(
                    collateral.contains(&dependent),
                    "{:?}: sponsor recognition must block sponsor cardinality",
                    pilot.operation,
                );

                // Strict: the intended relation is required separately
                // and is never part of its own collateral.
                assert!(!collateral.contains(&plan.key()));

                for blocked in &collateral {
                    assert_eq!(blocked.case, plan.case);
                    assert_eq!(
                        plan_of(analysis, blocked).map(|entry| entry.activity),
                        Some(RelationActivity::Active),
                    );
                }
            }
        }

        assert!(sponsored > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn an_inactive_sponsor_relation_claims_no_collateral() {
    for pilot in pilots() {
        let mut inactive = 0_usize;

        for (analysis, graph) in pilot.graphs() {
            for plan in analysis.plans() {
                if plan.activity != RelationActivity::Vacuous {
                    continue;
                }

                inactive += 1;

                for blocked in graph.dependency_collateral(&plan.key()) {
                    assert_eq!(
                        plan_of(analysis, &blocked).map(|entry| entry.activity),
                        Some(RelationActivity::Active),
                        "{:?}: an inactive dependent is never claimed blocked",
                        pilot.operation,
                    );
                }
            }
        }

        assert!(inactive > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn bound_collateral_is_the_graph_closure() {
    for pilot in pilots() {
        let mut filled = 0_usize;

        for analysis in &pilot.coverage {
            let mut bound = analysis.clone();
            let graph =
                resolve_coverage_dependencies(&mut bound, &pilot.relations).expect("resolution");

            for plan in bound.plans() {
                let expected = graph.dependency_collateral(&plan.key());

                for requirement in &plan.negative {
                    match requirement.collateral.policy {
                        CollateralPolicy::RequireIntendedAndDependencyClosure => {
                            assert_eq!(
                                requirement.collateral.dependency_closure, expected,
                                "{:?}",
                                pilot.operation,
                            );

                            if !expected.is_empty() {
                                filled += 1;
                            }
                        }

                        // Nothing here proves a non-runtime negative is
                        // isolated, so it claims no closure and reports
                        // additional collateral instead.
                        CollateralPolicy::ReportAdditional => assert!(
                            requirement.collateral.dependency_closure.is_empty(),
                            "{:?}",
                            pilot.operation,
                        ),
                    }
                }
            }

            // Binding changes no requirement identity, so resolving the
            // bound analysis again produces the same graph.
            let again =
                analyze_coverage_dependencies(&bound, &pilot.relations).expect("second resolution");

            assert_eq!(again.project(), graph.project(), "{:?}", pilot.operation);
        }

        assert!(filled > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn a_dependency_closure_is_strict_and_transitive() {
    for pilot in pilots() {
        for (analysis, graph) in pilot.graphs() {
            for plan in analysis.plans() {
                let key = plan.key();
                let collateral = graph.dependency_collateral(&key);

                // Strict: the intended relation is required in its own
                // right and is never counted as its own collateral.
                assert!(!collateral.contains(&key), "{:?}", pilot.operation);

                // Transitive: whatever a dependent blocks, the
                // prerequisite blocks too, and a dependent never blocks
                // itself back into the set.
                for blocked in &collateral {
                    let onward = graph.dependency_collateral(blocked);

                    assert!(onward.is_subset(&collateral), "{:?}", pilot.operation);
                    assert!(!onward.contains(blocked), "{:?}", pilot.operation);
                }
            }
        }
    }
}

// --- synthetic closures ---

#[test]
fn a_dependency_chain_closes_transitively() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edges = vec![
        prerequisite(&keys[0], &keys[1]),
        prerequisite(&keys[1], &keys[2]),
        prerequisite(&keys[2], &keys[3]),
    ];
    let graph = synthetic_graph(&keys, &edges).expect("chain");

    assert_eq!(graph.validate_acyclic(), Ok(()));
    assert_eq!(
        graph.dependency_collateral(&keys[0]),
        BTreeSet::from([keys[1].clone(), keys[2].clone(), keys[3].clone()]),
    );
    assert_eq!(
        graph.dependency_collateral(&keys[1]),
        BTreeSet::from([keys[2].clone(), keys[3].clone()]),
    );
    assert_eq!(graph.dependency_collateral(&keys[3]), BTreeSet::new());
}

#[test]
fn a_dependency_diamond_closes_once_per_relation() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edges = vec![
        prerequisite(&keys[0], &keys[1]),
        prerequisite(&keys[0], &keys[2]),
        prerequisite(&keys[1], &keys[3]),
        prerequisite(&keys[2], &keys[3]),
    ];
    let graph = synthetic_graph(&keys, &edges).expect("diamond");

    assert_eq!(graph.validate_acyclic(), Ok(()));
    assert_eq!(
        graph.dependency_collateral(&keys[0]),
        BTreeSet::from([keys[1].clone(), keys[2].clone(), keys[3].clone()]),
    );
    assert_eq!(
        graph.dependency_collateral(&keys[1]),
        BTreeSet::from([keys[3].clone()]),
    );
}

#[test]
fn an_inactive_dependent_is_traversed_but_never_claimed() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let nodes = vec![
        synthetic_node(&keys[0], RelationActivity::Active),
        synthetic_node(&keys[1], RelationActivity::Vacuous),
        synthetic_node(&keys[2], RelationActivity::Active),
        synthetic_node(&keys[3], RelationActivity::Active),
    ];
    let edges = vec![
        prerequisite(&keys[0], &keys[1]),
        prerequisite(&keys[1], &keys[2]),
        prerequisite(&keys[0], &keys[3]),
    ];
    let graph = build_coverage_graph(&synthetic_census(&nodes), &edges).expect("graph");

    assert_eq!(
        graph.dependency_collateral(&keys[0]),
        BTreeSet::from([keys[2].clone(), keys[3].clone()]),
    );
}

#[test]
fn an_absent_relation_case_claims_no_collateral() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let graph = synthetic_graph(&keys[..2], &[]).expect("graph");

    assert_eq!(graph.dependency_collateral(&keys[3]), BTreeSet::new());
}

// --- SCC policy ---

#[test]
fn a_self_loop_is_rejected() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edges = vec![prerequisite(&keys[0], &keys[0])];
    let graph = synthetic_graph(&keys, &edges).expect("graph");
    let components = graph.cyclic_components();

    assert_eq!(components.len(), 1);
    assert_eq!(
        components[0].members,
        vec![CoverageNodeId::RelationCase(keys[0].clone())],
    );
    assert_eq!(components[0].internal_edges, edges);
    assert_eq!(
        graph.validate_acyclic(),
        Err(CompileError::CoverageDependencyCycle { components }),
    );
}

#[test]
fn a_multi_node_cycle_is_rejected_with_canonical_diagnostics() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let forward = prerequisite(&keys[0], &keys[1]);
    let back = prerequisite(&keys[1], &keys[0]);

    // The declaration order is permuted; the diagnostic is not.
    let first = synthetic_graph(&keys, &[forward.clone(), back.clone()]).expect("graph");
    let second = synthetic_graph(&keys, &[back.clone(), forward.clone()]).expect("graph");
    let components = first.cyclic_components();

    assert_eq!(components, second.cyclic_components());
    assert_eq!(components.len(), 1);

    let mut members = vec![
        CoverageNodeId::RelationCase(keys[0].clone()),
        CoverageNodeId::RelationCase(keys[1].clone()),
    ];
    members.sort();

    let mut internal_edges = vec![forward, back];
    internal_edges.sort();

    assert_eq!(components[0].members, members);
    assert_eq!(components[0].internal_edges, internal_edges);
    assert_eq!(
        first.validate_acyclic(),
        Err(CompileError::CoverageDependencyCycle { components }),
    );
}

#[test]
fn an_acyclic_graph_has_no_component() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let graph = synthetic_graph(&keys, &[prerequisite(&keys[0], &keys[1])]).expect("graph");

    assert_eq!(
        graph.cyclic_components(),
        [] as [crate::coverage_graph::CoverageCycleComponent; 0]
    );
    assert_eq!(graph.validate_acyclic(), Ok(()));
}

// --- typed resolution failures ---

#[test]
fn an_unknown_reference_is_rejected() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let nodes = vec![synthetic_node(&keys[0], RelationActivity::Active)];
    let census = synthetic_census(&nodes);
    let edges = vec![prerequisite(&keys[0], &keys[1])];

    assert_eq!(
        build_coverage_graph(&census, &edges).map(|_| ()),
        Err(CompileError::UnknownCoverageSymbol {
            symbol: CoverageNodeId::RelationCase(keys[1].clone()),
        }),
    );
}

#[test]
fn a_duplicate_dependency_is_rejected() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edge = prerequisite(&keys[0], &keys[1]);

    assert_eq!(
        synthetic_graph(&keys, &[edge.clone(), edge.clone()]).map(|_| ()),
        Err(CompileError::DuplicateCoverageDependency {
            prerequisite: Box::new(edge.source),
            dependent: Box::new(edge.target),
            edge: edge.edge,
        }),
    );
}

#[test]
fn a_duplicate_symbol_is_rejected() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let node = synthetic_node(&keys[0], RelationActivity::Active);
    let mut census = CoverageDefinitionCensus::new();

    census.define_once(node.clone()).expect("first definition");
    assert!(census.define_once(node.clone()).is_err());

    // A shared symbol tolerates an exact repeat and rejects a
    // conflicting one: sharing is the meaning, disagreement is not.
    let mut shared = CoverageDefinitionCensus::new();

    shared.define(node.clone()).expect("first definition");
    shared.define(node).expect("identical definition");
    assert!(
        shared
            .define(synthetic_node(&keys[0], RelationActivity::Vacuous))
            .is_err()
    );
}

#[test]
fn a_duplicate_requirement_symbol_is_rejected_in_the_census() {
    let pilot = place(OperationId::CompactAsh);
    let mut analysis = pilot.coverage[0].clone();
    let operation = analysis
        .operations
        .get_mut(&OperationId::CompactAsh)
        .expect("operation");
    let plan = operation
        .requirements
        .values_mut()
        .find(|plan| !plan.positive.is_empty())
        .expect("a covered relation-case");
    let repeated = plan.positive[0].clone();

    plan.positive.push(repeated.clone());

    assert_eq!(
        coverage_definition_census(&analysis).map(|_| ()),
        Err(CompileError::DuplicateCoverageRequirement {
            requirement: repeated.id,
        }),
    );
}

#[test]
fn a_cross_operation_dependency_is_rejected() {
    let compact = synthetic_key(OperationId::CompactAsh, RelationKind::Cardinality);
    let live = synthetic_key(OperationId::TransferLive, RelationKind::Cardinality);
    let nodes = vec![
        synthetic_node(&compact, RelationActivity::Active),
        synthetic_node(&live, RelationActivity::Active),
    ];
    let edges = vec![prerequisite(&compact, &live)];

    assert_eq!(
        build_coverage_graph(&synthetic_census(&nodes), &edges).map(|_| ()),
        Err(CompileError::CrossOperationCoverageDependency {
            prerequisite: Box::new(CoverageNodeId::RelationCase(compact)),
            dependent: Box::new(CoverageNodeId::RelationCase(live)),
        }),
    );
}

#[test]
fn a_cross_operation_relation_edge_is_rejected_in_derivation() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let candidate = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates
        .first()
        .cloned()
        .expect("one candidate");
    let cases = crate::case::execution_cases(&relations, &candidate).expect("cases");
    let plans = classify_relation_cases(&relations, &cases).expect("classification");
    let analysis = crate::coverage::analyze_plan_coverage(&relations, &plans).expect("coverage");
    let census = coverage_definition_census(&analysis).expect("census");

    // Both operations are covered, so a dependency spanning them is a
    // claim neither operation's analysis owns.
    let crossing = RelationDependencyProjection {
        source: RelationId::new(
            OperationId::CompactAsh,
            RelationKind::Recognition,
            RelationSubject::Operation,
        ),
        target: RelationId::new(
            OperationId::TransferLive,
            RelationKind::Cardinality,
            RelationSubject::Operation,
        ),
        edge: RelationEdge::RecognitionBeforeCardinality,
    };

    let failure = derive_coverage_dependencies(&analysis, &census, &[crossing])
        .map(|_| ())
        .expect_err("a cross-operation edge is rejected");

    assert!(matches!(
        failure,
        CompileError::CrossOperationCoverageDependency { .. }
    ));

    // The same edge outside coverage scope contributes nothing rather
    // than a dangling reference.
    let outside = RelationDependencyProjection {
        source: RelationId::new(
            OperationId::CompactAsh,
            RelationKind::Recognition,
            RelationSubject::Operation,
        ),
        target: RelationId::new(
            OperationId::CompactAsh,
            RelationKind::Cardinality,
            RelationSubject::Operation,
        ),
        edge: RelationEdge::RecognitionBeforeCardinality,
    };

    let derived =
        derive_coverage_dependencies(&analysis, &census, &[outside]).expect("out-of-census edge");

    assert!(
        derived
            .iter()
            .all(|dependency| dependency.edge != CoverageEdge::RelationPrerequisite),
    );
}

// --- S2-03: one settled orientation, enforced per edge ---

#[test]
fn every_derived_dependency_joins_its_declared_endpoint_classes() {
    for pilot in pilots() {
        for (_, graph) in pilot.graphs() {
            let projection = graph.project();

            assert!(!projection.edges.is_empty(), "{:?}", pilot.operation);

            for dependency in &projection.edges {
                let (source, target) = dependency.edge.endpoint_classes();

                assert_eq!(
                    (dependency.source.class(), dependency.target.class()),
                    (source, target),
                    "{:?} {dependency:?}",
                    pilot.operation,
                );
            }
        }
    }
}

#[test]
fn a_reversed_dependency_is_rejected_rather_than_stored() {
    let mut reversed_variants = BTreeSet::new();

    for pilot in pilots() {
        for analysis in &pilot.coverage {
            let census = coverage_definition_census(analysis).expect("census");
            let edges = derive_coverage_dependencies(
                analysis,
                &census,
                &pilot.source_relation_dependencies(),
            )
            .expect("dependencies");

            for dependency in &edges {
                if dependency.edge.class() == CoverageEdgeClass::Prerequisite {
                    // Both endpoints are relation-cases, so the endpoint
                    // rule cannot see the reversal; the re-derived
                    // census is what catches that one.
                    continue;
                }

                let reversed = CoverageDependency {
                    source: dependency.target.clone(),
                    target: dependency.source.clone(),
                    edge: dependency.edge,
                };

                assert_eq!(
                    build_coverage_graph(&census, std::slice::from_ref(&reversed))
                        .expect_err("a reversed dependency is rejected"),
                    CompileError::CoverageDependencyEndpointClass {
                        prerequisite: Box::new(dependency.target.clone()),
                        dependent: Box::new(dependency.source.clone()),
                        edge: dependency.edge,
                    },
                    "{:?}",
                    pilot.operation,
                );

                reversed_variants.insert(dependency.edge);
            }
        }
    }

    // The pilots must actually exercise every ownership variant, or the
    // test above passes by never reaching one.
    assert_eq!(
        reversed_variants,
        BTreeSet::from([
            CoverageEdge::CarrierEnablesRelation,
            CoverageEdge::LayoutEnablesCarrier,
            CoverageEdge::ProjectionEnablesRelation,
            CoverageEdge::EvidenceEnablesRelation,
        ]),
    );
}

#[test]
fn a_reversed_collateral_claim_is_rejected() {
    // Collateral edges are attached from the closure rather than
    // derived, so they are exercised on a synthetic census.
    let key = synthetic_key(OperationId::CompactAsh, RelationKind::Recognition);
    let requirement = CoverageNodeId::Requirement(crate::coverage::CoverageRequirementId {
        relation: key.relation.clone(),
        case: key.case.clone(),
        boundary: crate::placement::DischargeBoundary::RuntimeCarrier,
        purpose: crate::coverage::CoveragePurpose::StructuralPresence,
    });
    let relation_case = CoverageNodeId::RelationCase(key.clone());

    let census = synthetic_census(&[
        synthetic_node(&key, RelationActivity::Active),
        CoverageNode {
            id: requirement.clone(),
            activity: None,
        },
    ]);

    // The declared orientation is accepted.
    build_coverage_graph(
        &census,
        &[CoverageDependency {
            source: requirement.clone(),
            target: relation_case.clone(),
            edge: CoverageEdge::DependencyCollateral,
        }],
    )
    .expect("a claim from the requirement to the relation-case");

    // Its reversal is not.
    assert_eq!(
        build_coverage_graph(
            &census,
            &[CoverageDependency {
                source: relation_case.clone(),
                target: requirement.clone(),
                edge: CoverageEdge::DependencyCollateral,
            }],
        )
        .expect_err("a reversed claim is rejected"),
        CompileError::CoverageDependencyEndpointClass {
            prerequisite: Box::new(relation_case),
            dependent: Box::new(requirement),
            edge: CoverageEdge::DependencyCollateral,
        },
    );
}
