//! The maturity link's authenticated graph, asserted against the real
//! sources.
//!
//! # The table is written out and the graph is compared with it
//!
//! The real graph's nodes, its nineteen edges with their binding times,
//! its three cuts and its dependency order are written down here in full
//! and compared with what the record, the constructor and the resolved
//! census produce. Writing them out is the point: a test that asked the
//! graph for its own edge count and agreed with the answer would pass on
//! the day an edge disappeared.
//!
//! # The negative cases are the ones that close the rows
//!
//! A success suite would leave the two central failures untested. The
//! component with two edge-disjoint cycles is built by hand, because no
//! real source produces one and the rule under test is exactly that one
//! cut is not enough for a component that has two cycles to cut. The
//! literal static root is built by hand for the same reason: the real
//! link never offers it, and the refusal exists so that a later link
//! cannot.

use std::collections::BTreeSet;

use tapscript::{
    StateAnnouncementId, StateAnnouncementWitness, StateLeafRole, StateProgramComponent,
    StateProgramWitness,
};

use crate::tests::{record, resolved_census, state_constructor};
use crate::{
    STATE_REFERENCE_LIMIT, StateAuthenticatedGraph, StateBindingTime as Time, StateCutEvidence,
    StateGraphEdgeDeclaration, StateGraphNode as Node, StateLinkRefusal, StateLinkSymbol as Key,
    assemble_state_graph, state_graph_from_sources, state_required_evidence,
};

/// Three nodes standing for one component's shape.
///
/// Support roles, because what is under test is the shape of a component
/// and not the meaning of any leaf: the analysis reads binding times and
/// endpoints, and naming these after real leaves would suggest the
/// refusal depended on which leaf it was.
const FIRST: Node = Node::Program(StateLeafRole::Support(0));
const SECOND: Node = Node::Program(StateLeafRole::Support(1));
const THIRD: Node = Node::Program(StateLeafRole::Support(2));

/// The announcement leaf's node.
const ANNOUNCEMENT: Node = Node::Program(StateLeafRole::Announcement);

/// The metadata leaf's node.
const METADATA: Node = Node::Program(StateLeafRole::MetadataCommitment);

/// The static root's node.
const ROOT: Node = Node::Definition(Key::StaticSubtreeRoot);

/// The real graph over the demonstration sources.
fn graph() -> StateAuthenticatedGraph {
    state_graph_from_sources(&record(), &state_constructor(), &resolved_census())
        .expect("the demonstration sources assemble")
}

/// The record's own witness schedule and components.
fn evidence() -> StateCutEvidence {
    StateCutEvidence::from_record(&record())
}

/// One offered dependency.
const fn declare(referrer: Node, referent: Node, binding: Time) -> StateGraphEdgeDeclaration {
    StateGraphEdgeDeclaration {
        referrer,
        referent,
        binding,
    }
}

/// The seven constructor-kind nodes, which every graph must carry.
///
/// The projection is an equality, so a hand-built case that omitted them
/// would refuse for the wrong reason and prove nothing about the rule it
/// was written for.
fn projection_edges() -> Vec<StateGraphEdgeDeclaration> {
    [
        Key::MetadataSchema,
        Key::StaticSubtreeRoot,
        Key::LeafVersion,
        Key::InternalKeyPolicy,
        Key::BranchSide,
        Key::NonceBudget,
        Key::TargetPolicy,
    ]
    .into_iter()
    .map(|key| declare(Node::Output, Node::Definition(key), Time::ConstructorPolicy))
    .collect()
}

/// One hand-built component over the three support nodes.
fn component_with(
    edges: [(Node, Node, Time); 4],
) -> Result<StateAuthenticatedGraph, StateLinkRefusal> {
    let mut declarations = projection_edges();
    declarations.extend(
        edges
            .into_iter()
            .map(|(referrer, referent, binding)| declare(referrer, referent, binding)),
    );
    assemble_state_graph(&state_constructor(), declarations, &evidence())
}

