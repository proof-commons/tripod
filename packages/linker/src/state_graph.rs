//! The maturity link's authenticated dependency graph.
//!
//! # What the graph states
//!
//! Every reference the maturity link resolves is a typed node here, and
//! every dependency between two of them is an edge carrying the time at
//! which the referent is bound. Three node kinds cover the whole link:
//! one key of the symbol census, one leaf program the constructor
//! commits, and the constructor's output, whose merkle root and output
//! key are a single node because they are a single commitment.
//!
//! A binding time is not a label on an edge that could have carried
//! another one. It says where the value comes from at the moment the
//! reference is resolved — the link's own bytes, the constructor's
//! recipe, an introspection, a witness the leaf then authenticates, or
//! a reconstruction from what was already authenticated — and that is
//! what decides whether removing the edge is a cut or a lie.
//!
//! # Why a cut is validated and removed rather than flagged
//!
//! Accepting a component as soon as one of its edges is the kind that
//! could break a cycle says nothing about that component's other
//! cycles. A component carrying two edge-disjoint cycles is accepted on
//! one such edge while the second cycle stands untouched, and the
//! result records nothing about the cycle that was never examined.
//! Removing exactly the edges whose binding time is checked against
//! evidence the record itself carries, and then proving that what
//! remains has no cycle at all, is a statement about the whole graph
//! rather than about one edge of it. The residual proof also yields
//! something a flag cannot: a dependency order, which exists precisely
//! when no cycle does.
//!
//! # Why the frozen constructor graph is retained
//!
//! [`FrozenStateReferenceGraph`] is the constructor's own statement of
//! the inputs it consumed, and it enumerates nothing on its own,
//! because its seven declarations carry empty dependency lists. That is
//! what makes it a projection rather than a competitor: the
//! constructor-kind nodes of this graph must be exactly its nodes, so a
//! kind added on one side and forgotten on the other is caught here
//! instead of being settled by whichever side a later reader consults.
//!
//! # Why the static root is never a literal beneath its own program
//!
//! A leaf cannot carry the root of the tree that commits to it: the
//! root is a function of the leaf's bytes, so a literal for it inside
//! those bytes is a fixed point, and searching for one by hashing until
//! the bytes stop changing is exactly what this link must not do. The
//! root reaches the program as a witness instead, and the tweak
//! equation against the internal key and the authenticated metadata is
//! what makes the witnessed bytes the committed ones. A link-time
//! constant in that position is therefore refused outright rather than
//! handed to the cycle analysis, because it is not a cycle that better
//! evidence could resolve.

use std::collections::{BTreeMap, BTreeSet};

use petgraph::algo::tarjan_scc;
use petgraph::graph::DiGraph;
use tapscript::{
    CandidateStateConstructor, StateAnnouncementId, StateAnnouncementProgram, StateLeafRole,
    StateProgramComponent, StateProgramWitness,
};

use crate::state_constructor_graph::{FrozenStateReferenceGraph, STATE_REFERENCE_LIMIT};
use crate::state_error::StateLinkRefusal;
use crate::state_symbol::{StateLinkSymbol, StateResolvedCensus};

// --- Binding times ------------------------------------------------------

/// When one reference's value is settled.
///
/// Five, because five different things settle a value in this link, and
/// three of them survive the moment the leaf is committed. The first two
/// are fixed before any spend and cannot break a cycle: an edge whose
/// referent is already in the bytes is exactly the dependency a cycle is
/// made of. The last three are bound while the leaf runs, which is why
/// removing one leaves a graph that can still be linked.
///
/// A reference to a previously linked bundle would be a backward,
/// acyclic edge — a resolution class and not a cut — because a bundle
/// that has already been linked is finished before this link starts and
/// nothing in it can depend on what this link produces. It is named here
/// so that [`Self::WitnessedThenAuthenticated`] is not mistaken for it,
/// and no variant is minted for it: nothing produces one, and a variant
/// nothing produces is a census entry nothing consumes. The guide
/// records migration between static subtrees as outstanding (§17.3), so
/// a second generation is where such an edge would first appear.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateBindingTime {
    /// The link writes the value into the leaf's bytes before the object
    /// exists, so it is settled by the time the tree is committed.
    LinkTimeConstant,
    /// The constructor's own recipe fixes the value, and a constructor
    /// field consumes it rather than a push site.
    ConstructorPolicy,
    /// The program reads the value off the target while the leaf runs
    /// and carries no literal for it.
    SpendTimeIntrospection,
    /// The witness supplies the value and the leaf then binds it by an
    /// equation it verifies, so the bytes used are the bytes committed.
    WitnessedThenAuthenticated,
    /// The program rebuilds the value from what it has introspected and
    /// authenticated, rather than carrying a literal for it.
    InProgramReconstruction,
}

