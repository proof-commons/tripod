//! Frozen STATE dependencies and unresolved-cycle refusals.
//!
//! Edges point from dependent to dependency, matching the bundle reference graph.
//! Wave 6 owns binding-time resolution, authenticated cuts, the residual-DAG
//! proof, canonical link ordering and integration into the link pipeline.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use petgraph::algo::tarjan_scc;
use petgraph::graph::DiGraph;
use tapscript::{StateConstructorReference, StateReferenceDeclaration};

/// Maximum distinct references, including dependencies without declarations.
///
/// This matches the constructor's local census limit of 64. Repeated declarations
/// and edges consume no additional identities; at most 64 squared edges survive.
pub const STATE_REFERENCE_LIMIT: usize = 64;

/// One dependency edge with an identity assigned in canonical endpoint order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateReferenceEdge {
    id: usize,
    referrer: StateConstructorReference,
    referent: StateConstructorReference,
}

impl StateReferenceEdge {
    /// The edge's zero-based index in canonical endpoint order.
    #[must_use]
    pub const fn id(self) -> usize {
        self.id
    }

    /// The dependent reference.
    #[must_use]
    pub const fn referrer(self) -> StateConstructorReference {
        self.referrer
    }

    /// The dependency reference.
    #[must_use]
    pub const fn referent(self) -> StateConstructorReference {
        self.referent
    }
}

/// One canonical strongly connected component of STATE references.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateReferenceComponent {
    id: StateConstructorReference,
    members: BTreeSet<StateConstructorReference>,
    cyclic_edges: BTreeSet<usize>,
}

impl StateReferenceComponent {
    /// The smallest reference in this component, independent of traversal order.
    #[must_use]
    pub const fn id(&self) -> StateConstructorReference {
        self.id
    }

    /// Every member in the reference type's own order.
    #[must_use]
    pub const fn members(&self) -> &BTreeSet<StateConstructorReference> {
        &self.members
    }

    /// Canonical identities of all edges internal to this component.
    #[must_use]
    pub const fn cyclic_edges(&self) -> &BTreeSet<usize> {
        &self.cyclic_edges
    }

    /// Whether this component contains a cycle, including a singleton self-loop.
    #[must_use]
    pub fn is_cyclic(&self) -> bool {
        self.members.len() > 1 || !self.cyclic_edges.is_empty()
    }
}

/// A closed refusal from freezing or checking STATE references.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateReferenceGraphRefusal {
    /// The input contains more distinct references than the admitted bound.
    ResourceBoundExceeded {
        /// The maximum number of distinct references.
        limit: usize,
    },
    /// Every reported component remains unresolved; no cut was selected.
    ConstructorReferenceCycleUnresolved {
        /// All cyclic components, ordered by their smallest references.
        components: Vec<StateReferenceComponent>,
    },
}

impl fmt::Display for StateReferenceGraphRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceBoundExceeded { limit } => {
                write!(formatter, "STATE reference limit exceeded: {limit}")
            }
            Self::ConstructorReferenceCycleUnresolved { components } => write!(
                formatter,
                "constructor-reference-cycle-unresolved: {} components",
                components.len()
            ),
        }
    }
}

impl std::error::Error for StateReferenceGraphRefusal {}

/// An immutable graph derived solely from typed STATE declarations.
///
/// References identify nodes by value; their numeric indices and edge identities
/// follow canonical order. Duplicate declarations union their dependencies.
/// A dependency need not have its own declaration: freezing records its identity
/// without claiming its binding is resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenStateReferenceGraph {
    nodes: BTreeSet<StateConstructorReference>,
    edges: Vec<StateReferenceEdge>,
    components: Vec<StateReferenceComponent>,
}