/// The residual-cycle refusal, for the closed root's census.
///
/// A real refusal rather than a fabricated one: a residual component is
/// built by the analysis and has no constructor outside it, and a
/// component invented for the census would say nothing about the one the
/// linker actually reports.
pub(super) fn residual_cycle_refusal() -> StateLinkRefusal {
    component_with([
        (FIRST, SECOND, Time::ConstructorPolicy),
        (SECOND, FIRST, Time::SpendTimeIntrospection),
        (FIRST, THIRD, Time::ConstructorPolicy),
        (THIRD, FIRST, Time::ConstructorPolicy),
    ])
    .expect_err("the second cycle survives the one cut")
}

/// The real graph's edges, in canonical triple order.
fn expected_edges() -> Vec<(Node, Node, Time)> {
    vec![
        (
            Node::Definition(Key::InternalKey),
            Node::Definition(Key::InternalKeyPolicy),
            Time::ConstructorPolicy,
        ),
        (
            ROOT,
            Node::Definition(Key::LeafVersion),
            Time::ConstructorPolicy,
        ),
        (ROOT, ANNOUNCEMENT, Time::ConstructorPolicy),
        (
            METADATA,
            Node::Definition(Key::MetadataSchema),
            Time::ConstructorPolicy,
        ),
        (
            ANNOUNCEMENT,
            Node::Definition(Key::StateAsset),
            Time::LinkTimeConstant,
        ),
        (
            ANNOUNCEMENT,
            Node::Definition(Key::StateAmount),
            Time::LinkTimeConstant,
        ),
        (
            ANNOUNCEMENT,
            Node::Definition(Key::InternalKey),
            Time::LinkTimeConstant,
        ),
        (
            ANNOUNCEMENT,
            Node::Definition(Key::MaturityLeadMin),
            Time::LinkTimeConstant,
        ),
        (
            ANNOUNCEMENT,
            Node::Definition(Key::MaturityLeadMax),
            Time::LinkTimeConstant,
        ),
        (
            ANNOUNCEMENT,
            Node::Definition(Key::CommittedOperatorKey),
            Time::LinkTimeConstant,
        ),
        (ANNOUNCEMENT, ROOT, Time::WitnessedThenAuthenticated),
        (ANNOUNCEMENT, Node::Output, Time::SpendTimeIntrospection),
        (ANNOUNCEMENT, Node::Output, Time::InProgramReconstruction),
        (Node::Output, ROOT, Time::ConstructorPolicy),
        (
            Node::Output,
            Node::Definition(Key::InternalKeyPolicy),
            Time::ConstructorPolicy,
        ),
        (
            Node::Output,
            Node::Definition(Key::BranchSide),
            Time::ConstructorPolicy,
        ),
        (
            Node::Output,
            Node::Definition(Key::NonceBudget),
            Time::ConstructorPolicy,
        ),
        (
            Node::Output,
            Node::Definition(Key::TargetPolicy),
            Time::ConstructorPolicy,
        ),
        (Node::Output, METADATA, Time::ConstructorPolicy),
    ]
}

/// The real graph's dependency order, dependencies first.
fn expected_order() -> Vec<Node> {
    vec![
        Node::Definition(Key::StateAsset),
        Node::Definition(Key::StateAmount),
        Node::Definition(Key::MaturityLeadMin),
        Node::Definition(Key::MaturityLeadMax),
        Node::Definition(Key::CommittedOperatorKey),
        Node::Definition(Key::MetadataSchema),
        Node::Definition(Key::LeafVersion),
        Node::Definition(Key::InternalKeyPolicy),
        Node::Definition(Key::InternalKey),
        Node::Definition(Key::BranchSide),
        Node::Definition(Key::NonceBudget),
        Node::Definition(Key::TargetPolicy),
        METADATA,
        ANNOUNCEMENT,
        ROOT,
        Node::Output,
    ]
}

