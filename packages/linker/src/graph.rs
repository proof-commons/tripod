//! Pass two of §14.3, and the cycle policy of §14.4.
//!
//! # The graph is derived, not declared
//!
//! Every edge below comes from a relocation the bundle emitted: a leaf
//! whose program pushes a symbol depends on that symbol, and the
//! constructor depends on everything it binds. Nothing here is a
//! hand-written adjacency list, so a backend that started pushing a new
//! symbol grows an edge without anyone remembering to add one.
//!
//! # The one edge that is not a relocation
//!
//! There is exactly one, and it is the important one. The ASH
//! constructor's witness program is the taproot output committing to
//! the taptree over the leaves, and those leaves push that same program
//! as a literal to recognize an ASH input and the successor. So the
//! symbol's value is a function of the constructor, which is a function
//! of the leaves, which are functions of the symbol.
//!
//! That edge is added unconditionally, because it is a fact about the
//! emitted design rather than an option. What the caller's declared
//! strategy changes is the edge's *classification*, which is exactly
//! what §14.4 asks for: find the component, classify every cyclic edge,
//! require an explicit authenticated resolution strategy, and reject an
//! unclassified one. Finding the component is not accepting it, and no
//! path in this module searches for the fixed point by hashing until
//! bytes stop changing — §14.4 prohibits that outright, and the
//! prohibition is kept by there being no loop to prohibit.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use tapscript::{
    BundleSymbol, CandidateRelocatableTapscriptBundle, LeafRole, RelocationSite, SymbolBinding,
};

use crate::deployment::SelfCommitmentStrategy;
use crate::error::LinkRefusal;

/// One node of the typed reference graph.
///
/// Symbols and the constructor, and nothing else: a leaf is reached
/// through its own leaf-script symbol, so a leaf is not a separate node
/// kind that could disagree with the symbol naming it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceNode {
    /// The static constructor, which binds every committed leaf.
    Constructor,
    /// One typed symbol.
    Symbol(BundleSymbol),
}

/// One edge's stable identity.
///
/// Assigned from the canonical order of the edge set, so the same
/// bundle and the same deployment always number the same edge the same
/// way and a refusal naming an edge names the same edge twice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReferenceEdgeId(usize);

impl ReferenceEdgeId {
    /// The edge's index in canonical order.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// One strongly connected component's stable identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SccId(usize);

impl SccId {
    /// The component's index in canonical order.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// How one reference is discharged.
///
/// The classification the package contract fixes. A cyclic edge must
/// carry one that is not [`Self::UnsupportedDependency`], and the
/// linker refuses rather than guessing which one applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ReferenceClass {
    /// A value the bundle itself settles before any deployment.
    StaticLinkTimeConstant,
    /// A value the program reads from the target at spend time.
    IdentityIntrospection,
    /// A value the program reconstructs from what it introspects,
    /// rather than carrying a literal for it.
    InProgramConstructorReconstruction,
    /// A value fixed by a separately authenticated ceremony, with the
    /// continuity left as an explicit outstanding obligation.
    AuthenticatedWitnessedRootContinuity,
    /// A value the deployment parameters settle.
    DeploymentRelocation,
    /// A dependency no strategy resolves.
    UnsupportedDependency,
}

impl ReferenceClass {
    /// Whether this classification resolves a cyclic edge.
    #[must_use]
    pub const fn resolves_a_cycle(self) -> bool {
        matches!(
            self,
            Self::IdentityIntrospection
                | Self::InProgramConstructorReconstruction
                | Self::AuthenticatedWitnessedRootContinuity
        )
    }
}

/// One typed reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReferenceEdge {
    id: ReferenceEdgeId,
    from: ReferenceNode,
    to: ReferenceNode,
    class: ReferenceClass,
    sites: NonZeroUsize,
}

impl ReferenceEdge {
    /// The edge's stable identity.
    #[must_use]
    pub const fn id(self) -> ReferenceEdgeId {
        self.id
    }

    /// The referring node.
    ///
    /// Named `referrer` rather than `from`: an inherent `from` on a
    /// type shadows the conversion trait's associated function at every
    /// call site, which turns a method reference into a resolution
    /// puzzle for no gain.
    #[must_use]
    pub const fn referrer(self) -> ReferenceNode {
        self.from
    }

    /// The referenced node.
    #[must_use]
    pub const fn referent(self) -> ReferenceNode {
        self.to
    }

    /// How the reference is discharged.
    #[must_use]
    pub const fn class(self) -> ReferenceClass {
        self.class
    }

    /// How many places the reference covers.
    #[must_use]
    pub const fn sites(self) -> NonZeroUsize {
        self.sites
    }
}

/// One strongly connected component of the frozen graph.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StronglyConnectedComponent {
    id: SccId,
    members: BTreeSet<ReferenceNode>,
    cyclic_edges: BTreeSet<ReferenceEdgeId>,
}

impl StronglyConnectedComponent {
    /// The component's stable identity.
    #[must_use]
    pub const fn id(&self) -> SccId {
        self.id
    }

    /// Its members, normalized by stable key.
    #[must_use]
    pub const fn members(&self) -> &BTreeSet<ReferenceNode> {
        &self.members
    }

