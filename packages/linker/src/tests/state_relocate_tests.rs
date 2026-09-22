//! The maturity leaf's relocations, asserted against the real record.
//!
//! # The figures are read, never restated
//!
//! Every count below is taken from the composed announcement record and
//! the resolved census themselves, reached through public entry points.
//! The one table written out is the attribution of sites to components,
//! because that is a claim about where the leaf pushes each value and
//! nothing else in the tree states it.
//!
//! # Where these fixtures live
//!
//! The second deployment's sources and the demonstration singleton are
//! declared here rather than in the shared fixture module: the census
//! bite's originals stay where that bite put them, and unifying the
//! maturity fixtures is the wave's closure bite rather than this one's.
//!
//! # A second deployment is what makes a substitution visible
//!
//! A link whose resolved values are the ones the record was composed
//! against is the identity on bytes, so it cannot tell a substitution
//! from a copy. The second deployment differs in the asset, the operator
//! key and both lead bounds, and agrees in the internal key and the
//! amount, which the constructor's key policy and the architecture's
//! declaration fix for every deployment — so the same census both moves
//! nine sites and leaves six alone, and a link that moved everything
//! would fail here rather than pass.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ARCHITECTURE, AssetId};
use tapscript::upstream::{AnnouncementLeadBounds, Cycle, StateSingletonDeclaration};
use tapscript::{
    EstablishedOperatorProfile, StackItem, StateAnnouncementId, StateAnnouncementProgram,
    StateLeafRole, StateOperatorPatternId, StatePatternId, StateProgramComponent,
    TapscriptInstruction, TapscriptProgram, selected_operator_profile,
};
use target_elements::EncodingClass;

use crate::operator_deployment::OperatorDeploymentBinding;
use crate::state_deployment::{
    StateLeadBoundOrigin, StateLeadBounds, StateLinkDeploymentParameters,
};
use crate::tests::{
    bridge, depth, identity, operator_key, plan, record, reviewed_target, state_constructor,
};
use crate::{
    StateConsumerCensus, StateConsumerSites, StateDefinitionCensus, StateLinkRefusal,
    StateLinkSymbol as Key, StateRelocation, StateResolvedCensus, StateSingletonAsset,
    check_linked_state_program, collect_state_definitions, discover_state_relocations,
    resolve_state_census, state_declared_type, substitute_state,
};

// --- Fixtures ----------------------------------------------------------

/// The issued identifier the demonstration record carries.
///
/// The same bytes the record's fixture asset carries, so a substitution
/// here is the identity and the width assertions are what generalize.
pub(super) fn demonstration_singleton() -> StateSingletonAsset {
    StateSingletonAsset::new([0x11; 32])
}

/// A second deployment's issued identifier.
///
/// Public, meaningless material standing for no issued asset, and
/// different from the record's in every byte.
pub(super) fn second_singleton() -> StateSingletonAsset {
    StateSingletonAsset::new([0xa1; 32])
}

/// The architecture's own declaration of the identity singleton.
pub(super) fn demonstration_declaration() -> StateSingletonDeclaration {
    let spec = ARCHITECTURE
        .asset(AssetId::Pid)
        .expect("the identity asset is declared");
    StateSingletonDeclaration::from_architecture_asset(spec)
        .expect("the declaration is a singleton")
}

/// A second deployment's lead window: test material, like the first.
pub(super) fn second_lead_bounds() -> StateLeadBounds {
    let bounds = AnnouncementLeadBounds::new(Cycle::new(3), Cycle::new(5))
        .expect("the fixture window is nonzero and ordered");
    StateLeadBounds::new(bounds, StateLeadBoundOrigin::Fixture)
}

/// A second deployment's operator binding, under another public key.
pub(super) fn second_binding() -> OperatorDeploymentBinding {
    let target = reviewed_target();
    let internal_key = StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0xb6; 32])
        .expect("fixture internal key has the reviewed width");
    let profile = EstablishedOperatorProfile::establish(selected_operator_profile(), &target)
        .expect("the reviewed target establishes the source selection");
    OperatorDeploymentBinding::bind(
        &target,
        operator_key(0xa3),
        profile,
        identity(0x11, 0x22),
        &internal_key,
    )
    .expect("the second candidate deployment binds")
}

