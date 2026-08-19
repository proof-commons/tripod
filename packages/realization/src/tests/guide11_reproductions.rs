//! Guide-11 reproductions and, where a wave has landed, guarantees.
//!
//! # Two kinds of test live here
//!
//! Every test began as a reproduction: it demonstrated a finding from
//! the Guide-11 preflight register by *passing* while the defect was
//! present, asserting that the wrong thing happened. As each wave
//! repairs its finding it flips the assertions of that finding's tests,
//! which then stand as the guarantee that the repair holds.
//!
//! `G11-R08` is **CLOSED** by Wave 4. Its tests below assert the safe
//! behaviour: each body determines exactly one subject, so the bodies
//! that once admitted several typed keys now admit one and refuse the
//! rest. The mutation tests at the end change only the subject of an
//! otherwise correct declaration, which is the smallest edit that could
//! move a key while leaving the semantics alone.
//!
//! No test here touches a production code path: they are constructions
//! over the public and crate-visible surfaces exactly as an external
//! caller or the existing suites reach them.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId, ProjectionId, ProjectionRule, RootId, RootUse};

use crate::validate::validate_relation_identity;
use crate::{
    ObservedSide, RealizationError, Relation, RelationDeclaration, RelationId, RelationKind,
    RelationSubject, TransactionSide,
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

/// Assert that one declaration is refused for its subject, and that the
/// refusal names the subject the body actually determines.
#[track_caller]
fn refused_for_subject(declaration: &RelationDeclaration, expected: &RelationSubject) {
    let error = validate_relation_identity(declaration)
        .expect_err("a subject other than the derived one must be refused");

    assert_eq!(
        error,
        RealizationError::RelationSubjectMismatch {
            declared: declaration.id.clone(),
            expected: expected.clone(),
        },
    );
}

/// `G11-R08`: a closure body is welded to its side, and to no family.
///
/// The closure constrains which families may appear on one side. Filing
/// it under a family it admits would put the closure under a member of
/// its own result — and every other admitted member would serve just as
/// well, which is the defect. The side is the subject; each admitted
/// family is refused.
#[test]
fn g11_r08_one_closure_body_has_exactly_the_side_as_its_subject() {
    let allowed = BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]);
    let body = || Relation::AllowedObjectFamilies {
        side: ObservedSide::Input,
        allowed: allowed.clone(),
    };
    let expected = RelationSubject::TransactionSide {
        side: TransactionSide::Input,
    };

    validate_relation_identity(&declaration(
        id(RelationKind::AllowedObjectFamilies, expected.clone()),
        body(),
    ))
    .expect("the side the closure constrains is its subject");

    for object in [ObjectId::ReceiptLive, ObjectId::PlainLbtc] {
        refused_for_subject(
            &declaration(
                id(
                    RelationKind::AllowedObjectFamilies,
                    RelationSubject::ObjectFamily {
                        side: TransactionSide::Input,
                        object,
                    },
                ),
                body(),
            ),
            &expected,
        );
    }

    // The other side is refused too: both sides exist and both are well
    // typed, so a transposition is the sharpest way a closure could be
    // misfiled.
    refused_for_subject(
        &declaration(
            id(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide {
                    side: TransactionSide::Output,
                },
            ),
            body(),
        ),
        &expected,
    );
}

/// `G11-R08`: a root-policy body is welded to the operation, and to no
/// root it governs.
#[test]
fn g11_r08_one_root_policy_body_has_exactly_the_operation_as_its_subject() {
    let Some(first) = RootId::ALL.first().copied() else {
        return;
    };
    let expected = BTreeMap::from([(first, RootUse::Forbidden)]);
    let body = || Relation::RootPolicy {
        expected: expected.clone(),
    };

    validate_relation_identity(&declaration(
        id(RelationKind::RootPolicy, RelationSubject::Operation),
        body(),
    ))
    .expect("a root policy fixes what the whole operation may do");

    refused_for_subject(
        &declaration(
            id(
                RelationKind::RootPolicy,
                RelationSubject::Root { root: first },
            ),
            body(),
        ),
        &RelationSubject::Operation,
    );
}

