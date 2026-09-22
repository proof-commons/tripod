//! The candidate linked maturity bundle, asserted against the real
//! artifacts.
//!
//! # The figures are read, never restated
//!
//! Every count below — the site count, both roots, the obligation
//! counts, the recipe count — is taken from the record, the census, the
//! committed tree or the closure themselves, reached through public
//! entry points. The one figure written out is the moved-site count,
//! because that is a claim about how much of the leaf a second
//! deployment changes and nothing else in the tree states it.
//!
//! # Each artifact is compared against the stage that made it
//!
//! A bundle carrying a *summary* of a stage would pass a test that only
//! asked whether a field were present. So each carried artifact is
//! compared against the value its own stage produces when run
//! independently over the same inputs, which is the one comparison a
//! summary cannot survive.
//!
//! # Two deployments, because one cannot show a substitution
//!
//! The demonstration deployment resolves every key to the value the
//! record was already composed against, so its substitution is the
//! identity on bytes: its linked leaf is the composed program, its
//! linked static subtree is the one the supplied constructor already
//! committed, and even the static root does not move under the
//! application. That is a property of those fixtures rather than of the
//! link, and a suite that only used them could not tell an application
//! over the linked subtree from one over the pre-link subtree.
//!
//! So every statement whose content is that something *moved* is put to
//! the second deployment, whose asset, operator key and both lead bounds
//! differ from the record's fixtures while the internal key and the
//! amount do not: the same census moves nine sites and leaves six alone.
//! It is also what makes the continuity refusal reachable, because two
//! deployments produce two linked leaves, two static subtrees and two
//! roots — exactly the pair the no-migration rule refuses to reconcile.

use std::collections::BTreeSet;

use architecture::{ARCHITECTURE, AssetId};
use realization::{Cycle, Maturity, ProtocolAmount, StateMetadata, announce_maturity};
use tapscript::upstream::{
    MaturityAnnouncementRepresentationProjection, StateSingletonDeclaration,
};
use tapscript::{
    CandidateStateConstructor, STATE_NUMS_KEY, StateConstructorRefusal, StateCurveCapability,
    StateLeafRole, StateNonceBudget, StateStaticNode, StateStaticSubtree, StateTweakOutcome,
};

use crate::tests::state_relocate_tests::{second_bridge, second_resolved_census, second_singleton};
use crate::tests::{
    ScriptedCurve, bridge, declaration, linked_bundle, linked_leaf, linked_taptree, record,
    resolved_census, reviewed_target, singleton, state_constructor, state_metadata,
};
use crate::{
    CandidateLinkedMaturityBundle, LinkRefusal, LinkedArtifactStatus, StateConsumerCensus,
    StateLinkObligation, StateLinkRefusal, StateLinkSources, StateLinkSymbol, StateRelocation,
    StateResolvedEntry, close_state_carriers, collect_state_definitions, link_state_candidate,
    measure_state_resources, resolve_state_census, state_application_fixed_point,
    state_applied_references_agree, state_bundle_continuity, state_graph_from_sources,
};

// --- Fixtures ----------------------------------------------------------

/// A curve capability that refuses every tweak.
///
/// The point validation still answers, so what this reaches is the
/// search exhausting its budget rather than an internal key refused
/// before the search starts — which is the branch the fixture exists
/// for, because a budget is exhausted only by candidates that were
/// tried and refused.
struct RefusingCurve;

impl StateCurveCapability for RefusingCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        key == &STATE_NUMS_KEY
    }

    fn output_key(&self, _: &[u8; 32], _: &[u8; 32]) -> StateTweakOutcome {
        StateTweakOutcome::TweakAboveGroupOrder
    }
}

/// A second semantic metadata, distinct from the fixture's in one field.
///
/// One field rather than all six, so what the tests observe follows from
/// the metadata changing at all rather than from how much of it changed.
fn other_metadata() -> StateMetadata {
    StateMetadata {
        omega: ProtocolAmount::new(7).expect("the fixture quantities are in domain"),
        y_l: ProtocolAmount::new(2).expect("the fixture quantities are in domain"),
        y_t: ProtocolAmount::new(3).expect("the fixture quantities are in domain"),
        q: ProtocolAmount::new(4).expect("the fixture quantities are in domain"),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    }
}