impl FrozenStateReferenceGraph {
    /// Freeze canonical nodes, edges and strongly connected components.
    ///
    /// Cycles are retained for inspection. Call
    /// [`Self::require_acyclic_without_selecting_cut`] to refuse them.
    ///
    /// # Errors
    /// Returns [`StateReferenceGraphRefusal::ResourceBoundExceeded`] as soon as
    /// a distinct reference exceeds [`STATE_REFERENCE_LIMIT`].
    pub fn new(
        declarations: &[StateReferenceDeclaration],
    ) -> Result<Self, StateReferenceGraphRefusal> {
        let mut nodes = BTreeSet::new();
        let mut endpoints = BTreeSet::new();
        for declaration in declarations {
            insert_reference(&mut nodes, declaration.reference)?;
            for dependency in &declaration.dependencies {
                insert_reference(&mut nodes, *dependency)?;
                endpoints.insert((declaration.reference, *dependency));
            }
        }
        let edges: Vec<_> = endpoints
            .into_iter()
            .enumerate()
            .map(|(id, (referrer, referent))| StateReferenceEdge {
                id,
                referrer,
                referent,
            })
            .collect();
        let components = strong_components(&nodes, &edges);
        Ok(Self {
            nodes,
            edges,
            components,
        })
    }

    /// Every declared or dependency reference in canonical order.
    #[must_use]
    pub const fn nodes(&self) -> &BTreeSet<StateConstructorReference> {
        &self.nodes
    }

    /// A node's zero-based canonical index, if the reference occurs in the graph.
    #[must_use]
    pub fn node_id(&self, reference: StateConstructorReference) -> Option<usize> {
        self.nodes.iter().position(|node| *node == reference)
    }

    /// Every distinct edge in canonical endpoint order.
    #[must_use]
    pub fn edges(&self) -> &[StateReferenceEdge] {
        &self.edges
    }

    /// Every component, ordered by its smallest reference.
    #[must_use]
    pub fn components(&self) -> &[StateReferenceComponent] {
        &self.components
    }

    /// Every cyclic component in canonical component order.
    pub fn cyclic_components(&self) -> impl Iterator<Item = &StateReferenceComponent> {
        self.components
            .iter()
            .filter(|component| component.is_cyclic())
    }

    /// Refuse all unresolved cycles without selecting or accepting a cut.
    ///
    /// Success establishes only acyclicity of these declarations. Wave 6 owns
    /// binding-time resolution, authenticated cuts and the residual-DAG proof.
    ///
    /// # Errors
    /// Returns [`StateReferenceGraphRefusal::ConstructorReferenceCycleUnresolved`]
    /// with every cyclic component, including singleton self-loops.
    pub fn require_acyclic_without_selecting_cut(&self) -> Result<(), StateReferenceGraphRefusal> {
        let components: Vec<_> = self.cyclic_components().cloned().collect();
        if components.is_empty() {
            Ok(())
        } else {
            Err(StateReferenceGraphRefusal::ConstructorReferenceCycleUnresolved { components })
        }
    }
}

fn insert_reference(
    nodes: &mut BTreeSet<StateConstructorReference>,
    reference: StateConstructorReference,
) -> Result<(), StateReferenceGraphRefusal> {
    nodes.insert(reference);
    if nodes.len() > STATE_REFERENCE_LIMIT {
        Err(StateReferenceGraphRefusal::ResourceBoundExceeded {
            limit: STATE_REFERENCE_LIMIT,
        })
    } else {
        Ok(())
    }
}

fn strong_components(
    nodes: &BTreeSet<StateConstructorReference>,
    edges: &[StateReferenceEdge],
) -> Vec<StateReferenceComponent> {
    let mut graph = DiGraph::<StateConstructorReference, ()>::new();
    let indices: BTreeMap<_, _> = nodes
        .iter()
        .map(|node| (*node, graph.add_node(*node)))
        .collect();
    for edge in edges {
        graph.add_edge(indices[&edge.referrer], indices[&edge.referent], ());
    }
    let mut components = Vec::new();
    for connected in tarjan_scc(&graph) {
        let members: BTreeSet<_> = connected.into_iter().map(|index| graph[index]).collect();
        if let Some(id) = members.first().copied() {
            let cyclic_edges = edges
                .iter()
                .filter(|edge| members.contains(&edge.referrer) && members.contains(&edge.referent))
                .map(|edge| edge.id)
                .collect();
            components.push(StateReferenceComponent {
                id,
                members,
                cyclic_edges,
            });
        }
    }
    components.sort_by_key(StateReferenceComponent::id);
    components
}