impl StateBindingTime {
    /// Whether removing an edge bound this way cuts a cycle.
    #[must_use]
    pub const fn resolves_a_cycle(self) -> bool {
        matches!(
            self,
            Self::SpendTimeIntrospection
                | Self::WitnessedThenAuthenticated
                | Self::InProgramReconstruction
        )
    }
}

// --- Nodes and edges ----------------------------------------------------

/// One node of the maturity link's dependency graph.
///
/// The output is one node and not two. Its merkle root and its output
/// key are one commitment computed in one step from the same inputs, and
/// splitting them would invent an edge between two halves of a single
/// value while saying nothing new about what either depends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateGraphNode {
    /// One typed key of the symbol census.
    Definition(StateLinkSymbol),
    /// One leaf program the constructor commits, by its role.
    Program(StateLeafRole),
    /// The constructor's output: its merkle root and output key.
    Output,
}

/// One dependency, with the time at which its referent is bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateGraphEdge {
    id: usize,
    referrer: StateGraphNode,
    referent: StateGraphNode,
    binding: StateBindingTime,
}

impl StateGraphEdge {
    /// The edge's zero-based index in canonical triple order.
    #[must_use]
    pub const fn id(self) -> usize {
        self.id
    }

    /// The dependent node.
    #[must_use]
    pub const fn referrer(self) -> StateGraphNode {
        self.referrer
    }

    /// The node depended on.
    #[must_use]
    pub const fn referent(self) -> StateGraphNode {
        self.referent
    }

    /// When the referent's value is settled for this reference.
    #[must_use]
    pub const fn binding(self) -> StateBindingTime {
        self.binding
    }
}

/// One offered dependency, before identities are assigned.
///
/// The triple is the identity: two declarations naming one pair of nodes
/// under one binding time are one edge, while the same pair under two
/// binding times is two, because a value read at spend time and the same
/// value written into the bytes are two different dependencies with two
/// different consequences for the cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateGraphEdgeDeclaration {
    /// The dependent node.
    pub referrer: StateGraphNode,
    /// The node depended on.
    pub referent: StateGraphNode,
    /// When the referent's value is settled for this reference.
    pub binding: StateBindingTime,
}

// --- Cut evidence -------------------------------------------------------

/// What one record's own schedule and components offer a cut.
///
/// Read off the record rather than stated beside it, because a cut is
/// validated against what the emitted program actually witnesses and
/// walks. The witness roles are held in the record's declared order and
/// that order carries no meaning: validation asks whether a role occurs,
/// never where it occurs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateCutEvidence {
    witnesses: Vec<StateProgramWitness>,
    components: BTreeSet<StateProgramComponent>,
}

impl StateCutEvidence {
    /// The roles and components one composed record declares.
    #[must_use]
    pub fn from_record(record: &StateAnnouncementProgram) -> Self {
        let mut witnesses: Vec<StateProgramWitness> = Vec::new();
        for (role, _) in record.witness() {
            if !witnesses.contains(role) {
                witnesses.push(*role);
            }
        }
        Self {
            witnesses,
            components: record.components().keys().copied().collect(),
        }
    }