/// The second deployment's candidate linked bundle.
fn second_bundle() -> CandidateLinkedMaturityBundle {
    link_state_candidate(
        &reviewed_target(),
        &StateLinkSources::new(
            &record(),
            &second_bridge(),
            &state_constructor(),
            &second_singleton(),
            &declaration(),
            &state_metadata(),
            &ScriptedCurve,
        ),
    )
    .expect("the second deployment's sources link")
}

/// The constructor the link itself applied.
fn applied(bundle: &CandidateLinkedMaturityBundle) -> &CandidateStateConstructor {
    bundle
        .instances()
        .first()
        .expect("the link retains its own application as the first instance")
        .constructor()
}

// --- (a) The real bundle -----------------------------------------------

// (a) Every carried artifact equal to the one its own stage produces when
// run independently over the same inputs. The applied constructor is
// read back out of the bundle, because it is what the later stages ran
// against and a re-derivation here would be a second artifact.
#[test]
fn every_carried_artifact_equals_what_its_own_stage_produces() {
    let target = reviewed_target();
    let bundle = linked_bundle();
    let constructor = applied(&bundle);

    let definitions = collect_state_definitions(
        &target,
        &bridge(),
        constructor,
        &singleton(),
        &declaration(),
        tapscript::StateWitnessSchedule::WholeMetadata,
    )
    .expect("the demonstration sources define every key");
    let resolved = resolve_state_census(
        &definitions,
        &StateConsumerCensus::from_sources(&record(), constructor),
    )
    .expect("the demonstration census resolves");
    assert_eq!(bundle.resolved(), &resolved);

    let graph = state_graph_from_sources(&record(), constructor, &resolved)
        .expect("the demonstration graph assembles");
    assert_eq!(bundle.graph(), &graph);
    assert_eq!(bundle.graph().cuts(), graph.cuts());
    assert_eq!(bundle.graph().dependency_order(), graph.dependency_order());

    let leaf = linked_leaf();
    assert_eq!(
        bundle.program(StateLeafRole::Announcement),
        Some(&leaf),
        "the carried program is the leaf the relocation stage produces",
    );
    assert_eq!(bundle.relocations(), Some(leaf.relocations()));

    let taptree = linked_taptree(&leaf);
    assert_eq!(bundle.taptree(), &taptree);
    assert_eq!(bundle.taptree().static_root(), taptree.static_root());
    assert_eq!(bundle.taptree().merkle_root(), taptree.merkle_root());
    assert_eq!(bundle.control_recipes(), taptree.tree().recipes());

    let resources =
        measure_state_resources(&target, &record(), &leaf).expect("the linked leaf measures");
    assert_eq!(bundle.resources(), &resources);

    let closure = close_state_carriers(&record(), &bridge(), &leaf, &taptree)
        .expect("the demonstration closure closes");
    assert_eq!(bundle.carrier_closure(), &closure);
    assert_eq!(bundle.carrier_closure().rows(), closure.rows());
    assert_eq!(bundle.abi_obligations(), closure.obligations());
}

// (a) One program, at the one role the admitted recipe carries, and every
// delegating accessor reading through to the artifact that holds it.
#[test]
fn one_announcement_program_and_every_delegating_accessor_reads_through() {
    let bundle = linked_bundle();
    let constructor = applied(&bundle);

    assert_eq!(bundle.programs().len(), 1);
    assert_eq!(
        bundle.programs().keys().copied().collect::<Vec<_>>(),
        vec![StateLeafRole::Announcement],
    );
    assert_eq!(bundle.program(StateLeafRole::MetadataCommitment), None);

    assert_eq!(bundle.plan(), bridge().plan());
    assert_eq!(bundle.contract(), bridge().revision());
    assert_eq!(bundle.deployment(), &bridge());
    assert_eq!(bundle.static_subtree(), bundle.taptree().subtree());
    assert_eq!(bundle.static_subtree(), constructor.static_subtree());
    assert_eq!(bundle.control_recipes(), bundle.taptree().tree().recipes());
    assert_eq!(
        bundle.lifecycle().release_complete(),
        bridge().plan().lifecycle().release_complete(),
    );

    // The definition projection is the resolved census's own, so there
    // is no second census to drift from it.
    let projected: Vec<_> = bundle.definitions().collect();
    let embedded: Vec<_> = bundle
        .resolved()
        .entries()
        .values()
        .map(StateResolvedEntry::definition)
        .collect();
    assert_eq!(projected, embedded);
    assert_eq!(projected.len(), 13);
}

