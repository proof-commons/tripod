//! Guide-11 Wave-0 reproductions of the sixth static review.
//!
//! # These tests assert the defect, not the repair
//!
//! Each test demonstrates a finding from the Guide-11 preflight register
//! by *passing* while the defect is present: it asserts that the wrong
//! thing happens. Waves 1 to 4 flip each assertion as they repair the
//! finding, so a test here failing after a repair is the repair working
//! rather than a regression.
//!
//! The finding reproduced here is `G11-R08`: a relation body does not
//! determine one relation subject, so the same semantic body can be filed
//! under several different typed keys. No production code path is touched.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId, ProjectionId, ProjectionRule, RootId, RootUse};

use crate::validate::validate_relation_identity;
use crate::{
    ObservedSide, Relation, RelationDeclaration, RelationId, RelationKind, RelationSubject,
    TransactionSide,
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

/// A delta policy naming no projection at all.
fn canonical_delta_body() -> Relation {
    Relation::CanonicalDeltaPolicy {
        expected: BTreeSet::new(),
    }
}

/// An open-flow policy naming no projection at all.
fn open_flow_body() -> Relation {
    Relation::OpenFlowPolicy {
        allowed: BTreeSet::new(),
    }
}

/// `G11-R08`: one closure body is welded to two different object
/// families.
///
/// `subject_describes_body` admits any member of the admitted set as the
/// subject, so a closure over two families can be filed under either one
/// without changing a byte of its semantics.
#[test]
fn g11_r08_one_closure_body_admits_two_subjects() {
    let allowed = BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]);
    let body = || Relation::AllowedObjectFamilies {
        side: ObservedSide::Input,
        allowed: allowed.clone(),
    };

    for object in [ObjectId::ReceiptLive, ObjectId::PlainLbtc] {
        let declaration = declaration(
            id(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::ObjectFamily {
                    side: TransactionSide::Input,
                    object,
                },
            ),
            body(),
        );
        validate_relation_identity(&declaration)
            .expect("the defect: the same closure body is welded to more than one object family");
    }
}

/// `G11-R08`: one root-policy body is welded to the operation subject and
/// to every root it governs.
#[test]
fn g11_r08_one_root_policy_body_admits_several_subjects() {
    let Some(first) = RootId::ALL.first().copied() else {
        return;
    };
    let expected = BTreeMap::from([(first, RootUse::Forbidden)]);
    let body = || Relation::RootPolicy {
        expected: expected.clone(),
    };

    let subjects = [
        RelationSubject::Operation,
        RelationSubject::Root { root: first },
    ];
    for subject in subjects {
        let declaration = declaration(id(RelationKind::RootPolicy, subject), body());
        validate_relation_identity(&declaration)
            .expect("the defect: one root-policy body is welded to more than one subject");
    }
}

/// `G11-R08`: one projection-policy body is welded to the operation
/// subject and to every projection it governs.
#[test]
fn g11_r08_one_projection_policy_body_admits_several_subjects() {
    let Some(first) = ProjectionId::ALL.first().copied() else {
        return;
    };
    let expected = BTreeMap::from([(first, ProjectionRule::Forbidden)]);
    let body = || Relation::ProjectionPolicy {
        expected: expected.clone(),
    };

    let subjects = [
        RelationSubject::Operation,
        RelationSubject::Projection { projection: first },
    ];
    for subject in subjects {
        let declaration = declaration(id(RelationKind::ProjectionPolicy, subject), body());
        validate_relation_identity(&declaration)
            .expect("the defect: one projection-policy body is welded to more than one subject");
    }
}

/// `G11-R08`: the delta and open-flow policies admit the operation
/// subject and *any* projection at all, governed or not.
///
/// Neither body names a projection, so there is no membership check to
/// make and none is made: the key can move to any projection in the
/// vocabulary while the body stays identical.
#[test]
fn g11_r08_the_delta_and_open_flow_policies_admit_any_projection() {
    for projection in ProjectionId::ALL.iter().copied() {
        let delta = declaration(
            id(
                RelationKind::CanonicalDeltaPolicy,
                RelationSubject::Projection { projection },
            ),
            canonical_delta_body(),
        );
        validate_relation_identity(&delta)
            .expect("the defect: the delta policy is welded to an arbitrary projection");

        let flow = declaration(
            id(
                RelationKind::OpenFlowPolicy,
                RelationSubject::Projection { projection },
            ),
            open_flow_body(),
        );
        validate_relation_identity(&flow)
            .expect("the defect: the open-flow policy is welded to an arbitrary projection");
    }

    // And the operation subject too, so the same body has as many
    // admitted keys as the projection vocabulary has members, plus one.
    validate_relation_identity(&declaration(
        id(
            RelationKind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
        ),
        canonical_delta_body(),
    ))
    .expect("the defect: the same body is also welded to the operation subject");
    assert!(
        ProjectionId::ALL.len() > 1,
        "more than one projection exists, so the alternatives are genuinely several",
    );
}