    /// State the available evidence directly.
    #[must_use]
    pub const fn new(
        witnesses: Vec<StateProgramWitness>,
        components: BTreeSet<StateProgramComponent>,
    ) -> Self {
        Self {
            witnesses,
            components,
        }
    }

    /// Every distinct witness role, in the record's declared order.
    #[must_use]
    pub fn witnesses(&self) -> &[StateProgramWitness] {
        &self.witnesses
    }

    /// Every component and adapter the record walks.
    #[must_use]
    pub const fn components(&self) -> &BTreeSet<StateProgramComponent> {
        &self.components
    }
}

/// What a record must show before one binding time may cut a cycle.
///
/// The table is fixed here rather than derived from a record, because it
/// is this link's demand on a record and not a description of one: a
/// record that stopped witnessing the root would satisfy a derived table
/// trivially, and failing it is the whole point.
///
/// Every witness a row names is read by a component the row names. That
/// is what makes a row checkable rather than a list of things that
/// happen to be present. The successor reconstruction reads the root and
/// the successor's key prefix itself, while the nonce that fixes the
/// successor's representation is read by the copy-through component
/// whose derived metadata that reconstruction authenticates, so both
/// components are named; dropping the nonce instead would admit a
/// reconstruction over metadata whose representation nobody witnessed.
///
/// The two static binding times return `None`. They are not cuts, and a
/// row for them would state a demand nothing makes.
#[must_use]
pub const fn state_required_evidence(
    binding: StateBindingTime,
) -> Option<(
    &'static [StateProgramWitness],
    &'static [StateAnnouncementId],
)> {
    use StateAnnouncementId as Component;
    use StateProgramWitness as Role;
    match binding {
        StateBindingTime::LinkTimeConstant | StateBindingTime::ConstructorPolicy => None,
        StateBindingTime::SpendTimeIntrospection => Some((
            &[Role::PredecessorOutputKeyPrefix],
            &[Component::MetadataAuthentication],
        )),
        StateBindingTime::WitnessedThenAuthenticated => Some((
            &[
                Role::StaticSubtreeRoot,
                Role::PredecessorMetadata,
                Role::PredecessorOutputKeyPrefix,
            ],
            &[Component::MetadataAuthentication],
        )),
        StateBindingTime::InProgramReconstruction => Some((
            &[
                Role::StaticSubtreeRoot,
                Role::SuccessorOutputKeyPrefix,
                Role::SuccessorNonce,
            ],
            &[Component::SuccessorReconstruction, Component::CopyThrough],
        )),
    }
}

// --- The residual graph -------------------------------------------------

/// One strongly connected component of the graph left after the cuts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateResidualComponent {
    id: StateGraphNode,
    members: BTreeSet<StateGraphNode>,
    cyclic_edges: BTreeSet<usize>,
}

impl StateResidualComponent {
    /// The smallest member, independent of traversal order.
    #[must_use]
    pub const fn id(&self) -> StateGraphNode {
        self.id
    }

    /// Every member, in the node type's own order.
    #[must_use]
    pub const fn members(&self) -> &BTreeSet<StateGraphNode> {
        &self.members
    }

    /// Every surviving edge with both ends inside this component.
    #[must_use]
    pub const fn cyclic_edges(&self) -> &BTreeSet<usize> {
        &self.cyclic_edges
    }

    /// Whether this component is a cycle, a self-loop included.
    #[must_use]
    pub fn is_cyclic(&self) -> bool {
        self.members.len() > 1 || !self.cyclic_edges.is_empty()
    }
}

/// The admitted graph: every edge, the validated cuts, and the proof.
///
/// Holding the cuts beside the residual is what lets a reader check the
/// proof rather than take it: the edges removed are named, so a claim
/// that the rest is acyclic can be re-derived from the same two lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateAuthenticatedGraph {
    frozen: FrozenStateReferenceGraph,
    nodes: BTreeSet<StateGraphNode>,
    edges: Vec<StateGraphEdge>,
    cuts: BTreeSet<usize>,
    residual: Vec<StateResidualComponent>,
    order: Vec<StateGraphNode>,
}

