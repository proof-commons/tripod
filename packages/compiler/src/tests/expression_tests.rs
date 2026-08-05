//! Expression graph construction and predicate-binding tests (P2-005).

use std::collections::BTreeSet;

use architecture::{BoundId, OperationId};
use proptest::prelude::*;
use realization::{
    DependencyEdge, ExprId, ExpressionDeclaration, ExpressionDependencyProjection, ExpressionNode,
    ExpressionRole, FactId, ProofAlternativeId, ProofKind, Relation, RelationDeclaration,
    RelationId, RelationKind, RelationSubject, SemanticType,
};

use super::bound_input;
use crate::{
    CompileError,
    expression::{build_expression_analysis, build_expression_graph},
    foundation::{analyze_foundation, validate_predicate_bindings},
    relation::build_relation_graph,
};

// --- synthetic fixtures ---

const ASH_SCOPE: &[OperationId] = &[OperationId::CompactAsh];

fn ash_relation(kind: RelationKind) -> RelationId {
    RelationId::new(OperationId::CompactAsh, kind, RelationSubject::Operation)
}

fn owned(role: ExpressionRole) -> ExprId {
    ExprId::relation(ash_relation(RelationKind::Cardinality), role)
}

fn foreign(role: ExpressionRole) -> ExprId {
    ExprId::relation(
        RelationId::new(
            OperationId::TransferLive,
            RelationKind::Cardinality,
            RelationSubject::Operation,
        ),
        role,
    )
}

fn shared_fact() -> ExprId {
    ExprId::fact(FactId::BoundValue {
        bound: BoundId::AshBatchMax,
    })
}

fn count_literal(id: ExprId, value: u64) -> ExpressionDeclaration {
    ExpressionDeclaration {
        id,
        ty: SemanticType::Count,
        node: ExpressionNode::Count(realization::Count::new(value)),
    }
}

fn bound_fact_declaration() -> ExpressionDeclaration {
    ExpressionDeclaration {
        id: shared_fact(),
        ty: SemanticType::Count,
        node: ExpressionNode::Fact(FactId::BoundValue {
            bound: BoundId::AshBatchMax,
        }),
    }
}

fn sum(id: ExprId, terms: Vec<ExprId>) -> ExpressionDeclaration {
    ExpressionDeclaration {
        id,
        ty: SemanticType::Count,
        node: ExpressionNode::CheckedSum {
            ty: SemanticType::Count,
            terms,
        },
    }
}

fn edge(
    dependency: &ExprId,
    consumer: &ExprId,
    role: DependencyEdge,
) -> ExpressionDependencyProjection {
    ExpressionDependencyProjection {
        source: dependency.clone(),
        target: consumer.clone(),
        edge: role,
    }
}

fn operand(position: u32) -> DependencyEdge {
    DependencyEdge::Operand { position }
}

// --- positive coverage ---

#[test]
fn empty_pilot_expression_graph_is_valid() {
    for scope in [
        vec![OperationId::CompactAsh],
        vec![OperationId::CompactAsh, OperationId::TransferLive],
    ] {
        let input = bound_input(&scope);
        let analysis = build_expression_analysis(&input).expect("builds");

        assert!(analysis.project().nodes.is_empty());
        assert!(analysis.project().edges.is_empty());

        // The complete foundation also builds on the real pilots.
        analyze_foundation(&input).expect("foundation builds");
    }
}

#[test]
fn scoped_closure_includes_required_shared_ancestors_only() {
    let total = owned(ExpressionRole::InputTotal);

    let nodes = [
        bound_fact_declaration(),
        sum(total.clone(), vec![shared_fact()]),
        ExpressionDeclaration {
            id: ExprId::fact(FactId::BoundValue {
                bound: BoundId::TransferInputMax,
            }),
            ty: SemanticType::Count,
            node: ExpressionNode::Fact(FactId::BoundValue {
                bound: BoundId::TransferInputMax,
            }),
        },
        // A foreign operation's expression stays out entirely.
        count_literal(foreign(ExpressionRole::Minimum), 7),
    ];
    let edges = [edge(&shared_fact(), &total, operand(0))];

    let analysis = build_expression_graph(ASH_SCOPE, &nodes, &edges).expect("builds");
    let ids = analysis
        .project()
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(ids, BTreeSet::from([shared_fact(), total]));
}

