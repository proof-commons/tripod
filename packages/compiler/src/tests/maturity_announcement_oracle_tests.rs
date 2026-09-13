//! Independent announcement census: every expected identity, body, proof and edge
//! is literal test data. Production builders construct only the subject under test.
//! Corruptions must disagree with this census and be refused by production.

use super::announcement_input;
use crate::{
    CompileError,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    analyzed_operation::analyze_candidate_operations,
    analyzed_validate::{AnalyzedExpectations, derive_expectations, validate_against_expectations},
    case::{ExecutionCaseId, SponsorCase},
    constructibility::build_constructibility_analysis,
    coverage::CollateralPolicy,
    coverage_graph::{CoverageEdge, CoverageNodeId},
    lifecycle::build_lifecycle_analysis,
    maturity_announcement_plan::{
        ValidatedMaturityAnnouncementOperationPlan, plan_maturity_announcement_target_operation,
    },
    placement::{DischargeBoundary, PlacementSearchLimits, RelationActivity, RelationCaseKey},
    relation::build_relation_graph,
    requirement::{ProofDisposition, relation_requirements},
};
use architecture::{
    AssetId, BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId, ProjectionRule, RootId,
    RootUse,
};
use realization::{
    CardinalityMaximum, ConstructibilityClass, Count, ExternalEvidenceRequirement, ObservedSide,
    ProofAlternativeId, ProofKind, Relation, RelationDeclaration, RelationDependencyProjection,
    RelationEdge, RelationGraphProjection, RelationId, RelationKind, RelationSubject,
    RepresentationMode, TransactionSide,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

const OP: OperationId = OperationId::AnnounceMaturity;
const MODES: [RepresentationMode; 2] = [
    RepresentationMode::Explicit,
    RepresentationMode::PublicCommitted,
];
const EXITS: [OperationId; 6] = [
    OperationId::AdmitDeposits,
    OperationId::Cycle,
    OperationId::Redeem,
    OperationId::ReceiptRelabel,
    OperationId::Clear,
    OP,
];

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        10_000_000_u64.try_into().unwrap(),
        1_000_000_u64.try_into().unwrap(),
    )
}

static ANALYZED: LazyLock<ScopedAnalyzedProgram> =
    LazyLock::new(|| analyze_scoped_program(&announcement_input(), limits()).unwrap());
static PLAN: LazyLock<ValidatedMaturityAnnouncementOperationPlan> = LazyLock::new(|| {
    plan_maturity_announcement_target_operation(&announcement_input(), limits()).unwrap()
});
// Used solely by the production refusal arm, never to form oracle expectations.
static VALIDATOR: LazyLock<AnalyzedExpectations> =
    LazyLock::new(|| derive_expectations(&announcement_input(), limits()).unwrap());

fn family_identity(number: usize) -> RelationId {
    use RelationKind as Kind;
    let (kind, side, object) = match number {
        1 => (Kind::Cardinality, TransactionSide::Input, ObjectId::State),
        2 => (Kind::Cardinality, TransactionSide::Output, ObjectId::State),
        3 => (
            Kind::Cardinality,
            TransactionSide::Input,
            ObjectId::PlainLbtc,
        ),
        4 => (
            Kind::Cardinality,
            TransactionSide::Output,
            ObjectId::PlainLbtc,
        ),
        5 => (Kind::Recognition, TransactionSide::Input, ObjectId::State),
        6 => (Kind::Recognition, TransactionSide::Output, ObjectId::State),
        7 => (
            Kind::Recognition,
            TransactionSide::Input,
            ObjectId::PlainLbtc,
        ),
        8 => (
            Kind::Recognition,
            TransactionSide::Output,
            ObjectId::PlainLbtc,
        ),
        _ => unreachable!(),
    };
    RelationId::new(OP, kind, RelationSubject::ObjectFamily { side, object })
}