impl StateAuthenticatedGraph {
    /// The constructor's own frozen reference graph.
    #[must_use]
    pub const fn frozen(&self) -> &FrozenStateReferenceGraph {
        &self.frozen
    }

    /// Every node, in canonical order.
    #[must_use]
    pub const fn nodes(&self) -> &BTreeSet<StateGraphNode> {
        &self.nodes
    }

    /// A node's zero-based canonical index, if it occurs in the graph.
    #[must_use]
    pub fn node_id(&self, node: StateGraphNode) -> Option<usize> {
        self.nodes.iter().position(|member| *member == node)
    }

    /// Every edge, cuts included, in canonical triple order.
    #[must_use]
    pub fn edges(&self) -> &[StateGraphEdge] {
        &self.edges
    }

    /// The validated cut edges, which the residual graph does not carry.
    #[must_use]
    pub const fn cuts(&self) -> &BTreeSet<usize> {
        &self.cuts
    }

    /// Every edge the residual graph carries, in canonical order.
    pub fn residual_edges(&self) -> impl Iterator<Item = &StateGraphEdge> {
        self.edges
            .iter()
            .filter(|edge| !self.cuts.contains(&edge.id))
    }

    /// Every residual component, ordered by its smallest member.
    #[must_use]
    pub fn residual_components(&self) -> &[StateResidualComponent] {
        &self.residual
    }

    /// Every node, dependencies first, in the graph's own canonical
    /// order.
    #[must_use]
    pub fn dependency_order(&self) -> &[StateGraphNode] {
        &self.order
    }
}

// --- Assembly -----------------------------------------------------------

/// The node every literal-root refusal is about.
const STATIC_ROOT: StateGraphNode = StateGraphNode::Definition(StateLinkSymbol::StaticSubtreeRoot);

/// Assemble a graph from offered dependencies and validated cuts.
///
/// The order of the checks is the order in which each becomes
/// answerable. A literal root beneath its own program is refused before
/// any component is computed, because it is not a cycle a better cut
/// would resolve. The projection is compared next, because a graph whose
/// constructor-kind nodes are not the constructor's own is not a graph
/// of this constructor at all. Only then is evidence read, cuts removed
/// and the remainder proved acyclic.
///
/// # Errors
///
/// [`StateLinkRefusal::FrozenGraph`] when the constructor's own
/// declarations cannot be frozen,
/// [`StateLinkRefusal::GraphReferenceLimitExceeded`] when the offered
/// nodes exceed the shared reference bound,
/// [`StateLinkRefusal::LiteralStaticRootBeneathItself`] for a link-time
/// constant root under a program that commits to it,
/// [`StateLinkRefusal::ConstructorProjectionMismatch`] when the
/// constructor-kind nodes are not the frozen graph's own,
/// [`StateLinkRefusal::UnvalidatedCut`] when a binding time's required
/// evidence is not in the record, and
/// [`StateLinkRefusal::ResidualCycle`] when a cycle survives every
/// validated cut.
pub fn assemble_state_graph(
    constructor: &CandidateStateConstructor,
    declarations: impl IntoIterator<Item = StateGraphEdgeDeclaration>,
    evidence: &StateCutEvidence,
) -> Result<StateAuthenticatedGraph, StateLinkRefusal> {
    // Freezing alone cannot report a cycle here, because the seven
    // declarations carry empty dependency lists. The check that does
    // work is the projection equality below, which holds the two graphs
    // to one census of constructor kinds.
    let frozen = FrozenStateReferenceGraph::new(&constructor.reference_declarations())
        .map_err(StateLinkRefusal::FrozenGraph)?;

    let (nodes, edges) = collect_graph(declarations)?;
    require_no_literal_root(&edges)?;
    require_projection_agrees(&frozen, &nodes)?;
    require_validated_cuts(&edges, evidence)?;

    let cuts: BTreeSet<usize> = edges
        .iter()
        .filter(|edge| edge.binding.resolves_a_cycle())
        .map(|edge| edge.id)
        .collect();

    let residual = residual_components(&nodes, &edges, &cuts);
    if let Some(component) = residual.iter().find(|component| component.is_cyclic()) {
        return Err(StateLinkRefusal::ResidualCycle {
            component: component.clone(),
            cuts_removed: cuts,
        });
    }

    let order = dependency_order(&nodes, &edges, &cuts);
    Ok(StateAuthenticatedGraph {
        frozen,
        nodes,
        edges,
        cuts,
        residual,
        order,
    })
}

