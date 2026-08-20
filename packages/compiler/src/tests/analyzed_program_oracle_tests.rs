//! The independent assembly, requirement-union, and factorization
//! oracles (Guide-7 §14; §21 Wave 5).
//!
//! Guide 5 and Guide 6 already verify proof planning, placement,
//! coverage, closure, and SCC behaviour independently, so nothing here
//! reimplements a proof search, a carrier availability rule, a placement
//! odometer, a coverage mutation catalogue, or a Petgraph algorithm
//! (§14.1). What no component stage can check for itself is the *join*:
//! whether the assembled whole contains exactly the censuses its typed
//! source implies. That is the question this module answers.
//!
//! The independence rule of §14.3 is the discipline every expectation
//! below obeys: no analyzed-program field is ever used to derive the
//! expectation for that same field. The architecture census is read from
//! [`architecture::ARCHITECTURE`], the relation census from the
//! realization's own declarations, the case census from the sponsor and
//! representation declarations crossed with the plan's decisions, the
//! plan set from a fresh exact feasible search — the one source §14.3
//! explicitly sanctions — and the boundary partition from a restatement
//! of the discharge matrix rather than from
//! [`crate::placement::classify_relation_cases`].
//!
//! Three production functions are deliberately never called here:
//! [`crate::analyzed_validate::derive_expectations`] and
//! [`crate::analyzed_validate::validate_scoped_analyzed_program`], which
//! re-derive through the same component stages the assembler ran, and
//! the assembler-side derivation of whichever property is under test. The
//! corruption tests of §14.6 call the production validator on purpose:
//! there the point is that *both* sides reject, since a corruption only
//! one side sees is a blind spot in the other.
//!
//! Nothing asserted here claims completed evidence. A retained external
//! evidence requirement remains a requirement, a layout requirement
//! remains an obligation on a backend that does not exist, and a
//! lifecycle obligation outside compiler scope remains outstanding.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
    time::Instant,
};

use architecture::{ObjectId, OpenFlowKind, OperationId};
use realization::{
    ExternalEvidenceRequirement, LifecycleNodeId, ProofAlternativeId, ProofKind, Relation,
    RelationDeclaration, RelationId, RepresentationMode,
};

