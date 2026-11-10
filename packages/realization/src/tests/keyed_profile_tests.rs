use crate::keyed::{KeyedRealizationDraft, KeyedRealizationError};
use crate::relation::KeyedRelation;
use architecture::keyed::KeyedManifest;
use architecture::semantic::{KeyedOperationId, SemanticOperationId, SemanticProfileId};

fn control() -> (KeyedManifest, KeyedRealizationDraft) {
    let manifest = KeyedManifest::contract().unwrap();
    let draft = KeyedRealizationDraft::derive(&manifest).unwrap();
    draft.clone().validate(&manifest).unwrap();
    (manifest, draft)
}

#[test]
fn stable_source_and_merge_have_exactly_the_nine_paper_relations() {
    use crate::TransactionSide;
    use crate::relation::{KeyedCardinality, RecordRecurrence};
    let (_, accepted) = control();
    for (id, expected) in [
        (
            KeyedOperationId::Source,
            vec![
                KeyedRelation::HandleFamily {
                    side: TransactionSide::Input,
                    cardinality: KeyedCardinality::Zero,
                },
                KeyedRelation::HandleFamily {
                    side: TransactionSide::Output,
                    cardinality: KeyedCardinality::One,
                },
                KeyedRelation::SourceClassification,
                KeyedRelation::MembershipReading {
                    recurrence: RecordRecurrence::SourceSingleton,
                },
            ],
        ),
        (
            KeyedOperationId::Merge,
            vec![
                KeyedRelation::HandleFamily {
                    side: TransactionSide::Input,
                    cardinality: KeyedCardinality::OneOrMore,
                },
                KeyedRelation::HandleFamily {
                    side: TransactionSide::Output,
                    cardinality: KeyedCardinality::One,
                },
                KeyedRelation::BornFirst,
                KeyedRelation::ContinueFirst,
                KeyedRelation::MembershipReading {
                    recurrence: RecordRecurrence::ConsumedUnion,
                },
            ],
        ),
    ] {
        let operation = accepted
            .operations
            .iter()
            .find(|op| op.operation == SemanticOperationId::Keyed(id))
            .unwrap();
        assert_eq!(
            operation
                .relations
                .iter()
                .map(|row| row.relation)
                .collect::<Vec<_>>(),
            expected
        );
        assert!(operation.relations.iter().all(|row| row.id.object
            == architecture::semantic::SemanticObjectId::Keyed(
                architecture::semantic::KeyedObjectId::Handle
            )));
    }
}

#[test]
fn old_experimental_and_changed_digest_bindings_are_refused() {
    let (manifest, accepted) = control();
    let mut subject = accepted.clone();
    // Independently reconstructed from the 0.7.95-dev v1 encoder: 135 bytes.
    // The literal must never be upgraded silently by changing only its schema.
    subject.binding = architecture::keyed::KeyedArchitectureBinding {
        profile: SemanticProfileId::KeyedProvenance,
        schema_version: 1,
        digest: [
            82, 29, 248, 235, 187, 21, 83, 172, 38, 247, 128, 67, 180, 202, 46, 205, 5, 250, 41,
            28, 30, 167, 186, 232, 87, 171, 203, 148, 207, 241, 53, 92,
        ],
    };
    assert_eq!(
        subject.clone().validate(&manifest).unwrap_err(),
        KeyedRealizationError::ArchitectureBindingMismatch
    );
    // Only the digest now differs from the accepted schema-2 binding.
    subject.binding.schema_version = 2;
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::ArchitectureBindingMismatch
    );
    let mut subject = accepted;
    subject.binding.digest[0] ^= 1;
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::ArchitectureBindingMismatch
    );
}

#[test]
fn keyed_relation_identity_mutations_are_refused_by_the_owner() {
    use crate::relation::KeyedRelationKind;
    let (manifest, accepted) = control();
    for change in 0..3 {
        let mut subject = accepted.clone();
        let operation = subject.operations[0].operation;
        let row = &mut subject.operations[0].relations[0];
        match change {
            0 => {
                row.id.operation =
                    SemanticOperationId::Attestation(architecture::OperationId::CompactAsh);
            }
            1 => row.id.kind = KeyedRelationKind::MembershipReading,
            _ => {
                row.id.object = architecture::semantic::SemanticObjectId::Attestation(
                    architecture::ObjectId::State,
                );
            }
        }
        let expected = if change == 2 {
            KeyedRealizationError::DeclarationCensus(operation)
        } else {
            KeyedRealizationError::RelationIdentity(row.id)
        };
        assert_eq!(subject.validate(&manifest).unwrap_err(), expected);
    }
}

#[test]
fn keyed_cardinality_and_recurrence_mutations_preserve_the_refusal_boundary() {
    use crate::relation::{KeyedCardinality, RecordRecurrence};
    let (manifest, accepted) = control();
    for operation_index in 0..accepted.operations.len() {
        for recurrence in [false, true] {
            let mut subject = accepted.clone();
            let operation = &mut subject.operations[operation_index];
            let identity = operation.operation;
            let row = operation
                .relations
                .iter_mut()
                .find(|row| {
                    if recurrence {
                        matches!(row.relation, KeyedRelation::MembershipReading { .. })
                    } else {
                        matches!(
                            row.relation,
                            KeyedRelation::HandleFamily {
                                side: crate::TransactionSide::Input,
                                ..
                            }
                        )
                    }
                })
                .unwrap();
            match &mut row.relation {
                KeyedRelation::MembershipReading { recurrence } => {
                    *recurrence = match recurrence {
                        RecordRecurrence::SourceSingleton => RecordRecurrence::ConsumedUnion,
                        RecordRecurrence::ConsumedUnion => RecordRecurrence::SourceSingleton,
                    }
                }
                KeyedRelation::HandleFamily { cardinality, .. } => {
                    *cardinality = KeyedCardinality::One;
                }
                _ => unreachable!("selected relation"),
            }
            assert_eq!(
                subject.validate(&manifest).unwrap_err(),
                KeyedRealizationError::DeclarationCensus(identity)
            );
        }
    }
}