/// The maturity link's own graph, read off its three typed sources.
///
/// Nothing here is a hand-written adjacency list standing beside the
/// artifacts: the pushed keys come from the resolved census, the leaf
/// programs from the constructor's static subtree, and the cut evidence
/// from the record's declared witness schedule and components. A record
/// that stopped pushing a key, or a constructor that committed a second
/// leaf, changes this graph without anyone remembering to edit it.
///
/// The three cuts are the three things the announcement leaf does at
/// spend time instead of carrying bytes: it reads the program it is
/// executing under, it authenticates a witnessed static root against
/// that program, and it rebuilds output zero's program from the root and
/// the derived metadata. Each is an edge into a cycle, and each is
/// removed only once the record shows the witnesses and components its
/// binding time requires.
///
/// # Errors
///
/// Every refusal of [`assemble_state_graph`], which this calls with the
/// dependencies these sources declare.
pub fn state_graph_from_sources(
    record: &StateAnnouncementProgram,
    constructor: &CandidateStateConstructor,
    resolved: &StateResolvedCensus,
) -> Result<StateAuthenticatedGraph, StateLinkRefusal> {
    use StateBindingTime as Time;
    use StateGraphNode as Node;
    use StateLinkSymbol as Key;

    let announcement = Node::Program(StateLeafRole::Announcement);
    let metadata = Node::Program(StateLeafRole::MetadataCommitment);
    let mut declarations = Vec::new();

    // The leaf carries one literal per pushed key, so each is a
    // link-time constant of the announcement program. The static root is
    // deliberately not among them: no push site resolves it, which is
    // what the census already records by giving it no record site.
    for (symbol, entry) in resolved.entries() {
        if !entry.sites().record_sites().is_empty() {
            declarations.push(declare(
                announcement,
                Node::Definition(*symbol),
                Time::LinkTimeConstant,
            ));
        }
    }

    declarations.extend([
        // The pushed internal key is the policy's admitted key, so the
        // two definitions are one fact under two keys and the census's
        // duplicate is an edge here rather than a coincidence.
        declare(
            Node::Definition(Key::InternalKey),
            Node::Definition(Key::InternalKeyPolicy),
            Time::ConstructorPolicy,
        ),
        declare(announcement, STATIC_ROOT, Time::WitnessedThenAuthenticated),
        declare(announcement, Node::Output, Time::SpendTimeIntrospection),
        declare(announcement, Node::Output, Time::InProgramReconstruction),
        declare(
            STATIC_ROOT,
            Node::Definition(Key::LeafVersion),
            Time::ConstructorPolicy,
        ),
        declare(
            metadata,
            Node::Definition(Key::MetadataSchema),
            Time::ConstructorPolicy,
        ),
        declare(Node::Output, STATIC_ROOT, Time::ConstructorPolicy),
        declare(Node::Output, metadata, Time::ConstructorPolicy),
    ]);

    for key in [
        Key::InternalKeyPolicy,
        Key::BranchSide,
        Key::NonceBudget,
        Key::TargetPolicy,
    ] {
        declarations.push(declare(
            Node::Output,
            Node::Definition(key),
            Time::ConstructorPolicy,
        ));
    }

    for entry in constructor.static_subtree().leaves() {
        declarations.push(declare(
            STATIC_ROOT,
            Node::Program(entry.leaf.role),
            Time::ConstructorPolicy,
        ));
    }

    assemble_state_graph(
        constructor,
        declarations,
        &StateCutEvidence::from_record(record),
    )
}

