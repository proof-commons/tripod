use std::collections::{BTreeMap, BTreeSet};

use petgraph::visit::EdgeRef;

use crate::{
    DisclosureDependencyDeclaration, DisclosureEdge, DisclosureNode, DisclosureNodeId,
    DisclosureReason, FactId, InitialVisibility, RealizationError, RealizationScope,
    TransactionSide,
    declassification::{
        analysis_from_reason_map, build_disclosure_graph, disclosure_reasons_by_node,
        phase1_disclosure_declarations,
    },
    derive, phase1_declassification,
};

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
