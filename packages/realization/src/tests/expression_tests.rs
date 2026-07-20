use std::collections::BTreeMap;

use architecture::{AssetId, ObjectId, OperationId, ProjectionId};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{
    Count, DependencyEdge, ExprId, ExpressionDeclaration, ExpressionNode, FactId, FactValues,
    OwnerId, ProtocolAmount, RealizationError, RelationId, RelationKind, RelationSubject,
    SemanticType, SemanticValue, TransactionSide, build_expression_graph, evaluate_expressions,
};

type ExpressionGraphParts = (
    DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
    BTreeMap<ExprId, NodeIndex<u32>>,
    Vec<ExprId>,
);

fn transfer_conservation_relation() -> RelationId {
    RelationId::new(
        OperationId::TransferLive,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

fn amount_fact(side: TransactionSide) -> FactId {
    FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side,
        object: ObjectId::ReceiptLive,
    }
}

fn fact_expression(fact: FactId) -> ExpressionDeclaration {
    ExpressionDeclaration {
        id: ExprId::fact(fact.clone()),
        ty: fact.semantic_type(),
        node: ExpressionNode::Fact(fact),
    }
}

fn conservation_declarations() -> Vec<ExpressionDeclaration> {
    let relation = transfer_conservation_relation();
    let input = ExprId::fact(amount_fact(TransactionSide::Input));
    let output = ExprId::fact(amount_fact(TransactionSide::Output));
    let predicate = ExprId::relation(relation, crate::ExpressionRole::Predicate);

    vec![
        fact_expression(amount_fact(TransactionSide::Output)),
        ExpressionDeclaration {
            id: predicate,
            ty: SemanticType::Bool,
            node: ExpressionNode::Equal {
                left: input,
                right: output,
            },
        },
        fact_expression(amount_fact(TransactionSide::Input)),
    ]
}

fn conservation_graph() -> ExpressionGraphParts {
    build_expression_graph(conservation_declarations()).unwrap()
}

#[test]
fn exact_amount_conservation_evaluates() {
    let (graph, nodes, order) = conservation_graph();
    let mut facts = FactValues::default();

    facts
        .insert(
            amount_fact(TransactionSide::Input),
            SemanticValue::Amount(ProtocolAmount::new(100).unwrap()),
        )
        .unwrap();
    facts
        .insert(
            amount_fact(TransactionSide::Output),
            SemanticValue::Amount(ProtocolAmount::new(100).unwrap()),
        )
        .unwrap();

    let evaluated = evaluate_expressions(&graph, &nodes, &order, &facts).unwrap();
    let predicate = ExprId::relation(
        transfer_conservation_relation(),
        crate::ExpressionRole::Predicate,
    );

    assert!(evaluated.bool(&predicate).unwrap());
}

#[test]
fn amount_conservation_failure_is_a_false_relation_not_an_evaluator_error() {
    let (graph, nodes, order) = conservation_graph();
    let mut facts = FactValues::default();

    facts
        .insert(
            amount_fact(TransactionSide::Input),
            SemanticValue::Amount(ProtocolAmount::new(100).unwrap()),
        )
        .unwrap();
    facts
        .insert(
            amount_fact(TransactionSide::Output),
            SemanticValue::Amount(ProtocolAmount::new(99).unwrap()),
        )
        .unwrap();

    let evaluated = evaluate_expressions(&graph, &nodes, &order, &facts).unwrap();
    let predicate = ExprId::relation(
        transfer_conservation_relation(),
        crate::ExpressionRole::Predicate,
    );

    assert!(!evaluated.bool(&predicate).unwrap());
}

#[test]
fn graph_construction_is_insertion_order_independent() {
    let declarations = conservation_declarations();
    let mut reversed = declarations.clone();
    reversed.reverse();

    let first = build_expression_graph(declarations).unwrap();
    let second = build_expression_graph(reversed).unwrap();

    assert_eq!(graph_snapshot(&first.0), graph_snapshot(&second.0));
    assert_eq!(first.2, second.2);
}

#[test]
fn petgraph_topology_orders_dependencies_before_consumers() {
    let (_, _, order) = conservation_graph();
    let input = ExprId::fact(amount_fact(TransactionSide::Input));
    let output = ExprId::fact(amount_fact(TransactionSide::Output));
    let predicate = ExprId::relation(
        transfer_conservation_relation(),
        crate::ExpressionRole::Predicate,
    );
    let position = |id: &ExprId| order.iter().position(|candidate| candidate == id).unwrap();

    assert!(position(&input) < position(&predicate));
    assert!(position(&output) < position(&predicate));
}

#[test]
fn unresolved_expression_dependency_is_rejected() {
    let relation = transfer_conservation_relation();
    let predicate = ExprId::relation(relation, crate::ExpressionRole::Predicate);

    let error = build_expression_graph([ExpressionDeclaration {
        id: predicate.clone(),
        ty: SemanticType::Bool,
        node: ExpressionNode::Equal {
            left: ExprId::fact(amount_fact(TransactionSide::Input)),
            right: ExprId::fact(amount_fact(TransactionSide::Output)),
        },
    }])
    .unwrap_err();

    assert!(matches!(
        error,
        RealizationError::UnknownExpressionDependency {
            expression,
            ..
        } if expression == predicate
    ));
}

#[test]
fn dependency_cycle_is_rejected_in_stable_order() {
    let first_relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::Constructibility,
        RelationSubject::Operation,
    );
    let second_relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::ProjectionPolicy,
        RelationSubject::Projection {
            projection: ProjectionId::TransitionCertificate,
        },
    );
    let first = ExprId::relation(first_relation, crate::ExpressionRole::Predicate);
    let second = ExprId::relation(second_relation, crate::ExpressionRole::Predicate);

    let error = build_expression_graph([
        ExpressionDeclaration {
            id: first.clone(),
            ty: SemanticType::Bool,
            node: ExpressionNode::All {
                terms: vec![second.clone()],
            },
        },
        ExpressionDeclaration {
            id: second.clone(),
            ty: SemanticType::Bool,
            node: ExpressionNode::All {
                terms: vec![first.clone()],
            },
        },
    ])
    .unwrap_err();

    let RealizationError::ExpressionDependencyCycle { components } = error else {
        panic!("expected a dependency-cycle failure");
    };
    let mut expected = vec![first, second];
    expected.sort();

    assert_eq!(components, vec![expected]);
}