pub(super) fn identity(number: usize) -> RelationId {
    use RelationKind as Kind;
    use RelationSubject as Subject;
    let (kind, subject) = match number {
        1..=8 => return family_identity(number),
        9 => (Kind::Authorization, Subject::Operation),
        10 => (
            Kind::AllowedObjectFamilies,
            Subject::TransactionSide {
                side: TransactionSide::Input,
            },
        ),
        11 => (
            Kind::AllowedObjectFamilies,
            Subject::TransactionSide {
                side: TransactionSide::Output,
            },
        ),
        12 => (Kind::SponsorIsolation, Subject::Sponsor),
        13 => (
            Kind::SubstrateConservation,
            Subject::Asset {
                asset: AssetId::Lbtc,
            },
        ),
        14 => (Kind::SponsorEnvelopeMultiplicity, Subject::Sponsor),
        15 => (Kind::OpenFlowPolicy, Subject::Operation),
        16 => (Kind::CanonicalDeltaPolicy, Subject::Operation),
        17 => (Kind::RootPolicy, Subject::Operation),
        18 => (Kind::ProjectionPolicy, Subject::Operation),
        19 => (Kind::Constructibility, Subject::Operation),
        20 => (
            Kind::Representation,
            Subject::Representation {
                object: ObjectId::State,
            },
        ),
        21..=26 => (
            Kind::Lifecycle,
            Subject::LifecycleExit {
                object: ObjectId::State,
                exit: EXITS[number - 21],
            },
        ),
        _ => panic!("unknown oracle row {number}"),
    };
    RelationId::new(OP, kind, subject)
}

fn family_body(number: usize) -> Relation {
    use ObjectId::{PlainLbtc, State};
    use ObservedSide::{Input, Output};
    let exact = CardinalityMaximum::Exact(Count::ONE);
    match number {
        1..=4 => {
            let (side, object, minimum, maximum) = match number {
                1 => (Input, State, Count::ONE, exact),
                2 => (Output, State, Count::ONE, exact),
                3 => (
                    Input,
                    PlainLbtc,
                    Count::ZERO,
                    CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
                ),
                4 => (Output, PlainLbtc, Count::ZERO, exact),
                _ => unreachable!(),
            };
            Relation::Cardinality {
                side,
                object,
                minimum,
                maximum,
            }
        }
        5..=8 => {
            let (side, object, asset) = match number {
                5 => (Input, State, AssetId::Pid),
                6 => (Output, State, AssetId::Pid),
                7 => (Input, PlainLbtc, AssetId::Lbtc),
                8 => (Output, PlainLbtc, AssetId::Lbtc),
                _ => unreachable!(),
            };
            Relation::Recognition {
                side,
                object,
                asset,
            }
        }
        _ => panic!("unknown family row {number}"),
    }
}

fn root_or_projection_body(number: usize) -> Relation {
    match number {
        17 => Relation::RootPolicy {
            expected: BTreeMap::from([
                (RootId::State, RootUse::Succession),
                (RootId::Resv, RootUse::Forbidden),
                (RootId::Pace, RootUse::Forbidden),
                (RootId::EntAuth, RootUse::Forbidden),
                (RootId::DistAuth, RootUse::Forbidden),
            ]),
        },
        18 => Relation::ProjectionPolicy {
            expected: BTreeMap::from([
                (
                    ProjectionId::TransitionCertificate,
                    ProjectionRule::Required,
                ),
                (ProjectionId::BurnEvent, ProjectionRule::Forbidden),
                (ProjectionId::ClearEvent, ProjectionRule::Forbidden),
                (ProjectionId::DistributionResidue, ProjectionRule::Forbidden),
            ]),
        },
        _ => panic!("unknown policy row {number}"),
    }
}

fn body(number: usize) -> Relation {
    match number {
        1..=8 => family_body(number),
        9 => Relation::OperatorAuthorization,
        10 | 11 => Relation::AllowedObjectFamilies {
            side: if number == 10 {
                ObservedSide::Input
            } else {
                ObservedSide::Output
            },
            allowed: BTreeSet::from([ObjectId::State, ObjectId::PlainLbtc]),
        },
        12 => Relation::SponsorIsolation,
        13 => Relation::SubstrateConservation {
            asset: AssetId::Lbtc,
        },
        14 => Relation::SponsorEnvelopeMultiplicity {
            maximum: Count::ONE,
        },
        15 => Relation::OpenFlowPolicy {
            allowed: BTreeSet::from([OpenFlowKind::FeeSponsor]),
        },
        16 => Relation::CanonicalDeltaPolicy {
            expected: BTreeSet::new(),
        },
        17 | 18 => root_or_projection_body(number),
        19 => Relation::Constructibility {
            class: ConstructibilityClass::Operator,
        },
        20 => Relation::Representation {
            object: ObjectId::State,
            allowed: BTreeSet::from(MODES),
        },
        21..=26 => Relation::LifecycleExit {
            object: ObjectId::State,
            exit: EXITS[number - 21],
        },
        _ => panic!("unknown oracle row {number}"),
    }
}

