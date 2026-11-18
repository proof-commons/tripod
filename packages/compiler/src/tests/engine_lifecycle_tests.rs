//! Shared lifecycle-stage controls and keyed empty-stage preservation.

use crate::{
    bind_input,
    engine_lifecycle::{
        self, ChoiceRecord, LifecycleDependency, LifecycleNodeKey, LifecycleProjection,
        LifecycleRecordError, ModeKey, ObjectKey,
    },
    engine_records::{self, DependencyKind, LoweredRelations, OperationKey},
};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};
use realization::keyed::KeyedRealizationDraft;

#[test]
fn keyed_empty_lifecycle_is_analyzed_and_projected_for_each_scope() {
    let source_id = SemanticOperationId::Keyed(KeyedOperationId::Source);
    let merge_id = SemanticOperationId::Keyed(KeyedOperationId::Merge);
    for scope in [vec![source_id], vec![merge_id], vec![source_id, merge_id]] {
        let manifest = KeyedManifest::contract().expect("paper binding");
        let source = KeyedRealizationDraft::derive(&manifest)
            .expect("declarations")
            .validate(&manifest)
            .expect("owner validation");
        let input = bind_input(
            &manifest,
            source,
            scope,
            super::bound_input(&[OperationId::CompactAsh]).policy(),
        )
        .expect("keyed binding");
        let analyzed = engine_records::analyze_bound_lifecycle(&input).expect("owner lowering");
        let projection = analyzed
            .result
            .expect("shared lifecycle traversal")
            .project();
        assert_eq!(
            projection,
            LifecycleProjection {
                nodes: vec![],
                edges: vec![],
                choices: vec![],
                requirements: vec![]
            }
        );
        assert_eq!(analyzed.lowering.lifecycle().required_choices, []);
    }
}

#[test]
fn the_bound_attestation_lifecycle_view_equals_the_existing_entry() {
    let input = super::bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let relations = crate::relation::build_relation_analysis(&input).expect("graph");
    let expected =
        crate::lifecycle::build_lifecycle_analysis(&input, &relations).expect("existing entry");
    let lowered = engine_records::analyze_bound_lifecycle(&input).expect("owner lowering");
    let actual = lowered
        .lowering
        .lifecycle_compatibility_view(lowered.result)
        .expect("bound record entry");
    assert_eq!(actual.choices, expected.choices);
    assert_eq!(actual.requirements, expected.requirements);
    assert_eq!(
        actual.graph.node_weights().collect::<Vec<_>>(),
        expected.graph.node_weights().collect::<Vec<_>>()
    );
    assert_eq!(
        actual.graph.edge_weights().collect::<Vec<_>>(),
        expected.graph.edge_weights().collect::<Vec<_>>()
    );
    assert_eq!(actual.node_by_id, expected.node_by_id);
}

#[test]
fn keyed_lifecycle_record_mutations_cannot_pass_as_an_empty_result() {
    let operation = OperationKey {
        namespace: 2,
        code: 2,
    };
    let object = ObjectKey {
        namespace: 2,
        code: 1,
    };
    let mode = ModeKey {
        namespace: 2,
        code: 1,
    };
    let control = engine_lifecycle::LifecycleRecords::empty(vec![operation]);
    assert!(engine_lifecycle::analyze_lifecycle(&control).is_ok());
    let mut missing_choice = control;
    missing_choice.required_choices.push((operation, object));
    assert_eq!(
        engine_lifecycle::analyze_lifecycle(&missing_choice).err(),
        Some(LifecycleRecordError::MissingRepresentationChoice { operation, object })
    );

    let mut connected = missing_choice;
    connected.choices.push(ChoiceRecord {
        operation,
        object,
        candidates: vec![mode],
    });
    connected.input_admissions = connected.choices.clone();
    let representation = LifecycleNodeKey::Representation { object, mode };
    let exit = LifecycleNodeKey::RequiredExit { object, operation };
    connected.nodes = vec![representation.clone(), exit.clone()];
    connected.edges.push(LifecycleDependency {
        source: representation,
        target: exit,
        kind: DependencyKind {
            namespace: 2,
            code: 1,
        },
    });
    let accepted = engine_lifecycle::analyze_lifecycle(&connected).expect("path control");
    assert_eq!(accepted.project().requirements.len(), 1);
    let mut missing_path = connected;
    missing_path.edges.clear();
    assert_eq!(
        engine_lifecycle::analyze_lifecycle(&missing_path).err(),
        Some(LifecycleRecordError::MissingLifecyclePath {
            object,
            representation: mode,
            exit: operation
        })
    );
}

