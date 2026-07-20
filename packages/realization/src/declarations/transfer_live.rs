//! Target-independent live-receipt transfer declaration.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    Architecture, AssetId, BoundId, ObjectId, OperationId, ProjectionId, ProjectionRule, RootId,
    RootUse,
};

use crate::{
    CardinalityMaximum, ConstructibilityClass, Count, ObservedSide, OperationRealization,
    ProofAlternativeId, ProofKind, RealizationError, Relation, RelationDeclaration, RelationId,
    RelationKind, RelationSubject, RepresentationMode, TransactionSide,
    validate::validate_live_transfer_architecture,
};

#[allow(clippy::too_many_lines)]
pub fn derive(architecture: &Architecture) -> Result<OperationRealization, RealizationError> {
    validate_live_transfer_architecture(architecture)?;

    let input_cardinality = id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    let output_cardinality = id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        },
    );
    let sponsor_input_cardinality = id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    );
    let sponsor_output_cardinality = id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::PlainLbtc,
        },
    );
    let input_recognition = id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    let output_recognition = id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        },
    );
    let authorization = id(
        RelationKind::Authorization,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    let input_closure = id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    let output_closure = id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        },
    );
    let conservation = id(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let sponsor = id(RelationKind::SponsorIsolation, RelationSubject::Sponsor);
    let roots = id(RelationKind::RootPolicy, RelationSubject::Operation);
    let projections = id(RelationKind::ProjectionPolicy, RelationSubject::Operation);
    let constructibility = id(RelationKind::Constructibility, RelationSubject::Operation);
    let representation = id(
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::ReceiptLive,
        },
    );
    let transfer_exit = lifecycle(OperationId::TransferLive);
    let burn_exit = lifecycle(OperationId::Burn);
    let redeem_exit = lifecycle(OperationId::Redeem);

    let expected_roots = RootId::ALL
        .iter()
        .map(|root| (*root, RootUse::Forbidden))
        .collect::<BTreeMap<_, _>>();
    let expected_projections = ProjectionId::ALL
        .iter()
        .map(|projection| {
            let rule = if *projection == ProjectionId::TransitionCertificate {
                ProjectionRule::Required
            } else {
                ProjectionRule::Forbidden
            };

            (*projection, rule)
        })
        .collect::<BTreeMap<_, _>>();

    let relations = vec![
        declaration(
            input_cardinality.clone(),
            Relation::Cardinality {
                side: ObservedSide::Input,
                object: ObjectId::ReceiptLive,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Bound(BoundId::TransferInputMax),
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            output_cardinality.clone(),
            Relation::Cardinality {
                side: ObservedSide::Output,
                object: ObjectId::ReceiptLive,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Bound(BoundId::TransferOutputMax),
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            sponsor_input_cardinality.clone(),
            Relation::Cardinality {
                side: ObservedSide::Input,
                object: ObjectId::PlainLbtc,
                minimum: Count::ZERO,
                maximum: CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            sponsor_output_cardinality.clone(),
            Relation::Cardinality {
                side: ObservedSide::Output,
                object: ObjectId::PlainLbtc,
                minimum: Count::ZERO,
                maximum: CardinalityMaximum::Exact(Count::ONE),
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            input_recognition.clone(),
            Relation::Recognition {
                side: ObservedSide::Input,
                object: ObjectId::ReceiptLive,
                asset: AssetId::U,
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            output_recognition.clone(),
            Relation::Recognition {
                side: ObservedSide::Output,
                object: ObjectId::ReceiptLive,
                asset: AssetId::U,
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            authorization.clone(),
            Relation::OwnerAuthorization {
                object: ObjectId::ReceiptLive,
            },
            [input_recognition.clone()],
            [ProofKind::SignerMembership],
        ),
        declaration(
            input_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: ObservedSide::Input,
                allowed: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]),
            },
            [authorization.clone()],
            [ProofKind::ManifestShape],
        ),
        declaration(
            output_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: ObservedSide::Output,
                allowed: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]),
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            conservation.clone(),
            Relation::AmountConservation {
                asset: AssetId::U,
                input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
                output_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            },
            [
                input_cardinality,
                output_cardinality,
                input_recognition,
                output_recognition,
                input_closure,
                output_closure,
            ],
            [
                ProofKind::PublicArithmetic,
                ProofKind::ConfidentialConservation,
            ],
        ),
        declaration(
            sponsor,
            Relation::SponsorIsolation,
            [sponsor_input_cardinality, sponsor_output_cardinality],
            [ProofKind::ManifestShape],
        ),
        declaration(
            roots,
            Relation::RootPolicy {
                expected: expected_roots,
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            projections,
            Relation::ProjectionPolicy {
                expected: expected_projections,
            },
            [],
            [ProofKind::ManifestShape],
        ),
        declaration(
            constructibility.clone(),
            Relation::Constructibility {
                class: ConstructibilityClass::OwnersOf {
                    object: ObjectId::ReceiptLive,
                },
            },
            [authorization],
            [ProofKind::SignerMembership],
        ),
        declaration(
            representation.clone(),
            Relation::Representation {
                object: ObjectId::ReceiptLive,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PrivateCommitted,
                ]),
            },
            [conservation, constructibility],
            [
                ProofKind::PublicArithmetic,
                ProofKind::ConfidentialConservation,
            ],
        ),
        declaration(
            transfer_exit,
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::TransferLive,
            },
            [representation.clone()],
            [],
        ),
        declaration(
            burn_exit,
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::Burn,
            },
            [representation.clone()],
            [],
        ),
        declaration(
            redeem_exit,
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::Redeem,
            },
            [representation],
            [],
        ),
    ];

    Ok(OperationRealization {
        operation: OperationId::TransferLive,
        expressions: Vec::new(),
        relations,
    })
}

fn id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::TransferLive, kind, subject)
}

fn lifecycle(exit: OperationId) -> RelationId {
    id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::ReceiptLive,
            exit,
        },
    )
}

fn declaration<const P: usize, const A: usize>(
    id: RelationId,
    relation: Relation,
    prerequisites: [RelationId; P],
    proof_kinds: [ProofKind; A],
) -> RelationDeclaration {
    let proof_alternatives = proof_kinds
        .into_iter()
        .map(|kind| ProofAlternativeId::new(id.clone(), kind))
        .collect();

    RelationDeclaration {
        id,
        relation,
        prerequisites: prerequisites.into_iter().collect(),
        proof_alternatives,
    }
}