fn declarations() -> Vec<RelationDeclaration> {
    (1..=26)
        .map(|number| {
            let id = identity(number);
            let proof_alternatives = match number {
                13 => BTreeSet::from([ProofAlternativeId::new(
                    id.clone(),
                    ProofKind::SubstrateConservation,
                )]),
                20..=26 => BTreeSet::new(),
                _ => BTreeSet::from([ProofAlternativeId::new(
                    id.clone(),
                    ProofKind::ManifestShape,
                )]),
            };
            RelationDeclaration {
                id,
                relation: body(number),
                proof_alternatives,
            }
        })
        .collect()
}

fn dependencies() -> Vec<RelationDependencyProjection> {
    use RelationEdge::{
        AuthorizationBeforeClosure, AuthorizationBeforeConstructibility,
        OpenFlowPolicyBeforeSponsor, ProjectionPolicyBeforeOperation, RecognitionBeforeCardinality,
        RepresentationBeforeLifecycle, RootPolicyBeforeOperation, SponsorBeforeConstructibility,
        SponsorBeforeOperation, StaticRequirement,
    };
    [
        (5, 1, RecognitionBeforeCardinality),
        (6, 2, RecognitionBeforeCardinality),
        (7, 3, RecognitionBeforeCardinality),
        (8, 4, RecognitionBeforeCardinality),
        (9, 10, AuthorizationBeforeClosure),
        (9, 19, AuthorizationBeforeConstructibility),
        (15, 12, OpenFlowPolicyBeforeSponsor),
        (15, 14, StaticRequirement),
        (14, 12, StaticRequirement),
        (7, 12, StaticRequirement),
        (8, 12, StaticRequirement),
        (19, 20, StaticRequirement),
        (6, 20, StaticRequirement),
        (12, 19, SponsorBeforeConstructibility),
        (12, 13, SponsorBeforeOperation),
        (17, 19, RootPolicyBeforeOperation),
        (18, 19, ProjectionPolicyBeforeOperation),
        (20, 21, RepresentationBeforeLifecycle),
        (20, 22, RepresentationBeforeLifecycle),
        (20, 23, RepresentationBeforeLifecycle),
        (20, 24, RepresentationBeforeLifecycle),
        (20, 25, RepresentationBeforeLifecycle),
        (20, 26, RepresentationBeforeLifecycle),
    ]
    .into_iter()
    .map(|(source, target, edge)| RelationDependencyProjection {
        source: identity(source),
        target: identity(target),
        edge,
    })
    .collect()
}

fn cases() -> Vec<ExecutionCaseId> {
    MODES
        .into_iter()
        .flat_map(|mode| {
            [SponsorCase::Absent, SponsorCase::Present].map(|sponsor| ExecutionCaseId {
                operation: OP,
                sponsor,
                representations: BTreeMap::from([(ObjectId::State, mode)]),
            })
        })
        .collect()
}

fn activity(number: usize, sponsor: SponsorCase) -> RelationActivity {
    if [3, 4, 7, 8].contains(&number) && sponsor == SponsorCase::Absent {
        RelationActivity::Vacuous
    } else {
        RelationActivity::Active
    }
}

fn boundaries(number: usize) -> BTreeSet<DischargeBoundary> {
    use DischargeBoundary::{BackendStructural, CompilerStatic, ExternalEvidence, RuntimeCarrier};
    match number {
        9 | 13 | 19 => BTreeSet::from([ExternalEvidence]),
        20..=26 => BTreeSet::from([CompilerStatic, BackendStructural]),
        _ => BTreeSet::from([RuntimeCarrier]),
    }
}

