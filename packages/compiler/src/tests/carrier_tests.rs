//! Carrier role and carrier eligibility tests (Guide-5 §16.3, §16.4).

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, BoundId, ObjectId, OperationId};
use realization::{
    CardinalityMaximum, Count, ObservedSide, ProofAlternativeId, ProofKind, Relation,
    RelationDeclaration, RelationId, RelationKind, RelationSubject, TransactionSide,
};

use super::bound_input;
use crate::{
    CompileError,
    capability::CapabilityView,
    carrier::{
        CarrierAvailability, CarrierEligibility, CarrierIneligibility, CarrierQuantification,
        CarrierRole, coordinator_anchors, external_carrier, mandatory_input_families,
        relation_case_eligibility, structural_carrier,
    },
    case::{ExecutionCase, ExecutionCaseId, SponsorCase, execution_cases},
    placement::{CarrierMultiplicity, RelationCasePlan, SemanticScope, classify_relation_cases},
    proof::enumerate_feasible_plans,
    relation::{CompilerRelationAnalysis, build_relation_analysis, build_relation_graph},
    source::{OperandRole, derive_source_requirements},
};

struct Pilot {
    relations: CompilerRelationAnalysis,
    plans: Vec<RelationCasePlan>,
    eligibility: Vec<CarrierEligibility>,
}

fn pilot(operation: OperationId) -> Pilot {
    let input = bound_input(&[operation]);
    let relations = build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("plans")
        .candidates;
    let candidate = candidates.first().expect("a feasible candidate");
    let cases = execution_cases(&relations, candidate).expect("cases");
    let plans = classify_relation_cases(&relations, &cases).expect("classification");
    let eligibility = relation_case_eligibility(&relations, &plans).expect("eligibility");

    Pilot {
        relations,
        plans,
        eligibility,
    }
}

impl Pilot {
    fn analysis(&self, relation: &RelationId, sponsor: SponsorCase) -> &CarrierEligibility {
        self.eligibility
            .iter()
            .find(|entry| &entry.relation == relation && entry.case.sponsor == sponsor)
            .expect("relation is carried in this case")
    }

    fn carriers(&self, relation: &RelationId, sponsor: SponsorCase) -> BTreeSet<CarrierRole> {
        self.analysis(relation, sponsor)
            .eligible
            .iter()
            .map(|entry| entry.carrier.clone())
            .collect()
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

// --- mandatory coordinator anchors (§6.2, §16.3) ---

#[test]
fn each_pilot_anchors_its_coordinator_in_its_mandatory_input_family() {
    for (operation, object) in [
        (OperationId::CompactAsh, ObjectId::Ash),
        (OperationId::TransferLive, ObjectId::ReceiptLive),
    ] {
        let pilot = pilot(operation);

        assert_eq!(
            coordinator_anchors(&pilot.relations, operation),
            BTreeSet::from([object]),
            "{operation:?}",
        );
    }
}

#[test]
fn the_optional_sponsor_input_family_is_never_a_coordinator_anchor() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        assert!(
            !mandatory_input_families(&pilot.relations, operation).contains(&ObjectId::PlainLbtc)
        );
        assert!(!coordinator_anchors(&pilot.relations, operation).contains(&ObjectId::PlainLbtc));
    }
}

#[test]
fn no_unconditional_relation_is_carried_by_the_optional_sponsor_family() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        for analysis in &pilot.eligibility {
            let plan = pilot
                .plans
                .iter()
                .find(|plan| plan.relation == analysis.relation && plan.case == analysis.case)
                .expect("every carried relation is planned");

            if plan.activation != crate::placement::ActivationCondition::Always {
                continue;
            }

            for entry in &analysis.eligible {
                assert!(
                    !crate::carrier::is_sponsor_region_carrier(
                        &entry.carrier,
                        crate::sponsor_region::GATED_ORDINARY_LBTC_ROLE,
                    ),
                    "{:?} in {:?}",
                    analysis.relation,
                    analysis.case,
                );
            }
        }
    }
}

// --- owner authorization stays with every member (§16.4, §7.3) ---

#[test]
fn live_owner_authorization_is_eligible_only_at_every_live_receipt_member() {
    let pilot = pilot(OperationId::TransferLive);
    let relation = family(
        OperationId::TransferLive,
        RelationKind::Authorization,
        TransactionSide::Input,
        ObjectId::ReceiptLive,
    );

    for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
        let analysis = pilot.analysis(&relation, sponsor);

        assert_eq!(analysis.multiplicity, CarrierMultiplicity::EveryMember);
        assert_eq!(analysis.eligible.len(), 1);
        assert_eq!(
            analysis.eligible[0].carrier,
            CarrierRole::EveryInputFamilyMember {
                object: ObjectId::ReceiptLive,
            },
        );
        assert_eq!(
            analysis.eligible[0].quantification,
            CarrierQuantification::PerMember,
        );

        // No coordinator may stand in: the owner witness is tied to its
        // authorized input family.
        assert!(analysis.ineligible.iter().any(|entry| {
            matches!(entry.carrier, CarrierRole::OperationGlobal { .. })
                && matches!(entry.reason, CarrierIneligibility::SourceUnavailable { .. },)
        }));
    }
}

