//! Relation discharge classification tests (Guide-5 §9, §16.2).

use std::collections::BTreeSet;

use architecture::{AssetId, ObjectId, OperationId};
use realization::{Relation, RelationId, RelationKind, RelationSubject, TransactionSide};

use super::bound_input;
use crate::{
    CompileError,
    capability::CapabilityView,
    case::{ExecutionCase, SponsorCase, execution_cases},
    placement::{
        ActivationCondition, BackendStructuralRequirement, CarrierMultiplicity,
        CompilerStaticRequirement, DischargeBoundary, RelationActivity, RelationCasePlan,
        SemanticScope, classify_relation_cases, validate_relation_case_census,
    },
    proof::enumerate_feasible_plans,
    relation::{CompilerRelationAnalysis, build_relation_analysis},
};

struct Pilot {
    relations: CompilerRelationAnalysis,
    cases: Vec<ExecutionCase>,
    plans: Vec<RelationCasePlan>,
}

fn pilot(operation: OperationId) -> Pilot {
    let input = bound_input(&[operation]);
    let relations = build_relation_analysis(&input).expect("relations");
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");
    let candidate = plans.candidates.first().expect("a feasible candidate");
    let cases = execution_cases(&relations, candidate).expect("cases");
    let plans = classify_relation_cases(&relations, &cases).expect("classification");

    Pilot {
        relations,
        cases,
        plans,
    }
}

impl Pilot {
    /// Every plan of one relation, one per case.
    fn plans_of(&self, relation: &RelationId) -> Vec<&RelationCasePlan> {
        self.plans
            .iter()
            .filter(|plan| &plan.relation == relation)
            .collect()
    }

    /// The single plan of one relation in one sponsor case.
    fn plan(&self, relation: &RelationId, sponsor: SponsorCase) -> &RelationCasePlan {
        self.plans
            .iter()
            .find(|plan| &plan.relation == relation && plan.case.sponsor == sponsor)
            .expect("relation is planned in this case")
    }
}

fn id(operation: OperationId, kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(operation, kind, subject)
}

fn family(
    operation: OperationId,
    kind: RelationKind,
    side: TransactionSide,
    object: ObjectId,
) -> RelationId {
    id(
        operation,
        kind,
        RelationSubject::ObjectFamily { side, object },
    )
}

fn runtime_only(plan: &RelationCasePlan) -> (SemanticScope, CarrierMultiplicity) {
    assert_eq!(
        plan.boundaries,
        BTreeSet::from([DischargeBoundary::RuntimeCarrier]),
        "{:?}",
        plan.relation,
    );
    assert_eq!(plan.activity, RelationActivity::Active);
    assert_eq!(plan.runtime_requirements.len(), 1);

    let requirement = &plan.runtime_requirements[0];
    assert_eq!(requirement.relation, plan.relation);
    assert!(plan.compiler_requirements.is_empty());
    assert!(plan.structural_requirements.is_empty());
    assert!(plan.external_evidence.is_empty());

    (requirement.scope, requirement.multiplicity)
}

// --- compact ASH classification matrix (§9.1) ---

#[test]
fn compact_ash_input_recognition_is_member_local_every_member() {
    let pilot = pilot(OperationId::CompactAsh);
    let relation = family(
        OperationId::CompactAsh,
        RelationKind::Recognition,
        TransactionSide::Input,
        ObjectId::Ash,
    );

    for plan in pilot.plans_of(&relation) {
        assert_eq!(
            runtime_only(plan),
            (
                SemanticScope::MemberLocal {
                    side: TransactionSide::Input,
                    object: ObjectId::Ash,
                },
                CarrierMultiplicity::EveryMember,
            ),
        );
    }
}

#[test]
fn compact_ash_output_recognition_and_cardinality_are_family_global() {
    let pilot = pilot(OperationId::CompactAsh);

    for kind in [RelationKind::Recognition, RelationKind::Cardinality] {
        let relation = family(
            OperationId::CompactAsh,
            kind,
            TransactionSide::Output,
            ObjectId::Ash,
        );

        for plan in pilot.plans_of(&relation) {
            assert_eq!(
                runtime_only(plan),
                (
                    SemanticScope::FamilyGlobal {
                        side: TransactionSide::Output,
                        object: ObjectId::Ash,
                    },
                    CarrierMultiplicity::ExactlyOne,
                ),
            );
        }
    }
}