fn proof(number: usize) -> ProofDisposition {
    let relation = identity(number);
    match number {
        9 | 19 => ProofDisposition::ExternalEvidence {
            approved_proof: ProofAlternativeId::new(relation, ProofKind::ManifestShape),
            requirement: ExternalEvidenceRequirement::OperatorAuthorization { operation: OP },
        },
        13 => ProofDisposition::ExternalEvidence {
            approved_proof: ProofAlternativeId::new(relation, ProofKind::SubstrateConservation),
            requirement: ExternalEvidenceRequirement::SubstrateConservation {
                operation: OP,
                asset: AssetId::Lbtc,
            },
        },
        20..=26 => ProofDisposition::StaticallyValidated,
        _ => ProofDisposition::Selected {
            proof: ProofAlternativeId::new(relation, ProofKind::ManifestShape),
        },
    }
}

fn unique_declarations(rows: &[RelationDeclaration]) -> BTreeMap<RelationId, RelationDeclaration> {
    let unique = rows
        .iter()
        .enumerate()
        .filter(|(index, row)| !rows[..*index].iter().any(|previous| previous.id == row.id))
        .count();
    assert_eq!(rows.len(), unique, "duplicate declaration identity");
    let mut result = BTreeMap::new();
    for row in rows {
        assert!(result.insert(row.id.clone(), row.clone()).is_none());
    }
    result
}

type CensusRow = (
    RelationCaseKey,
    RelationActivity,
    BTreeSet<DischargeBoundary>,
);

fn expected_rows() -> Vec<CensusRow> {
    cases()
        .into_iter()
        .flat_map(|case| {
            (1..=26).map(move |number| {
                (
                    RelationCaseKey {
                        relation: identity(number),
                        case: case.clone(),
                    },
                    activity(number, case.sponsor),
                    boundaries(number),
                )
            })
        })
        .collect()
}

fn unique_rows(
    rows: &[CensusRow],
) -> BTreeMap<RelationCaseKey, (RelationActivity, BTreeSet<DischargeBoundary>)> {
    let unique = rows
        .iter()
        .enumerate()
        .filter(|(index, row)| !rows[..*index].iter().any(|previous| previous.0 == row.0))
        .count();
    assert_eq!(rows.len(), unique, "duplicate relation-case identity");
    let mut result = BTreeMap::new();
    for (key, active, boundary) in rows {
        assert!(
            result
                .insert(key.clone(), (*active, boundary.clone()))
                .is_none()
        );
    }
    result
}

fn analyzed_rows(program: &ScopedAnalyzedProgram) -> Vec<CensusRow> {
    program
        .proof_plans
        .values()
        .flat_map(|candidate| {
            candidate.operations[&OP]
                .relation_cases
                .iter()
                .map(|(key, row)| (key.clone(), row.activity, row.boundaries.clone()))
        })
        .collect()
}

fn assert_family(numbers: std::ops::RangeInclusive<usize>) {
    let source = announcement_input().realization().project();
    let actual = unique_declarations(&source.relations.nodes);
    let expected = unique_declarations(&declarations());
    for number in numbers {
        assert_eq!(
            actual.get(&identity(number)),
            expected.get(&identity(number)),
            "R{number}"
        );
    }
}

#[test]
fn literal_cardinalities_and_recognitions_agree() {
    assert_family(1..=8);
}
#[test]
fn literal_authorization_closure_and_substrate_agree() {
    assert_family(9..=13);
}
#[test]
fn literal_policies_and_constructibility_agree() {
    assert_family(14..=19);
}
#[test]
fn literal_representation_and_six_exits_agree() {
    assert_family(20..=26);
}

#[test]
fn declaration_census_equals_raw_and_retained_source_both_ways() {
    let expected = unique_declarations(&declarations());
    assert_eq!(expected.len(), 26);
    assert_eq!(
        unique_declarations(&announcement_input().realization().project().relations.nodes),
        expected
    );
    assert_eq!(
        unique_declarations(&PLAN.source().realization().relations.nodes),
        expected
    );
}

