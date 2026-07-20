//! Public-API boundary test for the target-independent realization
//! package.
//!
//! This integration test compiles as an external consumer. It proves
//! that architecture binding, checked semantic domains, typed stable
//! keys, and explicit partial scope are public without exposing local
//! graph handles or mutable registries.

use architecture::{ARCHITECTURE, AssetId, ObjectId, OperationId};
use realization::{
    ArchitectureBinding, ProofAlternativeId, ProofKind, RealizationScope, RelationId, RelationKind,
    RelationSubject,
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
