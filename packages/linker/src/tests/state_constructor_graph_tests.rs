use std::collections::BTreeSet;

use realization::{Cycle, Maturity, ProtocolAmount, StateMetadata};
use tapscript::{
    CandidateStateConstructor, STATE_NUMS_KEY, StackItem, StateConstructorReference as Reference,
    StateCurveCapability, StateInternalKeyPolicy, StateLeafRole, StateNonceBudget,
    StateReferenceCensus, StateReferenceDeclaration, StateStaticLeaf, StateStaticNode,
    StateStaticSubtree, StateTweakOutcome, TapscriptInstruction, TapscriptProgram,
};
use target_elements::LeafVersion;

use crate::{FrozenStateReferenceGraph, STATE_REFERENCE_LIMIT, StateReferenceGraphRefusal};

use super::reviewed_target;

fn declaration(value: u32, dependencies: &[u32]) -> StateReferenceDeclaration {
    StateReferenceDeclaration {
        reference: Reference::MetadataSchema(value),
        dependencies: dependencies
            .iter()
            .copied()
            .map(Reference::MetadataSchema)
            .collect(),
    }
}

fn freeze(declarations: &[StateReferenceDeclaration]) -> FrozenStateReferenceGraph {
    FrozenStateReferenceGraph::new(declarations).unwrap()
}

fn assert_cycles(graph: &FrozenStateReferenceGraph, expected: &[&[u32]]) {
    let components: Vec<_> = graph.cyclic_components().cloned().collect();
    let members: Vec<BTreeSet<_>> = expected
        .iter()
        .map(|values| {
            values
                .iter()
                .copied()
                .map(Reference::MetadataSchema)
                .collect()
        })
        .collect();
    assert_eq!(components.len(), expected.len());
    for (component, expected_members) in components.iter().zip(members) {
        assert_eq!(component.members(), &expected_members);
        assert_eq!(Some(&component.id()), expected_members.first());
        assert!(!component.cyclic_edges().is_empty());
    }
    assert_eq!(
        graph.require_acyclic_without_selecting_cut(),
        Err(StateReferenceGraphRefusal::ConstructorReferenceCycleUnresolved { components })
    );
}

#[test]
fn every_permutation_preserves_graph_components_and_identities() {
    let declarations = [
        declaration(9, &[4]),
        declaration(4, &[9, 1]),
        declaration(1, &[]),
    ];
    let expected = freeze(&declarations);
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let permuted = order.map(|index| declarations[index].clone());
        let actual = freeze(&permuted);
        assert_eq!(actual, expected);
        assert_eq!(actual.components(), expected.components());
        for (index, reference) in expected.nodes().iter().enumerate() {
            assert_eq!(actual.node_id(*reference), Some(index));
        }
        for (index, edge) in actual.edges().iter().enumerate() {
            assert_eq!(edge.id(), index);
        }
        assert_eq!(
            actual
                .components()
                .iter()
                .map(crate::StateReferenceComponent::id)
                .collect::<Vec<_>>(),
            vec![Reference::MetadataSchema(1), Reference::MetadataSchema(4)]
        );
    }
}

#[test]
fn cyclic_pair_identity_is_its_smaller_reference() {
    for declarations in [
        [declaration(9, &[4]), declaration(4, &[9])],
        [declaration(4, &[9]), declaration(9, &[4])],
    ] {
        let graph = freeze(&declarations);
        assert_eq!(graph.components()[0].id(), Reference::MetadataSchema(4));
        assert_cycles(&graph, &[&[4, 9]]);
    }
}

#[test]
fn dependency_order_duplicates_and_split_declarations_do_not_change_graph() {
    let expected = freeze(&[declaration(4, &[1, 2]), declaration(2, &[1])]);
    let actual = freeze(&[
        declaration(2, &[1, 1]),
        declaration(4, &[2, 1, 2]),
        declaration(4, &[1]),
        declaration(1, &[]),
    ]);
    assert_eq!(actual, expected);
    assert_eq!(actual.nodes().len(), 3);
    assert_eq!(actual.edges().len(), 3);
    assert_eq!(actual.require_acyclic_without_selecting_cut(), Ok(()));
}

#[test]
fn edges_point_from_dependent_to_dependency_and_include_undeclared_nodes() {
    let graph = freeze(&[declaration(9, &[4, 1])]);
    let endpoints: Vec<_> = graph
        .edges()
        .iter()
        .map(|edge| (edge.id(), edge.referrer(), edge.referent()))
        .collect();
    assert_eq!(
        endpoints,
        vec![
            (
                0,
                Reference::MetadataSchema(9),
                Reference::MetadataSchema(1)
            ),
            (
                1,
                Reference::MetadataSchema(9),
                Reference::MetadataSchema(4)
            ),
        ]
    );
    assert_eq!(graph.node_id(Reference::MetadataSchema(1)), Some(0));
    assert_eq!(graph.node_id(Reference::MetadataSchema(4)), Some(1));
    assert_eq!(graph.node_id(Reference::MetadataSchema(9)), Some(2));
    assert_eq!(graph.node_id(Reference::MetadataSchema(0)), None);
    assert!(
        graph
            .components()
            .iter()
            .all(|component| !component.is_cyclic())
    );
}

#[test]
fn singleton_self_loop_is_unresolved() {
    let graph = freeze(&[declaration(7, &[7])]);
    assert_eq!(graph.components().len(), 1);
    assert_eq!(graph.components()[0].cyclic_edges(), &BTreeSet::from([0]));
    assert_cycles(&graph, &[&[7]]);
}

