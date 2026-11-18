use std::collections::{BTreeMap, BTreeSet};

use petgraph::visit::EdgeRef;

use crate::{
    ArchitectureBinding, DisclosureDependencyDeclaration, DisclosureEdge, DisclosureNode,
    DisclosureNodeId, DisclosureReason, DisclosureSeed, FactId, InitialVisibility,
    RealizationError, RealizationScope, TransactionSide,
    declassification::{
        analysis_from_reason_map, build_disclosure_graph, disclosure_reasons_by_node,
        phase1_disclosure_declarations,
    },
    derive,
    derive::assemble_scoped_realization,
    error::SharedBoundDependencyKind,
    phase1_declassification,
};

fn shared_bound_seed_free_control() -> (RealizationScope, crate::ScopedRealizationSpec) {
    use architecture::{ARCHITECTURE, OperationId};
    let scope = RealizationScope::from_operations([OperationId::Burn, OperationId::Clear]).unwrap();
    let control = derive(&ARCHITECTURE, scope.clone()).unwrap();
    (scope, control)
}

#[test]
fn shared_bounds_without_incoming_dependencies_merge() {
    use architecture::{BoundId, OperationId};
    let (_, control) = shared_bound_seed_free_control();
    let shared = DisclosureNodeId::Fact(FactId::BoundValue {
        bound: BoundId::FeeSponsorInputMax,
    });
    for operation in [OperationId::Burn, OperationId::Clear] {
        assert!(
            control.operations[&operation]
                .disclosure_nodes
                .iter()
                .any(|node| node.id() == shared)
        );
        assert!(
            control.operations[&operation]
                .disclosure_edges
                .iter()
                .all(|edge| edge.target != shared)
        );
        assert!(
            control.operations[&operation]
                .disclosure_seeds
                .iter()
                .all(|seed| seed.node != shared)
        );
    }
    assert_eq!(
        control
            .disclosure_graph
            .node_weights()
            .filter(|node| node.id() == shared)
            .count(),
        1
    );
    super::derivation_tests::assert_pair_disclosure_union(
        &control,
        [OperationId::Burn, OperationId::Clear],
    );
}

#[test]
fn incoming_dependency_on_shared_bound_is_refused() {
    use architecture::{ARCHITECTURE, BoundId, OperationId};
    let (scope, control) = shared_bound_seed_free_control();
    let mut operations = control.operations;
    operations
        .get_mut(&OperationId::Burn)
        .unwrap()
        .disclosure_edges
        .push(DisclosureDependencyDeclaration {
            source: DisclosureNodeId::Fact(FactId::BoundValue {
                bound: BoundId::BurnInputMax,
            }),
            target: DisclosureNodeId::Fact(FactId::BoundValue {
                bound: BoundId::FeeSponsorInputMax,
            }),
            edge: DisclosureEdge::PublicObservableInput,
        });
    let error = assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        scope,
        operations,
    )
    .unwrap_err();
    assert_eq!(
        error,
        RealizationError::UnsupportedSharedBoundDependency {
            bound: BoundId::FeeSponsorInputMax,
            operation: OperationId::Burn,
            kind: SharedBoundDependencyKind::IncomingEdge,
        }
    );
    assert_eq!(error.trace_kind(), "unsupported_shared_bound_dependency");
    assert!(error.to_string().contains("incoming edge"));
}

#[test]
fn seed_on_shared_bound_is_refused() {
    use architecture::{ARCHITECTURE, BoundId, OperationId};
    let (scope, control) = shared_bound_seed_free_control();
    let mut operations = control.operations;
    operations
        .get_mut(&OperationId::Clear)
        .unwrap()
        .disclosure_seeds
        .push(DisclosureSeed {
            node: DisclosureNodeId::Fact(FactId::BoundValue {
                bound: BoundId::FeeSponsorInputMax,
            }),
            reason: DisclosureReason::PublicInterface,
        });
    let error = assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        scope,
        operations,
    )
    .unwrap_err();
    assert_eq!(
        error,
        RealizationError::UnsupportedSharedBoundDependency {
            bound: BoundId::FeeSponsorInputMax,
            operation: OperationId::Clear,
            kind: SharedBoundDependencyKind::Seed,
        }
    );
    assert_eq!(error.trace_kind(), "unsupported_shared_bound_dependency");
    assert!(error.to_string().contains("seed"));
}

