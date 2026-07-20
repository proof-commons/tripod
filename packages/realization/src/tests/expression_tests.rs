use architecture::{AssetId, ObjectId, OperationId, ProjectionId};
use petgraph::visit::EdgeRef;

use crate::{
    Count, ExprId, ExpressionDeclaration, ExpressionNode, ExpressionRegistry, ExpressionRole,
    FactId, FactValues, OwnerId, ProtocolAmount, RealizationError, RelationId, RelationKind,
    RelationSubject, SemanticType, SemanticValue, TransactionSide,
};

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

fn conservation_registry() -> ExpressionRegistry {
    let relation = transfer_conservation_relation();
    let input = ExprId::fact(amount_fact(TransactionSide::Input));
    let output = ExprId::fact(amount_fact(TransactionSide::Output));
    let predicate = ExprId::relation(relation, ExpressionRole::Predicate);

    ExpressionRegistry::new([
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
    ])
    .unwrap()
}

#[test]
fn exact_amount_conservation_evaluates() {
    let registry = conservation_registry();
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

    let evaluated = registry.evaluate(&facts).unwrap();
    let predicate = ExprId::relation(transfer_conservation_relation(), ExpressionRole::Predicate);

    assert!(evaluated.bool(&predicate).unwrap());
}

#[test]
fn amount_conservation_failure_is_a_false_relation_not_an_evaluator_error() {
    let registry = conservation_registry();
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

    let evaluated = registry.evaluate(&facts).unwrap();
    let predicate = ExprId::relation(transfer_conservation_relation(), ExpressionRole::Predicate);

    assert!(!evaluated.bool(&predicate).unwrap());
}

#[test]
fn expression_construction_is_insertion_order_independent() {
    let relation = transfer_conservation_relation();
    let declarations = vec![
        fact_expression(amount_fact(TransactionSide::Input)),
        fact_expression(amount_fact(TransactionSide::Output)),
        ExpressionDeclaration {
            id: ExprId::relation(relation, ExpressionRole::Predicate),
            ty: SemanticType::Bool,
            node: ExpressionNode::Equal {
                left: ExprId::fact(amount_fact(TransactionSide::Input)),
                right: ExprId::fact(amount_fact(TransactionSide::Output)),
            },
        },
    ];

    let mut reversed = declarations.clone();
    reversed.reverse();

    assert_eq!(
        ExpressionRegistry::new(declarations).unwrap(),
        ExpressionRegistry::new(reversed).unwrap(),
    );
}

#[test]
fn canonical_graph_construction_ignores_declaration_order() {
    let relation = transfer_conservation_relation();
    let declarations = vec![
        fact_expression(amount_fact(TransactionSide::Input)),
        fact_expression(amount_fact(TransactionSide::Output)),
        ExpressionDeclaration {
            id: ExprId::relation(relation, ExpressionRole::Predicate),
            ty: SemanticType::Bool,
            node: ExpressionNode::Equal {
                left: ExprId::fact(amount_fact(TransactionSide::Input)),
                right: ExprId::fact(amount_fact(TransactionSide::Output)),
            },
        },
    ];

    let mut reversed = declarations.clone();
    reversed.reverse();

    let first = ExpressionRegistry::new(declarations).unwrap();
    let second = ExpressionRegistry::new(reversed).unwrap();

    assert_eq!(graph_snapshot(&first), graph_snapshot(&second));
    assert_eq!(first.evaluation_order(), second.evaluation_order());
}

#[test]
fn petgraph_topology_orders_dependencies_before_consumers() {
    let registry = conservation_registry();
    let input = ExprId::fact(amount_fact(TransactionSide::Input));
    let output = ExprId::fact(amount_fact(TransactionSide::Output));
    let predicate = ExprId::relation(transfer_conservation_relation(), ExpressionRole::Predicate);
    let position = |id: &ExprId| {
        registry
            .evaluation_order()
            .iter()
            .position(|candidate| candidate == id)
            .unwrap()
    };

    assert!(position(&input) < position(&predicate));
    assert!(position(&output) < position(&predicate));
}

#[test]
fn unresolved_expression_dependency_is_rejected() {
    let relation = transfer_conservation_relation();
    let predicate = ExprId::relation(relation, ExpressionRole::Predicate);

    let error = ExpressionRegistry::new([ExpressionDeclaration {
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
    let first = ExprId::relation(first_relation, ExpressionRole::Predicate);
    let second = ExprId::relation(second_relation, ExpressionRole::Predicate);

    let error = ExpressionRegistry::new([
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
    let first_fact = FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side: TransactionSide::Input,
        object: ObjectId::ReceiptLive,
    };
    let second_fact = FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side: TransactionSide::Output,
        object: ObjectId::ReceiptLive,
    };
    let sum = ExprId::relation(relation, ExpressionRole::InputTotal);

    let registry = ExpressionRegistry::new([
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
        registry.evaluate(&facts),
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
    let predicate = ExprId::relation(relation, ExpressionRole::Predicate);
    let registry = ExpressionRegistry::new([
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
        !registry
            .evaluate(&missing)
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
        registry
            .evaluate(&complete)
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
    let minimum = ExprId::relation(relation.clone(), ExpressionRole::Minimum);
    let predicate = ExprId::relation(relation, ExpressionRole::Predicate);

    let registry = ExpressionRegistry::new([
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

    assert!(!registry.evaluate(&one).unwrap().bool(&predicate).unwrap());

    let mut two = FactValues::default();
    two.insert(count_fact, SemanticValue::Count(Count::new(2)))
        .unwrap();

    assert!(registry.evaluate(&two).unwrap().bool(&predicate).unwrap());
}

fn graph_snapshot(
    registry: &ExpressionRegistry,
) -> (Vec<ExprId>, Vec<(ExprId, ExprId, crate::DependencyEdge)>) {
    let graph = registry.dependency_graph();
    let nodes = graph
        .node_indices()
        .map(|node| graph[node].clone())
        .collect::<Vec<_>>();
    let mut edges = graph
        .edge_references()
        .map(|edge| {
            (
                graph[edge.source()].clone(),
                graph[edge.target()].clone(),
                *edge.weight(),
            )
        })
        .collect::<Vec<_>>();

    edges.sort();

    (nodes, edges)
}