#[test]
fn input_recognition_admits_a_member_or_a_complete_family_proof_coordinator() {
    for (operation, object) in [
        (OperationId::CompactAsh, ObjectId::Ash),
        (OperationId::TransferLive, ObjectId::ReceiptLive),
    ] {
        let pilot = pilot(operation);
        let relation = family(
            operation,
            RelationKind::Recognition,
            TransactionSide::Input,
            object,
        );
        let analysis = pilot.analysis(&relation, SponsorCase::Absent);

        let member = analysis
            .eligible
            .iter()
            .find(|entry| entry.carrier == CarrierRole::EveryInputFamilyMember { object })
            .expect("the every-member carrier is eligible");
        assert_eq!(member.quantification, CarrierQuantification::PerMember);
        assert!(
            member
                .routings
                .iter()
                .all(|routing| routing.availability == CarrierAvailability::Intrinsic)
        );

        let coordinator = analysis
            .eligible
            .iter()
            .find(|entry| {
                entry.carrier
                    == CarrierRole::OperationGlobal {
                        operation,
                        anchor: object,
                    }
            })
            .expect("a complete-family-proof coordinator is eligible");
        assert_eq!(
            coordinator.quantification,
            CarrierQuantification::CompleteFamilyProof,
        );
        assert_ne!(
            coordinator.layout_provided(),
            [] as [&crate::carrier::SourceRouting; 0]
        );
    }
}

#[test]
fn output_families_are_not_executable_carriers() {
    for (operation, object) in [
        (OperationId::CompactAsh, ObjectId::Ash),
        (OperationId::TransferLive, ObjectId::ReceiptLive),
    ] {
        let pilot = pilot(operation);
        let relation = family(
            operation,
            RelationKind::Recognition,
            TransactionSide::Output,
            object,
        );
        let analysis = pilot.analysis(&relation, SponsorCase::Absent);

        assert_eq!(
            analysis.scope,
            SemanticScope::FamilyGlobal {
                side: TransactionSide::Output,
                object,
            },
        );
        assert_eq!(
            pilot.carriers(&relation, SponsorCase::Absent),
            BTreeSet::from([CarrierRole::OperationGlobal {
                operation,
                anchor: object,
            }]),
        );
    }
}

// --- sponsor-local confinement (§7.3) ---

#[test]
fn a_sponsor_local_witness_reaches_the_coordinator_only_as_region_evidence() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let relation = id(
            operation,
            RelationKind::SponsorIsolation,
            RelationSubject::Sponsor,
        );
        let analysis = pilot.analysis(&relation, SponsorCase::Present);

        for entry in &analysis.eligible {
            assert!(matches!(entry.carrier, CarrierRole::OperationGlobal { .. }));
            assert!(entry.confines_sponsor_region());

            for routing in &entry.routings {
                if routing.source.availability == realization::AvailabilityClass::SponsorLocal {
                    assert_eq!(
                        routing.availability,
                        CarrierAvailability::SponsorRegionConfined,
                    );
                }
            }
        }
    }
}

#[test]
fn no_carrier_routing_names_a_sponsor_amount() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        for analysis in &pilot.eligibility {
            for entry in &analysis.eligible {
                for routing in &entry.routings {
                    assert!(!crate::source::is_sponsor_amount_operand(
                        routing.source.operand.role()
                    ));
                }
            }
        }
    }
}

// --- non-runtime boundaries keep their own carriers (§16.6) ---

#[test]
fn external_and_structural_relations_receive_no_runtime_carrier() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let external = id(
            operation,
            RelationKind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        );

        assert!(
            !pilot
                .eligibility
                .iter()
                .any(|entry| entry.relation == external)
        );

        // The non-runtime boundaries have their own roles, and neither
        // is an operation-global runtime coordinator.
        let requirement = realization::ExternalEvidenceRequirement::SubstrateConservation {
            operation,
            asset: AssetId::Lbtc,
        };
        assert_eq!(
            external_carrier(requirement.clone()),
            CarrierRole::ExternalEvidence { requirement },
        );
        assert_eq!(
            structural_carrier(operation),
            CarrierRole::BackendStructural { operation },
        );
    }
}

#[test]
fn every_active_runtime_requirement_has_an_eligible_carrier() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let carried = pilot
            .plans
            .iter()
            .filter(|plan| !plan.runtime_requirements.is_empty())
            .count();

        assert_eq!(pilot.eligibility.len(), carried);
        assert!(
            pilot
                .eligibility
                .iter()
                .all(|entry| !entry.eligible.is_empty())
        );
    }
}

#[test]
fn repeated_eligibility_analysis_is_equal() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let second = relation_case_eligibility(&pilot.relations, &pilot.plans).expect("second");

        assert_eq!(pilot.eligibility, second);
    }
}

// --- synthetic defects a validated realization could never carry ---

fn declaration(
    id: RelationId,
    relation: Relation,
    proof: Option<ProofKind>,
) -> RelationDeclaration {
    let proof_alternatives = proof
        .into_iter()
        .map(|kind| ProofAlternativeId::new(id.clone(), kind))
        .collect();

    RelationDeclaration {
        id,
        relation,
        proof_alternatives,
    }
}

