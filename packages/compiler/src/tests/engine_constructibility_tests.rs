//! Bound-owner and unavailable-required-source controls for the shared traversal.
use crate::{
    engine_constructibility as core,
    engine_records::{self, OperationKey},
};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};
use std::collections::BTreeSet;

#[test]
fn both_bound_owners_reach_constructibility() {
    let attestation = super::bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let old = crate::constructibility::build_constructibility_analysis(&attestation)
        .expect("attestation control");
    let shared = engine_records::analyze_bound_constructibility(&attestation).expect("same core");
    assert_eq!(shared.graph.node_count(), old.graph.node_count());
    assert_eq!(shared.graph.edge_count(), old.graph.edge_count());
    assert_eq!(shared.operations.len(), old.operations.len());
    for scope in [
        vec![KeyedOperationId::Source],
        vec![KeyedOperationId::Merge],
        vec![KeyedOperationId::Source, KeyedOperationId::Merge],
    ] {
        let manifest = KeyedManifest::contract().expect("binding");
        let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
            .expect("declaration")
            .validate(&manifest)
            .expect("owner validation");
        let bound = crate::bind_input(
            &manifest,
            source,
            scope.into_iter().map(SemanticOperationId::Keyed).collect(),
            attestation.policy(),
        )
        .expect("binding");
        let records = bound.owner().lower_constructibility().expect("lowering");
        assert_eq!(
            records,
            bound.owner().lower_constructibility().expect("re-derive")
        );
        let result = engine_records::analyze_bound_constructibility(&bound).expect("empty stage");
        assert_eq!(result.graph.node_count(), 0);
        assert_eq!(result.graph.edge_count(), 0);
        assert_eq!(result.operations, []);
    }
}

#[test]
fn unavailable_required_source_is_refused_in_both_namespaces() {
    for profile in [
        architecture::semantic::SemanticProfileId::Attestation,
        architecture::semantic::SemanticProfileId::KeyedProvenance,
    ] {
        let namespace = crate::engine_identity::namespace(profile);
        let operation = OperationKey { namespace, code: 1 };
        let availability = core::Availability {
            namespace,
            kind: 1,
            object: 1,
            sponsor_local: false,
        };
        let root = core::NodeKey::operation(operation);
        let source = core::NodeKey {
            kind: 1,
            operation,
            subject: vec![1],
            availability: Some(availability),
            independent: false,
        };
        let authorization = core::Authorization {
            key: crate::engine_identity::AuthorizationKey::new(
                profile,
                vec![crate::engine_identity::word(0u16)],
            ),
            permissionless: true,
            allowed: BTreeSet::from([availability]),
        };
        let mut records = core::Records {
            scope: vec![operation],
            nodes: vec![
                core::Node { id: root.clone() },
                core::Node { id: source.clone() },
            ],
            edges: vec![core::Dependency {
                source: source.clone(),
                target: root.clone(),
                edge: core::Edge {
                    kind: 0,
                    required: true,
                    sponsor_only: false,
                },
            }],
            operations: vec![(operation, vec![authorization])],
        };
        assert!(core::analyze_constructibility(&records).is_ok());
        records.operations[0].1[0].allowed.clear();
        assert_eq!(
            core::analyze_constructibility(&records).err(),
            Some(core::RecordError::PermissionlessPrivateDependency {
                operation,
                node: source.clone(),
                path: vec![source, root]
            })
        );
    }
}
