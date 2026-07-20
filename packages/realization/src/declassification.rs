//! Dependency-derived public-availability and declassification analysis.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[cfg(test)]
use architecture::ObjectId;
use architecture::OperationId;
use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{FactId, RealizationError, RelationId};
#[cfg(test)]
use crate::{RelationKind, RelationSubject, TransactionSide};

/// Visibility before operation execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InitialVisibility {
    Public,
    Private,
}

/// Stable disclosure-graph node identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisclosureNodeId {
    Fact(FactId),
    Relation(RelationId),
}

/// Direct Petgraph node weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisclosureNode {
    Fact {
        id: FactId,
        initial_visibility: InitialVisibility,
    },
    Relation {
        id: RelationId,
    },
}

impl DisclosureNode {
    #[must_use]
    pub fn id(&self) -> DisclosureNodeId {
        match self {
            Self::Fact { id, .. } => DisclosureNodeId::Fact(id.clone()),
            Self::Relation { id } => DisclosureNodeId::Relation(id.clone()),
        }
    }
}

/// Direct Petgraph dependency edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisclosureEdge {
    RelationOperand,
    ConstructibilityInput,
    PublicObservableInput,
}

/// Canonical graph-construction declaration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisclosureDependencyDeclaration {
    pub source: DisclosureNodeId,
    pub target: DisclosureNodeId,
    pub edge: DisclosureEdge,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisclosureDependencyProjection {
    pub source: DisclosureNodeId,
    pub target: DisclosureNodeId,
    pub edge: DisclosureEdge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisclosureGraphProjection {
    pub nodes: Vec<DisclosureNodeId>,
    pub edges: Vec<DisclosureDependencyProjection>,
}

/// One public-requirement seed.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisclosureSeed {
    pub node: DisclosureNodeId,
    pub reason: DisclosureReason,
}

/// Why a fact must be public.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DisclosureReason {
    PublicState,
    PublicEvent,
    PublicInterface,
    PermissionlessConstructibility {
        operation: OperationId,
        relation: RelationId,
    },
    TargetSafety {
        relation: RelationId,
    },
    DeploymentPolicy {
        policy: String,
    },
}

/// Required-public and newly-disclosed fact analysis.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeclassificationAnalysis {
    pub required_public: BTreeMap<FactId, BTreeSet<DisclosureReason>>,
    pub newly_disclosed: BTreeMap<FactId, BTreeSet<DisclosureReason>>,
    pub retained_private: BTreeSet<FactId>,
}

#[allow(clippy::type_complexity)]
pub fn build_disclosure_graph(
    nodes: impl IntoIterator<Item = DisclosureNode>,
    edges: impl IntoIterator<Item = DisclosureDependencyDeclaration>,
) -> Result<
    (
        DiGraph<DisclosureNode, DisclosureEdge, u32>,
        BTreeMap<DisclosureNodeId, NodeIndex<u32>>,
    ),
    RealizationError,
> {
    let mut nodes = nodes.into_iter().collect::<Vec<_>>();
    nodes.sort_by_key(DisclosureNode::id);

    for pair in nodes.windows(2) {
        if pair[0].id() == pair[1].id() {
            return Err(RealizationError::DuplicateDisclosureNode(pair[0].id()));
        }
    }

    let mut edges = edges.into_iter().collect::<Vec<_>>();
    edges.sort();

    let mut graph =
        DiGraph::<DisclosureNode, DisclosureEdge, u32>::with_capacity(nodes.len(), edges.len());
    let mut node_by_id = BTreeMap::new();

    for node_weight in nodes {
        let id = node_weight.id();
        let node = graph.add_node(node_weight);
        node_by_id.insert(id, node);
    }

    for edge in edges {
        let source = node_by_id
            .get(&edge.source)
            .copied()
            .ok_or_else(|| RealizationError::UnknownDisclosureNode(edge.source.clone()))?;
        let target = node_by_id
            .get(&edge.target)
            .copied()
            .ok_or_else(|| RealizationError::UnknownDisclosureNode(edge.target.clone()))?;
        graph.add_edge(source, target, edge.edge);
    }

    Ok((graph, node_by_id))
}