/// The second deployment's bound sources.
pub(super) fn second_bridge() -> StateLinkDeploymentParameters {
    StateLinkDeploymentParameters::bind(
        &reviewed_target(),
        plan(),
        second_lead_bounds(),
        identity(0x11, 0x22),
        second_binding(),
        depth(),
        &record(),
    )
    .expect("the second deployment's sources bind")
}

/// Pass one over one deployment's sources.
fn definitions(
    deployment: &StateLinkDeploymentParameters,
    singleton: &StateSingletonAsset,
) -> StateDefinitionCensus {
    collect_state_definitions(
        &reviewed_target(),
        deployment,
        &state_constructor(),
        singleton,
        &demonstration_declaration(),
        tapscript::StateWitnessSchedule::WholeMetadata,
    )
    .expect("the sources define every key")
}

/// The consumers of the composed record and the constructor.
fn consumers() -> StateConsumerCensus {
    StateConsumerCensus::from_sources(&record(), &state_constructor())
}

/// The demonstration deployment's resolved census.
pub(super) fn resolved_census() -> StateResolvedCensus {
    resolve_state_census(
        &definitions(&bridge(), &demonstration_singleton()),
        &consumers(),
    )
    .expect("the demonstration census resolves")
}

/// The second deployment's resolved census.
pub(super) fn second_resolved_census() -> StateResolvedCensus {
    resolve_state_census(
        &definitions(&second_bridge(), &second_singleton()),
        &consumers(),
    )
    .expect("the second deployment's census resolves")
}

/// Every index at which two instruction sequences differ.
fn differing(before: &[TapscriptInstruction], after: &[TapscriptInstruction]) -> BTreeSet<usize> {
    before
        .iter()
        .zip(after)
        .enumerate()
        .filter(|(_, (before, after))| before != after)
        .map(|(index, _)| index)
        .collect()
}

/// The record's own statement of what one key's sites carry.
fn fixture_item(composed: &StateAnnouncementProgram, symbol: Key) -> Option<&StackItem> {
    composed
        .consumers()
        .iter()
        .find(|(consumed, _)| Key::from_program(**consumed) == symbol)
        .map(|(_, consumer)| &consumer.item)
}

/// One resolved census with one key's consumer sites replaced.
fn census_with(symbol: Key, sites: BTreeSet<usize>, field: bool) -> StateResolvedCensus {
    let mut map = consumers().consumers().clone();
    map.insert(symbol, StateConsumerSites::new(sites, field));
    resolve_state_census(
        &definitions(&bridge(), &demonstration_singleton()),
        &StateConsumerCensus::new(map),
    )
    .expect("the census resolves")
}

// --- (a) The demonstration census ---------------------------------------

// One record per occurrence, in site order, over the six keys the
// program pushes and none of the seven it does not.
#[test]
fn the_demonstration_census_has_one_record_per_occurrence() {
    let target = reviewed_target();
    let composed = record();
    let resolved = resolved_census();
    let census = discover_state_relocations(&target, &composed, &resolved)
        .expect("the demonstration census discovers");

    assert_eq!(census.len(), resolved.push_site_count());
    assert!(!census.is_empty());
    assert_eq!(census.symbols(), resolved.program_keys());

    let recorded: BTreeSet<usize> = resolved
        .entries()
        .values()
        .flat_map(|entry| entry.sites().record_sites().iter().copied())
        .collect();
    assert_eq!(census.sites(), recorded);
    assert_eq!(census.sites().len(), census.len());

    // Site order, strictly increasing: sorting and deduplicating leaves
    // the sequence alone exactly when every occurrence appears once.
    let order: Vec<usize> = census
        .relocations()
        .iter()
        .map(StateRelocation::site)
        .collect();
    let mut canonical = order.clone();
    canonical.sort_unstable();
    canonical.dedup();
    assert_eq!(order, canonical);

    for (&symbol, entry) in resolved.entries() {
        assert_eq!(
            census.by_symbol(symbol).count(),
            entry.sites().record_sites().len(),
            "{symbol:?} has the wrong number of records"
        );
    }
}