#[test]
fn all_twenty_three_literal_dependency_triples_agree() {
    let mut expected = dependencies();
    assert_eq!(expected.len(), 23);
    expected.sort();
    assert!(expected.windows(2).all(|pair| pair[0] != pair[1]));
    for mut actual in [
        announcement_input().realization().project().relations.edges,
        PLAN.source().realization().relations.edges.clone(),
    ] {
        actual.sort();
        assert_eq!(actual, expected);
    }
}

#[test]
fn four_cases_cross_all_twenty_six_relations_without_erasure() {
    let expected = expected_rows();
    assert_eq!(expected.len(), 104);
    assert_eq!(
        unique_rows(&analyzed_rows(&ANALYZED)),
        unique_rows(&expected)
    );
    for case in cases() {
        assert_eq!(expected.iter().filter(|row| row.0.case == case).count(), 26);
    }
    for mode in MODES {
        assert_eq!(
            expected
                .iter()
                .filter(|row| row.0.case.representations[&ObjectId::State] == mode)
                .count(),
            52
        );
    }
}

#[test]
fn landed_plan_relation_cases_equal_the_literal_product_both_ways() {
    let rows: Vec<_> = PLAN
        .representations()
        .flat_map(|projection| {
            projection.relations().flat_map(|relation| {
                relation
                    .cases
                    .values()
                    .map(|row| (row.key.clone(), row.activity, row.boundaries.clone()))
            })
        })
        .collect();
    assert_eq!(rows.len(), 104);
    assert_eq!(unique_rows(&rows), unique_rows(&expected_rows()));
    let identities: BTreeSet<_> = (1..=26).map(identity).collect();
    for projection in PLAN.representations() {
        let actual: Vec<_> = projection
            .relations()
            .map(|row| row.relation.clone())
            .collect();
        assert_eq!(actual.len(), 26);
        assert_eq!(
            actual.len(),
            actual
                .iter()
                .enumerate()
                .filter(|(index, id)| !actual[..*index].contains(id))
                .count()
        );
        let mut retained = BTreeSet::new();
        for id in actual {
            assert!(retained.insert(id));
        }
        assert_eq!(retained, identities);
    }
}

#[test]
fn active_and_vacuous_are_disjoint_and_exhaustive() {
    let census: BTreeSet<_> = (1..=26).map(identity).collect();
    for projection in PLAN.representations() {
        for case in projection.cases() {
            let active: BTreeSet<_> = (1..=26)
                .filter(|number| activity(*number, case.id.sponsor) == RelationActivity::Active)
                .map(identity)
                .collect();
            let vacuous: BTreeSet<_> = (1..=26)
                .filter(|number| activity(*number, case.id.sponsor) == RelationActivity::Vacuous)
                .map(identity)
                .collect();
            assert_eq!(case.active_relations, active);
            assert_eq!(case.vacuous_relations, vacuous);
            assert!(case.active_relations.is_disjoint(&case.vacuous_relations));
            assert_eq!(
                case.active_relations
                    .union(&case.vacuous_relations)
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                census
            );
            assert_eq!(
                case.active_relations.len(),
                if case.id.sponsor == SponsorCase::Absent {
                    22
                } else {
                    26
                }
            );
        }
    }
}

#[test]
fn proof_classification_is_sixteen_selected_three_external_seven_static() {
    for projection in PLAN.representations() {
        let mut counts = [0; 3];
        for number in 1..=26 {
            let row = projection.relation(&identity(number)).unwrap();
            assert_eq!(row.proof, proof(number), "R{number}");
            counts[match row.proof {
                ProofDisposition::Selected { .. } => 0,
                ProofDisposition::ExternalEvidence { .. } => 1,
                ProofDisposition::StaticallyValidated => 2,
            }] += 1;
        }
        assert_eq!(counts, [16, 3, 7]);
    }
}