/// `G11-R08`: a projection-policy body is welded to the operation, and
/// to no projection it governs.
#[test]
fn g11_r08_one_projection_policy_body_has_exactly_the_operation_as_its_subject() {
    let Some(first) = ProjectionId::ALL.first().copied() else {
        return;
    };
    let expected = BTreeMap::from([(first, ProjectionRule::Forbidden)]);
    let body = || Relation::ProjectionPolicy {
        expected: expected.clone(),
    };

    validate_relation_identity(&declaration(
        id(RelationKind::ProjectionPolicy, RelationSubject::Operation),
        body(),
    ))
    .expect("a projection policy fixes what the whole operation may derive");

    refused_for_subject(
        &declaration(
            id(
                RelationKind::ProjectionPolicy,
                RelationSubject::Projection { projection: first },
            ),
            body(),
        ),
        &RelationSubject::Operation,
    );
}

/// `G11-R08`: the delta and open-flow policies name no projection, and
/// are subjected to none.
///
/// Neither body names a projection, so there was no membership check to
/// make and none was made: the key could move to any projection in the
/// vocabulary while the body stayed identical. Both are operation-wide
/// policies, and the operation is now the only subject either admits.
#[test]
fn g11_r08_the_delta_and_open_flow_policies_have_exactly_the_operation_as_their_subject() {
    validate_relation_identity(&declaration(
        id(
            RelationKind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
        ),
        canonical_delta_body(),
    ))
    .expect("the delta policy fixes what the whole operation may settle");
    validate_relation_identity(&declaration(
        id(RelationKind::OpenFlowPolicy, RelationSubject::Operation),
        open_flow_body(),
    ))
    .expect("the open-flow policy fixes what the whole operation may leave open");

    assert!(
        ProjectionId::ALL.len() > 1,
        "more than one projection exists, so the refused alternatives are genuinely several",
    );
    for projection in ProjectionId::ALL.iter().copied() {
        refused_for_subject(
            &declaration(
                id(
                    RelationKind::CanonicalDeltaPolicy,
                    RelationSubject::Projection { projection },
                ),
                canonical_delta_body(),
            ),
            &RelationSubject::Operation,
        );
        refused_for_subject(
            &declaration(
                id(
                    RelationKind::OpenFlowPolicy,
                    RelationSubject::Projection { projection },
                ),
                open_flow_body(),
            ),
            &RelationSubject::Operation,
        );
    }
}

/// `G11-R08`: changing only the subject fails derivation.
///
/// Each row below is a body paired with the subject it determines and
/// one wrong subject of a different class. Nothing but the subject
/// moves: the body, the kind, and the operation are those of a
/// declaration that validates. This is the mutation the old predicate
/// could not see, because it asked whether a subject was *among* the
/// admissible ones rather than whether it was *the* one.
#[test]
fn g11_r08_changing_only_the_relation_subject_fails_derivation() {
    let rows: Vec<(Relation, RelationKind, RelationSubject, RelationSubject)> = vec![
        (
            Relation::AllowedObjectFamilies {
                side: ObservedSide::Output,
                allowed: BTreeSet::from([ObjectId::ReceiptLive]),
            },
            RelationKind::AllowedObjectFamilies,
            RelationSubject::TransactionSide {
                side: TransactionSide::Output,
            },
            RelationSubject::Operation,
        ),
        (
            Relation::PermissionlessAuthorization,
            RelationKind::Authorization,
            RelationSubject::Operation,
            RelationSubject::Sponsor,
        ),
        (
            Relation::OwnerAuthorization {
                object: ObjectId::ReceiptLive,
            },
            RelationKind::Authorization,
            RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                object: ObjectId::ReceiptLive,
            },
            RelationSubject::TransactionSide {
                side: TransactionSide::Input,
            },
        ),
        (
            Relation::SponsorIsolation,
            RelationKind::SponsorIsolation,
            RelationSubject::Sponsor,
            RelationSubject::Operation,
        ),
        (
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::Burn,
            },
            RelationKind::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::Burn,
            },
            RelationSubject::Representation {
                object: ObjectId::ReceiptLive,
            },
        ),
    ];

    for (body, kind, derived, mutated) in rows {
        validate_relation_identity(&declaration(id(kind, derived.clone()), body.clone()))
            .expect("the derived subject validates before the mutation");
        assert_ne!(derived, mutated, "the mutation must change the subject");
        refused_for_subject(&declaration(id(kind, mutated), body), &derived);
    }
}
