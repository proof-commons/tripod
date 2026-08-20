//! Relation discharge classification tests (Guide-5 §9, §16.2).

use std::collections::BTreeSet;

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    Relation, RelationId, RelationKind, RelationSubject, RepresentationMode, TransactionSide,
};

use super::bound_input;
use crate::{
    CompileError,
    capability::CapabilityView,
    carrier::{CarrierQuantification, CarrierRole, relation_case_eligibility},
    case::{ExecutionCase, SponsorCase, execution_cases},
    layout::LayoutRequirement,
    placement::{
        ActivationCondition, BackendStructuralRequirement, CarrierMultiplicity,
        CompilerStaticRequirement, DischargeBoundary, PlacedCarrier, PlacedProofPlanCandidate,
        PlacementSearchLimits, RelationActivity, RelationCasePlan, SemanticScope,
        classify_relation_cases, enumerate_feasible_placements, place_proof_plan, resolve_activity,
        validate_relation_case_census,
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
    assert_eq!(
        plan.compiler_requirements,
        [] as [CompilerStaticRequirement; 0]
    );
    assert_eq!(
        plan.structural_requirements,
        [] as [BackendStructuralRequirement; 0]
    );
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
        assert_eq!(
            plan.runtime_requirements,
            [] as [crate::placement::RuntimePlacementRequirement; 0]
        );
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
        assert_eq!(
            plan.runtime_requirements,
            [] as [crate::placement::RuntimePlacementRequirement; 0]
        );

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
        assert_eq!(
            plan.runtime_requirements,
            [] as [crate::placement::RuntimePlacementRequirement; 0]
        );
        assert_eq!(
            plan.structural_requirements,
            [] as [BackendStructuralRequirement; 0]
        );
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
            assert_eq!(
                plan.runtime_requirements,
                [] as [crate::placement::RuntimePlacementRequirement; 0]
            );
            assert_eq!(
                plan.structural_requirements,
                [] as [BackendStructuralRequirement; 0]
            );
            assert_eq!(
                plan.compiler_requirements,
                [] as [CompilerStaticRequirement; 0]
            );
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
    assert_eq!(
        absent.runtime_requirements,
        [] as [crate::placement::RuntimePlacementRequirement; 0]
    );

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

#[test]
fn a_representation_conditional_relation_reads_its_own_objects_mode() {
    // Synthetic: the classification matrix produces only unconditional
    // and sponsor-conditional relations today, so this condition is
    // resolved directly. The property it fixes is the one a second
    // independently represented object family would otherwise break —
    // one family's mode activating another family's relation.
    let case = crate::case::ExecutionCaseId {
        operation: OperationId::TransferLive,
        sponsor: SponsorCase::Absent,
        representations: std::collections::BTreeMap::from([
            (ObjectId::Ash, RepresentationMode::Explicit),
            (ObjectId::ReceiptLive, RepresentationMode::PrivateCommitted),
        ]),
    };
    let conditional = |object| ActivationCondition::WhenRepresentation {
        object,
        mode: RepresentationMode::PrivateCommitted,
    };

    assert_eq!(
        resolve_activity(conditional(ObjectId::Ash), &case),
        RelationActivity::Vacuous,
    );
    assert_eq!(
        resolve_activity(conditional(ObjectId::ReceiptLive), &case),
        RelationActivity::Active,
    );
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

    assert_eq!(unexpected, [] as [crate::placement::RelationCaseKey; 0]);
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

// --- exact placement search over the pilots (§10) ---

/// Generous pilot limits: the search must complete, so a limit that
/// truncated it would hide a defect rather than bound one.
fn placement_limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(100_000).expect("nonzero"),
    )
}

fn placed(operation: OperationId) -> PlacedProofPlanCandidate {
    let input = bound_input(&[operation]);
    let relations = build_relation_analysis(&input).expect("relations");
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");
    let candidate = plans.candidates.first().expect("a feasible candidate");

    place_proof_plan(&relations, candidate, placement_limits()).expect("placement")
}

#[test]
fn every_pilot_plan_has_a_complete_feasible_placement_set() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let placed = placed(operation);

        assert!(!placed.feasible_placements.is_empty(), "{operation:?}");

        // Canonical order, no duplicate placement, and no weighting: the
        // whole feasible set survives side by side.
        let mut canonical = placed.feasible_placements.clone();
        canonical.sort();
        canonical.dedup();

        assert_eq!(canonical, placed.feasible_placements, "{operation:?}");
    }
}