use super::bound_input;
use crate::{
    BoundCompilerInput, CompileError,
    analyzed::{
        AnalyzedProofPlan, ArchitectureScopeStatus, LifecycleCompleteness, ScopedAnalyzedProgram,
        ScopedAnalyzedProgramProjection, analyze_scoped_program,
    },
    analyzed_operation::{
        AnalyzedOperation, analyze_operation, analyze_operation_placements,
        relation_case_requirements,
    },
    analyzed_validate::{AnalyzedExpectations, derive_expectations, validate_against_expectations},
    capability::{CapabilityView, RequiredCapability},
    case::{ExecutionCaseId, SponsorCase},
    constructibility::build_constructibility_analysis,
    coverage::{CoverageBoundary, analyze_placed_coverage},
    coverage_graph::resolve_coverage_dependencies,
    layout::LayoutRequirement,
    lifecycle::{
        LifecycleExitStatus, LifecycleRequirement, RepresentationChoiceId, build_lifecycle_analysis,
    },
    placement::{
        PlacedCarrier, PlacedProofPlanCandidate, PlacementCandidate, PlacementCandidateProjection,
        PlacementSearchLimits, RelationActivity, RelationCaseKey, enumerate_feasible_placements,
        place_feasible_proof_plans, place_proof_plan,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_analysis, build_relation_graph},
    requirement::{RelationRequirements, relation_requirements},
    source::{SourceRequirement, derive_source_requirements, proof_capabilities},
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// One scope assembled end to end, with the input it was assembled
/// from.
struct Analyzed {
    operations: Vec<OperationId>,
    input: BoundCompilerInput,
    program: ScopedAnalyzedProgram,
    projection: ScopedAnalyzedProgramProjection,
}

fn analyze(operations: &[OperationId]) -> Analyzed {
    let input = bound_input(operations);
    let program = analyze_scoped_program(&input, limits()).expect("scoped analyzed program");
    let projection = program.project();

    Analyzed {
        operations: operations.to_vec(),
        input,
        program,
        projection,
    }
}

// Each scope is assembled once and shared. The assembly runs a proof
// search and one placement search per plan and operation, so a
// per-assertion assembly would pay for the whole pipeline repeatedly.
static COMPACT_ASH: LazyLock<Analyzed> = LazyLock::new(|| analyze(&[OperationId::CompactAsh]));
static TRANSFER_LIVE: LazyLock<Analyzed> = LazyLock::new(|| analyze(&[OperationId::TransferLive]));
static COMBINED: LazyLock<Analyzed> =
    LazyLock::new(|| analyze(&[OperationId::CompactAsh, OperationId::TransferLive]));

fn scopes() -> [&'static Analyzed; 3] {
    [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED]
}

impl Analyzed {
    fn scope(&self) -> BTreeSet<OperationId> {
        self.operations.iter().copied().collect()
    }

    /// One assembled plan, by its typed key.
    fn plan(&self, key: &ProofPlanCandidate) -> &AnalyzedProofPlan {
        &self.program.proof_plans[key]
    }
}

// --- §14.2 the directly restated censuses ---

/// Every operation the validated architecture declares.
///
/// Read from the architecture value itself, so a thirteenth operation
/// becoming a fourteenth changes this expectation without anyone
/// remembering to edit a number.
fn oracle_architecture_operations() -> BTreeSet<OperationId> {
    architecture::ARCHITECTURE
        .operations
        .iter()
        .map(|operation| operation.id)
        .collect()
}

/// The relation declarations one compiler scope analyzes.
///
/// Taken from the realization's own declaration list restricted to the
/// scope, never from the analyzed program's foundation graph or from its
/// relation-requirement keys: those are two of the things being checked.
fn oracle_declarations(analyzed: &Analyzed) -> BTreeMap<RelationId, RelationDeclaration> {
    let scope = analyzed.scope();

    analyzed
        .input
        .realization()
        .project()
        .relations
        .nodes
        .into_iter()
        .filter(|declaration| scope.contains(&declaration.id.operation()))
        .map(|declaration| (declaration.id.clone(), declaration))
        .collect()
}

/// The relations one operation declares, restated.
fn oracle_operation_relations(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    operation: OperationId,
) -> BTreeSet<RelationId> {
    declarations
        .keys()
        .filter(|relation| relation.operation() == operation)
        .cloned()
        .collect()
}

/// The sponsor cases one operation admits, restated.
///
/// An operation whose declared open-flow policy admits a fee sponsor has
/// both cases; any other operation has the sponsorless case only. Read
/// off the declaration, never off the operation's name.
fn oracle_sponsor_cases(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    operation: OperationId,
) -> Vec<SponsorCase> {
    let sponsored = declarations.values().any(|declaration| {
        declaration.id.operation() == operation
            && matches!(
                &declaration.relation,
                Relation::OpenFlowPolicy { allowed }
                    if allowed.contains(&OpenFlowKind::FeeSponsor)
            )
    });

    if sponsored {
        vec![SponsorCase::Absent, SponsorCase::Present]
    } else {
        vec![SponsorCase::Absent]
    }
}

/// The representation each of one operation's represented objects has
/// under one plan, restated.
///
/// The represented objects come from the operation's representation
/// declarations, and the mode from the plan's own decision for that
/// object — never from a mode some other object happened to select.
fn oracle_representations(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    plan: &ProofPlanCandidate,
    operation: OperationId,
) -> BTreeMap<ObjectId, RepresentationMode> {
    let mut representations = BTreeMap::new();

    for declaration in declarations.values() {
        if declaration.id.operation() != operation {
            continue;
        }

        let Relation::Representation { object, .. } = &declaration.relation else {
            continue;
        };

        let mode = plan
            .representations
            .get(&RepresentationChoiceId {
                operation,
                object: *object,
            })
            .copied()
            .expect("the plan decided every represented object in scope");

        representations.insert(*object, mode);
    }

    representations
}

/// One operation's execution cases under one plan, restated (§9.4).
fn oracle_cases(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    plan: &ProofPlanCandidate,
    operation: OperationId,
) -> BTreeSet<ExecutionCaseId> {
    let representations = oracle_representations(declarations, plan, operation);

    oracle_sponsor_cases(declarations, operation)
        .into_iter()
        .map(|sponsor| ExecutionCaseId {
            operation,
            sponsor,
            representations: representations.clone(),
        })
        .collect()
}

/// One operation's relation-case keys under one plan, restated (§9.5).
///
/// The §14.3 example written out: the relations of the operation crossed
/// with its independently derived cases, not a read-back of the stored
/// keys.
fn oracle_relation_case_keys(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    plan: &ProofPlanCandidate,
    operation: OperationId,
) -> BTreeSet<RelationCaseKey> {
    let mut keys = BTreeSet::new();

    for relation in oracle_operation_relations(declarations, operation) {
        for case in oracle_cases(declarations, plan, operation) {
            keys.insert(RelationCaseKey {
                relation: relation.clone(),
                case,
            });
        }
    }

    keys
}

/// The discharge boundaries one relation answers at, restated.
///
/// The §8.2–§8.5 disposition matrix written out again: a compiler-static
/// obligation is answered by the compiler's own analysis, a structural
/// one by the emitted bundle, an external one by a typed report, and
/// everything a target actually evaluates is runtime. A representation
/// or lifecycle relation is genuinely hybrid and states two.
fn oracle_boundaries(relation: &Relation) -> BTreeSet<CoverageBoundary> {
    use CoverageBoundary as Boundary;

    match relation {
        Relation::Constructibility { .. } => BTreeSet::from([Boundary::CompilerStatic]),
        Relation::PermissionlessAuthorization => BTreeSet::from([Boundary::BackendStructural]),
        Relation::Representation { .. } | Relation::LifecycleExit { .. } => {
            BTreeSet::from([Boundary::CompilerStatic, Boundary::BackendStructural])
        }
        Relation::SubstrateConservation { .. } => BTreeSet::from([Boundary::ExternalEvidence]),
        Relation::Cardinality { .. }
        | Relation::AllowedObjectFamilies { .. }
        | Relation::Recognition { .. }
        | Relation::AmountConservation { .. }
        | Relation::OwnerAuthorization { .. }
        | Relation::SponsorIsolation
        | Relation::SponsorEnvelopeMultiplicity { .. }
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::ExpressionPredicate { .. } => BTreeSet::from([Boundary::RuntimeCarrier]),
    }
}

/// Whether one relation is active in one case, restated.
///
/// A relation over the optional sponsor family exists only where that
/// family does. Sponsor isolation and sponsor envelope multiplicity are
/// unconditional: with no sponsor region they establish that none
/// exists, which is a claim about every case.
fn oracle_activity(relation: &Relation, case: &ExecutionCaseId) -> RelationActivity {
    let conditional = match relation {
        Relation::Cardinality { object, .. } | Relation::Recognition { object, .. } => {
            *object == ObjectId::PlainLbtc
        }
        _ => false,
    };

    if conditional && case.sponsor != SponsorCase::Present {
        RelationActivity::Vacuous
    } else {
        RelationActivity::Active
    }
}

/// The relation-case keys of one operation answering at one boundary,
/// restated.
///
/// `active_only` selects the active-runtime partition: a vacuous
/// relation-case is present in the census with no runtime obligation
/// attached, so it belongs to the runtime boundary and not to the set
/// a placement must carry.
fn oracle_keys_at(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    plan: &ProofPlanCandidate,
    operation: OperationId,
    boundary: CoverageBoundary,
    active_only: bool,
) -> BTreeSet<RelationCaseKey> {
    oracle_relation_case_keys(declarations, plan, operation)
        .into_iter()
        .filter(|key| {
            let relation = &declarations[&key.relation].relation;

            oracle_boundaries(relation).contains(&boundary)
                && (!active_only
                    || oracle_activity(relation, &key.case) == RelationActivity::Active)
        })
        .collect()
}

/// The external evidence one scope's declarations require, restated.
fn oracle_evidence(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
) -> BTreeSet<ExternalEvidenceRequirement> {
    declarations
        .values()
        .filter_map(|declaration| match &declaration.relation {
            Relation::SubstrateConservation { asset } => {
                Some(ExternalEvidenceRequirement::SubstrateConservation {
                    operation: declaration.id.operation(),
                    asset: *asset,
                })
            }
            _ => None,
        })
        .collect()
}

/// One plan's lifecycle rows, restated from the pilot declarations.
///
/// Every required exit the realization declares for an object the scope
/// represents, paired with the mode the plan selected for that object.
/// An exit inside the compiler scope is available; anything else is an
/// outstanding future obligation, which is what makes a partial pilot
/// scope honest rather than apparently complete.
fn oracle_lifecycle_rows(
    analyzed: &Analyzed,
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    plan: &ProofPlanCandidate,
) -> BTreeSet<LifecycleRequirement> {
    let scope = analyzed.scope();
    let mut represented = BTreeMap::new();

    for declaration in declarations.values() {
        let Relation::Representation { object, .. } = &declaration.relation else {
            continue;
        };

        let mode = plan
            .representations
            .get(&RepresentationChoiceId {
                operation: declaration.id.operation(),
                object: *object,
            })
            .copied()
            .expect("the plan decided every represented object in scope");

        represented.insert(*object, mode);
    }

    let mut rows = BTreeSet::new();

    for node in &analyzed.input.realization().project().lifecycle.nodes {
        let LifecycleNodeId::RequiredExit { object, operation } = &node.id else {
            continue;
        };

        let Some(mode) = represented.get(object).copied() else {
            continue;
        };

        rows.insert(LifecycleRequirement {
            object: *object,
            representation: mode,
            exit: *operation,
            status: if scope.contains(operation) {
                LifecycleExitStatus::AvailableInCompilerScope
            } else {
                LifecycleExitStatus::DeclaredOutsideCompilerScope
            },
        });
    }

    rows
}

/// One plan's outstanding lifecycle obligations, restated.
fn oracle_lifecycle_obligations(
    analyzed: &Analyzed,
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    plan: &ProofPlanCandidate,
) -> BTreeSet<LifecycleRequirement> {
    oracle_lifecycle_rows(analyzed, declarations, plan)
        .into_iter()
        .filter(|row| row.status == LifecycleExitStatus::DeclaredOutsideCompilerScope)
        .collect()
}

/// One operation's relations, analyzed on their own.
///
/// The §14.5 comparison route for a placement factor: the *whole-plan*
/// enumerator applied to a one-operation scope produces that operation's
/// placement set without the operation-restricted obligation census
/// [`analyze_operation_placements`] builds, so the two routes are
/// independent implementations of the same factor.
fn single_operation_relations(operation: OperationId) -> CompilerRelationAnalysis {
    build_relation_analysis(&bound_input(&[operation])).expect("single-operation relations")
}

static COMPACT_ASH_RELATIONS: LazyLock<CompilerRelationAnalysis> =
    LazyLock::new(|| single_operation_relations(OperationId::CompactAsh));
static TRANSFER_LIVE_RELATIONS: LazyLock<CompilerRelationAnalysis> =
    LazyLock::new(|| single_operation_relations(OperationId::TransferLive));

fn operation_relations(operation: OperationId) -> &'static CompilerRelationAnalysis {
    match operation {
        OperationId::CompactAsh => &COMPACT_ASH_RELATIONS,
        OperationId::TransferLive => &TRANSFER_LIVE_RELATIONS,
        other => panic!("no pilot relations for {other:?}"),
    }
}

