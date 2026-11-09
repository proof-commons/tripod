//! Both namespaces use the same typed dependency resolver and SCC policy.
use crate::{
    coverage_graph::{CoverageEdge as E, CoverageNodeClass as C},
    engine_coverage_graph as core,
    engine_placement::{CaseKey, Key},
    engine_records::{OperationKey, RelationKey},
};

fn key(namespace: u16, code: u16) -> Key {
    let operation = OperationKey { namespace, code: 0 };
    Key {
        relation: RelationKey {
            operation,
            kind: code,
            subject: Vec::new(),
        },
        case: CaseKey::Keyed(operation),
    }
}
fn node(key: Key) -> core::Node {
    core::Node {
        id: core::Key::Relation(key.clone()),
        operation: key.relation.operation,
        class: C::RelationCase,
        relation_case: Some(key),
        active: Some(true),
    }
}
#[test]
fn shared_graph_rejects_unknown_duplicate_cyclic_and_cross_namespace_dependencies() {
    for namespace in [1, 2] {
        let first = node(key(namespace, 0));
        let second = node(key(namespace, 1));
        let edge = core::Dependency {
            source: first.id.clone(),
            target: second.id.clone(),
            edge: E::RelationPrerequisite,
        };
        let nodes = vec![first.clone(), second.clone()];
        let control = core::build(&nodes, std::slice::from_ref(&edge)).expect("accepted graph");
        assert!(control.validate_acyclic().is_ok());
        assert_eq!(
            control.dependency_collateral(&key(namespace, 0)),
            std::collections::BTreeSet::from([key(namespace, 1)])
        );
        let unknown = core::Dependency {
            target: core::Key::Relation(key(namespace, 99)),
            ..edge.clone()
        };
        assert_eq!(
            core::build(&nodes, std::slice::from_ref(&unknown)).err(),
            Some(core::Error::UnknownSymbol(Box::new(unknown.target)))
        );
        assert_eq!(
            core::build(&nodes, &[edge.clone(), edge.clone()]).err(),
            Some(core::Error::DuplicateDependency(Box::new(edge.clone())))
        );
        let reverse = core::Dependency {
            source: second.id.clone(),
            target: first.id.clone(),
            edge: E::RelationPrerequisite,
        };
        let cyclic = core::build(&nodes, &[edge.clone(), reverse]).expect("resolved cycle");
        assert!(matches!(
            cyclic.validate_acyclic(),
            Err(core::Error::Cycle(_))
        ));
        let foreign = node(key(3 - namespace, 1));
        let cross = core::Dependency {
            target: foreign.id.clone(),
            ..edge
        };
        assert_eq!(
            core::build(&[first, foreign], std::slice::from_ref(&cross)).err(),
            Some(core::Error::CrossOperation(Box::new(cross)))
        );
        assert!(core::check_definition(Some(&second), &second, false).is_ok());
        assert_eq!(
            core::check_definition(Some(&second), &second, true),
            Err(core::Error::DuplicateSymbol(Box::new(second.id)))
        );
    }
}
#[test]
fn source_evidence_graph_has_no_runtime_carrier_and_rejects_reversed_evidence_edges() {
    use architecture::{
        OperationId,
        keyed::KeyedManifest,
        semantic::{KeyedOperationId, SemanticOperationId},
    };
    let manifest = KeyedManifest::probe().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let input = crate::bind_input(
        &manifest,
        source,
        vec![SemanticOperationId::Keyed(KeyedOperationId::Source)],
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("binding");
    let coverage = crate::engine_keyed::lower_coverage(input.owner());
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for plan in coverage {
        let relation = node(plan.key);
        for requirement in plan.positive.into_iter().chain(plan.negative) {
            nodes.push(core::Node {
                id: core::Key::Requirement(requirement),
                operation: relation.operation,
                class: C::Requirement,
                relation_case: None,
                active: None,
            });
        }
        for evidence in plan.evidence {
            let evidence = core::Node {
                id: core::Key::Evidence(evidence),
                operation: relation.operation,
                class: C::ExternalEvidence,
                relation_case: None,
                active: None,
            };
            edges.push(core::Dependency {
                source: evidence.id.clone(),
                target: relation.id.clone(),
                edge: E::EvidenceEnablesRelation,
            });
            nodes.push(evidence);
        }
        nodes.push(relation);
    }
    let control = core::build(&nodes, &edges).expect("evidence graph");
    assert!(control.validate_acyclic().is_ok());
    let projection = control.project();
    assert_eq!(projection.nodes.len(), 24);
    assert_eq!(projection.edges.len(), 4);
    assert_eq!(
        projection
            .nodes
            .iter()
            .filter(|node| node.class == C::Carrier)
            .count(),
        0
    );
    let mut reversed = edges;
    let row = &mut reversed[0];
    std::mem::swap(&mut row.source, &mut row.target);
    assert_eq!(
        core::build(&nodes, &reversed).err(),
        Some(core::Error::EndpointClass(Box::new(reversed[0].clone())))
    );
}
