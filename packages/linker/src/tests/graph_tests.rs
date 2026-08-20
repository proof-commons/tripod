//! Pass two and the cycle policy (§14.3, §14.4).

use std::collections::BTreeSet;

use tapscript::BundleSymbol;

use crate::graph::{apply_cycle_policy, resolve_references};
use crate::tests::relocatable_bundle;
use crate::{ReferenceClass, ReferenceNode, SelfCommitmentStrategy, leaf_symbol};

#[test]
fn the_graph_is_the_same_graph_whatever_the_strategy_is() {
    // The dependency is a fact about the emitted design; only its
    // classification is a caller's decision. So the node set and the
    // edge endpoints must be identical under all three strategies, and
    // exactly one edge's class may differ.
    let bundle = relocatable_bundle();
    let stated = resolve_references(
        &bundle,
        SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
    )
    .expect("references resolve");
    let unstated =
        resolve_references(&bundle, SelfCommitmentStrategy::NotStated).expect("references resolve");

    assert_eq!(stated.nodes(), unstated.nodes());
    assert_eq!(stated.edges().len(), unstated.edges().len());

    let differing: Vec<_> = stated
        .edges()
        .iter()
        .filter(|(id, edge)| {
            unstated.edges().get(id).map(|other| other.class()) != Some(edge.class())
        })
        .map(|(_, edge)| (edge.referrer(), edge.referent()))
        .collect();
    assert_eq!(
        differing,
        vec![(
            ReferenceNode::Symbol(BundleSymbol::AshConstructorProgram),
            ReferenceNode::Constructor
        )]
    );
}

#[test]
fn resolution_is_deterministic_across_repeated_runs() {
    // §14.3 freezes the graph, and a frozen graph that renumbered its
    // edges between runs would make a refusal naming an edge useless.
    // Two independent resolutions of one bundle must be equal values.
    let bundle = relocatable_bundle();
    let first = resolve_references(&bundle, SelfCommitmentStrategy::NotStated).expect("resolves");
    let second = resolve_references(&bundle, SelfCommitmentStrategy::NotStated).expect("resolves");

    assert_eq!(first, second);
}

#[test]
fn every_leaf_symbol_refers_to_the_ash_program_and_the_constructor_binds_every_leaf() {
    // The two edge families that close the loop, checked as census
    // rather than as an example: all twelve leaves push the ASH
    // program, and the constructor binds all twelve leaves.
    let bundle = relocatable_bundle();
    let graph = resolve_references(&bundle, SelfCommitmentStrategy::NotStated).expect("resolves");

    let leaves: BTreeSet<ReferenceNode> = bundle
        .constructor()
        .leaves()
        .keys()
        .map(|leaf| ReferenceNode::Symbol(leaf_symbol(*leaf)))
        .collect();
    assert_eq!(leaves.len(), 12);

    let to_ash: BTreeSet<ReferenceNode> = graph
        .referrers(ReferenceNode::Symbol(BundleSymbol::AshConstructorProgram))
        .map(|edge| edge.referrer())
        .collect();
    assert_eq!(to_ash, leaves);

    for leaf in &leaves {
        assert!(
            graph
                .referrers(*leaf)
                .any(|edge| edge.referrer() == ReferenceNode::Constructor),
            "the constructor does not bind {leaf:?}",
        );
    }
}

#[test]
fn an_unclassified_cyclic_edge_is_rejected_and_a_cut_one_is_not() {
    // §14.4's two halves. With no strategy the loop's own edge is
    // unclassified and the component is refused; with a strategy the
    // same component is accepted, because one explicit authenticated
    // cut is what resolving a cycle means. The ordinary static and
    // deployment edges inside the loop are unchanged either way, which
    // is why requiring every edge to be cycle-resolving would be the
    // wrong reading — it would reject every cycle without exception.
    let bundle = relocatable_bundle();

    let unstated =
        resolve_references(&bundle, SelfCommitmentStrategy::NotStated).expect("resolves");
    assert!(apply_cycle_policy(&unstated).is_err());

    let stated = resolve_references(
        &bundle,
        SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
    )
    .expect("resolves");
    assert!(apply_cycle_policy(&stated).is_ok());

    let introspecting = resolve_references(&bundle, SelfCommitmentStrategy::IdentityIntrospection)
        .expect("resolves");
    assert!(apply_cycle_policy(&introspecting).is_ok());
}

#[test]
fn only_the_cycle_resolving_classes_cut_a_cycle() {
    // The classification vocabulary is the package contract's, and the
    // subset that resolves a cycle is exactly the three that say how.
    // A static constant and a deployment relocation are references, not
    // resolutions, and an unsupported dependency is the absence of one.
    for class in [
        ReferenceClass::IdentityIntrospection,
        ReferenceClass::InProgramConstructorReconstruction,
        ReferenceClass::AuthenticatedWitnessedRootContinuity,
    ] {
        assert!(class.resolves_a_cycle(), "{class:?} should cut a cycle");
    }
    for class in [
        ReferenceClass::StaticLinkTimeConstant,
        ReferenceClass::DeploymentRelocation,
        ReferenceClass::UnsupportedDependency,
    ] {
        assert!(
            !class.resolves_a_cycle(),
            "{class:?} should not cut a cycle"
        );
    }
}

#[test]
fn components_outside_the_cycle_are_single_nodes() {
    // A graph reporting spurious components would make the cycle policy
    // fire on things that are not cycles. Every component but the one
    // is a single node with no edge inside it.
    let bundle = relocatable_bundle();
    let graph = resolve_references(&bundle, SelfCommitmentStrategy::NotStated).expect("resolves");

    let cyclic = graph.cycles().count();
    assert_eq!(cyclic, 1);
    for component in graph.components() {
        if component.is_cyclic() {
            continue;
        }
        assert_eq!(component.members().len(), 1);
        assert!(component.cyclic_edges().is_empty());
    }
}
