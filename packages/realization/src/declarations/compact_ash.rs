//! Target-independent compact-ASH declaration.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    Architecture, AssetId, BoundId, ObjectId, OperationId, ProjectionId, ProjectionRule, RootId,
    RootUse,
};

use crate::{
    CardinalityMaximum, ConstructibilityClass, Count, OperationRealization, ProofAlternativeId,
    ProofKind, RealizationError, Relation, RelationDeclaration, RelationId, RelationKind,
    RelationSubject, RepresentationMode, TransactionSide,
    validate::validate_compact_ash_architecture,
};

#[allow(clippy::too_many_lines)]
pub fn derive(architecture: &Architecture) -> Result<OperationRealization, RealizationError> {
    validate_compact_ash_architecture(architecture)?;

    let input_cardinality = relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    );
    let output_cardinality = relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::Ash,
        },
    );
    let input_recognition = relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    );
    let output_recognition = relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::Ash,
        },
    );
    let conservation = relation_id(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let permissionless = relation_id(RelationKind::Authorization, RelationSubject::Operation);
    let input_closure = relation_id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    );
    let output_closure = relation_id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::Ash,
        },
    );
    let sponsor = relation_id(RelationKind::SponsorIsolation, RelationSubject::Sponsor);
    let roots = relation_id(RelationKind::RootPolicy, RelationSubject::Operation);
    let projections = relation_id(RelationKind::ProjectionPolicy, RelationSubject::Operation);
    let constructibility = relation_id(RelationKind::Constructibility, RelationSubject::Operation);
    let representation = relation_id(
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::Ash,
        },
    );
    let compact_lifecycle = relation_id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::Ash,
            exit: OperationId::CompactAsh,
        },
    );
    let clear_lifecycle = relation_id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::Ash,
            exit: OperationId::Clear,
        },
    );

    let ash_objects = BTreeSet::from([ObjectId::Ash]);
    let allowed_inputs = BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]);
    let allowed_outputs = BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]);
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
        RelationDeclaration {
            id: input_cardinality.clone(),
            relation: Relation::Cardinality {
                side: crate::ObservedSide::Input,
                object: ObjectId::Ash,
                minimum: Count::new(2),
                maximum: CardinalityMaximum::Bound(BoundId::AshBatchMax),
            },
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(input_cardinality.clone(), ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: output_cardinality.clone(),
            relation: Relation::Cardinality {
                side: crate::ObservedSide::Output,
                object: ObjectId::Ash,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Exact(Count::ONE),
            },
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(output_cardinality.clone(), ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: input_recognition.clone(),
            relation: Relation::Recognition {
                side: crate::ObservedSide::Input,
                object: ObjectId::Ash,
                asset: AssetId::U,
            },
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(input_recognition.clone(), ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: output_recognition.clone(),
            relation: Relation::Recognition {
                side: crate::ObservedSide::Output,
                object: ObjectId::Ash,
                asset: AssetId::U,
            },
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(output_recognition.clone(), ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: permissionless.clone(),
            relation: Relation::PermissionlessAuthorization,
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(permissionless.clone(), ProofKind::PublicConstructibility),
        },
        RelationDeclaration {
            id: input_closure.clone(),
            relation: Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Input,
                allowed: allowed_inputs,
            },
            prerequisites: BTreeSet::from([permissionless.clone()]),
            proof_alternatives: proofs(input_closure.clone(), ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: output_closure.clone(),
            relation: Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Output,
                allowed: allowed_outputs,
            },
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(output_closure.clone(), ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: conservation.clone(),
            relation: Relation::AmountConservation {
                asset: AssetId::U,
                input_objects: ash_objects.clone(),
                output_objects: ash_objects,
            },
            prerequisites: BTreeSet::from([
                input_cardinality,
                output_cardinality,
                input_recognition.clone(),
                output_recognition.clone(),
                input_closure,
                output_closure,
            ]),
            proof_alternatives: proofs(conservation, ProofKind::PublicArithmetic),
        },
        RelationDeclaration {
            id: sponsor.clone(),
            relation: Relation::SponsorIsolation,
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(sponsor.clone(), ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: roots.clone(),
            relation: Relation::RootPolicy {
                expected: expected_roots,
            },
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(roots, ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: projections.clone(),
            relation: Relation::ProjectionPolicy {
                expected: expected_projections,
            },
            prerequisites: BTreeSet::new(),
            proof_alternatives: proofs(projections, ProofKind::ManifestShape),
        },
        RelationDeclaration {
            id: constructibility.clone(),
            relation: Relation::Constructibility {
                class: ConstructibilityClass::PublicPermissionless,
            },
            prerequisites: BTreeSet::from([permissionless, input_recognition, sponsor]),
            proof_alternatives: proofs(constructibility.clone(), ProofKind::PublicConstructibility),
        },
        RelationDeclaration {
            id: representation.clone(),
            relation: Relation::Representation {
                object: ObjectId::Ash,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PublicCommitted,
                ]),
            },
            prerequisites: BTreeSet::from([constructibility, output_recognition]),
            proof_alternatives: proofs(representation.clone(), ProofKind::PublicConstructibility),
        },
        RelationDeclaration {
            id: compact_lifecycle,
            relation: Relation::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::CompactAsh,
            },
            prerequisites: BTreeSet::from([representation.clone()]),
            proof_alternatives: BTreeSet::new(),
        },
        RelationDeclaration {
            id: clear_lifecycle,
            relation: Relation::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::Clear,
            },
            prerequisites: BTreeSet::from([representation]),
            proof_alternatives: BTreeSet::new(),
        },
    ];

    Ok(OperationRealization {
        operation: OperationId::CompactAsh,
        expressions: Vec::new(),
        relations,
    })
}

fn relation_id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::CompactAsh, kind, subject)
}

fn proofs(relation: RelationId, kind: ProofKind) -> BTreeSet<ProofAlternativeId> {
    BTreeSet::from([ProofAlternativeId::new(relation, kind)])
}