/// One offered dependency.
const fn declare(
    referrer: StateGraphNode,
    referent: StateGraphNode,
    binding: StateBindingTime,
) -> StateGraphEdgeDeclaration {
    StateGraphEdgeDeclaration {
        referrer,
        referent,
        binding,
    }
}

/// Collect canonical nodes and edges from the offered dependencies.
///
/// # Errors
///
/// [`StateLinkRefusal::GraphReferenceLimitExceeded`] as soon as a
/// distinct node exceeds the shared reference bound.
fn collect_graph(
    declarations: impl IntoIterator<Item = StateGraphEdgeDeclaration>,
) -> Result<(BTreeSet<StateGraphNode>, Vec<StateGraphEdge>), StateLinkRefusal> {
    let mut nodes = BTreeSet::new();
    let mut triples = BTreeSet::new();
    for declaration in declarations {
        insert_node(&mut nodes, declaration.referrer)?;
        insert_node(&mut nodes, declaration.referent)?;
        triples.insert((
            declaration.referrer,
            declaration.referent,
            declaration.binding,
        ));
    }

    let edges = triples
        .into_iter()
        .enumerate()
        .map(|(id, (referrer, referent, binding))| StateGraphEdge {
            id,
            referrer,
            referent,
            binding,
        })
        .collect();

    Ok((nodes, edges))
}

/// Record one node against the shared reference bound.
///
/// # Errors
///
/// [`StateLinkRefusal::GraphReferenceLimitExceeded`] when the distinct
/// nodes exceed [`STATE_REFERENCE_LIMIT`].
fn insert_node(
    nodes: &mut BTreeSet<StateGraphNode>,
    node: StateGraphNode,
) -> Result<(), StateLinkRefusal> {
    nodes.insert(node);
    if nodes.len() > STATE_REFERENCE_LIMIT {
        Err(StateLinkRefusal::GraphReferenceLimitExceeded {
            limit: STATE_REFERENCE_LIMIT,
        })
    } else {
        Ok(())
    }
}

/// Refuse a static root written into a program that commits to it.
///
/// # Errors
///
/// [`StateLinkRefusal::LiteralStaticRootBeneathItself`] naming the
/// program whose bytes would have to contain their own root.
fn require_no_literal_root(edges: &[StateGraphEdge]) -> Result<(), StateLinkRefusal> {
    let literal = edges.iter().find_map(|edge| {
        if let StateGraphNode::Program(program) = edge.referrer
            && edge.referent == STATIC_ROOT
            && edge.binding == StateBindingTime::LinkTimeConstant
        {
            Some(program)
        } else {
            None
        }
    });

    literal.map_or(Ok(()), |program| {
        Err(StateLinkRefusal::LiteralStaticRootBeneathItself { program })
    })
}

/// Require the constructor-kind nodes to be the frozen graph's own.
///
/// # Errors
///
/// [`StateLinkRefusal::ConstructorProjectionMismatch`] carrying both
/// sets, because either side may be the one that moved.
fn require_projection_agrees(
    frozen: &FrozenStateReferenceGraph,
    nodes: &BTreeSet<StateGraphNode>,
) -> Result<(), StateLinkRefusal> {
    let declared: BTreeSet<StateLinkSymbol> = frozen
        .nodes()
        .iter()
        .map(StateLinkSymbol::from_reference)
        .collect();
    let offered: BTreeSet<StateLinkSymbol> = nodes
        .iter()
        .filter_map(|node| match node {
            StateGraphNode::Definition(symbol) if !symbol.is_program_symbol() => Some(*symbol),
            _ => None,
        })
        .collect();

    if declared == offered {
        Ok(())
    } else {
        Err(StateLinkRefusal::ConstructorProjectionMismatch {
            frozen: declared,
            graph: offered,
        })
    }
}

