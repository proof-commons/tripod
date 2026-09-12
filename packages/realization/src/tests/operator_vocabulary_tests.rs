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

fn state_fact(field: crate::StateField) -> crate::FactId {
    crate::FactId::StateField {
        operation: OperationId::AnnounceMaturity,
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
    assert_fact(
        state_fact(crate::StateField::Omega),
        crate::SemanticType::Amount,
        crate::DisclosureReason::PublicState,
    );
}

#[test]
fn state_yl_has_its_owner_type_and_public_state_seed() {
    assert_fact(
        state_fact(crate::StateField::YL),
        crate::SemanticType::Amount,
        crate::DisclosureReason::PublicState,
    );
}

#[test]
fn state_yt_has_its_owner_type_and_public_state_seed() {
    assert_fact(
        state_fact(crate::StateField::YT),
        crate::SemanticType::Amount,
        crate::DisclosureReason::PublicState,
    );
}

#[test]
fn state_q_has_its_owner_type_and_public_state_seed() {
    assert_fact(
        state_fact(crate::StateField::Q),
        crate::SemanticType::Amount,
        crate::DisclosureReason::PublicState,
    );
}

#[test]
fn state_cycle_has_its_owner_type_and_public_state_seed() {
    assert_fact(
        state_fact(crate::StateField::Cycle),
        crate::SemanticType::Cycle,
        crate::DisclosureReason::PublicState,
    );
}

#[test]
fn state_maturity_has_its_owner_type_and_public_state_seed() {
    assert_fact(
        state_fact(crate::StateField::Maturity),
        crate::SemanticType::Maturity,
        crate::DisclosureReason::PublicState,
    );
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
    use crate::{ExprId, ExpressionDeclaration, ExpressionNode, ExpressionRole, SemanticType};
    let fact = state_fact(field);
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
        .insert(state_fact(crate::StateField::Cycle), value)
        .unwrap();
}

#[test]
fn maturity_values_have_only_the_maturity_accessor() {
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
            .insert(state_fact(crate::StateField::Maturity), value)
            .unwrap();
    }
}

#[test]
fn state_family_amount_remains_distinct_from_metadata() {
    let family = crate::FactId::FamilyAmount {
        operation: OperationId::AnnounceMaturity,
        side: TransactionSide::Input,
        object: ObjectId::State,
    };
    assert_eq!(family.semantic_type(), crate::SemanticType::Amount);
    for field in crate::StateField::ALL {
        assert_ne!(family, state_fact(*field));
    }
}