// (a) The real graph: the node set, the edge table, the three cuts, the
// acyclic residual and the canonical order.
#[test]
fn the_real_graph_holds_every_key_both_leaves_and_the_output() {
    let graph = graph();
    let expected: BTreeSet<Node> = Key::ALL
        .iter()
        .copied()
        .map(Node::Definition)
        .chain([METADATA, ANNOUNCEMENT, Node::Output])
        .collect();

    assert_eq!(*graph.nodes(), expected);
    assert_eq!(graph.nodes().len(), 16);
    assert_eq!(graph.node_id(Node::Definition(Key::StateAsset)), Some(0));
    assert_eq!(graph.node_id(METADATA), Some(13));
    assert_eq!(graph.node_id(Node::Output), Some(15));
}

#[test]
fn the_real_graph_is_the_written_edge_table() {
    let graph = graph();
    let observed: Vec<(Node, Node, Time)> = graph
        .edges()
        .iter()
        .map(|edge| (edge.referrer(), edge.referent(), edge.binding()))
        .collect();

    assert_eq!(observed, expected_edges());
    assert_eq!(graph.edges().len(), 19);
    for (index, edge) in graph.edges().iter().enumerate() {
        assert_eq!(edge.id(), index);
    }
}

#[test]
fn the_three_cuts_are_removed_and_the_residual_is_acyclic() {
    let graph = graph();

    assert_eq!(*graph.cuts(), BTreeSet::from([10, 11, 12]));
    for &cut in graph.cuts() {
        let binding = graph.edges()[cut].binding();
        assert!(binding.resolves_a_cycle());
        assert!(state_required_evidence(binding).is_some());
    }

    assert_eq!(graph.residual_edges().count(), 16);
    assert!(
        graph
            .residual_edges()
            .all(|edge| !edge.binding().resolves_a_cycle())
    );
    assert_eq!(graph.residual_components().len(), 16);
    assert!(
        graph
            .residual_components()
            .iter()
            .all(|component| !component.is_cyclic())
    );
}

#[test]
fn the_dependency_order_places_every_referent_before_its_referrer() {
    let graph = graph();
    assert_eq!(graph.dependency_order(), expected_order().as_slice());

    let position = |node: Node| {
        graph
            .dependency_order()
            .iter()
            .position(|placed| *placed == node)
    };
    for edge in graph.residual_edges() {
        assert!(position(edge.referent()) < position(edge.referrer()));
    }
}

// (b) The frozen constructor graph as a projection that agrees.
#[test]
fn the_frozen_projection_agrees_with_the_constructor_kind_nodes() {
    let graph = graph();

    assert_eq!(graph.frozen().components().len(), 7);
    assert_eq!(graph.frozen().edges(), []);
    assert!(
        graph
            .frozen()
            .components()
            .iter()
            .all(|component| !component.is_cyclic())
    );

    let projected: BTreeSet<Node> = graph
        .frozen()
        .nodes()
        .iter()
        .map(|reference| Node::Definition(Key::from_reference(reference)))
        .collect();
    let constructor_kinds: BTreeSet<Node> = graph
        .nodes()
        .iter()
        .copied()
        .filter(|node| matches!(node, Node::Definition(key) if !key.is_program_symbol()))
        .collect();

    assert_eq!(projected.len(), 7);
    assert_eq!(projected, constructor_kinds);
}

// (c) One cut per component is not the rule; every cycle is cut.
#[test]
fn one_cut_leaves_the_other_edge_disjoint_cycle() {
    let refusal = residual_cycle_refusal();
    let StateLinkRefusal::ResidualCycle {
        component,
        cuts_removed,
    } = &refusal
    else {
        panic!("a surviving cycle is a residual-cycle refusal: {refusal:?}");
    };

    assert!(component.is_cyclic());
    assert_eq!(*component.members(), BTreeSet::from([FIRST, THIRD]));
    assert_eq!(component.cyclic_edges().len(), 2);
    assert_eq!(component.id(), FIRST);
    assert_eq!(cuts_removed.len(), 1);
}

#[test]
fn a_cut_on_each_cycle_assembles() {
    let graph = component_with([
        (FIRST, SECOND, Time::ConstructorPolicy),
        (SECOND, FIRST, Time::SpendTimeIntrospection),
        (FIRST, THIRD, Time::ConstructorPolicy),
        (THIRD, FIRST, Time::SpendTimeIntrospection),
    ])
    .expect("two cycles cut twice leave nothing cyclic");

    assert_eq!(graph.cuts().len(), 2);
    assert!(
        graph
            .residual_components()
            .iter()
            .all(|component| !component.is_cyclic())
    );
}