/// Check every edge against whatever its binding time requires.
///
/// Every edge and not only the cutting ones: the table says what a
/// binding time demands, the binding time says whether removing the edge
/// is a cut, and checking each edge against its own row keeps those two
/// statements from drifting into a cut with no stated evidence.
///
/// # Errors
///
/// [`StateLinkRefusal::UnvalidatedCut`] naming the edge and exactly what
/// the record does not show.
fn require_validated_cuts(
    edges: &[StateGraphEdge],
    evidence: &StateCutEvidence,
) -> Result<(), StateLinkRefusal> {
    for edge in edges {
        let Some((witnesses, components)) = state_required_evidence(edge.binding) else {
            continue;
        };

        let missing_witnesses: Vec<StateProgramWitness> = witnesses
            .iter()
            .filter(|role| !evidence.witnesses.contains(role))
            .copied()
            .collect();
        let missing_components: BTreeSet<StateAnnouncementId> = components
            .iter()
            .filter(|id| {
                !evidence
                    .components
                    .contains(&StateProgramComponent::Semantic(**id))
            })
            .copied()
            .collect();

        if !missing_witnesses.is_empty() || !missing_components.is_empty() {
            return Err(StateLinkRefusal::UnvalidatedCut {
                referrer: edge.referrer,
                referent: edge.referent,
                binding: edge.binding,
                missing_witnesses,
                missing_components,
            });
        }
    }

    Ok(())
}

/// The strongly connected components of the graph left after the cuts.
///
/// Determinism is not inherited from the algorithm: node indices are
/// assigned in canonical node order, surviving edges are inserted in
/// canonical edge order, members are normalized into sets, and the
/// components are sorted by their least member.
fn residual_components(
    nodes: &BTreeSet<StateGraphNode>,
    edges: &[StateGraphEdge],
    cuts: &BTreeSet<usize>,
) -> Vec<StateResidualComponent> {
    let mut graph = DiGraph::<StateGraphNode, usize>::new();
    let indices: BTreeMap<_, _> = nodes
        .iter()
        .map(|node| (*node, graph.add_node(*node)))
        .collect();
    for edge in edges.iter().filter(|edge| !cuts.contains(&edge.id)) {
        if let (Some(referrer), Some(referent)) =
            (indices.get(&edge.referrer), indices.get(&edge.referent))
        {
            graph.add_edge(*referrer, *referent, edge.id);
        }
    }

    let mut components = Vec::new();
    for connected in tarjan_scc(&graph) {
        let members: BTreeSet<_> = connected
            .into_iter()
            .filter_map(|index| graph.node_weight(index).copied())
            .collect();
        if let Some(id) = members.first().copied() {
            let cyclic_edges = edges
                .iter()
                .filter(|edge| !cuts.contains(&edge.id))
                .filter(|edge| members.contains(&edge.referrer) && members.contains(&edge.referent))
                .map(|edge| edge.id)
                .collect();
            components.push(StateResidualComponent {
                id,
                members,
                cyclic_edges,
            });
        }
    }

    components.sort_by_key(StateResidualComponent::id);
    components
}

/// Every node with its dependencies before it, canonically.
///
/// Among the nodes all of whose surviving referents are already placed,
/// the smallest: the sequence is then a property of the graph rather
/// than of the order the declarations arrived in. A finite graph with no
/// cycle always offers such a node, so the search stops only when every
/// node is placed; stopping rather than looping keeps that reasoning
/// local instead of resting on a panic that the proof above has already
/// excluded.
fn dependency_order(
    nodes: &BTreeSet<StateGraphNode>,
    edges: &[StateGraphEdge],
    cuts: &BTreeSet<usize>,
) -> Vec<StateGraphNode> {
    let mut order = Vec::with_capacity(nodes.len());
    let mut placed: BTreeSet<StateGraphNode> = BTreeSet::new();

    while placed.len() < nodes.len() {
        let next = nodes.iter().copied().find(|&node| {
            !placed.contains(&node)
                && edges
                    .iter()
                    .filter(|edge| !cuts.contains(&edge.id) && edge.referrer == node)
                    .all(|edge| placed.contains(&edge.referent))
        });
        let Some(node) = next else { break };
        placed.insert(node);
        order.push(node);
    }

    order
}