    /// Every edge with both ends inside it.
    #[must_use]
    pub const fn cyclic_edges(&self) -> &BTreeSet<ReferenceEdgeId> {
        &self.cyclic_edges
    }

    /// Whether the component is a cycle rather than a single node.
    #[must_use]
    pub fn is_cyclic(&self) -> bool {
        self.members.len() > 1 || !self.cyclic_edges.is_empty()
    }
}

/// The frozen reference graph pass two builds (§14.3).
///
/// Frozen in the sense the rule means: it is built once, from a
/// definition census that is already complete, and nothing mutates it
/// afterwards. There is no method here that adds an edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenReferenceGraph {
    nodes: BTreeSet<ReferenceNode>,
    edges: BTreeMap<ReferenceEdgeId, ReferenceEdge>,
    components: Vec<StronglyConnectedComponent>,
}

impl FrozenReferenceGraph {
    /// Every node, in canonical order.
    #[must_use]
    pub const fn nodes(&self) -> &BTreeSet<ReferenceNode> {
        &self.nodes
    }

    /// Every edge, in canonical order.
    #[must_use]
    pub const fn edges(&self) -> &BTreeMap<ReferenceEdgeId, ReferenceEdge> {
        &self.edges
    }

    /// Every strongly connected component, in canonical order.
    #[must_use]
    pub fn components(&self) -> &[StronglyConnectedComponent] {
        &self.components
    }

    /// Every component that is a cycle.
    pub fn cycles(&self) -> impl Iterator<Item = &StronglyConnectedComponent> {
        self.components
            .iter()
            .filter(|component| component.is_cyclic())
    }

    /// Every edge referring to one node.
    pub fn referrers(&self, node: ReferenceNode) -> impl Iterator<Item = &ReferenceEdge> {
        self.edges.values().filter(move |edge| edge.to == node)
    }
}

/// The symbol naming one leaf's committed program.
#[must_use]
pub const fn leaf_symbol(leaf: LeafRole) -> BundleSymbol {
    match leaf {
        LeafRole::Coordinator { shape } => BundleSymbol::CoordinatorProgram { shape },
        LeafRole::Member { ash_inputs } => BundleSymbol::MemberProgram { ash_inputs },
    }
}

/// Pass two: resolve every reference and freeze the graph (§14.3).
///
/// # Errors
///
/// [`LinkRefusal::UnknownReferenceTarget`] when a relocation names a
/// symbol the definition census does not hold.
pub fn resolve_references(
    bundle: &CandidateRelocatableTapscriptBundle,
    strategy: SelfCommitmentStrategy,
) -> Result<FrozenReferenceGraph, LinkRefusal> {
    // Collected as a set first so the identities below are assigned
    // from the canonical order rather than from iteration order, and
    // so a symbol referenced twice from one leaf under two target roles
    // becomes one edge whose site count is the sum.
    let mut collected: BTreeMap<(ReferenceNode, ReferenceNode), (ReferenceClass, usize)> =
        BTreeMap::new();
    let mut nodes: BTreeSet<ReferenceNode> = BTreeSet::from([ReferenceNode::Constructor]);

    for symbol in bundle.symbols().keys() {
        nodes.insert(ReferenceNode::Symbol(*symbol));
    }

    for relocation in bundle.relocations() {
        let to = ReferenceNode::Symbol(relocation.symbol());
        if !nodes.contains(&to) {
            return Err(LinkRefusal::UnknownReferenceTarget(ReferenceEdgeId(
                collected.len(),
            )));
        }

        let from = match relocation.site() {
            RelocationSite::ProgramInstructions { leaf, .. } => {
                ReferenceNode::Symbol(leaf_symbol(*leaf))
            }
            RelocationSite::ConstructorBinding => ReferenceNode::Constructor,
        };

        let class = match bundle
            .symbols()
            .get(&relocation.symbol())
            .map(|entry| entry.binding())
        {
            Some(SymbolBinding::ResolvedAtLink) => ReferenceClass::DeploymentRelocation,
            Some(SymbolBinding::DefinedByBundle) | None => ReferenceClass::StaticLinkTimeConstant,
        };

        let entry = collected.entry((from, to)).or_insert((class, 0));
        entry.1 = entry.1.saturating_add(relocation.multiplicity().get());
    }

    // The self-commitment edge. Added unconditionally: the emitted
    // programs push the ASH constructor's witness program as a literal,
    // and that program is the taproot output over the very leaves the
    // constructor binds. The caller's strategy classifies the edge; it
    // does not decide whether the dependency exists.
    let self_commitment = (
        ReferenceNode::Symbol(BundleSymbol::AshConstructorProgram),
        ReferenceNode::Constructor,
    );
    let class = match strategy {
        SelfCommitmentStrategy::NotStated => ReferenceClass::UnsupportedDependency,
        SelfCommitmentStrategy::IdentityIntrospection => {
            ReferenceClass::InProgramConstructorReconstruction
        }
        SelfCommitmentStrategy::ExternallyAuthenticatedCommitment => {
            ReferenceClass::AuthenticatedWitnessedRootContinuity
        }
    };
    collected.insert(self_commitment, (class, 1));

    let mut edges = BTreeMap::new();
    for (index, ((from, to), (class, sites))) in collected.into_iter().enumerate() {
        let id = ReferenceEdgeId(index);
        edges.insert(
            id,
            ReferenceEdge {
                id,
                from,
                to,
                class,
                sites: NonZeroUsize::new(sites).unwrap_or(NonZeroUsize::MIN),
            },
        );
    }

    let components = strong_components(&nodes, &edges);

    Ok(FrozenReferenceGraph {
        nodes,
        edges,
        components,
    })
}