#[test]
fn union_refuses_conflicting_disclosure_visibility() {
    use architecture::{ARCHITECTURE, BoundId, OperationId};
    let scope = RealizationScope::from_operations([OperationId::Burn, OperationId::Clear]).unwrap();
    let control = derive(&ARCHITECTURE, scope.clone()).unwrap();
    let id = DisclosureNodeId::Fact(FactId::BoundValue {
        bound: BoundId::FeeSponsorInputMax,
    });
    let mut changed = control.operations;
    let node = changed
        .get_mut(&OperationId::Burn)
        .unwrap()
        .disclosure_nodes
        .iter_mut()
        .find(|node| node.id() == id)
        .unwrap();
    let DisclosureNode::Fact {
        initial_visibility, ..
    } = node
    else {
        unreachable!()
    };
    assert_eq!(*initial_visibility, InitialVisibility::Public);
    *initial_visibility = InitialVisibility::Private;
    assert_eq!(
        assemble_scoped_realization(
            &ARCHITECTURE,
            ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
            scope,
            changed
        )
        .unwrap_err(),
        RealizationError::DuplicateDisclosureNode(id)
    );
}

#[test]
fn union_preserves_within_operation_disclosure_duplicates() {
    use architecture::{ARCHITECTURE, BoundId, OperationId};
    let shared_scope =
        RealizationScope::from_operations([OperationId::CompactAsh, OperationId::Clear]).unwrap();
    let shared_control = derive(&ARCHITECTURE, shared_scope.clone()).unwrap();
    let shared_id = DisclosureNodeId::Fact(FactId::BoundValue {
        bound: BoundId::AshBatchMax,
    });
    let mut shared_nodes = shared_control.operations;
    let repeated_bound = shared_nodes[&OperationId::CompactAsh]
        .disclosure_nodes
        .iter()
        .find(|node| node.id() == shared_id)
        .unwrap()
        .clone();
    shared_nodes
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .disclosure_nodes
        .push(repeated_bound);
    assert_eq!(
        assemble_scoped_realization(
            &ARCHITECTURE,
            ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
            shared_scope,
            shared_nodes,
        )
        .unwrap_err(),
        RealizationError::DuplicateDisclosureNode(shared_id)
    );

    let scope =
        RealizationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap();
    let control = derive(&ARCHITECTURE, scope.clone()).unwrap();
    let binding = ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap();
    let mut nodes = control.operations.clone();
    let duplicate = nodes[&OperationId::CompactAsh].disclosure_nodes[0].clone();
    nodes
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .disclosure_nodes
        .push(duplicate.clone());
    assert_eq!(
        assemble_scoped_realization(&ARCHITECTURE, binding.clone(), scope.clone(), nodes)
            .unwrap_err(),
        RealizationError::DuplicateDisclosureNode(duplicate.id())
    );
    let mut edges = control.operations;
    let duplicate = edges[&OperationId::CompactAsh].disclosure_edges[0].clone();
    edges
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .disclosure_edges
        .push(duplicate.clone());
    assert_eq!(
        assemble_scoped_realization(&ARCHITECTURE, binding, scope, edges).unwrap_err(),
        RealizationError::DuplicateDisclosureDependency(duplicate)
    );
}

#[test]
fn phase1_disclosure_classifies_pilots_correctly() {
    let analysis = phase1_declassification();

    assert!(analysis.newly_disclosed.is_empty());
    assert!(
        analysis
            .required_public
            .contains_key(&FactId::FamilyAmount {
                operation: architecture::OperationId::CompactAsh,
                side: TransactionSide::Input,
                object: architecture::ObjectId::Ash,
            })
    );
    assert!(analysis.required_public.contains_key(&FactId::FamilyCount {
        operation: architecture::OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    }));
    assert!(analysis.retained_private.contains(&FactId::FamilyAmount {
        operation: architecture::OperationId::TransferLive,
        side: TransactionSide::Input,
        object: architecture::ObjectId::ReceiptLive,
    }));
}

#[test]
fn production_phase1_declassification_equals_operation_derived_analysis() {
    let realization = derive(
        &architecture::ARCHITECTURE,
        RealizationScope::phase1_pilots(),
    )
    .expect("Phase-1 realization derives");

    assert_eq!(
        phase1_declassification(),
        realization.declassification,
        "the public Phase-1 declassification helper and production operation declarations have drifted"
    );
}

#[test]
fn disclosure_worklist_matches_full_scan_oracle() {
    let (graph, nodes, seeds) = phase1_disclosure_declarations();
    let worklist = disclosure_reasons_by_node(&graph, &nodes, &seeds).unwrap();
    let oracle = analyze_disclosure_by_full_scan(&graph, &nodes, &seeds);

    assert_eq!(worklist_by_id(&graph, &worklist), oracle);
    assert_eq!(
        analysis_from_reason_map(&graph, &worklist),
        analysis_from_reason_map_by_id(&graph, &oracle),
    );
}

