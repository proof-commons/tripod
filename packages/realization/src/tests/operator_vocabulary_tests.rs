//! Operator authorization remains an explicit external premise.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use petgraph::graph::DiGraph;

use crate::{
    ConstructibilityClass, ExternalEvidenceRequirement, ObservedCanonicalPartition,
    OperationObservation, OwnerId, RealizationError, Relation, RelationDeclaration,
    RelationId, RelationKind, RelationStatus, RelationSubject, TransactionSide,
};

fn declaration(kind: RelationKind, relation: Relation) -> RelationDeclaration {
    RelationDeclaration {
        id: RelationId::new(OperationId::AnnounceMaturity, kind, RelationSubject::Operation),
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
        protocol_signers: if signed { BTreeSet::from([OwnerId([7; 32])]) } else { BTreeSet::new() },
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition { issuances: Vec::new(), flows: Vec::new() },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::new(),
        bounds: BTreeMap::new(),
    };
    let report = crate::evaluate::evaluate_operation(
        &graph, &nodes, &order, &DiGraph::new(), &BTreeMap::new(), &[], &observation,
    ).unwrap();
    assert_eq!(report.verdict(&id).unwrap().status, RelationStatus::EvidenceRequired {
        requirement: ExternalEvidenceRequirement::OperatorAuthorization { operation: id.operation() },
    });
    assert!(!report.is_evidence_complete());
}

#[test]
fn operator_authorization_without_signers_requires_evidence() {
    assert_operator_evidence(Relation::OperatorAuthorization, RelationKind::Authorization, false);
}

#[test]
fn operator_authorization_with_signers_requires_evidence() {
    assert_operator_evidence(Relation::OperatorAuthorization, RelationKind::Authorization, true);
}

#[test]
fn operator_constructibility_without_signers_requires_evidence() {
    assert_operator_evidence(Relation::Constructibility { class: ConstructibilityClass::Operator }, RelationKind::Constructibility, false);
}

#[test]
fn operator_constructibility_with_signers_requires_evidence() {
    assert_operator_evidence(Relation::Constructibility { class: ConstructibilityClass::Operator }, RelationKind::Constructibility, true);
}

#[test]
fn operator_authorization_weld_accepts_operation_subject() {
    crate::validate::validate_relation_identity(&declaration(RelationKind::Authorization, Relation::OperatorAuthorization)).unwrap();
}

#[test]
fn operator_authorization_weld_rejects_owner_subject() {
    let mut declaration = declaration(RelationKind::Authorization, Relation::OperatorAuthorization);
    declaration.id = RelationId::new(OperationId::AnnounceMaturity, RelationKind::Authorization, RelationSubject::ObjectFamily { side: TransactionSide::Input, object: ObjectId::ReceiptLive });
    assert!(matches!(crate::validate::validate_relation_identity(&declaration), Err(RealizationError::RelationSubjectMismatch { expected: RelationSubject::Operation, .. })));
}

#[test]
fn operator_authorization_weld_rejects_wrong_kind() {
    assert!(matches!(crate::validate::validate_relation_identity(&declaration(RelationKind::Constructibility, Relation::OperatorAuthorization)), Err(RealizationError::RelationKindMismatch { expected: RelationKind::Authorization, .. })));
}

#[test]
fn operator_evidence_has_an_operation_and_no_asset() {
    let requirement = ExternalEvidenceRequirement::OperatorAuthorization { operation: OperationId::AnnounceMaturity };
    assert_eq!(requirement.operation(), OperationId::AnnounceMaturity);
    assert_eq!(requirement.asset(), None);
}
