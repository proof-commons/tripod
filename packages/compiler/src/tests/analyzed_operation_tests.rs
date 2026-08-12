//! Factorized operation-analysis tests (Guide-7 §8, §9, §10, §21 Wave 2).
//!
//! The pilots are analyzed one operation at a time, and the factor is
//! compared against the whole-plan analysis of the same single-operation
//! scope: the two must agree exactly, because the factor is a
//! restriction of that analysis rather than a second derivation of it.
//!
//! The product theorem itself is checked on a small synthetic
//! two-operation instance, where both the global product and the
//! per-operation factors can be enumerated exhaustively and compared. The
//! real-pilot product comparison is deliberately absent here: it costs
//! roughly ninety seconds and belongs in the manual lane Wave 5 owns.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use realization::{
    CardinalityMaximum, Count, ObservedSide, ProofAlternativeId, ProofKind, Relation,
    RelationDeclaration, RelationId, RelationKind, RelationSubject, TransactionSide,
};

use super::bound_input;
use crate::{
    BoundCompilerInput, CompileError,
    analyzed_operation::{
        AnalyzedOperation, OperationPlacementAnalysis, RelationCaseDefect,
        RelationCaseRequirements, analyze_candidate_operations, analyze_operation,
        analyze_operation_placements, operation_execution_cases, validate_analyzed_operation,
        validate_operation_factorization,
    },
    capability::CapabilityView,
    carrier::relation_case_eligibility,
    case::{SponsorCase, execution_cases},
    constructibility::{CompilerConstructibilityAnalysis, build_constructibility_analysis},
    coverage::{RelationCoverageProjection, analyze_placed_coverage},
    coverage_graph::resolve_coverage_dependencies,
    disclosure::CompilerDisclosureAnalysis,
    layout::layout_requirements,
    lifecycle::{CompilerLifecycleAnalysis, build_lifecycle_analysis},
    placement::{
        DischargeBoundary, PlacementCandidate, PlacementSearchLimits, RelationActivity,
        RelationCaseKey, classify_relation_cases, enumerate_feasible_placements, place_proof_plan,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_analysis, build_relation_graph},
    requirement::{RelationRequirements, active_source_requirements, relation_requirements},
    source::derive_source_requirements,
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

struct Fixture {
    input: BoundCompilerInput,
    relations: CompilerRelationAnalysis,
    constructibility: CompilerConstructibilityAnalysis,
    lifecycle: CompilerLifecycleAnalysis,
    candidates: Vec<ProofPlanCandidate>,
}

fn fixture(operations: &[OperationId]) -> Fixture {
    let input = bound_input(operations);
    let relations = build_relation_analysis(&input).expect("relations");
    let constructibility = build_constructibility_analysis(&input).expect("constructibility");
    let lifecycle = build_lifecycle_analysis(&input, &relations).expect("lifecycle");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates;

    assert!(!candidates.is_empty());

    Fixture {
        input,
        relations,
        constructibility,
        lifecycle,
        candidates,
    }
}

impl Fixture {
    fn requirements(
        &self,
        candidate: &ProofPlanCandidate,
    ) -> BTreeMap<RelationId, RelationRequirements> {
        relation_requirements(
            &self.input,
            &self.relations,
            &self.constructibility,
            &self.lifecycle,
            candidate,
        )
        .expect("relation requirements")
    }

    fn analyze(&self, candidate: &ProofPlanCandidate, operation: OperationId) -> AnalyzedOperation {
        analyze_operation(
            &self.relations,
            &self.requirements(candidate),
            candidate,
            operation,
            limits(),
        )
        .expect("operation factor")
    }

    /// The in-scope relations of one operation.
    fn relations_of(&self, operation: OperationId) -> BTreeSet<RelationId> {
        self.relations
            .graph
            .node_weights()
            .map(|node| node.source.id.clone())
            .filter(|relation| relation.operation() == operation)
            .collect()
    }
}

/// Every disposition of one factor's relation-cases.
fn dispositions(analyzed: &AnalyzedOperation) -> BTreeSet<(RelationActivity, DischargeBoundary)> {
    analyzed
        .relation_cases
        .values()
        .flat_map(|bundle| {
            bundle
                .boundaries
                .iter()
                .map(move |boundary| (bundle.activity, *boundary))
        })
        .collect()
}

/// The bundles of one factor whose boundaries are exactly one boundary.
fn only_at(
    analyzed: &AnalyzedOperation,
    boundary: DischargeBoundary,
) -> Vec<&RelationCaseRequirements> {
    analyzed
        .relation_cases
        .values()
        .filter(|bundle| bundle.boundaries == BTreeSet::from([boundary]))
        .collect()
}

// --- pilot operation factors (§21 Wave 2) ---

/// One pilot factor, checked exactly against its own relation and case
/// censuses for every feasible plan of the operation.
fn pilot_factor_is_exact(operation: OperationId) {
    let fixture = fixture(&[operation]);
    let relations = fixture.relations_of(operation);

    assert!(!relations.is_empty());

    for candidate in &fixture.candidates {
        let analyzed = fixture.analyze(candidate, operation);
        let cases = operation_execution_cases(&fixture.relations, candidate, operation)
            .expect("operation cases");

        assert_eq!(analyzed.operation, operation);
        assert_eq!(analyzed.execution_cases.len(), cases.len());
        assert!(!analyzed.execution_cases.is_empty());

        // §9.5: every operation relation crossed with every operation
        // case, exactly once.
        let expected = relations
            .iter()
            .flat_map(|relation| {
                analyzed
                    .execution_cases
                    .iter()
                    .map(move |case| RelationCaseKey {
                        relation: relation.clone(),
                        case: case.clone(),
                    })
            })
            .collect::<BTreeSet<_>>();

        assert_eq!(
            analyzed
                .relation_cases
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected,
        );
        assert_eq!(
            analyzed.relation_cases.len(),
            relations.len() * analyzed.execution_cases.len(),
        );

        // §10.1: coverage keys are the relation-case keys.
        assert_eq!(
            analyzed
                .coverage
                .requirements
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected,
        );
        assert_eq!(analyzed.coverage.cases, analyzed.execution_cases);
        assert!(!analyzed.feasible_placements.is_empty());
        assert!(!analyzed.layout_requirements.is_empty());

        validate_analyzed_operation(&fixture.relations, candidate, &analyzed)
            .expect("factor validates");
    }
}

#[test]
fn compact_ash_operation_factor_is_exact() {
    pilot_factor_is_exact(OperationId::CompactAsh);
}

#[test]
fn live_transfer_operation_factor_is_exact() {
    pilot_factor_is_exact(OperationId::TransferLive);
}

#[test]
fn pilot_factors_carry_every_relation_case_disposition() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let fixture = fixture(&[operation]);
        let candidate = fixture.candidates.first().expect("a candidate");
        let analyzed = fixture.analyze(candidate, operation);
        let present = dispositions(&analyzed);

        // Every boundary class is represented, and the inactive
        // disposition is present rather than omitted.
        for boundary in [
            DischargeBoundary::RuntimeCarrier,
            DischargeBoundary::CompilerStatic,
            DischargeBoundary::BackendStructural,
        ] {
            assert!(
                present.contains(&(RelationActivity::Active, boundary)),
                "{operation:?} {boundary:?}",
            );
        }

        assert!(
            analyzed
                .relation_cases
                .values()
                .any(|bundle| bundle.activity == RelationActivity::Vacuous),
            "{operation:?} states no inactive relation-case",
        );
    }
}