#[test]
fn duplicate_disclosure_dependency_is_rejected() {
    let source = FactId::FamilyCount {
        operation: architecture::OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    };
    let target = FactId::FamilyAmount {
        operation: architecture::OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    };
    let dependency = DisclosureDependencyDeclaration {
        source: DisclosureNodeId::Fact(source.clone()),
        target: DisclosureNodeId::Fact(target.clone()),
        edge: DisclosureEdge::RelationOperand,
    };

    let error = build_disclosure_graph(
        [
            DisclosureNode::Fact {
                id: source,
                initial_visibility: InitialVisibility::Public,
            },
            DisclosureNode::Fact {
                id: target,
                initial_visibility: InitialVisibility::Private,
            },
        ],
        [dependency.clone(), dependency.clone()],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::DuplicateDisclosureDependency(dependency)
    );
}

#[test]
fn cyclic_disclosure_dependencies_reach_the_least_fixed_point() {
    let first = FactId::FamilyCount {
        operation: architecture::OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    };
    let second = FactId::FamilyAmount {
        operation: architecture::OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    };
    let unseeded = FactId::BoundValue {
        bound: architecture::BoundId::AshBatchMax,
    };
    let reason = DisclosureReason::PublicInterface;
    let (graph, nodes) = build_disclosure_graph(
        [
            DisclosureNode::Fact {
                id: first.clone(),
                initial_visibility: InitialVisibility::Private,
            },
            DisclosureNode::Fact {
                id: second.clone(),
                initial_visibility: InitialVisibility::Private,
            },
            DisclosureNode::Fact {
                id: unseeded.clone(),
                initial_visibility: InitialVisibility::Private,
            },
        ],
        [
            DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(first.clone()),
                target: DisclosureNodeId::Fact(second.clone()),
                edge: DisclosureEdge::RelationOperand,
            },
            DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(second.clone()),
                target: DisclosureNodeId::Fact(first.clone()),
                edge: DisclosureEdge::RelationOperand,
            },
        ],
    )
    .unwrap();
    let worklist = disclosure_reasons_by_node(
        &graph,
        &nodes,
        &[crate::DisclosureSeed {
            node: DisclosureNodeId::Fact(first.clone()),
            reason: reason.clone(),
        }],
    )
    .unwrap();
    let analysis = analysis_from_reason_map(&graph, &worklist);

    assert_eq!(
        analysis.newly_disclosed,
        BTreeMap::from([
            (first, BTreeSet::from([reason.clone()])),
            (second, BTreeSet::from([reason])),
        ]),
    );
    assert_eq!(analysis.retained_private, BTreeSet::from([unseeded]));
}

fn analyze_disclosure_by_full_scan(
    graph: &petgraph::graph::DiGraph<crate::DisclosureNode, crate::DisclosureEdge, u32>,
    node_by_id: &BTreeMap<DisclosureNodeId, petgraph::graph::NodeIndex<u32>>,
    seeds: &[crate::DisclosureSeed],
) -> BTreeMap<DisclosureNodeId, BTreeSet<DisclosureReason>> {
    let mut result = BTreeMap::new();

    for seed in seeds {
        assert!(node_by_id.contains_key(&seed.node));
        result
            .entry(seed.node.clone())
            .or_insert_with(BTreeSet::new)
            .insert(seed.reason.clone());
    }

    loop {
        let before = result.clone();

        for edge in graph.edge_references() {
            let dependency = graph[edge.source()].id();
            let consumer = graph[edge.target()].id();
            let reasons = result.get(&consumer).cloned().unwrap_or_default();

            if !reasons.is_empty() {
                result
                    .entry(dependency)
                    .or_insert_with(BTreeSet::new)
                    .extend(reasons);
            }
        }

        if result == before {
            break;
        }
    }

    result
}

fn worklist_by_id(
    graph: &petgraph::graph::DiGraph<crate::DisclosureNode, crate::DisclosureEdge, u32>,
    reasons: &BTreeMap<petgraph::graph::NodeIndex<u32>, BTreeSet<DisclosureReason>>,
) -> BTreeMap<DisclosureNodeId, BTreeSet<DisclosureReason>> {
    reasons
        .iter()
        .map(|(node, reasons)| (graph[*node].id(), reasons.clone()))
        .collect()
}