#[test]
fn runtime_external_and_hybrid_boundaries_have_the_literal_counts() {
    let rows = analyzed_rows(&ANALYZED);
    for case in cases() {
        let active: Vec<_> = rows
            .iter()
            .filter(|row| row.0.case == case && row.1 == RelationActivity::Active)
            .collect();
        assert_eq!(
            active
                .iter()
                .filter(|row| row.2.contains(&DischargeBoundary::RuntimeCarrier))
                .count(),
            if case.sponsor == SponsorCase::Absent {
                12
            } else {
                16
            }
        );
        for number in [9, 13, 19, 20, 21, 22, 23, 24, 25, 26] {
            let row = active
                .iter()
                .find(|row| row.0.relation == identity(number))
                .unwrap();
            assert_eq!(row.2, boundaries(number));
        }
    }
}

fn case_edges() -> BTreeSet<(RelationCaseKey, RelationCaseKey)> {
    let mut result = BTreeSet::new();
    for case in cases() {
        for edge in dependencies() {
            assert!(result.insert((
                RelationCaseKey {
                    relation: edge.source,
                    case: case.clone()
                },
                RelationCaseKey {
                    relation: edge.target,
                    case: case.clone()
                }
            )));
        }
    }
    result
}

#[test]
fn same_case_prerequisites_equal_the_literal_cross_product() {
    let mut actual = BTreeSet::new();
    for candidate in ANALYZED.proof_plans.values() {
        for edge in &candidate.operations[&OP].coverage_dependencies.edges {
            if edge.edge == CoverageEdge::RelationPrerequisite {
                let (CoverageNodeId::RelationCase(source), CoverageNodeId::RelationCase(target)) =
                    (&edge.source, &edge.target)
                else {
                    panic!("prerequisite endpoints");
                };
                assert!(actual.insert((source.clone(), target.clone())));
            }
        }
    }
    assert_eq!(actual.len(), 92);
    assert_eq!(actual, case_edges());
}

fn closure(start: &RelationCaseKey) -> BTreeSet<RelationCaseKey> {
    let edges = case_edges();
    let mut reached = BTreeSet::from([start.clone()]);
    loop {
        let previous = reached.len();
        for (source, target) in &edges {
            if reached.contains(source) {
                reached.insert(target.clone());
            }
        }
        if previous == reached.len() {
            break;
        }
    }
    let active = unique_rows(&expected_rows());
    reached
        .into_iter()
        .filter(|key| key != start && active[key].0 == RelationActivity::Active)
        .collect()
}

#[test]
fn active_collateral_closure_equals_an_independent_repeated_scan() {
    let mut checked = 0;
    for candidate in ANALYZED.proof_plans.values() {
        for (key, row) in &candidate.operations[&OP].coverage.requirements {
            let expected = closure(key);
            for negative in &row.negative {
                match negative.collateral.policy {
                    CollateralPolicy::RequireIntendedAndDependencyClosure => {
                        assert_eq!(negative.collateral.dependency_closure, expected);
                        checked += 1;
                    }
                    CollateralPolicy::ReportAdditional => {
                        assert!(negative.collateral.dependency_closure.is_empty());
                    }
                }
            }
        }
    }
    assert!(checked > 0);
}

// Raw-source assembly follows the existing analyzed-program oracle pattern.
// The pristine bound input remains the validator's authority throughout.
fn rebuild(raw: &RelationGraphProjection) -> Result<ScopedAnalyzedProgram, CompileError> {
    let input = announcement_input();
    let relations = build_relation_graph(&[OP], &raw.nodes, &raw.edges)?;
    let construction = build_constructibility_analysis(&input)?;
    let lifecycle = build_lifecycle_analysis(&input, &relations)?;
    let mut program = (*ANALYZED).clone();
    program.source.realization.relations = raw.clone();
    program.foundation.relations = relations.project();
    for candidate in program.proof_plans.values_mut() {
        candidate.relation_requirements = relation_requirements(
            &input,
            &relations,
            &construction,
            &lifecycle,
            &candidate.proof_plan,
        )?;
        candidate.operations = analyze_candidate_operations(
            &relations,
            &candidate.relation_requirements,
            &candidate.proof_plan,
            limits(),
        )?;
    }
    validate_against_expectations(&input, &VALIDATOR, &program)?;
    Ok(program)
}

