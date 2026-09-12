//! Target-independent layout requirement tests (Guide-5 §16.5–§16.7).

use architecture::{AssetId, ObjectId, OperationId};
use realization::{RelationId, RelationKind, RelationSubject, TransactionSide};

use super::bound_input;
use crate::{
    CompileError,
    capability::{CapabilityView, RequiredCapability},
    carrier::{CarrierEligibility, CarrierRole, relation_case_eligibility},
    case::{ExecutionCase, SponsorCase, execution_cases},
    layout::{
        LayoutRequirement, layout_requirements, names_sponsor_amount, validate_layout_census,
    },
    placement::{RelationCasePlan, classify_relation_cases},
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_analysis},
    source::{OperandRole, RequiredSourceKind, SourceRequirement},
};

struct Pilot {
    relations: CompilerRelationAnalysis,
    candidate: ProofPlanCandidate,
    cases: Vec<ExecutionCase>,
    plans: Vec<RelationCasePlan>,
    eligibility: Vec<CarrierEligibility>,
    requirements: Vec<LayoutRequirement>,
}

fn pilot(operation: OperationId) -> Pilot {
    let input = bound_input(&[operation]);
    let relations = build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("plans")
        .candidates;
    let candidate = candidates.first().expect("a feasible candidate").clone();
    let cases = execution_cases(&relations, &candidate).expect("cases");
    let plans = classify_relation_cases(&relations, &cases).expect("classification");
    let eligibility = relation_case_eligibility(&relations, &plans).expect("eligibility");
    let requirements =
        layout_requirements(&relations, &plans, &eligibility).expect("layout requirements");

    Pilot {
        relations,
        candidate,
        cases,
        plans,
        eligibility,
        requirements,
    }
}

fn id(operation: OperationId, kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(operation, kind, subject)
}

fn conservation(operation: OperationId) -> RelationId {
    id(
        operation,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

fn protocol_object(operation: OperationId) -> ObjectId {
    match operation {
        OperationId::CompactAsh => ObjectId::Ash,
        _ => ObjectId::ReceiptLive,
    }
}

// --- global conservation (§16.5) ---

#[test]
fn conservation_requires_a_complete_carrier_and_authenticated_family_totals() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let relation = conservation(operation);
        let object = protocol_object(operation);

        // Only a complete global coordinator is eligible; no member
        // carrier can discharge a claim about family totals.
        for analysis in pilot
            .eligibility
            .iter()
            .filter(|analysis| analysis.relation == relation)
        {
            assert_ne!(
                analysis.eligible,
                [] as [crate::carrier::EligibleCarrier; 0]
            );

            for entry in &analysis.eligible {
                assert!(
                    matches!(entry.carrier, CarrierRole::OperationGlobal { .. }),
                    "{:?}",
                    entry.carrier,
                );
            }
        }

        for side in [TransactionSide::Input, TransactionSide::Output] {
            assert!(
                pilot
                    .requirements
                    .contains(&LayoutRequirement::AuthenticateFamilyCensus {
                        relation: relation.clone(),
                        side,
                        object,
                    }),
                "{operation:?} {side:?}",
            );
            assert!(
                pilot
                    .requirements
                    .contains(&LayoutRequirement::CompleteAndDisjointFamilies {
                        relation: relation.clone(),
                        side,
                    }),
            );
        }
    }
}

#[test]
fn a_canonical_coordinator_is_required_at_the_mandatory_input_family() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        assert!(
            pilot
                .requirements
                .contains(&LayoutRequirement::CanonicalCoordinator {
                    operation,
                    anchor: CarrierRole::OperationGlobal {
                        operation,
                        anchor: protocol_object(operation),
                    },
                }),
        );

        // No coordinator is ever anchored in the optional sponsor
        // family.
        for requirement in &pilot.requirements {
            if let LayoutRequirement::CanonicalCoordinator { anchor, .. } = requirement {
                assert_ne!(
                    *anchor,
                    CarrierRole::OperationGlobal {
                        operation,
                        anchor: ObjectId::PlainLbtc,
                    },
                );
            }
        }
    }
}