#[test]
fn keyed_output_cardinality_mutations_are_refused_for_both_operations() {
    use crate::relation::KeyedCardinality;
    let (manifest, accepted) = control();
    for (index, declaration) in accepted.operations.iter().enumerate() {
        let mut subject = accepted.clone();
        let row = subject.operations[index]
            .relations
            .iter_mut()
            .find(|row| {
                matches!(
                    row.relation,
                    KeyedRelation::HandleFamily {
                        side: crate::TransactionSide::Output,
                        ..
                    }
                )
            })
            .unwrap();
        row.relation = KeyedRelation::HandleFamily {
            side: crate::TransactionSide::Output,
            cardinality: KeyedCardinality::OneOrMore,
        };
        assert_eq!(
            subject.validate(&manifest).unwrap_err(),
            KeyedRealizationError::DeclarationCensus(declaration.operation)
        );
    }
}

#[test]
fn keyed_declaration_census_refuses_nonempty_shell_fields() {
    let (manifest, accepted) = control();
    let operation = accepted.operations[0].operation;
    let reference = crate::declarations::derive_operation(
        &architecture::ARCHITECTURE,
        architecture::OperationId::CompactAsh,
    )
    .expect("attestation fixture");
    macro_rules! refused_field {
        ($field:ident) => {{
            let mut subject = accepted.clone();
            subject.operations[0]
                .$field
                .push(reference.$field[0].clone());
            assert_eq!(
                subject.validate(&manifest).unwrap_err(),
                KeyedRealizationError::DeclarationCensus(operation)
            );
        }};
    }
    refused_field!(relation_dependencies);
    refused_field!(constructibility_nodes);
    refused_field!(constructibility_edges);
    refused_field!(lifecycle_nodes);
    refused_field!(lifecycle_edges);
    refused_field!(disclosure_nodes);
    refused_field!(disclosure_edges);
    refused_field!(disclosure_seeds);
    let mut subject = accepted;
    let fact = crate::FactId::FamilyCount {
        operation: architecture::OperationId::CompactAsh,
        side: crate::TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    };
    subject.operations[0]
        .expressions
        .push(crate::ExpressionDeclaration {
            id: crate::ExprId::fact(fact.clone()),
            ty: crate::SemanticType::Count,
            node: crate::ExpressionNode::Fact(fact),
        });
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::DeclarationCensus(operation)
    );
}

#[test]
fn a_changed_profile_binding_is_refused_by_the_realization_owner() {
    let (manifest, mut subject) = control();
    subject.binding.profile = SemanticProfileId::Attestation;
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::ArchitectureBindingMismatch
    );
}

#[test]
fn removing_either_clause_is_refused_even_without_dangling_edges() {
    for clause in [KeyedRelation::BornFirst, KeyedRelation::ContinueFirst] {
        let (manifest, mut subject) = control();
        let merge = SemanticOperationId::Keyed(KeyedOperationId::Merge);
        let operation = subject
            .operations
            .iter_mut()
            .find(|operation| operation.operation == merge)
            .unwrap();
        let before = operation.relations.len();
        operation.relations.retain(|row| row.relation != clause);
        assert_eq!(operation.relations.len() + 1, before);
        assert_eq!(
            subject.validate(&manifest).unwrap_err(),
            KeyedRealizationError::DeclarationCensus(merge)
        );
    }
}

#[test]
fn a_duplicated_relation_is_refused_by_the_graph_owner() {
    let (manifest, mut subject) = control();
    let repeated = subject.operations[0].relations[0];
    subject.operations[0].relations.push(repeated);
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::DuplicateRelation(repeated.id)
    );
}

#[test]
fn a_foreign_operation_is_refused_before_graph_assembly() {
    let (manifest, mut subject) = control();
    let foreign = SemanticOperationId::Attestation(architecture::OperationId::CompactAsh);
    subject.operations[0].operation = foreign;
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::ForeignOperation(foreign)
    );
}

#[test]
fn keyed_projection_is_permutation_stable_and_has_no_attestation_graphs() {
    let (manifest, mut subject) = control();
    let baseline = subject.clone().validate(&manifest).unwrap().project();
    assert_eq!(baseline.len(), 9);
    for operation in &mut subject.operations {
        assert_eq!(operation.expressions, []);
        assert_eq!(operation.constructibility_nodes, []);
        assert_eq!(operation.lifecycle_nodes, []);
        assert_eq!(operation.disclosure_nodes, []);
        operation.relations.reverse();
    }
    subject.operations.reverse();
    assert_eq!(subject.validate(&manifest).unwrap().project(), baseline);
}

#[test]
fn a_keyed_declaration_cannot_smuggle_a_lifecycle_constraint() {
    let (manifest, mut subject) = control();
    let operation = subject.operations[0].operation;
    subject.operations[0]
        .lifecycle_nodes
        .push(crate::LifecycleNode {
            id: crate::LifecycleNodeId::Representation {
                object: architecture::ObjectId::State,
                mode: crate::RepresentationMode::Explicit,
            },
        });
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::DeclarationCensus(operation)
    );
}