pub fn analyze_disclosure(
    graph: &DiGraph<DisclosureNode, DisclosureEdge, u32>,
    node_by_id: &BTreeMap<DisclosureNodeId, NodeIndex<u32>>,
    seeds: &[DisclosureSeed],
) -> Result<DeclassificationAnalysis, RealizationError> {
    let reasons_by_node = disclosure_reasons_by_node(graph, node_by_id, seeds)?;
    Ok(analysis_from_reason_map(graph, &reasons_by_node))
}

pub fn disclosure_reasons_by_node(
    graph: &DiGraph<DisclosureNode, DisclosureEdge, u32>,
    node_by_id: &BTreeMap<DisclosureNodeId, NodeIndex<u32>>,
    seeds: &[DisclosureSeed],
) -> Result<BTreeMap<NodeIndex<u32>, BTreeSet<DisclosureReason>>, RealizationError> {
    let mut reasons_by_node: BTreeMap<NodeIndex<u32>, BTreeSet<DisclosureReason>> = BTreeMap::new();
    let mut queue = VecDeque::new();
    let mut sorted_seeds = seeds.to_vec();
    sorted_seeds.sort();

    for seed in sorted_seeds {
        let node = node_by_id
            .get(&seed.node)
            .copied()
            .ok_or_else(|| RealizationError::UnknownDisclosureNode(seed.node.clone()))?;
        let entry = reasons_by_node.entry(node).or_default();

        if entry.insert(seed.reason) {
            queue.push_back(node);
        }
    }

    while let Some(consumer) = queue.pop_front() {
        let consumer_reasons = reasons_by_node.get(&consumer).cloned().unwrap_or_default();
        let mut dependencies = graph
            .edges_directed(consumer, Direction::Incoming)
            .map(|edge| edge.source())
            .collect::<Vec<_>>();
        dependencies.sort_by(|left, right| graph[*left].id().cmp(&graph[*right].id()));
        dependencies.dedup();

        for dependency in dependencies {
            let entry = reasons_by_node.entry(dependency).or_default();
            let old_len = entry.len();
            entry.extend(consumer_reasons.iter().cloned());

            if entry.len() != old_len {
                queue.push_back(dependency);
            }
        }
    }

    Ok(reasons_by_node)
}