#[test]
fn every_placement_carries_exactly_the_active_runtime_relation_cases() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let placed = placed(operation);
        let required = placed
            .relation_case_plans
            .iter()
            .filter(|plan| !plan.runtime_requirements.is_empty())
            .map(|plan| (plan.relation.clone(), plan.case.clone()))
            .collect::<BTreeSet<_>>();

        assert!(!required.is_empty());

        for placement in &placed.feasible_placements {
            let carried = placement
                .assignments
                .iter()
                .map(|assignment| (assignment.relation.clone(), assignment.case.clone()))
                .collect::<BTreeSet<_>>();

            assert_eq!(carried, required, "{operation:?}");
        }
    }
}

#[test]
fn no_placement_carries_a_static_structural_or_external_relation_case() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let placed = placed(operation);
        let non_runtime = placed
            .relation_case_plans
            .iter()
            .filter(|plan| plan.runtime_requirements.is_empty())
            .map(|plan| (plan.relation.clone(), plan.case.clone()))
            .collect::<BTreeSet<_>>();

        // The pilots really do declare such relation-cases — the static,
        // structural, external, and vacuous dispositions — so the
        // assertion below is not vacuous itself.
        assert!(!non_runtime.is_empty());

        for placement in &placed.feasible_placements {
            for assignment in &placement.assignments {
                assert!(
                    !non_runtime.contains(&(assignment.relation.clone(), assignment.case.clone())),
                    "{:?} in {operation:?}",
                    assignment.relation,
                );
            }
        }
    }
}

#[test]
fn live_owner_authorization_is_placed_only_per_member() {
    let placed = placed(OperationId::TransferLive);
    let relation = family(
        OperationId::TransferLive,
        RelationKind::Authorization,
        TransactionSide::Input,
        ObjectId::ReceiptLive,
    );

    for placement in &placed.feasible_placements {
        let assignment = placement
            .assignments
            .iter()
            .find(|assignment| assignment.relation == relation)
            .expect("owner authorization is placed");

        assert_eq!(
            assignment.carriers,
            vec![PlacedCarrier {
                carrier: CarrierRole::EveryInputFamilyMember {
                    object: ObjectId::ReceiptLive,
                },
                quantification: CarrierQuantification::PerMember,
            }],
        );
    }
}

#[test]
fn no_global_relation_is_ever_placed_on_a_member_role() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let placed = placed(operation);

        for placement in &placed.feasible_placements {
            for assignment in &placement.assignments {
                let plan = placed
                    .relation_case_plans
                    .iter()
                    .find(|plan| {
                        plan.relation == assignment.relation && plan.case == assignment.case
                    })
                    .expect("every placed relation-case is planned");
                let scope = plan.runtime_requirements[0].scope;

                if matches!(scope, SemanticScope::MemberLocal { .. }) {
                    continue;
                }

                for carrier in &assignment.carriers {
                    assert!(
                        !matches!(carrier.carrier, CarrierRole::EveryInputFamilyMember { .. }),
                        "{:?} in {operation:?}",
                        assignment.relation,
                    );
                }
            }
        }
    }
}

#[test]
fn a_complete_family_proof_placement_states_the_census_it_relies_on() {
    for (operation, object) in [
        (OperationId::CompactAsh, ObjectId::Ash),
        (OperationId::TransferLive, ObjectId::ReceiptLive),
    ] {
        let placed = placed(operation);
        let relation = family(
            operation,
            RelationKind::Recognition,
            TransactionSide::Input,
            object,
        );
        let mut proofs = 0_usize;

        for placement in &placed.feasible_placements {
            let assignment = placement
                .assignments
                .iter()
                .find(|assignment| assignment.relation == relation)
                .expect("input recognition is placed");

            if assignment.carriers[0].quantification != CarrierQuantification::CompleteFamilyProof {
                continue;
            }

            proofs += 1;

            assert!(placement.layout_requirements.contains(
                &LayoutRequirement::AuthenticateFamilyCensus {
                    relation: relation.clone(),
                    side: TransactionSide::Input,
                    object,
                }
            ),);
        }

        assert!(proofs > 0, "{operation:?}");
    }
}