// --- per-disposition retention (§8.2–§8.6) ---

#[test]
fn inactive_relation_cases_are_present_and_carry_nothing() {
    let operation = OperationId::CompactAsh;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let analyzed = fixture.analyze(candidate, operation);

    let inactive = analyzed
        .relation_cases
        .values()
        .filter(|bundle| bundle.activity == RelationActivity::Vacuous)
        .collect::<Vec<_>>();

    assert!(!inactive.is_empty());

    for bundle in inactive {
        // The sponsor cases are the only vacuity the pilots produce, so
        // an inactive bundle must belong to the sponsorless case.
        assert_eq!(bundle.key.case.sponsor, SponsorCase::Absent);
        assert!(bundle.active_sources.is_empty());
        assert!(bundle.carrier_assignments.is_empty());
        assert!(bundle.coverage.carrier.is_empty());
        assert!(!bundle.boundaries.is_empty());
    }
}

/// A relation over the optional sponsor family owns source rows whose
/// own activation is unconditional — the architecture bound its
/// cardinality cites is a fact of the model, not of the sponsor region.
/// Those rows pass the row-level activation test in the sponsorless
/// case while the relation itself is vacuous there, so the two
/// activation axes are genuinely independent and the bundle states the
/// relation's answer rather than the row's.
#[test]
fn an_inactive_relation_case_states_none_of_its_unconditional_rows() {
    let operation = OperationId::CompactAsh;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let requirements = fixture.requirements(candidate);
    let analyzed = fixture.analyze(candidate, operation);

    let sponsor = RelationId::new(
        operation,
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    );

    let bundle = analyzed
        .relation_cases
        .values()
        .find(|bundle| {
            bundle.key.relation == sponsor && bundle.key.case.sponsor == SponsorCase::Absent
        })
        .expect("the sponsor cardinality relation-case");

    assert_eq!(bundle.activity, RelationActivity::Vacuous);
    assert!(bundle.active_sources.is_empty());

    // The relation's own rows are not empty, and some of them pass the
    // row-level activation test in this very case: the emptiness above
    // is the relation's vacuity, never an absent row.
    let owned = &requirements[&sponsor];

    assert!(!owned.source_requirements.is_empty());
    assert!(!active_source_requirements(owned, &bundle.key.case).is_empty());
}