// (a) The status is read and never written, and the outstanding set is
// the five, structurally non-empty.
#[test]
fn the_status_is_a_prototype_and_the_outstanding_set_is_the_five() {
    let bundle = linked_bundle();

    assert_eq!(bundle.status(), LinkedArtifactStatus::Prototype);
    assert_ne!(
        bundle.status(),
        LinkedArtifactStatus::CandidateOperationProven
    );
    assert_ne!(bundle.status(), LinkedArtifactStatus::ProductionApproved);

    let expected = [
        StateLinkObligation::CurrentStateValidationUndischarged,
        StateLinkObligation::SuccessorNonceSearchUndischarged,
        StateLinkObligation::FinalizationAndWitnessPopulationUndischarged,
        StateLinkObligation::AbiUnsupplied,
        StateLinkObligation::ModelScopeRelationsUnenforced,
    ];
    let outstanding = bundle.obligations();
    assert_eq!(
        outstanding.obligations().copied().collect::<BTreeSet<_>>(),
        expected.into_iter().collect::<BTreeSet<_>>(),
    );
    assert_eq!(outstanding.count().get(), expected.len());
    assert_eq!(outstanding.obligations().count(), expected.len());
    for obligation in expected {
        assert!(outstanding.holds(obligation));
    }

    // The ABI obligation is not a count of the contract. The contract is
    // the closure's, and this set says only that nobody has answered it.
    assert_eq!(bundle.abi_obligations().len(), 38);
}

// (a) No content digest and no minted identity: every fixed-width value
// the bundle exposes is one the constructor computed over bytes a spend
// runs, and no accessor returns anything else of that shape.
#[test]
fn no_identity_is_minted_anywhere_in_the_bundle() {
    let bundle = linked_bundle();
    let constructor = applied(&bundle);

    assert_eq!(
        bundle.taptree().static_root(),
        constructor.static_subtree().root(),
    );
    assert_eq!(bundle.taptree().merkle_root(), constructor.merkle_root());
    assert_eq!(
        bundle.taptree().metadata_hash(),
        &constructor
            .control_recipe(StateLeafRole::MetadataCommitment)
            .expect("the metadata leaf has a recipe")
            .executing_leaf_hash,
    );

    // The one remaining fixed-width value is the output key, and it is
    // the capability's answer rather than this crate's: the link
    // computes no hash and no tweak of its own.
    assert_eq!(constructor.output_key(), &[0x42; 32]);
}

// --- (b) The fixed point -----------------------------------------------

// (b) The census re-collected over the applied constructor agrees with
// the first on every key but one, and differs exactly at the static
// root. Put to the second deployment, because the demonstration one
// resolves every key to the record's own fixture and therefore moves
// nothing at all — not even the root, since its linked leaf is the
// composed program and its subtree the supplied constructor's.
#[test]
fn the_applied_census_moves_the_static_root_and_nothing_else() {
    assert_eq!(linked_bundle().resolved(), &resolved_census());

    let second = second_bundle();
    let before = second_resolved_census();
    let after = second.resolved();

    for symbol in StateLinkSymbol::ALL {
        let left = before.entries().get(&symbol);
        let right = after.entries().get(&symbol);
        if symbol == StateLinkSymbol::StaticSubtreeRoot {
            assert_ne!(left, right, "the static root is what the application moves");
        } else {
            assert_eq!(
                left, right,
                "{symbol:?} must not move under the application"
            );
        }
    }

    assert_eq!(state_application_fixed_point(&before, after), Ok(()));
    assert_eq!(before.program_keys(), after.program_keys());
    assert_eq!(before.push_site_count(), after.push_site_count());
}