// --- negative coverage ---

#[test]
fn cross_operation_dependency_rejects() {
    let consumer = owned(ExpressionRole::InputTotal);
    let dependency = foreign(ExpressionRole::Minimum);
    let nodes = [
        sum(consumer.clone(), vec![dependency.clone()]),
        count_literal(dependency.clone(), 7),
    ];

    let error = build_expression_graph(
        ASH_SCOPE,
        &nodes,
        &[edge(&dependency, &consumer, operand(0))],
    )
    .unwrap_err();

    assert_eq!(
        error,
        CompileError::ExpressionDependencyEscapesScope {
            dependency,
            consumer,
        },
    );
}

#[test]
fn unknown_dependency_endpoint_rejects() {
    let consumer = owned(ExpressionRole::InputTotal);
    let ghost = owned(ExpressionRole::Maximum);

    let error = build_expression_graph(
        ASH_SCOPE,
        &[sum(consumer.clone(), vec![ghost.clone()])],
        &[edge(&ghost, &consumer, operand(0))],
    )
    .unwrap_err();

    assert_eq!(
        error,
        CompileError::UnknownExpressionDependencyEndpoint { expression: ghost },
    );
}

#[test]
fn duplicate_expression_and_dependency_reject() {
    let id = owned(ExpressionRole::Minimum);

    let duplicate_node = build_expression_graph(
        ASH_SCOPE,
        &[count_literal(id.clone(), 1), count_literal(id.clone(), 2)],
        &[],
    )
    .unwrap_err();
    assert_eq!(
        duplicate_node,
        CompileError::DuplicateExpression {
            expression: id.clone(),
        },
    );

    let consumer = owned(ExpressionRole::InputTotal);
    let duplicate_edge = build_expression_graph(
        ASH_SCOPE,
        &[
            count_literal(id.clone(), 1),
            sum(consumer.clone(), vec![id.clone()]),
        ],
        &[
            edge(&id, &consumer, operand(0)),
            edge(&id, &consumer, operand(0)),
        ],
    )
    .unwrap_err();
    assert_eq!(
        duplicate_edge,
        CompileError::DuplicateExpressionDependency {
            dependency: id,
            consumer,
        },
    );
}

#[test]
fn expression_cycles_reject_with_canonical_components() {
    let a = owned(ExpressionRole::Minimum);
    let b = owned(ExpressionRole::Maximum);
    let nodes = [
        sum(a.clone(), vec![b.clone()]),
        sum(b.clone(), vec![a.clone()]),
    ];

    let error = build_expression_graph(
        ASH_SCOPE,
        &nodes,
        &[edge(&a, &b, operand(0)), edge(&b, &a, operand(0))],
    )
    .unwrap_err();

    let CompileError::ExpressionDependencyCycle { components } = error else {
        panic!("cycle must be a cycle error");
    };
    assert_eq!(components.len(), 1);
    assert!(components[0].members.is_sorted());
    assert_eq!(components[0].members.len(), 2);
    assert!(components[0].internal_edges.is_sorted());
}

// --- predicate binding (§14) ---

fn predicate_relation(expression: ExprId) -> RelationDeclaration {
    let id = ash_relation(RelationKind::CanonicalDeltaPolicy);

    RelationDeclaration {
        id: id.clone(),
        relation: Relation::ExpressionPredicate { expression },
        proof_alternatives: BTreeSet::from([ProofAlternativeId::new(
            id,
            ProofKind::PublicArithmetic,
        )]),
    }
}

