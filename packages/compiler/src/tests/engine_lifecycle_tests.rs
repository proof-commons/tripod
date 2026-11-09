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
        let manifest = KeyedManifest::probe().expect("paper binding");
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
        let analyzed = engine_records::analyze_bound_lifecycle(&input);
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
    let lowered = engine_records::analyze_bound_lifecycle(&input);
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
fn keyed_lifecycle_constraints_cannot_pass_as_an_empty_result() {
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
