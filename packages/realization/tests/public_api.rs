//! Public-API boundary test for the target-independent realization
//! package.
//!
//! This integration test compiles as an external consumer. It proves
//! that architecture binding, checked semantic domains, typed stable
//! keys, and explicit partial scope are public without exposing local
//! graph handles or mutable registries.

use std::collections::BTreeSet;

use architecture::{ARCHITECTURE, AssetId, ObjectId, OperationId};
use realization::{
    ArchitectureBinding, DependencyEdge, ExprId, ExpressionDeclaration, ExpressionNode,
    ExpressionRegistry, ExpressionRole, FactId, FactValues, OwnerId, ProofAlternativeId, ProofKind,
    RealizationScope, RelationId, RelationKind, RelationSubject, SemanticType, SemanticValue,
    TransactionSide,
};

#[test]
fn phase1_foundation_is_publicly_constructible() {
    let binding = ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap();

    assert_eq!(
        binding.architecture_schema_version(),
        ARCHITECTURE.document.architecture_schema_version,
    );

    let scope = RealizationScope::phase1_pilots();

    assert!(scope.contains(OperationId::CompactAsh));
    assert!(scope.contains(OperationId::TransferLive));
}

#[test]
fn stable_relation_keys_use_architecture_owned_ids() {
    let relation = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );

    let proof = ProofAlternativeId::new(relation.clone(), ProofKind::PublicArithmetic);

    assert_eq!(relation.operation(), OperationId::CompactAsh);
    assert_eq!(relation.kind(), RelationKind::Conservation);
    assert_eq!(proof.relation(), &relation);

    let output_relation = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: realization::TransactionSide::Output,
            object: ObjectId::Ash,
        },
    );

    assert_ne!(relation, output_relation);
}

#[test]
fn downstream_code_can_evaluate_typed_authorization_without_local_handles() {
    let (registry, owners, signers, predicate) = authorization_fixture();
    let alice = OwnerId([1_u8; 32]);
    let bob = OwnerId([2_u8; 32]);
    let mut facts = FactValues::default();

    facts
        .insert(
            owners,
            SemanticValue::OwnerSet(BTreeSet::from([alice, bob])),
        )
        .unwrap();

    facts
        .insert(
            signers,
            SemanticValue::OwnerSet(BTreeSet::from([alice, bob])),
        )
        .unwrap();

    assert!(registry.evaluate(&facts).unwrap().bool(&predicate).unwrap());
}

#[test]
fn dependency_graph_is_direct_petgraph() {
    let (registry, _, _, _) = authorization_fixture();
    let graph: &petgraph::graph::DiGraph<ExprId, DependencyEdge, u32> = registry.dependency_graph();

    assert_eq!(graph.node_count(), registry.len());
}

fn authorization_fixture() -> (ExpressionRegistry, FactId, FactId, ExprId) {
    let relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::Authorization,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    let owners = FactId::InputOwners {
        operation: OperationId::TransferLive,
        object: ObjectId::ReceiptLive,
    };
    let signers = FactId::Signers {
        operation: OperationId::TransferLive,
    };
    let predicate = ExprId::relation(relation, ExpressionRole::Predicate);

    let registry = ExpressionRegistry::new([
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
            id: predicate.clone(),
            ty: SemanticType::Bool,
            node: ExpressionNode::OwnerSubset {
                required: ExprId::fact(owners.clone()),
                presented: ExprId::fact(signers.clone()),
            },
        },
    ])
    .unwrap();

    (registry, owners, signers, predicate)
}