#[test]
fn every_placement_layout_dependency_is_within_the_operation_census() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let placed = placed(operation);
        let census = placed.layout_requirements.iter().collect::<BTreeSet<_>>();

        for placement in &placed.feasible_placements {
            for requirement in &placement.layout_requirements {
                assert!(census.contains(requirement), "{operation:?}");
            }
        }
    }
}

/// One pilot's relation-case plans and eligible carrier sets.
fn carried(
    operation: OperationId,
) -> (
    Vec<RelationCasePlan>,
    Vec<crate::carrier::CarrierEligibility>,
) {
    let input = bound_input(&[operation]);
    let relations = build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("plans")
        .candidates;
    let candidate = candidates.first().expect("a feasible candidate");
    let cases = execution_cases(&relations, candidate).expect("cases");
    let plans = classify_relation_cases(&relations, &cases).expect("classification");
    let eligibility = relation_case_eligibility(&relations, &plans).expect("eligibility");

    (plans, eligibility)
}

#[test]
fn repeated_placement_search_is_equal_and_projects_equally() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let (plans, eligibility) = carried(operation);
        let first = enumerate_feasible_placements(&plans, &eligibility, placement_limits())
            .expect("first search");
        let second = enumerate_feasible_placements(&plans, &eligibility, placement_limits())
            .expect("second search");

        assert_eq!(first, second, "{operation:?}");
        assert_eq!(first.project(), second.project(), "{operation:?}");
        assert_eq!(
            first.project().candidates.len(),
            first.candidates.len(),
            "{operation:?}",
        );
    }
}

#[test]
fn placement_exhaustion_returns_no_partial_result() {
    let (plans, eligibility) = carried(OperationId::CompactAsh);
    let one = std::num::NonZeroU64::new(1).expect("nonzero");
    let generous = std::num::NonZeroU64::new(1_000_000).expect("nonzero");

    assert_eq!(
        enumerate_feasible_placements(
            &plans,
            &eligibility,
            PlacementSearchLimits::new(one, generous),
        ),
        Err(CompileError::PlacementSearchStateLimitExceeded { maximum: 1 }),
    );
    assert_eq!(
        enumerate_feasible_placements(
            &plans,
            &eligibility,
            PlacementSearchLimits::new(generous, one),
        ),
        Err(CompileError::PlacementCandidateLimitExceeded { maximum: 1 }),
    );
}

#[test]
fn the_placement_state_budget_counts_every_visited_state_and_its_boundary_is_exact() {
    let (plans, eligibility) = carried(OperationId::CompactAsh);
    let generous = std::num::NonZeroU64::new(1_000_000).expect("nonzero");
    let budget = |states: u64| {
        PlacementSearchLimits::new(
            std::num::NonZeroU64::new(states).expect("nonzero"),
            generous,
        )
    };

    let visited = enumerate_feasible_placements(&plans, &eligibility, budget(1_000_000))
        .expect("placements")
        .search
        .states_visited;

    assert!(visited > 1, "the pilot placement search recurses");

    // Inclusive rule, matching the proof search: a budget of exactly the
    // visited count completes.
    assert_eq!(
        enumerate_feasible_placements(&plans, &eligibility, budget(visited))
            .expect("the exact budget completes")
            .search
            .states_visited,
        visited,
    );

    assert_eq!(
        enumerate_feasible_placements(&plans, &eligibility, budget(visited - 1)),
        Err(CompileError::PlacementSearchStateLimitExceeded {
            maximum: visited - 1,
        }),
    );

    assert_eq!(
        enumerate_feasible_placements(&plans, &eligibility, budget(visited + 1))
            .expect("a spare budget completes")
            .search
            .states_visited,
        visited,
    );
}
