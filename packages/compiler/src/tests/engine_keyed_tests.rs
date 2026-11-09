//! Both statically bound owners reach the same relation-record algorithm.
//! These are graph-stage controls, not a claim of complete keyed analysis.

use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};
use realization::keyed::KeyedRealizationDraft;

use crate::{
    bind_input,
    engine_records::{
        self, DependencyKind, DependencyRecord, LoweredRelations, RelationRecordError,
    },
    input::KeyedBindingError,
};

const SOURCE: SemanticOperationId = SemanticOperationId::Keyed(KeyedOperationId::Source);
const MERGE: SemanticOperationId = SemanticOperationId::Keyed(KeyedOperationId::Merge);

fn bound(scope: Vec<SemanticOperationId>) -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::probe().expect("paper binding");
    let source = KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("owner validation");
    bind_input(
        &manifest,
        source,
        scope,
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("keyed binding")
}

#[test]
fn source_merge_and_combined_scopes_use_the_shared_graph_entry() {
    for (scope, census) in [
        (vec![SOURCE], 4),
        (vec![MERGE], 5),
        (vec![SOURCE, MERGE], 9),
    ] {
        let input = bound(scope.clone());
        let analyzed = engine_records::analyze_bound_relations(&input);
        let graph = analyzed.result.expect("keyed graph");
        let expected = input
            .owner()
            .source()
            .project()
            .into_iter()
            .filter(|row| scope.contains(&row.id.operation))
            .map(|row| crate::engine_keyed::relation_key(row.id))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(graph.graph.node_count(), census);
        assert_eq!(graph.graph.edge_count(), 0);
        assert_eq!(
            graph
                .graph
                .node_weights()
                .map(|row| row.id.clone())
                .collect::<std::collections::BTreeSet<_>>(),
            expected
        );
        assert_eq!(graph.evaluation_order.len(), census);
        // Each call statically lowers the retained owner again, with no cached
        // projected source accepted as authority.
        let repeated = engine_records::analyze_bound_relations(&input);
        assert_eq!(analyzed.lowering.records(), repeated.lowering.records());
        assert!(repeated.result.is_ok());
    }
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    let analyzed = engine_records::analyze_bound_relations(&attestation);
    assert!(analyzed.result.is_ok());
    assert!(
        analyzed
            .lowering
            .records()
            .nodes
            .iter()
            .all(|node| node.id.operation.namespace == 1)
    );
}

#[test]
fn keyed_binding_refuses_an_attestation_scope_with_its_numeric_collision() {
    let manifest = KeyedManifest::probe().expect("paper binding");
    let source = KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("owner validation");
    let policy = super::bound_input(&[OperationId::CompactAsh]).policy();
    assert!(bind_input(&manifest, source.clone(), vec![SOURCE], policy).is_ok());
    let foreign = SemanticOperationId::Attestation(OperationId::CreateRequest);
    assert_eq!(SOURCE.encoding()[1], foreign.encoding()[1]);
    assert_eq!(
        bind_input(&manifest, source, vec![foreign], policy).err(),
        Some(KeyedBindingError::ForeignOperation(foreign))
    );
}

#[test]
fn keyed_declared_graph_refuses_unknown_and_cyclic_dependencies() {
    let input = bound(vec![MERGE]);
    let analyzed = engine_records::analyze_bound_relations(&input);
    assert!(analyzed.result.is_ok());
    let control = analyzed.lowering.records().clone();
    let first = control
        .nodes
        .iter()
        .find(|node| node.id.operation == crate::engine_keyed::operation_key(MERGE))
        .expect("MERGE relation")
        .id
        .clone();
    let mut unknown_id = first.clone();
    unknown_id.kind = u16::MAX;
    let kind = DependencyKind {
        namespace: 2,
        code: 1,
    };
    let mut unknown = control.clone();
    unknown.edges.push(DependencyRecord {
        source: first.clone(),
        target: unknown_id.clone(),
        kind,
    });
    assert_eq!(
        engine_records::analyze_relations(&unknown).err(),
        Some(RelationRecordError::UnknownEndpoint {
            relation: unknown_id
        })
    );
    let mut cyclic = control;
    cyclic.edges.push(DependencyRecord {
        source: first.clone(),
        target: first,
        kind,
    });
    assert!(matches!(
        engine_records::analyze_relations(&cyclic),
        Err(RelationRecordError::Cycle { .. })
    ));
}
