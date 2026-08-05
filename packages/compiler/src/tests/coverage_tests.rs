//! Relation-indexed coverage tests (Guide-6 Tranches B–D, H).
//!
//! The positive matrix is asserted against the real Phase-1 pilots
//! rather than a fixture, so the Guide-6 acceptance matrix is restated
//! here and compared with what the derivation produced. The negative
//! tests corrupt an already-derived analysis, because a census
//! validator that only ever sees its own well-formed output proves
//! nothing.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    CardinalityMaximum, ConstructibilityClass, Count, ExprId, ExpressionRole, ObservedSide,
    Relation, RelationDeclaration, RelationId, RelationKind, RelationSubject, RepresentationMode,
    TransactionSide,
};

use super::bound_input;
use crate::{
    CompileError,
    capability::{CapabilityView, RequiredCapability},
    case::SponsorCase,
    coverage::{
        CardinalityCeiling, CollateralPolicy, CollateralRequirement, CoverageBoundary,
        CoveragePurpose, CoverageRequirementId, EvidenceRole, NegativeCoverageRequirement,
        PlanCoverageAnalysis, RelationCoveragePlan, RelationMutation, analyze_plan_coverage,
        bind_dependency_collateral, relation_mutations, validate_coverage_census,
    },
    placement::{RelationActivity, RelationCaseKey, RelationCasePlan, classify_relation_cases},
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::CompilerRelationAnalysis,
};

/// One pilot's coverage across its complete feasible plan set.
struct Coverage {
    operation: OperationId,
    relations: CompilerRelationAnalysis,
    candidates: Vec<ProofPlanCandidate>,
    /// One entry per feasible proof plan.
    analyzed: Vec<(Vec<RelationCasePlan>, PlanCoverageAnalysis)>,
}

fn analyze(operation: OperationId) -> Coverage {
    let input = bound_input(&[operation]);
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates;

    assert!(!candidates.is_empty());

    let analyzed = candidates
        .iter()
        .map(|candidate| {
            let cases = crate::case::execution_cases(&relations, candidate).expect("cases");
            let plans = classify_relation_cases(&relations, &cases).expect("classification");
            let coverage = analyze_plan_coverage(&relations, &plans).expect("coverage");

            (plans, coverage)
        })
        .collect();

    Coverage {
        operation,
        relations,
        candidates,
        analyzed,
    }
}

fn pilots() -> [Coverage; 2] {
    [
        analyze(OperationId::CompactAsh),
        analyze(OperationId::TransferLive),
    ]
}

impl Coverage {
    fn relation_ids(&self) -> BTreeSet<RelationId> {
        self.relations
            .graph
            .node_weights()
            .map(|node| node.source.id.clone())
            .collect()
    }

    fn declaration(&self, relation: &RelationId) -> RelationDeclaration {
        self.relations
            .graph
            .node_weights()
            .find(|node| &node.source.id == relation)
            .expect("relation is in scope")
            .source
            .clone()
    }

    /// Every derived coverage plan across the whole plan set.
    fn plans(&self) -> impl Iterator<Item = &RelationCoveragePlan> {
        self.analyzed
            .iter()
            .flat_map(|(_, coverage)| coverage.plans())
    }

    /// The coverage plans of one relation across the whole plan set.
    fn plans_of<'a>(
        &'a self,
        relation: &'a RelationId,
    ) -> impl Iterator<Item = &'a RelationCoveragePlan> {
        self.plans().filter(move |plan| &plan.relation == relation)
    }

    /// One relation of one kind, which each pilot declares exactly one
    /// of.
    fn relation_of_kind(&self, kind: RelationKind) -> RelationId {
        let mut found = self
            .relation_ids()
            .into_iter()
            .filter(|relation| relation.kind() == kind)
            .collect::<Vec<_>>();

        assert_eq!(found.len(), 1, "{:?} declares one {kind:?}", self.operation);
        found.pop().expect("one relation")
    }

    /// The sponsor-family relation of one kind and side, which both
    /// pilots now declare.
    fn sponsor_relation(&self, kind: RelationKind, side: TransactionSide) -> RelationId {
        let relation = RelationId::new(
            self.operation,
            kind,
            RelationSubject::ObjectFamily {
                side,
                object: ObjectId::PlainLbtc,
            },
        );

        assert!(
            self.relation_ids().contains(&relation),
            "{:?} declares {kind:?} {side:?} for the sponsor family",
            self.operation,
        );
        relation
    }
}