#[test]
fn every_layout_provided_source_has_an_explicit_routing_requirement() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        for analysis in &pilot.eligibility {
            for entry in &analysis.eligible {
                for routing in entry.layout_provided() {
                    assert!(
                        pilot
                            .requirements
                            .contains(&LayoutRequirement::MakeSourceAvailable {
                                relation: analysis.relation.clone(),
                                case: analysis.case.clone(),
                                carrier: entry.carrier.clone(),
                                source: routing.source.clone(),
                            }),
                        "{:?} at {:?}",
                        analysis.relation,
                        entry.carrier,
                    );
                }
            }
        }
    }
}

// --- external conservation (§16.6) ---

#[test]
fn substrate_conservation_stays_external_with_no_carrier_and_no_layout() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let relation = id(
            operation,
            RelationKind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        );

        // No runtime carrier.
        assert!(
            !pilot
                .eligibility
                .iter()
                .any(|analysis| analysis.relation == relation)
        );

        // No layout requirement stands in for the missing evidence.
        assert!(
            !pilot
                .requirements
                .iter()
                .any(|requirement| requirement.relation() == Some(&relation)),
        );

        // The typed evidence requirement and its capability are both
        // retained rather than discharged.
        assert!(
            pilot
                .candidate
                .required_capabilities
                .contains(&RequiredCapability::WholeTransactionValueConservation)
        );
        assert!(pilot.candidate.external_evidence.iter().any(|requirement| {
            matches!(
                requirement,
                realization::ExternalEvidenceRequirement::SubstrateConservation {
                    operation: named,
                    asset: AssetId::Lbtc,
                } if *named == operation
            )
        }));
    }
}

// --- sponsor requirements and erasure (§8.4, §16.7) ---

#[test]
fn both_sponsor_cases_require_an_isolated_region() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let relation = id(
            operation,
            RelationKind::SponsorIsolation,
            RelationSubject::Sponsor,
        );

        assert_eq!(pilot.cases.len(), 2);

        for case in &pilot.cases {
            assert!(
                pilot
                    .requirements
                    .contains(&LayoutRequirement::IsolateSponsorRegion {
                        relation: relation.clone(),
                        case: case.id.clone(),
                    }),
                "{:?}",
                case.id.sponsor,
            );
        }

        // The sponsorless case is not a silent omission: the global
        // relation still requires the region to be provably absent.
        let sponsorless = pilot
            .cases
            .iter()
            .find(|case| case.id.sponsor == SponsorCase::Absent)
            .expect("a sponsorless case");
        assert!(
            pilot
                .requirements
                .contains(&LayoutRequirement::IsolateSponsorRegion {
                    relation,
                    case: sponsorless.id.clone(),
                }),
        );
    }
}

#[test]
fn sponsor_envelope_multiplicity_and_disjointness_are_required() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let multiplicity = id(
            operation,
            RelationKind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
        );

        for side in [TransactionSide::Input, TransactionSide::Output] {
            assert!(
                pilot
                    .requirements
                    .contains(&LayoutRequirement::CompleteAndDisjointFamilies {
                        relation: multiplicity.clone(),
                        side,
                    }),
            );
        }
    }
}

#[test]
fn no_layout_requirement_names_a_sponsor_amount() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);

        for requirement in &pilot.requirements {
            assert!(!names_sponsor_amount(requirement), "{requirement:?}");

            if let LayoutRequirement::MakeSourceAvailable { source, .. } = requirement {
                assert!(!matches!(
                    source.operand.role(),
                    OperandRole::ObjectFamilyAmount {
                        object: ObjectId::PlainLbtc,
                        ..
                    },
                ));
            }
        }
    }
}

#[test]
fn a_requirement_naming_a_sponsor_amount_is_rejected() {
    let pilot = pilot(OperationId::TransferLive);
    let analysis = pilot.eligibility.first().expect("an analysis");
    let mut forged = pilot.requirements.clone();

    forged.push(LayoutRequirement::MakeSourceAvailable {
        relation: analysis.relation.clone(),
        case: analysis.case.clone(),
        carrier: CarrierRole::OperationGlobal {
            operation: OperationId::TransferLive,
            anchor: ObjectId::ReceiptLive,
        },
        source: SourceRequirement {
            operand: crate::OperandId::new(
                analysis.relation.clone(),
                OperandRole::ObjectFamilyAmount {
                    side: TransactionSide::Input,
                    object: ObjectId::PlainLbtc,
                },
            ),
            source: RequiredSourceKind::AuthenticatedConsensusValue,
            availability: realization::AvailabilityClass::Public,
            activation: crate::source::RequirementActivation::WhenSponsorPresent,
        },
    });

    assert_eq!(
        validate_layout_census(&pilot.plans, &pilot.eligibility, &forged),
        Err(CompileError::SponsorValueRead),
    );
}