#[test]
fn non_runtime_relation_cases_receive_no_runtime_carrier() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let fixture = fixture(&[operation]);
        let candidate = fixture.candidates.first().expect("a candidate");
        let analyzed = fixture.analyze(candidate, operation);

        for boundary in [
            DischargeBoundary::CompilerStatic,
            DischargeBoundary::BackendStructural,
            DischargeBoundary::ExternalEvidence,
        ] {
            for bundle in only_at(&analyzed, boundary) {
                assert!(
                    bundle.carrier_assignments.is_empty(),
                    "{:?} carries a runtime carrier",
                    bundle.key,
                );
                assert!(bundle.coverage.carrier.is_empty());
            }
        }
    }
}

#[test]
fn external_evidence_relation_cases_retain_their_typed_requirement() {
    let operation = OperationId::CompactAsh;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let analyzed = fixture.analyze(candidate, operation);

    let external = only_at(&analyzed, DischargeBoundary::ExternalEvidence);

    assert!(!external.is_empty());

    for bundle in external {
        assert!(!bundle.external_evidence.is_empty());
        assert_eq!(bundle.external_evidence, bundle.coverage.external_evidence);
        assert!(bundle.carrier_assignments.is_empty());
        assert!(bundle.compiler_requirements.is_empty());
        assert!(bundle.structural_requirements.is_empty());
    }
}

#[test]
fn active_runtime_relation_cases_retain_every_alternative() {
    let operation = OperationId::TransferLive;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let analyzed = fixture.analyze(candidate, operation);

    let runtime = analyzed
        .relation_cases
        .values()
        .filter(|bundle| bundle.activity == RelationActivity::Active)
        .filter(|bundle| {
            bundle
                .boundaries
                .contains(&DischargeBoundary::RuntimeCarrier)
        })
        .collect::<Vec<_>>();

    assert!(!runtime.is_empty());

    for bundle in runtime {
        assert!(!bundle.carrier_assignments.is_empty(), "{:?}", bundle.key);

        // Every alternative's layout is stated by the bundle and by the
        // operation census.
        for alternative in &bundle.carrier_assignments {
            assert!(!alternative.carriers.is_empty());

            for requirement in &alternative.layout {
                assert!(bundle.layout_requirements.contains(requirement));
                assert!(analyzed.layout_requirements.contains(requirement));
            }
        }
    }
}