#[test]
fn checked_amount_sum_reestablishes_the_amount_domain() {
    let relation = transfer_conservation_relation();
    let first_fact = amount_fact(TransactionSide::Input);
    let second_fact = amount_fact(TransactionSide::Output);
    let sum = ExprId::relation(relation, crate::ExpressionRole::InputTotal);
    let (graph, nodes, order) = build_expression_graph([
        fact_expression(first_fact.clone()),
        fact_expression(second_fact.clone()),
        ExpressionDeclaration {
            id: sum,
            ty: SemanticType::Amount,
            node: ExpressionNode::CheckedSum {
                ty: SemanticType::Amount,
                terms: vec![
                    ExprId::fact(first_fact.clone()),
                    ExprId::fact(second_fact.clone()),
                ],
            },
        },
    ])
    .unwrap();
    let maximum = ProtocolAmount::new((1_u64 << 51) - 1).unwrap();
    let mut facts = FactValues::default();

    facts
        .insert(first_fact, SemanticValue::Amount(maximum))
        .unwrap();
    facts
        .insert(second_fact, SemanticValue::Amount(ProtocolAmount::ONE))
        .unwrap();

    assert_eq!(
        evaluate_expressions(&graph, &nodes, &order, &facts),
        Err(RealizationError::AmountOutOfDomain { value: 1_u64 << 51 }),
    );
}

