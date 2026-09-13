//! The weld between a relation's identity and its body.
//!
//! Every case here builds a relation whose declared key and declared
//! body disagree, and requires the owner validator to say so. The
//! declarations the pilots actually ship are built by careful
//! constructors, which is exactly why these cases are hand-built: what
//! is under test is the owner's derivation, not the constructors'.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ARCHITECTURE, AssetId, ObjectId, OperationId, ProjectionId, RootId, RootUse};

use crate::validate::{validate_relation_identity, validate_scoped_realization};
use crate::{
    CardinalityMaximum, Count, ExprId, ExpressionRole, ObservedSide, RealizationError,
    RealizationScope, Relation, RelationDeclaration, RelationId, RelationKind, RelationSubject,
    TransactionSide, derive,
};

fn declaration(id: RelationId, relation: Relation) -> RelationDeclaration {
    RelationDeclaration {
        id,
        relation,
        proof_alternatives: BTreeSet::new(),
    }
}

fn id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::TransferLive, kind, subject)
}

fn family(side: TransactionSide, object: ObjectId) -> RelationSubject {
    RelationSubject::ObjectFamily { side, object }
}

fn cardinality_body(side: ObservedSide) -> Relation {
    Relation::Cardinality {
        side,
        object: ObjectId::ReceiptLive,
        minimum: Count::ONE,
        maximum: CardinalityMaximum::Exact(Count::new(2)),
    }
}

#[test]
fn an_honest_cardinality_relation_passes_its_own_weld() {
    let declaration = declaration(
        id(
            RelationKind::Cardinality,
            family(TransactionSide::Input, ObjectId::ReceiptLive),
        ),
        cardinality_body(ObservedSide::Input),
    );
    validate_relation_identity(&declaration).expect("identity and body agree");
}

#[test]
fn a_transposed_relation_kind_rejects() {
    // The body is a cardinality relation; the key calls it recognition.
    let declaration = declaration(
        id(
            RelationKind::Recognition,
            family(TransactionSide::Input, ObjectId::ReceiptLive),
        ),
        cardinality_body(ObservedSide::Input),
    );
    let error = validate_relation_identity(&declaration).expect_err("the transposition is refused");
    assert!(
        matches!(
            error,
            RealizationError::RelationKindMismatch {
                expected: RelationKind::Cardinality,
                ..
            }
        ),
        "the kind mismatch must name the body's own family, got {error}",
    );
}

#[test]
fn a_transposed_transaction_side_rejects() {
    // Both sides exist and both are well typed, so nothing but the weld
    // catches a key filed under the wrong one.
    let declaration = declaration(
        id(
            RelationKind::Cardinality,
            family(TransactionSide::Output, ObjectId::ReceiptLive),
        ),
        cardinality_body(ObservedSide::Input),
    );
    let error = validate_relation_identity(&declaration).expect_err("the transposition is refused");
    assert!(
        matches!(error, RealizationError::RelationSubjectMismatch { .. }),
        "a wrong side is a subject mismatch, got {error}",
    );
}

#[test]
fn a_conservation_relation_filed_under_another_asset_rejects() {
    let declaration = declaration(
        id(
            RelationKind::Conservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ),
        Relation::AmountConservation {
            asset: AssetId::U,
            input_objects: BTreeSet::new(),
            output_objects: BTreeSet::new(),
        },
    );
    let error = validate_relation_identity(&declaration).expect_err("the wrong asset is refused");
    assert!(
        matches!(error, RealizationError::RelationSubjectMismatch { .. }),
        "a wrong asset is a subject mismatch, got {error}",
    );
}

#[test]
fn a_closure_relation_filed_under_any_family_rejects() {
    // A closure constrains a side, not a family, so no family is its
    // subject — neither one it forbids nor one it admits. Admitting the
    // members was the old defect: every admitted family did equally
    // well, so the key was a choice rather than an identity.
    for object in [ObjectId::PlainLbtc, ObjectId::ReceiptLive] {
        let declaration = declaration(
            id(
                RelationKind::AllowedObjectFamilies,
                family(TransactionSide::Input, object),
            ),
            Relation::AllowedObjectFamilies {
                side: ObservedSide::Input,
                allowed: BTreeSet::from([ObjectId::ReceiptLive]),
            },
        );
        let error =
            validate_relation_identity(&declaration).expect_err("no family is a closure's subject");
        assert!(
            matches!(error, RealizationError::RelationSubjectMismatch { .. }),
            "a family subject on a closure is a subject mismatch, got {error}",
        );
    }
}

#[test]
fn a_root_policy_filed_under_any_root_rejects() {
    // A root policy fixes what the whole operation may do with every
    // root, so it singles out none of them. Governed and ungoverned
    // roots alike are refused; only the operation remains.
    let expected = BTreeMap::from([(RootId::ALL[0], RootUse::Forbidden)]);
    for root in RootId::ALL.iter().copied() {
        let declaration = declaration(
            id(RelationKind::RootPolicy, RelationSubject::Root { root }),
            Relation::RootPolicy {
                expected: expected.clone(),
            },
        );
        let error =
            validate_relation_identity(&declaration).expect_err("no root is a policy's subject");
        assert!(
            matches!(error, RealizationError::RelationSubjectMismatch { .. }),
            "a root subject on a root policy is a subject mismatch, got {error}",
        );
    }
}

