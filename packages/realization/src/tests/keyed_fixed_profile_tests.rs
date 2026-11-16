use crate::TransactionSide::{Input, Output};
use crate::keyed_fixed::{
    KeyedFixedRealizationDraft as Draft, KeyedFixedRealizationError as Error,
};
use crate::keyed_fixed_relation::{
    KeyedFixedCardinality as Count, KeyedFixedRelation as Relation, RecordRecurrence,
};
use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId as Op};

fn control() -> (KeyedFixedManifest, Draft) {
    let manifest = KeyedFixedManifest::contract().unwrap();
    let draft = Draft::derive(&manifest).unwrap();
    draft.clone().validate(&manifest).unwrap();
    (manifest, draft)
}

#[test]
fn complete_relation_census_and_side_identity() {
    let (manifest, draft) = control();
    let expected = [
        (
            Op::Source,
            vec![
                Relation::HandleFamily {
                    side: Input,
                    cardinality: Count::Zero,
                },
                Relation::HandleFamily {
                    side: Output,
                    cardinality: Count::One,
                },
                Relation::SourceClassification,
                Relation::MembershipReading {
                    recurrence: RecordRecurrence::SourceSingleton,
                },
            ],
            vec![
                [3, 1, 3, 1, 1],
                [3, 1, 3, 1, 2],
                [3, 1, 3, 1, 6],
                [3, 1, 3, 1, 7],
            ],
        ),
        (
            Op::Merge,
            vec![
                Relation::HandleFamily {
                    side: Input,
                    cardinality: Count::OneOrMore,
                },
                Relation::HandleFamily {
                    side: Output,
                    cardinality: Count::One,
                },
                Relation::BornFirst,
                Relation::OwnQualification,
                Relation::SuccessorQualification,
                Relation::MembershipReading {
                    recurrence: RecordRecurrence::ConsumedUnion,
                },
            ],
            vec![
                [3, 2, 3, 1, 1],
                [3, 2, 3, 1, 2],
                [3, 2, 3, 1, 3],
                [3, 2, 3, 1, 4],
                [3, 2, 3, 1, 5],
                [3, 2, 3, 1, 7],
            ],
        ),
    ];
    for (operation, relations, encodings) in expected {
        let actual = crate::declarations::derive_operation(&manifest, operation).unwrap();
        assert_eq!(
            actual
                .relations
                .iter()
                .map(|row| row.id.encoding())
                .collect::<Vec<_>>(),
            encodings
        );
        assert_eq!(
            actual
                .relations
                .iter()
                .map(|r| r.relation)
                .collect::<Vec<_>>(),
            relations
        );
    }
    let scoped = draft.clone().validate(&manifest).unwrap();
    scoped.validate_against(&manifest).unwrap();
    assert_eq!(scoped.binding(), &draft.binding);
    assert_eq!(scoped.graph().node_count(), 10);
    let rows = scoped.project();
    let encodings = rows
        .iter()
        .map(|row| row.id.encoding())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(encodings.len(), 10);
    assert!(encodings.iter().all(|row| row[0] == 3 && row[2] == 3));
    let mut permuted = scoped.draft();
    permuted.operations.reverse();
    for operation in &mut permuted.operations {
        operation.relations.reverse();
    }
    assert_eq!(permuted.validate(&manifest).unwrap().project(), rows);
}

#[test]
fn relation_kind_identity_mismatch_is_refused() {
    let (manifest, mut subject) = control();
    let row = subject
        .operations
        .iter_mut()
        .find(|operation| operation.operation == Op::Merge)
        .unwrap()
        .relations
        .iter_mut()
        .find(|row| row.relation == Relation::BornFirst)
        .unwrap();
    row.id.kind = crate::keyed_fixed_relation::KeyedFixedRelationKind::MembershipReading;
    let expected = Error::RelationIdentity(row.id);
    assert_eq!(subject.validate(&manifest).unwrap_err(), expected);
}

#[test]
fn sealed_entry_rejects_invalid_manifest_before_declaring() {
    let (mut manifest, _) = control();
    manifest.schema_version = 2;
    let expected =
        Error::InvalidManifest(architecture::keyed_fixed::KeyedFixedManifestError::SchemaVersion);
    assert_eq!(Draft::derive(&manifest).unwrap_err(), expected);
    assert_eq!(
        crate::declarations::derive_operation(&manifest, Op::Source).unwrap_err(),
        expected
    );
}