// Each site is named by the component whose range holds it, no range is
// shared, every pre-value is the record's own fixture item, and every
// delta is zero because each key's width is fixed by its encoding.
#[test]
fn every_demonstration_record_names_the_component_that_holds_its_site() {
    let target = reviewed_target();
    let composed = record();
    let census = discover_state_relocations(&target, &composed, &resolved_census())
        .expect("the demonstration census discovers");

    for relocation in census.relocations() {
        assert_eq!(relocation.leaf(), StateLeafRole::Announcement);
        assert_eq!(
            relocation.expected(),
            state_declared_type(relocation.symbol())
        );
        assert!(relocation.aliases().is_empty());

        let range = composed
            .components()
            .get(&relocation.component())
            .expect("the named component has a range");
        assert!(range.contains(&relocation.site()));

        assert_eq!(
            Some(relocation.pre_value()),
            fixture_item(&composed, relocation.symbol())
        );
        // This deployment supplies what the record was composed against,
        // so the substitution is the identity and the delta is zero on
        // both counts.
        assert_eq!(relocation.pre_value(), relocation.linked_value());
        assert_eq!(relocation.delta().script_bytes(), 0);
    }

    let mut attribution: BTreeMap<StateProgramComponent, usize> = BTreeMap::new();
    for relocation in census.relocations() {
        *attribution.entry(relocation.component()).or_default() += 1;
    }
    assert_eq!(
        attribution,
        BTreeMap::from([
            (
                StateProgramComponent::Operator(StateOperatorPatternId::OperatorAuthorizationV1),
                1
            ),
            (
                StateProgramComponent::Structural(StatePatternId::StateInputRecognitionV1),
                2
            ),
            (
                StateProgramComponent::Semantic(StateAnnouncementId::MetadataAuthentication),
                2
            ),
            (
                StateProgramComponent::Semantic(StateAnnouncementId::LeadWindow),
                6
            ),
            (
                StateProgramComponent::Semantic(StateAnnouncementId::SuccessorReconstruction),
                4
            ),
        ])
    );
}

// The component ranges partition the program, which is why no site of it
// can lie outside every range.
#[test]
fn the_component_ranges_tile_the_whole_program() {
    let composed = record();
    let mut ranges: Vec<_> = composed.components().values().cloned().collect();
    ranges.sort_by_key(|range| range.start);

    let mut next = 0;
    for range in &ranges {
        assert_eq!(range.start, next, "the ranges leave a gap or overlap");
        next = range.end;
    }
    assert_eq!(next, composed.program().len());
    assert_eq!(ranges.len(), composed.components().len());
}

// --- (b) The second deployment ------------------------------------------

// Substitution moves exactly the sites of the keys the deployment
// changes, and the linked program is the pristine one everywhere else.
#[test]
fn the_second_deployment_moves_exactly_the_sites_of_the_keys_it_changes() {
    let target = reviewed_target();
    let composed = record();
    let linked = substitute_state(&target, &composed, &second_resolved_census())
        .expect("the second deployment links");

    assert_eq!(linked.leaf(), StateLeafRole::Announcement);
    assert_eq!(linked.program().len(), composed.program().len());
    assert_eq!(linked.execution(), composed.execution());

    let moved = differing(
        composed.program().instructions(),
        linked.program().instructions(),
    );
    let claimed: BTreeSet<usize> = linked
        .relocations()
        .relocations()
        .iter()
        .filter(|relocation| relocation.pre_value() != relocation.linked_value())
        .map(StateRelocation::site)
        .collect();
    assert_eq!(moved, claimed);

    let resolved = second_resolved_census();
    let sites = |symbol: Key| -> BTreeSet<usize> {
        resolved
            .entries()
            .get(&symbol)
            .expect("the key resolves")
            .sites()
            .record_sites()
            .clone()
    };
    let changed: BTreeSet<usize> = [
        Key::StateAsset,
        Key::CommittedOperatorKey,
        Key::MaturityLeadMin,
        Key::MaturityLeadMax,
    ]
    .into_iter()
    .flat_map(sites)
    .collect();
    assert_eq!(moved, changed);

    let untouched: BTreeSet<usize> = [Key::InternalKey, Key::StateAmount]
        .into_iter()
        .flat_map(sites)
        .collect();
    assert!(moved.is_disjoint(&untouched));
    assert_eq!(moved.len() + untouched.len(), linked.relocations().len());

    // Every width is fixed by its encoding, whatever the deployment.
    for relocation in linked.relocations().relocations() {
        assert_eq!(relocation.delta().script_bytes(), 0);
        assert_eq!(
            relocation.pre_value().len(),
            relocation.linked_value().len()
        );
    }

    let bytes = linked.program().encode(&target);
    assert_eq!(
        TapscriptProgram::decode(&target, &bytes).expect("the linked program parses"),
        *linked.program()
    );
}