#[test]
fn one_cut_on_an_edge_both_cycles_share_assembles() {
    let graph = component_with([
        (FIRST, SECOND, Time::SpendTimeIntrospection),
        (SECOND, FIRST, Time::ConstructorPolicy),
        (SECOND, THIRD, Time::ConstructorPolicy),
        (THIRD, FIRST, Time::ConstructorPolicy),
    ])
    .expect("one cut on the shared edge cuts both cycles");

    assert_eq!(graph.cuts().len(), 1);
    assert!(
        graph
            .residual_components()
            .iter()
            .all(|component| !component.is_cyclic())
    );
}

// (d) The static root: never a literal beneath itself, and never a cut
// without the witnesses its authentication reads.
#[test]
fn a_link_time_static_root_under_its_own_program_is_refused() {
    let mut declarations = projection_edges();
    declarations.push(declare(ANNOUNCEMENT, ROOT, Time::LinkTimeConstant));
    // A cycle in the same offering, so the refusal below is the literal
    // one rather than whichever check happened to run first.
    declarations.extend([
        declare(FIRST, THIRD, Time::ConstructorPolicy),
        declare(THIRD, FIRST, Time::ConstructorPolicy),
    ]);

    let refusal = assemble_state_graph(&state_constructor(), declarations, &evidence())
        .expect_err("a leaf cannot carry the root of the tree committing to it");

    assert_eq!(
        refusal,
        StateLinkRefusal::LiteralStaticRootBeneathItself {
            program: StateLeafRole::Announcement,
        }
    );
}

#[test]
fn a_witnessed_root_without_the_root_witness_is_not_a_cut() {
    let complete = evidence();
    let without_root: Vec<StateProgramWitness> = complete
        .witnesses()
        .iter()
        .copied()
        .filter(|role| *role != StateProgramWitness::StaticSubtreeRoot)
        .collect();
    let reduced = StateCutEvidence::new(without_root, complete.components().clone());

    let mut declarations = projection_edges();
    declarations.push(declare(
        ANNOUNCEMENT,
        ROOT,
        Time::WitnessedThenAuthenticated,
    ));

    let refusal = assemble_state_graph(&state_constructor(), declarations, &reduced)
        .expect_err("a root nobody witnesses authenticates nothing");
    let StateLinkRefusal::UnvalidatedCut {
        referrer,
        referent,
        binding,
        missing_witnesses,
        missing_components,
    } = &refusal
    else {
        panic!("missing evidence is an unvalidated cut: {refusal:?}");
    };

    assert_eq!(*referrer, ANNOUNCEMENT);
    assert_eq!(*referent, ROOT);
    assert_eq!(*binding, Time::WitnessedThenAuthenticated);
    assert_eq!(
        missing_witnesses.as_slice(),
        [StateProgramWitness::StaticSubtreeRoot]
    );
    assert!(missing_components.is_empty());
}

#[test]
fn the_record_shows_exactly_what_the_cut_rows_require() {
    let record = record();
    let evidence = StateCutEvidence::from_record(&record);

    let declared: Vec<StateProgramWitness> =
        record.witness().iter().map(|(role, _)| *role).collect();
    assert_eq!(evidence.witnesses(), declared.as_slice());
    assert_eq!(evidence.witnesses().len(), 7);
    assert_eq!(
        *evidence.components(),
        record.components().keys().copied().collect::<BTreeSet<_>>()
    );

    for binding in [
        Time::SpendTimeIntrospection,
        Time::WitnessedThenAuthenticated,
        Time::InProgramReconstruction,
    ] {
        let (witnesses, components) =
            state_required_evidence(binding).expect("a cutting binding time states its evidence");
        for role in witnesses {
            assert!(evidence.witnesses().contains(role));
        }
        for id in components {
            assert!(
                evidence
                    .components()
                    .contains(&StateProgramComponent::Semantic(*id))
            );
        }
    }
}