// (b) The applied constructor's subtree is the linked one, and not the
// pre-link one the census was first collected against.
#[test]
fn the_applied_subtree_is_the_linked_one_in_both_deployments() {
    let first = linked_bundle();
    let second = second_bundle();
    let supplied = state_constructor();

    for bundle in [&first, &second] {
        let constructor = applied(bundle);
        assert_eq!(
            constructor.static_subtree().root(),
            bundle.static_subtree().root(),
        );
        assert_eq!(
            state_applied_references_agree(constructor, &supplied, bundle.static_subtree()),
            Ok(()),
        );
    }

    // The demonstration deployment substitutes the record's own values
    // back into it, so its linked subtree is the one the supplied
    // constructor already committed; the second deployment's values
    // differ, and there the application is visibly over other bytes.
    assert_eq!(
        applied(&first).static_subtree().root(),
        supplied.static_subtree().root(),
    );
    assert_ne!(
        applied(&second).static_subtree().root(),
        supplied.static_subtree().root(),
        "the application is over the linked subtree, not the pre-link one",
    );
}

// (b) The census half of the fixed point refuses a moved pushed key, and
// names it.
#[test]
fn a_pushed_key_that_moved_under_the_application_is_refused() {
    assert_eq!(
        state_application_fixed_point(&resolved_census(), &second_resolved_census()),
        Err(StateLinkRefusal::CensusMovedUnderApplication {
            symbol: StateLinkSymbol::StateAsset,
        }),
    );
}

// (b) The reference half refuses a static root that is not the subtree
// the application was handed. The subtree is the second deployment's,
// because the demonstration one is the subtree the supplied constructor
// already commits and would agree.
#[test]
fn an_applied_reference_that_is_not_the_handed_subtree_is_refused() {
    let second = second_bundle();
    let supplied = state_constructor();

    assert_eq!(
        state_applied_references_agree(&supplied, &supplied, second.static_subtree()),
        Err(StateLinkRefusal::AppliedReferenceDisagreement {
            symbol: StateLinkSymbol::StaticSubtreeRoot,
        }),
    );
}

// --- (c) The second deployment -----------------------------------------

// (c) A second deployment links to a bundle whose leaf differs at exactly
// the sites its own values differ at, and whose closure, resources and
// obligations are the first's.
#[test]
fn the_second_deployment_moves_nine_sites_and_nothing_else() {
    let first = linked_bundle();
    let second = second_bundle();

    let composed = record();
    let pristine = composed.program().instructions();
    let leaf = second
        .program(StateLeafRole::Announcement)
        .expect("the second deployment links its announcement leaf");
    let linked = leaf.program().instructions();

    assert_eq!(pristine.len(), linked.len());
    let moved: BTreeSet<usize> = pristine
        .iter()
        .zip(linked)
        .enumerate()
        .filter(|(_, (before, after))| before != after)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(moved.len(), 9);

    let census = second
        .relocations()
        .expect("the second deployment carries its relocations");
    assert_eq!(census.sites().len(), census.len());
    assert_eq!(
        moved,
        census
            .relocations()
            .iter()
            .filter(|relocation| relocation.pre_value() != relocation.linked_value())
            .map(StateRelocation::site)
            .collect::<BTreeSet<_>>(),
    );

    assert_ne!(
        first.static_subtree().root(),
        second.static_subtree().root(),
        "two deployments commit two different linked leaves",
    );
    assert_eq!(second.carrier_closure(), first.carrier_closure());
    assert_eq!(second.resources(), first.resources());
    assert_eq!(second.obligations(), first.obligations());
    assert_eq!(second.evidence(), first.evidence());
    assert_eq!(second.status(), LinkedArtifactStatus::Prototype);
}

// --- (d) Continuity ----------------------------------------------------