fn reject_raw(mutate: impl FnOnce(&mut RelationGraphProjection)) {
    let mut raw = announcement_input().realization().project().relations;
    mutate(&mut raw);
    let mut expected = declarations();
    expected.sort_by(|left, right| left.id.cmp(&right.id));
    let mut actual = raw.nodes.clone();
    actual.sort_by(|left, right| left.id.cmp(&right.id));
    assert_ne!(
        actual, expected,
        "independent declaration census missed corruption"
    );
    assert!(
        rebuild(&raw).is_err(),
        "production accepted corrupted raw assembly"
    );
}

#[test]
fn omission_is_refused_and_disagrees() {
    reject_raw(|raw| raw.nodes.retain(|row| row.id != identity(6)));
}
#[test]
fn duplication_is_refused_and_disagrees() {
    reject_raw(|raw| raw.nodes.push(raw.nodes[0].clone()));
}
#[test]
fn extra_identity_is_refused_and_disagrees() {
    reject_raw(|raw| {
        let mut extra = raw
            .nodes
            .iter()
            .find(|row| row.id == identity(21))
            .unwrap()
            .clone();
        extra.id = RelationId::new(
            OP,
            RelationKind::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::State,
                exit: OperationId::Burn,
            },
        );
        extra.relation = Relation::LifecycleExit {
            object: ObjectId::State,
            exit: OperationId::Burn,
        };
        raw.nodes.push(extra);
    });
}
#[test]
fn altered_body_is_refused_and_disagrees() {
    reject_raw(|raw| {
        raw.nodes
            .iter_mut()
            .find(|row| row.id == identity(1))
            .unwrap()
            .relation = Relation::Cardinality {
            side: ObservedSide::Input,
            object: ObjectId::State,
            minimum: Count::ZERO,
            maximum: CardinalityMaximum::Exact(Count::ONE),
        };
    });
}
#[test]
fn coherent_relation_and_dependency_omission_is_refused_and_disagrees() {
    reject_raw(|raw| {
        raw.nodes.retain(|row| row.id != identity(6));
        raw.edges
            .retain(|edge| edge.source != identity(6) && edge.target != identity(6));
        assert_eq!(raw.nodes.len(), 25);
        assert_eq!(raw.edges.len(), 21);
    });
}

#[test]
fn case_mode_change_is_refused_and_disagrees() {
    let mut program = (*ANALYZED).clone();
    let factor = program
        .proof_plans
        .values_mut()
        .next()
        .unwrap()
        .operations
        .get_mut(&OP)
        .unwrap();
    let (old, mut row) = factor.relation_cases.pop_first().unwrap();
    let mut key = old;
    assert!(
        key.case
            .representations
            .insert(ObjectId::State, RepresentationMode::PrivateCommitted)
            .is_some()
    );
    row.key = key.clone();
    assert!(factor.relation_cases.insert(key, row).is_none());
    assert_ne!(
        unique_rows(&analyzed_rows(&program)),
        unique_rows(&expected_rows())
    );
    assert!(validate_against_expectations(&announcement_input(), &VALIDATOR, &program).is_err());
}
#[test]
fn activity_flip_is_refused_and_disagrees() {
    let mut program = (*ANALYZED).clone();
    let factor = program
        .proof_plans
        .values_mut()
        .next()
        .unwrap()
        .operations
        .get_mut(&OP)
        .unwrap();
    let row = factor
        .relation_cases
        .values_mut()
        .find(|row| row.activity == RelationActivity::Vacuous)
        .unwrap();
    row.activity = RelationActivity::Active;
    assert_ne!(
        unique_rows(&analyzed_rows(&program)),
        unique_rows(&expected_rows())
    );
    assert!(validate_against_expectations(&announcement_input(), &VALIDATOR, &program).is_err());
}

#[test]
fn raw_source_assembly_accepts_the_independent_literal_census() {
    let mut nodes = declarations();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    let mut edges = dependencies();
    edges.sort();
    let program = rebuild(&RelationGraphProjection { nodes, edges }).unwrap();
    assert_eq!(
        unique_rows(&analyzed_rows(&program)),
        unique_rows(&expected_rows())
    );
}