#[test]
fn an_expression_predicate_relation_has_an_honest_identity() {
    // The point of the new kind: before it existed this body could only
    // be declared under a key describing some other relation family.
    let subject = RelationSubject::Operation;
    let expression = ExprId::relation(
        id(RelationKind::ExpressionPredicate, subject.clone()),
        ExpressionRole::Predicate,
    );
    let declaration = declaration(
        id(RelationKind::ExpressionPredicate, subject),
        Relation::ExpressionPredicate { expression },
    );
    validate_relation_identity(&declaration).expect("identity and body agree");
}

#[test]
fn an_expression_predicate_relation_cannot_borrow_another_kind() {
    let expression = ExprId::relation(
        id(RelationKind::Authorization, RelationSubject::Operation),
        ExpressionRole::Predicate,
    );
    let declaration = declaration(
        id(RelationKind::Authorization, RelationSubject::Operation),
        Relation::ExpressionPredicate { expression },
    );
    let error = validate_relation_identity(&declaration).expect_err("the borrowed kind is refused");
    assert!(
        matches!(
            error,
            RealizationError::RelationKindMismatch {
                expected: RelationKind::ExpressionPredicate,
                ..
            }
        ),
        "the body's family is the expression predicate, got {error}",
    );
}

#[test]
fn the_pilot_realization_passes_the_weld_unmodified() {
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .expect("the pilot realization derives");
    for operation in realization.operations.values() {
        for relation in &operation.relations {
            validate_relation_identity(relation)
                .unwrap_or_else(|error| panic!("pilot relation is not welded: {error}"));
        }
    }
}

#[test]
fn a_surplus_architecture_family_relation_rejects() {
    let mut realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .expect("the pilot realization derives");
    // A well-formed relation, welded to its own body, for a family the
    // architecture never declared on this operation. Only the other
    // direction of the family census can see it.
    let operation = realization
        .operations
        .get_mut(&OperationId::TransferLive)
        .expect("the pilot declares the live transfer");
    operation.relations.push(declaration(
        id(
            RelationKind::Recognition,
            family(TransactionSide::Input, ObjectId::Ash),
        ),
        Relation::Recognition {
            side: ObservedSide::Input,
            object: ObjectId::Ash,
            asset: AssetId::U,
        },
    ));

    let error = validate_scoped_realization(&ARCHITECTURE, &realization)
        .expect_err("the surplus family relation is refused");
    assert!(
        matches!(error, RealizationError::SurplusArchitectureRelation { .. }),
        "a surplus family row is its own failure, got {error}",
    );
}

#[test]
fn a_projection_policy_is_subjected_to_the_operation_and_not_to_a_projection_it_governs() {
    let expected = BTreeMap::from([(
        ProjectionId::TransitionCertificate,
        architecture::ProjectionRule::Required,
    )]);
    let policy = |subject| {
        declaration(
            id(RelationKind::ProjectionPolicy, subject),
            Relation::ProjectionPolicy {
                expected: expected.clone(),
            },
        )
    };

    validate_relation_identity(&policy(RelationSubject::Operation))
        .expect("the operation whose projections are fixed is the subject");

    // Governing a projection is not being about it. The policy governs
    // every projection in the vocabulary — by requiring, permitting, or
    // forbidding each — so a governed projection is no more its subject
    // than an ungoverned one.
    let error = validate_relation_identity(&policy(RelationSubject::Projection {
        projection: ProjectionId::TransitionCertificate,
    }))
    .expect_err("no projection is a projection policy's subject");
    assert!(
        matches!(error, RealizationError::RelationSubjectMismatch { .. }),
        "a projection subject on a projection policy is a subject mismatch, got {error}",
    );
}

#[test]
fn announcement_relations_satisfy_the_identity_weld() {
    let realization = super::announce_maturity_tests::realization();
    for relation in realization.relations() {
        validate_relation_identity(relation).unwrap();
    }
    validate_scoped_realization(&ARCHITECTURE, &realization).unwrap();
}

#[test]
fn state_is_a_surplus_family_in_the_receipt_pilot() {
    let mut realization = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::TransferLive]).unwrap(),
    )
    .unwrap();
    let extra = declaration(
        id(
            RelationKind::Recognition,
            family(TransactionSide::Input, ObjectId::State),
        ),
        Relation::Recognition {
            side: ObservedSide::Input,
            object: ObjectId::State,
            asset: AssetId::Pid,
        },
    );
    let relation = extra.id.clone();
    realization
        .operations
        .get_mut(&OperationId::TransferLive)
        .unwrap()
        .relations
        .push(extra);
    assert_eq!(
        validate_scoped_realization(&ARCHITECTURE, &realization),
        Err(RealizationError::SurplusArchitectureRelation { relation })
    );
}

#[test]
fn announcement_rejects_a_surplus_receipt_family() {
    let mut declaration = super::announce_maturity_tests::realization()
        .operations
        .remove(&OperationId::AnnounceMaturity)
        .unwrap();
    let relation = RelationId::new(
        OperationId::AnnounceMaturity,
        RelationKind::Recognition,
        family(TransactionSide::Input, ObjectId::ReceiptLive),
    );
    declaration.relations.push(crate::RelationDeclaration {
        id: relation.clone(),
        relation: Relation::Recognition {
            side: ObservedSide::Input,
            object: ObjectId::ReceiptLive,
            asset: AssetId::U,
        },
        proof_alternatives: BTreeSet::new(),
    });
    assert_eq!(
        super::announce_maturity_tests::rebuild(declaration).unwrap_err(),
        RealizationError::SurplusArchitectureRelation { relation }
    );
}