// (d) The no-migration rule: each bundle against itself is accepted, and
// one against the other is refused with both roots.
#[test]
fn two_bundles_static_subtrees_do_not_migrate() {
    let first = linked_bundle();
    let second = second_bundle();

    assert_eq!(
        state_bundle_continuity(first.static_subtree(), first.static_subtree()),
        Ok(()),
    );
    assert_eq!(
        state_bundle_continuity(second.static_subtree(), second.static_subtree()),
        Ok(()),
    );
    assert_eq!(
        state_bundle_continuity(first.static_subtree(), second.static_subtree()),
        Err(StateLinkRefusal::StaticSubtreeDiscontinuity {
            predecessor: *first.static_subtree().root(),
            successor: *second.static_subtree().root(),
        }),
    );
}

// --- (e) Application and retention -------------------------------------

// (e) Applying the bundle's own metadata reproduces the retained
// instance, and a different metadata is a different constructor over the
// same subtree.
#[test]
fn an_application_is_metadata_specific_over_one_fixed_subtree() {
    let target = reviewed_target();
    let mut bundle = linked_bundle();

    let again = bundle
        .apply_constructor(&target, &state_metadata(), &ScriptedCurve)
        .expect("the bundle's own metadata applies");
    assert_eq!(&again, applied(&bundle));
    assert_eq!(again.encoded_metadata().semantic, state_metadata());

    let other = bundle
        .retain(&target, &other_metadata(), &ScriptedCurve)
        .expect("a second metadata is retained as a second instance")
        .clone();
    assert_eq!(other.metadata().semantic, other_metadata());
    assert_eq!(
        other.constructor().encoded_metadata(),
        other.metadata(),
        "a retained instance carries the metadata its constructor is of",
    );

    // The same subtree, a different metadata leaf, a different outer
    // root. The output key is the scripted capability's constant, so it
    // is deliberately not asserted to differ.
    assert_eq!(
        other.constructor().static_subtree(),
        bundle.static_subtree()
    );
    assert_ne!(other.constructor().metadata_bytes(), again.metadata_bytes());
    assert_ne!(other.constructor().leaf_program(), again.leaf_program());
    assert_ne!(other.constructor().merkle_root(), again.merkle_root());

    assert_eq!(bundle.instances().len(), 2);
    assert_eq!(bundle.instances().last(), Some(&other));
}

// (e) One semantic metadata has one instance.
#[test]
fn retaining_one_metadata_twice_is_refused() {
    let target = reviewed_target();
    let mut bundle = linked_bundle();

    let held = *applied(&bundle).encoded_metadata();
    assert_eq!(
        bundle
            .retain(&target, &state_metadata(), &ScriptedCurve)
            .err(),
        Some(StateLinkRefusal::InstanceAlreadyRetained { metadata: held }),
    );
    assert_eq!(bundle.instances().len(), 1);
}