#[test]
fn compact_ash_conservation_and_policies_are_transaction_global() {
    let pilot = pilot(OperationId::CompactAsh);
    let global = [
        id(
            OperationId::CompactAsh,
            RelationKind::Conservation,
            RelationSubject::Asset { asset: AssetId::U },
        ),
        id(
            OperationId::CompactAsh,
            RelationKind::SponsorIsolation,
            RelationSubject::Sponsor,
        ),
        id(
            OperationId::CompactAsh,
            RelationKind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
        ),
        id(
            OperationId::CompactAsh,
            RelationKind::RootPolicy,
            RelationSubject::Operation,
        ),
        id(
            OperationId::CompactAsh,
            RelationKind::ProjectionPolicy,
            RelationSubject::Operation,
        ),
    ];

    for relation in &global {
        let plans = pilot.plans_of(relation);
        assert_eq!(plans.len(), 2, "{relation:?}");

        for plan in plans {
            assert_eq!(
                runtime_only(plan),
                (
                    SemanticScope::TransactionGlobal,
                    CarrierMultiplicity::ExactlyOne
                ),
                "{relation:?}",
            );
        }
    }
}

#[test]
fn permissionless_authorization_is_a_backend_structural_secret_free_path() {
    let pilot = pilot(OperationId::CompactAsh);
    let relation = id(
        OperationId::CompactAsh,
        RelationKind::Authorization,
        RelationSubject::Operation,
    );

    for plan in pilot.plans_of(&relation) {
        assert_eq!(
            plan.boundaries,
            BTreeSet::from([DischargeBoundary::BackendStructural]),
        );
        assert!(plan.runtime_requirements.is_empty());
        assert_eq!(
            plan.structural_requirements,
            vec![BackendStructuralRequirement::SecretFreeOperationPath {
                operation: OperationId::CompactAsh,
            }],
        );
    }
}

#[test]
fn representation_is_static_selection_plus_structural_encoding() {
    let pilot = pilot(OperationId::CompactAsh);
    let relation = id(
        OperationId::CompactAsh,
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::Ash,
        },
    );

    for plan in pilot.plans_of(&relation) {
        assert_eq!(
            plan.boundaries,
            BTreeSet::from([
                DischargeBoundary::CompilerStatic,
                DischargeBoundary::BackendStructural,
            ]),
        );
        assert!(plan.runtime_requirements.is_empty());

        let selected = plan.case.representations[&ObjectId::Ash];

        assert!(plan.compiler_requirements.iter().any(|requirement| matches!(
            requirement,
            CompilerStaticRequirement::RepresentationSelection { object, allowed, selected: chosen }
                if *object == ObjectId::Ash && *chosen == selected && allowed.contains(&selected)
        )));
        assert_eq!(
            plan.structural_requirements,
            vec![
                BackendStructuralRequirement::EncodeAndAuthenticateRepresentation {
                    object: ObjectId::Ash,
                    representation: selected,
                }
            ],
        );
    }
}

#[test]
fn constructibility_is_compiler_static_only() {
    let pilot = pilot(OperationId::CompactAsh);
    let relation = id(
        OperationId::CompactAsh,
        RelationKind::Constructibility,
        RelationSubject::Operation,
    );

    for plan in pilot.plans_of(&relation) {
        assert_eq!(
            plan.boundaries,
            BTreeSet::from([DischargeBoundary::CompilerStatic]),
        );
        assert!(plan.runtime_requirements.is_empty());
        assert!(plan.structural_requirements.is_empty());
        assert_eq!(plan.compiler_requirements.len(), 1);
    }
}