/// One operation's placement factor under one plan, by the whole-plan
/// route.
fn oracle_placement_factor(
    operation: OperationId,
    plan: &ProofPlanCandidate,
) -> PlacedProofPlanCandidate {
    place_proof_plan(operation_relations(operation), plan, limits())
        .expect("whole-plan placement of a one-operation scope")
}

// --- §14.2 architecture census, scope, and missing set ---

#[test]
fn the_oracle_reproduces_the_architecture_scope_status() {
    let census = oracle_architecture_operations();

    assert_eq!(census.len(), 13);

    for analyzed in scopes() {
        let scope = analyzed.scope();
        let expected = census.difference(&scope).copied().collect::<BTreeSet<_>>();

        // The scope the oracle expects is the scope the test asked for,
        // and the analyzed value's own retained scope must be that same
        // set rather than one it chose.
        assert_eq!(
            analyzed
                .projection
                .source
                .compilation_scope
                .operations()
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            scope,
        );

        let ArchitectureScopeStatus::Partial { missing } = &analyzed.projection.architecture_scope
        else {
            panic!("a pilot scope claims to cover the architecture");
        };

        assert_eq!(missing, &expected);
        assert!(missing.is_disjoint(&scope));
        assert_eq!(missing.len(), 13 - scope.len());
    }
}

// --- §14.2 and §14.3 the exact proof-plan set ---

#[test]
fn the_oracle_reproduces_the_exact_proof_plan_set() {
    for analyzed in scopes() {
        // The §14.3-sanctioned source: the exact feasible search itself,
        // run again, rather than a read-back of the analyzed keys.
        let expected = enumerate_feasible_plans(&analyzed.input, &CapabilityView::Unconstrained)
            .expect("feasible proof plans")
            .candidates
            .into_iter()
            .collect::<BTreeSet<_>>();

        assert!(!expected.is_empty());
        assert_eq!(
            analyzed
                .projection
                .proof_plans
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected,
        );
    }
}

// --- §14.2 relations per operation ---

#[test]
fn the_oracle_reproduces_the_relation_census_per_operation() {
    for analyzed in scopes() {
        let declarations = oracle_declarations(analyzed);
        let expected = declarations.keys().cloned().collect::<BTreeSet<_>>();

        assert!(!expected.is_empty());

        for plan in analyzed.projection.proof_plans.values() {
            assert_eq!(
                plan.relation_requirements
                    .keys()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                expected,
            );
        }

        // Each operation owns exactly its own declared relations, and
        // the scope census is their disjoint union.
        let mut union = BTreeSet::new();

        for operation in analyzed.scope() {
            let owned = oracle_operation_relations(&declarations, operation);

            assert!(!owned.is_empty());
            assert!(
                owned
                    .iter()
                    .all(|relation| relation.operation() == operation)
            );
            assert!(union.is_disjoint(&owned));

            union.extend(owned);
        }

        assert_eq!(union, expected);
    }
}

// --- §14.2 cases per plan and operation ---

#[test]
fn the_oracle_reproduces_the_execution_case_census() {
    for analyzed in scopes() {
        let declarations = oracle_declarations(analyzed);

        for (key, plan) in &analyzed.projection.proof_plans {
            for operation in analyzed.scope() {
                let expected = oracle_cases(&declarations, key, operation);

                // Both pilots declare a fee-sponsor open flow, so both
                // expand into exactly the sponsorless and sponsored
                // cases of one fixed representation.
                assert_eq!(expected.len(), 2);
                assert_eq!(plan.operations[&operation].execution_cases, expected);
                assert_eq!(plan.operations[&operation].coverage.cases, expected);
            }
        }
    }
}

// --- §14.2 relation-case and coverage keys ---

#[test]
fn the_oracle_reproduces_the_relation_case_and_coverage_keys() {
    for analyzed in scopes() {
        let declarations = oracle_declarations(analyzed);

        for (key, plan) in &analyzed.projection.proof_plans {
            for operation in analyzed.scope() {
                let expected = oracle_relation_case_keys(&declarations, key, operation);
                let factor = &plan.operations[&operation];

                assert_eq!(
                    expected.len(),
                    oracle_operation_relations(&declarations, operation).len() * 2,
                );
                assert_eq!(
                    factor
                        .relation_cases
                        .keys()
                        .cloned()
                        .collect::<BTreeSet<_>>(),
                    expected,
                );

                // §10.1: the coverage keys are the relation-case keys.
                assert_eq!(
                    factor
                        .coverage
                        .requirements
                        .keys()
                        .cloned()
                        .collect::<BTreeSet<_>>(),
                    expected,
                );
            }
        }
    }
}

// --- §14.2 the boundary partitions ---