// Retaining the transition's own successor keeps both semantic sides over one exact linked subtree. A second deployment is the perturbed arm: its different program must fail both continuity checks even though it can construct the same successor metadata.
#[test]
fn retained_maturity_transition_preserves_both_continuities_and_refuses_deployment_migration() {
    let target = reviewed_target();
    let mut bundle = linked_bundle();
    let predecessor = bundle.instances()[0].clone();
    let input = state_metadata();
    assert_eq!(predecessor.metadata().semantic, input);
    let bounds = bridge().lead_bounds().bounds();
    let (request, _) = bounds
        .window(input.cycle)
        .expect("the fixture cycle has a lead window");
    let output = announce_maturity(&input, request, bounds)
        .expect("the earliest cycle in the fixture window is admissible");
    let successor = bundle
        .retain(&target, &output, &ScriptedCurve)
        .expect("the transition's successor is retained beside its predecessor")
        .clone();
    assert_eq!(bundle.instances().len(), 2);
    assert_eq!(
        bundle.instances(),
        &[predecessor.clone(), successor.clone()]
    );
    assert_eq!(successor.metadata().semantic, output);
    assert_eq!(
        successor.metadata(),
        successor.constructor().encoded_metadata()
    );
    let before = predecessor.metadata().semantic;
    let after = successor.metadata().semantic;
    assert_eq!(
        (before.omega, before.y_l, before.y_t, before.q, before.cycle),
        (after.omega, after.y_l, after.y_t, after.q, after.cycle),
    );
    assert_ne!(before.maturity, after.maturity);
    let before = predecessor.constructor();
    let after = successor.constructor();
    for (left, right) in [(before, after), (after, before)] {
        assert_eq!(
            state_bundle_continuity(left.static_subtree(), right.static_subtree()),
            Ok(())
        );
        assert_eq!(left.continuity(right), Ok(()));
    }

    let other = second_bundle();
    let migrated = other
        .apply_constructor(&target, &output, &ScriptedCurve)
        .expect("the second deployment also constructs the successor metadata");
    assert_eq!(
        migrated.encoded_metadata().semantic,
        after.encoded_metadata().semantic
    );
    assert_ne!(
        before.static_subtree().root(),
        other.static_subtree().root()
    );
    assert_eq!(
        state_bundle_continuity(before.static_subtree(), other.static_subtree()),
        Err(StateLinkRefusal::StaticSubtreeDiscontinuity {
            predecessor: *before.static_subtree().root(),
            successor: *other.static_subtree().root(),
        }),
    );
    assert_eq!(
        before.continuity(&migrated),
        Err(StateConstructorRefusal::ConflictingLeaf)
    );
}

// Rebuilding the linked leaf with its original identity preserves the whole descriptor; changing only that identity preserves the root but fails the linker's equality. The refusal carries roots, not the descriptor difference, so a validated report has to state separately which equality failed: the same refusal variant also reports program migration, and its payload cannot describe the retained-tree distinction.
#[test]
fn equal_static_roots_do_not_establish_retained_descriptor_continuity() {
    let target = reviewed_target();
    let bundle = linked_bundle();
    let original = &bundle.static_subtree().leaves()[0];
    assert_eq!(original.identity, 0);
    let with_identity = |identity| {
        StateStaticSubtree::new(
            &target,
            Some(StateStaticNode::Leaf {
                identity,
                leaf: original.leaf.clone(),
            }),
        )
        .expect("the unchanged linked announcement leaf is a complete static subtree")
    };
    let honest = with_identity(original.identity);
    assert_eq!(&honest, bundle.static_subtree());
    assert_eq!(
        state_bundle_continuity(bundle.static_subtree(), &honest),
        Ok(())
    );

    let rebuilt = with_identity(1);
    assert_ne!(&rebuilt, bundle.static_subtree());
    assert_eq!(rebuilt.root(), bundle.static_subtree().root());
    let refusal = state_bundle_continuity(bundle.static_subtree(), &rebuilt);
    assert_eq!(
        refusal,
        Err(StateLinkRefusal::StaticSubtreeDiscontinuity {
            predecessor: *bundle.static_subtree().root(),
            successor: *rebuilt.root(),
        })
    );
    let Err(StateLinkRefusal::StaticSubtreeDiscontinuity {
        predecessor,
        successor,
    }) = refusal
    else {
        panic!("a changed descriptor must be refused even with an equal root");
    };
    assert_eq!(predecessor, successor);
    let constructed = CandidateStateConstructor::derive(
        &target,
        &state_metadata(),
        &rebuilt,
        bundle.policy().internal_key(),
        bundle.policy().budget(),
        &ScriptedCurve,
    )
    .expect("the identity change leaves the committed bytes admissible");
    assert_eq!(constructed.static_subtree(), &rebuilt);
    assert_eq!(applied(&bundle).continuity(&constructed), Ok(()));
    assert_eq!(constructed.continuity(applied(&bundle)), Ok(()));
}