#[test]
fn substrate_conservation_is_external_evidence_with_no_runtime_requirement() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let relation = id(
            operation,
            RelationKind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        );

        for plan in pilot.plans_of(&relation) {
            assert_eq!(
                plan.boundaries,
                BTreeSet::from([DischargeBoundary::ExternalEvidence]),
            );
            assert!(plan.runtime_requirements.is_empty());
            assert!(plan.structural_requirements.is_empty());
            assert!(plan.compiler_requirements.is_empty());
            assert_eq!(
                plan.external_evidence,
                BTreeSet::from([
                    realization::ExternalEvidenceRequirement::SubstrateConservation {
                        operation,
                        asset: AssetId::Lbtc,
                    }
                ]),
            );
        }
    }
}

#[test]
fn sponsor_family_relations_are_vacuous_without_a_sponsor_region() {
    let pilot = pilot(OperationId::CompactAsh);
    let relation = family(
        OperationId::CompactAsh,
        RelationKind::Recognition,
        TransactionSide::Input,
        ObjectId::PlainLbtc,
    );

    let absent = pilot.plan(&relation, SponsorCase::Absent);
    assert_eq!(absent.activation, ActivationCondition::WhenSponsorPresent);
    assert_eq!(absent.activity, RelationActivity::Vacuous);
    assert!(absent.runtime_requirements.is_empty());

    let present = pilot.plan(&relation, SponsorCase::Present);
    assert_eq!(present.activation, ActivationCondition::WhenSponsorPresent);
    assert_eq!(present.activity, RelationActivity::Active);
    assert_eq!(present.runtime_requirements.len(), 1);
}

#[test]
fn sponsor_isolation_stays_active_without_a_sponsor_region() {
    let pilot = pilot(OperationId::CompactAsh);
    let relation = id(
        OperationId::CompactAsh,
        RelationKind::SponsorIsolation,
        RelationSubject::Sponsor,
    );

    for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
        let plan = pilot.plan(&relation, sponsor);

        assert_eq!(plan.activation, ActivationCondition::Always);
        assert_eq!(plan.activity, RelationActivity::Active);
        assert_eq!(plan.runtime_requirements.len(), 1);
    }
}

// --- live transfer classification matrix (§9.2) ---

#[test]
fn live_owner_authorization_is_every_consumed_member() {
    let pilot = pilot(OperationId::TransferLive);
    let relation = family(
        OperationId::TransferLive,
        RelationKind::Authorization,
        TransactionSide::Input,
        ObjectId::ReceiptLive,
    );

    let plans = pilot.plans_of(&relation);
    assert_eq!(plans.len(), 2);

    for plan in plans {
        assert_eq!(
            runtime_only(plan),
            (
                SemanticScope::MemberLocal {
                    side: TransactionSide::Input,
                    object: ObjectId::ReceiptLive,
                },
                CarrierMultiplicity::EveryMember,
            ),
        );
    }
}

#[test]
fn live_conservation_is_transaction_global_never_per_member() {
    let pilot = pilot(OperationId::TransferLive);
    let relation = id(
        OperationId::TransferLive,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );

    for plan in pilot.plans_of(&relation) {
        let (scope, multiplicity) = runtime_only(plan);

        assert_eq!(scope, SemanticScope::TransactionGlobal);
        assert_eq!(multiplicity, CarrierMultiplicity::ExactlyOne);
        assert!(!matches!(scope, SemanticScope::MemberLocal { .. }));
    }
}

#[test]
fn live_sponsor_cardinality_is_conditional_but_receipt_cardinality_is_not() {
    let pilot = pilot(OperationId::TransferLive);

    let sponsor = family(
        OperationId::TransferLive,
        RelationKind::Cardinality,
        TransactionSide::Input,
        ObjectId::PlainLbtc,
    );
    let receipt = family(
        OperationId::TransferLive,
        RelationKind::Cardinality,
        TransactionSide::Input,
        ObjectId::ReceiptLive,
    );

    assert_eq!(
        pilot.plan(&sponsor, SponsorCase::Absent).activity,
        RelationActivity::Vacuous,
    );
    assert_eq!(
        pilot.plan(&receipt, SponsorCase::Absent).activity,
        RelationActivity::Active,
    );
}

