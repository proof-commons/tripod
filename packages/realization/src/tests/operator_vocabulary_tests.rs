//! Operator authorization remains an explicit external premise.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use petgraph::graph::DiGraph;

use crate::{
    ConstructibilityClass, ExternalEvidenceRequirement, ObservedCanonicalPartition,
    OperationObservation, OwnerId, RealizationError, Relation, RelationDeclaration, RelationId,
    RelationKind, RelationStatus, RelationSubject, TransactionSide,
};

fn declaration(kind: RelationKind, relation: Relation) -> RelationDeclaration {
    RelationDeclaration {
        id: RelationId::new(
            OperationId::AnnounceMaturity,
            kind,
            RelationSubject::Operation,
        ),
        relation,
        proof_alternatives: BTreeSet::new(),
    }
}

fn assert_operator_evidence(relation: Relation, kind: RelationKind, signed: bool) {
    let declaration = declaration(kind, relation);
    let id = declaration.id.clone();
    let (graph, nodes, order) = crate::relation::build_relation_graph([declaration], []).unwrap();
    let observation = OperationObservation {
        operation: OperationId::AnnounceMaturity,
        objects: Vec::new(),
        protocol_signers: if signed {
            BTreeSet::from([OwnerId([7; 32])])
        } else {
            BTreeSet::new()
        },
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: Vec::new(),
        },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::new(),
        bounds: BTreeMap::new(),
    };
    let report = crate::evaluate::evaluate_operation(
        &graph,
        &nodes,
        &order,
        &DiGraph::new(),
        &BTreeMap::new(),
        &[],
        &observation,
    )
    .unwrap();
    assert_eq!(
        report.verdict(&id).unwrap().status,
        RelationStatus::EvidenceRequired {
            requirement: ExternalEvidenceRequirement::OperatorAuthorization {
                operation: id.operation()
            },
        }
    );
    assert!(!report.is_evidence_complete());
}

#[test]
fn operator_authorization_without_signers_requires_evidence() {
    assert_operator_evidence(
        Relation::OperatorAuthorization,
        RelationKind::Authorization,
        false,
    );
}

#[test]
fn operator_authorization_with_signers_requires_evidence() {
    assert_operator_evidence(
        Relation::OperatorAuthorization,
        RelationKind::Authorization,
        true,
    );
}

#[test]
fn operator_constructibility_without_signers_requires_evidence() {
    assert_operator_evidence(
        Relation::Constructibility {
            class: ConstructibilityClass::Operator,
        },
        RelationKind::Constructibility,
        false,
    );
}

#[test]
fn operator_constructibility_with_signers_requires_evidence() {
    assert_operator_evidence(
        Relation::Constructibility {
            class: ConstructibilityClass::Operator,
        },
        RelationKind::Constructibility,
        true,
    );
}

#[test]
fn operator_authorization_weld_accepts_operation_subject() {
    crate::validate::validate_relation_identity(&declaration(
        RelationKind::Authorization,
        Relation::OperatorAuthorization,
    ))
    .unwrap();
}