// (e) An application whose every candidate nonce is refused exhausts its
// budget, and the construction's own refusal travels unchanged.
#[test]
fn an_application_under_an_exhausted_budget_is_refused() {
    let target = reviewed_target();
    let mut bundle = linked_bundle();

    assert_eq!(
        bundle.apply_constructor(&target, &state_metadata(), &RefusingCurve),
        Err(StateLinkRefusal::ConstructorApplication(
            StateConstructorRefusal::RepresentationSearchExhausted,
        )),
    );
    assert_eq!(
        bundle
            .retain(&target, &other_metadata(), &RefusingCurve)
            .err(),
        Some(StateLinkRefusal::ConstructorApplication(
            StateConstructorRefusal::RepresentationSearchExhausted,
        )),
    );
    assert_eq!(bundle.instances().len(), 1);
}

// --- (f) Stage refusals through the entry ------------------------------

// (f) A constructor whose subtree commits some other program is refused
// before any census is collected against it, and the refusal reaches the
// caller inside the shared root rather than renamed by it. The other
// program is the second deployment's linked leaf, which differs from the
// composed record at nine sites; the demonstration deployment's own
// linked leaf is the composed program and would be accepted, rightly.
#[test]
fn a_constructor_committing_another_program_is_refused_by_the_entry() {
    let second = second_bundle();
    let over_another_leaf = applied(&second).clone();

    let refused = link_state_candidate(
        &reviewed_target(),
        &StateLinkSources::new(
            &record(),
            &bridge(),
            &over_another_leaf,
            &singleton(),
            &declaration(),
            &state_metadata(),
            &ScriptedCurve,
        ),
    );

    assert_eq!(
        refused.err(),
        Some(LinkRefusal::StateLink(
            StateLinkRefusal::SuppliedConstructorCommitsAnotherProgram,
        )),
    );
}

// (f) A census refusal reaches the caller the same way, wrapped and
// unchanged. The relocation, graph, resource and carrier stages have no
// representative here: the entry collects its own census and substitutes
// its own leaf, so no caller can hand in the artifact any of those
// stages would refuse, and each is exercised against its own entry in
// its own module.
#[test]
fn a_census_refusal_reaches_the_caller_wrapped_and_unchanged() {
    let other = ARCHITECTURE
        .asset(AssetId::Pace)
        .expect("the pace authority asset is declared");
    let mismatched =
        StateSingletonDeclaration::from_architecture_asset(other).expect("it declares a singleton");

    let refused = link_state_candidate(
        &reviewed_target(),
        &StateLinkSources::new(
            &record(),
            &bridge(),
            &state_constructor(),
            &singleton(),
            &mismatched,
            &state_metadata(),
            &ScriptedCurve,
        ),
    );

    assert_eq!(
        refused.err(),
        Some(LinkRefusal::StateLink(
            StateLinkRefusal::SingletonDeclarationMismatch,
        )),
    );
}

// (f) The tree stage's refusals are the shared root's, and none is
// reachable through this entry. The reason is recomputed rather than
// asserted: one leaf is declared, at the announcement role and the
// reviewed version; the subtree bound against is the applied
// constructor's own; and a one-leaf complete tree is one level deep
// against a cap that cannot be less than one.
#[test]
fn the_committed_tree_is_one_leaf_deep_under_the_deployment_cap() {
    let bundle = linked_bundle();

    assert_eq!(bundle.control_recipes().len(), 1);
    assert_eq!(
        bundle.control_recipes().keys().copied().collect::<Vec<_>>(),
        vec![StateLeafRole::Announcement],
    );
    assert_eq!(bundle.taptree().cost().static_cost(), 0);
    assert_eq!(bundle.taptree().cost().complete_cost(), 2);
    assert_eq!(bundle.taptree().depths().len(), 2);
    assert_eq!(
        bundle.taptree().depths().get(&StateLeafRole::Announcement),
        Some(&1),
    );
    assert_eq!(
        bundle
            .taptree()
            .depths()
            .get(&StateLeafRole::MetadataCommitment),
        Some(&1),
    );
}

// --- The plan's own statements -----------------------------------------