fn boolean_declaration(id: ExprId) -> ExpressionDeclaration {
    ExpressionDeclaration {
        id,
        ty: SemanticType::Bool,
        node: ExpressionNode::Bool(true),
    }
}

#[test]
fn predicate_bindings_validate_presence_type_and_ownership() {
    let predicate = owned(ExpressionRole::Predicate);
    let relations = build_relation_graph(ASH_SCOPE, &[predicate_relation(predicate.clone())], &[])
        .expect("relations build");

    // Present, boolean, owned: valid.
    let expressions =
        build_expression_graph(ASH_SCOPE, &[boolean_declaration(predicate.clone())], &[])
            .expect("expressions build");
    validate_predicate_bindings(&relations, &expressions).expect("valid binding");

    // Absent expression.
    let empty = build_expression_graph(ASH_SCOPE, &[], &[]).expect("empty builds");
    assert_eq!(
        validate_predicate_bindings(&relations, &empty).unwrap_err(),
        CompileError::UnknownPredicateExpression {
            relation: ash_relation(RelationKind::CanonicalDeltaPolicy),
            expression: predicate.clone(),
        },
    );

    // Non-boolean expression.
    let non_boolean =
        build_expression_graph(ASH_SCOPE, &[count_literal(predicate.clone(), 1)], &[])
            .expect("count builds");
    assert_eq!(
        validate_predicate_bindings(&relations, &non_boolean).unwrap_err(),
        CompileError::NonBooleanPredicateExpression {
            relation: ash_relation(RelationKind::CanonicalDeltaPolicy),
            expression: predicate,
            actual: SemanticType::Count,
        },
    );

    // A predicate owned by another operation is foreign even if the
    // expression itself could be constructed.
    let foreign_predicate = foreign(ExpressionRole::Predicate);
    let foreign_relations = build_relation_graph(
        ASH_SCOPE,
        &[predicate_relation(foreign_predicate.clone())],
        &[],
    )
    .expect("relations build");
    let boolean_scope: &[OperationId] = &[OperationId::CompactAsh, OperationId::TransferLive];
    let foreign_expressions = build_expression_graph(
        boolean_scope,
        &[boolean_declaration(foreign_predicate.clone())],
        &[],
    )
    .expect("expressions build");
    assert_eq!(
        validate_predicate_bindings(&foreign_relations, &foreign_expressions).unwrap_err(),
        CompileError::PredicateExpressionOutsideScope {
            relation: ash_relation(RelationKind::CanonicalDeltaPolicy),
            expression: foreign_predicate,
        },
    );
}

// --- permutation determinism ---

fn synthetic_expressions() -> (
    Vec<ExpressionDeclaration>,
    Vec<ExpressionDependencyProjection>,
) {
    let a = owned(ExpressionRole::Minimum);
    let b = owned(ExpressionRole::Maximum);
    let total = owned(ExpressionRole::InputTotal);
    let nodes = vec![
        bound_fact_declaration(),
        count_literal(a.clone(), 1),
        count_literal(b.clone(), 2),
        sum(total.clone(), vec![a.clone(), b.clone(), shared_fact()]),
    ];
    let edges = vec![
        edge(&a, &total, operand(0)),
        edge(&b, &total, operand(1)),
        edge(&shared_fact(), &total, operand(2)),
    ];

    (nodes, edges)
}

proptest! {
    #[test]
    fn expression_permutations_preserve_projection(
        node_permutation in Just(synthetic_expressions().0).prop_shuffle(),
        edge_permutation in Just(synthetic_expressions().1).prop_shuffle(),
    ) {
        let baseline = {
            let (nodes, edges) = synthetic_expressions();
            build_expression_graph(ASH_SCOPE, &nodes, &edges).expect("baseline").project()
        };
        let permuted = build_expression_graph(ASH_SCOPE, &node_permutation, &edge_permutation)
            .expect("permutation")
            .project();

        prop_assert_eq!(baseline, permuted);
    }
}