fn analysis_from_reason_map_by_id(
    graph: &petgraph::graph::DiGraph<crate::DisclosureNode, crate::DisclosureEdge, u32>,
    reasons: &BTreeMap<DisclosureNodeId, BTreeSet<DisclosureReason>>,
) -> crate::DeclassificationAnalysis {
    let mut required_public = BTreeMap::new();
    let mut newly_disclosed = BTreeMap::new();
    let mut retained_private = BTreeSet::new();

    for node in graph.node_indices() {
        let DisclosureNode::Fact {
            id,
            initial_visibility,
        } = &graph[node]
        else {
            continue;
        };
        match reasons.get(&DisclosureNodeId::Fact(id.clone())) {
            Some(reasons) if !reasons.is_empty() => {
                required_public.insert(id.clone(), reasons.clone());
                if *initial_visibility == InitialVisibility::Private {
                    newly_disclosed.insert(id.clone(), reasons.clone());
                }
            }
            _ if *initial_visibility == InitialVisibility::Private => {
                retained_private.insert(id.clone());
            }
            _ => {}
        }
    }

    crate::DeclassificationAnalysis {
        required_public,
        newly_disclosed,
        retained_private,
    }
}

// Sponsor-value opacity read-set guard (F2-006): a PLAIN_LBTC
// family amount is sponsor-local data. Its absence from every semantic
// graph is enforced structurally at derivation time, not by
// convention.

#[test]
fn pilot_read_sets_contain_no_sponsor_amount() {
    use architecture::{ObjectId, OperationId};

    let realization = derive(
        &architecture::ARCHITECTURE,
        RealizationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap(),
    )
    .expect("both pilots derive");

    let is_sponsor_amount = |fact: &FactId| {
        matches!(
            fact,
            FactId::FamilyAmount {
                object: ObjectId::PlainLbtc,
                ..
            }
        )
    };

    assert!(
        !realization
            .disclosure_graph
            .node_weights()
            .any(|node| matches!(node, DisclosureNode::Fact { id, .. } if is_sponsor_amount(id)))
    );
    assert!(
        !realization
            .declassification
            .required_public
            .keys()
            .chain(realization.declassification.newly_disclosed.keys())
            .chain(realization.declassification.retained_private.iter())
            .any(is_sponsor_amount)
    );
    assert!(
        !realization
            .constructibility_node_by_id
            .keys()
            .any(|node| matches!(
                node,
                crate::ConstructibilityNodeId::Fact { fact, .. } if is_sponsor_amount(fact)
            ))
    );
    assert!(!realization.expression_graph.node_weights().any(
        |declaration| matches!(&declaration.node, crate::ExpressionNode::Fact(fact) if is_sponsor_amount(fact))
    ));
}

#[test]
fn a_sponsor_amount_read_is_rejected_structurally() {
    use architecture::{ObjectId, OperationId};

    let mut realization = derive(
        &architecture::ARCHITECTURE,
        RealizationScope::from_operations([OperationId::CompactAsh]).unwrap(),
    )
    .expect("pilot derives");

    // Inject the forbidden fact directly into the disclosure graph: a
    // hypothetical backend that models the sponsor input amount — even
    // as a private fact — violates the opacity read-set.
    realization.disclosure_graph.add_node(DisclosureNode::Fact {
        id: FactId::FamilyAmount {
            operation: OperationId::CompactAsh,
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
        initial_visibility: InitialVisibility::Private,
    });

    assert_eq!(
        crate::validate::validate_scoped_realization(&architecture::ARCHITECTURE, &realization),
        Err(RealizationError::SponsorValueRead),
    );
}

#[test]
fn announcement_sponsor_amount_declaration_rejects() {
    use super::announce_maturity_tests as announcement;
    for side in [TransactionSide::Input, TransactionSide::Output] {
        let mut declaration = announcement::realization()
            .operations
            .remove(&announcement::OP)
            .unwrap();
        declaration.disclosure_nodes.push(DisclosureNode::Fact {
            id: FactId::FamilyAmount {
                operation: announcement::OP,
                side,
                object: architecture::ObjectId::PlainLbtc,
            },
            initial_visibility: InitialVisibility::Private,
        });
        assert_eq!(
            announcement::rebuild(declaration).unwrap_err(),
            RealizationError::SponsorValueRead
        );
    }
}

#[test]
fn announcement_derivation_preserves_the_phase1_disclosure_helper() {
    let before = phase1_declassification();
    let announcement = super::announce_maturity_tests::realization();
    assert_eq!(announcement.declassification.required_public.len(), 15);
    assert_eq!(phase1_declassification(), before);
    assert_eq!(
        derive(
            &architecture::ARCHITECTURE,
            RealizationScope::phase1_pilots()
        )
        .unwrap()
        .declassification,
        before
    );
}