#[test]
fn the_oracle_reproduces_the_boundary_key_partitions() {
    for analyzed in scopes() {
        let declarations = oracle_declarations(analyzed);

        for (key, plan) in &analyzed.projection.proof_plans {
            for operation in analyzed.scope() {
                let factor = &plan.operations[&operation];

                for boundary in [
                    CoverageBoundary::CompilerStatic,
                    CoverageBoundary::BackendStructural,
                    CoverageBoundary::ExternalEvidence,
                    CoverageBoundary::RuntimeCarrier,
                ] {
                    let expected = oracle_keys_at(&declarations, key, operation, boundary, false);
                    let stored = factor
                        .relation_cases
                        .iter()
                        .filter(|(_, bundle)| bundle.boundaries.contains(&boundary))
                        .map(|(key, _)| key.clone())
                        .collect::<BTreeSet<_>>();

                    assert!(!expected.is_empty(), "{operation:?} {boundary:?}");
                    assert_eq!(stored, expected, "{operation:?} {boundary:?}");
                }

                // The active runtime partition is what a placement must
                // carry: every stored placement's assignment census is
                // exactly this set, and a vacuous relation-case is in
                // the census with no carrier rather than omitted.
                let active = oracle_keys_at(
                    &declarations,
                    key,
                    operation,
                    CoverageBoundary::RuntimeCarrier,
                    true,
                );

                assert!(!factor.feasible_placements.is_empty());

                for placement in &factor.feasible_placements {
                    assert_eq!(
                        placement
                            .assignments
                            .keys()
                            .cloned()
                            .collect::<BTreeSet<_>>(),
                        active,
                    );
                }

                for (case_key, bundle) in &factor.relation_cases {
                    let relation = &declarations[&case_key.relation].relation;

                    assert_eq!(bundle.activity, oracle_activity(relation, &case_key.case));
                    assert_eq!(
                        bundle.carrier_assignments.is_empty(),
                        !active.contains(case_key),
                    );
                }

                // Exactly four inactive sponsor relation-cases per
                // operation in the unsponsored case (Guide-6 §2.2),
                // restated as a count the oracle derives rather than
                // reads back.
                let vacuous = factor
                    .relation_cases
                    .values()
                    .filter(|bundle| bundle.activity == RelationActivity::Vacuous)
                    .count();

                assert_eq!(vacuous, 4, "{operation:?}");
            }
        }
    }
}

// --- §14.2 and §14.5 the placement factors and layout census ---

#[test]
fn the_oracle_reproduces_the_placement_factors_and_layout_census() {
    for analyzed in scopes() {
        for (key, plan) in &analyzed.projection.proof_plans {
            for operation in analyzed.scope() {
                let expected = oracle_placement_factor(operation, key);
                let factor = &plan.operations[&operation];

                assert_eq!(
                    factor.feasible_placements,
                    expected
                        .feasible_placements
                        .iter()
                        .map(PlacementCandidate::project)
                        .collect::<BTreeSet<_>>(),
                    "{operation:?}",
                );

                assert_eq!(
                    factor.layout_requirements,
                    expected
                        .layout_requirements
                        .iter()
                        .cloned()
                        .collect::<BTreeSet<_>>(),
                    "{operation:?}",
                );

                // A factor with one placement would make the product
                // theorem vacuous for that operation.
                assert!(factor.feasible_placements.len() > 1);
            }
        }
    }
}

// --- §14.2 the external evidence union ---

#[test]
fn the_oracle_reproduces_the_external_evidence_union() {
    for analyzed in scopes() {
        let declarations = oracle_declarations(analyzed);
        let expected = oracle_evidence(&declarations);

        // One whole-transaction substrate obligation per analyzed
        // operation, retained as an unresolved requirement.
        assert_eq!(expected.len(), analyzed.scope().len());
        assert_eq!(analyzed.projection.required_external_evidence, expected);

        for plan in analyzed.projection.proof_plans.values() {
            let owned = plan
                .relation_requirements
                .values()
                .flat_map(|bundle| bundle.external_evidence.iter().cloned())
                .collect::<BTreeSet<_>>();

            assert_eq!(owned, expected);
        }
    }
}

// --- §14.2 the lifecycle obligations ---

#[test]
fn the_oracle_reproduces_the_lifecycle_obligations() {
    for analyzed in scopes() {
        let declarations = oracle_declarations(analyzed);

        for (key, plan) in &analyzed.projection.proof_plans {
            let expected = oracle_lifecycle_obligations(analyzed, &declarations, key);

            assert!(!expected.is_empty());

            let LifecycleCompleteness::Incomplete { obligations } = &plan.lifecycle else {
                panic!("a pilot plan claims a complete deployment lifecycle");
            };

            assert_eq!(obligations, &expected);

            // The plan's own rows are the complete set the oracle
            // expects, of which the obligations are the outstanding
            // part.
            assert_eq!(
                key.lifecycle.iter().cloned().collect::<BTreeSet<_>>(),
                oracle_lifecycle_rows(analyzed, &declarations, key),
            );
        }
    }

    assert_eq!(
        outstanding_exits(&COMPACT_ASH),
        BTreeSet::from([OperationId::Clear]),
    );
    assert_eq!(
        outstanding_exits(&TRANSFER_LIVE),
        BTreeSet::from([OperationId::Burn, OperationId::Redeem]),
    );
}

/// The exits one analyzed scope leaves outstanding, across its plans.
fn outstanding_exits(analyzed: &Analyzed) -> BTreeSet<OperationId> {
    analyzed
        .projection
        .proof_plans
        .values()
        .filter_map(|plan| match &plan.lifecycle {
            LifecycleCompleteness::Incomplete { obligations } => Some(obligations),
            LifecycleCompleteness::Complete => None,
        })
        .flatten()
        .map(|obligation| obligation.exit)
        .collect()
}

// --- §14.4 the requirement-union oracle ---