/// The mutation classes one coverage plan requires at one boundary.
fn mutations_at(
    plan: &RelationCoveragePlan,
    boundary: CoverageBoundary,
) -> BTreeSet<RelationMutation> {
    plan.negative
        .iter()
        .filter(|requirement| requirement.id.boundary == boundary)
        .map(|requirement| requirement.mutation)
        .collect()
}

/// Every mutation class one coverage plan requires, at any boundary.
fn mutations(plan: &RelationCoveragePlan) -> BTreeSet<RelationMutation> {
    plan.negative
        .iter()
        .map(|requirement| requirement.mutation)
        .collect()
}

/// The boundaries the acceptance matrix assigns one relation.
///
/// The Guide-6 §15 matrix restated independently of the derivation, so
/// a derivation that quietly moved a relation to another boundary
/// disagrees with this rather than with itself.
fn expected_boundaries(relation: &Relation) -> BTreeSet<CoverageBoundary> {
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

// --- §11.1 exact relation-case coverage census ---

#[test]
fn every_relation_is_covered_in_every_applicable_case_of_every_plan() {
    for pilot in pilots() {
        let relations = pilot.relation_ids();

        for (plans, coverage) in &pilot.analyzed {
            let planned = plans
                .iter()
                .map(|plan| RelationCaseKey {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                })
                .collect::<BTreeSet<_>>();

            assert_eq!(coverage.keys(), planned, "{:?}", pilot.operation);

            // The census is the exact product of the relations and the
            // plan's own cases — no relation disappears because it is
            // inactive, static, structural, or externally evidenced.
            let cases = coverage
                .plans()
                .map(|plan| plan.case.clone())
                .collect::<BTreeSet<_>>();
            assert_eq!(coverage.keys().len(), relations.len() * cases.len());
        }
    }
}

#[test]
fn coverage_is_aggregated_per_operation_and_never_across_operations() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let candidate = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates
        .first()
        .expect("a feasible plan")
        .clone();
    let cases = crate::case::execution_cases(&relations, &candidate).expect("cases");
    let plans = classify_relation_cases(&relations, &cases).expect("classification");
    let coverage = analyze_plan_coverage(&relations, &plans).expect("coverage");

    assert_eq!(
        coverage.operations.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([OperationId::CompactAsh, OperationId::TransferLive]),
    );

    // Each operation analysis holds only its own relations and cases:
    // the total is a sum over operations, never a product of them.
    for (operation, analysis) in &coverage.operations {
        assert_eq!(analysis.operation, *operation);
        assert!(
            analysis
                .cases
                .iter()
                .all(|case| case.operation == *operation)
        );
        assert!(
            analysis
                .requirements
                .keys()
                .all(|key| key.relation.operation() == *operation)
        );
    }

    let total = coverage
        .operations
        .values()
        .map(|analysis| analysis.requirements.len())
        .sum::<usize>();
    assert_eq!(total, plans.len());
}

// --- §15 acceptance matrix ---

#[test]
fn every_pilot_relation_covers_exactly_its_acceptance_matrix_boundaries() {
    for pilot in pilots() {
        for relation in pilot.relation_ids() {
            let expected = expected_boundaries(&pilot.declaration(&relation).relation);
            let mut seen = 0_usize;

            for plan in pilot.plans_of(&relation) {
                seen += 1;
                assert_eq!(plan.boundaries, expected, "{relation:?}");
            }

            assert!(seen > 0, "{relation:?}");
        }
    }
}