#[test]
fn owner_authorization_is_set_inclusion() {
    let relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::Authorization,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    let required_fact = FactId::InputOwners {
        operation: OperationId::TransferLive,
        object: ObjectId::ReceiptLive,
    };
    let presented_fact = FactId::Signers {
        operation: OperationId::TransferLive,
    };
    let predicate = ExprId::relation(relation, crate::ExpressionRole::Predicate);
    let (graph, nodes, order) = build_expression_graph([
        fact_expression(required_fact.clone()),
        fact_expression(presented_fact.clone()),
        ExpressionDeclaration {
            id: predicate.clone(),
            ty: SemanticType::Bool,
            node: ExpressionNode::OwnerSubset {
                required: ExprId::fact(required_fact.clone()),
                presented: ExprId::fact(presented_fact.clone()),
            },
        },
    ])
    .unwrap();
    let alice = OwnerId([1_u8; 32]);
    let bob = OwnerId([2_u8; 32]);
    let mut missing = FactValues::default();

    missing
        .insert(
            required_fact.clone(),
            SemanticValue::OwnerSet([alice, bob].into_iter().collect()),
        )
        .unwrap();
    missing
        .insert(
            presented_fact.clone(),
            SemanticValue::OwnerSet(std::iter::once(alice).collect()),
        )
        .unwrap();

    assert!(
        !evaluate_expressions(&graph, &nodes, &order, &missing)
            .unwrap()
            .bool(&predicate)
            .unwrap()
    );

    let mut complete = FactValues::default();
    complete
        .insert(
            required_fact,
            SemanticValue::OwnerSet([alice, bob].into_iter().collect()),
        )
        .unwrap();
    complete
        .insert(
            presented_fact,
            SemanticValue::OwnerSet([alice, bob].into_iter().collect()),
        )
        .unwrap();

    assert!(
        evaluate_expressions(&graph, &nodes, &order, &complete)
            .unwrap()
            .bool(&predicate)
            .unwrap()
    );
}

#[test]
fn count_comparison_is_distinct_from_amount_comparison() {
    let relation = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    );
    let count_fact = FactId::FamilyCount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: ObjectId::Ash,
    };
    let observed = ExprId::fact(count_fact.clone());
    let minimum = ExprId::relation(relation.clone(), crate::ExpressionRole::Minimum);
    let predicate = ExprId::relation(relation, crate::ExpressionRole::Predicate);
    let (graph, nodes, order) = build_expression_graph([
        fact_expression(count_fact.clone()),
        ExpressionDeclaration {
            id: minimum.clone(),
            ty: SemanticType::Count,
            node: ExpressionNode::Count(Count::new(2)),
        },
        ExpressionDeclaration {
            id: predicate.clone(),
            ty: SemanticType::Bool,
            node: ExpressionNode::LessOrEqual {
                left: minimum,
                right: observed,
            },
        },
    ])
    .unwrap();
    let mut one = FactValues::default();
    one.insert(count_fact.clone(), SemanticValue::Count(Count::ONE))
        .unwrap();

    assert!(
        !evaluate_expressions(&graph, &nodes, &order, &one)
            .unwrap()
            .bool(&predicate)
            .unwrap()
    );

    let mut two = FactValues::default();
    two.insert(count_fact, SemanticValue::Count(Count::new(2)))
        .unwrap();

    assert!(
        evaluate_expressions(&graph, &nodes, &order, &two)
            .unwrap()
            .bool(&predicate)
            .unwrap()
    );
}

fn graph_snapshot(
    graph: &DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
) -> (Vec<ExprId>, Vec<(ExprId, ExprId, DependencyEdge)>) {
    let nodes = graph
        .node_indices()
        .map(|node| graph[node].id.clone())
        .collect::<Vec<_>>();
    let mut edges = graph
        .edge_references()
        .map(|edge| {
            (
                graph[edge.source()].id.clone(),
                graph[edge.target()].id.clone(),
                *edge.weight(),
            )
        })
        .collect::<Vec<_>>();

    edges.sort();

    (nodes, edges)
}