// --- structural obligations and census completeness ---

#[test]
fn representation_and_secret_free_paths_reach_the_layout() {
    let ash = pilot(OperationId::CompactAsh);
    let object = ObjectId::Ash;
    let representation = ash.cases[0].id.representations[&object];

    assert!(ash.requirements.iter().any(|requirement| matches!(
        requirement,
        LayoutRequirement::EnforceRepresentation {
            object: named,
            representation: mode,
            ..
        } if *named == object && *mode == representation
    )));

    // Compact ASH is the permissionless pilot, so its path must be
    // secret-free.
    assert!(ash.requirements.iter().any(|requirement| matches!(
        requirement,
        LayoutRequirement::SecretFreeOperationPath { .. }
    )));

    // Live transfer is owner-authorized and declares no such path.
    let live = pilot(OperationId::TransferLive);
    assert!(!live.requirements.iter().any(|requirement| matches!(
        requirement,
        LayoutRequirement::SecretFreeOperationPath { .. }
    )));
}

#[test]
fn a_dropped_layout_requirement_is_rejected() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let mut short = pilot.requirements.clone();

        // Every requirement is either a carrier dependency or a
        // structural obligation, so dropping any one of them leaves the
        // census incomplete.
        let position = short
            .iter()
            .position(|requirement| {
                matches!(requirement, LayoutRequirement::MakeSourceAvailable { .. })
            })
            .expect("a routing requirement");
        short.remove(position);

        assert!(matches!(
            validate_layout_census(&pilot.plans, &pilot.eligibility, &short),
            Err(CompileError::MissingLayoutRequirement { .. }),
        ));
    }
}

#[test]
fn requirements_are_canonically_ordered_and_free_of_duplicates() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let mut sorted = pilot.requirements.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(sorted, pilot.requirements);
    }
}

#[test]
fn repeated_layout_derivation_is_equal() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let pilot = pilot(operation);
        let second = layout_requirements(&pilot.relations, &pilot.plans, &pilot.eligibility)
            .expect("second derivation");

        assert_eq!(pilot.requirements, second);
    }
}

fn assert_operator_layout(relation: realization::Relation, kind: RelationKind) {
    let mut pilot = pilot(OperationId::CompactAsh);
    let id = RelationId::new(
        OperationId::AnnounceMaturity,
        kind,
        RelationSubject::Operation,
    );
    let node = crate::relation::CompilerRelationNode {
        id: crate::relation::AnalysisNodeId::SourceRelation(id.clone()),
        source: realization::RelationDeclaration {
            id: id.clone(),
            relation,
            proof_alternatives: std::collections::BTreeSet::new(),
        },
    };
    pilot.relations.graph.add_node(node);
    let mut case = pilot.cases[0].id.clone();
    case.operation = OperationId::AnnounceMaturity;
    let eligibility = CarrierEligibility {
        relation: id,
        case,
        scope: crate::placement::SemanticScope::TransactionGlobal,
        multiplicity: crate::placement::CarrierMultiplicity::ExactlyOne,
        eligible: Vec::new(),
        ineligible: Vec::new(),
    };
    assert!(
        crate::layout::derive_layout_requirements(&pilot.relations, &[], &[eligibility])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn operator_authorization_introduces_no_layout_obligation() {
    assert_operator_layout(
        realization::Relation::OperatorAuthorization,
        RelationKind::Authorization,
    );
}

#[test]
fn operator_constructibility_introduces_no_layout_obligation() {
    assert_operator_layout(
        realization::Relation::Constructibility {
            class: realization::ConstructibilityClass::Operator,
        },
        RelationKind::Constructibility,
    );
}
