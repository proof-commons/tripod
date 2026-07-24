//! Public-API boundary test for the target-independent realization
//! package.
//!
//! This integration test compiles as an external consumer. It proves
//! that architecture binding, checked semantic domains, typed stable
//! keys, and explicit partial scope are public without exposing local
//! graph handles or mutable registries.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId,
};
use realization::{
    ArchitectureBinding, Count, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition,
    ObservedObject, ObservedObjectKind, ObservedObjectRef, ObservedSide, OperationObservation,
    ProofAlternativeId, ProofKind, ProtocolAmount, RealizationScope, RelationId, RelationKind,
    RelationSubject, RepresentationMode, derive,
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
        RelationKind::AllowedObjectFamilies,
        RelationSubject::ObjectFamily {
            side: realization::TransactionSide::Output,
            object: ObjectId::Ash,
        },
    );

    assert_ne!(relation, output_relation);
}

#[test]
fn downstream_code_can_evaluate_without_local_graph_handles() {
    let realization = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::CompactAsh]).unwrap(),
    )
    .unwrap();

    let report = realization
        .evaluate_operation(&compact_ash_observation())
        .unwrap();

    assert!(report.is_conformant());
}

#[test]
fn validated_realization_is_readable_but_not_externally_mutable() {
    // The negative half of this boundary — that an external consumer
    // cannot assign or clear any invariant-bearing field — is proved
    // by the compile_fail doctests on ScopedRealizationSpec. This
    // test proves the read-only surface stays sufficient.
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

    assert_eq!(
        realization.architecture().architecture_schema_version(),
        ARCHITECTURE.document.architecture_schema_version,
    );
    assert_eq!(realization.scope(), &RealizationScope::phase1_pilots());

    let operations: Vec<OperationId> = realization
        .operations()
        .map(|(operation, declaration)| {
            assert_eq!(operation, declaration.operation);
            operation
        })
        .collect();
    assert_eq!(
        operations,
        vec![OperationId::TransferLive, OperationId::CompactAsh],
    );

    assert_eq!(
        realization
            .operation(OperationId::CompactAsh)
            .unwrap()
            .operation,
        OperationId::CompactAsh,
    );

    let declassification = realization.declassification();
    assert_eq!(realization.project().declassification, *declassification);
}

#[test]
fn stable_projection_exposes_typed_graph_shape_without_local_handles() {
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();
    let projection = realization.project();

    assert_eq!(projection.scope, RealizationScope::phase1_pilots());
    assert_eq!(projection.operations.len(), 2);
    assert_eq!(
        projection.relations.nodes.len(),
        realization.relations().count()
    );
    assert_ne!(
        projection.constructibility.edges,
        [] as [realization::ConstructibilityDependencyProjection; 0]
    );
}

fn compact_ash_observation() -> OperationObservation {
    let input0 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    let input1 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let output0 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 0,
    };

    OperationObservation {
        operation: OperationId::CompactAsh,
        objects: vec![
            ash(ObservedSide::Input, 0, 40),
            ash(ObservedSide::Input, 1, 60),
            ash(ObservedSide::Output, 0, 100),
        ],
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![ObservedCanonicalFlow {
                asset: AssetId::U,
                sources: vec![input0, input1],
                destinations: vec![output0],
                movement_kind: Some(DeltaKind::OwnerlessLateral),
                destructions: Vec::new(),
            }],
        },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

fn ash(side: ObservedSide, ordinal: u32, value: u64) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::Ash),
        asset: ObservedAsset::Declared(AssetId::U),
        value: ProtocolAmount::new(value).unwrap(),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}
