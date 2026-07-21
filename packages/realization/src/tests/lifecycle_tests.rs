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

#[test]
fn lifecycle_cycles_are_reported_by_stable_components() {
    let first = LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode: RepresentationMode::Explicit,
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
        [
            LifecycleDependencyDeclaration {
                source: first.clone(),
                target: second.clone(),
                edge: LifecycleEdge::RequiresExit,
            },
            LifecycleDependencyDeclaration {
                source: second.clone(),
                target: first.clone(),
                edge: LifecycleEdge::RequiresExit,
            },
        ],
    )
    .unwrap_err();
    let mut expected = vec![first, second];
    expected.sort();

    assert_eq!(
        error,
        RealizationError::LifecycleCycle {
            components: vec![expected],
        },
    );
}
