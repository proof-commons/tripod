//! Relation DAG construction tests (C1-005 / P2-005).

use std::collections::BTreeSet;

use architecture::OperationId;
use proptest::prelude::*;
use realization::{
    ProofAlternativeId, ProofKind, Relation, RelationDeclaration, RelationDependencyProjection,
    RelationEdge, RelationId, RelationKind, RelationSubject,
};

use super::bound_input;
use crate::{
    CompileError,
    relation::{AnalysisNodeId, build_relation_analysis, build_relation_graph},
};

// --- synthetic fixtures ---

fn rid(operation: OperationId, kind: RelationKind) -> RelationId {
    RelationId::new(operation, kind, RelationSubject::Operation)
}

fn decl(id: RelationId) -> RelationDeclaration {
    let alternative = ProofAlternativeId::new(id.clone(), ProofKind::ManifestShape);

    RelationDeclaration {
        id,
        relation: Relation::PermissionlessAuthorization,
        proof_alternatives: BTreeSet::from([alternative]),
    }
}

fn dep(source: &RelationId, target: &RelationId) -> RelationDependencyProjection {
    RelationDependencyProjection {
        source: source.clone(),
        target: target.clone(),
        edge: RelationEdge::StaticRequirement,
    }
}

fn ash(kind: RelationKind) -> RelationId {
    rid(OperationId::CompactAsh, kind)
}

const ASH_SCOPE: &[OperationId] = &[OperationId::CompactAsh];

// --- positive coverage against the real pilots ---

#[test]
fn every_pilot_scope_builds_with_exact_censuses() {
    for scope in [
        vec![OperationId::CompactAsh],
        vec![OperationId::TransferLive],
        vec![OperationId::CompactAsh, OperationId::TransferLive],
    ] {
        let input = bound_input(&scope);
        let analysis = build_relation_analysis(&input).expect("analysis builds");
        let source = input.realization().project();

        let expected_nodes = source
            .relations
            .nodes
            .iter()
            .filter(|node| scope.contains(&node.id.operation()))
            .map(|node| node.id.clone())
            .collect::<BTreeSet<_>>();
        let expected_edges = source
            .relations
            .edges
            .iter()
            .filter(|edge| {
                scope.contains(&edge.source.operation()) && scope.contains(&edge.target.operation())
            })
            .cloned()
            .collect::<BTreeSet<_>>();

        let projection = analysis.project();
        let graph_nodes = projection
            .nodes
            .iter()
            .map(|node| node.source.id.clone())
            .collect::<BTreeSet<_>>();
        let graph_edges = projection
            .edges
            .iter()
            .map(|edge| {
                let AnalysisNodeId::SourceRelation(source_id) = &edge.source;
                let AnalysisNodeId::SourceRelation(target_id) = &edge.target;
                let crate::relation::CompilerRelationEdge::SourceDependency(kind) = edge.edge;

                RelationDependencyProjection {
                    source: source_id.clone(),
                    target: target_id.clone(),
                    edge: kind,
                }
            })
            .collect::<BTreeSet<_>>();

        assert_eq!(
            expected_nodes, graph_nodes,
            "relation census, scope {scope:?}"
        );
        assert_eq!(
            expected_edges, graph_edges,
            "dependency census, scope {scope:?}"
        );

        // Every node retains its complete source declaration with its
        // proof alternatives attached to the exact source relation.
        for node in &projection.nodes {
            let source_declaration = source
                .relations
                .nodes
                .iter()
                .find(|candidate| candidate.id == node.source.id)
                .expect("source declaration exists");

            assert_eq!(&node.source, source_declaration);
            for alternative in &node.source.proof_alternatives {
                assert_eq!(alternative.relation(), &node.source.id);
            }
        }
    }
}

#[test]
fn the_substrate_conservation_relation_is_retained() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let input = bound_input(&[operation]);
        let analysis = build_relation_analysis(&input).expect("analysis builds");
        let expected = RelationId::new(
            operation,
            RelationKind::SubstrateConservation,
            RelationSubject::Asset {
                asset: architecture::AssetId::Lbtc,
            },
        );

        assert!(
            analysis
                .project()
                .nodes
                .iter()
                .any(|node| node.source.id == expected),
            "T2 evidence relation must survive compiler census for {operation:?}",
        );
    }
}

#[test]
fn internal_order_places_every_prerequisite_before_its_dependent() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let analysis = build_relation_analysis(&input).expect("analysis builds");

    let position = |id: &AnalysisNodeId| {
        analysis
            .evaluation_order
            .iter()
            .position(|candidate| candidate == id)
            .expect("scheduled")
    };

    for edge in analysis.project().edges {
        assert!(
            position(&edge.source) < position(&edge.target),
            "prerequisite {:?} must precede {:?}",
            edge.source,
            edge.target,
        );
    }
}

// --- negative coverage with synthetic declarations ---

#[test]
fn duplicate_relation_rejects() {
    let a = ash(RelationKind::Authorization);

    let error =
        build_relation_graph(ASH_SCOPE, &[decl(a.clone()), decl(a.clone())], &[]).unwrap_err();

    assert_eq!(error, CompileError::DuplicateRelation { relation: a });
}

#[test]
fn duplicate_dependency_rejects() {
    let a = ash(RelationKind::Authorization);
    let b = ash(RelationKind::Constructibility);

    let error = build_relation_graph(
        ASH_SCOPE,
        &[decl(a.clone()), decl(b.clone())],
        &[dep(&a, &b), dep(&a, &b)],
    )
    .unwrap_err();

    assert_eq!(
        error,
        CompileError::DuplicateRelationDependency {
            prerequisite: a,
            dependent: b,
            edge: RelationEdge::StaticRequirement,
        },
    );
}