#[test]
fn every_omitted_or_changed_relation_is_refused() {
    let (manifest, draft) = control();
    for (op_index, operation) in draft.operations.iter().enumerate() {
        for (row_index, row) in operation.relations.iter().enumerate() {
            let mut subject = draft.clone();
            subject.operations[op_index].relations.remove(row_index);
            assert_eq!(
                subject.validate(&manifest).unwrap_err(),
                Error::DeclarationCensus(operation.operation)
            );
            let mut subject = draft.clone();
            subject.operations[op_index].relations.push(*row);
            assert_eq!(
                subject.validate(&manifest).unwrap_err(),
                Error::DuplicateRelation(row.id)
            );
            let mut subject = draft.clone();
            subject.operations[op_index].relations[row_index]
                .id
                .operation = if operation.operation == Op::Source {
                Op::Merge
            } else {
                Op::Source
            };
            assert!(matches!(
                subject.validate(&manifest),
                Err(Error::RelationIdentity(_))
            ));
            if let Relation::HandleFamily { side, .. } = row.relation {
                let mut subject = draft.clone();
                subject.operations[op_index].relations[row_index].relation =
                    Relation::HandleFamily {
                        side,
                        cardinality: if side == Input {
                            Count::One
                        } else {
                            Count::Zero
                        },
                    };
                assert_eq!(
                    subject.validate(&manifest).unwrap_err(),
                    Error::DeclarationCensus(operation.operation)
                );
            }
            if let Relation::MembershipReading { recurrence } = row.relation {
                let mut subject = draft.clone();
                subject.operations[op_index].relations[row_index].relation =
                    Relation::MembershipReading {
                        recurrence: if recurrence == RecordRecurrence::SourceSingleton {
                            RecordRecurrence::ConsumedUnion
                        } else {
                            RecordRecurrence::SourceSingleton
                        },
                    };
                assert_eq!(
                    subject.validate(&manifest).unwrap_err(),
                    Error::DeclarationCensus(operation.operation)
                );
            }
        }
    }
    let mut subject = draft.clone();
    subject.operations.pop();
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        Error::OperationCensus
    );
    let mut subject = draft.clone();
    subject.operations.push(subject.operations[0].clone());
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        Error::OperationCensus
    );
    for coordinate in 0..3 {
        let mut subject = draft.clone();
        match coordinate {
            0 => {
                subject.binding.profile =
                    architecture::semantic::SemanticProfileId::KeyedProvenance;
            }
            1 => subject.binding.schema_version = 2,
            _ => {
                subject.binding.digest = architecture::validate_draft(
                    &architecture::keyed::KeyedManifest::contract().unwrap(),
                )
                .unwrap()
                .binding()
                .digest;
            }
        }
        assert_eq!(
            subject.validate(&manifest).unwrap_err(),
            Error::ArchitectureBindingMismatch
        );
    }
}

#[test]
fn empty_stage_fields_are_revalidated_against_attestation_injection() {
    let (manifest, accepted) = control();
    let reference = crate::declarations::derive_operation(
        &architecture::ARCHITECTURE,
        architecture::OperationId::CompactAsh,
    )
    .unwrap();
    macro_rules! refused_field {
        ($field:ident) => {{
            let mut subject = accepted.clone();
            subject.operations[0]
                .$field
                .push(reference.$field[0].clone());
            assert_eq!(
                subject.validate(&manifest).unwrap_err(),
                Error::DeclarationCensus(Op::Source)
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
        side: Input,
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
        Error::DeclarationCensus(Op::Source)
    );
}

#[test]
fn positivity_and_exact_member_pair_form_and_position() {
    use crate::keyed_fixed::{
        KeyedFixedField::{Explicit, Other},
        KeyedFixedParameters, KeyedFixedValue as Value, KeyedFixedValueRefusal,
        is_keyed_fixed_member,
    };
    assert_eq!(Value::new(0), Err(KeyedFixedValueRefusal::Zero));
    assert_eq!(Value::new(1).unwrap().get(), 1);
    // No selected target ceiling in this target-independent owner.
    assert_eq!(Value::new(u64::MAX).unwrap().get(), u64::MAX);
    for (key, asset, value, program) in [
        ([0; 32], [4; 32], 1, vec![1]),
        ([9; 32], [7; 32], 42, vec![2]),
    ] {
        let parameters = KeyedFixedParameters {
            key,
            asset,
            value: Value::new(value).unwrap(),
        };
        assert!(is_keyed_fixed_member(
            parameters,
            &program,
            &program,
            0,
            Explicit(asset),
            Explicit(value)
        ));
        // A changed key alone is invisible here: authenticating the program for
        // the same triple is the caller's obligation, not this predicate's.
        let key_only = KeyedFixedParameters {
            key: [5; 32],
            ..parameters
        };
        assert_ne!(key_only.key, parameters.key);
        assert!(is_keyed_fixed_member(
            key_only,
            &program,
            &program,
            0,
            Explicit(asset),
            Explicit(value)
        ));
        assert!(!is_keyed_fixed_member(
            parameters,
            &program,
            &[3],
            0,
            Explicit(asset),
            Explicit(value)
        ));
        assert!(!is_keyed_fixed_member(
            parameters,
            &program,
            &program,
            1,
            Explicit(asset),
            Explicit(value)
        ));
        assert!(!is_keyed_fixed_member(
            parameters,
            &program,
            &program,
            0,
            Explicit([8; 32]),
            Explicit(value)
        ));
        assert!(!is_keyed_fixed_member(
            parameters,
            &program,
            &program,
            0,
            Explicit(asset),
            Explicit(value + 1)
        ));
        assert!(!is_keyed_fixed_member(
            parameters,
            &program,
            &program,
            0,
            Other,
            Explicit(value)
        ));
        assert!(!is_keyed_fixed_member(
            parameters,
            &program,
            &program,
            0,
            Explicit(asset),
            Other
        ));
    }
}
