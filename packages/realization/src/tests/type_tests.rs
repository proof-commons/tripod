use architecture::{AssetId, ObjectId, OperationId};

use crate::{
    Count, ExprId, ExpressionDeclaration, ExpressionNode, ExpressionRegistry, ExpressionRole,
    FactId, RealizationError, RelationId, RelationKind, RelationSubject, SemanticType,
    TransactionSide,
};

#[test]
fn amount_and_count_operands_cannot_be_compared() {
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
    let amount_fact = FactId::FamilyAmount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: ObjectId::Ash,
    };
    let predicate = ExprId::relation(relation, ExpressionRole::Predicate);

    let error = ExpressionRegistry::new([
        ExpressionDeclaration {
            id: ExprId::fact(count_fact.clone()),
            ty: SemanticType::Count,
            node: ExpressionNode::Fact(count_fact.clone()),
        },
        ExpressionDeclaration {
            id: ExprId::fact(amount_fact.clone()),
            ty: SemanticType::Amount,
            node: ExpressionNode::Fact(amount_fact.clone()),
        },
        ExpressionDeclaration {
            id: predicate,
            ty: SemanticType::Bool,
            node: ExpressionNode::Equal {
                left: ExprId::fact(count_fact),
                right: ExprId::fact(amount_fact),
            },
        },
    ])
    .unwrap_err();

    assert!(matches!(
        error,
        RealizationError::BinaryOperandTypeMismatch {
            left_type: SemanticType::Count,
            right_type: SemanticType::Amount,
            ..
        }
    ));
}

#[test]
fn owner_sets_cannot_be_ordered() {
    let relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::Authorization,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let owners = FactId::InputOwners {
        operation: OperationId::TransferLive,
        object: ObjectId::ReceiptLive,
    };
    let signers = FactId::Signers {
        operation: OperationId::TransferLive,
    };

    let error = ExpressionRegistry::new([
        ExpressionDeclaration {
            id: ExprId::fact(owners.clone()),
            ty: SemanticType::OwnerSet,
            node: ExpressionNode::Fact(owners.clone()),
        },
        ExpressionDeclaration {
            id: ExprId::fact(signers.clone()),
            ty: SemanticType::OwnerSet,
            node: ExpressionNode::Fact(signers.clone()),
        },
        ExpressionDeclaration {
            id: ExprId::relation(relation, ExpressionRole::Predicate),
            ty: SemanticType::Bool,
            node: ExpressionNode::LessOrEqual {
                left: ExprId::fact(owners),
                right: ExprId::fact(signers),
            },
        },
    ])
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::InvalidOrderedType(SemanticType::OwnerSet),
    );
}

#[test]
fn a_sum_cannot_be_declared_over_booleans() {
    let relation = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Constructibility,
        RelationSubject::Operation,
    );

    let error = ExpressionRegistry::new([ExpressionDeclaration {
        id: ExprId::relation(relation, ExpressionRole::Predicate),
        ty: SemanticType::Bool,
        node: ExpressionNode::CheckedSum {
            ty: SemanticType::Bool,
            terms: Vec::new(),
        },
    }])
    .unwrap_err();

    assert_eq!(error, RealizationError::InvalidSumType(SemanticType::Bool),);
}

#[test]
fn literal_count_remains_a_count() {
    let relation = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    );
    let expression = ExprId::relation(relation, ExpressionRole::Minimum);

    let registry = ExpressionRegistry::new([ExpressionDeclaration {
        id: expression.clone(),
        ty: SemanticType::Count,
        node: ExpressionNode::Count(Count::new(2)),
    }])
    .unwrap();

    assert_eq!(registry.get(&expression).unwrap().ty, SemanticType::Count,);
}