#[test]
fn unknown_endpoints_reject() {
    let a = ash(RelationKind::Authorization);
    let ghost = ash(RelationKind::RootPolicy);

    let missing_prerequisite =
        build_relation_graph(ASH_SCOPE, &[decl(a.clone())], &[dep(&ghost, &a)]).unwrap_err();
    assert_eq!(
        missing_prerequisite,
        CompileError::UnknownRelationDependencyEndpoint {
            relation: ghost.clone(),
        },
    );

    let missing_dependent =
        build_relation_graph(ASH_SCOPE, &[decl(a.clone())], &[dep(&a, &ghost)]).unwrap_err();
    assert_eq!(
        missing_dependent,
        CompileError::UnknownRelationDependencyEndpoint { relation: ghost },
    );
}

#[test]
fn cross_scope_dependencies_reject_in_both_directions() {
    let inside = ash(RelationKind::Authorization);
    let outside = rid(OperationId::TransferLive, RelationKind::Authorization);
    let nodes = [decl(inside.clone()), decl(outside.clone())];

    let outgoing = build_relation_graph(ASH_SCOPE, &nodes, &[dep(&inside, &outside)]).unwrap_err();
    assert_eq!(
        outgoing,
        CompileError::RelationDependencyEscapesScope {
            prerequisite: inside.clone(),
            dependent: outside.clone(),
        },
    );

    let incoming = build_relation_graph(ASH_SCOPE, &nodes, &[dep(&outside, &inside)]).unwrap_err();
    assert_eq!(
        incoming,
        CompileError::RelationDependencyEscapesScope {
            prerequisite: outside,
            dependent: inside,
        },
    );
}

#[test]
fn out_of_scope_pairs_are_ignored_entirely() {
    let a = rid(OperationId::TransferLive, RelationKind::Authorization);
    let b = rid(OperationId::TransferLive, RelationKind::Constructibility);

    let analysis = build_relation_graph(
        ASH_SCOPE,
        &[decl(a.clone()), decl(b.clone())],
        &[dep(&a, &b)],
    )
    .expect("fully out-of-scope declarations are simply not selected");

    assert!(analysis.project().nodes.is_empty());
    assert!(analysis.project().edges.is_empty());
}

#[test]
fn self_loop_and_cycles_reject_with_canonical_components() {
    let a = ash(RelationKind::Authorization);
    let b = ash(RelationKind::Constructibility);
    let c = ash(RelationKind::RootPolicy);
    let d = ash(RelationKind::ProjectionPolicy);

    // Self-loop.
    let error = build_relation_graph(ASH_SCOPE, &[decl(a.clone())], &[dep(&a, &a)]).unwrap_err();
    let CompileError::RelationDependencyCycle { components } = error else {
        panic!("self-loop must be a cycle error");
    };
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].members, vec![a.clone()]);

    // Two independent cyclic components; diagnostics sorted by member
    // vectors, members and internal edges sorted within components.
    let error = build_relation_graph(
        ASH_SCOPE,
        &[
            decl(a.clone()),
            decl(b.clone()),
            decl(c.clone()),
            decl(d.clone()),
        ],
        &[dep(&a, &b), dep(&b, &a), dep(&c, &d), dep(&d, &c)],
    )
    .unwrap_err();
    let CompileError::RelationDependencyCycle { components } = error else {
        panic!("two-node cycles must be a cycle error");
    };

    assert_eq!(components.len(), 2);
    for component in &components {
        assert_eq!(component.members.len(), 2);
        assert!(component.members.is_sorted());
        assert!(component.internal_edges.is_sorted());
        assert_eq!(component.internal_edges.len(), 2);
    }
    assert!(components.is_sorted());
}

#[test]
fn foreign_proof_alternative_rejects() {
    let a = ash(RelationKind::Authorization);
    let other = ash(RelationKind::RootPolicy);
    let mut declaration = decl(a.clone());
    declaration.proof_alternatives = BTreeSet::from([ProofAlternativeId::new(
        other.clone(),
        ProofKind::ManifestShape,
    )]);

    let error = build_relation_graph(ASH_SCOPE, &[declaration], &[]).unwrap_err();

    assert_eq!(
        error,
        CompileError::ForeignProofAlternative {
            relation: a,
            alternative_relation: other,
        },
    );
}

// --- permutation determinism ---

fn synthetic_source() -> (Vec<RelationDeclaration>, Vec<RelationDependencyProjection>) {
    let a = ash(RelationKind::Authorization);
    let b = ash(RelationKind::Constructibility);
    let c = ash(RelationKind::RootPolicy);
    let d = ash(RelationKind::ProjectionPolicy);
    let nodes = vec![
        decl(a.clone()),
        decl(b.clone()),
        decl(c.clone()),
        decl(d.clone()),
    ];
    let edges = vec![dep(&a, &b), dep(&a, &c), dep(&b, &d), dep(&c, &d)];

    (nodes, edges)
}

proptest! {
    #[test]
    fn insertion_permutations_preserve_projection(
        node_permutation in Just(synthetic_source().0).prop_shuffle(),
        edge_permutation in Just(synthetic_source().1).prop_shuffle(),
    ) {
        let baseline = {
            let (nodes, edges) = synthetic_source();
            build_relation_graph(ASH_SCOPE, &nodes, &edges).expect("baseline").project()
        };
        let permuted = build_relation_graph(ASH_SCOPE, &node_permutation, &edge_permutation)
            .expect("permutation")
            .project();

        prop_assert_eq!(baseline, permuted);
    }
}