/// What one relation owns under one plan, with the §7.3 ownership rule
/// restated.
///
/// The rule, not the row content: a statically validated relation owns
/// nothing because no proof was selected for it, an externally evidenced
/// relation owns its approved class's items together with the typed
/// report requirement, and a proof-required relation owns exactly the
/// items of the alternative the plan chose. The per-relation derivations
/// themselves have their own oracles (§14.1) and are the item source
/// here; what is checked is who owns what, and that the owners union
/// back to the aggregate.
fn oracle_owned(
    declaration: &RelationDeclaration,
    plan: &ProofPlanCandidate,
) -> (
    BTreeSet<RequiredCapability>,
    BTreeSet<SourceRequirement>,
    BTreeSet<ExternalEvidenceRequirement>,
) {
    match &declaration.relation {
        // No decision to make, so nothing owned.
        Relation::Representation { .. } | Relation::LifecycleExit { .. } => {
            (BTreeSet::new(), BTreeSet::new(), BTreeSet::new())
        }

        Relation::SubstrateConservation { asset } => {
            let approved =
                ProofAlternativeId::new(declaration.id.clone(), ProofKind::SubstrateConservation);

            assert!(
                declaration.proof_alternatives.contains(&approved),
                "the realization approves the external proof class",
            );

            (
                proof_capabilities(declaration, approved.proof()),
                derive_source_requirements(declaration, approved.proof())
                    .expect("external source rows")
                    .into_iter()
                    .collect(),
                BTreeSet::from([ExternalEvidenceRequirement::SubstrateConservation {
                    operation: declaration.id.operation(),
                    asset: *asset,
                }]),
            )
        }

        _ => {
            let selected = plan
                .proofs
                .get(&declaration.id)
                .expect("the plan selected a proof for every proof-required relation");

            assert!(
                declaration.proof_alternatives.contains(selected),
                "the selection is realization-approved",
            );

            (
                proof_capabilities(declaration, selected.proof()),
                derive_source_requirements(declaration, selected.proof())
                    .expect("selected source rows")
                    .into_iter()
                    .collect(),
                BTreeSet::new(),
            )
        }
    }
}

#[test]
fn the_relation_owned_union_equals_the_candidate_aggregates() {
    for analyzed in scopes() {
        let declarations = oracle_declarations(analyzed);

        for (key, plan) in &analyzed.projection.proof_plans {
            let mut capabilities = BTreeSet::new();
            let mut sources = BTreeSet::new();
            let mut evidence = BTreeSet::new();

            for declaration in declarations.values() {
                let (owned_capabilities, owned_sources, owned_evidence) =
                    oracle_owned(declaration, key);

                // The ownership index itself: the stored bundle claims
                // exactly what the rule says this relation owns.
                let bundle = &plan.relation_requirements[&declaration.id];

                assert_eq!(bundle.required_capabilities, owned_capabilities);
                assert_eq!(bundle.source_requirements, owned_sources);
                assert_eq!(bundle.external_evidence, owned_evidence);

                capabilities.extend(owned_capabilities);
                sources.extend(owned_sources);
                evidence.extend(owned_evidence);
            }

            // §7.6 and §14.4: exact set equality with the candidate's
            // own aggregates, in both directions. A missing item is a
            // lost requirement; an unowned one is a requirement no
            // relation introduced.
            assert!(!capabilities.is_empty());
            assert!(!sources.is_empty());
            assert!(!evidence.is_empty());

            assert_eq!(key.required_capabilities, capabilities);
            assert_eq!(
                key.source_requirements
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                sources,
            );
            assert_eq!(key.external_evidence, evidence);
        }
    }
}

#[test]
fn the_external_evidence_capability_and_source_survive_the_union() {
    // The regression §14.4 exists for: an external relation's capability
    // and source row are exactly the items an aggregate has lost before,
    // and they have an owner here.
    let analyzed = &COMBINED;
    let declarations = oracle_declarations(analyzed);

    for (key, plan) in &analyzed.projection.proof_plans {
        let mut checked = 0;

        for declaration in declarations.values() {
            if !matches!(declaration.relation, Relation::SubstrateConservation { .. }) {
                continue;
            }

            let (capabilities, sources, evidence) = oracle_owned(declaration, key);

            assert!(!capabilities.is_empty());
            assert!(!evidence.is_empty());
            assert!(capabilities.is_subset(&key.required_capabilities));
            assert!(evidence.is_subset(&key.external_evidence));

            for row in &sources {
                assert!(key.source_requirements.contains(row));
            }

            // The evidence is a retained requirement and nothing more:
            // no verdict, digest, or discharge status accompanies it.
            assert_eq!(
                plan.relation_requirements[&declaration.id].external_evidence,
                evidence,
            );

            checked += 1;
        }

        assert_eq!(checked, analyzed.scope().len());
    }
}

// --- §14.5 the ordinary factorization level ---

#[test]
fn every_analyzed_constraint_is_operation_owned() {
    let analyzed = &COMBINED;

    for plan in analyzed.projection.proof_plans.values() {
        assert_eq!(plan.operations.len(), 2);

        for (operation, factor) in &plan.operations {
            assert_eq!(factor.operation, *operation);

            for case in &factor.execution_cases {
                assert_eq!(case.operation, *operation);
            }

            for (key, bundle) in &factor.relation_cases {
                assert_eq!(key.relation.operation(), *operation);
                assert_eq!(key.case.operation, *operation);
                assert_eq!(&bundle.key, key);

                for requirement in &bundle.layout_requirements {
                    assert!(factor.layout_requirements.contains(requirement));
                }
            }

            // No placement names a second operation's obligation: a
            // stored member of the cross-operation product would.
            for placement in &factor.feasible_placements {
                for key in placement.assignments.keys() {
                    assert_eq!(key.relation.operation(), *operation);
                    assert_eq!(key.case.operation, *operation);
                }

                for requirement in &placement.layout_requirements {
                    assert!(factor.layout_requirements.contains(requirement));
                }
            }

            // No coverage dependency leaves the factor.
            for node in &factor.coverage_dependencies.nodes {
                assert_eq!(node.id.operation(), *operation);
            }

            for edge in &factor.coverage_dependencies.edges {
                assert_eq!(edge.source.operation(), *operation);
                assert_eq!(edge.target.operation(), *operation);
            }
        }

        // The stored content is the two factors. Their product is far
        // larger, which is the whole reason the factors are what is
        // stored.
        let sizes = plan
            .operations
            .values()
            .map(|factor| factor.feasible_placements.len())
            .collect::<Vec<_>>();

        assert!(sizes.iter().sum::<usize>() < sizes.iter().product::<usize>());
    }
}

#[test]
fn no_relation_dependency_crosses_an_operation() {
    // Condition 2 of §9.8, checked against the realization's own
    // declared dependency edges rather than against the analyzed
    // foundation graph.
    for analyzed in scopes() {
        let scope = analyzed.scope();
        let source = analyzed.input.realization().project();
        let mut checked = 0;

        for edge in &source.relations.edges {
            if !scope.contains(&edge.source.operation())
                && !scope.contains(&edge.target.operation())
            {
                continue;
            }

            assert_eq!(
                edge.source.operation(),
                edge.target.operation(),
                "a relation dependency crosses two operations",
            );

            checked += 1;
        }

        assert!(checked > 0);
    }
}

// --- §14.5 the manual real-pilot product level (§24.4) ---