// --- exact relation-case census (§16.2) ---

#[test]
fn every_relation_is_planned_in_every_applicable_case_exactly_once() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let relation_count = pilot.relations.graph.node_count();

        assert_eq!(pilot.cases.len(), 2);
        assert_eq!(pilot.plans.len(), relation_count * pilot.cases.len());

        validate_relation_case_census(&pilot.relations, &pilot.cases, &pilot.plans)
            .expect("exact census");
    }
}

#[test]
fn every_disposition_class_is_represented_in_the_pilot_census() {
    let pilot = pilot(OperationId::TransferLive);

    let boundaries = pilot
        .plans
        .iter()
        .flat_map(|plan| plan.boundaries.iter().copied())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        boundaries,
        BTreeSet::from([
            DischargeBoundary::CompilerStatic,
            DischargeBoundary::BackendStructural,
            DischargeBoundary::RuntimeCarrier,
            DischargeBoundary::ExternalEvidence,
        ]),
    );

    assert!(
        pilot
            .plans
            .iter()
            .any(|plan| plan.activity == RelationActivity::Vacuous)
    );
    assert!(
        pilot
            .plans
            .iter()
            .any(|plan| plan.activity == RelationActivity::Active)
    );
}

#[test]
fn a_repeated_relation_case_plan_is_rejected() {
    let pilot = pilot(OperationId::CompactAsh);
    let mut doubled = pilot.plans.clone();
    doubled.push(pilot.plans[0].clone());

    assert_eq!(
        validate_relation_case_census(&pilot.relations, &pilot.cases, &doubled),
        Err(CompileError::DuplicateRelationCasePlan {
            relation: pilot.plans[0].relation.clone(),
            case: pilot.plans[0].case.clone(),
        }),
    );
}

#[test]
fn a_dropped_relation_case_plan_is_rejected() {
    let pilot = pilot(OperationId::CompactAsh);
    let mut short = pilot.plans.clone();
    let dropped = short.remove(0);

    let error = validate_relation_case_census(&pilot.relations, &pilot.cases, &short)
        .expect_err("census defect");
    let CompileError::RelationCaseCensusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("expected a relation-case census mismatch");
    };

    assert!(unexpected.is_empty());
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].relation, dropped.relation);
    assert_eq!(missing[0].case, dropped.case);
}

// --- sponsor erasure and determinism ---

#[test]
fn no_runtime_requirement_names_a_sponsor_amount() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        for plan in &pilot.plans {
            for requirement in &plan.runtime_requirements {
                for row in &requirement.sources {
                    assert!(!crate::source::is_sponsor_amount_operand(
                        row.operand.role()
                    ));
                }
            }
        }
    }
}

#[test]
fn every_relation_variant_of_the_pilots_is_classified() {
    // The classification match is exhaustive over relation variants; the
    // pilots exercise every variant they declare, and no plan is left
    // without a disposition.
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        for plan in &pilot.plans {
            let declaration = pilot
                .relations
                .graph
                .node_weights()
                .find(|node| node.source.id == plan.relation)
                .expect("relation exists");

            assert!(!plan.boundaries.is_empty(), "{:?}", plan.relation);

            let requirement_count = plan.compiler_requirements.len()
                + plan.structural_requirements.len()
                + plan.runtime_requirements.len()
                + plan.external_evidence.len();

            match (&declaration.source.relation, plan.activity) {
                // A vacuous relation states its disposition and carries
                // no requirement in this case.
                (_, RelationActivity::Vacuous) => assert_eq!(requirement_count, 0),
                (Relation::Representation { .. } | Relation::LifecycleExit { .. }, _) => {
                    assert_eq!(requirement_count, 2);
                }
                (_, RelationActivity::Active) => assert!(requirement_count >= 1),
            }
        }
    }
}

#[test]
fn repeated_classification_is_equal() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let second =
            classify_relation_cases(&pilot.relations, &pilot.cases).expect("second classification");

        assert_eq!(pilot.plans, second);
    }
}