#[test]
fn operator_authorization_weld_rejects_owner_subject() {
    let mut declaration = declaration(RelationKind::Authorization, Relation::OperatorAuthorization);
    declaration.id = RelationId::new(
        OperationId::AnnounceMaturity,
        RelationKind::Authorization,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    assert!(matches!(
        crate::validate::validate_relation_identity(&declaration),
        Err(RealizationError::RelationSubjectMismatch {
            expected: RelationSubject::Operation,
            ..
        })
    ));
}

#[test]
fn operator_authorization_weld_rejects_wrong_kind() {
    assert!(matches!(
        crate::validate::validate_relation_identity(&declaration(
            RelationKind::Constructibility,
            Relation::OperatorAuthorization
        )),
        Err(RealizationError::RelationKindMismatch {
            expected: RelationKind::Authorization,
            ..
        })
    ));
}

#[test]
fn operator_evidence_has_an_operation_and_no_asset() {
    let requirement = ExternalEvidenceRequirement::OperatorAuthorization {
        operation: OperationId::AnnounceMaturity,
    };
    assert_eq!(requirement.operation(), OperationId::AnnounceMaturity);
    assert_eq!(requirement.asset(), None);
}

fn assert_fact(fact: crate::FactId, ty: crate::SemanticType, reason: crate::DisclosureReason) {
    assert_eq!(fact.operation(), Some(OperationId::AnnounceMaturity));
    assert_eq!(fact.semantic_type(), ty);
    let (graph, nodes) = crate::declassification::build_disclosure_graph(
        [crate::DisclosureNode::Fact {
            id: fact.clone(),
            initial_visibility: crate::InitialVisibility::Private,
        }],
        [],
    )
    .unwrap();
    assert!(matches!(
        &graph[nodes[&crate::DisclosureNodeId::Fact(fact.clone())]],
        crate::DisclosureNode::Fact {
            initial_visibility: crate::InitialVisibility::Public,
            ..
        }
    ));
    let analysis = crate::declassification::analyze_disclosure(&graph, &nodes, &[]).unwrap();
    assert_eq!(
        analysis.required_public,
        BTreeMap::from([(fact, BTreeSet::from([reason]))]),
    );
    assert!(analysis.newly_disclosed.is_empty());
    assert!(analysis.retained_private.is_empty());
}

fn state_fact(side: TransactionSide, field: crate::StateField) -> crate::FactId {
    crate::FactId::StateField {
        operation: OperationId::AnnounceMaturity,
        side,
        field,
    }
}

#[test]
fn state_field_names_are_complete_distinct_and_in_declaration_order() {
    use crate::StateField;
    assert_eq!(
        StateField::ALL,
        &[
            StateField::Omega,
            StateField::YL,
            StateField::YT,
            StateField::Q,
            StateField::Cycle,
            StateField::Maturity
        ]
    );
    let names = StateField::ALL
        .iter()
        .map(|field| field.name())
        .collect::<Vec<_>>();
    assert_eq!(names, ["omega", "y-l", "y-t", "q", "cycle", "maturity"]);
    assert_eq!(names.into_iter().collect::<BTreeSet<_>>().len(), 6);
}

#[test]
fn state_omega_has_its_owner_type_and_public_state_seed() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        assert_fact(
            state_fact(side, crate::StateField::Omega),
            crate::SemanticType::Amount,
            crate::DisclosureReason::PublicState,
        );
    }
}

#[test]
fn state_yl_has_its_owner_type_and_public_state_seed() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        assert_fact(
            state_fact(side, crate::StateField::YL),
            crate::SemanticType::Amount,
            crate::DisclosureReason::PublicState,
        );
    }
}

#[test]
fn state_yt_has_its_owner_type_and_public_state_seed() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        assert_fact(
            state_fact(side, crate::StateField::YT),
            crate::SemanticType::Amount,
            crate::DisclosureReason::PublicState,
        );
    }
}

#[test]
fn state_q_has_its_owner_type_and_public_state_seed() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        assert_fact(
            state_fact(side, crate::StateField::Q),
            crate::SemanticType::Amount,
            crate::DisclosureReason::PublicState,
        );
    }
}

#[test]
fn state_cycle_has_its_owner_type_and_public_state_seed() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        assert_fact(
            state_fact(side, crate::StateField::Cycle),
            crate::SemanticType::Cycle,
            crate::DisclosureReason::PublicState,
        );
    }
}

#[test]
fn state_maturity_has_its_owner_type_and_public_state_seed() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        assert_fact(
            state_fact(side, crate::StateField::Maturity),
            crate::SemanticType::Maturity,
            crate::DisclosureReason::PublicState,
        );
    }
}

#[test]
fn requested_cycle_has_its_owner_type_and_public_request_seed() {
    assert_fact(
        crate::FactId::RequestedAnnouncementCycle {
            operation: OperationId::AnnounceMaturity,
        },
        crate::SemanticType::Cycle,
        crate::DisclosureReason::PublicRequest,
    );
}

#[test]
fn minimum_lead_has_its_owner_type_and_public_request_seed() {
    assert_fact(
        crate::FactId::AnnouncementLead {
            operation: OperationId::AnnounceMaturity,
            bound: crate::AnnouncementLeadBound::Minimum,
        },
        crate::SemanticType::Cycle,
        crate::DisclosureReason::PublicRequest,
    );
}