// --- (c) The one-symbol cross-check --------------------------------------

// Rebuilding the pristine program with one key's value alone moves
// exactly that key's sites, and none at all where the value is the
// record's own. This recomputes from outside what discovery checks
// inside, over the deployment where four of the six keys move.
#[test]
fn rebuilding_one_key_at_a_time_moves_exactly_that_key_s_sites() {
    let target = reviewed_target();
    let composed = record();
    let census = discover_state_relocations(&target, &composed, &second_resolved_census())
        .expect("the second deployment's census discovers");
    let pristine = composed.program().instructions();

    for symbol in census.symbols() {
        let records: Vec<&StateRelocation> = census.by_symbol(symbol).collect();
        assert_ne!(records.len(), 0);

        let mut rebuilt = pristine.to_vec();
        for relocation in &records {
            rebuilt[relocation.site()] =
                TapscriptInstruction::Push(relocation.linked_value().clone());
        }

        let sites: BTreeSet<usize> = records.iter().map(|record| record.site()).collect();
        let moves = records
            .iter()
            .any(|record| record.pre_value() != record.linked_value());
        let expected = if moves { sites } else { BTreeSet::new() };
        assert_eq!(differing(pristine, &rebuilt), expected, "{symbol:?}");
    }
}

// --- (d) The checks, reached with a program the link did not build -------

// An instruction no relocation covers is refused at its own index.
#[test]
fn an_untracked_mutation_is_refused_at_its_index() {
    let target = reviewed_target();
    let composed = record();
    let linked = substitute_state(&target, &composed, &second_resolved_census())
        .expect("the second deployment links");

    let covered = linked.relocations().sites();
    let (victim, _) = linked
        .program()
        .instructions()
        .iter()
        .enumerate()
        .find(|(index, instruction)| {
            !covered.contains(index) && matches!(instruction, TapscriptInstruction::Push(_))
        })
        .expect("the leaf pushes literals no relocation covers");

    let mut instructions = linked.program().instructions().to_vec();
    let replacement = TapscriptInstruction::Push(
        StackItem::new(&target, vec![0x7e; 3]).expect("three bytes is a literal"),
    );
    assert_ne!(instructions[victim], replacement);
    instructions[victim] = replacement;
    let doctored = TapscriptProgram::new(instructions).expect("the doctored program is typed");

    assert_eq!(
        check_linked_state_program(&target, &composed, &doctored, linked.relocations()),
        Err(StateLinkRefusal::UntrackedProgramMutation { site: victim })
    );
}

// A site still carrying its pristine item is refused by name, which is
// what the pristine program itself is under the second deployment's
// census.
#[test]
fn a_site_that_kept_its_pristine_item_is_refused() {
    let target = reviewed_target();
    let composed = record();
    let census = discover_state_relocations(&target, &composed, &second_resolved_census())
        .expect("the second deployment's census discovers");

    let first = census
        .relocations()
        .iter()
        .find(|relocation| relocation.pre_value() != relocation.linked_value())
        .expect("the second deployment changes at least one value");

    assert_eq!(
        check_linked_state_program(&target, &composed, composed.program(), &census),
        Err(StateLinkRefusal::RelocationNotApplied {
            symbol: first.symbol(),
            site: first.site(),
        })
    );
}

// --- (e) The witnessed root ----------------------------------------------

// No relocation names the root, and the refusal reserved for a literal
// one is unreachable for the reason this test recomputes: the root
// resolves to no pushable item at all.
#[test]
fn the_witnessed_root_carries_no_relocation_and_no_pushable_item() {
    let target = reviewed_target();
    let composed = record();
    let resolved = resolved_census();
    let census = discover_state_relocations(&target, &composed, &resolved)
        .expect("the demonstration census discovers");

    assert!(!census.symbols().contains(&Key::StaticSubtreeRoot));

    let root = resolved
        .entries()
        .get(&Key::StaticSubtreeRoot)
        .expect("the root is a resolved key");
    assert!(root.definition().value().push_item(&target).is_none());
    assert!(root.sites().record_sites().is_empty());
    assert!(root.sites().constructor_field());

    // A census that did give the root a site is refused for the item it
    // has not got, before anything is placed.
    let site = census.relocations()[0].site();
    let claimed = census_with(Key::StaticSubtreeRoot, BTreeSet::from([site]), true);
    assert_eq!(
        discover_state_relocations(&target, &composed, &claimed),
        Err(StateLinkRefusal::UnresolvedRelocation(
            Key::StaticSubtreeRoot
        ))
    );
}

