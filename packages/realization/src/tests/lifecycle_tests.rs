use architecture::{ARCHITECTURE, ObjectId, OperationId};

use crate::{
    LifecycleDependencyDeclaration, LifecycleEdge, LifecycleNode, LifecycleNodeId,
    RealizationError, RealizationScope, Relation, RepresentationMode, ScopedRealizationSpec,
    build_lifecycle_graph, derive, require_lifecycle_exit,
};

// --- Lifecycle relation/graph weld (SR2-03) ----------------------
//
// The lifecycle is stated twice — as `Representation` and
// `LifecycleExit` relations, and as a graph of representation and
// required-exit nodes — so each mutation below breaks the agreement in
// one direction and must be caught by a focused typed error rather
// than surviving because the other statement still looks well formed.

/// The compact-ASH pilot alone, derived and therefore valid.
fn compact_pilot() -> ScopedRealizationSpec {
    derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::CompactAsh]).unwrap(),
    )
    .expect("the compact-ASH pilot derives")
}

fn validated(realization: &ScopedRealizationSpec) -> Result<(), RealizationError> {
    crate::validate::validate_scoped_realization(&ARCHITECTURE, realization)
}

fn ash_representation(mode: RepresentationMode) -> LifecycleNodeId {
    LifecycleNodeId::Representation {
        object: ObjectId::Ash,
        mode,
    }
}

fn ash_exit(operation: OperationId) -> LifecycleNodeId {
    LifecycleNodeId::RequiredExit {
        object: ObjectId::Ash,
        operation,
    }
}

/// Add a node to the lifecycle graph and its index, as a tampered
/// declaration set would have produced.
fn add_lifecycle_node(realization: &mut ScopedRealizationSpec, id: LifecycleNodeId) {
    let index = realization
        .lifecycle_graph
        .add_node(LifecycleNode { id: id.clone() });
    realization.lifecycle_node_by_id.insert(id, index);
}

#[test]
fn the_derived_pilots_satisfy_the_lifecycle_weld() {
    // The positive direction, and the reason no separate "no extra
    // edges" check is needed: the graph carries exactly the complete
    // per-object product of allowed representations and declared
    // exits.
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

    validated(&realization).unwrap();

    assert_eq!(
        realization.lifecycle_graph.edge_count(),
        2 * 2 + 2 * 3,
        "ASH has two representations and two exits; a live receipt two and three",
    );
}

#[test]
fn a_dropped_lifecycle_relation_cannot_hide_behind_its_graph_path() {
    // The finding's own scenario: drop the `CLEAR` lifecycle relation
    // and its dependency, leave the graph intact. Every earlier check
    // still passes, and the compiler's relation census would simply
    // never see the obligation.
    let mut realization = compact_pilot();
    let declaration = realization
        .operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap();

    let dropped = declaration
        .relations
        .iter()
        .find(|relation| {
            matches!(
                relation.relation,
                Relation::LifecycleExit {
                    exit: OperationId::Clear,
                    ..
                }
            )
        })
        .expect("the pilot declares a CLEAR lifecycle exit")
        .id
        .clone();

    declaration
        .relations
        .retain(|relation| relation.id != dropped);
    declaration
        .relation_dependencies
        .retain(|dependency| dependency.prerequisite != dropped && dependency.dependent != dropped);

    assert_eq!(
        validated(&realization),
        Err(RealizationError::UndeclaredLifecycleNode(ash_exit(
            OperationId::Clear
        ))),
    );
}

#[test]
fn an_extra_graph_exit_is_rejected() {
    let mut realization = compact_pilot();

    add_lifecycle_node(&mut realization, ash_exit(OperationId::Burn));

    assert_eq!(
        validated(&realization),
        Err(RealizationError::UndeclaredLifecycleNode(ash_exit(
            OperationId::Burn
        ))),
    );
}

#[test]
fn an_extra_graph_representation_is_rejected() {
    let mut realization = compact_pilot();

    add_lifecycle_node(
        &mut realization,
        ash_representation(RepresentationMode::PrivateCommitted),
    );

    assert_eq!(
        validated(&realization),
        Err(RealizationError::UndeclaredLifecycleNode(
            ash_representation(RepresentationMode::PrivateCommitted)
        )),
    );
}

#[test]
fn a_representation_declared_only_by_relation_is_rejected() {
    let mut realization = compact_pilot();

    for relation in &mut realization
        .operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .relations
    {
        if let Relation::Representation { allowed, .. } = &mut relation.relation {
            allowed.insert(RepresentationMode::PrivateCommitted);
        }
    }

    assert_eq!(
        validated(&realization),
        Err(RealizationError::MissingLifecycleRepresentation {
            object: ObjectId::Ash,
            mode: RepresentationMode::PrivateCommitted,
        }),
    );
}

#[test]
fn an_allowed_representation_reaching_an_undeclared_exit_is_rejected() {
    // The edge is the mutation, but the exit it lands on is what no
    // relation declares, so the weld names that node.
    let mut realization = compact_pilot();
    let exit = ash_exit(OperationId::Redeem);

    add_lifecycle_node(&mut realization, exit.clone());

    let source =
        realization.lifecycle_node_by_id[&ash_representation(RepresentationMode::Explicit)];
    let target = realization.lifecycle_node_by_id[&exit];
    realization
        .lifecycle_graph
        .add_edge(source, target, LifecycleEdge::RequiresExit);

    assert_eq!(
        validated(&realization),
        Err(RealizationError::UndeclaredLifecycleNode(exit)),
    );
}

#[test]
fn a_missing_required_edge_is_rejected_with_both_nodes_present() {
    let mut realization = compact_pilot();
    let source =
        realization.lifecycle_node_by_id[&ash_representation(RepresentationMode::Explicit)];
    let target = realization.lifecycle_node_by_id[&ash_exit(OperationId::Clear)];
    let edge = realization
        .lifecycle_graph
        .find_edge(source, target)
        .expect("the pilot declares this exit path");

    realization.lifecycle_graph.remove_edge(edge);

    assert_eq!(
        validated(&realization),
        Err(RealizationError::MissingLifecyclePath {
            object: ObjectId::Ash,
            mode: RepresentationMode::Explicit,
            exit: OperationId::Clear,
        }),
    );
}

#[test]
fn a_required_exit_no_representation_can_reach_is_rejected() {
    // Coherently removing the representation relation *and* its graph
    // nodes leaves a closed but incoherent lifecycle: exits nothing may
    // ever reach.
    let mut realization = compact_pilot();

    realization
        .operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .relations
        .retain(|relation| !matches!(relation.relation, Relation::Representation { .. }));

    let (graph, nodes, _) = build_lifecycle_graph(
        [
            LifecycleNode {
                id: ash_exit(OperationId::CompactAsh),
            },
            LifecycleNode {
                id: ash_exit(OperationId::Clear),
            },
        ],
        Vec::<LifecycleDependencyDeclaration>::new(),
    )
    .unwrap();
    realization.lifecycle_graph = graph;
    realization.lifecycle_node_by_id = nodes;

    assert_eq!(
        validated(&realization),
        Err(RealizationError::LifecycleExitWithoutRepresentation {
            object: ObjectId::Ash,
            exit: OperationId::CompactAsh,
        }),
    );
}

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
