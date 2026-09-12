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
        RelationSubject::TransactionSide {
            side: realization::TransactionSide::Output,
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
        value: realization::ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}

// --- Guide 3: compiler-consumption constructibility API ---

#[test]
fn pilot_constructibility_authorizations_derive_from_the_owner() {
    let spec = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::phase1_pilots(),
    )
    .unwrap();

    assert_eq!(
        spec.constructibility_authorizations(architecture::OperationId::CompactAsh)
            .unwrap(),
        &[realization::ConstructibilityAuthorization::Permissionless],
    );

    let transfer = spec
        .constructibility_authorizations(architecture::OperationId::TransferLive)
        .unwrap();
    assert_eq!(transfer.len(), 1);
    assert!(matches!(
        &transfer[0],
        realization::ConstructibilityAuthorization::InputOwners { objects }
            if objects.contains(&architecture::ObjectId::ReceiptLive)
    ));

    // Outside the pilot scope: a typed refusal, never a silent default.
    assert!(matches!(
        spec.constructibility_authorizations(architecture::OperationId::Burn),
        Err(realization::RealizationError::OperationOutsideScope(
            architecture::OperationId::Burn
        )),
    ));
}

#[test]
fn authorization_discharge_semantics_are_owner_defined() {
    use architecture::ObjectId;
    use realization::{AvailabilityClass, ConstructibilityAuthorization as Auth};

    let permissionless = Auth::Permissionless;
    let owners = Auth::InputOwners {
        objects: std::iter::once(ObjectId::ReceiptLive).collect(),
    };

    // Public is always dischargeable; sponsor-local availability is
    // dischargeable here because confinement is a separate rule.
    for case in [&permissionless, &owners] {
        assert!(case.discharges(AvailabilityClass::Public));
        assert!(case.discharges(AvailabilityClass::SponsorLocal));
    }

    // Private classes must be named by the case.
    assert!(owners.discharges(AvailabilityClass::InputOwners {
        object: ObjectId::ReceiptLive,
    }));
    assert!(!owners.discharges(AvailabilityClass::InputOwners {
        object: ObjectId::ReceiptTimeLocked,
    }));
    assert!(!owners.discharges(AvailabilityClass::Operator));
    assert!(!permissionless.discharges(AvailabilityClass::InputOwners {
        object: ObjectId::ReceiptLive,
    }));
    assert!(!permissionless.discharges(AvailabilityClass::Operator));
}

#[test]
fn announcement_is_available_through_explicit_public_scope() {
    let realization = derive(&ARCHITECTURE, RealizationScope::from_operations([OperationId::AnnounceMaturity]).unwrap()).unwrap();
    realization.validate_against(&ARCHITECTURE).unwrap();
    assert_eq!(realization.scope().operations(), &[OperationId::AnnounceMaturity]);
    assert!(realization.operation(OperationId::AnnounceMaturity).unwrap().expressions.is_empty());
    assert_eq!(realization.constructibility_authorizations(OperationId::AnnounceMaturity).unwrap(),
        &[realization::ConstructibilityAuthorization::Operator]);
    assert_eq!(realization.declassification().required_public.len(), 9);
    assert_eq!(RealizationScope::phase1_pilots().operations(),
        &[OperationId::TransferLive, OperationId::CompactAsh]);
}