#[test]
fn maximum_lead_has_its_owner_type_and_public_request_seed() {
    assert_fact(
        crate::FactId::AnnouncementLead {
            operation: OperationId::AnnounceMaturity,
            bound: crate::AnnouncementLeadBound::Maximum,
        },
        crate::SemanticType::Cycle,
        crate::DisclosureReason::PublicRequest,
    );
}

fn reject_numeric(field: crate::StateField, sum: bool) {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        use crate::{ExprId, ExpressionDeclaration, ExpressionNode, ExpressionRole, SemanticType};
        let fact = state_fact(side, field);
        let ty = fact.semantic_type();
        let input = ExprId::fact(fact.clone());
        let id = ExprId::relation(
            RelationId::new(
                OperationId::AnnounceMaturity,
                RelationKind::ExpressionPredicate,
                RelationSubject::Operation,
            ),
            ExpressionRole::Predicate,
        );
        let node = if sum {
            ExpressionNode::CheckedSum {
                ty,
                terms: vec![input.clone()],
            }
        } else {
            ExpressionNode::LessOrEqual {
                left: input.clone(),
                right: input.clone(),
            }
        };
        let error = crate::build_expression_graph([
            ExpressionDeclaration {
                id: input,
                ty,
                node: ExpressionNode::Fact(fact),
            },
            ExpressionDeclaration {
                id,
                ty: if sum { ty } else { SemanticType::Bool },
                node,
            },
        ])
        .unwrap_err();
        assert_eq!(
            error,
            if sum {
                RealizationError::InvalidSumType(ty)
            } else {
                RealizationError::InvalidOrderedType(ty)
            }
        );
    }
}

#[test]
fn checked_sum_refuses_cycle() {
    reject_numeric(crate::StateField::Cycle, true);
}

#[test]
fn less_or_equal_refuses_cycle() {
    reject_numeric(crate::StateField::Cycle, false);
}

#[test]
fn checked_sum_refuses_maturity() {
    reject_numeric(crate::StateField::Maturity, true);
}

#[test]
fn less_or_equal_refuses_maturity() {
    reject_numeric(crate::StateField::Maturity, false);
}

#[test]
fn cycle_values_have_only_the_cycle_accessor() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        let cycle = crate::Cycle::new(8);
        let value = crate::SemanticValue::Cycle(cycle);
        assert_eq!(value.semantic_type(), crate::SemanticType::Cycle);
        assert_eq!(value.as_cycle(), Some(cycle));
        assert_eq!(value.as_maturity(), None);
        assert_eq!(value.as_amount(), None);
        assert_eq!(value.as_count(), None);
        assert_eq!(value.as_bool(), None);
        assert_eq!(value.as_owner_set(), None);
        let mut facts = crate::FactValues::default();
        facts
            .insert(state_fact(side, crate::StateField::Cycle), value)
            .unwrap();
    }
}

#[test]
fn maturity_values_have_only_the_maturity_accessor() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        for maturity in [
            crate::Maturity::Unannounced,
            crate::Maturity::Announced {
                cycle: crate::Cycle::new(9),
            },
            crate::Maturity::Complete,
        ] {
            let value = crate::SemanticValue::Maturity(maturity);
            assert_eq!(value.semantic_type(), crate::SemanticType::Maturity);
            assert_eq!(value.as_maturity(), Some(maturity));
            assert_eq!(value.as_cycle(), None);
            assert_eq!(value.as_amount(), None);
            assert_eq!(value.as_count(), None);
            assert_eq!(value.as_bool(), None);
            assert_eq!(value.as_owner_set(), None);
            let mut facts = crate::FactValues::default();
            facts
                .insert(state_fact(side, crate::StateField::Maturity), value)
                .unwrap();
        }
    }
}

#[test]
fn state_family_amount_remains_distinct_from_metadata() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        let family = crate::FactId::FamilyAmount {
            operation: OperationId::AnnounceMaturity,
            side: TransactionSide::Input,
            object: ObjectId::State,
        };
        assert_eq!(family.semantic_type(), crate::SemanticType::Amount);
        for field in crate::StateField::ALL {
            assert_ne!(family, state_fact(side, *field));
        }
    }
}