// --- factor exactness against the whole-plan analysis ---

/// One operation factor's placement, layout, and coverage must equal
/// what the whole-plan analysis of the same single-operation scope
/// produces: the factor restricts that analysis, it does not re-derive
/// it differently.
#[test]
fn operation_factors_equal_the_whole_plan_analysis() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let fixture = fixture(&[operation]);

        for candidate in &fixture.candidates {
            let analyzed = fixture.analyze(candidate, operation);
            let placed =
                place_proof_plan(&fixture.relations, candidate, limits()).expect("whole plan");

            assert_eq!(
                analyzed.feasible_placements,
                placed
                    .feasible_placements
                    .iter()
                    .map(PlacementCandidate::project)
                    .collect::<BTreeSet<_>>(),
            );
            assert_eq!(
                analyzed.layout_requirements,
                placed.layout_requirements.iter().cloned().collect(),
            );

            let mut coverage =
                analyze_placed_coverage(&fixture.relations, &placed).expect("placed coverage");
            let graph = resolve_coverage_dependencies(&mut coverage, &fixture.relations)
                .expect("coverage dependencies");

            assert_eq!(
                analyzed.coverage,
                coverage
                    .operations
                    .get(&operation)
                    .expect("operation coverage")
                    .project(),
            );
            assert_eq!(analyzed.coverage_dependencies, graph.project());
        }
    }
}

#[test]
fn operation_placements_reuse_the_exact_search_on_the_operation_census() {
    let operation = OperationId::CompactAsh;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");

    let analysis = analyze_operation_placements(&fixture.relations, candidate, operation, limits())
        .expect("operation placements");

    // The same exact search, handed the same operation-restricted
    // obligation census, independently in the test.
    let cases = execution_cases(&fixture.relations, candidate)
        .expect("cases")
        .into_iter()
        .filter(|case| case.id.operation == operation)
        .collect::<Vec<_>>();
    let plans = classify_relation_cases(&fixture.relations, &cases).expect("plans");
    let eligibility = relation_case_eligibility(&fixture.relations, &plans).expect("eligibility");
    let expected =
        enumerate_feasible_placements(&plans, &eligibility, limits()).expect("placements");

    assert_eq!(analysis.placements, expected.candidates);
    assert_eq!(
        analysis.layout_requirements,
        layout_requirements(&fixture.relations, &plans, &eligibility).expect("layout"),
    );
}

#[test]
fn repeated_operation_analysis_is_equal() {
    let operation = OperationId::TransferLive;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");

    assert_eq!(
        fixture.analyze(candidate, operation),
        fixture.analyze(candidate, operation),
    );
}

// --- multi-operation factorization (§9.3, §9.8) ---

#[test]
fn combined_scope_factors_are_operation_owned() {
    let operations = [OperationId::CompactAsh, OperationId::TransferLive];
    let combined = fixture(&operations);
    let candidate = combined.candidates.first().expect("a candidate");
    let factors = analyze_candidate_operations(
        &combined.relations,
        &combined.requirements(candidate),
        candidate,
        limits(),
    )
    .expect("operation factors");

    assert_eq!(
        factors.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from(operations),
    );

    for (operation, analyzed) in &factors {
        assert_eq!(analyzed.operation, *operation);

        for case in &analyzed.execution_cases {
            assert_eq!(case.operation, *operation);
        }

        for key in analyzed.relation_cases.keys() {
            assert_eq!(key.relation.operation(), *operation);
            assert_eq!(key.case.operation, *operation);
        }

        for key in analyzed.coverage.requirements.keys() {
            assert_eq!(key.relation.operation(), *operation);
        }

        for placement in &analyzed.feasible_placements {
            for key in placement.assignments.keys() {
                assert_eq!(key.relation.operation(), *operation);
            }
        }
    }

    // The factors of a two-operation scope are exactly the factors of
    // the two single-operation scopes: nothing about one operation
    // changes because another is in scope.
    for operation in operations {
        let alone = fixture(&[operation]);
        assert!(
            alone
                .candidates
                .iter()
                .any(|plan| alone.analyze(plan, operation) == factors[&operation]),
        );
    }
}

