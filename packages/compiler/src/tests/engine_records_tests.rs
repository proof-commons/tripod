//! Neutral graph controls, independent of either contract's relation vocabulary.

use crate::engine_records::{
    DependencyKind, DependencyRecord, OperationKey, RelationKey, RelationRecord,
    RelationRecordError, RelationRecords, analyze_relations,
};

fn key(namespace: u16, kind: u16) -> RelationKey {
    RelationKey {
        operation: OperationKey { namespace, code: 1 },
        kind,
        subject: vec![0],
    }
}

fn node(id: RelationKey) -> RelationRecord {
    RelationRecord {
        proof_owners: vec![id.clone()],
        id,
    }
}

fn edge(source: RelationKey, target: RelationKey) -> DependencyRecord {
    DependencyRecord {
        kind: DependencyKind {
            namespace: source.operation.namespace,
            code: 1,
        },
        source,
        target,
    }
}

fn control(namespace: u16) -> RelationRecords {
    let first = key(namespace, 1);
    let second = key(namespace, 2);
    RelationRecords {
        scope: vec![first.operation],
        nodes: vec![node(first.clone()), node(second.clone())],
        edges: vec![edge(first, second)],
    }
}

#[test]
fn neutral_graph_checks_unknown_endpoints_and_cycles_in_each_namespace() {
    for namespace in [1, 2] {
        let accepted = control(namespace);
        assert!(analyze_relations(&accepted).is_ok());
        let mut unknown = accepted.clone();
        unknown.edges[0].target = key(namespace, 3);
        assert_eq!(
            analyze_relations(&unknown).err(),
            Some(RelationRecordError::UnknownEndpoint {
                relation: key(namespace, 3)
            })
        );

        let mut cyclic = accepted;
        cyclic
            .edges
            .push(edge(key(namespace, 2), key(namespace, 1)));
        let Some(RelationRecordError::Cycle { components }) = analyze_relations(&cyclic).err()
        else {
            panic!("cycle must be refused");
        };
        assert_eq!(components.len(), 1);
        assert_eq!(
            components[0].members,
            vec![key(namespace, 1), key(namespace, 2)]
        );
        assert_eq!(components[0].internal_edges.len(), 2);
    }
}

#[test]
fn neutral_graph_preserves_duplicate_and_ownership_refusals() {
    for namespace in [1, 2] {
        let accepted = control(namespace);
        assert!(analyze_relations(&accepted).is_ok());
        let mut duplicate = accepted.clone();
        duplicate.nodes.push(duplicate.nodes[0].clone());
        assert_eq!(
            analyze_relations(&duplicate).err(),
            Some(RelationRecordError::DuplicateRelation {
                relation: key(namespace, 1)
            })
        );
        let mut duplicate_edge = accepted.clone();
        duplicate_edge.edges.push(duplicate_edge.edges[0].clone());
        assert_eq!(
            analyze_relations(&duplicate_edge).err(),
            Some(RelationRecordError::DuplicateDependency {
                prerequisite: key(namespace, 1),
                dependent: key(namespace, 2),
                kind: accepted.edges[0].kind
            })
        );
        let mut foreign = accepted;
        foreign.nodes[0].proof_owners[0] = key(namespace, 2);
        assert_eq!(
            analyze_relations(&foreign).err(),
            Some(RelationRecordError::ForeignProofOwner {
                relation: key(namespace, 1),
                alternative_relation: key(namespace, 2)
            })
        );
    }
}

#[test]
fn namespace_is_part_of_identity_and_scope_in_both_directions() {
    for (inside, outside) in [(1, 2), (2, 1)] {
        let mut accepted = control(inside);
        accepted.nodes.extend(control(outside).nodes);
        let graph = analyze_relations(&accepted).expect("outside declarations are not selected");
        assert_eq!(graph.graph.node_count(), 2);
        assert!(
            graph
                .graph
                .node_weights()
                .all(|node| node.id.operation.namespace == inside)
        );
        for (source, target) in [
            (key(inside, 1), key(outside, 2)),
            (key(outside, 1), key(inside, 2)),
        ] {
            let mut escaped = accepted.clone();
            escaped.edges.push(edge(source.clone(), target.clone()));
            assert_eq!(
                analyze_relations(&escaped).err(),
                Some(RelationRecordError::EscapesScope {
                    prerequisite: source,
                    dependent: target
                })
            );
        }
        let mut foreign_owner = accepted.clone();
        foreign_owner.nodes[0].proof_owners[0] = key(outside, 1);
        assert_eq!(
            analyze_relations(&foreign_owner).err(),
            Some(RelationRecordError::ForeignProofOwner {
                relation: key(inside, 1),
                alternative_relation: key(outside, 1)
            })
        );
    }
}

#[test]
fn neutral_graph_retains_both_namespaces_when_both_are_explicitly_scoped() {
    let mut accepted = control(1);
    let second = control(2);
    accepted.scope.extend(second.scope);
    accepted.nodes.extend(second.nodes);
    accepted.edges.extend(second.edges);
    let graph = analyze_relations(&accepted).expect("disjoint complete scopes");
    assert_eq!(graph.graph.node_count(), 4);
    assert_eq!(graph.graph.edge_count(), 2);
    assert_eq!(graph.node_by_id.len(), 4);
}