#[test]
fn three_cycle_excludes_incoming_and_outgoing_acyclic_nodes() {
    let graph = freeze(&[
        declaration(0, &[2]),
        declaration(2, &[3]),
        declaration(3, &[4]),
        declaration(4, &[2, 8]),
        declaration(8, &[]),
        declaration(9, &[]),
    ]);
    assert_eq!(graph.components().len(), 4);
    assert_cycles(&graph, &[&[2, 3, 4]]);
    let cycle = graph.cyclic_components().next().unwrap();
    assert_eq!(cycle.cyclic_edges().len(), 3);
    for edge in graph.edges() {
        assert_eq!(
            cycle.cyclic_edges().contains(&edge.id()),
            cycle.members().contains(&edge.referrer())
                && cycle.members().contains(&edge.referent())
        );
    }
}

#[test]
fn refusal_reports_every_cycle_in_canonical_order() {
    let graph = freeze(&[
        declaration(9, &[9]),
        declaration(5, &[3]),
        declaration(3, &[5]),
        declaration(1, &[1, 3]),
    ]);
    assert_cycles(&graph, &[&[1], &[3, 5], &[9]]);
}

#[test]
fn empty_graph_is_acyclic() {
    let graph = freeze(&[]);
    assert!(graph.nodes().is_empty());
    assert_eq!(graph.edges(), []);
    assert_eq!(graph.components(), []);
    assert_eq!(graph.cyclic_components().count(), 0);
    assert_eq!(graph.require_acyclic_without_selecting_cut(), Ok(()));
}

#[test]
fn exact_reference_bound_is_admitted_despite_repeated_declarations() {
    let limit = u32::try_from(STATE_REFERENCE_LIMIT).unwrap();
    let declarations: Vec<_> = (0..limit).map(|value| declaration(value, &[])).collect();
    let mut repeated = declarations.clone();
    repeated.extend(declarations.clone());
    let graph = freeze(&repeated);
    assert_eq!(graph, freeze(&declarations));
    assert_eq!(graph.nodes().len(), STATE_REFERENCE_LIMIT);
    assert_eq!(graph.components().len(), STATE_REFERENCE_LIMIT);
    assert_eq!(graph.require_acyclic_without_selecting_cut(), Ok(()));
}

#[test]
fn excessive_declared_references_are_refused() {
    let limit = u32::try_from(STATE_REFERENCE_LIMIT).unwrap();
    let declarations: Vec<_> = (0..=limit).map(|value| declaration(value, &[])).collect();
    assert_eq!(
        FrozenStateReferenceGraph::new(&declarations),
        Err(StateReferenceGraphRefusal::ResourceBoundExceeded {
            limit: STATE_REFERENCE_LIMIT
        })
    );
}

#[test]
fn dependency_only_references_also_count_toward_the_bound() {
    let limit = u32::try_from(STATE_REFERENCE_LIMIT).unwrap();
    let dependencies: Vec<_> = (1..limit).collect();
    assert_eq!(
        freeze(&[declaration(0, &dependencies)]).nodes().len(),
        STATE_REFERENCE_LIMIT
    );
    let excessive: Vec<_> = (1..=limit).collect();
    assert_eq!(
        FrozenStateReferenceGraph::new(&[declaration(0, &excessive)]),
        Err(StateReferenceGraphRefusal::ResourceBoundExceeded {
            limit: STATE_REFERENCE_LIMIT
        })
    );
}

struct ScriptedCurve;

impl StateCurveCapability for ScriptedCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        assert_eq!(key, &STATE_NUMS_KEY);
        true
    }

    fn output_key(&self, key: &[u8; 32], _: &[u8; 32]) -> StateTweakOutcome {
        assert_eq!(key, &STATE_NUMS_KEY);
        StateTweakOutcome::OutputKey {
            key: [0x42; 32],
            parity: true,
        }
    }
}

fn candidate() -> CandidateStateConstructor {
    let target = reviewed_target();
    let metadata = StateMetadata {
        omega: ProtocolAmount::new(1).unwrap(),
        y_l: ProtocolAmount::new(2).unwrap(),
        y_t: ProtocolAmount::new(3).unwrap(),
        q: ProtocolAmount::new(4).unwrap(),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    };
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Push(
        StackItem::script_number(&target, 1).unwrap(),
    )])
    .unwrap();
    let subtree = StateStaticSubtree::new(
        &target,
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role: StateLeafRole::Announcement,
                program,
                version: LeafVersion::TAPSCRIPT.get(),
            },
        }),
    )
    .unwrap();
    CandidateStateConstructor::derive(
        &target,
        &metadata,
        &subtree,
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &ScriptedCurve).unwrap(),
        StateNonceBudget::new(4096).unwrap(),
        &ScriptedCurve,
    )
    .unwrap()
}

#[test]
fn real_constructor_has_seven_acyclic_singleton_components() {
    let declarations = candidate().reference_declarations();
    assert_eq!(declarations.len(), 7);
    assert!(
        declarations
            .iter()
            .all(|entry| entry.dependencies.is_empty())
    );
    let graph = freeze(&declarations);
    assert_eq!(graph.nodes().len(), 7);
    assert_eq!(graph.edges(), []);
    assert_eq!(graph.components().len(), 7);
    assert_eq!(graph.cyclic_components().count(), 0);
    for component in graph.components() {
        assert_eq!(component.members(), &BTreeSet::from([component.id()]));
    }
    assert_eq!(graph.require_acyclic_without_selecting_cut(), Ok(()));
    let census = StateReferenceCensus::new(&declarations).unwrap();
    let components: Vec<Vec<_>> = graph
        .components()
        .iter()
        .map(|component| component.members().iter().copied().collect())
        .collect();
    assert_eq!(components, census.components);
}