#[test]
fn no_operation_factor_carries_a_cross_operation_dependency() {
    let fixture = fixture(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let factors = analyze_candidate_operations(
        &fixture.relations,
        &fixture.requirements(candidate),
        candidate,
        limits(),
    )
    .expect("operation factors");

    for (operation, analyzed) in &factors {
        assert!(!analyzed.coverage_dependencies.nodes.is_empty());

        for node in &analyzed.coverage_dependencies.nodes {
            assert_eq!(node.id.operation(), *operation);
        }

        for edge in &analyzed.coverage_dependencies.edges {
            assert_eq!(edge.source.operation(), *operation);
            assert_eq!(edge.target.operation(), *operation);
        }
    }

    validate_operation_factorization(&fixture.relations, candidate, &factors)
        .expect("factorization conditions hold");
}

// --- rejected factors ---

#[test]
fn an_out_of_scope_operation_factor_is_rejected() {
    let fixture = fixture(&[OperationId::CompactAsh]);
    let candidate = fixture.candidates.first().expect("a candidate");

    let error = analyze_operation(
        &fixture.relations,
        &fixture.requirements(candidate),
        candidate,
        OperationId::TransferLive,
        limits(),
    )
    .expect_err("an operation outside the scope has no factor");

    assert_eq!(
        error,
        CompileError::AnalyzedOperationOutOfScope {
            operation: OperationId::TransferLive,
        },
    );
}

#[test]
fn an_incomplete_factor_census_is_rejected() {
    let fixture = fixture(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let mut factors = analyze_candidate_operations(
        &fixture.relations,
        &fixture.requirements(candidate),
        candidate,
        limits(),
    )
    .expect("operation factors");

    factors.remove(&OperationId::TransferLive);

    let error = validate_operation_factorization(&fixture.relations, candidate, &factors)
        .expect_err("a missing operation factor is a defect");

    assert_eq!(
        error,
        CompileError::AnalyzedOperationCensusMismatch {
            missing: vec![OperationId::TransferLive],
            unexpected: Vec::new(),
        },
    );
}

#[test]
fn a_dropped_relation_case_is_rejected() {
    let operation = OperationId::CompactAsh;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let mut analyzed = fixture.analyze(candidate, operation);
    let dropped = analyzed
        .relation_cases
        .keys()
        .next()
        .expect("a relation-case")
        .clone();

    analyzed.relation_cases.remove(&dropped);

    let error = validate_analyzed_operation(&fixture.relations, candidate, &analyzed)
        .expect_err("a dropped relation-case is a defect");

    assert_eq!(
        error,
        CompileError::RelationCaseCensusMismatch {
            missing: vec![dropped],
            unexpected: Vec::new(),
        },
    );
}

#[test]
fn a_runtime_carrier_on_a_static_relation_case_is_rejected() {
    let operation = OperationId::CompactAsh;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let mut analyzed = fixture.analyze(candidate, operation);

    // Move one active runtime relation-case's alternatives onto a
    // compiler-static one, which may never receive a runtime carrier.
    let alternatives = analyzed
        .relation_cases
        .values()
        .find(|bundle| !bundle.carrier_assignments.is_empty())
        .expect("a placed relation-case")
        .carrier_assignments
        .clone();
    let key = only_at(&analyzed, DischargeBoundary::CompilerStatic)
        .first()
        .expect("a compiler-static relation-case")
        .key
        .clone();

    let bundle = analyzed
        .relation_cases
        .get_mut(&key)
        .expect("the relation-case");

    bundle.carrier_assignments = alternatives.clone();
    bundle.coverage.carrier = alternatives;

    let error = validate_analyzed_operation(&fixture.relations, candidate, &analyzed)
        .expect_err("a static relation-case may not carry a runtime carrier");

    assert_eq!(
        error,
        CompileError::InvalidRelationCaseRequirements {
            relation: key.relation,
            case: key.case,
            defect: RelationCaseDefect::CarrierOnNonRuntimeCase,
        },
    );
}

#[test]
fn a_bundle_disagreeing_with_its_coverage_is_rejected() {
    let operation = OperationId::CompactAsh;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let mut analyzed = fixture.analyze(candidate, operation);
    let key = analyzed
        .relation_cases
        .values()
        .find(|bundle| bundle.activity == RelationActivity::Active)
        .expect("an active relation-case")
        .key
        .clone();

    analyzed
        .relation_cases
        .get_mut(&key)
        .expect("the relation-case")
        .activity = RelationActivity::Vacuous;

    let error = validate_analyzed_operation(&fixture.relations, candidate, &analyzed)
        .expect_err("a bundle may not disagree with its own coverage");

    assert_eq!(
        error,
        CompileError::InvalidRelationCaseRequirements {
            relation: key.relation,
            case: key.case,
            defect: RelationCaseDefect::ActivityMismatch,
        },
    );
}

#[test]
fn a_foreign_operation_factor_is_rejected() {
    let operations = [OperationId::CompactAsh, OperationId::TransferLive];
    let fixture = fixture(&operations);
    let candidate = fixture.candidates.first().expect("a candidate");
    let mut factors = analyze_candidate_operations(
        &fixture.relations,
        &fixture.requirements(candidate),
        candidate,
        limits(),
    )
    .expect("operation factors");

    let foreign = factors[&OperationId::TransferLive].clone();

    factors.insert(OperationId::CompactAsh, foreign);

    let error = validate_operation_factorization(&fixture.relations, candidate, &factors)
        .expect_err("a factor may not carry another operation's analysis");

    assert_eq!(
        error,
        CompileError::ForeignOperationFactor {
            operation: OperationId::CompactAsh,
            found: OperationId::TransferLive,
        },
    );
}

// --- synthetic factorization regression (§9.9) ---

fn declaration(id: RelationId, relation: Relation, proof: ProofKind) -> RelationDeclaration {
    RelationDeclaration {
        proof_alternatives: BTreeSet::from([ProofAlternativeId::new(id.clone(), proof)]),
        id,
        relation,
    }
}

fn cardinality(operation: OperationId, object: ObjectId) -> RelationDeclaration {
    declaration(
        RelationId::new(
            operation,
            RelationKind::Cardinality,
            RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                object,
            },
        ),
        Relation::Cardinality {
            side: ObservedSide::Input,
            object,
            minimum: Count::new(1),
            maximum: CardinalityMaximum::Bound(architecture::BoundId::AshBatchMax),
        },
        ProofKind::ManifestShape,
    )
}

/// A two-operation instance small enough to enumerate both ways.
///
/// Two mandatory input families per operation give each cardinality
/// obligation several admissible carriers, so the per-operation
/// placement sets are larger than one and their product is a real
/// product rather than a trivial identity.
fn synthetic_scope() -> (CompilerRelationAnalysis, ProofPlanCandidate) {
    let operations = [OperationId::CompactAsh, OperationId::TransferLive];
    let objects = [ObjectId::Ash, ObjectId::State];
    let mut nodes = Vec::new();

    for operation in operations {
        for object in objects {
            nodes.push(cardinality(operation, object));
        }
    }

    let relations =
        build_relation_graph(&operations, &nodes, &[]).expect("synthetic relation analysis");

    let mut source_requirements = Vec::new();

    for node in &nodes {
        source_requirements.extend(
            derive_source_requirements(node, ProofKind::ManifestShape).expect("source rows"),
        );
    }

    source_requirements.sort();
    source_requirements.dedup();

    let candidate = ProofPlanCandidate {
        proofs: nodes
            .iter()
            .map(|node| {
                (
                    node.id.clone(),
                    ProofAlternativeId::new(node.id.clone(), ProofKind::ManifestShape),
                )
            })
            .collect(),
        representations: BTreeMap::new(),
        required_capabilities: BTreeSet::new(),
        source_requirements,
        external_evidence: BTreeSet::new(),
        lifecycle: Vec::new(),
        disclosure: CompilerDisclosureAnalysis {
            inherited_required_public: BTreeMap::new(),
            added_required_public: BTreeMap::new(),
            retained_private: BTreeSet::new(),
        },
    };

    (relations, candidate)
}

/// The whole-scope product placement set of the synthetic instance.
fn synthetic_product(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
) -> BTreeSet<BTreeMap<RelationCaseKey, BTreeSet<crate::placement::PlacedCarrier>>> {
    place_proof_plan(relations, candidate, limits())
        .expect("whole-scope placement")
        .feasible_placements
        .iter()
        .map(|placement| placement.project().assignments)
        .collect()
}

/// The factor placement sets of the synthetic instance, one per
/// operation.
fn synthetic_factors(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
) -> Vec<OperationPlacementAnalysis> {
    [OperationId::CompactAsh, OperationId::TransferLive]
        .into_iter()
        .map(|operation| {
            analyze_operation_placements(relations, candidate, operation, limits())
                .expect("operation factor")
        })
        .collect()
}

#[test]
fn the_global_product_equals_the_operation_factor_product() {
    let (relations, candidate) = synthetic_scope();
    let global = synthetic_product(&relations, &candidate);
    let factors = synthetic_factors(&relations, &candidate);

    assert_eq!(factors.len(), 2);

    for factor in &factors {
        assert!(
            factor.placements.len() > 1,
            "a trivial factor proves nothing"
        );
    }

    // The product taken in the test, from the factors alone: every
    // combination of one placement per operation, merged by
    // relation-case key. The keys are operation-owned, so no merge can
    // collide.
    let mut product = BTreeSet::new();

    for left in &factors[0].placements {
        for right in &factors[1].placements {
            let mut merged = left.project().assignments;

            for (key, carriers) in right.project().assignments {
                assert!(merged.insert(key, carriers).is_none());
            }

            product.insert(merged);
        }
    }

    assert_eq!(global, product);
    assert_eq!(
        global.len(),
        factors[0].placements.len() * factors[1].placements.len(),
    );
}

#[test]
fn synthetic_factors_are_operation_local() {
    let (relations, candidate) = synthetic_scope();

    for factor in synthetic_factors(&relations, &candidate) {
        for plan in &factor.relation_case_plans {
            assert_eq!(plan.relation.operation(), factor.operation);
            assert_eq!(plan.case.operation, factor.operation);
        }

        for placement in &factor.placements {
            for assignment in &placement.assignments {
                assert_eq!(assignment.relation.operation(), factor.operation);
            }
        }
    }
}

/// The coverage projection of one relation-case is retained by its
/// bundle rather than recomputed beside it.
#[test]
fn relation_case_bundles_carry_their_coverage_projection() {
    let operation = OperationId::TransferLive;
    let fixture = fixture(&[operation]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let analyzed = fixture.analyze(candidate, operation);

    for (key, bundle) in &analyzed.relation_cases {
        let projection: &RelationCoverageProjection =
            analyzed.coverage.requirements.get(key).expect("coverage");

        assert_eq!(&bundle.coverage, projection);
        assert_eq!(bundle.activity, projection.activity);
        assert_eq!(bundle.boundaries, projection.boundaries);
    }
}