#[test]
fn exactly_the_cutting_binding_times_state_required_evidence() {
    for binding in [
        Time::LinkTimeConstant,
        Time::ConstructorPolicy,
        Time::SpendTimeIntrospection,
        Time::WitnessedThenAuthenticated,
        Time::InProgramReconstruction,
    ] {
        assert_eq!(
            binding.resolves_a_cycle(),
            state_required_evidence(binding).is_some()
        );
    }
}

#[test]
fn the_reconstruction_row_names_the_component_that_reads_the_nonce() {
    assert!(
        StateAnnouncementId::CopyThrough
            .witness()
            .contains(&StateAnnouncementWitness::SuccessorNonce)
    );
    assert!(
        !StateAnnouncementId::SuccessorReconstruction
            .witness()
            .contains(&StateAnnouncementWitness::SuccessorNonce)
    );
    assert!(
        StateAnnouncementId::SuccessorReconstruction
            .witness()
            .contains(&StateAnnouncementWitness::DerivedSuccessorMetadata)
    );

    let (_, components) =
        state_required_evidence(Time::InProgramReconstruction).expect("the row states components");
    assert!(components.contains(&StateAnnouncementId::CopyThrough));
    assert!(components.contains(&StateAnnouncementId::SuccessorReconstruction));
}

// (e) The shared reference bound, applied to the whole graph.
fn star_of(programs: u32) -> Result<StateAuthenticatedGraph, StateLinkRefusal> {
    let mut declarations = projection_edges();
    declarations.extend((0..programs).map(|index| {
        declare(
            Node::Output,
            Node::Program(StateLeafRole::Support(index)),
            Time::ConstructorPolicy,
        )
    }));
    assemble_state_graph(&state_constructor(), declarations, &evidence())
}

#[test]
fn the_shared_reference_bound_admits_sixty_four_nodes_and_no_more() {
    assert_eq!(STATE_REFERENCE_LIMIT, 64);

    // The output, the seven constructor kinds and the support programs.
    let graph = star_of(56).expect("sixty-four distinct nodes are admitted");
    assert_eq!(graph.nodes().len(), STATE_REFERENCE_LIMIT);

    assert_eq!(
        star_of(57).expect_err("sixty-five are not"),
        StateLinkRefusal::GraphReferenceLimitExceeded {
            limit: STATE_REFERENCE_LIMIT,
        }
    );
}

// (f) The projection is an equality, and both sides travel.
#[test]
fn a_constructor_kind_missing_from_the_graph_is_a_projection_mismatch() {
    let declarations: Vec<StateGraphEdgeDeclaration> = projection_edges()
        .into_iter()
        .filter(|declaration| declaration.referent != Node::Definition(Key::TargetPolicy))
        .collect();

    let refusal = assemble_state_graph(&state_constructor(), declarations, &evidence())
        .expect_err("the constructor declares a kind this graph does not carry");
    let StateLinkRefusal::ConstructorProjectionMismatch { frozen, graph } = &refusal else {
        panic!("a disagreeing projection is a projection mismatch: {refusal:?}");
    };

    assert_eq!(frozen.len(), 7);
    assert_eq!(graph.len(), 6);
    assert!(frozen.contains(&Key::TargetPolicy));
    assert!(!graph.contains(&Key::TargetPolicy));
}

// (g) The declaration order is not a property of the graph.
#[test]
fn the_declaration_order_does_not_reach_the_graph() {
    let graph = graph();
    let mut declarations: Vec<StateGraphEdgeDeclaration> = graph
        .edges()
        .iter()
        .map(|edge| declare(edge.referrer(), edge.referent(), edge.binding()))
        .collect();
    declarations.reverse();

    let rebuilt = assemble_state_graph(&state_constructor(), declarations, &evidence())
        .expect("the same dependencies in the opposite order");

    assert_eq!(rebuilt.edges(), graph.edges());
    assert_eq!(rebuilt.cuts(), graph.cuts());
    assert_eq!(rebuilt.dependency_order(), graph.dependency_order());
    assert_eq!(rebuilt, graph);
}