#[test]
fn burn_choice_cannot_be_supplied_by_transfer() {
    let burn = OperationKey {
        namespace: 1,
        code: OperationId::Burn.code(),
    };
    let transfer = OperationKey {
        namespace: 1,
        code: OperationId::TransferLive.code(),
    };
    let object = ObjectKey {
        namespace: 1,
        code: architecture::ObjectId::ReceiptLive.code(),
    };
    let mode = ModeKey {
        namespace: 1,
        code: 0,
    };
    let mut records = engine_lifecycle::LifecycleRecords::empty(vec![transfer, burn]);
    records.required_choices.push((burn, object));
    records.choices.push(ChoiceRecord {
        operation: transfer,
        object,
        candidates: vec![mode],
    });
    assert_eq!(
        engine_lifecycle::analyze_lifecycle(&records).err(),
        Some(LifecycleRecordError::MissingRepresentationChoice {
            operation: burn,
            object
        })
    );
    records.choices.push(ChoiceRecord {
        operation: burn,
        object,
        candidates: vec![mode],
    });
    assert!(engine_lifecycle::analyze_lifecycle(&records).is_ok());
}

#[test]
fn exit_eligibility_is_representation_specific() {
    use crate::lifecycle::LifecycleExitStatus;
    assert_literal_input_admissions();
    let burn = OperationKey {
        namespace: 1,
        code: OperationId::Burn.code(),
    };
    let object = ObjectKey {
        namespace: 1,
        code: architecture::ObjectId::ReceiptLive.code(),
    };
    let explicit = ModeKey {
        namespace: 1,
        code: 0,
    };
    let private = ModeKey {
        namespace: 1,
        code: 1,
    };
    let mut records = engine_lifecycle::LifecycleRecords::empty(vec![burn]);
    records.choices.push(ChoiceRecord {
        operation: burn,
        object,
        candidates: vec![explicit, private],
    });
    records.input_admissions.push(ChoiceRecord {
        operation: burn,
        object,
        candidates: vec![explicit],
    });
    let exit = LifecycleNodeKey::RequiredExit {
        object,
        operation: burn,
    };
    records.nodes.push(exit.clone());
    for mode in [explicit, private] {
        let source = LifecycleNodeKey::Representation { object, mode };
        records.nodes.push(source.clone());
        records.edges.push(LifecycleDependency {
            source,
            target: exit.clone(),
            kind: DependencyKind {
                namespace: 1,
                code: 0,
            },
        });
    }
    let rows = engine_lifecycle::analyze_lifecycle(&records)
        .expect("connected graph")
        .requirements;
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].status,
        LifecycleExitStatus::AvailableInCompilerScope
    );
    assert_eq!(
        rows[1].status,
        LifecycleExitStatus::UnavailableInCompilerScope
    );
    records.input_admissions[0].candidates.push(private);
    let changed = engine_lifecycle::analyze_lifecycle(&records)
        .expect("changed admission")
        .requirements;
    assert_eq!(
        changed[1].status,
        LifecycleExitStatus::AvailableInCompilerScope
    );
}

fn assert_literal_input_admissions() {
    let namespace = crate::engine_identity::ATTESTATION;
    let explicit_mode = ModeKey { namespace, code: 0 };
    let private_mode = ModeKey { namespace, code: 1 };
    let public_mode = ModeKey { namespace, code: 2 };
    for (operation, expected) in [
        (
            OperationId::CompactAsh,
            vec![(
                architecture::ObjectId::Ash,
                vec![explicit_mode, public_mode],
            )],
        ),
        (
            OperationId::TransferLive,
            vec![(
                architecture::ObjectId::ReceiptLive,
                vec![explicit_mode, private_mode],
            )],
        ),
        (
            OperationId::AnnounceMaturity,
            vec![(
                architecture::ObjectId::State,
                vec![explicit_mode, public_mode],
            )],
        ),
        (
            OperationId::Burn,
            vec![(architecture::ObjectId::ReceiptLive, vec![explicit_mode])],
        ),
        (
            OperationId::Clear,
            vec![
                (
                    architecture::ObjectId::Ash,
                    vec![explicit_mode, public_mode],
                ),
                (
                    architecture::ObjectId::State,
                    vec![explicit_mode, public_mode],
                ),
            ],
        ),
    ] {
        let source = realization::derive(
            &architecture::ARCHITECTURE,
            realization::RealizationScope::from_operations([operation]).expect("operation scope"),
        )
        .expect("realization declarations");
        let projection = source.project().lifecycle;
        let declaration = source
            .operations()
            .find(|(id, _)| *id == operation)
            .expect("operation realization")
            .1;
        let lowered = crate::engine_attestation::lifecycle::lower(
            &[operation],
            declaration.relations.iter(),
            &projection,
        );
        let mut actual = lowered.records.input_admissions;
        actual.sort_by_key(|row| (row.operation, row.object));
        let mut expected = expected
            .into_iter()
            .map(|(object, candidates)| ChoiceRecord {
                operation: crate::engine_attestation::operation_key(operation),
                object: ObjectKey {
                    namespace,
                    code: object.code(),
                },
                candidates,
            })
            .collect::<Vec<_>>();
        expected.sort_by_key(|row| (row.operation, row.object));
        assert_eq!(actual, expected, "input admission for {operation:?}");
    }
}