fn cardinality(operation: OperationId, object: ObjectId, minimum: Count) -> RelationDeclaration {
    declaration(
        family(
            operation,
            RelationKind::Cardinality,
            TransactionSide::Input,
            object,
        ),
        Relation::Cardinality {
            side: ObservedSide::Input,
            object,
            minimum,
            maximum: CardinalityMaximum::Bound(BoundId::AshBatchMax),
        },
        Some(ProofKind::ManifestShape),
    )
}

/// One synthetic operation, its relation analysis, and one sponsorless
/// case carrying the declarations' own source rows.
fn synthetic(
    operation: OperationId,
    declarations: &[(RelationDeclaration, ProofKind)],
) -> (CompilerRelationAnalysis, Vec<RelationCasePlan>) {
    let nodes = declarations
        .iter()
        .map(|(node, _)| node.clone())
        .collect::<Vec<_>>();
    let relations = build_relation_graph(&[operation], &nodes, &[]).expect("synthetic relations");

    let id = ExecutionCaseId {
        operation,
        sponsor: SponsorCase::Absent,
        representations: BTreeMap::new(),
    };
    let mut active_sources = Vec::new();

    for (node, proof) in declarations {
        active_sources.extend(
            derive_source_requirements(node, *proof)
                .expect("source rows")
                .into_iter()
                .filter(|row| crate::case::is_active(&id, row)),
        );
    }

    active_sources.sort();

    let case = ExecutionCase { id, active_sources };
    let plans = classify_relation_cases(&relations, std::slice::from_ref(&case))
        .expect("synthetic classification");

    (relations, plans)
}

#[test]
fn an_operation_without_a_mandatory_input_family_has_no_coordinator() {
    // No input family is declared at all, so the transaction-global
    // relation has no anchor to coordinate from.
    let (relations, plans) = synthetic(
        OperationId::CompactAsh,
        &[(
            declaration(
                id(
                    OperationId::CompactAsh,
                    RelationKind::SponsorIsolation,
                    RelationSubject::Sponsor,
                ),
                Relation::SponsorIsolation,
                Some(ProofKind::ManifestShape),
            ),
            ProofKind::ManifestShape,
        )],
    );

    assert_eq!(
        relation_case_eligibility(&relations, &plans),
        Err(CompileError::MissingCanonicalCoordinator {
            operation: OperationId::CompactAsh,
        }),
    );
}

#[test]
fn an_unconditional_relation_with_only_optional_carriers_is_rejected() {
    // The one declared input family may be absent, so the family's own
    // unconditional cardinality relation has only an optional carrier.
    let optional = cardinality(OperationId::CompactAsh, ObjectId::Ash, Count::ZERO);
    let relation = optional.id.clone();
    let (relations, plans) = synthetic(
        OperationId::CompactAsh,
        &[(optional, ProofKind::ManifestShape)],
    );

    let error = relation_case_eligibility(&relations, &plans).expect_err("optional-carrier defect");
    let CompileError::UnconditionalRelationOnOptionalCarrier {
        relation: named, ..
    } = error
    else {
        panic!("expected an optional-carrier defect");
    };

    assert_eq!(named, relation);
}

#[test]
fn an_owner_authorization_outside_its_input_family_has_no_eligible_carrier() {
    let authorization = declaration(
        family(
            OperationId::CompactAsh,
            RelationKind::Authorization,
            TransactionSide::Input,
            ObjectId::ReceiptLive,
        ),
        Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        },
        Some(ProofKind::SignerMembership),
    );
    let relation = authorization.id.clone();
    let (relations, plans) = synthetic(
        OperationId::CompactAsh,
        &[
            (
                cardinality(OperationId::CompactAsh, ObjectId::Ash, Count::new(2)),
                ProofKind::ManifestShape,
            ),
            (authorization, ProofKind::SignerMembership),
        ],
    );

    let error = relation_case_eligibility(&relations, &plans).expect_err("no eligible carrier");
    let CompileError::NoEligibleCarrier {
        relation: named, ..
    } = error
    else {
        panic!("expected an eligibility failure");
    };

    assert_eq!(named, relation);
}

#[test]
fn an_owner_witness_is_available_only_at_its_own_family_member() {
    let pilot = pilot(OperationId::TransferLive);
    let relation = family(
        OperationId::TransferLive,
        RelationKind::Authorization,
        TransactionSide::Input,
        ObjectId::ReceiptLive,
    );
    let analysis = pilot.analysis(&relation, SponsorCase::Absent);
    let entry = &analysis.eligible[0];

    let witness = entry
        .routings
        .iter()
        .find(|routing| routing.source.operand.role() == &OperandRole::ProtocolSignerSet)
        .expect("the owner witness row");

    assert_eq!(witness.availability, CarrierAvailability::Intrinsic);
    assert_eq!(
        witness.source.availability,
        realization::AvailabilityClass::InputOwners {
            object: ObjectId::ReceiptLive,
        },
    );
}
