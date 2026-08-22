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
    carrier::{CarrierQuantification, CarrierRole, relation_case_eligibility},
    case::{ConservedAmountVisibility, SponsorCase, conserved_amount_visibility},
    coverage::{
        CardinalityCeiling, CarrierAssignmentAlternative, CarrierCoverageRequirement,
        CollateralPolicy, CollateralRequirement, CoverageBoundary, CoveragePurpose,
        CoverageRequirementId, EvidenceRole, NegativeCoverageRequirement, PlanCoverageAnalysis,
        PositiveCoverageRequirement, ProjectionSubject, RelationCoveragePlan, RelationMutation,
        analyze_placed_coverage, analyze_plan_coverage, bind_dependency_collateral,
        coverage_names_sponsor_value, relation_mutations, validate_conditional_coverage,
        validate_coverage_census, validate_placement_coverage, validate_representation_coverage,
    },
    layout::LayoutRequirement,
    placement::{
        PlacedCarrier, PlacedProofPlanCandidate, PlacementSearchLimits, RelationActivity,
        RelationCaseKey, RelationCasePlan, classify_relation_cases, place_feasible_proof_plans,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::CompilerRelationAnalysis,
    source::{OperandId, OperandRole},
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

    assert_ne!(candidates, [] as [ProofPlanCandidate; 0]);

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
fn expected_boundaries(
    relation: &Relation,
    amounts: ConservedAmountVisibility,
) -> BTreeSet<CoverageBoundary> {
    use CoverageBoundary as Boundary;

    if let (Relation::AmountConservation { .. }, ConservedAmountVisibility::Committed) =
        (relation, amounts)
    {
        return BTreeSet::from([Boundary::ExternalEvidence]);
    }

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
            let declaration = pilot.declaration(&relation);
            let mut seen = 0_usize;

            for plan in pilot.plans_of(&relation) {
                seen += 1;

                // The expectation is re-derived per case rather than
                // once per relation: conservation lands on a different
                // boundary under a committed representation, and a
                // per-relation expectation would have to be wrong for
                // one of the two cases.
                let amounts = conserved_amount_visibility(&declaration, &plan.case)
                    .expect("a pilot case fixes one representation per family");

                assert_eq!(
                    plan.boundaries,
                    expected_boundaries(&declaration.relation, amounts),
                    "{relation:?} {:?}",
                    plan.case,
                );
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
            assert_eq!(plan.negative, [] as [NegativeCoverageRequirement; 0]);
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
    // Both visibilities, because conservation is the one relation whose
    // discharge the representation moves: checking only the readable
    // answer would leave the committed one free to require nothing.
    for amounts in [
        ConservedAmountVisibility::Readable,
        ConservedAmountVisibility::Committed,
    ] {
        for relation in every_relation_variant() {
            let required = relation_mutations(&relation, amounts);

            assert!(!required.is_empty(), "{relation:?} {amounts:?}");

            // Every mutation is stated at a boundary the acceptance
            // matrix gives the relation, so no negative can be answered
            // by an artifact that never observes the relation.
            let boundaries = expected_boundaries(&relation, amounts);

            for (boundary, _) in &required {
                assert!(
                    boundaries.contains(boundary),
                    "{relation:?} {amounts:?} {boundary:?}",
                );
            }
        }
    }
}

#[test]
fn committed_conservation_moves_its_negatives_to_the_evidence_boundary() {
    let relation = Relation::AmountConservation {
        asset: AssetId::U,
        input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
        output_objects: BTreeSet::from([ObjectId::ReceiptLive]),
    };

    let boundaries = |amounts| {
        relation_mutations(&relation, amounts)
            .into_iter()
            .map(|(boundary, _)| boundary)
            .collect::<BTreeSet<_>>()
    };

    // The whole §19.4 divergence in one comparison: one relation, two
    // representations, two disjoint boundaries. An implementation that
    // kept the runtime mutation as well would still satisfy a
    // containment check, so the sets are compared exactly.
    assert_eq!(
        boundaries(ConservedAmountVisibility::Readable),
        BTreeSet::from([CoverageBoundary::RuntimeCarrier]),
    );
    assert_eq!(
        boundaries(ConservedAmountVisibility::Committed),
        BTreeSet::from([CoverageBoundary::ExternalEvidence]),
    );

    // The committed negatives are the report's own three, matching the
    // substrate obligation discharged at the same boundary.
    let committed = relation_mutations(&relation, ConservedAmountVisibility::Committed)
        .into_iter()
        .map(|(_, mutation)| mutation)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        committed,
        BTreeSet::from([
            RelationMutation::ExternalEvidenceMissing,
            RelationMutation::ExternalEvidenceFailed,
            RelationMutation::ExternalEvidenceIdentityMismatch,
        ]),
    );
}

#[test]
fn a_permissionless_constructibility_class_adds_a_private_dependency_mutation() {
    let classes = |class| {
        relation_mutations(
            &Relation::Constructibility { class },
            ConservedAmountVisibility::Readable,
        )
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

// --- Tranches F and G: carriers, layout, and projections ---

/// One pilot placed end to end, with the coverage of each placed plan.
struct Placed {
    operation: OperationId,
    relations: CompilerRelationAnalysis,
    candidates: Vec<PlacedProofPlanCandidate>,
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

fn place(operation: OperationId) -> Placed {
    let input = bound_input(&[operation]);
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates;
    let placed = place_feasible_proof_plans(&relations, &candidates, limits())
        .expect("placement analysis")
        .placed;
    let coverage = placed
        .iter()
        .map(|entry| analyze_placed_coverage(&relations, entry).expect("placed coverage"))
        .collect();

    Placed {
        operation,
        relations,
        candidates: placed,
        coverage,
    }
}

fn placed_pilots() -> [Placed; 2] {
    [
        place(OperationId::CompactAsh),
        place(OperationId::TransferLive),
    ]
}

impl Placed {
    fn plans(&self) -> impl Iterator<Item = &RelationCoveragePlan> {
        self.coverage.iter().flat_map(PlanCoverageAnalysis::plans)
    }

    fn pairs(&self) -> impl Iterator<Item = (&PlacedProofPlanCandidate, &PlanCoverageAnalysis)> {
        self.candidates.iter().zip(self.coverage.iter())
    }
}

// --- §9.2 exact assignment alternatives ---

#[test]
fn carrier_coverage_compresses_the_feasible_placement_product() {
    for pilot in placed_pilots() {
        for (entry, coverage) in pilot.pairs() {
            // The plan really does have a large combined placement set:
            // compression is only meaningful because it is.
            assert!(
                entry.feasible_placements.len() > 100,
                "{:?} placed {} times",
                pilot.operation,
                entry.feasible_placements.len(),
            );

            // The alternatives recomputed by a raw scan over every
            // feasible placement, independently of the compression.
            let mut raw: BTreeMap<RelationCaseKey, BTreeSet<BTreeSet<PlacedCarrier>>> =
                BTreeMap::new();

            for placement in &entry.feasible_placements {
                for assignment in &placement.assignments {
                    raw.entry(assignment.key())
                        .or_default()
                        .insert(assignment.carriers.iter().cloned().collect());
                }
            }

            let compressed = coverage
                .plans()
                .filter_map(|plan| {
                    plan.carrier.as_ref().map(|carrier| {
                        (
                            plan.key(),
                            carrier
                                .allowed_assignments
                                .iter()
                                .map(|alternative| alternative.carriers.clone())
                                .collect::<BTreeSet<_>>(),
                        )
                    })
                })
                .collect::<BTreeMap<_, _>>();

            assert_eq!(compressed, raw, "{:?}", pilot.operation);

            // Every relation-case admits a handful of alternatives, not
            // a share of the combined product. Measured on the pilots:
            // 216 feasible placements compress to 39 alternatives over
            // 30 carried relation-cases for compact ASH, and to 41 over
            // 32 for live transfer, with no relation-case offering more
            // than three.
            let widest = raw
                .values()
                .map(BTreeSet::len)
                .max()
                .expect("a carried relation-case");
            let total = raw.values().map(BTreeSet::len).sum::<usize>();

            assert!(
                widest <= 4,
                "{:?} widest relation-case has {widest} alternatives",
                pilot.operation,
            );
            assert!(
                total < entry.feasible_placements.len(),
                "{:?} kept {total} alternatives",
                pilot.operation,
            );
        }
    }
}

#[test]
fn carrier_coverage_preserves_multiplicity() {
    // An every-member obligation stays quantified: a per-member role
    // and a complete-family proof are distinct alternatives, and
    // neither is deduplicated into the other.
    let pilot = place(OperationId::TransferLive);
    let owner = pilot
        .plans()
        .find(|plan| plan.relation.kind() == RelationKind::Authorization)
        .expect("live transfer authorizes owners")
        .relation
        .clone();
    let mut owners = 0_usize;
    let mut recognitions = 0_usize;

    for plan in pilot.plans() {
        let Some(carrier) = &plan.carrier else {
            continue;
        };

        if plan.relation == owner {
            owners += 1;

            // Owner authorization admits the per-member role only: no
            // coordinator may stand in for every owner.
            for alternative in &carrier.allowed_assignments {
                assert_eq!(
                    alternative.carriers,
                    BTreeSet::from([PlacedCarrier {
                        carrier: CarrierRole::EveryInputFamilyMember {
                            object: ObjectId::ReceiptLive,
                        },
                        quantification: CarrierQuantification::PerMember,
                    }]),
                );
            }
        }

        let quantifications = carrier
            .allowed_assignments
            .iter()
            .flat_map(|alternative| alternative.carriers.iter())
            .map(|placed| placed.quantification)
            .collect::<BTreeSet<_>>();

        if quantifications.contains(&CarrierQuantification::CompleteFamilyProof) {
            recognitions += 1;
            assert!(
                quantifications.contains(&CarrierQuantification::PerMember),
                "{:?} lost its per-member alternative",
                plan.relation,
            );
        }
    }

    assert!(owners > 0);
    // Input recognition is the obligation a complete-family proof may
    // discharge, and it kept both alternatives.
    assert!(recognitions > 0);
}

// --- §9.4 no runtime carrier for a non-runtime relation ---

#[test]
fn only_active_runtime_relation_cases_carry_a_carrier_requirement() {
    for pilot in placed_pilots() {
        let mut carried = 0_usize;
        let mut uncarried = 0_usize;

        for plan in pilot.plans() {
            let runtime = plan.activity == RelationActivity::Active
                && plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier);

            if runtime {
                carried += 1;
                assert!(plan.carrier.is_some(), "{:?}", plan.relation);
            } else {
                uncarried += 1;
                assert!(plan.carrier.is_none(), "{:?}", plan.relation);
            }
        }

        assert!(carried > 0, "{:?}", pilot.operation);
        assert!(uncarried > 0, "{:?}", pilot.operation);
    }
}

/// One placed pilot plan with everything a placement validation needs.
fn placed_case() -> (
    PlacedProofPlanCandidate,
    CompilerRelationAnalysis,
    PlanCoverageAnalysis,
) {
    let pilot = place(OperationId::CompactAsh);
    let entry = pilot.candidates.first().expect("a placed plan").clone();
    let coverage = pilot.coverage.first().expect("its coverage").clone();

    (entry, pilot.relations, coverage)
}

fn revalidate(
    entry: &PlacedProofPlanCandidate,
    relations: &CompilerRelationAnalysis,
    coverage: &PlanCoverageAnalysis,
) -> Result<(), CompileError> {
    let eligibility =
        relation_case_eligibility(relations, &entry.relation_case_plans).expect("eligibility");

    validate_placement_coverage(
        &eligibility,
        &entry.feasible_placements,
        &entry.layout_requirements,
        coverage,
    )
}

#[test]
fn a_non_runtime_relation_case_with_a_runtime_carrier_is_rejected() {
    let (entry, relations, mut coverage) = placed_case();
    let key = key_where(&coverage, |plan| plan.carrier.is_none());

    plan_mut(&mut coverage, &key).carrier = Some(CarrierCoverageRequirement {
        relation_case: key.clone(),
        allowed_assignments: BTreeSet::from([CarrierAssignmentAlternative {
            carriers: BTreeSet::from([PlacedCarrier {
                carrier: CarrierRole::OperationGlobal {
                    operation: OperationId::CompactAsh,
                    anchor: ObjectId::Ash,
                },
                quantification: CarrierQuantification::Single,
            }]),
            layout: BTreeSet::new(),
        }]),
    });

    assert_eq!(
        revalidate(&entry, &relations, &coverage),
        Err(CompileError::UnexpectedRuntimeCarrierCoverage {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn an_active_runtime_relation_case_without_a_carrier_is_rejected() {
    let (entry, relations, mut coverage) = placed_case();
    let key = key_where(&coverage, |plan| plan.carrier.is_some());

    plan_mut(&mut coverage, &key).carrier = None;

    assert_eq!(
        revalidate(&entry, &relations, &coverage),
        Err(CompileError::MissingCarrierCoverage {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn an_active_runtime_relation_case_without_a_projection_is_rejected() {
    let (entry, relations, mut coverage) = placed_case();
    let key = key_where(&coverage, |plan| plan.carrier.is_some());

    plan_mut(&mut coverage, &key)
        .projections
        .remove(&CoverageBoundary::RuntimeCarrier);

    assert_eq!(
        revalidate(&entry, &relations, &coverage),
        Err(CompileError::MissingProjectionCoverage {
            relation: key.relation,
            case: key.case,
        }),
    );
}

// --- §11.3 layout binding ---

#[test]
fn every_selected_assignment_layout_dependency_is_referenced_by_coverage() {
    for pilot in placed_pilots() {
        for (entry, coverage) in pilot.pairs() {
            let stated = entry.layout_requirements.iter().collect::<BTreeSet<_>>();
            let mut referenced = 0_usize;

            for plan in coverage.plans() {
                let Some(carrier) = &plan.carrier else {
                    continue;
                };

                for alternative in &carrier.allowed_assignments {
                    for requirement in &alternative.layout {
                        referenced += 1;
                        // Coverage references only what the plan's own
                        // census states.
                        assert!(stated.contains(requirement), "{requirement:?}");
                    }
                }
            }

            assert!(referenced > 0, "{:?}", pilot.operation);

            // And every placement's own dependency is covered.
            for placement in &entry.feasible_placements {
                for assignment in &placement.assignments {
                    let plan = coverage
                        .plan(&assignment.key())
                        .expect("the placed relation-case is covered");
                    let carrier = plan.carrier.as_ref().expect("it is carried");
                    let carriers = assignment.carriers.iter().cloned().collect::<BTreeSet<_>>();

                    assert!(
                        carrier
                            .allowed_assignments
                            .iter()
                            .any(|alternative| alternative.carriers == carriers),
                    );
                }
            }
        }
    }
}

#[test]
fn an_unexpected_layout_reference_is_rejected() {
    let (entry, relations, mut coverage) = placed_case();
    let key = key_where(&coverage, |plan| plan.carrier.is_some());
    let unexpected = LayoutRequirement::SecretFreeOperationPath {
        relation: key.relation.clone(),
        case: key.case.clone(),
    };

    assert!(!entry.layout_requirements.contains(&unexpected));

    let carrier = plan_mut(&mut coverage, &key)
        .carrier
        .as_mut()
        .expect("a carried relation-case");
    let mut alternative = carrier
        .allowed_assignments
        .iter()
        .next()
        .expect("an alternative")
        .clone();

    alternative.layout.insert(unexpected);
    carrier.allowed_assignments.insert(alternative);

    assert_eq!(
        revalidate(&entry, &relations, &coverage),
        Err(CompileError::UnexpectedCoverageLayoutRequirement {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn a_dropped_assignment_alternative_is_rejected() {
    let (entry, relations, mut coverage) = placed_case();
    let key = key_where(&coverage, |plan| {
        plan.carrier
            .as_ref()
            .is_some_and(|carrier| carrier.allowed_assignments.len() > 1)
    });
    let carrier = plan_mut(&mut coverage, &key)
        .carrier
        .as_mut()
        .expect("a carried relation-case");
    let dropped = carrier
        .allowed_assignments
        .iter()
        .next()
        .expect("an alternative")
        .clone();

    carrier.allowed_assignments.remove(&dropped);

    assert_eq!(
        revalidate(&entry, &relations, &coverage),
        Err(CompileError::MissingCoverageLayoutRequirement {
            relation: key.relation,
            case: key.case,
        }),
    );
}

// --- §10 accepted semantic projections ---

#[test]
fn each_boundary_projects_its_own_subject() {
    for pilot in placed_pilots() {
        let mut seen = BTreeSet::new();

        for plan in pilot.plans() {
            for (boundary, requirement) in &plan.projections {
                assert_eq!(requirement.boundary, *boundary);
                assert_eq!(requirement.relation, plan.relation);
                assert_eq!(requirement.case, plan.case);
                seen.insert(*boundary);

                match boundary {
                    CoverageBoundary::RuntimeCarrier => {
                        assert_eq!(
                            requirement.subject,
                            ProjectionSubject::RuntimeRelationVerdict
                        );
                        assert!(requirement.compare_relation_verdict);
                        assert!(!requirement.operands.is_empty(), "{:?}", plan.relation);
                    }
                    CoverageBoundary::CompilerStatic => {
                        assert_eq!(
                            requirement.subject,
                            ProjectionSubject::CompilerSelectionResult,
                        );
                        assert!(!requirement.compare_relation_verdict);
                        assert_eq!(
                            requirement.sources,
                            [] as [crate::source::SourceRequirement; 0]
                        );
                    }
                    CoverageBoundary::BackendStructural => {
                        assert_eq!(
                            requirement.subject,
                            ProjectionSubject::EmittedStructuralFact
                        );
                        assert!(!requirement.compare_relation_verdict);
                        assert_eq!(
                            requirement.sources,
                            [] as [crate::source::SourceRequirement; 0]
                        );
                    }
                    CoverageBoundary::ExternalEvidence => {
                        assert_eq!(
                            requirement.subject,
                            ProjectionSubject::ExternalReportSubjects {
                                requirements: plan.external_evidence.clone(),
                            },
                        );
                        assert!(!requirement.compare_relation_verdict);
                    }
                }
            }
        }

        // Every boundary the pilots use really does project.
        assert_eq!(
            seen,
            BTreeSet::from([
                CoverageBoundary::CompilerStatic,
                CoverageBoundary::BackendStructural,
                CoverageBoundary::RuntimeCarrier,
                CoverageBoundary::ExternalEvidence,
            ]),
            "{:?}",
            pilot.operation,
        );
    }
}

#[test]
fn an_accepted_projection_is_answered_by_the_evidence_role_of_its_boundary() {
    // A projection comparison is answered by whatever answers the
    // boundary: a compiler-static selection result, an emitted
    // structural fact, or a typed report. Claiming target execution for
    // one of those would report an obligation no target evaluates as one
    // it did.
    for pilot in placed_pilots() {
        let mut checked = BTreeSet::new();

        for plan in pilot.plans() {
            for requirement in &plan.positive {
                if requirement.id.purpose != CoveragePurpose::AcceptedProjection {
                    continue;
                }

                checked.insert(requirement.id.boundary);

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
                            &requirement.role,
                            EvidenceRole::ExternalReport { requirement: subject, .. }
                                if plan.external_evidence.contains(subject)
                        ));
                    }
                }
            }
        }

        assert_eq!(
            checked,
            BTreeSet::from([
                CoverageBoundary::CompilerStatic,
                CoverageBoundary::BackendStructural,
                CoverageBoundary::RuntimeCarrier,
                CoverageBoundary::ExternalEvidence,
            ]),
            "{:?}",
            pilot.operation,
        );
    }
}

#[test]
fn a_runtime_projection_carries_the_source_rows_its_carrier_receives() {
    for pilot in placed_pilots() {
        let mut compared = 0_usize;

        for plan in pilot.plans() {
            let Some(requirement) = plan.projections.get(&CoverageBoundary::RuntimeCarrier) else {
                continue;
            };

            if requirement.sources.is_empty() {
                continue;
            }

            compared += 1;

            // Canonical and free of duplicates.
            let mut canonical = requirement.sources.clone();
            canonical.sort();
            canonical.dedup();
            assert_eq!(canonical, requirement.sources);

            // Every row belongs to the relation being projected.
            for source in &requirement.sources {
                assert_eq!(source.operand.relation(), &plan.relation);
            }
        }

        assert!(compared > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn an_inactive_relation_case_projects_nothing() {
    for pilot in placed_pilots() {
        let mut inactive = 0_usize;

        for plan in pilot.plans() {
            if plan.activity != RelationActivity::Vacuous {
                continue;
            }

            inactive += 1;
            assert!(plan.projections.is_empty(), "{:?}", plan.relation);
            assert!(plan.carrier.is_none());
        }

        assert!(inactive > 0, "{:?}", pilot.operation);
    }
}

// --- §10.2 / §15.3 sponsor opacity ---

#[test]
fn no_pilot_coverage_names_an_erased_sponsor_value() {
    for pilot in placed_pilots() {
        for coverage in &pilot.coverage {
            assert!(
                !coverage_names_sponsor_value(coverage),
                "{:?}",
                pilot.operation
            );
        }

        // The guard is not vacuous: the sponsor family is present in
        // coverage as membership, isolation, and multiplicity.
        let sponsor = pilot
            .plans()
            .filter(|plan| {
                matches!(
                    plan.relation.subject(),
                    RelationSubject::ObjectFamily {
                        object: ObjectId::PlainLbtc,
                        ..
                    } | RelationSubject::Sponsor,
                )
            })
            .count();
        assert!(sponsor > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn a_sponsor_amount_operand_in_coverage_is_rejected() {
    let (entry, relations, mut coverage) = placed_case();
    let key = key_where(&coverage, |plan| plan.carrier.is_some());
    let sponsor_amount = OperandId::new(
        key.relation.clone(),
        OperandRole::ObjectFamilyAmount {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    );

    plan_mut(&mut coverage, &key)
        .positive
        .push(PositiveCoverageRequirement {
            id: CoverageRequirementId {
                relation: key.relation.clone(),
                case: key.case.clone(),
                boundary: CoverageBoundary::RuntimeCarrier,
                purpose: CoveragePurpose::AcceptedProjection,
            },
            role: EvidenceRole::TargetExecution,
            representation: None,
            operands: vec![sponsor_amount],
        });

    assert!(coverage_names_sponsor_value(&coverage));
    assert_eq!(
        revalidate(&entry, &relations, &coverage),
        Err(CompileError::SponsorValueRead),
    );
}

// --- §11.5 / §11.6 plan-set censuses ---

#[test]
fn the_conditional_triplet_holds_across_the_complete_case_set() {
    for pilot in placed_pilots() {
        validate_conditional_coverage(&pilot.coverage).expect("conditional coverage");
    }
}

#[test]
fn a_conditional_relation_with_no_active_case_is_rejected() {
    let pilot = place(OperationId::CompactAsh);
    let mut coverage = pilot.coverage;

    // Drop every sponsored case, leaving the sponsor relations inactive
    // everywhere: the triplet is then incomplete.
    for analysis in &mut coverage {
        for operation in analysis.operations.values_mut() {
            operation
                .requirements
                .retain(|key, _| key.case.sponsor == SponsorCase::Absent);
        }
    }

    assert!(matches!(
        validate_conditional_coverage(&coverage),
        Err(CompileError::MissingPositiveCoverage { .. }),
    ));
}

#[test]
fn the_representation_census_is_covered_across_the_complete_plan_set() {
    for pilot in placed_pilots() {
        validate_representation_coverage(&pilot.relations, &pilot.coverage)
            .expect("representation census");

        // Both modes of the pilot really are covered, one per plan.
        let modes = pilot
            .plans()
            .filter(|plan| plan.relation.kind() == RelationKind::Representation)
            .flat_map(|plan| plan.positive.iter())
            .filter_map(|requirement| requirement.representation)
            .collect::<BTreeSet<_>>();
        let expected = match pilot.operation {
            OperationId::TransferLive => BTreeSet::from([
                RepresentationMode::Explicit,
                RepresentationMode::PrivateCommitted,
            ]),
            _ => BTreeSet::from([
                RepresentationMode::Explicit,
                RepresentationMode::PublicCommitted,
            ]),
        };

        assert_eq!(modes, expected, "{:?}", pilot.operation);
    }
}

#[test]
fn a_representation_mode_omitted_from_the_plan_set_is_rejected() {
    let pilot = place(OperationId::CompactAsh);
    let single = vec![pilot.coverage.first().expect("a plan").clone()];

    assert!(matches!(
        validate_representation_coverage(&pilot.relations, &single),
        Err(CompileError::MissingRepresentationCoverage { .. }),
    ));
}

// --- §16 stable projection and determinism ---

#[test]
fn repeated_placed_coverage_is_equal_and_projects_equally() {
    for pilot in placed_pilots() {
        for (entry, coverage) in pilot.pairs() {
            let again = analyze_placed_coverage(&pilot.relations, entry).expect("second analysis");

            assert_eq!(&again, coverage, "{:?}", pilot.operation);
            assert_eq!(again.project(), coverage.project());
        }
    }
}

#[test]
fn a_permuted_placement_order_is_covered_equally() {
    for pilot in placed_pilots() {
        for (entry, coverage) in pilot.pairs() {
            let mut permuted = entry.clone();

            permuted.feasible_placements.reverse();
            permuted.relation_case_plans.reverse();

            let again =
                analyze_placed_coverage(&pilot.relations, &permuted).expect("permuted analysis");

            assert_eq!(&again, coverage, "{:?}", pilot.operation);
            assert_eq!(again.project(), coverage.project());
        }
    }
}

#[test]
fn the_projection_excludes_the_combined_placement_product() {
    for pilot in placed_pilots() {
        for (entry, coverage) in pilot.pairs() {
            let projection = coverage.project();

            // One projected entry per relation-case, however many
            // whole-transaction placements the plan admits.
            assert_eq!(projection.requirements.len(), coverage.keys().len());
            assert!(projection.requirements.len() < entry.feasible_placements.len());
        }
    }
}
