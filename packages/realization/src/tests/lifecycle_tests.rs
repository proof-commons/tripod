use architecture::{ObjectId, OperationId};

use crate::{
    LifecycleDependencyDeclaration, LifecycleEdge, LifecycleNode, LifecycleNodeId,
    RealizationError, RepresentationMode, build_lifecycle_graph, derive, require_lifecycle_exit,
};

#[test]
fn phase1_lifecycle_paths_exist_for_supported_representations() {
    let realization = derive(
        &architecture::ARCHITECTURE,
        crate::RealizationScope::phase1_pilots(),
    )
    .unwrap();

    for mode in [
        RepresentationMode::Explicit,
        RepresentationMode::PublicCommitted,
    ] {
        for exit in [OperationId::CompactAsh, OperationId::Clear] {
            require_lifecycle_exit(
                &realization.lifecycle_graph,
                &realization.lifecycle_node_by_id,
                ObjectId::Ash,
                mode,
                exit,
            )
            .unwrap();
        }
    }

    for mode in [
        RepresentationMode::Explicit,
        RepresentationMode::PrivateCommitted,
    ] {
        for exit in [
            OperationId::TransferLive,
            OperationId::Burn,
            OperationId::Redeem,
        ] {
            require_lifecycle_exit(
                &realization.lifecycle_graph,
                &realization.lifecycle_node_by_id,
                ObjectId::ReceiptLive,
                mode,
                exit,
            )
            .unwrap();
        }
    }
}

#[test]
fn missing_lifecycle_path_fails_with_stable_ids() {
    let source = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::Explicit,
    };
    let target = LifecycleNodeId::RequiredExit {
        object: ObjectId::Ash,
        operation: OperationId::Clear,
    };
    let (graph, nodes, _) = build_lifecycle_graph(
        [LifecycleNode { id: source }, LifecycleNode { id: target }],
        Vec::<LifecycleDependencyDeclaration>::new(),
    )
    .unwrap();

    assert_eq!(
        require_lifecycle_exit(
            &graph,
            &nodes,
            ObjectId::Ash,
            RepresentationMode::Explicit,
            OperationId::Clear,
        ),
        Err(RealizationError::MissingLifecyclePath {
            object: ObjectId::Ash,
            mode: RepresentationMode::Explicit,
            exit: OperationId::Clear,
        }),
    );
}

#[test]
fn duplicate_lifecycle_dependency_is_rejected() {
    let source = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::Explicit,
    };
    let target = LifecycleNodeId::RequiredExit {
        object: ObjectId::Ash,
        operation: OperationId::Clear,
    };
    let dependency = LifecycleDependencyDeclaration {
        source: source.clone(),
        target: target.clone(),
        edge: LifecycleEdge::RequiresExit,
    };

    let error = build_lifecycle_graph(
        [LifecycleNode { id: source }, LifecycleNode { id: target }],
        [dependency.clone(), dependency.clone()],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::DuplicateLifecycleDependency(dependency)
    );
}

// Lifecycle edge semantic-shape validation (F4-002). A `RequiresExit`
// edge must run representation -> required-exit over one object; a
// reversed, representation-to-representation, exit-to-exit, or
// cross-object edge is rejected before it can be satisfied through
// generic reachability. With the shape enforced the graph is bipartite
// and cannot cycle, so no valid-shaped cycle case exists to exercise the
// retained cycle guard.

#[test]
fn reversed_lifecycle_edge_is_rejected() {
    let representation = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::Explicit,
    };
    let exit = LifecycleNodeId::RequiredExit {
        object: ObjectId::Ash,
        operation: OperationId::Clear,
    };
    let error = build_lifecycle_graph(
        [
            LifecycleNode {
                id: representation.clone(),
            },
            LifecycleNode { id: exit.clone() },
        ],
        [LifecycleDependencyDeclaration {
            source: exit.clone(),
            target: representation.clone(),
            edge: LifecycleEdge::RequiresExit,
        }],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::MalformedLifecycleEdge {
            source_node: exit,
            target_node: representation,
        },
    );
}

#[test]
fn representation_to_representation_lifecycle_edge_is_rejected() {
    let first = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::Explicit,
    };
    let second = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::PublicCommitted,
    };
    let error = build_lifecycle_graph(
        [
            LifecycleNode { id: first.clone() },
            LifecycleNode { id: second.clone() },
        ],
        [LifecycleDependencyDeclaration {
            source: first.clone(),
            target: second.clone(),
            edge: LifecycleEdge::RequiresExit,
        }],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::MalformedLifecycleEdge {
            source_node: first,
            target_node: second,
        },
    );
}

#[test]
fn exit_to_exit_lifecycle_edge_is_rejected() {
    let first = LifecycleNodeId::RequiredExit {
        object: ObjectId::Ash,
        operation: OperationId::CompactAsh,
    };
    let second = LifecycleNodeId::RequiredExit {
        object: ObjectId::Ash,
        operation: OperationId::Clear,
    };
    let error = build_lifecycle_graph(
        [
            LifecycleNode { id: first.clone() },
            LifecycleNode { id: second.clone() },
        ],
        [LifecycleDependencyDeclaration {
            source: first.clone(),
            target: second.clone(),
            edge: LifecycleEdge::RequiresExit,
        }],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::MalformedLifecycleEdge {
            source_node: first,
            target_node: second,
        },
    );
}

#[test]
fn cross_object_lifecycle_edge_is_rejected() {
    let source = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::Explicit,
    };
    let target = LifecycleNodeId::RequiredExit {
        object: ObjectId::ReceiptLive,
        operation: OperationId::Redeem,
    };
    let error = build_lifecycle_graph(
        [
            LifecycleNode { id: source.clone() },
            LifecycleNode { id: target.clone() },
        ],
        [LifecycleDependencyDeclaration {
            source: source.clone(),
            target: target.clone(),
            edge: LifecycleEdge::RequiresExit,
        }],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::CrossObjectLifecycleEdge {
            source_node: source,
            target_node: target,
        },
    );
}

#[test]
fn well_shaped_lifecycle_edge_is_accepted() {
    let source = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::Explicit,
    };
    let target = LifecycleNodeId::RequiredExit {
        object: ObjectId::Ash,
        operation: OperationId::Clear,
    };
    let (graph, nodes, _) = build_lifecycle_graph(
        [
            LifecycleNode { id: source.clone() },
            LifecycleNode { id: target.clone() },
        ],
        [LifecycleDependencyDeclaration {
            source,
            target,
            edge: LifecycleEdge::RequiresExit,
        }],
    )
    .unwrap();

    require_lifecycle_exit(
        &graph,
        &nodes,
        ObjectId::Ash,
        RepresentationMode::Explicit,
        OperationId::Clear,
    )
    .unwrap();
}