// The open premises are the plan's own, and every premise the closure's
// rows carry is among them.
#[test]
fn the_open_premises_are_the_plans_and_cover_the_closures_rows() {
    let bundle = linked_bundle();

    let from_plan: BTreeSet<_> = bundle
        .plan()
        .representations()
        .flat_map(MaturityAnnouncementRepresentationProjection::relations)
        .flat_map(|requirement| requirement.external_evidence.iter().cloned())
        .collect();
    assert_eq!(bundle.evidence(), &from_plan);

    for row in bundle.carrier_closure().rows() {
        for open in row.external_requirements() {
            assert!(
                bundle.evidence().contains(open),
                "a premise a row carries is outside the plan's own set",
            );
        }
    }
}

// The policy is the supplied constructor's own, read from the
// declarations that are where it states these values.
#[test]
fn the_policy_is_the_supplied_constructors_own() {
    let bundle = linked_bundle();
    let supplied = state_constructor();
    let constructor = applied(&bundle);

    assert_eq!(bundle.policy().generation(), supplied.generation());
    assert_eq!(bundle.policy().generation(), constructor.generation());
    assert_eq!(bundle.policy().target_policy(), bridge().revision());
    assert_eq!(
        bundle.policy().leaf_version(),
        reviewed_target().definition().leaf_version(),
    );
    assert_eq!(bundle.policy().internal_key().key(), &STATE_NUMS_KEY);
    assert_eq!(bundle.policy().budget(), StateNonceBudget::default());
}

// --- (g) The closed root's bundle half ---------------------------------

/// Which test reaches one of the bundle's refusals, or why nothing can.
pub(super) fn bundle_reachability(refusal: &StateLinkRefusal) -> &'static str {
    match refusal {
        StateLinkRefusal::SuppliedConstructorCommitsAnotherProgram => {
            "a_constructor_committing_another_program_is_refused_by_the_entry"
        }
        StateLinkRefusal::ConstructorApplication(_) => {
            "an_application_under_an_exhausted_budget_is_refused"
        }
        StateLinkRefusal::AppliedReferenceDisagreement { .. } => {
            "an_applied_reference_that_is_not_the_handed_subtree_is_refused"
        }
        StateLinkRefusal::CensusMovedUnderApplication { .. } => {
            "a_pushed_key_that_moved_under_the_application_is_refused"
        }
        StateLinkRefusal::StaticSubtreeDiscontinuity { .. } => {
            "two_bundles_static_subtrees_do_not_migrate"
        }
        StateLinkRefusal::InstanceAlreadyRetained { .. } => {
            "retaining_one_metadata_twice_is_refused"
        }
        StateLinkRefusal::ConstructorPolicyIncomplete { .. } => {
            "unreachable: the constructor's declarations are a fixed array with one entry per \
             reference kind, built from its own fields, so no policy kind can be absent"
        }
        _ => "accounted for elsewhere in the closed root",
    }
}

// (g) Every bundle refusal is reached by a named test or declared
// unreachable with the reason a reader can recompute.
#[test]
fn every_bundle_refusal_is_reached_or_declared() {
    let metadata = *state_constructor().encoded_metadata();

    let refusals = [
        StateLinkRefusal::SuppliedConstructorCommitsAnotherProgram,
        StateLinkRefusal::ConstructorApplication(
            StateConstructorRefusal::RepresentationSearchExhausted,
        ),
        StateLinkRefusal::AppliedReferenceDisagreement {
            symbol: StateLinkSymbol::StaticSubtreeRoot,
        },
        StateLinkRefusal::CensusMovedUnderApplication {
            symbol: StateLinkSymbol::StateAsset,
        },
        StateLinkRefusal::StaticSubtreeDiscontinuity {
            predecessor: [0x01; 32],
            successor: [0x02; 32],
        },
        StateLinkRefusal::InstanceAlreadyRetained { metadata },
        StateLinkRefusal::ConstructorPolicyIncomplete {
            missing: StateLinkSymbol::NonceBudget,
        },
    ];

    let accounts: BTreeSet<&str> = refusals.iter().map(bundle_reachability).collect();
    assert_eq!(refusals.len(), 7);
    assert_eq!(accounts.len(), 7);
    assert_eq!(
        refusals
            .iter()
            .filter(|refusal| bundle_reachability(refusal).starts_with("unreachable"))
            .count(),
        1,
    );
}