// A recorded site that is not a push is refused rather than overwritten.
#[test]
fn a_site_that_is_not_a_push_is_refused() {
    let target = reviewed_target();
    let composed = record();
    let opcode = composed
        .program()
        .instructions()
        .iter()
        .position(|instruction| matches!(instruction, TapscriptInstruction::Opcode(_)))
        .expect("the leaf schedules primitives");

    let mut sites = consumers()
        .consumers()
        .get(&Key::StateAsset)
        .expect("the asset is consumed")
        .record_sites()
        .clone();
    sites.insert(opcode);

    assert_eq!(
        discover_state_relocations(
            &target,
            &composed,
            &census_with(Key::StateAsset, sites, false)
        ),
        Err(StateLinkRefusal::SiteIsNotAPush {
            symbol: Key::StateAsset,
            site: opcode,
        })
    );
}

// A census pointing a key at a push that carries something else is
// caught by the rebuild, where a byte search would have called the
// coincidence a site.
#[test]
fn a_census_site_carrying_another_value_is_refused_by_the_rebuild() {
    let target = reviewed_target();
    let composed = record();
    let census = discover_state_relocations(&target, &composed, &resolved_census())
        .expect("the demonstration census discovers");

    let foreign = census
        .by_symbol(Key::InternalKey)
        .next()
        .expect("the internal key is pushed")
        .site();
    let mut sites = consumers()
        .consumers()
        .get(&Key::StateAsset)
        .expect("the asset is consumed")
        .record_sites()
        .clone();
    sites.insert(foreign);

    assert_eq!(
        discover_state_relocations(
            &target,
            &composed,
            &census_with(Key::StateAsset, sites, false)
        ),
        Err(StateLinkRefusal::RelocationCensusDisagreement {
            symbol: Key::StateAsset,
            expected: BTreeSet::new(),
            observed: BTreeSet::from([foreign]),
        })
    );
}

// --- (f) Permutation and simultaneity ------------------------------------

/// The second deployment's census, built from inputs offered in reverse.
fn reversed_resolved_census() -> StateResolvedCensus {
    let full = definitions(&second_bridge(), &second_singleton());
    let mut census = StateDefinitionCensus::default();
    for (symbol, definition) in full.definitions().iter().rev() {
        census
            .define(*symbol, definition.value().clone(), definition.origin())
            .expect("each key is claimed once");
    }

    let source = consumers();
    let mut map = BTreeMap::new();
    for (symbol, sites) in source.consumers().iter().rev() {
        map.insert(*symbol, sites.clone());
    }

    resolve_state_census(&census, &StateConsumerCensus::new(map))
        .expect("the reversed census resolves")
}

// The census does not depend on the order its definitions and consumers
// were offered in, and the one rebuild with every value together is the
// program the per-key rebuilds reach in either order.
#[test]
fn the_census_and_the_rebuild_are_independent_of_input_order() {
    let target = reviewed_target();
    let composed = record();
    let forward = second_resolved_census();
    let reversed = reversed_resolved_census();
    assert_eq!(reversed, forward);

    let census = discover_state_relocations(&target, &composed, &forward)
        .expect("the second deployment's census discovers");
    let again = discover_state_relocations(&target, &composed, &reversed)
        .expect("the reversed census discovers");
    assert_eq!(census.relocations(), again.relocations());

    let linked = substitute_state(&target, &composed, &forward).expect("the deployment links");
    let symbols: Vec<Key> = census.symbols().into_iter().collect();

    for order in [symbols.clone(), symbols.into_iter().rev().collect()] {
        let mut sequential = composed.program().instructions().to_vec();
        for symbol in order {
            for relocation in census.by_symbol(symbol) {
                sequential[relocation.site()] =
                    TapscriptInstruction::Push(relocation.linked_value().clone());
            }
        }
        assert_eq!(sequential, linked.program().instructions());
    }
}