#[test]
fn every_active_relation_case_states_a_positive_and_a_negative_at_each_boundary() {
    for pilot in pilots() {
        let mut active = 0_usize;

        for plan in pilot.plans() {
            if plan.activity != RelationActivity::Active {
                continue;
            }

            active += 1;

            for boundary in &plan.boundaries {
                assert!(
                    plan.positive
                        .iter()
                        .any(|requirement| requirement.id.boundary == *boundary),
                    "{:?} {boundary:?}",
                    plan.relation,
                );
                assert!(
                    plan.negative
                        .iter()
                        .any(|requirement| requirement.id.boundary == *boundary),
                    "{:?} {boundary:?}",
                    plan.relation,
                );
            }
        }

        assert!(active > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn each_boundary_is_answered_by_its_own_evidence_role() {
    for pilot in pilots() {
        for plan in pilot.plans() {
            for requirement in &plan.positive {
                if plan.activity == RelationActivity::Vacuous {
                    assert_eq!(requirement.role, EvidenceRole::InactiveCaseAcceptance);
                    continue;
                }

                match requirement.id.boundary {
                    CoverageBoundary::CompilerStatic => {
                        assert_eq!(requirement.role, EvidenceRole::CompilerAnalysisResult);
                    }
                    CoverageBoundary::BackendStructural => {
                        assert_eq!(requirement.role, EvidenceRole::EmittedStructure);
                    }
                    CoverageBoundary::RuntimeCarrier => {
                        assert_eq!(requirement.role, EvidenceRole::TargetExecution);
                    }
                    CoverageBoundary::ExternalEvidence => {
                        assert!(matches!(
                            requirement.role,
                            EvidenceRole::ExternalReport { .. }
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn representation_covers_a_static_selection_and_a_structural_encoding() {
    for pilot in pilots() {
        let relation = pilot.relation_of_kind(RelationKind::Representation);
        let mut seen = 0_usize;

        for plan in pilot.plans_of(&relation) {
            seen += 1;

            assert_eq!(
                mutations_at(plan, CoverageBoundary::CompilerStatic),
                BTreeSet::from([RelationMutation::UnsupportedRepresentation]),
            );
            assert_eq!(
                mutations_at(plan, CoverageBoundary::BackendStructural),
                BTreeSet::from([RelationMutation::UnauthenticatedRepresentation]),
            );

            // The plan fixed the mode, and the requirement records it
            // rather than leaving the condition implicit.
            let modes = plan
                .positive
                .iter()
                .map(|requirement| requirement.representation)
                .collect::<BTreeSet<_>>();
            assert_eq!(modes.len(), 1);
            assert!(modes.iter().all(Option::is_some));
        }

        assert!(seen > 0);
    }
}

#[test]
fn the_permissionless_path_is_covered_structurally_and_never_at_runtime() {
    let pilot = analyze(OperationId::CompactAsh);
    let relation = pilot
        .relation_ids()
        .into_iter()
        .find(|relation| {
            matches!(
                pilot.declaration(relation).relation,
                Relation::PermissionlessAuthorization,
            )
        })
        .expect("compact ASH is permissionless");
    let mut seen = 0_usize;

    for plan in pilot.plans_of(&relation) {
        seen += 1;

        assert_eq!(
            plan.boundaries,
            BTreeSet::from([CoverageBoundary::BackendStructural]),
        );
        assert_eq!(
            mutations(plan),
            BTreeSet::from([RelationMutation::UnexpectedProtocolSecret]),
        );
        assert!(
            plan.positive
                .iter()
                .all(|requirement| requirement.id.purpose == CoveragePurpose::StructuralPresence)
        );
        assert!(plan.negative.iter().all(|requirement| matches!(
            requirement.id.purpose,
            CoveragePurpose::StructuralRejection(_)
        )));
    }

    assert!(seen > 0);
}

#[test]
fn substrate_conservation_stays_external_evidence_with_its_capability_retained() {
    for pilot in pilots() {
        let relation = pilot.relation_of_kind(RelationKind::SubstrateConservation);
        let mut seen = 0_usize;

        for plan in pilot.plans_of(&relation) {
            seen += 1;

            assert_eq!(
                plan.boundaries,
                BTreeSet::from([CoverageBoundary::ExternalEvidence]),
            );
            assert!(!plan.external_evidence.is_empty());
            assert_eq!(
                mutations(plan),
                BTreeSet::from([
                    RelationMutation::ExternalEvidenceMissing,
                    RelationMutation::ExternalEvidenceFailed,
                    RelationMutation::ExternalEvidenceIdentityMismatch,
                ]),
            );

            // The capability the evidence depends on is retained, and
            // the evidence itself stays unresolved: no runtime carrier
            // and no verdict stands in for a report that does not
            // exist yet.
            for requirement in &plan.positive {
                assert!(matches!(
                    &requirement.role,
                    EvidenceRole::ExternalReport { capability, requirement: subject }
                        if *capability == RequiredCapability::WholeTransactionValueConservation
                            && plan.external_evidence.contains(subject)
                ));
                assert!(matches!(
                    requirement.id.purpose,
                    CoveragePurpose::ExternalEvidenceVerified { .. }
                ));
            }
        }

        assert!(seen > 0);
    }
}

// --- §6.5 / §11.5 conditional coverage ---

#[test]
fn sponsor_relations_are_inactive_valid_unsponsored_and_focused_rejected_when_sponsored() {
    for pilot in pilots() {
        for side in [TransactionSide::Input, TransactionSide::Output] {
            for kind in [RelationKind::Cardinality, RelationKind::Recognition] {
                let relation = pilot.sponsor_relation(kind, side);
                let mut dispositions = BTreeMap::new();

                for plan in pilot.plans_of(&relation) {
                    dispositions.insert(plan.case.sponsor, plan.activity);

                    match plan.case.sponsor {
                        SponsorCase::Absent => {
                            assert_eq!(plan.activity, RelationActivity::Vacuous);
                            assert!(plan.negative.is_empty(), "{relation:?}");
                            assert!(
                                plan.positive
                                    .iter()
                                    .any(|requirement| requirement.id.purpose
                                        == CoveragePurpose::InactiveAccept)
                            );
                        }
                        SponsorCase::Present => {
                            assert_eq!(plan.activity, RelationActivity::Active);
                            assert!(!plan.negative.is_empty(), "{relation:?}");
                            assert!(
                                plan.positive
                                    .iter()
                                    .any(|requirement| requirement.id.purpose
                                        == CoveragePurpose::ActiveAccept)
                            );
                            assert!(plan.negative.iter().all(|requirement| matches!(
                                requirement.id.purpose,
                                CoveragePurpose::FocusedReject(_)
                            )));
                        }
                    }
                }

                // The complete case set carries the conditional
                // triplet: inactive valid, and active valid beside an
                // active-invalid mutation.
                assert_eq!(
                    dispositions,
                    BTreeMap::from([
                        (SponsorCase::Absent, RelationActivity::Vacuous),
                        (SponsorCase::Present, RelationActivity::Active),
                    ]),
                    "{relation:?}",
                );
            }
        }
    }
}

#[test]
fn an_inactive_relation_case_states_exactly_one_inactive_valid_requirement() {
    for pilot in pilots() {
        let mut inactive = 0_usize;

        for plan in pilot.plans() {
            if plan.activity != RelationActivity::Vacuous {
                continue;
            }

            inactive += 1;
            assert!(plan.negative.is_empty());
            assert_eq!(plan.positive.len(), plan.boundaries.len());
            assert!(
                plan.positive
                    .iter()
                    .all(|requirement| requirement.id.purpose == CoveragePurpose::InactiveAccept)
            );
            assert!(
                plan.positive
                    .iter()
                    .all(|requirement| requirement.operands.is_empty())
            );
        }

        assert!(inactive > 0, "{:?}", pilot.operation);
    }
}

// --- §7.3 cardinality edges ---

#[test]
fn a_minimum_of_zero_requires_no_below_minimum_mutation() {
    let mut zero_minimum = 0_usize;
    let mut positive_minimum = 0_usize;

    for pilot in pilots() {
        for relation in pilot.relation_ids() {
            let Relation::Cardinality {
                minimum, maximum, ..
            } = pilot.declaration(&relation).relation
            else {
                continue;
            };

            for plan in pilot.plans_of(&relation) {
                if plan.activity != RelationActivity::Active {
                    continue;
                }

                let required = mutations(plan);
                let below = required.contains(&RelationMutation::CardinalityBelowMinimum);

                if minimum.get() == 0 {
                    zero_minimum += 1;
                    assert!(!below, "{relation:?} declares minimum zero");
                } else {
                    positive_minimum += 1;
                    assert!(below, "{relation:?} declares a positive minimum");
                }

                // Every finite maximum is exceeded, and an
                // architecture-owned one is cited by its bound rather
                // than copied as a value.
                let ceiling = match maximum {
                    CardinalityMaximum::Exact(count) => CardinalityCeiling::Declared(count),
                    CardinalityMaximum::Bound(bound) => CardinalityCeiling::Bound(bound),
                };
                assert!(
                    required.contains(&RelationMutation::CardinalityAboveMaximum { ceiling }),
                    "{relation:?}",
                );
            }
        }
    }

    // Both edges are exercised by the pilots: the repaired sponsor
    // cardinalities are the minimum-zero consumers.
    assert!(zero_minimum > 0);
    assert!(positive_minimum > 0);
}

#[test]
fn a_bound_owned_maximum_is_cited_and_never_copied_as_a_value() {
    let pilot = analyze(OperationId::CompactAsh);
    let relation = pilot.sponsor_relation(RelationKind::Cardinality, TransactionSide::Input);
    let Relation::Cardinality { maximum, .. } = pilot.declaration(&relation).relation else {
        panic!("the sponsor cardinality declares a maximum");
    };
    let CardinalityMaximum::Bound(bound) = maximum else {
        panic!("the sponsor input maximum is architecture-owned");
    };

    let mut seen = 0_usize;

    for plan in pilot.plans_of(&relation) {
        if plan.activity != RelationActivity::Active {
            continue;
        }

        seen += 1;
        assert!(
            mutations(plan).contains(&RelationMutation::CardinalityAboveMaximum {
                ceiling: CardinalityCeiling::Bound(bound),
            })
        );
    }

    assert!(seen > 0);
}

// --- §7.1 exhaustive mutation catalogue ---

/// One value of every realization relation variant.
///
/// Constructed rather than derived from the pilots: the catalogue must
/// answer for every variant the enum admits, including ones no pilot
/// declares.
fn every_relation_variant() -> Vec<Relation> {
    let predicate = ExprId::relation(
        RelationId::new(
            OperationId::CompactAsh,
            RelationKind::Cardinality,
            RelationSubject::Operation,
        ),
        ExpressionRole::Predicate,
    );

    vec![
        Relation::Cardinality {
            side: ObservedSide::Input,
            object: ObjectId::Ash,
            minimum: Count::ONE,
            maximum: CardinalityMaximum::Exact(Count::ONE),
        },
        Relation::AllowedObjectFamilies {
            side: ObservedSide::Input,
            allowed: BTreeSet::from([ObjectId::Ash]),
        },
        Relation::Recognition {
            side: ObservedSide::Input,
            object: ObjectId::Ash,
            asset: AssetId::U,
        },
        Relation::AmountConservation {
            asset: AssetId::U,
            input_objects: BTreeSet::from([ObjectId::Ash]),
            output_objects: BTreeSet::from([ObjectId::Ash]),
        },
        Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        },
        Relation::PermissionlessAuthorization,
        Relation::SponsorIsolation,
        Relation::SponsorEnvelopeMultiplicity {
            maximum: Count::ONE,
        },
        Relation::RootPolicy {
            expected: BTreeMap::new(),
        },
        Relation::ProjectionPolicy {
            expected: BTreeMap::new(),
        },
        Relation::CanonicalDeltaPolicy {
            expected: BTreeSet::new(),
        },
        Relation::OpenFlowPolicy {
            allowed: BTreeSet::new(),
        },
        Relation::Constructibility {
            class: ConstructibilityClass::PublicPermissionless,
        },
        Relation::Representation {
            object: ObjectId::Ash,
            allowed: BTreeSet::from([RepresentationMode::Explicit]),
        },
        Relation::LifecycleExit {
            object: ObjectId::Ash,
            exit: OperationId::CompactAsh,
        },
        Relation::ExpressionPredicate {
            expression: predicate,
        },
        Relation::SubstrateConservation {
            asset: AssetId::Lbtc,
        },
    ]
}

#[test]
fn every_relation_variant_requires_at_least_one_mutation_class() {
    for relation in every_relation_variant() {
        let required = relation_mutations(&relation);

        assert!(!required.is_empty(), "{relation:?}");

        // Every mutation is stated at a boundary the acceptance matrix
        // gives the relation, so no negative can be answered by an
        // artifact that never observes the relation.
        let boundaries = expected_boundaries(&relation);

        for (boundary, _) in &required {
            assert!(boundaries.contains(boundary), "{relation:?} {boundary:?}");
        }
    }
}

#[test]
fn a_permissionless_constructibility_class_adds_a_private_dependency_mutation() {
    let classes = |class| {
        relation_mutations(&Relation::Constructibility { class })
            .into_iter()
            .map(|(_, mutation)| mutation)
            .collect::<BTreeSet<_>>()
    };
    let permissionless = classes(ConstructibilityClass::PublicPermissionless);
    let owners = classes(ConstructibilityClass::OwnersOf {
        object: ObjectId::ReceiptLive,
    });

    assert!(permissionless.contains(&RelationMutation::PermissionlessPrivateDependency));
    assert!(!owners.contains(&RelationMutation::PermissionlessPrivateDependency));
    assert!(owners.contains(&RelationMutation::ConstructibilityWitnessUnavailable));
    assert!(permissionless.contains(&RelationMutation::ConstructibilityWitnessUnavailable));
}

// --- §7.4 dependency-collateral policy ---

#[test]
fn a_runtime_negative_requires_its_dependency_closure_and_never_claims_isolation() {
    for pilot in pilots() {
        for plan in pilot.plans() {
            for requirement in &plan.negative {
                let expected = match requirement.id.boundary {
                    CoverageBoundary::RuntimeCarrier => {
                        CollateralPolicy::RequireIntendedAndDependencyClosure
                    }
                    CoverageBoundary::CompilerStatic
                    | CoverageBoundary::BackendStructural
                    | CoverageBoundary::ExternalEvidence => CollateralPolicy::ReportAdditional,
                };

                assert_eq!(requirement.collateral.policy, expected);
                // Not yet derived — never "proved independent".
                assert!(requirement.collateral.dependency_closure.is_empty());
            }
        }
    }
}

#[test]
fn the_dependency_collateral_hook_fills_every_runtime_negative() {
    let pilot = analyze(OperationId::CompactAsh);
    let (_, coverage) = pilot.analyzed.first().expect("a plan").clone();
    let mut bound = coverage.clone();
    let sentinel = coverage
        .keys()
        .into_iter()
        .next()
        .expect("a covered relation-case");

    bind_dependency_collateral(&mut bound, &|_key| BTreeSet::from([sentinel.clone()]));

    let mut filled = 0_usize;

    for plan in bound.plans() {
        for requirement in &plan.negative {
            match requirement.collateral.policy {
                CollateralPolicy::RequireIntendedAndDependencyClosure => {
                    filled += 1;
                    assert_eq!(
                        requirement.collateral.dependency_closure,
                        BTreeSet::from([sentinel.clone()]),
                    );
                }
                CollateralPolicy::ReportAdditional => {
                    assert!(requirement.collateral.dependency_closure.is_empty());
                }
            }
        }
    }

    assert!(filled > 0);
}

// --- §14 census rejections ---

/// One pilot analysis with its plans, ready to be corrupted.
fn corruptible() -> (Vec<RelationCasePlan>, PlanCoverageAnalysis) {
    analyze(OperationId::CompactAsh)
        .analyzed
        .first()
        .expect("a plan")
        .clone()
}

/// One relation-case key of the analysis matching a predicate.
fn key_where(
    coverage: &PlanCoverageAnalysis,
    predicate: impl Fn(&RelationCoveragePlan) -> bool,
) -> RelationCaseKey {
    coverage
        .plans()
        .find(|plan| predicate(plan))
        .expect("a matching coverage plan")
        .key()
}

fn plan_mut<'a>(
    coverage: &'a mut PlanCoverageAnalysis,
    key: &RelationCaseKey,
) -> &'a mut RelationCoveragePlan {
    coverage
        .operations
        .get_mut(&key.relation.operation())
        .expect("the operation is analyzed")
        .requirements
        .get_mut(key)
        .expect("the relation-case is covered")
}

#[test]
fn a_duplicated_coverage_requirement_is_rejected() {
    let (plans, mut coverage) = corruptible();
    let key = key_where(&coverage, |plan| !plan.positive.is_empty());
    let plan = plan_mut(&mut coverage, &key);
    let duplicate = plan.positive[0].clone();
    let id = duplicate.id.clone();

    plan.positive.push(duplicate);

    assert_eq!(
        validate_coverage_census(&plans, &coverage),
        Err(CompileError::DuplicateCoverageRequirement { requirement: id }),
    );
}

#[test]
fn a_missing_relation_case_is_rejected() {
    let (plans, mut coverage) = corruptible();
    let key = key_where(&coverage, |_| true);

    coverage
        .operations
        .get_mut(&key.relation.operation())
        .expect("the operation is analyzed")
        .requirements
        .remove(&key);

    assert_eq!(
        validate_coverage_census(&plans, &coverage),
        Err(CompileError::CoverageCensusMismatch {
            missing: vec![key],
            unexpected: Vec::new(),
        }),
    );
}

#[test]
fn an_unexpected_relation_case_is_rejected() {
    let (mut plans, coverage) = corruptible();
    let key = key_where(&coverage, |_| true);

    plans.retain(|plan| plan.relation != key.relation || plan.case != key.case);

    assert_eq!(
        validate_coverage_census(&plans, &coverage),
        Err(CompileError::CoverageCensusMismatch {
            missing: Vec::new(),
            unexpected: vec![key],
        }),
    );
}

#[test]
fn an_active_relation_case_without_positive_coverage_is_rejected() {
    let (plans, mut coverage) = corruptible();
    let key = key_where(&coverage, |plan| {
        plan.activity == RelationActivity::Active
            && plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier)
    });

    plan_mut(&mut coverage, &key).positive.clear();

    assert_eq!(
        validate_coverage_census(&plans, &coverage),
        Err(CompileError::MissingPositiveCoverage {
            relation: key.relation,
            case: key.case,
            boundary: CoverageBoundary::RuntimeCarrier,
        }),
    );
}

#[test]
fn an_active_relation_case_without_negative_coverage_is_rejected() {
    let (plans, mut coverage) = corruptible();
    let key = key_where(&coverage, |plan| {
        plan.activity == RelationActivity::Active
            && plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier)
    });

    plan_mut(&mut coverage, &key).negative.clear();

    assert_eq!(
        validate_coverage_census(&plans, &coverage),
        Err(CompileError::MissingNegativeCoverage {
            relation: key.relation,
            case: key.case,
            boundary: CoverageBoundary::RuntimeCarrier,
        }),
    );
}

#[test]
fn an_inactive_relation_case_without_inactive_coverage_is_rejected() {
    let (plans, mut coverage) = corruptible();
    let key = key_where(&coverage, |plan| plan.activity == RelationActivity::Vacuous);

    plan_mut(&mut coverage, &key).positive.clear();

    assert_eq!(
        validate_coverage_census(&plans, &coverage),
        Err(CompileError::MissingInactiveCoverage {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn an_inactive_relation_case_demanding_a_rejection_is_rejected() {
    let (plans, mut coverage) = corruptible();
    let key = key_where(&coverage, |plan| plan.activity == RelationActivity::Vacuous);

    plan_mut(&mut coverage, &key)
        .negative
        .push(NegativeCoverageRequirement {
            id: CoverageRequirementId {
                relation: key.relation.clone(),
                case: key.case.clone(),
                boundary: CoverageBoundary::RuntimeCarrier,
                purpose: CoveragePurpose::FocusedReject(RelationMutation::WrongRecognizedAsset),
            },
            role: EvidenceRole::TargetExecution,
            mutation: RelationMutation::WrongRecognizedAsset,
            collateral: CollateralRequirement {
                policy: CollateralPolicy::RequireIntendedAndDependencyClosure,
                dependency_closure: BTreeSet::new(),
            },
        });

    assert_eq!(
        validate_coverage_census(&plans, &coverage),
        Err(CompileError::UnexpectedCoverageRejection {
            relation: key.relation,
            case: key.case,
        }),
    );
}

// --- §16.3 determinism ---

#[test]
fn repeated_coverage_analysis_is_equal() {
    for pilot in pilots() {
        for (plans, coverage) in &pilot.analyzed {
            let again = analyze_plan_coverage(&pilot.relations, plans).expect("second analysis");

            assert_eq!(&again, coverage, "{:?}", pilot.operation);
        }
    }
}

#[test]
fn a_permuted_relation_case_plan_order_is_covered_equally() {
    for pilot in pilots() {
        for (plans, coverage) in &pilot.analyzed {
            let mut permuted = plans.clone();
            permuted.reverse();

            let again =
                analyze_plan_coverage(&pilot.relations, &permuted).expect("permuted analysis");

            assert_eq!(&again, coverage, "{:?}", pilot.operation);
        }
    }
}

#[test]
fn every_feasible_proof_plan_is_covered() {
    for pilot in pilots() {
        assert_eq!(pilot.analyzed.len(), pilot.candidates.len());

        for (_, coverage) in &pilot.analyzed {
            assert!(!coverage.operations.is_empty());
        }
    }
}