/// The deterministic strongly-connected components of the graph.
///
/// §14.4 names Petgraph for this, and the workspace already carries it.
/// Determinism is not inherited from the algorithm: the node indices
/// are assigned from the canonical node order, the edges are inserted
/// in canonical edge order, each component's members are normalized
/// into a set, and the components are then sorted by their least
/// member. Two runs over one bundle produce one answer, in one order.
fn strong_components(
    nodes: &BTreeSet<ReferenceNode>,
    edges: &BTreeMap<ReferenceEdgeId, ReferenceEdge>,
) -> Vec<StronglyConnectedComponent> {
    let mut graph: DiGraph<ReferenceNode, ReferenceEdgeId> = DiGraph::new();
    let mut indices: BTreeMap<ReferenceNode, NodeIndex> = BTreeMap::new();
    for node in nodes {
        indices.insert(*node, graph.add_node(*node));
    }
    for edge in edges.values() {
        if let (Some(from), Some(to)) = (indices.get(&edge.from), indices.get(&edge.to)) {
            graph.add_edge(*from, *to, edge.id);
        }
    }

    let mut components: Vec<BTreeSet<ReferenceNode>> = tarjan_scc(&graph)
        .into_iter()
        .map(|component| {
            component
                .into_iter()
                .filter_map(|index| graph.node_weight(index).copied())
                .collect()
        })
        .collect();
    components.sort();

    components
        .into_iter()
        .enumerate()
        .map(|(index, members)| {
            let cyclic_edges = edges
                .values()
                .filter(|edge| members.contains(&edge.from) && members.contains(&edge.to))
                .map(|edge| edge.id)
                .collect();
            StronglyConnectedComponent {
                id: SccId(index),
                members,
                cyclic_edges,
            }
        })
        .collect()
}

/// Apply §14.4's cycle policy to a frozen graph.
///
/// Two conditions, and they are different questions. Every cyclic edge
/// must carry a classification at all — an edge classified
/// [`ReferenceClass::UnsupportedDependency`] is the unclassified case
/// the rule rejects outright. And the component as a whole must be cut
/// somewhere: at least one of its edges must carry an explicit
/// authenticated resolution strategy, because a cycle every one of
/// whose edges is an ordinary static or deployment reference is a cycle
/// nothing breaks.
///
/// Requiring *every* edge to be cycle-resolving would be the wrong
/// reading and would reject every cycle without exception: a
/// constructor binding its own leaves is an ordinary static reference,
/// and it is cyclic here only because something else closes the loop.
/// One cut is what a resolution is.
///
/// The rejection is specific where the linker can say why. A component
/// demanding that a symbol's own value appear inside the thing
/// computing it is an impossible static fixed point rather than merely
/// an unclassified cycle, and only repeated hashing until bytes
/// stabilize could search for it — which §14.4 prohibits, and which no
/// loop in this module performs.
///
/// # Errors
///
/// [`LinkRefusal::ImpossibleStaticFixedPoint`] for a self-commitment
/// component with no stated strategy, and
/// [`LinkRefusal::UnsupportedReferenceCycle`] for any other cycle left
/// uncut or carrying an unclassified edge.
pub fn apply_cycle_policy(graph: &FrozenReferenceGraph) -> Result<(), LinkRefusal> {
    for component in graph.cycles() {
        let classified = |id: &ReferenceEdgeId| graph.edges.get(id).map(|edge| edge.class);

        let offending: BTreeSet<ReferenceEdgeId> = component
            .cyclic_edges
            .iter()
            .filter(|id| classified(id) == Some(ReferenceClass::UnsupportedDependency))
            .copied()
            .collect();
        let cut = component
            .cyclic_edges
            .iter()
            .any(|id| classified(id).is_some_and(ReferenceClass::resolves_a_cycle));

        if offending.is_empty() && cut {
            continue;
        }

        let edges = if offending.is_empty() {
            component.cyclic_edges.clone()
        } else {
            offending
        };

        let fixed_point = component.members.iter().find_map(|member| match member {
            ReferenceNode::Symbol(symbol @ BundleSymbol::AshConstructorProgram) => Some(*symbol),
            _ => None,
        });

        return Err(fixed_point.map_or_else(
            || LinkRefusal::UnsupportedReferenceCycle {
                component: component.id,
                members: component.members.clone(),
                edges: edges.clone(),
            },
            |symbol| LinkRefusal::ImpossibleStaticFixedPoint {
                component: component.id,
                symbol,
            },
        ));
    }

    Ok(())
}