#[test]
fn announcement_selectors_and_published_parameters_have_exact_names() {
    assert_eq!(
        crate::AnnouncementLeadBound::Minimum.bound_id(),
        architecture::BoundId::MaturityLeadMin
    );
    assert_eq!(
        crate::AnnouncementLeadBound::Maximum.bound_id(),
        architecture::BoundId::MaturityLeadMax
    );
    assert_eq!(
        crate::StateLawParameter::ALL,
        &[crate::StateLawParameter::Zeta]
    );
    assert_eq!(crate::StateLawParameter::Zeta.name(), "zeta");
}

#[test]
fn state_side_keys_are_distinct_and_values_do_not_overwrite_each_other() {
    let mut values = crate::FactValues::default();
    for (side, amount) in [(TransactionSide::Input, 3), (TransactionSide::Output, 7)] {
        let fact = state_fact(side, crate::StateField::Omega);
        values
            .insert(
                fact,
                crate::SemanticValue::Amount(crate::ProtocolAmount::new(amount).unwrap()),
            )
            .unwrap();
    }
    assert_ne!(
        state_fact(TransactionSide::Input, crate::StateField::Omega),
        state_fact(TransactionSide::Output, crate::StateField::Omega)
    );
    for (side, amount) in [(TransactionSide::Input, 3), (TransactionSide::Output, 7)] {
        assert_eq!(
            values
                .get(&state_fact(side, crate::StateField::Omega))
                .unwrap()
                .as_amount()
                .unwrap()
                .get(),
            amount
        );
    }
}

#[test]
fn cycle_bounds_cannot_be_inserted_or_declared_as_count_facts() {
    for bound in [
        architecture::BoundId::MaturityLeadMin,
        architecture::BoundId::MaturityLeadMax,
    ] {
        let fact = crate::FactId::BoundValue { bound };
        let expected = RealizationError::CycleBoundUsedAsCount(bound);
        assert_eq!(
            crate::FactValues::default()
                .insert(fact.clone(), crate::SemanticValue::Count(crate::Count::ONE)),
            Err(expected.clone())
        );
        let declaration = crate::ExpressionDeclaration {
            id: crate::ExprId::fact(fact.clone()),
            ty: crate::SemanticType::Count,
            node: crate::ExpressionNode::Fact(fact),
        };
        assert_eq!(
            crate::build_expression_graph([declaration]).unwrap_err(),
            expected
        );
    }
}

#[test]
fn observations_reject_cycle_bounds_in_the_count_map() {
    for bound in [
        architecture::BoundId::MaturityLeadMin,
        architecture::BoundId::MaturityLeadMax,
    ] {
        let observation = OperationObservation {
            operation: OperationId::AnnounceMaturity,
            objects: Vec::new(),
            protocol_signers: BTreeSet::new(),
            sponsor_signers: BTreeSet::new(),
            canonical_partition: ObservedCanonicalPartition {
                issuances: Vec::new(),
                flows: Vec::new(),
            },
            open_flows: Vec::new(),
            root_effects: Vec::new(),
            projections: BTreeSet::new(),
            bounds: BTreeMap::from([(bound, crate::Count::ONE)]),
        };
        assert_eq!(
            observation.validate_and_normalize(),
            Err(RealizationError::CycleBoundUsedAsCount(bound))
        );
    }
}

#[test]
fn state_fields_resolve_to_distinct_architecture_data_identifiers() {
    use crate::StateField;
    use architecture::{ARCHITECTURE, DataId, QuantityId};
    let expected = [
        DataId::StateOmega,
        DataId::StateYLive,
        DataId::StateYTimeLocked,
        DataId::StateQ,
        DataId::StateCycle,
        DataId::StateMaturity,
    ];
    let actual = StateField::ALL
        .iter()
        .map(|field| field.data_id())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert_eq!(actual.iter().copied().collect::<BTreeSet<_>>().len(), 6);
    for id in [QuantityId::Floor, QuantityId::RedemptionPayout] {
        let quantity = ARCHITECTURE.quantity(id).unwrap();
        assert_eq!(quantity.reads, &expected[..3]);
    }
    let issuance = ARCHITECTURE.quantity(QuantityId::CycleIssuance).unwrap();
    assert_eq!(
        issuance.reads.iter().copied().collect::<BTreeSet<_>>(),
        expected[..4].iter().copied().collect::<BTreeSet<_>>()
    );
    let window = ARCHITECTURE
        .quantity(QuantityId::AnnouncementWindow)
        .unwrap();
    assert_eq!(window.reads, &expected[4..]);
}