pub fn analysis_from_reason_map(
    graph: &DiGraph<DisclosureNode, DisclosureEdge, u32>,
    reasons_by_node: &BTreeMap<NodeIndex<u32>, BTreeSet<DisclosureReason>>,
) -> DeclassificationAnalysis {
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

        match reasons_by_node.get(&node) {
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

    DeclassificationAnalysis {
        required_public,
        newly_disclosed,
        retained_private,
    }
}

#[must_use]
pub fn project_disclosure_graph(
    graph: &DiGraph<DisclosureNode, DisclosureEdge, u32>,
) -> DisclosureGraphProjection {
    let mut nodes = graph
        .node_weights()
        .map(DisclosureNode::id)
        .collect::<Vec<_>>();
    nodes.sort();
    let mut edges = graph
        .edge_references()
        .map(|edge| DisclosureDependencyProjection {
            source: graph[edge.source()].id(),
            target: graph[edge.target()].id(),
            edge: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();

    DisclosureGraphProjection { nodes, edges }
}

/// Phase-1 pilot disclosure analysis.
///
/// # Panics
///
/// Panics only if the built-in Phase-1 realization stops deriving.
#[must_use]
pub fn phase1_declassification() -> DeclassificationAnalysis {
    crate::derive::derive(
        &architecture::ARCHITECTURE,
        crate::RealizationScope::phase1_pilots(),
    )
    .expect("the built-in Phase-1 realization must derive")
    .declassification
}

#[allow(clippy::type_complexity)]
/// Phase-1 disclosure graph fixture.
///
/// # Panics
///
/// Panics only if the hard-coded Phase-1 disclosure declaration fixture is
/// internally inconsistent.
#[cfg(test)]
#[allow(clippy::too_many_lines)]
pub(crate) fn phase1_disclosure_declarations() -> (
    DiGraph<DisclosureNode, DisclosureEdge, u32>,
    BTreeMap<DisclosureNodeId, NodeIndex<u32>>,
    Vec<DisclosureSeed>,
) {
    let compact_constructibility = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Constructibility,
        RelationSubject::Operation,
    );
    let compact_representation = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::Ash,
        },
    );
    let compact_input_amount = FactId::FamilyAmount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: ObjectId::Ash,
    };
    let compact_output_amount = FactId::FamilyAmount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Output,
        object: ObjectId::Ash,
    };
    let compact_input_count = FactId::FamilyCount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: ObjectId::Ash,
    };
    let compact_bound = FactId::BoundValue {
        bound: architecture::BoundId::AshBatchMax,
    };
    let live_input_amount = FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side: TransactionSide::Input,
        object: ObjectId::ReceiptLive,
    };
    let live_output_amount = FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side: TransactionSide::Output,
        object: ObjectId::ReceiptLive,
    };
    let nodes = vec![
        fact(compact_input_amount.clone(), InitialVisibility::Public),
        fact(compact_output_amount.clone(), InitialVisibility::Public),
        fact(compact_input_count.clone(), InitialVisibility::Public),
        fact(compact_bound.clone(), InitialVisibility::Public),
        fact(live_input_amount.clone(), InitialVisibility::Private),
        fact(live_output_amount.clone(), InitialVisibility::Private),
        DisclosureNode::Relation {
            id: compact_constructibility.clone(),
        },
        DisclosureNode::Relation {
            id: compact_representation.clone(),
        },
        DisclosureNode::Relation {
            id: RelationId::new(
                OperationId::TransferLive,
                RelationKind::Conservation,
                RelationSubject::Asset {
                    asset: architecture::AssetId::U,
                },
            ),
        },
    ];
    let edges = vec![
        edge(
            DisclosureNodeId::Fact(compact_input_amount),
            DisclosureNodeId::Relation(compact_constructibility.clone()),
            DisclosureEdge::ConstructibilityInput,
        ),
        edge(
            DisclosureNodeId::Fact(compact_input_count),
            DisclosureNodeId::Relation(compact_constructibility.clone()),
            DisclosureEdge::ConstructibilityInput,
        ),
        edge(
            DisclosureNodeId::Fact(compact_bound),
            DisclosureNodeId::Relation(compact_constructibility.clone()),
            DisclosureEdge::ConstructibilityInput,
        ),
        edge(
            DisclosureNodeId::Fact(compact_output_amount),
            DisclosureNodeId::Relation(compact_representation.clone()),
            DisclosureEdge::PublicObservableInput,
        ),
        edge(
            DisclosureNodeId::Fact(live_input_amount),
            DisclosureNodeId::Relation(RelationId::new(
                OperationId::TransferLive,
                RelationKind::Conservation,
                RelationSubject::Asset {
                    asset: architecture::AssetId::U,
                },
            )),
            DisclosureEdge::RelationOperand,
        ),
        edge(
            DisclosureNodeId::Fact(live_output_amount),
            DisclosureNodeId::Relation(RelationId::new(
                OperationId::TransferLive,
                RelationKind::Conservation,
                RelationSubject::Asset {
                    asset: architecture::AssetId::U,
                },
            )),
            DisclosureEdge::RelationOperand,
        ),
    ];
    let seeds = vec![
        DisclosureSeed {
            node: DisclosureNodeId::Relation(compact_constructibility.clone()),
            reason: DisclosureReason::PermissionlessConstructibility {
                operation: OperationId::CompactAsh,
                relation: compact_constructibility,
            },
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(compact_representation),
            reason: DisclosureReason::PublicInterface,
        },
    ];
    let (graph, node_by_id) =
        build_disclosure_graph(nodes, edges).expect("phase1 disclosure graph builds");

    (graph, node_by_id, seeds)
}

#[cfg(test)]
fn fact(id: FactId, initial_visibility: InitialVisibility) -> DisclosureNode {
    DisclosureNode::Fact {
        id,
        initial_visibility,
    }
}

#[cfg(test)]
fn edge(
    source: DisclosureNodeId,
    target: DisclosureNodeId,
    edge: DisclosureEdge,
) -> DisclosureDependencyDeclaration {
    DisclosureDependencyDeclaration {
        source,
        target,
        edge,
    }
}
