use architecture::{ObjectId, OperationId};

use crate::{
    AvailabilityClass, ConstructibilityDependencyDeclaration, ConstructibilityEdge,
    ConstructibilityEdgeRole, ConstructibilityNode, ConstructibilityNodeId, RealizationError,
    RequirementStrength, WitnessRole, build_constructibility_graph, derive,
    validate_constructibility,
};

#[test]
fn phase1_constructibility_graph_is_direct_petgraph_and_validates() {
    let realization = derive(
        &architecture::ARCHITECTURE,
        crate::RealizationScope::phase1_pilots(),
    )
    .unwrap();
    let graph: &petgraph::graph::DiGraph<ConstructibilityNode, ConstructibilityEdge, u32> =
        &realization.constructibility_graph;

    assert!(graph.node_count() > 0);
    validate_constructibility(
        graph,
        &realization.constructibility_node_by_id,
        OperationId::CompactAsh,
        true,
    )
    .unwrap();
    validate_constructibility(
        graph,
        &realization.constructibility_node_by_id,
        OperationId::TransferLive,
        false,
    )
    .unwrap();
}

#[test]
fn permissionless_private_dependency_fails() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let owner_witness = ConstructibilityNodeId::Witness {
        operation: OperationId::CompactAsh,
        role: WitnessRole::ProtocolOwnerAuthorization,
        availability: AvailabilityClass::InputOwners {
            object: ObjectId::Ash,
        },
    };
    let (graph, nodes, _) = build_constructibility_graph(
        [
            ConstructibilityNode {
                id: operation.clone(),
            },
            ConstructibilityNode {
                id: owner_witness.clone(),
            },
        ],
        [ConstructibilityDependencyDeclaration {
            source: owner_witness.clone(),
            target: operation,
            edge: ConstructibilityEdge {
                role: ConstructibilityEdgeRole::RequiredWitness,
                strength: RequirementStrength::Required,
            },
        }],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, true),
        Err(RealizationError::PermissionlessPrivateDependency {
            operation: OperationId::CompactAsh,
            source_node: owner_witness,
        }),
    );
}

#[test]
fn sponsor_local_dependency_must_stay_on_sponsor_edge() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let sponsor = ConstructibilityNodeId::Witness {
        operation: OperationId::CompactAsh,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    let (graph, nodes, _) = build_constructibility_graph(
        [
            ConstructibilityNode {
                id: operation.clone(),
            },
            ConstructibilityNode {
                id: sponsor.clone(),
            },
        ],
        [ConstructibilityDependencyDeclaration {
            source: sponsor.clone(),
            target: operation,
            edge: ConstructibilityEdge {
                role: ConstructibilityEdgeRole::RequiredWitness,
                strength: RequirementStrength::Required,
            },
        }],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, true),
        Err(RealizationError::SponsorDependencyEscaped {
            operation: OperationId::CompactAsh,
            source_node: sponsor,
        }),
    );
}