// --- (g) The closed root, for the relocation half ------------------------

/// Which test reaches one relocation refusal, or why nothing can.
///
/// Called from the census bite's walk over the whole closed root, which
/// names these variants one by one, so a variant added to the root still
/// fails to compile there until somebody accounts for it here.
pub(super) fn relocation_reachability(refusal: &StateLinkRefusal) -> &'static str {
    match refusal {
        StateLinkRefusal::UnresolvedRelocation(_) => {
            "the_witnessed_root_carries_no_relocation_and_no_pushable_item"
        }
        StateLinkRefusal::SiteIsNotAPush { .. } => "a_site_that_is_not_a_push_is_refused",
        StateLinkRefusal::RelocationCensusDisagreement { .. } => {
            "a_census_site_carrying_another_value_is_refused_by_the_rebuild"
        }
        StateLinkRefusal::RelocationNotApplied { .. } => {
            "a_site_that_kept_its_pristine_item_is_refused"
        }
        StateLinkRefusal::UntrackedProgramMutation { .. } => {
            "an_untracked_mutation_is_refused_at_its_index"
        }
        StateLinkRefusal::SiteOutsideEveryComponent { .. } => {
            "unreachable: the component ranges tile the program, so an index outside them is \
             outside the program and is refused as not a push first"
        }
        StateLinkRefusal::LiteralStaticRootRelocation => {
            "unreachable: the root resolves to no pushable item, so no entry for it can carry one"
        }
        StateLinkRefusal::RoundTripMismatch => {
            "unreachable: every instruction is a typed primitive or a literal inside the target's \
             bound, and the encoder writes the minimal form the parser accepts"
        }
        StateLinkRefusal::LinkedProgramDoesNotSchedule { .. } => {
            "unreachable: a program passing the site and untracked checks is the pristine program \
             with equal-width payloads, which schedules because that one did"
        }
        StateLinkRefusal::AbstractExecutionMoved => {
            "unreachable: the walk types a literal by its width alone, and every substitution here \
             preserves width"
        }
        StateLinkRefusal::ResourceDeltaOverflow { .. } => {
            "unreachable: both widths are at most the target's literal bound and its prefix"
        }
        _ => "accounted for elsewhere in the closed root",
    }
}

// Every relocation refusal is reached by a named test or declared
// unreachable with the reason a test recomputes.
#[test]
fn every_relocation_refusal_is_reached_or_declared() {
    let target = reviewed_target();
    let cause = StackItem::new(&target, vec![0; 1_000_000])
        .expect_err("a million bytes is not a literal the reviewed contract admits");

    let refusals = [
        StateLinkRefusal::UnresolvedRelocation(Key::StaticSubtreeRoot),
        StateLinkRefusal::SiteIsNotAPush {
            symbol: Key::StateAsset,
            site: 0,
        },
        StateLinkRefusal::SiteOutsideEveryComponent {
            symbol: Key::StateAsset,
            site: 0,
        },
        StateLinkRefusal::LiteralStaticRootRelocation,
        StateLinkRefusal::RelocationCensusDisagreement {
            symbol: Key::StateAsset,
            expected: BTreeSet::new(),
            observed: BTreeSet::from([0]),
        },
        StateLinkRefusal::RelocationNotApplied {
            symbol: Key::StateAmount,
            site: 0,
        },
        StateLinkRefusal::UntrackedProgramMutation { site: 0 },
        StateLinkRefusal::RoundTripMismatch,
        StateLinkRefusal::LinkedProgramDoesNotSchedule { cause },
        StateLinkRefusal::AbstractExecutionMoved,
        StateLinkRefusal::ResourceDeltaOverflow {
            symbol: Key::InternalKey,
            site: 0,
        },
    ];

    let accounts: BTreeSet<&str> = refusals.iter().map(relocation_reachability).collect();
    assert_eq!(accounts.len(), refusals.len());
    assert!(
        !accounts.contains("accounted for elsewhere in the closed root"),
        "a relocation refusal fell through to the resource half"
    );
    assert_eq!(
        refusals
            .iter()
            .filter(|refusal| relocation_reachability(refusal).starts_with("unreachable"))
            .count(),
        6
    );
}