/// The old combined placement product of the real two-pilot scope,
/// compared with the test-side product of the two stored factors.
///
/// Ignored in ordinary development because the ordinary factorization
/// signal now covers the same property generically: factor exactness
/// against the whole-plan route, the synthetic exhaustive product
/// comparison in `analyzed_operation_tests`, and the structural
/// operation-ownership checks above. This remains runnable for the
/// Phase-2 exit record of §24.4:
///
/// ```sh
/// cargo test -p tripod-compiler \
///   combined_scope_product_matches_operation_factors -- --ignored --nocapture
/// ```
///
/// Printing is the point of the manual lane — the gate record wants the
/// measured factor sizes, product size, and wall time — so the workspace
/// stdout denial is lifted here and nowhere else.
#[test]
#[ignore = "phase-exit regression; run with -- --ignored"]
#[allow(clippy::print_stdout)]
fn combined_scope_product_matches_operation_factors() {
    let analyzed = &COMBINED;
    let relations = build_relation_analysis(&analyzed.input).expect("combined relations");
    let candidates = enumerate_feasible_plans(&analyzed.input, &CapabilityView::Unconstrained)
        .expect("feasible proof plans")
        .candidates;

    let started = Instant::now();
    let placed = place_feasible_proof_plans(&relations, &candidates, limits())
        .expect("combined-scope placement product");
    let elapsed = started.elapsed();

    assert_eq!(placed.placed.len(), candidates.len());

    for entry in &placed.placed {
        // The old whole-plan enumerator's combined set, as assignment
        // censuses over both operations.
        let global = entry
            .feasible_placements
            .iter()
            .map(PlacementCandidate::project)
            .collect::<BTreeSet<_>>();

        // The test-side product of the two stored factors: every
        // combination of one placement per operation, merged by
        // relation-case key. The keys are operation-owned, so no merge
        // can collide.
        let factors = analyzed
            .plan(&entry.proof_plan)
            .operations
            .values()
            .map(|factor| factor.feasible_placements.iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        assert_eq!(factors.len(), 2);

        let mut product = BTreeSet::<PlacementCandidateProjection>::new();

        for left in &factors[0] {
            for right in &factors[1] {
                let mut assignments = left.assignments.clone();

                for (key, carriers) in &right.assignments {
                    assert!(
                        assignments.insert(key.clone(), carriers.clone()).is_none(),
                        "operation-owned keys cannot collide",
                    );
                }

                product.insert(PlacementCandidateProjection {
                    assignments,
                    layout_requirements: left
                        .layout_requirements
                        .union(&right.layout_requirements)
                        .cloned()
                        .collect(),
                });
            }
        }

        println!(
            "factor sizes {:?}; product {}; global {}",
            factors.iter().map(Vec::len).collect::<Vec<_>>(),
            product.len(),
            global.len(),
        );

        assert_eq!(global, product);
        assert_eq!(global.len(), factors[0].len() * factors[1].len());

        // The production analyzed value stores the factors only: the
        // product just enumerated appears nowhere inside it.
        for factor in analyzed.plan(&entry.proof_plan).operations.values() {
            assert!(factor.feasible_placements.len() < global.len());
        }
    }

    println!(
        "combined-scope product over {} plans in {:.1} s",
        candidates.len(),
        elapsed.as_secs_f64(),
    );
}

// --- §14.6 the corruption oracle ---

/// One assembled pilot program with the production validator's own
/// expectations beside it.
///
/// Both sides are exercised: the production validator must reject the
/// mutation, and the independent oracle above must disagree with the
/// mutated value. A corruption only one side sees is a blind spot in the
/// other.
struct Corruptible {
    analyzed: Analyzed,
    expectations: AnalyzedExpectations,
}

static CORRUPTIBLE: LazyLock<Corruptible> = LazyLock::new(|| {
    let analyzed = analyze(&[OperationId::CompactAsh]);
    let expectations = derive_expectations(&analyzed.input, limits()).expect("expectations");

    Corruptible {
        analyzed,
        expectations,
    }
});

impl Corruptible {
    /// Apply one mutation and require the production validator to
    /// reject it.
    fn reject(&self, mutate: impl FnOnce(&mut ScopedAnalyzedProgram)) -> ScopedAnalyzedProgram {
        let mut program = self.analyzed.program.clone();

        mutate(&mut program);
        validate_against_expectations(&self.analyzed.input, &self.expectations, &program)
            .expect_err("the production validator rejects the corruption");

        program
    }

    fn declarations(&self) -> BTreeMap<RelationId, RelationDeclaration> {
        oracle_declarations(&self.analyzed)
    }

    fn key(&self) -> ProofPlanCandidate {
        self.analyzed
            .program
            .proof_plans
            .keys()
            .next()
            .expect("an analyzed plan")
            .clone()
    }
}

/// The first plan of one program, mutably.
fn plan_mut(program: &mut ScopedAnalyzedProgram) -> &mut AnalyzedProofPlan {
    let key = program
        .proof_plans
        .keys()
        .next()
        .expect("an analyzed plan")
        .clone();

    program
        .proof_plans
        .get_mut(&key)
        .expect("the analyzed plan")
}

/// The compact-ASH factor of one program's first plan, mutably.
fn factor_mut(program: &mut ScopedAnalyzedProgram) -> &mut AnalyzedOperation {
    plan_mut(program)
        .operations
        .get_mut(&OperationId::CompactAsh)
        .expect("the compact-ASH factor")
}

#[test]
fn a_claimed_complete_architecture_scope_is_rejected_and_disagrees() {
    // §15.1.
    let fixture = &CORRUPTIBLE;
    let program = fixture.reject(|program| {
        program.architecture_scope = ArchitectureScopeStatus::Complete;
    });

    let expected = oracle_architecture_operations()
        .difference(&fixture.analyzed.scope())
        .copied()
        .collect::<BTreeSet<_>>();

    assert!(!expected.is_empty());
    assert_ne!(
        program.architecture_scope,
        ArchitectureScopeStatus::Partial { missing: expected },
    );
}

#[test]
fn a_removed_proof_plan_is_rejected_and_disagrees() {
    // §15.2.
    let fixture = &CORRUPTIBLE;
    let key = fixture.key();
    let program = fixture.reject(|program| {
        program.proof_plans.remove(&key);
    });

    let expected =
        enumerate_feasible_plans(&fixture.analyzed.input, &CapabilityView::Unconstrained)
            .expect("feasible proof plans")
            .candidates
            .into_iter()
            .collect::<BTreeSet<_>>();

    assert_ne!(
        program.proof_plans.keys().cloned().collect::<BTreeSet<_>>(),
        expected,
    );
}

#[test]
fn a_removed_relation_requirement_is_rejected_and_disagrees() {
    // §15.3.
    let fixture = &CORRUPTIBLE;
    let declarations = fixture.declarations();
    let dropped = declarations
        .keys()
        .next()
        .expect("a declared relation")
        .clone();

    let program = fixture.reject(|program| {
        plan_mut(program).relation_requirements.remove(&dropped);
    });

    assert_ne!(
        plan_of(&program)
            .relation_requirements
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        declarations.keys().cloned().collect::<BTreeSet<_>>(),
    );
}

/// The first plan of one program.
fn plan_of(program: &ScopedAnalyzedProgram) -> &AnalyzedProofPlan {
    program
        .proof_plans
        .values()
        .next()
        .expect("an analyzed plan")
}

#[test]
fn a_removed_sponsored_case_is_rejected_and_disagrees() {
    // §15.4.
    let fixture = &CORRUPTIBLE;
    let key = fixture.key();
    let declarations = fixture.declarations();
    let program = fixture.reject(|program| {
        let factor = factor_mut(program);
        let sponsored = factor
            .execution_cases
            .iter()
            .find(|case| case.sponsor == SponsorCase::Present)
            .expect("the sponsored case")
            .clone();

        factor.execution_cases.remove(&sponsored);
    });

    assert_ne!(
        plan_of(&program).operations[&OperationId::CompactAsh].execution_cases,
        oracle_cases(&declarations, &key, OperationId::CompactAsh),
    );
}

#[test]
fn a_removed_placement_is_rejected_and_disagrees() {
    // §15.5.
    let fixture = &CORRUPTIBLE;
    let key = fixture.key();
    let program = fixture.reject(|program| {
        let factor = factor_mut(program);
        let dropped = factor
            .feasible_placements
            .iter()
            .next()
            .expect("a feasible placement")
            .clone();

        factor.feasible_placements.remove(&dropped);
    });

    let expected = oracle_placement_factor(OperationId::CompactAsh, &key)
        .feasible_placements
        .iter()
        .map(PlacementCandidate::project)
        .collect::<BTreeSet<_>>();

    assert_ne!(
        plan_of(&program).operations[&OperationId::CompactAsh].feasible_placements,
        expected,
    );
}

#[test]
fn a_removed_coverage_entry_is_rejected_and_disagrees() {
    // §15.6.
    let fixture = &CORRUPTIBLE;
    let key = fixture.key();
    let declarations = fixture.declarations();
    let program = fixture.reject(|program| {
        let factor = factor_mut(program);
        let dropped = factor
            .coverage
            .requirements
            .keys()
            .next()
            .expect("a covered relation-case")
            .clone();

        factor.coverage.requirements.remove(&dropped);
    });

    assert_ne!(
        plan_of(&program).operations[&OperationId::CompactAsh]
            .coverage
            .requirements
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        oracle_relation_case_keys(&declarations, &key, OperationId::CompactAsh),
    );
}

#[test]
fn a_dropped_future_exit_is_rejected_and_disagrees() {
    // §15.7.
    let fixture = &CORRUPTIBLE;
    let key = fixture.key();
    let declarations = fixture.declarations();
    let program = fixture.reject(|program| {
        plan_mut(program).lifecycle = LifecycleCompleteness::Complete;
    });

    let expected = oracle_lifecycle_obligations(&fixture.analyzed, &declarations, &key);

    assert!(!expected.is_empty());
    assert_ne!(
        plan_of(&program).lifecycle,
        LifecycleCompleteness::Incomplete {
            obligations: expected,
        },
    );
}

#[test]
fn a_sponsor_amount_source_row_is_rejected_and_disagrees() {
    // §15.8.
    let fixture = &CORRUPTIBLE;
    let key = fixture.key();
    let declarations = fixture.declarations();
    let owner = declarations
        .values()
        .find(|declaration| matches!(declaration.relation, Relation::AmountConservation { .. }))
        .expect("a conservation relation")
        .clone();
    let row = sponsor_amount_row(&owner.id);

    let error = {
        let mut program = fixture.analyzed.program.clone();

        plan_mut(&mut program)
            .relation_requirements
            .get_mut(&owner.id)
            .expect("the relation bundle")
            .source_requirements
            .insert(row.clone());

        validate_against_expectations(&fixture.analyzed.input, &fixture.expectations, &program)
            .expect_err("the erased sponsor amount is rejected")
    };

    assert!(matches!(error, CompileError::SponsorValueRead));

    // The oracle disagrees for its own reason: the ownership rule says
    // this relation owns no such row.
    let (_, sources, _) = oracle_owned(&owner, &key);

    assert!(!sources.contains(&row));
}

/// A source row naming the erased sponsor family's amount.
fn sponsor_amount_row(relation: &RelationId) -> SourceRequirement {
    SourceRequirement {
        operand: crate::OperandId::new(
            relation.clone(),
            crate::source::OperandRole::ObjectFamilyAmount {
                side: realization::TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        ),
        source: crate::source::RequiredSourceKind::AuthenticatedConsensusValue,
        availability: realization::AvailabilityClass::Public,
        activation: crate::source::RequirementActivation::Always,
    }
}

// --- §12.5 the complete permutation matrix ---

/// One operation's factor, re-derived from permuted lower-stage inputs.
///
/// The permutation is applied to the vectors a stage actually consumes,
/// then the factor is rebuilt through the same stages the assembler
/// runs. A projection that depended on an input order would differ here;
/// the canonical containers are what make it not.
struct Permutable {
    relations: CompilerRelationAnalysis,
    requirements: BTreeMap<RelationId, RelationRequirements>,
    plan: ProofPlanCandidate,
    operation: OperationId,
    stored: AnalyzedOperation,
}

fn permutable(operation: OperationId) -> Permutable {
    let analyzed = match operation {
        OperationId::CompactAsh => &COMPACT_ASH,
        OperationId::TransferLive => &TRANSFER_LIVE,
        other => panic!("no pilot fixture for {other:?}"),
    };

    let relations = build_relation_analysis(&analyzed.input).expect("relations");
    let constructibility =
        build_constructibility_analysis(&analyzed.input).expect("constructibility");
    let lifecycle =
        build_lifecycle_analysis(&analyzed.input, &relations).expect("lifecycle analysis");
    let plan = analyzed
        .program
        .proof_plans
        .keys()
        .next()
        .expect("an analyzed plan")
        .clone();
    let requirements = relation_requirements(
        &analyzed.input,
        &relations,
        &constructibility,
        &lifecycle,
        &plan,
    )
    .expect("relation requirements");
    let stored = analyzed.plan(&plan).operations[&operation].clone();

    Permutable {
        relations,
        requirements,
        plan,
        operation,
        stored,
    }
}

fn permutables() -> [Permutable; 2] {
    [
        permutable(OperationId::CompactAsh),
        permutable(OperationId::TransferLive),
    ]
}

impl Permutable {
    /// The operation's placement analysis, as the assembler derives it.
    fn placement(&self) -> crate::analyzed_operation::OperationPlacementAnalysis {
        analyze_operation_placements(&self.relations, &self.plan, self.operation, limits())
            .expect("operation placements")
    }

    /// Rebuild coverage and the dependency graph from one permuted
    /// placed value, and compare with the stored factor.
    fn assert_coverage_equal(&self, placed: &PlacedProofPlanCandidate) {
        let mut coverage = analyze_placed_coverage(&self.relations, placed).expect("coverage");
        let graph =
            resolve_coverage_dependencies(&mut coverage, &self.relations).expect("dependencies");
        let projection = coverage.operations[&self.operation].project();

        assert_eq!(projection, self.stored.coverage, "{:?}", self.operation);
        assert_eq!(
            graph.project(),
            self.stored.coverage_dependencies,
            "{:?}",
            self.operation,
        );
    }

    /// One permuted placed value built from this operation's own
    /// placement analysis.
    fn placed(&self) -> PlacedProofPlanCandidate {
        let placement = self.placement();

        PlacedProofPlanCandidate {
            proof_plan: self.plan.clone(),
            execution_cases: placement.execution_cases,
            relation_case_plans: placement.relation_case_plans,
            feasible_placements: placement.placements,
            layout_requirements: placement.layout_requirements,
        }
    }
}

#[test]
fn a_permuted_carrier_order_places_equally() {
    for permutable in permutables() {
        let placement = permutable.placement();
        let mut eligibility = placement.eligibility.clone();

        for entry in &mut eligibility {
            entry.eligible.reverse();
            entry.ineligible.reverse();
        }

        let permuted =
            enumerate_feasible_placements(&placement.relation_case_plans, &eligibility, limits())
                .expect("permuted placement search");

        assert_eq!(
            permuted
                .candidates
                .iter()
                .map(PlacementCandidate::project)
                .collect::<BTreeSet<_>>(),
            permutable.stored.feasible_placements,
            "{:?}",
            permutable.operation,
        );
    }
}

#[test]
fn a_permuted_placement_order_covers_equally() {
    for permutable in permutables() {
        let mut placed = permutable.placed();
        placed.feasible_placements.reverse();

        permutable.assert_coverage_equal(&placed);
    }
}

#[test]
fn a_permuted_layout_order_covers_and_bundles_equally() {
    for permutable in permutables() {
        let mut placement = permutable.placement();
        placement.layout_requirements.reverse();

        let mut coverage =
            analyze_placed_coverage(&permutable.relations, &permutable.placed()).expect("coverage");
        resolve_coverage_dependencies(&mut coverage, &permutable.relations).expect("dependencies");

        let projection = coverage.operations[&permutable.operation].project();
        let bundles = relation_case_requirements(&placement, &permutable.requirements, &projection)
            .expect("relation-case bundles");

        assert_eq!(bundles, permutable.stored.relation_cases);
    }
}

#[test]
fn a_permuted_relation_case_order_covers_equally() {
    for permutable in permutables() {
        let mut placed = permutable.placed();
        placed.relation_case_plans.reverse();

        permutable.assert_coverage_equal(&placed);
    }
}

#[test]
fn a_permuted_coverage_dependency_insertion_order_resolves_equally() {
    // The coverage symbol census is built in the order the relation-case
    // plans arrive, so reversing them reverses the graph's insertion
    // order. The projection must not observe it.
    for permutable in permutables() {
        let mut placed = permutable.placed();
        placed.relation_case_plans.reverse();
        placed.feasible_placements.reverse();

        permutable.assert_coverage_equal(&placed);
    }
}

#[test]
fn a_permuted_execution_case_order_analyzes_equally() {
    for permutable in permutables() {
        let mut placed = permutable.placed();
        placed.execution_cases.reverse();

        permutable.assert_coverage_equal(&placed);
    }
}

#[test]
fn permuted_relation_dependencies_analyze_equally() {
    // Only the dependency edges are permuted: the declarations keep
    // their order, so anything that differed would have observed the
    // edge insertion order alone.
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let permutable = permutable(operation);
        let input = bound_input(&[operation]);
        let source = input.realization().project();
        let mut edges = source.relations.edges;
        edges.reverse();

        let relations =
            build_relation_graph(input.scope().operations(), &source.relations.nodes, &edges)
                .expect("permuted relation graph");
        let factor = analyze_operation(
            &relations,
            &permutable.requirements,
            &permutable.plan,
            operation,
            limits(),
        )
        .expect("operation factor");

        assert_eq!(factor, permutable.stored, "{operation:?}");
    }
}

#[test]
fn permuted_expression_declarations_analyze_equally() {
    // The pilots declare no expressions, so this permutation is
    // vacuous — and the vacuity is asserted rather than assumed: a
    // pilot that grew an expression would fail here instead of leaving
    // the permutation silently untested.
    for analyzed in scopes() {
        let source = analyzed.input.realization().project();

        assert_eq!(
            source.expressions.nodes,
            [] as [realization::ExpressionDeclaration; 0]
        );
        assert_eq!(
            source.expressions.edges,
            [] as [realization::ExpressionDependencyProjection; 0]
        );
        assert_eq!(
            analyzed.projection.foundation.expressions.nodes,
            [] as [crate::expression::CompilerExpressionNodeProjection; 0]
        );
        assert_eq!(
            analyzed.projection.foundation.expressions.edges,
            [] as [crate::expression::CompilerExpressionDependencyProjection; 0]
        );

        let mut nodes = source.expressions.nodes;
        nodes.reverse();

        assert_eq!(
            nodes.len(),
            analyzed.projection.foundation.expressions.nodes.len(),
        );
    }
}

#[test]
fn repeated_clean_analysis_projects_equally() {
    // Including the combined scope: the two-operation assembly is where
    // a shared or accumulated intermediate would show.
    for analyzed in scopes() {
        let again =
            analyze_scoped_program(&analyzed.input, limits()).expect("second clean analysis");

        assert_eq!(
            again.project(),
            analyzed.projection,
            "{:?}",
            analyzed.operations
        );
    }
}

#[test]
fn the_placed_carriers_of_one_factor_are_canonical() {
    // The carrier sets are set-shaped in the projection, so no carrier
    // ordering survives into a comparison at all.
    let analyzed = &COMBINED;

    for plan in analyzed.projection.proof_plans.values() {
        for factor in plan.operations.values() {
            for placement in &factor.feasible_placements {
                for carriers in placement.assignments.values() {
                    assert!(!carriers.is_empty());

                    let sorted = carriers.iter().cloned().collect::<Vec<PlacedCarrier>>();
                    let mut resorted = sorted.clone();
                    resorted.sort();

                    assert_eq!(sorted, resorted);
                }
            }
        }
    }
}

/// Layout requirements are compared as sets everywhere they are stored.
#[test]
fn the_layout_census_of_one_factor_is_canonical() {
    let analyzed = &COMBINED;

    for plan in analyzed.projection.proof_plans.values() {
        for factor in plan.operations.values() {
            let census = factor
                .layout_requirements
                .iter()
                .cloned()
                .collect::<Vec<LayoutRequirement>>();
            let mut resorted = census.clone();
            resorted.sort();

            assert_eq!(census, resorted);
            assert_ne!(census, [] as [LayoutRequirement; 0]);
        }
    }
}
